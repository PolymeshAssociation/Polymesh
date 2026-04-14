use polymesh_worker_common::*;

use polkavm::{
    Caller, Config, Engine, Instance, InstancePre, Linker, Module, ProgramBlob, ProgramCounter,
};

use crate::backend::{Backend, BackendKind, BackendModule, BackendModuleInstance};

fn host_msm_unchecked(caller: Caller<()>, fat_ptr: u64) -> u32 {
    let (ptr, len) = ark_host_msm_impl::unpack_fat_pointer(fat_ptr);
    let mut buffer = caller
        .instance
        .read_memory(ptr, len)
        .expect("Failed to read memory from module");
    let res_len = ark_host_msm_impl::host_msm_unchecked(buffer.as_mut_slice(), len);

    caller
        .instance
        .write_memory(ptr, &buffer[..res_len as usize])
        .expect("Failed to write memory to module");
    res_len
}

/// Polkavm module instance.
pub struct PolkavmModuleInstance {
    instance: Instance,
    initialize: ProgramCounter,
    execute: ProgramCounter,
    scratch: u32,
    scratch_size: u32,
}

impl BackendModuleInstance for PolkavmModuleInstance {
    fn allocate_scratch_pad(&mut self, min_size: u32) -> Result<(u32, u32), WorkerError> {
        if self.scratch_size >= min_size {
            Ok((self.scratch, self.scratch_size))
        } else {
            Err(WorkerError::ModuleMemoryError)
        }
    }

    fn release_scratch_pad(&mut self, _ptr: u32, _size: u32) -> Result<(), WorkerError> {
        Ok(())
    }

    fn write_memory(&mut self, ptr: u32, data: &[u8]) -> Result<(), WorkerError> {
        if ptr != self.scratch || data.len() as u32 > self.scratch_size {
            return Err(WorkerError::ModuleMemoryError);
        }
        self.instance.write_memory(ptr, data).map_err(|err| {
            log::error!("Error during writing to scratch buffer: {err}");
            WorkerError::ModuleMemoryError
        })
    }

    fn read_memory_into(&mut self, ptr: u32, buffer: &mut [u8]) -> Result<(), WorkerError> {
        if ptr != self.scratch || buffer.len() as u32 > self.scratch_size {
            return Err(WorkerError::ModuleMemoryError);
        }
        self.instance.read_memory_into(ptr, buffer).map_err(|err| {
            log::error!("Error during reading from scratch buffer: {err}");
            WorkerError::ModuleMemoryError
        })?;
        Ok(())
    }

    fn call_initialize(&mut self, params_len: u32, save: u32) -> Result<u64, WorkerError> {
        self.instance
            .call_typed_and_get_result::<u64, (u32, u32)>(
                &mut (),
                self.initialize,
                (params_len, save),
            )
            .map_err(|err| {
                log::error!("Error during calling initialize: {err:?}");
                WorkerError::ModuleInitializationFailed
            })
    }

    fn call_execute(&mut self, req_len: u32) -> Result<u64, WorkerError> {
        self.instance
            .call_typed_and_get_result::<u64, (u32,)>(&mut (), self.execute, (req_len,))
            .map_err(|err| {
                log::error!("Error during calling execute: {err:?}");
                WorkerError::ModuleExecutionFailed
            })
    }
}

/// Polkavm module.
pub struct PolkavmModule {
    instance_pre: InstancePre,
    initialize: ProgramCounter,
    execute: ProgramCounter,
    get_scratch_pad: ProgramCounter,
    get_scratch_pad_size: ProgramCounter,
}

impl BackendModule for PolkavmModule {
    fn instantiate(&self) -> Option<Box<dyn BackendModuleInstance>> {
        let mut instance = self.instance_pre.instantiate().ok()?;

        // Get the scratch buffer pointer.
        let scratch = instance
            .call_typed_and_get_result::<u32, ()>(&mut (), self.get_scratch_pad, ())
            .unwrap();
        let scratch_size = instance
            .call_typed_and_get_result::<u32, ()>(&mut (), self.get_scratch_pad_size, ())
            .unwrap();

        Some(Box::new(PolkavmModuleInstance {
            instance,
            initialize: self.initialize,
            execute: self.execute,
            scratch,
            scratch_size,
        }))
    }
}

pub struct PolkavmBackend {
    engine: Engine,
}

impl PolkavmBackend {
    pub fn new() -> Self {
        let config = Config::from_env().unwrap();
        let engine = Engine::new(&config).unwrap();
        Self { engine }
    }
}

impl Backend for PolkavmBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::PolkaVM
    }

    fn load_module(&self, module_bytes: &[u8]) -> Option<Box<dyn BackendModule>> {
        let blob = ProgramBlob::parse(module_bytes.into()).ok()?;
        let module = Module::from_blob(&self.engine, &Default::default(), blob).ok()?;
        let mut linker: Linker = Linker::new();

        linker
            .define_typed("host_msm_unchecked", host_msm_unchecked)
            .expect("Failed to define host function");

        // Find the `initialize` and `execute` functions from the module exports.
        let mut initialize = None;
        let mut execute = None;
        let mut get_scratch_pad = None;
        let mut get_scratch_pad_size = None;
        for export in module.exports() {
            let name = export.symbol().as_bytes();
            if name == b"initialize" {
                initialize = Some(export.program_counter());
            }
            if name == b"execute" {
                execute = Some(export.program_counter());
            }
            if name == b"get_scratch_pad" {
                get_scratch_pad = Some(export.program_counter());
            }
            if name == b"get_scratch_pad_size" {
                get_scratch_pad_size = Some(export.program_counter());
            }
        }

        let instance_pre = linker.instantiate_pre(&module).ok()?;

        Some(Box::new(PolkavmModule {
            instance_pre,
            initialize: initialize?,
            execute: execute?,
            get_scratch_pad: get_scratch_pad?,
            get_scratch_pad_size: get_scratch_pad_size?,
        }))
    }
}
