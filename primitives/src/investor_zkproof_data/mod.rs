use crate::{IdentityId, InvestorUid, Ticker};

use confidential_identity_v2::ScopeClaimProof;
use schnorrkel::Signature;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::boxed::Box;

/// Investor ZKProof data using PIUS v1.
pub mod v1;

/// Investor ZKProof data using PIUS v2.
pub mod v2;

/// Manages ZKProofs generated with different versions of PIUS.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum InvestorZKProofData {
    /// Investor ZKProof generated using PIUS v1.
    V1(v1::InvestorZKProofData),
    /// Investor ZKProof generated using PIUS v2.
    V2(Box<v2::InvestorZKProofData>),
}

impl InvestorZKProofData {
    /// Generates a new ZKProof using PIUS v1.
    pub fn new_v1(did: &IdentityId, investor: &InvestorUid, ticker: &Ticker) -> Self {
        let proof = v1::InvestorZKProofData::new(did, investor, ticker);
        Self::V1(proof)
    }

    /// Generates a new ZKProof using PIUS v2.
    pub fn new_v2(did: &IdentityId, investor: &InvestorUid, ticker: &Ticker) -> Self {
        let proof = v2::InvestorZKProofData::new(did, investor, ticker);
        Self::V2(Box::new(proof))
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
        Self::V2(Box::new(v2::InvestorZKProofData(proof)))
    }
}

impl From<v2::InvestorZKProofData> for InvestorZKProofData {
    fn from(proof: v2::InvestorZKProofData) -> Self {
        Self::V2(Box::new(proof))
    }
}
