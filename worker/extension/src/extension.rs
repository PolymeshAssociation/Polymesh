use core::any::TypeId;
use std::collections::BTreeMap;

use sp_externalities::Externalities;

pub use polymesh_worker::{WORKER_VERSION, worker::*};
use polymesh_worker_common::{
    BackendBitmask, Protocol, ProtocolNumber, WorkRequest, WorkerConfigFlags, WorkerSessionConfig,
    WorkerSessionId,
};

use crate::*;

/// Worker sessions started during one runtime call (or one `ApiRef` for block building).
#[derive(Default)]
pub struct WorkerSessions {
    sessions: BTreeMap<WorkerSessionId, WorkerSessionRef>,
    next_id: WorkerSessionId,
}

sp_externalities::decl_extension! {
    /// Holds the worker sessions; they are dropped together with the call's extensions.
    pub struct WorkerSessionExt(WorkerSessions);
}

fn sessions(ext: &mut dyn Externalities) -> Option<&mut WorkerSessions> {
    ext.extension_by_type_id(TypeId::of::<WorkerSessionExt>())
        .and_then(|any| any.downcast_mut::<WorkerSessionExt>())
        .map(|ext| &mut ext.0)
}

/// Allocate the next session id of the current runtime call, registering the extension if needed.
///
/// Returns `None` if the externalities don't support extensions.
pub(crate) fn next_session_id(ext: &mut dyn Externalities) -> Option<WorkerSessionId> {
    if sessions(ext).is_none() {
        let sessions = Box::new(WorkerSessionExt(WorkerSessions::default()));
        if let Err(err) = ext.register_extension_with_type_id(WorkerSessionExt::type_id(), sessions)
        {
            log::warn!("Failed to register worker session extension: {err:?}");
        }
    }
    let sessions = sessions(ext)?;
    let session_id = sessions.next_id;
    sessions.next_id = session_id.wrapping_add(1);
    Some(session_id)
}

/// Store a session in the current runtime call.
pub(crate) fn insert_session(ext: &mut dyn Externalities, session: WorkerSessionRef) {
    if let Some(sessions) = sessions(ext) {
        sessions.sessions.insert(session.id, session);
    }
}

/// Get a session of the current runtime call.
pub(crate) fn get_session(
    ext: &mut dyn Externalities,
    session_id: WorkerSessionId,
) -> Result<WorkerSessionRef, WorkerError> {
    sessions(ext)
        .and_then(|sessions| sessions.sessions.get(&session_id).cloned())
        .ok_or(WorkerError::SessionNotFound(session_id))
}

/// Remove a session from the current runtime call, releasing its resources.
pub(crate) fn remove_session(
    ext: &mut dyn Externalities,
    session_id: WorkerSessionId,
) -> Result<WorkerSessionRef, WorkerError> {
    sessions(ext)
        .and_then(|sessions| sessions.sessions.remove(&session_id))
        .ok_or(WorkerError::SessionNotFound(session_id))
}

/// Polymesh Worker extension.
#[derive(Clone)]
pub struct PolymeshWorkerExt(PolymeshWorkerRef);

impl PolymeshWorkerExt {
    pub fn new() -> Self {
        Self(crate::WORKER.clone())
    }

    pub fn execute_request(
        &self,
        protocol: ProtocolNumber,
        backends: BackendBitmask,
        request: Vec<u8>,
        loader: &mut SubstrateModuleLoader,
    ) -> Result<WorkResponseResult, WorkerError> {
        let protocol = Protocol::from_number(protocol);
        self.0
            .execute_request(protocol, backends, WorkRequest(request), loader)
    }

    /// Start a new session with the given id.
    pub fn start_session(
        &self,
        session_id: WorkerSessionId,
        flags_and_backends: WorkerConfigFlagsAndBackends,
        default_protocol: ProtocolNumber,
        loader: &mut SubstrateModuleLoader,
    ) -> WorkerSessionRef {
        let (flags, backends) = unpack_flags_and_backends(flags_and_backends);
        let config = WorkerSessionConfig::new(flags, backends);
        let protocol = Protocol::from_number(default_protocol);
        let session = self.0.start_session(session_id, config, protocol, loader);
        log::debug!("Created session id: {}", session_id);
        session
    }

    /// Execute a batch of work requests within the given session.
    pub fn session_execute_batch(
        &self,
        session: &WorkerSessionRef,
        flags: WorkerConfigFlags,
        requests: Vec<u8>,
    ) -> Vec<WorkStatusFlagsAndId> {
        let session_id = session.id;
        let config = WorkRequestConfig::new(flags);
        let requests = match codec::Decode::decode(&mut &requests[..]) {
            Ok(reqs) => reqs,
            Err(err) => {
                log::warn!("Failed to decode work requests: {err}");
                return Vec::new();
            }
        };

        let results = self.0.session_execute_batch(session, config, requests);
        log::debug!(
            "Submitted batch of work for session id: {}, number of requests: {}, flags: {:?}",
            session_id,
            results.len(),
            flags
        );
        results
            .into_iter()
            .map(|(request_id, status)| {
                log::debug!(
                    "Submitted work for session id: {}, request id: {}, status: {:?}",
                    session_id,
                    request_id,
                    status
                );
                pack_work_status_flags_and_id(status, 0, request_id)
            })
            .collect()
    }

    /// Execute a protocol-specific work request within the given session.
    pub fn session_execute_request(
        &self,
        session: &WorkerSessionRef,
        flags: WorkerConfigFlags,
        work_req: Vec<u8>,
    ) -> WorkStatusFlagsAndId {
        let config = WorkRequestConfig::new(flags);
        let (request_id, status) =
            self.0
                .session_execute_request(session, config, WorkRequest(work_req));
        log::debug!(
            "Submitted work for session id: {}, request id: {}, status: {:?}",
            session.id,
            request_id,
            status
        );

        pack_work_status_flags_and_id(status, 0, request_id)
    }

    /// Execute a protocol-specific work request for the given session and wait for the results.
    pub fn session_execute_request_and_wait(
        &self,
        session: &WorkerSessionRef,
        flags: WorkerConfigFlags,
        work_req: Vec<u8>,
    ) -> Result<WorkResponseResult, WorkerError> {
        let config = WorkRequestConfig::new(flags);
        log::debug!(
            "Submitting work and waiting for result for session id: {}, flags: {:?}",
            session.id,
            flags
        );
        self.0
            .session_execute_request_and_wait(session, config, WorkRequest(work_req))
    }

    /// Push the result of a work request execution back to the given session.
    pub fn session_push_result(
        &self,
        session: &WorkerSessionRef,
        request_id: WorkRequestId,
        result: WorkResponseResult,
    ) {
        log::debug!(
            "Pushing result for session id: {}, request id: {}, result: {:?}",
            session.id,
            request_id,
            result
        );
        self.0.session_push_result(session, request_id, result)
    }

    /// Get the result of a work request execution for the given session and request id.
    pub fn session_get_result(
        &self,
        session: &WorkerSessionRef,
        request_id: WorkRequestId,
    ) -> Result<WorkResponseResult, WorkerError> {
        log::debug!(
            "Getting result for session id: {}, request id: {}",
            session.id,
            request_id
        );
        self.0.session_get_result(session, request_id)
    }
}
