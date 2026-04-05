pub use polymesh_worker_protocol_common::*;

mod backend;

pub use backend::{Backend, BackendKind, BackendModuleInstance, BackendModuleLoader, Backends};

pub struct StaticModules;

impl BackendModuleLoader for StaticModules {
    fn get_module_bytes(&self, protocol: Protocol, kind: BackendKind) -> Option<Vec<u8>> {
        // TODO: support loading from Substrate on-chain storage.
        match (protocol.id, kind) {
            (PROTOCOL_PDART, BackendKind::PolkaVM) => {
                Some(include_bytes!("../polymesh-worker-protocol-dart-v0.polkavm").to_vec())
            }
            (PROTOCOL_PDART, BackendKind::Wasmtime | BackendKind::Wasmer) => {
                Some(include_bytes!("../polymesh-worker-protocol-dart-v0.wasm").to_vec())
            }
            _ => None,
        }
    }
}
