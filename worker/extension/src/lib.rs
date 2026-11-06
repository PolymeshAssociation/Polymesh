#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime_interface::{pass_by::*, runtime_interface};
use sp_std::vec::Vec;

pub use polymesh_worker_common::*;

#[cfg(feature = "std")]
pub use polymesh_worker::worker::PolymeshWorker;

#[cfg(feature = "std")]
mod extension;
#[cfg(feature = "std")]
pub use extension::*;

mod loader;
pub use loader::*;

#[cfg(feature = "std")]
lazy_static::lazy_static! {
    pub static ref WORKER: PolymeshWorkerRef = PolymeshWorker::new();
}

/// Native interface for runtime module for Polymesh Worker
#[runtime_interface]
pub trait NativePolymeshWorker {
    /// The host worker version.
    fn worker_version() -> WorkerVersion {
        WORKER_VERSION
    }

    /// Execute a protocol-specific work request without session.
    fn execute_request(
        &mut self,
        protocol: ProtocolNumber,
        backends: BackendBitmask,
        request: PassFatPointerAndRead<Vec<u8>>,
    ) -> AllocateAndReturnByCodec<Result<WorkResponseResult, WorkerError>> {
        let ext = PolymeshWorkerExt::new();
        let mut loader = SubstrateModuleLoader(&mut **self);
        ext.execute_request(protocol, backends, request, &mut loader)
    }

    /// Start a new session.  This is normally done at the start of a block.
    ///
    /// Sessions live in an externalities extension, so they are dropped at the end of the
    /// runtime call even if `end_session` is never called.
    fn start_session(
        &mut self,
        flags_and_backends: WorkerConfigFlagsAndBackends,
        default_protocol: ProtocolNumber,
    ) -> WorkerSessionId {
        // Unknown session id: requests fall back to runtime execution.
        let Some(session_id) = next_session_id(&mut **self) else {
            return WorkerSessionId::MAX;
        };
        let session = {
            let mut loader = SubstrateModuleLoader(&mut **self);
            PolymeshWorkerExt::new().start_session(
                session_id,
                flags_and_backends,
                default_protocol,
                &mut loader,
            )
        };
        insert_session(&mut **self, session);
        session_id
    }

    /// Execute a batch of work requests within the given session.
    fn session_execute_batch(
        &mut self,
        session_id: WorkerSessionId,
        flags: WorkerConfigFlags,
        work_reqs: PassFatPointerAndRead<Vec<u8>>,
    ) -> AllocateAndReturnByCodec<Vec<WorkStatusFlagsAndId>> {
        match get_session(&mut **self, session_id) {
            Ok(session) => {
                PolymeshWorkerExt::new().session_execute_batch(&session, flags, work_reqs)
            }
            Err(err) => {
                log::error!("Failed to submit work request: {err:?}");
                Vec::new()
            }
        }
    }

    /// Execute a protocol-specific work request for the given session id.
    fn session_execute_request(
        &mut self,
        session_id: WorkerSessionId,
        flags: WorkerConfigFlags,
        request: PassFatPointerAndRead<Vec<u8>>,
    ) -> WorkStatusFlagsAndId {
        match get_session(&mut **self, session_id) {
            Ok(session) => {
                PolymeshWorkerExt::new().session_execute_request(&session, flags, request)
            }
            Err(err) => {
                log::error!("Failed to submit work request: {err:?}");
                pack_work_status_flags_and_id(WorkStatus::SessionNotFound, 0, 0)
            }
        }
    }

    /// Execute work request in the give session and wait for the results.
    fn session_execute_request_and_wait(
        &mut self,
        session_id: WorkerSessionId,
        flags: WorkerConfigFlags,
        request: PassFatPointerAndRead<Vec<u8>>,
    ) -> AllocateAndReturnByCodec<Result<WorkResponseResult, WorkerError>> {
        let session = get_session(&mut **self, session_id)?;
        PolymeshWorkerExt::new().session_execute_request_and_wait(&session, flags, request)
    }

    /// Push the work result for the given session id and request id.
    ///
    /// This is used by the runtime when execution falls back to the runtime.
    fn session_push_result(
        &mut self,
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
        result: PassFatPointerAndDecode<WorkResponseResult>,
    ) -> WorkerErrorNum {
        let res = get_session(&mut **self, session_id).map(|session| {
            PolymeshWorkerExt::new().session_push_result(&session, request_id, result)
        });
        WorkerError::from(res).into()
    }

    /// Get the number of requests in the session.
    fn session_num_requests(session_id: WorkerSessionId) -> WorkerErrorNum {
        let res = sp_externalities::with_externalities(|ext| get_session(ext, session_id))
            .unwrap_or(Err(WorkerError::SessionNotFound(session_id)))
            .map(|session| session.num_requests());
        WorkerError::from(res).into()
    }

    /// Get the request result for the given session id and request id.
    fn session_get_result(
        &mut self,
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
    ) -> AllocateAndReturnByCodec<Result<WorkResponseResult, WorkerError>> {
        let session = get_session(&mut **self, session_id)?;
        PolymeshWorkerExt::new().session_get_result(&session, request_id)
    }

    /// End the session with the given session id.  This is normally done at the end of a block.
    fn end_session(session_id: WorkerSessionId) {
        log::debug!("Ending session id: {}", session_id);
        let res = sp_externalities::with_externalities(|ext| remove_session(ext, session_id));
        if let Some(Err(err)) = res {
            log::error!("Failed to end session: {err:?}");
        }
    }
}
