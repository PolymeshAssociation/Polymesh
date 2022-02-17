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

use crate::{Claim, ClaimType, IdentityId, Scope, Ticker};
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
    // TODO: Add support for cross-ticker stats.
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
    pub claim: Option<Claim>,
}

impl From<Option<Claim>> for Stat2ndKey {
    fn from(claim: Option<Claim>) -> Stat2ndKey {
        Stat2ndKey { claim }
    }
}

impl From<Claim> for Stat2ndKey {
    fn from(claim: Claim) -> Stat2ndKey {
        Stat2ndKey { claim: Some(claim) }
    }
}

impl From<&Claim> for Stat2ndKey {
    fn from(claim: &Claim) -> Stat2ndKey {
        Stat2ndKey {
            claim: Some(claim.clone()),
        }
    }
}

// TODO: Maybe make a `ClaimStat` type, since all of the claims should have the same `Scope::Ticker(ticker)` value.
//   so using `Claim` in `Stat2ndKey` and `StatUpdate` would waste a lot of space.
//pub struct ClaimStat {
//  .. same variants as `Claim`, but without the `Scope` value.
//}
