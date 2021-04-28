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

use crate::traits::{
    checkpoint, compliance_manager, contracts, external_agents, portfolio, statistics,
};
use crate::CommonTrait;
use codec::{Decode, Encode};
use frame_support::decl_event;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::{Currency, Get, UnixTime};
use frame_support::weights::Weight;
use polymesh_primitives::asset::{AssetName, AssetType, FundingRoundName};
use polymesh_primitives::ethereum::EthereumAddress;
use polymesh_primitives::migrate::MigrationError;
use polymesh_primitives::{
    AssetIdentifier, Document, DocumentId, IdentityId, PortfolioId, ScopeId, SmartExtensionName,
    SmartExtensionType, Ticker,
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

pub trait WeightInfo {
    fn register_ticker() -> Weight;
    fn accept_ticker_transfer() -> Weight;
    fn accept_asset_ownership_transfer() -> Weight;
    fn create_asset(n: u32, i: u32, f: u32) -> Weight;
    fn freeze() -> Weight;
    fn unfreeze() -> Weight;
    fn rename_asset(n: u32) -> Weight;
    fn issue() -> Weight;
    fn redeem() -> Weight;
    fn make_divisible() -> Weight;
    fn add_documents(d: u32) -> Weight;
    fn remove_documents(d: u32) -> Weight;
    fn set_funding_round(f: u32) -> Weight;
    fn update_identifiers(i: u32) -> Weight;
    fn claim_classic_ticker() -> Weight;
    fn reserve_classic_ticker() -> Weight;
    fn add_extension() -> Weight;
    fn remove_smart_extension() -> Weight;
    fn archive_extension() -> Weight;
    fn unarchive_extension() -> Weight;
    fn controller_transfer() -> Weight;
}

/// The module's configuration trait.
pub trait Trait:
    crate::balances::Trait
    + external_agents::Trait
    + pallet_session::Trait
    + statistics::Trait
    + contracts::Trait
    + portfolio::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>>
        + From<checkpoint::Event<Self>>
        + Into<<Self as frame_system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;

    type ComplianceManager: compliance_manager::Trait<Self::Balance>;

    /// Maximum number of smart extensions can attach to an asset.
    /// This hard limit is set to avoid the cases where an asset transfer
    /// gas usage go beyond the block gas limit.
    type MaxNumberOfTMExtensionForAsset: Get<u32>;

    /// Time used in computation of checkpoints.
    type UnixTime: UnixTime;

    /// Max length for the name of an asset.
    type AssetNameMaxLength: Get<u32>;

    /// Max length of the funding round name.
    type FundingRoundNameMaxLength: Get<u32>;

    type AssetFn: AssetFnTrait<Self::Balance, Self::AccountId, Self::Origin>;

    type WeightInfo: WeightInfo;
    type CPWeightInfo: crate::traits::checkpoint::WeightInfo;
}

/// Errors of migration on this pallet.
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub enum AssetMigrationError {
    /// Migration of document fails on the given ticker and document id.
    AssetDocumentFail(Ticker, DocumentId),
}

decl_event! {
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
        Moment = <T as pallet_timestamp::Trait>::Moment,
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// Event for transfer of tokens.
        /// caller DID, ticker, from portfolio, to portfolio, value
        Transfer(IdentityId, Ticker, PortfolioId, PortfolioId, Balance),
        /// Emit when tokens get issued.
        /// caller DID, ticker, beneficiary DID, value, funding round, total issued in this funding round
        Issued(IdentityId, Ticker, IdentityId, Balance, FundingRoundName, Balance),
        /// Emit when tokens get redeemed.
        /// caller DID, ticker,  from DID, value
        Redeemed(IdentityId, Ticker, IdentityId, Balance),
        /// Event for creation of the asset.
        /// caller DID/ owner DID, ticker, divisibility, asset type, beneficiary DID
        AssetCreated(IdentityId, Ticker, bool, AssetType, IdentityId),
        /// Event emitted when any token identifiers are updated.
        /// caller DID, ticker, a vector of (identifier type, identifier value)
        IdentifiersUpdated(IdentityId, Ticker, Vec<AssetIdentifier>),
        /// Event for change in divisibility.
        /// caller DID, ticker, divisibility
        DivisibilityChanged(IdentityId, Ticker, bool),
        /// An additional event to Transfer; emitted when `transfer_with_data` is called.
        /// caller DID , ticker, from DID, to DID, value, data
        TransferWithData(IdentityId, Ticker, IdentityId, IdentityId, Balance, Vec<u8>),
        /// is_issuable() output
        /// ticker, return value (true if issuable)
        IsIssuable(Ticker, bool),
        /// Emit when ticker is registered.
        /// caller DID / ticker owner did, ticker, ticker owner, expiry
        TickerRegistered(IdentityId, Ticker, Option<Moment>),
        /// Emit when ticker is transferred.
        /// caller DID / ticker transferred to DID, ticker, from
        TickerTransferred(IdentityId, Ticker, IdentityId),
        /// Emit when token ownership is transferred.
        /// caller DID / token ownership transferred to DID, ticker, from
        AssetOwnershipTransferred(IdentityId, Ticker, IdentityId),
        /// An event emitted when an asset is frozen.
        /// Parameter: caller DID, ticker.
        AssetFrozen(IdentityId, Ticker),
        /// An event emitted when an asset is unfrozen.
        /// Parameter: caller DID, ticker.
        AssetUnfrozen(IdentityId, Ticker),
        /// An event emitted when a token is renamed.
        /// Parameters: caller DID, ticker, new token name.
        AssetRenamed(IdentityId, Ticker, AssetName),
        /// An event carrying the name of the current funding round of a ticker.
        /// Parameters: caller DID, ticker, funding round name.
        FundingRoundSet(IdentityId, Ticker, FundingRoundName),
        /// Emitted when extension is added successfully.
        /// caller DID, ticker, extension AccountId, extension name, type of smart Extension
        ExtensionAdded(IdentityId, Ticker, AccountId, SmartExtensionName, SmartExtensionType),
        /// Emitted when extension get archived.
        /// caller DID, ticker, AccountId
        ExtensionArchived(IdentityId, Ticker, AccountId),
        /// Emitted when extension get archived.
        /// caller DID, ticker, AccountId
        ExtensionUnArchived(IdentityId, Ticker, AccountId),
        /// A new document attached to an asset
        DocumentAdded(IdentityId, Ticker, DocumentId, Document),
        /// A document removed from an asset
        DocumentRemoved(IdentityId, Ticker, DocumentId),
        /// A extension got removed.
        /// caller DID, ticker, AccountId
        ExtensionRemoved(IdentityId, Ticker, AccountId),
        /// A Polymath Classic token was claimed and transferred to a non-systematic DID.
        ClassicTickerClaimed(IdentityId, Ticker, EthereumAddress),
        /// Migration error event.
        MigrationFailure(MigrationError<AssetMigrationError>),
        /// Event for when a forced transfer takes place.
        /// caller DID/ controller DID, ticker, Portfolio of token holder, value.
        ControllerTransfer(IdentityId, Ticker, PortfolioId, Balance),
    }
}
