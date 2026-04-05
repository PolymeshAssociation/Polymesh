use codec::{Decode, Encode};
use polymesh_worker_protocol_common::{Error, Protocol, WorkRequest, WorkResponse};

use wasmtime::*;

use crate::backend::{Backend, BackendKind, BackendModuleInstance, BackendModuleLoader};

/// Wasmtime module instance.
pub struct WasmtimeModuleInstance {
    instance: Instance,
    execute: TypedFunc<u32, u32>,
    store: Store<()>,
    scratch: u32,
    scratch_size: u32,
}

impl BackendModuleInstance for WasmtimeModuleInstance {
    fn execute(&mut self, req: &WorkRequest) -> WorkResponse {
        // Encode the request to the scratch buffer and call the module's `execute` function.
        let buf = req.encode();
        let req_len = buf.len() as u32;
        if let Some(mem) = self.instance.get_memory(&mut self.store, "memory") {
            if let Err(err) = mem.write(&mut self.store, self.scratch as usize, &buf) {
                eprintln!("Failed to write to module memory: {err}");
                return WorkResponse::Error(Error::ModuleMemoryError);
            }
        } else {
            return WorkResponse::Error(Error::ModuleMemoryError);
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
        if let Some(mem) = self.instance.get_memory(&mut self.store, "memory") {
            let mut res_bytes = vec![0u8; res_len as usize];
            if let Err(err) = mem.read(&mut self.store, self.scratch as usize, &mut res_bytes) {
                eprintln!("Failed to read from module memory: {err}");
                return WorkResponse::Error(Error::ModuleMemoryError);
            }
            Decode::decode(&mut &res_bytes[..])
                .unwrap_or(WorkResponse::Error(Error::DecodingFailed))
        } else {
            WorkResponse::Error(Error::ModuleMemoryError)
        }
    }
}

pub struct WasmtimeBackend {
    engine: Engine,
}

impl WasmtimeBackend {
    pub fn new() -> Self {
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
        let engine = Engine::new(&config).expect("Failed to create Wasmtime engine");
        Self { engine }
    }
}

impl Backend for WasmtimeBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Wasmtime
    }

    fn load_module(
        &self,
        protocol: Protocol,
        loader: &dyn BackendModuleLoader,
    ) -> Option<Box<dyn BackendModuleInstance>> {
        println!("Initializing...");
        let now = std::time::Instant::now();
        let mut store = Store::new(&self.engine, ());

        // Load the module bytes and compile the module.
        let module_bytes = loader.get_module_bytes(protocol, self.kind())?;
        let module = Module::from_binary(&self.engine, &module_bytes)
            .expect("Failed to create module from binary");
        println!(
            "Module loaded and compiled, time taken: {:?}",
            now.elapsed()
        );

        // Once we've got that all set up we can then move to the instantiation
        // phase, pairing together a compiled module as well as a set of imports.
        // Note that this is where the wasm `start` function, if any, would run.
        println!("Instantiating module...");
        let imports = [];
        let instance =
            Instance::new(&mut store, &module, &imports).expect("Failed to instantiate module");

        // Get the scratch buffer pointer.
        let now = std::time::Instant::now();
        let get_scratch_pad = instance
            .get_typed_func::<(), u32>(&mut store, "get_scratch_pad")
            .ok()?;
        let scratch = get_scratch_pad.call(&mut store, ()).ok()?;
        let get_scratch_pad_size = instance
            .get_typed_func::<(), u32>(&mut store, "get_scratch_pad_size")
            .ok()?;
        let scratch_size = get_scratch_pad_size.call(&mut store, ()).ok()?;
        println!("Scratch pad pointer: {scratch}, size: {scratch_size}");
        println!("Time taken for scratch pad setup: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        println!("Get initialize function...");
        let initialize = instance
            .get_typed_func::<(u32, u32), u32>(&mut store, "initialize")
            .ok()?;
        println!("Get execute function...");
        let execute = instance
            .get_typed_func::<u32, u32>(&mut store, "execute")
            .ok()?;
        println!(
            "Time taken for getting initialize/execute functions: {:?}",
            now.elapsed()
        );

        // init parameters
        let now = std::time::Instant::now();
        let params_len = initialize.call(&mut store, (0, 0)).ok()?;
        println!("initialize result: params_len={params_len}");
        println!("Time taken for initialize: {:?}", now.elapsed());

        // Save parameters back to the scratch buffer.
        let now = std::time::Instant::now();
        let save_result = initialize.call(&mut store, (0, 1)).ok()?;
        println!("Save parameters result: {save_result}");
        println!("Time taken for saving parameters: {:?}", now.elapsed());

        // Time loading the parameters back into the module.
        let now = std::time::Instant::now();
        let load_result = initialize.call(&mut store, (params_len, 0)).ok()?;
        println!("Load parameters result: {load_result}");
        println!("Time taken for loading parameters: {:?}", now.elapsed());

        Some(Box::new(WasmtimeModuleInstance {
            instance,
            execute,
            store,
            scratch,
            scratch_size,
        }))
    }
}
