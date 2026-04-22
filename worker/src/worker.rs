use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

use crate::{
    backend::{BackendManager, BackendManagerRef, BackendModuleLoader},
    cache::{
        modules::{ProtocolModuleInstance, ProtocolModuleRef},
        work::WorkRequestCache,
    },
};
use polymesh_worker_common::{
    BackendBitmask, Protocol, WorkRequest, WorkRequestHash, WorkRequestId, WorkResponseResult,
    WorkStatus, WorkerSessionId, config::*, error::*,
};

pub const WORK_CACHE_CAPACITY: usize = 10_000;

/// The mutable state of a worker session, which tracks the pending work requests and their results.
struct WorkerSessionInner {
    responses: BTreeMap<WorkRequestId, WorkResponseResult>,
    hashes: BTreeMap<WorkRequestId, WorkRequestHash>,

    next_id: WorkRequestId,
    pending_count: usize,
}

impl WorkerSessionInner {
    fn new() -> Self {
        Self {
            responses: BTreeMap::new(),
            hashes: BTreeMap::new(),

            next_id: 0,
            pending_count: 0,
        }
    }

    fn new_request(
        &mut self,
        req_hash_and_cached_value: Option<(WorkRequestHash, Option<WorkResponseResult>)>,
    ) -> WorkRequestId {
        let req_id = self.next_id;
        self.next_id += 1;
        self.pending_count += 1;

        if let Some((req_hash, cached_value)) = req_hash_and_cached_value {
            self.hashes.insert(req_id, req_hash);
            if let Some(cached_value) = cached_value {
                // If there is a cached value for the request, we can immediately push the cached response to the session and mark the request as completed.
                self.push_response(req_id, cached_value);
            }
        }
        req_id
    }

    fn push_response(
        &mut self,
        req_id: WorkRequestId,
        res: WorkResponseResult,
    ) -> Option<(WorkRequestHash, WorkResponseResult)> {
        // If the request has a hash, then the response should be cached, so we return the hash and response for caching after pushing the results to the session.
        let cache_resp = self
            .hashes
            .remove(&req_id)
            .map(|req_hash| (req_hash, res.clone()));

        self.responses.insert(req_id, res);
        self.pending_count -= 1;

        cache_resp
    }
}

pub type WorkerSessionRef = Arc<WorkerSession>;

pub struct WorkerSession {
    pub id: WorkerSessionId,
    pub config: WorkerSessionConfig,
    pub protocol: Protocol,
    pub module: Option<ProtocolModuleRef>,

    inner: RwLock<WorkerSessionInner>,
}

impl WorkerSession {
    /// Create a new worker session with the given id and config.
    pub fn new(
        id: WorkerSessionId,
        config: WorkerSessionConfig,
        protocol: Protocol,
        module: Option<ProtocolModuleRef>,
    ) -> WorkerSessionRef {
        Arc::new(Self {
            id,
            config,
            protocol,
            module,

            inner: RwLock::new(WorkerSessionInner::new()),
        })
    }

    /// New work request.
    pub fn new_request(
        &self,
        req_hash_and_cached_value: Option<(WorkRequestHash, Option<WorkResponseResult>)>,
    ) -> WorkRequestId {
        let mut inner = self.inner.write().unwrap();
        inner.new_request(req_hash_and_cached_value)
    }

