use crate::BackendBitmask;

pub type WorkerConfigFlags = u32;

pub type WorkerConfigFlagsAndBackends = u64;

pub fn pack_flags_and_backends(
    flags: WorkerConfigFlags,
    backends: BackendBitmask,
) -> WorkerConfigFlagsAndBackends {
    ((flags as WorkerConfigFlagsAndBackends) << 32) | (backends as WorkerConfigFlagsAndBackends)
}

pub fn unpack_flags_and_backends(
    value: WorkerConfigFlagsAndBackends,
) -> (WorkerConfigFlags, BackendBitmask) {
    let flags = (value >> 32) as WorkerConfigFlags;
    let backends = (value & 0xFFFFFFFF) as BackendBitmask;
    (flags, backends)
}

// Config flags.
pub const WORK_CONFIG_FLAG_USE_CACHE: u32 = 1 << 0;
pub const WORK_CONFIG_FLAG_USE_THREAD_POOL: u32 = 1 << 1;
/// Initialize the protocol module from the context data for this session.
///
/// Normally the runtime will initialize the protocol module for the session, since it will have saved context data to do fast initialization.
///
/// However if the context data has not been generated yet, then the runtime can start the module without initialization
/// and submit work request(s) to generate the context data.  For faster context generation multiple parallel work requests
/// can be summited to generate different parts of the context data in parallel.
pub const SESSION_CONFIG_FLAG_INIT_MODULE: u32 = 1 << 2;

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
    /// Initialize the module for this session.
    pub init_module: bool,
    /// The backends to use for the session.
    pub backends: BackendBitmask,
}

impl WorkerSessionConfig {
    /// Create a new session config with the given parameters.
    pub fn new(flags: WorkerConfigFlags, backends: BackendBitmask) -> Self {
        let work = WorkRequestConfig::new(flags);
        let init_module = (flags & SESSION_CONFIG_FLAG_INIT_MODULE) != 0;

        Self {
            work,
            init_module,
            backends,
        }
    }

    /// Convert the session config to a combined flags and backends value.
    pub fn to_flags_and_backends(&self) -> WorkerConfigFlagsAndBackends {
        let mut flags = self.work.to_flags();
        if self.init_module {
            flags |= SESSION_CONFIG_FLAG_INIT_MODULE;
        }
        pack_flags_and_backends(flags, self.backends)
    }
}
