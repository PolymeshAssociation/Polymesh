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
//! It consist every required functionality related to securityToken and every function
//! execution can be differentiate at the token level by providing the ticker of the token.
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
//! - Custodian functionality.
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
//! - `create_checkpoint` - Function used to create the checkpoint.
//! - `issue` - Function is used to issue(or mint) new tokens to the primary issuance agent.
//! - `controller_redeem` - Forces a redemption of an DID's tokens. Can only be called by token owner.
//! - `make_divisible` - Change the divisibility of the token to divisible. Only called by the token owner.
//! - `can_transfer` - Checks whether a transaction with given parameters can take place or not.
//! - `add_documents` - Add documents for a given token, Only be called by the token owner.
//! - `remove_documents` - Remove documents for a given token, Only be called by the token owner.
//! - `increase_custody_allowance` - Used to increase the allowance for a given custodian.
//! - `increase_custody_allowance_of` - Used to increase the allowance for a given custodian by providing the off chain signature.
//! - `transfer_by_custodian` - Used to transfer the tokens by the approved custodian.
//! - `set_funding_round` - Sets the name of the current funding round.
//! - `update_identifiers` - Updates the asset identifiers. Only called by the token owner.
//! - `add_extension` - It is used to permission the Smart-Extension address for a given ticker.
//! - `archive_extension` - Extension gets archived it means extension is no more use to verify the compliance or any smart logic it posses.
//! - `unarchive_extension` - Extension gets un-archived it means extension is use to verify the compliance or any smart logic it posses.
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
//! - `custodian_allowance`- Returns the allowance provided to a custodian for a given ticker and token holder.
//! - `total_custody_allowance` - Returns the total allowance approved by the token holder.
//! - `extension_details` - It provides the list of Smart extension added for the given tokens.
//! - `extensions` - It provides the list of Smart extension added for the given tokens and for the given type.
//! - `frozen` - It tells whether the given ticker is frozen or not.
//! - `is_ticker_available` - It checks whether the given ticker is available or not.
//! - `is_ticker_registry_valid` - It checks whether the ticker is own by a given IdentityId or not.
//! - `is_ticker_available_or_registered_to` - It provides the status of a given ticker.
//! - `total_supply` - It provides the total supply of a ticker.
//! - `get_balance_at` - It provides the balance of a DID at a certain checkpoint.
//! - `verify_restriction` - It is use to verify the restriction implied by the smart extension and the Compliance Manager.
//! - `call_extension` - A helper function that is used to call the smart extension function.
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
#![feature(bool_to_option)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod ethereum;

use codec::{Decode, Encode};
use core::result::Result as StdResult;
use currency::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
    ensure,
    traits::{Currency, Get},
    weights::Weight,
};
use frame_system::{self as system, ensure_signed};
use hex_literal::hex;
use pallet_contracts::{ExecReturnValue, Gas};
use pallet_identity as identity;
use pallet_statistics::{self as statistics, Counter};
use polymesh_common_utilities::{
    asset::{AcceptTransfer, Trait as AssetTrait, GAS_LIMIT},
    balances::Trait as BalancesTrait,
    compliance_manager::Trait as ComplianceManagerTrait,
    constants::*,
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    CommonTrait, Context, SystematicIssuers,
};
use polymesh_primitives::{
    AuthorizationData, AuthorizationError, Document, DocumentName, IdentityId, Signatory,
    SmartExtension, SmartExtensionName, SmartExtensionType, Ticker,
};
use polymesh_primitives_derive::{SliceU8StrongTyped, VecU8StrongTyped};
use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating, Verify};

#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::TryFrom, prelude::*};

type Portfolio<T> = pallet_portfolio::Module<T>;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + BalancesTrait
    + IdentityTrait
    + pallet_session::Trait
    + statistics::Trait
    + pallet_contracts::Trait
    + pallet_portfolio::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type ComplianceManager: ComplianceManagerTrait<Self::Balance>;
    /// Maximum number of smart extensions can attach to a asset.
    /// This hard limit is set to avoid the cases where a asset transfer
    /// gas usage go beyond the block gas limit.
    type MaxNumberOfTMExtensionForAsset: Get<u32>;
}

/// The type of an asset represented by a token.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum AssetType {
    EquityCommon,
    EquityPreferred,
    Commodity,
    FixedIncome,
    REIT,
    Fund,
    RevenueShareAgreement,
    StructuredProduct,
    Derivative,
    Custom(Vec<u8>),
}

impl Default for AssetType {
    fn default() -> Self {
        AssetType::Custom(b"undefined".to_vec())
    }
}

/// The type of an identifier associated with a token.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifierType {
    Cins,
    Cusip,
    Isin,
    Dti,
}

impl Default for IdentifierType {
    fn default() -> Self {
        IdentifierType::Isin
    }
}

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

/// A wrapper for a token name.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct AssetName(pub Vec<u8>);

/// A wrapper for an asset ID.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct AssetIdentifier(pub Vec<u8>);

/// A wrapper for a funding round name.
#[derive(Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped)]
pub struct FundingRoundName(pub Vec<u8>);

impl Default for FundingRoundName {
    fn default() -> Self {
        FundingRoundName("".as_bytes().to_vec())
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
    pub primary_issuance_agent: Option<IdentityId>,
}

/// struct to store the signed data.
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SignData<U> {
    pub custodian_did: IdentityId,
    pub holder_did: IdentityId,
    pub ticker: Ticker,
    pub value: U,
    pub nonce: u16,
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

/// The total asset balance and the balance of the asset in a specified portfolio of an identity.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FocusedBalances<Balance> {
    /// The total balance of the asset held by the identity.
    pub total: Balance,
    /// The balance of the asset in the default portfolio of the identity.
    pub portfolio: Balance,
}

/// The Countries Currency Codes
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
    Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, SliceU8StrongTyped,
)]
pub struct CountryCurrencyCodes([u8; 3]);

pub mod weight_for {
    use super::*;

    /// Weight for `_is_valid_transfer()` transfer.
    pub fn weight_for_is_valid_transfer<T: Trait>(
        no_of_tms: u32,
        weight_from_cm: Weight,
    ) -> Weight {
        8 * 10_000_000 // Weight used for encoding a param in `verify_restriction()` call.
            .saturating_add(GAS_LIMIT.saturating_mul(no_of_tms.into())) // used gas limit for a single TM extension call.
            .saturating_add(weight_from_cm) // weight that comes from the compliance manager.
    }

    /// Weight for `unsafe_transfer_by_custodian()`.
    pub fn weight_for_unsafe_transfer_by_custodian<T: Trait>(
        weight_for_transfer_rest: Weight,
    ) -> Weight {
        weight_for_transfer_rest
            .saturating_add(T::DbWeight::get().reads_writes(3, 2)) // Read and write of `unsafe_transfer_by_custodian()`
            .saturating_add(T::DbWeight::get().reads_writes(4, 5)) // read and write for `unsafe_transfer()`
    }
}

/// An Ethereum address (i.e. 20 bytes, used to represent an Ethereum account).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, Debug)]
pub struct EthereumAddress([u8; 20]);

/// Data imported from Polymath Classic regarding ticker registration/creation.
/// Only used at genesis config and not stored on-chain.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct ClassicTickerImport {
    /// Owner of the registration.
    pub eth_owner: ethereum::EthereumAddress,
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
    pub eth_owner: ethereum::EthereumAddress,
    /// Has the ticker been elevated to a created asset on classic?
    pub is_created: bool,
}

decl_storage! {
    trait Store for Module<T: Trait> as Asset {
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
        /// A map of pairs of a ticker name and an `IdentifierType` to asset identifiers.
        pub Identifiers get(fn identifiers): map hasher(blake2_128_concat) (Ticker, IdentifierType) => AssetIdentifier;
        /// Checkpoints created per token.
        /// (ticker) -> no. of checkpoints
        pub TotalCheckpoints get(fn total_checkpoints_of): map hasher(blake2_128_concat) Ticker => u64;
        /// Total supply of the token at the checkpoint.
        /// (ticker, checkpointId) -> total supply at given checkpoint
        pub CheckpointTotalSupply get(fn total_supply_at): map hasher(blake2_128_concat) (Ticker, u64) => T::Balance;
        /// Balance of a DID at a checkpoint.
        /// (ticker, did, checkpoint ID) -> Balance of a DID at a checkpoint
        CheckpointBalance get(fn balance_at_checkpoint): map hasher(blake2_128_concat) (Ticker, IdentityId, u64) => T::Balance;
        /// Last checkpoint updated for a DID's balance.
        /// (ticker, did) -> List of checkpoints where user balance changed
        UserCheckpoints get(fn user_checkpoints): map hasher(blake2_128_concat) (Ticker, IdentityId) => Vec<u64>;
        /// Allowance provided to the custodian.
        /// (ticker, token holder, custodian) -> balance
        pub CustodianAllowance get(fn custodian_allowance): map hasher(blake2_128_concat) (Ticker, IdentityId, IdentityId) => T::Balance;
        /// Total custodian allowance for a given token holder.
        /// (ticker, token holder) -> balance
        pub TotalCustodyAllowance get(fn total_custody_allowance): map hasher(blake2_128_concat) (Ticker, IdentityId) => T::Balance;
        /// Store the nonce for off chain signature to increase the custody allowance.
        /// (ticker, token holder, nonce) -> bool
        AuthenticationNonce get(fn authentication_nonce): map hasher(blake2_128_concat) (Ticker, IdentityId, u16) => bool;
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
        /// (ticker, document_name) -> document
        pub AssetDocuments get(fn asset_documents):
            double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) DocumentName => Document;

        /// Ticker registration details on Polymath Classic / Ethereum.
        pub ClassicTickers get(fn classic_ticker_registration): map hasher(blake2_128_concat) Ticker => Option<ClassicTickerRegistration>;
    }
    add_extra_genesis {
        config(classic_migration_tickers): Vec<ClassicTickerImport>;
        config(classic_migration_tconfig): TickerRegistrationConfig<T::Moment>;
        config(classic_migration_contract_did): IdentityId;
        config(reserved_country_currency_codes): Vec<(IdentityId, Vec<CountryCurrencyCodes>)>;
        build(|config: &GenesisConfig<T>| {
            let cm_did = SystematicIssuers::ClassicMigration.as_id();
            for import in &config.classic_migration_tickers {
                // Use DID of someone at Polymath if it's a contract-made ticker registration.
                let did = if import.is_contract { config.classic_migration_contract_did } else { cm_did };

                // Register the ticker...
                let tconfig = || config.classic_migration_tconfig.clone();
                let expiry = <Module<T>>::ticker_registration_checks(&import.ticker, did, true, tconfig);
                <Module<T>>::_register_ticker(&import.ticker, did, expiry.unwrap());

                // ..and associate it with additional info needed for claiming.
                let classic_ticker = ClassicTickerRegistration {
                    eth_owner: import.eth_owner,
                    is_created: import.is_created,
                };
                ClassicTickers::insert(&import.ticker, classic_ticker);
            }

            // Reserving country currency logic
            let tconfig = || config.ticker_registration_config.clone();
            let sys_did = &config.reserved_country_currency_codes[0].0;
            for code in &config.reserved_country_currency_codes[0].1 {
                let ticker = Ticker::try_from(code.as_slice()).expect("cannot convert country code to ticker");
                let expiry = <Module<T>>::ticker_registration_checks(&ticker, *sys_did, true, tconfig);
                <Module<T>>::_register_ticker(&ticker, *sys_did, expiry.unwrap());
            }

        });
    }
}

