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

//! # Asset Module
//!
//! The Asset module provides functionality for creating and managing assets on the Polymesh blockchain.
//! In Ethereum analogies, assets are similar to ERC-20 tokens.  On Polymesh assets are uniquely identified by their `AssetId`
//! instead of a contract address.
//!
//! ## Overview
//!
//! The Asset module allows users to:
//!
//! - Register unique tickers for assets.
//! - Create new assets with specified properties such as divisibility and custom types.
//! - Manage asset ownership and transfer ownership to other identities.
//! - Issue and redeem tokens for assets.
//! - Freeze and unfreeze asset transfers and minting.
//! - Rename assets and update their metadata.
//! - Add and remove documents associated with assets.
//! - Set and update funding rounds for assets.
//! - Link and unlink tickers to asset IDs.
//! - Pre-approve asset receivements and manage mandatory mediators for assets.
//! - Ensure compliance with asset transfer rules and restrictions.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `register_unique_ticker` - Registers or extends the registration of a ticker.
//! - `accept_ticker_transfer` - Accepts a ticker transfer authorization.
//! - `accept_asset_ownership_transfer` - Accepts an asset ownership transfer authorization.
//! - `create_asset` - Creates a new asset.
//! - `freeze` - Freezes transfers and minting of an asset.
//! - `unfreeze` - Unfreezes transfers and minting of an asset.
//! - `rename_asset` - Renames an asset.
//! - `controller_transfer` - Forces a transfer between two DIDs.
//! - `issue` - Issues new tokens for an asset.
//! - `redeem` - Redeems tokens from an asset.
//! - `set_asset_metadata` - Sets metadata for an asset.
//! - `set_asset_metadata_details` - Sets metadata details for an asset.
//! - `register_and_set_local_asset_metadata` - Registers and sets local metadata for an asset.
//! - `register_asset_metadata_local_type` - Registers a local metadata type for an asset.
//! - `register_asset_metadata_global_type` - Registers a global metadata type.
//! - `remove_metadata_value` - Removes metadata value for an asset.
//! - `remove_local_metadata_key` - Removes a local metadata key for an asset.
//! - `update_asset_type` - Updates the type of an asset.
//! - `add_documents` - Adds documents to an asset.
//! - `remove_documents` - Removes documents from an asset.
//! - `set_funding_round` - Sets the funding round for an asset.
//! - `make_divisible` - Makes an asset divisible.
//! - `pre_approve_asset` - Pre-approves the receivement of an asset.
//! - `remove_asset_pre_approval` - Removes the pre-approval of an asset.
//! - `exempt_asset_affirmation` - Exempts an asset from affirmation.
//! - `remove_asset_affirmation_exemption` - Removes the affirmation exemption of an asset.
//! - `add_mandatory_mediators` - Adds mandatory mediators for an asset.
//! - `remove_mandatory_mediators` - Removes mandatory mediators from an asset.
//! - `link_ticker_to_asset_id` - Links a ticker to an asset ID.
//! - `unlink_ticker_from_asset_id` - Unlinks a ticker from an asset ID.
//! - `create_asset_with_custom_type` - Creates an asset with a custom type.
//! - `register_custom_asset_type` - Registers a new custom asset type.
//! - `update_identifiers` - Updates the identifiers for an asset.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod checkpoint;

#[cfg(feature = "runtime-benchmarks")]
use frame_system::RawOrigin;

mod migrations;
mod types;

use codec::{Decode, Encode};
use core::mem;
use currency::*;
use frame_support::dispatch::{DispatchResult, DispatchResultWithPostInfo, PostDispatchInfo};
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchError;
use frame_support::traits::{Currency, Get, UnixTime};
use frame_support::weights::Weight;
use frame_support::BoundedBTreeSet;
use frame_system::ensure_root;
use frame_system::pallet_prelude::*;
use sp_io::hashing::blake2_128;
use sp_runtime::traits::Zero;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

