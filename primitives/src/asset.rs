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

use crate::compliance_manager::AssetComplianceResult;
use crate::identity_id::PortfolioValidityResult;
use crate::impl_checked_inc;
use crate::transfer_compliance::TransferConditionResult;
use codec::{Decode, Encode};
use polymesh_primitives_derive::VecU8StrongTyped;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::Vec;

use crate::{Balance, IdentityId};

// TODO: Remove base64 hack.
use base64;
use frame_support::dispatch::DispatchError;
/// A wrapper for a base-64 encoded vector of bytes.
#[derive(
    Encode,
    Decode,
    TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
    VecU8StrongTyped,
    Default,
    PartialOrd,
    Ord
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
}

impl Default for AssetType {
    fn default() -> Self {
        Self::EquityCommon
    }
}

/// A wrapper for a funding round name.
#[derive(Decode, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct FundingRoundName(pub Vec<u8>);

/// struct to store the token details.
#[derive(Encode, Decode, TypeInfo, Default, Clone, PartialEq, Debug)]
pub struct SecurityToken {
    /// Total supply of the asset.
    pub total_supply: Balance,
    /// Asset's owner DID.
    pub owner_did: IdentityId,
    /// Whether the asset is divisible.
    pub divisible: bool,
    /// Type of the asset.
    pub asset_type: AssetType,
}

/// Result of a granular can transfer.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Decode, Encode, Clone, PartialEq, Eq)]
pub struct GranularCanTransferResult {
    /// Granularity check failed.
    pub invalid_granularity: bool,
    /// Receiver is equal to sender.
    pub self_transfer: bool,
    /// Receiver is missing cdd.
    pub invalid_receiver_cdd: bool,
    /// Sender is missing cdd.
    pub invalid_sender_cdd: bool,
    /// Scope claim is missing.
    pub missing_scope_claim: bool,
    /// Receiver had a custodian error.
    pub receiver_custodian_error: bool,
    /// Sender had a custodian error.
    pub sender_custodian_error: bool,
    /// Sender had an insufficient balance.
    pub sender_insufficient_balance: bool,
    /// Portfolio validity result.
    pub portfolio_validity_result: PortfolioValidityResult,
    /// Asset is frozen.
    pub asset_frozen: bool,
    /// Result of transfer condition check.
    pub transfer_condition_result: Vec<TransferConditionResult>,
    /// Result of compliance check.
    pub compliance_result: AssetComplianceResult,
    /// Final evaluation result.
    pub result: bool,
}

impl From<v1::GranularCanTransferResult> for GranularCanTransferResult {
    fn from(old: v1::GranularCanTransferResult) -> Self {
        Self {
            invalid_granularity: old.invalid_granularity,
            self_transfer: old.self_transfer,
            invalid_receiver_cdd: old.invalid_receiver_cdd,
            invalid_sender_cdd: old.invalid_sender_cdd,
            missing_scope_claim: old.missing_scope_claim,
            receiver_custodian_error: old.receiver_custodian_error,
            sender_custodian_error: old.sender_custodian_error,
            sender_insufficient_balance: old.sender_insufficient_balance,
            portfolio_validity_result: old.portfolio_validity_result,
            asset_frozen: old.asset_frozen,
            transfer_condition_result: old
                .statistics_result
                .into_iter()
                .map(|tm| tm.into())
                .collect(),
            compliance_result: old.compliance_result,
            result: old.result,
        }
    }
}

/// Deprecated v1 GranularCanTransferResult.
pub mod v1 {
    use super::*;
    use crate::statistics::v1::TransferManagerResult;

    /// Result of a granular can transfer.
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
    #[derive(Decode, Encode, Clone, PartialEq, Eq)]
    pub struct GranularCanTransferResult {
        /// Granularity check failed.
        pub invalid_granularity: bool,
        /// Receiver is equal to sender.
        pub self_transfer: bool,
        /// Receiver is missing cdd.
        pub invalid_receiver_cdd: bool,
        /// Sender is missing cdd.
        pub invalid_sender_cdd: bool,
        /// Scope claim is missing.
        pub missing_scope_claim: bool,
        /// Receiver had a custodian error.
        pub receiver_custodian_error: bool,
        /// Sender had a custodian error.
        pub sender_custodian_error: bool,
        /// Sender had an insufficient balance.
        pub sender_insufficient_balance: bool,
        /// Portfolio validity result.
        pub portfolio_validity_result: PortfolioValidityResult,
        /// Asset is frozen.
        pub asset_frozen: bool,
        /// Result of statistics check.
        pub statistics_result: Vec<TransferManagerResult>,
        /// Result of compliance check.
        pub compliance_result: AssetComplianceResult,
        /// Final evaluation result.
        pub result: bool,
    }
}
