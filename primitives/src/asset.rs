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
use codec::{Decode, Encode};
use frame_support::dispatch::DispatchError;
use polymesh_primitives_derive::VecU8StrongTyped;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::Vec;

/// A wrapper for a base-64 encoded vector of bytes.
#[derive(
    Encode, Decode, Clone, Debug, PartialEq, Eq, VecU8StrongTyped, Default, PartialOrd, Ord,
)]
pub struct Base64Vec(pub Vec<u8>);

impl Base64Vec {
    /// Decodes a Base64-encoded vector of bytes.
    ///
    /// ## Errors
    /// - `DecodeBase64Error` if `self` is not Base64-encoded.
    pub fn decode(&self) -> Result<Vec<u8>, DispatchError> {
        base64::decode(&self.0[..]).map_err(|_| DecodeBase64Error.into())
    }

    /// Creates a new Base64-encoded object by encoding a byte vector `inp`.
    pub fn new(inp: Vec<u8>) -> Self {
        Self::from(base64::encode(inp))
    }
}

/// The error type for `Base64Vec`.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct DecodeBase64Error;

impl From<DecodeBase64Error> for DispatchError {
    fn from(_: DecodeBase64Error) -> DispatchError {
        // TODO: why does this error message look unrelated?
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
    /// The ticker has no owner.
    NotOwned,
    /// The ticker has an owner but its asset has no owner.
    TickerOwned,
    /// Both the ticker and the asset have owners.
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
    /// Name of the asset.
    pub name: AssetName,
    /// Total supply of the asset.
    pub total_supply: U,
    /// Asset's owner DID.
    pub owner_did: IdentityId,
    /// Whether the asset is divisible.
    pub divisible: bool,
    /// Type of asset.
    pub asset_type: AssetType,
    /// Asset's primary issuance agent.
    pub primary_issuance_agent: Option<IdentityId>,
}

/// struct to store the ticker registration details.
#[derive(Encode, Decode, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistration<U> {
    /// Ticker's owner.
    pub owner: IdentityId,
    /// Ownership expiry in units of `U`.
    pub expiry: Option<U>,
}

/// struct to store the ticker registration config.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistrationConfig<U> {
    /// Maximum length of the ticker.
    pub max_ticker_length: u8,
    /// Maximum duration of the registration procedure.
    pub registration_length: Option<U>,
}

/// Enum that represents the current status of a ticker.
#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug)]
pub enum TickerRegistrationStatus {
    /// Registered by another party. Cannot be used by the calling party.
    RegisteredByOther,
    /// Not registered by any party. Can be used by the calling party.
    Available,
    /// Registered by the calling party. Can be used by the calling party.
    RegisteredByDid,
}

/// Enum that uses as the return type for the restriction verification.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RestrictionResult {
    /// Restriction is valid.
    Valid,
    /// Restriction is invalid.
    Invalid,
    /// Restriction is valid by forcing.
    ///
    /// FIXME: this variant is not used. Clean it up?
    ForceValid,
}

impl Default for RestrictionResult {
    fn default() -> Self {
        RestrictionResult::Invalid
    }
}

///
pub enum TransactionError {
    /// The tip should be above zero.
    ZeroTip = 0,
    /// Transaction needs an Identity associated to an account.
    MissingIdentity = 1,
    /// CDD is required.
    CddRequired = 2,
    /// Invalid auth id.
    InvalidAuthorization = 3,
}

/// A wrapper for a funding round name.
#[derive(
    Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default, VecU8StrongTyped,
)]
pub struct FundingRoundName(pub Vec<u8>);
