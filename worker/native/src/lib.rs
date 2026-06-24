use codec::Decode;

use polymesh_worker::backend::{Backend, BackendModule};
use polymesh_worker_common::*;

mod dart;
mod testing;

use dart::NativeDartModule;
use testing::NativeTestingModule;

#[derive(Clone, Debug)]
pub struct NativeBackend;

impl Backend for NativeBackend {
    fn new_boxed() -> Result<Box<dyn Backend>, WorkerError> {
        Ok(Box::new(Self) as _)
    }

    fn kind(&self) -> BackendKind {
        BackendKind::Native
    }

    fn load_module(&self, module_bytes: &[u8]) -> Option<Box<dyn BackendModule>> {
        let protocol = Protocol::decode(&mut &module_bytes[..]).ok();
        match protocol {
            Some(protocol) => {
                // Native modules don't have module code bytes, so we just encode the protocol.
                if protocol == polymesh_worker_protocol_dart_v1::PROTOCOL {
                    Some(Box::new(NativeDartModule))
                } else if protocol == polymesh_worker_protocol_testing::PROTOCOL {
                    Some(Box::new(NativeTestingModule))
                } else {
                    None
                }
            }
            None => {
                // Fallback to the native DART module.
                Some(Box::new(NativeDartModule))
            }
        }
    }
}
