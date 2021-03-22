use crate::{investor_zkproof_data::InvestorZKProofData, Claim, IdentityId};

/// Validator for PIUS v1;
pub mod v1;
/// Validator for PIUS v2;
pub mod v2;

/// Evaluates if the claim is a valid proof.
pub fn evaluate_claim(claim: &Claim, id: &IdentityId, general_proof: &InvestorZKProofData) -> bool {
    match general_proof {
        InvestorZKProofData::V1(proof) => v1::evaluate_claim(claim, id, proof),

        InvestorZKProofData::V2(proof) => v2::evaluate_claim(claim, id, proof),
    }
}