use pallet_base::{
    ensure_opt_string_limited, ensure_string_limited, try_next_pre, Error::CounterOverflow,
};
use pallet_external_agents::Config as EAConfig;
use pallet_identity::PermissionedCallOriginData;
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::asset::{
    AssetHolder, AssetHolderKind, AssetId, AssetName, AssetType, CheckpointId, CustomAssetTypeId,
    FundingRoundName,
};
use polymesh_primitives::asset_metadata::{
    AssetMetadataGlobalKey, AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName,
    AssetMetadataSpec, AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::constants::*;
use polymesh_primitives::portfolio::{Fund, FundDescription};
use polymesh_primitives::protocol_fee::{ChargeProtocolFee, ProtocolOp};
use polymesh_primitives::settlement::{AssetCount, InstructionId};
use polymesh_primitives::traits::{
    AffirmationFnTrait, AssetFnConfig, AssetFnTrait, ComplianceFnConfig, NFTTrait,
    SettlementFnTrait,
};
use polymesh_primitives::{
    extract_auth, storage_migrate_on, storage_migration_ver, AccountId as AccountId32,
    AssetIdentifier, Balance, Document, DocumentId, HoldingsUpdateReason, IdentityId, Memo,
    SecondaryKey, Ticker, WeightMeter,
};

pub use types::{
    AssetDetails, AssetOwnershipRelation, TickerRegistration, TickerRegistrationConfig,
    TickerRegistrationStatus,
};

type Checkpoint<T> = checkpoint::Pallet<T>;
type ExternalAgents<T> = pallet_external_agents::Pallet<T>;
type IdentityPallet<T> = pallet_identity::Pallet<T>;
type PortfolioPallet<T> = pallet_portfolio::Pallet<T>;
type Statistics<T> = pallet_statistics::Pallet<T>;

/// Combinded config traits of the asset and checkpoint pallets.
pub trait AssetConfig: Config + checkpoint::Config {}

impl<T: Config + checkpoint::Config> AssetConfig for T {}

storage_migration_ver!(7);

pub use pallet::*;

/// The module's configuration trait.
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + EAConfig
        + pallet_statistics::Config
        + pallet_portfolio::Config
        + AssetFnConfig
    {
        type Currency: Currency<Self::AccountId>;

        type ComplianceManager: ComplianceFnConfig;

        /// Time used in computation of checkpoints.
        type UnixTime: UnixTime;

        /// Max length for the name of an asset.
        #[pallet::constant]
        type AssetNameMaxLength: Get<u32>;

        /// Max length of the funding round name.
        #[pallet::constant]
        type FundingRoundNameMaxLength: Get<u32>;

        /// Max length for the Asset Metadata type name.
        #[pallet::constant]
        type AssetMetadataNameMaxLength: Get<u32>;

        /// Max length for the Asset Metadata value.
        #[pallet::constant]
        type AssetMetadataValueMaxLength: Get<u32>;

        /// Max length for the Asset Metadata type definition.
        #[pallet::constant]
        type AssetMetadataTypeDefMaxLength: Get<u32>;

        type WeightInfo: WeightInfo;

        /// The implementation of nft functions.
        type NFTFn: NFTTrait<Self::RuntimeOrigin>;

        /// The implementation of settlement asset transfer functions.
        type SettlementFn: SettlementFnTrait<Self>;

        /// Maximum number of mediators for an asset.
        #[pallet::constant]
        type MaxAssetMediators: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event for creation of the asset.
        /// caller DID/ owner DID, AssetId, divisibility, asset type, beneficiary DID, asset name, identifiers, funding round
        AssetCreated(
            IdentityId,
            AssetId,
            bool,
            AssetType,
            IdentityId,
            AssetName,
            Vec<AssetIdentifier>,
            Option<FundingRoundName>,
        ),
        /// Event emitted when any token identifiers are updated.
        /// caller DID, AssetId, a vector of (identifier type, identifier value)
        IdentifiersUpdated(IdentityId, AssetId, Vec<AssetIdentifier>),
        /// Event for change in divisibility.
        /// caller DID, AssetId, divisibility
        DivisibilityChanged(IdentityId, AssetId, bool),
        /// Emit when ticker is registered.
        /// caller DID / ticker owner did, ticker, ticker owner, expiry
        TickerRegistered(IdentityId, Ticker, Option<T::Moment>),
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
        ControllerTransfer(IdentityId, AssetId, AssetHolder, Balance),
        /// A custom asset type already exists on-chain.
        /// caller DID, the ID of the custom asset type, the string contents registered.
        CustomAssetTypeExists(IdentityId, CustomAssetTypeId, Vec<u8>),
        /// A custom asset type was registered on-chain.
        /// caller DID, the ID of the custom asset type, the string contents registered.
        CustomAssetTypeRegistered(IdentityId, CustomAssetTypeId, Vec<u8>),
        /// Set asset metadata value.
        /// (Caller DID, AssetId, metadata value, optional value details)
        SetAssetMetadataValue(
            IdentityId,
            AssetId,
            AssetMetadataValue,
            Option<AssetMetadataValueDetail<T::Moment>>,
        ),
        /// Set asset metadata value details (expire, lock status).
        /// (Caller DID, AssetId, value details)
        SetAssetMetadataValueDetails(IdentityId, AssetId, AssetMetadataValueDetail<T::Moment>),
        /// Register asset metadata local type.
        /// (Caller DID, AssetId, Local type name, Local type key, type specs)
        RegisterAssetMetadataLocalType(
            IdentityId,
            AssetId,
            AssetMetadataName,
            AssetMetadataLocalKey,
            AssetMetadataSpec,
        ),
        /// Register asset metadata global type.
        /// (Global type name, Global type key, type specs)
        RegisterAssetMetadataGlobalType(
            AssetMetadataName,
            AssetMetadataGlobalKey,
            AssetMetadataSpec,
        ),
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
        /// the [`AssetHolder`] of the source, the [`AssetHolder`] of the destination and the [`HoldingsUpdateReason`].
        AssetBalanceUpdated(
            IdentityId,
            AssetId,
            Balance,
            Option<AssetHolder>,
            Option<AssetHolder>,
            HoldingsUpdateReason,
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
        /// Asset Global Metadata Spec has been Updated.
        /// Parameters: [`AssetMetadataName`] of the metadata, [`AssetMetadataSpec`] of the metadata.
        GlobalMetadataSpecUpdated(AssetMetadataName, AssetMetadataSpec),
        /// An asset transfer has been created.
        CreatedAssetTransfer {
            asset_id: AssetId,
            from: T::AccountId,
            to: T::AccountId,
            amount: Balance,
            memo: Option<Memo>,
            pending_transfer_id: Option<InstructionId>,
        },
        /// A spender allowance was set for an asset.
        Approval {
            owner: T::AccountId,
            spender: T::AccountId,
            asset_id: AssetId,
            amount: Balance,
        },
        /// A spender used part of an allowance.
        AllowanceSpent {
            owner: T::AccountId,
            spender: T::AccountId,
            asset_id: AssetId,
            amount_spent: Balance,
            remaining_allowance: Balance,
        },
    }

    /// Map each [`Ticker`] to its registration details ([`TickerRegistration`]).
    #[pallet::storage]
    pub type UniqueTickerRegistration<T: Config> =
        StorageMap<_, Blake2_128Concat, Ticker, TickerRegistration<T::Moment>, OptionQuery>;

    /// Returns [`TickerRegistrationConfig`] for assessing if a ticker is valid.
    #[pallet::storage]
    pub type TickerConfig<T: Config> =
        StorageValue<_, TickerRegistrationConfig<T::Moment>, ValueQuery>;

    /// Maps each [`AssetId`] to its underling [`AssetDetails`].
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Assets<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, AssetDetails, OptionQuery>;

    /// Maps each [`AssetId`] to its underling [`AssetName`].
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetNames<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, AssetName, OptionQuery>;

    /// Tracks the total [`Balance`] for each [`AssetId`] per [`IdentityId`].
    // NB: It is safe to use `identity` hasher here because assets can not be distributed to non-existent identities.
    #[pallet::storage]
    pub type BalanceOf<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, AssetId, Identity, IdentityId, Balance, ValueQuery>;

    /// Maps each [`AssetId`] to its asset identifiers ([`AssetIdentifier`]).
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetIdentifiers<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, Vec<AssetIdentifier>, ValueQuery>;

    /// The next `AssetType::Custom` ID in the sequence.
    ///
    /// Numbers in the sequence start from 1 rather than 0.
    #[pallet::storage]
    pub type CustomTypeIdSequence<T: Config> = StorageValue<_, CustomAssetTypeId, ValueQuery>;

    /// Maps custom asset type ids to the registered string contents.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type CustomTypes<T: Config> =
        StorageMap<_, Twox64Concat, CustomAssetTypeId, Vec<u8>, ValueQuery>;

    /// Inverse map of `CustomTypes`, from registered string contents to custom asset type ids.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type CustomTypesInverse<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, CustomAssetTypeId, OptionQuery>;

    /// Maps each [`AssetId`] to the name of its founding round ([`FundingRoundName`]).
    #[pallet::storage]
    #[pallet::unbounded]
    pub type FundingRound<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, FundingRoundName, ValueQuery>;

    /// The total [`Balance`] of tokens issued in all recorded funding rounds ([`FundingRoundName`]).
    #[pallet::storage]
    #[pallet::unbounded]
    pub type IssuedInFundingRound<T: Config> =
        StorageMap<_, Blake2_128Concat, (AssetId, FundingRoundName), Balance, ValueQuery>;

    /// Returns `true` if transfers for the asset are frozen. Otherwise, returns `false`.
    #[pallet::storage]
    pub type Frozen<T: Config> = StorageMap<_, Blake2_128Concat, AssetId, bool, ValueQuery>;

    /// All [`Document`] attached to an asset.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetDocuments<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Twox64Concat,
        DocumentId,
        Document,
        OptionQuery,
    >;

    /// [`DocumentId`] counter per [`AssetId`].
    #[pallet::storage]
    pub type AssetDocumentsIdSequence<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, DocumentId, ValueQuery>;

    /// Metatdata values for an asset.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetMetadataValues<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Twox64Concat,
        AssetMetadataKey,
        AssetMetadataValue,
        OptionQuery,
    >;

    /// Details for an asset's Metadata values.
    #[pallet::storage]
    pub type AssetMetadataValueDetails<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Twox64Concat,
        AssetMetadataKey,
        AssetMetadataValueDetail<T::Moment>,
        OptionQuery,
    >;

    /// Asset Metadata Local Name -> Key.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetMetadataLocalNameToKey<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Blake2_128Concat,
        AssetMetadataName,
        AssetMetadataLocalKey,
        OptionQuery,
    >;

    /// Asset Metadata Global Name -> Key.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetMetadataGlobalNameToKey<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetMetadataName, AssetMetadataGlobalKey, OptionQuery>;

    /// Asset Metadata Local Key -> Name.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetMetadataLocalKeyToName<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Twox64Concat,
        AssetMetadataLocalKey,
        AssetMetadataName,
        OptionQuery,
    >;

    /// Asset Metadata Global Key -> Name.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetMetadataGlobalKeyToName<T: Config> =
        StorageMap<_, Twox64Concat, AssetMetadataGlobalKey, AssetMetadataName, OptionQuery>;

    /// Asset Metadata Local Key specs.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetMetadataLocalSpecs<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Twox64Concat,
        AssetMetadataLocalKey,
        AssetMetadataSpec,
        OptionQuery,
    >;

    /// Asset Metadata Global Key specs.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetMetadataGlobalSpecs<T: Config> =
        StorageMap<_, Twox64Concat, AssetMetadataGlobalKey, AssetMetadataSpec, OptionQuery>;

    /// A list of assets that exempt all users from affirming its receivement.
    #[pallet::storage]
    pub type AssetsExemptFromAffirmation<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, bool, ValueQuery>;

    /// All assets that don't need an affirmation to be received by an identity.
    #[pallet::storage]
    pub type PreApprovedAsset<T: Config> =
        StorageDoubleMap<_, Identity, IdentityId, Blake2_128Concat, AssetId, bool, ValueQuery>;

    /// Identities that require receiver affirmation for all incoming transfers.
    /// The list of mandatory mediators for every ticker.
    #[pallet::storage]
    pub type MandatoryMediators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        AssetId,
        BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
        ValueQuery,
    >;

    /// The last [`AssetMetadataLocalKey`] used for [`AssetId`].
    #[pallet::storage]
    pub type CurrentAssetMetadataLocalKey<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, AssetMetadataLocalKey, OptionQuery>;

    /// The last [`AssetMetadataGlobalKey`] used for a global key.
    #[pallet::storage]
    pub type CurrentAssetMetadataGlobalKey<T: Config> =
        StorageValue<_, AssetMetadataGlobalKey, OptionQuery>;

    /// All tickers owned by a user.
    #[pallet::storage]
    pub type TickersOwnedByUser<T: Config> =
        StorageDoubleMap<_, Identity, IdentityId, Blake2_128Concat, Ticker, bool, ValueQuery>;

    /// All assets owned by a user.
    #[pallet::storage]
    pub type SecurityTokensOwnedByUser<T: Config> =
        StorageDoubleMap<_, Identity, IdentityId, Blake2_128Concat, AssetId, bool, ValueQuery>;

    /// Maps all [`AssetId`] that are mapped to a [`Ticker`].
    #[pallet::storage]
    pub type AssetIdTicker<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, Ticker, OptionQuery>;

    /// Maps all [`Ticker`] that are linked to an [`AssetId`].
    #[pallet::storage]
    pub type TickerAssetId<T: Config> =
        StorageMap<_, Blake2_128Concat, Ticker, AssetId, OptionQuery>;

    /// A per account nonce that is used for generating an [`AssetId`].
    #[pallet::storage]
    pub type AssetNonce<T: Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

    /// Tracks the total [`Balance`] held by the account for each [`AssetId`].
    #[pallet::storage]
    pub type AssetBalance<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        AccountId32,
        Blake2_128Concat,
        AssetId,
        Balance,
        ValueQuery,
    >;

    /// Tracks the [`Balance`] of locked tokens for assets that are held by the key (i.e., not in a portfolio).
    #[pallet::storage]
    pub type LockedBalance<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        AccountId32,
        Blake2_128Concat,
        AssetId,
        Balance,
        ValueQuery,
    >;

    /// Maps (owner, spender, asset_id) to the approved allowance amount.
    ///
    /// A non-existent entry returns 0 (via `ValueQuery`), matching ERC-20 behavior.
    /// When an allowance is revoked (set to 0), the entry is removed to bound storage growth.
    ///
    /// Uses `StorageNMap` so that all allowances for a given owner can be iterated
    /// via prefix.
    #[pallet::storage]
    pub type Allowances<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>,
            NMapKey<Blake2_128Concat, T::AccountId>,
            NMapKey<Blake2_128Concat, AssetId>,
        ),
        Balance,
        ValueQuery,
    >;

    /// The next available asset index for the ERC20 asset id mapping.
    /// This is incremented each time a new ERC20 asset mapping is created.
    #[pallet::storage]
    pub type NextAssetIndex<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Mapping an asset index (derived from the precompile address) to a `AssetId`.
    #[pallet::storage]
    pub type Erc20IndexToAssetId<T: Config> = StorageMap<_, Identity, u32, AssetId, OptionQuery>;

    /// Mapping a `ERC20AssetId` to an asset index (used for deriving precompile addresses).
    #[pallet::storage]
    pub type Erc20AssetIdToIndex<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, u32, OptionQuery>;

    /// Storage version.
    #[pallet::storage]
    pub type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let mut weight = Weight::zero();

            storage_migrate_on!(StorageVersion::<T>, 7, {
                weight = migrations::migrate_to_v7::<T>();
            });

            weight
        }
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        pub ticker_registration_config: TickerRegistrationConfig<polymesh_primitives::Moment>,
        pub reserved_country_currency_codes: Vec<Ticker>,
        pub asset_metadata: Vec<(AssetMetadataName, AssetMetadataSpec)>,
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T>
    where
        T: AssetConfig,
    {
        fn build(&self) {
            StorageVersion::<T>::put(Version::new(7));

            // Set ticker registratoin config.
            TickerConfig::<T>::put(TickerRegistrationConfig {
                max_ticker_length: self.ticker_registration_config.max_ticker_length,
                registration_length: self
                    .ticker_registration_config
                    .registration_length
                    // TODO: Fix this hack?
                    .map(|m| (m as u32).into()),
            });

            // Reserving country currency logic
            let fiat_tickers_reservation_did =
                polymesh_primitives::SystematicIssuers::FiatTickersReservation.as_id();
            for currency_ticker in &self.reserved_country_currency_codes {
                let _ = <Pallet<T>>::unverified_register_ticker(
                    *currency_ticker,
                    fiat_tickers_reservation_did,
                    None,
                    false,
                );
            }

            // Register Asset Metadata.
            for (name, spec) in &self.asset_metadata {
                <Pallet<T>>::base_register_asset_metadata_global_type(name.clone(), spec.clone())
                    .expect("Shouldn't fail");
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T: AssetConfig,
    {
        /// Registers a unique ticker or extends the validity of an existing ticker.
        ///
        /// This function allows the caller to register a new ticker or extend the registration
        /// of an existing ticker. The ticker validity does not carry forward when renewing.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `ticker` - The ticker to register.
        ///
        /// # Events
        /// * `TickerRegistered` - When a ticker is successfully registered.
        ///
        /// # Errors
        /// * `TickerAlreadyRegistered` - If the ticker is already registered.
        /// * `TickerTooLong` - If the ticker length exceeds the maximum allowed length.
        /// * `InvalidTickerCharacter` - If the ticker contains invalid characters.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_unique_ticker())]
        pub fn register_unique_ticker(origin: OriginFor<T>, ticker: Ticker) -> DispatchResult {
            Self::base_register_unique_ticker(origin, ticker)
        }

        /// Accepts a ticker transfer.
        ///
        /// Consumes the authorization `auth_id` (see `pallet_identity::consume_auth`).
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `auth_id` - Authorization ID of ticker transfer authorization.
        ///
        /// # Events
        /// * `TickerTransferred` - When a ticker is successfully transferred.
        ///
        /// # Errors
        /// * `TickerRegistrationNotFound` - If the ticker registration is not found.
        /// * `TickerIsAlreadyLinkedToAnAsset` - If the ticker is already linked to an asset.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::accept_ticker_transfer())]
        pub fn accept_ticker_transfer(origin: OriginFor<T>, auth_id: u64) -> DispatchResult {
            Self::base_accept_ticker_transfer(origin, auth_id)
        }

        /// Accepts an asset ownership transfer.
        ///
        /// Consumes the authorization `auth_id` (see `pallet_identity::consume_auth`).
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `auth_id` - Authorization ID of the asset ownership transfer authorization.
        ///
        /// # Events
        /// * `AssetOwnershipTransferred` - When a asset ownership is successfully transferred.
        ///
        /// # Errors
        /// * `TickerRegistrationNotFound` - If the ticker registration is not found.
        /// * `TickerIsAlreadyLinkedToAnAsset` - If the ticker is already linked to an asset.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::accept_asset_ownership_transfer())]
        pub fn accept_asset_ownership_transfer(
            origin: OriginFor<T>,
            auth_id: u64,
        ) -> DispatchResult {
            Self::base_accept_asset_ownership_transfer(origin, auth_id)
        }

        /// Creates a new asset.
        ///
        /// The total supply will initially be zero. To mint tokens, use [`Pallet::issue`].
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_name` - The [`AssetName`] of the new asset.
        /// * `divisible` - Sets [`AssetDetails::divisible`], where `true` means the asset is divisible.
        /// * `asset_type` - The [`AssetType`] of the new asset.
        /// * `asset_identifiers` - A vector of [`AssetIdentifier`].
        /// * `funding_round_name` - The name of the funding round ([`FundingRoundName`]).
        ///
        /// # Events
        /// * `AssetCreated` - When a new asset is successfully created.
        ///
        /// # Errors
        /// * `MaxLengthOfAssetNameExceeded` - If the asset name length exceeds the maximum allowed length.
        /// * `InvalidCustomAssetTypeId` - If the custom asset type ID is invalid.
        /// * `InvalidAssetIdentifier` - If any of the asset identifiers are invalid.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::create_asset(
            asset_name.len() as u32,
            asset_identifiers.len() as u32,
            funding_round_name.as_ref().map_or(0, |name| name.len()) as u32
        ))]
        pub fn create_asset(
            origin: OriginFor<T>,
            asset_name: AssetName,
            divisible: bool,
            asset_type: AssetType,
            asset_identifiers: Vec<AssetIdentifier>,
            funding_round_name: Option<FundingRoundName>,
        ) -> DispatchResult {
            Self::base_create_asset(
                origin,
                asset_name,
                divisible,
                asset_type,
                asset_identifiers,
                funding_round_name,
            )?;
            Ok(())
        }

        /// Freezes transfers of a given asset.
        ///
        /// This function allows the asset issuer or an external agent to freeze transfers of a given asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The asset to freeze.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `AssetFrozen` - When an asset is successfully frozen.
        ///
        /// # Errors
        /// * `NoSuchAsset` - If the asset does not exist.
        /// * `AlreadyFrozen` - If the asset is already frozen.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::freeze())]
        pub fn freeze(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
            Self::base_set_freeze(origin, asset_id, true)
        }

        /// Unfreezes transfers of a given asset.
        ///
        /// This function allows the asset issuer or an external agent to unfreeze transfers of a given asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The asset to unfreeze.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `AssetUnfrozen` - When an asset is successfully unfrozen.
        ///
        /// # Errors
        /// * `NoSuchAsset` - If the asset does not exist.
        /// * `NotFrozen` - If the asset is not frozen.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::unfreeze())]
        pub fn unfreeze(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
            Self::base_set_freeze(origin, asset_id, false)
        }

        /// Updates the [`AssetName`] associated to an asset.
        ///
        /// This function allows the asset issuer or an external agent to update the name of an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `asset_name` - The new [`AssetName`] that will be associated to the asset.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `AssetRenamed` - When an asset is successfully renamed.
        ///
        /// # Errors
        /// * `MaxLengthOfAssetNameExceeded` - If the asset name length exceeds the maximum allowed length.
        /// * `NoSuchAsset` - If the asset does not exist.
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::rename_asset(asset_name.len() as u32))]
        pub fn rename_asset(
            origin: OriginFor<T>,
            asset_id: AssetId,
            asset_name: AssetName,
        ) -> DispatchResult {
            Self::base_rename_asset(origin, asset_id, asset_name)
        }

        /// Issue (i.e mint) new tokens to the caller, which must be an authorized external agent.
        ///
        /// This function allows the asset issuer or an external agent to mint new tokens for a given asset.
        ///
        /// # Arguments
        /// * `origin`: A signer that has permissions to act as an agent of `ticker`.
        /// * `asset_id`: the [`AssetId`] associated to the asset.
        /// * `amount`: The amount of tokens that will be issued.
        /// * `asset_holder_kind`: The [`AssetHolderKind`] of the portfolio that will receive the minted tokens.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        ///
        /// # Events
        /// * `AssetBalanceUpdated` - When the asset balance is successfully updated.
        ///
        /// # Errors
        /// * `UnexpectedNonFungibleToken` - If the asset is a non-fungible token.
        /// * `InvalidGranularity` - If the amount to issue does not meet the granularity requirements.
        /// * `TotalSupplyOverflow` - If the total supply exceeds the maximum allowed limit.
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::issue())]
        pub fn issue(
            origin: OriginFor<T>,
            asset_id: AssetId,
            amount: Balance,
            asset_holder_kind: AssetHolderKind,
        ) -> DispatchResult {
            Self::base_issue(origin, asset_id, amount, asset_holder_kind)
        }

        /// Redeems (i.e burns) existing tokens by reducing the balance of the caller's portfolio and the total supply of the asset.
        ///
        /// This function allows the asset issuer or an external agent to redeem tokens from a given asset.
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the asset.
        /// * `value`: amount of tokens to redeem.
        /// * `asset_holder_kind`: the [`AssetHolderKind`] that will have its balance reduced.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        ///
        /// # Events
        /// * `AssetBalanceUpdated` - When the asset balance is successfully updated.
        ///
        /// # Errors
        /// * `UnexpectedNonFungibleToken` - If the asset is a non-fungible token.
        /// * `InvalidGranularity` - If the value to redeem does not meet the granularity requirements.
        /// * `TotalSupplyOverflow` - If the total supply exceeds the maximum allowed limit.
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::redeem())]
        pub fn redeem(
            origin: OriginFor<T>,
            asset_id: AssetId,
            value: Balance,
            asset_holder_kind: AssetHolderKind,
        ) -> DispatchResult {
            let mut weight_meter = WeightMeter::max_limit_no_minimum();
            Self::base_redeem(
                origin,
                asset_id,
                value,
                asset_holder_kind,
                &mut weight_meter,
            )
        }

        /// If the asset associated to `asset_id` is indivisible, sets [`AssetDetails::divisible`] to true.
        ///
        /// This function allows the asset issuer or an external agent to make an indivisible asset divisible.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `DivisibilityChanged` - When the divisibility of an asset is successfully changed.
        ///
        /// # Errors
        /// * `NoSuchAsset` - If the asset does not exist.
        /// * `UnexpectedNonFungibleToken` - If the asset is a non-fungible token.
        /// * `AssetAlreadyDivisible` - If the asset is already divisible.
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::make_divisible())]
        pub fn make_divisible(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
            Self::base_make_divisible(origin, asset_id)
        }

        /// Add documents for a given asset.
        ///
        /// This function allows the asset issuer or an external agent to add documents to an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `docs` - Documents to be attached to the asset.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `DocumentAdded` - When a document is successfully added to an asset.
        ///
        /// # Errors
        /// * `CounterOverflow` - If the document ID counter overflows.
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::add_documents(docs.len() as u32))]
        pub fn add_documents(
            origin: OriginFor<T>,
            docs: Vec<Document>,
            asset_id: AssetId,
        ) -> DispatchResult {
            Self::base_add_documents(origin, docs, asset_id)
        }

        /// Remove documents for a given asset.
        ///
        /// This function allows the asset issuer or an external agent to remove documents from an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `docs_id` - A vector of all [`DocumentId`] that will be removed from the asset.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `DocumentRemoved` - When a document is successfully removed from an asset.
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_documents(docs_id.len() as u32))]
        pub fn remove_documents(
            origin: OriginFor<T>,
            docs_id: Vec<DocumentId>,
            asset_id: AssetId,
        ) -> DispatchResult {
            Self::base_remove_documents(origin, docs_id, asset_id)
        }

        /// Sets the name of the current funding round.
        ///
        /// This function allows the asset issuer or an external agent to set the name of the current funding round for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `funding_round_name` - The [`FundingRoundName`] of the current funding round.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `FundingRoundSet` - When the funding round name is successfully set.
        ///
        /// # Errors
        /// * `FundingRoundNameMaxLengthExceeded` - If the funding round name length exceeds the maximum allowed length.
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::set_funding_round(funding_round_name.len() as u32))]
        pub fn set_funding_round(
            origin: OriginFor<T>,
            asset_id: AssetId,
            funding_round_name: FundingRoundName,
        ) -> DispatchResult {
            Self::base_set_funding_round(origin, asset_id, funding_round_name)
        }

        /// Updates the asset identifiers associated to the asset.
        ///
        /// This function allows the asset issuer or an external agent to update the asset identifiers for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `asset_identifiers` - A vector of [`AssetIdentifier`] that will be associated to the asset.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `IdentifiersUpdated` - When the asset identifiers are successfully updated.
        ///
        /// # Errors
        /// * `InvalidAssetIdentifier` - If any of the asset identifiers are invalid.
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::update_identifiers(asset_identifiers.len() as u32))]
        pub fn update_identifiers(
            origin: OriginFor<T>,
            asset_id: AssetId,
            asset_identifiers: Vec<AssetIdentifier>,
        ) -> DispatchResult {
            Self::base_update_identifiers(origin, asset_id, asset_identifiers)
        }

        /// Forces a transfer of tokens from `asset_holder` to the caller's default portfolio.
        ///
        /// This function allows the asset issuer or an external agent to force a transfer of tokens from one portfolio to another.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `value` - The [`Balance`] of tokens that will be transferred.
        /// * `source` - The [`AssetHolder`] that will have its balance reduced.
        /// * `destination_kind` - The [`AssetHolderKind`] of the destination.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        ///
        /// # Events
        /// * `ControllerTransfer` - When tokens are successfully transferred.
        ///
        /// # Errors
        /// * `UnexpectedNonFungibleToken` - If the asset is a non-fungible token.
        /// * `InvalidGranularity` - If the amount to transfer does not meet the granularity requirements.
        /// * `TotalSupplyOverflow` - If the total supply exceeds the maximum allowed limit.
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::controller_transfer())]
        pub fn controller_transfer(
            origin: OriginFor<T>,
            asset_id: AssetId,
            value: Balance,
            source: AssetHolder,
            destination_kind: AssetHolderKind,
        ) -> DispatchResult {
            let mut weight_meter = WeightMeter::max_limit_no_minimum();
            Self::base_controller_transfer(
                origin,
                asset_id,
                value,
                source,
                destination_kind,
                &mut weight_meter,
            )
        }

        /// Registers a custom asset type.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `ty` - Contains the string representation of the asset type.
        ///
        /// # Events
        /// * `CustomAssetTypeRegistered` - When a new custom asset type is successfully registered.
        /// * `CustomAssetTypeAlreadyRegistered` - When the custom asset type is already registered.
        ///
        /// # Errors
        /// * `TooLong` - If the custom asset type length exceeds the maximum allowed length.
        #[pallet::call_index(15)]
        #[pallet::weight(<T as Config>::WeightInfo::register_custom_asset_type(ty.len() as u32))]
        pub fn register_custom_asset_type(origin: OriginFor<T>, ty: Vec<u8>) -> DispatchResult {
            Self::base_register_custom_asset_type(origin, ty)?;
            Ok(())
        }

        /// Creates a new asset with a new custom asset type.
        ///
        /// The total supply will initially be zero. To mint tokens, use [`Pallet::issue`].
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_name` - The [`AssetName`] of the new asset.
        /// * `divisible` - Sets [`AssetDetails::divisible`], where `true` means the asset is divisible.
        /// * `custom_asset_type` - The custom asset type of the asset.
        /// * `asset_identifiers` - A vector of [`AssetIdentifier`].
        /// * `funding_round_name` - The name of the funding round ([`FundingRoundName`]).
        ///
        /// # Events
        /// * `AssetCreated` - When a new asset is successfully created.
        /// * `CustomAssetTypeRegistered` - When a new custom asset type is successfully registered.
        /// * `CustomAssetTypeAlreadyRegistered` - When the custom asset type is already registered.
        ///
        /// # Errors
        /// * `MaxLengthOfAssetNameExceeded` - If the asset name length exceeds the maximum allowed length.
        /// * `InvalidCustomAssetTypeId` - If the custom asset type ID is invalid.
        /// * `InvalidAssetIdentifier` - If any of the asset identifiers are invalid.
        /// * `TooLong` - If the custom asset type length exceeds the maximum allowed length.
        #[pallet::call_index(16)]
        #[pallet::weight(<T as Config>::WeightInfo::create_asset(
            asset_name.len() as u32,
            asset_identifiers.len() as u32,
            funding_round_name.as_ref().map_or(0, |name| name.len()) as u32,
        )
        .saturating_add(<T as Config>::WeightInfo::register_custom_asset_type(
            custom_asset_type.len() as u32,
        )))]
        pub fn create_asset_with_custom_type(
            origin: OriginFor<T>,
            asset_name: AssetName,
            divisible: bool,
            custom_asset_type: Vec<u8>,
            asset_identifiers: Vec<AssetIdentifier>,
            funding_round_name: Option<FundingRoundName>,
        ) -> DispatchResult {
            Self::base_create_asset_with_custom_type(
                origin,
                asset_name,
                divisible,
                custom_asset_type,
                asset_identifiers,
                funding_round_name,
            )?;
            Ok(())
        }

        /// Set asset metadata value.
        ///
        /// This function allows the asset issuer or an external agent to set metadata for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `key` - The [`AssetMetadataKey`] associated to the asset.
        /// * `value` - The [`AssetMetadataValue`] of the given metadata key.
        /// * `details` - Optional [`AssetMetadataValueDetail`] (expire, lock status).
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `SetAssetMetadataValue` - When the asset metadata value is successfully set.
        ///
        /// # Errors
        /// * `AssetMetadataKeyIsMissing` - If the metadata key is missing.
        /// * `AssetMetadataValueIsLocked` - If the metadata value is locked.
        /// * `AssetMetadataValueMaxLengthExceeded` - If the metadata value length exceeds the maximum allowed length.
        #[pallet::call_index(17)]
        #[pallet::weight(<T as Config>::WeightInfo::set_asset_metadata())]
        pub fn set_asset_metadata(
            origin: OriginFor<T>,
            asset_id: AssetId,
            key: AssetMetadataKey,
            value: AssetMetadataValue,
            detail: Option<AssetMetadataValueDetail<T::Moment>>,
        ) -> DispatchResult {
            Self::base_set_asset_metadata(origin, asset_id, key, value, detail)
        }

        /// Set asset metadata value details (expire, lock status).
        ///
        /// This function allows the asset issuer or an external agent to set metadata details for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `key` - The [`AssetMetadataKey`] associated to the asset.
        /// * `details` - The [`AssetMetadataValueDetail`] (expire, lock status) that will be associated to the asset.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `SetAssetMetadataValueDetails` - When the asset metadata value details are successfully set.
        ///
        /// # Errors
        /// * `AssetMetadataKeyIsMissing` - If the metadata key is missing.
        /// * `AssetMetadataValueIsLocked` - If the metadata value is locked.
        /// * `AssetMetadataValueIsEmpty` - If the metadata value is empty.
        #[pallet::call_index(18)]
        #[pallet::weight(<T as Config>::WeightInfo::set_asset_metadata_details())]
        pub fn set_asset_metadata_details(
            origin: OriginFor<T>,
            asset_id: AssetId,
            key: AssetMetadataKey,
            detail: AssetMetadataValueDetail<T::Moment>,
        ) -> DispatchResult {
            Self::base_set_asset_metadata_details(origin, asset_id, key, detail)
        }

        /// Registers and set local asset metadata.
        ///
        /// This function allows the asset issuer or an external agent to register and set local metadata for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `name` - The [`AssetMetadataName`].
        /// * `spec` - The asset metadata specifications ([`AssetMetadataSpec`]).
        /// * `value` - The [`AssetMetadataValue`] of the given metadata key.
        /// * `details` - Optional [`AssetMetadataValueDetail`] (expire, lock status).
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `RegisterAssetMetadataLocalType` - When the local asset metadata type is successfully registered.
        /// * `SetAssetMetadataValue` - When the asset metadata value is successfully set.
        ///
        /// # Errors
        /// * `AssetMetadataLocalKeyAlreadyExists` - If the local metadata key already exists.
        /// * `AssetMetadataValueIsLocked` - If the metadata value is locked.
        /// * `AssetMetadataValueMaxLengthExceeded` - If the metadata value length exceeds the maximum allowed length.
        #[pallet::call_index(19)]
        #[pallet::weight(<T as Config>::WeightInfo::register_and_set_local_asset_metadata())]
        pub fn register_and_set_local_asset_metadata(
            origin: OriginFor<T>,
            asset_id: AssetId,
            name: AssetMetadataName,
            spec: AssetMetadataSpec,
            value: AssetMetadataValue,
            detail: Option<AssetMetadataValueDetail<T::Moment>>,
        ) -> DispatchResult {
            Self::base_register_and_set_local_asset_metadata(
                origin, asset_id, name, spec, value, detail,
            )
        }

        /// Registers asset metadata local type.
        ///
        /// This function allows the asset issuer or an external agent to register a local metadata type for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `name` - The [`AssetMetadataName`].
        /// * `spec` - The asset metadata specifications ([`AssetMetadataSpec`]).
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `RegisterAssetMetadataLocalType` - When the local asset metadata type is successfully registered.
        ///
        /// # Errors
        /// * `AssetMetadataLocalKeyAlreadyExists` - If the local metadata key already exists.
        /// * `AssetMetadataNameMaxLengthExceeded` - If the metadata name length exceeds the maximum allowed length.
        /// * `AssetMetadataTypeDefMaxLengthExceeded` - If the metadata type definition length exceeds the maximum allowed length.
        #[pallet::call_index(20)]
        #[pallet::weight(<T as Config>::WeightInfo::register_asset_metadata_local_type())]
        pub fn register_asset_metadata_local_type(
            origin: OriginFor<T>,
            asset_id: AssetId,
            name: AssetMetadataName,
            spec: AssetMetadataSpec,
        ) -> DispatchResult {
            Self::base_register_asset_metadata_local_type(origin, asset_id, name, spec)
        }

        /// Registers asset metadata global type.
        ///
        /// This function allows the root origin to register a global metadata type.
        ///
        /// # Arguments
        /// * `origin` - The root origin.
        /// * `name` - The [`AssetMetadataName`].
        /// * `spec` - The asset metadata specifications ([`AssetMetadataSpec`]).
        ///
        /// # Events
        /// * `RegisterAssetMetadataGlobalType` - When the global asset metadata type is successfully registered.
        ///
        /// # Errors
        /// * `AssetMetadataGlobalKeyAlreadyExists` - If the global metadata key already exists.
        /// * `AssetMetadataNameMaxLengthExceeded` - If the metadata name length exceeds the maximum allowed length.
        /// * `AssetMetadataTypeDefMaxLengthExceeded` - If the metadata type definition length exceeds the maximum allowed length.
        #[pallet::call_index(21)]
        #[pallet::weight(<T as Config>::WeightInfo::register_asset_metadata_global_type())]
        pub fn register_asset_metadata_global_type(
            origin: OriginFor<T>,
            name: AssetMetadataName,
            spec: AssetMetadataSpec,
        ) -> DispatchResult {
            // Only allow global metadata types to be registered by root.
            ensure_root(origin)?;

            Self::base_register_asset_metadata_global_type(name, spec)
        }

        /// Updates the type of an asset.
        ///
        /// This function allows the asset issuer or an external agent to update the type of an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `asset_type` - The new [`AssetType`] of the asset.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `AssetTypeChanged` - When the asset type is successfully changed.
        ///
        /// # Errors
        /// * `NoSuchAsset` - If the asset does not exist.
        /// * `InvalidCustomAssetTypeId` - If the custom asset type ID is invalid.
        /// * `IncompatibleAssetTypeUpdate` - If the new asset type is incompatible with the existing asset type.
        #[pallet::call_index(22)]
        #[pallet::weight(<T as Config>::WeightInfo::update_asset_type())]
        pub fn update_asset_type(
            origin: OriginFor<T>,
            asset_id: AssetId,
            asset_type: AssetType,
        ) -> DispatchResult {
            Self::base_update_asset_type(origin, asset_id, asset_type)
        }

        /// Removes the asset metadata key and value of a local key.
        ///
        /// This function allows the asset issuer or an external agent to remove a local metadata key and its value for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the local metadata key.
        /// * `local_key` - The [`AssetMetadataLocalKey`] that will be removed.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `LocalMetadataKeyDeleted` - When the local metadata key is successfully deleted.
        ///
        /// # Errors
        /// * `AssetMetadataKeyIsMissing` - If the local metadata key is missing.
        /// * `AssetMetadataValueIsLocked` - If the metadata value is locked.
        /// * `AssetMetadataKeyBelongsToNFTCollection` - If the metadata key belongs to an NFT collection.
        #[pallet::call_index(23)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_local_metadata_key())]
        pub fn remove_local_metadata_key(
            origin: OriginFor<T>,
            asset_id: AssetId,
            local_key: AssetMetadataLocalKey,
        ) -> DispatchResult {
            Self::base_remove_local_metadata_key(origin, asset_id, local_key)
        }

        /// Removes the asset metadata value of a metadata key.
        ///
        /// This function allows the asset issuer or an external agent to remove a metadata value for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] associated to the metadata key.
        /// * `metadata_key` - The [`AssetMetadataKey`] that will have its value deleted.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `MetadataValueDeleted` - When the metadata value is successfully deleted.
        ///
        /// # Errors
        /// * `AssetMetadataKeyIsMissing` - If the metadata key is missing.
        /// * `AssetMetadataValueIsLocked` - If the metadata value is locked.
        #[pallet::call_index(24)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_metadata_value())]
        pub fn remove_metadata_value(
            origin: OriginFor<T>,
            asset_id: AssetId,
            metadata_key: AssetMetadataKey,
        ) -> DispatchResult {
            Self::base_remove_metadata_value(origin, asset_id, metadata_key)
        }

        /// Pre-approves the receivement of the asset for all identities.
        ///
        /// This function allows the root origin to pre-approve the receivement of an asset for all identities.
        ///
        /// # Arguments
        /// * `origin` - The root origin.
        /// * `asset_id` - The [`AssetId`] that will be exempt from affirmation.
        ///
        /// # Events
        /// * `AssetAffirmationExemption` - When the asset is successfully exempted from affirmation.
        #[pallet::call_index(25)]
        #[pallet::weight(<T as Config>::WeightInfo::exempt_asset_affirmation())]
        pub fn exempt_asset_affirmation(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
            Self::base_exempt_asset_affirmation(origin, asset_id)
        }

        /// Removes the pre-approval of the asset for all identities.
        ///
        /// This function allows the root origin to remove the pre-approval of an asset for all identities.
        ///
        /// # Arguments
        /// * `origin` - The root origin.
        /// * `asset_id` - The [`AssetId`] that will have its exemption removed.
        ///
        /// # Events
        /// * `RemoveAssetAffirmationExemption` - When the asset's affirmation exemption is successfully removed.
        #[pallet::call_index(26)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_asset_affirmation_exemption())]
        pub fn remove_asset_affirmation_exemption(
            origin: OriginFor<T>,
            asset_id: AssetId,
        ) -> DispatchResult {
            Self::base_remove_asset_affirmation_exemption(origin, asset_id)
        }

        /// Pre-approves the receivement of an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] that will be exempt from affirmation.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `PreApprovedAsset` - When the asset is successfully pre-approved for receivement.
        #[pallet::call_index(27)]
        #[pallet::weight(<T as Config>::WeightInfo::pre_approve_asset())]
        pub fn pre_approve_asset(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
            Self::base_pre_approve_asset(origin, asset_id)
        }

        /// Removes the pre-approval of an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] that will have its exemption removed.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `RemovePreApprovedAsset` - When the asset's pre-approval is successfully removed.
        #[pallet::call_index(28)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_asset_pre_approval())]
        pub fn remove_asset_pre_approval(
            origin: OriginFor<T>,
            asset_id: AssetId,
        ) -> DispatchResult {
            Self::base_remove_asset_pre_approval(origin, asset_id)
        }

        /// Sets all identities in the `mediators` set as mandatory mediators for any instruction transferring `asset_id`.
        ///
        /// This function allows the asset issuer or an external agent to add mandatory mediators for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] of the asset that will require the mediators.
        /// * `mediators` - A set of [`IdentityId`] of all the mandatory mediators for the given ticker.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `AssetMediatorsAdded` - When the mandatory mediators are successfully added.
        ///
        /// # Errors
        /// * `NumberOfAssetMediatorsExceeded` - If the number of mandatory mediators exceeds the maximum allowed limit.
        #[pallet::call_index(29)]
        #[pallet::weight(<T as Config>::WeightInfo::add_mandatory_mediators(mediators.len() as u32))]
        pub fn add_mandatory_mediators(
            origin: OriginFor<T>,
            asset_id: AssetId,
            mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
        ) -> DispatchResult {
            Self::base_add_mandatory_mediators(origin, asset_id, mediators)?;
            Ok(())
        }

        /// Removes all identities in the `mediators` set from the mandatory mediators list for the given `asset_id`.
        ///
        /// This function allows the asset issuer or an external agent to remove mandatory mediators for an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_id` - The [`AssetId`] of the asset that will have mediators removed.
        /// * `mediators` - A set of [`IdentityId`] of all the mediators that will be removed from the mandatory mediators list.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `AssetMediatorsRemoved` - When the mandatory mediators are successfully removed.
        #[pallet::call_index(30)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_mandatory_mediators(mediators.len() as u32))]
        pub fn remove_mandatory_mediators(
            origin: OriginFor<T>,
            asset_id: AssetId,
            mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
        ) -> DispatchResult {
            Self::base_remove_mandatory_mediators(origin, asset_id, mediators)?;
            Ok(())
        }

        /// Establishes a connection between a ticker and an AssetId.
        ///
        /// This function allows the asset issuer or an external agent to link a ticker to an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `ticker` - The [`Ticker`] that will be linked to the given `asset_id`.
        /// * `asset_id` - The [`AssetId`] that will be connected to `ticker`.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `TickerLinkedToAsset` - When the ticker is successfully linked to the asset.
        ///
        /// # Errors
        /// * `TickerNotRegisteredToCaller` - If the ticker is not registered to the caller.
        /// * `TickerRegistrationExpired` - If the ticker registration has expired.
        /// * `TickerRegistrationNotFound` - If the ticker registration is not found.
        /// * `TickerIsAlreadyLinkedToAnAsset` - If the ticker is already linked to an asset.
        /// * `AssetIsAlreadyLinkedToATicker` - If the asset is already linked to a ticker.
        #[pallet::call_index(31)]
        #[pallet::weight(<T as Config>::WeightInfo::link_ticker_to_asset_id())]
        pub fn link_ticker_to_asset_id(
            origin: OriginFor<T>,
            ticker: Ticker,
            asset_id: AssetId,
        ) -> DispatchResult {
            Self::base_link_ticker_to_asset_id(origin, ticker, asset_id)?;
            Ok(())
        }

        /// Removes the link between a ticker and an asset.
        ///
        /// This function allows the asset issuer or an external agent to unlink a ticker from an asset.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `ticker` - The [`Ticker`] that will be unlinked from the given `asset_id`.
        /// * `asset_id` - The [`AssetId`] that will be unlinked from `ticker`.
        ///
        /// # Permissions
        /// * Asset
        ///
        /// # Events
        /// * `TickerUnlinkedFromAsset` - When the ticker is successfully unlinked from the asset.
        ///
        /// # Errors
        /// * `TickerNotRegisteredToCaller` - If the ticker is not registered to the caller.
        /// * `TickerRegistrationNotFound` - If the ticker registration is not found.
        /// * `TickerIsNotLinkedToTheAsset` - If the ticker is not linked to the asset.
        #[pallet::call_index(32)]
        #[pallet::weight(<T as Config>::WeightInfo::unlink_ticker_from_asset_id())]
        pub fn unlink_ticker_from_asset_id(
            origin: OriginFor<T>,
            ticker: Ticker,
            asset_id: AssetId,
        ) -> DispatchResult {
            Self::base_unlink_ticker_from_asset_id(origin, ticker, asset_id)?;
            Ok(())
        }

        /// Updates the global metadata specification.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
        /// * `asset_metadata_name` - The [`AssetMetadataName`] associated with the global metadata.
        /// * `asset_metadata_spec` - The new [`AssetMetadataSpec`] that will be associated with the global metadata.
        ///
        /// # Events
        /// * `GlobalMetadataSpecUpdated` - When the global metadata specification is successfully updated.
        ///
        /// # Errors
        /// * `BadOrigin` - If the origin is not authorized.
        /// * `TooLong` - If the metadata url or description length exceeds the maximum allowed length.
        /// * `AssetMetadataTypeDefMaxLengthExceeded` - If the metadata type definition length exceeds the maximum allowed length.
        #[pallet::call_index(33)]
        #[pallet::weight(<T as Config>::WeightInfo::update_global_metadata_spec())]
        pub fn update_global_metadata_spec(
            origin: OriginFor<T>,
            asset_metadata_name: AssetMetadataName,
            asset_metadata_spec: AssetMetadataSpec,
        ) -> DispatchResult {
            Self::base_update_global_metadata_spec(origin, asset_metadata_name, asset_metadata_spec)
        }

        /// Transfer assets from the caller's default portfolio to the target address's default portfolio.
        ///
        /// The settlement engine is used to perform the asset transfer.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which will be the sender of the assets.
        /// * `asset_id` - The [`AssetId`] associated to the asset.
        /// * `to` - The target address to which the assets will be sent.
        /// * `amount` - The [`Balance`] of tokens that will be transferred.
        /// * `memo` - An optional [`Memo`] that can be attached to the transfer instruction.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        ///
        /// # Events
        /// * `CreatedAssetTransfer` - When an asset transfer instruction is created.
        /// * `InstructionCreated` - The asset transfer settlement was created.
        /// * `InstructionAffirmed` - The asset transfer settlement was affirmed by the caller as the sender.
        /// * `InstructionAutomaticallyAffirmed` - If the receiver pre-approved the asset, the instruction is automatically affirmed for the receiver.
        /// * `InstructionExecuted` - The asset transfer settlement was executed successfully (if the receiver pre-approved the asset).
        ///
        /// # Errors
        /// * `InsufficientBalance` - If the sender's balance is not sufficient to cover the transfer amount.
        /// * `InvalidTransfer` - If the transfer validation check fails.
        /// * `ReceiverIdentityNotFound` - If the receiver's identity is not found.
        /// * `UnexpectedOFFChainAsset` - If the asset could not be found on-chain.
        /// * `MissingIdentity` - The caller doesn't have an identity.
        #[pallet::call_index(34)]
        #[pallet::weight(
            <T as Config>::SettlementFn::transfer_funds_weight_limit(
                None,
                &Fund { description: FundDescription::Fungible { asset_id: *asset_id, amount: *amount }, memo: memo.clone() },
            )
        )]
        pub fn transfer_asset(
            origin: OriginFor<T>,
            asset_id: AssetId,
            to: T::AccountId,
            amount: Balance,
            memo: Option<Memo>,
        ) -> DispatchResultWithPostInfo {
            Self::base_transfer_asset(
                origin,
                asset_id,
                to,
                amount,
                memo,
                #[cfg(feature = "runtime-benchmarks")]
                false,
            )
        }

        /// Receiver affirms a pending asset transfer.
        ///
        /// If someone tries to transfer asset to an account that requires receiver affirmations, then the receiver will need to affirm the transfer
        /// before the transfer is executed.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which will be the receiver of the assets.
        /// * `transfer_id` - The [`InstructionId`] associated to the pending transfer.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        ///
        /// # Events
        /// * `InstructionAffirmed` - The asset transfer settlement was affirmed by the caller as the receiver.
        /// * `InstructionExecuted` - The asset transfer settlement was executed successfully.
        /// * `AssetBalanceUpdated` - The asset balance was updated for both the sender and receiver portfolios.
        ///
        /// # Errors
        /// * `UnknownInstruction` - If the instruction associated to the given transfer ID does not exist.
        /// * `InvalidTransfer` - If the transfer validation check fails.
        #[pallet::call_index(35)]
        #[pallet::weight(
            <T as Config>::SettlementFn::receiver_affirm_transfer_and_try_execute_weight_meter(
                <T as Config>::WeightInfo::receiver_affirm_asset_transfer_base_weight(),
                AssetCount::new(1, 0, 0)
            ).limit()
        )]
        pub fn receiver_affirm_asset_transfer(
            origin: OriginFor<T>,
            transfer_id: InstructionId,
        ) -> DispatchResultWithPostInfo {
            Self::base_receiver_affirm_asset_transfer(
                origin,
                transfer_id,
                #[cfg(feature = "runtime-benchmarks")]
                false,
            )
        }

        /// Reject a pending asset transfer.
        ///
        /// If someone tries to transfer asset to an account that requires receiver affirmations, then the receiver can reject the transfer.
        /// The sender can also reject the transfer before the receiver affirms it.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which can be either the sender or receiver.
        /// * `transfer_id` - The [`InstructionId`] associated to the pending transfer.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        ///
        /// # Events
        /// * `InstructionRejected` - The asset transfer settlement was rejected by the caller.
        ///
        /// # Errors
        /// * `InvalidInstructionStatusForRejection` - Either the instruction doesn't exist or it has already been executed or rejected.
        #[pallet::call_index(36)]
        #[pallet::weight(<T as Config>::SettlementFn::reject_transfer_weight_meter(AssetCount::new(1, 0, 0)).limit())]
        pub fn reject_asset_transfer(
            origin: OriginFor<T>,
            transfer_id: InstructionId,
        ) -> DispatchResultWithPostInfo {
            Self::base_reject_asset_transfer(origin, transfer_id)
        }

        /// Set the allowance for `spender` to transfer up to `amount` of `asset_id` from
        /// the caller's account.
        ///
        /// Replaces any existing allowance for this (owner, spender, asset_id) combination.
        /// Setting `amount` to 0 revokes the allowance (removes the storage entry).
        /// Setting `amount` to `Balance::MAX` grants an unlimited allowance that is never
        /// decremented on spend.
        ///
        /// # Arguments
        /// * `origin` — Signed origin. Caller must have a registered DID.
        /// * `asset_id` — The asset for which the allowance is set.
        /// * `spender` — The AccountId authorized to spend.
        /// * `amount` — Maximum amount the spender may transfer. 0 = revoke. Balance::MAX = unlimited.
        ///
        /// # Errors
        /// * `BadOrigin` — Unsigned origin.
        /// * `MissingIdentity` — Caller's key is not linked to a DID.
        #[pallet::call_index(37)]
        #[pallet::weight(<T as Config>::WeightInfo::approve())]
        pub fn approve(
            origin: OriginFor<T>,
            asset_id: AssetId,
            spender: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            let caller_data = IdentityPallet::<T>::ensure_origin_call_permissions(origin)?;
            let owner = caller_data.sender;

            if amount == 0 {
                Allowances::<T>::remove((&owner, &spender, &asset_id));
            } else {
                Allowances::<T>::insert((&owner, &spender, &asset_id), amount);
            }

            Self::deposit_event(Event::Approval {
                owner,
                spender,
                asset_id,
                amount,
            });

            Ok(())
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The user is not authorized.
        Unauthorized,
        /// The token has already been created.
        AssetAlreadyCreated,
        /// The ticker length is over the limit.
        TickerTooLong,
        /// The ticker has non-alphanumeric parts.
        TickerNotAlphanumeric,
        /// The ticker is already registered to someone else.
        TickerAlreadyRegistered,
        /// The total supply is above the limit.
        TotalSupplyAboveLimit,
        /// No token associated to the given asset ID.
        NoSuchAsset,
        /// The token is already frozen.
        AlreadyFrozen,
        /// Not an owner of the token on Ethereum.
        NotAnOwner,
        /// An overflow while calculating the balance.
        BalanceOverflow,
        /// An overflow while calculating the total supply.
        TotalSupplyOverflow,
        /// An invalid granularity.
        InvalidGranularity,
        /// The asset must be frozen.
        NotFrozen,
        /// Transfer validation check failed.
        InvalidTransfer,
        /// The sender balance is not sufficient.
        InsufficientBalance,
        /// The token is already divisible.
        AssetAlreadyDivisible,
        /// An invalid Ethereum `EcdsaSignature`.
        InvalidEthereumSignature,
        /// Registration of ticker has expired.
        TickerRegistrationExpired,
        /// Transfers to self are not allowed
        SenderSameAsReceiver,
        /// The given Document does not exist.
        NoSuchDoc,
        /// Maximum length of asset name has been exceeded.
        MaxLengthOfAssetNameExceeded,
        /// Maximum length of the funding round name has been exceeded.
        FundingRoundNameMaxLengthExceeded,
        /// Some `AssetIdentifier` was invalid.
        InvalidAssetIdentifier,
        /// Investor Uniqueness claims are not allowed for this asset.
        InvestorUniquenessClaimNotAllowed,
        /// Invalid `CustomAssetTypeId`.
        InvalidCustomAssetTypeId,
        /// Maximum length of the asset metadata type name has been exceeded.
        AssetMetadataNameMaxLengthExceeded,
        /// Maximum length of the asset metadata value has been exceeded.
        AssetMetadataValueMaxLengthExceeded,
        /// Maximum length of the asset metadata type definition has been exceeded.
        AssetMetadataTypeDefMaxLengthExceeded,
        /// Asset Metadata key is missing.
        AssetMetadataKeyIsMissing,
        /// Asset Metadata value is locked.
        AssetMetadataValueIsLocked,
        /// Asset Metadata Local type already exists for asset.
        AssetMetadataLocalKeyAlreadyExists,
        /// Asset Metadata Global type already exists.
        AssetMetadataGlobalKeyAlreadyExists,
        /// Tickers should start with at least one valid byte.
        TickerFirstByteNotValid,
        /// Attempt to call an extrinsic that is only permitted for fungible tokens.
        UnexpectedNonFungibleToken,
        /// Attempt to update the type of a non fungible token to a fungible token or the other way around.
        IncompatibleAssetTypeUpdate,
        /// Attempt to delete a key that is needed for an NFT collection.
        AssetMetadataKeyBelongsToNFTCollection,
        /// Attempt to lock a metadata value that is empty.
        AssetMetadataValueIsEmpty,
        /// Number of asset mediators would exceed the maximum allowed.
        NumberOfAssetMediatorsExceeded,
        /// Invalid ticker character - valid set: A`..`Z` `0`..`9` `_` `-` `.` `/`.
        InvalidTickerCharacter,
        /// Failed to transfer the asset - asset is frozen.
        InvalidTransferFrozenAsset,
        /// Failed to transfer an NFT - compliance failed.
        InvalidTransferComplianceFailure,
        /// Failed to transfer the asset - receiver DID is not active.
        InvalidTransferInvalidReceiverDID,
        /// The ticker registration associated to the ticker was not found.
        TickerRegistrationNotFound,
        /// The given ticker is already linked to an asset.
        TickerIsAlreadyLinkedToAnAsset,
        /// An unexpected error when generating a new asset ID.
        AssetIdGenerationError,
        /// The ticker doesn't belong to the caller.
        TickerNotRegisteredToCaller,
        /// The given asset is already linked to a ticker.
        AssetIsAlreadyLinkedToATicker,
        /// The given ticker is not linked to the given asset.
        TickerIsNotLinkedToTheAsset,
        /// The extrinsic expected a different `AuthorizationType` than what the `data.auth_type()` is.
        BadAuthorizationType,
        /// The key does not have permission to access the account.
        UnauthorizedHolderKey,
        /// No key was found for the Identity
        KeyNotFoundForDid,
        /// Insufficient tokens are locked.
        InsufficientTokensLocked,
        /// The spender's allowance for this asset is insufficient for the requested transfer amount.
        InsufficientAllowance,
        /// Transfering ownership to the same owner is not allowed.
        SelfOwnershipTransferNotAllowed,
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
        fn update_global_metadata_spec() -> Weight;
        fn receiver_affirm_asset_transfer_base_weight() -> Weight;
        fn approve() -> Weight;
        fn spend_allowance() -> Weight;
    }
}

