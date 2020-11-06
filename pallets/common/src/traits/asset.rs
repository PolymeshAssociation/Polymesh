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
use frame_support::dispatch::{DispatchResult, DispatchResultWithPostInfo};
use polymesh_primitives::{
    calendar::CheckpointId, AssetIdentifier, IdentityId, PortfolioId, ScopeId, Ticker,
};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_std::prelude::*;

pub const GAS_LIMIT: u64 = 1_000_000_000;
/// This trait is used by the `identity` pallet to interact with the `pallet-asset`.
pub trait AssetSubTrait {
    /// Accept and process a ticker transfer
    ///
    /// # Arguments
    /// * `to_did` did of the receiver
    /// * `auth_id` Authorization id of the authorization created by current ticker owner
    fn accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult;
    /// Accept and process a primary issuance agent transfer
    ///
    /// # Arguments
    /// * `to_did` did of the receiver
    /// * `auth_id` Authorization id of the authorization created by current ticker owner
    fn accept_primary_issuance_agent_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult;
    /// Accept and process a token ownership transfer
    ///
    /// # Arguments
    /// * `to_did` did of the receiver
    /// * `auth_id` Authorization id of the authorization created by current token owner
    fn accept_asset_ownership_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult;

    /// Update balance of given IdentityId under the scopeId.
    ///
    /// # Arguments
    /// * `of` - The `ScopeId` of the given `IdentityId`.
    /// * `target_did` - The `IdentityId` whose balance needs to be updated.
    /// * `ticker`- Ticker of the asset whose count need to be updated for the given identity.
    fn update_balance_of_scope_id(of: ScopeId, whom: IdentityId, ticker: Ticker) -> DispatchResult;
}
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct IssueAssetItem<U> {
    pub investor_did: IdentityId,
    pub value: U,
}

/// A wrapper for a token name.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct AssetName(pub Vec<u8>);

/// The type of an asset represented by a token.
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

/// A wrapper for a funding round name.
#[derive(Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped)]
pub struct FundingRoundName(pub Vec<u8>);

impl Default for FundingRoundName {
    fn default() -> Self {
        FundingRoundName("".as_bytes().to_vec())
    }
}

pub trait Trait<Balance, Account, O> {
    fn total_supply(ticker: &Ticker) -> Balance;
    fn balance(ticker: &Ticker, did: IdentityId) -> Balance;
    fn _mint_from_sto(
        ticker: &Ticker,
        caller: Account,
        sender_did: IdentityId,
        assets_purchased: Balance,
    ) -> DispatchResult;
    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool;
    fn get_balance_at(ticker: &Ticker, did: IdentityId, at: CheckpointId) -> Balance;
    fn primary_issuance_agent_or_owner(ticker: &Ticker) -> IdentityId;
    fn primary_issuance_agent(ticker: &Ticker) -> Option<IdentityId>;
    fn max_number_of_tm_extension() -> u32;
    fn base_transfer(
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
    ) -> DispatchResultWithPostInfo;

    fn create_asset(
        origin: O,
        name: AssetName,
        ticker: Ticker,
        total_supply: Balance,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult;
}
