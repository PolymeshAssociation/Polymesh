use polymesh_worker_common::WorkerError;

use wasmer::{
    sys::{EngineBuilder, Features},
    *,
};
#[cfg(not(feature = "llvm"))]
use wasmer_compiler_cranelift::Cranelift;
#[cfg(feature = "llvm")]
use wasmer_compiler_llvm::LLVM;

use crate::backend::{Backend, BackendKind, BackendModule, BackendModuleInstance};

#[derive(Default)]
struct FnEnv {
    memory: Option<Memory>,
}

fn host_msm_unchecked(mut env: FunctionEnvMut<FnEnv>, fat_ptr: u64) -> u32 {
    let (ptr, len) = ark_host_msm_impl::unpack_fat_pointer(fat_ptr);
    let (env, store) = env.data_and_store_mut();
    let Some(memory) = env.memory.as_ref().map(|m| m.view(&store)) else {
        log::error!("Memory not found in host environment");
        return 0;
    };
    let mut buffer = vec![0u8; len as usize];
    if let Err(err) = memory.read(ptr as u64, &mut buffer) {
        log::error!("Failed to read from module memory: {err}");
        return 0;
    }
    let res_len = ark_host_msm_impl::host_msm_unchecked(&mut buffer, len);

    if let Err(err) = memory.write(ptr as u64, &buffer[..res_len as usize]) {
        log::error!("Failed to write to module memory: {err}");
        return 0;
    }

    res_len
}

/// Wasmer module instance.
pub struct WasmerModuleInstance {
    memory: Memory,
    initialize: TypedFunction<(u32, u32), u64>,
    execute: TypedFunction<u32, u64>,
    store: Store,
    scratch: u32,
    scratch_size: u32,
}

impl BackendModuleInstance for WasmerModuleInstance {
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
        let memory_view = self.memory.view(&self.store);
        memory_view.write(ptr as u64, data).map_err(|err| {
            log::error!("Error during writing to scratch buffer: {err}");
            WorkerError::ModuleMemoryError
        })
    }

    fn read_memory_into(&mut self, ptr: u32, buffer: &mut [u8]) -> Result<(), WorkerError> {
        if ptr != self.scratch || buffer.len() as u32 > self.scratch_size {
            return Err(WorkerError::ModuleMemoryError);
        }
        let memory_view = self.memory.view(&self.store);
        memory_view.read(ptr as u64, buffer).map_err(|err| {
            log::error!("Error during reading from scratch buffer: {err}");
            WorkerError::ModuleMemoryError
        })?;
        Ok(())
    }

    fn call_initialize(&mut self, params_len: u32, save: u32) -> Result<u64, WorkerError> {
        self.initialize
            .call(&mut self.store, params_len, save)
            .map_err(|err| {
                log::error!("Error during calling initialize: {err:?}");
                WorkerError::ModuleInitializationFailed
            })
    }

    fn call_execute(&mut self, req_len: u32) -> Result<u64, WorkerError> {
        self.execute.call(&mut self.store, req_len).map_err(|err| {
            log::error!("Error during calling execute: {err:?}");
            WorkerError::ModuleExecutionFailed
        })
    }
}

pub struct WasmerModule {
    module: Module,
    engine: Engine,
}

impl BackendModule for WasmerModule {
    fn instantiate(&self) -> Option<Box<dyn BackendModuleInstance>> {
        // Once we've got that all set up we can then move to the instantiation
        // phase, pairing together a compiled module as well as a set of imports.
        // Note that this is where the wasm `start` function, if any, would run.
        let mut store = Store::new(self.engine.clone());
        let env = FunctionEnv::new(&mut store, FnEnv::default());
        let imports = imports! {
            "env" => {
                "host_msm_unchecked" => Function::new_typed_with_env(&mut store, &env, host_msm_unchecked),
            },
        };
        let instance = Instance::new(&mut store, &self.module, &imports).ok()?;

        // Get the instance memory for the host functions to access.
        let memory = instance.exports.get_memory("memory").ok()?.clone();
        env.as_mut(&mut store).memory = Some(memory.clone());

        // Get the scratch buffer pointer.
        let get_scratch_pad = instance
            .exports
            .get_typed_function::<(), u32>(&store, "get_scratch_pad")
            .ok()?;
        let scratch = get_scratch_pad.call(&mut store).ok()?;
        let get_scratch_pad_size = instance
            .exports
            .get_typed_function::<(), u32>(&mut store, "get_scratch_pad_size")
            .ok()?;
        let scratch_size = get_scratch_pad_size.call(&mut store).ok()?;

        let initialize = instance
            .exports
            .get_typed_function::<(u32, u32), u64>(&mut store, "initialize")
            .ok()?;
        let execute = instance
            .exports
            .get_typed_function::<u32, u64>(&mut store, "execute")
            .ok()?;

        Some(Box::new(WasmerModuleInstance {
            memory,
            initialize,
            execute,
            store,
            scratch,
            scratch_size,
        }))
    }
}

pub struct WasmerBackend {
    engine: Engine,
}

impl WasmerBackend {
    pub fn new() -> Result<Self, WorkerError> {
        // Setup the Wasm features to support.
        let mut features = Features::default();
        features.bulk_memory = true;
        features.multi_value = true;
        features.simd = true;
        features.wide_arithmetic = true;

        #[cfg(feature = "llvm")]
        let compiler = LLVM::default();
        #[cfg(not(feature = "llvm"))]
        let compiler = Cranelift::default();

        let engine = EngineBuilder::new(compiler)
            .set_features(Some(features))
            .engine();

        Ok(Self {
            engine: engine.into(),
        })
    }
}

impl Backend for WasmerBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Wasmer
    }

    fn load_module(&self, module_bytes: &[u8]) -> Option<Box<dyn BackendModule>> {
        let module = Module::from_binary(&self.engine, &module_bytes).ok()?;

        Some(Box::new(WasmerModule {
            module,
            engine: self.engine.clone(),
        }))
    }
}
