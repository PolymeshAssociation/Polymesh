// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
#![feature(const_option)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod checkpoint;

use arrayvec::ArrayVec;
use codec::{Decode, Encode};
use core::mem;
use core::result::Result as StdResult;
use currency::*;
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure, fail,
    traits::Get,
};
use frame_system::ensure_root;
use pallet_base::{
    ensure_opt_string_limited, ensure_string_limited, try_next_pre, Error::CounterOverflow,
};
use pallet_identity::{self as identity, PermissionedCallOriginData};
pub use polymesh_common_utilities::traits::asset::{Config, Event, RawEvent, WeightInfo};
use polymesh_common_utilities::{
    asset::{AssetFnTrait, AssetSubTrait},
    compliance_manager::Config as ComplianceManagerConfig,
    constants::*,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    with_transaction, SystematicIssuers,
};
use polymesh_primitives::{
    agent::AgentGroup,
    asset::{AssetName, AssetType, CustomAssetTypeId, FundingRoundName, GranularCanTransferResult},
    asset_metadata::{
        AssetMetadataGlobalKey, AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName,
        AssetMetadataSpec, AssetMetadataValue, AssetMetadataValueDetail,
    },
    calendar::CheckpointId,
    ethereum::{self, EcdsaSignature, EthereumAddress},
    extract_auth, storage_migrate_on, storage_migration_ver,
    transfer_compliance::TransferConditionResult,
    AssetIdentifier, Balance, Document, DocumentId, IdentityId, PortfolioId, ScopeId, SecondaryKey,
    Ticker,
};
use scale_info::TypeInfo;
use sp_runtime::traits::Zero;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::TryFrom, prelude::*};

type Checkpoint<T> = checkpoint::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;
type Statistics<T> = pallet_statistics::Module<T>;

/// Ownership status of a ticker/token.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssetOwnershipRelation {
    NotOwned,
    TickerOwned,
    AssetOwned,
}

impl Default for AssetOwnershipRelation {
    fn default() -> Self {
        Self::NotOwned
    }
}

/// struct to store the token details.
#[derive(Encode, Decode, TypeInfo, Default, Clone, PartialEq, Debug)]
pub struct SecurityToken {
    pub total_supply: Balance,
    pub owner_did: IdentityId,
    pub divisible: bool,
    pub asset_type: AssetType,
}

/// struct to store the ticker registration details.
#[derive(Encode, Decode, TypeInfo, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistration<U> {
    pub owner: IdentityId,
    pub expiry: Option<U>,
}

/// struct to store the ticker registration config.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistrationConfig<U> {
    pub max_ticker_length: u8,
    pub registration_length: Option<U>,
}

/// Enum that represents the current status of a ticker.
#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug)]
pub enum TickerRegistrationStatus {
    RegisteredByOther,
    Available,
    RegisteredByDid,
}

/// Enum that uses as the return type for the restriction verification.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RestrictionResult {
    Valid,
    Invalid,
    ForceValid,
}

impl Default for RestrictionResult {
    fn default() -> Self {
        RestrictionResult::Invalid
    }
}

/// Data imported from Polymath Classic regarding ticker registration/creation.
/// Only used at genesis config and not stored on-chain.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClassicTickerImport {
    /// Owner of the registration.
    pub eth_owner: EthereumAddress,
    /// Name of the ticker registered.
    pub ticker: Ticker,
    /// Is `eth_owner` an Ethereum contract (e.g., in case of a multisig)?
    pub is_contract: bool,
    /// Has the ticker been elevated to a created asset on classic?
    pub is_created: bool,
}

/// Data about a ticker registration from Polymath Classic on-genesis importation.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub struct ClassicTickerRegistration {
    /// Owner of the registration.
    pub eth_owner: EthereumAddress,
    /// Has the ticker been elevated to a created asset on classic?
    pub is_created: bool,
}

storage_migration_ver!(1);

decl_storage! {
    trait Store for Module<T: Config> as Asset {
        /// Ticker registration details.
        /// (ticker) -> TickerRegistration
        pub Tickers get(fn ticker_registration): map hasher(blake2_128_concat) Ticker => TickerRegistration<T::Moment>;
        /// Ticker registration config.
        /// (ticker) -> TickerRegistrationConfig
        pub TickerConfig get(fn ticker_registration_config) config(): TickerRegistrationConfig<T::Moment>;
        /// Details of the token corresponding to the token ticker.
        /// (ticker) -> SecurityToken details [returns SecurityToken struct]
        pub Tokens get(fn token_details): map hasher(blake2_128_concat) Ticker => SecurityToken;
        /// Asset name of the token corresponding to the token ticker.
        /// (ticker) -> `AssetName`
        pub AssetNames get(fn asset_names): map hasher(blake2_128_concat) Ticker => AssetName;
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
        pub CustomTypesInverse get(fn custom_types_inverse): map hasher(blake2_128_concat) Vec<u8> => CustomAssetTypeId;

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
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) DocumentId => Document;
        /// Per-ticker document ID counter.
        /// (ticker) -> doc_id
        pub AssetDocumentsIdSequence get(fn asset_documents_id_sequence): map hasher(blake2_128_concat) Ticker => DocumentId;
        /// Ticker registration details on Polymath Classic / Ethereum.
        pub ClassicTickers get(fn classic_ticker_registration): map hasher(blake2_128_concat) Ticker => Option<ClassicTickerRegistration>;
        /// Balances get stored on the basis of the `ScopeId`.
        /// Right now it is only helpful for the UI purposes but in future it can be used to do miracles on-chain.
        /// (ScopeId, IdentityId) => Balance.
        pub BalanceOfAtScope get(fn balance_of_at_scope): double_map hasher(identity) ScopeId, hasher(identity) IdentityId => Balance;
        /// Store aggregate balance of those identities that has the same `ScopeId`.
        /// (Ticker, ScopeId) => Balance.
        pub AggregateBalance get(fn aggregate_balance_of): double_map hasher(blake2_128_concat) Ticker, hasher(identity) ScopeId => Balance;
        /// Tracks the ScopeId of the identity for a given ticker.
        /// (Ticker, IdentityId) => ScopeId.
        pub ScopeIdOf get(fn scope_id_of): double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => ScopeId;

        /// Decides whether investor uniqueness requirement is enforced for this asset.
        /// `false` means that it is enforced.
        ///
        /// Ticker => bool.
        pub DisableInvestorUniqueness get(fn disable_iu): map hasher(blake2_128_concat) Ticker => bool;

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

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1).unwrap()): Version;
    }
    add_extra_genesis {
        config(classic_migration_tickers): Vec<ClassicTickerImport>;
        config(classic_migration_tconfig): TickerRegistrationConfig<T::Moment>;
        config(classic_migration_contract_did): IdentityId;
        config(reserved_country_currency_codes): Vec<Ticker>;
        build(|config: &GenesisConfig<T>| {
            use frame_system::RawOrigin;

            for &import in &config.classic_migration_tickers {
                <Module<T>>::reserve_classic_ticker(
                    RawOrigin::Root.into(),
                    import,
                    config.classic_migration_contract_did,
                    config.classic_migration_tconfig.clone()
                ).expect("`reserve_classic_ticker` failed on genesis");
            }

            // Reserving country currency logic
            let fiat_tickers_reservation_did = SystematicIssuers::FiatTickersReservation.as_id();
            for currency_ticker in &config.reserved_country_currency_codes {
                <Module<T>>::unverified_register_ticker(&currency_ticker, fiat_tickers_reservation_did, None);
            }
        });
    }
}

type Identity<T> = identity::Module<T>;

