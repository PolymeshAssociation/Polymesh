use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

use crossbeam::channel::{Receiver, Sender, unbounded};

use crate::cache::*;
use crate::*;

pub const WORK_CACHE_CAPACITY: usize = 10_000;

lazy_static::lazy_static! {
    /// The global set of batch verifiers.
    static ref BATCH_VERIFIERS: BatchVerifiers = {
        BatchVerifiers::new(None)
    };
}

lazy_static::lazy_static! {
    /// Cache the responses for work requests in a LRU cache.
    static ref CACHE_WORK_RESPONSE: WorkRequestCache = {
        WorkRequestCache::new(WORK_CACHE_CAPACITY)
    };
}

pub fn cache_work_response(req_hash: WorkRequestHash, result: Result<WorkResponse, Error>) {
    CACHE_WORK_RESPONSE.insert(req_hash, result)
}

pub fn get_cached_work_response(req_hash: &WorkRequestHash) -> Option<Result<WorkResponse, Error>> {
    CACHE_WORK_RESPONSE.get(req_hash)
}

pub(crate) struct ExecutionContext {
    execution: WorkRequestExecution,
    use_cache: bool,
    tx: Sender<WorkRequestResult>,
    #[cfg(feature = "testing")]
    skip_verify: bool,
}

impl ExecutionContext {
    pub fn execute(&self, req: WorkRequest) {
        let req_id = self.execution.req_id;
        let req_hash = self.execution.req_hash;
        let result = req.execute(req_hash);
        if self.use_cache {
            cache_work_response(req_hash, result.clone());
        }
        let _ = self.tx.send(WorkRequestResult { id: req_id, result });
    }
}

#[derive(Debug)]
struct InnerBatchVerifiers {
    batches: HashMap<BatchId, BatchVerifier>,
    next_id: BatchId,
    /// Only available for benchmarking to isolate the proof verification
    /// costs from runtime costs.
    #[cfg(feature = "testing")]
    skip_verify: bool,
}

impl InnerBatchVerifiers {
    pub fn new() -> Self {
        Self {
            batches: Default::default(),
            next_id: 0,
            #[cfg(feature = "testing")]
            skip_verify: false,
        }
    }

    /// Create a new batch verifier and return its ID.
    pub fn create_batch(&mut self, use_cache: bool, use_pool: bool) -> BatchId {
        let id = self.next_id;
        self.next_id = id + 1;
        self.batches
            .insert(id, BatchVerifier::new(use_cache, use_pool));
        id
    }

    /// Set whether to use the thread pool for this batch.
    pub fn batch_use_thread_pool(&mut self, id: BatchId, use_pool: bool) -> Result<(), Error> {
        if let Some(batch) = self.batches.get_mut(&id) {
            batch.use_thread_pool = use_pool;
            Ok(())
        } else {
            return Err(Error::MissingBatch);
        }
    }

    /// Set whether to use the cache for this batch.
    pub fn batch_use_cache(&mut self, id: BatchId, use_cache: bool) -> Result<(), Error> {
        if let Some(batch) = self.batches.get_mut(&id) {
            batch.use_cache = use_cache;
            Ok(())
        } else {
            return Err(Error::MissingBatch);
        }
    }

    /// Only used when execution falls back to the runtime.
    pub(crate) fn batch_push_result(
        &mut self,
        id: BatchId,
        execution: &WorkRequestExecution,
        result: Result<WorkResponse, Error>,
    ) -> Result<(), Error> {
        if let Some(batch) = self.batches.get_mut(&id) {
            let req_id = execution.req_id;
            if batch.use_cache {
                cache_work_response(execution.req_hash, result.clone());
            }
            batch.push_result(WorkRequestResult { id: req_id, result });
            Ok(())
        } else {
            Err(Error::MissingBatch)
        }
    }

