use crate::{host_functions::native_rng, IdentityId, InvestorUid, Ticker};

use confidential_identity::{
    claim_proofs::{Investor, Provider},
    CddClaimData, InvestorTrait as _, ProviderTrait as _, ScopeClaimData, ScopeClaimProof,
};

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct InvestorZKProofData(pub ScopeClaimProof);

impl InvestorZKProofData {
    pub fn new(did: &IdentityId, investor: &InvestorUid, ticker: &Ticker) -> Self {
        let cdd_claim = CddClaimData::new(did.as_bytes(), investor.as_slice());
        let scope_claim = ScopeClaimData::new(ticker.as_bytes(), investor.as_slice());
        let cdd_id = Provider::create_cdd_id(&cdd_claim);

        let mut rng = native_rng::Rng::default();
        let proof = Investor::create_scope_claim_proof(&cdd_claim, &scope_claim, &mut rng);

        Self(proof)
    }
}
