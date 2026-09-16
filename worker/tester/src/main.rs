use codec::{Compact, Decode, Encode};
use polymesh_dart::{
    AccountAssetRegistrationProof, BatchedAccountAssetRegistrationProof, LegEncrypted,
    PolymeshLimits, SenderAffirmationProof, curve_tree::AccountTreeConfig,
};
use polymesh_worker::{backend::*, *};
use polymesh_worker_common::{PROTOCOL_PDART, ResolvedInitializationMethod};
use polymesh_worker_protocol_dart_v1::{
    AccountTreeRoot, DartWorkRequest, DartWorkResponse, VerifyDartAssetRequest,
};

/// Inflate a SCALE-encoded `LegEncrypted` (`Compact(len) ++ canonical_bytes`) by appending
/// `pad_len` trailing zero bytes inside the wrapped blob. `WrappedCanonical::decode` retains the
/// padding in its `wrapped: Vec<u8>`, while `deserialize_compressed` reads only the canonical
/// prefix, so the decoded leg (and thus the proof) is unchanged.
fn pad_leg(raw_leg_enc: &[u8], pad_len: usize) -> LegEncrypted {
    let mut input = &raw_leg_enc[..];
    let Compact(canonical_len) = Compact::<u32>::decode(&mut input).unwrap();
    let mut expanded = Compact((canonical_len as usize + pad_len) as u32).encode();
    expanded.extend_from_slice(input);
    expanded.resize(expanded.len() + pad_len, 0);
    // NB: use the codec `Decode` trait explicitly; `LegEncrypted` also has an inherent
    // `decode(&self)` method that would otherwise shadow it.
    <LegEncrypted as Decode>::decode(&mut &expanded[..]).unwrap()
}

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
        if hash != ref_hash {
            // Instrumented guest rebuilds may shift the blob; the context params are unchanged,
            // so warn instead of aborting the sweep.
            println!("WARNING: saved context hash != reference ({ref_hash})");
        }
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
        let proof = AccountAssetRegistrationProof::<PolymeshLimits>::decode(&mut &raw_proof[..])
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

    // ---- PoC: padded-leg sweep on SenderAffirmation ----
    //
    // For each pad size we build a fresh module instance (a VM trap corrupts the store, so per-size
    // isolation is required), initialize it from the saved context, then run the padded request 4x
    // and print the RAW nested Result so the four outcomes stay distinguishable:
    //   Ok(Ok(WorkResponse[..]))          -> guest ran, proof processed (no divergence)
    //   Ok(Err(ExecuteWorkFailed))        -> guest trapped (heap/scratch exhaustion or panic) => DIVERGENCE
    //   Ok(Err(CustomProtocolError([..])))-> proof rejected as invalid (padding broke verification)
    //   Ok(Err(DecodingFailed))           -> guest `execute` returned 0 (clean decode reject)
    //   Err(WorkerError::..)              -> host-side failure (req > 10MB scratch, etc.)
    {
        let raw_proof = include_bytes!("../data/sender-affirm-proof.dat");
        let raw_leg_enc = include_bytes!("../data/settlement_2_leg_0.bin");
        let raw_account_root = include_bytes!("../data/block_12_current_account_root.bin");

        let proof = SenderAffirmationProof::<PolymeshLimits, AccountTreeConfig>::decode(
            &mut &raw_proof[..],
        )
        .expect("Failed to decode proof");
        let root: AccountTreeRoot =
            Decode::decode(&mut &raw_account_root[..]).expect("Failed to decode account root");

        // Sweep points (KiB). Fine bisection between 5M and 6M to pin the heap-trap threshold.
        let pad_sizes_kib: [usize; 18] = [
            0, 1024, 2048, 3072, 3994, 5120, 5376, 5632, 5888, 6144, 8192, 9728, 10752, 12288,
            15360, 18432, 20480, 10240,
        ];

        for pad_kib in pad_sizes_kib {
            let pad_len = pad_kib * 1024;
            let leg_enc = pad_leg(&raw_leg_enc[..], pad_len);
            let req = DartWorkRequest::VerifyProof(VerifyDartAssetRequest::SenderAffirmation {
                proof: proof.clone(),
                leg_enc,
                root: root.clone(),
            });
            let work = WorkRequest::new(req);
            let req_len = work.0.len();

            // Fresh instance per pad size, initialized from the saved fast-path context.
            let mut inst = module.instantiate().expect("Failed to instantiate module");
            inst.initialize(saved_ctx.as_deref())
                .expect("Failed to initialize instance with saved context");

            for i in 0..4 {
                let now = std::time::Instant::now();
                let res = inst.execute(&work);
                let pretty = match &res {
                    Ok(Ok(resp)) => {
                        let decoded = resp.decode::<DartWorkResponse>();
                        match decoded {
                            Ok(_) => format!("Ok(Ok(WorkResponse[{} bytes], decoded OK))", resp.0.len()),
                            Err(e) => format!(
                                "Ok(Ok(WorkResponse[{} bytes], DECODE-ERR {:?}))",
                                resp.0.len(),
                                e
                            ),
                        }
                    }
                    Ok(Err(e)) => format!("Ok(Err({:?}))", e),
                    Err(e) => format!("Err({:?})", e),
                };
                println!(
                    "SWEEP backend={_kind:?} pad_kib={pad_kib} pad_bytes={pad_len} req_len={req_len} iter={i} elapsed={:?} => {pretty}",
                    now.elapsed()
                );
            }
        }
    }
}
