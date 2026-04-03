use std::collections::{HashMap, LinkedList};
use std::sync::{Arc, RwLock};

use crate::{Error, WorkRequestHash, WorkResponse};

/// A simple cache for work request responses.
pub struct WorkRequestCache {
    cache: Arc<RwLock<WorkRequestCacheInner>>,
}

impl WorkRequestCache {
    /// Create a new cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(WorkRequestCacheInner::new(capacity))),
        }
    }

    /// Get a cached response for the given request hash.
    pub fn get(&self, req_hash: &WorkRequestHash) -> Option<Result<WorkResponse, Error>> {
        let cache = self.cache.read().unwrap();
        cache.get(req_hash)
    }

    /// Insert a response into the cache.
    pub fn insert(&self, req_hash: WorkRequestHash, resp: Result<WorkResponse, Error>) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(req_hash, resp);
    }
}

struct WorkRequestCacheInner {
    map: HashMap<WorkRequestHash, Result<WorkResponse, Error>>,
    order: LinkedList<WorkRequestHash>,
    capacity: usize,
}

impl WorkRequestCacheInner {
    fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: LinkedList::new(),
            capacity,
        }
    }

    fn get(&self, req_hash: &WorkRequestHash) -> Option<Result<WorkResponse, Error>> {
        self.map.get(req_hash).cloned()
    }

    fn insert(&mut self, req_hash: WorkRequestHash, resp: Result<WorkResponse, Error>) {
        if !self.map.contains_key(&req_hash) {
            if self.map.len() >= self.capacity {
                if let Some(oldest) = self.order.pop_front() {
                    self.map.remove(&oldest);
                }
            }
            self.order.push_back(req_hash.clone());
        }
        self.map.insert(req_hash, resp);
    }
}
