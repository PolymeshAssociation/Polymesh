use codec::Encode;
use confidential_identity::{compute_cdd_id, compute_scope_id, mocked};
use polymesh_primitives::{IdentityId, InvestorUid, InvestorZKProofData, Ticker};
use std::convert::TryFrom;

fn main() {
    let user_did = b"did:poly:1906c0a0f58364d3f71c4e94e1361af9810666445564840c96f9f1a965cf6045";
    let ticker_name = b"YOLO";
    let did = IdentityId::try_from(&user_did[..]).unwrap();
    let ticker = Ticker::try_from(&ticker_name[..]).unwrap();
    let uid = InvestorUid::from(mocked::make_investor_uid(did.as_bytes()));
    let proof = InvestorZKProofData::new(&did, &uid, &ticker);
    let cdd_claim = InvestorZKProofData::make_cdd_claim(&did, &uid);
    let cdd_id = compute_cdd_id(&cdd_claim).compress().to_bytes();
    let scope_claim = InvestorZKProofData::make_scope_claim(&ticker.as_slice(), &uid);
    let scope_id = compute_scope_id(&scope_claim).compress().to_bytes();
    println!("ScopeId: 0x{}", hex::encode(scope_id));
    println!("CddId: 0x{}", hex::encode(cdd_id));
    println!("Proof: 0x{}", hex::encode(proof.encode()));
}
