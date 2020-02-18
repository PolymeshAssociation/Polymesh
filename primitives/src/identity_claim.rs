use crate::{identity_id::IdentityId, Moment};
use codec::{Decode, Encode};

/// All possible claims in polymesh
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum IdentityClaimData {
    /// User is Accredited
    Accredited,
    /// User is Accredited
    Affiliate,
    /// User has an active BuyLockup (end date defined in claim expiry)
    BuyLockup,
    /// User has an active SellLockup (date defined in claim expiry)
    SellLockup,
    /// User has passed CDD
    CustomerDueDiligence,
    /// User is KYC'd
    KnowYourCustomer,
    /// This claim contains a string that represents the jurisdiction of the user
    Jurisdiction(Vec<u8>),
    /// User is whitelisted
    Whitelisted,
    /// Empty claim
    NoData,
}

impl Default for IdentityClaimData {
    fn default() -> Self {
        IdentityClaimData::NoData
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct IdentityClaim {
    pub claim_issuer: IdentityId,
    pub issuance_date: Moment,
    pub last_update_date: Moment,
    pub expiry: Moment,
    pub claim: IdentityClaimData,
}