    /// Starts the execution of a work request by creating an execution context.
    ///
    /// The execution context is used to handle the different execution modes:
    /// - Runtime: The request is executed in the runtime.
    /// - Host: The request is executed in the host, blocking the current thread.
    /// - HostPool: The request is executed in the host, using a thread pool.
    ///
    pub(crate) fn batch_create_execution(
        &mut self,
        id: BatchId,
        req: &WorkRequest,
    ) -> Result<ExecutionContext, Error> {
        if let Some(batch) = self.batches.get_mut(&id) {
            let (req_id, tx) = batch.next_req();
            let execution = req.execution(req_id, batch.use_thread_pool);

            // Always use the cache for proof generation.
            let use_cache = match req.kind {
                WorkRequestKind::VerifyProof(_) => batch.use_cache,
                WorkRequestKind::GenerateProof(_) => true,
            };
            Ok(ExecutionContext {
                execution,
                tx,
                use_cache,
                #[cfg(feature = "testing")]
                skip_verify: self.skip_verify,
            })
        } else {
            Err(Error::MissingBatch)
        }
    }

    /// Finishes the batch and returns the batch verifier.
    pub fn batch_finish(&mut self, id: BatchId) -> Option<BatchVerifier> {
        #[cfg(feature = "testing")]
        if self.skip_verify {
            self.skip_verify = false;
        }
        self.batches.remove(&id)
    }

    /// Closes the batch and removes it from the set of batches.
    pub fn batch_close(&mut self, id: BatchId) {
        #[cfg(feature = "testing")]
        if self.skip_verify {
            self.skip_verify = false;
        }
        self.batches.remove(&id);
    }

    /// Only available for benchmarking to isolate the proof verification
    /// costs from runtime costs.
    #[cfg(feature = "testing")]
    pub fn set_skip_verify(&mut self, skip: bool) {
        self.skip_verify = skip;
    }
}

pub struct BatchVerifiers {
    inner: Arc<RwLock<InnerBatchVerifiers>>,
    pool: rayon::ThreadPool,
}

impl BatchVerifiers {
    pub fn new(threads: Option<usize>) -> Self {
        let mut builder = rayon::ThreadPoolBuilder::new();
        if let Some(threads) = threads {
            builder = builder.num_threads(threads);
        }
        let pool = builder.build().unwrap();
        Self {
            inner: Arc::new(RwLock::new(InnerBatchVerifiers::new())),
            pool,
        }
    }

    pub fn create_batch(use_cache: bool, use_pool: bool) -> BatchId {
        let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
        inner.create_batch(use_cache, use_pool)
    }

    pub fn use_thread_pool(id: BatchId, use_pool: bool) -> Result<(), Error> {
        let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
        inner.batch_use_thread_pool(id, use_pool)
    }

    pub fn use_cache(id: BatchId, use_cache: bool) -> Result<(), Error> {
        let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
        inner.batch_use_cache(id, use_cache)
    }

    pub fn submit(id: BatchId, req: WorkRequest) -> Result<WorkRequestExecution, Error> {
        let ctx = {
            let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
            inner.batch_create_execution(id, &req)?
        };
        let execution = ctx.execution.clone();

        if ctx.use_cache {
            if let Some(result) = get_cached_work_response(&execution.req_hash) {
                if req.kind.use_cached_value() {
                    log::debug!(
                        "Cache hit for {:?} request: hash={:?}",
                        req.kind,
                        execution.req_hash
                    );
                    Self::push_result(id, &execution, result)?;
                    return Ok(execution);
                } else {
                    log::debug!(
                        "Cache hit for {:?} request: hash={:?}, discarding cache value.",
                        req.kind,
                        execution.req_hash
                    );
                }
            } else {
                log::debug!(
                    "Cache miss for {:?} request: hash={:?}",
                    req.kind,
                    execution.req_hash
                );
            }
        }

        // Only available for benchmarking to isolate the proof verification
        #[cfg(feature = "testing")]
        match req.kind {
            WorkRequestKind::VerifyProof(_) => {
                // Only available for benchmarking to isolate the proof verification
                // costs from runtime costs.
                if ctx.skip_verify {
                    let result = req.skip_verify();
                    Self::push_result(id, &execution, result)?;
                    return Ok(execution);
                }
            }
            WorkRequestKind::GenerateProof(_) => {}
        }

        match execution.kind {
            ExecutionKind::Runtime => (),
            ExecutionKind::Host => {
                ctx.execute(req);
            }
            ExecutionKind::HostPool => {
                BATCH_VERIFIERS.pool.spawn(move || ctx.execute(req));
            }
        }
        Ok(execution)
    }

