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

use frame_support::dispatch::{DispatchError, DispatchResult};
use polymesh_primitives::{
    asset::{AssetName, AssetType, FundingRoundName, SecurityToken},
    calendar::CheckpointId,
    AssetIdentifier, IdentifiedOriginData, IdentityId, PortfolioId, ScopeId, Ticker,
};
use sp_std::prelude::Vec;

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

pub trait AssetFnTrait<Balance, Account, Origin> {
    /// Returns the total supply of the asset.
    fn total_supply(ticker: &Ticker) -> Balance;
    /// Returns the `ticker` asset balance of `did`.
    fn balance(ticker: &Ticker, did: IdentityId) -> Balance;
    /// Checks if `did` is the owner of `ticker`.
    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool;
    /// Gets the `ticker` asset balance of `did` at a particular checkpoint.
    fn get_balance_at(ticker: &Ticker, did: IdentityId, at: CheckpointId) -> Balance;
    /// Gets the PIA of `ticker` if it's assigned or else the owner of the token.
    fn primary_issuance_agent_or_owner(ticker: &Ticker) -> IdentityId;
    /// Transfer the `value` balance of the `ticker` asset from `from_portfolio` to `to_portfolio`.
    fn base_transfer(
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
    ) -> DispatchResult;
    /// Common functionality of both confidential and non-confidential asset creation.
    ///
    /// In case of success, returns the result of permission check of `origin`.
    fn base_create_asset(
        origin: Origin,
        name: AssetName,
        ticker: Ticker,
        total_supply: Balance,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> Result<IdentifiedOriginData<Account>, DispatchError>;
    /// Ensures that the caller has the required extrinsic and asset permissions.
    fn ensure_perms_owner_asset(
        origin: Origin,
        ticker: &Ticker,
    ) -> Result<IdentityId, DispatchError>;
    /// Creates a new non-confidential asset.
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
    /// Sets the total supply of an asset. Should only be called after carrying out the necessary
    /// precondition checks.
    fn unchecked_set_total_supply(
        did: IdentityId,
        ticker: Ticker,
        total_supply: Balance,
    ) -> DispatchResult;
    /// Returns the divisibility of the asset.
    fn is_divisible(ticker: Ticker) -> bool;
    /// Returns asset details from blockchain storage.
    fn token_details(ticker: &Ticker) -> SecurityToken<Balance>;
    /// Registers a ticker.
    fn register_ticker(origin: Origin, ticker: Ticker) -> DispatchResult;
    #[cfg(feature = "runtime-benchmarks")]
    /// Adds an artificial IU claim for benchmarks
    fn add_investor_uniqueness_claim(did: IdentityId, ticker: Ticker);
}
