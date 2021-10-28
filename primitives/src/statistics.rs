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

use crate::{ClaimType, Ticker};
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
#[derive(Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransferManagerResult {
    /// Transfer manager that was checked.
    pub tm: TransferManager,
    /// Final evaluation result.
    pub result: bool,
}

/// Wrapper around `sp_arithmetic::Permill`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
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
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug)]
pub enum AssetScope {
    /// Ticker scope.
    Ticker(Ticker),
    //TickerGroup(TickerGroupId),
    //Company(CompanyID),
}

/// Stats type.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug)]
pub enum StatType {
    /// Count(None) - Investor count stats (for maximum investor rule).
    /// Count(Some(ClaimType::Accredited)) - (non-)Accredited investor count
    ///    (To limit the number of non-accredited investors).
    /// Count(Some(ClaimType::Jurisdiction)) - Per-jurisdiction investor count.
    Count(Option<ClaimType>),
    /// Balance stat can be used for Percentage rules, since the `total_supply` of an asset can change (burn/mint)
    /// it is better to store a balance.
    /// Balance(None) - Total balance per investor.
    /// Balance(Some(ClaimType::Accredited)) - Total balance for Accredited or non-Accredited investors.
    /// Balance(Some(ClaimType::Jurisdiction)) - Per-jurisdiction balance (for per-jurisdiction max percentage rules).
    Balance(Option<ClaimType>),
}

impl StatType {
    /// Get the `ClaimType` from `StatType`.
    pub fn claim_type(&self) -> Option<ClaimType> {
        match self {
            Self::Count(claim_type) => *claim_type,
            Self::Balance(claim_type) => *claim_type,
        }
    }

    /// An issuer is needed if the `StatType` has a `ClaimType`.
    pub fn need_issuer(&self) -> bool {
        self.claim_type().is_some()
    }
}

// TODO: Maybe make a `ClaimStat` type, since all of the claims should have the same `Scope::Ticker(ticker)` value.
//   so using `Claim` in `Stat2ndKey` and `StatUpdate` would waste a lot of space.
//pub struct ClaimStat {
//  .. same variants as `Claim`, but without the `Scope` value.
//}
