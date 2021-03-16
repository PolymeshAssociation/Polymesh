use crate::{investor_zkproof_data::InvestorZKProofData, Claim, IdentityId};

pub mod v1;
pub mod v2;

/// Data structure used to check if any of its internal claims exist in context.
#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValidProofOfInvestor;

impl ValidProofOfInvestor {
    /// Evaluates if the claim is a valid proof.
    pub fn evaluate_claim(
        claim: &Claim,
        id: &IdentityId,
        general_proof: &InvestorZKProofData,
    ) -> bool {
        match general_proof {
            InvestorZKProofData::V1(proof) => {
                v1::ValidProofOfInvestor::evaluate_claim(claim, id, proof)
            }

            InvestorZKProofData::V2(proof) => {
                v2::ValidProofOfInvestor::evaluate_claim(claim, id, proof)
            }
        }
    }
}
