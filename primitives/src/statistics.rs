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

use crate::{Claim, ClaimType, CountryCode, IdentityId, Scope, Ticker};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{hash::Hash, hash::Hasher, ops::Deref, ops::DerefMut, prelude::*};

/// Transfer manager counter
pub type Counter = u64;
/// Transfer manager percentage
pub type Percentage = HashablePermill;

/// Transfer managers that can be attached to a Token for compliance.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransferManager {
    /// CTM limits the number of active investors in a Token.
    CountTransferManager(Counter),
    /// PTM limits the percentage of token owned by a single Identity.
    PercentageTransferManager(Percentage),
}

/// Result of a transfer manager check.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransferManagerResult {
    /// Transfer manager that was checked.
    pub tm: TransferManager,
    /// Final evaluation result.
    pub result: bool,
}

/// Wrapper around `sp_arithmetic::Permill`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct HashablePermill(pub sp_arithmetic::Permill);

impl Hash for HashablePermill {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.0.deconstruct().to_le_bytes())
    }
}

impl Deref for HashablePermill {
    type Target = sp_arithmetic::Permill;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HashablePermill {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Asset scope for stats.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AssetScope {
    /// Ticker scope.  Used for per-ticker stats.
    Ticker(Ticker),
    // TODO: Add support for cross-ticker stats.  Support needs to be
    // added to the Assets pallet first.
    //TickerGroup(TickerGroupId),
    //Company(CompanyId),
}

impl From<Ticker> for AssetScope {
    fn from(ticker: Ticker) -> AssetScope {
        AssetScope::Ticker(ticker)
    }
}

impl From<AssetScope> for Scope {
    fn from(asset: AssetScope) -> Scope {
        match asset {
            AssetScope::Ticker(ticker) => Scope::Ticker(ticker),
        }
    }
}

impl AssetScope {
    /// Get claim scope from asset scope.
    pub fn claim_scope(&self) -> Scope {
        match self {
            AssetScope::Ticker(ticker) => Scope::Ticker(*ticker),
        }
    }
}

/// Stats Operation type.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StatOpType {
    /// Count - Investor count stats.
    Count,
    /// Balance - Balance stat can be used for Percentage rules, since the `total_supply` of an asset can change (burn/mint)
    Balance,
}

/// Stats type.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StatType {
    /// Stats operation type.
    pub op: StatOpType,
    /// ClaimType and issuer for this stat type.
    pub claim_issuer: Option<(ClaimType, IdentityId)>,
}

/// First stats key in double map.
#[derive(Decode, Encode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Stat1stKey {
    /// Asset scope.
    pub asset: AssetScope,
    /// Stat type.
    pub stat_type: StatType,
}

impl Stat1stKey {
    /// Get claim scope from asset scope.
    pub fn claim_scope(&self) -> Scope {
        self.asset.claim_scope()
    }
}

/// Second stats key in double map.
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stat2ndKey {
    /// For per-Claim stats (Jurisdiction, Accredited, etc...).
    /// Non-Accredited stats would be stored with a `None` here.
    pub claim: Option<StatClaim>,
}

impl Stat2ndKey {
    /// Check if the claim matches the key's claim.
    pub fn match_claim(&self, claim: &Option<Claim>) -> bool {
        match (self.claim, claim) {
            (None, None) => true,
            (Some(claim1), Some(claim2)) => claim1.match_claim(claim2),
            _ => false,
        }
    }
}

impl From<Option<Claim>> for Stat2ndKey {
    fn from(claim: Option<Claim>) -> Stat2ndKey {
        let claim = claim.and_then(|c| StatClaim::new_from(&c));
        Stat2ndKey { claim }
    }
}

impl From<Claim> for Stat2ndKey {
    fn from(claim: Claim) -> Stat2ndKey {
        Stat2ndKey { claim: StatClaim::new_from(&claim) }
    }
}

impl From<Option<StatClaim>> for Stat2ndKey {
    fn from(claim: Option<StatClaim>) -> Stat2ndKey {
        Stat2ndKey { claim }
    }
}

impl From<&StatClaim> for Stat2ndKey {
    fn from(claim: &StatClaim) -> Stat2ndKey {
        Stat2ndKey {
            claim: Some(*claim),
        }
    }
}

/// Stats supported claims.
///
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StatClaim {
    /// User is Accredited.
    Accredited,
    /// User is an Affiliate.
    Affiliate,
    /// User has an active BuyLockup (end date defined in claim expiry).
    BuyLockup,
    /// User has an active SellLockup (date defined in claim expiry).
    SellLockup,
    /// User is KYC'd.
    KnowYourCustomer,
    /// This claim contains a string that represents the jurisdiction of the user.
    Jurisdiction(CountryCode),
    /// User is Exempted.
    Exempted,
    /// User is Blocked.
    Blocked,
}

impl StatClaim {
    /// Create a `StatClaim` from a `Claim`.
    pub fn new_from(claim: &Claim) -> Option<Self> {
        match claim {
            Claim::Accredited(..) => Some(StatClaim::Accredited),
            Claim::Affiliate(..) => Some(StatClaim::Affiliate),
            Claim::BuyLockup(..) => Some(StatClaim::BuyLockup),
            Claim::SellLockup(..) => Some(StatClaim::SellLockup),
            Claim::KnowYourCustomer(..) => Some(StatClaim::KnowYourCustomer),
            Claim::Jurisdiction(cc, _) => Some(StatClaim::Jurisdiction(*cc)),
            Claim::Exempted(..) => Some(StatClaim::Exempted),
            Claim::Blocked(..) => Some(StatClaim::Blocked),
            _ => None,
        }
    }

    /// It returns the claim type.
    pub fn claim_type(&self) -> ClaimType {
        match self {
            StatClaim::Accredited => ClaimType::Accredited,
            StatClaim::Affiliate => ClaimType::Affiliate,
            StatClaim::BuyLockup => ClaimType::BuyLockup,
            StatClaim::SellLockup => ClaimType::SellLockup,
            StatClaim::KnowYourCustomer => ClaimType::KnowYourCustomer,
            StatClaim::Jurisdiction(..) => ClaimType::Jurisdiction,
            StatClaim::Exempted => ClaimType::Exempted,
            StatClaim::Blocked => ClaimType::Blocked,
        }
    }

    /// Check if the claim has the same type
    pub fn match_claim(&self, claim: &Claim) -> bool {
        match (self, claim) {
            (StatClaim::Accredited, Claim::Accredited(..)) => true,
            (StatClaim::Affiliate, Claim::Affiliate(..)) => true,
            (StatClaim::BuyLockup, Claim::BuyLockup(..)) => true,
            (StatClaim::SellLockup, Claim::SellLockup(..)) => true,
            (StatClaim::KnowYourCustomer, Claim::KnowYourCustomer(..)) => true,
            (StatClaim::Jurisdiction(cc1), Claim::Jurisdiction(cc2, _)) => cc1 == cc2,
            (StatClaim::Exempted, Claim::Exempted(..)) => true,
            (StatClaim::Blocked, Claim::Blocked(..)) => true,
            _ => true,
        }
    }
}
