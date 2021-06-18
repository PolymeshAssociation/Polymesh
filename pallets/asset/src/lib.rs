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
//! - `issue` - Function is used to issue(or mint) new tokens to the primary issuance agent.
//! - `redeem` - Redeems tokens from PIA's (Primary Issuance Agent) default portfolio.
//! - `make_divisible` - Change the divisibility of the token to divisible. Only called by the token owner.
//! - `can_transfer` - Checks whether a transaction with given parameters can take place or not.
//! - `add_documents` - Add documents for a given token, Only be called by the token owner.
//! - `remove_documents` - Remove documents for a given token, Only be called by the token owner.
//! - `set_funding_round` - Sets the name of the current funding round.
//! - `update_identifiers` - Updates the asset identifiers. Only called by the token owner.
//! - `add_extension` - It is used to permission the Smart-Extension address for a given ticker.
//! - `archive_extension` - Extension gets archived meaning it is no longer used to verify compliance or any smart logic it possesses.
//! - `unarchive_extension` - Extension gets unarchived meaning it is used again to verify compliance or any smart logic it possesses.
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
//! - `call_extension` - A helper function that is used to call the smart extension function.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
#![feature(bool_to_option, const_option)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod checkpoint;

use arrayvec::ArrayVec;
use codec::{Decode, Encode};
use core::result::Result as StdResult;
use currency::*;
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure, fail,
    traits::Get,
};
use frame_system::ensure_root;
use pallet_base::{ensure_opt_string_limited, ensure_string_limited};
use pallet_identity::{self as identity, PermissionedCallOriginData};
pub use polymesh_common_utilities::traits::asset::{Config, Event, RawEvent, WeightInfo};
use polymesh_common_utilities::{
    asset::{AssetFnTrait, AssetMigrationError, AssetSubTrait},
    compliance_manager::Config as ComplianceManagerConfig,
    constants::*,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    //traits::contracts::ContractsFn,
    with_transaction,
    Context,
    SystematicIssuers,
};
use polymesh_primitives::{
    agent::AgentGroup,
    asset::{AssetName, AssetType, FundingRoundName, GranularCanTransferResult},
    calendar::CheckpointId,
    ethereum::{self, EcdsaSignature, EthereumAddress},
    extract_auth,
    statistics::TransferManagerResult,
    storage_migrate_on, storage_migration_ver, AssetIdentifier, Document, DocumentId, IdentityId,
    MetaVersion as ExtVersion, PortfolioId, ScopeId, SmartExtension, SmartExtensionType, Ticker,
};
use sp_runtime::traits::{CheckedAdd, Saturating, Zero};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::TryFrom, prelude::*};

type Checkpoint<T> = checkpoint::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;
type Statistics<T> = pallet_statistics::Module<T>;

/// Ownership status of a ticker/token.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SecurityToken<U> {
    pub name: AssetName,
    pub total_supply: U,
    pub owner_did: IdentityId,
    pub divisible: bool,
    pub asset_type: AssetType,
}

mod migrate {
    use super::*;
    use polymesh_primitives::migrate::{Empty, Migrate};

    /// struct to store the token details.
    #[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
    pub struct SecurityTokenOld<U> {
        pub name: AssetName,
        pub total_supply: U,
        pub owner_did: IdentityId,
        pub divisible: bool,
        pub asset_type: AssetType,
        pub primary_issuance_agent: Option<IdentityId>,
    }

    impl<U: Decode + Encode> Migrate for SecurityTokenOld<U> {
        type Context = Empty;
        type Into = SecurityToken<U>;
        fn migrate(self, _: Self::Context) -> Option<Self::Into> {
            let Self {
                name,
                total_supply,
                owner_did,
                divisible,
                asset_type,
                primary_issuance_agent: _,
            } = self;
            Some(Self::Into {
                name,
                total_supply,
                owner_did,
                divisible,
                asset_type,
            })
        }
    }
}

/// struct to store the ticker registration details.
#[derive(Encode, Decode, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistration<U> {
    pub owner: IdentityId,
    pub expiry: Option<U>,
}

/// struct to store the ticker registration config.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, PartialEq, Debug)]
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
#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq, Eq)]
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
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct ClassicTickerRegistration {
    /// Owner of the registration.
    pub eth_owner: EthereumAddress,
    /// Has the ticker been elevated to a created asset on classic?
    pub is_created: bool,
}

