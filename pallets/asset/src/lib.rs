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
//! - `is_ticker_available` - It checks whether the given ticker is available or not.
//! - `is_ticker_registry_valid` - It checks whether the ticker is owned by a given IdentityId or not.
//! - `is_ticker_available_or_registered_to` - It provides the status of a given ticker.
//! - `total_supply` - It provides the total supply of a ticker.
//! - `get_balance_at` - It provides the balance of a DID at a certain checkpoint.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod checkpoint;

mod base_functions;
mod base_validations_writes;
mod error;
mod rpc_functions;
mod trait_impl;
mod types;

use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchResult, Weight};
use frame_support::traits::{Get, PalletInfoAccess};
use frame_support::BoundedBTreeSet;
use frame_support::{decl_module, decl_storage};
use sp_std::prelude::*;

pub use polymesh_common_utilities::traits::asset::{Config, Event, RawEvent, WeightInfo};
use polymesh_primitives::asset::{AssetName, AssetType, CustomAssetTypeId, FundingRoundName};
use polymesh_primitives::asset_metadata::{
    AssetMetadataGlobalKey, AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName,
    AssetMetadataSpec, AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::{
    storage_migration_ver, AssetIdentifier, Balance, Document, DocumentId, IdentityId, PortfolioId,
    PortfolioKind, Ticker, WeightMeter,
};

pub use error::Error;
pub use types::{
    AssetOwnershipRelation, SecurityToken, TickerRegistration, TickerRegistrationConfig,
    TickerRegistrationStatus,
};

type Checkpoint<T> = checkpoint::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;
type Statistics<T> = pallet_statistics::Module<T>;

storage_migration_ver!(3);

decl_storage! {
    trait Store for Module<T: Config> as Asset {
        /// Ticker registration details.
        /// (ticker) -> TickerRegistration
        pub Tickers get(fn ticker_registration): map hasher(blake2_128_concat) Ticker => Option<TickerRegistration<T::Moment>>;
        /// Ticker registration config.
        /// (ticker) -> TickerRegistrationConfig
        pub TickerConfig get(fn ticker_registration_config) config(): TickerRegistrationConfig<T::Moment>;
        /// Details of the token corresponding to the token ticker.
        /// (ticker) -> SecurityToken details [returns SecurityToken struct]
        pub Tokens get(fn tokens): map hasher(blake2_128_concat) Ticker => Option<SecurityToken>;
        /// Asset name of the token corresponding to the token ticker.
        /// (ticker) -> `AssetName`
        pub AssetNames get(fn asset_names): map hasher(blake2_128_concat) Ticker => Option<AssetName>;
        /// The total asset ticker balance per identity.
        /// (ticker, DID) -> Balance
        // NB: It is safe to use `identity` hasher here because assets can not be distributed to non-existent identities.
        pub BalanceOf get(fn balance_of): double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => Balance;
        /// A map of a ticker name and asset identifiers.
        pub Identifiers get(fn identifiers): map hasher(blake2_128_concat) Ticker => Vec<AssetIdentifier>;

        /// The next `AssetType::Custom` ID in the sequence.
        ///
        /// Numbers in the sequence start from 1 rather than 0.
        pub CustomTypeIdSequence get(fn custom_type_id_seq): CustomAssetTypeId;
        /// Maps custom asset type ids to the registered string contents.
        pub CustomTypes get(fn custom_types): map hasher(twox_64_concat) CustomAssetTypeId => Vec<u8>;
        /// Inverse map of `CustomTypes`, from registered string contents to custom asset type ids.
        pub CustomTypesInverse get(fn custom_types_inverse): map hasher(blake2_128_concat) Vec<u8> => Option<CustomAssetTypeId>;

        /// The name of the current funding round.
        /// ticker -> funding round
        FundingRound get(fn funding_round): map hasher(blake2_128_concat) Ticker => FundingRoundName;
        /// The total balances of tokens issued in all recorded funding rounds.
        /// (ticker, funding round) -> balance
        IssuedInFundingRound get(fn issued_in_funding_round): map hasher(blake2_128_concat) (Ticker, FundingRoundName) => Balance;
        /// The set of frozen assets implemented as a membership map.
        /// ticker -> bool
        pub Frozen get(fn frozen): map hasher(blake2_128_concat) Ticker => bool;
        /// Tickers and token owned by a user
        /// (user, ticker) -> AssetOwnership
        pub AssetOwnershipRelations get(fn asset_ownership_relation):
            double_map hasher(identity) IdentityId, hasher(blake2_128_concat) Ticker => AssetOwnershipRelation;
        /// Documents attached to an Asset
        /// (ticker, doc_id) -> document
        pub AssetDocuments get(fn asset_documents):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) DocumentId => Option<Document>;
        /// Per-ticker document ID counter.
        /// (ticker) -> doc_id
        pub AssetDocumentsIdSequence get(fn asset_documents_id_sequence): map hasher(blake2_128_concat) Ticker => DocumentId;

        /// Metatdata values for an asset.
        pub AssetMetadataValues get(fn asset_metadata_values):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataKey =>
                Option<AssetMetadataValue>;
        /// Details for an asset's Metadata values.
        pub AssetMetadataValueDetails get(fn asset_metadata_value_details):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataKey =>
                Option<AssetMetadataValueDetail<T::Moment>>;

        /// Asset Metadata Local Name -> Key.
        pub AssetMetadataLocalNameToKey get(fn asset_metadata_local_name_to_key):
            double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) AssetMetadataName =>
                Option<AssetMetadataLocalKey>;
        /// Asset Metadata Global Name -> Key.
        pub AssetMetadataGlobalNameToKey get(fn asset_metadata_global_name_to_key):
            map hasher(blake2_128_concat) AssetMetadataName => Option<AssetMetadataGlobalKey>;

        /// Asset Metadata Local Key -> Name.
        pub AssetMetadataLocalKeyToName get(fn asset_metadata_local_key_to_name):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataLocalKey =>
                Option<AssetMetadataName>;
        /// Asset Metadata Global Key -> Name.
        pub AssetMetadataGlobalKeyToName get(fn asset_metadata_global_key_to_name):
            map hasher(twox_64_concat) AssetMetadataGlobalKey => Option<AssetMetadataName>;

        /// Asset Metadata Local Key specs.
        pub AssetMetadataLocalSpecs get(fn asset_metadata_local_specs):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AssetMetadataLocalKey =>
                Option<AssetMetadataSpec>;
        /// Asset Metadata Global Key specs.
        pub AssetMetadataGlobalSpecs get(fn asset_metadata_global_specs):
            map hasher(twox_64_concat) AssetMetadataGlobalKey => Option<AssetMetadataSpec>;

        /// Next Asset Metadata Local Key.
        pub AssetMetadataNextLocalKey get(fn asset_metadata_next_local_key):
            map hasher(blake2_128_concat) Ticker => AssetMetadataLocalKey;
        /// Next Asset Metadata Global Key.
        pub AssetMetadataNextGlobalKey get(fn asset_metadata_next_global_key): AssetMetadataGlobalKey;

        /// A list of tickers that exempt all users from affirming the receivement of the asset.
        pub TickersExemptFromAffirmation get(fn tickers_exempt_from_affirmation):
            map hasher(blake2_128_concat) Ticker => bool;

        /// All tickers that don't need an affirmation to be received by an identity.
        pub PreApprovedTicker get(fn pre_approved_tickers):
            double_map hasher(identity) IdentityId, hasher(blake2_128_concat) Ticker => bool;

        /// The list of mandatory mediators for every ticker.
        pub MandatoryMediators get(fn mandatory_mediators):
            map hasher(blake2_128_concat) Ticker => BoundedBTreeSet<IdentityId, T::MaxAssetMediators>;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(3)): Version;
    }
    add_extra_genesis {
        config(reserved_country_currency_codes): Vec<Ticker>;
        build(|config: &GenesisConfig<T>| {
            // Reserving country currency logic
            let fiat_tickers_reservation_did = polymesh_common_utilities::SystematicIssuers::FiatTickersReservation.as_id();
            for currency_ticker in &config.reserved_country_currency_codes {
                <Module<T>>::unverified_register_ticker(&currency_ticker, fiat_tickers_reservation_did, None);
            }
        });
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {

        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        const AssetNameMaxLength: u32 = T::AssetNameMaxLength::get();
        const FundingRoundNameMaxLength: u32 = T::FundingRoundNameMaxLength::get();

        const AssetMetadataNameMaxLength: u32 = T::AssetMetadataNameMaxLength::get();
        const AssetMetadataValueMaxLength: u32 = T::AssetMetadataValueMaxLength::get();
        const AssetMetadataTypeDefMaxLength: u32 = T::AssetMetadataTypeDefMaxLength::get();

        // Remove all storage related to classic tickers in this module
        fn on_runtime_upgrade() -> Weight {
            use polymesh_primitives::storage_migrate_on;
            storage_migrate_on!(StorageVersion, 3, {
                let prefixes = &[
                    "BalanceOfAtScope",
                    "AggregateBalance",
                    "ScopeIdOf",
                    "DisableInvestorUniqueness",
                ];
                for prefix in prefixes {
                    let res = frame_support::storage::migration::clear_storage_prefix(<Pallet<T>>::name().as_bytes(), prefix.as_bytes(), b"", None, None);
                    log::info!("Cleared storage prefix[{prefix}]: cursor={:?}, backend={}, unique={}, loops={}",
                        res.maybe_cursor, res.backend, res.unique, res.loops);
                }
            });
            Weight::zero()
        }

        /// Registers a new ticker or extends validity of an existing ticker.
        /// NB: Ticker validity does not get carry forward when renewing ticker.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `ticker` ticker to register.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::register_ticker()]
        pub fn register_ticker(origin, ticker: Ticker) -> DispatchResult {
            Self::base_register_ticker(origin, ticker)
        }

        /// Accepts a ticker transfer.
        ///
        /// Consumes the authorization `auth_id` (see `pallet_identity::consume_auth`).
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `auth_id` Authorization ID of ticker transfer authorization.
        ///
        /// ## Errors
        /// - `AuthorizationError::BadType` if `auth_id` is not a valid ticket transfer authorization.
        ///
        #[weight = <T as Config>::WeightInfo::accept_ticker_transfer()]
        pub fn accept_ticker_transfer(origin, auth_id: u64) -> DispatchResult {
            Self::base_accept_ticker_transfer(origin, auth_id)
        }

        /// This function is used to accept a token ownership transfer.
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `auth_id` Authorization ID of the token ownership transfer authorization.
        #[weight = <T as Config>::WeightInfo::accept_asset_ownership_transfer()]
        pub fn accept_asset_ownership_transfer(origin, auth_id: u64) -> DispatchResult {
            Self::base_accept_token_ownership_transfer(origin, auth_id)
        }

        /// Initializes a new security token, with the initiating account as its owner.
        /// The total supply will initially be zero. To mint tokens, use `issue`.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `name` - the name of the token.
        /// * `ticker` - the ticker symbol of the token.
        /// * `divisible` - a boolean to identify the divisibility status of the token.
        /// * `asset_type` - the asset type.
        /// * `identifiers` - a vector of asset identifiers.
        /// * `funding_round` - name of the funding round.
        ///
        /// ## Errors
        /// - `InvalidAssetIdentifier` if any of `identifiers` are invalid.
        /// - `MaxLengthOfAssetNameExceeded` if `name`'s length exceeds `T::AssetNameMaxLength`.
        /// - `FundingRoundNameMaxLengthExceeded` if the name of the funding round is longer that
        /// `T::FundingRoundNameMaxLength`.
        /// - `AssetAlreadyCreated` if asset was already created.
        /// - `TickerTooLong` if `ticker`'s length is greater than `config.max_ticker_length` chain
        /// parameter.
        /// - `TickerNotAlphanumeric` if `ticker` is not yet registered, and contains non-alphanumeric characters or any character after first occurrence of `\0`.
        ///
        /// ## Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::create_asset(
            name.len() as u32,
            identifiers.len() as u32,
            funding_round.as_ref().map_or(0, |name| name.len()) as u32
        )]
        pub fn create_asset(
            origin,
            name: AssetName,
            ticker: Ticker,
            divisible: bool,
            asset_type: AssetType,
            identifiers: Vec<AssetIdentifier>,
            funding_round: Option<FundingRoundName>,
        ) -> DispatchResult {
            Self::base_create_asset(origin, name, ticker, divisible, asset_type, identifiers, funding_round)
                .map(drop)
        }

        /// Freezes transfers of a given token.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the ticker of the token.
        ///
        /// ## Errors
        /// - `AlreadyFrozen` if `ticker` is already frozen.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::freeze()]
        pub fn freeze(origin, ticker: Ticker) -> DispatchResult {
            Self::base_set_freeze(origin, ticker, true)
        }

        /// Unfreezes transfers of a given token.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the ticker of the frozen token.
        ///
        /// ## Errors
        /// - `NotFrozen` if `ticker` is not frozen yet.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::unfreeze()]
        pub fn unfreeze(origin, ticker: Ticker) -> DispatchResult {
            Self::base_set_freeze(origin, ticker, false)
        }

        /// Renames a given token.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the ticker of the token.
        /// * `name` - the new name of the token.
        ///
        /// ## Errors
        /// - `MaxLengthOfAssetNameExceeded` if length of `name` is greater than
        /// `T::AssetNameMaxLength`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::rename_asset(ticker.len() as u32)]
        pub fn rename_asset(origin, ticker: Ticker, name: AssetName) -> DispatchResult {
            Self::base_rename_asset(origin, ticker, name)
        }

        /// Issue, or mint, new tokens to the caller, which must be an authorized external agent.
        ///
        /// # Arguments
        /// * `origin` - A signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` - The [`Ticker`] of the token.
        /// * `amount` - The amount of tokens that will be issued.
        /// * `portfolio_kind` - The [`PortfolioKind`] of the portfolio that will receive the minted tokens.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::issue()]
        pub fn issue(origin, ticker: Ticker, amount: Balance, portfolio_kind: PortfolioKind) -> DispatchResult {
            Self::base_issue(origin, ticker, amount, portfolio_kind)
        }

        /// Redeems existing tokens by reducing the balance of the caller's default portfolio and the total supply of the token
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` Ticker of the token.
        /// * `value` Amount of tokens to redeem.
        ///
        /// # Errors
        /// - `Unauthorized` If called by someone without the appropriate external agent permissions
        /// - `InvalidGranularity` If the amount is not divisible by 10^6 for non-divisible tokens
        /// - `InsufficientPortfolioBalance` If the caller's default portfolio doesn't have enough free balance
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::redeem()]
        pub fn redeem(origin, ticker: Ticker, value: Balance) -> DispatchResult {
            let mut weight_meter = WeightMeter::max_limit_no_minimum();
            Self::base_redeem(origin, ticker, value, PortfolioKind::Default, &mut weight_meter)
        }

        /// Makes an indivisible token divisible.
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` Ticker of the token.
        ///
        /// ## Errors
        /// - `AssetAlreadyDivisible` if `ticker` is already divisible.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::make_divisible()]
        pub fn make_divisible(origin, ticker: Ticker) -> DispatchResult {
            Self::base_make_divisible(origin, ticker)
        }

        /// Add documents for a given token.
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` Ticker of the token.
        /// * `docs` Documents to be attached to `ticker`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_documents(docs.len() as u32)]
        pub fn add_documents(origin, docs: Vec<Document>, ticker: Ticker) -> DispatchResult {
            Self::base_add_documents(origin, docs, ticker)
        }

        /// Remove documents for a given token.
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` Ticker of the token.
        /// * `ids` Documents ids to be removed from `ticker`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_documents(ids.len() as u32)]
        pub fn remove_documents(origin, ids: Vec<DocumentId>, ticker: Ticker) -> DispatchResult {
            Self::base_remove_documents(origin, ids, ticker)
        }

        /// Sets the name of the current funding round.
        ///
        /// # Arguments
        /// * `origin` - a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` - the ticker of the token.
        /// * `name` - the desired name of the current funding round.
        ///
        /// ## Errors
        /// - `FundingRoundNameMaxLengthExceeded` if length of `name` is greater than
        /// `T::FundingRoundNameMaxLength`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_funding_round( name.len() as u32 )]
        pub fn set_funding_round(origin, ticker: Ticker, name: FundingRoundName) -> DispatchResult {
            Self::base_set_funding_round(origin, ticker, name)
        }

        /// Updates the asset identifiers.
        ///
        /// # Arguments
        /// * `origin` - a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` - the ticker of the token.
        /// * `identifiers` - the asset identifiers to be updated in the form of a vector of pairs
        ///    of `IdentifierType` and `AssetIdentifier` value.
        ///
        /// ## Errors
        /// - `InvalidAssetIdentifier` if `identifiers` contains any invalid identifier.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::update_identifiers( identifiers.len() as u32)]
        pub fn update_identifiers(
            origin,
            ticker: Ticker,
            identifiers: Vec<AssetIdentifier>
        ) -> DispatchResult {
            Self::base_update_identifiers(origin, ticker, identifiers)
        }

        /// Forces a transfer of token from `from_portfolio` to the caller's default portfolio.
        ///
        /// # Arguments
        /// * `origin` Must be an external agent with appropriate permissions for a given ticker.
        /// * `ticker` Ticker symbol of the asset.
        /// * `value`  Amount of tokens need to force transfer.
        /// * `from_portfolio` From whom portfolio tokens gets transferred.
        #[weight = <T as Config>::WeightInfo::controller_transfer()]
        pub fn controller_transfer(origin, ticker: Ticker, value: Balance, from_portfolio: PortfolioId) -> DispatchResult {
            let mut weight_meter = WeightMeter::max_limit_no_minimum();
            Self::base_controller_transfer(origin, ticker, value, from_portfolio, &mut weight_meter)
        }

        /// Registers a custom asset type.
        ///
        /// The provided `ty` will be bound to an ID in storage.
        /// The ID can then be used in `AssetType::Custom`.
        /// Should the `ty` already exist in storage, no second ID is assigned to it.
        ///
        /// # Arguments
        /// * `origin` who called the extrinsic.
        /// * `ty` contains the string representation of the asset type.
        #[weight = <T as Config>::WeightInfo::register_custom_asset_type(ty.len() as u32)]
        pub fn register_custom_asset_type(origin, ty: Vec<u8>) -> DispatchResult {
            Self::base_register_custom_asset_type(origin, ty).map(drop)
        }

        /// Utility extrinsic to batch `create_asset` and `register_custom_asset_type`.
        #[weight = <T as Config>::WeightInfo::create_asset(
            name.len() as u32,
            identifiers.len() as u32,
            funding_round.as_ref().map_or(0, |name| name.len()) as u32
        ) + <T as Config>::WeightInfo::register_custom_asset_type(custom_asset_type.len() as u32)]
        pub fn create_asset_with_custom_type(
            origin,
            name: AssetName,
            ticker: Ticker,
            divisible: bool,
            custom_asset_type: Vec<u8>,
            identifiers: Vec<AssetIdentifier>,
            funding_round: Option<FundingRoundName>,
        ) -> DispatchResult {
            let origin_data = Identity::<T>::ensure_origin_call_permissions(origin)?;
            let asset_type_id = Self::unsafe_register_custom_asset_type(
                origin_data.primary_did,
                custom_asset_type,
            )?;
            Self::unsafe_create_asset(
                origin_data.primary_did,
                origin_data.secondary_key,
                name,
                ticker,
                divisible,
                AssetType::Custom(asset_type_id),
                identifiers,
                funding_round,
            )?;
            Ok(())
        }

        /// Set asset metadata value.
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` Ticker of the token.
        /// * `key` Metadata key.
        /// * `value` Metadata value.
        /// * `details` Optional Metadata value details (expire, lock status).
        ///
        /// # Errors
        /// * `AssetMetadataKeyIsMissing` if the metadata type key doesn't exist.
        /// * `AssetMetadataValueIsLocked` if the metadata value for `key` is locked.
        /// * `AssetMetadataValueMaxLengthExceeded` if the metadata value exceeds the maximum length.
        ///
        /// # Permissions
        /// * Agent
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_asset_metadata()]
        pub fn set_asset_metadata(origin, ticker: Ticker, key: AssetMetadataKey, value: AssetMetadataValue, detail: Option<AssetMetadataValueDetail<T::Moment>>) -> DispatchResult {
            Self::base_set_asset_metadata(origin, ticker, key, value, detail)
        }

        /// Set asset metadata value details (expire, lock status).
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` Ticker of the token.
        /// * `key` Metadata key.
        /// * `details` Metadata value details (expire, lock status).
        ///
        /// # Errors
        /// * `AssetMetadataKeyIsMissing` if the metadata type key doesn't exist.
        /// * `AssetMetadataValueIsLocked` if the metadata value for `key` is locked.
        ///
        /// # Permissions
        /// * Agent
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_asset_metadata_details()]
        pub fn set_asset_metadata_details(origin, ticker: Ticker, key: AssetMetadataKey, detail: AssetMetadataValueDetail<T::Moment>) -> DispatchResult {
            Self::base_set_asset_metadata_details(origin, ticker, key, detail)
        }

        /// Registers and set local asset metadata.
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` Ticker of the token.
        /// * `name` Metadata name.
        /// * `spec` Metadata type definition.
        /// * `value` Metadata value.
        /// * `details` Optional Metadata value details (expire, lock status).
        ///
        /// # Errors
        /// * `AssetMetadataLocalKeyAlreadyExists` if a local metadata type with `name` already exists for `ticker`.
        /// * `AssetMetadataNameMaxLengthExceeded` if the metadata `name` exceeds the maximum length.
        /// * `AssetMetadataTypeDefMaxLengthExceeded` if the metadata `spec` type definition exceeds the maximum length.
        /// * `AssetMetadataValueMaxLengthExceeded` if the metadata value exceeds the maximum length.
        ///
        /// # Permissions
        /// * Agent
        /// * Asset
        #[weight = <T as Config>::WeightInfo::register_and_set_local_asset_metadata()]
        pub fn register_and_set_local_asset_metadata(origin, ticker: Ticker, name: AssetMetadataName, spec: AssetMetadataSpec, value: AssetMetadataValue, detail: Option<AssetMetadataValueDetail<T::Moment>>) -> DispatchResult {
            Self::base_register_and_set_local_asset_metadata(origin, ticker, name, spec, value, detail)
        }

        /// Registers asset metadata local type.
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` Ticker of the token.
        /// * `name` Metadata name.
        /// * `spec` Metadata type definition.
        ///
        /// # Errors
        /// * `AssetMetadataLocalKeyAlreadyExists` if a local metadata type with `name` already exists for `ticker`.
        /// * `AssetMetadataNameMaxLengthExceeded` if the metadata `name` exceeds the maximum length.
        /// * `AssetMetadataTypeDefMaxLengthExceeded` if the metadata `spec` type definition exceeds the maximum length.
        ///
        /// # Permissions
        /// * Agent
        /// * Asset
        #[weight = <T as Config>::WeightInfo::register_asset_metadata_local_type()]
        pub fn register_asset_metadata_local_type(origin, ticker: Ticker, name: AssetMetadataName, spec: AssetMetadataSpec) -> DispatchResult {
            Self::base_register_asset_metadata_local_type(origin, ticker, name, spec)
        }

        /// Registers asset metadata global type.
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `name` Metadata name.
        /// * `spec` Metadata type definition.
        ///
        /// # Errors
        /// * `AssetMetadataGlobalKeyAlreadyExists` if a globa metadata type with `name` already exists.
        /// * `AssetMetadataNameMaxLengthExceeded` if the metadata `name` exceeds the maximum length.
        /// * `AssetMetadataTypeDefMaxLengthExceeded` if the metadata `spec` type definition exceeds the maximum length.
        #[weight = <T as Config>::WeightInfo::register_asset_metadata_global_type()]
        pub fn register_asset_metadata_global_type(origin, name: AssetMetadataName, spec: AssetMetadataSpec) -> DispatchResult {
            Self::base_register_asset_metadata_global_type(origin, name, spec)
        }

        /// Redeems existing tokens by reducing the balance of the caller's portfolio and the total supply of the token
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` Ticker of the token.
        /// * `value` Amount of tokens to redeem.
        /// * `portfolio` From whom portfolio tokens gets transferred.
        ///
        /// # Errors
        /// - `Unauthorized` If called by someone without the appropriate external agent permissions
        /// - `InvalidGranularity` If the amount is not divisible by 10^6 for non-divisible tokens
        /// - `InsufficientPortfolioBalance` If the caller's `portfolio` doesn't have enough free balance
        /// - `PortfolioDoesNotExist` If the portfolio doesn't exist.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::redeem_from_portfolio()]
        pub fn redeem_from_portfolio(origin, ticker: Ticker, value: Balance, portfolio: PortfolioKind) -> DispatchResult {
            let mut weight_meter = WeightMeter::max_limit_no_minimum();
            Self::base_redeem(origin, ticker, value, portfolio, &mut weight_meter)
        }

        /// Updates the type of an asset.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the ticker of the token.
        /// * `asset_type` - the new type of the token.
        ///
        /// ## Errors
        /// - `InvalidCustomAssetTypeId` if `asset_type` is of type custom and has an invalid type id.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::update_asset_type()]
        pub fn update_asset_type(origin, ticker: Ticker, asset_type: AssetType) -> DispatchResult {
            Self::base_update_asset_type(origin, ticker, asset_type)
        }

        /// Removes the asset metadata key and value of a local key.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the ticker of the local metadata key.
        /// * `local_key` - the local metadata key.
        ///
        /// # Errors
        ///  - `SecondaryKeyNotAuthorizedForAsset` - if called by someone without the appropriate external agent permissions.
        ///  - `UnauthorizedAgent` - if called by someone without the appropriate external agent permissions.
        ///  - `AssetMetadataKeyIsMissing` - if the key doens't exist.
        ///  - `AssetMetadataValueIsLocked` - if the value of the key is locked.
        ///  - AssetMetadataKeyBelongsToNFTCollection - if the key is a mandatory key in an NFT collection.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_local_metadata_key()]
        pub fn remove_local_metadata_key(origin, ticker: Ticker, local_key: AssetMetadataLocalKey) -> DispatchResult {
            Self::base_remove_local_metadata_key(origin, ticker, local_key)
        }

        /// Removes the asset metadata value of a metadata key.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the ticker of the local metadata key.
        /// * `metadata_key` - the metadata key that will have its value deleted.
        ///
        /// # Errors
        ///  - `SecondaryKeyNotAuthorizedForAsset` - if called by someone without the appropriate external agent permissions.
        ///  - `UnauthorizedAgent` - if called by someone without the appropriate external agent permissions.
        ///  - `AssetMetadataKeyIsMissing` - if the key doens't exist.
        ///  - `AssetMetadataValueIsLocked` - if the value of the key is locked.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_metadata_value()]
        pub fn remove_metadata_value(origin, ticker: Ticker, metadata_key: AssetMetadataKey) -> DispatchResult {
            Self::base_remove_metadata_value(origin, ticker, metadata_key)
        }

        /// Pre-approves the receivement of the asset for all identities.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the [`Ticker`] that will be exempt from affirmation.
        ///
        /// # Permissions
        /// * Root
        #[weight = <T as Config>::WeightInfo::exempt_ticker_affirmation()]
        pub fn exempt_ticker_affirmation(origin, ticker: Ticker) -> DispatchResult {
            Self::base_exempt_ticker_affirmation(origin, ticker)
        }

        /// Removes the pre-approval of the asset for all identities.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the [`Ticker`] that will have its exemption removed.
        ///
        /// # Permissions
        /// * Root
        #[weight = <T as Config>::WeightInfo::remove_ticker_affirmation_exemption()]
        pub fn remove_ticker_affirmation_exemption(origin, ticker: Ticker) -> DispatchResult {
            Self::base_remove_ticker_affirmation_exemption(origin, ticker)
        }

        /// Pre-approves the receivement of an asset.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the [`Ticker`] that will be exempt from affirmation.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::pre_approve_ticker()]
        pub fn pre_approve_ticker(origin, ticker: Ticker) -> DispatchResult {
            Self::base_pre_approve_ticker(origin, ticker)
        }

        /// Removes the pre approval of an asset.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the [`Ticker`] that will have its exemption removed.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_ticker_pre_approval()]
        pub fn remove_ticker_pre_approval(origin, ticker: Ticker) -> DispatchResult {
            Self::base_remove_ticker_pre_approval(origin, ticker)
        }

        /// Sets all identities in the `mediators` set as mandatory mediators for any instruction transfering `ticker`.
        ///
        /// # Arguments
        /// * `origin`: The secondary key of the sender.
        /// * `ticker`: The [`Ticker`] of the asset that will require the mediators.
        /// * `mediators`: A set of [`IdentityId`] of all the mandatory mediators for the given ticker.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_mandatory_mediators(mediators.len() as u32)]
        pub fn add_mandatory_mediators(
            origin,
            ticker: Ticker,
            mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>
        ) {
            Self::base_add_mandatory_mediators(origin, ticker, mediators)?;
        }

        /// Removes all identities in the `mediators` set from the mandatory mediators list for the given `ticker`.
        ///
        /// # Arguments
        /// * `origin`: The secondary key of the sender.
        /// * `ticker`: The [`Ticker`] of the asset that will have mediators removed.
        /// * `mediators`: A set of [`IdentityId`] of all the mediators that will be removed from the mandatory mediators list.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_mandatory_mediators(mediators.len() as u32)]
        pub fn remove_mandatory_mediators(
            origin,
            ticker: Ticker,
            mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>
        ) {
            Self::base_remove_mandatory_mediators(origin, ticker, mediators)?;
        }
    }
}
