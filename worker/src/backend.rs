use std::sync::{Arc, RwLock};

use codec::Encode;
use polymesh_worker_common::*;

use crate::cache::modules::{BackendModuleCache, ProtocolModuleRef};

#[cfg(feature = "polkavm")]
mod polkavm;
#[cfg(feature = "wasmer")]
mod wasmer;
#[cfg(feature = "wasmtime")]
mod wasmtime;

#[cfg(feature = "std")]
lazy_static::lazy_static! {
    /// Global backends.
    pub static ref BACKENDS: RwLock<Backends> = {
        RwLock::new(Backends::new(None))
    };
}

pub struct Backends {
    pub backends: Vec<Box<dyn Backend>>,
}

impl Backends {
    pub fn init_global_backends<N: Backend>(kinds: &[BackendKind]) {
        let native = N::new_boxed().ok();
        // Try loading all backends and see which ones are available.
        *BACKENDS.write().unwrap() = Self::with_backends(kinds, native);
    }

    pub fn new(native: Option<Box<dyn Backend>>) -> Self {
        // Try loading all backends and see which ones are available.
        Self::with_backends(
            &[
                BackendKind::Native,
                BackendKind::PolkaVM,
                BackendKind::Wasmer,
                BackendKind::Wasmtime,
            ],
            native,
        )
    }

    /// Load only the provided backends
    pub fn with_backends(kinds: &[BackendKind], mut native: Option<Box<dyn Backend>>) -> Self {
        let mut backends: Vec<Box<dyn Backend>> = Vec::new();
        for kind in kinds {
            match kind {
                BackendKind::Native => {
                    if let Some(native) = native.take() {
                        backends.push(native);
                    }
                }
                BackendKind::PolkaVM => {
                    #[cfg(feature = "polkavm")]
                    {
                        match polkavm::PolkavmBackend::new_boxed() {
                            Ok(backend) => backends.push(backend as _),
                            Err(err) => log::warn!(
                                "Failed to initialize PolkaVM backend, disabling it: {err:?}"
                            ),
                        }
                    }
                }
                BackendKind::Wasmtime => {
                    #[cfg(feature = "wasmtime")]
                    {
                        match wasmtime::WasmtimeBackend::new_boxed() {
                            Ok(backend) => backends.push(backend as _),
                            Err(err) => log::warn!(
                                "Failed to initialize Wasmtime backend, disabling it: {err:?}"
                            ),
                        }
                    }
                }
                BackendKind::Wasmer => {
                    #[cfg(feature = "wasmer")]
                    {
                        match wasmer::WasmerBackend::new_boxed() {
                            Ok(backend) => backends.push(backend as _),
                            Err(err) => log::warn!(
                                "Failed to initialize Wasmer backend, disabling it: {err:?}"
                            ),
                        }
                    }
                }
            }
        }
        Self { backends }
    }

    /// Try loading a module for the given protocol and version from the first backend that supports it.
    pub fn load_module(
        &self,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<Box<dyn BackendModule>> {
        for backend in &self.backends {
            let kind = backend.kind();
            if let Some(module_bytes) = loader.get_module_bytes(protocol, kind) {
                if let Some(module) = backend.load_module(&module_bytes) {
                    return Some(module);
                }
            }
        }
        None
    }

    /// Get avaiable backends.
    pub fn get_available_backends() -> Vec<BackendKind> {
        BACKENDS
            .read()
            .unwrap()
            .backends
            .iter()
            .map(|b| b.kind())
            .collect()
    }
}

pub type BackendManagerRef = Arc<BackendManager>;

pub struct BackendManager {
    inner: RwLock<BackendModuleCache>,
}

impl BackendManager {
    /// Create a new backend manager with all available backends.
    pub fn new() -> BackendManagerRef {
        Arc::new(Self {
            inner: RwLock::new(BackendModuleCache::new()),
        })
    }

