use crate::{IdentityId, InvestorUid, Ticker};

use codec::{Decode, Encode};
use confidential_identity::ScopeClaimProof;
use schnorrkel::Signature;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

pub mod v1;
pub mod v2;

#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum InvestorZKProofData {
    V1(v1::InvestorZKProofData),
    V2(v2::InvestorZKProofData),
}

impl InvestorZKProofData {
    pub fn new(did: &IdentityId, investor: &InvestorUid, ticker: &Ticker) -> Self {
        Self::new_v2(did, investor, ticker)
    }

    pub fn new_v1(did: &IdentityId, investor: &InvestorUid, ticker: &Ticker) -> Self {
        let proof = v1::InvestorZKProofData::new(did, investor, ticker);
        Self::V1(proof)
    }

    pub fn new_v2(did: &IdentityId, investor: &InvestorUid, ticker: &Ticker) -> Self {
        let proof = v2::InvestorZKProofData::new(did, investor, ticker);
        Self::V2(proof)
    }
}

impl From<Signature> for InvestorZKProofData {
    fn from(proof: Signature) -> Self {
        Self::V1(v1::InvestorZKProofData(proof))
    }
}

impl From<v1::InvestorZKProofData> for InvestorZKProofData {
    fn from(proof: v1::InvestorZKProofData) -> Self {
        Self::V1(proof)
    }
}

impl From<ScopeClaimProof> for InvestorZKProofData {
    fn from(proof: ScopeClaimProof) -> Self {
        Self::V2(v2::InvestorZKProofData(proof))
    }
}

impl From<v2::InvestorZKProofData> for InvestorZKProofData {
    fn from(proof: v2::InvestorZKProofData) -> Self {
        Self::V2(proof)
    }
}
