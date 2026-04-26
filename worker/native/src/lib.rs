use polymesh_worker::backend::{Backend, BackendModule, BackendModuleInstance};
use polymesh_worker_common::*;

use polymesh_worker_protocol_dart_v1::*;

#[derive(Clone, Debug)]
pub struct NativeModuleInstance;

impl BackendModuleInstance for NativeModuleInstance {
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
        Ok(execute_work_request(req))
    }
}

#[derive(Clone, Debug)]
pub struct NativeModule;

impl BackendModule for NativeModule {
    fn instantiate(&self) -> Option<Box<dyn BackendModuleInstance>> {
        Some(Box::new(NativeModuleInstance))
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

    fn load_module(&self, _module_bytes: &[u8]) -> Option<Box<dyn BackendModule>> {
        Some(Box::new(NativeModule))
    }
}
