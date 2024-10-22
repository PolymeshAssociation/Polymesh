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
//! The Asset module is one place to create the security tokens on the Polymesh blockchain.
//! The module provides based functionality related to security tokens.
//! Functions in the module differentiate between tokens using its `Ticker`.
//! In Ethereum analogy every token has different smart contract address which act as the unique identity
//! of the token while here token lives at low-level where token ticker act as the differentiator.
//!
//! ## Overview
//!
//! The Asset module provides functions for:
//!
//! - Creating the tokens.
//! - Creation of checkpoints on the token level.
//! - Management of the token (Document mgt etc).
//! - Transfer/redeem functionality of the token.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `register_ticker` - Used to either register a new ticker or extend registration of an existing ticker.
//! - `accept_ticker_transfer` - Used to accept a ticker transfer authorization.
//! - `accept_asset_ownership_transfer` - Used to accept the token transfer authorization.
//! - `create_asset` - Initializes a new security token.
//! - `freeze` - Freezes transfers and minting of a given token.
//! - `unfreeze` - Unfreezes transfers and minting of a given token.
//! - `rename_asset` - Renames a given asset.
//! - `controller_transfer` - Forces a transfer between two DID.
//! - `issue` - Function is used to issue(or mint) new tokens to the caller.
//! - `redeem` - Redeems tokens from the caller's default portfolio.
//! - `make_divisible` - Change the divisibility of the token to divisible.
//! - `can_transfer` - Checks whether a transaction with given parameters can take place or not.
//! - `add_documents` - Add documents for a given token.
//! - `remove_documents` - Remove documents for a given token.
//! - `set_funding_round` - Sets the name of the current funding round.
//! - `update_identifiers` - Updates the asset identifiers.
//! - `set_asset_metadata` - Set asset metadata value.
//! - `set_asset_metadata_details` - Set asset metadata value details (expire, lock status).
//! - `register_asset_metadata_local_type` - Register asset metadata local type.
//! - `register_asset_metadata_global_type` - Register asset metadata global type.
//! - `redeem_from_portfolio` - Redeems tokens from the caller's portfolio.
//!
//! ### Public Functions
//!
//! - `ticker_registration` - Provide ticker registration details.
//! - `ticker_registration_config` - Provide the ticker registration configuration details.
//! - `token_details` - Returns details of the token.
//! - `balance_of` - Returns the balance of the DID corresponds to the ticker.
//! - `identifiers` - It provides the identifiers for a given ticker.
//! - `total_checkpoints_of` - Returns the checkpoint Id.
//! - `total_supply_at` - Returns the total supply at a given checkpoint.
//! - `extension_details` - It provides the list of Smart extension added for the given tokens.
//! - `extensions` - It provides the list of Smart extension added for the given tokens and for the given type.
//! - `frozen` - It tells whether the given ticker is frozen or not.
//! - `total_supply` - It provides the total supply of a ticker.
//! - `get_balance_at` - It provides the balance of a DID at a certain checkpoint.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod checkpoint;

mod error;
mod migrations;
mod types;

use codec::{Decode, Encode};
use core::mem;
use currency::*;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::BoundedBTreeSet;
use frame_support::{decl_module, decl_storage, ensure};
use frame_system::ensure_root;
use sp_io::hashing::blake2_128;
use sp_runtime::traits::Zero;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