type Identity<T> = identity::Module<T>;

// Public interface for this runtime module.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        /// This function is used to either register a new ticker or extend validity of an existing ticker.
        /// NB: Ticker validity does not get carry forward when renewing ticker.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `ticker` ticker to register.
        #[weight = T::DbWeight::get().reads_writes(4, 3) + 500_000_000]
        pub fn register_ticker(origin, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let to_did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let expiry = Self::ticker_registration_checks(&ticker, to_did, false, || Self::ticker_registration_config())?;
            T::ProtocolFee::charge_fee(ProtocolOp::AssetRegisterTicker)?;
            Self::_register_ticker(&ticker, to_did, expiry);
            Ok(())
        }

        /// This function is used to accept a ticker transfer.
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `auth_id` Authorization ID of ticker transfer authorization.
        #[weight = T::DbWeight::get().reads_writes(4, 5) + 200_000_000]
        pub fn accept_ticker_transfer(origin, auth_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let to_did = Context::current_identity_or::<Identity<T>>(&sender)?;

            Self::_accept_ticker_transfer(to_did, auth_id)
        }

        /// This function is used to accept a primary issuance agent transfer.
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the signing key of the caller (i.e who signed the transaction to execute this function).
        /// * `auth_id` Authorization ID of primary issuance agent transfer authorization.
        #[weight = 300_000_000]
        pub fn accept_primary_issuance_agent_transfer(origin, auth_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let to_did = Context::current_identity_or::<Identity<T>>(&sender)?;

            Self::_accept_primary_issuance_agent_transfer(to_did, auth_id)
        }

        /// This function is used to accept a token ownership transfer.
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `auth_id` Authorization ID of the token ownership transfer authorization.
        #[weight = T::DbWeight::get().reads_writes(4, 5) + 200_000_000]
        pub fn accept_asset_ownership_transfer(origin, auth_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let to_did = Context::current_identity_or::<Identity<T>>(&sender)?;

            Self::_accept_token_ownership_transfer(to_did, auth_id)
        }

        /// Initializes a new security token
        /// makes the initiating account the owner of the security token
        /// & the balance of the owner is set to total supply.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `name` - the name of the token.
        /// * `ticker` - the ticker symbol of the token.
        /// * `total_supply` - the total supply of the token.
        /// * `divisible` - a boolean to identify the divisibility status of the token.
        /// * `asset_type` - the asset type.
        /// * `identifiers` - a vector of asset identifiers.
        /// * `funding_round` - name of the funding round.
        ///
        /// # Weight
        /// `3_000_000_000 + 20_000 * identifiers.len()`
        #[weight = 3_000_000_000 + 20_000 * u64::try_from(identifiers.len()).unwrap_or_default()]
        pub fn create_asset(
            origin,
            name: AssetName,
            ticker: Ticker,
            total_supply: T::Balance,
            divisible: bool,
            asset_type: AssetType,
            identifiers: Vec<(IdentifierType, AssetIdentifier)>,
            funding_round: Option<FundingRoundName>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            Self::ensure_create_asset_parameters(&ticker, &name, total_supply)?;

            // Ensure its registered by DID or at least expired, thus available.
            let available = match Self::is_ticker_available_or_registered_to(&ticker, did) {
                TickerRegistrationStatus::RegisteredByOther => return Err(Error::<T>::TickerAlreadyRegistered.into()),
                TickerRegistrationStatus::RegisteredByDid => false,
                TickerRegistrationStatus::Available => true,
            };

            if !divisible {
                ensure!(total_supply % ONE_UNIT.into() == 0.into(), Error::<T>::InvalidTotalSupply);
            }

            let token_did = <identity::Module<T>>::get_token_did(&ticker)?;
            // Ensure there's no pre-existing entry for the DID.
            // This should never happen, but let's be defensive here.
            identity::Module::<T>::ensure_no_id_record(token_did)?;

            // Charge protocol fees.
            T::ProtocolFee::charge_fees(&{
                let mut fees = arrayvec::ArrayVec::<[_; 2]>::new();
                if available {
                    fees.push(ProtocolOp::AssetRegisterTicker);
                }
                // Waive the asset fee iff classic ticker hasn't expired,
                // and it was already created on classic.
                if available || ClassicTickers::get(&ticker).filter(|r| r.is_created).is_none() {
                    fees.push(ProtocolOp::AssetCreateAsset);
                }
                fees
            })?;

            //==========================================================================
            // At this point all checks have been made; **only** storage changes follow!
            //==========================================================================

            identity::Module::<T>::commit_token_did(token_did, ticker);

            // Register the ticker or finish its registration.
            if available {
                // Ticker not registered by anyone (or registry expired), so register.
                Self::_register_ticker(&ticker, did, None);
            } else {
                // Ticker already registered by the user.
                <Tickers<T>>::mutate(&ticker, |tr| tr.expiry = None);
            }

            let token = SecurityToken {
                name,
                total_supply,
                owner_did: did,
                divisible,
                asset_type: asset_type.clone(),
                primary_issuance_agent: Some(did),
            };
            <Tokens<T>>::insert(&ticker, token);
            <BalanceOf<T>>::insert(ticker, did, total_supply);
            Portfolio::<T>::set_default_portfolio_balance(did, &ticker, total_supply);
            <AssetOwnershipRelations>::insert(did, ticker, AssetOwnershipRelation::AssetOwned);
            Self::deposit_event(RawEvent::AssetCreated(
                did,
                ticker,
                total_supply,
                divisible,
                asset_type,
                did,
            ));
            for (typ, val) in &identifiers {
                <Identifiers>::insert((ticker, typ.clone()), val.clone());
            }
            // Add funding round name.
            <FundingRound>::insert(ticker, funding_round.unwrap_or_default());

            // Update the investor count of an asset.
            <statistics::Module<T>>::update_transfer_stats(&ticker, None, Some(total_supply), total_supply);

            Self::deposit_event(RawEvent::IdentifiersUpdated(did, ticker, identifiers));
            <IssuedInFundingRound<T>>::insert((ticker, Self::funding_round(ticker)), total_supply);
            Self::deposit_event(RawEvent::Transfer(
                did,
                ticker,
                IdentityId::default(),
                did,
                total_supply
            ));
            Self::deposit_event(RawEvent::Issued(
                did,
                ticker,
                did,
                total_supply,
                Self::funding_round(ticker),
                total_supply,
                Some(did),
            ));
            Ok(())
        }

        /// Freezes transfers and minting of a given token.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the ticker of the token.
        #[weight = T::DbWeight::get().reads_writes(4, 1) + 300_000_000]
        pub fn freeze(origin, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_did = Context::current_identity_or::<Identity<T>>(&sender)?;

            // verify the ownership of the token
            ensure!(Self::is_owner(&ticker, sender_did), Error::<T>::Unauthorized);
            ensure!(<Tokens<T>>::contains_key(&ticker), Error::<T>::NoSuchAsset);

            ensure!(!Self::frozen(&ticker), Error::<T>::AlreadyFrozen);
            <Frozen>::insert(&ticker, true);
            Self::deposit_event(RawEvent::AssetFrozen(sender_did, ticker));
            Ok(())
        }

        /// Unfreezes transfers and minting of a given token.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the ticker of the frozen token.
        #[weight = T::DbWeight::get().reads_writes(4, 1) + 300_000_000]
        pub fn unfreeze(origin, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_did = Context::current_identity_or::<Identity<T>>(&sender)?;

            // verify the ownership of the token
            ensure!(Self::is_owner(&ticker, sender_did), Error::<T>::Unauthorized);
            ensure!(<Tokens<T>>::contains_key(&ticker), Error::<T>::NoSuchAsset);

            ensure!(Self::frozen(&ticker), Error::<T>::NotFrozen);
            <Frozen>::insert(&ticker, false);
            Self::deposit_event(RawEvent::AssetUnfrozen(sender_did, ticker));
            Ok(())
        }

        /// Renames a given token.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `ticker` - the ticker of the token.
        /// * `name` - the new name of the token.
        #[weight = T::DbWeight::get().reads_writes(2, 1) + 300_000_000]
        pub fn rename_asset(origin, ticker: Ticker, name: AssetName) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_did = Context::current_identity_or::<Identity<T>>(&sender)?;

            // verify the ownership of the token
            ensure!(Self::is_owner(&ticker, sender_did), Error::<T>::Unauthorized);
            ensure!(<Tokens<T>>::contains_key(&ticker), Error::<T>::NoSuchAsset);

            <Tokens<T>>::mutate(&ticker, |token| token.name = name.clone());
            Self::deposit_event(RawEvent::AssetRenamed(sender_did, ticker, name));
            Ok(())
        }

        /// Forces a transfer between two DIDs & This can only be called by security token owner.
        /// This function doesn't validate any type of restriction beside a valid CDD check.
        ///
        /// # Arguments
        /// * `origin` secondary key of the token owner DID.
        /// * `ticker` symbol of the token.
        /// * `from_did` DID of the token holder from whom balance token will be transferred.
        /// * `to_did` DID of token holder to whom token balance will be transferred.
        /// * `value` Amount of tokens.
        /// * `data` Some off chain data to validate the restriction.
        /// * `operator_data` It is a string which describes the reason of this control transfer call.
        #[weight = T::DbWeight::get().reads_writes(3, 2) + 500_000_000]
        pub fn controller_transfer(origin, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, value: T::Balance, data: Vec<u8>, operator_data: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            Self::unsafe_transfer(did, &ticker, from_did, to_did, value)?;

            Self::deposit_event(RawEvent::ControllerTransfer(did, ticker, from_did, to_did, value, data, operator_data));

            Ok(())
        }

        /// Function used to create the checkpoint.
        /// NB: Only called by the owner of the security token i.e owner DID.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token owner. (Only token owner can call this function).
        /// * `ticker` Ticker of the token.
        #[weight = T::DbWeight::get().reads_writes(3, 2) + 400_000_000]
        pub fn create_checkpoint(origin, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            let _ = Self::_create_checkpoint(&ticker)?;
            Self::deposit_event(RawEvent::CheckpointCreated(did, ticker, Self::total_checkpoints_of(&ticker)));
            Ok(())
        }

        /// Function is used to issue(or mint) new tokens to the primary issuance agent.
        /// It can only be executed by the token owner.
        ///
        /// # Arguments
        /// * `origin` Secondary key of token owner.
        /// * `ticker` Ticker of the token.
        /// * `value` Amount of tokens that get issued.
        #[weight = T::DbWeight::get().reads_writes(6, 3) + 800_000_000]
        pub fn issue(origin, ticker: Ticker, value: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            let beneficiary = Self::token_details(&ticker).primary_issuance_agent.unwrap_or(did);
            Self::_mint(&ticker, sender, beneficiary, value, Some(ProtocolOp::AssetIssue))
        }

        /// Forces a redemption of an DID's tokens. Can only be called by token owner.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token owner.
        /// * `ticker` Ticker of the token.
        /// * `token_holder_did` DID from whom balance get reduced.
        /// * `value` Amount of the tokens needs to redeem.
        /// * `data` An off chain data blob used to validate the redeem functionality.
        /// * `operator_data` Any data blob that defines the reason behind the force redeem.
        #[weight = T::DbWeight::get().reads_writes(6, 3) + 800_000_000]
        pub fn controller_redeem(origin, ticker: Ticker, token_holder_did: IdentityId, value: T::Balance, data: Vec<u8>, operator_data: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);
            // Granularity check
            ensure!(Self::check_granularity(&ticker, value), Error::<T>::InvalidGranularity);
            ensure!(<BalanceOf<T>>::contains_key(&ticker, &token_holder_did), Error::<T>::NotAnAssetHolder);
            let FocusedBalances {
                total: burner_balance,
                portfolio: burner_def_balance,
            } = Self::balance(&ticker, token_holder_did);
            ensure!(burner_balance >= value, Error::<T>::InsufficientBalance);
            ensure!(burner_def_balance >= value, Error::<T>::InsufficientDefaultPortfolioBalance);

            // Reduce sender's balance
            let updated_burner_def_balance = burner_def_balance
                .checked_sub(&value)
                .ok_or(Error::<T>::DefaultPortfolioBalanceUnderflow)?;
            // No check since the total balance is always >= the default
            // portfolio balance. The default portfolio balance is already checked above.
            let updated_burner_balance = burner_balance - value;

            // Decrease total supply
            let mut token = Self::token_details(&ticker);
            // No check since the total supply is always >= the default
            // portfolio balance. The default portfolio balance is already checked above.
            token.total_supply -= value;

            Self::_update_checkpoint(&ticker, token_holder_did, burner_balance);

            <BalanceOf<T>>::insert(&ticker, &token_holder_did, updated_burner_balance);
            Portfolio::<T>::set_default_portfolio_balance(token_holder_did, &ticker, updated_burner_def_balance);
            <Tokens<T>>::insert(&ticker, token);
            <statistics::Module<T>>::update_transfer_stats( &ticker, Some(updated_burner_balance), None, value);

            Self::deposit_event(RawEvent::ControllerRedemption(did, ticker, token_holder_did, value, data, operator_data));

            Ok(())
        }

        /// Makes an indivisible token divisible. Only called by the token owner.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token owner.
        /// * `ticker` Ticker of the token.
        #[weight = T::DbWeight::get().reads_writes(2, 1) + 300_000_000]
        pub fn make_divisible(origin, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            // Read the token details
            let mut token = Self::token_details(&ticker);
            ensure!(!token.divisible, Error::<T>::AssetAlreadyDivisible);
            token.divisible = true;
            <Tokens<T>>::insert(&ticker, token);
            Self::deposit_event(RawEvent::DivisibilityChanged(did, ticker, true));
            Ok(())
        }

        /// Add documents for a given token. To be called only by the token owner.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token owner.
        /// * `ticker` Ticker of the token.
        /// * `documents` Documents to be attached to `ticker`.
        ///
        /// # Weight
        /// `500_000_000 + 600_000 * documents.len()`
        #[weight = T::DbWeight::get().reads_writes(2, 1) + 500_000_000 + 600_000 * u64::try_from(documents.len()).unwrap_or_default()]
        pub fn add_documents(origin, documents: Vec<(DocumentName, Document)>, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);

            T::ProtocolFee::batch_charge_fee(ProtocolOp::AssetAddDocument, documents.len())?;

            for (document_name, document) in documents {
                <AssetDocuments>::insert(ticker, &document_name, document.clone());
                Self::deposit_event(RawEvent::DocumentAdded(ticker, document_name, document));
            }

            Ok(())
        }

        /// Remove documents for a given token. To be called only by the token owner.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token owner.
        /// * `ticker` Ticker of the token.
        /// * `doc_names` Documents to be removed from `ticker`.
        ///
        /// # Weight
        /// `500_000_000 + 600_000 * do_ids.len()`
        #[weight = T::DbWeight::get().reads_writes(2, 1) + 500_000_000 + 600_000 * u64::try_from(doc_names.len()).unwrap_or_default()]
        pub fn remove_documents(origin, doc_names: Vec<DocumentName>, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);

            for document_name in doc_names {
                <AssetDocuments>::remove(ticker, &document_name);
                Self::deposit_event(RawEvent::DocumentRemoved(ticker, document_name));
            }

            Ok(())
        }

        /// ERC-2258 Implementation

        /// Used to increase the allowance for a given custodian
        /// Any investor/token holder can add a custodian and transfer the token transfer ownership to the custodian
        /// Through that investor balance will remain the same but the given token are only transfer by the custodian.
        /// This implementation make sure to have an accurate investor count from omnibus wallets.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token holder.
        /// * `ticker` Ticker of the token.
        /// * `custodian_did` DID of the custodian (i.e whom allowance provided).
        /// * `value` Allowance amount.
        #[weight = T::DbWeight::get().reads_writes(4, 2) + 500_000_000]
        pub fn increase_custody_allowance(origin, ticker: Ticker, custodian_did: IdentityId, value: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_did = Context::current_identity_or::<Identity<T>>(&sender)?;
            Self::unsafe_increase_custody_allowance(sender_did, ticker, sender_did, custodian_did, value)?;
            Ok(())
        }

        /// Used to increase the allowance for a given custodian by providing the off chain signature.
        ///
        /// # Arguments
        /// * `origin` Secondary key of a DID who posses off chain signature.
        /// * `ticker` Ticker of the token.
        /// * `holder_did` DID of the token holder (i.e who wants to increase the custody allowance).
        /// * `holder_account_id` Secondary key which signs the off chain data blob.
        /// * `custodian_did` DID of the custodian (i.e whom allowance provided).
        /// * `value` Allowance amount.
        /// * `nonce` A u16 number which avoid the replay attack.
        /// * `signature` Signature provided by the holder_did.
        #[weight = T::DbWeight::get().reads_writes(6, 3) + 600_000_000]
        pub fn increase_custody_allowance_of(
            origin,
            ticker: Ticker,
            holder_did: IdentityId,
            holder_account_id: T::AccountId,
            custodian_did: IdentityId,
            value: T::Balance,
            nonce: u16,
            signature: T::OffChainSignature
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let caller_did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(
                !Self::authentication_nonce((ticker, holder_did, nonce)),
                Error::<T>::SignatureAlreadyUsed
            );

            let msg = SignData {
                custodian_did,
                holder_did,
                ticker,
                value,
                nonce
            };
            // holder_account_id should be a part of the holder_did
            ensure!(
                signature.verify(&msg.encode()[..], &holder_account_id),
                Error::<T>::InvalidSignature
            );
            // Validate the holder secondary key
            let holder_signer = Signatory::Account(holder_account_id);
            ensure!(
                <identity::Module<T>>::is_signer_authorized(holder_did, &holder_signer),
                Error::<T>::HolderMustBeSecondaryKeyForHolderDid
            );
            Self::unsafe_increase_custody_allowance(caller_did, ticker, holder_did, custodian_did, value)?;
            <AuthenticationNonce>::insert((ticker, holder_did, nonce), true);
            Ok(())
        }

        /// Used to transfer the tokens by the approved custodian.
        ///
        /// # Arguments
        /// * `origin` Secondary key of the custodian.
        /// * `ticker` Ticker of the token.
        /// * `holder_did` DID of the token holder (i.e whom balance get reduced).
        /// * `receiver_did` DID of the receiver.
        /// * `value` Amount of tokens need to transfer.
        #[weight = T::DbWeight::get().reads_writes(6, 3) + 600_000_000]
        pub fn transfer_by_custodian(
            origin,
            ticker: Ticker,
            holder_did: IdentityId,
            receiver_did: IdentityId,
            value: T::Balance
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let custodian_did = Context::current_identity_or::<Identity<T>>(&sender)?;

            Self::unsafe_transfer_by_custodian(custodian_did, ticker, holder_did, receiver_did, value)
        }

        /// Sets the name of the current funding round.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the token owner DID.
        /// * `ticker` - the ticker of the token.
        /// * `name` - the desired name of the current funding round.
        #[weight = T::DbWeight::get().reads_writes(2, 1) + 600_000_000]
        pub fn set_funding_round(origin, ticker: Ticker, name: FundingRoundName) ->
            DispatchResult
        {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);
            <FundingRound>::insert(ticker, name.clone());
            Self::deposit_event(RawEvent::FundingRoundSet(did, ticker, name));
            Ok(())
        }

        /// Updates the asset identifiers. Can only be called by the token owner.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the token owner.
        /// * `ticker` - the ticker of the token.
        /// * `identifiers` - the asset identifiers to be updated in the form of a vector of pairs
        ///    of `IdentifierType` and `AssetIdentifier` value.
        ///
        /// # Weight
        /// `150_000 + 20_000 * identifiers.len()`
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 700_000_000 + 20_000 * u64::try_from(identifiers.len()).unwrap_or_default()]
        pub fn update_identifiers(
            origin,
            ticker: Ticker,
            identifiers: Vec<(IdentifierType, AssetIdentifier)>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            for (typ, val) in &identifiers {
                <Identifiers>::insert((ticker, typ.clone()), val.clone());
            }
            Self::deposit_event(RawEvent::IdentifiersUpdated(did, ticker, identifiers));
            Ok(())
        }

        /// Permissioning the Smart-Extension address for a given ticker.
        ///
        /// # Arguments
        /// * `origin` - Signatory who owns to ticker/asset.
        /// * `ticker` - ticker for whom extension get added.
        /// * `extension_details` - Details of the smart extension.
        #[weight = T::DbWeight::get().reads_writes(2, 2) + 600_000_000]
        pub fn add_extension(origin, ticker: Ticker, extension_details: SmartExtension<T::AccountId>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let my_did = Context::current_identity_or::<identity::Module<T>>(&sender)?;

            ensure!(Self::is_owner(&ticker, my_did), Error::<T>::Unauthorized);

            // Verify the details of smart extension & store it
            ensure!(!<ExtensionDetails<T>>::contains_key((ticker, &extension_details.extension_id)), Error::<T>::ExtensionAlreadyPresent);

            // Ensure the hard limit on the count of maximum transfer manager an asset can have.
            Self::ensure_max_limit_for_tm_extension(&extension_details.extension_type, &ticker)?;

            <ExtensionDetails<T>>::insert((ticker, &extension_details.extension_id), extension_details.clone());
            <Extensions<T>>::mutate((ticker, &extension_details.extension_type), |ids| {
                ids.push(extension_details.extension_id.clone())
            });
            Self::deposit_event(RawEvent::ExtensionAdded(my_did, ticker, extension_details.extension_id, extension_details.extension_name, extension_details.extension_type));
            Ok(())
        }

        /// Archived the extension. Extension is use to verify the compliance or any smart logic it posses.
        ///
        /// # Arguments
        /// * `origin` - Signatory who owns the ticker/asset.
        /// * `ticker` - Ticker symbol of the asset.
        /// * `extension_id` - AccountId of the extension that need to be archived.
        #[weight = T::DbWeight::get().reads_writes(3, 1) + 800_000_000]
        pub fn archive_extension(origin, ticker: Ticker, extension_id: T::AccountId) -> DispatchResult {
            // Ensure the extrinsic is signed and have valid extension id.
            let did = Self::ensure_signed_and_validate_extension_id(origin, &ticker, &extension_id)?;

            // Mutate the extension details
            ensure!(!(<ExtensionDetails<T>>::get((ticker, &extension_id))).is_archive, Error::<T>::AlreadyArchived);
            <ExtensionDetails<T>>::mutate((ticker, &extension_id), |details| details.is_archive = true);
            Self::deposit_event(RawEvent::ExtensionArchived(did, ticker, extension_id));
            Ok(())
        }

        /// Un-archived the extension. Extension is use to verify the compliance or any smart logic it posses.
        ///
        /// # Arguments
        /// * `origin` - Signatory who owns the ticker/asset.
        /// * `ticker` - Ticker symbol of the asset.
        /// * `extension_id` - AccountId of the extension that need to be un-archived.
        #[weight = T::DbWeight::get().reads_writes(2, 2) + 800_000_000]
        pub fn unarchive_extension(origin, ticker: Ticker, extension_id: T::AccountId) -> DispatchResult {
            // Ensure the extrinsic is signed and have valid extension id.
            let did = Self::ensure_signed_and_validate_extension_id(origin, &ticker, &extension_id)?;

            // Mutate the extension details
            ensure!((<ExtensionDetails<T>>::get((ticker, &extension_id))).is_archive, Error::<T>::AlreadyUnArchived);
            <ExtensionDetails<T>>::mutate((ticker, &extension_id), |details| details.is_archive = false);
            Self::deposit_event(RawEvent::ExtensionUnArchived(did, ticker, extension_id));
            Ok(())
        }

        /// Sets the primary issuance agent to None. The caller must be the asset issuer. The asset
        /// issuer can always update the primary issuance agent using `transfer_primary_issuance_agent`. If the issuer
        /// removes their primary issuance agent then it will be immovable until either they transfer
        /// the primary issuance agent to an actual DID, or they add a claim to allow that DID to move the
        /// asset.
        ///
        /// # Arguments
        /// * `origin` - The asset issuer.
        /// * `ticker` - Ticker symbol of the asset.
        #[weight = 250_000_000]
        pub fn remove_primary_issuance_agent(
            origin,
            ticker: Ticker,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            let mut old_primary_issuance_agent = None;
            <Tokens<T>>::mutate(&ticker, |token| {
                old_primary_issuance_agent = token.primary_issuance_agent;
                token.primary_issuance_agent = None
            });
            Self::deposit_event(RawEvent::PrimaryIssuanceAgentTransfered(did, ticker, old_primary_issuance_agent, None));
            Ok(())
        }

        /// Remove the given smart extension id from the list of extension under a given ticker.
        ///
        /// # Arguments
        /// * `origin` - The asset issuer.
        /// * `ticker` - Ticker symbol of the asset.
        #[weight = 250_000_000]
        pub fn remove_smart_extension(origin, ticker: Ticker, extension_id: T::AccountId) -> DispatchResult {
            // Ensure the extrinsic is signed and have valid extension id.
            let did = Self::ensure_signed_and_validate_extension_id(origin, &ticker, &extension_id)?;

            let extension_type = Self::extension_details((&ticker, &extension_id)).extension_type;

            // Remove the storage reference for the given extension_id.
            <Extensions<T>>::mutate(&(ticker, extension_type), |extension_list| {
                if let Some(pos) = extension_list.iter().position(|ext| ext == &extension_id) {
                    extension_list.remove(pos);
                }
            });
            <ExtensionDetails<T>>::remove((&ticker, &extension_id));
            Self::deposit_event(RawEvent::ExtensionRemoved(did, ticker, extension_id));
            Ok(())
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
        #[weight = 250_000_000]
        pub fn claim_classic_ticker(origin, ticker: Ticker, ethereum_signature: ethereum::EcdsaSignature) -> DispatchResult {
            // Ensure the ticker is a classic one and fetch details.
            let ClassicTickerRegistration { eth_owner, .. } = ClassicTickers::get(ticker)
                .ok_or(Error::<T>::NoSuchClassicTicker)?;

            // Ensure ticker registration is still attached to the systematic DID.
            let sys_did = SystematicIssuers::ClassicMigration.as_id();
            match Self::is_ticker_available_or_registered_to(&ticker, sys_did) {
                TickerRegistrationStatus::RegisteredByOther => return Err(Error::<T>::TickerAlreadyRegistered.into()),
                TickerRegistrationStatus::Available => return Err(Error::<T>::TickerRegistrationExpired.into()),
                TickerRegistrationStatus::RegisteredByDid => {}
            }

            // Ensure we're signed & get did.
            let sender = ensure_signed(origin)?;
            let owner_did = Context::current_identity_or::<Identity<T>>(&sender)?;

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
            Self::deposit_event(RawEvent::ClassicTickerClaimed(owner_did, ticker, eth_signer));

            Ok(())
        }
    }
}