//==========================================================================
// All base functions!
//==========================================================================

impl<T: AssetConfig> Pallet<T> {
    /// Registers `ticker` to the caller.
    fn base_register_unique_ticker(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        let caller_did = IdentityPallet::<T>::ensure_perms(origin)?;

        let ticker_registration_config = TickerConfig::<T>::get();
        let ticker_registration_status = Self::validate_ticker_registration_rules(
            &ticker,
            &caller_did,
            ticker_registration_config.max_ticker_length,
        )?;

        let expiry = ticker_registration_config
            .registration_length
            .map(|x| <pallet_timestamp::Pallet<T>>::get() + x);
        Self::unverified_register_ticker(
            ticker,
            caller_did,
            expiry,
            ticker_registration_status.charge_fee(),
        )?;

        Ok(())
    }

    /// Accepts and executes the ticker transfer.
    fn base_accept_ticker_transfer(origin: T::RuntimeOrigin, auth_id: u64) -> DispatchResult {
        let to = IdentityPallet::<T>::ensure_perms(origin)?;
        <IdentityPallet<T>>::accept_auth_with(&to.into(), auth_id, |data, auth_by| {
            let ticker = extract_auth!(data, TransferTicker(t));

            Self::ensure_ticker_not_linked(&ticker)?;

            let ticker_registration = UniqueTickerRegistration::<T>::get(&ticker)
                .ok_or(Error::<T>::TickerRegistrationNotFound)?;

            <IdentityPallet<T>>::ensure_auth_by(auth_by, ticker_registration.owner)?;

            Self::transfer_ticker(ticker_registration, ticker, to);
            Ok(())
        })
    }

