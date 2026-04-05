use codec::{Decode, Encode};
use polymesh_worker_protocol_common::{Error, Protocol, WorkRequest, WorkResponse};

use wasmer::*;

use crate::backend::{Backend, BackendKind, BackendModuleInstance, BackendModuleLoader};

/// Wasmer module instance.
pub struct WasmerModuleInstance {
    instance: Instance,
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
        let memory = self
            .instance
            .exports
            .get_memory("memory")
            .expect("Failed to get memory export");
        {
            let memory_view = memory.view(&self.store);
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
            let memory_view = memory.view(&self.store);
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
        let mut store = Store::default();

        let module_bytes = loader.get_module_bytes(protocol, self.kind())?;
        let module = Module::from_binary(&store, &module_bytes).ok()?;

        // Once we've got that all set up we can then move to the instantiation
        // phase, pairing together a compiled module as well as a set of imports.
        // Note that this is where the wasm `start` function, if any, would run.
        println!("Instantiating module...");
        let imports = Imports::new();
        let instance = Instance::new(&mut store, &module, &imports).ok()?;

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
            execute,
            store,
            scratch,
            scratch_size,
        }))
    }
}