decl_event! {
    pub enum Event<T>
        where
        Balance = <T as CommonTrait>::Balance,
        Moment = <T as pallet_timestamp::Trait>::Moment,
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// Event for transfer of tokens.
        /// caller DID, ticker, from DID, to DID, value
        Transfer(IdentityId, Ticker, IdentityId, IdentityId, Balance),
        /// Event when an approval is made.
        /// caller DID, ticker, owner DID, spender DID, value
        Approval(IdentityId, Ticker, IdentityId, IdentityId, Balance),
        /// Emit when tokens get issued.
        /// caller DID, ticker, beneficiary DID, value, funding round, total issued in this funding round,
        /// primary issuance agent
        Issued(IdentityId, Ticker, IdentityId, Balance, FundingRoundName, Balance, Option<IdentityId>),
        /// Emit when tokens get redeemed.
        /// caller DID, ticker,  from DID, value
        Redeemed(IdentityId, Ticker, IdentityId, Balance),
        /// Event for forced transfer of tokens.
        /// caller DID/ controller DID, ticker, from DID, to DID, value, data, operator data
        ControllerTransfer(IdentityId, Ticker, IdentityId, IdentityId, Balance, Vec<u8>, Vec<u8>),
        /// Event for when a forced redemption takes place.
        /// caller DID/ controller DID, ticker, token holder DID, value, data, operator data
        ControllerRedemption(IdentityId, Ticker, IdentityId, Balance, Vec<u8>, Vec<u8>),
        /// Event for creation of the asset.
        /// caller DID/ owner DID, ticker, total supply, divisibility, asset type, beneficiary DID
        AssetCreated(IdentityId, Ticker, Balance, bool, AssetType, IdentityId),
        /// Event emitted when a token identifiers are updated.
        /// caller DID, ticker, a vector of (identifier type, identifier value)
        IdentifiersUpdated(IdentityId, Ticker, Vec<(IdentifierType, AssetIdentifier)>),
        /// Event for change in divisibility.
        /// caller DID, ticker, divisibility
        DivisibilityChanged(IdentityId, Ticker, bool),
        /// An additional event to Transfer; emitted when transfer_with_data is called.
        /// caller DID , ticker, from DID, to DID, value, data
        TransferWithData(IdentityId, Ticker, IdentityId, IdentityId, Balance, Vec<u8>),
        /// is_issuable() output
        /// ticker, return value (true if issuable)
        IsIssuable(Ticker, bool),
        /// Emit when tokens transferred by the custodian.
        /// caller DID / custodian DID , ticker, holder/from did, to did, amount
        CustodyTransfer(IdentityId, Ticker, IdentityId, IdentityId, Balance),
        /// Emit when allowance get increased.
        /// caller DID, ticker, holder did, custodian did, oldAllowance, newAllowance
        CustodyAllowanceChanged(IdentityId, Ticker, IdentityId, IdentityId, Balance, Balance),
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
        /// Emitted event for Checkpoint creation.
        /// caller DID. ticker, checkpoint count.
        CheckpointCreated(IdentityId, Ticker, u64),
        /// An event emitted when the primary issuance agent of an asset is transferred.
        /// First DID is the old primary issuance agent and the second DID is the new primary issuance agent.
        PrimaryIssuanceAgentTransfered(IdentityId, Ticker, Option<IdentityId>, Option<IdentityId>),
        /// A new document attached to an asset
        DocumentAdded(Ticker, DocumentName, Document),
        /// A document removed from an asset
        DocumentRemoved(Ticker, DocumentName),
        /// A extension got removed.
        /// caller DID, ticker, AccountId
        ExtensionRemoved(IdentityId, Ticker, AccountId),
        /// A Polymath Classic token was claimed and transferred to a non-systematic DID.
        ClassicTickerClaimed(IdentityId, Ticker, ethereum::EthereumAddress),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// DID not found.
        DIDNotFound,
        /// Not a ticker transfer auth.
        NoTickerTransferAuth,
        /// Not a primary issuance agent transfer auth.
        NoPrimaryIssuanceAgentTransferAuth,
        /// Not a token ownership transfer auth.
        NotTickerOwnershipTransferAuth,
        /// The user is not authorized.
        Unauthorized,
        /// When extension already archived.
        AlreadyArchived,
        /// When extension already un-archived.
        AlreadyUnArchived,
        /// When extension is already added.
        ExtensionAlreadyPresent,
        /// When smart extension failed to execute result.
        IncorrectResult,
        /// The sender must be a secondary key for the DID.
        HolderMustBeSecondaryKeyForHolderDid,
        /// The token has already been created.
        AssetAlreadyCreated,
        /// The ticker length is over the limit.
        TickerTooLong,
        /// The ticker is already registered to someone else.
        TickerAlreadyRegistered,
        /// The token name cannot exceed 64 bytes.
        AssetNameTooLong,
        /// An invalid total supply.
        InvalidTotalSupply,
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
        /// An underflow while calculating the balance.
        BalanceUnderflow,
        /// An underflow while calculating the default portfolio balance.
        DefaultPortfolioBalanceUnderflow,
        /// An overflow while calculating the allowance.
        AllowanceOverflow,
        /// An underflow in calculating the allowance.
        AllowanceUnderflow,
        /// An overflow in calculating the total allowance.
        TotalAllowanceOverflow,
        /// An underflow in calculating the total allowance.
        TotalAllowanceUnderflow,
        /// An overflow while calculating the checkpoint.
        CheckpointOverflow,
        /// An overflow while calculating the total supply.
        TotalSupplyOverflow,
        /// No such allowance.
        NoSuchAllowance,
        /// Insufficient allowance.
        InsufficientAllowance,
        /// The list of investors is empty.
        NoInvestors,
        /// An invalid granularity.
        InvalidGranularity,
        /// The account does not hold this token.
        NotAnAssetHolder,
        /// The asset must be frozen.
        NotFrozen,
        /// No such smart extension.
        NoSuchSmartExtension,
        /// Transfer validation check failed.
        InvalidTransfer,
        /// The sender balance is not sufficient.
        InsufficientBalance,
        /// The balance of the sender's default portfolio is not sufficient.
        InsufficientDefaultPortfolioBalance,
        /// An invalid signature.
        InvalidSignature,
        /// The signature is already in use.
        SignatureAlreadyUsed,
        /// The token is already divisible.
        AssetAlreadyDivisible,
        /// An invalid custodian DID.
        InvalidCustodianDid,
        /// Number of Transfer Manager extensions attached to an asset is equal to MaxNumberOfTMExtensionForAsset.
        MaximumTMExtensionLimitReached,
        /// An invalid Ethereum `EcdsaSignature`.
        InvalidEthereumSignature,
        /// The given ticker is not a classic one.
        NoSuchClassicTicker,
        /// Registration of ticker has expired.
        TickerRegistrationExpired,
    }
}