// Public interface for this runtime module.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        const AssetNameMaxLength: u32 = T::AssetNameMaxLength::get();
        const FundingRoundNameMaxLength: u32 = T::FundingRoundNameMaxLength::get();

        const AssetMetadataNameMaxLength: u32 = T::AssetMetadataNameMaxLength::get();
        const AssetMetadataValueMaxLength: u32 = T::AssetMetadataValueMaxLength::get();
        const AssetMetadataTypeDefMaxLength: u32 = T::AssetMetadataTypeDefMaxLength::get();

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            use frame_support::weights::constants::WEIGHT_PER_MICROS;
            // Keep track of upgrade cost.
            let mut weight = 0u64;
            storage_migrate_on!(StorageVersion::get(), 1, {
                let mut total_len = 0u64;
                // Get list of assets with invalid asset_types.
                let fix_list = Tokens::iter()
                    .filter(|(_, token)| {
                        total_len += 1;
                        // Check if the asset_type is invalid.
                        Self::ensure_asset_type_valid(token.asset_type).is_err()
                    }).map(|(ticker, _)| ticker).collect::<Vec<_>>();

                // Calculate weight based on the number of assets
                // and how many need to be fixed.
                // Based on storage read/write cost: read 50 micros, write 200 micros.
                let fix_len = fix_list.len() as u64;
                weight = weight
                    .saturating_add(total_len.saturating_mul(50 * WEIGHT_PER_MICROS))
                    .saturating_add(fix_len.saturating_mul(50 * WEIGHT_PER_MICROS))
                    .saturating_add(fix_len.saturating_mul(200 * WEIGHT_PER_MICROS));

                // Replace invalid asset_types with the default AssetType.
                for ticker in fix_list {
                    Tokens::mutate(&ticker, |token| {
                        token.asset_type = AssetType::default();
                    });
                }
            });

            weight
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
        /// * `disable_iu` - whether or not investor uniqueness enforcement should be disabled.
        ///   This cannot be changed after creating the asset.
        ///
        /// ## Errors
        /// - `InvalidAssetIdentifier` if any of `identifiers` are invalid.
        /// - `MaxLengthOfAssetNameExceeded` if `name`'s length exceeds `T::AssetNameMaxLength`.
        /// - `FundingRoundNameMaxLengthExceeded` if the name of the funding round is longer that
        /// `T::FundingRoundNameMaxLength`.
        /// - `AssetAlreadyCreated` if asset was already created.
        /// - `TickerTooLong` if `ticker`'s length is greater than `config.max_ticker_length` chain
        /// parameter.
        /// - `TickerNotAscii` if `ticker` is not yet registered, and contains non-ascii printable characters (from code 32 to 126) or any character after first occurrence of `\0`.
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
            disable_iu: bool,
        ) -> DispatchResult {
            Self::base_create_asset(origin, name, ticker, divisible, asset_type, identifiers, funding_round, disable_iu)
                .map(drop)
        }

        /// Freezes transfers and minting of a given token.
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
            Self::set_freeze(origin, ticker, true)
        }

        /// Unfreezes transfers and minting of a given token.
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
            Self::set_freeze(origin, ticker, false)
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

        /// Issue, or mint, new tokens to the caller,
        /// which must be an authorized external agent.
        ///
        /// # Arguments
        /// * `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` of the token.
        /// * `amount` of tokens that get issued.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::issue()]
        pub fn issue(origin, ticker: Ticker, amount: Balance) -> DispatchResult {
            // Ensure origin is agent with custody and permissions for default portfolio.
            let did = Self::ensure_agent_with_custody_and_perms(origin, ticker)?;
            Self::_mint(&ticker, did, amount, Some(ProtocolOp::AssetIssue))
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
            Self::base_redeem(origin, ticker, value)
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

        /// Claim a systematically reserved Polymath Classic (PMC) `ticker`
        /// and transfer it to the `origin`'s identity.
        ///
        /// To verify that the `origin` is in control of the Ethereum account on the books,
        /// an `ethereum_signature` containing the `origin`'s DID as the message
        /// must be provided by that Ethereum account.
        ///
        /// # Errors
        /// - `NoSuchClassicTicker` if this is not a systematically reserved PMC ticker.
        /// - `TickerAlreadyRegistered` if the ticker was already registered, e.g., by `origin`.
        /// - `TickerRegistrationExpired` if the ticker's registration has expired.
        /// - `BadOrigin` if not signed.
        /// - `InvalidEthereumSignature` if the `ethereum_signature` is not valid.
        /// - `NotAnOwner` if the ethereum account is not the owner of the PMC ticker.
        #[weight = <T as Config>::WeightInfo::claim_classic_ticker()]
        pub fn claim_classic_ticker(origin, ticker: Ticker, ethereum_signature: EcdsaSignature) -> DispatchResult {
            Self::base_claim_classic_ticker(origin, ticker, ethereum_signature)
        }

        /// Reserve a Polymath Classic (PMC) ticker.
        /// Must be called by root, and assigns the ticker to a systematic DID.
        ///
        /// # Arguments
        /// * `origin` which must be root.
        /// * `classic_ticker_import` specification for the PMC ticker.
        /// * `contract_did` to reserve the ticker to if `classic_ticker_import.is_contract` holds.
        /// * `config` to use for expiry and ticker length.
        ///
        /// # Errors
        /// * `AssetAlreadyCreated` if `classic_ticker_import.ticker` was created as an asset.
        /// * `TickerTooLong` if the `config` considers the `classic_ticker_import.ticker` too long.
        /// * `TickerAlreadyRegistered` if `classic_ticker_import.ticker` was already registered.
        #[weight = <T as Config>::WeightInfo::reserve_classic_ticker()]
        pub fn reserve_classic_ticker(
            origin,
            classic_ticker_import: ClassicTickerImport,
            contract_did: IdentityId,
            config: TickerRegistrationConfig<T::Moment>,
        ) -> DispatchResult {
            Self::base_reserve_classic_ticker(origin, classic_ticker_import, contract_did, config)
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
            Self::base_controller_transfer(origin, ticker, value, from_portfolio)
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
            disable_iu: bool,
        ) -> DispatchResult {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            with_transaction(|| {
                let asset_type_id = Self::unsafe_register_custom_asset_type(primary_did, custom_asset_type)?;
                Self::unsafe_create_asset(
                    primary_did,
                    secondary_key,
                    name,
                    ticker,
                    divisible,
                    AssetType::Custom(asset_type_id),
                    identifiers,
                    funding_round,
                    disable_iu,
                ).map(drop)
            })
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
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The user is not authorized.
        Unauthorized,
        /// The token has already been created.
        AssetAlreadyCreated,
        /// The ticker length is over the limit.
        TickerTooLong,
        /// The ticker has non-ascii-encoded parts.
        TickerNotAscii,
        /// The ticker is already registered to someone else.
        TickerAlreadyRegistered,
        /// The total supply is above the limit.
        TotalSupplyAboveLimit,
        /// No such token.
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
        /// The given ticker is not a classic one.
        NoSuchClassicTicker,
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
    }
}

impl<T: Config> AssetFnTrait<T::AccountId, T::Origin> for Module<T> {
    /// Get the asset `id` balance of `who`.
    fn balance(ticker: &Ticker, who: IdentityId) -> Balance {
        Self::balance_of(ticker, &who)
    }

    fn create_asset(
        origin: T::Origin,
        name: AssetName,
        ticker: Ticker,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
        disable_iu: bool,
    ) -> DispatchResult {
        Self::create_asset(
            origin,
            name,
            ticker,
            divisible,
            asset_type,
            identifiers,
            funding_round,
            disable_iu,
        )
    }

    fn register_ticker(origin: T::Origin, ticker: Ticker) -> DispatchResult {
        Self::base_register_ticker(origin, ticker)
    }

