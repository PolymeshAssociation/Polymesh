use crate::{
    scalar_blake2_from_bytes, CddId, Claim, IdentityId, InvestorZKProofData, Scope, Ticker,
};
use cryptography::claim_proofs::ProofPublicKey;
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};

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
            Claim::InvestorUniqueness(Scope::Ticker(ticker), scope_id, cdd_id) => {
                let message = InvestorZKProofData::make_message(id, ticker);
                Self::verify_proof(cdd_id, id, scope_id, ticker, proof, &message)
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
        let investor = Scalar::from_bits(investor_raw.to_bytes());
        let scope_did = scalar_blake2_from_bytes(ticker.as_slice());

        if let Some(cdd_id) = CompressedRistretto::from_slice(cdd_id_raw.as_slice()).decompress() {
            if let Some(scope_id) =
                CompressedRistretto::from_slice(scope_id_raw.as_bytes()).decompress()
            {
                let verifier = ProofPublicKey::new(cdd_id, investor, scope_id, scope_did);
                return verifier.verify_id_match_proof(message.as_ref(), &proof.0);
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposition::{exists, Proposition};
    use crate::{Claim, Context, InvestorUid, InvestorZKProofData};
    use cryptography::claim_proofs::{compute_cdd_id, compute_scope_id};
    use sp_std::convert::{From, TryFrom};

    #[test]
    fn generate_and_validate_claim() {
        let investor_id = IdentityId::from(100);
        let investor_uid = InvestorUid::from(b"inv0".as_ref());
        let asset_ticker = Ticker::try_from(b"1".as_ref()).unwrap();

        let exists_affiliate_claim = Claim::Affiliate(Scope::Ticker(asset_ticker));
        let proposition = exists(&exists_affiliate_claim);

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
        assert_eq!(proposition.evaluate(&context), true);

        let proof: InvestorZKProofData =
            InvestorZKProofData::new(&investor_id, &investor_uid, &asset_ticker);
        let cdd_claim = InvestorZKProofData::make_cdd_claim(&investor_id, &investor_uid);
        let cdd_id = compute_cdd_id(&cdd_claim).compress().to_bytes().into();
        let scope_claim = InvestorZKProofData::make_scope_claim(&asset_ticker, &investor_uid);
        let scope_id = compute_scope_id(&scope_claim).compress().to_bytes().into();

        let claim = Claim::InvestorUniqueness(Scope::Ticker(asset_ticker), scope_id, cdd_id);

        assert!(ValidProofOfInvestor::evaluate_claim(
            &claim,
            &investor_id,
            &proof
        ));
    }
}
