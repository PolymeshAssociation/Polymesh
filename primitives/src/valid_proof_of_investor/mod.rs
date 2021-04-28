use crate::{investor_zkproof_data::InvestorZKProofData, Claim, IdentityId, Scope};

/// Validator for PIUS v1;
pub mod v1;
/// Validator for PIUS v2;
pub mod v2;

/// Evaluates if the claim is a valid proof.
pub fn evaluate_claim(
    scope: &Scope,
    claim: &Claim,
    id: &IdentityId,
    general_proof: &InvestorZKProofData,
) -> bool {
    match general_proof {
        InvestorZKProofData::V1(proof) => v1::evaluate_claim(scope, claim, id, proof),

        InvestorZKProofData::V2(proof) => v2::evaluate_claim(scope, claim, id, proof),
    }
}