    #[cfg(feature = "runtime-benchmarks")]
    /// Adds an artificial IU claim for benchmarks
    fn add_investor_uniqueness_claim(did: IdentityId, ticker: Ticker) {
        use polymesh_primitives::{CddId, Claim, InvestorUid, Scope};
        Identity::<T>::base_add_claim(
            did,
            Claim::InvestorUniqueness(
                Scope::Ticker(ticker),
                did,
                CddId::new_v1(did, InvestorUid::from(did.to_bytes())),
            ),
            did,
            None,
        );
        let current_balance = Self::balance_of(ticker, did);
        AggregateBalance::insert(ticker, &did, current_balance);
        BalanceOfAtScope::insert(did, did, current_balance);
        ScopeIdOf::insert(ticker, did, did);
    }

    fn issue(origin: T::Origin, ticker: Ticker, total_supply: Balance) -> DispatchResult {
        Self::issue(origin, ticker, total_supply)
    }
}

impl<T: Config> AssetSubTrait for Module<T> {
    fn update_balance_of_scope_id(scope_id: ScopeId, target_did: IdentityId, ticker: Ticker) {
        // If `target_did` already has another ScopeId, clean up the old ScopeId data.
        if ScopeIdOf::contains_key(&ticker, &target_did) {
            let old_scope_id = Self::scope_id(&ticker, &target_did);
            // Delete the balance of target_did at old_scope_id.
            let target_balance = BalanceOfAtScope::take(old_scope_id, target_did);
            // Reduce the aggregate balance of identities with the same ScopeId by the deleted balance.
            AggregateBalance::mutate(ticker, old_scope_id, {
                |bal| *bal = bal.saturating_sub(target_balance)
            });
        }

        let balance_at_scope = Self::balance_of_at_scope(scope_id, target_did);

        // Used `balance_at_scope` variable to skip re-updating the aggregate balance of the given identityId whom
        // has the scope claim already.
        if balance_at_scope == Zero::zero() {
            let current_balance = Self::balance_of(ticker, target_did);
            // Update the balance of `target_did` under `scope_id`.
            BalanceOfAtScope::insert(scope_id, target_did, current_balance);
            // current aggregate balance + current identity balance is always less than the total supply of `ticker`.
            AggregateBalance::mutate(ticker, scope_id, |bal| *bal = *bal + current_balance);
        }
        // Caches the `ScopeId` for a given IdentityId and ticker.
        // this is needed to avoid the on-chain iteration of the claims to find the ScopeId.
        ScopeIdOf::insert(ticker, target_did, scope_id);
    }

    /// Returns balance for a given scope id and target DID.
    fn balance_of_at_scope(scope_id: &ScopeId, target: &IdentityId) -> Balance {
        Self::balance_of_at_scope(scope_id, target)
    }

    fn scope_id(ticker: &Ticker, did: &IdentityId) -> ScopeId {
        if DisableInvestorUniqueness::get(ticker) {
            *did
        } else {
            Self::scope_id_of(ticker, did)
        }
    }

    /// Ensure that Investor Uniqueness is allowed for the ticker.
    fn ensure_investor_uniqueness_claims_allowed(ticker: &Ticker) -> DispatchResult {
        ensure!(
            !DisableInvestorUniqueness::get(ticker),
            Error::<T>::InvestorUniquenessClaimNotAllowed
        );
        Ok(())
    }
}

/// All functions in the decl_module macro become part of the public interface of the module
/// If they are there, they are accessible via extrinsic calls whether they are public or not
/// However, in the impl module section (this, below) the functions can be public and private
/// Private functions are internal to this module e.g.: _transfer
/// Public functions can be called from other modules e.g.: lock and unlock (being called from the tcr module)
/// All functions in the impl module section are not part of public interface because they are not part of the Call enum.
impl<T: Config> Module<T> {
    /// Ensure that all `idents` are valid.
    fn ensure_asset_idents_valid(idents: &[AssetIdentifier]) -> DispatchResult {
        ensure!(
            idents.iter().all(|i| i.is_valid()),
            Error::<T>::InvalidAssetIdentifier
        );
        Ok(())
    }

    /// Ensure `AssetType` is valid.
    /// This checks that the `AssetType::Custom(custom_type_id)` is valid.
    fn ensure_asset_type_valid(asset_type: AssetType) -> DispatchResult {
        if let AssetType::Custom(custom_type_id) = asset_type {
            ensure!(
                CustomTypes::contains_key(custom_type_id),
                Error::<T>::InvalidCustomAssetTypeId
            );
        }
        Ok(())
    }

    pub fn base_register_ticker(origin: T::Origin, ticker: Ticker) -> DispatchResult {
        let to_did = Identity::<T>::ensure_perms(origin)?;
        let expiry = Self::ticker_registration_checks(&ticker, to_did, false, || {
            Self::ticker_registration_config()
        })?;

        T::ProtocolFee::charge_fee(ProtocolOp::AssetRegisterTicker)?;
        Self::unverified_register_ticker(&ticker, to_did, expiry);

        Ok(())
    }

    /// Update identitifiers of `ticker` as `did`.
    ///
    /// Does not verify that actor `did` is permissioned for this call or that `idents` are valid.
    fn unverified_update_idents(did: IdentityId, ticker: Ticker, idents: Vec<AssetIdentifier>) {
        Identifiers::insert(ticker, idents.clone());
        Self::deposit_event(RawEvent::IdentifiersUpdated(did, ticker, idents));
    }

    fn ensure_agent_with_custody_and_perms(
        origin: T::Origin,
        ticker: Ticker,
    ) -> Result<IdentityId, DispatchError> {
        let data = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, ticker)?;

