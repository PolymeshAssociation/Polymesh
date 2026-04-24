use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use codec::{Decode, Encode};
use polymesh_dart::{
    AccountAssetRegistrationProof, BatchedAccountAssetRegistrationProof, LegEncrypted,
    SenderAffirmationProof, curve_tree::AccountTreeConfig,
};
use polymesh_worker::{backend::*, *};
use polymesh_worker_common::PROTOCOL_PDART;
use polymesh_worker_protocol_dart_v0::{
    AccountTreeRoot, DartWorkRequest, DartWorkResponse, VerifyDartAssetRequest,
};

pub fn signer_to_did(signer_name: &str) -> [u8; 32] {
    let mut did = [0u8; 32];
    let name_bytes = signer_name.as_bytes();
    let len = name_bytes.len().min(32);
    did[..len].copy_from_slice(&name_bytes[..len]);
    did
}

fn bench_backend_kind(kind: BackendKind, c: &mut Criterion) {
    let native_backend = polymesh_worker_native::NativeBackend;
    println!("Using backend: {:?}", kind);
    let backends = Backends::with_backends(&[kind], Some(Box::new(native_backend)));

    let mut loader = StaticModules::new();
    let protocol = Protocol {
        id: PROTOCOL_PDART,
        version: ProtocolVersion {
            major: 0,
            minor: 1,
            patch: 0,
        },
    };

    let mut group = c.benchmark_group(format!("Backend: {:?}", kind));

    // Benchmark loading the module.
    group.bench_with_input(
        format!("LoadModule: {:?}", kind),
        &backends,
        |b, backends| {
            b.iter(|| {
                black_box(
                    backends
                        .load_module(protocol, &mut loader)
                        .expect("No backend available for the given protocol and version"),
                );
            })
        },
    );
    // Load the module.
    let module = backends
        .load_module(protocol, &mut loader)
        .expect("No backend available for the given protocol and version");

    // Load the module context.
    let config_hash = loader
        .get_protocol_module_config_hash(protocol)
        .expect("Failed to get protocol module config hash");
    let config = loader
        .get_protocol_module_config(protocol, config_hash)
        .expect("Failed to get protocol module config");
    let context_bytes = if let Some(context_hash) = config.context_hash {
        loader.get_module_context_bytes(protocol, context_hash)
    } else {
        None
    };
    println!("Context loaded: {}", context_bytes.is_some());

    // Benchmark instantiating the module.
    group.bench_with_input(format!("Instantiate: {:?}", kind), &module, |b, module| {
        b.iter(|| {
            black_box(module.instantiate().expect("Failed to instantiate module"));
        })
    });

    // Instantiate the module.
    let mut instance = module.instantiate().expect("Failed to instantiate module");

    // Benchmark initializing the module.
    group.bench_with_input(
        format!("InitializeNoContext: {:?}", kind),
        &module,
        |b, module| {
            let mut instance = module.instantiate().expect("Failed to instantiate module");
            b.iter(|| {
                black_box(
                    instance
                        .initialize(None)
                        .expect("Failed to initialize module"),
                );
            })
        },
    );

    // Initialize the module.
    {
        instance
            .initialize(context_bytes.as_deref())
            .expect("Failed to initialize module");
    }

    // Benchmark saving the module context.
    group.bench_with_input(format!("SaveContext: {:?}", kind), &module, |b, module| {
        let mut instance = module.instantiate().expect("Failed to instantiate module");
        b.iter(|| {
            black_box(instance.save_context().expect("Failed to save context"));
        })
    });

    // Save the module context.
    let saved_ctx = instance.save_context().expect("Failed to save context");
    if let Some(ref ctx) = saved_ctx {
        let hash = sp_core::blake2_256(ctx);
        println!("Saved context size: {} bytes", ctx.len());
        println!("Saved context hash: 0x{}", hex::encode(hash));
    }

    // Benchmark initializing with saved context.
    group.bench_with_input(
        format!("InitializeWithContext: {:?}", kind),
        &(&module, &saved_ctx),
        |b, (module, saved_ctx)| {
            let mut instance = module.instantiate().expect("Failed to instantiate module");
            b.iter(|| {
                black_box(
                    instance
                        .initialize(saved_ctx.as_deref())
                        .expect("Failed to initialize with saved context"),
                );
            })
        },
    );

    // Test loading the context back into a new instance.
    {
        instance
            .initialize(saved_ctx.as_deref())
            .expect("Failed to initialize with saved context");
    }

    let mut execute_work = |name: &str, req: DartWorkRequest| -> u32 {
        let req = WorkRequest::new(req);
        black_box(instance.execute(&req).map(|res| {
            res?.decode()
                .map(|res: DartWorkResponse| res.encoded_size() as u32)
        }))
        .expect(&format!("Failed to execute work: {}", name))
        .expect(&format!("Protocol worker failed: {}", name))
    };

    // Benchmark verifying the register account asset proof.
    {
        let raw_proof = include_bytes!("../data/register-account-proof.dat");
        let proof = AccountAssetRegistrationProof::decode(&mut &raw_proof[..])
            .expect("Failed to decode proof");
        let did = signer_to_did("investor");

        group.bench_with_input(
            format!("VerifyRegisterAccountAssetProof: {:?}", kind),
            &(proof, did),
            |b, (proof, did)| {
                b.iter(|| {
                    black_box(execute_work(
                        "verify_register_account_asset_proof",
                        DartWorkRequest::VerifyProof(
                            VerifyDartAssetRequest::BatchedAccountAssetRegistration {
                                did: *did,
                                proof: BatchedAccountAssetRegistrationProof {
                                    proofs: vec![proof.clone()]
                                        .try_into()
                                        .expect("Proof vector has incorrect length"),
                                },
                            },
                        ),
                    ));
                })
            },
        );
    }

    // Benchmark verifying the sender affirmation proof.
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

        group.bench_with_input(
            format!("VerifySenderAffirmProof: {:?}", kind),
            &(proof, leg_enc, root),
            |b, (proof, leg_enc, root)| {
                b.iter(|| {
                    black_box(execute_work(
                        "verify_sender_affirm_proof",
                        DartWorkRequest::VerifyProof(VerifyDartAssetRequest::SenderAffirmation {
                            proof: proof.clone(),
                            leg_enc: leg_enc.clone(),
                            root: root.clone(),
                        }),
                    ));
                })
            },
        );
    }
}

fn bench_native(c: &mut Criterion) {
    bench_backend_kind(BackendKind::Native, c);
}

fn bench_polkavm(c: &mut Criterion) {
    bench_backend_kind(BackendKind::PolkaVM, c);
}

fn bench_wasmer(c: &mut Criterion) {
    bench_backend_kind(BackendKind::Wasmer, c);
}

fn bench_wasmtime(c: &mut Criterion) {
    bench_backend_kind(BackendKind::Wasmtime, c);
}

criterion_group!(
    backend_benches,
    bench_native,
    bench_polkavm,
    bench_wasmer,
    bench_wasmtime
);
criterion_main!(backend_benches);
