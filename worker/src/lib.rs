use codec::Encode;

pub use polymesh_worker_common::{
    BackendBitmask, BackendCodeHash, BackendContextHash, BackendKind, BackendModuleDefinition,
    FALLBACK_TO_RUNTIME, MODULE_CODE_SIZE_LIMIT, PROTOCOL_PDART, Protocol, ProtocolError,
    ProtocolId, ProtocolNumber, ProtocolVersion, WorkFlags, WorkRequest, WorkRequestId,
    WorkResponse, WorkResponseResult, WorkSeed, WorkStatus, WorkStatusFlagsAndId, WorkerSessionId,
    WorkerVersion, config::*, error::*,
};
use polymesh_worker_common::{
    BackendModuleKind, ProtocolInitializationMethod, ProtocolModuleConfig, ProtocolModuleConfigHash,
};

pub mod backend;
pub mod cache;
pub mod worker;

/// The max supported worker version.
///
/// This version number should be incremented if the worker <-> backend module interface changes in a non-backwards-compatible way.
/// Only changes like the following (the module exported/imported function signatures):
/// - Changing how modules are initialized.
/// - Changing how scratch space is managed and accessed.
/// - Changing the execute work functinon signature.
/// - Adding new import functions (like host_msm_unchecked) that the modules can use to access host functionalities.
///
/// If the runtime requires a newer worker version, then it can fallback to executing the work in the runtime instead of the worker.
/// This allows node running older versions to continue syncing with the network (at a slower speed).
pub const WORKER_VERSION: WorkerVersion = 1;

/// Decompress the given module code bytes if they are compressed, otherwise return the original bytes.
pub fn decompress_module_code(bytes: &[u8]) -> Option<Vec<u8>> {
    // Try to decompress the bytes, if it fails, assume it's not compressed and return the original bytes.
    match sp_maybe_compressed_blob::decompress(bytes, MODULE_CODE_SIZE_LIMIT) {
        Ok(decompressed) => Some(decompressed.to_vec()),
        Err(_) => Some(bytes.to_vec()),
    }
}

pub struct StaticModules {
    initialized: bool,
    config: ProtocolModuleConfig,
    config_hash: ProtocolModuleConfigHash,
    polkavm_code_hash: BackendCodeHash,
    wasm_code_hash: BackendCodeHash,
    native_code_hash: BackendCodeHash,
}

impl StaticModules {
    pub fn new() -> Self {
        let config = ProtocolModuleConfig {
            protocol: Protocol {
                id: PROTOCOL_PDART,
                version: ProtocolVersion::new(0, 1, 0),
            },
            initialization_method: ProtocolInitializationMethod::SaveContextFromFirstInstance,
            modules: Vec::new(),
        };
        Self {
            initialized: false,
            config,
            config_hash: [0u8; 32],
            polkavm_code_hash: [0u8; 32],
            wasm_code_hash: [0u8; 32],
            native_code_hash: [42u8; 32],
        }
    }

    fn polkavm_bytes(&self) -> &'static [u8] {
        #[cfg(not(feature = "testing"))]
        {
            include_bytes!("../polymesh-worker-protocol-dart-v1.polkavm.zst")
        }
        #[cfg(feature = "testing")]
        {
            include_bytes!("../polymesh-worker-protocol-dart-v1.testing.polkavm.zst")
        }
    }

    fn wasm_bytes(&self) -> &'static [u8] {
        #[cfg(not(feature = "testing"))]
        {
            include_bytes!("../polymesh-worker-protocol-dart-v1.wasm.zst")
        }
        #[cfg(feature = "testing")]
        {
            include_bytes!("../polymesh-worker-protocol-dart-v1.testing.wasm.zst")
        }
    }

    fn initialize(&mut self) {
        if self.initialized {
            return;
        }
        // Precompute the code and context hashes for the static modules.
        self.polkavm_code_hash = blake2b256_hash(self.polkavm_bytes());
        self.wasm_code_hash = blake2b256_hash(self.wasm_bytes());

        // Setup module definitions for the protocol config.
        self.config.modules.push(BackendModuleDefinition {
            module_kind: BackendModuleKind::PolkaVM,
            module_version: 1,
            code_hash: self.polkavm_code_hash,
        });
        self.config.modules.push(BackendModuleDefinition {
            module_kind: BackendModuleKind::Wasm,
            module_version: 1,
            code_hash: self.wasm_code_hash,
        });
        self.config.modules.push(BackendModuleDefinition {
            module_kind: BackendModuleKind::Native,
            module_version: 1,
            code_hash: self.native_code_hash,
        });
        self.config_hash = blake2b256_hash(&self.config.encode());
        self.initialized = true;
    }
}

impl backend::BackendModuleLoader for StaticModules {
    fn get_protocol_module_config_hash(
        &mut self,
        protocol: Protocol,
    ) -> Option<ProtocolModuleConfigHash> {
        if protocol.id == PROTOCOL_PDART {
            self.initialize();
            Some(self.config_hash)
        } else {
            None
        }
    }

    /// Try loading the prtocol module config for the given protocol and config hash.
    fn get_protocol_module_config(
        &mut self,
        protocol: Protocol,
        config_hash: ProtocolModuleConfigHash,
    ) -> Option<ProtocolModuleConfig> {
        if protocol.id == PROTOCOL_PDART && config_hash == self.config_hash {
            self.initialize();
            Some(self.config.clone())
        } else {
            None
        }
    }

    fn get_module_code_bytes(
        &mut self,
        protocol: Protocol,
        kind: BackendModuleKind,
        code_hash: BackendCodeHash,
    ) -> Option<Vec<u8>> {
        if protocol.id == PROTOCOL_PDART {
            self.initialize();
            match kind {
                BackendModuleKind::PolkaVM if code_hash == self.polkavm_code_hash => {
                    decompress_module_code(self.polkavm_bytes())
                }
                BackendModuleKind::Wasm if code_hash == self.wasm_code_hash => {
                    decompress_module_code(self.wasm_bytes())
                }
                BackendModuleKind::Native if code_hash == self.native_code_hash => Some(vec![]),
                _ => None,
            }
        } else {
            None
        }
    }

    fn get_module_context_bytes(
        &mut self,
        _protocol: Protocol,
        _ctx_hash: BackendContextHash,
    ) -> Option<Vec<u8>> {
        None
    }
}

pub fn blake2b256_hash(data: &[u8]) -> [u8; 32] {
    use digest::{Digest, generic_array::typenum::U32};
    type Blake2b256 = blake2::Blake2b<U32>;
    Blake2b256::digest(data).into()
}