    /// Accept and process an asset ownership transfer.
    fn base_accept_asset_ownership_transfer(
        origin: T::RuntimeOrigin,
        auth_id: u64,
    ) -> DispatchResult {
        let caller_did = IdentityPallet::<T>::ensure_perms(origin)?;

        <IdentityPallet<T>>::accept_auth_with(&caller_did.into(), auth_id, |auth_data, auth_by| {
            let asset_id = extract_auth!(auth_data, TransferAssetOwnership(asset_id));

            let mut asset_details = Self::try_get_asset_details(&asset_id)?;

            ensure!(
                asset_details.owner_did != caller_did,
                Error::<T>::SelfOwnershipTransferNotAllowed
            );

            // Ensure the authorization was created by a permissioned agent.
            <ExternalAgents<T>>::ensure_agent_permissioned(&asset_id, auth_by)?;

            // If the asset is linked to a unique ticker, the ticker registration must be updated.
            if let Some(ticker) = AssetIdTicker::<T>::get(&asset_id) {
                let ticker_registration = UniqueTickerRegistration::<T>::try_get(&ticker)
                    .map_err(|_| Error::<T>::TickerRegistrationNotFound)?;
                Self::transfer_ticker(ticker_registration, ticker, caller_did);
            }

            // Updates asset ownership
            let previous_owner = asset_details.owner_did;
            SecurityTokensOwnedByUser::<T>::remove(previous_owner, asset_id);
            SecurityTokensOwnedByUser::<T>::insert(caller_did, asset_id, true);

            // Removes the old owner from the external agents list and adds the new owner as a full agent
            ExternalAgents::<T>::unchecked_add_agent(asset_id, caller_did, AgentGroup::Full)?;
            ExternalAgents::<T>::try_mutate_agents_group(asset_id, previous_owner, None)?;

            // Updates asset details.
            asset_details.owner_did = caller_did;
            Assets::<T>::insert(asset_id, asset_details);
            Self::deposit_event(Event::AssetOwnershipTransferred(
                caller_did,
                asset_id,
                previous_owner,
            ));
            Ok(())
        })
    }

    /// If all rules for creating an asset are being respected, creates a new [`AssetDetails`].
    /// See also [`Pallet::validate_asset_creation_rules`].
    fn base_create_asset(
        origin: T::RuntimeOrigin,
        asset_name: AssetName,
        divisible: bool,
        asset_type: AssetType,
        asset_identifiers: Vec<AssetIdentifier>,
        funding_round_name: Option<FundingRoundName>,
    ) -> Result<IdentityId, DispatchError> {
        let caller_data = IdentityPallet::<T>::ensure_origin_call_permissions(origin)?;
        let caller_primary_did = caller_data.primary_did;

        Self::validate_and_create_asset(
            caller_data,
            asset_name,
            divisible,
            asset_type,
            asset_identifiers,
            funding_round_name,
        )?;

        Ok(caller_primary_did)
    }

    /// Freezes or unfreezes transfers for the asset associated to `asset_id`.
    fn base_set_freeze(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        freeze: bool,
    ) -> DispatchResult {
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

        Self::ensure_asset_exists(&asset_id)?;

        if freeze {
            ensure!(
                Frozen::<T>::get(&asset_id) == false,
                Error::<T>::AlreadyFrozen
            );
            Frozen::<T>::insert(asset_id, true);
            Self::deposit_event(Event::AssetFrozen(caller_did, asset_id));
        } else {
            ensure!(Frozen::<T>::get(&asset_id) == true, Error::<T>::NotFrozen);
            Frozen::<T>::insert(asset_id, false);
            Self::deposit_event(Event::AssetUnfrozen(caller_did, asset_id));
        }

        Ok(())
    }

    /// If `asset_name` is valid, updates the [`AssetName`] of the underling asset given by `asset_id`.
    fn base_rename_asset(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        asset_name: AssetName,
    ) -> DispatchResult {
        Self::ensure_valid_asset_name(&asset_name)?;
        Self::ensure_asset_exists(&asset_id)?;

        let caller_did = ExternalAgents::<T>::ensure_perms(origin, &asset_id)?;

        AssetNames::<T>::insert(asset_id, asset_name.clone());
        Self::deposit_event(Event::AssetRenamed(caller_did, asset_id, asset_name));
        Ok(())
    }

    /// Issues `amount_to_issue` tokens of `asset_id` into the caller's account/portfolio.
    fn base_issue(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        amount_to_issue: Balance,
        asset_holder_kind: AssetHolderKind,
    ) -> DispatchResult {
        let caller_holding_ctx =
            Self::ensure_asset_and_holding_permissions(origin, asset_id, asset_holder_kind, false)?;
        let mut asset_details = Self::try_get_asset_details(&asset_id)?;
        Self::validate_issuance_rules(&asset_details, amount_to_issue)?;
        Self::unverified_issue_tokens(
            asset_id,
            &mut asset_details,
            caller_holding_ctx,
            amount_to_issue,
            true,
            &mut WeightMeter::max_limit_no_minimum(),
        )?;
        Ok(())
    }

