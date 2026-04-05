use codec::{Decode, Encode};
use polymesh_worker_protocol_common::*;

mod native;
mod polkavm;
mod wasmtime;

pub struct Backends {
    pub backends: Vec<Box<dyn Backend>>,
}

impl Backends {
    pub fn new() -> Self {
        let mut backends: Vec<Box<dyn Backend>> = Vec::new();
        backends.push(Box::new(native::NativeBackend));
        backends.push(Box::new(polkavm::PolkavmBackend::new()));
        backends.push(Box::new(wasmtime::WasmtimeBackend::new()));
        Self { backends }
    }

    /// Load only the provided backends
    pub fn with_backends(kinds: Vec<BackendKind>) -> Self {
        let mut backends: Vec<Box<dyn Backend>> = Vec::new();
        for kind in kinds {
            match kind {
                BackendKind::Native => backends.push(Box::new(native::NativeBackend)),
                BackendKind::PolkaVM => backends.push(Box::new(polkavm::PolkavmBackend::new())),
                BackendKind::Wasmtime => backends.push(Box::new(wasmtime::WasmtimeBackend::new())),
                BackendKind::Wasmer => {
                    eprintln!("Wasmer backend is not yet implemented.");
                    //backends.push(Box::new(wasmer::WasmerBackend::new()));
                }
            }
        }
        Self { backends }
    }

    /// Try loading a module for the given protocol and version from the first backend that supports it.
    pub fn load_module(
        &self,
        protocol: Protocol,
        loader: &dyn BackendModuleLoader,
    ) -> Option<Box<dyn BackendModuleInstance>> {
        for backend in &self.backends {
            if let Some(module) = backend.load_module(protocol, loader) {
                return Some(module);
            }
        }
        None
    }

    /// Execute a work request on the first backend that supports it.
    pub fn execute(&self, req: &WorkRequest, loader: &dyn BackendModuleLoader) -> WorkResponse {
        let protocol = req.protocol;
        if let Some(mut module) = self.load_module(protocol, loader) {
            module.execute(req)
        } else {
            WorkResponse::Error(Error::NoBackendAvailable)
        }
    }
}

pub trait BackendModuleLoader {
    /// Try loading a module for the given protocol, version and backend kind.
    ///
    /// This allows the PolkaVM and Wasmtime backends to load modulbe blobs from Substrate's chain storage.
    fn get_module_bytes(&self, protocol: Protocol, kind: BackendKind) -> Option<Vec<u8>>;
}

/// The backend kind.
///
/// This is used to allow disabling certain backends in the future if they are found to be insecure or have other issues.
/// It also allows us to have multiple backends for the same protocol if needed.
#[derive(Clone, Copy, Debug, Encode, Decode)]
pub enum BackendKind {
    Native,
    PolkaVM,
    Wasmtime,
    Wasmer,
}

pub trait BackendModuleInstance: Send + Sync {
    /// Execute a work request on the given module instance.
    fn execute(&mut self, req: &WorkRequest) -> WorkResponse;
}

/// A trait for backends that can be used to verify proofs.
pub trait Backend: Send + Sync {
    /// The kind of the backend.
    fn kind(&self) -> BackendKind;

    /// Try loading a module for the given protocol and version.
    fn load_module(
        &self,
        protocol: Protocol,
        loader: &dyn BackendModuleLoader,
    ) -> Option<Box<dyn BackendModuleInstance>>;
}
