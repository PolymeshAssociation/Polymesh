use crate::BackendBitmask;

pub type WorkerConfigFlags = u32;

// Config flags.
pub const WORKER_CONFIG_FLAG_USE_CACHE: u32 = 1 << 0;
pub const WORKER_CONFIG_FLAG_USE_THREAD_POOL: u32 = 1 << 1;

/// Configuration for worker session.
#[derive(Debug, Clone)]
pub struct WorkerSessionConfig {
    /// Whether to use cache for the session.
    pub use_cache: bool,
    /// Whether to use thread pool for the session.
    pub use_thread_pool: bool,
    /// The backends to use for the session.
    pub backends: BackendBitmask,
    #[cfg(feature = "testing")]
    /// Whether to skip verification for the session. This is only for testing and should not be used in production.
    pub skip_verify: bool,
}

impl WorkerSessionConfig {
    /// Create a new session config with the given parameters.
    pub fn new(flags: WorkerConfigFlags, backends: BackendBitmask) -> Self {
        let use_cache = (flags & WORKER_CONFIG_FLAG_USE_CACHE) != 0;
        let use_thread_pool = (flags & WORKER_CONFIG_FLAG_USE_THREAD_POOL) != 0;
        Self {
            use_cache,
            use_thread_pool,
            backends,
            #[cfg(feature = "testing")]
            skip_verify: false,
        }
    }
}
