use codec::{Decode, Encode};

use polymesh_worker::backend::{Backend, BackendModule};
use polymesh_worker::{ProtocolError, WorkRequest, WorkerError};
use polymesh_worker_common::*;

#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[path = "child_process.rs"]
mod mode;
#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
#[path = "in_process.rs"]
mod mode;

pub use mode::run_child_if_requested;

/// Verify that the embedded native DART v1 module loads, initializes, and executes work.
pub fn self_test_dart_v1() -> Result<(), WorkerError> {
    let module = NativeBackend
        .load_module(&polymesh_worker_protocol_dart_v1::PROTOCOL.encode())
        .ok_or(WorkerError::NoBackendAvailable)?;
    let mut instance = module
        .instantiate()
        .ok_or(WorkerError::ModuleInitializationFailed)?;

    instance.initialize(None)?;
    let context = instance
        .save_context()?
        .ok_or(WorkerError::ModuleSaveContextFailed)?;
    instance.initialize(Some(&context))?;

    match instance.execute(&WorkRequest(vec![0]))? {
        Err(ProtocolError::DecodingFailed) => Ok(()),
        _ => Err(WorkerError::ModuleExecutionFailed),
    }
}

pub fn execute_native_work(
    execute: impl FnOnce() -> WorkResponseResult,
) -> Result<WorkResponseResult, WorkerError> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = sp_panic_handler::AbortGuard::force_unwind();
        execute()
    }));
    match result {
        Ok(result) => Ok(result),
        Err(_) => {
            log::warn!("NATIVE_WORKER: panic while executing a work request");
            Err(WorkerError::ModuleExecutionFailed)
        }
    }
}

#[derive(Clone, Debug)]
pub struct NativeBackend;

impl Backend for NativeBackend {
    fn new_boxed() -> Result<Box<dyn Backend>, WorkerError> {
        Ok(Box::new(Self) as _)
    }

    fn kind(&self) -> BackendKind {
        BackendKind::Native
    }

    // For the native backend we can support all module versions starting from 1.
    fn min_supported_version(&self) -> BackendModuleVersion {
        1
    }

    fn load_module(&self, module_bytes: &[u8]) -> Option<Box<dyn BackendModule>> {
        let protocol = Protocol::decode(&mut &module_bytes[..]).ok();
        match protocol {
            Some(protocol) => {
                // Native modules don't have module code bytes, so we just encode the protocol.
                if protocol == polymesh_worker_protocol_dart_v1::PROTOCOL {
                    mode::dart_module()
                } else if protocol == polymesh_worker_protocol_testing::PROTOCOL {
                    mode::testing_module()
                } else {
                    None
                }
            }
            None => {
                // Fallback to the native DART module.
                mode::dart_module()
            }
        }
    }
}
