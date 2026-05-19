use codec::Encode;
use parking_lot::RwLock;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    sync::Arc,
};

use crossbeam::channel::{Receiver, Sender, unbounded};

use crate::{
    backend::{BackendManager, BackendManagerRef, BackendModuleLoader},
    cache::{
        modules::{ProtocolModuleInstance, ProtocolModuleRef},
        work::WorkRequestCache,
    },
};
use polymesh_worker_common::{
    BackendBitmask, BackendCodeHash, Protocol, WorkRequest, WorkRequestHash, WorkRequestId,
    WorkResponseResult, WorkStatus, WorkerSessionId, config::*, error::*,
};

pub const WORK_CACHE_CAPACITY: usize = 10_000;

struct PushWorkResponse {
    request_id: WorkRequestId,
    // If `None` there was a host worker error.
    result: Option<WorkResponseResult>,
    cached: bool,
}

/// The mutable state of a worker session, which tracks the pending work requests and their results.
struct WorkerSessionInner {
    responses: BTreeMap<WorkRequestId, WorkResponseResult>,
    hashes: BTreeMap<WorkRequestId, WorkRequestHash>,

    next_id: WorkRequestId,
    pending_count: usize,

    rx: Receiver<PushWorkResponse>,
}

impl WorkerSessionInner {
    fn new(rx: Receiver<PushWorkResponse>) -> Self {
        Self {
            responses: BTreeMap::new(),
            hashes: BTreeMap::new(),

            next_id: 0,
            pending_count: 0,
            rx,
        }
    }

    fn new_request(
        &mut self,
        req_hash: Option<WorkRequestHash>,
        cached_value: Option<WorkResponseResult>,
    ) -> WorkRequestId {
        let req_id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.pending_count = self.pending_count.saturating_add(1);

        if let Some(req_hash) = req_hash {
            self.hashes.insert(req_id, req_hash);
            if let Some(cached_value) = cached_value {
                // If there is a cached value for the request, we can immediately push the cached response to the session and mark the request as completed.
                self.push_response(req_id, Some(cached_value));
            }
        }
        req_id
    }

    fn push_response(
        &mut self,
        req_id: WorkRequestId,
        res: Option<WorkResponseResult>,
    ) -> Option<(WorkRequestHash, WorkResponseResult)> {
        // If the request has a hash, then the response should be cached, so we return the hash and response for caching after pushing the results to the session.
        let req_hash = self.hashes.remove(&req_id);

        // If we have the request hash and a response, we return them for caching.  Otherwise, we return None, which indicates that there is no cacheable response for the request.
        let cache_resp = match (req_hash, res.as_ref()) {
            (Some(hash), Some(res)) => Some((hash, res.clone())),
            _ => None,
        };

        // Even if there is no response (host worker error), we still consider the request as completed and decrease the pending count (to avoid waiting indefinitely).
        self.pending_count = self.pending_count.saturating_sub(1);

        if let Some(res) = res {
            self.responses.insert(req_id, res);
        }

        cache_resp
    }

    pub fn get_or_wait_for(
        &mut self,
        req_id: WorkRequestId,
        cache: Option<&WorkRequestCache>,
    ) -> Option<WorkResponseResult> {
        self.wait_for_pending(Some(req_id), cache);
        self.responses.get(&req_id).cloned()
    }

    pub fn next_result(
        &mut self,
        cache: Option<&WorkRequestCache>,
    ) -> Option<(WorkRequestId, WorkResponseResult)> {
        self.wait_for_pending(None, cache);
        self.responses.pop_first()
    }

    pub fn wait_for_pending(
        &mut self,
        wait_for: Option<WorkRequestId>,
        cache: Option<&WorkRequestCache>,
    ) {
        // If nothing is pending, return immediately.
        if self.pending_count == 0 {
            return;
        }
        if let Some(wait_for) = wait_for {
            if self.responses.contains_key(&wait_for) {
                return;
            }
        }
        while self.pending_count > 0 {
            let res = match self.rx.recv() {
                Ok(res) => res,
                Err(err) => {
                    log::error!("Failed to receive work response: {err:?}");
                    return;
                }
            };

            if let Some((req_hash, cache_resp)) = self.push_response(res.request_id, res.result) {
                if !res.cached {
                    // If the response is not already cached, we insert it into the cache for future reuse.
                    if let Some(cache) = cache {
                        cache.insert(req_hash, cache_resp);
                    }
                }
            }

            if wait_for == Some(res.request_id) {
                // Got the request we are waiting for, return the response.
                return;
            }
        }
        // No more pending requests.
    }
}