use pallet_base::{
    ensure_opt_string_limited, ensure_string_limited, try_next_pre, Error::CounterOverflow,
};
use pallet_identity::PermissionedCallOriginData;
use pallet_portfolio::{Error as PortfolioError, PortfolioAssetBalances};
use polymesh_common_utilities::asset::AssetFnTrait;
use polymesh_common_utilities::compliance_manager::ComplianceFnConfig;
use polymesh_common_utilities::constants::*;
use polymesh_common_utilities::protocol_fee::{ChargeProtocolFee, ProtocolOp};
pub use polymesh_common_utilities::traits::asset::{Config, Event, RawEvent, WeightInfo};
use polymesh_common_utilities::traits::nft::NFTTrait;
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::asset::{
    AssetId, AssetName, AssetType, CheckpointId, CustomAssetTypeId, FundingRoundName,
};
use polymesh_primitives::asset_metadata::{
    AssetMetadataGlobalKey, AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName,
    AssetMetadataSpec, AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::settlement::InstructionId;
use polymesh_primitives::{
    extract_auth, storage_migrate_on, storage_migration_ver, AssetIdentifier, Balance, Document,
    DocumentId, IdentityId, Memo, PortfolioId, PortfolioKind, PortfolioUpdateReason, SecondaryKey,
    Ticker, WeightMeter,
};

pub use error::Error;
pub use types::{
    AssetDetails, AssetOwnershipRelation, TickerRegistration, TickerRegistrationConfig,
    TickerRegistrationStatus,
};

type Checkpoint<T> = checkpoint::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;
type Statistics<T> = pallet_statistics::Module<T>;

storage_migration_ver!(5);

decl_storage! {
    trait Store for Module<T: Config> as Asset {
        /// Maps each [`Ticker`] to its registration details ([`TickerRegistration`]).
        pub UniqueTickerRegistration get(fn unique_ticker_registration): map hasher(blake2_128_concat) Ticker => Option<TickerRegistration<T::Moment>>;
        /// Returns [`TickerRegistrationConfig`] for assessing if a ticker is valid.
        pub TickerConfig get(fn ticker_registration_config) config(): TickerRegistrationConfig<T::Moment>;
        /// Maps each [`AssetId`] to its underling [`AssetDetails`].
        pub Assets get(fn assets_details): map hasher(blake2_128_concat) AssetId => Option<AssetDetails>;
        /// Maps each [`AssetId`] to its underling [`AssetName`].
        pub AssetNames get(fn asset_names): map hasher(blake2_128_concat) AssetId => Option<AssetName>;
        /// Tracks the total [`Balance`] for each [`AssetId`] per [`IdentityId`].
        // NB: It is safe to use `identity` hasher here because assets can not be distributed to non-existent identities.
        pub BalanceOf get(fn balance_of): double_map hasher(blake2_128_concat) AssetId, hasher(identity) IdentityId => Balance;
        /// Maps each [`AssetId`] to its asset identifiers ([`AssetIdentifier`]).
        pub AssetIdentifiers get(fn asset_identifiers): map hasher(blake2_128_concat) AssetId => Vec<AssetIdentifier>;

        /// The next `AssetType::Custom` ID in the sequence.
        ///
        /// Numbers in the sequence start from 1 rather than 0.
        pub CustomTypeIdSequence get(fn custom_type_id_seq): CustomAssetTypeId;
        /// Maps custom asset type ids to the registered string contents.
        pub CustomTypes get(fn custom_types): map hasher(twox_64_concat) CustomAssetTypeId => Vec<u8>;
        /// Inverse map of `CustomTypes`, from registered string contents to custom asset type ids.
        pub CustomTypesInverse get(fn custom_types_inverse): map hasher(blake2_128_concat) Vec<u8> => Option<CustomAssetTypeId>;

        /// Maps each [`AssetId`] to the name of its founding round ([`FundingRoundName`]).
        pub FundingRound get(fn funding_round): map hasher(blake2_128_concat) AssetId => FundingRoundName;
        /// The total [`Balance`] of tokens issued in all recorded funding rounds ([`FundingRoundName`]).
        pub IssuedInFundingRound get(fn issued_in_funding_round): map hasher(blake2_128_concat) (AssetId, FundingRoundName) => Balance;
        /// Returns `true` if transfers for the token associated to [`AssetId`] are frozen. Otherwise, returns `false`.
        pub Frozen get(fn frozen): map hasher(blake2_128_concat) AssetId => bool;
        /// All [`Document`] attached to an asset.
        pub AssetDocuments get(fn asset_documents):
            double_map hasher(blake2_128_concat) AssetId, hasher(twox_64_concat) DocumentId => Option<Document>;
        /// [`DocumentId`] counter per [`AssetId`].
        pub AssetDocumentsIdSequence get(fn asset_documents_id_sequence): map hasher(blake2_128_concat) AssetId => DocumentId;

        /// Metatdata values for an asset.
        pub AssetMetadataValues get(fn asset_metadata_values):
            double_map hasher(blake2_128_concat) AssetId, hasher(twox_64_concat) AssetMetadataKey =>
                Option<AssetMetadataValue>;
        /// Details for an asset's Metadata values.
        pub AssetMetadataValueDetails get(fn asset_metadata_value_details):
            double_map hasher(blake2_128_concat) AssetId, hasher(twox_64_concat) AssetMetadataKey =>
                Option<AssetMetadataValueDetail<T::Moment>>;

        /// Asset Metadata Local Name -> Key.
        pub AssetMetadataLocalNameToKey get(fn asset_metadata_local_name_to_key):
            double_map hasher(blake2_128_concat) AssetId, hasher(blake2_128_concat) AssetMetadataName =>
                Option<AssetMetadataLocalKey>;
        /// Asset Metadata Global Name -> Key.
        pub AssetMetadataGlobalNameToKey get(fn asset_metadata_global_name_to_key):
            map hasher(blake2_128_concat) AssetMetadataName => Option<AssetMetadataGlobalKey>;

        /// Asset Metadata Local Key -> Name.
        pub AssetMetadataLocalKeyToName get(fn asset_metadata_local_key_to_name):
            double_map hasher(blake2_128_concat) AssetId, hasher(twox_64_concat) AssetMetadataLocalKey =>
                Option<AssetMetadataName>;
        /// Asset Metadata Global Key -> Name.
        pub AssetMetadataGlobalKeyToName get(fn asset_metadata_global_key_to_name):
            map hasher(twox_64_concat) AssetMetadataGlobalKey => Option<AssetMetadataName>;

        /// Asset Metadata Local Key specs.
        pub AssetMetadataLocalSpecs get(fn asset_metadata_local_specs):
            double_map hasher(blake2_128_concat) AssetId, hasher(twox_64_concat) AssetMetadataLocalKey =>
                Option<AssetMetadataSpec>;
        /// Asset Metadata Global Key specs.
        pub AssetMetadataGlobalSpecs get(fn asset_metadata_global_specs):
            map hasher(twox_64_concat) AssetMetadataGlobalKey => Option<AssetMetadataSpec>;

        /// A list of assets that exempt all users from affirming its receivement.
        pub AssetsExemptFromAffirmation get(fn assets_exempt_from_affirmation):
            map hasher(blake2_128_concat) AssetId => bool;

        /// All assets that don't need an affirmation to be received by an identity.
        pub PreApprovedAsset get(fn pre_approved_asset):
            double_map hasher(identity) IdentityId, hasher(blake2_128_concat) AssetId => bool;

        /// The list of mandatory mediators for every ticker.
        pub MandatoryMediators get(fn mandatory_mediators):
            map hasher(blake2_128_concat) AssetId => BoundedBTreeSet<IdentityId, T::MaxAssetMediators>;

        /// The last [`AssetMetadataLocalKey`] used for [`AssetId`].
        pub CurrentAssetMetadataLocalKey get(fn current_asset_metadata_local_key):
            map hasher(blake2_128_concat) AssetId => Option<AssetMetadataLocalKey>;

        /// The last [`AssetMetadataGlobalKey`] used for a global key.
        pub CurrentAssetMetadataGlobalKey get(fn current_asset_metadata_global_key): Option<AssetMetadataGlobalKey>;

        /// All tickers owned by a user.
        pub TickersOwnedByUser get(fn tickers_owned_by_user):
            double_map hasher(identity) IdentityId, hasher(blake2_128_concat) Ticker => bool;

        /// All security tokens owned by a user.
        pub SecurityTokensOwnedByUser get(fn security_tokens_owned_by_user):
            double_map hasher(identity) IdentityId, hasher(blake2_128_concat) AssetId => bool;

        /// Maps all [`AssetId`] that are mapped to a [`Ticker`].
        pub AssetIdTicker get(fn asset_id_ticker): map hasher(blake2_128_concat) AssetId => Option<Ticker>;

        /// Maps all [`Ticker`] that are linked to an [`AssetId`].
        pub TickerAssetId get(fn ticker_asset_id): map hasher(blake2_128_concat) Ticker => Option<AssetId>;

        /// A per account nonce that is used for generating an [`AssetId`].
        pub AssetNonce: map hasher(identity) T::AccountId => u64;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(5)): Version;
    }

    add_extra_genesis {
        config(reserved_country_currency_codes): Vec<Ticker>;
        config(asset_metadata): Vec<(AssetMetadataName, AssetMetadataSpec)>;

        build(|config: &GenesisConfig<T>| {
            // Reserving country currency logic
            let fiat_tickers_reservation_did =
                polymesh_common_utilities::SystematicIssuers::FiatTickersReservation.as_id();
            for currency_ticker in &config.reserved_country_currency_codes {
                let _ = <Module<T>>::unverified_register_ticker(
                    *currency_ticker,
                    fiat_tickers_reservation_did,
                    None,
                    false
                );
            }

            // Register Asset Metadata.
            for (name, spec) in &config.asset_metadata {
                <Module<T>>::base_register_asset_metadata_global_type(name.clone(), spec.clone())
                    .expect("Shouldn't fail");
            }
        });
    }

}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {

        type Error = Error<T>;

        const AssetNameMaxLength: u32 = T::AssetNameMaxLength::get();
        const FundingRoundNameMaxLength: u32 = T::FundingRoundNameMaxLength::get();
        const AssetMetadataNameMaxLength: u32 = T::AssetMetadataNameMaxLength::get();
        const AssetMetadataValueMaxLength: u32 = T::AssetMetadataValueMaxLength::get();
        const AssetMetadataTypeDefMaxLength: u32 = T::AssetMetadataTypeDefMaxLength::get();
        const MaxAssetMediators: u32 = T::MaxAssetMediators::get();

        /// initialize the default event for this module
        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            storage_migrate_on!(StorageVersion, 5, {
                migrations::migrate_to_v5::<T>();
            });
            // Only needed on staging, but safe to run on other networks.
            migrations::migrate_to_v5_fixup_asset_id_maps::<T>();

            Weight::zero()
        }

        /// Registers a unique ticker or extends validity of an existing ticker.
        /// NB: Ticker validity does not get carry forward when renewing ticker.
        ///
        /// # Arguments
        /// * `origin`: it contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `ticker`: [`Ticker`] to register.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::register_unique_ticker()]
        pub fn register_unique_ticker(origin, ticker: Ticker) -> DispatchResult {
            Self::base_register_unique_ticker(origin, ticker)
        }

        /// Accepts a ticker transfer.
        ///
        /// Consumes the authorization `auth_id` (see `pallet_identity::consume_auth`).
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin`: it contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `auth_id`: authorization ID of ticker transfer authorization.
        #[weight = <T as Config>::WeightInfo::accept_ticker_transfer()]
        pub fn accept_ticker_transfer(origin, auth_id: u64) -> DispatchResult {
            Self::base_accept_ticker_transfer(origin, auth_id)
        }

        /// This function is used to accept a token ownership transfer.
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin`: it contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `auth_id`: authorization ID of the token ownership transfer authorization.
        #[weight = <T as Config>::WeightInfo::accept_asset_ownership_transfer()]
        pub fn accept_asset_ownership_transfer(origin, auth_id: u64) -> DispatchResult {
            Self::base_accept_token_ownership_transfer(origin, auth_id)
        }

        /// Initializes a new [`AssetDetails`], with the initiating account as its owner.
        /// The total supply will initially be zero. To mint tokens, use [`Module::issue`].
        ///
        /// # Arguments
        /// * `origin`: contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `asset_name`: the [`AssetName`] associated to the security token.
        /// * `divisible`: sets [`AssetDetails::divisible`], where `true` means the token is divisible.
        /// * `asset_type`: the [`AssetType`] that represents the security type of the [`AssetDetails`].
        /// * `asset_identifiers`: a vector of [`AssetIdentifier`].
        /// * `funding_round_name`: the name of the funding round ([`FundingRoundName`]).
        ///
        /// ## Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::create_asset(
            asset_name.len() as u32,
            asset_identifiers.len() as u32,
            funding_round_name.as_ref().map_or(0, |name| name.len()) as u32
        )]
        pub fn create_asset(
            origin,
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
                funding_round_name
            )?;
            Ok(())
        }

        /// Freezes transfers of a given token.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::freeze()]
        pub fn freeze(origin, asset_id: AssetId) -> DispatchResult {
            Self::base_set_freeze(origin, asset_id, true)
        }

        /// Unfreezes transfers of a given token.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::unfreeze()]
        pub fn unfreeze(origin, asset_id: AssetId) -> DispatchResult {
            Self::base_set_freeze(origin, asset_id, false)
        }

        /// Updates the [`AssetName`] associated to a security token.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `asset_name`: the [`AssetName`] that will be associated to the token.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::rename_asset(asset_name.len() as u32)]
        pub fn rename_asset(origin, asset_id: AssetId, asset_name: AssetName) -> DispatchResult {
            Self::base_rename_asset(origin, asset_id, asset_name)
        }

        /// Issue (i.e mint) new tokens to the caller, which must be an authorized external agent.
        ///
        /// # Arguments
        /// * `origin`: A signer that has permissions to act as an agent of `ticker`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `amount`: The amount of tokens that will be issued.
        /// * `portfolio_kind`: The [`PortfolioKind`] of the portfolio that will receive the minted tokens.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::issue()]
        pub fn issue(origin, asset_id: AssetId, amount: Balance, portfolio_kind: PortfolioKind) -> DispatchResult {
            Self::base_issue(origin, asset_id, amount, portfolio_kind)
        }

        /// Redeems (i.e burns) existing tokens by reducing the balance of the caller's portfolio and the total supply of the token.
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `value`: amount of tokens to redeem.
        /// * `portfolio_kind`: the [`PortfolioKind`] that will have its balance reduced.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::redeem()]
        pub fn redeem(origin, asset_id: AssetId, value: Balance, portfolio_kind: PortfolioKind) -> DispatchResult {
            let mut weight_meter = WeightMeter::max_limit_no_minimum();
            Self::base_redeem(origin, asset_id, value, portfolio_kind, &mut weight_meter)
        }

        /// If the token associated to `asset_id` is indivisible, sets [`AssetDetails::divisible`] to true.
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `ticker`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::make_divisible()]
        pub fn make_divisible(origin, asset_id: AssetId) -> DispatchResult {
            Self::base_make_divisible(origin, asset_id)
        }

        /// Add documents for a given token.
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `docs`: documents to be attached to the token.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_documents(docs.len() as u32)]
        pub fn add_documents(origin, docs: Vec<Document>, asset_id: AssetId) -> DispatchResult {
            Self::base_add_documents(origin, docs, asset_id)
        }

        /// Remove documents for a given token.
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `docs_id`: a vector of all [`DocumentId`] that will be removed from the token.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_documents(docs_id.len() as u32)]
        pub fn remove_documents(origin, docs_id: Vec<DocumentId>, asset_id: AssetId) -> DispatchResult {
            Self::base_remove_documents(origin, docs_id, asset_id)
        }

        /// Sets the name of the current funding round.
        ///
        /// # Arguments
        /// * `origin`:  a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `founding_round_name`: the [`FoundingRoundName`] of the current funding round.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_funding_round(founding_round_name.len() as u32)]
        pub fn set_funding_round(origin, asset_id: AssetId, founding_round_name: FundingRoundName) -> DispatchResult {
            Self::base_set_funding_round(origin, asset_id, founding_round_name)
        }

        /// Updates the asset identifiers associated to the token.
        ///
        /// # Arguments
        /// * `origin`: a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `asset_identifiers`: a vector of [`AssetIdentifier`] that will be associated to the token.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::update_identifiers(asset_identifiers.len() as u32)]
        pub fn update_identifiers(
            origin,
            asset_id: AssetId,
            asset_identifiers: Vec<AssetIdentifier>
        ) -> DispatchResult {
            Self::base_update_identifiers(origin, asset_id, asset_identifiers)
        }

        /// Forces a transfer of token from `from_portfolio` to the caller's default portfolio.
        ///
        /// # Arguments
        /// * `origin`: a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `value`:  the [`Balance`] of tokens that will be transferred.
        /// * `from_portfolio`: the [`PortfolioId`] that will have its balance reduced.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::controller_transfer()]
        pub fn controller_transfer(origin, asset_id: AssetId, value: Balance, from_portfolio: PortfolioId) -> DispatchResult {
            let mut weight_meter = WeightMeter::max_limit_no_minimum();
            Self::base_controller_transfer(origin, asset_id, value, from_portfolio, &mut weight_meter)
        }

        /// Registers a custom asset type.
        ///
        /// The provided `ty` will be bound to an ID in storage.
        /// The ID can then be used in `AssetType::Custom`.
        /// Should the `ty` already exist in storage, no second ID is assigned to it.
        ///
        /// # Arguments
        /// * `origin`: who called the extrinsic.
        /// * `ty`: contains the string representation of the asset type.
        #[weight = <T as Config>::WeightInfo::register_custom_asset_type(ty.len() as u32)]
        pub fn register_custom_asset_type(origin, ty: Vec<u8>) -> DispatchResult {
            Self::base_register_custom_asset_type(origin, ty)?;
            Ok(())
        }

        /// Initializes a new [`AssetDetails`], with the initiating account as its owner.
        /// The total supply will initially be zero. To mint tokens, use [`Module::issue`].
        /// Note: Utility extrinsic to batch [`Module::create_asset`] and [`Module::register_custom_asset_type`].
        ///
        /// # Arguments
        /// * `origin`: contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `asset_name`: the [`AssetName`] associated to the security token.
        /// * `divisible`: sets [`AssetDetails::divisible`], where `true` means the token is divisible.
        /// * `custom_asset_type`: the custom asset type of the token.
        /// * `asset_identifiers`: a vector of [`AssetIdentifier`].
        /// * `funding_round_name`: the name of the funding round ([`FundingRoundName`]).
        ///
        /// ## Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::create_asset(
            asset_name.len() as u32,
            asset_identifiers.len() as u32,
            funding_round_name.as_ref().map_or(0, |name| name.len()) as u32,
        )
        .saturating_add(<T as Config>::WeightInfo::register_custom_asset_type(
            custom_asset_type.len() as u32,
        ))]
        pub fn create_asset_with_custom_type(
            origin,
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
                funding_round_name
            )?;
            Ok(())
        }

        /// Set asset metadata value.
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `key`: the [`AssetMetadataKey`] associated to the token.
        /// * `value`: the [`AssetMetadataValue`] of the given metadata key.
        /// * `details`: optional [`AssetMetadataValueDetail`] (expire, lock status).
        ///
        /// # Permissions
        /// * Agent
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_asset_metadata()]
        pub fn set_asset_metadata(
            origin,
            asset_id: AssetId,
            key: AssetMetadataKey,
            value: AssetMetadataValue,
            detail: Option<AssetMetadataValueDetail<T::Moment>>
        ) -> DispatchResult {
            Self::base_set_asset_metadata(origin, asset_id, key, value, detail)
        }

        /// Set asset metadata value details (expire, lock status).
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `key`: the [`AssetMetadataKey`] associated to the token.
        /// * `details`: the [`AssetMetadataValueDetail`] (expire, lock status) that will be associated to the token.
        ///
        /// # Permissions
        /// * Agent
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_asset_metadata_details()]
        pub fn set_asset_metadata_details(
            origin,
            asset_id: AssetId,
            key: AssetMetadataKey,
            detail: AssetMetadataValueDetail<T::Moment>
        ) -> DispatchResult {
            Self::base_set_asset_metadata_details(origin, asset_id, key, detail)
        }

        /// Registers and set local asset metadata.
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `name`: the [`AssetMetadataName`].
        /// * `spec`: the asset metadata specifications ([`AssetMetadataSpec`]).
        /// * `value`: the [`AssetMetadataValue`] of the given metadata key.
        /// * `details`: optional [`AssetMetadataValueDetail`] (expire, lock status).
        ///
        /// # Permissions
        /// * Agent
        /// * Asset
        #[weight = <T as Config>::WeightInfo::register_and_set_local_asset_metadata()]
        pub fn register_and_set_local_asset_metadata(
            origin,
            asset_id: AssetId,
            name: AssetMetadataName,
            spec: AssetMetadataSpec,
            value: AssetMetadataValue,
            detail: Option<AssetMetadataValueDetail<T::Moment>>
        ) -> DispatchResult {
            Self::base_register_and_set_local_asset_metadata(origin, asset_id, name, spec, value, detail)
        }

        /// Registers asset metadata local type.
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `name`: the [`AssetMetadataName`].
        /// * `spec`: the asset metadata specifications ([`AssetMetadataSpec`]).
        ///
        /// # Permissions
        /// * Agent
        /// * Asset
        #[weight = <T as Config>::WeightInfo::register_asset_metadata_local_type()]
        pub fn register_asset_metadata_local_type(
            origin,
            asset_id: AssetId,
            name: AssetMetadataName,
            spec: AssetMetadataSpec
        ) -> DispatchResult {
            Self::base_register_asset_metadata_local_type(origin, asset_id, name, spec)
        }

        /// Registers asset metadata global type.
        ///
        /// # Arguments
        /// * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
        /// * `name`: the [`AssetMetadataName`].
        /// * `spec`: the asset metadata specifications ([`AssetMetadataSpec`]).
        #[weight = <T as Config>::WeightInfo::register_asset_metadata_global_type()]
        pub fn register_asset_metadata_global_type(
            origin,
            name: AssetMetadataName,
            spec: AssetMetadataSpec
        ) -> DispatchResult {
            // Only allow global metadata types to be registered by root.
            ensure_root(origin)?;

            Self::base_register_asset_metadata_global_type(name, spec)
        }

        /// Updates the type of an asset.
        ///
        /// # Arguments
        /// * `origin`: it contains the secondary key of the sender
        /// * `asset_id`: the [`AssetId`] associated to the token.
        /// * `asset_type`: the new [`AssetType`] of the token.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::update_asset_type()]
        pub fn update_asset_type(origin, asset_id: AssetId, asset_type: AssetType) -> DispatchResult {
            Self::base_update_asset_type(origin, asset_id, asset_type)
        }

        /// Removes the asset metadata key and value of a local key.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] associated to the local metadata key.
        /// * `local_key`: the [`AssetMetadataLocalKey`] that will be removed.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_local_metadata_key()]
        pub fn remove_local_metadata_key(origin, asset_id: AssetId, local_key: AssetMetadataLocalKey) -> DispatchResult {
            Self::base_remove_local_metadata_key(origin, asset_id, local_key)
        }

        /// Removes the asset metadata value of a metadata key.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] associated to the metadata key.
        /// * `metadata_key`: the [`AssetMetadataKey`] that will have its value deleted.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_metadata_value()]
        pub fn remove_metadata_value(origin, asset_id: AssetId, metadata_key: AssetMetadataKey) -> DispatchResult {
            Self::base_remove_metadata_value(origin, asset_id, metadata_key)
        }

        /// Pre-approves the receivement of the asset for all identities.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] that will be exempt from affirmation.
        ///
        /// # Permissions
        /// * Root
        #[weight = <T as Config>::WeightInfo::exempt_asset_affirmation()]
        pub fn exempt_asset_affirmation(origin, asset_id: AssetId) -> DispatchResult {
            Self::base_exempt_asset_affirmation(origin, asset_id)
        }

        /// Removes the pre-approval of the asset for all identities.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] that will have its exemption removed.
        ///
        /// # Permissions
        /// * Root
        #[weight = <T as Config>::WeightInfo::remove_asset_affirmation_exemption()]
        pub fn remove_asset_affirmation_exemption(origin, asset_id: AssetId) -> DispatchResult {
            Self::base_remove_asset_affirmation_exemption(origin, asset_id)
        }

        /// Pre-approves the receivement of an asset.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] that will be exempt from affirmation.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::pre_approve_asset()]
        pub fn pre_approve_asset(origin, asset_id: AssetId) -> DispatchResult {
            Self::base_pre_approve_asset(origin, asset_id)
        }

        /// Removes the pre approval of an asset.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] that will have its exemption removed.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_asset_pre_approval()]
        pub fn remove_asset_pre_approval(origin, asset_id: AssetId) -> DispatchResult {
            Self::base_remove_asset_pre_approval(origin, asset_id)
        }

        /// Sets all identities in the `mediators` set as mandatory mediators for any instruction transfering `asset_id`.
        ///
        /// # Arguments
        /// * `origin`: The secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] of the asset that will require the mediators.
        /// * `mediators`: A set of [`IdentityId`] of all the mandatory mediators for the given ticker.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_mandatory_mediators(mediators.len() as u32)]
        pub fn add_mandatory_mediators(
            origin,
            asset_id: AssetId,
            mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>
        ) {
            Self::base_add_mandatory_mediators(origin, asset_id, mediators)?;
        }

        /// Removes all identities in the `mediators` set from the mandatory mediators list for the given `asset_id`.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `asset_id`: the [`AssetId`] of the asset that will have mediators removed.
        /// * `mediators`: A set of [`IdentityId`] of all the mediators that will be removed from the mandatory mediators list.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_mandatory_mediators(mediators.len() as u32)]
        pub fn remove_mandatory_mediators(
            origin,
            asset_id: AssetId,
            mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>
        ) {
            Self::base_remove_mandatory_mediators(origin, asset_id, mediators)?;
        }

        /// Establishes a connection between a ticker and an AssetId.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `ticker`: the [`Ticker`] that will be linked to the given `asset_id`.
        /// * `asset_id`: the [`AssetId`] that will be connected to `ticker`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::link_ticker_to_asset_id()]
        pub fn link_ticker_to_asset_id(origin, ticker: Ticker, asset_id: AssetId) {
            Self::base_link_ticker_to_asset_id(origin, ticker, asset_id)?;
        }

        /// Removes the link between a ticker and an asset.
        ///
        /// # Arguments
        /// * `origin`: the secondary key of the sender.
        /// * `ticker`: the [`Ticker`] that will be unlinked from the given `asset_id`.
        /// * `asset_id`: the [`AssetId`] that will be unlink from `ticker`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::unlink_ticker_from_asset_id()]
        pub fn unlink_ticker_from_asset_id(origin, ticker: Ticker, asset_id: AssetId) {
            Self::base_unlink_ticker_from_asset_id(origin, ticker, asset_id)?;
        }
    }
}