    /// Reduces `value` tokens from `asset_holder_kind` and [`AssetDetails::total_supply`].
    fn base_redeem(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        value: Balance,
        asset_holder_kind: AssetHolderKind,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let caller_holding_ctx =
            Self::ensure_asset_and_holding_permissions(origin, asset_id, asset_holder_kind, true)?;

        let mut asset_details = Self::try_get_asset_details(&asset_id)?;
        ensure!(
            asset_details.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        let new_balance = Self::ensure_sufficient_balance(&caller_holding_ctx, &asset_id, value)?;
        Self::set_holders_balance(caller_holding_ctx.clone(), asset_id, new_balance)?;

        asset_details.total_supply = asset_details
            .total_supply
            .checked_sub(value)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;

        let holder_did = pallet_identity::Pallet::<T>::asset_holder_did(&caller_holding_ctx)?;
        Checkpoint::<T>::advance_update_balances(
            &asset_id,
            &[(holder_did, BalanceOf::<T>::get(asset_id, holder_did))],
        )?;

        let updated_did_balance = BalanceOf::<T>::get(asset_id, holder_did).saturating_sub(value);

        // Update identity balances and total supply
        BalanceOf::<T>::insert(asset_id, holder_did, updated_did_balance);
        Assets::<T>::insert(asset_id, asset_details);

        // Update statistic info.
        Statistics::<T>::update_asset_stats(
            asset_id,
            Some(&holder_did),
            None,
            Some(updated_did_balance),
            None,
            value,
            weight_meter,
        )?;

        Self::deposit_event(Event::AssetBalanceUpdated(
            holder_did,
            asset_id,
            value,
            Some(caller_holding_ctx),
            None,
            HoldingsUpdateReason::Redeemed,
        ));
        Ok(())
    }

    fn base_make_divisible(origin: T::RuntimeOrigin, asset_id: AssetId) -> DispatchResult {
        let caller_did = ExternalAgents::<T>::ensure_perms(origin, &asset_id)?;

        Assets::<T>::try_mutate(&asset_id, |asset_details| -> DispatchResult {
            let asset_details = asset_details.as_mut().ok_or(Error::<T>::NoSuchAsset)?;
            ensure!(
                asset_details.asset_type.is_fungible(),
                Error::<T>::UnexpectedNonFungibleToken
            );
            ensure!(!asset_details.divisible, Error::<T>::AssetAlreadyDivisible);
            asset_details.divisible = true;

            Self::deposit_event(Event::DivisibilityChanged(caller_did, asset_id, true));
            Ok(())
        })
    }

    fn base_add_documents(
        origin: T::RuntimeOrigin,
        docs: Vec<Document>,
        asset_id: AssetId,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

        // Ensure strings are limited.
        for doc in &docs {
            ensure_string_limited::<T>(&doc.uri)?;
            ensure_string_limited::<T>(&doc.name)?;
            ensure_opt_string_limited::<T>(doc.doc_type.as_deref())?;
        }

        // Ensure we can advance documents ID sequence by `len`.
        let pre = AssetDocumentsIdSequence::<T>::try_mutate(asset_id, |id| {
            id.0.checked_add(docs.len() as u32)
                .ok_or(CounterOverflow::<T>)
                .map(|new| mem::replace(id, DocumentId(new)))
        })?;

        // Charge fee.
        T::ProtocolFee::batch_charge_fee(ProtocolOp::AssetAddDocuments, docs.len())?;

        // Add the documents & emit events.
        for (id, doc) in (pre.0..).map(DocumentId).zip(docs) {
            AssetDocuments::<T>::insert(asset_id, id, doc.clone());
            Self::deposit_event(Event::DocumentAdded(did, asset_id, id, doc));
        }
        Ok(())
    }

    fn base_remove_documents(
        origin: T::RuntimeOrigin,
        docs_id: Vec<DocumentId>,
        asset_id: AssetId,
    ) -> DispatchResult {
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
        for doc_id in docs_id {
            AssetDocuments::<T>::remove(asset_id, doc_id);
            Self::deposit_event(Event::DocumentRemoved(caller_did, asset_id, doc_id));
        }
        Ok(())
    }

    fn base_set_funding_round(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        funding_round_name: FundingRoundName,
    ) -> DispatchResult {
        Self::ensure_valid_funding_round_name(&funding_round_name)?;
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

        FundingRound::<T>::insert(asset_id, funding_round_name.clone());
        Self::deposit_event(Event::FundingRoundSet(
            caller_did,
            asset_id,
            funding_round_name,
        ));
        Ok(())
    }

    fn base_update_identifiers(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        asset_identifiers: Vec<AssetIdentifier>,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
        Self::ensure_valid_asset_identifiers(&asset_identifiers)?;
        Self::unverified_update_asset_identifiers(did, asset_id, asset_identifiers);
        Ok(())
    }

    fn base_controller_transfer(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        transfer_value: Balance,
        source: AssetHolder,
        destination_kind: AssetHolderKind,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let destination =
            Self::ensure_asset_and_holding_permissions(origin, asset_id, destination_kind, false)?;

        let holder_did = pallet_identity::Pallet::<T>::asset_holder_did(&destination)?;

        Self::validate_asset_transfer(
            asset_id,
            &source,
            &destination,
            transfer_value,
            true,
            weight_meter,
        )?;
        Self::unverified_transfer_asset(
            source.clone(),
            destination,
            asset_id,
            transfer_value,
            None,
            None,
            holder_did,
            weight_meter,
        )?;

        Self::deposit_event(Event::ControllerTransfer(
            holder_did,
            asset_id,
            source,
            transfer_value,
        ));
        Ok(())
    }

    /// Registers a new custom asset type.
    fn base_register_custom_asset_type(
        origin: T::RuntimeOrigin,
        asset_type_bytes: Vec<u8>,
    ) -> DispatchResult {
        let caller_did = IdentityPallet::<T>::ensure_perms(origin)?;
        Self::validate_custom_asset_type_rules(&asset_type_bytes)?;

        Self::unverified_register_custom_asset_type(caller_did, asset_type_bytes)?;
        Ok(())
    }

    /// Creates an asset with [`AssetType::Custom`].
    fn base_create_asset_with_custom_type(
        origin: T::RuntimeOrigin,
        asset_name: AssetName,
        divisible: bool,
        asset_type_bytes: Vec<u8>,
        asset_identifiers: Vec<AssetIdentifier>,
        funding_round_name: Option<FundingRoundName>,
    ) -> DispatchResult {
        let caller_data = IdentityPallet::<T>::ensure_origin_call_permissions(origin)?;

        Self::validate_custom_asset_type_rules(&asset_type_bytes)?;
        let custom_asset_type_id =
            Self::unverified_register_custom_asset_type(caller_data.primary_did, asset_type_bytes)?;
        let asset_type = AssetType::Custom(custom_asset_type_id);

        Self::validate_and_create_asset(
            caller_data,
            asset_name,
            divisible,
            asset_type,
            asset_identifiers,
            funding_round_name,
        )?;

        Ok(())
    }

    fn base_set_asset_metadata(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        key: AssetMetadataKey,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

        Self::unverified_set_asset_metadata(caller_did, asset_id, key, value, detail)
    }

    fn base_set_asset_metadata_details(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        key: AssetMetadataKey,
        detail: AssetMetadataValueDetail<T::Moment>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

        // Check key exists.
        ensure!(
            Self::check_asset_metadata_key_exists(&asset_id, &key),
            Error::<T>::AssetMetadataKeyIsMissing
        );

        // Check if value is currently locked.
        ensure!(
            !Self::is_asset_metadata_locked(&asset_id, key),
            Error::<T>::AssetMetadataValueIsLocked
        );

        // Prevent locking an asset metadata with no value
        if detail.is_locked(<pallet_timestamp::Pallet<T>>::get()) {
            AssetMetadataValues::<T>::try_get(&asset_id, &key)
                .map_err(|_| Error::<T>::AssetMetadataValueIsEmpty)?;
        }

        // Set asset metadata value details.
        AssetMetadataValueDetails::<T>::insert(asset_id, key, &detail);

        Self::deposit_event(Event::SetAssetMetadataValueDetails(did, asset_id, detail));
        Ok(())
    }

    fn base_register_and_set_local_asset_metadata(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

        // Register local metadata type.
        let key = Self::unverified_register_asset_metadata_local_type(did, asset_id, name, spec)?;

        Self::unverified_set_asset_metadata(did, asset_id, key, value, detail)
    }

    fn base_register_asset_metadata_local_type(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

        Self::unverified_register_asset_metadata_local_type(did, asset_id, name, spec).map(drop)
    }

    fn base_register_asset_metadata_global_type(
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        Self::ensure_asset_metadata_name_limited(&name)?;
        Self::ensure_asset_metadata_spec_limited(&spec)?;

        // Check if key already exists.
        ensure!(
            !AssetMetadataGlobalNameToKey::<T>::contains_key(&name),
            Error::<T>::AssetMetadataGlobalKeyAlreadyExists
        );

        // Next global key.
        let key = Self::update_current_asset_metadata_global_key()?;

        // Store global key <-> name mapping.
        AssetMetadataGlobalNameToKey::<T>::insert(&name, key);
        AssetMetadataGlobalKeyToName::<T>::insert(key, &name);

        // Store global specs.
        AssetMetadataGlobalSpecs::<T>::insert(key, &spec);

        Self::deposit_event(Event::RegisterAssetMetadataGlobalType(name, key, spec));
        Ok(())
    }

    fn base_update_asset_type(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        asset_type: AssetType,
    ) -> DispatchResult {
        Self::ensure_asset_exists(&asset_id)?;
        Self::ensure_valid_asset_type(&asset_type)?;
        let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
        Assets::<T>::try_mutate(&asset_id, |asset| -> DispatchResult {
            let asset = asset.as_mut().ok_or(Error::<T>::NoSuchAsset)?;
            // Ensures that both parameters are non fungible types or if both are fungible types.
            ensure!(
                asset.asset_type.is_fungible() == asset_type.is_fungible(),
                Error::<T>::IncompatibleAssetTypeUpdate
            );
            asset.asset_type = asset_type;
            Ok(())
        })?;
        Self::deposit_event(Event::AssetTypeChanged(did, asset_id, asset_type));
        Ok(())
    }

    fn base_remove_local_metadata_key(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        local_key: AssetMetadataLocalKey,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
        // Verifies if the key exists.
        let name = AssetMetadataLocalKeyToName::<T>::try_get(asset_id, &local_key)
            .map_err(|_| Error::<T>::AssetMetadataKeyIsMissing)?;
        // Verifies if the value is locked
        let metadata_key = AssetMetadataKey::Local(local_key);
        if let Some(value_detail) = AssetMetadataValueDetails::<T>::get(&asset_id, &metadata_key) {
            ensure!(
                !value_detail.is_locked(<pallet_timestamp::Pallet<T>>::get()),
                Error::<T>::AssetMetadataValueIsLocked
            );
        }
        // Verifies if the key belongs to an NFT collection
        ensure!(
            !T::NFTFn::is_collection_key(&asset_id, &metadata_key),
            Error::<T>::AssetMetadataKeyBelongsToNFTCollection
        );
        // Remove key from storage
        AssetMetadataValues::<T>::remove(&asset_id, &metadata_key);
        AssetMetadataValueDetails::<T>::remove(&asset_id, &metadata_key);
        AssetMetadataLocalNameToKey::<T>::remove(&asset_id, &name);
        AssetMetadataLocalKeyToName::<T>::remove(&asset_id, &local_key);
        AssetMetadataLocalSpecs::<T>::remove(&asset_id, &local_key);
        Self::deposit_event(Event::LocalMetadataKeyDeleted(
            caller_did, asset_id, local_key,
        ));
        Ok(())
    }

    fn base_remove_metadata_value(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        metadata_key: AssetMetadataKey,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
        // Verifies if the key exists.
        match metadata_key {
            AssetMetadataKey::Global(global_key) => {
                if !AssetMetadataGlobalKeyToName::<T>::contains_key(&global_key) {
                    return Err(Error::<T>::AssetMetadataKeyIsMissing.into());
                }
            }
            AssetMetadataKey::Local(local_key) => {
                if !AssetMetadataLocalKeyToName::<T>::contains_key(asset_id, &local_key) {
                    return Err(Error::<T>::AssetMetadataKeyIsMissing.into());
                }
            }
        }
        // Verifies if the value is locked
        if let Some(value_detail) = AssetMetadataValueDetails::<T>::get(&asset_id, &metadata_key) {
            ensure!(
                !value_detail.is_locked(<pallet_timestamp::Pallet<T>>::get()),
                Error::<T>::AssetMetadataValueIsLocked
            );
        }
        // Remove the metadata value from storage
        AssetMetadataValues::<T>::remove(&asset_id, &metadata_key);
        AssetMetadataValueDetails::<T>::remove(&asset_id, &metadata_key);
        Self::deposit_event(Event::MetadataValueDeleted(
            caller_did,
            asset_id,
            metadata_key,
        ));
        Ok(())
    }

    /// Pre-approves the receivement of the asset for all identities.
    fn base_exempt_asset_affirmation(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
    ) -> DispatchResult {
        ensure_root(origin)?;
        AssetsExemptFromAffirmation::<T>::insert(&asset_id, true);
        Self::deposit_event(Event::AssetAffirmationExemption(asset_id));
        Ok(())
    }

    /// Removes the pre-approval of the asset for all identities.
    fn base_remove_asset_affirmation_exemption(
        origin: T::RuntimeOrigin,
        assset_id: AssetId,
    ) -> DispatchResult {
        ensure_root(origin)?;
        AssetsExemptFromAffirmation::<T>::remove(&assset_id);
        Self::deposit_event(Event::RemoveAssetAffirmationExemption(assset_id));
        Ok(())
    }

    /// Pre-approves the receivement of an asset.
    fn base_pre_approve_asset(origin: T::RuntimeOrigin, asset_id: AssetId) -> DispatchResult {
        let caller_did = IdentityPallet::<T>::ensure_perms(origin)?;
        PreApprovedAsset::<T>::insert(&caller_did, &asset_id, true);
        Self::deposit_event(Event::PreApprovedAsset(caller_did, asset_id));
        Ok(())
    }

    /// Removes the pre approval of an asset.
    fn base_remove_asset_pre_approval(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
    ) -> DispatchResult {
        let caller_did = IdentityPallet::<T>::ensure_perms(origin)?;
        PreApprovedAsset::<T>::remove(&caller_did, &asset_id);
        Self::deposit_event(Event::RemovePreApprovedAsset(caller_did, asset_id));
        Ok(())
    }

    /// Sets all identities in the `mediators` set as mandatory mediators for any instruction transfering `asset_id`.
    fn base_add_mandatory_mediators(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        new_mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
        // Tries to add all new identities as mandatory mediators for the asset
        MandatoryMediators::<T>::try_mutate(asset_id, |mandatory_mediators| -> DispatchResult {
            for new_mediator in &new_mediators {
                mandatory_mediators
                    .try_insert(*new_mediator)
                    .map_err(|_| Error::<T>::NumberOfAssetMediatorsExceeded)?;
            }
            Ok(())
        })?;

        Self::deposit_event(Event::AssetMediatorsAdded(
            caller_did,
            asset_id,
            new_mediators.into_inner(),
        ));
        Ok(())
    }

    /// Removes all identities in the `mediators` set from the mandatory mediators list for the given `asset_id`.
    fn base_remove_mandatory_mediators(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
        // Removes the identities from the mandatory mediators list
        MandatoryMediators::<T>::mutate(asset_id, |mandatory_mediators| {
            for mediator in &mediators {
                mandatory_mediators.remove(mediator);
            }
        });
        Self::deposit_event(Event::AssetMediatorsRemoved(
            caller_did,
            asset_id,
            mediators.into_inner(),
        ));
        Ok(())
    }

    /// Transfers an asset from one identity portfolio to another
    pub fn base_transfer(
        from: AssetHolder,
        to: AssetHolder,
        asset_id: AssetId,
        transfer_value: Balance,
        instruction_id: Option<InstructionId>,
        instruction_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // NB: This function does not check if the sender/receiver have custodian permissions on the portfolios.
        // The custodian permissions must be checked before this function is called.
        // The only place this function is used right now is the settlement engine and the settlement engine
        // checks custodial permissions when the instruction is authorized.

        Self::validate_asset_transfer(asset_id, &from, &to, transfer_value, false, weight_meter)?;

        Self::unverified_transfer_asset(
            from,
            to,
            asset_id,
            transfer_value,
            instruction_id,
            instruction_memo,
            caller_did,
            weight_meter,
        )?;

        Ok(())
    }

    /// Check and decrement spender allowance for a fungible transfer.
    ///
    /// - `Balance::MAX` (infinite allowance): no storage write.
    /// - Depletes to zero: removes the storage entry.
    /// - Emits `AllowanceSpent` on every successful spend (including the infinite-allowance path).
    pub fn spend_allowance(
        owner: &T::AccountId,
        spender: &T::AccountId,
        asset_id: AssetId,
        amount: Balance,
    ) -> DispatchResult {
        let current = Allowances::<T>::get((owner, spender, &asset_id));
        ensure!(current >= amount, Error::<T>::InsufficientAllowance);

        // Infinite allowance — no deduction.
        let remaining_allowance = if current == Balance::MAX {
            Balance::MAX
        } else {
            let new_allowance = current.saturating_sub(amount);
            if new_allowance == 0 {
                Allowances::<T>::remove((owner, spender, &asset_id));
            } else {
                Allowances::<T>::insert((owner, spender, &asset_id), new_allowance);
            }
            new_allowance
        };

        Self::deposit_event(Event::AllowanceSpent {
            owner: owner.clone(),
            spender: spender.clone(),
            asset_id,
            amount_spent: amount,
            remaining_allowance,
        });
        Ok(())
    }

    pub fn base_link_ticker_to_asset_id(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        asset_id: AssetId,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
        // The caller must own the ticker and the ticker can't be expired
        UniqueTickerRegistration::<T>::try_mutate(
            ticker,
            |ticker_registration| -> DispatchResult {
                match ticker_registration {
                    Some(ticker_registration) => {
                        ensure!(
                            ticker_registration.owner == caller_did,
                            Error::<T>::TickerNotRegisteredToCaller
                        );
                        if let Some(ticker_expiry) = ticker_registration.expiry {
                            ensure!(
                                ticker_expiry > pallet_timestamp::Pallet::<T>::get(),
                                Error::<T>::TickerRegistrationExpired
                            );
                        }
                        ticker_registration.expiry = None;
                        Ok(())
                    }
                    None => Err(Error::<T>::TickerRegistrationNotFound.into()),
                }
            },
        )?;
        // The ticker can't be linked to any other asset
        Self::ensure_ticker_not_linked(&ticker)?;
        // The asset can't be linked to any other ticker
        ensure!(
            !AssetIdTicker::<T>::contains_key(asset_id),
            Error::<T>::AssetIsAlreadyLinkedToATicker
        );
        // Links the ticker to the asset
        TickerAssetId::<T>::insert(ticker, asset_id);
        AssetIdTicker::<T>::insert(asset_id, ticker);
        Self::deposit_event(Event::TickerLinkedToAsset(caller_did, ticker, asset_id));
        Ok(())
    }

    pub fn base_unlink_ticker_from_asset_id(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        asset_id: AssetId,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = ExternalAgents::<T>::ensure_perms(origin, &asset_id)?;

        // The caller must own the ticker
        let ticker_registration = UniqueTickerRegistration::<T>::take(ticker)
            .ok_or(Error::<T>::TickerRegistrationNotFound)?;
        ensure!(
            ticker_registration.owner == caller_did,
            Error::<T>::TickerNotRegisteredToCaller
        );

        // The ticker must be linked to the given asset
        ensure!(
            TickerAssetId::<T>::get(ticker) == Some(asset_id),
            Error::<T>::TickerIsNotLinkedToTheAsset
        );

        // Removes the storage links
        TickersOwnedByUser::<T>::remove(caller_did, ticker);
        TickerAssetId::<T>::remove(ticker);
        AssetIdTicker::<T>::remove(asset_id);
        Self::deposit_event(Event::TickerUnlinkedFromAsset(caller_did, ticker, asset_id));
        Ok(())
    }

    /// Updates the global metadata spec. This function is only callable by the root account.
    fn base_update_global_metadata_spec(
        origin: T::RuntimeOrigin,
        asset_metadata_name: AssetMetadataName,
        asset_metadata_spec: AssetMetadataSpec,
    ) -> DispatchResult {
        ensure_root(origin)?;

        let metadata_key = AssetMetadataGlobalNameToKey::<T>::get(&asset_metadata_name)
            .ok_or(Error::<T>::AssetMetadataKeyIsMissing)?;

        Self::ensure_asset_metadata_spec_limited(&asset_metadata_spec)?;

        AssetMetadataGlobalSpecs::<T>::try_mutate(metadata_key, |spec| -> DispatchResult {
            match spec {
                Some(spec) => {
                    *spec = asset_metadata_spec.clone();
                    Ok(())
                }
                None => Err(Error::<T>::AssetMetadataKeyIsMissing.into()),
            }
        })?;

        Self::deposit_event(Event::GlobalMetadataSpecUpdated(
            asset_metadata_name,
            asset_metadata_spec,
        ));

        Ok(())
    }

    pub fn base_transfer_asset(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        to: T::AccountId,
        amount: Balance,
        memo: Option<Memo>,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> DispatchResultWithPostInfo {
        let from = frame_system::ensure_signed(origin.clone())?;

        let to_holder = AssetHolder::try_from(to.encode())
            .map_err(|_| pallet_base::Error::<T>::InvalidAccountId)?;
        let fund = Fund {
            description: FundDescription::Fungible { asset_id, amount },
            memo: memo.clone(),
        };

        let mut weight_meter = WeightMeter::from_limit_unchecked(
            Weight::zero(),
            <T as Config>::SettlementFn::transfer_funds_weight_limit(None, &fund),
        );

        let instruction_id = T::SettlementFn::transfer_funds(
            origin,
            None,
            to_holder,
            fund,
            &mut weight_meter,
            #[cfg(feature = "runtime-benchmarks")]
            bench_base_weight,
        )?;

        Self::deposit_event(Event::CreatedAssetTransfer {
            asset_id,
            from,
            to,
            amount,
            memo,
            pending_transfer_id: instruction_id,
        });

        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }

    pub fn base_receiver_affirm_asset_transfer(
        origin: T::RuntimeOrigin,
        transfer_id: InstructionId,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> DispatchResultWithPostInfo {
        let asset_count = AssetCount::new(1, 0, 0);
        let mut weight_meter =
            <T as Config>::SettlementFn::receiver_affirm_transfer_and_try_execute_weight_meter(
                <T as Config>::WeightInfo::receiver_affirm_asset_transfer_base_weight(),
                asset_count,
            );

        // Affirm the transfer as the receiver and execute it.
        T::SettlementFn::receiver_affirm_transfer_and_try_execute(
            origin,
            transfer_id,
            asset_count,
            &mut weight_meter,
            #[cfg(feature = "runtime-benchmarks")]
            bench_base_weight,
        )?;

        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }

    pub fn base_reject_asset_transfer(
        origin: T::RuntimeOrigin,
        transfer_id: InstructionId,
    ) -> DispatchResultWithPostInfo {
        let asset_count = AssetCount::new(1, 0, 0);
        let mut weight_meter =
            <T as Config>::SettlementFn::reject_transfer_weight_meter(asset_count);

        // Reject the transfer.
        T::SettlementFn::reject_transfer(origin, transfer_id, asset_count, &mut weight_meter)?;

        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }
}

//==========================================================================
// All validattion functions!
//==========================================================================

impl<T: AssetConfig> Pallet<T> {
    /// Returns [`TickerRegistrationStatus`] if all registration rules are satisfied.
    fn validate_ticker_registration_rules(
        ticker: &Ticker,
        ticker_owner_did: &IdentityId,
        max_ticker_length: u8,
    ) -> Result<TickerRegistrationStatus, DispatchError> {
        Self::verify_ticker_characters(&ticker)?;

        Self::ensure_ticker_not_linked(&ticker)?;

        Self::ensure_ticker_length(ticker, max_ticker_length)?;

        let ticker_registration_status = Self::can_reregister_ticker(ticker, ticker_owner_did);

        if !ticker_registration_status.can_reregister() {
            return Err(Error::<T>::TickerAlreadyRegistered.into());
        }

        Ok(ticker_registration_status)
    }

    /// Returns `Ok` if the ticker contains only the following characters: `A`..`Z` `0`..`9` `_` `-` `.` `/`.
    pub fn verify_ticker_characters(ticker: &Ticker) -> DispatchResult {
        let ticker_bytes = ticker.as_ref();

        // The first byte of the ticker cannot be NULL
        if *ticker_bytes.first().unwrap_or(&0) == 0 {
            return Err(Error::<T>::TickerFirstByteNotValid.into());
        }

        // Allows the following characters: `A`..`Z` `0`..`9` `_` `-` `.` `/`
        let valid_characters = BTreeSet::from([b'_', b'-', b'.', b'/']);
        for (byte_index, ticker_byte) in ticker_bytes.iter().enumerate() {
            if !ticker_byte.is_ascii_uppercase()
                && !ticker_byte.is_ascii_digit()
                && !valid_characters.contains(ticker_byte)
            {
                if ticker_bytes[byte_index..].iter().all(|byte| *byte == 0) {
                    return Ok(());
                }

                return Err(Error::<T>::InvalidTickerCharacter.into());
            }
        }
        Ok(())
    }

    /// Returns `Ok` if `ticker` is not linked to an [`AssetId`]. Otherwise, returns [`Error::TickerIsAlreadyLinkedToAnAsset`].
    fn ensure_ticker_not_linked(ticker: &Ticker) -> DispatchResult {
        ensure!(
            !TickerAssetId::<T>::contains_key(ticker),
            Error::<T>::TickerIsAlreadyLinkedToAnAsset
        );
        Ok(())
    }

    /// Returns `Ok` if `ticker` length is less or equal to `max_ticker_length`. Otherwise, returns [`Error::<T>::TickerTooLong`].
    fn ensure_ticker_length(ticker: &Ticker, max_ticker_length: u8) -> DispatchResult {
        ensure!(
            ticker.len() <= max_ticker_length as usize,
            Error::<T>::TickerTooLong
        );
        Ok(())
    }

    /// Returns [`TickerRegistrationStatus`] containing information regarding whether the ticker can be registered and if the fee must be charged.
    fn can_reregister_ticker(ticker: &Ticker, caller_did: &IdentityId) -> TickerRegistrationStatus {
        match UniqueTickerRegistration::<T>::get(ticker) {
            Some(ticker_registration) => {
                // Checks if the ticker has an expiration time
                match ticker_registration.expiry {
                    Some(expiration_time) => {
                        // Checks if the registration has expired
                        if <pallet_timestamp::Pallet<T>>::get() > expiration_time {
                            return TickerRegistrationStatus::new(true, true);
                        }
                        // The ticker is still valid and was registered by the caller
                        if &ticker_registration.owner == caller_did {
                            return TickerRegistrationStatus::new(true, false);
                        }
                        // The ticker is still valid and was NOT registered by the caller
                        TickerRegistrationStatus::new(false, false)
                    }
                    None => {
                        // The ticker is still valid and was registered by the caller
                        if &ticker_registration.owner == caller_did {
                            return TickerRegistrationStatus::new(true, false);
                        }
                        // The ticker is still valid and was NOT registered by the caller
                        TickerRegistrationStatus::new(false, false)
                    }
                }
            }
            None => TickerRegistrationStatus::new(true, true),
        }
    }

    /// Returns [`TickerRegistrationStatus`] if all rules for creating an asset are satisfied.
    fn validate_asset_creation_rules(
        asset_id: &AssetId,
        asset_name: &AssetName,
        asset_type: &AssetType,
        funding_round_name: Option<&FundingRoundName>,
        asset_identifiers: &[AssetIdentifier],
    ) -> DispatchResult {
        if let Some(funding_round_name) = funding_round_name {
            Self::ensure_valid_funding_round_name(funding_round_name)?;
        }
        Self::ensure_valid_asset_name(asset_name)?;
        Self::ensure_valid_asset_type(asset_type)?;
        Self::ensure_valid_asset_identifiers(asset_identifiers)?;

        // Ensure there's no pre-existing entry for the `asset_id`
        Self::ensure_new_asset_id(asset_id)?;

        Ok(())
    }

    /// Returns the [`AssetDetails`] associated to `asset_id`, if one exists. Otherwise, returns [`Error::<T>::NoSuchAsset`].
    pub fn try_get_asset_details(asset_id: &AssetId) -> Result<AssetDetails, DispatchError> {
        let asset_details = Assets::<T>::try_get(asset_id).or(Err(Error::<T>::NoSuchAsset))?;
        Ok(asset_details)
    }

    /// Returns `Ok` if `funding_round_name` is valid. Otherwise, returns [`Error::<T>::FundingRoundNameMaxLengthExceeded`].
    fn ensure_valid_funding_round_name(funding_round_name: &FundingRoundName) -> DispatchResult {
        ensure!(
            funding_round_name.len() <= T::FundingRoundNameMaxLength::get() as usize,
            Error::<T>::FundingRoundNameMaxLengthExceeded
        );
        Ok(())
    }

    /// Returns `Ok` if `asset_name` is valid. Otherwise, returns [`Error::<T>::MaxLengthOfAssetNameExceeded`].
    fn ensure_valid_asset_name(asset_name: &AssetName) -> DispatchResult {
        ensure!(
            asset_name.len() <= T::AssetNameMaxLength::get() as usize,
            Error::<T>::MaxLengthOfAssetNameExceeded
        );
        Ok(())
    }

    /// Returns `Ok` if `asset_type` is valid. Otherwise, returns [`Error::<T>::InvalidCustomAssetTypeId`].
    fn ensure_valid_asset_type(asset_type: &AssetType) -> DispatchResult {
        if let AssetType::Custom(custom_type_id) = asset_type {
            ensure!(
                CustomTypes::<T>::contains_key(custom_type_id),
                Error::<T>::InvalidCustomAssetTypeId
            );
        }
        Ok(())
    }

    /// Returns `Ok` if all `asset_identifiers` are valid. Otherwise, returns [`Error::<T>::InvalidAssetIdentifier`].
    fn ensure_valid_asset_identifiers(asset_identifiers: &[AssetIdentifier]) -> DispatchResult {
        ensure!(
            asset_identifiers.iter().all(|i| i.is_valid()),
            Error::<T>::InvalidAssetIdentifier
        );
        Ok(())
    }

    /// Returns `Ok` if:
    ///  - `origin` is a permissioned agent for `asset_id`;
    ///  - If [`AssetHolderKind::DefaultPortfolio`], ensures the caller has permissions over the default portfolio;
    ///  - If [`AssetHolderKind::UserPortfolio`], ensures the portfolio exists and the caller has permissions over it;
    ///  - If `ensure_custody` is `true`, also ensures the caller has custody of the portfolio.
    ///  - If [`AssetHolderKind::Account`], ensures the caller has permissions over the account.
    pub fn ensure_asset_and_holding_permissions(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        asset_holder_kind: AssetHolderKind,
        ensure_custody: bool,
    ) -> Result<AssetHolder, DispatchError> {
        let origin_data = ExternalAgents::<T>::ensure_agent_asset_perms(origin, &asset_id)?;

        let holder = AssetHolder::try_from((
            origin_data.primary_did,
            origin_data.sender.encode(),
            asset_holder_kind,
        ))?;

        match &holder {
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::ensure_portfolio_validity(portfolio_id)?;

                if ensure_custody {
                    PortfolioPallet::<T>::ensure_portfolio_custody(portfolio_id, portfolio_id.did)?;
                }

                PortfolioPallet::<T>::ensure_user_portfolio_permission(
                    origin_data.secondary_key.as_ref(),
                    &portfolio_id,
                )?;

                Ok(holder)
            }
            AssetHolder::Account(account_id) => {
                Self::ensure_account_permissions(
                    account_id,
                    origin_data.secondary_key.as_ref(),
                    origin_data.primary_did,
                )?;
                Ok(holder)
            }
        }
    }

    /// Returns `Ok` if all rules for creating a custom type are satisfied.
    fn validate_custom_asset_type_rules(asset_type_bytes: &[u8]) -> DispatchResult {
        ensure_string_limited::<T>(asset_type_bytes)?;
        Ok(())
    }

    /// Returns `Ok` if [`AssetDetails::divisible`] or `value` % ONE_UNIT == 0. Otherwise, returns [`Error::<T>::InvalidGranularity`].
    fn ensure_asset_granular(asset_details: &AssetDetails, value: &Balance) -> DispatchResult {
        if asset_details.divisible || value % ONE_UNIT == 0 {
            return Ok(());
        }
        Err(Error::<T>::InvalidGranularity.into())
    }

    /// Returns `true` if [`AssetDetails::divisible`], otherwise returns `false`.
    pub fn is_divisible(asset_id: &AssetId) -> bool {
        Assets::<T>::get(asset_id)
            .map(|t| t.divisible)
            .unwrap_or_default()
    }

    pub fn check_asset_metadata_key_exists(asset_id: &AssetId, key: &AssetMetadataKey) -> bool {
        match key {
            AssetMetadataKey::Global(key) => AssetMetadataGlobalKeyToName::<T>::contains_key(key),
            AssetMetadataKey::Local(key) => {
                AssetMetadataLocalKeyToName::<T>::contains_key(asset_id, key)
            }
        }
    }

    fn is_asset_metadata_locked(asset_id: &AssetId, key: AssetMetadataKey) -> bool {
        AssetMetadataValueDetails::<T>::get(asset_id, key).map_or(false, |details| {
            details.is_locked(<pallet_timestamp::Pallet<T>>::get())
        })
    }

    /// Ensure asset metadata `value` is within the global limit.
    fn ensure_asset_metadata_value_limited(value: &AssetMetadataValue) -> DispatchResult {
        ensure!(
            value.len() <= T::AssetMetadataValueMaxLength::get() as usize,
            Error::<T>::AssetMetadataValueMaxLengthExceeded
        );
        Ok(())
    }

    /// Ensure asset metadata `name` is within the global limit.
    fn ensure_asset_metadata_name_limited(name: &AssetMetadataName) -> DispatchResult {
        ensure!(
            name.len() <= T::AssetMetadataNameMaxLength::get() as usize,
            Error::<T>::AssetMetadataNameMaxLengthExceeded
        );
        Ok(())
    }

    /// Ensure asset metadata `spec` is within the global limit.
    fn ensure_asset_metadata_spec_limited(spec: &AssetMetadataSpec) -> DispatchResult {
        ensure_opt_string_limited::<T>(spec.url.as_deref())?;
        ensure_opt_string_limited::<T>(spec.description.as_deref())?;
        if let Some(ref type_def) = spec.type_def {
            ensure!(
                type_def.len() <= T::AssetMetadataTypeDefMaxLength::get() as usize,
                Error::<T>::AssetMetadataTypeDefMaxLengthExceeded
            );
        }
        Ok(())
    }

    /// Returns `None` if there's no asset associated to the given asset_id,
    /// returns Some(true) if the asset exists and is of type `AssetType::NonFungible`, and returns Some(false) otherwise.
    pub fn nft_asset(asset_id: &AssetId) -> Option<bool> {
        let asset = Assets::<T>::try_get(asset_id).ok()?;
        Some(asset.asset_type.is_non_fungible())
    }

    /// Ensure that the document `doc` exists for `ticker`.
    pub fn ensure_doc_exists(asset_id: &AssetId, doc: &DocumentId) -> DispatchResult {
        ensure!(
            AssetDocuments::<T>::contains_key(asset_id, doc),
            Error::<T>::NoSuchDoc
        );
        Ok(())
    }

    pub fn get_balance_at(asset_id: AssetId, did: IdentityId, at: CheckpointId) -> Balance {
        <Checkpoint<T>>::balance_at(asset_id, did, at)
            .unwrap_or_else(|| BalanceOf::<T>::get(&asset_id, &did))
    }

    pub fn validate_asset_transfer(
        asset_id: AssetId,
        sender: &AssetHolder,
        receiver: &AssetHolder,
        transfer_value: Balance,
        is_controller_transfer: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let asset_details = Self::try_get_asset_details(&asset_id)?;
        ensure!(
            asset_details.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        let sender_did = pallet_identity::Pallet::<T>::asset_holder_did(&sender)?;
        let receiver_did = pallet_identity::Pallet::<T>::asset_holder_did(&receiver)?;

        ensure!(sender_did != receiver_did, Error::<T>::SenderSameAsReceiver);

        ensure!(
            BalanceOf::<T>::get(asset_id, &sender_did) >= transfer_value,
            Error::<T>::InsufficientBalance
        );
        ensure!(
            BalanceOf::<T>::get(asset_id, &receiver_did)
                .checked_add(transfer_value)
                .is_some(),
            Error::<T>::BalanceOverflow
        );

        // Verifies that both portfolios exist an that the sender portfolio has sufficient balance
        Self::ensure_valid_holdings(sender, receiver, &asset_id, transfer_value)?;

        // Controllers are exempt from statistics, compliance and frozen rules.
        if is_controller_transfer {
            return Ok(());
        }

        Self::ensure_asset_is_not_frozen(&asset_id)?;

        ensure!(
            IdentityPallet::<T>::is_did_active(receiver_did),
            Error::<T>::InvalidTransferInvalidReceiverDID
        );

        // Verifies that the statistics restrictions are satisfied
        Statistics::<T>::verify_transfer_restrictions(
            asset_id,
            &sender_did,
            &receiver_did,
            BalanceOf::<T>::get(asset_id, &sender_did),
            BalanceOf::<T>::get(asset_id, &receiver_did),
            transfer_value,
            asset_details.total_supply,
            weight_meter,
        )?;

        // Verifies that all compliance rules are being respected
        if !T::ComplianceManager::is_compliant(&asset_id, sender_did, receiver_did, weight_meter)? {
            return Err(Error::<T>::InvalidTransferComplianceFailure.into());
        }

        Ok(())
    }

    /// Returns `Ok` if the asset is not frozen.
    pub fn ensure_asset_is_not_frozen(asset_id: &AssetId) -> DispatchResult {
        ensure!(
            !Frozen::<T>::get(asset_id),
            Error::<T>::InvalidTransferFrozenAsset
        );
        Ok(())
    }

    /// Returns a vector containing all errors for the transfer. An empty vec means there's no error.
    pub fn asset_transfer_report(
        sender: &AssetHolder,
        receiver: &AssetHolder,
        asset_id: &AssetId,
        transfer_value: Balance,
        skip_locked_check: bool,
        weight_meter: &mut WeightMeter,
    ) -> Vec<DispatchError> {
        let mut asset_transfer_errors = Vec::new();

        // If the asset doesn't exist or is an NFT, there's no point in assessing anything else
        let asset_details = {
            match Assets::<T>::try_get(asset_id) {
                Ok(asset_details) => asset_details,
                Err(_) => return vec![Error::<T>::NoSuchAsset.into()],
            }
        };
        if !asset_details.asset_type.is_fungible() {
            return vec![Error::<T>::UnexpectedNonFungibleToken.into()];
        }

        let sender_did = {
            match pallet_identity::Pallet::<T>::asset_holder_did(sender) {
                Ok(did) => did,
                Err(e) => return vec![e.into()],
            }
        };

        let receiver_did = {
            match pallet_identity::Pallet::<T>::asset_holder_did(receiver) {
                Ok(did) => did,
                Err(e) => return vec![e.into()],
            }
        };

        if let Err(e) = Self::ensure_asset_granular(&asset_details, &transfer_value) {
            asset_transfer_errors.push(e);
        }

        let sender_current_balance = BalanceOf::<T>::get(&asset_id, &sender_did);
        if sender_current_balance < transfer_value {
            asset_transfer_errors.push(Error::<T>::InsufficientBalance.into());
        }

        let receiver_current_balance = BalanceOf::<T>::get(&asset_id, &receiver_did);
        if receiver_current_balance
            .checked_add(transfer_value)
            .is_none()
        {
            asset_transfer_errors.push(Error::<T>::BalanceOverflow.into());
        }

        if let AssetHolder::Portfolio(sender_portfolio_id) = sender {
            if let Err(e) = PortfolioPallet::<T>::ensure_portfolio_validity(sender_portfolio_id) {
                asset_transfer_errors.push(e);
            }
        }

        if let AssetHolder::Portfolio(receiver_portfolio_id) = receiver {
            if let Err(e) = PortfolioPallet::<T>::ensure_portfolio_validity(receiver_portfolio_id) {
                asset_transfer_errors.push(e);
            }
        }

        if skip_locked_check {
            if Self::get_holders_balance(sender, asset_id) < transfer_value {
                asset_transfer_errors.push(Error::<T>::InsufficientBalance.into());
            }
        } else {
            if let Err(e) = Self::ensure_sufficient_balance(sender, asset_id, transfer_value) {
                asset_transfer_errors.push(e);
            }
        }

        if !IdentityPallet::<T>::is_did_active(receiver_did) {
            asset_transfer_errors.push(Error::<T>::InvalidTransferInvalidReceiverDID.into());
        }

        if Frozen::<T>::get(asset_id) {
            asset_transfer_errors.push(Error::<T>::InvalidTransferFrozenAsset.into());
        }

        if let Err(e) = Statistics::<T>::verify_transfer_restrictions(
            *asset_id,
            &sender_did,
            &receiver_did,
            sender_current_balance,
            receiver_current_balance,
            transfer_value,
            asset_details.total_supply,
            weight_meter,
        ) {
            asset_transfer_errors.push(e);
        }

        match T::ComplianceManager::is_compliant(asset_id, sender_did, receiver_did, weight_meter) {
            Ok(is_compliant) => {
                if !is_compliant {
                    asset_transfer_errors.push(Error::<T>::InvalidTransferComplianceFailure.into());
                }
            }
            Err(e) => {
                asset_transfer_errors.push(e);
            }
        }

        asset_transfer_errors
    }

    /// Returns [`AssetDetails::total_supply`] for the given `asset_id`.
    pub fn total_supply(asset_id: &AssetId) -> Balance {
        Assets::<T>::get(asset_id)
            .map(|t| t.total_supply)
            .unwrap_or_default()
    }

    /// Calls [`Pallet::validate_asset_creation_rules`] and [`Pallet::unverified_create_asset`].
    fn validate_and_create_asset(
        caller_data: PermissionedCallOriginData<T::AccountId>,
        asset_name: AssetName,
        divisible: bool,
        asset_type: AssetType,
        asset_identifiers: Vec<AssetIdentifier>,
        funding_round_name: Option<FundingRoundName>,
    ) -> DispatchResult {
        let asset_id = Self::generate_asset_id(caller_data.sender, true);

        Self::validate_asset_creation_rules(
            &asset_id,
            &asset_name,
            &asset_type,
            funding_round_name.as_ref(),
            &asset_identifiers,
        )?;

        Self::unverified_create_asset(
            caller_data.primary_did,
            asset_id,
            divisible,
            asset_name,
            asset_type,
            funding_round_name,
            asset_identifiers,
        )?;

        Ok(())
    }

    /// Returns `Ok` if all rules for issuing a tokens are satisfied.
    fn validate_issuance_rules(
        asset_details: &AssetDetails,
        amount_to_issue: Balance,
    ) -> DispatchResult {
        ensure!(
            asset_details.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        Self::ensure_asset_granular(asset_details, &amount_to_issue)?;

        let new_supply = asset_details
            .total_supply
            .checked_add(amount_to_issue)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;
        ensure!(new_supply <= MAX_SUPPLY, Error::<T>::TotalSupplyAboveLimit);
        Ok(())
    }

    /// Returns `Ok` if there's no asset associated to `asset_id`. Otherwise, returns [`Error::<T>::AssetIdGenerationError`].
    fn ensure_new_asset_id(asset_id: &AssetId) -> DispatchResult {
        ensure!(
            !Assets::<T>::contains_key(asset_id),
            Error::<T>::AssetIdGenerationError
        );
        Ok(())
    }

    /// Returns `Ok` if there's a asset associated to `asset_id`. Otherwise, returns [`Error::<T>::NoSuchAsset`].
    fn ensure_asset_exists(asset_id: &AssetId) -> DispatchResult {
        ensure!(Assets::<T>::contains_key(asset_id), Error::<T>::NoSuchAsset);
        Ok(())
    }

    pub fn generate_asset_id(caller_acc: T::AccountId, update: bool) -> AssetId {
        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        let nonce = Self::get_nonce(&caller_acc, update);
        blake2_128(&(b"modlpy/pallet_asset", genesis_hash, caller_acc, nonce).encode()).into()
    }

    fn get_nonce(caller_acc: &T::AccountId, update: bool) -> u64 {
        let nonce = AssetNonce::<T>::get(caller_acc);

        if update {
            AssetNonce::<T>::insert(caller_acc, nonce.wrapping_add(1));
        }

        nonce
    }

    /// Returns the balance of `acc_id` for `asset_id`.
    fn get_account_balance(acc_id: &AccountId32, asset_id: &AssetId) -> Balance {
        AssetBalance::<T>::get(acc_id, asset_id)
    }

    /// Returns the locked balance of `acc_id` for `asset_id`.
    fn get_account_locked_balance(account: &AccountId32, asset_id: &AssetId) -> Balance {
        LockedBalance::<T>::get(account, asset_id)
    }

    /// Sets [`AssetBalance`] for `acc_id` and `asset_id` to `new_balance`.
    fn set_account_balance(
        acc_id: AccountId32,
        asset_id: AssetId,
        new_balance: Balance,
    ) -> DispatchResult {
        let old_balance = Self::get_account_balance(&acc_id, &asset_id);

        if new_balance.is_zero() {
            AssetBalance::<T>::remove(&acc_id, &asset_id);
            if old_balance > 0 {
                let acc_id = pallet_base::pallet_account_id::<T>(&acc_id)?;
                IdentityPallet::<T>::remove_account_key_ref_count(&acc_id);
            }
            return Ok(());
        }

        if old_balance.is_zero() {
            let acc_id = pallet_base::pallet_account_id::<T>(&acc_id)?;
            IdentityPallet::<T>::add_account_key_ref_count(&acc_id);
        }
        AssetBalance::<T>::insert(acc_id, asset_id, new_balance);
        Ok(())
    }

    /// Sets [`LockedBalance`] for `acc_id` and `asset_id` to `new_locked_balance`.
    fn set_account_locked_balance(
        acc_id: AccountId32,
        asset_id: AssetId,
        new_locked_balance: Balance,
    ) {
        if new_locked_balance.is_zero() {
            LockedBalance::<T>::remove(&acc_id, &asset_id);
        } else {
            LockedBalance::<T>::insert(acc_id, asset_id, new_locked_balance);
        }
    }

    /// If [`AssetHolder::Portfolio`], returns the balance from the portfolio pallet.
    /// If [`AssetHolder::Account`], returns the balance from the asset pallet.
    pub fn get_holders_balance(holder: &AssetHolder, asset_id: &AssetId) -> Balance {
        match holder {
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::get_portfolio_balance(portfolio_id, asset_id)
            }
            AssetHolder::Account(account_id) => Self::get_account_balance(account_id, asset_id),
        }
    }

    /// If [`AssetHolder::Portfolio`], returns the locked balance from the portfolio pallet.
    /// If [`AssetHolder::Account`], returns the locked balance from the asset pallet.
    fn get_holders_locked_balance(holder: &AssetHolder, asset_id: &AssetId) -> Balance {
        match holder {
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::get_portfolio_locked_balance(portfolio_id, asset_id)
            }
            AssetHolder::Account(account_id) => {
                Self::get_account_locked_balance(account_id, asset_id)
            }
        }
    }

    /// If [`AssetHolder::Portfolio`], updates the balance in the portfolio pallet.
    /// If [`AssetHolder::Account`], updates the balance in the asset pallet.
    fn set_holders_balance(
        asset_holder: AssetHolder,
        asset_id: AssetId,
        new_balance: Balance,
    ) -> DispatchResult {
        match asset_holder {
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::set_portfolio_balance(&portfolio_id, asset_id, new_balance);
                Ok(())
            }
            AssetHolder::Account(account_id) => {
                Self::set_account_balance(account_id, asset_id, new_balance)
            }
        }
    }

    /// If [`AssetHolder::Portfolio`], updates the locked balance in the portfolio pallet.
    /// If [`AssetHolder::Account`], updates the locked balance in the asset pallet.
    fn set_holders_locked_balance(
        asset_holder: AssetHolder,
        asset_id: AssetId,
        new_locked_balance: Balance,
    ) {
        match asset_holder {
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::set_portfolio_locked_balance(
                    portfolio_id,
                    asset_id,
                    new_locked_balance,
                );
            }
            AssetHolder::Account(account_id) => {
                Self::set_account_locked_balance(account_id, asset_id, new_locked_balance);
            }
        }
    }

