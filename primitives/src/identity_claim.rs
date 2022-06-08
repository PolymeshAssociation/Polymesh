// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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

use crate::{identity_id::IdentityId, CddId, Moment, Ticker};

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::From, prelude::*};

use super::jurisdiction::CountryCode;

/// It is the asset Id.
pub type ScopeId = IdentityId;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
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

impl Scope {
    /// Returns its inner content as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ticker(ticker) => ticker.as_slice(),
            Self::Identity(did) => did.as_bytes(),
            Self::Custom(data) => data.as_slice(),
        }
    }
}

/// All possible claims in polymesh
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Claim {
    /// User is Accredited.
    Accredited(Scope),
    /// User is an Affiliate.
    Affiliate(Scope),
    /// User has an active BuyLockup (end date defined in claim expiry).
    BuyLockup(Scope),
    /// User has an active SellLockup (date defined in claim expiry).
    SellLockup(Scope),
    /// User has passed CDD.
    CustomerDueDiligence(CddId),
    /// User is KYC'd.
    KnowYourCustomer(Scope),
    /// This claim contains a string that represents the jurisdiction of the user.
    Jurisdiction(CountryCode, Scope),
    /// User is exempted.
    Exempted(Scope),
    /// User is Blocked.
    Blocked(Scope),
    /// Confidential claim that will allow an investor to justify that it's identity can be
    /// a potential asset holder of given `scope`.
    ///
    /// All investors must have this claim (or a `InvestorUniquenessV2`), which will help the issuer apply compliance rules
    /// on the `ScopeId` instead of the investor's `IdentityId`, as `ScopeId` is unique at the
    /// investor entity level for a given scope (will always be a `Ticker`).
    InvestorUniqueness(Scope, ScopeId, CddId),
    /// Empty claim.
    NoData,
    /// Confidential claim using latest version from cryptography library.
    ///
    /// All investors must have this claim (or a `InvestorUniqueness`), which will help the issuer apply compliance rules
    /// on the `ScopeId` instead of the investor's `IdentityId`, as `ScopeId` is unique at the
    /// investor entity level for a given scope (will always be a `Ticker`).
    InvestorUniquenessV2(CddId),
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
            Claim::InvestorUniquenessV2(..) => ClaimType::InvestorUniquenessV2,
            Claim::NoData => ClaimType::NoType,
        }
    }

    /// The scope of this claim.
    pub fn as_scope(&self) -> Option<&Scope> {
        match self {
            Claim::Accredited(scope)
            | Claim::Affiliate(scope)
            | Claim::BuyLockup(scope)
            | Claim::SellLockup(scope)
            | Claim::KnowYourCustomer(scope)
            | Claim::Jurisdiction(.., scope)
            | Claim::Exempted(scope)
            | Claim::Blocked(scope)
            | Claim::InvestorUniqueness(scope, ..) => Some(scope),
            Claim::CustomerDueDiligence(..) | Claim::InvestorUniquenessV2(..) | Claim::NoData => {
                None
            }
        }
    }

    /// It returns a CDD claim with a default as CddId.
    pub fn default_cdd_id() -> Claim {
        Claim::CustomerDueDiligence(CddId::default())
    }
}

/// Claim type represent the claim without its data.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub enum ClaimType {
    /// User is Accredited.
    Accredited,
    /// User is an Affiliate.
    Affiliate,
    /// User has an active BuyLockup (end date defined in claim expiry).
    BuyLockup,
    /// User has an active SellLockup (date defined in claim expiry).
    SellLockup,
    /// User has passed CDD.
    CustomerDueDiligence,
    /// User is KYC'd.
    KnowYourCustomer,
    /// This claim contains a string that represents the jurisdiction of the user.
    Jurisdiction,
    /// User is exempted.
    Exempted,
    /// User is Blocked.
    Blocked,
    /// User identity can be bounded under a `ScopeId`.
    InvestorUniqueness,
    /// Empty type.
    NoType,
    /// New Investor uniqueness claim.
    InvestorUniquenessV2,
}

impl Default for ClaimType {
    fn default() -> Self {
        ClaimType::NoType
    }
}

/// All information of a particular claim
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, TypeInfo, Clone, Default, PartialEq, Eq)]
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
