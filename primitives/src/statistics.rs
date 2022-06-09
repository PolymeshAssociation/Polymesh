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

use crate::{Claim, ClaimType, CountryCode, IdentityId, Scope, Ticker};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{hash::Hash, hash::Hasher, ops::Deref, ops::DerefMut, prelude::*};

/// Transfer manager percentage
pub type Percentage = HashablePermill;

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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatOpType {
    /// Count - Investor count stats.
    Count,
    /// Balance - Balance stat can be used for Percentage rules, since the `total_supply` of an asset can change (burn/mint)
    Balance,
}

/// Stats type.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatType {
    /// Stats operation type.
    pub op: StatOpType,
    /// ClaimType and issuer for this stat type.
    pub claim_issuer: Option<(ClaimType, IdentityId)>,
}

impl StatType {
    /// Investor count.
    pub fn investor_count() -> Self {
        Self {
            op: StatOpType::Count,
            claim_issuer: None,
        }
    }
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
    /// Investor count.
    pub fn investor_count(ticker: Ticker) -> Self {
        Self {
            asset: ticker.into(),
            stat_type: StatType::investor_count(),
        }
    }

    /// Get claim scope from asset scope.
    pub fn claim_scope(&self) -> Scope {
        self.asset.claim_scope()
    }
}

/// Second stats key in double map.
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stat2ndKey {
    /// For `MaxInvestorCount` and `MaxInvestorOwnership` transfer rules.
    NoClaimStat,
    /// For per-Claim stats (Jurisdiction, Accredited, etc...).
    Claim(StatClaim),
}

impl Stat2ndKey {
    /// Create a `Stat2ndKey` from a `ClaimType` and optional `Claim`.
    pub fn new_from(claim_type: &ClaimType, claim: Option<Claim>) -> Self {
        match (claim_type, claim) {
            // No Accredited claim.
            (ClaimType::Accredited, None) => Self::Claim(StatClaim::Accredited(false)),
            // No Affiliate claim.
            (ClaimType::Affiliate, None) => Self::Claim(StatClaim::Affiliate(false)),
            // Has Accredited claim.
            (ClaimType::Accredited, Some(Claim::Accredited(..))) => {
                Self::Claim(StatClaim::Accredited(true))
            }
            // Has Affiliate claim.
            (ClaimType::Affiliate, Some(Claim::Affiliate(..))) => {
                Self::Claim(StatClaim::Affiliate(true))
            }
            // Has Jurisdiction claim.
            (ClaimType::Jurisdiction, Some(Claim::Jurisdiction(cc, _))) => {
                Self::Claim(StatClaim::Jurisdiction(Some(cc)))
            }
            // No Jurisdiction claim.
            (ClaimType::Jurisdiction, None) => Self::Claim(StatClaim::Jurisdiction(None)),
            // Unsupported claim type, just map it to `NoClaimStat` variant.
            _ => Self::NoClaimStat,
        }
    }
}

impl From<Option<Claim>> for Stat2ndKey {
    fn from(claim: Option<Claim>) -> Stat2ndKey {
        claim.and_then(|c| StatClaim::new_from(&c, true)).into()
    }
}

impl From<Claim> for Stat2ndKey {
    fn from(claim: Claim) -> Stat2ndKey {
        StatClaim::new_from(&claim, true).into()
    }
}

impl From<Option<StatClaim>> for Stat2ndKey {
    fn from(claim: Option<StatClaim>) -> Stat2ndKey {
        match claim {
            None => Stat2ndKey::NoClaimStat,
            Some(claim) => Stat2ndKey::Claim(claim),
        }
    }
}

impl From<&StatClaim> for Stat2ndKey {
    fn from(claim: &StatClaim) -> Stat2ndKey {
        Stat2ndKey::Claim(*claim)
    }
}

/// Stats supported claims.
///
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatClaim {
    /// User is Accredited or non-Accredited.
    Accredited(bool),
    /// User is an Affiliate or non-Affiliate.
    Affiliate(bool),
    /// This claim contains a string that represents the jurisdiction of the user.
    Jurisdiction(Option<CountryCode>),
}

impl StatClaim {
    /// Create a `StatClaim` from a `Claim`.
    pub fn new_from(claim: &Claim, has: bool) -> Option<Self> {
        match (claim, has) {
            (Claim::Accredited(..), has) => Some(StatClaim::Accredited(has)),
            (Claim::Affiliate(..), has) => Some(StatClaim::Affiliate(has)),
            (Claim::Jurisdiction(cc, _), true) => Some(StatClaim::Jurisdiction(Some(*cc))),
            (Claim::Jurisdiction(..), false) => Some(StatClaim::Jurisdiction(None)),
            _ => None,
        }
    }

    /// It returns the claim type.
    pub fn claim_type(&self) -> ClaimType {
        match self {
            StatClaim::Accredited(_) => ClaimType::Accredited,
            StatClaim::Affiliate(_) => ClaimType::Affiliate,
            StatClaim::Jurisdiction(..) => ClaimType::Jurisdiction,
        }
    }
}

/// Stats update.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatUpdate {
    /// Stat key to update.  (Claim or NoClaim)
    pub key2: Stat2ndKey,
    /// None - Remove stored value if any.
    pub value: Option<u128>,
}

pub(crate) mod v1 {
    use super::*;

    /// Transfer manager counter.
    pub type Counter = u64;

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
}
