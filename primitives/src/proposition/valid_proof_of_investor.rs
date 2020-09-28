use crate::{CddId, Claim, Context, IdentityId, InvestorZKProofData, Proposition, Ticker};
use cryptography::claim_proofs::ProofPublicKey;
use curve25519_dalek::ristretto::CompressedRistretto;

// ZKProofs claims
// =========================================================

/// Proposition that checks if any of its internal claims exists in context.
#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValidProofOfInvestorProposition {
    /// The Investor proof should be associated to this ticker.
    pub ticker: Ticker,
}

impl Proposition for ValidProofOfInvestorProposition {
    /// Evaluate proposition against `context`.
    fn evaluate(&self, context: &Context) -> bool {
        context
            .claims
            .iter()
            .any(|claim| self.evaluate_claim(claim, context))
    }
}

impl ValidProofOfInvestorProposition {
    /// Evaluates if the claim is a valid proof.
    fn evaluate_claim(&self, claim: &Claim, context: &Context) -> bool {
        match claim {
            Claim::InvestorZKProof(ref _ticker_scope, ref scope_id, ref cdd_id, ref proof) => {
                let message = InvestorZKProofData::make_message(&context.id, &self.ticker);
                Self::verify_proof(cdd_id, &context.id, scope_id, &self.ticker, proof, &message)
            }
            _ => false,
        }
    }

    /// It double check that `proof` matches with the rest of the parameters.
    fn verify_proof(
        cdd_id_raw: &CddId,
        investor_raw: &IdentityId,
        scope_id_raw: &IdentityId,
        ticker: &Ticker,
        proof: &InvestorZKProofData,
        message: impl AsRef<[u8]>,
    ) -> bool {
        if let Some(cdd_id) = CompressedRistretto::from_slice(cdd_id_raw.as_slice()).decompress() {
            if let Some(scope_id) =
                CompressedRistretto::from_slice(scope_id_raw.as_bytes()).decompress()
            {
                let verifier = ProofPublicKey::new(
                    cdd_id,
                    investor_raw.as_bytes(),
                    scope_id,
                    ticker.as_slice(),
                );
                return verifier.verify_id_match_proof(message.as_ref(), &proof.0);
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        proposition::{exists, has_valid_proof_of_investor},
        Claim, Context, InvestorUid, InvestorZKProofData, Scope,
    };
    use cryptography::claim_proofs::{compute_cdd_id, compute_scope_id};
    use sp_std::convert::{From, TryFrom};

    #[test]
    fn generate_and_validate_claim() {
        let investor_id = IdentityId::from(100);
        let investor_uid = InvestorUid::from(b"inv0".as_ref());
        let asset_ticker = Ticker::try_from(b"1".as_ref()).unwrap();

        let exists_affiliate_claim = Claim::Affiliate(Scope::Ticker(asset_ticker));
        let proposition =
            exists(&exists_affiliate_claim).and(has_valid_proof_of_investor(asset_ticker));

        let context = Context {
            claims: vec![],
            id: investor_id,
            primary_issuance_agent: None,
        };
        assert_eq!(proposition.evaluate(&context), false);

        let context = Context {
            claims: vec![Claim::Affiliate(Scope::Ticker(asset_ticker))],
            id: investor_id,
            primary_issuance_agent: None,
        };
        assert_eq!(proposition.evaluate(&context), false);

        let proof: InvestorZKProofData =
            InvestorZKProofData::new(&investor_id, &investor_uid, &asset_ticker);
        let cdd_claim = InvestorZKProofData::make_cdd_claim(&investor_id, &investor_uid);
        let cdd_id = compute_cdd_id(&cdd_claim).compress().to_bytes().into();
        let scope_claim = InvestorZKProofData::make_scope_claim(&asset_ticker, &investor_uid);
        let scope_id = compute_scope_id(&scope_claim).compress().to_bytes().into();

        let context = Context {
            claims: vec![
                Claim::Affiliate(Scope::Ticker(asset_ticker)),
                Claim::InvestorZKProof(Scope::Ticker(asset_ticker), scope_id, cdd_id, proof),
            ],
            id: investor_id,
            primary_issuance_agent: None,
        };
        assert_eq!(proposition.evaluate(&context), true);
    }
}
