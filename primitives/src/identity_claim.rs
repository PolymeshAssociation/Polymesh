use crate::{identity_id::IdentityId, Moment};
use codec::{Decode, Encode};
use sp_std::prelude::*;

/// Scope: Almost all claim needs a valid scope identity.
pub type Scope = IdentityId;

/// All possible claims in polymesh
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Claim {
    /// User is Accredited
    Accredited(Scope),
    /// User is Accredited
    Affiliate(Scope),
    /// User has an active BuyLockup (end date defined in claim expiry)
    BuyLockup(Scope),
    /// User has an active SellLockup (date defined in claim expiry)
    SellLockup(Scope),
    /// User has passed CDD
    CustomerDueDiligence,
    /// User is KYC'd
    KnowYourCustomer(Scope),
    /// This claim contains a string that represents the jurisdiction of the user
    Jurisdiction(JurisdictionName, Scope),
    /// User is whitelisted
    Whitelisted(Scope),
    /// User is Blacklisted
    BlackListed(Scope),
    /// Empty claim
    NoData,
}

impl Default for Claim {
    fn default() -> Self {
        Claim::NoData
    }
}

impl Claim {
    /// It returns the claim type.
    pub fn claim_type(&self) -> ClaimType {
        match self {
            Claim::Accredited(..) => ClaimType::Accredited,
            Claim::Affiliate(..) => ClaimType::Affiliate,
            Claim::BuyLockup(..) => ClaimType::BuyLockup,
            Claim::SellLockup(..) => ClaimType::SellLockup,
            Claim::CustomerDueDiligence => ClaimType::CustomerDueDiligence,
            Claim::KnowYourCustomer(..) => ClaimType::KnowYourCustomer,
            Claim::Jurisdiction(..) => ClaimType::Jurisdiction,
            Claim::Whitelisted(..) => ClaimType::Whitelisted,
            Claim::BlackListed(..) => ClaimType::BlackListed,
            Claim::NoData => ClaimType::NoType,
        }
    }

    /// The scope of this claim.
    pub fn as_scope(&self) -> Option<&Scope> {
        match self {
            Claim::Accredited(ref scope) => Some(scope),
            Claim::Affiliate(ref scope) => Some(scope),
            Claim::BuyLockup(ref scope) => Some(scope),
            Claim::SellLockup(ref scope) => Some(scope),
            Claim::CustomerDueDiligence => None,
            Claim::KnowYourCustomer(ref scope) => Some(scope),
            Claim::Jurisdiction(.., ref scope) => Some(scope),
            Claim::Whitelisted(ref scope) => Some(scope),
            Claim::BlackListed(ref scope) => Some(scope),
            Claim::NoData => None,
        }
    }
}

/// Claim type represent the claim without its data.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum ClaimType {
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
    Jurisdiction,
    /// User is whitelisted
    Whitelisted,
    /// User is BlackListed.
    BlackListed,
    /// Empty type
    NoType,
}

impl Default for ClaimType {
    fn default() -> Self {
        ClaimType::NoType
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
    pub expiry: Option<Moment>,
    /// Claim data
    pub claim: Claim,
}

impl From<Claim> for IdentityClaim {
    fn from(data: Claim) -> Self {
        IdentityClaim {
            claim: data,
            ..Default::default()
        }
    }
}
