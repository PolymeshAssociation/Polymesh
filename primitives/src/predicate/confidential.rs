use crate::{Claim, Context, Predicate};
use cryptography::claim_proofs::ProofPublicKey;

// ZKProofs claims
// =========================================================

/// Predicate that checks if any of its internal claims exists in context.
#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValidProofOfInvestor {}

impl Predicate for ValidProofOfInvestor {
    /// Evaluate predicate against `context`.
    fn evaluate(&self, context: &Context) -> bool {
        match context.claims.first() {
            Some(ref claim) => Self::evaluate_claim(claim, context),
            _ => false,
        }
    }
}

impl ValidProofOfInvestor {
    fn evaluate_claim(claim: &Claim, context: &Context) -> bool {
        match claim {
            Claim::ConfidentialScopeClaim(ref asset_scope_id, ref cdd_id, ref proof) => {
                let investor = &context.source;
                let ticker = &context.ticker;
                let message = cdd_id
                    .to_slice()
                    .iter()
                    .chain(investor.iter())
                    .chain(asset_scope_id.iter())
                    .chain(ticker.iter())
                    .collect::<Vec<_>>();

                // Verify for a valid proof.
                let verifier = ProofPublicKey::new(cdd_id, investor, asset_scope_id, ticker);
                verifier.verify_id_match_proof(message, proof)
            }
            _ => false,
        }
    }
}
