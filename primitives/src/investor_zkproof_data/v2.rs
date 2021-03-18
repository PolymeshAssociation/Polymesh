use crate::{host_functions::native_rng, IdentityId, InvestorUid, Ticker};

use confidential_identity::{
    claim_proofs::Investor, CddClaimData, InvestorTrait as _, ScopeClaimData, ScopeClaimProof,
};

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct InvestorZKProofData(pub ScopeClaimProof);

impl InvestorZKProofData {
    pub fn new(did: &IdentityId, investor: &InvestorUid, ticker: &Ticker) -> Self {
        let cdd_claim = Self::make_cdd_claim(did, investor);
        let scope_claim = Self::make_scope_claim(ticker.as_bytes(), investor);

        let mut rng = native_rng::Rng::default();
        let proof = Investor::create_scope_claim_proof(&cdd_claim, &scope_claim, &mut rng);

        Self(proof)
    }

    /// Returns the CDD claim of the given `investor_did` and `investor_uid`.
    pub fn make_cdd_claim(
        investor_did: &IdentityId,
        investor_unique_id: &InvestorUid,
    ) -> CddClaimData {
        CddClaimData::new(&investor_did.to_bytes(), &investor_unique_id.to_bytes())
    }

    /// Returns the Scope claim of the given `ticker` and `investor_uid`.
    pub fn make_scope_claim(scope: &[u8], investor_unique_id: &InvestorUid) -> ScopeClaimData {
        ScopeClaimData::new(scope, &investor_unique_id.to_bytes())
    }
}
