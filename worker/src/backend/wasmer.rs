use codec::{Decode, Encode};
use polymesh_worker_protocol_common::{Error, Protocol, WorkRequest, WorkResponse};

use wasmer::{
    sys::{EngineBuilder, Features},
    *,
};
#[cfg(not(feature = "llvm"))]
use wasmer_compiler_cranelift::Cranelift;
#[cfg(feature = "llvm")]
use wasmer_compiler_llvm::LLVM;

use crate::backend::{Backend, BackendKind, BackendModuleInstance, BackendModuleLoader};

#[derive(Default)]
struct FnEnv {
    memory: Option<Memory>,
}

fn host_msm_unchecked(mut env: FunctionEnvMut<FnEnv>, fat_ptr: u64) -> u32 {
    let (ptr, len) = ark_host_msm_impl::unpack_fat_pointer(fat_ptr);
    let (env, store) = env.data_and_store_mut();
    let Some(memory) = env.memory.as_ref().map(|m| m.view(&store)) else {
        eprintln!("Memory not found in host environment");
        return 0;
    };
    let mut buffer = vec![0u8; len as usize];
    if let Err(err) = memory.read(ptr as u64, &mut buffer) {
        eprintln!("Failed to read from module memory: {err}");
        return 0;
    }
    let res_len = ark_host_msm_impl::host_msm_unchecked(&mut buffer, len);

    if let Err(err) = memory.write(ptr as u64, &buffer[..res_len as usize]) {
        eprintln!("Failed to write to module memory: {err}");
        return 0;
    }

    res_len
}

/// Wasmer module instance.
pub struct WasmerModuleInstance {
    instance: Instance,
    memory: Memory,
    execute: TypedFunction<u32, u32>,
    store: Store,
    scratch: u32,
    scratch_size: u32,
}

impl BackendModuleInstance for WasmerModuleInstance {
    fn execute(&mut self, req: &WorkRequest) -> WorkResponse {
        // Encode the request to the scratch buffer and call the module's `execute` function.
        let buf = req.encode();
        let req_len = buf.len() as u32;
        {
            let memory_view = self.memory.view(&self.store);
            memory_view
                .write(self.scratch as u64, &buf)
                .expect("Failed to write to memory");
        }
        // Execute the module's `execute` function, which will read the request from the scratch buffer, process it and write the response back to the scratch buffer.
        let res_len = match self.execute.call(&mut self.store, req_len) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Failed to call execute function: {err}");
                return WorkResponse::Error(Error::InvalidModule);
            }
        };

        // Read the response from the scratch buffer and decode it.
        let mut res_bytes = vec![0u8; res_len as usize];
        {
            let memory_view = self.memory.view(&self.store);
            memory_view
                .read(self.scratch as u64, &mut res_bytes)
                .expect("Failed to read from memory");
        }
        Decode::decode(&mut &res_bytes[..]).unwrap_or(WorkResponse::Error(Error::DecodingFailed))
    }
}

pub struct WasmerBackend {}

impl WasmerBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl Backend for WasmerBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Wasmer
    }

    fn load_module(
        &self,
        protocol: Protocol,
        loader: &dyn BackendModuleLoader,
    ) -> Option<Box<dyn BackendModuleInstance>> {
        println!("Initializing...");
        let now = std::time::Instant::now();

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
        let mut store = Store::new(engine);

        println!("Loading module...");
        let module_bytes = loader.get_module_bytes(protocol, self.kind())?;
        println!("Module loaded, time taken: {:?}", now.elapsed());
        let module = Module::from_binary(&store, &module_bytes)
            .expect("Failed to create module from binary");

        println!("Module loaded, time taken: {:?}", now.elapsed());

        // Once we've got that all set up we can then move to the instantiation
        // phase, pairing together a compiled module as well as a set of imports.
        // Note that this is where the wasm `start` function, if any, would run.
        println!("Instantiating module...");
        let now = std::time::Instant::now();
        let env = FunctionEnv::new(&mut store, FnEnv::default());
        let imports = imports! {
            "env" => {
                "host_msm_unchecked" => Function::new_typed_with_env(&mut store, &env, host_msm_unchecked),
            },
        };
        let instance =
            Instance::new(&mut store, &module, &imports).expect("Failed to instantiate module"); //.ok()?;
        println!("Module instantiated, time taken: {:?}", now.elapsed());

        // Get the instance memory for the host functions to access.
        let memory = instance
            .exports
            .get_memory("memory")
            .expect("Failed to get memory export")
            .clone();
        env.as_mut(&mut store).memory = Some(memory.clone());

        // Get the scratch buffer pointer.
        let now = std::time::Instant::now();
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
        println!("Scratch pad pointer: {scratch}, size: {scratch_size}");
        println!("Time taken for scratch pad setup: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        println!("Get initialize function...");
        let initialize = instance
            .exports
            .get_typed_function::<(u32, u32), u32>(&mut store, "initialize")
            .ok()?;
        println!("Get execute function...");
        let execute = instance
            .exports
            .get_typed_function::<u32, u32>(&mut store, "execute")
            .ok()?;
        println!(
            "Time taken for getting initialize/execute functions: {:?}",
            now.elapsed()
        );

        // init parameters
        let now = std::time::Instant::now();
        let params_len = initialize.call(&mut store, 0, 0).ok()?;
        println!("initialize result: params_len={params_len}");
        println!("Time taken for initialize: {:?}", now.elapsed());

        // Save parameters back to the scratch buffer.
        let now = std::time::Instant::now();
        let save_result = initialize.call(&mut store, 0, 1).ok()?;
        println!("Save parameters result: {save_result}");
        println!("Time taken for saving parameters: {:?}", now.elapsed());

        // Time loading the parameters back into the module.
        let now = std::time::Instant::now();
        let load_result = initialize.call(&mut store, params_len, 0).ok()?;
        println!("Load parameters result: {load_result}");
        println!("Time taken for loading parameters: {:?}", now.elapsed());

        Some(Box::new(WasmerModuleInstance {
            instance,
            memory,
            execute,
            store,
            scratch,
            scratch_size,
        }))
    }
}