//==========================================================================
// All base functions!
//==========================================================================

impl<T: Config> Module<T> {
    /// Registers `ticker` to the caller.
    fn base_register_unique_ticker(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;

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
        let to = Identity::<T>::ensure_perms(origin)?;
        <Identity<T>>::accept_auth_with(&to.into(), auth_id, |data, auth_by| {
            let ticker = extract_auth!(data, TransferTicker(t));

            Self::ensure_ticker_not_linked(&ticker)?;

            let ticker_registration = UniqueTickerRegistration::<T>::get(&ticker)
                .ok_or(Error::<T>::TickerRegistrationNotFound)?;

            <Identity<T>>::ensure_auth_by(auth_by, ticker_registration.owner)?;

            Self::transfer_ticker(ticker_registration, ticker, to);
            Ok(())
        })
    }

    /// Accept and process a token ownership transfer.
    fn base_accept_token_ownership_transfer(
        origin: T::RuntimeOrigin,
        auth_id: u64,
    ) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;

        <Identity<T>>::accept_auth_with(&caller_did.into(), auth_id, |auth_data, auth_by| {
            let asset_id = extract_auth!(auth_data, TransferAssetOwnership(asset_id));

            let mut asset_details = Self::try_get_asset_details(&asset_id)?;

            // Ensure the authorization was created by a permissioned agent.
            <ExternalAgents<T>>::ensure_agent_permissioned(&asset_id, auth_by)?;

            // If the asset is linked to a unique ticker, the ticker registration must be updated.
            if let Some(ticker) = AssetIdTicker::get(&asset_id) {
                let ticker_registration = UniqueTickerRegistration::<T>::try_get(&ticker)
                    .map_err(|_| Error::<T>::TickerRegistrationNotFound)?;
                Self::transfer_ticker(ticker_registration, ticker, caller_did);
            }

            // Updates token ownership
            let previous_owner = asset_details.owner_did;
            SecurityTokensOwnedByUser::remove(previous_owner, asset_id);
            SecurityTokensOwnedByUser::insert(caller_did, asset_id, true);

            // Updates token details.
            asset_details.owner_did = caller_did;
            Assets::insert(asset_id, asset_details);
            Self::deposit_event(RawEvent::AssetOwnershipTransferred(
                caller_did,
                asset_id,
                previous_owner,
            ));
            Ok(())
        })
    }

    /// If all rules for creating an asset are being respected, creates a new [`AssetDetails`].
    /// See also [`Module::validate_asset_creation_rules`].
    fn base_create_asset(
        origin: T::RuntimeOrigin,
        asset_name: AssetName,
        divisible: bool,
        asset_type: AssetType,
        asset_identifiers: Vec<AssetIdentifier>,
        funding_round_name: Option<FundingRoundName>,
    ) -> Result<IdentityId, DispatchError> {
        let caller_data = Identity::<T>::ensure_origin_call_permissions(origin)?;
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

    /// Freezes or unfreezes transfers for the token associated to `asset_id`.
    fn base_set_freeze(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        freeze: bool,
    ) -> DispatchResult {
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;

        Self::ensure_asset_exists(&asset_id)?;

        if freeze {
            ensure!(Frozen::get(&asset_id) == false, Error::<T>::AlreadyFrozen);
            Frozen::insert(asset_id, true);
            Self::deposit_event(RawEvent::AssetFrozen(caller_did, asset_id));
        } else {
            ensure!(Frozen::get(&asset_id) == true, Error::<T>::NotFrozen);
            Frozen::insert(asset_id, false);
            Self::deposit_event(RawEvent::AssetUnfrozen(caller_did, asset_id));
        }

        Ok(())
    }

    /// If `asset_name` is valid, updates the [`AssetName`] of the underling token given by `asset_id`.
    fn base_rename_asset(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        asset_name: AssetName,
    ) -> DispatchResult {
        Self::ensure_valid_asset_name(&asset_name)?;
        Self::ensure_asset_exists(&asset_id)?;

        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;

        AssetNames::insert(asset_id, asset_name.clone());
        Self::deposit_event(RawEvent::AssetRenamed(caller_did, asset_id, asset_name));
        Ok(())
    }

    /// Issues `amount_to_issue` tokens for `asset_id` into the caller's portfolio.
    fn base_issue(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        amount_to_issue: Balance,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        let caller_portfolio = Self::ensure_origin_asset_and_portfolio_permissions(
            origin,
            asset_id,
            portfolio_kind,
            false,
        )?;
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let mut asset_details = Self::try_get_asset_details(&asset_id)?;
        Self::validate_issuance_rules(&asset_details, amount_to_issue)?;
        Self::unverified_issue_tokens(
            asset_id,
            &mut asset_details,
            caller_portfolio,
            amount_to_issue,
            true,
            &mut weight_meter,
        )?;
        Ok(())
    }

    /// Reduces `value` tokens from `portfolio_kind` and [`AssetDetails::total_supply`].
    fn base_redeem(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        value: Balance,
        portfolio_kind: PortfolioKind,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let portfolio = Self::ensure_origin_asset_and_portfolio_permissions(
            origin,
            asset_id,
            portfolio_kind,
            true,
        )?;

        let mut asset_details = Self::try_get_asset_details(&asset_id)?;
        Self::ensure_token_granular(&asset_details, &value)?;

        // Ensures the token is fungible
        ensure!(
            asset_details.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        Portfolio::<T>::reduce_portfolio_balance(&portfolio, &asset_id, value)?;

        asset_details.total_supply = asset_details
            .total_supply
            .checked_sub(value)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;

        <Checkpoint<T>>::advance_update_balances(
            &asset_id,
            &[(portfolio.did, Self::balance_of(asset_id, portfolio.did))],
        )?;

        let updated_balance = Self::balance_of(asset_id, portfolio.did) - value;

        // Update identity balances and total supply
        BalanceOf::insert(asset_id, &portfolio.did, updated_balance);
        Assets::insert(asset_id, asset_details);

        // Update statistic info.
        Statistics::<T>::update_asset_stats(
            asset_id,
            Some(&portfolio.did),
            None,
            Some(updated_balance),
            None,
            value,
            weight_meter,
        )?;

        Self::deposit_event(RawEvent::AssetBalanceUpdated(
            portfolio.did,
            asset_id,
            value,
            Some(portfolio),
            None,
            PortfolioUpdateReason::Redeemed,
        ));
        Ok(())
    }

    fn base_make_divisible(origin: T::RuntimeOrigin, asset_id: AssetId) -> DispatchResult {
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;

        Assets::try_mutate(&asset_id, |asset_details| -> DispatchResult {
            let asset_details = asset_details.as_mut().ok_or(Error::<T>::NoSuchAsset)?;
            ensure!(
                asset_details.asset_type.is_fungible(),
                Error::<T>::UnexpectedNonFungibleToken
            );
            ensure!(!asset_details.divisible, Error::<T>::AssetAlreadyDivisible);
            asset_details.divisible = true;

            Self::deposit_event(RawEvent::DivisibilityChanged(caller_did, asset_id, true));
            Ok(())
        })
    }

    fn base_add_documents(
        origin: T::RuntimeOrigin,
        docs: Vec<Document>,
        asset_id: AssetId,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;

        // Ensure strings are limited.
        for doc in &docs {
            ensure_string_limited::<T>(&doc.uri)?;
            ensure_string_limited::<T>(&doc.name)?;
            ensure_opt_string_limited::<T>(doc.doc_type.as_deref())?;
        }

        // Ensure we can advance documents ID sequence by `len`.
        let pre = AssetDocumentsIdSequence::try_mutate(asset_id, |id| {
            id.0.checked_add(docs.len() as u32)
                .ok_or(CounterOverflow::<T>)
                .map(|new| mem::replace(id, DocumentId(new)))
        })?;

        // Charge fee.
        T::ProtocolFee::batch_charge_fee(ProtocolOp::AssetAddDocuments, docs.len())?;

        // Add the documents & emit events.
        for (id, doc) in (pre.0..).map(DocumentId).zip(docs) {
            AssetDocuments::insert(asset_id, id, doc.clone());
            Self::deposit_event(RawEvent::DocumentAdded(did, asset_id, id, doc));
        }
        Ok(())
    }

    fn base_remove_documents(
        origin: T::RuntimeOrigin,
        docs_id: Vec<DocumentId>,
        asset_id: AssetId,
    ) -> DispatchResult {
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
        for doc_id in docs_id {
            AssetDocuments::remove(asset_id, doc_id);
            Self::deposit_event(RawEvent::DocumentRemoved(caller_did, asset_id, doc_id));
        }
        Ok(())
    }

    fn base_set_funding_round(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        funding_round_name: FundingRoundName,
    ) -> DispatchResult {
        Self::ensure_valid_funding_round_name(&funding_round_name)?;
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;

        FundingRound::insert(asset_id, funding_round_name.clone());
        Self::deposit_event(RawEvent::FundingRoundSet(
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
        let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
        Self::ensure_valid_asset_identifiers(&asset_identifiers)?;
        Self::unverified_update_asset_identifiers(did, asset_id, asset_identifiers);
        Ok(())
    }

    fn base_controller_transfer(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        transfer_value: Balance,
        sender_portfolio: PortfolioId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let caller_portfolio = Self::ensure_origin_asset_and_portfolio_permissions(
            origin,
            asset_id,
            PortfolioKind::Default,
            false,
        )?;

        Self::validate_asset_transfer(
            asset_id,
            &sender_portfolio,
            &caller_portfolio,
            transfer_value,
            true,
            weight_meter,
        )?;

        Self::unverified_transfer_asset(
            sender_portfolio,
            caller_portfolio,
            asset_id,
            transfer_value,
            None,
            None,
            caller_portfolio.did,
            weight_meter,
        )?;

        Self::deposit_event(RawEvent::ControllerTransfer(
            caller_portfolio.did,
            asset_id,
            sender_portfolio,
            transfer_value,
        ));
        Ok(())
    }

    /// Registers a new custom asset type.
    fn base_register_custom_asset_type(
        origin: T::RuntimeOrigin,
        asset_type_bytes: Vec<u8>,
    ) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;
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
        let caller_data = Identity::<T>::ensure_origin_call_permissions(origin)?;

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
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;

        Self::unverified_set_asset_metadata(caller_did, asset_id, key, value, detail)
    }

    fn base_set_asset_metadata_details(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        key: AssetMetadataKey,
        detail: AssetMetadataValueDetail<T::Moment>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;

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
            AssetMetadataValues::try_get(&asset_id, &key)
                .map_err(|_| Error::<T>::AssetMetadataValueIsEmpty)?;
        }

        // Set asset metadata value details.
        AssetMetadataValueDetails::<T>::insert(asset_id, key, &detail);

        Self::deposit_event(RawEvent::SetAssetMetadataValueDetails(
            did, asset_id, detail,
        ));
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
        let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;

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
        let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;

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
            !AssetMetadataGlobalNameToKey::contains_key(&name),
            Error::<T>::AssetMetadataGlobalKeyAlreadyExists
        );

        // Next global key.
        let key = Self::update_current_asset_metadata_global_key()?;

        // Store global key <-> name mapping.
        AssetMetadataGlobalNameToKey::insert(&name, key);
        AssetMetadataGlobalKeyToName::insert(key, &name);

        // Store global specs.
        AssetMetadataGlobalSpecs::insert(key, &spec);

        Self::deposit_event(RawEvent::RegisterAssetMetadataGlobalType(name, key, spec));
        Ok(())
    }

    fn base_update_asset_type(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        asset_type: AssetType,
    ) -> DispatchResult {
        Self::ensure_asset_exists(&asset_id)?;
        Self::ensure_valid_asset_type(&asset_type)?;
        let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
        Assets::try_mutate(&asset_id, |token| -> DispatchResult {
            let token = token.as_mut().ok_or(Error::<T>::NoSuchAsset)?;
            // Ensures that both parameters are non fungible types or if both are fungible types.
            ensure!(
                token.asset_type.is_fungible() == asset_type.is_fungible(),
                Error::<T>::IncompatibleAssetTypeUpdate
            );
            token.asset_type = asset_type;
            Ok(())
        })?;
        Self::deposit_event(RawEvent::AssetTypeChanged(did, asset_id, asset_type));
        Ok(())
    }

    fn base_remove_local_metadata_key(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        local_key: AssetMetadataLocalKey,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
        // Verifies if the key exists.
        let name = AssetMetadataLocalKeyToName::try_get(asset_id, &local_key)
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
        AssetMetadataValues::remove(&asset_id, &metadata_key);
        AssetMetadataValueDetails::<T>::remove(&asset_id, &metadata_key);
        AssetMetadataLocalNameToKey::remove(&asset_id, &name);
        AssetMetadataLocalKeyToName::remove(&asset_id, &local_key);
        AssetMetadataLocalSpecs::remove(&asset_id, &local_key);
        Self::deposit_event(RawEvent::LocalMetadataKeyDeleted(
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
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
        // Verifies if the key exists.
        match metadata_key {
            AssetMetadataKey::Global(global_key) => {
                if !AssetMetadataGlobalKeyToName::contains_key(&global_key) {
                    return Err(Error::<T>::AssetMetadataKeyIsMissing.into());
                }
            }
            AssetMetadataKey::Local(local_key) => {
                if !AssetMetadataLocalKeyToName::contains_key(asset_id, &local_key) {
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
        AssetMetadataValues::remove(&asset_id, &metadata_key);
        AssetMetadataValueDetails::<T>::remove(&asset_id, &metadata_key);
        Self::deposit_event(RawEvent::MetadataValueDeleted(
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
        AssetsExemptFromAffirmation::insert(&asset_id, true);
        Self::deposit_event(RawEvent::AssetAffirmationExemption(asset_id));
        Ok(())
    }

    /// Removes the pre-approval of the asset for all identities.
    fn base_remove_asset_affirmation_exemption(
        origin: T::RuntimeOrigin,
        assset_id: AssetId,
    ) -> DispatchResult {
        ensure_root(origin)?;
        AssetsExemptFromAffirmation::remove(&assset_id);
        Self::deposit_event(RawEvent::RemoveAssetAffirmationExemption(assset_id));
        Ok(())
    }

    /// Pre-approves the receivement of an asset.
    fn base_pre_approve_asset(origin: T::RuntimeOrigin, asset_id: AssetId) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;
        PreApprovedAsset::insert(&caller_did, &asset_id, true);
        Self::deposit_event(RawEvent::PreApprovedAsset(caller_did, asset_id));
        Ok(())
    }

    /// Removes the pre approval of an asset.
    fn base_remove_asset_pre_approval(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
    ) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;
        PreApprovedAsset::remove(&caller_did, &asset_id);
        Self::deposit_event(RawEvent::RemovePreApprovedAsset(caller_did, asset_id));
        Ok(())
    }

    /// Sets all identities in the `mediators` set as mandatory mediators for any instruction transfering `asset_id`.
    fn base_add_mandatory_mediators(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        new_mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
        // Tries to add all new identities as mandatory mediators for the asset
        MandatoryMediators::<T>::try_mutate(asset_id, |mandatory_mediators| -> DispatchResult {
            for new_mediator in &new_mediators {
                mandatory_mediators
                    .try_insert(*new_mediator)
                    .map_err(|_| Error::<T>::NumberOfAssetMediatorsExceeded)?;
            }
            Ok(())
        })?;

        Self::deposit_event(RawEvent::AssetMediatorsAdded(
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
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
        // Removes the identities from the mandatory mediators list
        MandatoryMediators::<T>::mutate(asset_id, |mandatory_mediators| {
            for mediator in &mediators {
                mandatory_mediators.remove(mediator);
            }
        });
        Self::deposit_event(RawEvent::AssetMediatorsRemoved(
            caller_did,
            asset_id,
            mediators.into_inner(),
        ));
        Ok(())
    }

    /// Transfers an asset from one identity portfolio to another
    pub fn base_transfer(
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
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

        Self::validate_asset_transfer(
            asset_id,
            &from_portfolio,
            &to_portfolio,
            transfer_value,
            false,
            weight_meter,
        )?;

        Self::unverified_transfer_asset(
            from_portfolio,
            to_portfolio,
            asset_id,
            transfer_value,
            instruction_id,
            instruction_memo,
            caller_did,
            weight_meter,
        )?;

        Ok(())
    }

    pub fn base_link_ticker_to_asset_id(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        asset_id: AssetId,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
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
            !AssetIdTicker::contains_key(asset_id),
            Error::<T>::AssetIsAlreadyLinkedToATicker
        );
        // Links the ticker to the asset
        TickerAssetId::insert(ticker, asset_id);
        AssetIdTicker::insert(asset_id, ticker);
        Self::deposit_event(RawEvent::TickerLinkedToAsset(caller_did, ticker, asset_id));
        Ok(())
    }

    pub fn base_unlink_ticker_from_asset_id(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        asset_id: AssetId,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = ExternalAgents::<T>::ensure_perms(origin, asset_id)?;

        // The caller must own the ticker
        let ticker_registration = UniqueTickerRegistration::<T>::take(ticker)
            .ok_or(Error::<T>::TickerRegistrationNotFound)?;
        ensure!(
            ticker_registration.owner == caller_did,
            Error::<T>::TickerNotRegisteredToCaller
        );

        // The ticker must be linked to the given asset
        ensure!(
            TickerAssetId::get(ticker) == Some(asset_id),
            Error::<T>::TickerIsNotLinkedToTheAsset
        );

        // Removes the storage links
        TickersOwnedByUser::remove(caller_did, ticker);
        TickerAssetId::remove(ticker);
        AssetIdTicker::remove(asset_id);
        Self::deposit_event(RawEvent::TickerUnlinkedFromAsset(
            caller_did, ticker, asset_id,
        ));
        Ok(())
    }
}

//==========================================================================
// All validattion functions!
//==========================================================================

impl<T: Config> Module<T> {
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
            !TickerAssetId::contains_key(ticker),
            Error::<T>::TickerIsAlreadyLinkedToAnAsset
        );
        Ok(())
    }

    /// Returns `Ok` if `ticker` length is less or equal to `max_ticker_length`. Otherwise, returns [`Error::TickerTooLong`].
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
        caller_did: IdentityId,
        secondary_key: Option<SecondaryKey<T::AccountId>>,
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

        // Ensure that the caller has relevant portfolio permissions
        Portfolio::<T>::ensure_portfolio_custody_and_permission(
            PortfolioId::default_portfolio(caller_did),
            caller_did,
            secondary_key.as_ref(),
        )?;

        Ok(())
    }

    /// Returns the [`AssetDetails`] associated to `asset_id`, if one exists. Otherwise, returns [`Error::NoSuchAsset`].
    pub fn try_get_asset_details(asset_id: &AssetId) -> Result<AssetDetails, DispatchError> {
        let asset_details = Assets::try_get(asset_id).or(Err(Error::<T>::NoSuchAsset))?;
        Ok(asset_details)
    }

    /// Returns `Ok` if `funding_round_name` is valid. Otherwise, returns [`Error::FundingRoundNameMaxLengthExceeded`].
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
                CustomTypes::contains_key(custom_type_id),
                Error::<T>::InvalidCustomAssetTypeId
            );
        }
        Ok(())
    }

    /// Returns `Ok` if all `asset_identifiers` are valid. Otherwise, returns [`Error::InvalidAssetIdentifier`].
    fn ensure_valid_asset_identifiers(asset_identifiers: &[AssetIdentifier]) -> DispatchResult {
        ensure!(
            asset_identifiers.iter().all(|i| i.is_valid()),
            Error::<T>::InvalidAssetIdentifier
        );
        Ok(())
    }

    /// Ensures that `origin` is a permissioned agent for `asset_id`, that the portfolio is valid and that calller
    /// has the access to the portfolio. If `ensure_custody` is `true`, also enforces the caller to have custody
    /// of the portfolio.
    pub fn ensure_origin_asset_and_portfolio_permissions(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        portfolio_kind: PortfolioKind,
        ensure_custody: bool,
    ) -> Result<PortfolioId, DispatchError> {
        let origin_data = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, asset_id)?;
        let portfolio_id = PortfolioId::new(origin_data.primary_did, portfolio_kind);
        Portfolio::<T>::ensure_portfolio_validity(&portfolio_id)?;
        if ensure_custody {
            Portfolio::<T>::ensure_portfolio_custody(portfolio_id, origin_data.primary_did)?;
        }
        Portfolio::<T>::ensure_user_portfolio_permission(
            origin_data.secondary_key.as_ref(),
            portfolio_id,
        )?;
        Ok(portfolio_id)
    }

    /// Returns `Ok` if all rules for creating a custom type are satisfied.
    fn validate_custom_asset_type_rules(asset_type_bytes: &[u8]) -> DispatchResult {
        ensure_string_limited::<T>(asset_type_bytes)?;
        Ok(())
    }

    /// Returns `Ok` if [`AssetDetails::divisible`] or `value` % ONE_UNIT == 0. Otherwise, returns [`Error::<T>::InvalidGranularity`].
    fn ensure_token_granular(asset_details: &AssetDetails, value: &Balance) -> DispatchResult {
        if asset_details.divisible || value % ONE_UNIT == 0 {
            return Ok(());
        }
        Err(Error::<T>::InvalidGranularity.into())
    }

    /// Returns `true` if [`AssetDetails::divisible`], otherwise returns `false`.
    pub fn is_divisible(asset_id: &AssetId) -> bool {
        Assets::get(asset_id)
            .map(|t| t.divisible)
            .unwrap_or_default()
    }

    pub fn check_asset_metadata_key_exists(asset_id: &AssetId, key: &AssetMetadataKey) -> bool {
        match key {
            AssetMetadataKey::Global(key) => AssetMetadataGlobalKeyToName::contains_key(key),
            AssetMetadataKey::Local(key) => {
                AssetMetadataLocalKeyToName::contains_key(asset_id, key)
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
        let token = Assets::try_get(asset_id).ok()?;
        Some(token.asset_type.is_non_fungible())
    }

    /// Ensure that the document `doc` exists for `ticker`.
    pub fn ensure_doc_exists(asset_id: &AssetId, doc: &DocumentId) -> DispatchResult {
        ensure!(
            AssetDocuments::contains_key(asset_id, doc),
            Error::<T>::NoSuchDoc
        );
        Ok(())
    }

    pub fn get_balance_at(asset_id: AssetId, did: IdentityId, at: CheckpointId) -> Balance {
        <Checkpoint<T>>::balance_at(asset_id, did, at)
            .unwrap_or_else(|| Self::balance_of(&asset_id, &did))
    }

    pub fn validate_asset_transfer(
        asset_id: AssetId,
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        transfer_value: Balance,
        is_controller_transfer: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let asset_details = Self::try_get_asset_details(&asset_id)?;
        ensure!(
            asset_details.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        ensure!(
            BalanceOf::get(asset_id, &sender_portfolio.did) >= transfer_value,
            Error::<T>::InsufficientBalance
        );
        ensure!(
            BalanceOf::get(asset_id, &receiver_portfolio.did)
                .checked_add(transfer_value)
                .is_some(),
            Error::<T>::BalanceOverflow
        );

        // Verifies that both portfolios exist an that the sender portfolio has sufficient balance
        Portfolio::<T>::ensure_portfolio_transfer_validity(
            sender_portfolio,
            receiver_portfolio,
            &asset_id,
            transfer_value,
        )?;

        // Controllers are exempt from statistics, compliance and frozen rules.
        if is_controller_transfer {
            return Ok(());
        }

        // Verifies that the asset is not frozen
        ensure!(
            !Frozen::get(asset_id),
            Error::<T>::InvalidTransferFrozenAsset
        );

        ensure!(
            Identity::<T>::has_valid_cdd(receiver_portfolio.did),
            Error::<T>::InvalidTransferInvalidReceiverCDD
        );

        ensure!(
            Identity::<T>::has_valid_cdd(sender_portfolio.did),
            Error::<T>::InvalidTransferInvalidSenderCDD
        );

        // Verifies that the statistics restrictions are satisfied
        Statistics::<T>::verify_transfer_restrictions(
            asset_id,
            &sender_portfolio.did,
            &receiver_portfolio.did,
            Self::balance_of(asset_id, sender_portfolio.did),
            Self::balance_of(asset_id, receiver_portfolio.did),
            transfer_value,
            asset_details.total_supply,
            weight_meter,
        )?;

        // Verifies that all compliance rules are being respected
        if !T::ComplianceManager::is_compliant(
            &asset_id,
            sender_portfolio.did,
            receiver_portfolio.did,
            weight_meter,
        )? {
            return Err(Error::<T>::InvalidTransferComplianceFailure.into());
        }

        Ok(())
    }

    /// Returns a vector containing all errors for the transfer. An empty vec means there's no error.
    pub fn asset_transfer_report(
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        asset_id: &AssetId,
        transfer_value: Balance,
        skip_locked_check: bool,
        weight_meter: &mut WeightMeter,
    ) -> Vec<DispatchError> {
        let mut asset_transfer_errors = Vec::new();

        // If the security token doesn't exist or if the token is an NFT, there's no point in assessing anything else
        let asset_details = {
            match Assets::try_get(asset_id) {
                Ok(asset_details) => asset_details,
                Err(_) => return vec![Error::<T>::NoSuchAsset.into()],
            }
        };
        if !asset_details.asset_type.is_fungible() {
            return vec![Error::<T>::UnexpectedNonFungibleToken.into()];
        }

        if let Err(e) = Self::ensure_token_granular(&asset_details, &transfer_value) {
            asset_transfer_errors.push(e);
        }

        let sender_current_balance = BalanceOf::get(asset_id, &sender_portfolio.did);
        if sender_current_balance < transfer_value {
            asset_transfer_errors.push(Error::<T>::InsufficientBalance.into());
        }

        let receiver_current_balance = BalanceOf::get(asset_id, &receiver_portfolio.did);
        if receiver_current_balance
            .checked_add(transfer_value)
            .is_none()
        {
            asset_transfer_errors.push(Error::<T>::BalanceOverflow.into());
        }

        if sender_portfolio.did == receiver_portfolio.did {
            asset_transfer_errors
                .push(PortfolioError::<T>::InvalidTransferSenderIdMatchesReceiverId.into());
        }

        if let Err(e) = Portfolio::<T>::ensure_portfolio_validity(sender_portfolio) {
            asset_transfer_errors.push(e);
        }

        if let Err(e) = Portfolio::<T>::ensure_portfolio_validity(receiver_portfolio) {
            asset_transfer_errors.push(e);
        }

        if skip_locked_check {
            if PortfolioAssetBalances::get(sender_portfolio, asset_id) < transfer_value {
                asset_transfer_errors
                    .push(PortfolioError::<T>::InsufficientPortfolioBalance.into());
            }
        } else {
            if let Err(e) = Portfolio::<T>::ensure_sufficient_balance(
                sender_portfolio,
                asset_id,
                transfer_value,
            ) {
                asset_transfer_errors.push(e);
            }
        }

        if !Identity::<T>::has_valid_cdd(receiver_portfolio.did) {
            asset_transfer_errors.push(Error::<T>::InvalidTransferInvalidReceiverCDD.into());
        }

        if !Identity::<T>::has_valid_cdd(sender_portfolio.did) {
            asset_transfer_errors.push(Error::<T>::InvalidTransferInvalidSenderCDD.into());
        }

        if Frozen::get(asset_id) {
            asset_transfer_errors.push(Error::<T>::InvalidTransferFrozenAsset.into());
        }

        if let Err(e) = Statistics::<T>::verify_transfer_restrictions(
            *asset_id,
            &sender_portfolio.did,
            &receiver_portfolio.did,
            sender_current_balance,
            receiver_current_balance,
            transfer_value,
            asset_details.total_supply,
            weight_meter,
        ) {
            asset_transfer_errors.push(e);
        }

        match T::ComplianceManager::is_compliant(
            asset_id,
            sender_portfolio.did,
            receiver_portfolio.did,
            weight_meter,
        ) {
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
        Assets::get(asset_id)
            .map(|t| t.total_supply)
            .unwrap_or_default()
    }

    /// Calls [`Module::validate_asset_creation_rules`] and [`Module::unverified_create_asset`].
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
            caller_data.primary_did,
            caller_data.secondary_key,
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

    /// Returns `Ok` if all rules for issuing a token are satisfied.
    fn validate_issuance_rules(
        asset_details: &AssetDetails,
        amount_to_issue: Balance,
    ) -> DispatchResult {
        ensure!(
            asset_details.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        Self::ensure_token_granular(asset_details, &amount_to_issue)?;

        let new_supply = asset_details
            .total_supply
            .checked_add(amount_to_issue)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;
        ensure!(new_supply <= MAX_SUPPLY, Error::<T>::TotalSupplyAboveLimit);
        Ok(())
    }

    /// Returns `Ok` if there's no token associated to `asset_id`. Otherwise, returns [`Error::AssetIdGenerationError`].
    fn ensure_new_asset_id(asset_id: &AssetId) -> DispatchResult {
        ensure!(
            !Assets::contains_key(asset_id),
            Error::<T>::AssetIdGenerationError
        );
        Ok(())
    }

    /// Returns `Ok` if there's a token associated to `asset_id`. Otherwise, returns [`Error::NoSuchAsset`].
    fn ensure_asset_exists(asset_id: &AssetId) -> DispatchResult {
        ensure!(Assets::contains_key(asset_id), Error::<T>::NoSuchAsset);
        Ok(())
    }

    pub fn generate_asset_id(caller_acc: T::AccountId, update: bool) -> AssetId {
        let genesis_hash = frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero());
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
}

//==========================================================================
// All Storage Writes!
//==========================================================================

impl<T: Config> Module<T> {
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
            TickersOwnedByUser::remove(&ticker_registration.owner, &ticker);
        }

        // Write the ticker registration data to the storage
        UniqueTickerRegistration::<T>::insert(ticker, TickerRegistration { owner, expiry });
        TickersOwnedByUser::insert(owner, ticker, true);

        Self::deposit_event(RawEvent::TickerRegistered(owner, ticker, expiry));
        Ok(())
    }

    /// Transfer the given `ticker`'s registration from `req.owner` to `to`.
    fn transfer_ticker(mut reg: TickerRegistration<T::Moment>, ticker: Ticker, to: IdentityId) {
        let from = reg.owner;
        TickersOwnedByUser::remove(from, ticker);
        TickersOwnedByUser::insert(to, ticker, true);
        reg.owner = to;
        UniqueTickerRegistration::<T>::insert(&ticker, reg);
        Self::deposit_event(RawEvent::TickerTransferred(to, ticker, from));
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
        Assets::insert(asset_id, asset_details);

        AssetNames::insert(asset_id, &asset_name);
        if let Some(funding_round_name) = funding_round_name.as_ref() {
            FundingRound::insert(asset_id, funding_round_name);
        }

        SecurityTokensOwnedByUser::insert(caller_did, asset_id, true);
        Self::deposit_event(RawEvent::AssetCreated(
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

    /// Inserts `asset_identifiers` for the given `asset_id` and emits [`RawEvent::IdentifiersUpdated`].
    fn unverified_update_asset_identifiers(
        did: IdentityId,
        asset_id: AssetId,
        asset_identifiers: Vec<AssetIdentifier>,
    ) {
        AssetIdentifiers::insert(asset_id, asset_identifiers.clone());
        Self::deposit_event(RawEvent::IdentifiersUpdated(
            did,
            asset_id,
            asset_identifiers,
        ));
    }

    /// All storage writes for issuing a token.
    /// Note: if `charge_fee`` is `true` [`ProtocolOp::AssetIssue`] is charged.
    fn unverified_issue_tokens(
        asset_id: AssetId,
        asset_details: &mut AssetDetails,
        issuer_portfolio: PortfolioId,
        amount_to_issue: Balance,
        charge_fee: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        if charge_fee {
            T::ProtocolFee::charge_fee(ProtocolOp::AssetIssue)?;
        }

        let current_issuer_balance = BalanceOf::get(&asset_id, &issuer_portfolio.did);
        <Checkpoint<T>>::advance_update_balances(
            &asset_id,
            &[(issuer_portfolio.did, current_issuer_balance)],
        )?;

        let new_issuer_balance = current_issuer_balance + amount_to_issue;
        BalanceOf::insert(asset_id, issuer_portfolio.did, new_issuer_balance);

        asset_details.total_supply += amount_to_issue;
        Assets::insert(asset_id, asset_details);

        // No check since the total balance is always <= the total supply
        let new_issuer_portfolio_balance =
            Portfolio::<T>::portfolio_asset_balances(issuer_portfolio, asset_id) + amount_to_issue;
        Portfolio::<T>::set_portfolio_balance(
            issuer_portfolio,
            &asset_id,
            new_issuer_portfolio_balance,
        );

        Statistics::<T>::update_asset_stats(
            asset_id,
            None,
            Some(&issuer_portfolio.did),
            None,
            Some(new_issuer_balance),
            amount_to_issue,
            weight_meter,
        )?;

        let funding_round_name = FundingRound::get(&asset_id);
        IssuedInFundingRound::mutate((&asset_id, &funding_round_name), |balance| {
            *balance = balance.saturating_add(amount_to_issue)
        });

        Self::deposit_event(RawEvent::AssetBalanceUpdated(
            issuer_portfolio.did,
            asset_id,
            amount_to_issue,
            None,
            Some(issuer_portfolio),
            PortfolioUpdateReason::Issued {
                funding_round_name: Some(funding_round_name),
            },
        ));
        Ok(())
    }

    // Transfers `transfer_value` from `sender_portfolio` to `receiver_portfolio`.
    pub fn unverified_transfer_asset(
        sender_portfolio: PortfolioId,
        receiver_portfolio: PortfolioId,
        asset_id: AssetId,
        transfer_value: Balance,
        instruction_id: Option<InstructionId>,
        instruction_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Gets the current balance and advances the checkpoint
        let sender_current_balance = BalanceOf::get(&asset_id, &sender_portfolio.did);
        let receiver_current_balance = BalanceOf::get(&asset_id, &receiver_portfolio.did);
        <Checkpoint<T>>::advance_update_balances(
            &asset_id,
            &[
                (sender_portfolio.did, sender_current_balance),
                (receiver_portfolio.did, receiver_current_balance),
            ],
        )?;

        // Updates the balance in the asset pallet
        let sender_new_balance = sender_current_balance - transfer_value;
        let receiver_new_balance = receiver_current_balance + transfer_value;
        BalanceOf::insert(asset_id, sender_portfolio.did, sender_new_balance);
        BalanceOf::insert(asset_id, receiver_portfolio.did, receiver_new_balance);

        // Updates the balances in the portfolio pallet
        Portfolio::<T>::unchecked_transfer_portfolio_balance(
            &sender_portfolio,
            &receiver_portfolio,
            &asset_id,
            transfer_value,
        );

        // Update statistics info.
        Statistics::<T>::update_asset_stats(
            asset_id,
            Some(&sender_portfolio.did),
            Some(&receiver_portfolio.did),
            Some(sender_new_balance),
            Some(receiver_new_balance),
            transfer_value,
            weight_meter,
        )?;

        Self::deposit_event(RawEvent::AssetBalanceUpdated(
            caller_did,
            asset_id,
            transfer_value,
            Some(sender_portfolio),
            Some(receiver_portfolio),
            PortfolioUpdateReason::Transferred {
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
        match CustomTypesInverse::try_get(&asset_type_bytes) {
            Ok(type_id) => {
                Self::deposit_event(Event::<T>::CustomAssetTypeExists(
                    caller_did,
                    type_id,
                    asset_type_bytes,
                ));
                Ok(type_id)
            }
            Err(_) => {
                let type_id = CustomTypeIdSequence::try_mutate(try_next_pre::<T, _>)?;
                CustomTypesInverse::insert(&asset_type_bytes, type_id);
                CustomTypes::insert(type_id, &asset_type_bytes);
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
        AssetMetadataValues::insert(asset_id, key, &value);

        // Set asset metadata value details.
        if let Some(ref detail) = detail {
            AssetMetadataValueDetails::<T>::insert(asset_id, key, detail);
        }

        Self::deposit_event(RawEvent::SetAssetMetadataValue(
            did, asset_id, value, detail,
        ));
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
            !AssetMetadataLocalNameToKey::contains_key(asset_id, &name),
            Error::<T>::AssetMetadataLocalKeyAlreadyExists
        );

        // Next local key for asset.
        let key = Self::update_current_asset_metadata_local_key(&asset_id)?;

        // Store local key <-> name mapping.
        AssetMetadataLocalNameToKey::insert(asset_id, &name, key);
        AssetMetadataLocalKeyToName::insert(asset_id, key, &name);

        // Store local specs.
        AssetMetadataLocalSpecs::insert(asset_id, key, &spec);

        Self::deposit_event(RawEvent::RegisterAssetMetadataLocalType(
            did, asset_id, name, key, spec,
        ));
        Ok(key.into())
    }

    /// Adds one to `CurrentCollectionId`.
    fn update_current_asset_metadata_global_key() -> Result<AssetMetadataGlobalKey, DispatchError> {
        CurrentAssetMetadataGlobalKey::try_mutate(|current_global_key| match current_global_key {
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
        })
    }

    /// Adds one to the `AssetMetadataLocalKey` for the given `ticker`.
    fn update_current_asset_metadata_local_key(
        asset_id: &AssetId,
    ) -> Result<AssetMetadataLocalKey, DispatchError> {
        CurrentAssetMetadataLocalKey::try_mutate(asset_id, |current_local_key| {
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
}

//==========================================================================
// All RPC functions!
//==========================================================================

//==========================================================================
// Trait implementation!
//==========================================================================

impl<T: Config> AssetFnTrait<T::AccountId, T::RuntimeOrigin> for Module<T> {
    fn ensure_granular(asset_id: &AssetId, value: Balance) -> DispatchResult {
        let asset_details = Self::try_get_asset_details(&asset_id)?;
        Self::ensure_token_granular(&asset_details, &value)
    }

    fn skip_asset_affirmation(identity_id: &IdentityId, asset_id: &AssetId) -> bool {
        if AssetsExemptFromAffirmation::get(asset_id) {
            return true;
        }
        PreApprovedAsset::get(identity_id, asset_id)
    }

    fn asset_affirmation_exemption(asset_id: &AssetId) -> bool {
        AssetsExemptFromAffirmation::get(asset_id)
    }

    fn asset_balance(asset_id: &AssetId, did: &IdentityId) -> Balance {
        BalanceOf::get(asset_id, did)
    }

    fn asset_total_supply(asset_id: &AssetId) -> Result<Balance, DispatchError> {
        Ok(Self::try_get_asset_details(&asset_id)?.total_supply)
    }

    fn generate_asset_id(caller_acc: T::AccountId) -> AssetId {
        Self::generate_asset_id(caller_acc, false)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn register_unique_ticker(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        Self::register_unique_ticker(origin, ticker)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn create_asset(
        origin: T::RuntimeOrigin,
        asset_name: AssetName,
        divisible: bool,
        asset_type: AssetType,
        asset_identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult {
        Self::create_asset(
            origin,
            asset_name,
            divisible,
            asset_type,
            asset_identifiers,
            funding_round,
        )
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn issue(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        amount: Balance,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        Self::issue(origin, asset_id, amount, portfolio_kind)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn register_asset_metadata_type(
        origin: T::RuntimeOrigin,
        asset_id: Option<AssetId>,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        match asset_id {
            Some(asset_id) => {
                Self::register_asset_metadata_local_type(origin, asset_id, name, spec)
            }
            None => Self::register_asset_metadata_global_type(origin, name, spec),
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn add_mandatory_mediators(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        mediators: BTreeSet<IdentityId>,
    ) -> DispatchResult {
        Self::add_mandatory_mediators(origin, asset_id, mediators.try_into().unwrap_or_default())
    }
}
