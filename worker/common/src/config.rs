use crate::BackendBitmask;

pub type WorkerConfigFlags = u32;

// Config flags.
pub const WORK_CONFIG_FLAG_USE_CACHE: u32 = 1 << 0;
pub const WORK_CONFIG_FLAG_USE_THREAD_POOL: u32 = 1 << 1;

/// Configuration for a work request.
#[derive(Debug, Clone)]
pub struct WorkRequestConfig {
    /// Whether to use cache for the session.
    pub use_cache: bool,
    /// Whether to use thread pool for the session.
    pub use_thread_pool: bool,
}

impl WorkRequestConfig {
    /// Create a new work request config with the given parameters.
    pub const fn new(flags: WorkerConfigFlags) -> Self {
        let use_cache = (flags & WORK_CONFIG_FLAG_USE_CACHE) != 0;
        let use_thread_pool = (flags & WORK_CONFIG_FLAG_USE_THREAD_POOL) != 0;
        Self {
            use_cache,
            use_thread_pool,
        }
    }

    /// Convert the work request config back to flags.
    pub const fn to_flags(&self) -> WorkerConfigFlags {
        let mut flags = 0;
        if self.use_cache {
            flags |= WORK_CONFIG_FLAG_USE_CACHE;
        }
        if self.use_thread_pool {
            flags |= WORK_CONFIG_FLAG_USE_THREAD_POOL;
        }
        flags
    }

    /// Merge two configs using bitwise AND for the flags.
    pub const fn flags_and(&self, other: &Self) -> Self {
        let flags = self.to_flags() & other.to_flags();
        Self::new(flags)
    }
}

/// Configuration for worker session.
#[derive(Debug, Clone)]
pub struct WorkerSessionConfig {
    /// Work request configuration for the session.
    pub work: WorkRequestConfig,
    /// The backends to use for the session.
    pub backends: BackendBitmask,
}

impl WorkerSessionConfig {
    /// Create a new session config with the given parameters.
    pub fn new(flags: WorkerConfigFlags, backends: BackendBitmask) -> Self {
        let work = WorkRequestConfig::new(flags);
        Self { work, backends }
    }
}