    /// Adds `balance_to_add` to the locked balance of `asset_holder` for `asset_id`.
    pub fn add_locked_balance(
        asset_holder: AssetHolder,
        asset_id: AssetId,
        balance_to_add: Balance,
    ) -> DispatchResult {
        let _ = Self::ensure_sufficient_balance(&asset_holder, &asset_id, balance_to_add)?;
        match asset_holder {
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::unchecked_lock_tokens(portfolio_id, asset_id, balance_to_add);
            }
            AssetHolder::Account(account_id) => {
                LockedBalance::<T>::mutate(account_id, asset_id, |l| {
                    *l = l.saturating_add(balance_to_add)
                });
            }
        }
        Ok(())
    }

    /// Subtracts `balance_to_remove` from the locked balance of `asset_holder` for `asset_id`.
    pub fn remove_locked_balance(
        asset_holder: AssetHolder,
        asset_id: AssetId,
        balance_to_remove: Balance,
    ) -> DispatchResult {
        let current_locked_balance = Self::get_holders_locked_balance(&asset_holder, &asset_id);
        ensure!(
            current_locked_balance >= balance_to_remove,
            Error::<T>::InsufficientTokensLocked
        );
        Self::set_holders_locked_balance(
            asset_holder,
            asset_id,
            current_locked_balance - balance_to_remove,
        );
        Ok(())
    }

    /// Returns `Ok` if:
    /// - If the receiver portfolio exists;
    /// - The sender has sufficient balance for the transfer (taking into account locks).
    pub fn ensure_valid_holdings(
        sender: &AssetHolder,
        receiver: &AssetHolder,
        asset_id: &AssetId,
        value: Balance,
    ) -> DispatchResult {
        match receiver {
            AssetHolder::Portfolio(receiver_portfolio_id) => {
                PortfolioPallet::<T>::ensure_portfolio_validity(receiver_portfolio_id)?;
            }
            AssetHolder::Account(_) => {
                let _ = pallet_identity::Pallet::<T>::asset_holder_did(receiver)?;
            }
        }

        let _ = Self::ensure_sufficient_balance(sender, asset_id, value)?;

        Ok(())
    }

    /// If the asset is granular (see: [`Pallet::ensure_asset_granular`]) and the holder has at least
    /// `value` balance that is not locked, returns what would be the new balance of `holder` after a transfer of `value`.
    pub fn ensure_sufficient_balance(
        holder: &AssetHolder,
        asset_id: &AssetId,
        value: Balance,
    ) -> Result<Balance, DispatchError> {
        Self::ensure_granular(asset_id, value)?;

        let current_balance = Self::get_holders_balance(holder, asset_id);
        let locked_balance = Self::get_holders_locked_balance(holder, asset_id);

        let final_balance = current_balance
            .checked_sub(value)
            .ok_or(Error::<T>::InsufficientBalance)?;

        ensure!(
            final_balance >= locked_balance,
            Error::<T>::InsufficientBalance
        );

        Ok(final_balance)
    }

    /// Transfers `value` of `asset_id` from `sender` to `receiver`.
    pub fn transfer_holders_balance(
        sender: AssetHolder,
        receiver: AssetHolder,
        asset_id: AssetId,
        value: Balance,
    ) -> DispatchResult {
        let sender_old_balance = Self::get_holders_balance(&sender, &asset_id);
        let sender_new_balance = sender_old_balance.saturating_sub(value);
        Self::set_holders_balance(sender, asset_id, sender_new_balance)?;

        let rcv_old_balance = Self::get_holders_balance(&receiver, &asset_id);
        let rcv_new_balance = rcv_old_balance.saturating_add(value);
        Self::set_holders_balance(receiver, asset_id, rcv_new_balance)?;
        Ok(())
    }

    /// Returns `Ok` if:
    /// - Asset holder is a portfolio and the caller has permissions and custody over it; or
    /// - Asset holder is an account and the caller has permissions over it.
    pub fn ensure_holder_permissions(
        asset_holder: &AssetHolder,
        custodian_did: IdentityId,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
    ) -> DispatchResult {
        match asset_holder {
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::ensure_portfolio_custody_and_permission(
                    portfolio_id,
                    custodian_did,
                    secondary_key,
                )
            }
            AssetHolder::Account(account_id) => {
                Self::ensure_account_permissions(account_id, secondary_key, custodian_did)
            }
        }
    }

    /// Returns `Ok` if:
    /// - secondary key is provided and matches the account_id; or
    /// - the primary key of the DID matches the account_id.
    fn ensure_account_permissions(
        account_id: &AccountId32,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        primary_did: IdentityId,
    ) -> DispatchResult {
        let account_id = pallet_base::pallet_account_id::<T>(account_id)?;

        if let Some(sk) = secondary_key {
            ensure!(sk.key == account_id, Error::<T>::UnauthorizedHolderKey);
            return Ok(());
        }

        let primary_key = pallet_identity::Pallet::<T>::get_primary_key(primary_did)
            .ok_or(Error::<T>::KeyNotFoundForDid)?;

        ensure!(primary_key == account_id, Error::<T>::UnauthorizedHolderKey);
        Ok(())
    }

    /// Returns `true` if the affirmation of the asset holder can be skipped for the given `asset_id`.
    ///
    /// Default is `true` (skip affirmation) unless:
    /// - The identity has opted in to mandatory receiver affirmation via [`MandatoryReceiverAffirmation`], AND
    /// - The asset is not globally exempt, AND
    /// - The identity has not pre-approved this specific asset.
    ///
    /// Returns an error if the asset holder does not have a valid identity.
    pub fn skip_asset_holder_affirmation(
        asset_holder: &AssetHolder,
        asset_id: &AssetId,
    ) -> Result<bool, DispatchError> {
        if let AssetHolder::Portfolio(portfolio_id) = asset_holder {
            return Ok(PortfolioPallet::<T>::skip_portfolio_affirmation(
                portfolio_id,
                asset_id,
            ));
        }

        // For Account holders: default is to skip affirmation (no affirmation required).
        // Only require affirmation if the identity has opted in via MandatoryReceiverAffirmation.
        let did = IdentityPallet::<T>::asset_holder_did(asset_holder)?;
        if T::AffirmationFn::identity_requires_affirmation(&did) {
            return Ok(Self::skip_asset_affirmation(&did, asset_id));
        }

        // Default: no affirmation required.
        Ok(true)
    }

    /// Returns `Ok` if:
    /// - The DID associated to the asset holder exists, and
    /// - If the asset holder is a portfolio and it is valid.
    pub fn ensure_valid_holder(asset_holder: &AssetHolder) -> DispatchResult {
        let holder_did = pallet_identity::Pallet::<T>::asset_holder_did(asset_holder)?;
        pallet_identity::Pallet::<T>::ensure_id_record_exists(holder_did)?;
        if let AssetHolder::Portfolio(portfolio_id) = asset_holder {
            PortfolioPallet::<T>::ensure_portfolio_validity(portfolio_id)?;
        }
        Ok(())
    }
}

