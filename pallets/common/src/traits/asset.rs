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
use frame_support::dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo};
use polymesh_primitives::{
    calendar::CheckpointId, AssetIdentifier, IdentityId, PortfolioId, ScopeId, Ticker,
};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_std::prelude::*;

pub const GAS_LIMIT: u64 = 13_000_000_000;

/// This trait is used by the `identity` pallet to interact with the `pallet-asset`.
pub trait AssetSubTrait<Balance> {
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

    /// Returns balance for a given scope id and target DID.
    ///
    /// # Arguments
    /// * `scope_id` - The `ScopeId` of the given `IdentityId`.
    /// * `target` - The `IdentityId` whose balance needs to be queried.
    fn balance_of_at_scope(scope_id: &ScopeId, target: &IdentityId) -> Balance;
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
        Self::Custom(b"undefined".to_vec())
    }
}

/// A wrapper for a funding round name.
#[derive(Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped)]
pub struct FundingRoundName(pub Vec<u8>);

impl Default for FundingRoundName {
    fn default() -> Self {
        Self(Vec::new())
    }
}

pub trait Trait<Balance, Account, Origin> {
    fn total_supply(ticker: &Ticker) -> Balance;
    fn balance(ticker: &Ticker, did: IdentityId) -> Balance;
    fn _mint_from_sto(
        ticker: &Ticker,
        caller: Account,
        sender_did: IdentityId,
        assets_purchased: Balance,
    ) -> DispatchResult;
    /// Check if an Identity is the owner of a ticker.
    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool;
    /// Get an Identity's balance of a token at a particular checkpoint.
    fn get_balance_at(ticker: &Ticker, did: IdentityId, at: CheckpointId) -> Balance;
    /// Get the PIA of a token if it's assigned or else the owner of the token.
    fn primary_issuance_agent_or_owner(ticker: &Ticker) -> IdentityId;
    /// Get the PIA of a token.
    fn primary_issuance_agent(ticker: &Ticker) -> Option<IdentityId>;
    /// Transfer an asset from one portfolio to another.
    fn base_transfer(
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
    ) -> DispatchResultWithPostInfo;
    /// Ensure that the caller has the required extrinsic and asset permissions.
    fn ensure_perms_owner_asset(
        origin: Origin,
        ticker: &Ticker,
    ) -> Result<IdentityId, DispatchError>;

    fn create_asset(
        origin: Origin,
        name: AssetName,
        ticker: Ticker,
        total_supply: Balance,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult;
}
