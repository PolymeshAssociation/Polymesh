use codec::{Decode, Encode};
use polymesh_dart::{
    AccountAssetRegistrationProof, BatchedAccountAssetRegistrationProof, LegEncrypted,
    SenderAffirmationProof, curve_tree::AccountTreeConfig,
};
use polymesh_worker::*;
use polymesh_worker_protocol_dart_v0::{AccountTreeRoot, DartWorkRequest, VerifyDartAssetRequest};

pub fn signer_to_did(signer_name: &str) -> [u8; 32] {
    let mut did = [0u8; 32];
    let name_bytes = signer_name.as_bytes();
    let len = name_bytes.len().min(32);
    did[..len].copy_from_slice(&name_bytes[..len]);
    did
}

pub fn main() {
    // Parse the first command line argument as the backend kind, default to PolkaVM if not provided or invalid.
    let backend_kind = std::env::args()
        .nth(1)
        .and_then(|arg| match arg.to_lowercase().as_str() {
            "polkavm" => Some(BackendKind::PolkaVM),
            "native" => Some(BackendKind::Native),
            "wasmtime" => Some(BackendKind::Wasmtime),
            _ => None,
        })
        .unwrap_or(BackendKind::PolkaVM);

    println!("Using backend: {:?}", backend_kind);

    let backends = Backends::with_backends(vec![backend_kind]);
    let loader = StaticModules;
    let protocol = Protocol {
        id: PROTOCOL_PDART,
        version: ProtocolVersion {
            major: 0,
            minor: 1,
            patch: 0,
        },
    };

    // Load the module.
    let mut module = backends
        .load_module(protocol, &loader)
        .expect("No backend available for the given protocol and version");

    let mut execute_work = |name: &str, req: DartWorkRequest| {
        let req = WorkRequest::new(protocol, req);
        for _ in 0..4 {
            let now = std::time::Instant::now();
            let res: Result<u32, Error> = module.execute(&req).decode();
            println!("{name} Result: {:?}", res);
            println!("{name} Execution time: {:?}", now.elapsed());
        }
    };

    // Verify the register account asset proof.
    {
        let raw_proof = include_bytes!("../register-account-proof.dat");
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
        let raw_proof = include_bytes!("../sender-affirm-proof.dat");
        let raw_leg_enc = include_bytes!("../settlement_2_leg_0.bin");
        let raw_account_root = include_bytes!("../block_12_current_account_root.bin");

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
