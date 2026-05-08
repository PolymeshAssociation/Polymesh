pub use polymesh_worker::{WORKER_VERSION, worker::*};
use polymesh_worker_common::{
    BackendBitmask, Protocol, ProtocolNumber, WorkRequest, WorkerConfigFlags, WorkerSessionConfig,
    WorkerSessionId,
};

use crate::*;

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

    pub fn start_session(
        &self,
        flags: WorkerConfigFlags,
        backends: BackendBitmask,
        default_protocol: ProtocolNumber,
        loader: &mut SubstrateModuleLoader,
    ) -> WorkerSessionId {
        let default_protocol = Protocol::from_number(default_protocol);
        let config = WorkerSessionConfig::new(flags, backends);

        // Start a new session with the given config and default protocol.  The session id is returned to the caller, which can be used for subsequent calls to submit work or end the session.
        let session = self.0.start_session(config, default_protocol, loader);
        log::debug!("Created session id: {}", session.id);
        session.id
    }

    /// Execute a batch of work requests within the given session.
    pub fn session_execute_batch(
        &self,
        session_id: WorkerSessionId,
        flags: WorkerConfigFlags,
        requests: Vec<u8>,
    ) -> Vec<WorkStatusFlagsAndId> {
        let config = WorkRequestConfig::new(flags);
        let requests = match codec::Decode::decode(&mut &requests[..]) {
            Ok(reqs) => reqs,
            Err(err) => {
                log::warn!("Failed to decode work requests: {err}");
                return Vec::new();
            }
        };

        let results = self.0.session_execute_batch(session_id, config, requests);
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
        session_id: WorkerSessionId,
        flags: WorkerConfigFlags,
        work_req: Vec<u8>,
    ) -> WorkStatusFlagsAndId {
        let config = WorkRequestConfig::new(flags);
        let (request_id, status) =
            self.0
                .session_execute_request(session_id, config, WorkRequest(work_req));
        log::debug!(
            "Submitted work for session id: {}, request id: {}, status: {:?}",
            session_id,
            request_id,
            status
        );

        pack_work_status_flags_and_id(status, 0, request_id)
    }

    /// Execute a protocol-specific work request for the given session and wait for the results.
    pub fn session_execute_request_and_wait(
        &self,
        session_id: WorkerSessionId,
        flags: WorkerConfigFlags,
        work_req: Vec<u8>,
    ) -> Result<WorkResponseResult, WorkerError> {
        let config = WorkRequestConfig::new(flags);
        log::debug!(
            "Submitting work and waiting for result for session id: {}, flags: {:?}",
            session_id,
            flags
        );
        self.0
            .session_execute_request_and_wait(session_id, config, WorkRequest(work_req))
    }

    /// Push the result of a work request execution back to the worker for the given session and request id.
    pub fn session_push_result(
        &self,
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
        result: WorkResponseResult,
    ) -> WorkerError {
        log::debug!(
            "Pushing result for session id: {}, request id: {}, result: {:?}",
            session_id,
            request_id,
            result
        );
        self.0
            .session_push_result(session_id, request_id, result.into())
            .into()
    }

    /// Get the number of requests in the session.
    pub fn session_num_requests(&self, session_id: WorkerSessionId) -> WorkerError {
        self.0.session_num_requests(session_id).into()
    }

    /// Get the result of a work request execution for the given session and request id.
    pub fn session_get_result(
        &self,
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
    ) -> Result<WorkResponseResult, WorkerError> {
        log::debug!(
            "Getting result for session id: {}, request id: {}",
            session_id,
            request_id
        );
        self.0.session_get_result(session_id, request_id)
    }

    /// End the session with the given session id.  This will clean up any resources associated with the session in the worker.
    pub fn end_session(&self, session_id: WorkerSessionId) {
        log::debug!("Ending session id: {}", session_id);
        if let Err(err) = self.0.end_session(session_id) {
            log::error!("Failed to end session: {:?}", err);
            return;
        }
    }
}
