// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate as polymesh_primitives;
use crate::{identity_id::IdentityId, CddId, Moment, Ticker};

use codec::{Decode, Encode};
use polymesh_primitives_derive::Migrate;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::From, prelude::*};

use super::jurisdiction::{CountryCode, JurisdictionName};
use crate::migrate::{Empty, Migrate};

/// It is the asset Id.
pub type ScopeId = IdentityId;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
/// Scope: Almost all claim needs a valid scope.
pub enum Scope {
    /// Scoped to an Identity
    Identity(IdentityId),
    /// Scoped to a `Ticker`.
    Ticker(Ticker),
    /// Scoped to arbitrary bytes
    Custom(Vec<u8>),
}

impl From<IdentityId> for Scope {
    fn from(did: IdentityId) -> Self {
        Self::Identity(did)
    }
}

impl From<Ticker> for Scope {
    fn from(ticker: Ticker) -> Self {
        Self::Ticker(ticker)
    }
}

impl From<Vec<u8>> for Scope {
    fn from(vec: Vec<u8>) -> Self {
        Self::Custom(vec)
    }
}

impl Migrate for ScopeOld {
    type Into = Scope;
    type Context = Empty;

    fn migrate(self, _: Self::Context) -> Option<Self::Into> {
        Some(Scope::Identity(self))
    }
}

/// Old country code
type CountryCodeOld = JurisdictionName;
/// Scope that was used in the last version
pub type ScopeOld = IdentityId;

impl From<Option<CddId>> for Empty {
    fn from(_: Option<CddId>) -> Self {
        Self
    }
}

/// All possible claims in polymesh
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Migrate)]
#[migrate_context(Option<CddId>)]
pub enum Claim {
    /// User is Accredited
    Accredited(#[migrate] Scope),
    /// User is Accredited
    Affiliate(#[migrate] Scope),
    /// User has an active BuyLockup (end date defined in claim expiry)
    BuyLockup(#[migrate] Scope),
    /// User has an active SellLockup (date defined in claim expiry)
    SellLockup(#[migrate] Scope),
    /// User has passed CDD
    #[migrate_from(#[allow(missing_docs)] CustomerDueDiligence)]
    CustomerDueDiligence(#[migrate_with(context?)] CddId),
    /// User is KYC'd
    KnowYourCustomer(#[migrate] Scope),
    /// This claim contains a string that represents the jurisdiction of the user
    Jurisdiction(#[migrate] CountryCode, #[migrate] Scope),
    /// User is exempted
    Exempted(#[migrate] Scope),
    /// User is Blocked
    Blocked(#[migrate] Scope),
    /// Confidential claim that will allow an investor to justify that it's identity can be
    /// a potential asset holder of given `scope`.
    ///
    /// This claim is mandatory to have for an investor. It will help issuer to apply compliance rules
    /// on the `ScopeId` instead of investor identityId as ScopeId is unique at investor entity level
    /// for a given scope (always be a Ticker).
    InvestorUniqueness(#[migrate] Scope, ScopeId, CddId),
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
            Claim::CustomerDueDiligence(..) => ClaimType::CustomerDueDiligence,
            Claim::KnowYourCustomer(..) => ClaimType::KnowYourCustomer,
            Claim::Jurisdiction(..) => ClaimType::Jurisdiction,
            Claim::Exempted(..) => ClaimType::Exempted,
            Claim::Blocked(..) => ClaimType::Blocked,
            Claim::InvestorUniqueness(..) => ClaimType::InvestorUniqueness,
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
            Claim::CustomerDueDiligence(..) => None,
            Claim::KnowYourCustomer(ref scope) => Some(scope),
            Claim::Jurisdiction(.., ref scope) => Some(scope),
            Claim::Exempted(ref scope) => Some(scope),
            Claim::Blocked(ref scope) => Some(scope),
            Claim::InvestorUniqueness(ref ticker_scope, ..) => Some(ticker_scope),
            Claim::NoData => None,
        }
    }

    /// It returns a CDD claim with a wildcard as CddId.
    pub fn make_cdd_wildcard() -> Claim {
        Claim::CustomerDueDiligence(CddId::default())
    }
}

/// Claim type represent the claim without its data.
///
/// # TODO
/// - Could we use `std::mem::Discriminant`?
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
    /// User is exempted
    Exempted,
    /// User is Blocked.
    Blocked,
    /// User identity can be bounded under a `ScopeId`.
    InvestorUniqueness,
    /// Empty type
    NoType,
}

impl Default for ClaimType {
    fn default() -> Self {
        ClaimType::NoType
    }
}

/// All information of a particular claim
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Migrate)]
#[migrate_context(Option<CddId>)]
pub struct IdentityClaim {
    /// Issuer of the claim
    pub claim_issuer: IdentityId,
    /// Issuance date
    pub issuance_date: Moment,
    /// Last updated date
    pub last_update_date: Moment,
    /// Expiry date
    pub expiry: Option<Moment>,
    /// Claim data
    #[migrate]
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