// A value placed in storage that represents the current version of this storage. This value
// is used by the `on_runtime_upgrade` logic to determine whether we run storage migration logic.
storage_migration_ver!(3);

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
        pub Tokens get(fn token_details): map hasher(blake2_128_concat) Ticker => SecurityToken<T::Balance>;
        /// The total asset ticker balance per identity.
        /// (ticker, DID) -> Balance
        pub BalanceOf get(fn balance_of): double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) IdentityId => T::Balance;
        /// A map of a ticker name and asset identifiers.
        pub Identifiers get(fn identifiers): map hasher(blake2_128_concat) Ticker => Vec<AssetIdentifier>;

        /// The name of the current funding round.
        /// ticker -> funding round
        FundingRound get(fn funding_round): map hasher(blake2_128_concat) Ticker => FundingRoundName;
        /// The total balances of tokens issued in all recorded funding rounds.
        /// (ticker, funding round) -> balance
        IssuedInFundingRound get(fn issued_in_funding_round): map hasher(blake2_128_concat) (Ticker, FundingRoundName) => T::Balance;
        /// List of Smart extension added for the given tokens.
        /// ticker, AccountId (SE address) -> SmartExtension detail
        pub ExtensionDetails get(fn extension_details): map hasher(blake2_128_concat) (Ticker, T::AccountId) => SmartExtension<T::AccountId>;
        /// List of Smart extension added for the given tokens and for the given type.
        /// ticker, type of SE -> address/AccountId of SE
        pub Extensions get(fn extensions): map hasher(blake2_128_concat) (Ticker, SmartExtensionType) => Vec<T::AccountId>;
        /// The set of frozen assets implemented as a membership map.
        /// ticker -> bool
        pub Frozen get(fn frozen): map hasher(blake2_128_concat) Ticker => bool;
        /// Tickers and token owned by a user
        /// (user, ticker) -> AssetOwnership
        pub AssetOwnershipRelations get(fn asset_ownership_relation):
            double_map hasher(twox_64_concat) IdentityId, hasher(blake2_128_concat) Ticker => AssetOwnershipRelation;
        /// Documents attached to an Asset
        /// (ticker, doc_id) -> document
        pub AssetDocuments get(fn asset_documents):
            double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) DocumentId => Document;
        /// Per-ticker document ID counter.
        /// (ticker) -> doc_id
        pub AssetDocumentsIdSequence get(fn asset_documents_id_sequence): map hasher(blake2_128_concat) Ticker => DocumentId;
        /// Ticker registration details on Polymath Classic / Ethereum.
        pub ClassicTickers get(fn classic_ticker_registration): map hasher(blake2_128_concat) Ticker => Option<ClassicTickerRegistration>;
        /// Supported extension version.
        pub CompatibleSmartExtVersion get(fn compatible_extension_version): map hasher(blake2_128_concat) SmartExtensionType => ExtVersion;
        /// Balances get stored on the basis of the `ScopeId`.
        /// Right now it is only helpful for the UI purposes but in future it can be used to do miracles on-chain.
        /// (ScopeId, IdentityId) => Balance.
        pub BalanceOfAtScope get(fn balance_of_at_scope): double_map hasher(identity) ScopeId, hasher(identity) IdentityId => T::Balance;
        /// Store aggregate balance of those identities that has the same `ScopeId`.
        /// (Ticker, ScopeId) => Balance.
        pub AggregateBalance get(fn aggregate_balance_of): double_map hasher(blake2_128_concat) Ticker, hasher(identity) ScopeId => T::Balance;
        /// Tracks the ScopeId of the identity for a given ticker.
        /// (Ticker, IdentityId) => ScopeId.
        pub ScopeIdOf get(fn scope_id_of): double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => ScopeId;
        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(2).unwrap()): Version;
    }
    add_extra_genesis {
        config(classic_migration_tickers): Vec<ClassicTickerImport>;
        config(classic_migration_tconfig): TickerRegistrationConfig<T::Moment>;
        config(classic_migration_contract_did): IdentityId;
        config(reserved_country_currency_codes): Vec<Ticker>;
        /// Smart Extension supported version at genesis.
        config(versions): Vec<(SmartExtensionType, ExtVersion)>;
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
            config.versions
                .iter()
                .filter(|(t, _)| !<CompatibleSmartExtVersion>::contains_key(&t))
                .for_each(|(se_type, ver)| {
                    CompatibleSmartExtVersion::insert(se_type, ver);
            });

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

        const MaxNumberOfTMExtensionForAsset: u32 = T::MaxNumberOfTMExtensionForAsset::get();
        const AssetNameMaxLength: u32 = T::AssetNameMaxLength::get();
        const FundingRoundNameMaxLength: u32 = T::FundingRoundNameMaxLength::get();

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            // Migrate `AssetDocuments`.
            use frame_support::{Blake2_128Concat, Twox64Concat};
            use polymesh_primitives::{ migrate::{migrate_double_map_only_values, Migrate, Empty}, document::DocumentOld};

            storage_migrate_on!(StorageVersion::get(), 2, {
                migrate_double_map_only_values::<_, _, Blake2_128Concat, _, Twox64Concat, _, _, _>(
                    b"Asset", b"AssetDocuments",
                    |t: Ticker, id: DocumentId, doc: DocumentOld|
                        doc.migrate(Empty).ok_or_else(|| AssetMigrationError::AssetDocumentFail(t, id)))
                .for_each(|doc_migrate_status| {
                    if let Err(migrate_err) = doc_migrate_status {
                        Self::deposit_event( RawEvent::MigrationFailure(migrate_err));
                    }
                })
            });

            storage_migrate_on!(StorageVersion::get(), 3, {
                use crate::migrate::SecurityTokenOld;
                use polymesh_primitives::migrate::migrate_map_keys_and_value;
                migrate_map_keys_and_value::<_, _, Blake2_128Concat, _, _, _>(
                    b"Asset", b"Tokens", b"Tokens",
                    |ticker: Ticker, token: SecurityTokenOld<T::Balance>| {
                        ExternalAgents::<T>::add_agent_if_not(ticker, token.owner_did, AgentGroup::Full).unwrap();

                        if let Some(pia) = token.primary_issuance_agent {
                            ExternalAgents::<T>::add_agent_if_not(ticker, pia, AgentGroup::PolymeshV1PIA).unwrap();
                        }
                        token.migrate(Empty).map(|t| (ticker, t))
                    },
                );
            });

            1_000
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
            let to_did = Identity::<T>::ensure_perms(origin)?;
            Self::base_accept_ticker_transfer(to_did, auth_id)
        }

        /// This function is used to accept a token ownership transfer.
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `auth_id` Authorization ID of the token ownership transfer authorization.
        #[weight = <T as Config>::WeightInfo::accept_asset_ownership_transfer()]
        pub fn accept_asset_ownership_transfer(origin, auth_id: u64) -> DispatchResult {
            let to_did = Identity::<T>::ensure_perms(origin)?;
            Self::base_accept_token_ownership_transfer(to_did, auth_id)
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
            funding_round: Option<FundingRoundName>
        ) -> DispatchResult {
            Self::base_create_asset(origin, name, ticker, divisible, asset_type, identifiers, funding_round)
                .map(|_| ())
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

        /// Function is used to issue(or mint) new tokens to the primary issuance agent.
        /// It can be executed by the token owner or the PIA.
        ///
        /// # Arguments
        /// * `origin` Secondary key of token owner.
        /// * `ticker` Ticker of the token.
        /// * `value` Amount of tokens that get issued.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::issue()]
        pub fn issue(origin, ticker: Ticker, value: T::Balance) -> DispatchResult {
            // Ensure origin is agent with custody and permissions for default portfolio.
            let PermissionedCallOriginData {
                sender,
                primary_did,
                ..
            } = Self::ensure_agent_with_custody_and_perms(origin, ticker)?;

            Self::_mint(&ticker, sender, primary_did, value, Some(ProtocolOp::AssetIssue))
        }

        /// Redeems existing tokens by reducing the balance of the PIA's default portfolio and the total supply of the token
        ///
        /// # Arguments
        /// * `origin` Secondary key of token owner.
        /// * `ticker` Ticker of the token.
        /// * `value` Amount of tokens to redeem.
        ///
        /// # Errors
        /// - `Unauthorized` If called by someone other than the token owner or the PIA
        /// - `InvalidGranularity` If the amount is not divisible by 10^6 for non-divisible tokens
        /// - `InsufficientPortfolioBalance` If the PIA's default portfolio doesn't have enough free balance
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::redeem()]
        pub fn redeem(origin, ticker: Ticker, value: T::Balance) -> DispatchResult {
            Self::base_redeem(origin, ticker, value)
        }

        /// Makes an indivisible token divisible. Only called by the token owner.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token owner.
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

        /// Add documents for a given token. To be called only by the token owner.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token owner.
        /// * `ticker` Ticker of the token.
        /// * `docs` Documents to be attached to `ticker`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_documents(docs.len() as u32)]
        pub fn add_documents(origin, docs: Vec<Document>, ticker: Ticker) -> DispatchResult {
            Self::base_add_documents(origin, docs, ticker)
        }

        /// Remove documents for a given token. To be called only by the token owner.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token owner.
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
        /// * `origin` - the secondary key of the token owner DID.
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

        /// Updates the asset identifiers. Can only be called by the token owner.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the token owner.
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

        /*
        /// Permissioning the Smart-Extension address for a given ticker.
        ///
        /// # Arguments
        /// * `origin` - Signatory who owns to ticker/asset.
        /// * `ticker` - ticker for whom extension get added.
        /// * `extension_details` - Details of the smart extension.
        ///
        /// ## Errors
        /// - `ExtensionAlreadyPresent` if `extension_details` is already linked to `ticker`.
        /// - `IncompatibleExtensionVersion` if `extension_details` is not compatible.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_extension()]
        pub fn add_extension(origin, ticker: Ticker, extension_details: SmartExtension<T::AccountId>) -> DispatchResult {
            Self::base_add_extension(origin, ticker, extension_details)
        }

        /// Remove the given smart extension id from the list of extension under a given ticker.
        ///
        /// # Arguments
        /// * `origin` - The asset issuer.
        /// * `ticker` - Ticker symbol of the asset.
        ///
        /// ## Errors
        /// - `MissingExtensionDetails` if `ticker` is not linked to `extension_id`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_smart_extension()]
        pub fn remove_smart_extension(origin, ticker: Ticker, extension_id: T::AccountId) -> DispatchResult {
            Self::base_remove_smart_extension(origin, ticker, extension_id)
        }

        /// Archived the extension, which was used to verify compliance according to any smart logic it possesses.
        ///
        /// # Arguments
        /// * `origin` - Signatory who owns the ticker/asset.
        /// * `ticker` - Ticker symbol of the asset.
        /// * `extension_id` - AccountId of the extension that need to be archived.
        ///
        /// ## Errors
        /// -  `AlreadyArchived` if `extension_id` of `ticker` is already archived.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::archive_extension()]
        pub fn archive_extension(origin, ticker: Ticker, extension_id: T::AccountId) -> DispatchResult {
            Self::set_archive_on_extension(origin, ticker, extension_id, true)
        }

        /// Unarchived the extension. Extension is used to verify the compliance or any smart logic it possesses.
        ///
        /// # Arguments
        /// * `origin` - Signatory who owns the ticker/asset.
        /// * `ticker` - Ticker symbol of the asset.
        /// * `extension_id` - AccountId of the extension that need to be unarchived.
        ///
        /// ## Errors
        /// -  `AlreadyArchived` if `extension_id` of `ticker` is already archived.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::unarchive_extension()]
        pub fn unarchive_extension(origin, ticker: Ticker, extension_id: T::AccountId) -> DispatchResult {
            Self::set_archive_on_extension(origin, ticker, extension_id, false)
        }
        */

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

        /// Forces a transfer of token from `from_portfolio` to the PIA's default portfolio.
        /// Only PIA is allowed to execute this.
        ///
        /// # Arguments
        /// * `origin` Must be a PIA for a given ticker.
        /// * `ticker` Ticker symbol of the asset.
        /// * `value`  Amount of tokens need to force transfer.
        /// * `from_portfolio` From whom portfolio tokens gets transferred.
        #[weight = <T as Config>::WeightInfo::controller_transfer()]
        pub fn controller_transfer(origin, ticker: Ticker, value: T::Balance, from_portfolio: PortfolioId) -> DispatchResult {
            Self::base_controller_transfer(origin, ticker, value, from_portfolio)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The user is not authorized.
        Unauthorized,
        /// When extension already archived.
        AlreadyArchived,
        /// When extension already un-archived.
        AlreadyUnArchived,
        /// When extension is already added.
        ExtensionAlreadyPresent,
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
        /// Not an owner of the token.
        NotAnOwner,
        /// An overflow while calculating the balance.
        BalanceOverflow,
        /// An overflow while calculating the total supply.
        TotalSupplyOverflow,
        /// An invalid granularity.
        InvalidGranularity,
        /// The asset must be frozen.
        NotFrozen,
        /// No such smart extension.
        NoSuchSmartExtension,
        /// Transfer validation check failed.
        InvalidTransfer,
        /// The sender balance is not sufficient.
        InsufficientBalance,
        /// The token is already divisible.
        AssetAlreadyDivisible,
        /// Number of Transfer Manager extensions attached to an asset is equal to MaxNumberOfTMExtensionForAsset.
        MaximumTMExtensionLimitReached,
        /// Given smart extension is not compatible with the asset.
        IncompatibleExtensionVersion,
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
    }
}

