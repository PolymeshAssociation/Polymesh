use crate::{investor_zkproof_data::v2::InvestorZKProofData, CddId, Claim, IdentityId, Scope};

use confidential_identity::{
    claim_proofs::{slice_to_ristretto_point, slice_to_scalar, Verifier},
    cryptography_core::cdd_claim::CddId as CryptoCddId,
    VerifierTrait as _,
};

// ZKProofs claims
// =========================================================

/// Data structure used to check if any of its internal claims exist in context.
#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValidProofOfInvestor;

impl ValidProofOfInvestor {
    /// Evaluates if the claim is a valid proof.
    pub fn evaluate_claim(claim: &Claim, id: &IdentityId, proof: &InvestorZKProofData) -> bool {
        match claim {
            Claim::InvestorUniqueness(scope, _, cdd_id) => {
                Self::verify_proof(id, scope, cdd_id, proof)
            }
            _ => false,
        }
    }

    fn verify_proof(
        investor: &IdentityId,
        scope: &Scope,
        cdd_id: &CddId,
        proof: &InvestorZKProofData,
    ) -> bool {
        let investor = slice_to_scalar(investor.as_bytes());
        let scope = slice_to_scalar(scope.as_bytes());
        let cdd_id = CryptoCddId(slice_to_ristretto_point(cdd_id.as_slice()));

        Verifier::verify_scope_claim_proof(&proof.0, &investor, &scope, &cdd_id).is_ok()
    }
}