impl<T: Trait> AssetTrait<T::Balance, T::AccountId> for Module<T> {
    fn _mint_from_sto(
        ticker: &Ticker,
        caller: T::AccountId,
        sender: IdentityId,
        assets_purchased: T::Balance,
    ) -> DispatchResult {
        Self::_mint(ticker, caller, sender, assets_purchased, None)
    }

    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool {
        Self::_is_owner(ticker, did)
    }

    /// Get the asset `id` balance of `who`.
    fn balance(ticker: &Ticker, who: IdentityId) -> T::Balance {
        Self::balance_of(ticker, &who)
    }

    // Get the total supply of an asset `id`
    fn total_supply(ticker: &Ticker) -> T::Balance {
        Self::token_details(ticker).total_supply
    }

    fn get_balance_at(ticker: &Ticker, did: IdentityId, at: u64) -> T::Balance {
        Self::get_balance_at(*ticker, did, at)
    }

    fn unsafe_increase_custody_allowance(
        caller_did: IdentityId,
        ticker: Ticker,
        holder_did: IdentityId,
        custodian_did: IdentityId,
        value: T::Balance,
    ) -> DispatchResult {
        Self::unsafe_increase_custody_allowance(
            caller_did,
            ticker,
            holder_did,
            custodian_did,
            value,
        )
    }

