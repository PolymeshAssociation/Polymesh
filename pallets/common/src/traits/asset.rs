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

#[cfg(feature = "runtime-benchmarks")]
use polymesh_primitives::PortfolioKind;

use frame_support::decl_event;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::{Currency, Get, UnixTime};
use frame_support::weights::Weight;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::Vec;

use polymesh_primitives::asset::{
    AssetId, AssetName, AssetType, CustomAssetTypeId, FundingRoundName,
};
use polymesh_primitives::asset_metadata::{
    AssetMetadataGlobalKey, AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName,
    AssetMetadataSpec, AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::{
    AssetIdentifier, Balance, Document, DocumentId, IdentityId, PortfolioId, PortfolioUpdateReason,
    Ticker,
};

use crate::traits::nft::NFTTrait;
use crate::traits::{checkpoint, compliance_manager, external_agents, portfolio, statistics};

/// The module's configuration trait.
pub trait Config:
    crate::balances::Config + external_agents::Config + statistics::Config + portfolio::Config
{
    /// The overarching event type.
    type RuntimeEvent: From<Event<Self>>
        + From<checkpoint::Event>
        + Into<<Self as frame_system::Config>::RuntimeEvent>;

    type Currency: Currency<Self::AccountId>;

    type ComplianceManager: compliance_manager::ComplianceFnConfig;

    /// Time used in computation of checkpoints.
    type UnixTime: UnixTime;

    /// Max length for the name of an asset.
    type AssetNameMaxLength: Get<u32>;

    /// Max length of the funding round name.
    type FundingRoundNameMaxLength: Get<u32>;

    /// Max length for the Asset Metadata type name.
    type AssetMetadataNameMaxLength: Get<u32>;

    /// Max length for the Asset Metadata value.
    type AssetMetadataValueMaxLength: Get<u32>;

    /// Max length for the Asset Metadata type definition.
    type AssetMetadataTypeDefMaxLength: Get<u32>;

    type AssetFn: AssetFnTrait<Self::AccountId, Self::RuntimeOrigin>;

    type WeightInfo: WeightInfo;

    type CPWeightInfo: crate::traits::checkpoint::WeightInfo;

    type NFTFn: NFTTrait<Self::RuntimeOrigin>;

    /// Maximum number of mediators for an asset.
    type MaxAssetMediators: Get<u32>;
}

decl_event! {
    pub enum Event<T>
    where
        Moment = <T as pallet_timestamp::Config>::Moment,
    {
        /// Event for creation of the asset.
        /// caller DID/ owner DID, AssetId, divisibility, asset type, beneficiary DID, asset name, identifiers, funding round
        AssetCreated(IdentityId, AssetId, bool, AssetType, IdentityId, AssetName, Vec<AssetIdentifier>, Option<FundingRoundName>),
        /// Event emitted when any token identifiers are updated.
        /// caller DID, AssetId, a vector of (identifier type, identifier value)
        IdentifiersUpdated(IdentityId, AssetId, Vec<AssetIdentifier>),
        /// Event for change in divisibility.
        /// caller DID, AssetId, divisibility
        DivisibilityChanged(IdentityId, AssetId, bool),
        /// Emit when ticker is registered.
        /// caller DID / ticker owner did, ticker, ticker owner, expiry
        TickerRegistered(IdentityId, Ticker, Option<Moment>),
        /// Emit when ticker is transferred.
        /// caller DID / ticker transferred to DID, ticker, from
        TickerTransferred(IdentityId, Ticker, IdentityId),
        /// Emit when token ownership is transferred.
        /// caller DID / token ownership transferred to DID, AssetId, from
        AssetOwnershipTransferred(IdentityId, AssetId, IdentityId),
        /// An event emitted when an asset is frozen.
        /// Parameter: caller DID, AssetId.
        AssetFrozen(IdentityId, AssetId),
        /// An event emitted when an asset is unfrozen.
        /// Parameter: caller DID, AssetId.
        AssetUnfrozen(IdentityId, AssetId),
        /// An event emitted when a token is renamed.
        /// Parameters: caller DID, AssetId, new token name.
        AssetRenamed(IdentityId, AssetId, AssetName),
        /// An event carrying the name of the current funding round of an asset.
        /// Parameters: caller DID, AssetId, funding round name.
        FundingRoundSet(IdentityId, AssetId, FundingRoundName),
        /// A new document attached to an asset
        DocumentAdded(IdentityId, AssetId, DocumentId, Document),
        /// A document removed from an asset
        DocumentRemoved(IdentityId, AssetId, DocumentId),
        /// Event for when a forced transfer takes place.
        /// caller DID/ controller DID, ExtensionRemoved, Portfolio of token holder, value.
        ControllerTransfer(IdentityId, AssetId, PortfolioId, Balance),
        /// A custom asset type already exists on-chain.
        /// caller DID, the ID of the custom asset type, the string contents registered.
        CustomAssetTypeExists(IdentityId, CustomAssetTypeId, Vec<u8>),
        /// A custom asset type was registered on-chain.
        /// caller DID, the ID of the custom asset type, the string contents registered.
        CustomAssetTypeRegistered(IdentityId, CustomAssetTypeId, Vec<u8>),
        /// Set asset metadata value.
        /// (Caller DID, AssetId, metadata value, optional value details)
        SetAssetMetadataValue(IdentityId, AssetId, AssetMetadataValue, Option<AssetMetadataValueDetail<Moment>>),
        /// Set asset metadata value details (expire, lock status).
        /// (Caller DID, AssetId, value details)
        SetAssetMetadataValueDetails(IdentityId, AssetId, AssetMetadataValueDetail<Moment>),
        /// Register asset metadata local type.
        /// (Caller DID, AssetId, Local type name, Local type key, type specs)
        RegisterAssetMetadataLocalType(IdentityId, AssetId, AssetMetadataName, AssetMetadataLocalKey, AssetMetadataSpec),
        /// Register asset metadata global type.
        /// (Global type name, Global type key, type specs)
        RegisterAssetMetadataGlobalType(AssetMetadataName, AssetMetadataGlobalKey, AssetMetadataSpec),
        /// An event emitted when the type of an asset changed.
        /// Parameters: caller DID, AssetId, new token type.
        AssetTypeChanged(IdentityId, AssetId, AssetType),
        /// An event emitted when a local metadata key has been removed.
        /// Parameters: caller AssetId, Local type name
        LocalMetadataKeyDeleted(IdentityId, AssetId, AssetMetadataLocalKey),
        /// An event emitted when a local metadata value has been removed.
        /// Parameters: caller AssetId, Local type name
        MetadataValueDeleted(IdentityId, AssetId, AssetMetadataKey),
        /// Emitted when Tokens were issued, redeemed or transferred.
        /// Contains the [`IdentityId`] of the receiver/issuer/redeemer, the [`AssetId`] for the token, the balance that was issued/transferred/redeemed,
        /// the [`PortfolioId`] of the source, the [`PortfolioId`] of the destination and the [`PortfolioUpdateReason`].
        AssetBalanceUpdated(
            IdentityId,
            AssetId,
            Balance,
            Option<PortfolioId>,
            Option<PortfolioId>,
            PortfolioUpdateReason,
        ),
        /// An asset has been added to the list of pre aprroved receivement (valid for all identities).
        /// Parameters: [`AssetId`] of the pre approved asset.
        AssetAffirmationExemption(AssetId),
        /// An asset has been removed from the list of pre aprroved receivement (valid for all identities).
        /// Parameters: [`AssetId`] of the asset.
        RemoveAssetAffirmationExemption(AssetId),
        /// An identity has added an asset to the list of pre aprroved receivement.
        /// Parameters: [`IdentityId`] of caller, [`AssetId`] of the pre approved asset.
        PreApprovedAsset(IdentityId, AssetId),
        /// An identity has removed an asset to the list of pre aprroved receivement.
        /// Parameters: [`IdentityId`] of caller, [`AssetId`] of the asset.
        RemovePreApprovedAsset(IdentityId, AssetId),
        /// An identity has added mandatory mediators to an asset.
        /// Parameters: [`IdentityId`] of caller, [`AssetId`] of the asset, the identity of all mediators added.
        AssetMediatorsAdded(IdentityId, AssetId, BTreeSet<IdentityId>),
        /// An identity has removed mediators from an asset.
        /// Parameters: [`IdentityId`] of caller, [`AssetId`] of the asset, the identity of all mediators removed.
        AssetMediatorsRemoved(IdentityId, AssetId, BTreeSet<IdentityId>),
        /// An identity has linked a ticker to an asset.
        /// Parameters: [`IdentityId`] of caller, [`Ticker`] of the asset, the asset identifier [`AssetId`].
        TickerLinkedToAsset(IdentityId, Ticker, AssetId),
        /// An identity has unlinked a ticker from an asset.
        /// Parameters: [`IdentityId`] of caller, unlinked [`Ticker`], the asset identifier [`AssetId`].
        TickerUnlinkedFromAsset(IdentityId, Ticker, AssetId),
    }
}

pub trait WeightInfo {
    fn register_unique_ticker() -> Weight;
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
    fn controller_transfer() -> Weight;
    fn register_custom_asset_type(n: u32) -> Weight;
    fn set_asset_metadata() -> Weight;
    fn set_asset_metadata_details() -> Weight;
    fn register_and_set_local_asset_metadata() -> Weight;
    fn register_asset_metadata_local_type() -> Weight;
    fn register_asset_metadata_global_type() -> Weight;
    fn update_asset_type() -> Weight;
    fn remove_local_metadata_key() -> Weight;
    fn remove_metadata_value() -> Weight;
    fn base_transfer() -> Weight;
    fn exempt_asset_affirmation() -> Weight;
    fn remove_asset_affirmation_exemption() -> Weight;
    fn pre_approve_asset() -> Weight;
    fn remove_asset_pre_approval() -> Weight;
    fn add_mandatory_mediators(n: u32) -> Weight;
    fn remove_mandatory_mediators(n: u32) -> Weight;
    fn link_ticker_to_asset_id() -> Weight;
    fn unlink_ticker_from_asset_id() -> Weight;
}

pub trait AssetFnTrait<Account, Origin> {
    /// Returns `Ok` if [`AssetDetails::divisible`] or `value` % ONE_UNIT == 0.
    fn ensure_granular(asset_id: &AssetId, value: Balance) -> DispatchResult;

    /// Returns `true` if the given `identity_id` is exempt from affirming the receivement of `asset_id`, otherwise returns `false`.
    fn skip_asset_affirmation(identity_id: &IdentityId, asset_id: &AssetId) -> bool;

    /// Returns `true` if the receivement of `asset_id` is exempt from being affirmed, otherwise returns `false`.
    fn asset_affirmation_exemption(asset_id: &AssetId) -> bool;

    /// Returns the `did` balance for the given `asset_id`.
    fn asset_balance(asset_id: &AssetId, did: &IdentityId) -> Balance;

    /// Returns the total supply for the given `asset_id`.
    fn asset_total_supply(asset_id: &AssetId) -> Result<Balance, DispatchError>;

    /// Returns the next [`AssetID`] for the `caller_acc`.
    fn generate_asset_id(caller_acc: Account) -> AssetId;

    #[cfg(feature = "runtime-benchmarks")]
    fn register_unique_ticker(origin: Origin, ticker: Ticker) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn create_asset(
        origin: Origin,
        asset_name: AssetName,
        divisible: bool,
        asset_type: AssetType,
        asset_identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn issue(
        origin: Origin,
        asset_id: AssetId,
        amount: Balance,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn register_asset_metadata_type(
        origin: Origin,
        asset_id: Option<AssetId>,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn add_mandatory_mediators(
        origin: Origin,
        asset_id: AssetId,
        mediators: BTreeSet<IdentityId>,
    ) -> DispatchResult;
}
