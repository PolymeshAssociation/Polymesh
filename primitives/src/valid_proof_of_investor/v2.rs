use crate::{investor_zkproof_data::v2::InvestorZKProofData, CddId, Claim, IdentityId, Scope};

use confidential_identity::{
    claim_proofs::{slice_to_scalar, Verifier},
    cryptography_core::cdd_claim::CddId as CryptoCddId,
    CompressedRistretto, VerifierTrait as _,
};

/// Evaluates if the claim is a valid proof and if `scope_id` matches with the one inside `proof`.
pub fn evaluate_claim(
    scope: &Scope,
    claim: &Claim,
    id: &IdentityId,
    proof: &InvestorZKProofData,
) -> bool {
    match claim {
        Claim::InvestorUniquenessV2(cdd_id) => verify_proof(id, scope.as_bytes(), cdd_id, proof),
        _ => false,
    }
}

fn verify_proof(
    user: &IdentityId,
    scope: &[u8],
    cdd_id: &CddId,
    proof: &InvestorZKProofData,
) -> bool {
    if let Some(cdd_id_point) = CompressedRistretto::from_slice(cdd_id.as_slice()).decompress() {
        let scope = slice_to_scalar(scope);
        let user = slice_to_scalar(user.as_bytes());
        let cdd_id = CryptoCddId(cdd_id_point);

        return Verifier::verify_scope_claim_proof(&proof.0, &user, &scope, &cdd_id).is_ok();
    }
    false
}
