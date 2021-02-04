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

use codec::{Decode, Encode};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_std::prelude::Vec;

/// A wrapper for a token name.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct AssetName(pub Vec<u8>);

/// The type of an asset represented by a token.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum AssetType {
    /// Common stock.
    EquityCommon,
    /// Preferred stock.
    EquityPreferred,
    /// Commodity.
    Commodity,
    /// Fixed income security, for example, a bond.
    FixedIncome,
    /// Real estate investment trust.
    REIT,
    /// Investment fund.
    Fund,
    /// Revenue share partnership agreement.
    RevenueShareAgreement,
    /// Structured product, aka market-linked investment.
    StructuredProduct,
    /// Derivative contract.
    Derivative,
    /// Anything else.
    Custom(Vec<u8>),
}

impl Default for AssetType {
    fn default() -> Self {
        Self::Custom(b"undefined".to_vec())
    }
}

/// A wrapper for a funding round name.
#[derive(
    Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default, VecU8StrongTyped,
)]
pub struct FundingRoundName(pub Vec<u8>);
