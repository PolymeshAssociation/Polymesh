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
use pallet_portfolio::{Error as PortfolioError, PortfolioAssetBalances};
use polymesh_common_utilities::asset::AssetFnTrait;
use polymesh_common_utilities::compliance_manager::ComplianceFnConfig;
use polymesh_common_utilities::constants::*;
use polymesh_common_utilities::protocol_fee::{ChargeProtocolFee, ProtocolOp};
pub use polymesh_common_utilities::traits::asset::{Config, Event, RawEvent, WeightInfo};
use polymesh_common_utilities::traits::nft::NFTTrait;
use polymesh_common_utilities::with_transaction;
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::asset::{
    AssetName, AssetType, CheckpointId, CustomAssetTypeId, FundingRoundName,
    GranularCanTransferResult,
};
use polymesh_primitives::asset_metadata::{
    AssetMetadataGlobalKey, AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName,
    AssetMetadataSpec, AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::settlement::InstructionId;
use polymesh_primitives::transfer_compliance::TransferConditionResult;
use polymesh_primitives::{
    extract_auth, storage_migrate_on, storage_migration_ver, AssetIdentifier, Balance, Document,
    DocumentId, IdentityId, Memo, PortfolioId, PortfolioKind, PortfolioUpdateReason, SecondaryKey,
    Ticker, WeightMeter,
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

storage_migration_ver!(4);

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
        pub FundingRound get(fn funding_round): map hasher(blake2_128_concat) Ticker => FundingRoundName;
        /// The total balances of tokens issued in all recorded funding rounds.
        /// (ticker, funding round) -> balance
        pub IssuedInFundingRound get(fn issued_in_funding_round): map hasher(blake2_128_concat) (Ticker, FundingRoundName) => Balance;
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
        #[deprecated]
        pub AssetMetadataNextLocalKey get(fn asset_metadata_next_local_key):
            map hasher(blake2_128_concat) Ticker => AssetMetadataLocalKey;

        /// Next Asset Metadata Global Key.
        #[deprecated]
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

        /// Asset id nonce.
        AssetNonce get(fn asset_nonce) build(|_| 1u64): u64;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(4)): Version;

        /// The last [`AssetMetadataLocalKey`] used for [`Ticker`].
        pub CurrentAssetMetadataLocalKey get(fn current_asset_metadata_local_key):
            map hasher(blake2_128_concat) Ticker => Option<AssetMetadataLocalKey>;

        /// The last [`AssetMetadataGlobalKey`] used for a global key.
        pub CurrentAssetMetadataGlobalKey get(fn current_asset_metadata_global_key): Option<AssetMetadataGlobalKey>;
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

        /// initialize the default event for this module
        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            storage_migrate_on!(StorageVersion, 4, {
                migration::migrate_to_v4::<T>();
            });
            Weight::zero()
        }

        const AssetNameMaxLength: u32 = T::AssetNameMaxLength::get();
        const FundingRoundNameMaxLength: u32 = T::FundingRoundNameMaxLength::get();
        const AssetMetadataNameMaxLength: u32 = T::AssetMetadataNameMaxLength::get();
        const AssetMetadataValueMaxLength: u32 = T::AssetMetadataValueMaxLength::get();
        const AssetMetadataTypeDefMaxLength: u32 = T::AssetMetadataTypeDefMaxLength::get();
        const MaxAssetMediators: u32 = T::MaxAssetMediators::get();

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
            Self::base_create_asset(origin, name, Some(ticker), divisible, asset_type, identifiers, funding_round)?;
            Ok(())
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
        /// * `asset_identifiers` - the asset identifiers to be updated in the form of a vector of pairs of `IdentifierType` and `AssetIdentifier` value.
        ///
        /// ## Errors
        /// - `InvalidAssetIdentifier` if `identifiers` contains any invalid identifier.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::update_identifiers(asset_identifiers.len() as u32)]
        pub fn update_identifiers(
            origin,
            ticker: Ticker,
            asset_identifiers: Vec<AssetIdentifier>
        ) -> DispatchResult {
            Self::base_update_identifiers(origin, ticker, asset_identifiers)
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
            Self::base_register_custom_asset_type(origin, ty)?;
            Ok(())
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
            Self::base_create_asset_with_custom_type(
                origin,
                Some(ticker),
                name,
                divisible,
                custom_asset_type,
                identifiers,
                funding_round
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
            // Only allow global metadata types to be registered by root.
            ensure_root(origin)?;

            Self::base_register_asset_metadata_global_type(name, spec)
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

        /// Initializes a new security token, with the initiating account as its owner.
        /// The total supply will initially be zero. To mint tokens, use `issue`.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `name` - the name of the token.
        /// * `ticker` - the ticker symbol of the token.  (Optional.  A random ticker will be generated if no `ticker` is provided)
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
        pub fn create_asset_v2(
            origin,
            name: AssetName,
            ticker: Option<Ticker>,
            divisible: bool,
            asset_type: AssetType,
            identifiers: Vec<AssetIdentifier>,
            funding_round: Option<FundingRoundName>,
        ) -> DispatchResult {
            Self::base_create_asset(origin, name, ticker, divisible, asset_type, identifiers, funding_round)?;
            Ok(())
        }
    }
}

//==========================================================================
// All base functions!
//==========================================================================

