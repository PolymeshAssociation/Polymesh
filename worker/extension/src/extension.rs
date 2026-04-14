use codec::Decode;

pub use polymesh_worker::{WORKER_VERSION, backend::BackendModuleLoader, worker::*};
use polymesh_worker_common::{
    BackendBitmask, BackendCodeAndContextHash, BackendCodeHash, BackendContextHash, BackendKind,
    Protocol, ProtocolNumber, WorkRequest, WorkerConfigFlags, WorkerSessionConfig, WorkerSessionId,
};

use crate::*;

pub struct SubstrateModuleLoader<'a>(pub &'a mut dyn sp_externalities::Externalities);

impl<'a> SubstrateModuleLoader<'a> {
    fn storage(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.0.storage(key)
    }
}

//impl BackendModuleLoader for SubstrateModuleLoader {
impl<'a> BackendModuleLoader for SubstrateModuleLoader<'a> {
    fn get_module_code_and_context_hash(
        &mut self,
        protocol: Protocol,
        kind: BackendKind,
    ) -> Option<BackendCodeAndContextHash> {
        // Generate the storage key for the module code hash based on the protocol and backend kind.
        let mut key = Vec::new();
        protocol_module_code_hash_key(&mut key, kind, protocol);
        // Load the module code hash from the host storage.
        if let Some(hash_value) = self.storage(key.as_slice()) {
            let code_hash = Decode::decode(&mut &hash_value[..]).ok()?;
            return Some(code_hash);
        }
        None
    }

    fn get_module_code_bytes(
        &mut self,
        protocol: Protocol,
        kind: BackendKind,
        code_hash: BackendCodeHash,
    ) -> Option<Vec<u8>> {
        // Generate the storage key for the module bytes based on the protocol, backend kind and code hash.
        let mut key = Vec::new();
        protocol_module_bytes_key(&mut key, kind, protocol, code_hash);
        // Load the module bytes from the host storage.
        self.storage(key.as_slice())
    }

    fn get_module_context_bytes(
        &mut self,
        protocol: Protocol,
        ctx_hash: BackendContextHash,
    ) -> Option<Vec<u8>> {
        // Generate the storage key for the module bytes based on the protocol, backend kind and code hash.
        let mut key = Vec::new();
        protocol_module_context_key(&mut key, protocol, ctx_hash);
        // Load the module bytes from the host storage.
        self.storage(key.as_slice())
    }
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

    /// Execute a protocol-specific work request within the given session.
    pub fn session_execute_request(
        &self,
        session_id: WorkerSessionId,
        work_req: Vec<u8>,
    ) -> WorkStatusFlagsAndId {
        let (request_id, status) = self
            .0
            .session_execute_request(session_id, WorkRequest(work_req));
        log::debug!(
            "Submitted work for session id: {}, request id: {}, status: {:?}",
            session_id,
            request_id,
            status
        );

        work_status_flags_and_id(status, 0, request_id)
    }

    /// Execute a protocol-specific work request for the given session and wait for the results.
    pub fn session_execute_request_and_wait(
        &self,
        session_id: WorkerSessionId,
        use_cache: bool,
        work_req: Vec<u8>,
    ) -> Result<WorkResponseResult, WorkerError> {
        log::debug!(
            "Submitting work and waiting for result for session id: {}, use_cache: {}",
            session_id,
            use_cache
        );
        self.0
            .session_execute_request_and_wait(session_id, use_cache, WorkRequest(work_req))
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