    /// Push the work response result for the given request id.
    pub fn push_response(
        &self,
        req_id: WorkRequestId,
        res: WorkResponseResult,
    ) -> Option<(WorkRequestHash, WorkResponseResult)> {
        let mut inner = self.inner.write().unwrap();
        inner.push_response(req_id, res)
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

    /// Merge the session config with a work request config to get the effective config for the work request execution.
    pub fn merge_config(&self, req_config: WorkRequestConfig) -> WorkRequestConfig {
        req_config.flags_and(&self.config.work)
    }

    /// Get module instance for the session's protocol, if the session has a module reference.
    pub fn get_protocol_instance(&self) -> Option<ProtocolModuleInstance> {
        let module = self.module.as_ref()?;
        module.get_instance()
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
        module: Option<ProtocolModuleRef>,
    ) -> WorkerSessionRef {
        let session_id = self.next_session_id;
        self.next_session_id += 1;
        let session = WorkerSession::new(session_id, config, protocol, module);
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
    cache: WorkRequestCache,
}

impl PolymeshWorker {
    /// Create the polymesh worker shared state.
    pub fn new() -> PolymeshWorkerRef {
        Arc::new(Self {
            inner: RwLock::new(PolymeshWorkerInner::new()),
            backend: BackendManager::new(),
            cache: WorkRequestCache::new(WORK_CACHE_CAPACITY),
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
        let module = self
            .backend
            .load_protocol(config.backends, protocol, loader);

        let mut inner = self.inner.write().unwrap();
        let session = inner.create_session(config, protocol, module);
        log::debug!("Started session with id: {}", session.id);
        session
    }

    /// Execute a protocol-specific work request for the given session id.
    pub fn session_execute_request(
        &self,
        session_id: WorkerSessionId,
        config: WorkRequestConfig,
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

        // Merge the session config with the work request config to get the effective config for the work request execution.
        let config = session.merge_config(config);

        let req_hash_cached_value = if config.use_cache {
            let hash = work.hash_using(crate::blake2b256_hash);

            // Check if the response for the work request is already cached, and if so, return the cached response.
            let cached = self.cache.get(&hash).and_then(|cached_resp| {
                if cfg!(feature = "testing") {
                    log::debug!(
                        "Cache hit for work request with hash: {:x?}, discarding value (testing feature flag).",
                        hash
                    );
                    None
                } else {
                    // In production, we return the cached response to save time and resources.
                    log::debug!("Cache hit for work request with hash: {:x?}", hash);
                    Some(cached_resp)
                }
            });

            Some((hash, cached))
        } else {
            None
        };

        // Create a new request id.
        let request_id = session.new_request(req_hash_cached_value);
        log::debug!(
            "Submitted work request with id: {} for session id: {}",
            request_id,
            session_id
        );

        // Try executing the work request and get the status.  If execution fails, fallback to runtime execution by returning a special status.
        let status = match self.try_execute_work(session, config, request_id, work) {
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
        mut config: WorkRequestConfig,
        work: WorkRequest,
    ) -> Result<WorkResponseResult, WorkerError> {
        // Force the `use_thread_pool` flag to be false for synchronous execution, since the caller is waiting for the results and we don't want to spawn a new thread for the execution.
        config.use_thread_pool = false;

        // First get the session for the given session id.
        let inner = self.inner.read().unwrap();
        let session = match inner.get_session(session_id) {
            Ok(session) => session,
            Err(err) => {
                log::error!("Failed to submit work request: {err:?}");
                return Err(WorkerError::SessionNotFound(session_id));
            }
        };

        // Merge the session config with the work request config to get the effective config for the work request execution.
        let config = session.merge_config(config);

        // If caching is enabled, hash the work request to get the cache key and check if the response for the work request is already cached, and if so, return the cached response.
        let req_hash = if config.use_cache {
            let hash = work.hash_using(crate::blake2b256_hash);

            // Check if the response for the work request is already cached, and if so, return the cached response.
            if let Some(cached_resp) = self.cache.get(&hash) {
                if cfg!(feature = "testing") {
                    log::debug!(
                        "Cache hit for work request with hash: {:x?}, discarding value (testing feature flag).",
                        hash
                    );
                } else {
                    // In production, we return the cached response to save time and resources.
                    log::debug!("Cache hit for work request with hash: {:x?}", hash);
                    return Ok(cached_resp);
                }
            }

            Some(hash)
        } else {
            None
        };

        // Get the protocol module instance for the session's protocol.
        let mut instance = session
            .get_protocol_instance()
            .or_else(|| self.backend.get_protocol_instance(session.protocol))
            .ok_or(WorkerError::ModuleExecutionFailed)?;

        // Execute the work request using the protocol module instance.
        let result = instance.execute(&work)?;

        if let Some(req_hash) = req_hash {
            self.cache.insert(req_hash, result.clone());
        }

        Ok(result)
    }

    fn push_response(
        &self,
        session: WorkerSessionRef,
        request_id: WorkRequestId,
        result: WorkResponseResult,
    ) -> Result<(), WorkerError> {
        // Push the work response result to the session.
        if let Some((req_hash, cache_resp)) = session.push_response(request_id, result) {
            // If the request has a hash, then the response should be cached, so we insert the hash and response to the cache.
            self.cache.insert(req_hash, cache_resp);
        }

        Ok(())
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

        self.push_response(session, request_id, result)
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
        _config: WorkRequestConfig,
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
        self.push_response(session, request_id, work_result)?;

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