    pub fn get_or_wait_for(
        id: BatchId,
        req_id: WorkRequestId,
    ) -> Result<Result<WorkResponse, Error>, Error> {
        let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
        if let Some(batch) = inner.batches.get_mut(&id) {
            batch.get_or_wait_for(req_id)
        } else {
            Err(Error::MissingBatch)
        }
    }

    pub fn next_result(
        id: BatchId,
    ) -> Result<Option<(WorkRequestId, Result<WorkResponse, Error>)>, Error> {
        let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
        if let Some(batch) = inner.batches.get_mut(&id) {
            batch.next_result()
        } else {
            Err(Error::MissingBatch)
        }
    }

    /// Only used when execution falls back to the runtime or the response was cached.
    pub fn push_result(
        id: BatchId,
        execution: &WorkRequestExecution,
        result: Result<WorkResponse, Error>,
    ) -> Result<(), Error> {
        let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
        inner.batch_push_result(id, execution, result)
    }

    pub fn finish(id: BatchId) -> Option<BatchVerifier> {
        let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
        inner.batch_finish(id)
    }

    pub fn close(id: BatchId) {
        let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
        inner.batch_close(id);
    }

    /// Only available for benchmarking to isolate the proof verification
    /// costs from runtime costs.
    #[cfg(feature = "testing")]
    pub fn set_skip_verify(skip: bool) {
        let mut inner = BATCH_VERIFIERS.inner.write().unwrap();
        inner.set_skip_verify(skip);
    }
}

#[derive(Debug)]
pub struct BatchVerifier {
    tx: Sender<WorkRequestResult>,
    rx: Receiver<WorkRequestResult>,

    next_id: WorkRequestId,
    pending_count: usize,
    responses: BTreeMap<WorkRequestId, Result<WorkResponse, Error>>,

    use_cache: bool,
    use_thread_pool: bool,
}

impl BatchVerifier {
    pub fn new(use_cache: bool, use_thread_pool: bool) -> Self {
        let (tx, rx) = unbounded();
        Self {
            next_id: 0,
            tx,
            rx,
            pending_count: 0,
            responses: BTreeMap::new(),
            use_cache,
            use_thread_pool,
        }
    }

    pub fn next_req(&mut self) -> (WorkRequestId, Sender<WorkRequestResult>) {
        let id = self.next_id;
        self.next_id = id + 1;
        self.pending_count += 1;
        (id, self.tx.clone())
    }

    pub fn push_result(&mut self, result: WorkRequestResult) {
        self.pending_count = self.pending_count.saturating_sub(1);
        self.responses.insert(result.id, result.result);
    }

    pub fn get_or_wait_for(
        &mut self,
        req_id: WorkRequestId,
    ) -> Result<Result<WorkResponse, Error>, Error> {
        self.wait_for_pending(Some(req_id))?;
        self.responses
            .get(&req_id)
            .cloned()
            .ok_or(Error::VerifyFailed) // This should never happen.
    }

    pub fn next_result(
        &mut self,
    ) -> Result<Option<(WorkRequestId, Result<WorkResponse, Error>)>, Error> {
        self.wait_for_pending(None)?;
        Ok(self.responses.pop_first())
    }

    pub fn finalize(
        mut self,
    ) -> Result<BTreeMap<WorkRequestId, Result<WorkResponse, Error>>, Error> {
        self.wait_for_pending(None)?;
        let Self { tx, responses, .. } = self;
        drop(tx);
        Ok(responses)
    }

    pub fn wait_for_pending(&mut self, wait_for: Option<WorkRequestId>) -> Result<(), Error> {
        // If nothing is pending, return immediately.
        if self.pending_count == 0 {
            return Ok(());
        }
        if let Some(wait_for) = wait_for {
            if self.responses.contains_key(&wait_for) {
                return Ok(());
            }
        }
        while self.pending_count > 0 {
            let res = self.rx.recv().map_err(|err| {
                log::error!("Failed to recv Proof response: {err:?}");
                Error::VerifyFailed
            })?;
            self.pending_count = self.pending_count.saturating_sub(1);
            self.responses.insert(res.id, res.result);
            if let Some(wait_for) = wait_for {
                if res.id == wait_for {
                    break;
                }
            }
        }
        Ok(())
    }
}
