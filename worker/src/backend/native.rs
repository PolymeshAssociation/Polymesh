use polymesh_worker_protocol_common::*;
use polymesh_worker_protocol_dart_v0::*;

use crate::backend::{Backend, BackendKind, BackendModuleInstance, BackendModuleLoader};

#[derive(Clone, Debug)]
pub struct NativeModuleInstance;

impl BackendModuleInstance for NativeModuleInstance {
    fn execute(&mut self, req: &WorkRequest) -> WorkResponse {
        execute_work_request(req)
    }
}

#[derive(Clone, Debug)]
pub struct NativeBackend;

impl Backend for NativeBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Native
    }

    fn load_module(
        &self,
        protocol: Protocol,
        _loader: &dyn BackendModuleLoader,
    ) -> Option<Box<dyn BackendModuleInstance>> {
        if protocol.id != PROTOCOL_PDART {
            return None;
        }
        // TODO: version check.
        let now = std::time::Instant::now();
        if initialize().is_err() {
            return None;
        }
        println!("Native backend initialization time: {:?}", now.elapsed());
        Some(Box::new(NativeModuleInstance))
    }
}
