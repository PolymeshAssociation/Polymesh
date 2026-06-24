use polymesh_worker::backend::{BackendModule, BackendModuleInstance};
use polymesh_worker_common::*;

use polymesh_worker_protocol_testing::*;

#[derive(Clone, Debug)]
pub struct NativeTestingModuleInstance;

impl BackendModuleInstance for NativeTestingModuleInstance {
    fn allocate_scratch_pad(&mut self, _min_size: u32) -> Result<(u32, u32), WorkerError> {
        Err(WorkerError::ModuleMemoryError)
    }

    fn release_scratch_pad(&mut self, _ptr: u32, _size: u32) -> Result<(), WorkerError> {
        Ok(())
    }

    fn write_memory(&mut self, _ptr: u32, _data: &[u8]) -> Result<(), WorkerError> {
        Err(WorkerError::ModuleMemoryError)
    }

    fn read_memory_into(&mut self, _ptr: u32, _buffer: &mut [u8]) -> Result<(), WorkerError> {
        Err(WorkerError::ModuleMemoryError)
    }

    fn read_memory(&mut self, _ptr: u32, _len: u32) -> Result<Vec<u8>, WorkerError> {
        Err(WorkerError::ModuleMemoryError)
    }

    fn call_initialize(&mut self, _params_len: u32, _save: u32) -> Result<u64, WorkerError> {
        Ok(0)
    }

    fn call_execute(&mut self, _req_len: u32) -> Result<u64, WorkerError> {
        Ok(0)
    }

    fn initialize(&mut self, load_ctx: Option<&[u8]>) -> Result<u32, WorkerError> {
        Ok(initialize(load_ctx).map_err(|_| WorkerError::ModuleInitializationFailed)? as u32)
    }

    fn save_context(&mut self) -> Result<Option<Vec<u8>>, WorkerError> {
        Ok(Some(
            save_context().map_err(|_| WorkerError::ModuleSaveContextFailed)?,
        ))
    }

    fn execute(&mut self, req: &WorkRequest) -> Result<WorkResponseResult, WorkerError> {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = sp_panic_handler::AbortGuard::force_unwind();
            execute_work_request(req)
        }));
        match result {
            Ok(result) => Ok(result),
            Err(panic) => {
                let message = if let Some(message) = panic.downcast_ref::<String>() {
                    format!(
                        "native worker panicked while executing a work request: {}",
                        message
                    )
                } else if let Some(message) = panic.downcast_ref::<&'static str>() {
                    format!(
                        "native worker panicked while executing a work request: {}",
                        message
                    )
                } else {
                    "native worker panicked while executing a work request".to_owned()
                };
                log::warn!("NATIVE_WORKER: {}", message);
                // The work request execution panicked, reject the work request (for proof validation this means the proof is rejected).
                Ok(Err(ProtocolError::ExecuteWorkFailed))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct NativeTestingModule;

impl BackendModule for NativeTestingModule {
    fn instantiate(&self) -> Option<Box<dyn BackendModuleInstance>> {
        Some(Box::new(NativeTestingModuleInstance))
    }
}
