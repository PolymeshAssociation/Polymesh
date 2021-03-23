use crate::{investor_zkproof_data::v2::InvestorZKProofData, CddId, Claim, IdentityId};

use confidential_identity::{
    claim_proofs::{slice_to_scalar, Verifier},
    cryptography_core::cdd_claim::CddId as CryptoCddId,
    CompressedRistretto, VerifierTrait as _,
};

/// Evaluates if the claim is a valid proof and if `scope_id` matches with the one inside `proof`.
pub fn evaluate_claim(claim: &Claim, id: &IdentityId, proof: &InvestorZKProofData) -> bool {
    if let Claim::InvestorUniqueness(scope, scope_id, cdd_id) = claim {
        let proof_scope_id = proof.0.scope_id.compress().to_bytes();
        if proof_scope_id == scope_id.as_bytes() {
            return verify_proof(id, scope.as_bytes(), cdd_id, proof);
        }
    }

    false
}

fn verify_proof(
    user: &IdentityId,
    scope: &[u8],
    cdd_id: &CddId,
    proof: &InvestorZKProofData,
) -> bool {
    if let Some(cdd_id_point) = CompressedRistretto::from_slice(cdd_id.as_slice()).decompress() {
        let scope_did = slice_to_scalar(scope);
        let user = slice_to_scalar(user.as_bytes());
        let cdd_id = CryptoCddId(cdd_id_point);

        return Verifier::verify_scope_claim_proof(&proof.0, &user, &scope_did, &cdd_id).is_ok();
    }
    false
}
