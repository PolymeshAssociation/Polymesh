use codec::Decode;

use polymesh_worker::backend::{Backend, BackendModule};
use polymesh_worker_common::*;

#[cfg(not(target_os = "linux"))]
mod dart;
#[cfg(not(target_os = "linux"))]
mod testing;

#[cfg(not(target_os = "linux"))]
use dart::NativeDartModule;
#[cfg(not(target_os = "linux"))]
use testing::NativeTestingModule;

#[cfg(target_os = "linux")]
pub use polymesh_worker::native_process::run_child_if_requested;

#[cfg(not(target_os = "linux"))]
pub fn run_child_if_requested() {}

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

#[cfg(target_os = "linux")]
fn testing_module() -> Option<Box<dyn BackendModule>> {
    process_module(include_bytes!(
        "../../modules/testing/v0/polymesh-worker-protocol-testing.native.so.zst"
    ))
}

#[cfg(target_os = "linux")]
fn dart_module() -> Option<Box<dyn BackendModule>> {
    #[cfg(feature = "testing")]
    let library = include_bytes!(
        "../../modules/dart/v1/polymesh-worker-protocol-dart-v1.testing.native.so.zst"
    );
    #[cfg(not(feature = "testing"))]
    let library =
        include_bytes!("../../modules/dart/v1/polymesh-worker-protocol-dart-v1.native.so.zst");
    process_module(library)
}

#[cfg(target_os = "linux")]
fn process_module(compressed: &[u8]) -> Option<Box<dyn BackendModule>> {
    let library = polymesh_worker::decompress_module_code(compressed)?;
    Some(Box::new(
        polymesh_worker::native_process::NativeProcessModule::new(library),
    ))
}

#[cfg(not(target_os = "linux"))]
fn testing_module() -> Option<Box<dyn BackendModule>> {
    Some(Box::new(NativeTestingModule))
}

#[cfg(not(target_os = "linux"))]
fn dart_module() -> Option<Box<dyn BackendModule>> {
    Some(Box::new(NativeDartModule))
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
                    dart_module()
                } else if protocol == polymesh_worker_protocol_testing::PROTOCOL {
                    testing_module()
                } else {
                    None
                }
            }
            None => {
                // Fallback to the native DART module.
                dart_module()
            }
        }
    }
}
