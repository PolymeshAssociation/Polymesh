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

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_io::hashing::blake2_128;
use sp_std::prelude::Vec;

use polymesh_primitives_derive::VecU8StrongTyped;

use crate::settlement::InstructionId;
use crate::ticker::Ticker;
use crate::{impl_checked_inc, PortfolioId, PortfolioKind, PortfolioNumber};
use crate::{AccountId as AccountId32, IdentityId, Memo};

/// An unique asset identifier.
#[derive(Serialize, Deserialize)]
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssetId([u8; 16]);

impl From<[u8; 16]> for AssetId {
    fn from(mut value: [u8; 16]) -> Self {
        // Version 8.
        value[6] = (value[6] & 0x0f) | 0x80;
        // Standard RFC4122 variant (bits 10xx)
        value[8] = (value[8] & 0x3f) | 0x80;
        AssetId(value)
    }
}

impl AssetId {
    /// Creates a new [`AssetId`] instance;
    pub fn new(value: [u8; 16]) -> Self {
        value.into()
    }

    /// Creates an [`AssetId`] from raw bytes without RFC4122 bit normalization.
    pub fn from_raw(value: [u8; 16]) -> Self {
        Self(value)
    }

    /// Converts [`AssetId`] type into a shared reference of bytes.
    pub fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Converts [`AssetId`] into its raw bytes.
    pub fn to_bytes(&self) -> [u8; 16] {
        self.0
    }
}

impl From<Ticker> for AssetId {
    fn from(ticker: Ticker) -> AssetId {
        blake2_128(&(b"legacy_ticker", ticker).encode()).into()
    }
}

/// A per-asset checkpoint ID.
#[derive(Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo)]
#[derive(Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct CheckpointId(pub u64);
impl_checked_inc!(CheckpointId);

/// A wrapper for a token name.
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetName(pub Vec<u8>);

/// The ID of a custom asset type.
#[derive(Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct CustomAssetTypeId(pub u32);
impl_checked_inc!(CustomAssetTypeId);

/// The type of security represented by a token.
#[derive(Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
#[derive(Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct FundingRoundName(pub Vec<u8>);

/// Represents the holder of an asset, which can be either a portfolio or an account.
#[derive(Decode, Encode, MaxEncodedLen, Ord, PartialOrd, TypeInfo)]
#[derive(Clone, Debug, DecodeWithMemTracking, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub enum AssetHolder {
    /// The asset is held in a portfolio.
    Portfolio(PortfolioId),
    /// The asset is held by the key.
    Account(AccountId32),
}

impl From<PortfolioId> for AssetHolder {
    fn from(portfolio_id: PortfolioId) -> Self {
        AssetHolder::Portfolio(portfolio_id)
    }
}

impl TryFrom<(IdentityId, Vec<u8>, AssetHolderKind)> for AssetHolder {
    type Error = &'static str;

    fn try_from(
        (did, acc_owner, kind): (IdentityId, Vec<u8>, AssetHolderKind),
    ) -> Result<Self, Self::Error> {
        match kind {
            AssetHolderKind::Account => AssetHolder::try_from(acc_owner),
            AssetHolderKind::DefaultPortfolio => {
                Ok(AssetHolder::Portfolio(PortfolioId::default_portfolio(did)))
            }
            AssetHolderKind::UserPortfolio(number) => Ok(AssetHolder::Portfolio(
                PortfolioId::user_portfolio(did, number),
            )),
        }
    }
}

impl TryFrom<Vec<u8>> for AssetHolder {
    type Error = &'static str;

    fn try_from(encoded_account_id: Vec<u8>) -> Result<Self, Self::Error> {
        let account_id: [u8; 32] = encoded_account_id
            .try_into()
            .map_err(|_| "AccountId must be 32 bytes long")?;
        Ok(AssetHolder::Account(account_id.into()))
    }
}

/// The kind of holder, without the owner information.
///
/// Note: Used only for input parameters where the owner is the caller and can be retrieved from the origin.
#[derive(Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
pub enum AssetHolderKind {
    /// The asset is held by the key.
    Account,
    /// The asset is held in the default portfolio.
    #[default]
    DefaultPortfolio,
    /// The asset is held in a user-defined portfolio.
    UserPortfolio(PortfolioNumber),
}

impl From<AssetHolder> for AssetHolderKind {
    fn from(asset_holder: AssetHolder) -> Self {
        match asset_holder {
            AssetHolder::Portfolio(portfolio_id) => match portfolio_id.kind {
                PortfolioKind::Default => AssetHolderKind::DefaultPortfolio,
                PortfolioKind::User(number) => AssetHolderKind::UserPortfolio(number),
            },
            AssetHolder::Account(_) => AssetHolderKind::Account,
        }
    }
}

/// Reason for the holdings update.
#[derive(Decode, DecodeWithMemTracking, Encode, Eq, PartialEq, TypeInfo)]
#[derive(Clone, Debug)]
pub enum HoldingsUpdateReason {
    /// Tokens were issued.
    Issued {
        /// If the asset is fungible the [`FundingRoundName`] of the minted tokens.
        funding_round_name: Option<FundingRoundName>,
    },
    /// Tokens were redeemed.
    Redeemed,
    /// Tokens were transferred.
    Transferred {
        /// The [`InstructionId`] of the instruction which originated the transfer.
        instruction_id: Option<InstructionId>,
        /// The [`Memo`] of the instruction.
        instruction_memo: Option<Memo>,
    },
    /// Tokens were transferred via a controller call.
    ControllerTransfer,
}
