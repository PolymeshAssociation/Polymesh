use polymesh_worker_common::WorkerError;

use wasmtime::*;

use crate::backend::{Backend, BackendKind, BackendModule, BackendModuleInstance};

fn host_msm_unchecked(mut caller: Caller<'_, ()>, fat_ptr: u64) -> wasmtime::Result<u32> {
    let (ptr, len) = ark_host_msm_impl::unpack_fat_pointer(fat_ptr);
    let mem = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => bail!("failed to find host memory"),
    };
    let ptr = ptr as usize;
    let len = len as usize;
    let buffer = if let Some(buffer) = mem.data_mut(&mut caller).get_mut(ptr..ptr + len) {
        buffer
    } else {
        bail!("pointer/length out of bounds");
    };

    let res = ark_host_msm_impl::host_msm_unchecked(buffer, len as u32);

    Ok(res)
}

/// Wasmtime module instance.
pub struct WasmtimeModuleInstance {
    initialize: TypedFunc<(u32, u32), u64>,
    execute: TypedFunc<u32, u64>,
    memory: Memory,
    store: Store<()>,
    scratch: u32,
    scratch_size: u32,
}

impl BackendModuleInstance for WasmtimeModuleInstance {
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
        self.memory
            .write(&mut self.store, ptr as usize, data)
            .map_err(|err| {
                log::error!("Error during writing to scratch buffer: {err}");
                WorkerError::ModuleMemoryError
            })
    }

    fn read_memory_into(&mut self, ptr: u32, buffer: &mut [u8]) -> Result<(), WorkerError> {
        if ptr != self.scratch || buffer.len() as u32 > self.scratch_size {
            return Err(WorkerError::ModuleMemoryError);
        }
        self.memory
            .read(&mut self.store, ptr as usize, buffer)
            .map_err(|err| {
                log::error!("Error during reading from scratch buffer: {err}");
                WorkerError::ModuleMemoryError
            })?;
        Ok(())
    }

    fn call_initialize(&mut self, params_len: u32, save: u32) -> Result<u64, WorkerError> {
        self.initialize
            .call(&mut self.store, (params_len, save))
            .map_err(|err| {
                log::error!("Error during calling initialize: {err}");
                WorkerError::ModuleInitializationFailed
            })
    }

    fn call_execute(&mut self, req_len: u32) -> Result<u64, WorkerError> {
        self.execute.call(&mut self.store, req_len).map_err(|err| {
            log::error!("Error during calling execute: {err}");
            WorkerError::ModuleExecutionFailed
        })
    }
}

pub struct WasmtimeModule {
    module: Module,
    engine: Engine,
}

impl BackendModule for WasmtimeModule {
    fn instantiate(&self) -> Option<Box<dyn BackendModuleInstance>> {
        let mut store = Store::new(&self.engine, ());
        let host_msm = Func::wrap(&mut store, host_msm_unchecked);
        let imports = [host_msm.into()];

        // Once we've got that all set up we can then move to the instantiation
        // phase, pairing together a compiled module as well as a set of imports.
        // Note that this is where the wasm `start` function, if any, would run.
        let instance = Instance::new(&mut store, &self.module, &imports).ok()?;

        // Get the scratch buffer pointer.
        let get_scratch_pad = instance
            .get_typed_func::<(), u32>(&mut store, "get_scratch_pad")
            .ok()?;
        let scratch = get_scratch_pad.call(&mut store, ()).ok()?;
        let get_scratch_pad_size = instance
            .get_typed_func::<(), u32>(&mut store, "get_scratch_pad_size")
            .ok()?;
        let scratch_size = get_scratch_pad_size.call(&mut store, ()).ok()?;

        let initialize = instance
            .get_typed_func::<(u32, u32), u64>(&mut store, "initialize")
            .ok()?;
        let execute = instance
            .get_typed_func::<u32, u64>(&mut store, "execute")
            .ok()?;

        // Get the instance memory for the host functions to access.
        let memory = instance.get_memory(&mut store, "memory")?;

        Some(Box::new(WasmtimeModuleInstance {
            initialize,
            execute,
            memory,
            store,
            scratch,
            scratch_size,
        }))
    }
}

pub struct WasmtimeBackend {
    engine: Engine,
}

impl WasmtimeBackend {
    pub fn new() -> Result<Self, WorkerError> {
        let mut config = Config::new();

        // Enable the Cranelift optimizing compiler.
        config.strategy(Strategy::Cranelift);

        // Enable Webassembly features.
        config.wasm_wide_arithmetic(true);
        config.wasm_bulk_memory(true);
        config.wasm_multi_value(true);
        config.wasm_simd(true);

        // Enable signals-based traps. This is required to elide explicit
        // bounds-checking.
        config.signals_based_traps(true);

        // Configure linear memories such that explicit bounds-checking can be
        // elided.
        config.memory_reservation(1 << 32);
        config.memory_guard_size(1 << 32);
        let engine = Engine::new(&config).map_err(|err| {
            log::error!("Failed to create Wasmtime engine: {err}");
            WorkerError::BackendNotSupported
        })?;
        Ok(Self { engine })
    }
}

impl Backend for WasmtimeBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Wasmtime
    }

    fn load_module(&self, module_bytes: &[u8]) -> Option<Box<dyn BackendModule>> {
        let module = Module::from_binary(&self.engine, &module_bytes).ok()?;

        Some(Box::new(WasmtimeModule {
            module,
            engine: self.engine.clone(),
        }))
    }
}