    /// Try loading a protocol.
    pub fn load_protocol(
        &self,
        allowed: BackendBitmask,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<ProtocolModuleRef> {
        if protocol.is_none() {
            return None;
        }

        self.inner
            .write()
            .unwrap()
            .load_protocol(allowed, protocol, loader)
    }
}

pub trait BackendModuleLoader {
    /// Try loading the protocol module config hash for the given protocol.
    ///
    /// The config hash can be used to check if the protocol module has been updated and needs to be reloaded.
    fn get_protocol_module_config_hash(
        &mut self,
        protocol: Protocol,
    ) -> Option<ProtocolModuleConfigHash>;

    /// Try loading the prtocol module config for the given protocol and config hash.
    fn get_protocol_module_config(
        &mut self,
        protocol: Protocol,
        config_hash: ProtocolModuleConfigHash,
    ) -> Option<ProtocolModuleConfig>;

    /// Try loading a module for the given protocol.
    fn get_module_code_bytes(
        &mut self,
        protocol: Protocol,
        kind: BackendModuleKind,
        code_hash: BackendCodeHash,
    ) -> Option<Vec<u8>>;

    /// Try loading the protocol context for the given protocol and context hash.
    fn get_module_context_bytes(
        &mut self,
        protocol: Protocol,
        ctx_hash: BackendContextHash,
    ) -> Option<Vec<u8>>;

    /// Try loading a module for the given protocol, version and backend kind.
    ///
    /// This allows the PolkaVM and Wasmtime backends to load modulbe blobs from Substrate's chain storage.
    fn get_module_bytes(&mut self, protocol: Protocol, kind: BackendKind) -> Option<Vec<u8>> {
        if let Some(config_hash) = self.get_protocol_module_config_hash(protocol) {
            if let Some(config) = self.get_protocol_module_config(protocol, config_hash) {
                let module_kind = kind.to_module_kind();
                for module in config.modules {
                    if module.module_kind == module_kind {
                        return self.get_module_code_bytes(protocol, module_kind, module.code_hash);
                    }
                }
            }
        }
        None
    }
}

pub trait BackendModuleInstance: Send + Sync {
    /// Allocate a scratch buffer from the module, with at least the given size.
    ///
    /// The backend/module can use a static scratch buffer or allocate a new one for each request, but it must ensure that the buffer is at least `min_size` bytes and return its pointer and size.
    ///
    /// TODO: Take a closure to automatically release the scratch buffer.
    fn allocate_scratch_pad(&mut self, min_size: u32) -> Result<(u32, u32), WorkerError>;

    /// Release the scratch buffer allocated for the current request, if the backend/module allocates a new one for each request.  If the backend/module uses a static scratch buffer, this function can be a no-op.
    fn release_scratch_pad(&mut self, ptr: u32, size: u32) -> Result<(), WorkerError>;

    /// Write data to the module's memory from the given buffer.
    fn write_memory(&mut self, ptr: u32, data: &[u8]) -> Result<(), WorkerError>;

    /// Read data from the module's memory into the given buffer.
    fn read_memory_into(&mut self, ptr: u32, buffer: &mut [u8]) -> Result<(), WorkerError>;

    /// Read data from the module's memory into a new buffer and return it.
    fn read_memory(&mut self, ptr: u32, len: u32) -> Result<Vec<u8>, WorkerError> {
        let mut buffer = vec![0u8; len as usize];
        self.read_memory_into(ptr, &mut buffer)?;
        Ok(buffer)
    }

    /// Call `initialize` function.
    fn call_initialize(&mut self, params_len: u32, save: u32) -> Result<u64, WorkerError>;

    /// Call `execute` function.
    ///
    /// The return value is a fat pointer to the response.  To return a protocol error, the `len` of the fat pointer will be set to `u32::MAX` and the `ptr` will be the error code as a `u32`.
    fn call_execute(&mut self, req_len: u32) -> Result<u64, WorkerError>;

