use schnellru::{ByLength, LruMap};
use std::sync::{Arc, RwLock};

use polymesh_worker_common::{WorkRequestHash, WorkResponseResult};

/// A simple cache for work request responses.
pub struct WorkRequestCache {
    cache: Arc<RwLock<LruMap<WorkRequestHash, WorkResponseResult, ByLength>>>,
}

impl WorkRequestCache {
    /// Create a new cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruMap::new(ByLength::new(capacity as u32)))),
        }
    }

    /// Get a cached response for the given request hash.
    pub fn get(&self, req_hash: &WorkRequestHash) -> Option<WorkResponseResult> {
        let mut cache = self.cache.write().unwrap();
        cache.get(req_hash).cloned()
    }

    /// Insert a response into the cache.
    pub fn insert(&self, req_hash: WorkRequestHash, resp: WorkResponseResult) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(req_hash, resp);
    }
}
