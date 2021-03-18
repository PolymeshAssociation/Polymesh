use crate::{investor_zkproof_data::v2::InvestorZKProofData, CddId, Claim, IdentityId};

use confidential_identity::{
    claim_proofs::{slice_to_scalar, Verifier},
    cryptography_core::cdd_claim::CddId as CryptoCddId,
    CompressedRistretto, Scalar, VerifierTrait as _,
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
            Claim::InvestorUniqueness(_, scope_did, cdd_id) => {
                Self::verify_proof(id, scope_did, cdd_id, proof)
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
        if let Some(cdd_id_point) = CompressedRistretto::from_slice(cdd_id.as_slice()).decompress()
        {
            if let Some(scope) = Scalar::from_canonical_bytes(scope.0.clone()) {
                let investor = slice_to_scalar(investor.as_bytes());
                let cdd_id = CryptoCddId(cdd_id_point);

                return Verifier::verify_scope_claim_proof(&proof.0, &investor, &scope, &cdd_id)
                    .is_ok();
            }
        }
        false
    }
}