    fn unsafe_decrease_custody_allowance(
        caller_did: IdentityId,
        ticker: Ticker,
        holder_did: IdentityId,
        custodian_did: IdentityId,
        value: T::Balance,
    ) {
        Self::unsafe_decrease_custody_allowance(
            caller_did,
            ticker,
            holder_did,
            custodian_did,
            value,
        )
    }

    fn unsafe_system_transfer(
        sender: IdentityId,
        ticker: &Ticker,
        from_did: IdentityId,
        to_did: IdentityId,
        value: T::Balance,
    ) {
        Self::unsafe_system_transfer(sender, ticker, from_did, to_did, value);
    }

    fn unsafe_transfer_by_custodian(
        custodian_did: IdentityId,
        ticker: Ticker,
        holder_did: IdentityId,
        receiver_did: IdentityId,
        value: T::Balance,
    ) -> DispatchResultWithPostInfo {
        Self::unsafe_transfer_by_custodian(custodian_did, ticker, holder_did, receiver_did, value)
    }

    fn primary_issuance_agent(ticker: &Ticker) -> IdentityId {
        let token_details = Self::token_details(ticker);
        token_details
            .primary_issuance_agent
            .unwrap_or(token_details.owner_did)
    }

    fn max_number_of_tm_extension() -> u32 {
        T::MaxNumberOfTMExtensionForAsset::get()
    }
}

impl<T: Trait> AcceptTransfer for Module<T> {
    fn accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult {
        Self::_accept_ticker_transfer(to_did, auth_id)
    }

    fn accept_primary_issuance_agent_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult {
        Self::_accept_primary_issuance_agent_transfer(to_did, auth_id)
    }

    fn accept_asset_ownership_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult {
        Self::_accept_token_ownership_transfer(to_did, auth_id)
    }
}