        // Ensure the caller has not assigned custody of their default portfolio and that they are permissioned.
        let portfolio = PortfolioId::default_portfolio(data.primary_did);
        let skey = data.secondary_key.as_ref();
        Portfolio::<T>::ensure_portfolio_custody_and_permission(portfolio, data.primary_did, skey)?;
        Ok(data.primary_did)
    }

    /// Ensure that `did` is the owner of `ticker`.
    pub fn ensure_owner(ticker: &Ticker, did: IdentityId) -> DispatchResult {
        ensure!(Self::is_owner(ticker, did), Error::<T>::Unauthorized);
        Ok(())
    }

    /// Ensure that `ticker` is a valid created asset.
    fn ensure_asset_exists(ticker: &Ticker) -> DispatchResult {
        ensure!(Tokens::contains_key(&ticker), Error::<T>::NoSuchAsset);
        Ok(())
    }

    /// Ensure that the document `doc` exists for `ticker`.
    pub fn ensure_doc_exists(ticker: &Ticker, doc: &DocumentId) -> DispatchResult {
        ensure!(
            AssetDocuments::contains_key(ticker, doc),
            Error::<T>::NoSuchDoc
        );
        Ok(())
    }

    pub fn is_owner(ticker: &Ticker, did: IdentityId) -> bool {
        match Self::asset_ownership_relation(&did, &ticker) {
            AssetOwnershipRelation::AssetOwned | AssetOwnershipRelation::TickerOwned => true,
            AssetOwnershipRelation::NotOwned => false,
        }
    }

    fn maybe_ticker(ticker: &Ticker) -> Option<TickerRegistration<T::Moment>> {
        <Tickers<T>>::contains_key(ticker).then(|| <Tickers<T>>::get(ticker))
    }

    pub fn is_ticker_available(ticker: &Ticker) -> bool {
        // Assumes uppercase ticker
        if let Some(ticker) = Self::maybe_ticker(ticker) {
            ticker
                .expiry
                .filter(|&e| <pallet_timestamp::Pallet<T>>::get() > e)
                .is_some()
        } else {
            true
        }
    }

    /// Returns `true` iff the ticker exists, is owned by `did`, and ticker hasn't expired.
    pub fn is_ticker_registry_valid(ticker: &Ticker, did: IdentityId) -> bool {
        // Assumes uppercase ticker
        if let Some(ticker) = Self::maybe_ticker(ticker) {
            let now = <pallet_timestamp::Pallet<T>>::get();
            ticker.owner == did && ticker.expiry.filter(|&e| now > e).is_none()
        } else {
            false
        }
    }

    /// Returns:
    /// - `RegisteredByOther` if ticker is registered to someone else.
    /// - `Available` if ticker is available for registry.
    /// - `RegisteredByDid` if ticker is already registered to provided did.
    pub fn is_ticker_available_or_registered_to(
        ticker: &Ticker,
        did: IdentityId,
    ) -> TickerRegistrationStatus {
        // Assumes uppercase ticker
        match Self::maybe_ticker(ticker) {
            Some(TickerRegistration { expiry, owner }) => match expiry {
                // Ticker registered to someone but expired and can be registered again.
                Some(expiry) if <pallet_timestamp::Pallet<T>>::get() > expiry => {
                    TickerRegistrationStatus::Available
                }
                // Ticker is already registered to provided did (may or may not expire in future).
                _ if owner == did => TickerRegistrationStatus::RegisteredByDid,
                // Ticker registered to someone else and hasn't expired.
                _ => TickerRegistrationStatus::RegisteredByOther,
            },
            // Ticker not registered yet.
            None => TickerRegistrationStatus::Available,
        }
    }

    /// Ensure `ticker` is fully printable ASCII (SPACE to '~').
    fn ensure_ticker_ascii(ticker: &Ticker) -> DispatchResult {
        let bytes = ticker.as_slice();
        // Find first byte not printable ASCII.
        let good = bytes
            .iter()
            .position(|b| !matches!(b, 32..=126))
            // Everything after must be a NULL byte.
            .map_or(true, |nm_pos| bytes[nm_pos..].iter().all(|b| *b == 0));
        ensure!(good, Error::<T>::TickerNotAscii);
        Ok(())
    }

    /// Before registering a ticker, do some checks, and return the expiry moment.
    fn ticker_registration_checks(
        ticker: &Ticker,
        to_did: IdentityId,
        no_re_register: bool,
        config: impl FnOnce() -> TickerRegistrationConfig<T::Moment>,
    ) -> Result<Option<T::Moment>, DispatchError> {
        Self::ensure_ticker_ascii(&ticker)?;
        Self::ensure_asset_fresh(&ticker)?;

        let config = config();

        // Ensure the ticker is not too long.
        Self::ensure_ticker_length(&ticker, &config)?;

        // Ensure that the ticker is not registered by someone else (or `to_did`, possibly).
        if match Self::is_ticker_available_or_registered_to(&ticker, to_did) {
            TickerRegistrationStatus::RegisteredByOther => true,
            TickerRegistrationStatus::RegisteredByDid => no_re_register,
            _ => false,
        } {
            fail!(Error::<T>::TickerAlreadyRegistered);
        }

        Ok(config
            .registration_length
            .map(|exp| <pallet_timestamp::Pallet<T>>::get() + exp))
    }

    /// Registers the given `ticker` to the `owner` identity with an optional expiry time.
    ///
    /// ## Expected constraints
    /// - `owner` should be a valid IdentityId.
    /// - `ticker` should be valid, please see `ticker_registration_checks`.
    /// - `ticker` should be available or already registered by `owner`.
    fn unverified_register_ticker(ticker: &Ticker, owner: IdentityId, expiry: Option<T::Moment>) {
        if let Some(ticker_details) = Self::maybe_ticker(ticker) {
            AssetOwnershipRelations::remove(ticker_details.owner, ticker);
        }

        let ticker_registration = TickerRegistration { owner, expiry };

        // Store ticker registration details
        <Tickers<T>>::insert(ticker, ticker_registration);
        AssetOwnershipRelations::insert(owner, ticker, AssetOwnershipRelation::TickerOwned);

        // Not a classic ticker anymore if it was.
        ClassicTickers::remove(&ticker);

        Self::deposit_event(RawEvent::TickerRegistered(owner, *ticker, expiry));
    }

    // Get the total supply of an asset `id`.
    pub fn total_supply(ticker: Ticker) -> Balance {
        Self::token_details(ticker).total_supply
    }

    pub fn get_balance_at(ticker: Ticker, did: IdentityId, at: CheckpointId) -> Balance {
        <Checkpoint<T>>::balance_at(ticker, did, at)
            .unwrap_or_else(|| Self::balance_of(&ticker, &did))
    }

    pub fn _is_valid_transfer(
        ticker: &Ticker,
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        value: Balance,
    ) -> StdResult<u8, DispatchError> {
        if Self::frozen(ticker) {
            return Ok(ERC1400_TRANSFERS_HALTED);
        }

        if Self::missing_scope_claim(ticker, &to_portfolio, &from_portfolio) {
            return Ok(SCOPE_CLAIM_MISSING);
        }

        if Self::portfolio_failure(&from_portfolio, &to_portfolio, ticker, value) {
            return Ok(PORTFOLIO_FAILURE);
        }

        if Self::statistics_failures(&from_portfolio.did, &to_portfolio.did, ticker, value) {
            return Ok(TRANSFER_MANAGER_FAILURE);
        }

        let status_code = T::ComplianceManager::verify_restriction(
            ticker,
            Some(from_portfolio.did),
            Some(to_portfolio.did),
            value,
        )
        .unwrap_or(COMPLIANCE_MANAGER_FAILURE);

        if status_code != ERC1400_TRANSFER_SUCCESS {
            return Ok(COMPLIANCE_MANAGER_FAILURE);
        }

        Ok(ERC1400_TRANSFER_SUCCESS)
    }

    // Transfers tokens from one identity to another
    pub fn unsafe_transfer(
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
    ) -> DispatchResult {
        Self::ensure_granular(ticker, value)?;

        ensure!(
            from_portfolio.did != to_portfolio.did,
            Error::<T>::SenderSameAsReceiver
        );

        let from_total_balance = Self::balance_of(ticker, from_portfolio.did);
        ensure!(from_total_balance >= value, Error::<T>::InsufficientBalance);
        let updated_from_total_balance = from_total_balance - value;

        let to_total_balance = Self::balance_of(ticker, to_portfolio.did);
        let updated_to_total_balance = to_total_balance
            .checked_add(value)
            .ok_or(Error::<T>::BalanceOverflow)?;

        <Checkpoint<T>>::advance_update_balances(
            ticker,
            &[
                (from_portfolio.did, from_total_balance),
                (to_portfolio.did, to_total_balance),
            ],
        )?;

        // reduce sender's balance
        BalanceOf::insert(ticker, &from_portfolio.did, updated_from_total_balance);
        // increase receiver's balance
        BalanceOf::insert(ticker, &to_portfolio.did, updated_to_total_balance);
        // transfer portfolio balances
        Portfolio::<T>::unchecked_transfer_portfolio_balance(
            &from_portfolio,
            &to_portfolio,
            ticker,
            value,
        );

        let from_scope_id = Self::scope_id(ticker, &from_portfolio.did);
        let to_scope_id = Self::scope_id(ticker, &to_portfolio.did);

        Self::update_scope_balance(
            ticker,
            value,
            from_scope_id,
            from_portfolio.did,
            updated_from_total_balance,
            true,
        );
        Self::update_scope_balance(
            ticker,
            value,
            to_scope_id,
            to_portfolio.did,
            updated_to_total_balance,
            false,
        );

        // Update statistic info.
        // Using the aggregate balance to update the unique investor count.
        Statistics::<T>::update_asset_stats(
            ticker,
            Some(&from_portfolio.did),
            Some(&to_portfolio.did),
            Some(Self::aggregate_balance_of(ticker, &from_scope_id)),
            Some(Self::aggregate_balance_of(ticker, &to_scope_id)),
            value,
        );

        Self::deposit_event(RawEvent::Transfer(
            from_portfolio.did,
            *ticker,
            from_portfolio,
            to_portfolio,
            value,
        ));
        Ok(())
    }

    /// Updates scope balances after a transfer
    pub fn update_scope_balance(
        ticker: &Ticker,
        value: Balance,
        scope_id: ScopeId,
        did: IdentityId,
        updated_balance: Balance,
        is_sender: bool,
    ) {
        // Calculate the new aggregate balance for given did.
        // It should not be underflow/overflow but still to be defensive.
        let aggregate_balance = Self::aggregate_balance_of(ticker, &scope_id);
        let new_aggregate_balance = if is_sender {
            aggregate_balance.saturating_sub(value)
        } else {
            aggregate_balance.saturating_add(value)
        };

        AggregateBalance::insert(ticker, &scope_id, new_aggregate_balance);
        BalanceOfAtScope::insert(scope_id, did, updated_balance);
    }

    fn _mint(
        ticker: &Ticker,
        to_did: IdentityId,
        value: Balance,
        protocol_fee_data: Option<ProtocolOp>,
    ) -> DispatchResult {
        Self::ensure_granular(ticker, value)?;

        // Read the token details
        let mut token = Self::token_details(ticker);
        // Prepare the updated total supply.
        let updated_total_supply = token
            .total_supply
            .checked_add(value)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;
        Self::ensure_within_max_supply(updated_total_supply)?;
        // Increase receiver balance.
        let current_to_balance = Self::balance_of(ticker, to_did);
        // No check since the total balance is always <= the total supply. The
        // total supply is already checked above.
        let mut updated_to_balance = current_to_balance + value;
        // No check since the default portfolio balance is always <= the total
        // supply. The total supply is already checked above.
        let updated_to_def_balance = Portfolio::<T>::portfolio_asset_balances(
            PortfolioId::default_portfolio(to_did),
            ticker,
        ) + value;

        // In transaction because we don't want fee to be charged if advancing fails.
        with_transaction(|| {
            // Charge the fee.
            if let Some(op) = protocol_fee_data {
                T::ProtocolFee::charge_fee(op)?;
            }

            // Advance checkpoint schedules and update last checkpoint.
            <Checkpoint<T>>::advance_update_balances(ticker, &[(to_did, current_to_balance)])
        })?;

        // Increase total supply.
        token.total_supply = updated_total_supply;
        BalanceOf::insert(ticker, &to_did, updated_to_balance);
        Portfolio::<T>::set_default_portfolio_balance(to_did, ticker, updated_to_def_balance);
        Tokens::insert(ticker, token);

        // If investor uniqueness is disabled for the ticker,
        // the `scope_id` will always equal `to_did`.
        let scope_id = Self::scope_id(ticker, &to_did);
        if scope_id != ScopeId::default() {
            // scope_id can only be default if investor uniqueness
            // is enabled and the issuer doesn't have a claim yet.

            // Update scope balances.
            Self::update_scope_balance(&ticker, value, scope_id, to_did, updated_to_balance, false);

            // Using the aggregate balance to update the unique investor count.
            updated_to_balance = Self::aggregate_balance_of(ticker, &scope_id);
        }
        Statistics::<T>::update_asset_stats(
            &ticker,
            None,
            Some(&to_did),
            None,
            Some(updated_to_balance),
            value,
        );

        let round = Self::funding_round(ticker);
        let ticker_round = (*ticker, round.clone());
        // No check since the issued balance is always <= the total
        // supply. The total supply is already checked above.
        let issued_in_this_round = Self::issued_in_funding_round(&ticker_round) + value;
        IssuedInFundingRound::insert(&ticker_round, issued_in_this_round);

        Self::deposit_event(Event::<T>::Transfer(
            to_did,
            *ticker,
            PortfolioId::default(),
            PortfolioId::default_portfolio(to_did),
            value,
        ));
        Self::deposit_event(Event::<T>::Issued(
            to_did,
            *ticker,
            to_did,
            value,
            round,
            issued_in_this_round,
        ));

        Ok(())
    }

    fn ensure_granular(ticker: &Ticker, value: Balance) -> DispatchResult {
        ensure!(
            Self::check_granularity(&ticker, value),
            Error::<T>::InvalidGranularity
        );
        Ok(())
    }

    fn check_granularity(ticker: &Ticker, value: Balance) -> bool {
        Self::is_divisible(ticker) || Self::is_unit_multiple(value)
    }

    /// Is `value` a multiple of "one unit"?
    fn is_unit_multiple(value: Balance) -> bool {
        value % ONE_UNIT == 0
    }

    pub fn is_divisible(ticker: &Ticker) -> bool {
        Self::token_details(ticker).divisible
    }

    /// Accepts and executes the ticker transfer.
    fn base_accept_ticker_transfer(origin: T::Origin, auth_id: u64) -> DispatchResult {
        let to = Identity::<T>::ensure_perms(origin)?;
        <Identity<T>>::accept_auth_with(&to.into(), auth_id, |data, auth_by| {
            let ticker = extract_auth!(data, TransferTicker(t));

            Self::ensure_asset_fresh(&ticker)?;

            let owner = Self::ticker_registration(&ticker).owner;
            <Identity<T>>::ensure_auth_by(auth_by, owner)?;

            Self::transfer_ticker(ticker, to, owner);
            ClassicTickers::remove(&ticker); // Not a classic ticker anymore if it was.
            Ok(())
        })
    }

    /// Transfer the given `ticker`'s registration from `from` to `to`.
    fn transfer_ticker(ticker: Ticker, to: IdentityId, from: IdentityId) {
        AssetOwnershipRelations::remove(from, ticker);
        AssetOwnershipRelations::insert(to, ticker, AssetOwnershipRelation::TickerOwned);
        <Tickers<T>>::mutate(&ticker, |tr| tr.owner = to);
        Self::deposit_event(RawEvent::TickerTransferred(to, ticker, from));
    }

    /// Accept and process a token ownership transfer.
    fn base_accept_token_ownership_transfer(origin: T::Origin, id: u64) -> DispatchResult {
        let to = Identity::<T>::ensure_perms(origin)?;
        <Identity<T>>::accept_auth_with(&to.into(), id, |data, auth_by| {
            let ticker = extract_auth!(data, TransferAssetOwnership(t));

            Self::ensure_asset_exists(&ticker)?;
            <ExternalAgents<T>>::ensure_agent_permissioned(ticker, auth_by)?;

            let owner = Self::ticker_registration(&ticker).owner;
            AssetOwnershipRelations::remove(owner, ticker);
            AssetOwnershipRelations::insert(to, ticker, AssetOwnershipRelation::AssetOwned);
            <Tickers<T>>::mutate(&ticker, |tr| tr.owner = to);
            Tokens::mutate(&ticker, |tr| tr.owner_did = to);
            Self::deposit_event(RawEvent::AssetOwnershipTransferred(to, ticker, owner));
            Ok(())
        })
    }

    /// RPC: Function allows external users to know whether the transfer extrinsic
    /// will be valid or not beforehand.
    pub fn unsafe_can_transfer(
        from_custodian: Option<IdentityId>,
        from_portfolio: PortfolioId,
        to_custodian: Option<IdentityId>,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
    ) -> StdResult<u8, &'static str> {
        Ok(if Self::invalid_granularity(ticker, value) {
            // Granularity check
            INVALID_GRANULARITY
        } else if Self::self_transfer(&from_portfolio, &to_portfolio) {
            INVALID_RECEIVER_DID
        } else if Self::invalid_cdd(from_portfolio.did) {
            INVALID_SENDER_DID
        } else if Self::missing_scope_claim(ticker, &to_portfolio, &from_portfolio) {
            SCOPE_CLAIM_MISSING
        } else if Self::custodian_error(
            from_portfolio,
            from_custodian.unwrap_or(from_portfolio.did),
        ) {
            CUSTODIAN_ERROR
        } else if Self::invalid_cdd(to_portfolio.did) {
            INVALID_RECEIVER_DID
        } else if Self::custodian_error(to_portfolio, to_custodian.unwrap_or(to_portfolio.did)) {
            CUSTODIAN_ERROR
        } else if Self::insufficient_balance(&ticker, from_portfolio.did, value) {
            ERC1400_INSUFFICIENT_BALANCE
        } else if Self::portfolio_failure(&from_portfolio, &to_portfolio, ticker, value) {
            PORTFOLIO_FAILURE
        } else {
            // Compliance manager & Smart Extension check
            Self::_is_valid_transfer(&ticker, from_portfolio, to_portfolio, value)
                .unwrap_or(ERC1400_TRANSFER_FAILURE)
        })
    }

    /// Transfers an asset from one identity portfolio to another
    pub fn base_transfer(
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
    ) -> DispatchResult {
        // NB: This function does not check if the sender/receiver have custodian permissions on the portfolios.
        // The custodian permissions must be checked before this function is called.
        // The only place this function is used right now is the settlement engine and the settlement engine
        // checks custodial permissions when the instruction is authorized.

        // Validate the transfer
        let is_transfer_success =
            Self::_is_valid_transfer(&ticker, from_portfolio, to_portfolio, value)?;

        ensure!(
            is_transfer_success == ERC1400_TRANSFER_SUCCESS,
            Error::<T>::InvalidTransfer
        );

        Self::unsafe_transfer(from_portfolio, to_portfolio, ticker, value)?;

        Ok(())
    }

    /// Performs necessary checks on parameters of `create_asset`.
    fn ensure_create_asset_parameters(ticker: &Ticker) -> DispatchResult {
        Self::ensure_asset_fresh(&ticker)?;
        Self::ensure_ticker_length(&ticker, &Self::ticker_registration_config())
    }

    /// Ensure asset `ticker` doesn't exist yet.
    fn ensure_asset_fresh(ticker: &Ticker) -> DispatchResult {
        ensure!(
            !Tokens::contains_key(ticker),
            Error::<T>::AssetAlreadyCreated
        );
        Ok(())
    }

    /// Ensure `supply <= MAX_SUPPLY`.
    fn ensure_within_max_supply(supply: Balance) -> DispatchResult {
        ensure!(
            supply <= MAX_SUPPLY.into(),
            Error::<T>::TotalSupplyAboveLimit
        );
        Ok(())
    }

    /// Ensure ticker length is within limit per `config`.
    fn ensure_ticker_length<U>(
        ticker: &Ticker,
        config: &TickerRegistrationConfig<U>,
    ) -> DispatchResult {
        ensure!(
            ticker.len() <= usize::try_from(config.max_ticker_length).unwrap_or_default(),
            Error::<T>::TickerTooLong
        );
        Ok(())
    }

    fn base_create_asset(
        origin: T::Origin,
        name: AssetName,
        ticker: Ticker,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
        disable_iu: bool,
    ) -> Result<IdentityId, DispatchError> {
        let PermissionedCallOriginData {
            primary_did,
            secondary_key,
            ..
        } = Identity::<T>::ensure_origin_call_permissions(origin)?;
        Self::unsafe_create_asset(
            primary_did,
            secondary_key,
            name,
            ticker,
            divisible,
            asset_type,
            identifiers,
            funding_round,
            disable_iu,
        )
    }

    fn unsafe_create_asset(
        did: IdentityId,
        secondary_key: Option<SecondaryKey<T::AccountId>>,
        name: AssetName,
        ticker: Ticker,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
        disable_iu: bool,
    ) -> Result<IdentityId, DispatchError> {
        Self::ensure_asset_name_bounded(&name)?;
        if let Some(fr) = &funding_round {
            Self::ensure_funding_round_name_bounded(fr)?;
        }
        Self::ensure_asset_idents_valid(&identifiers)?;
        Self::ensure_asset_type_valid(asset_type)?;

        Self::ensure_create_asset_parameters(&ticker)?;

        // Ensure its registered by DID or at least expired, thus available.
        let available = match Self::is_ticker_available_or_registered_to(&ticker, did) {
            TickerRegistrationStatus::RegisteredByOther => {
                fail!(Error::<T>::TickerAlreadyRegistered)
            }
            TickerRegistrationStatus::RegisteredByDid => false,
            TickerRegistrationStatus::Available => true,
        };

        // If `ticker` isn't registered, it will be, so ensure it is fully ascii.
        if available {
            Self::ensure_ticker_ascii(&ticker)?;
        }

        let token_did = Identity::<T>::get_token_did(&ticker)?;
        // Ensure there's no pre-existing entry for the DID.
        // This should never happen, but let's be defensive here.
        Identity::<T>::ensure_no_id_record(token_did)?;

        // Ensure that the caller has relevant portfolio permissions
        let user_default_portfolio = PortfolioId::default_portfolio(did);
        Portfolio::<T>::ensure_portfolio_custody_and_permission(
            user_default_portfolio,
            did,
            secondary_key.as_ref(),
        )?;

        // Charge protocol fees.
        T::ProtocolFee::charge_fees(&{
            let mut fees = ArrayVec::<_, 2>::new();
            if available {
                fees.push(ProtocolOp::AssetRegisterTicker);
            }
            // Waive the asset fee iff classic ticker hasn't expired,
            // and it was already created on classic.
            if available
                || ClassicTickers::get(&ticker)
                    .filter(|r| r.is_created)
                    .is_none()
            {
                fees.push(ProtocolOp::AssetCreateAsset);
            }
            fees
        })?;

        //==========================================================================
        // At this point all checks have been made; **only** storage changes follow!
        //==========================================================================

        Identity::<T>::commit_token_did(token_did, ticker);

        // Register the ticker or finish its registration.
        if available {
            // Ticker not registered by anyone (or registry expired), so register.
            Self::unverified_register_ticker(&ticker, did, None);
        } else {
            // Ticker already registered by the user.
            <Tickers<T>>::mutate(&ticker, |tr| tr.expiry = None);
        }

        let token = SecurityToken {
            total_supply: Zero::zero(),
            owner_did: did,
            divisible,
            asset_type: asset_type.clone(),
        };
        Tokens::insert(&ticker, token);
        AssetNames::insert(&ticker, name);
        DisableInvestorUniqueness::insert(&ticker, disable_iu);
        // NB - At the time of asset creation it is obvious that the asset issuer will not have an
        // `InvestorUniqueness` claim. So we are skipping the scope claim based stats update as
        // those data points will get added in to the system whenever the asset issuer
        // has an InvestorUniqueness claim. This also applies when issuing assets.
        AssetOwnershipRelations::insert(did, ticker, AssetOwnershipRelation::AssetOwned);
        Self::deposit_event(RawEvent::AssetCreated(
            did, ticker, divisible, asset_type, did, disable_iu,
        ));

        // Add funding round name.
        FundingRound::insert(ticker, funding_round.unwrap_or_default());

        Self::unverified_update_idents(did, ticker, identifiers);

        // Grant owner full agent permissions.
        <ExternalAgents<T>>::unchecked_add_agent(ticker, did, AgentGroup::Full).unwrap();

        Ok(did)
    }

    fn set_freeze(origin: T::Origin, ticker: Ticker, freeze: bool) -> DispatchResult {
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

    fn base_rename_asset(origin: T::Origin, ticker: Ticker, name: AssetName) -> DispatchResult {
        Self::ensure_asset_name_bounded(&name)?;
        Self::ensure_asset_exists(&ticker)?;
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        AssetNames::insert(&ticker, name.clone());
        Self::deposit_event(RawEvent::AssetRenamed(did, ticker, name));
        Ok(())
    }

    /// Ensure `name` is within the global limit for asset name lengths.
    fn ensure_asset_name_bounded(name: &AssetName) -> DispatchResult {
        ensure!(
            name.len() as u32 <= T::AssetNameMaxLength::get(),
            Error::<T>::MaxLengthOfAssetNameExceeded
        );
        Ok(())
    }

    fn base_redeem(origin: T::Origin, ticker: Ticker, value: Balance) -> DispatchResult {
        // Ensure origin is agent with custody and permissions for default portfolio.
        let agent = Self::ensure_agent_with_custody_and_perms(origin, ticker)?;

        Self::ensure_granular(&ticker, value)?;

        // Reduce caller's portfolio balance. This makes sure that the caller has enough unlocked tokens.
        // If `advance_update_balances` fails, `reduce_portfolio_balance` shouldn't modify storage.
        let agent_portfolio = PortfolioId::default_portfolio(agent);
        with_transaction(|| {
            Portfolio::<T>::reduce_portfolio_balance(&agent_portfolio, &ticker, value)?;

            <Checkpoint<T>>::advance_update_balances(
                &ticker,
                &[(agent, Self::balance_of(ticker, agent))],
            )
        })?;

        let updated_balance = Self::balance_of(ticker, agent) - value;

        // Update identity balances and total supply
        BalanceOf::insert(ticker, &agent, updated_balance);
        Tokens::mutate(ticker, |token| token.total_supply -= value);

        // Update scope balances
        let scope_id = Self::scope_id(&ticker, &agent);
        Self::update_scope_balance(&ticker, value, scope_id, agent, updated_balance, true);

        // Update statistic info.
        // Using the aggregate balance to update the unique investor count.
        let updated_from_balance = Some(Self::aggregate_balance_of(ticker, &scope_id));
        Statistics::<T>::update_asset_stats(
            &ticker,
            Some(&agent),
            None,
            updated_from_balance,
            None,
            value,
        );

        Self::deposit_event(RawEvent::Transfer(
            agent,
            ticker,
            agent_portfolio,
            PortfolioId::default(),
            value,
        ));
        Self::deposit_event(RawEvent::Redeemed(agent, ticker, agent, value));

        Ok(())
    }

    fn base_make_divisible(origin: T::Origin, ticker: Ticker) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        Tokens::try_mutate(&ticker, |token| -> DispatchResult {
            ensure!(!token.divisible, Error::<T>::AssetAlreadyDivisible);
            token.divisible = true;

            Self::deposit_event(RawEvent::DivisibilityChanged(did, ticker, true));
            Ok(())
        })
    }

    fn base_add_documents(
        origin: T::Origin,
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
        origin: T::Origin,
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
        origin: T::Origin,
        ticker: Ticker,
        name: FundingRoundName,
    ) -> DispatchResult {
        Self::ensure_funding_round_name_bounded(&name)?;
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        FundingRound::insert(ticker, name.clone());
        Self::deposit_event(RawEvent::FundingRoundSet(did, ticker, name));
        Ok(())
    }

    /// Ensure `name` is within the global limit for asset name lengths.
    fn ensure_funding_round_name_bounded(name: &FundingRoundName) -> DispatchResult {
        ensure!(
            name.len() as u32 <= T::FundingRoundNameMaxLength::get(),
            Error::<T>::FundingRoundNameMaxLengthExceeded
        );
        Ok(())
    }

    fn base_update_identifiers(
        origin: T::Origin,
        ticker: Ticker,
        identifiers: Vec<AssetIdentifier>,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        Self::ensure_asset_idents_valid(&identifiers)?;
        Self::unverified_update_idents(did, ticker, identifiers);
        Ok(())
    }

    fn is_asset_metadata_locked(ticker: Ticker, key: AssetMetadataKey) -> bool {
        AssetMetadataValueDetails::<T>::get(ticker, key).map_or(false, |details| {
            details.is_locked(<pallet_timestamp::Pallet<T>>::get())
        })
    }

    fn check_asset_metadata_key_exists(ticker: Ticker, key: AssetMetadataKey) -> bool {
        match key {
            AssetMetadataKey::Global(key) => AssetMetadataGlobalKeyToName::contains_key(key),
            AssetMetadataKey::Local(key) => AssetMetadataLocalKeyToName::contains_key(ticker, key),
        }
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

    fn base_set_asset_metadata(
        origin: T::Origin,
        ticker: Ticker,
        key: AssetMetadataKey,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        Self::unverified_set_asset_metadata(did, ticker, key, value, detail)
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
            Self::check_asset_metadata_key_exists(ticker, key),
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

    fn base_set_asset_metadata_details(
        origin: T::Origin,
        ticker: Ticker,
        key: AssetMetadataKey,
        detail: AssetMetadataValueDetail<T::Moment>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        // Check key exists.
        ensure!(
            Self::check_asset_metadata_key_exists(ticker, key),
            Error::<T>::AssetMetadataKeyIsMissing
        );

        // Check if value is currently locked.
        ensure!(
            !Self::is_asset_metadata_locked(ticker, key),
            Error::<T>::AssetMetadataValueIsLocked
        );

        // Set asset metadata value details.
        AssetMetadataValueDetails::<T>::insert(ticker, key, &detail);

        Self::deposit_event(RawEvent::SetAssetMetadataValueDetails(did, ticker, detail));
        Ok(())
    }

    fn base_register_and_set_local_asset_metadata(
        origin: T::Origin,
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
        origin: T::Origin,
        ticker: Ticker,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        Self::unverified_register_asset_metadata_local_type(did, ticker, name, spec).map(drop)
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
        let key = AssetMetadataNextLocalKey::try_mutate(ticker, try_next_pre::<T, _>)?;

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

    fn base_register_asset_metadata_global_type(
        origin: T::Origin,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        Self::ensure_asset_metadata_name_limited(&name)?;
        Self::ensure_asset_metadata_spec_limited(&spec)?;

        // Only allow global metadata types to be registered by root.
        ensure_root(origin)?;

        // Check if key already exists.
        ensure!(
            !AssetMetadataGlobalNameToKey::contains_key(&name),
            Error::<T>::AssetMetadataGlobalKeyAlreadyExists
        );

        // Next global key.
        let key = AssetMetadataNextGlobalKey::try_mutate(try_next_pre::<T, _>)?;

        // Store global key <-> name mapping.
        AssetMetadataGlobalNameToKey::insert(&name, key);
        AssetMetadataGlobalKeyToName::insert(key, &name);

        // Store global specs.
        AssetMetadataGlobalSpecs::insert(key, &spec);

        Self::deposit_event(RawEvent::RegisterAssetMetadataGlobalType(name, key, spec));
        Ok(())
    }

    fn base_claim_classic_ticker(
        origin: T::Origin,
        ticker: Ticker,
        ethereum_signature: ethereum::EcdsaSignature,
    ) -> DispatchResult {
        // Ensure we're signed & get did.
        let owner_did = Identity::<T>::ensure_perms(origin)?;

        // Ensure the ticker is a classic one and fetch details.
        let ClassicTickerRegistration { eth_owner, .. } =
            ClassicTickers::get(ticker).ok_or(Error::<T>::NoSuchClassicTicker)?;

        // Ensure ticker registration is still attached to the systematic DID.
        let sys_did = SystematicIssuers::ClassicMigration.as_id();
        match Self::is_ticker_available_or_registered_to(&ticker, sys_did) {
            TickerRegistrationStatus::RegisteredByOther => {
                fail!(Error::<T>::TickerAlreadyRegistered)
            }
            TickerRegistrationStatus::Available => fail!(Error::<T>::TickerRegistrationExpired),
            TickerRegistrationStatus::RegisteredByDid => {}
        }

        // Have the caller prove that they own *some* Ethereum account
        // by having the signed signature contain the `owner_did`.
        //
        // We specifically use `owner_did` rather than `sender` such that
        // if the signing key's owner DID is changed after the creating
        // `ethereum_signature`, then the call is rejected
        // (caller might not have Ethereum account's private key).
        let eth_signer = ethereum::eth_check(owner_did, b"classic_claim", &ethereum_signature)
            .ok_or(Error::<T>::InvalidEthereumSignature)?;

        // Now we have an Ethereum account; ensure it's the *right one*.
        ensure!(eth_signer == eth_owner, Error::<T>::NotAnOwner);

        // Success; transfer the ticker to `owner_did`.
        Self::transfer_ticker(ticker, owner_did, sys_did);

        // Emit event.
        Self::deposit_event(RawEvent::ClassicTickerClaimed(
            owner_did, ticker, eth_signer,
        ));
        Ok(())
    }

    fn base_reserve_classic_ticker(
        origin: T::Origin,
        classic_ticker_import: ClassicTickerImport,
        contract_did: IdentityId,
        config: TickerRegistrationConfig<T::Moment>,
    ) -> DispatchResult {
        ensure_root(origin)?;

        let cm_did = SystematicIssuers::ClassicMigration.as_id();
        // Use DID of someone at Polymath if it's a contract-made ticker registration.
        let did = if classic_ticker_import.is_contract {
            contract_did
        } else {
            cm_did
        };

        // Register the ticker...
        let expiry =
            Self::ticker_registration_checks(&classic_ticker_import.ticker, did, true, || config)?;
        Self::unverified_register_ticker(&classic_ticker_import.ticker, did, expiry);

        // ..and associate it with additional info needed for claiming.
        let classic_ticker = ClassicTickerRegistration {
            eth_owner: classic_ticker_import.eth_owner,
            is_created: classic_ticker_import.is_created,
        };
        ClassicTickers::insert(&classic_ticker_import.ticker, classic_ticker);
        Ok(())
    }

    fn base_controller_transfer(
        origin: T::Origin,
        ticker: Ticker,
        value: Balance,
        from_portfolio: PortfolioId,
    ) -> DispatchResult {
        // Ensure `origin` has perms.
        let agent = Self::ensure_agent_with_custody_and_perms(origin, ticker)?;
        let to_portfolio = PortfolioId::default_portfolio(agent);

        // Transfer `value` of ticker tokens from `investor_did` to controller
        Self::unsafe_transfer(from_portfolio, to_portfolio, &ticker, value)?;
        Self::deposit_event(RawEvent::ControllerTransfer(
            agent,
            ticker,
            from_portfolio,
            value,
        ));
        Ok(())
    }

    pub fn unsafe_can_transfer_granular(
        from_custodian: Option<IdentityId>,
        from_portfolio: PortfolioId,
        to_custodian: Option<IdentityId>,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
    ) -> GranularCanTransferResult {
        let invalid_granularity = Self::invalid_granularity(ticker, value);
        let self_transfer = Self::self_transfer(&from_portfolio, &to_portfolio);
        let invalid_receiver_cdd = Self::invalid_cdd(from_portfolio.did);
        let invalid_sender_cdd = Self::invalid_cdd(from_portfolio.did);
        let missing_scope_claim = Self::missing_scope_claim(ticker, &to_portfolio, &from_portfolio);
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
        );
        let compliance_result = T::ComplianceManager::verify_restriction_granular(
            ticker,
            Some(from_portfolio.did),
            Some(to_portfolio.did),
        );

        GranularCanTransferResult {
            invalid_granularity,
            self_transfer,
            invalid_receiver_cdd,
            invalid_sender_cdd,
            missing_scope_claim,
            receiver_custodian_error,
            sender_custodian_error,
            sender_insufficient_balance,
            asset_frozen,
            result: !invalid_granularity
                && !self_transfer
                && !invalid_receiver_cdd
                && !invalid_sender_cdd
                && !missing_scope_claim
                && !receiver_custodian_error
                && !sender_custodian_error
                && !sender_insufficient_balance
                && portfolio_validity_result.result
                && !asset_frozen
                && transfer_condition_result.iter().all(|result| result.result)
                && compliance_result.result,
            transfer_condition_result,
            compliance_result,
            portfolio_validity_result,
        }
    }

    fn invalid_granularity(ticker: &Ticker, value: Balance) -> bool {
        !Self::check_granularity(&ticker, value)
    }

    fn self_transfer(from: &PortfolioId, to: &PortfolioId) -> bool {
        from.did == to.did
    }

    fn invalid_cdd(did: IdentityId) -> bool {
        !Identity::<T>::has_valid_cdd(did)
    }

    fn missing_scope_claim(ticker: &Ticker, from: &PortfolioId, to: &PortfolioId) -> bool {
        // We want this function missing_scope_claim to return true iff:
        // - ticker enforces scope claims (i.e. DisableInvestorUniqueness::get(ticker) == false ) AND
        // - to.did / from.did don’t have a scope claim (i.e. !Identity::<T>::verify_iu_claims_for_transfer(*ticker, to.did, from.did) )
        !DisableInvestorUniqueness::get(ticker)
            && !Identity::<T>::verify_iu_claims_for_transfer(*ticker, to.did, from.did)
    }

    fn custodian_error(from: PortfolioId, custodian: IdentityId) -> bool {
        Portfolio::<T>::ensure_portfolio_custody(from, custodian).is_err()
    }

    fn insufficient_balance(ticker: &Ticker, did: IdentityId, value: Balance) -> bool {
        Self::balance_of(&ticker, did) < value
    }

    fn portfolio_failure(
        from_portfolio: &PortfolioId,
        to_portfolio: &PortfolioId,
        ticker: &Ticker,
        value: Balance,
    ) -> bool {
        Portfolio::<T>::ensure_portfolio_transfer_validity(
            from_portfolio,
            to_portfolio,
            ticker,
            value,
        )
        .is_err()
    }

    fn setup_statistics_failures(
        from_did: &IdentityId,
        to_did: &IdentityId,
        ticker: &Ticker,
    ) -> (ScopeId, ScopeId, SecurityToken) {
        (
            Self::scope_id(ticker, &from_did),
            Self::scope_id(ticker, &to_did),
            Tokens::get(ticker),
        )
    }

    fn statistics_failures(
        from_did: &IdentityId,
        to_did: &IdentityId,
        ticker: &Ticker,
        value: Balance,
    ) -> bool {
        let (from_scope_id, to_scope_id, token) =
            Self::setup_statistics_failures(from_did, to_did, ticker);
        Statistics::<T>::verify_transfer_restrictions(
            ticker,
            from_scope_id,
            to_scope_id,
            from_did,
            to_did,
            Self::aggregate_balance_of(ticker, &from_scope_id),
            Self::aggregate_balance_of(ticker, &to_scope_id),
            value,
            token.total_supply,
        )
        .is_err()
    }

    fn transfer_condition_failures_granular(
        from_did: &IdentityId,
        to_did: &IdentityId,
        ticker: &Ticker,
        value: Balance,
    ) -> Vec<TransferConditionResult> {
        let (from_scope_id, to_scope_id, token) =
            Self::setup_statistics_failures(from_did, to_did, ticker);
        Statistics::<T>::get_transfer_restrictions_results(
            ticker,
            from_scope_id,
            to_scope_id,
            from_did,
            to_did,
            Self::aggregate_balance_of(ticker, &from_scope_id),
            Self::aggregate_balance_of(ticker, &to_scope_id),
            value,
            token.total_supply,
        )
    }

    fn base_register_custom_asset_type(
        origin: T::Origin,
        ty: Vec<u8>,
    ) -> Result<CustomAssetTypeId, DispatchError> {
        let did = Identity::<T>::ensure_perms(origin)?;
        Self::unsafe_register_custom_asset_type(did, ty)
    }

    fn unsafe_register_custom_asset_type(
        did: IdentityId,
        ty: Vec<u8>,
    ) -> Result<CustomAssetTypeId, DispatchError> {
        ensure_string_limited::<T>(&ty)?;

        Ok(match CustomTypesInverse::try_get(&ty) {
            Ok(id) => {
                Self::deposit_event(Event::<T>::CustomAssetTypeExists(did, id, ty));
                id
            }
            Err(()) => {
                let id = CustomTypeIdSequence::try_mutate(try_next_pre::<T, _>)?;
                CustomTypesInverse::insert(&ty, id);
                CustomTypes::insert(id, ty.clone());
                Self::deposit_event(Event::<T>::CustomAssetTypeRegistered(did, id, ty));
                id
            }
        })
    }
}
