// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_io::hashing::blake2_128;
use sp_std::prelude::Vec;

use crate::impl_checked_inc;
use crate::ticker::Ticker;
use polymesh_primitives_derive::VecU8StrongTyped;

/// An unique asset identifier.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssetID([u8; 16]);

impl From<[u8; 16]> for AssetID {
    fn from(value: [u8; 16]) -> Self {
        AssetID(value)
    }
}

impl AssetID {
    /// Creates a new [`AssetID`] instance;
    pub fn new(value: [u8; 16]) -> Self {
        AssetID(value)
    }

    /// Converts [`AssetID`] type into a shared reference of bytes.
    pub fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Ticker> for AssetID {
    fn from(ticker: Ticker) -> AssetID {
        blake2_128(&(b"legacy_ticker", ticker).encode()).into()
    }
}

/// A per-asset checkpoint ID.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CheckpointId(pub u64);
impl_checked_inc!(CheckpointId);

/// A wrapper for a token name.
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetName(pub Vec<u8>);

/// The ID of a custom asset type.
#[derive(Encode, Decode, TypeInfo, Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct CustomAssetTypeId(pub u32);
impl_checked_inc!(CustomAssetTypeId);

/// The type of security represented by a token.
#[derive(Encode, Decode, TypeInfo, Copy, Clone, Debug, PartialEq, Eq)]
pub enum AssetType {
    /// Common stock - a security that represents ownership in a corporation.
    EquityCommon,
    /// Preferred stock. Preferred stockholders have a higher claim to dividends or asset
    /// distribution than common stockholders.
    EquityPreferred,
    /// Commodity - a basic good used in commerce that is interchangeable with other commodities of
    /// the same type.
    Commodity,
    /// Fixed income security - an investment that provides a return in the form of fixed periodic
    /// interest payments and the eventual return of principal at maturity. Examples: bonds,
    /// treasury bills, certificates of deposit.
    FixedIncome,
    /// Real estate investment trust - a company that owns, operates, or finances income-producing
    /// properties.
    REIT,
    /// Investment fund - a supply of capital belonging to numerous investors used to collectively
    /// purchase securities while each investor retains ownership and control of his own shares.
    Fund,
    /// Revenue share partnership agreement - a document signed by all partners in a partnership
    /// that has procedures when distributing business profits or losses.
    RevenueShareAgreement,
    /// Structured product, aka market-linked investment - a pre-packaged structured finance
    /// investment strategy based on a single security, a basket of securities, options, indices,
    /// commodities, debt issuance or foreign currencies, and to a lesser extent, derivatives.
    StructuredProduct,
    /// Derivative contract - a contract between two parties for buying or selling a security at a
    /// predetermined price within a specific time period. Examples: forwards, futures, options or
    /// swaps.
    Derivative,
    /// Anything else.
    Custom(CustomAssetTypeId),
    /// Stablecoins are cryptocurrencies designed to minimize the volatility of the price of the stablecoin,
    /// relative to some "stable" asset or basket of assets.
    /// A stablecoin can be pegged to a cryptocurrency, fiat money, or to exchange-traded commodities.
    StableCoin,
    /// Non-fungible token.
    NonFungible(NonFungibleType),
}

/// Defines all non-fungible variants.
#[derive(Encode, Decode, TypeInfo, Copy, Clone, Debug, PartialEq, Eq)]
pub enum NonFungibleType {
    /// Derivative contract - a contract between two parties for buying or selling a security at a
    /// predetermined price within a specific time period.
    /// Examples: forwards, futures, options or swaps.
    Derivative,
    /// Fixed income security - an investment that provides a return in the form of fixed periodic
    /// interest payments and the eventual return of principal at maturity.
    /// Examples: bonds, treasury bills, certificates of deposit.
    FixedIncome,
    /// Invoice - a list of goods sent or services provided, with a statement of the sum due for these.
    Invoice,
    /// The Id of a user definied type.
    Custom(CustomAssetTypeId),
}

impl Default for AssetType {
    fn default() -> Self {
        Self::EquityCommon
    }
}

impl AssetType {
    /// Returns true if the asset type is non-fungible.
    pub fn is_non_fungible(&self) -> bool {
        if let AssetType::NonFungible(_) = self {
            return true;
        }
        false
    }

    /// Returns true if the asset type is fungible.
    pub fn is_fungible(&self) -> bool {
        match self {
            AssetType::EquityCommon
            | AssetType::EquityPreferred
            | AssetType::Commodity
            | AssetType::FixedIncome
            | AssetType::REIT
            | AssetType::Fund
            | AssetType::RevenueShareAgreement
            | AssetType::StructuredProduct
            | AssetType::Derivative
            | AssetType::Custom(_)
            | AssetType::StableCoin => true,
            AssetType::NonFungible(_) => false,
        }
    }
}

/// A wrapper for a funding round name.
#[derive(Decode, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct FundingRoundName(pub Vec<u8>);