//==========================================================================
// All Storage Writes!
//==========================================================================

impl<T: AssetConfig> Pallet<T> {
    /// All storage writes for registering `ticker` to `owner` with an optional `expiry`.
    /// Note: If `charge_fee` is `true` one fee is charged ([`ProtocolOp::AssetRegisterTicker`]).
    fn unverified_register_ticker(
        ticker: Ticker,
        owner: IdentityId,
        expiry: Option<T::Moment>,
        charge_fee: bool,
    ) -> DispatchResult {
        if charge_fee {
            T::ProtocolFee::charge_fee(ProtocolOp::AssetRegisterTicker)?;
        }

        // If the ticker was already registered, removes the previous owner
        if let Some(ticker_registration) = UniqueTickerRegistration::<T>::get(ticker) {
            TickersOwnedByUser::<T>::remove(&ticker_registration.owner, &ticker);
        }

        // Write the ticker registration data to the storage
        UniqueTickerRegistration::<T>::insert(ticker, TickerRegistration { owner, expiry });
        TickersOwnedByUser::<T>::insert(owner, ticker, true);

        Self::deposit_event(Event::TickerRegistered(owner, ticker, expiry));
        Ok(())
    }

    /// Transfer the given `ticker`'s registration from `req.owner` to `to`.
    fn transfer_ticker(mut reg: TickerRegistration<T::Moment>, ticker: Ticker, to: IdentityId) {
        let from = reg.owner;
        TickersOwnedByUser::<T>::remove(from, ticker);
        TickersOwnedByUser::<T>::insert(to, ticker, true);
        reg.owner = to;
        UniqueTickerRegistration::<T>::insert(&ticker, reg);
        Self::deposit_event(Event::TickerTransferred(to, ticker, from));
    }