impl<T: Config> AssetFnTrait<T::Balance, T::AccountId, T::Origin> for Module<T> {
    /// Get the asset `id` balance of `who`.
    fn balance(ticker: &Ticker, who: IdentityId) -> T::Balance {
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

    fn create_asset_and_mint(
        origin: T::Origin,
        name: AssetName,
        ticker: Ticker,
        total_supply: T::Balance,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult {
        Self::base_create_asset_and_mint(
            origin,
            name,
            ticker,
            total_supply,
            divisible,
            asset_type,
            identifiers,
            funding_round,
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
        <AggregateBalance<T>>::insert(ticker, &did, current_balance);
        <BalanceOfAtScope<T>>::insert(did, did, current_balance);
        <ScopeIdOf>::insert(ticker, did, did);
    }

    fn issue(origin: T::Origin, ticker: Ticker, total_supply: T::Balance) -> DispatchResult {
        Self::issue(origin, ticker, total_supply)
    }
}

impl<T: Config> AssetSubTrait<T::Balance> for Module<T> {
    fn accept_ticker_transfer(to: IdentityId, from: IdentityId, ticker: Ticker) -> DispatchResult {
        Self::ensure_asset_fresh(&ticker)?;

        let owner = Self::ticker_registration(&ticker).owner;
        <Identity<T>>::ensure_auth_by(from, owner)?;

        Self::transfer_ticker(ticker, to, owner);
        ClassicTickers::remove(&ticker); // Not a classic ticker anymore if it was.
        Ok(())
    }

    fn accept_asset_ownership_transfer(
        to: IdentityId,
        from: IdentityId,
        ticker: Ticker,
    ) -> DispatchResult {
        Self::ensure_asset_exists(&ticker)?;
        <ExternalAgents<T>>::ensure_agent_permissioned(ticker, from)?;

        let owner = Self::ticker_registration(&ticker).owner;
        AssetOwnershipRelations::remove(owner, ticker);
        AssetOwnershipRelations::insert(to, ticker, AssetOwnershipRelation::AssetOwned);
        <Tickers<T>>::mutate(&ticker, |tr| tr.owner = to);
        <Tokens<T>>::mutate(&ticker, |tr| tr.owner_did = to);
        Self::deposit_event(RawEvent::AssetOwnershipTransferred(to, ticker, owner));
        Ok(())
    }

    fn update_balance_of_scope_id(scope_id: ScopeId, target_did: IdentityId, ticker: Ticker) {
        // If `target_did` already has another ScopeId, clean up the old ScopeId data.
        if ScopeIdOf::contains_key(&ticker, &target_did) {
            let old_scope_id = Self::scope_id_of(&ticker, &target_did);
            // Delete the balance of target_did at old_scope_id.
            let target_balance = <BalanceOfAtScope<T>>::take(old_scope_id, target_did);
            // Reduce the aggregate balance of identities with the same ScopeId by the deleted balance.
            <AggregateBalance<T>>::mutate(ticker, old_scope_id, {
                |bal| *bal = bal.saturating_sub(target_balance)
            });
        }

        let balance_at_scope = Self::balance_of_at_scope(scope_id, target_did);

        // Used `balance_at_scope` variable to skip re-updating the aggregate balance of the given identityId whom
        // has the scope claim already.
        if balance_at_scope == Zero::zero() {
            let current_balance = Self::balance_of(ticker, target_did);
            // Update the balance of `target_did` under `scope_id`.
            <BalanceOfAtScope<T>>::insert(scope_id, target_did, current_balance);
            // current aggregate balance + current identity balance is always less than the total supply of `ticker`.
            <AggregateBalance<T>>::mutate(ticker, scope_id, |bal| *bal = *bal + current_balance);
        }
        // Caches the `ScopeId` for a given IdentityId and ticker.
        // this is needed to avoid the on-chain iteration of the claims to find the ScopeId.
        ScopeIdOf::insert(ticker, target_did, scope_id);
    }

    /// Returns balance for a given scope id and target DID.
    fn balance_of_at_scope(scope_id: &ScopeId, target: &IdentityId) -> T::Balance {
        Self::balance_of_at_scope(scope_id, target)
    }

    fn scope_id_of(ticker: &Ticker, did: &IdentityId) -> ScopeId {
        Self::scope_id_of(ticker, did)
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
    ) -> Result<PermissionedCallOriginData<T::AccountId>, DispatchError> {
        let data = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, ticker)?;

        // Ensure the PIA has not assigned custody of their default portfolio and that caller is permissioned.
        let portfolio = PortfolioId::default_portfolio(data.primary_did);
        let skey = data.secondary_key.as_ref();
        Portfolio::<T>::ensure_portfolio_custody_and_permission(portfolio, data.primary_did, skey)?;
        Ok(data)
    }

    /// Ensure that `did` is the owner of `ticker`.
    pub fn ensure_owner(ticker: &Ticker, did: IdentityId) -> DispatchResult {
        ensure!(Self::is_owner(ticker, did), Error::<T>::Unauthorized);
        Ok(())
    }

    /// Ensure that `ticker` is a valid created asset.
    fn ensure_asset_exists(ticker: &Ticker) -> DispatchResult {
        ensure!(<Tokens<T>>::contains_key(&ticker), Error::<T>::NoSuchAsset);
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
                .filter(|&e| <pallet_timestamp::Module<T>>::get() > e)
                .is_some()
        } else {
            true
        }
    }

    /// Returns `true` iff the ticker exists, is owned by `did`, and ticker hasn't expired.
    pub fn is_ticker_registry_valid(ticker: &Ticker, did: IdentityId) -> bool {
        // Assumes uppercase ticker
        if let Some(ticker) = Self::maybe_ticker(ticker) {
            let now = <pallet_timestamp::Module<T>>::get();
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
                Some(expiry) if <pallet_timestamp::Module<T>>::get() > expiry => {
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
            .map(|exp| <pallet_timestamp::Module<T>>::get() + exp))
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
    pub fn total_supply(ticker: Ticker) -> T::Balance {
        Self::token_details(ticker).total_supply
    }

    pub fn get_balance_at(ticker: Ticker, did: IdentityId, at: CheckpointId) -> T::Balance {
        <Checkpoint<T>>::balance_at(ticker, did, at)
            .unwrap_or_else(|| Self::balance_of(&ticker, &did))
    }

    pub fn _is_valid_transfer(
        ticker: &Ticker,
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        value: T::Balance,
    ) -> StdResult<u8, DispatchError> {
        if Self::frozen(ticker) {
            return Ok(ERC1400_TRANSFERS_HALTED);
        }

        if Self::missing_scope_claim(ticker, &to_portfolio, &from_portfolio) {
            return Ok(SCOPE_CLAIM_MISSING);
        }

        if Self::portfolio_failure(&from_portfolio, &to_portfolio, ticker, &value) {
            return Ok(PORTFOLIO_FAILURE);
        }

        if Self::statistics_failures(&from_portfolio, &to_portfolio, ticker, value) {
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
        value: T::Balance,
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
            .checked_add(&value)
            .ok_or(Error::<T>::BalanceOverflow)?;

        <Checkpoint<T>>::advance_update_balances(
            ticker,
            &[
                (from_portfolio.did, from_total_balance),
                (to_portfolio.did, to_total_balance),
            ],
        )?;

        // reduce sender's balance
        <BalanceOf<T>>::insert(ticker, &from_portfolio.did, updated_from_total_balance);
        // increase receiver's balance
        <BalanceOf<T>>::insert(ticker, &to_portfolio.did, updated_to_total_balance);
        // transfer portfolio balances
        Portfolio::<T>::unchecked_transfer_portfolio_balance(
            &from_portfolio,
            &to_portfolio,
            ticker,
            value,
        );

        let from_scope_id = Self::scope_id_of(ticker, &from_portfolio.did);
        let to_scope_id = Self::scope_id_of(ticker, &to_portfolio.did);

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
        Statistics::<T>::update_transfer_stats(
            ticker,
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
        value: T::Balance,
        scope_id: ScopeId,
        did: IdentityId,
        updated_balance: T::Balance,
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

        <AggregateBalance<T>>::insert(ticker, &scope_id, new_aggregate_balance);
        <BalanceOfAtScope<T>>::insert(scope_id, did, updated_balance);
    }

    pub fn _mint(
        ticker: &Ticker,
        caller: T::AccountId,
        to_did: IdentityId,
        value: T::Balance,
        protocol_fee_data: Option<ProtocolOp>,
    ) -> DispatchResult {
        Self::ensure_granular(ticker, value)?;

        // Read the token details
        let mut token = Self::token_details(ticker);
        // Prepare the updated total supply.
        let updated_total_supply = token
            .total_supply
            .checked_add(&value)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;
        Self::ensure_within_max_supply(updated_total_supply)?;
        // Increase receiver balance.
        let current_to_balance = Self::balance_of(ticker, to_did);
        // No check since the total balance is always <= the total supply. The
        // total supply is already checked above.
        let updated_to_balance = current_to_balance + value;
        // No check since the default portfolio balance is always <= the total
        // supply. The total supply is already checked above.
        let updated_to_def_balance = Portfolio::<T>::portfolio_asset_balances(
            PortfolioId::default_portfolio(to_did),
            ticker,
        ) + value;

        let caller_did = Context::current_identity_or::<Identity<T>>(&caller)?;

        // In transaction because we don't want fee to be charged if advancing fails.
        with_transaction(|| {
            // Charge the fee.
            if let Some(op) = protocol_fee_data {
                T::ProtocolFee::charge_fee(op)?;
            }

            // Advance checkpoint schedules and update last checkpoint.
            <Checkpoint<T>>::advance_update_balances(ticker, &[(to_did, current_to_balance)])
        })?;

        // Increase total supply
        token.total_supply = updated_total_supply;
        <BalanceOf<T>>::insert(ticker, &to_did, updated_to_balance);
        Portfolio::<T>::set_default_portfolio_balance(to_did, ticker, updated_to_def_balance);
        <Tokens<T>>::insert(ticker, token);

        let updated_to_balance = if ScopeIdOf::contains_key(ticker, &to_did) {
            let scope_id = Self::scope_id_of(ticker, &to_did);
            Self::update_scope_balance(&ticker, value, scope_id, to_did, updated_to_balance, false);
            // Using the aggregate balance to update the unique investor count.
            Self::aggregate_balance_of(ticker, &scope_id)
        } else {
            // Since the PIA does not have a scope claim yet, we assume this is their only identity
            value
        };
        Statistics::<T>::update_transfer_stats(&ticker, None, Some(updated_to_balance), value);

        let round = Self::funding_round(ticker);
        let ticker_round = (*ticker, round.clone());
        // No check since the issued balance is always <= the total
        // supply. The total supply is already checked above.
        let issued_in_this_round = Self::issued_in_funding_round(&ticker_round) + value;
        <IssuedInFundingRound<T>>::insert(&ticker_round, issued_in_this_round);

        Self::deposit_event(RawEvent::Transfer(
            caller_did,
            *ticker,
            PortfolioId::default(),
            PortfolioId::default_portfolio(to_did),
            value,
        ));
        Self::deposit_event(RawEvent::Issued(
            caller_did,
            *ticker,
            to_did,
            value,
            round,
            issued_in_this_round,
        ));

        Ok(())
    }

    fn ensure_granular(ticker: &Ticker, value: T::Balance) -> DispatchResult {
        ensure!(
            Self::check_granularity(&ticker, value),
            Error::<T>::InvalidGranularity
        );
        Ok(())
    }

    fn check_granularity(ticker: &Ticker, value: T::Balance) -> bool {
        // Read the token details
        let token = Self::token_details(ticker);
        token.divisible || Self::is_unit_multiple(value)
    }

    /// Is `value` a multiple of "one unit"?
    fn is_unit_multiple(value: T::Balance) -> bool {
        value % ONE_UNIT.into() == 0u32.into()
    }

    /// Accepts and executes the ticker transfer.
    pub fn base_accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult {
        <Identity<T>>::accept_auth_with(&to_did.into(), auth_id, |data, auth_by| {
            let ticker = extract_auth!(data, TransferTicker(t));
            <Self as AssetSubTrait<_>>::accept_ticker_transfer(to_did, auth_by, ticker)
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
    pub fn base_accept_token_ownership_transfer(to: IdentityId, id: u64) -> DispatchResult {
        <Identity<T>>::accept_auth_with(&to.into(), id, |data, auth_by| {
            let ticker = extract_auth!(data, TransferAssetOwnership(t));
            <Self as AssetSubTrait<_>>::accept_asset_ownership_transfer(to, auth_by, ticker)
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
        value: T::Balance,
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
        } else if Self::portfolio_failure(&from_portfolio, &to_portfolio, ticker, &value) {
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
        value: T::Balance,
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
            !<Tokens<T>>::contains_key(ticker),
            Error::<T>::AssetAlreadyCreated
        );
        Ok(())
    }

    /// Ensure `supply <= MAX_SUPPLY`.
    fn ensure_within_max_supply(supply: T::Balance) -> DispatchResult {
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

    /*
    // Return bool to know whether the given extension is compatible with the supported version of asset.
    fn is_ext_compatible(ext_type: &SmartExtensionType, extension_id: T::AccountId) -> bool {
        // Access version.
        let ext_version = T::ContractsFn::extension_info(extension_id).version;
        Self::compatible_extension_version(ext_type) == ext_version
    }

    /// Ensure the number of attached transfer manager extension should be < `MaxNumberOfTMExtensionForAsset`.
    fn ensure_max_limit_for_tm_extension(
        ext_type: &SmartExtensionType,
        ticker: &Ticker,
    ) -> DispatchResult {
        if *ext_type == SmartExtensionType::TransferManager {
            let no_of_ext = u32::try_from(
                <Extensions<T>>::get((ticker, SmartExtensionType::TransferManager)).len(),
            )
            .unwrap_or_default();
            ensure!(
                no_of_ext < T::MaxNumberOfTMExtensionForAsset::get(),
                Error::<T>::MaximumTMExtensionLimitReached
            );
        }
        Ok(())
    }

    /// Ensure the extrinsic is signed and have valid extension id.
    fn ensure_signed_and_validate_extension_id(
        origin: T::Origin,
        ticker: &Ticker,
        id: &T::AccountId,
    ) -> Result<IdentityId, DispatchError> {
        let did = <ExternalAgents<T>>::ensure_perms(origin, *ticker)?;
        ensure!(
            <ExtensionDetails<T>>::contains_key((ticker, id)),
            Error::<T>::NoSuchSmartExtension
        );
        Ok(did)
    }
    */

    pub fn base_create_asset_and_mint(
        origin: T::Origin,
        name: AssetName,
        ticker: Ticker,
        total_supply: T::Balance,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> DispatchResult {
        with_transaction(|| {
            let (sender, did) = Self::base_create_asset(
                origin,
                name,
                ticker,
                divisible,
                asset_type,
                identifiers,
                funding_round,
            )?;

            // Mint total supply to PIA
            if total_supply > Zero::zero() {
                Self::_mint(&ticker, sender, did, total_supply, None)?
            }
            Ok(())
        })
    }

    fn base_create_asset(
        origin: T::Origin,
        name: AssetName,
        ticker: Ticker,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> Result<(T::AccountId, IdentityId), DispatchError> {
        ensure!(
            name.len() as u32 <= T::AssetNameMaxLength::get(),
            Error::<T>::MaxLengthOfAssetNameExceeded
        );
        if let AssetType::Custom(ty) = &asset_type {
            ensure_string_limited::<T>(ty)?;
        }
        ensure!(
            funding_round
                .as_ref()
                .map_or(0u32, |name| name.len() as u32)
                <= T::FundingRoundNameMaxLength::get(),
            Error::<T>::FundingRoundNameMaxLengthExceeded
        );
        Self::ensure_asset_idents_valid(&identifiers)?;

        let PermissionedCallOriginData {
            sender,
            primary_did: did,
            secondary_key,
        } = Identity::<T>::ensure_origin_call_permissions(origin)?;

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
            name,
            total_supply: Zero::zero(),
            owner_did: did,
            divisible,
            asset_type: asset_type.clone(),
        };
        <Tokens<T>>::insert(&ticker, token);
        // NB - At the time of asset creation it is obvious that asset issuer/ primary issuance agent will not have
        // `InvestorUniqueness` claim. So we are skipping the scope claim based stats update as
        // those data points will get added in to the system whenever asset issuer/ primary issuance agent
        // have InvestorUniqueness claim. This also applies when issuing assets.
        <AssetOwnershipRelations>::insert(did, ticker, AssetOwnershipRelation::AssetOwned);
        Self::deposit_event(RawEvent::AssetCreated(
            did, ticker, divisible, asset_type, did,
        ));

        // Add funding round name.
        FundingRound::insert(ticker, funding_round.unwrap_or_default());

        Self::unverified_update_idents(did, ticker, identifiers);

        // Grant owner full agent permissions.
        <ExternalAgents<T>>::unchecked_add_agent(ticker, did, AgentGroup::Full).unwrap();

        Ok((sender, did))
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
        ensure!(
            name.len() as u32 <= T::AssetNameMaxLength::get(),
            Error::<T>::MaxLengthOfAssetNameExceeded
        );

        // Verify the ownership of token.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        Self::ensure_asset_exists(&ticker)?;
        <Tokens<T>>::mutate(&ticker, |token| token.name = name.clone());
        Self::deposit_event(RawEvent::AssetRenamed(did, ticker, name));
        Ok(())
    }

    fn base_redeem(origin: T::Origin, ticker: Ticker, value: T::Balance) -> DispatchResult {
        // Ensure origin is agent with custody and permissions for default portfolio.
        let pia = Self::ensure_agent_with_custody_and_perms(origin, ticker)?.primary_did;

        Self::ensure_granular(&ticker, value)?;

        // Reduce PIA's portfolio balance. This makes sure that the PIA has enough unlocked tokens.
        // If `advance_update_balances` fails, `reduce_portfolio_balance` shouldn't modify storage.
        let pia_portfolio = PortfolioId::default_portfolio(pia);
        with_transaction(|| {
            Portfolio::<T>::reduce_portfolio_balance(&pia_portfolio, &ticker, &value)?;

            <Checkpoint<T>>::advance_update_balances(
                &ticker,
                &[(pia, Self::balance_of(ticker, pia))],
            )
        })?;

        let updated_balance = Self::balance_of(ticker, pia) - value;

        // Update identity balances and total supply
        <BalanceOf<T>>::insert(ticker, &pia, updated_balance);
        <Tokens<T>>::mutate(ticker, |token| token.total_supply -= value);

        // Update scope balances
        let scope_id = Self::scope_id_of(ticker, &pia);
        Self::update_scope_balance(&ticker, value, scope_id, pia, updated_balance, true);

        // Update statistic info.
        // Using the aggregate balance to update the unique investor count.
        let updated_from_balance = Some(Self::aggregate_balance_of(ticker, &scope_id));
        Statistics::<T>::update_transfer_stats(&ticker, updated_from_balance, None, value);

        Self::deposit_event(RawEvent::Transfer(
            pia,
            ticker,
            pia_portfolio,
            PortfolioId::default(),
            value,
        ));
        Self::deposit_event(RawEvent::Redeemed(pia, ticker, pia, value));

        Ok(())
    }

    fn base_make_divisible(origin: T::Origin, ticker: Ticker) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        <Tokens<T>>::try_mutate(&ticker, |token| -> DispatchResult {
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

        // Charge fee.
        let len = docs.len();
        T::ProtocolFee::batch_charge_fee(ProtocolOp::AssetAddDocument, len)?;

        // Add the documents & emit events.
        AssetDocumentsIdSequence::mutate(ticker, |DocumentId(ref mut id)| {
            for (id, doc) in (*id..).map(DocumentId).zip(docs) {
                AssetDocuments::insert(ticker, id, doc.clone());
                Self::deposit_event(RawEvent::DocumentAdded(did, ticker, id, doc));
            }
            *id += len as u32;
        });

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
        ensure!(
            name.len() as u32 <= T::FundingRoundNameMaxLength::get(),
            Error::<T>::FundingRoundNameMaxLengthExceeded
        );
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        FundingRound::insert(ticker, name.clone());
        Self::deposit_event(RawEvent::FundingRoundSet(did, ticker, name));

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

    /*
    fn base_add_extension(
        origin: T::Origin,
        ticker: Ticker,
        details: SmartExtension<T::AccountId>,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        // Enforce length limits.
        ensure_string_limited::<T>(&details.extension_name)?;
        if let SmartExtensionType::Custom(ty) = &details.extension_type {
            ensure_string_limited::<T>(ty)?;
        }

        // Verify the details of smart extension & store it.
        ensure!(
            !<ExtensionDetails<T>>::contains_key((ticker, &details.extension_id)),
            Error::<T>::ExtensionAlreadyPresent
        );
        // Ensure the version compatibility with the asset.
        ensure!(
            Self::is_ext_compatible(&details.extension_type, details.extension_id.clone()),
            Error::<T>::IncompatibleExtensionVersion
        );
        // Ensure the hard limit on the count of maximum transfer manager an asset can have.
        Self::ensure_max_limit_for_tm_extension(&details.extension_type, &ticker)?;

        // Update the storage.
        let id = details.extension_id.clone();
        let name = details.extension_name.clone();
        let ty = details.extension_type.clone();
        <Extensions<T>>::append((ticker, &ty), id.clone());
        <ExtensionDetails<T>>::insert((ticker, &id), details);
        Self::deposit_event(Event::<T>::ExtensionAdded(did, ticker, id, name, ty));

        Ok(())
    }

    fn base_remove_smart_extension(
        origin: T::Origin,
        ticker: Ticker,
        extension_id: T::AccountId,
    ) -> DispatchResult {
        // Ensure the extrinsic is signed and have valid extension id.
        let did = Self::ensure_signed_and_validate_extension_id(origin, &ticker, &extension_id)?;
        let extension_detail_key = (ticker, extension_id.clone());
        ensure!(
            <ExtensionDetails<T>>::contains_key(&extension_detail_key),
            Error::<T>::NoSuchSmartExtension
        );

        let extension_type = Self::extension_details(&extension_detail_key).extension_type;

        // Remove the storage reference for the given extension_id.
        // The order of SEs do not matter, so `swap_remove` is OK.
        <Extensions<T>>::mutate(&(ticker, extension_type), |extension_list| {
            if let Some(pos) = extension_list.iter().position(|ext| ext == &extension_id) {
                extension_list.swap_remove(pos);
            }
        });
        <ExtensionDetails<T>>::remove(extension_detail_key);

        Self::deposit_event(RawEvent::ExtensionRemoved(did, ticker, extension_id));
        Ok(())
    }

    fn set_archive_on_extension(
        origin: T::Origin,
        ticker: Ticker,
        extension_id: T::AccountId,
        archive: bool,
    ) -> DispatchResult {
        // Ensure the extrinsic is signed and have valid extension id.
        let did = Self::ensure_signed_and_validate_extension_id(origin, &ticker, &extension_id)?;

        // Mutate the extension details
        <ExtensionDetails<T>>::try_mutate((ticker, &extension_id), |details| {
            ensure!(
                details.is_archive != archive,
                archive
                    .then_some(Error::<T>::AlreadyArchived)
                    .unwrap_or(Error::<T>::AlreadyUnArchived)
            );
            details.is_archive = archive;

            let event = match archive {
                true => RawEvent::ExtensionArchived(did, ticker, extension_id.clone()),
                false => RawEvent::ExtensionUnArchived(did, ticker, extension_id.clone()),
            };

            Self::deposit_event(event);
            Ok(())
        })
    }
    */

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
        value: T::Balance,
        from_portfolio: PortfolioId,
    ) -> DispatchResult {
        // Ensure `origin` has perms.
        let pia = Self::ensure_agent_with_custody_and_perms(origin, ticker)?.primary_did;
        let to_portfolio = PortfolioId::default_portfolio(pia);

        // Transfer `value` of ticker tokens from `investor_did` to controller
        Self::unsafe_transfer(from_portfolio, to_portfolio, &ticker, value)?;
        Self::deposit_event(RawEvent::ControllerTransfer(
            pia,
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
        value: T::Balance,
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
            &value,
        );
        let asset_frozen = Self::frozen(ticker);
        let statistics_result =
            Self::statistics_failures_granular(&from_portfolio, &to_portfolio, ticker, value);
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
                && statistics_result.iter().all(|result| result.result)
                && compliance_result.result,
            statistics_result,
            compliance_result,
            portfolio_validity_result,
        }
    }

    fn invalid_granularity(ticker: &Ticker, value: T::Balance) -> bool {
        !Self::check_granularity(&ticker, value)
    }

    fn self_transfer(from: &PortfolioId, to: &PortfolioId) -> bool {
        from.did == to.did
    }

    fn invalid_cdd(did: IdentityId) -> bool {
        !Identity::<T>::has_valid_cdd(did)
    }

    fn missing_scope_claim(ticker: &Ticker, from: &PortfolioId, to: &PortfolioId) -> bool {
        !Identity::<T>::verify_iu_claims_for_transfer(*ticker, to.did, from.did)
    }

    fn custodian_error(from: PortfolioId, custodian: IdentityId) -> bool {
        Portfolio::<T>::ensure_portfolio_custody(from, custodian).is_err()
    }

    fn insufficient_balance(ticker: &Ticker, did: IdentityId, value: T::Balance) -> bool {
        Self::balance_of(&ticker, did) < value
    }

    fn portfolio_failure(
        from_portfolio: &PortfolioId,
        to_portfolio: &PortfolioId,
        ticker: &Ticker,
        value: &T::Balance,
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
        from_portfolio: &PortfolioId,
        to_portfolio: &PortfolioId,
        ticker: &Ticker,
    ) -> (ScopeId, ScopeId, SecurityToken<T::Balance>) {
        (
            Self::scope_id_of(ticker, &from_portfolio.did),
            Self::scope_id_of(ticker, &to_portfolio.did),
            <Tokens<T>>::get(ticker),
        )
    }

    fn statistics_failures(
        from_portfolio: &PortfolioId,
        to_portfolio: &PortfolioId,
        ticker: &Ticker,
        value: T::Balance,
    ) -> bool {
        let (from_scope_id, to_scope_id, token) =
            Self::setup_statistics_failures(from_portfolio, to_portfolio, ticker);
        Statistics::<T>::verify_tm_restrictions(
            ticker,
            from_scope_id,
            to_scope_id,
            value,
            Self::aggregate_balance_of(ticker, &from_scope_id),
            Self::aggregate_balance_of(ticker, &to_scope_id),
            token.total_supply,
        )
        .is_err()
    }

    fn statistics_failures_granular(
        from_portfolio: &PortfolioId,
        to_portfolio: &PortfolioId,
        ticker: &Ticker,
        value: T::Balance,
    ) -> Vec<TransferManagerResult> {
        let (from_scope_id, to_scope_id, token) =
            Self::setup_statistics_failures(from_portfolio, to_portfolio, ticker);
        Statistics::<T>::verify_tm_restrictions_granular(
            ticker,
            from_scope_id,
            to_scope_id,
            value,
            Self::aggregate_balance_of(ticker, &from_scope_id),
            Self::aggregate_balance_of(ticker, &to_scope_id),
            token.total_supply,
        )
    }
}
