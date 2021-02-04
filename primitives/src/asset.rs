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

use crate::IdentityId;
use base64;
use frame_support::dispatch::DispatchError;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_std::prelude::Vec;

#[derive(
    Encode, Decode, Clone, Debug, PartialEq, Eq, VecU8StrongTyped, Default, PartialOrd, Ord,
)]
pub struct Base64Vec(pub Vec<u8>);

impl Base64Vec {
    pub fn decode(&self) -> Result<Vec<u8>, DecodeBase64Error> {
        base64::decode(&self.0[..]).map_err(|_| DecodeBase64Error)
    }

    pub fn new(inp: Vec<u8>) -> Self {
        Self::from(base64::encode(inp))
    }
}

/// Status of an Authorization after consume is called on it.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct DecodeBase64Error;

impl From<DecodeBase64Error> for DispatchError {
    fn from(_: DecodeBase64Error) -> DispatchError {
        DispatchError::Other("Authorization does not exist")
    }
}

/// A wrapper for a token name.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct AssetName(pub Vec<u8>);

/// The type of security represented by a token.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
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
    Custom(Vec<u8>),
}

impl Default for AssetType {
    fn default() -> Self {
        Self::Custom(b"undefined".to_vec())
    }
}

/// Ownership status of a ticker/token.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssetOwnershipRelation {
    NotOwned,
    TickerOwned,
    AssetOwned,
}

impl Default for AssetOwnershipRelation {
    fn default() -> Self {
        Self::NotOwned
    }
}

/// struct to store the token details.
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SecurityToken<U> {
    pub name: AssetName,
    pub total_supply: U,
    pub owner_did: IdentityId,
    pub divisible: bool,
    pub asset_type: AssetType,
    pub primary_issuance_agent: Option<IdentityId>,
}

/// struct to store the ticker registration details.
#[derive(Encode, Decode, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistration<U> {
    pub owner: IdentityId,
    pub expiry: Option<U>,
}

/// struct to store the ticker registration config.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistrationConfig<U> {
    pub max_ticker_length: u8,
    pub registration_length: Option<U>,
}

/// Enum that represents the current status of a ticker.
#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug)]
pub enum TickerRegistrationStatus {
    RegisteredByOther,
    Available,
    RegisteredByDid,
}

/// Enum that uses as the return type for the restriction verification.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RestrictionResult {
    Valid,
    Invalid,
    ForceValid,
}

impl Default for RestrictionResult {
    fn default() -> Self {
        RestrictionResult::Invalid
    }
}

pub enum TransactionError {
    /// 0-6 are used by substrate. Skipping them to avoid confusion
    ZeroTip = 0,
    /// Transaction needs an Identity associated to an account.
    MissingIdentity = 1,
    /// CDD is required
    CddRequired = 2,
    /// Invalid auth id
    InvalidAuthorization = 3,
}

/// A wrapper for a funding round name.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Default, VecU8StrongTyped,
)]
pub struct FundingRoundName(pub Vec<u8>);
