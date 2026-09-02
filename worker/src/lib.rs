use std::collections::BTreeMap;

use codec::Encode;

pub use polymesh_worker_common::{
    BackendBitmask, BackendCodeHash, BackendContextHash, BackendKind, BackendModuleDefinition,
    FALLBACK_TO_RUNTIME, MODULE_CODE_SIZE_LIMIT, PROTOCOL_PDART, Protocol, ProtocolError,
    ProtocolId, ProtocolNumber, ProtocolVersion, WorkFlags, WorkRequest, WorkRequestId,
    WorkResponse, WorkResponseResult, WorkSeed, WorkStatus, WorkStatusFlagsAndId, WorkerSessionId,
    WorkerVersion, config::*, error::*,
};
use polymesh_worker_common::{
    BackendModuleKind, PROTOCOL_TESTING, ProtocolInitializationMethod, ProtocolModuleConfig,
    ProtocolModuleConfigHash,
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
    match sp_maybe_compressed_blob::decompress(bytes, MODULE_CODE_SIZE_LIMIT) {
        Ok(decompressed) => Some(decompressed.to_vec()),
        Err(err) => {
            log::error!("Failed to decompress module code: {err}");
            None
        }
    }
}

pub struct StaticProtocol {
    config: ProtocolModuleConfig,
    config_hash: ProtocolModuleConfigHash,
    polkavm_code: &'static [u8],
    polkavm_code_hash: BackendCodeHash,
    wasm_code: &'static [u8],
    wasm_code_hash: BackendCodeHash,
    native_code: Vec<u8>,
    native_code_hash: BackendCodeHash,
}

impl StaticProtocol {
    pub fn new(protocol: Protocol, polkavm_code: &'static [u8], wasm_code: &'static [u8]) -> Self {
        let mut modules = Vec::new();

        let polkavm_code_hash = blake2b256_hash(polkavm_code);
        let wasm_code_hash = blake2b256_hash(wasm_code);
        let native_code = protocol.encode();
        let native_code_hash = blake2b256_hash(&native_code);
        // Setup module definitions for the protocol config.
        modules.push(BackendModuleDefinition {
            module_kind: BackendModuleKind::PolkaVM,
            module_version: 1,
            code_hash: polkavm_code_hash,
        });
        modules.push(BackendModuleDefinition {
            module_kind: BackendModuleKind::Wasm,
            module_version: 1,
            code_hash: wasm_code_hash,
        });
        modules.push(BackendModuleDefinition {
            module_kind: BackendModuleKind::Native,
            module_version: 1,
            code_hash: native_code_hash,
        });
        let config = ProtocolModuleConfig {
            protocol,
            initialization_method: ProtocolInitializationMethod::SaveContextFromFirstInstance,
            modules,
        };
        let config_hash = blake2b256_hash(&config.encode());

        Self {
            config,
            config_hash,
            polkavm_code,
            polkavm_code_hash,
            wasm_code,
            wasm_code_hash,
            native_code,
            native_code_hash,
        }
    }

    pub fn get_code_bytes(
        &self,
        kind: BackendModuleKind,
        code_hash: BackendCodeHash,
    ) -> Option<Vec<u8>> {
        match kind {
            BackendModuleKind::PolkaVM if code_hash == self.polkavm_code_hash => {
                decompress_module_code(self.polkavm_code)
            }
            BackendModuleKind::Wasm if code_hash == self.wasm_code_hash => {
                decompress_module_code(self.wasm_code)
            }
            BackendModuleKind::Native if code_hash == self.native_code_hash => {
                Some(self.native_code.clone())
            }
            _ => None,
        }
    }
}

pub struct StaticModules {
    initialized: bool,
    protocols: BTreeMap<Protocol, StaticProtocol>,
}

impl StaticModules {
    pub fn new() -> Self {
        Self {
            initialized: false,
            protocols: BTreeMap::new(),
        }
    }

    fn dart_polkavm_bytes(&self) -> &'static [u8] {
        #[cfg(not(feature = "testing"))]
        {
            include_bytes!("../polymesh-worker-protocol-dart-v1.polkavm.zst")
        }
        #[cfg(feature = "testing")]
        {
            include_bytes!("../polymesh-worker-protocol-dart-v1.testing.polkavm.zst")
        }
    }

    fn dart_wasm_bytes(&self) -> &'static [u8] {
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
        // Add P-DART protocol static modules.
        let protocol = Protocol {
            id: PROTOCOL_PDART,
            version: ProtocolVersion::new(0, 1, 0),
        };
        self.protocols.insert(
            protocol,
            StaticProtocol::new(protocol, self.dart_polkavm_bytes(), self.dart_wasm_bytes()),
        );

        // Add Testing protocol static modules if the testing feature is enabled.
        //#[cfg(feature = "testing")]
        {
            let protocol = Protocol {
                id: PROTOCOL_TESTING,
                version: ProtocolVersion::new(0, 1, 0),
            };
            let polkavm_code = include_bytes!(
                "../protocol/testing/v0/polymesh-worker-protocol-testing.polkavm.zst"
            );
            let wasm_code =
                include_bytes!("../protocol/testing/v0/polymesh-worker-protocol-testing.wasm.zst");
            self.protocols.insert(
                protocol,
                StaticProtocol::new(protocol, polkavm_code, wasm_code),
            );
        }

        self.initialized = true;
    }
}

impl backend::BackendModuleLoader for StaticModules {
    fn get_protocol_module_config_hash(
        &mut self,
        protocol: Protocol,
    ) -> Option<ProtocolModuleConfigHash> {
        self.initialize();
        if let Some(static_protocol) = self.protocols.get(&protocol) {
            Some(static_protocol.config_hash)
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
        self.initialize();
        if let Some(static_protocol) = self.protocols.get(&protocol) {
            if static_protocol.config_hash == config_hash {
                Some(static_protocol.config.clone())
            } else {
                None
            }
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
        self.initialize();
        if let Some(static_protocol) = self.protocols.get(&protocol) {
            static_protocol.get_code_bytes(kind, code_hash)
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
