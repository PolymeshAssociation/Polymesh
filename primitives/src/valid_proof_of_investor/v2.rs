use crate::{
    investor_zkproof_data::v2::InvestorZKProofData, CddId, Claim, IdentityId, InvestorUid,
};

use confidential_identity::{
    claim_proofs::{slice_to_scalar, Verifier},
    cryptography_core::cdd_claim::CddId as CryptoCddId,
    mocked::make_investor_uid,
    CompressedRistretto, Scalar, VerifierTrait as _,
};

/// Evaluates if the claim is a valid proof.
pub fn evaluate_claim(claim: &Claim, id: &IdentityId, proof: &InvestorZKProofData) -> bool {
    match claim {
        Claim::InvestorUniqueness(scope, scope_did, cdd_id) => {
            // NB As Investor UID generation is still fixed, we can double-check that `scope` was
            // used to generate `scope_did`.
            // We will need a new way to proof this relation when `InvestorUid` will be not mocked.
            let investor: InvestorUid = make_investor_uid(id.as_bytes()).into();
            let target_scope_id = InvestorZKProofData::make_scope_id(scope.as_bytes(), &investor);

            target_scope_id == *scope_did && verify_proof(id, scope_did, cdd_id, proof)
        }
        _ => false,
    }
}

fn verify_proof(
    investor: &IdentityId,
    scope: &IdentityId,
    cdd_id: &CddId,
    proof: &InvestorZKProofData,
) -> bool {
    if let Some(cdd_id_point) = CompressedRistretto::from_slice(cdd_id.as_slice()).decompress() {
        if let Some(scope) = Scalar::from_canonical_bytes(scope.0.clone()) {
            let investor = slice_to_scalar(investor.as_bytes());
            let cdd_id = CryptoCddId(cdd_id_point);

            return Verifier::verify_scope_claim_proof(&proof.0, &investor, &scope, &cdd_id)
                .is_ok();
        }
    }
    false
}