    /// Initialize the module instance.
    fn initialize(&mut self, load_ctx: Option<&[u8]>) -> Result<u32, WorkerError> {
        let res_fat_ptr = if let Some(ctx) = load_ctx {
            let (ptr, size) = self.allocate_scratch_pad(ctx.len() as u32)?;
            // If we have a load context, write it to the scratch buffer and call the module's `initialize` function to load it.
            if let Err(err) = self.write_memory(ptr, ctx) {
                log::error!("Failed to write load context to scratch buffer: {err:?}");
                self.release_scratch_pad(ptr, size)?;
                return Err(err);
            }
            match self.call_initialize(ctx.len() as u32, 0) {
                Ok(fat_ptr) => fat_ptr,
                Err(err) => {
                    log::error!("Failed to call initialize with load context: {err:?}");
                    self.release_scratch_pad(ptr, size)?;
                    return Err(err);
                }
            }
        } else {
            // init parameters
            self.call_initialize(0, 0)?
        };
        let (_ptr, len) = unpack_fat_pointer(res_fat_ptr);
        Ok(len)
    }

    /// Try saving context data for faster initialization in the future.
    fn save_context(&mut self) -> Result<Option<Vec<u8>>, WorkerError> {
        // Call the module's `initialize` function with the save flag to save the context to the scratch buffer, then read it back.
        let (ptr, saved_len) = unpack_fat_pointer(self.call_initialize(0, 1)?);

        if saved_len == 0 {
            // Module doesn't support saving context.
            return Ok(None);
        }

        let ctx_bytes = self.read_memory(ptr, saved_len)?;
        Ok(Some(ctx_bytes))
    }

    /// Execute a work request on the given module instance.
    fn execute(&mut self, req: &WorkRequest) -> Result<WorkResponseResult, WorkerError> {
        // Encode the request to the scratch buffer and call the module's `execute` function.
        let buf = req.encode();
        let req_len = buf.len() as u32;
        let (ptr, size) = self.allocate_scratch_pad(req_len)?;
        if let Err(err) = self.write_memory(ptr, &buf) {
            log::error!("Failed to write request to scratch buffer: {err:?}");
            self.release_scratch_pad(ptr, size)?;
            return Err(err);
        }

        // Execute the module's `execute` function, which will read the request from the scratch buffer, process it and write the response back to the scratch buffer.
        let res_fat_ptr = match self.call_execute(req_len) {
            Ok(fat_ptr) => fat_ptr,
            Err(err) => {
                log::error!("Failed to call execute: {err:?}");
                self.release_scratch_pad(ptr, size)?;
                return Err(err);
            }
        };

        match unpack_fat_results(res_fat_ptr) {
            Ok((resp_ptr, resp_len)) => {
                // Read the response from the scratch buffer and decode it.
                let res_bytes = self.read_memory(resp_ptr, resp_len)?;
                Ok(Ok(WorkResponse(res_bytes)))
            }
            Err(err) => Ok(Err(err)),
        }
    }
}

pub trait BackendModule: Send + Sync {
    /// Create a new module instance from this module.
    fn instantiate(&self) -> Option<Box<dyn BackendModuleInstance>>;
}

/// A trait for backends that can be used to verify proofs.
pub trait Backend: Send + Sync {
    fn new_boxed() -> Result<Box<dyn Backend>, WorkerError>
    where
        Self: Sized;

    /// The kind of the backend.
    fn kind(&self) -> BackendKind;

    /// Maximum supported module version, used for compatibility checking.
    fn max_supported_version(&self) -> BackendModuleVersion {
        1
    }

    /// Minimum supported module version, used for compatibility checking.
    fn min_supported_version(&self) -> BackendModuleVersion {
        1
    }

    /// Check if a module is compatible.
    fn is_module_compatible(&self, module_version: BackendModuleVersion) -> bool {
        module_version >= self.min_supported_version()
            && module_version <= self.max_supported_version()
    }

    /// Check if this backend is in the given bitmask of backends.
    fn is_in_bitmask(&self, bitmask: BackendBitmask) -> bool {
        self.kind().is_supported_by(bitmask)
    }

    /// Try loading a module for the given protocol and version.
    fn load_module(&self, module_bytes: &[u8]) -> Option<Box<dyn BackendModule>>;
}
