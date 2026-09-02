use frame_benchmarking::benchmarks;
use frame_support::dispatch::RawOrigin;
use frame_support::pallet_prelude::*;
use sp_std::vec;

use polymesh_worker_common::{BackendModuleDefinition, BackendModuleKind};

use crate::*;

fn dummy_protocol() -> (ProtocolId, ProtocolMetadata) {
    let protocol_id = ProtocolId(1);
    let metadata = ProtocolMetadata {
        protocol_name: "Test Protocol"
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("Failed to convert protocol name to BoundedVec"),
        protocol_description: "A test protocol for benchmarking"
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("Failed to convert protocol description to BoundedVec"),
        protocol_version: ProtocolVersion::default(),
    };
    (protocol_id, metadata)
}

fn register_dummy_protocol<T: Config>() -> Result<Protocol, DispatchError> {
    let (protocol_id, metadata) = dummy_protocol();
    let protocol = Protocol {
        id: protocol_id,
        version: ProtocolVersion::default(),
    };
    Pallet::<T>::register_protocol(RawOrigin::Root.into(), protocol_id, metadata)?;

    Ok(protocol)
}

benchmarks! {
    register_protocol {
        let (protocol_id, metadata) = dummy_protocol();
    }: _(RawOrigin::Root, protocol_id, metadata)

    upload_protocol_module_code {
        let protocol = register_dummy_protocol::<T>().expect("Failed to register dummy protocol");
        let code = vec![0u8; T::MaxModuleCodeSize::get() as usize].try_into().expect("Failed to convert code to BoundedVec"); // dummy code
    }: _(RawOrigin::Root, protocol, code)

    upload_protocol_module_context {
        let protocol = register_dummy_protocol::<T>().expect("Failed to register dummy protocol");
        let context = vec![0u8; T::MaxModuleContextSize::get() as usize].try_into().expect("Failed to convert context to BoundedVec"); // dummy context
    }: _(RawOrigin::Root, protocol, context)

    upload_protocol_module_config {
        let m in 1 .. T::MaxModulesPerConfig::get() as u32;

        // Register a dummy protocol.
        let protocol = register_dummy_protocol::<T>().expect("Failed to register dummy protocol");

        // Upload dummy code for each module.
        let mut modules = vec![];
        for idx in 0..m {
            let code = vec![idx as u8; 1024]; // dummy code
            let code_hash = code.using_encoded(sp_io::hashing::blake2_256);
            Pallet::<T>::upload_protocol_module_code(RawOrigin::Root.into(), protocol, code.try_into().expect("Failed to convert code to BoundedVec"))
                .expect("Failed to upload dummy module code");
            modules.push(BackendModuleDefinition {
                module_kind: BackendModuleKind::Wasm,
                module_version: 1,
                code_hash,
            });
        }

        // Add a native module to the config.
        let native_code = protocol.encode();
        let native_code_hash = blake2_256(&native_code);
        modules.push(BackendModuleDefinition {
            module_kind: BackendModuleKind::Native,
            module_version: 1,
            code_hash: native_code_hash,
        });

        // Create a dummy protocol module config with the uploaded modules.
        let config = ProtocolModuleConfig {
            protocol,
            initialization_method: ProtocolInitializationMethod::ContextData(
                vec![0u8; T::MaxModuleContextSize::get() as usize]
                    .try_into()
                    .expect("Failed to convert context to BoundedVec"), // dummy context
            ),
            modules: modules
                .try_into()
                .expect("Failed to convert backends to BoundedVec"),
        };
    }: _(RawOrigin::Root, protocol, config)
}