/// All functions in the decl_module macro become part of the public interface of the module
/// If they are there, they are accessible via extrinsics calls whether they are public or not
/// However, in the impl module section (this, below) the functions can be public and private
/// Private functions are internal to this module e.g.: _transfer
/// Public functions can be called from other modules e.g.: lock and unlock (being called from the tcr module)
/// All functions in the impl module section are not part of public interface because they are not part of the Call enum.
impl<T: Trait> Module<T> {
    // Public immutables
    pub fn _is_owner(ticker: &Ticker, did: IdentityId) -> bool {
        let token = Self::token_details(ticker);
        token.owner_did == did
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

    /// Before registering a ticker, do some checks, and return the expiry moment.
    fn ticker_registration_checks(
        ticker: &Ticker,
        to_did: IdentityId,
        no_re_register: bool,
        config: impl FnOnce() -> TickerRegistrationConfig<T::Moment>,
    ) -> Result<Option<T::Moment>, DispatchError> {
        ensure!(
            !<Tokens<T>>::contains_key(&ticker),
            Error::<T>::AssetAlreadyCreated
        );

        let config = config();

        // Ensure the ticker is not too long.
        ensure!(
            ticker.len() <= usize::try_from(config.max_ticker_length).unwrap_or_default(),
            Error::<T>::TickerTooLong
        );

        // Ensure that the ticker is not registered by someone else (or `to_did`, possibly).
        if match Self::is_ticker_available_or_registered_to(&ticker, to_did) {
            TickerRegistrationStatus::RegisteredByOther => true,
            TickerRegistrationStatus::RegisteredByDid => no_re_register,
            _ => false,
        } {
            return Err(Error::<T>::TickerAlreadyRegistered.into());
        }

        Ok(config
            .registration_length
            .map(|exp| <pallet_timestamp::Module<T>>::get() + exp))
    }

    /// Without charging any fees,
    /// register the given `ticker` to the `owner` identity,
    /// with the registration being removed at `expiry`.
    fn _register_ticker(ticker: &Ticker, owner: IdentityId, expiry: Option<T::Moment>) {
        if let Some(ticker_details) = Self::maybe_ticker(ticker) {
            <AssetOwnershipRelations>::remove(ticker_details.owner, ticker);
        }

        let ticker_registration = TickerRegistration { owner, expiry };

        // Store ticker registration details
        <Tickers<T>>::insert(ticker, ticker_registration);
        <AssetOwnershipRelations>::insert(owner, ticker, AssetOwnershipRelation::TickerOwned);

        // Not a classic ticker anymore if it was.
        ClassicTickers::remove(&ticker);

        Self::deposit_event(RawEvent::TickerRegistered(owner, *ticker, expiry));
    }

    /// Get the asset `ticker` balance of `did`, both total and that of the default portfolio.
    pub fn balance(ticker: &Ticker, did: IdentityId) -> FocusedBalances<T::Balance> {
        FocusedBalances {
            total: Self::balance_of(ticker, &did),
            portfolio: Portfolio::<T>::default_portfolio_balance(did, ticker),
        }
    }

    // Get the total supply of an asset `id`.
    pub fn total_supply(ticker: Ticker) -> T::Balance {
        Self::token_details(ticker).total_supply
    }

    pub fn get_balance_at(ticker: Ticker, did: IdentityId, at: u64) -> T::Balance {
        let ticker_did = (ticker, did);
        if !<TotalCheckpoints>::contains_key(ticker) ||
            at == 0 || //checkpoints start from 1
            at > Self::total_checkpoints_of(&ticker)
        {
            // No checkpoints data exist
            return Self::balance_of(&ticker, &did);
        }

        if <UserCheckpoints>::contains_key(&ticker_did) {
            let user_checkpoints = Self::user_checkpoints(&ticker_did);
            if at > *user_checkpoints.last().unwrap_or(&0) {
                // Using unwrap_or to be defensive.
                // or part should never be triggered due to the check on 2 lines above
                // User has not transacted after checkpoint creation.
                // This means their current balance = their balance at that cp.
                return Self::balance_of(&ticker, &did);
            }
            // Uses the first checkpoint that was created after target checkpoint
            // and the user has data for that checkpoint
            return Self::balance_at_checkpoint((
                ticker,
                did,
                Self::find_ceiling(&user_checkpoints, at),
            ));
        }
        // User has no checkpoint data.
        // This means that user's balance has not changed since first checkpoint was created.
        // Maybe the user never held any balance.
        Self::balance_of(&ticker, &did)
    }

    fn find_ceiling(arr: &[u64], key: u64) -> u64 {
        // This function assumes that key <= last element of the array,
        // the array consists of unique sorted elements,
        // array len > 0
        let mut end = arr.len();
        let mut start = 0;
        let mut mid = (start + end) / 2;

        while mid != 0 && end >= start {
            // Due to our assumptions, we can even remove end >= start condition from here
            if key > arr[mid - 1] && key <= arr[mid] {
                // This condition and the fact that key <= last element of the array mean that
                // start should never become greater than end.
                return arr[mid];
            } else if key > arr[mid] {
                start = mid + 1;
            } else {
                end = mid;
            }
            mid = (start + end) / 2;
        }

        // This should only be reached when mid becomes 0.
        arr[0]
    }

    pub fn _is_valid_transfer(
        ticker: &Ticker,
        extension_caller: T::AccountId,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        value: T::Balance,
    ) -> StdResult<(u8, Weight), DispatchError> {
        if Self::frozen(ticker) {
            return Ok((ERC1400_TRANSFERS_HALTED, T::DbWeight::get().reads(1)));
        }
        let primary_issuance_agent = <Tokens<T>>::get(ticker).primary_issuance_agent;
        let (status_code, weight_for_transfer) = T::ComplianceManager::verify_restriction(
            ticker,
            from_did,
            to_did,
            value,
            primary_issuance_agent,
        )?;
        Ok(if status_code != ERC1400_TRANSFER_SUCCESS {
            (COMPLIANCE_MANAGER_FAILURE, weight_for_transfer)
        } else {
            let mut result = true;
            let mut is_valid = false;
            let mut is_invalid = false;
            let mut force_valid = false;
            let current_holder_count = <statistics::Module<T>>::investor_count_per_asset(ticker);
            let tms = Self::extensions((ticker, SmartExtensionType::TransferManager))
                .into_iter()
                .filter(|tm| !Self::extension_details((ticker, tm)).is_archive)
                .collect::<Vec<T::AccountId>>();
            let tm_count = u32::try_from(tms.len()).unwrap_or_default();
            if !tms.is_empty() {
                for tm in tms.into_iter() {
                    let result = Self::verify_restriction(
                        ticker,
                        extension_caller.clone(),
                        from_did,
                        to_did,
                        value,
                        current_holder_count,
                        tm,
                    );
                    match result {
                        RestrictionResult::Valid => is_valid = true,
                        RestrictionResult::Invalid => is_invalid = true,
                        RestrictionResult::ForceValid => force_valid = true,
                    }
                }
                //is_valid = force_valid ? true : (is_invalid ? false : is_valid);
                result = force_valid || !is_invalid && is_valid;
            }
            // Compute the result for transfer
            Self::compute_transfer_result(result, tm_count, weight_for_transfer)
        })
    }

    // Transfers tokens from one identity to another
    pub fn unsafe_transfer(
        sender: IdentityId,
        ticker: &Ticker,
        from_did: IdentityId,
        to_did: IdentityId,
        value: T::Balance,
    ) -> DispatchResult {
        // Granularity check
        ensure!(
            Self::check_granularity(ticker, value),
            Error::<T>::InvalidGranularity
        );
        ensure!(
            <BalanceOf<T>>::contains_key(ticker, &from_did),
            Error::<T>::NotAnAssetHolder
        );
        ensure!(from_did != to_did, Error::<T>::InvalidTransfer);
        let FocusedBalances {
            total: from_total_balance,
            portfolio: from_def_balance,
        } = Self::balance(ticker, from_did);
        ensure!(from_total_balance >= value, Error::<T>::InsufficientBalance);
        ensure!(
            from_def_balance >= value,
            Error::<T>::InsufficientDefaultPortfolioBalance
        );
        let updated_from_def_balance = from_def_balance
            .checked_sub(&value)
            .ok_or(Error::<T>::DefaultPortfolioBalanceUnderflow)?;
        // No check since the total balance is always >= the default
        // portfolio balance. The default portfolio balance is already checked above.
        let updated_from_total_balance = from_total_balance - value;

        let FocusedBalances {
            total: to_total_balance,
            portfolio: to_def_balance,
        } = Self::balance(ticker, to_did);
        let updated_to_total_balance = to_total_balance
            .checked_add(&value)
            .ok_or(Error::<T>::BalanceOverflow)?;
        // No check since the default portfolio balance is always <= the total
        // balance. The total balance is already checked above.
        let updated_to_def_balance = to_def_balance + value;

        Self::_update_checkpoint(ticker, from_did, from_total_balance);
        Self::_update_checkpoint(ticker, to_did, to_total_balance);
        // reduce sender's balance
        <BalanceOf<T>>::insert(ticker, &from_did, updated_from_total_balance);
        Portfolio::<T>::set_default_portfolio_balance(from_did, ticker, updated_from_def_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert(ticker, &to_did, updated_to_total_balance);
        Portfolio::<T>::set_default_portfolio_balance(to_did, ticker, updated_to_def_balance);

        // Update statistic info.
        <statistics::Module<T>>::update_transfer_stats(
            ticker,
            Some(updated_from_total_balance),
            Some(updated_to_total_balance),
            value,
        );

        Self::deposit_event(RawEvent::Transfer(sender, *ticker, from_did, to_did, value));
        Ok(())
    }

    pub fn _create_checkpoint(ticker: &Ticker) -> DispatchResult {
        if <TotalCheckpoints>::contains_key(ticker) {
            let mut checkpoint_count = Self::total_checkpoints_of(ticker);
            checkpoint_count = checkpoint_count
                .checked_add(1)
                .ok_or(Error::<T>::CheckpointOverflow)?;
            <TotalCheckpoints>::insert(ticker, checkpoint_count);
            <CheckpointTotalSupply<T>>::insert(
                &(*ticker, checkpoint_count),
                Self::token_details(ticker).total_supply,
            );
        } else {
            <TotalCheckpoints>::insert(ticker, 1);
            <CheckpointTotalSupply<T>>::insert(
                &(*ticker, 1),
                Self::token_details(ticker).total_supply,
            );
        }
        Ok(())
    }

    fn _update_checkpoint(ticker: &Ticker, user_did: IdentityId, user_balance: T::Balance) {
        if <TotalCheckpoints>::contains_key(ticker) {
            let checkpoint_count = Self::total_checkpoints_of(ticker);
            let ticker_user_did_checkpont = (*ticker, user_did, checkpoint_count);
            if !<CheckpointBalance<T>>::contains_key(&ticker_user_did_checkpont) {
                <CheckpointBalance<T>>::insert(&ticker_user_did_checkpont, user_balance);
                <UserCheckpoints>::mutate(&(*ticker, user_did), |user_checkpoints| {
                    user_checkpoints.push(checkpoint_count);
                });
            }
        }
    }

    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool {
        Self::_is_owner(ticker, did)
    }

    pub fn _mint(
        ticker: &Ticker,
        caller: T::AccountId,
        to_did: IdentityId,
        value: T::Balance,
        protocol_fee_data: Option<ProtocolOp>,
    ) -> DispatchResult {
        // Granularity check
        ensure!(
            Self::check_granularity(ticker, value),
            Error::<T>::InvalidGranularity
        );
        // Read the token details
        let mut token = Self::token_details(ticker);
        // Prepare the updated total supply.
        let updated_total_supply = token
            .total_supply
            .checked_add(&value)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;
        ensure!(
            updated_total_supply <= MAX_SUPPLY.into(),
            Error::<T>::TotalSupplyAboveLimit
        );
        //Increase receiver balance
        let FocusedBalances {
            total: current_to_balance,
            portfolio: current_to_def_balance,
        } = Self::balance(ticker, to_did);
        // No check since the total balance is always <= the total supply. The
        // total supply is already checked above.
        let updated_to_balance = current_to_balance + value;
        // No check since the default portfolio balance is always <= the total
        // supply. The total supply is already checked above.
        let updated_to_def_balance = current_to_def_balance + value;

        // Charge the given fee.
        if let Some(op) = protocol_fee_data {
            T::ProtocolFee::charge_fee(op)?;
        }
        Self::_update_checkpoint(ticker, to_did, current_to_balance);

        // Increase total supply
        token.total_supply = updated_total_supply;
        <BalanceOf<T>>::insert(ticker, &to_did, updated_to_balance);
        Portfolio::<T>::set_default_portfolio_balance(to_did, ticker, updated_to_def_balance);
        let primary_issuance_agent = token.primary_issuance_agent;
        <Tokens<T>>::insert(ticker, token);

        // Update the investor count of an asset.
        <statistics::Module<T>>::update_transfer_stats(
            &ticker,
            None,
            Some(updated_to_balance),
            value,
        );

        let round = Self::funding_round(ticker);
        let ticker_round = (*ticker, round.clone());
        // No check since the issued balance is always <= the total
        // supply. The total supply is already checked above.
        let issued_in_this_round = Self::issued_in_funding_round(&ticker_round) + value;
        <IssuedInFundingRound<T>>::insert(&ticker_round, issued_in_this_round);
        Self::deposit_event(RawEvent::Transfer(
            Context::current_identity_or::<Identity<T>>(&caller)?,
            *ticker,
            IdentityId::default(),
            to_did,
            value,
        ));
        Self::deposit_event(RawEvent::Issued(
            Context::current_identity_or::<Identity<T>>(&caller)?,
            *ticker,
            to_did,
            value,
            round,
            issued_in_this_round,
            primary_issuance_agent,
        ));

        Ok(())
    }

    fn check_granularity(ticker: &Ticker, value: T::Balance) -> bool {
        // Read the token details
        let token = Self::token_details(ticker);
        token.divisible || value % ONE_UNIT.into() == 0.into()
    }

    fn _check_custody_allowance(
        ticker: &Ticker,
        holder_did: IdentityId,
        value: T::Balance,
    ) -> DispatchResult {
        let remaining_balance = Self::balance_of(&ticker, &holder_did)
            .checked_sub(&value)
            .ok_or(Error::<T>::BalanceUnderflow)?;
        ensure!(
            remaining_balance >= Self::total_custody_allowance(&(*ticker, holder_did)),
            Error::<T>::InsufficientBalance
        );
        Ok(())
    }

    fn unsafe_increase_custody_allowance(
        caller_did: IdentityId,
        ticker: Ticker,
        holder_did: IdentityId,
        custodian_did: IdentityId,
        value: T::Balance,
    ) -> DispatchResult {
        let new_custody_allowance = Self::total_custody_allowance((ticker, holder_did))
            .checked_add(&value)
            .ok_or(Error::<T>::TotalAllowanceOverflow)?;
        // Ensure that balance of the token holder is >= the total custody allowance + value
        ensure!(
            Self::balance_of(&ticker, &holder_did) >= new_custody_allowance,
            Error::<T>::InsufficientBalance
        );
        // Ensure the valid DID
        ensure!(
            <identity::DidRecords<T>>::contains_key(custodian_did),
            Error::<T>::InvalidCustodianDid
        );

        let old_allowance = Self::custodian_allowance((ticker, holder_did, custodian_did));
        let new_current_allowance = old_allowance
            .checked_add(&value)
            .ok_or(Error::<T>::AllowanceOverflow)?;
        // Update Storage
        <CustodianAllowance<T>>::insert(
            (ticker, holder_did, custodian_did),
            &new_current_allowance,
        );
        <TotalCustodyAllowance<T>>::insert((ticker, holder_did), new_custody_allowance);
        Self::deposit_event(RawEvent::CustodyAllowanceChanged(
            caller_did,
            ticker,
            holder_did,
            custodian_did,
            old_allowance,
            new_current_allowance,
        ));
        Ok(())
    }

    fn unsafe_decrease_custody_allowance(
        caller_did: IdentityId,
        ticker: Ticker,
        holder_did: IdentityId,
        custodian_did: IdentityId,
        value: T::Balance,
    ) {
        let new_custody_allowance =
            Self::total_custody_allowance((ticker, holder_did)).saturating_sub(value);

        let old_allowance = Self::custodian_allowance((ticker, holder_did, custodian_did));
        let new_current_allowance = old_allowance.saturating_sub(value);

        // Update Storage
        <CustodianAllowance<T>>::insert(
            (ticker, holder_did, custodian_did),
            &new_current_allowance,
        );
        <TotalCustodyAllowance<T>>::insert((ticker, holder_did), new_custody_allowance);
        Self::deposit_event(RawEvent::CustodyAllowanceChanged(
            caller_did,
            ticker,
            holder_did,
            custodian_did,
            old_allowance,
            new_current_allowance,
        ));
    }

    /// Accept and process a ticker transfer.
    pub fn _accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult {
        ensure!(
            <identity::Authorizations<T>>::contains_key(Signatory::from(to_did), auth_id),
            AuthorizationError::Invalid
        );

        let auth = <identity::Authorizations<T>>::get(Signatory::from(to_did), auth_id);

        let ticker = match auth.authorization_data {
            AuthorizationData::TransferTicker(ticker) => ticker,
            _ => return Err(Error::<T>::NoTickerTransferAuth.into()),
        };

        ensure!(
            !<Tokens<T>>::contains_key(&ticker),
            Error::<T>::AssetAlreadyCreated
        );
        let ticker_details = Self::ticker_registration(&ticker);

        <identity::Module<T>>::consume_auth(
            ticker_details.owner,
            Signatory::from(to_did),
            auth_id,
        )?;

        Self::transfer_ticker(ticker, to_did, ticker_details.owner);
        ClassicTickers::remove(&ticker); // Not a classic ticker anymore if it was.
        Ok(())
    }

    /// Transfer the given `ticker`'s registration from `from` to `to`.
    fn transfer_ticker(ticker: Ticker, to: IdentityId, from: IdentityId) {
        <AssetOwnershipRelations>::remove(from, ticker);
        <AssetOwnershipRelations>::insert(to, ticker, AssetOwnershipRelation::TickerOwned);
        <Tickers<T>>::mutate(&ticker, |tr| tr.owner = to);
        Self::deposit_event(RawEvent::TickerTransferred(to, ticker, from));
    }

    /// Accept and process a primary issuance agent transfer.
    pub fn _accept_primary_issuance_agent_transfer(
        to_did: IdentityId,
        auth_id: u64,
    ) -> DispatchResult {
        ensure!(
            <identity::Authorizations<T>>::contains_key(Signatory::from(to_did), auth_id),
            AuthorizationError::Invalid
        );

        let auth = <identity::Authorizations<T>>::get(Signatory::from(to_did), auth_id);

        let ticker = match auth.authorization_data {
            AuthorizationData::TransferPrimaryIssuanceAgent(ticker) => ticker,
            _ => return Err(Error::<T>::NoPrimaryIssuanceAgentTransferAuth.into()),
        };

        let token = <Tokens<T>>::get(&ticker);
        <identity::Module<T>>::consume_auth(token.owner_did, Signatory::from(to_did), auth_id)?;

        let mut old_primary_issuance_agent = None;
        <Tokens<T>>::mutate(&ticker, |token| {
            old_primary_issuance_agent = token.primary_issuance_agent;
            token.primary_issuance_agent = Some(to_did);
        });

        Self::deposit_event(RawEvent::PrimaryIssuanceAgentTransfered(
            to_did,
            ticker,
            old_primary_issuance_agent,
            Some(to_did),
        ));

        Ok(())
    }

    /// Accept and process a token ownership transfer.
    pub fn _accept_token_ownership_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult {
        ensure!(
            <identity::Authorizations<T>>::contains_key(Signatory::from(to_did), auth_id),
            AuthorizationError::Invalid
        );

        let auth = <identity::Authorizations<T>>::get(Signatory::from(to_did), auth_id);

        let ticker = match auth.authorization_data {
            AuthorizationData::TransferAssetOwnership(ticker) => ticker,
            _ => return Err(Error::<T>::NotTickerOwnershipTransferAuth.into()),
        };

        ensure!(<Tokens<T>>::contains_key(&ticker), Error::<T>::NoSuchAsset);

        let token_details = Self::token_details(&ticker);
        let ticker_details = Self::ticker_registration(&ticker);

        <identity::Module<T>>::consume_auth(
            token_details.owner_did,
            Signatory::from(to_did),
            auth_id,
        )?;

        <AssetOwnershipRelations>::remove(ticker_details.owner, ticker);

        <AssetOwnershipRelations>::insert(to_did, ticker, AssetOwnershipRelation::AssetOwned);

        <Tickers<T>>::mutate(&ticker, |tr| {
            tr.owner = to_did;
        });
        <Tokens<T>>::mutate(&ticker, |tr| {
            tr.owner_did = to_did;
        });

        Self::deposit_event(RawEvent::AssetOwnershipTransferred(
            to_did,
            ticker,
            token_details.owner_did,
        ));

        Ok(())
    }

    pub fn verify_restriction(
        ticker: &Ticker,
        extension_caller: T::AccountId,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        value: T::Balance,
        holder_count: Counter,
        dest: T::AccountId,
    ) -> RestrictionResult {
        // 4 byte selector of verify_transfer - 0xD9386E41
        let selector = hex!("D9386E41");
        let balance_to = match to_did {
            Some(did) => T::Balance::encode(&<BalanceOf<T>>::get(ticker, &did)),
            None => T::Balance::encode(&(0.into())),
        };
        let balance_from = match from_did {
            Some(did) => T::Balance::encode(&<BalanceOf<T>>::get(ticker, &did)),
            None => T::Balance::encode(&(0.into())),
        };
        let encoded_to = Option::<IdentityId>::encode(&to_did);
        let encoded_from = Option::<IdentityId>::encode(&from_did);
        let encoded_value = T::Balance::encode(&value);
        let total_supply = T::Balance::encode(&<Tokens<T>>::get(&ticker).total_supply);
        let current_holder_count = Counter::encode(&holder_count);

        // Creation of the encoded data for the verifyTransfer function of the extension
        // i.e fn verify_transfer(
        //        from: Option<IdentityId>,
        //        to: Option<IdentityId>,
        //        value: Balance,
        //        balance_from: Balance,
        //        balance_to: Balance,
        //        total_supply: Balance,
        //        current_holder_count: Counter
        //    ) -> RestrictionResult { }

        let encoded_data = [
            &selector[..],
            &encoded_from[..],
            &encoded_to[..],
            &encoded_value[..],
            &balance_from[..],
            &balance_to[..],
            &total_supply[..],
            &current_holder_count[..],
        ]
        .concat();

        // Calling extension to verify the compliance requirement
        // native currency value should be `0` as no funds need to transfer to the smart extension
        // We are passing arbitrary high `gas_limit` value to make sure extension's function execute successfully
        // TODO: Once gas estimate function will be introduced, arbitrary gas value will be replaced by the estimated gas
        let is_allowed =
            Self::call_extension(extension_caller, dest, 0.into(), GAS_LIMIT, encoded_data);
        if is_allowed.is_success() {
            if let Ok(allowed) = RestrictionResult::decode(&mut &is_allowed.data[..]) {
                return allowed;
            }
        }
        RestrictionResult::Invalid
    }

    /// A helper function that is used to call the smart extension function.
    ///
    /// # Arguments
    /// * `from` - Caller of the extension.
    /// * `dest` - Address/AccountId of the smart extension whom get called.
    /// * `value` - Amount of native currency that need to transfer to the extension.
    /// * `gas_limit` - Maximum amount of gas passed to successfully execute the function.
    /// * `data` - Encoded data that contains function selector and function arguments values.
    pub fn call_extension(
        from: T::AccountId,
        dest: T::AccountId,
        _value: T::Balance,
        gas_limit: Gas,
        data: Vec<u8>,
    ) -> ExecReturnValue {
        // TODO: Fix the value conversion into Currency
        match <pallet_contracts::Module<T>>::bare_call(from, dest, 0.into(), gas_limit, data) {
            Ok(encoded_value) => encoded_value,
            Err(err) => {
                let reason: &'static str = err.reason.into();
                // status 0 is used for extension call successfully executed
                ExecReturnValue {
                    status: 1,
                    data: reason.as_bytes().to_vec(),
                }
            }
        }
    }

    /// RPC: Function allows external users to know wether the transfer extrinsic
    /// will be valid or not beforehand.
    pub fn unsafe_can_transfer(
        sender: T::AccountId,
        ticker: Ticker,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        amount: T::Balance,
    ) -> StdResult<u8, &'static str> {
        // Granularity check
        if !Self::check_granularity(&ticker, amount) {
            return Ok(INVALID_GRANULARITY);
        }
        if from_did == to_did {
            return Ok(INVALID_RECEIVER_DID);
        }
        // Non-Issuance case check
        if let Some(from_id) = from_did {
            if Identity::<T>::has_valid_cdd(from_id) {
                let FocusedBalances {
                    total: balance,
                    portfolio: def_balance,
                } = Self::balance(&ticker, from_id);
                if balance < amount
                    || def_balance < amount
                    || balance - amount < Self::total_custody_allowance((ticker, from_id))
                {
                    return Ok(ERC1400_INSUFFICIENT_BALANCE);
                }
            } else {
                return Ok(INVALID_SENDER_DID);
            }
        }
        // Non-Redeem case check
        if let Some(to_id) = to_did {
            if !Identity::<T>::has_valid_cdd(to_id) {
                return Ok(INVALID_RECEIVER_DID);
            }
        }
        // Compliance manager & Smart Extension check
        Ok(
            Self::_is_valid_transfer(&ticker, sender, from_did, to_did, amount)
                .map(|(status, _)| status)
                .unwrap_or(ERC1400_TRANSFER_FAILURE),
        )
    }

    /// Transfers an asset using custodial allowance
    fn unsafe_transfer_by_custodian(
        custodian_did: IdentityId,
        ticker: Ticker,
        holder_did: IdentityId,
        receiver_did: IdentityId,
        value: T::Balance,
    ) -> DispatchResultWithPostInfo {
        let mut custodian_allowance =
            Self::custodian_allowance((ticker, holder_did, custodian_did));
        // using checked_sub (safe math) to avoid underflow
        custodian_allowance = custodian_allowance
            .checked_sub(&value)
            .ok_or(Error::<T>::AllowanceUnderflow)?;
        // using checked_sub (safe math) to avoid underflow
        let new_total_allowance = Self::total_custody_allowance((ticker, holder_did))
            .checked_sub(&value)
            .ok_or(Error::<T>::TotalAllowanceUnderflow)?;
        // Validate the transfer
        let (is_transfer_success, weight_for_transfer) = Self::_is_valid_transfer(
            &ticker,
            <identity::Module<T>>::did_records(custodian_did).primary_key,
            Some(holder_did),
            Some(receiver_did),
            value,
        )?;
        ensure!(
            is_transfer_success == ERC1400_TRANSFER_SUCCESS,
            Error::<T>::InvalidTransfer
        );
        Self::unsafe_transfer(custodian_did, &ticker, holder_did, receiver_did, value)?;
        // Update Storage of allowance
        <CustodianAllowance<T>>::insert((ticker, holder_did, custodian_did), &custodian_allowance);
        <TotalCustodyAllowance<T>>::insert((ticker, holder_did), new_total_allowance);
        Self::deposit_event(RawEvent::CustodyTransfer(
            custodian_did,
            ticker,
            holder_did,
            receiver_did,
            value,
        ));
        Ok(
            Some(weight_for::weight_for_unsafe_transfer_by_custodian::<T>(
                weight_for_transfer,
            ))
            .into(),
        )
    }

    /// Internal function to process a transfer without any checks.
    /// Used for reverting failed settlements
    fn unsafe_system_transfer(
        sender: IdentityId,
        ticker: &Ticker,
        from_did: IdentityId,
        to_did: IdentityId,
        value: T::Balance,
    ) {
        let FocusedBalances {
            total: from_balance,
            portfolio: from_def_balance,
        } = Self::balance(ticker, from_did);
        let updated_from_balance = from_balance.saturating_sub(value);
        let updated_from_def_balance = from_def_balance.saturating_sub(value);
        let FocusedBalances {
            total: to_balance,
            portfolio: to_def_balance,
        } = Self::balance(ticker, to_did);
        let updated_to_balance = to_balance.saturating_add(value);
        let updated_to_def_balance = to_def_balance.saturating_add(value);

        Self::_update_checkpoint(ticker, from_did, from_balance);
        Self::_update_checkpoint(ticker, to_did, to_balance);

        // reduce sender's balance
        <BalanceOf<T>>::insert(ticker, &from_did, updated_from_balance);
        Portfolio::<T>::set_default_portfolio_balance(from_did, ticker, updated_from_def_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert(ticker, &to_did, updated_to_balance);
        Portfolio::<T>::set_default_portfolio_balance(to_did, ticker, updated_to_def_balance);

        // Update statistic info.
        <statistics::Module<T>>::update_transfer_stats(
            ticker,
            Some(updated_from_balance),
            Some(updated_to_balance),
            value,
        );

        Self::deposit_event(RawEvent::Transfer(sender, *ticker, from_did, to_did, value));
    }

    /// Performs necessary checks on parameters of `create_asset`.
    fn ensure_create_asset_parameters(
        ticker: &Ticker,
        name: &AssetName,
        total_supply: T::Balance,
    ) -> DispatchResult {
        // Ensure that the ticker is new.
        ensure!(
            !<Tokens<T>>::contains_key(&ticker),
            Error::<T>::AssetAlreadyCreated
        );
        let ticker_config = Self::ticker_registration_config();
        // Limit the ticker length.
        ensure!(
            ticker.len() <= usize::try_from(ticker_config.max_ticker_length).unwrap_or_default(),
            Error::<T>::TickerTooLong
        );
        // Check the name length.
        // TODO: Limit the maximum size of a name.
        ensure!(name.as_slice().len() <= 64, Error::<T>::AssetNameTooLong);
        // Limit the total supply.
        ensure!(
            total_supply <= MAX_SUPPLY.into(),
            Error::<T>::TotalSupplyAboveLimit
        );
        Ok(())
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

    /// Compute the result of the transfer
    pub fn compute_transfer_result(
        final_result: bool,
        tm_count: u32,
        cm_result: Weight,
    ) -> (u8, Weight) {
        let weight_for_valid_transfer =
            weight_for::weight_for_is_valid_transfer::<T>(tm_count, cm_result);
        let transfer_status = match final_result {
            true => ERC1400_TRANSFER_SUCCESS,
            false => SMART_EXTENSION_FAILURE,
        };
        (transfer_status, weight_for_valid_transfer)
    }

    /// Ensure the extrinsic is signed and have valid extension id.
    fn ensure_signed_and_validate_extension_id(
        origin: T::Origin,
        ticker: &Ticker,
        id: &T::AccountId,
    ) -> Result<IdentityId, DispatchError> {
        let sender = ensure_signed(origin)?;
        let did = Context::current_identity_or::<identity::Module<T>>(&sender)?;

        ensure!(Self::is_owner(ticker, did), Error::<T>::Unauthorized);
        ensure!(
            <ExtensionDetails<T>>::contains_key((ticker, id)),
            Error::<T>::NoSuchSmartExtension
        );
        Ok(did)
    }
}