    /// All storage writes for creating an asset.
    /// Note: two fees are charged ([`ProtocolOp::AssetCreateAsset`] and [`ProtocolOp::AssetRegisterTicker`]).
    fn unverified_create_asset(
        caller_did: IdentityId,
        asset_id: AssetId,
        divisible: bool,
        asset_name: AssetName,
        asset_type: AssetType,
        funding_round_name: Option<FundingRoundName>,
        asset_identifiers: Vec<AssetIdentifier>,
    ) -> DispatchResult {
        T::ProtocolFee::charge_fee(ProtocolOp::AssetCreateAsset)?;

        let asset_details = AssetDetails::new(Zero::zero(), caller_did, divisible, asset_type);
        Assets::<T>::insert(asset_id, asset_details);

        AssetNames::<T>::insert(asset_id, &asset_name);
        if let Some(funding_round_name) = funding_round_name.as_ref() {
            FundingRound::<T>::insert(asset_id, funding_round_name);
        }

        SecurityTokensOwnedByUser::<T>::insert(caller_did, asset_id, true);

        // Updating ERC20 precompile storage
        let asset_index = NextAssetIndex::<T>::get();
        Erc20IndexToAssetId::<T>::insert(asset_index, asset_id);
        Erc20AssetIdToIndex::<T>::insert(asset_id, asset_index);
        let next_index = asset_index.saturating_add(1);
        NextAssetIndex::<T>::put(next_index);

        Self::deposit_event(Event::AssetCreated(
            caller_did,
            asset_id,
            divisible,
            asset_type,
            caller_did,
            asset_name,
            asset_identifiers.clone(),
            funding_round_name,
        ));

        // These emit events which should come after the main AssetCreated event
        Self::unverified_update_asset_identifiers(caller_did, asset_id, asset_identifiers);
        // Grant owner full agent permissions.
        <ExternalAgents<T>>::unchecked_add_agent(asset_id, caller_did, AgentGroup::Full)?;
        Ok(())
    }

    /// Inserts `asset_identifiers` for the given `asset_id` and emits [`Event::IdentifiersUpdated`].
    fn unverified_update_asset_identifiers(
        did: IdentityId,
        asset_id: AssetId,
        asset_identifiers: Vec<AssetIdentifier>,
    ) {
        AssetIdentifiers::<T>::insert(asset_id, asset_identifiers.clone());
        Self::deposit_event(Event::IdentifiersUpdated(did, asset_id, asset_identifiers));
    }

    /// All storage writes for issuing tokens.
    /// Note: if `charge_fee`` is `true` [`ProtocolOp::AssetIssue`] is charged.
    fn unverified_issue_tokens(
        asset_id: AssetId,
        asset_details: &mut AssetDetails,
        issuer: AssetHolder,
        amount_to_issue: Balance,
        charge_fee: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        if charge_fee {
            T::ProtocolFee::charge_fee(ProtocolOp::AssetIssue)?;
        }

        let issuer_did = pallet_identity::Pallet::<T>::asset_holder_did(&issuer)?;

        let current_issuer_balance = BalanceOf::<T>::get(&asset_id, &issuer_did);
        Checkpoint::<T>::advance_update_balances(
            &asset_id,
            &[(issuer_did, current_issuer_balance)],
        )?;

        let new_issuer_balance = current_issuer_balance.saturating_add(amount_to_issue);
        BalanceOf::<T>::insert(asset_id, issuer_did, new_issuer_balance);

        asset_details.total_supply = asset_details.total_supply.saturating_add(amount_to_issue);
        Assets::<T>::insert(asset_id, asset_details);

        // No check since the total balance is always <= the total supply
        let current_balance = Self::get_holders_balance(&issuer, &asset_id);
        let new_holder_balance = current_balance.saturating_add(amount_to_issue);

        Self::set_holders_balance(issuer.clone(), asset_id, new_holder_balance)?;

        Statistics::<T>::update_asset_stats(
            asset_id,
            None,
            Some(&issuer_did),
            None,
            Some(new_issuer_balance),
            amount_to_issue,
            weight_meter,
        )?;

        let funding_round_name = FundingRound::<T>::get(&asset_id);
        IssuedInFundingRound::<T>::mutate((&asset_id, &funding_round_name), |balance| {
            *balance = balance.saturating_add(amount_to_issue)
        });

        Self::deposit_event(Event::AssetBalanceUpdated(
            issuer_did,
            asset_id,
            amount_to_issue,
            None,
            Some(issuer),
            HoldingsUpdateReason::Issued {
                funding_round_name: Some(funding_round_name),
            },
        ));
        Ok(())
    }

    // Transfers `transfer_value` from `sender` to `receiver`.
    pub fn unverified_transfer_asset(
        sender: AssetHolder,
        receiver: AssetHolder,
        asset_id: AssetId,
        transfer_value: Balance,
        instruction_id: Option<InstructionId>,
        instruction_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Gets the current balance and advances the checkpoint
        let sender_did = pallet_identity::Pallet::<T>::asset_holder_did(&sender)?;
        let receiver_did = pallet_identity::Pallet::<T>::asset_holder_did(&receiver)?;

        let sender_current_balance = BalanceOf::<T>::get(&asset_id, &sender_did);
        let receiver_current_balance = BalanceOf::<T>::get(&asset_id, &receiver_did);
        <Checkpoint<T>>::advance_update_balances(
            &asset_id,
            &[
                (sender_did, sender_current_balance),
                (receiver_did, receiver_current_balance),
            ],
        )?;

        // Updates the balance in the asset pallet
        let sender_new_balance = sender_current_balance.saturating_sub(transfer_value);
        let receiver_new_balance = receiver_current_balance.saturating_add(transfer_value);
        BalanceOf::<T>::insert(asset_id, sender_did, sender_new_balance);
        BalanceOf::<T>::insert(asset_id, receiver_did, receiver_new_balance);

        // Updates the balances in the portfolio pallet
        Self::transfer_holders_balance(sender.clone(), receiver.clone(), asset_id, transfer_value)?;

        // Update statistics info.
        Statistics::<T>::update_asset_stats(
            asset_id,
            Some(&sender_did),
            Some(&receiver_did),
            Some(sender_new_balance),
            Some(receiver_new_balance),
            transfer_value,
            weight_meter,
        )?;

        Self::deposit_event(Event::AssetBalanceUpdated(
            caller_did,
            asset_id,
            transfer_value,
            Some(sender),
            Some(receiver),
            HoldingsUpdateReason::Transferred {
                instruction_id,
                instruction_memo,
            },
        ));
        Ok(())
    }

    /// Emits an event in case `asset_type_bytes` already exits, otherwise creates a new custom type.
    fn unverified_register_custom_asset_type(
        caller_did: IdentityId,
        asset_type_bytes: Vec<u8>,
    ) -> Result<CustomAssetTypeId, DispatchError> {
        match CustomTypesInverse::<T>::try_get(&asset_type_bytes) {
            Ok(type_id) => {
                Self::deposit_event(Event::<T>::CustomAssetTypeExists(
                    caller_did,
                    type_id,
                    asset_type_bytes,
                ));
                Ok(type_id)
            }
            Err(_) => {
                let type_id = CustomTypeIdSequence::<T>::try_mutate(try_next_pre::<T, _>)?;
                CustomTypesInverse::<T>::insert(&asset_type_bytes, type_id);
                CustomTypes::<T>::insert(type_id, &asset_type_bytes);
                Self::deposit_event(Event::<T>::CustomAssetTypeRegistered(
                    caller_did,
                    type_id,
                    asset_type_bytes,
                ));
                Ok(type_id)
            }
        }
    }

    fn unverified_set_asset_metadata(
        did: IdentityId,
        asset_id: AssetId,
        key: AssetMetadataKey,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Check value length limit.
        Self::ensure_asset_metadata_value_limited(&value)?;

        // Check key exists.
        ensure!(
            Self::check_asset_metadata_key_exists(&asset_id, &key),
            Error::<T>::AssetMetadataKeyIsMissing
        );

        // Check if value is currently locked.
        ensure!(
            !Self::is_asset_metadata_locked(&asset_id, key),
            Error::<T>::AssetMetadataValueIsLocked
        );

        // Set asset metadata value for asset.
        AssetMetadataValues::<T>::insert(asset_id, key, &value);

        // Set asset metadata value details.
        if let Some(ref detail) = detail {
            AssetMetadataValueDetails::<T>::insert(asset_id, key, detail);
        }

        Self::deposit_event(Event::SetAssetMetadataValue(did, asset_id, value, detail));
        Ok(())
    }

    fn unverified_register_asset_metadata_local_type(
        did: IdentityId,
        asset_id: AssetId,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> Result<AssetMetadataKey, DispatchError> {
        Self::ensure_asset_metadata_name_limited(&name)?;
        Self::ensure_asset_metadata_spec_limited(&spec)?;

        // Check if key already exists.
        ensure!(
            !AssetMetadataLocalNameToKey::<T>::contains_key(asset_id, &name),
            Error::<T>::AssetMetadataLocalKeyAlreadyExists
        );

        // Next local key for asset.
        let key = Self::update_current_asset_metadata_local_key(&asset_id)?;

        // Store local key <-> name mapping.
        AssetMetadataLocalNameToKey::<T>::insert(asset_id, &name, key);
        AssetMetadataLocalKeyToName::<T>::insert(asset_id, key, &name);

        // Store local specs.
        AssetMetadataLocalSpecs::<T>::insert(asset_id, key, &spec);

        Self::deposit_event(Event::RegisterAssetMetadataLocalType(
            did, asset_id, name, key, spec,
        ));
        Ok(key.into())
    }

    /// Adds one to `CurrentCollectionId`.
    fn update_current_asset_metadata_global_key() -> Result<AssetMetadataGlobalKey, DispatchError> {
        CurrentAssetMetadataGlobalKey::<T>::try_mutate(
            |current_global_key| match current_global_key {
                Some(current_key) => {
                    let new_key = try_next_pre::<T, _>(current_key)?;
                    *current_global_key = Some(new_key);
                    Ok::<AssetMetadataGlobalKey, DispatchError>(new_key)
                }
                None => {
                    let new_key = AssetMetadataGlobalKey(1);
                    *current_global_key = Some(new_key);
                    Ok::<AssetMetadataGlobalKey, DispatchError>(new_key)
                }
            },
        )
    }

    /// Adds one to the `AssetMetadataLocalKey` for the given `ticker`.
    fn update_current_asset_metadata_local_key(
        asset_id: &AssetId,
    ) -> Result<AssetMetadataLocalKey, DispatchError> {
        CurrentAssetMetadataLocalKey::<T>::try_mutate(asset_id, |current_local_key| {
            match current_local_key {
                Some(current_key) => {
                    let new_key = try_next_pre::<T, _>(current_key)?;
                    *current_local_key = Some(new_key);
                    Ok::<AssetMetadataLocalKey, DispatchError>(new_key)
                }
                None => {
                    let new_key = AssetMetadataLocalKey(1);
                    *current_local_key = Some(new_key);
                    Ok::<AssetMetadataLocalKey, DispatchError>(new_key)
                }
            }
        })
    }

    /// Transfers `transfer_value` of `asset_id` from `sender_pid` to `receiver_pid`.
    /// Note: This functions skips all compliance and statistics checks, only checking for balance.
    pub fn simplified_fungible_transfer(
        asset_id: AssetId,
        sender: AssetHolder,
        receiver: AssetHolder,
        transfer_value: Balance,
        inst_id: InstructionId,
        inst_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let sender_did = pallet_identity::Pallet::<T>::asset_holder_did(&sender)?;
        let receiver_did = pallet_identity::Pallet::<T>::asset_holder_did(&receiver)?;

        ensure!(
            BalanceOf::<T>::get(&asset_id, &sender_did) >= transfer_value,
            Error::<T>::InsufficientBalance
        );

        match &receiver {
            AssetHolder::Portfolio(receiver_pid) => {
                PortfolioPallet::<T>::ensure_portfolio_validity(receiver_pid)?
            }
            AssetHolder::Account(_) => {}
        }

        Self::ensure_sufficient_balance(&sender, &asset_id, transfer_value)?;

        Statistics::<T>::verify_transfer_restrictions(
            asset_id,
            &sender_did,
            &receiver_did,
            BalanceOf::<T>::get(asset_id, &sender_did),
            BalanceOf::<T>::get(asset_id, &receiver_did),
            transfer_value,
            Self::try_get_asset_details(&asset_id)?.total_supply,
            weight_meter,
        )?;

        Self::unverified_transfer_asset(
            sender,
            receiver,
            asset_id,
            transfer_value,
            Some(inst_id),
            inst_memo,
            caller_did,
            weight_meter,
        )?;

        Ok(())
    }
}

//==========================================================================
// All RPC functions!
//==========================================================================

//==========================================================================
// Trait implementation!
//==========================================================================

impl<T: AssetConfig> AssetFnTrait<T::AccountId> for Pallet<T> {
    fn ensure_granular(asset_id: &AssetId, value: Balance) -> DispatchResult {
        let asset_details = Self::try_get_asset_details(&asset_id)?;
        Self::ensure_asset_granular(&asset_details, &value)
    }

    fn skip_asset_affirmation(identity_id: &IdentityId, asset_id: &AssetId) -> bool {
        if AssetsExemptFromAffirmation::<T>::get(asset_id) {
            return true;
        }
        PreApprovedAsset::<T>::get(identity_id, asset_id)
    }

    fn asset_affirmation_exemption(asset_id: &AssetId) -> bool {
        AssetsExemptFromAffirmation::<T>::get(asset_id)
    }

    fn asset_balance(asset_id: &AssetId, did: &IdentityId) -> Balance {
        BalanceOf::<T>::get(asset_id, did)
    }

    fn asset_total_supply(asset_id: &AssetId) -> Result<Balance, DispatchError> {
        Ok(Self::try_get_asset_details(&asset_id)?.total_supply)
    }

    fn generate_asset_id(caller_acc: T::AccountId) -> AssetId {
        Self::generate_asset_id(caller_acc, false)
    }

    fn spend_allowance_weight() -> Weight {
        <T as Config>::WeightInfo::spend_allowance()
    }

    fn asset_is_not_frozen(asset_id: &AssetId) -> DispatchResult {
        Self::ensure_asset_is_not_frozen(asset_id)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn register_unique_ticker(caller: T::AccountId, ticker: Ticker) -> DispatchResult {
        let origin = RawOrigin::Signed(caller);
        Self::register_unique_ticker(origin.into(), ticker)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn create_asset(
        caller: T::AccountId,
        asset_name: AssetName,
        divisible: bool,
        asset_type: AssetType,
        asset_identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult {
        let origin = RawOrigin::Signed(caller);
        Self::create_asset(
            origin.into(),
            asset_name,
            divisible,
            asset_type,
            asset_identifiers,
            funding_round,
        )
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn issue(
        caller: T::AccountId,
        asset_id: AssetId,
        amount: Balance,
        asset_holder_kind: AssetHolderKind,
    ) -> DispatchResult {
        let origin = RawOrigin::Signed(caller);
        Self::issue(origin.into(), asset_id, amount, asset_holder_kind)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn register_asset_metadata_type(
        asset_and_caller: Option<(AssetId, T::AccountId)>,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        match asset_and_caller {
            Some((asset_id, caller)) => {
                let origin = RawOrigin::Signed(caller);
                Self::register_asset_metadata_local_type(origin.into(), asset_id, name, spec)
            }
            None => {
                let origin = RawOrigin::Root;
                Self::register_asset_metadata_global_type(origin.into(), name, spec)
            }
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn add_mandatory_mediators(
        caller: T::AccountId,
        asset_id: AssetId,
        mediators: BTreeSet<IdentityId>,
    ) -> DispatchResult {
        let origin = RawOrigin::Signed(caller);
        Self::add_mandatory_mediators(
            origin.into(),
            asset_id,
            mediators.try_into().unwrap_or_default(),
        )
    }
}
