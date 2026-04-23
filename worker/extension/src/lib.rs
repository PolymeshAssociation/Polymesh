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
    fn start_session(
        &mut self,
        flags_and_backends: WorkerConfigFlagsAndBackends,
        default_protocol: ProtocolNumber,
    ) -> WorkerSessionId {
        let ext = PolymeshWorkerExt::new();
        let mut loader = SubstrateModuleLoader(&mut **self);
        let (flags, backends) = unpack_flags_and_backends(flags_and_backends);
        ext.start_session(flags, backends, default_protocol, &mut loader)
    }

    /// Execute a protocol-specific work request for the given session id.
    fn session_execute_request(
        &mut self,
        session_id: WorkerSessionId,
        flags: WorkerConfigFlags,
        request: PassFatPointerAndRead<Vec<u8>>,
    ) -> WorkStatusFlagsAndId {
        let ext = PolymeshWorkerExt::new();
        ext.session_execute_request(session_id, flags, request)
    }

    /// Execute work request in the give session and wait for the results.
    fn session_execute_request_and_wait(
        &mut self,
        session_id: WorkerSessionId,
        flags: WorkerConfigFlags,
        request: PassFatPointerAndRead<Vec<u8>>,
    ) -> AllocateAndReturnByCodec<Result<WorkResponseResult, WorkerError>> {
        let ext = PolymeshWorkerExt::new();
        ext.session_execute_request_and_wait(session_id, flags, request)
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
        let ext = PolymeshWorkerExt::new();
        ext.session_push_result(session_id, request_id, result)
            .into()
    }

    /// Get the number of requests in the session.
    fn session_num_requests(session_id: WorkerSessionId) -> WorkerErrorNum {
        let ext = PolymeshWorkerExt::new();
        ext.session_num_requests(session_id).into()
    }

    /// Get the request result for the given session id and request id.
    fn session_get_result(
        &mut self,
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
    ) -> AllocateAndReturnByCodec<Result<WorkResponseResult, WorkerError>> {
        let ext = PolymeshWorkerExt::new();
        ext.session_get_result(session_id, request_id)
    }

    /// End the session with the given session id.  This is normally done at the end of a block.
    fn end_session(session_id: WorkerSessionId) {
        let ext = PolymeshWorkerExt::new();
        ext.end_session(session_id)
    }
}