impl<T: Config> Module<T> {
    /// Registers `ticker` to the caller.
    fn base_register_ticker(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;

        let ticker_registration_config = Self::ticker_registration_config();
        let ticker_registration_status = Self::validate_ticker_registration_rules(
            ticker,
            true,
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

            Self::ensure_asset_doesnt_exist(ticker)?;

            let reg =
                Self::ticker_registration(&ticker).ok_or(Error::<T>::TickerRegistrationExpired)?;
            <Identity<T>>::ensure_auth_by(auth_by, reg.owner)?;

            Self::transfer_ticker(reg, ticker, to);
            Ok(())
        })
    }

    /// Accept and process a token ownership transfer.
    fn base_accept_token_ownership_transfer(origin: T::RuntimeOrigin, id: u64) -> DispatchResult {
        let to = Identity::<T>::ensure_perms(origin)?;
        <Identity<T>>::accept_auth_with(&to.into(), id, |data, auth_by| {
            let ticker = extract_auth!(data, TransferAssetOwnership(t));

            // Get the token details and ensure it exists.
            let mut token = Self::token_details(&ticker)?;

            // Ensure the authorization was created by a permissioned agent.
            <ExternalAgents<T>>::ensure_agent_permissioned(ticker, auth_by)?;

            // Get the ticker registration and ensure it exists.
            let mut reg = Self::ticker_registration(&ticker).ok_or(Error::<T>::NoSuchAsset)?;
            let old_owner = reg.owner;
            AssetOwnershipRelations::remove(old_owner, ticker);
            AssetOwnershipRelations::insert(to, ticker, AssetOwnershipRelation::AssetOwned);
            // Update ticker registration.
            reg.owner = to;
            <Tickers<T>>::insert(&ticker, reg);
            // Update token details.
            token.owner_did = to;
            Tokens::insert(&ticker, token);
            Self::deposit_event(RawEvent::AssetOwnershipTransferred(to, ticker, old_owner));
            Ok(())
        })
    }

    /// Creates a new asset.
    fn base_create_asset(
        origin: T::RuntimeOrigin,
        asset_name: AssetName,
        ticker: Option<Ticker>,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round_name: Option<FundingRoundName>,
    ) -> Result<IdentityId, DispatchError> {
        let caller_data = Identity::<T>::ensure_origin_call_permissions(origin)?;

        Self::validate_and_create_asset(
            caller_data.primary_did,
            caller_data.secondary_key,
            asset_name,
            ticker,
            divisible,
            asset_type,
            identifiers,
            funding_round_name,
        )?;

        Ok(caller_data.primary_did)
    }

    fn base_set_freeze(origin: T::RuntimeOrigin, ticker: Ticker, freeze: bool) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        Self::ensure_asset_exists(&ticker)?;

        let (event, error) = match freeze {
            true => (
                RawEvent::AssetFrozen(did, ticker),
                Error::<T>::AlreadyFrozen,
            ),
            false => (RawEvent::AssetUnfrozen(did, ticker), Error::<T>::NotFrozen),
        };

        ensure!(Self::frozen(&ticker) != freeze, error);
        Frozen::insert(&ticker, freeze);

        Self::deposit_event(event);
        Ok(())
    }

    fn base_rename_asset(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        asset_name: AssetName,
    ) -> DispatchResult {
        Self::ensure_valid_asset_name(&asset_name)?;
        Self::ensure_asset_exists(&ticker)?;
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        AssetNames::insert(&ticker, asset_name.clone());
        Self::deposit_event(RawEvent::AssetRenamed(caller_did, ticker, asset_name));
        Ok(())
    }

    /// Issues `amount_to_issue` tokens for `ticker` into the caller's portfolio.
    fn base_issue(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        amount_to_issue: Balance,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        let caller_portfolio = Self::ensure_origin_ticker_and_portfolio_permissions(
            origin,
            ticker,
            portfolio_kind,
            false,
        )?;
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let mut security_token = Self::token_details(&ticker)?;
        Self::validate_issuance_rules(&security_token, amount_to_issue)?;
        Self::unverified_issue_tokens(
            ticker,
            &mut security_token,
            caller_portfolio,
            amount_to_issue,
            true,
            &mut weight_meter,
        )?;
        Ok(())
    }

    fn base_redeem(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        value: Balance,
        portfolio_kind: PortfolioKind,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let portfolio = Self::ensure_origin_ticker_and_portfolio_permissions(
            origin,
            ticker,
            portfolio_kind,
            true,
        )?;

        let mut token = Self::token_details(&ticker)?;
        Self::ensure_token_granular(&token, &value)?;

        // Ensures the token is fungible
        ensure!(
            token.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        // Reduce caller's portfolio balance. This makes sure that the caller has enough unlocked tokens.
        // If `advance_update_balances` fails, `reduce_portfolio_balance` shouldn't modify storage.

        with_transaction(|| {
            Portfolio::<T>::reduce_portfolio_balance(&portfolio, &ticker, value)?;

            // Try updating the total supply.
            token.total_supply = token
                .total_supply
                .checked_sub(value)
                .ok_or(Error::<T>::TotalSupplyOverflow)?;

            <Checkpoint<T>>::advance_update_balances(
                &ticker,
                &[(portfolio.did, Self::balance_of(ticker, portfolio.did))],
            )
        })?;

        let updated_balance = Self::balance_of(ticker, portfolio.did) - value;

        // Update identity balances and total supply
        BalanceOf::insert(ticker, &portfolio.did, updated_balance);
        Tokens::insert(ticker, token);

        // Update statistic info.
        Statistics::<T>::update_asset_stats(
            &ticker,
            Some(&portfolio.did),
            None,
            Some(updated_balance),
            None,
            value,
            weight_meter,
        )?;

        Self::deposit_event(RawEvent::AssetBalanceUpdated(
            portfolio.did,
            ticker,
            value,
            Some(portfolio),
            None,
            PortfolioUpdateReason::Redeemed,
        ));
        Ok(())
    }

    fn base_make_divisible(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        Tokens::try_mutate(&ticker, |token| -> DispatchResult {
            let token = token.as_mut().ok_or(Error::<T>::NoSuchAsset)?;
            // Ensures the token is fungible
            ensure!(
                token.asset_type.is_fungible(),
                Error::<T>::UnexpectedNonFungibleToken
            );
            ensure!(!token.divisible, Error::<T>::AssetAlreadyDivisible);
            token.divisible = true;

            Self::deposit_event(RawEvent::DivisibilityChanged(did, ticker, true));
            Ok(())
        })
    }

    fn base_add_documents(
        origin: T::RuntimeOrigin,
        docs: Vec<Document>,
        ticker: Ticker,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        // Ensure strings are limited.
        for doc in &docs {
            ensure_string_limited::<T>(&doc.uri)?;
            ensure_string_limited::<T>(&doc.name)?;
            ensure_opt_string_limited::<T>(doc.doc_type.as_deref())?;
        }

        // Ensure we can advance documents ID sequence by `len`.
        let pre = AssetDocumentsIdSequence::try_mutate(ticker, |id| {
            id.0.checked_add(docs.len() as u32)
                .ok_or(CounterOverflow::<T>)
                .map(|new| mem::replace(id, DocumentId(new)))
        })?;

        // Charge fee.
        T::ProtocolFee::batch_charge_fee(ProtocolOp::AssetAddDocuments, docs.len())?;

        // Add the documents & emit events.
        for (id, doc) in (pre.0..).map(DocumentId).zip(docs) {
            AssetDocuments::insert(ticker, id, doc.clone());
            Self::deposit_event(RawEvent::DocumentAdded(did, ticker, id, doc));
        }
        Ok(())
    }

    fn base_remove_documents(
        origin: T::RuntimeOrigin,
        ids: Vec<DocumentId>,
        ticker: Ticker,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        for id in ids {
            AssetDocuments::remove(ticker, id);
            Self::deposit_event(RawEvent::DocumentRemoved(did, ticker, id));
        }
        Ok(())
    }

    fn base_set_funding_round(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        funding_round_name: FundingRoundName,
    ) -> DispatchResult {
        Self::ensure_valid_funding_round_name(&funding_round_name)?;
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        FundingRound::insert(ticker, funding_round_name.clone());
        Self::deposit_event(RawEvent::FundingRoundSet(
            caller_did,
            ticker,
            funding_round_name,
        ));
        Ok(())
    }

    fn base_update_identifiers(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        asset_identifiers: Vec<AssetIdentifier>,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        Self::ensure_valid_asset_identifiers(&asset_identifiers)?;
        Self::unverified_update_idents(did, ticker, asset_identifiers);
        Ok(())
    }

    fn base_controller_transfer(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        transfer_value: Balance,
        sender_portfolio: PortfolioId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let caller_portfolio = Self::ensure_origin_ticker_and_portfolio_permissions(
            origin,
            ticker,
            PortfolioKind::Default,
            false,
        )?;

        Self::validate_asset_transfer(
            &ticker,
            &sender_portfolio,
            &caller_portfolio,
            transfer_value,
            true,
            weight_meter,
        )?;

        Self::unverified_transfer_asset(
            sender_portfolio,
            caller_portfolio,
            ticker,
            transfer_value,
            None,
            None,
            caller_portfolio.did,
            weight_meter,
        )?;

        Self::deposit_event(RawEvent::ControllerTransfer(
            caller_portfolio.did,
            ticker,
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
        ticker: Option<Ticker>,
        asset_name: AssetName,
        divisible: bool,
        asset_type_bytes: Vec<u8>,
        identifiers: Vec<AssetIdentifier>,
        funding_round_name: Option<FundingRoundName>,
    ) -> DispatchResult {
        let caller_data = Identity::<T>::ensure_origin_call_permissions(origin)?;

        Self::validate_custom_asset_type_rules(&asset_type_bytes)?;
        let custom_asset_type_id =
            Self::unverified_register_custom_asset_type(caller_data.primary_did, asset_type_bytes)?;
        let asset_type = AssetType::Custom(custom_asset_type_id);

        Self::validate_and_create_asset(
            caller_data.primary_did,
            caller_data.secondary_key,
            asset_name,
            ticker,
            divisible,
            asset_type,
            identifiers,
            funding_round_name,
        )?;

        Ok(())
    }

    fn base_set_asset_metadata(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        key: AssetMetadataKey,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        Self::unverified_set_asset_metadata(did, ticker, key, value, detail)
    }

    fn base_set_asset_metadata_details(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        key: AssetMetadataKey,
        detail: AssetMetadataValueDetail<T::Moment>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        // Check key exists.
        ensure!(
            Self::check_asset_metadata_key_exists(&ticker, &key),
            Error::<T>::AssetMetadataKeyIsMissing
        );

        // Check if value is currently locked.
        ensure!(
            !Self::is_asset_metadata_locked(ticker, key),
            Error::<T>::AssetMetadataValueIsLocked
        );

        // Prevent locking an asset metadata with no value
        if detail.is_locked(<pallet_timestamp::Pallet<T>>::get()) {
            AssetMetadataValues::try_get(&ticker, &key)
                .map_err(|_| Error::<T>::AssetMetadataValueIsEmpty)?;
        }

        // Set asset metadata value details.
        AssetMetadataValueDetails::<T>::insert(ticker, key, &detail);

        Self::deposit_event(RawEvent::SetAssetMetadataValueDetails(did, ticker, detail));
        Ok(())
    }

    fn base_register_and_set_local_asset_metadata(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        // Register local metadata type.
        let key = Self::unverified_register_asset_metadata_local_type(did, ticker, name, spec)?;

        Self::unverified_set_asset_metadata(did, ticker, key, value, detail)
    }

    fn base_register_asset_metadata_local_type(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        Self::unverified_register_asset_metadata_local_type(did, ticker, name, spec).map(drop)
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
        AssetMetadataNextGlobalKey::set(key);

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
        ticker: Ticker,
        asset_type: AssetType,
    ) -> DispatchResult {
        Self::ensure_asset_exists(&ticker)?;
        Self::ensure_valid_asset_type(&asset_type)?;
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        Tokens::try_mutate(&ticker, |token| -> DispatchResult {
            let token = token.as_mut().ok_or(Error::<T>::NoSuchAsset)?;
            // Ensures that both parameters are non fungible types or if both are fungible types.
            ensure!(
                token.asset_type.is_fungible() == asset_type.is_fungible(),
                Error::<T>::IncompatibleAssetTypeUpdate
            );
            token.asset_type = asset_type;
            Ok(())
        })?;
        Self::deposit_event(RawEvent::AssetTypeChanged(did, ticker, asset_type));
        Ok(())
    }

    fn base_remove_local_metadata_key(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        local_key: AssetMetadataLocalKey,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        // Verifies if the key exists.
        let name = AssetMetadataLocalKeyToName::try_get(ticker, &local_key)
            .map_err(|_| Error::<T>::AssetMetadataKeyIsMissing)?;
        // Verifies if the value is locked
        let metadata_key = AssetMetadataKey::Local(local_key);
        if let Some(value_detail) = AssetMetadataValueDetails::<T>::get(&ticker, &metadata_key) {
            ensure!(
                !value_detail.is_locked(<pallet_timestamp::Pallet<T>>::get()),
                Error::<T>::AssetMetadataValueIsLocked
            );
        }
        // Verifies if the key belongs to an NFT collection
        ensure!(
            !T::NFTFn::is_collection_key(&ticker, &metadata_key),
            Error::<T>::AssetMetadataKeyBelongsToNFTCollection
        );
        // Remove key from storage
        AssetMetadataValues::remove(&ticker, &metadata_key);
        AssetMetadataValueDetails::<T>::remove(&ticker, &metadata_key);
        AssetMetadataLocalNameToKey::remove(&ticker, &name);
        AssetMetadataLocalKeyToName::remove(&ticker, &local_key);
        AssetMetadataLocalSpecs::remove(&ticker, &local_key);
        Self::deposit_event(RawEvent::LocalMetadataKeyDeleted(
            caller_did, ticker, local_key,
        ));
        Ok(())
    }

    fn base_remove_metadata_value(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        metadata_key: AssetMetadataKey,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        // Verifies if the key exists.
        match metadata_key {
            AssetMetadataKey::Global(global_key) => {
                if !AssetMetadataGlobalKeyToName::contains_key(&global_key) {
                    return Err(Error::<T>::AssetMetadataKeyIsMissing.into());
                }
            }
            AssetMetadataKey::Local(local_key) => {
                if !AssetMetadataLocalKeyToName::contains_key(ticker, &local_key) {
                    return Err(Error::<T>::AssetMetadataKeyIsMissing.into());
                }
            }
        }
        // Verifies if the value is locked
        if let Some(value_detail) = AssetMetadataValueDetails::<T>::get(&ticker, &metadata_key) {
            ensure!(
                !value_detail.is_locked(<pallet_timestamp::Pallet<T>>::get()),
                Error::<T>::AssetMetadataValueIsLocked
            );
        }
        // Remove the metadata value from storage
        AssetMetadataValues::remove(&ticker, &metadata_key);
        AssetMetadataValueDetails::<T>::remove(&ticker, &metadata_key);
        Self::deposit_event(RawEvent::MetadataValueDeleted(
            caller_did,
            ticker,
            metadata_key,
        ));
        Ok(())
    }

    /// Pre-approves the receivement of the asset for all identities.
    fn base_exempt_ticker_affirmation(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        ensure_root(origin)?;
        TickersExemptFromAffirmation::insert(&ticker, true);
        Self::deposit_event(RawEvent::AssetAffirmationExemption(ticker));
        Ok(())
    }

    /// Removes the pre-approval of the asset for all identities.
    fn base_remove_ticker_affirmation_exemption(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
    ) -> DispatchResult {
        ensure_root(origin)?;
        TickersExemptFromAffirmation::remove(&ticker);
        Self::deposit_event(RawEvent::RemoveAssetAffirmationExemption(ticker));
        Ok(())
    }

    /// Pre-approves the receivement of an asset.
    fn base_pre_approve_ticker(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;
        PreApprovedTicker::insert(&caller_did, &ticker, true);
        Self::deposit_event(RawEvent::PreApprovedAsset(caller_did, ticker));
        Ok(())
    }

    /// Removes the pre approval of an asset.
    fn base_remove_ticker_pre_approval(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;
        PreApprovedTicker::remove(&caller_did, &ticker);
        Self::deposit_event(RawEvent::RemovePreApprovedAsset(caller_did, ticker));
        Ok(())
    }

    /// Sets all identities in the `mediators` set as mandatory mediators for any instruction transfering `ticker`.
    fn base_add_mandatory_mediators(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        new_mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        // Tries to add all new identities as mandatory mediators for the asset
        MandatoryMediators::<T>::try_mutate(ticker, |mandatory_mediators| -> DispatchResult {
            for new_mediator in &new_mediators {
                mandatory_mediators
                    .try_insert(*new_mediator)
                    .map_err(|_| Error::<T>::NumberOfAssetMediatorsExceeded)?;
            }
            Ok(())
        })?;

        Self::deposit_event(RawEvent::AssetMediatorsAdded(
            caller_did,
            ticker,
            new_mediators.into_inner(),
        ));
        Ok(())
    }

    /// Removes all identities in the `mediators` set from the mandatory mediators list for the given `ticker`.
    fn base_remove_mandatory_mediators(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        // Removes the identities from the mandatory mediators list
        MandatoryMediators::<T>::mutate(ticker, |mandatory_mediators| {
            for mediator in &mediators {
                mandatory_mediators.remove(mediator);
            }
        });
        Self::deposit_event(RawEvent::AssetMediatorsRemoved(
            caller_did,
            ticker,
            mediators.into_inner(),
        ));
        Ok(())
    }

    /// Transfers an asset from one identity portfolio to another
    pub fn base_transfer(
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
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
            &ticker,
            &from_portfolio,
            &to_portfolio,
            transfer_value,
            false,
            weight_meter,
        )?;

        Self::unverified_transfer_asset(
            from_portfolio,
            to_portfolio,
            *ticker,
            transfer_value,
            instruction_id,
            instruction_memo,
            caller_did,
            weight_meter,
        )?;

        Ok(())
    }
}

//==========================================================================
// All validattion functions!
//==========================================================================

impl<T: Config> Module<T> {
    /// Returns [`TickerRegistrationStatus`] if all registration rules are satisfied.
    fn validate_ticker_registration_rules(
        ticker: Ticker,
        is_named: bool,
        ticker_owner_did: &IdentityId,
        max_ticker_length: u8,
    ) -> Result<TickerRegistrationStatus, DispatchError> {
        // Only validate ticker characters for named tickers.
        if is_named {
            Self::verify_ticker_characters(&ticker)?;
            Self::ensure_ticker_length(ticker, max_ticker_length)?;
        }
        Self::ensure_asset_doesnt_exist(ticker)?;

        let ticker_registration_status =
            Self::can_reregister_ticker(ticker, is_named, ticker_owner_did);

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

    /// Returns `Ok` if `ticker` doensn't exist. Otherwise, returns [`Error::AssetAlreadyCreated`].
    fn ensure_asset_doesnt_exist(ticker: Ticker) -> DispatchResult {
        ensure!(
            !Tokens::contains_key(ticker),
            Error::<T>::AssetAlreadyCreated
        );
        Ok(())
    }

    /// Returns `Ok` if `ticker` length is less or equal to `max_ticker_length`. Otherwise, returns [`Error::TickerTooLong`].
    fn ensure_ticker_length(ticker: Ticker, max_ticker_length: u8) -> DispatchResult {
        ensure!(
            ticker.len() <= max_ticker_length as usize,
            Error::<T>::TickerTooLong
        );
        Ok(())
    }

    /// Returns [`TickerRegistrationStatus`] containing information regarding whether the ticker can be registered and if the fee must be charged.
    fn can_reregister_ticker(
        ticker: Ticker,
        is_named: bool,
        caller_did: &IdentityId,
    ) -> TickerRegistrationStatus {
        match <Tickers<T>>::get(ticker) {
            Some(ticker_registration) => {
                // Checks if the ticker has an expiration time
                match ticker_registration.expiry {
                    Some(expiration_time) => {
                        // Checks if the registration has expired
                        if <pallet_timestamp::Pallet<T>>::get() > expiration_time {
                            return TickerRegistrationStatus::new(true, is_named);
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
            None => TickerRegistrationStatus::new(true, is_named),
        }
    }

    /// Returns [`TickerRegistrationStatus`] if all rules for creating an asset are satisfied.
    fn validate_asset_creation_rules(
        caller_did: IdentityId,
        secondary_key: Option<SecondaryKey<T::AccountId>>,
        token_did: IdentityId,
        ticker: Ticker,
        is_named: bool,
        asset_name: &AssetName,
        asset_type: &AssetType,
        funding_round_name: Option<FundingRoundName>,
        asset_identifiers: &[AssetIdentifier],
    ) -> Result<TickerRegistrationStatus, DispatchError> {
        if let Some(funding_round_name) = funding_round_name {
            Self::ensure_valid_funding_round_name(&funding_round_name)?;
        }
        Self::ensure_valid_asset_name(asset_name)?;
        Self::ensure_valid_asset_type(asset_type)?;
        Self::ensure_valid_asset_identifiers(asset_identifiers)?;

        let ticker_registration_config = Self::ticker_registration_config();
        let ticker_registration_status = Self::validate_ticker_registration_rules(
            ticker,
            is_named,
            &caller_did,
            ticker_registration_config.max_ticker_length,
        )?;

        // Ensure there's no pre-existing entry for the DID.
        Identity::<T>::ensure_no_id_record(token_did)?;

        // Ensure that the caller has relevant portfolio permissions
        Portfolio::<T>::ensure_portfolio_custody_and_permission(
            PortfolioId::default_portfolio(caller_did),
            caller_did,
            secondary_key.as_ref(),
        )?;
        Ok(ticker_registration_status)
    }

    pub fn token_details(ticker: &Ticker) -> Result<SecurityToken, DispatchError> {
        Ok(Tokens::try_get(ticker).or(Err(Error::<T>::NoSuchAsset))?)
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
                CustomTypes::contains_key(custom_type_id),
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

    /// Ensures that `origin` is a permissioned agent for `ticker`, that the portfolio is valid and that calller
    /// has the access to the portfolio. If `ensure_custody` is `true`, also enforces the caller to have custody
    /// of the portfolio.
    pub fn ensure_origin_ticker_and_portfolio_permissions(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        portfolio_kind: PortfolioKind,
        ensure_custody: bool,
    ) -> Result<PortfolioId, DispatchError> {
        let origin_data = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, ticker)?;
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

    /// Returns `Ok` if [`SecurityToken::divisible`] or `value` % ONE_UNIT == 0. Otherwise, returns [`Error::<T>::InvalidGranularity`].
    fn ensure_token_granular(security_token: &SecurityToken, value: &Balance) -> DispatchResult {
        if security_token.divisible || value % ONE_UNIT == 0 {
            return Ok(());
        }
        Err(Error::<T>::InvalidGranularity.into())
    }

    /// Returns `true` if [`SecurityToken::divisible`], otherwise returns `false`.
    pub fn is_divisible(ticker: &Ticker) -> bool {
        Self::token_details(ticker)
            .map(|t| t.divisible)
            .unwrap_or_default()
    }

    pub fn check_asset_metadata_key_exists(ticker: &Ticker, key: &AssetMetadataKey) -> bool {
        match key {
            AssetMetadataKey::Global(key) => AssetMetadataGlobalKeyToName::contains_key(key),
            AssetMetadataKey::Local(key) => AssetMetadataLocalKeyToName::contains_key(ticker, key),
        }
    }

    fn is_asset_metadata_locked(ticker: Ticker, key: AssetMetadataKey) -> bool {
        AssetMetadataValueDetails::<T>::get(ticker, key).map_or(false, |details| {
            details.is_locked(<pallet_timestamp::Pallet<T>>::get())
        })
    }

    /// Ensure that `ticker` is a valid created asset.
    fn ensure_asset_exists(ticker: &Ticker) -> DispatchResult {
        ensure!(Tokens::contains_key(&ticker), Error::<T>::NoSuchAsset);
        Ok(())
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

    /// Returns `None` if there's no asset associated to the given ticker,
    /// returns Some(true) if the asset exists and is of type `AssetType::NonFungible`, and returns Some(false) otherwise.
    pub fn nft_asset(ticker: &Ticker) -> Option<bool> {
        let token = Tokens::try_get(ticker).ok()?;
        Some(token.asset_type.is_non_fungible())
    }

    /// Ensure that the document `doc` exists for `ticker`.
    pub fn ensure_doc_exists(ticker: &Ticker, doc: &DocumentId) -> DispatchResult {
        ensure!(
            AssetDocuments::contains_key(ticker, doc),
            Error::<T>::NoSuchDoc
        );
        Ok(())
    }

    pub fn get_balance_at(ticker: Ticker, did: IdentityId, at: CheckpointId) -> Balance {
        <Checkpoint<T>>::balance_at(ticker, did, at)
            .unwrap_or_else(|| Self::balance_of(&ticker, &did))
    }

    pub fn validate_asset_transfer(
        ticker: &Ticker,
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        transfer_value: Balance,
        is_controller_transfer: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let security_token = Self::token_details(&ticker)?;
        ensure!(
            security_token.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        ensure!(
            BalanceOf::get(ticker, &sender_portfolio.did) >= transfer_value,
            Error::<T>::InsufficientBalance
        );
        ensure!(
            BalanceOf::get(ticker, &receiver_portfolio.did)
                .checked_add(transfer_value)
                .is_some(),
            Error::<T>::BalanceOverflow
        );

        // Verifies that both portfolios exist an that the sender portfolio has sufficient balance
        Portfolio::<T>::ensure_portfolio_transfer_validity(
            sender_portfolio,
            receiver_portfolio,
            ticker,
            transfer_value,
        )?;

        // Controllers are exempt from statistics, compliance and frozen rules.
        if is_controller_transfer {
            return Ok(());
        }

        // Verifies that the asset is not frozen
        ensure!(!Frozen::get(ticker), Error::<T>::InvalidTransferFrozenAsset);

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
            ticker,
            &sender_portfolio.did,
            &receiver_portfolio.did,
            Self::balance_of(ticker, sender_portfolio.did),
            Self::balance_of(ticker, receiver_portfolio.did),
            transfer_value,
            security_token.total_supply,
            weight_meter,
        )?;

        // Verifies that all compliance rules are being respected
        if !T::ComplianceManager::is_compliant(
            ticker,
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
        ticker: &Ticker,
        transfer_value: Balance,
        skip_locked_check: bool,
        weight_meter: &mut WeightMeter,
    ) -> Vec<DispatchError> {
        let mut asset_transfer_errors = Vec::new();

        // If the security token doesn't exist or if the token is an NFT, there's no point in assessing anything else
        let security_token = {
            match Tokens::try_get(ticker) {
                Ok(security_token) => security_token,
                Err(_) => return vec![Error::<T>::NoSuchAsset.into()],
            }
        };
        if !security_token.asset_type.is_fungible() {
            return vec![Error::<T>::UnexpectedNonFungibleToken.into()];
        }

        if let Err(e) = Self::ensure_token_granular(&security_token, &transfer_value) {
            asset_transfer_errors.push(e);
        }

        let sender_current_balance = BalanceOf::get(ticker, &sender_portfolio.did);
        if sender_current_balance < transfer_value {
            asset_transfer_errors.push(Error::<T>::InsufficientBalance.into());
        }

        let receiver_current_balance = BalanceOf::get(ticker, &receiver_portfolio.did);
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
            if PortfolioAssetBalances::get(sender_portfolio, ticker) < transfer_value {
                asset_transfer_errors
                    .push(PortfolioError::<T>::InsufficientPortfolioBalance.into());
            }
        } else {
            if let Err(e) =
                Portfolio::<T>::ensure_sufficient_balance(sender_portfolio, ticker, transfer_value)
            {
                asset_transfer_errors.push(e);
            }
        }

        if !Identity::<T>::has_valid_cdd(receiver_portfolio.did) {
            asset_transfer_errors.push(Error::<T>::InvalidTransferInvalidReceiverCDD.into());
        }

        if !Identity::<T>::has_valid_cdd(sender_portfolio.did) {
            asset_transfer_errors.push(Error::<T>::InvalidTransferInvalidSenderCDD.into());
        }

        if Frozen::get(ticker) {
            asset_transfer_errors.push(Error::<T>::InvalidTransferFrozenAsset.into());
        }

        if let Err(e) = Statistics::<T>::verify_transfer_restrictions(
            ticker,
            &sender_portfolio.did,
            &receiver_portfolio.did,
            sender_current_balance,
            receiver_current_balance,
            transfer_value,
            security_token.total_supply,
            weight_meter,
        ) {
            asset_transfer_errors.push(e);
        }

        match T::ComplianceManager::is_compliant(
            ticker,
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

    // Get the total supply of an asset `id`.
    pub fn total_supply(ticker: &Ticker) -> Balance {
        Self::token_details(ticker)
            .map(|t| t.total_supply)
            .unwrap_or_default()
    }

    /// Calls [`Module::validate_asset_creation_rules`] and [`Module::unverified_create_asset`].
    fn validate_and_create_asset(
        caller_primary_identity: IdentityId,
        caller_secondary_key: Option<SecondaryKey<T::AccountId>>,
        asset_name: AssetName,
        ticker: Option<Ticker>,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round_name: Option<FundingRoundName>,
    ) -> DispatchResult {
        let (ticker, is_named) = match ticker {
            Some(ticker) => (ticker, true),
            None => (Self::make_asset_id()?, false),
        };
        let token_did = Identity::<T>::get_token_did(&ticker)?;
        let ticker_registration_status = Self::validate_asset_creation_rules(
            caller_primary_identity,
            caller_secondary_key,
            token_did,
            ticker,
            is_named,
            &asset_name,
            &asset_type,
            funding_round_name.clone(),
            &identifiers,
        )?;

        Self::unverified_create_asset(
            caller_primary_identity,
            token_did,
            ticker,
            divisible,
            asset_name,
            asset_type,
            funding_round_name,
            identifiers,
            ticker_registration_status.charge_fee(),
        )?;

        Ok(())
    }

    /// Returns `Ok` if all rules for issuing a token are satisfied.
    fn validate_issuance_rules(
        security_token: &SecurityToken,
        amount_to_issue: Balance,
    ) -> DispatchResult {
        ensure!(
            security_token.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        Self::ensure_token_granular(security_token, &amount_to_issue)?;

        let new_supply = security_token
            .total_supply
            .checked_add(amount_to_issue)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;
        ensure!(new_supply <= MAX_SUPPLY, Error::<T>::TotalSupplyAboveLimit);
        Ok(())
    }

    /// Generate a random asset id.
    fn make_asset_id() -> Result<Ticker, DispatchError> {
        let nonce = Self::asset_nonce() + 1u64;
        AssetNonce::put(&nonce);

        let parent_hash = frame_system::Pallet::<T>::parent_hash();
        let mut hash = blake2_128(&(b"AssetId", parent_hash, nonce).encode());
        // Set the highest bit of the first byte to mark it as an "unnamed" ticker.
        // This also forces the first byte to be non-ASCII.
        hash[0] |= 0x80;

        Ok(Ticker::from_slice_truncated(&hash))
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
        if let Some(ticker_registration) = <Tickers<T>>::get(ticker) {
            AssetOwnershipRelations::remove(&ticker_registration.owner, &ticker);
        }

        // Write the ticker registration data to the storage
        <Tickers<T>>::insert(ticker, TickerRegistration { owner, expiry });
        AssetOwnershipRelations::insert(owner, ticker, AssetOwnershipRelation::TickerOwned);

        Self::deposit_event(RawEvent::TickerRegistered(owner, ticker, expiry));
        Ok(())
    }

    /// Transfer the given `ticker`'s registration from `req.owner` to `to`.
    fn transfer_ticker(mut reg: TickerRegistration<T::Moment>, ticker: Ticker, to: IdentityId) {
        let from = reg.owner;
        AssetOwnershipRelations::remove(from, ticker);
        AssetOwnershipRelations::insert(to, ticker, AssetOwnershipRelation::TickerOwned);
        reg.owner = to;
        <Tickers<T>>::insert(&ticker, reg);
        Self::deposit_event(RawEvent::TickerTransferred(to, ticker, from));
    }

    /// All storage writes for creating an asset.
    /// Note: two fees are charged ([`ProtocolOp::AssetCreateAsset`] and [`ProtocolOp::AssetRegisterTicker`]).
    fn unverified_create_asset(
        caller_did: IdentityId,
        token_did: IdentityId,
        ticker: Ticker,
        divisible: bool,
        asset_name: AssetName,
        asset_type: AssetType,
        funding_round_name: Option<FundingRoundName>,
        identifiers: Vec<AssetIdentifier>,
        ticker_registration_fee: bool,
    ) -> DispatchResult {
        T::ProtocolFee::charge_fee(ProtocolOp::AssetCreateAsset)?;
        Self::unverified_register_ticker(ticker, caller_did, None, ticker_registration_fee)?;

        Identity::<T>::commit_token_did(token_did, ticker);
        let token = SecurityToken::new(Zero::zero(), caller_did, divisible, asset_type);
        Tokens::insert(ticker, token);

        AssetNames::insert(ticker, &asset_name);
        if let Some(ref funding_round_name) = funding_round_name {
            FundingRound::insert(ticker, funding_round_name);
        }

        AssetOwnershipRelations::insert(caller_did, ticker, AssetOwnershipRelation::AssetOwned);
        Self::deposit_event(RawEvent::AssetCreated(
            caller_did,
            ticker,
            divisible,
            asset_type,
            caller_did,
            asset_name,
            identifiers.clone(),
            funding_round_name,
        ));

        //These emit events which should come after the main AssetCreated event
        Self::unverified_update_idents(caller_did, ticker, identifiers);
        // Grant owner full agent permissions.
        <ExternalAgents<T>>::unchecked_add_agent(ticker, caller_did, AgentGroup::Full)?;
        Ok(())
    }

    /// Update identitifiers of `ticker` as `did`.
    ///
    /// Does not verify that actor `did` is permissioned for this call or that `idents` are valid.
    fn unverified_update_idents(did: IdentityId, ticker: Ticker, idents: Vec<AssetIdentifier>) {
        Identifiers::insert(ticker, idents.clone());
        Self::deposit_event(RawEvent::IdentifiersUpdated(did, ticker, idents));
    }

    /// All storage writes for issuing a token.
    /// Note: if `charge_fee`` is `true` [`ProtocolOp::AssetIssue`] is charged.
    fn unverified_issue_tokens(
        ticker: Ticker,
        security_token: &mut SecurityToken,
        issuer_portfolio: PortfolioId,
        amount_to_issue: Balance,
        charge_fee: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        if charge_fee {
            T::ProtocolFee::charge_fee(ProtocolOp::AssetIssue)?;
        }

        let current_issuer_balance = BalanceOf::get(&ticker, &issuer_portfolio.did);
        <Checkpoint<T>>::advance_update_balances(
            &ticker,
            &[(issuer_portfolio.did, current_issuer_balance)],
        )?;

        let new_issuer_balance = current_issuer_balance + amount_to_issue;
        BalanceOf::insert(ticker, issuer_portfolio.did, new_issuer_balance);

        security_token.total_supply += amount_to_issue;
        Tokens::insert(ticker, security_token);

        // No check since the total balance is always <= the total supply
        let new_issuer_portfolio_balance =
            Portfolio::<T>::portfolio_asset_balances(issuer_portfolio, ticker) + amount_to_issue;
        Portfolio::<T>::set_portfolio_balance(
            issuer_portfolio,
            &ticker,
            new_issuer_portfolio_balance,
        );

        Statistics::<T>::update_asset_stats(
            &ticker,
            None,
            Some(&issuer_portfolio.did),
            None,
            Some(new_issuer_balance),
            amount_to_issue,
            weight_meter,
        )?;

        let funding_round_name = FundingRound::get(&ticker);
        IssuedInFundingRound::mutate((&ticker, &funding_round_name), |balance| {
            *balance = balance.saturating_add(amount_to_issue)
        });

        Self::deposit_event(RawEvent::AssetBalanceUpdated(
            issuer_portfolio.did,
            ticker,
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
        ticker: Ticker,
        transfer_value: Balance,
        instruction_id: Option<InstructionId>,
        instruction_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Gets the current balance and advances the checkpoint
        let sender_current_balance = BalanceOf::get(&ticker, &sender_portfolio.did);
        let receiver_current_balance = BalanceOf::get(&ticker, &receiver_portfolio.did);
        <Checkpoint<T>>::advance_update_balances(
            &ticker,
            &[
                (sender_portfolio.did, sender_current_balance),
                (receiver_portfolio.did, receiver_current_balance),
            ],
        )?;

        // Updates the balance in the asset pallet
        let sender_new_balance = sender_current_balance - transfer_value;
        let receiver_new_balance = receiver_current_balance + transfer_value;
        BalanceOf::insert(ticker, sender_portfolio.did, sender_new_balance);
        BalanceOf::insert(ticker, receiver_portfolio.did, receiver_new_balance);

        // Updates the balances in the portfolio pallet
        Portfolio::<T>::unchecked_transfer_portfolio_balance(
            &sender_portfolio,
            &receiver_portfolio,
            &ticker,
            transfer_value,
        );

        // Update statistics info.
        Statistics::<T>::update_asset_stats(
            &ticker,
            Some(&sender_portfolio.did),
            Some(&receiver_portfolio.did),
            Some(sender_new_balance),
            Some(receiver_new_balance),
            transfer_value,
            weight_meter,
        )?;

        Self::deposit_event(RawEvent::AssetBalanceUpdated(
            caller_did,
            ticker,
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
        ticker: Ticker,
        key: AssetMetadataKey,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Check value length limit.
        Self::ensure_asset_metadata_value_limited(&value)?;

        // Check key exists.
        ensure!(
            Self::check_asset_metadata_key_exists(&ticker, &key),
            Error::<T>::AssetMetadataKeyIsMissing
        );

        // Check if value is currently locked.
        ensure!(
            !Self::is_asset_metadata_locked(ticker, key),
            Error::<T>::AssetMetadataValueIsLocked
        );

        // Set asset metadata value for asset.
        AssetMetadataValues::insert(ticker, key, &value);

        // Set asset metadata value details.
        if let Some(ref detail) = detail {
            AssetMetadataValueDetails::<T>::insert(ticker, key, detail);
        }

        Self::deposit_event(RawEvent::SetAssetMetadataValue(did, ticker, value, detail));
        Ok(())
    }

    fn unverified_register_asset_metadata_local_type(
        did: IdentityId,
        ticker: Ticker,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> Result<AssetMetadataKey, DispatchError> {
        Self::ensure_asset_metadata_name_limited(&name)?;
        Self::ensure_asset_metadata_spec_limited(&spec)?;

        // Check if key already exists.
        ensure!(
            !AssetMetadataLocalNameToKey::contains_key(ticker, &name),
            Error::<T>::AssetMetadataLocalKeyAlreadyExists
        );

        // Next local key for asset.
        let key = Self::update_current_asset_metadata_local_key(&ticker)?;
        AssetMetadataNextLocalKey::insert(ticker, key);

        // Store local key <-> name mapping.
        AssetMetadataLocalNameToKey::insert(ticker, &name, key);
        AssetMetadataLocalKeyToName::insert(ticker, key, &name);

        // Store local specs.
        AssetMetadataLocalSpecs::insert(ticker, key, &spec);

        Self::deposit_event(RawEvent::RegisterAssetMetadataLocalType(
            did, ticker, name, key, spec,
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
        ticker: &Ticker,
    ) -> Result<AssetMetadataLocalKey, DispatchError> {
        CurrentAssetMetadataLocalKey::try_mutate(
            ticker,
            |current_local_key| match current_local_key {
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
            },
        )
    }
}

//==========================================================================
// All RPC functions!
//==========================================================================

impl<T: Config> Module<T> {
    pub fn unsafe_can_transfer_granular(
        from_custodian: Option<IdentityId>,
        from_portfolio: PortfolioId,
        to_custodian: Option<IdentityId>,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
        weight_meter: &mut WeightMeter,
    ) -> Result<GranularCanTransferResult, DispatchError> {
        let invalid_granularity = Self::invalid_granularity(ticker, value);
        let self_transfer = Self::self_transfer(&from_portfolio, &to_portfolio);
        let invalid_receiver_cdd = Self::invalid_cdd(to_portfolio.did);
        let invalid_sender_cdd = Self::invalid_cdd(from_portfolio.did);
        let receiver_custodian_error =
            Self::custodian_error(to_portfolio, to_custodian.unwrap_or(to_portfolio.did));
        let sender_custodian_error =
            Self::custodian_error(from_portfolio, from_custodian.unwrap_or(from_portfolio.did));
        let sender_insufficient_balance =
            Self::insufficient_balance(&ticker, from_portfolio.did, value);
        let portfolio_validity_result = <Portfolio<T>>::ensure_portfolio_transfer_validity_granular(
            &from_portfolio,
            &to_portfolio,
            ticker,
            value,
        );
        let asset_frozen = Self::frozen(ticker);
        let transfer_condition_result = Self::transfer_condition_failures_granular(
            &from_portfolio.did,
            &to_portfolio.did,
            ticker,
            value,
            weight_meter,
        )?;
        let compliance_result = T::ComplianceManager::verify_restriction_granular(
            ticker,
            Some(from_portfolio.did),
            Some(to_portfolio.did),
            weight_meter,
        )?;

        Ok(GranularCanTransferResult {
            invalid_granularity,
            self_transfer,
            invalid_receiver_cdd,
            invalid_sender_cdd,
            receiver_custodian_error,
            sender_custodian_error,
            sender_insufficient_balance,
            asset_frozen,
            result: !invalid_granularity
                && !self_transfer
                && !invalid_receiver_cdd
                && !invalid_sender_cdd
                && !receiver_custodian_error
                && !sender_custodian_error
                && !sender_insufficient_balance
                && portfolio_validity_result.result
                && !asset_frozen
                && transfer_condition_result.iter().all(|result| result.result)
                && compliance_result.result,
            transfer_condition_result,
            compliance_result,
            consumed_weight: Some(weight_meter.consumed()),
            portfolio_validity_result,
        })
    }
}

impl<T: Config> Module<T> {
    fn invalid_granularity(ticker: &Ticker, value: Balance) -> bool {
        Self::ensure_granular(ticker, value).is_err()
    }

    fn self_transfer(from: &PortfolioId, to: &PortfolioId) -> bool {
        from.did == to.did
    }

    fn invalid_cdd(did: IdentityId) -> bool {
        !Identity::<T>::has_valid_cdd(did)
    }

    fn custodian_error(from: PortfolioId, custodian: IdentityId) -> bool {
        Portfolio::<T>::ensure_portfolio_custody(from, custodian).is_err()
    }

    fn insufficient_balance(ticker: &Ticker, did: IdentityId, value: Balance) -> bool {
        Self::balance_of(&ticker, did) < value
    }

    fn transfer_condition_failures_granular(
        from_did: &IdentityId,
        to_did: &IdentityId,
        ticker: &Ticker,
        value: Balance,
        weight_meter: &mut WeightMeter,
    ) -> Result<Vec<TransferConditionResult>, DispatchError> {
        let total_supply = Self::total_supply(ticker);
        Statistics::<T>::get_transfer_restrictions_results(
            ticker,
            from_did,
            to_did,
            Self::balance_of(ticker, from_did),
            Self::balance_of(ticker, to_did),
            value,
            total_supply,
            weight_meter,
        )
    }
}

//==========================================================================
// Trait implementation!
//==========================================================================

impl<T: Config> AssetFnTrait<T::AccountId, T::RuntimeOrigin> for Module<T> {
    fn ensure_granular(ticker: &Ticker, value: Balance) -> DispatchResult {
        if let Some(security_token) = Tokens::get(ticker) {
            return Self::ensure_token_granular(&security_token, &value);
        }
        Err(Error::<T>::InvalidGranularity.into())
    }

    fn balance(ticker: &Ticker, who: IdentityId) -> Balance {
        Self::balance_of(ticker, &who)
    }

    fn create_asset(
        origin: T::RuntimeOrigin,
        name: AssetName,
        ticker: Ticker,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult {
        Self::create_asset(
            origin,
            name,
            ticker,
            divisible,
            asset_type,
            identifiers,
            funding_round,
        )
    }

    fn register_ticker(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        Self::base_register_ticker(origin, ticker)
    }

    fn issue(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        amount: Balance,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        Self::issue(origin, ticker, amount, portfolio_kind)
    }

    fn skip_ticker_affirmation(identity_id: &IdentityId, ticker: &Ticker) -> bool {
        if TickersExemptFromAffirmation::get(ticker) {
            return true;
        }
        PreApprovedTicker::get(identity_id, ticker)
    }

    fn ticker_affirmation_exemption(ticker: &Ticker) -> bool {
        TickersExemptFromAffirmation::get(ticker)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn register_asset_metadata_type(
        origin: T::RuntimeOrigin,
        ticker: Option<Ticker>,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        match ticker {
            Some(ticker) => Self::register_asset_metadata_local_type(origin, ticker, name, spec),
            None => Self::register_asset_metadata_global_type(origin, name, spec),
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn add_mandatory_mediators(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        mediators: BTreeSet<IdentityId>,
    ) -> DispatchResult {
        Self::add_mandatory_mediators(origin, ticker, mediators.try_into().unwrap_or_default())
    }
}

pub mod migration {
    use frame_support::storage::{IterableStorageMap, StorageMap, StorageValue};
    use sp_runtime::runtime_logger::RuntimeLogger;

    use crate::{
        AssetMetadataGlobalKey, AssetMetadataNextGlobalKey, AssetMetadataNextLocalKey, Config,
        CurrentAssetMetadataGlobalKey, CurrentAssetMetadataLocalKey,
    };

    pub fn migrate_to_v4<T: Config>() {
        RuntimeLogger::init();
        log::info!(">>> Initializing CurrentAssetMetadataLocalKey and CurrentAssetMetadataGlobalKey storage");
        initialize_storage::<T>();
        log::info!(">>> Storage has been initialized");
    }

    fn initialize_storage<T: Config>() {
        for (ticker, current_local_key) in AssetMetadataNextLocalKey::iter() {
            CurrentAssetMetadataLocalKey::insert(ticker, current_local_key);
        }

        let global_key = AssetMetadataNextGlobalKey::get();
        if AssetMetadataGlobalKey::default() != global_key {
            CurrentAssetMetadataGlobalKey::put(global_key);
        }
    }
}
