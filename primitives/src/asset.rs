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

#![cfg_attr(not(feature = "std"), no_std)]

use crate::{IdentityId, Ticker};
use codec::{Decode, Encode};
use polymesh_primitives_derive::VecU8StrongTyped;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// The type of an asset represented by a token.
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum AssetType {
    EquityCommon,
    EquityPreferred,
    Commodity,
    FixedIncome,
    REIT,
    Fund,
    RevenueShareAgreement,
    StructuredProduct,
    Derivative,
    Custom(Vec<u8>),
}

impl Default for AssetType {
    fn default() -> Self {
        AssetType::Custom(b"undefined".to_vec())
    }
}

/// The type of an identifier associated with a token.
/// these are used off-chain.
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifierType {
    Cins,
    Cusip,
    Isin,
    Dti,
}

impl Default for IdentifierType {
    fn default() -> Self {
        IdentifierType::Isin
    }
}

/// Ownership status of a ticker/token.
#[allow(missing_docs)]
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

/// A wrapper for a token name.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct AssetName(pub Vec<u8>);

/// A wrapper for an asset ID.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct AssetIdentifier(pub Vec<u8>);

/// A wrapper for a funding round name.
#[derive(Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped)]
pub struct FundingRoundName(pub Vec<u8>);

impl Default for FundingRoundName {
    fn default() -> Self {
        FundingRoundName("".as_bytes().to_vec())
    }
}

/// struct to store the token details.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SecurityToken<U> {
    pub name: AssetName,
    pub total_supply: U,
    pub owner_did: IdentityId,
    pub divisible: bool,
    pub asset_type: AssetType,
    pub primary_issuance_agent: Option<IdentityId>,
}

/// struct to store the signed data.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SignData<U> {
    pub custodian_did: IdentityId,
    pub holder_did: IdentityId,
    pub ticker: Ticker,
    pub value: U,
    pub nonce: u16,
}

/// struct to store the ticker registration details.
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistration<U> {
    pub owner: IdentityId,
    pub expiry: Option<U>,
}

/// struct to store the ticker registration config.
#[allow(missing_docs)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistrationConfig<U> {
    pub max_ticker_length: u8,
    pub registration_length: Option<U>,
}

/// Enum that represents the current status of a ticker.
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug)]
pub enum TickerRegistrationStatus {
    RegisteredByOther,
    Available,
    RegisteredByDid,
}

/// Enum that uses as the return type for the restriction verification.
#[allow(missing_docs)]
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

/// The total asset balance and the balance of the asset in a specified portfolio of an identity.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FocusedBalances<Balance> {
    /// The total balance of the asset held by the identity.
    pub total: Balance,
    /// The balance of the asset in the default portfolio of the identity.
    pub portfolio: Balance,
}
