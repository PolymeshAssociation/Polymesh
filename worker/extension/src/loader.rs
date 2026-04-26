use sp_std::vec::Vec;

#[cfg(feature = "std")]
use codec::Decode;
use codec::Encode;

#[cfg(feature = "std")]
use polymesh_worker::blake2b256_hash;
use polymesh_worker_common::*;

/// Storage key definitions:
///
/// Pallet name: `WorkerModules` (twox_128 = e67cf3f4b484981dea7be98c7cbbd979)
///
/// The storage items are:
/// - `ProtocolConfigHash` (twox_128 = 7ada29e3bb7bec6691576e935738af7b): map Protocol => Hash
/// - `ProtocolConfig` (twox_128 = 63d64a3f5c7590e883a2f77f74b5714d): map (Protocol, Hash) => ProtocolModuleConfig
/// - `ProtocolModuleCode` (twox_128 = deae62fbb378690611a6e5113d494b1b): map (Protocol, Hash) => Vec<u8>
/// - `ProtocolContext` (twox_128 = 27c0c49da7153d33d35d497fc4ac56f0): map (Protocol, Hash) => Vec<u8>
///
/// The storage prefix is `concat(twox_128(pallet_name), twox_128(storage_name))`.

/// Pallet WorkerModules storage prefix.
pub const WORKER_MODULES_PREFIX: [u8; 16] = hex_literal::hex!("e67cf3f4b484981dea7be98c7cbbd979");

/// The prefix for `ProtocolConfigHash` storage item, which maps Protocol => Hash.
pub const PROTOCOL_CONFIG_HASH_PREFIX: [u8; 16] =
    hex_literal::hex!("7ada29e3bb7bec6691576e935738af7b");

/// The prefix for `ProtocolConfig` storage item, which double map (Protocol, Hash) => ProtocolModuleConfig.
pub const PROTOCOL_CONFIG_PREFIX: [u8; 16] = hex_literal::hex!("63d64a3f5c7590e883a2f77f74b5714d");

/// The prefix for `ProtocolModuleCode` storage item, which double map (Protocol, Hash) => Vec<u8>.
pub const PROTOCOL_MODULE_CODE_PREFIX: [u8; 16] =
    hex_literal::hex!("deae62fbb378690611a6e5113d494b1b");

/// The prefix for `ProtocolContext` storage item, which double map (Protocol, Hash) => Vec<u8>.
pub const PROTOCOL_CONTEXT_PREFIX: [u8; 16] = hex_literal::hex!("27c0c49da7153d33d35d497fc4ac56f0");

fn worker_modules_storage_key(
    storage_item_prefix: [u8; 16],
    protocol: Protocol,
    hash: Option<[u8; 32]>,
) -> Vec<u8> {
    if let Some(hash) = hash {
        (WORKER_MODULES_PREFIX, storage_item_prefix, protocol, hash).encode()
    } else {
        (WORKER_MODULES_PREFIX, storage_item_prefix, protocol).encode()
    }
}

/// Generate the storage key for `WorkerModules::ProtocolConfigHash(protocol)`.
pub fn worker_modules_config_hash_key(protocol: Protocol) -> Vec<u8> {
    worker_modules_storage_key(PROTOCOL_CONFIG_HASH_PREFIX, protocol, None)
}

/// Generate the storage key for `WorkerModules::ProtocolConfig(protocol, config_hash)`.
pub fn worker_modules_config_key(
    protocol: Protocol,
    config_hash: ProtocolModuleConfigHash,
) -> Vec<u8> {
    worker_modules_storage_key(PROTOCOL_CONFIG_PREFIX, protocol, Some(config_hash))
}

/// Generate the storage key for `WorkerModules::ProtocolModuleCode(protocol, code_hash)`.
pub fn worker_modules_code_key(protocol: Protocol, code_hash: BackendCodeHash) -> Vec<u8> {
    worker_modules_storage_key(PROTOCOL_MODULE_CODE_PREFIX, protocol, Some(code_hash))
}

/// Generate the storage key for `WorkerModules::ProtocolContext(protocol, context_hash)`.
pub fn worker_modules_context_key(protocol: Protocol, context_hash: BackendContextHash) -> Vec<u8> {
    worker_modules_storage_key(PROTOCOL_CONTEXT_PREFIX, protocol, Some(context_hash))
}

#[cfg(feature = "std")]
pub struct SubstrateModuleLoader<'a>(pub &'a mut dyn sp_externalities::Externalities);

#[cfg(feature = "std")]
impl<'a> SubstrateModuleLoader<'a> {
    fn storage(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.0.storage(key)
    }
}

#[cfg(feature = "std")]
fn verify_hash(msg: &str, expected: &[u8; 32], data: Vec<u8>) -> Option<Vec<u8>> {
    let actual = blake2b256_hash(&data);
    if &actual != expected {
        log::error!("{}: expected {:x?}, got {:x?}", msg, expected, actual);
        None
    } else {
        Some(data)
    }
}

#[cfg(feature = "std")]
impl<'a> polymesh_worker::backend::BackendModuleLoader for SubstrateModuleLoader<'a> {
    fn get_protocol_module_config_hash(
        &mut self,
        protocol: Protocol,
    ) -> Option<ProtocolModuleConfigHash> {
        self.storage(&worker_modules_config_hash_key(protocol))
            .and_then(|bytes| Decode::decode(&mut &bytes[..]).ok())
    }

    fn get_protocol_module_config(
        &mut self,
        protocol: Protocol,
        config_hash: ProtocolModuleConfigHash,
    ) -> Option<ProtocolModuleConfig> {
        self.storage(&worker_modules_config_key(protocol, config_hash))
            // Verify the config hash matches the read config, to prevent malicious block producers.
            .and_then(|bytes| {
                verify_hash(
                    &format!(
                        "Protocol module config hash mismatch for protocol {:?}",
                        protocol
                    ),
                    &config_hash,
                    bytes,
                )
            })
            .and_then(|config| Decode::decode(&mut &config[..]).ok())
    }

    fn get_module_code_bytes(
        &mut self,
        protocol: Protocol,
        _kind: BackendModuleKind,
        code_hash: BackendCodeHash,
    ) -> Option<Vec<u8>> {
        self.storage(&worker_modules_code_key(protocol, code_hash))
            // Verify the code hash matches the read code, to prevent malicious block producers.
            .and_then(|bytes| {
                verify_hash(
                    &format!(
                        "Protocol module code hash mismatch for protocol {:?}",
                        protocol
                    ),
                    &code_hash,
                    bytes,
                )
            })
    }

    fn get_module_context_bytes(
        &mut self,
        protocol: Protocol,
        ctx_hash: BackendContextHash,
    ) -> Option<Vec<u8>> {
        self.storage(&worker_modules_context_key(protocol, ctx_hash))
            // Verify the context hash matches the read context, to prevent malicious block producers.
            .and_then(|bytes| {
                verify_hash(
                    &format!(
                        "Protocol module context hash mismatch for protocol {:?}",
                        protocol
                    ),
                    &ctx_hash,
                    bytes,
                )
            })
    }
}
