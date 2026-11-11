use frame_benchmarking::benchmarks;
use frame_support::dispatch::RawOrigin;
use frame_support::pallet_prelude::*;
use sp_io::hashing::blake2_256;
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

fn dummy_modules<T: Config>(protocol: Protocol, module_count: u32) -> Vec<BackendModuleDefinition> {
    let mut modules = vec![];
    let module_version = 2;
    for idx in 0..(module_count - 1) {
        let code = vec![idx as u8; T::MaxModuleCodeSize::get() as usize];
        let code_hash = blake2_256(&code);
        Pallet::<T>::upload_protocol_module_code(
            RawOrigin::Root.into(),
            protocol,
            code.try_into()
                .expect("Failed to convert code to BoundedVec"),
            module_version,
        )
        .expect("Failed to upload dummy module code");
        modules.push(BackendModuleDefinition {
            module_kind: BackendModuleKind::Wasm,
            module_version,
            code_hash,
        });
    }

    let native_code = protocol.encode();
    let native_code_hash = blake2_256(&native_code);
    modules.push(BackendModuleDefinition {
        module_kind: BackendModuleKind::Native,
        module_version,
        code_hash: native_code_hash,
    });
    modules
}

benchmarks! {
    register_protocol {
        let (protocol_id, metadata) = dummy_protocol();
    }: _(RawOrigin::Root, protocol_id, metadata)

    upload_protocol_module_code {
        let protocol = register_dummy_protocol::<T>().expect("Failed to register dummy protocol");
        let code = vec![0u8; T::MaxModuleCodeSize::get() as usize].try_into().expect("Failed to convert code to BoundedVec"); // dummy code
    }: _(RawOrigin::Root, protocol, code, 2)

    upload_protocol_module_context {
        let protocol = register_dummy_protocol::<T>().expect("Failed to register dummy protocol");
        let context = vec![0u8; T::MaxModuleContextSize::get() as usize].try_into().expect("Failed to convert context to BoundedVec"); // dummy context
    }: _(RawOrigin::Root, protocol, context)

    upload_protocol_module_config_context_data {
        let m in 1 .. T::MaxModulesPerConfig::get() as u32;
        let c in 0 .. T::MaxModuleContextSize::get();

        let protocol = register_dummy_protocol::<T>().expect("Failed to register dummy protocol");
        let modules = dummy_modules::<T>(protocol, m);
        let context = vec![0u8; c as usize]
            .try_into()
            .expect("Failed to convert context to BoundedVec");
        let config = ProtocolModuleConfig {
            protocol,
            initialization_method: ProtocolInitializationMethod::ContextData(context),
            modules: modules
                .try_into()
                .expect("Failed to convert backends to BoundedVec"),
        };
    }: {
        Pallet::<T>::upload_protocol_module_config(RawOrigin::Root.into(), config)?;
    }

    upload_protocol_module_config_context_hash {
        let m in 1 .. T::MaxModulesPerConfig::get() as u32;

        let protocol = register_dummy_protocol::<T>().expect("Failed to register dummy protocol");
        let modules = dummy_modules::<T>(protocol, m);
        let context: BoundedVec<u8, T::MaxModuleContextSize> =
            vec![0u8; T::MaxModuleContextSize::get() as usize]
            .try_into()
            .expect("Failed to convert context to BoundedVec");
        let context_hash = blake2_256(&context);
        Pallet::<T>::upload_protocol_module_context(RawOrigin::Root.into(), protocol, context)
            .expect("Failed to upload dummy context");
        let config = ProtocolModuleConfig {
            protocol,
            initialization_method: ProtocolInitializationMethod::ContextHash(context_hash),
            modules: modules
                .try_into()
                .expect("Failed to convert backends to BoundedVec"),
        };
    }: {
        Pallet::<T>::upload_protocol_module_config(RawOrigin::Root.into(), config)?;
    }
}
