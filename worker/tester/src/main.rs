use codec::{Decode, Encode};
use polymesh_dart::{
    AccountAssetRegistrationProof, BatchedAccountAssetRegistrationProof, LegEncrypted,
    SenderAffirmationProof, curve_tree::AccountTreeConfig,
};
use polymesh_worker::{backend::*, *};
use polymesh_worker_common::{PROTOCOL_PDART, ResolvedInitializationMethod};
use polymesh_worker_protocol_dart_v1::{
    AccountTreeRoot, DartWorkRequest, DartWorkResponse, VerifyDartAssetRequest,
};

pub fn signer_to_did(signer_name: &str) -> [u8; 32] {
    let mut did = [0u8; 32];
    let name_bytes = signer_name.as_bytes();
    let len = name_bytes.len().min(32);
    did[..len].copy_from_slice(&name_bytes[..len]);
    did
}

pub fn main() {
    env_logger::init();
    // Parse the first command line argument as the backend kind, default to PolkaVM if not provided or invalid.
    let backend_kind = std::env::args()
        .nth(1)
        .and_then(|arg| match arg.to_lowercase().as_str() {
            "polkavm" => Some(BackendKind::PolkaVM),
            "native" => Some(BackendKind::Native),
            "wasmtime" => Some(BackendKind::Wasmtime),
            "wasmer" => Some(BackendKind::Wasmer),
            _ => None,
        });

    let (backends, _kind) = if let Some(backend_kind) = backend_kind {
        let native_backend = polymesh_worker_native::NativeBackend;
        println!("Using backend: {:?}", backend_kind);
        (
            Backends::with_backends(&[backend_kind], Some(Box::new(native_backend))),
            backend_kind,
        )
    } else {
        println!("No valid backend specified.");
        return;
    };
    let mut loader = StaticModules::new();
    let protocol = Protocol {
        id: PROTOCOL_PDART,
        version: ProtocolVersion {
            major: 0,
            minor: 1,
            patch: 0,
        },
    };

    // Load the module.
    let now = std::time::Instant::now();
    let module = backends
        .load_module(protocol, &mut loader)
        .expect("No backend available for the given protocol and version");
    println!("Module loaded in: {:?}", now.elapsed());

    // Load the module context.
    let now = std::time::Instant::now();
    let config_hash = loader
        .get_protocol_module_config_hash(protocol)
        .expect("Failed to get protocol module config hash");
    let config = loader
        .get_protocol_module_config(protocol, config_hash)
        .expect("Failed to get protocol module config");
    let initialization_method = loader
        .resolve_initialization_method(protocol, &config.initialization_method)
        .expect("Failed to resolve initialization method");
    println!("Module context loaded in: {:?}", now.elapsed());

    // Instantiate the module.
    let now = std::time::Instant::now();
    let mut instance = module.instantiate().expect("Failed to instantiate module");
    println!("Module instantiated in: {:?}", now.elapsed());

    // Initialize the module.
    match initialization_method {
        ResolvedInitializationMethod::NoInitializationNeeded => {
            // No initialization needed, do nothing.
        }
        ResolvedInitializationMethod::InitializeNoContext => {
            let now = std::time::Instant::now();
            if let Err(err) = instance.initialize(None) {
                log::error!("Failed to initialize module instance: {err:?}");
                return;
            }
            println!("Module initialized in: {:?}", now.elapsed());
        }
        ResolvedInitializationMethod::ContextData(context_bytes) => {
            let now = std::time::Instant::now();
            instance
                .initialize(Some(context_bytes.as_ref()))
                .expect("Failed to initialize module");
            println!("Module initialized in: {:?}", now.elapsed());
        }
    }

    // Save the module context.
    let saved_ctx = {
        let now = std::time::Instant::now();
        let save_result = instance.save_context().expect("Failed to save context");
        println!("Context saved: {:?}", save_result.is_some());
        println!("Time taken for saving context: {:?}", now.elapsed());

        save_result
    };
    if let Some(ref ctx) = saved_ctx {
        const REF_CONTEXT_DATA: &[u8] =
            include_bytes!("../../polymesh-worker-protocol-dart-v1.context.bin");
        let ref_hash = hex::encode(sp_core::blake2_256(REF_CONTEXT_DATA));
        let hash = hex::encode(sp_core::blake2_256(ctx));
        println!("Saved context size: {} bytes", ctx.len());
        println!("Saved context hash: 0x{}", hash);
        assert_eq!(
            hash, ref_hash,
            "Saved context does not match reference context"
        );
    } else {
        panic!("Context saving is not supported by the module");
    }

    // Test loading the context back into a new instance.
    {
        let now = std::time::Instant::now();
        instance
            .initialize(saved_ctx.as_deref())
            .expect("Failed to initialize with saved context");
        println!("Time taken for loading context: {:?}", now.elapsed());
    }

    let mut execute_work = |name: &str, req: DartWorkRequest| {
        let req = WorkRequest::new(req);
        for _ in 0..4 {
            println!();
            let now = std::time::Instant::now();
            let res: Result<Result<u32, ProtocolError>, WorkerError> =
                instance.execute(&req).map(|res| {
                    res?.decode()
                        .map(|res: DartWorkResponse| res.encoded_size() as u32)
                });
            println!("{name} Result: {:?}", res);
            println!("{name} Execution time: {:?}", now.elapsed());
        }
    };

    // Verify the register account asset proof.
    {
        let raw_proof = include_bytes!("../data/register-account-proof.dat");
        let proof = AccountAssetRegistrationProof::decode(&mut &raw_proof[..])
            .expect("Failed to decode proof");
        let did = signer_to_did("investor");

        execute_work(
            "verify_register_account_asset_proof",
            DartWorkRequest::VerifyProof(VerifyDartAssetRequest::BatchedAccountAssetRegistration {
                did,
                proof: BatchedAccountAssetRegistrationProof {
                    proofs: vec![proof]
                        .try_into()
                        .expect("Proof vector has incorrect length"),
                },
            }),
        );
    }

    // Verify sender affirmation proof.
    {
        let raw_proof = include_bytes!("../data/sender-affirm-proof.dat");
        let raw_leg_enc = include_bytes!("../data/settlement_2_leg_0.bin");
        let raw_account_root = include_bytes!("../data/block_12_current_account_root.bin");

        let proof = SenderAffirmationProof::<AccountTreeConfig>::decode(&mut &raw_proof[..])
            .expect("Failed to decode proof");
        let leg_enc: LegEncrypted =
            Decode::decode(&mut &raw_leg_enc[..]).expect("Failed to decode leg encryption");
        let root: AccountTreeRoot =
            Decode::decode(&mut &raw_account_root[..]).expect("Failed to decode account root");

        execute_work(
            "verify_sender_affirm_proof",
            DartWorkRequest::VerifyProof(VerifyDartAssetRequest::SenderAffirmation {
                proof,
                leg_enc,
                root,
            }),
        );
    }
}