#[derive(Clone)]
pub struct WorkerSessionRef(Arc<WorkerSession>);

impl Deref for WorkerSessionRef {
    type Target = WorkerSession;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct WorkerSession {
    pub id: WorkerSessionId,
    pub config: WorkerSessionConfig,
    pub protocol: Protocol,
    pub module: Option<ProtocolModuleRef>,

    tx: Sender<PushWorkResponse>,

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
        let (tx, rx) = unbounded();
        WorkerSessionRef(Arc::new(Self {
            id,
            config,
            protocol,
            module,

            tx,

            inner: RwLock::new(WorkerSessionInner::new(rx)),
        }))
    }

    /// Get the module code hash if the session has a module reference, otherwise get the protocol hash as the fallback.
    pub fn module_or_protocol_hash(&self) -> BackendCodeHash {
        if let Some(module) = &self.module {
            module.code_hash()
        } else {
            self.protocol.using_encoded(crate::blake2b256_hash)
        }
    }

    /// New work request.
    pub fn new_request(
        &self,
        req_hash: Option<WorkRequestHash>,
        cached_value: Option<WorkResponseResult>,
    ) -> WorkRequestId {
        let mut inner = self.inner.write();
        inner.new_request(req_hash, cached_value)
    }

    /// Push the work response result for the given request id.
    pub fn push_response(
        &self,
        request_id: WorkRequestId,
        result: Option<WorkResponseResult>,
        cached: bool,
    ) {
        if let Err(err) = self.tx.send(PushWorkResponse {
            request_id,
            result,
            cached,
        }) {
            let PushWorkResponse {
                request_id, result, ..
            } = err.0;

            log::warn!(
                "Failed to push work response to session for request id {}.",
                request_id
            );
            // Fallback to directly pushing the response.
            let mut inner = self.inner.write();
            inner.push_response(request_id, result);
        }
    }

    /// Get the work response result for the given request id.
    pub fn get_response(
        &self,
        req_id: WorkRequestId,
        cache: Option<&WorkRequestCache>,
    ) -> Option<WorkResponseResult> {
        let mut inner = self.inner.write();
        inner.get_or_wait_for(req_id, cache)
    }

    /// Get the next completed work request and its result.
    pub fn next_result(
        &self,
        cache: Option<&WorkRequestCache>,
    ) -> Option<(WorkRequestId, WorkResponseResult)> {
        let mut inner = self.inner.write();
        inner.next_result(cache)
    }

    /// Get the number of requests in the session.
    pub fn num_requests(&self) -> u32 {
        let inner = self.inner.read();
        inner.next_id
    }

    /// Merge the session config with a work request config to get the effective config for the work request execution.
    pub fn merge_config(&self, req_config: &WorkRequestConfig) -> WorkRequestConfig {
        req_config.flags_and(&self.config.work)
    }

    /// Get protocol module from the session, if the session has a module reference.  Otherwise, return None, and the caller can try getting the protocol module from the backend.
    pub fn get_protocol_module(&self) -> Option<ProtocolModuleRef> {
        self.module.clone()
    }

    /// Get module instance for the session's protocol, if the session has a module reference.
    pub fn get_protocol_instance(&self) -> Option<ProtocolModuleInstance> {
        let module = self.module.as_ref()?;
        module.get_instance()
    }

    // Try executing the work request on the host and return the status.
    fn try_execute_work(
        &self,
        module: ProtocolModuleRef,
        request_id: WorkRequestId,
        work: WorkRequest,
        cache: Option<(WorkRequestHash, WorkRequestCache)>,
    ) -> Result<WorkStatus, WorkerError> {
        // Get the protocol module instance for the session's protocol.
        let mut instance = module
            .get_instance()
            .ok_or(WorkerError::ModuleExecutionFailed)?;

        // Execute the work request using the protocol module instance.
        let result = instance.execute(&work)?;

        // If caching is enabled, we insert the result into the cache for future reuse.
        let cached = if let Some((req_hash, cache)) = cache {
            cache.insert(req_hash, result.clone());
            true
        } else {
            false
        };

        // Push the work response result to the session.
        self.push_response(request_id, Some(result), cached);

        Ok(WorkStatus::Completed)
    }
}

