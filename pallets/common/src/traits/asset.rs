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
    asset::{AssetName, AssetType, FundingRoundName},
    AssetIdentifier, IdentityId, ScopeId, Ticker,
};
use sp_std::prelude::Vec;

/// This trait is used by the `identity` pallet to interact with the `pallet-asset`.
pub trait AssetSubTrait<Balance> {
    /// Accept and process a ticker transfer
    ///
    /// # Arguments
    /// * `to` did of the receiver.
    /// * `from` sender of the authorization.
    /// * `ticker` that is being transferred.
    fn accept_ticker_transfer(to: IdentityId, from: IdentityId, ticker: Ticker) -> DispatchResult;

    /// Accept and process a primary issuance agent transfer
    ///
    /// # Arguments
    /// * `to` did of the receiver.
    /// * `from` sender of the authorization.
    /// * `ticker` that is being altered.
    fn accept_primary_issuance_agent_transfer(
        to: IdentityId,
        from: IdentityId,
        ticker: Ticker,
    ) -> DispatchResult;

    /// Accept and process a token ownership transfer
    ///
    /// # Arguments
    /// * `to` did of the receiver.
    /// * `from` sender of the authorization.
    /// * `ticker` that is being transferred.
    fn accept_asset_ownership_transfer(
        to: IdentityId,
        from: IdentityId,
        ticker: Ticker,
    ) -> DispatchResult;

    /// Update the `ticker` balance of `target_did` under `scope_id`. Clean up the balances related
    /// to any previous valid `old_scope_ids`.
    ///
    /// # Arguments
    /// * `scope_id` - The new `ScopeId` of `target_did` and `ticker`.
    /// * `target_did` - The `IdentityId` whose balance needs to be updated.
    /// * `ticker`- Ticker of the asset whose count need to be updated for the given identity.
    fn update_balance_of_scope_id(scope_id: ScopeId, target_did: IdentityId, ticker: Ticker);

    /// Returns balance for a given scope id and target DID.
    ///
    /// # Arguments
    /// * `scope_id` - The `ScopeId` of the given `IdentityId`.
    /// * `target` - The `IdentityId` whose balance needs to be queried.
    fn balance_of_at_scope(scope_id: &ScopeId, target: &IdentityId) -> Balance;

    /// Returns the `ScopeId` for a given `ticker` and `did`.
    fn scope_id_of(ticker: &Ticker, did: &IdentityId) -> ScopeId;
}

pub trait AssetFnTrait<Balance, Account, Origin> {
    fn balance(ticker: &Ticker, did: IdentityId) -> Balance;
    /// Get the PIA of a token if it's assigned or else the owner of the token.
    fn primary_issuance_agent_or_owner(ticker: &Ticker) -> IdentityId;
    /// Ensure that the caller has the required extrinsic and asset permissions.
    fn ensure_owner_perms(origin: Origin, ticker: &Ticker) -> Result<IdentityId, DispatchError>;

    fn create_asset(
        origin: Origin,
        name: AssetName,
        ticker: Ticker,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult;

    fn create_asset_and_mint(
        origin: Origin,
        name: AssetName,
        ticker: Ticker,
        total_supply: Balance,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult;

    fn register_ticker(origin: Origin, ticker: Ticker) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    /// Adds an artificial IU claim for benchmarks
    fn add_investor_uniqueness_claim(did: IdentityId, ticker: Ticker);

    fn issue(origin: Origin, ticker: Ticker, total_supply: Balance) -> DispatchResult;
}
