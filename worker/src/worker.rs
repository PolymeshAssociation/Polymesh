use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

use crate::backend::{BackendManager, BackendManagerRef, BackendModuleLoader};
use polymesh_worker_common::{
    BackendBitmask, Protocol, WorkRequest, WorkRequestId, WorkResponseResult, WorkStatus,
    WorkerSessionId, config::*, error::*,
};

/// The mutable state of a worker session, which tracks the pending work requests and their results.
struct WorkerSessionInner {
    responses: BTreeMap<WorkRequestId, WorkResponseResult>,

    next_id: WorkRequestId,
    pending_count: usize,
}

impl WorkerSessionInner {
    fn new() -> Self {
        Self {
            responses: BTreeMap::new(),
            next_id: 0,
            pending_count: 0,
        }
    }

    fn new_request(&mut self) -> WorkRequestId {
        let req_id = self.next_id;
        self.next_id += 1;
        self.pending_count += 1;
        req_id
    }

    fn push_results(&mut self, req_id: WorkRequestId, res: WorkResponseResult) -> usize {
        self.responses.insert(req_id, res);
        self.pending_count -= 1;
        self.pending_count
    }
}

pub type WorkerSessionRef = Arc<WorkerSession>;

pub struct WorkerSession {
    pub id: WorkerSessionId,
    pub config: WorkerSessionConfig,
    pub protocol: Protocol,

    inner: RwLock<WorkerSessionInner>,
}

impl WorkerSession {
    /// Create a new worker session with the given id and config.
    pub fn new(
        id: WorkerSessionId,
        config: WorkerSessionConfig,
        protocol: Protocol,
    ) -> WorkerSessionRef {
        Arc::new(Self {
            id,
            config,
            protocol,

            inner: RwLock::new(WorkerSessionInner::new()),
        })
    }

    /// New work request.
    pub fn new_request(&self) -> WorkRequestId {
        let mut inner = self.inner.write().unwrap();
        inner.new_request()
    }

    /// Push the work response result for the given request id.
    pub fn push_response(&self, req_id: WorkRequestId, res: WorkResponseResult) {
        let mut inner = self.inner.write().unwrap();
        inner.push_results(req_id, res);
    }

    /// Get the work response result for the given request id.
    pub fn get_response(&self, req_id: WorkRequestId) -> Option<WorkResponseResult> {
        let inner = self.inner.read().unwrap();
        inner.responses.get(&req_id).cloned()
    }

    /// Get the number of requests in the session.
    pub fn num_requests(&self) -> u32 {
        let inner = self.inner.read().unwrap();
        inner.next_id
    }
}

/// The main worker struct that manages sessions.
struct PolymeshWorkerInner {
    sessions: HashMap<WorkerSessionId, WorkerSessionRef>,

    next_session_id: WorkerSessionId,
}

impl PolymeshWorkerInner {
    fn new() -> Self {
        Self {
            next_session_id: 0,
            sessions: HashMap::new(),
        }
    }

    fn create_session(
        &mut self,
        config: WorkerSessionConfig,
        protocol: Protocol,
    ) -> WorkerSessionRef {
        let session_id = self.next_session_id;
        self.next_session_id += 1;
        let session = WorkerSession::new(session_id, config, protocol);
        self.sessions.insert(session_id, session.clone());
        session
    }

    fn get_session(&self, session_id: WorkerSessionId) -> Result<WorkerSessionRef, WorkerError> {
        self.sessions
            .get(&session_id)
            .cloned()
            .ok_or(WorkerError::SessionNotFound(session_id))
    }

    fn remove_session(&mut self, session_id: WorkerSessionId) -> Option<WorkerSessionRef> {
        self.sessions.remove(&session_id)
    }
}

pub type PolymeshWorkerRef = Arc<PolymeshWorker>;

/// The shared state of the Polymesh Worker, which manages sessions and backends.
pub struct PolymeshWorker {
    inner: RwLock<PolymeshWorkerInner>,
    backend: BackendManagerRef,
}

impl PolymeshWorker {
    /// Create the polymesh worker shared state.
    pub fn new() -> PolymeshWorkerRef {
        Arc::new(Self {
            inner: RwLock::new(PolymeshWorkerInner::new()),
            backend: BackendManager::new(),
        })
    }

