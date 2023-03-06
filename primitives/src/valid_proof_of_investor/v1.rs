use crate::{investor_zkproof_data::v1::InvestorZKProofData, CddId, Claim, IdentityId, Scope};
use confidential_identity_v1::{CompressedRistretto, ProofPublicKey};

// ZKProofs claims
// =========================================================

/// Evaluates if the claim is a valid proof.
pub fn evaluate_claim(
    scope: &Scope,
    claim: &Claim,
    id: &IdentityId,
    proof: &InvestorZKProofData,
) -> bool {
    match claim {
        Claim::InvestorUniqueness(_, scope_id, cdd_id) => {
            let message = InvestorZKProofData::make_message(id, scope.as_bytes());
            verify_proof(cdd_id, id, scope_id, scope, proof, &message)
        }
        _ => false,
    }
}

/// It double check that `proof` matches with the rest of the parameters.
fn verify_proof(
    cdd_id: &CddId,
    investor: &IdentityId,
    scope_id: &IdentityId,
    scope: &Scope,
    proof: &InvestorZKProofData,
    message: impl AsRef<[u8]>,
) -> bool {
    if let Some(cdd_id_point) = CompressedRistretto::from_slice(cdd_id.as_slice()).decompress() {
        if let Some(scope_id) = CompressedRistretto::from_slice(scope_id.as_bytes()).decompress() {
            let verifier = ProofPublicKey::new(
                cdd_id_point,
                &investor.to_bytes(),
                scope_id,
                scope.as_bytes(),
            );

            return verifier.verify_id_match_proof(message.as_ref(), &proof.0);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposition::{exists, Proposition};
    use crate::{
        investor_zkproof_data::v1::InvestorZKProofData, Claim, Context, InvestorUid, Ticker,
    };
    use confidential_identity_v1::compute_cdd_id;
    use sp_std::convert::From;

    #[test]
    fn generate_and_validate_claim() {
        let investor_id = IdentityId::from(100);
        let investor_uid = InvestorUid::from(b"inv0".as_ref());
        let asset_ticker = Ticker::from_slice_truncated(b"1".as_ref());

        let exists_affiliate_claim = Claim::Affiliate(Scope::Ticker(asset_ticker));
        let proposition = exists(&exists_affiliate_claim);

        let context = Context {
            claims: vec![].into_iter(),
            id: investor_id,
        };
        assert_eq!(proposition.evaluate(context), false);

        let context = Context {
            claims: vec![Claim::Affiliate(Scope::Ticker(asset_ticker))].into_iter(),
            id: investor_id,
        };
        assert_eq!(proposition.evaluate(context), true);

        let proof: InvestorZKProofData =
            InvestorZKProofData::new(&investor_id, &investor_uid, &asset_ticker);
        let cdd_claim = InvestorZKProofData::make_cdd_claim(&investor_id, &investor_uid);
        let cdd_id = compute_cdd_id(&cdd_claim).compress().to_bytes().into();
        let scope_id = InvestorZKProofData::make_scope_id(&asset_ticker.as_slice(), &investor_uid);

        let claim = Claim::InvestorUniqueness(Scope::Ticker(asset_ticker), scope_id, cdd_id);
        let scope = claim.as_scope().unwrap();

        assert!(evaluate_claim(scope, &claim, &investor_id, &proof));
    }
}