impl WorkerSessionRef {
    /// Hash the work request using the module code hash and the work request data, to get the cache key for caching the work response.
    pub fn hash_request(&self, work: &WorkRequest) -> WorkRequestHash {
        let code_hash = self.module_or_protocol_hash();
        (code_hash, work).using_encoded(crate::blake2b256_hash)
    }

    /// Execute a protocol-specific work request for the given session id.
    pub fn execute_request(
        &self,
        worker: &PolymeshWorker,
        config: &WorkRequestConfig,
        work: WorkRequest,
    ) -> (WorkRequestId, WorkStatus) {
        // Merge the session config with the work request config to get the effective config for the work request execution.
        let config = self.merge_config(config);

        let (req_hash, cached_value) = if config.use_cache {
            let hash = self.hash_request(&work);

            // Check if the response for the work request is already cached, and if so, return the cached response.
            let cached = worker.cache.get(&hash).and_then(|cached_resp| {
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

            (Some(hash), cached)
        } else {
            (None, None)
        };

        // If we got a cached value, then we can skip the execution.
        // The cached value will be saved to the session.
        let skip_execution = cached_value.is_some();

        // Create a new request id.
        let request_id = self.new_request(req_hash, cached_value);
        log::debug!(
            "Submitted work request with id: {} for session id: {}",
            request_id,
            self.id
        );

        if skip_execution {
            log::debug!(
                "Skipping execution for work request with id: {} since we have a cached response.",
                request_id
            );
            return (request_id, WorkStatus::Completed);
        }

        // Get the protocol module either from the session, if we fail to get the protocol module, we return an error status to fallback to runtime execution.
        let Some(module) = self.get_protocol_module() else {
            log::error!(
                "Failed to get protocol module for protocol {:?} to execute work request.",
                self.protocol
            );
            return (request_id, WorkStatus::ExecutionFailedFallbackToRuntime);
        };

        // Execute closure to support either thread pool execution or direct execution, and if execution fails, we return an error status to fallback to runtime execution.
        let cache = req_hash.map(|hash| (hash, worker.cache.clone()));

        let execute = move |session: &WorkerSession, is_thread: bool| {
            match session.try_execute_work(module, request_id, work, cache) {
                Ok(status) => status,
                Err(err) => {
                    log::error!("Failed to execute work request: {err:?}");
                    if is_thread {
                        // We need to push a host worker error response to the session for the request, since the caller is waiting for the response and we don't want to wait indefinitely.
                        session.push_response(request_id, None, false);
                    }
                    WorkStatus::ExecutionFailedFallbackToRuntime
                }
            }
        };

        let status = match (config.use_thread_pool, &worker.pool) {
            (true, Some(pool)) => {
                log::debug!(
                    "Spawning a new thread to execute work request with id: {} for session id: {}.",
                    request_id,
                    self.id
                );

                let session = self.clone();
                pool.spawn(move || {
                    execute(&session, true);
                });

                WorkStatus::Pending
            }
            _ => {
                // If we don't have a thread pool, we execute the work request directly on the current thread.
                log::debug!(
                    "Executing work request on the current thread since thread pool is not available."
                );
                execute(self, false)
            }
        };

        (request_id, status)
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
    pool: Option<rayon::ThreadPool>,
}

impl PolymeshWorker {
    /// Create the polymesh worker shared state.
    pub fn new() -> PolymeshWorkerRef {
        let mut builder = rayon::ThreadPoolBuilder::new();

        // Get env variable `POLYMESH_WORKER_NUM_THREADS` to set the number of threads for the worker thread pool.
        let num_threads = std::env::var("POLYMESH_WORKER_NUM_THREADS")
            .ok()
            .and_then(|val| val.parse::<usize>().ok());
        if let Some(num_threads) = num_threads {
            builder = builder.num_threads(num_threads);
        }

        // Add the `AbortGuard` to rayon thread pool.  So panics from threads do not bring down the entire process, but instead are caught and logged, and the work request is rejected.
        let builder = builder.spawn_handler(|thread| {
            let mut b = std::thread::Builder::new();
            if let Some(name) = thread.name() {
                b = b.name(name.to_owned());
            }
            if let Some(stack_size) = thread.stack_size() {
                b = b.stack_size(stack_size);
            }
            b.spawn(|| {
                let _guard = sp_panic_handler::AbortGuard::force_unwind();
                thread.run()
            })?;
            Ok(())
        });

        let pool = match builder.build() {
            Ok(pool) => Some(pool),
            Err(err) => {
                log::error!("Failed to create thread pool for worker: {err:?}");
                None
            }
        };
        Arc::new(Self {
            inner: RwLock::new(PolymeshWorkerInner::new()),
            backend: BackendManager::new(),
            cache: WorkRequestCache::new(WORK_CACHE_CAPACITY),
            pool,
        })
    }

    /// Get the session for the given session id.
    pub fn get_session(
        &self,
        session_id: WorkerSessionId,
    ) -> Result<WorkerSessionRef, WorkerError> {
        let inner = self.inner.read();
        inner.get_session(session_id)
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

        let mut inner = self.inner.write();
        let session = inner.create_session(config, protocol, module);
        log::debug!("Started session with id: {}", session.id);
        session
    }

    /// Execute a batch of work requests in the given session.
    pub fn session_execute_batch(
        &self,
        session_id: WorkerSessionId,
        config: WorkRequestConfig,
        batch: Vec<WorkRequest>,
    ) -> Vec<(WorkRequestId, WorkStatus)> {
        // First get the session for the given session id.
        let session = match self.get_session(session_id) {
            Ok(session) => session,
            Err(err) => {
                log::error!("Failed to submit work request: {err:?}");
                return vec![];
            }
        };

        batch
            .into_iter()
            .map(|work| session.execute_request(self, &config, work))
            .collect()
    }

    /// Execute a protocol-specific work request for the given session id.
    pub fn session_execute_request(
        &self,
        session_id: WorkerSessionId,
        config: WorkRequestConfig,
        work: WorkRequest,
    ) -> (WorkRequestId, WorkStatus) {
        // First get the session for the given session id.
        let session = match self.get_session(session_id) {
            Ok(session) => session,
            Err(err) => {
                log::error!("Failed to submit work request: {err:?}");
                return (0, WorkStatus::SessionNotFound);
            }
        };

        session.execute_request(self, &config, work)
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
        let session = self.get_session(session_id)?;

        // Merge the session config with the work request config to get the effective config for the work request execution.
        let config = session.merge_config(&config);

        // If caching is enabled, hash the work request to get the cache key and check if the response for the work request is already cached, and if so, return the cached response.
        let req_hash = if config.use_cache {
            let hash = session.hash_request(&work);

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
            .ok_or(WorkerError::ModuleExecutionFailed)?;

        // Execute the work request using the protocol module instance.
        let result = instance.execute(&work)?;

        if let Some(req_hash) = req_hash {
            self.cache.insert(req_hash, result.clone());
        }

        Ok(result)
    }

    /// Push the result of a work request execution back to the worker for the given session and request id.
    pub fn session_push_result(
        &self,
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
        result: WorkResponseResult,
    ) -> Result<(), WorkerError> {
        // First get the session for the given session id.
        let session = self.get_session(session_id)?;

        // Push the work response result to the session.
        session.push_response(request_id, Some(result), false);

        Ok(())
    }

    /// Get the number of requests in the session.
    pub fn session_num_requests(&self, session_id: WorkerSessionId) -> Result<u32, WorkerError> {
        // First get the session for the given session id.
        let session = self.get_session(session_id)?;

        Ok(session.num_requests())
    }

    /// Get the result of a work request execution for the given session and request id.
    pub fn session_get_result(
        &self,
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
    ) -> Result<WorkResponseResult, WorkerError> {
        // First get the session for the given session id.
        let session = self.get_session(session_id)?;

        // Get the work response result for the given request id.
        let result = session
            .get_response(request_id, Some(&self.cache))
            .ok_or(WorkerError::SessionRequestNotFound(session_id, request_id))?;
        Ok(result)
    }

    /// Get the next completed work request and its result for the given session id.
    pub fn session_next_result(
        &self,
        session_id: WorkerSessionId,
    ) -> Result<Option<(WorkRequestId, WorkResponseResult)>, WorkerError> {
        // First get the session for the given session id.
        let session = self.get_session(session_id)?;

        // Get the next completed work request and its result.
        Ok(session.next_result(Some(&self.cache)))
    }

    /// End the worker session with the given session id.
    pub fn end_session(&self, session_id: WorkerSessionId) -> Result<(), WorkerError> {
        let mut inner = self.inner.write();
        log::debug!("Ending session with id: {}", session_id);
        inner
            .remove_session(session_id)
            .ok_or(WorkerError::SessionNotFound(session_id))?;
        Ok(())
    }
}
