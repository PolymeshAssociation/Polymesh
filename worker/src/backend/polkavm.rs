use codec::{Decode, Encode};
use polymesh_worker_protocol_common::*;

use polkavm::{Caller, Config, Engine, Instance, Linker, Module, ProgramBlob};

use crate::backend::{Backend, BackendKind, BackendModuleInstance, BackendModuleLoader};

fn host_msm_unchecked(caller: Caller<()>, is_pallas: u32, ptr: u32, len: u32) -> u32 {
    let mut buffer = caller
        .instance
        .read_memory(ptr, len)
        .expect("Failed to read memory from module");
    let res_len = crate::host::host_msm_unchecked(is_pallas, buffer.as_mut_slice(), len);

    caller
        .instance
        .write_memory(ptr, &buffer[..res_len as usize])
        .expect("Failed to write memory to module");
    res_len
}

/// Polkavm module instance.
pub struct PolkavmModuleInstance {
    instance: Instance,
    scratch: u32,
    scratch_size: u32,
}

impl BackendModuleInstance for PolkavmModuleInstance {
    fn execute(&mut self, req: &WorkRequest) -> WorkResponse {
        // Encode the request to the scratch buffer and call the module's `execute` function.
        let buf = req.encode();
        let req_len = buf.len() as u32;
        self.instance
            .write_memory(self.scratch, buf.as_slice())
            .unwrap();

        // Execute the module's `execute` function, which will read the request from the scratch buffer, process it and write the response back to the scratch buffer.
        let res_len = self
            .instance
            .call_typed_and_get_result::<u32, (u32,)>(&mut (), "execute", (req_len,))
            .unwrap();

        // Read the response from the scratch buffer and decode it.
        let res_bytes = self.instance.read_memory(self.scratch, res_len).unwrap();
        Decode::decode(&mut &res_bytes[..]).unwrap_or(WorkResponse::Error(Error::DecodingFailed))
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

    fn load_module(
        &self,
        protocol: Protocol,
        loader: &dyn BackendModuleLoader,
    ) -> Option<Box<dyn BackendModuleInstance>> {
        let module_bytes = loader.get_module_bytes(protocol, self.kind())?;
        let blob = ProgramBlob::parse(module_bytes.into()).ok()?;
        let module = Module::from_blob(&self.engine, &Default::default(), blob).ok()?;
        let mut linker: Linker = Linker::new();

        linker
            .define_typed("host_msm_unchecked", host_msm_unchecked)
            .expect("Failed to define host function");

        let instance_pre = linker.instantiate_pre(&module).ok()?;
        let mut instance = instance_pre.instantiate().ok()?;

        // Get the scratch buffer pointer.
        let now = std::time::Instant::now();
        let scratch = instance
            .call_typed_and_get_result::<u32, ()>(&mut (), "get_scratch_pad", ())
            .unwrap();
        let scratch_size = instance
            .call_typed_and_get_result::<u32, ()>(&mut (), "get_scratch_pad_size", ())
            .unwrap();
        println!("Scratch pad pointer: {scratch}, size: {scratch_size}");
        println!("Time taken for scratch pad setup: {:?}", now.elapsed());

        // init parameters
        let now = std::time::Instant::now();
        let params_len = instance
            .call_typed_and_get_result::<u32, (u32, u32)>(&mut (), "initialize", (0, 0))
            .unwrap();
        println!("initialize result: params_len={params_len}");
        println!("Time taken for initialize: {:?}", now.elapsed());

        // Save parameters back to the scratch buffer.
        let now = std::time::Instant::now();
        let save_result = instance
            .call_typed_and_get_result::<u32, (u32, u32)>(&mut (), "initialize", (0, 1))
            .unwrap();
        println!("Save parameters result: {save_result}");
        println!("Time taken for saving parameters: {:?}", now.elapsed());

        // Time loading the parameters back into the module.
        let now = std::time::Instant::now();
        let load_result = instance
            .call_typed_and_get_result::<u32, (u32, u32)>(&mut (), "initialize", (params_len, 0))
            .unwrap();
        println!("Load parameters result: {load_result}");
        println!("Time taken for loading parameters: {:?}", now.elapsed());

        Some(Box::new(PolkavmModuleInstance {
            instance,
            scratch,
            scratch_size,
        }))
    }
}
