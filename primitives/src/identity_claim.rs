use crate::{identity_id::IdentityId, Moment};
use codec::{Decode, Encode};
use sp_std::prelude::Vec;

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
    Jurisdiction(JurisdictionName),
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

/// A wrapper for Jurisdiction name.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct JurisdictionName(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for JurisdictionName {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        JurisdictionName(v)
    }
}

/// All information of a particular claim
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct IdentityClaim {
    /// Issuer of the claim
    pub claim_issuer: IdentityId,
    /// Issuance date
    pub issuance_date: Moment,
    /// Last updated date
    pub last_update_date: Moment,
    /// Expirty date
    pub expiry: Moment,
    /// Claim data
    pub claim: IdentityClaimData,
}

/// Information required to fetch a claim of a particular did. (Claim_data, claim_issuer)
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct ClaimIdentifier(pub IdentityClaimData, pub IdentityId);

impl From<IdentityClaimData> for IdentityClaim {
    fn from(data: IdentityClaimData) -> Self {
        IdentityClaim {
            claim: data,
            ..Default::default()
        }
    }
}