    /// Preload the backend modules for the given protocol and backend bitmask.
    pub fn preload_protocol(
        &self,
        allowed_backends: BackendBitmask,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Result<(), WorkerError> {
        self.backend
            .load_protocol(allowed_backends, protocol, loader);

        Ok(())
    }

    pub fn execute_request(
        &self,
        protocol: Protocol,
        backends: BackendBitmask,
        request: WorkRequest,
        loader: &mut dyn BackendModuleLoader,
    ) -> Result<WorkResponseResult, WorkerError> {
        // Get the protocol module instance for the given protocol and backends.
        let module = self
            .backend
            .load_protocol(backends, protocol, loader)
            .ok_or(WorkerError::ModuleExecutionFailed)?;
        let mut instance = module
            .get_instance()
            .ok_or(WorkerError::ModuleExecutionFailed)?;

        // Execute the work request using the protocol module instance.
        let work_result = instance.execute(&request)?;

        Ok(work_result)
    }

    /// Create a new worker session with the given config.
    pub fn start_session(
        &self,
        config: WorkerSessionConfig,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> WorkerSessionRef {
        // Ensure the protocol is loaded before starting the session.
        // TODO: Cache the loaded protocol module reference in the session.
        self.backend
            .load_protocol(config.backends, protocol, loader);

        let mut inner = self.inner.write().unwrap();
        let session = inner.create_session(config, protocol);
        log::debug!("Started session with id: {}", session.id);
        session
    }

    /// Execute a protocol-specific work request for the given session id.
    pub fn session_execute_request(
        &self,
        session_id: WorkerSessionId,
        work: WorkRequest,
    ) -> (WorkRequestId, WorkStatus) {
        // First get the session for the given session id.
        let inner = self.inner.read().unwrap();
        let session = match inner.get_session(session_id) {
            Ok(session) => session,
            Err(err) => {
                log::error!("Failed to submit work request: {err:?}");
                return (0, WorkStatus::SessionNotFound);
            }
        };

        // Create a new request id.
        let request_id = session.new_request();
        log::debug!(
            "Submitted work request with id: {} for session id: {}",
            request_id,
            session_id
        );

        // Try executing the work request and get the status.  If execution fails, fallback to runtime execution by returning a special status.
        let status = match self.try_execute_work(session.clone(), request_id, work) {
            Ok(status) => status,
            Err(err) => {
                log::error!("Failed to execute work request: {err:?}");
                WorkStatus::ExecutionFailedFallbackToRuntime
            }
        };
        (request_id, status)
    }

    /// Execute a protocol-specific work request for the given session id and wait for the results.
    pub fn session_execute_request_and_wait(
        &self,
        session_id: WorkerSessionId,
        _use_cache: bool,
        work: WorkRequest,
    ) -> Result<WorkResponseResult, WorkerError> {
        // TODO: Only support synchronous execution.  Don't allocate a request id for these requests.
        let (request_id, status) = self.session_execute_request(session_id, work);
        if status == WorkStatus::SessionNotFound {
            return Err(WorkerError::SessionNotFound(session_id));
        } else if status != WorkStatus::Completed {
            return Err(WorkerError::ModuleExecutionFailed);
        }

        self.session_get_result(session_id, request_id)
    }

    /// Push the result of a work request execution back to the worker for the given session and request id.
    pub fn session_push_result(
        &self,
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
        result: WorkResponseResult,
    ) -> Result<(), WorkerError> {
        // First get the session for the given session id.
        let inner = self.inner.read().unwrap();
        let session = inner.get_session(session_id)?;

        // Push the work response result to the session.
        session.push_response(request_id, result);
        Ok(())
    }

    /// Get the number of requests in the session.
    pub fn session_num_requests(&self, session_id: WorkerSessionId) -> Result<u32, WorkerError> {
        // First get the session for the given session id.
        let inner = self.inner.read().unwrap();
        let session = inner.get_session(session_id)?;

        Ok(session.num_requests())
    }

    /// Get the result of a work request execution for the given session and request id.
    pub fn session_get_result(
        &self,
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
    ) -> Result<WorkResponseResult, WorkerError> {
        // First get the session for the given session id.
        let inner = self.inner.read().unwrap();
        let session = inner.get_session(session_id)?;

        // Get the work response result for the given request id.
        let result = session
            .get_response(request_id)
            .ok_or(WorkerError::SessionRequestNotFound(session_id, request_id))?;
        Ok(result)
    }

    // Try executing the work request on the host and return the status.
    fn try_execute_work(
        &self,
        session: WorkerSessionRef,
        request_id: WorkRequestId,
        work: WorkRequest,
    ) -> Result<WorkStatus, WorkerError> {
        // Get the protocol module instance for the session's protocol.
        let protocol = session.protocol;
        let mut instance = self
            .backend
            .get_protocol_instance(protocol)
            .ok_or(WorkerError::ModuleExecutionFailed)?;

        // Execute the work request using the protocol module instance.
        let work_result = instance.execute(&work)?;

        // Push the work response result to the session.
        session.push_response(request_id, work_result);

        Ok(WorkStatus::Completed)
    }

    /// End the worker session with the given session id.
    pub fn end_session(&self, session_id: WorkerSessionId) -> Result<(), WorkerError> {
        let mut inner = self.inner.write().unwrap();
        log::debug!("Ending session with id: {}", session_id);
        inner
            .remove_session(session_id)
            .ok_or(WorkerError::SessionNotFound(session_id))?;
        Ok(())
    }
}
