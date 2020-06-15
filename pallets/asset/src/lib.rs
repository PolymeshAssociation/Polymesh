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
//! - `transfer` - Transfer tokens from one DID to another DID as tokens are stored/managed on the DID level.
//! - `controller_transfer` - Forces a transfer between two DID.
//! - `approve` - Approve token transfer from one DID to another.
//! - `transfer_from` - If sufficient allowance provided, transfer from a DID to another DID without token owner's signature.
//! - `create_checkpoint` - Function used to create the checkpoint.
//! - `issue` - Function is used to issue(or mint) new tokens for the given DID.
//! - `batch_issue` - Batch version of issue function.
//! - `redeem` - Used to redeem the security tokens.
//! - `redeem_from` - Used to redeem the security tokens by some other DID who has approval.
//! - `controller_redeem` - Forces a redemption of an DID's tokens. Can only be called by token owner.
//! - `make_divisible` - Change the divisibility of the token to divisible. Only called by the token owner.
//! - `can_transfer` - Checks whether a transaction with given parameters can take place or not.
//! - `transfer_with_data` - This function can be used by the exchanges of other third parties to dynamically validate the transaction by passing the data blob.
//! - `transfer_from_with_data` - This function can be used by the exchanges of other third parties to dynamically validate the transaction by passing the data blob.
//! - `is_issuable` - Used to know whether the given token will issue new tokens or not.
//! - `batch_add_document` - Add documents for a given token, Only be called by the token owner.
//! - `batch_remove_document` - Remove documents for a given token, Only be called by the token owner.
//! - `batch_update_document` - Update documents for the given token, Only be called by the token owner.
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
#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use pallet_identity as identity;
use pallet_statistics as statistics;
use polymesh_common_utilities::{
    asset::{AcceptTransfer, IssueAssetItem, Trait as AssetTrait},
    balances::Trait as BalancesTrait,
    compliance_manager::Trait as ComplianceManagerTrait,
    constants::*,
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    CommonTrait, Context,
};
use polymesh_primitives::{
    AccountKey, AuthorizationData, AuthorizationError, Document, IdentityId, LinkData, Signatory,
    SmartExtension, SmartExtensionName, SmartExtensionType, Ticker,
};
use polymesh_primitives_derive::VecU8StrongTyped;

use codec::{Decode, Encode};
use core::result::Result as StdResult;
use currency::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::Currency,
    weights::{DispatchClass, FunctionOf, SimpleDispatchInfo},
};
use frame_system::{self as system, ensure_signed};
use hex_literal::hex;
use pallet_contracts::{ExecReturnValue, Gas};
use sp_runtime::traits::{CheckedAdd, CheckedSub, Verify};

#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::TryFrom, prelude::*};

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + BalancesTrait
    + IdentityTrait
    + pallet_session::Trait
    + statistics::Trait
    + pallet_contracts::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type ComplianceManager: ComplianceManagerTrait<Self::Balance>;
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
}

impl Default for IdentifierType {
    fn default() -> Self {
        IdentifierType::Isin
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
    pub link_id: u64,
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
    pub link_id: u64,
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
        /// Used to store the securityToken balance corresponds to ticker and Identity.
        /// (ticker, DID) -> Balance
        pub BalanceOf get(fn balance_of): double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) IdentityId => T::Balance;
        /// A map of pairs of a ticker name and an `IdentifierType` to asset identifiers.
        pub Identifiers get(fn identifiers): map hasher(blake2_128_concat) (Ticker, IdentifierType) => AssetIdentifier;
        /// (ticker, sender (DID), spender(DID)) -> allowance amount
        Allowance get(fn allowance): map hasher(blake2_128_concat) (Ticker, IdentityId, IdentityId) => T::Balance;
        /// Checkpoints created per token.
        /// (ticker) -> no. of checkpoints
        pub TotalCheckpoints get(fn total_checkpoints_of): map hasher(blake2_128_concat) Ticker => u64;
        /// Total supply of the token at the checkpoint.
        /// (ticker, checkpointId) -> total supply at given checkpoint
        pub CheckpointTotalSupply get(fn total_supply_at): map hasher(blake2_128_concat) (Ticker, u64) => T::Balance;
        /// Balance of a DID at a checkpoint.
        /// (ticker, DID, checkpoint ID) -> Balance of a DID at a checkpoint
        CheckpointBalance get(fn balance_at_checkpoint): map hasher(blake2_128_concat) (Ticker, IdentityId, u64) => T::Balance;
        /// Last checkpoint updated for a DID's balance.
        /// (ticker, DID) -> List of checkpoints where user balance changed
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
        /// * `origin` It contains the signing key of the caller (i.e who signed the transaction to execute this function).
        /// * `ticker` ticker to register.
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        pub fn register_ticker(origin, ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from((ensure_signed(origin)?).encode())?;
            let signer = Signatory::AccountKey(sender_key);
            let to_did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(!<Tokens<T>>::contains_key(&ticker), Error::<T>::AssetAlreadyCreated);

            let ticker_config = Self::ticker_registration_config();

            ensure!(
                ticker.len() <= usize::try_from(ticker_config.max_ticker_length).unwrap_or_default(),
                Error::<T>::TickerTooLong
            );

            // Ensure that the ticker is not registered by someone else
            ensure!(
                Self::is_ticker_available_or_registered_to(&ticker, to_did) != TickerRegistrationStatus::RegisteredByOther,
                Error::<T>::TickerAlreadyRegistered
            );

            let now = <pallet_timestamp::Module<T>>::get();
            let expiry = if let Some(exp) = ticker_config.registration_length { Some(now + exp) } else { None };

            Self::_register_ticker(&ticker, &signer, to_did, expiry)
        }

        /// This function is used to accept a ticker transfer.
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the signing key of the caller (i.e who signed the transaction to execute this function).
        /// * `auth_id` Authorization ID of ticker transfer authorization.
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        pub fn accept_ticker_transfer(origin, auth_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let to_did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            Self::_accept_ticker_transfer(to_did, auth_id)
        }

        /// This function is used to accept a token ownership transfer.
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the signing key of the caller (i.e who signed the transaction to execute this function).
        /// * `auth_id` Authorization ID of the token ownership transfer authorization.
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        pub fn accept_asset_ownership_transfer(origin, auth_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let to_did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            Self::_accept_token_ownership_transfer(to_did, auth_id)
        }

        /// Initializes a new security token
        /// makes the initiating account the owner of the security token
        /// & the balance of the owner is set to total supply.
        ///
        /// # Arguments
        /// * `origin` - contains the signing key of the caller (i.e who signed the transaction to execute this function).
        /// * `name` - the name of the token.
        /// * `ticker` - the ticker symbol of the token.
        /// * `total_supply` - the total supply of the token.
        /// * `divisible` - a boolean to identify the divisibility status of the token.
        /// * `asset_type` - the asset type.
        /// * `identifiers` - a vector of asset identifiers.
        /// * `funding_round` - name of the funding round.
        ///
        /// # Weight
        /// `400_000 + 20_000 * identifiers.len()`
        #[weight = FunctionOf(
            |(_, _, _, _, _, identifiers, _): (
                &AssetName,
                &Ticker,
                &T::Balance,
                &bool,
                &AssetType,
                &Vec<(IdentifierType, AssetIdentifier)>,
                &Option<FundingRoundName>
            )| {
                400_000 + 20_000 * u32::try_from(identifiers.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn create_asset(
            origin,
            name: AssetName,
            ticker: Ticker,
            total_supply: T::Balance,
            divisible: bool,
            asset_type: AssetType,
            identifiers: Vec<(IdentifierType, AssetIdentifier)>,
            funding_round: Option<FundingRoundName>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from((ensure_signed(origin)?).encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let signer = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(!<Tokens<T>>::contains_key(&ticker), Error::<T>::AssetAlreadyCreated);

            let ticker_config = Self::ticker_registration_config();

            ensure!(
                ticker.len() <= usize::try_from(ticker_config.max_ticker_length).unwrap_or_default(),
                Error::<T>::TickerTooLong
            );

            // checking max size for name and ticker
            // byte arrays (Vec) with no max size should be avoided
            ensure!(name.as_slice().len() <= 64, Error::<T>::AssetNameTooLong);

            let is_ticker_available_or_registered_to = Self::is_ticker_available_or_registered_to(&ticker, did);

            ensure!(
                is_ticker_available_or_registered_to != TickerRegistrationStatus::RegisteredByOther,
                Error::<T>::TickerAlreadyRegistered
            );

            if !divisible {
                ensure!(total_supply % ONE_UNIT.into() == 0.into(), Error::<T>::InvalidTotalSupply);
            }

            ensure!(total_supply <= MAX_SUPPLY.into(), Error::<T>::TotalSupplyAboveLimit);

            <<T as IdentityTrait>::ProtocolFee>::charge_fee(
                &signer,
                ProtocolOp::AssetCreateAsset,
            )?;
            <identity::Module<T>>::register_asset_did(&ticker)?;

            if is_ticker_available_or_registered_to == TickerRegistrationStatus::Available {
                // ticker not registered by anyone (or registry expired). we can charge fee and register this ticker
                Self::_register_ticker(&ticker, &signer, did, None)?;
            } else {
                // Ticker already registered by the user
                <Tickers<T>>::mutate(&ticker, |tr| tr.expiry = None);
            }

            let link = <identity::Module<T>>::add_link(Signatory::from(did), LinkData::AssetOwned(ticker), None);

            let token = SecurityToken {
                name,
                total_supply,
                owner_did: did,
                divisible,
                asset_type: asset_type.clone(),
                link_id: link,
            };
            <Tokens<T>>::insert(&ticker, token);
            <BalanceOf<T>>::insert(ticker, did, total_supply);
            Self::deposit_event(RawEvent::AssetCreated(
                did,
                ticker,
                total_supply,
                divisible,
                asset_type,
            ));
            for (typ, val) in &identifiers {
                <Identifiers>::insert((ticker, typ.clone()), val.clone());
            }
            // Add funding round name
            if let Some(round) = funding_round {
                <FundingRound>::insert(ticker, round);
            } else {
                <FundingRound>::insert(ticker, FundingRoundName::default());
            }
            Self::deposit_event(RawEvent::IdentifiersUpdated(did, ticker, identifiers));
            <IssuedInFundingRound<T>>::insert((ticker, Self::funding_round(ticker)), total_supply);
            Self::deposit_event(RawEvent::Issued(
                did,
                ticker,
                did,
                total_supply,
                Self::funding_round(ticker),
                total_supply
            ));
            Ok(())
        }

        /// Freezes transfers and minting of a given token.
        ///
        /// # Arguments
        /// * `origin` - the signing key of the sender.
        /// * `ticker` - the ticker of the token.
        #[weight = SimpleDispatchInfo::FixedNormal(150_000)]
        pub fn freeze(origin, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let signer_key = AccountKey::try_from(sender.encode())?;
            let sender_did = Context::current_identity_or::<Identity<T>>(&signer_key)?;

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
        /// * `origin` - the signing key of the sender.
        /// * `ticker` - the ticker of the frozen token.
        #[weight = SimpleDispatchInfo::FixedNormal(150_000)]
        pub fn unfreeze(origin, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let signer_key = AccountKey::try_from(sender.encode())?;
            let sender_did = Context::current_identity_or::<Identity<T>>(&signer_key)?;

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
        /// * `origin` - the signing key of the sender.
        /// * `ticker` - the ticker of the token.
        /// * `name` - the new name of the token.
        #[weight = SimpleDispatchInfo::FixedNormal(150_000)]
        pub fn rename_asset(origin, ticker: Ticker, name: AssetName) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let signer = AccountKey::try_from(sender.encode())?;
            let sender_did = Context::current_identity_or::<Identity<T>>(&signer)?;

            // verify the ownership of the token
            ensure!(Self::is_owner(&ticker, sender_did), Error::<T>::Unauthorized);
            ensure!(<Tokens<T>>::contains_key(&ticker), Error::<T>::NoSuchAsset);

            <Tokens<T>>::mutate(&ticker, |token| token.name = name.clone());
            Self::deposit_event(RawEvent::AssetRenamed(sender_did, ticker, name));
            Ok(())
        }

        /// Transfer tokens from one DID to another DID as tokens are stored/managed on the DID level.
        ///
        /// # Arguments
        /// * `origin` signing key of the sender.
        /// * `ticker` Ticker of the token.
        /// * `to_did` DID of the `to` token holder, to whom token needs to transferred.
        /// * `value` Value that needs to transferred.
        #[weight = SimpleDispatchInfo::FixedNormal(400_000)]
        pub fn transfer(origin, ticker: Ticker, to_did: IdentityId, value: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&ticker, did, value)?;
            ensure!(
                Self::_is_valid_transfer(&ticker, sender, Some(did), Some(to_did), value)? == ERC1400_TRANSFER_SUCCESS,
                Error::<T>::InvalidTransfer
            );

            Self::_transfer(did, &ticker, did, to_did, value)
        }

        /// Forces a transfer between two DIDs & This can only be called by security token owner.
        /// This function doesn't validate any type of restriction beside a valid CDD check.
        ///
        /// # Arguments
        /// * `origin` signing key of the token owner DID.
        /// * `ticker` symbol of the token.
        /// * `from_did` DID of the token holder from whom balance token will be transferred.
        /// * `to_did` DID of token holder to whom token balance will be transferred.
        /// * `value` Amount of tokens.
        /// * `data` Some off chain data to validate the restriction.
        /// * `operator_data` It is a string which describes the reason of this control transfer call.
        #[weight = SimpleDispatchInfo::FixedNormal(400_000)]
        pub fn controller_transfer(origin, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, value: T::Balance, data: Vec<u8>, operator_data: Vec<u8>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            Self::_transfer(did, &ticker, from_did, to_did, value)?;

            Self::deposit_event(RawEvent::ControllerTransfer(did, ticker, from_did, to_did, value, data, operator_data));

            Ok(())
        }

        /// Approve token transfer from one DID to another.
        /// once this is done, transfer_from can be called with corresponding values.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner (i.e sender).
        /// * `spender_did` DID of the spender.
        /// * `value` Amount of the tokens approved.
        #[weight = SimpleDispatchInfo::FixedNormal(200_000)]
        fn approve(origin, ticker: Ticker, spender_did: IdentityId, value: T::Balance) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(<BalanceOf<T>>::contains_key(ticker, did), Error::<T>::NotAnOwner);
            let allowance = Self::allowance((ticker, did, spender_did));
            let updated_allowance = allowance.checked_add(&value)
                .ok_or(Error::<T>::AllowanceOverflow)?;
            <Allowance<T>>::insert((ticker, did, spender_did), updated_allowance);

            Self::deposit_event(RawEvent::Approval(did, ticker, did, spender_did, value));

            Ok(())
        }

        /// If sufficient allowance provided, transfer from a DID to another DID without token owner's signature.
        ///
        /// # Arguments
        /// * `origin` Signing key of spender.
        /// * `ticker` Ticker of the token.
        /// * `from_did` DID from whom token is being transferred.
        /// * `to_did` DID to whom token is being transferred.
        /// * `value` Amount of the token for transfer.
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        pub fn transfer_from(origin, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, value: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            let ticker_from_did_did = (ticker, from_did, did);
            ensure!(<Allowance<T>>::contains_key(&ticker_from_did_did), Error::<T>::NoSuchAllowance);
            let allowance = Self::allowance(&ticker_from_did_did);
            ensure!(allowance >= value, Error::<T>::InsufficientAllowance);

            // using checked_sub (safe math) to avoid overflow
            let updated_allowance = allowance.checked_sub(&value)
                .ok_or(Error::<T>::AllowanceOverflow)?;
            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&ticker, from_did, value)?;

            ensure!(
                Self::_is_valid_transfer(&ticker, sender, Some(from_did), Some(to_did), value)? == ERC1400_TRANSFER_SUCCESS,
                Error::<T>::InvalidTransfer
            );
            Self::_transfer(did, &ticker, from_did, to_did, value)?;

            // Change allowance afterwards
            <Allowance<T>>::insert(&ticker_from_did_did, updated_allowance);

            Self::deposit_event(RawEvent::Approval(did, ticker, from_did, did, value));
            Ok(())
        }

        /// Function used to create the checkpoint.
        /// NB: Only called by the owner of the security token i.e owner DID.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner. (Only token owner can call this function).
        /// * `ticker` Ticker of the token.
        #[weight = SimpleDispatchInfo::FixedNormal(300_000)]
        pub fn create_checkpoint(origin, ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            let _ = Self::_create_checkpoint(&ticker)?;
            Self::deposit_event(RawEvent::CheckpointCreated(did, ticker, Self::total_checkpoints_of(&ticker)));
            Ok(())
        }

        /// Function is used to issue(or mint) new tokens for the given DID
        /// can only be executed by the token owner.
        ///
        /// # Arguments
        /// * `origin` Signing key of token owner.
        /// * `ticker` Ticker of the token.
        /// * `to_did` DID of the token holder to whom new tokens get issued.
        /// * `value` Amount of tokens that get issued.
        #[weight = SimpleDispatchInfo::FixedNormal(700_000)]
        pub fn issue(origin, ticker: Ticker, to_did: IdentityId, value: T::Balance, _data: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let signer = Signatory::AccountKey(sender_key);

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            Self::_mint(&ticker, sender, to_did, value, Some((&signer, ProtocolOp::AssetIssue)))
        }

        /// Function is used issue(or mint) new tokens for the given DIDs
        /// can only be executed by the token owner.
        ///
        /// # Arguments
        /// * `origin` Signing key of token owner.
        /// * `ticker` Ticker of the token.
        /// * `investor_dids` Array of the DID of the token holders to whom new tokens get issued.
        /// * `values` Array of the Amount of tokens that get issued.
        ///
        /// # Weight
        /// `300_000 + 400_000 * issue_asset_items.len().max(values.len())`
        #[weight = FunctionOf(
            |(issue_asset_items, _): (
                &Vec<IssueAssetItem<T::Balance>>,
                &Ticker,
            )| {
                let n = u32::try_from(issue_asset_items.len())
                    .unwrap_or_default();
                300_000 + 400_000 * n
            },
            DispatchClass::Normal,
            true
        )]
        pub fn batch_issue(origin, issue_asset_items: Vec<IssueAssetItem<T::Balance>>, ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(!issue_asset_items.is_empty(), Error::<T>::NoInvestors);
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            // A helper vec for calculated new investor balances
            let mut updated_balances = Vec::with_capacity(issue_asset_items.len());
            // A helper vec for calculated new investor balances
            let mut current_balances = Vec::with_capacity(issue_asset_items.len());
            // Get current token details for supply update
            let mut token = Self::token_details(ticker);

            let round = Self::funding_round(&ticker);
            let ticker_round = (ticker, round.clone());
            // Update the total token balance issued in this funding round.
            let mut issued_in_this_round = Self::issued_in_funding_round(&ticker_round);

            // A round of per-investor checks
            for i in 0..issue_asset_items.len() {
                ensure!(
                    Self::check_granularity(&ticker, issue_asset_items[i].value),
                    Error::<T>::InvalidGranularity
                );
                let updated_total_supply = token
                    .total_supply
                    .checked_add(&issue_asset_items[i].value)
                    .ok_or(Error::<T>::TotalSupplyOverflow)?;
                ensure!(updated_total_supply <= MAX_SUPPLY.into(), Error::<T>::TotalSupplyAboveLimit);

                current_balances.push(Self::balance(&ticker, &issue_asset_items[i].investor_did));
                updated_balances.push(current_balances[i]
                    .checked_add(&issue_asset_items[i].value)
                    .ok_or(Error::<T>::BalanceOverflow)?);

                // verify transfer check
                ensure!(
                    Self::_is_valid_transfer(&ticker, sender.clone(),  None, Some(issue_asset_items[i].investor_did), issue_asset_items[i].value)? == ERC1400_TRANSFER_SUCCESS,
                    Error::<T>::InvalidTransfer
                );

                issued_in_this_round = issued_in_this_round
                    .checked_add(&issue_asset_items[i].value)
                    .ok_or(Error::<T>::FundingRoundTotalOverflow)?;

                // New total supply must be valid
                token.total_supply = updated_total_supply;
            }
            <<T as IdentityTrait>::ProtocolFee>::batch_charge_fee(
                &Signatory::AccountKey(sender_key),
                ProtocolOp::AssetIssue,
                issue_asset_items.len()
            )?;
            <IssuedInFundingRound<T>>::insert(&ticker_round, issued_in_this_round);
            // Update investor balances and emit events quoting the updated total token balance issued.
            for i in 0..issue_asset_items.len() {
                Self::_update_checkpoint(&ticker, issue_asset_items[i].investor_did, current_balances[i]);
                <BalanceOf<T>>::insert(ticker, issue_asset_items[i].investor_did, updated_balances[i]);
                <statistics::Module<T>>::update_transfer_stats(&ticker, None, Some(updated_balances[i]), issue_asset_items[i].value);
                Self::deposit_event(RawEvent::Issued(
                    did,
                    ticker,
                    issue_asset_items[i].investor_did,
                    issue_asset_items[i].value,
                    round.clone(),
                    issued_in_this_round
                ));
            }
            <Tokens<T>>::insert(ticker, token);

            Ok(())
        }

        /// Used to redeem the security tokens.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token holder who wants to redeem the tokens.
        /// * `ticker` Ticker of the token.
        /// * `value` Amount of the tokens needs to redeem.
        /// * `_data` An off chain data blob used to validate the redeem functionality.
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        pub fn redeem(origin, ticker: Ticker, value: T::Balance, _data: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            // Granularity check
            ensure!(Self::check_granularity(&ticker, value), Error::<T>::InvalidGranularity);
            ensure!(<BalanceOf<T>>::contains_key(&ticker, &did), Error::<T>::NotAnOwner);
            let burner_balance = Self::balance(&ticker, &did);
            ensure!(burner_balance >= value, Error::<T>::InsufficientBalance);

            // Reduce sender's balance
            let updated_burner_balance = burner_balance
                .checked_sub(&value)
                .ok_or(Error::<T>::BalanceOverflow)?;
            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&ticker, did, value)?;

            // verify transfer check
            ensure!(
                Self::_is_valid_transfer(&ticker, sender, Some(did), None, value)? == ERC1400_TRANSFER_SUCCESS,
                Error::<T>::InvalidTransfer
            );

            //Decrease total supply
            let mut token = Self::token_details(&ticker);
            token.total_supply = token.total_supply.checked_sub(&value)
                .ok_or(Error::<T>::BalanceOverflow)?;

            Self::_update_checkpoint(&ticker, did, burner_balance);

            <BalanceOf<T>>::insert(ticker, did, updated_burner_balance);
            <Tokens<T>>::insert(&ticker, token);
            <statistics::Module<T>>::update_transfer_stats(&ticker, Some(updated_burner_balance), None, value);

            Self::deposit_event(RawEvent::Redeemed(did, ticker, did, value));
            Ok(())
        }

        /// Used to redeem the security tokens by some other DID who has approval.
        ///
        /// # Arguments
        /// * `origin` Signing key of the spender who has valid approval to redeem the tokens.
        /// * `ticker` Ticker of the token.
        /// * `from_did` DID from whom balance get reduced.
        /// * `value` Amount of the tokens needs to redeem.
        /// * `_data` An off chain data blob used to validate the redeem functionality.
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        pub fn redeem_from(origin, ticker: Ticker, from_did: IdentityId, value: T::Balance, _data: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            // Granularity check
            ensure!(Self::check_granularity(&ticker, value), Error::<T>::InvalidGranularity);
            ensure!(<BalanceOf<T>>::contains_key(&ticker, &did), Error::<T>::NotAnOwner);
            let burner_balance = Self::balance(&ticker, &did);
            ensure!(burner_balance >= value, Error::<T>::InsufficientBalance);

            // Reduce sender's balance
            let updated_burner_balance = burner_balance
                .checked_sub(&value)
                .ok_or(Error::<T>::BalanceOverflow)?;

            let ticker_from_did_did = (ticker, from_did, did);
            ensure!(<Allowance<T>>::contains_key(&ticker_from_did_did), Error::<T>::NoSuchAllowance);
            let allowance = Self::allowance(&ticker_from_did_did);
            ensure!(allowance >= value, Error::<T>::InsufficientAllowance);
            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&ticker, did, value)?;
            ensure!(
                Self::_is_valid_transfer(&ticker, sender, Some(from_did), None, value)? == ERC1400_TRANSFER_SUCCESS,
                Error::<T>::InvalidTransfer
            );

            let updated_allowance = allowance.checked_sub(&value)
                .ok_or(Error::<T>::AllowanceOverflow)?;

            //Decrease total supply
            let mut token = Self::token_details(&ticker);
            token.total_supply = token.total_supply.checked_sub(&value)
                .ok_or(Error::<T>::BalanceOverflow)?;

            Self::_update_checkpoint(&ticker, did, burner_balance);

            <Allowance<T>>::insert(&ticker_from_did_did, updated_allowance);
            <BalanceOf<T>>::insert(&ticker, &did, updated_burner_balance);
            <Tokens<T>>::insert(&ticker, token);
            <statistics::Module<T>>::update_transfer_stats( &ticker, Some(updated_burner_balance), None, value);

            Self::deposit_event(RawEvent::Redeemed(did, ticker, from_did, value));
            Self::deposit_event(RawEvent::Approval(did, ticker, from_did, did, value));

            Ok(())
        }

        /// Forces a redemption of an DID's tokens. Can only be called by token owner.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner.
        /// * `ticker` Ticker of the token.
        /// * `token_holder_did` DID from whom balance get reduced.
        /// * `value` Amount of the tokens needs to redeem.
        /// * `data` An off chain data blob used to validate the redeem functionality.
        /// * `operator_data` Any data blob that defines the reason behind the force redeem.
        #[weight = SimpleDispatchInfo::FixedNormal(400_000)]
        pub fn controller_redeem(origin, ticker: Ticker, token_holder_did: IdentityId, value: T::Balance, data: Vec<u8>, operator_data: Vec<u8>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);
            // Granularity check
            ensure!(Self::check_granularity(&ticker, value), Error::<T>::InvalidGranularity);
            ensure!(<BalanceOf<T>>::contains_key(&ticker, &token_holder_did), Error::<T>::NotAAssetHolder);
            let burner_balance = Self::balance(&ticker, &token_holder_did);
            ensure!(burner_balance >= value, Error::<T>::InsufficientBalance);

            // Reduce sender's balance
            let updated_burner_balance = burner_balance
                .checked_sub(&value)
                .ok_or(Error::<T>::BalanceOverflow)?;

            // Decrease total supply
            let mut token = Self::token_details(&ticker);
            token.total_supply = token.total_supply.checked_sub(&value).ok_or(Error::<T>::BalanceOverflow)?;

            Self::_update_checkpoint(&ticker, token_holder_did, burner_balance);

            <BalanceOf<T>>::insert(&ticker, &token_holder_did, updated_burner_balance);
            <Tokens<T>>::insert(&ticker, token);
            <statistics::Module<T>>::update_transfer_stats( &ticker, Some(updated_burner_balance), None, value);

            Self::deposit_event(RawEvent::ControllerRedemption(did, ticker, token_holder_did, value, data, operator_data));

            Ok(())
        }

        /// Makes an indivisible token divisible. Only called by the token owner.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner.
        /// * `ticker` Ticker of the token.
        #[weight = SimpleDispatchInfo::FixedNormal(150_000)]
        pub fn make_divisible(origin, ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            // Read the token details
            let mut token = Self::token_details(&ticker);
            ensure!(!token.divisible, Error::<T>::AssetAlreadyDivisible);
            token.divisible = true;
            <Tokens<T>>::insert(&ticker, token);
            Self::deposit_event(RawEvent::DivisibilityChanged(did, ticker, true));
            Ok(())
        }

        /// An ERC1594 transfer with data
        /// This function can be used by the exchanges or other third parties to dynamically validate the transaction
        /// by passing the data blob.
        ///
        /// # Arguments
        /// * `origin` Signing key of the sender.
        /// * `ticker` Ticker of the token.
        /// * `to_did` DID to whom tokens will be transferred.
        /// * `value` Amount of the tokens.
        /// * `data` Off chain data blob to validate the transfer.
        #[weight = SimpleDispatchInfo::FixedNormal(450_000)]
        pub fn transfer_with_data(origin, ticker: Ticker, to_did: IdentityId, value: T::Balance, data: Vec<u8>) -> DispatchResult {

            let sender_key = AccountKey::try_from(ensure_signed(origin.clone())?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            Self::transfer(origin, ticker, to_did, value)?;

            Self::deposit_event(RawEvent::TransferWithData(did, ticker, did, to_did, value, data));
            Ok(())
        }

        /// An ERC1594 transfer_from with data
        /// This function can be used by the exchanges or other third parties to dynamically validate the transaction
        /// by passing the data blob.
        ///
        /// # Arguments
        /// * `origin` Signing key of the spender.
        /// * `ticker` Ticker of the token.
        /// * `from_did` DID from whom tokens will be transferred.
        /// * `to_did` DID to whom tokens will be transferred.
        /// * `value` Amount of the tokens.
        /// * `data` Off chain data blob to validate the transfer.
        #[weight = SimpleDispatchInfo::FixedNormal(550_000)]
        pub fn transfer_from_with_data(origin, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, value: T::Balance, data: Vec<u8>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin.clone())?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            Self::transfer_from(origin, ticker, from_did,  to_did, value)?;

            Self::deposit_event(RawEvent::TransferWithData(did, ticker, from_did, to_did, value, data));
            Ok(())
        }

        /// Used to know whether the given token will issue new tokens or not.
        ///
        /// # Arguments
        /// * `_origin` Signing key.
        /// * `ticker` Ticker of the token whose issuance status need to know.
        #[weight = SimpleDispatchInfo::FixedNormal(5_000)]
        pub fn is_issuable(_origin, ticker:Ticker) {
            Self::deposit_event(RawEvent::IsIssuable(ticker, true));
        }

        /// Add documents for a given token. To be called only by the token owner.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner.
        /// * `ticker` Ticker of the token.
        /// * `documents` Documents to be attached to `ticker`.
        ///
        /// # Weight
        /// `200_000 + 60_000 * documents.len()`
        #[weight = FunctionOf(
            |(documents, _): (&Vec<Document>, &Ticker)| {
                200_000 + 60_000 * u32::try_from(documents.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn batch_add_document(origin, documents: Vec<Document>, ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);

            let ticker_did = <identity::Module<T>>::get_token_did(&ticker)?;
            let signer = Signatory::from(ticker_did);
            <<T as IdentityTrait>::ProtocolFee>::batch_charge_fee(
                &Signatory::AccountKey(sender_key),
                ProtocolOp::AssetAddDocument,
                documents.len()
            )?;
            documents.into_iter().for_each(|doc| {
                <identity::Module<T>>::add_link(signer, LinkData::DocumentOwned(doc), None);
            });

            Ok(())
        }

        /// Remove documents for a given token. To be called only by the token owner.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner.
        /// * `ticker` Ticker of the token.
        /// * `doc_ids` Documents to be removed from `ticker`.
        ///
        /// # Weight
        /// `200_000 + 60_000 * do_ids.len()`
        #[weight = FunctionOf(
            |(doc_ids, _): (&Vec<u64>, &Ticker)| {
                200_000 + 60_000 * u32::try_from(doc_ids.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn batch_remove_document(origin, doc_ids: Vec<u64>, ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);

            let ticker_did = <identity::Module<T>>::get_token_did(&ticker)?;
            let signer = Signatory::from(ticker_did);
            doc_ids.into_iter().for_each(|doc_id| {
                <identity::Module<T>>::remove_link(signer, doc_id)
            });

            Ok(())
        }

        /// Update documents for the given token, Only be called by the token owner.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner.
        /// * `ticker` Ticker of the token.
        /// * `docs` Vector of tuples (Document to be updated, Contents of new document).
        ///
        /// # Weight
        /// `200_000 + 60_000 * docs.len()`
        #[weight = FunctionOf(
            |(docs, _): (&Vec<(u64, Document)>, &Ticker)| {
                200_000 + 60_000 * u32::try_from(docs.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn batch_update_document(origin, docs: Vec<(u64, Document)>, ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);

            let ticker_did = <identity::Module<T>>::get_token_did(&ticker)?;
            let signer = Signatory::from(ticker_did);
            docs.into_iter().for_each(|(doc_id, doc)| {
                <identity::Module<T>>::update_link(signer, doc_id, LinkData::DocumentOwned(doc))
            });

            Ok(())
        }

        /// ERC-2258 Implementation

        /// Used to increase the allowance for a given custodian
        /// Any investor/token holder can add a custodian and transfer the token transfer ownership to the custodian
        /// Through that investor balance will remain the same but the given token are only transfer by the custodian.
        /// This implementation make sure to have an accurate investor count from omnibus wallets.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token holder.
        /// * `ticker` Ticker of the token.
        /// * `custodian_did` DID of the custodian (i.e whom allowance provided).
        /// * `value` Allowance amount.
        #[weight = SimpleDispatchInfo::FixedNormal(300_000)]
        pub fn increase_custody_allowance(origin, ticker: Ticker, custodian_did: IdentityId, value: T::Balance) -> DispatchResult {
            let sender_key = AccountKey::try_from((ensure_signed(origin)?).encode())?;
            let sender_did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            Self::_increase_custody_allowance(sender_did, ticker, sender_did, custodian_did, value)?;
            Ok(())
        }

        /// Used to increase the allowance for a given custodian by providing the off chain signature.
        ///
        /// # Arguments
        /// * `origin` Signing key of a DID who posses off chain signature.
        /// * `ticker` Ticker of the token.
        /// * `holder_did` DID of the token holder (i.e who wants to increase the custody allowance).
        /// * `holder_account_id` Signing key which signs the off chain data blob.
        /// * `custodian_did` DID of the custodian (i.e whom allowance provided).
        /// * `value` Allowance amount.
        /// * `nonce` A u16 number which avoid the replay attack.
        /// * `signature` Signature provided by the holder_did.
        #[weight = SimpleDispatchInfo::FixedNormal(450_000)]
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
            let sender_key = AccountKey::try_from(sender.encode())?;
            let caller_did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            ensure!(
                !Self::authentication_nonce((ticker, holder_did, nonce)),
                Error::<T>::SignatureAlreadyUsed
            );

            let msg = SignData {
                custodian_did: custodian_did,
                holder_did: holder_did,
                ticker,
                value,
                nonce
            };
            // holder_account_id should be a part of the holder_did
            ensure!(
                signature.verify(&msg.encode()[..], &holder_account_id),
                Error::<T>::InvalidSignature
            );
            // Validate the holder signing key
            let holder_signer = Signatory::AccountKey(AccountKey::try_from(holder_account_id.encode())?);
            ensure!(
                <identity::Module<T>>::is_signer_authorized(holder_did, &holder_signer),
                Error::<T>::HolderMustBeSigningKeyForHolderDid
            );
            Self::_increase_custody_allowance(caller_did, ticker, holder_did, custodian_did, value)?;
            <AuthenticationNonce>::insert((ticker, holder_did, nonce), true);
            Ok(())
        }

        /// Used to transfer the tokens by the approved custodian.
        ///
        /// # Arguments
        /// * `origin` Signing key of the custodian.
        /// * `ticker` Ticker of the token.
        /// * `holder_did` DID of the token holder (i.e whom balance get reduced).
        /// * `receiver_did` DID of the receiver.
        /// * `value` Amount of tokens need to transfer.
        #[weight = SimpleDispatchInfo::FixedNormal(750_000)]
        pub fn transfer_by_custodian(
            origin,
            ticker: Ticker,
            holder_did: IdentityId,
            receiver_did: IdentityId,
            value: T::Balance
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let custodian_did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            let mut custodian_allowance = Self::custodian_allowance((ticker, holder_did, custodian_did));
            // Check whether the custodian has enough allowance or not
            ensure!(custodian_allowance >= value, Error::<T>::InsufficientAllowance);
            // using checked_sub (safe math) to avoid underflow
            custodian_allowance = custodian_allowance.checked_sub(&value)
                .ok_or(Error::<T>::AllowanceUnderflow)?;
            // using checked_sub (safe math) to avoid underflow
            let new_total_allowance = Self::total_custody_allowance((ticker, holder_did))
                .checked_sub(&value)
                .ok_or(Error::<T>::TotalAllowanceUnderflow)?;
            // Validate the transfer
            ensure!(
                Self::_is_valid_transfer(&ticker, sender, Some(holder_did), Some(receiver_did), value)? == ERC1400_TRANSFER_SUCCESS,
                Error::<T>::InvalidTransfer
            );
            Self::_transfer(custodian_did, &ticker, holder_did, receiver_did, value)?;
            // Update Storage of allowance
            <CustodianAllowance<T>>::insert((ticker, custodian_did, holder_did), &custodian_allowance);
            <TotalCustodyAllowance<T>>::insert((ticker, holder_did), new_total_allowance);
            Self::deposit_event(RawEvent::CustodyTransfer(custodian_did, ticker, holder_did, receiver_did, value));
            Ok(())
        }

        /// Sets the name of the current funding round.
        ///
        /// # Arguments
        /// * `origin` - the signing key of the token owner DID.
        /// * `ticker` - the ticker of the token.
        /// * `name` - the desired name of the current funding round.
        #[weight = SimpleDispatchInfo::FixedNormal(150_000)]
        pub fn set_funding_round(origin, ticker: Ticker, name: FundingRoundName) ->
            DispatchResult
        {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);
            <FundingRound>::insert(ticker, name.clone());
            Self::deposit_event(RawEvent::FundingRoundSet(did, ticker, name));
            Ok(())
        }

        /// Updates the asset identifiers. Can only be called by the token owner.
        ///
        /// # Arguments
        /// * `origin` - the signing key of the token owner.
        /// * `ticker` - the ticker of the token.
        /// * `identifiers` - the asset identifiers to be updated in the form of a vector of pairs
        ///    of `IdentifierType` and `AssetIdentifier` value.
        ///
        /// # Weight
        /// `150_000 + 20_000 * identifiers.len()`
        #[weight = FunctionOf(
            |(_, identifiers): (&Ticker, &Vec<(IdentifierType, AssetIdentifier)>)| {
                150_000 + 20_000 * u32::try_from(identifiers.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn update_identifiers(
            origin,
            ticker: Ticker,
            identifiers: Vec<(IdentifierType, AssetIdentifier)>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
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
        #[weight = SimpleDispatchInfo::FixedNormal(250_000)]
        pub fn add_extension(origin, ticker: Ticker, extension_details: SmartExtension<T::AccountId>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let my_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, my_did), Error::<T>::Unauthorized);

            // Verify the details of smart extension & store it
            ensure!(!<ExtensionDetails<T>>::contains_key((ticker, &extension_details.extension_id)), Error::<T>::ExtensionAlreadyPresent);
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
        #[weight = SimpleDispatchInfo::FixedNormal(250_000)]
        pub fn archive_extension(origin, ticker: Ticker, extension_id: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let my_did =  Context::current_identity_or::<identity::Module<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, my_did), Error::<T>::Unauthorized);
            ensure!(
                <ExtensionDetails<T>>::contains_key((ticker, &extension_id)),
                Error::<T>::NoSuchSmartExtension
            );
            // Mutate the extension details
            ensure!(!(<ExtensionDetails<T>>::get((ticker, &extension_id))).is_archive, Error::<T>::AlreadyArchived);
            <ExtensionDetails<T>>::mutate((ticker, &extension_id), |details| { details.is_archive = true; });
            Self::deposit_event(RawEvent::ExtensionArchived(my_did, ticker, extension_id));
            Ok(())
        }

        /// Un-archived the extension. Extension is use to verify the compliance or any smart logic it posses.
        ///
        /// # Arguments
        /// * `origin` - Signatory who owns the ticker/asset.
        /// * `ticker` - Ticker symbol of the asset.
        /// * `extension_id` - AccountId of the extension that need to be un-archived.
        #[weight = SimpleDispatchInfo::FixedNormal(250_000)]
        pub fn unarchive_extension(origin, ticker: Ticker, extension_id: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let my_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, my_did), Error::<T>::Unauthorized);
            ensure!(
                <ExtensionDetails<T>>::contains_key((ticker, &extension_id)),
                Error::<T>::NoSuchSmartExtension
            );
            // Mutate the extension details
            ensure!((<ExtensionDetails<T>>::get((ticker, &extension_id))).is_archive, Error::<T>::AlreadyUnArchived);
            <ExtensionDetails<T>>::mutate((ticker, &extension_id), |details| { details.is_archive = false; });
            Self::deposit_event(RawEvent::ExtensionUnArchived(my_did, ticker, extension_id));
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
        /// caller DID, ticker, beneficiary DID, value, funding round, total issued in this funding round
        Issued(IdentityId, Ticker, IdentityId, Balance, FundingRoundName, Balance),
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
        /// caller DID/ owner DID, ticker, total supply, divisibility, asset type
        AssetCreated(IdentityId, Ticker, Balance, bool, AssetType),
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
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// DID not found.
        DIDNotFound,
        /// Not a ticker transfer auth.
        NoTickerTransferAuth,
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
        /// The sender must be a signing key for the DID.
        HolderMustBeSigningKeyForHolderDid,
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
        /// An overflow while calculating the allowance.
        AllowanceOverflow,
        /// An underflow in calculating the allowance.
        AllowanceUnderflow,
        /// An overflow in calculating the total allowance.
        TotalAllowanceOverflow,
        /// An underflow in calculating the total allowance.
        TotalAllowanceUnderflow,
        /// An overflow while calculating the current funding round total.
        FundingRoundTotalOverflow,
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
        NotAAssetHolder,
        /// The asset must be frozen.
        NotFrozen,
        /// No such smart extension.
        NoSuchSmartExtension,
        /// Transfer validation check failed.
        InvalidTransfer,
        /// The sender balance is not sufficient.
        InsufficientBalance,
        /// An invalid signature.
        InvalidSignature,
        /// The signature is already in use.
        SignatureAlreadyUsed,
        /// The token is already divisible.
        AssetAlreadyDivisible,
        /// An invalid custodian DID.
        InvalidCustodianDid,
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
}

impl<T: Trait> AcceptTransfer for Module<T> {
    fn accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult {
        Self::_accept_ticker_transfer(to_did, auth_id)
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

    pub fn is_ticker_available(ticker: &Ticker) -> bool {
        // Assumes uppercase ticker
        if <Tickers<T>>::contains_key(ticker) {
            let now = <pallet_timestamp::Module<T>>::get();
            if let Some(expiry) = Self::ticker_registration(*ticker).expiry {
                if now <= expiry {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn is_ticker_registry_valid(ticker: &Ticker, did: IdentityId) -> bool {
        // Assumes uppercase ticker
        if <Tickers<T>>::contains_key(ticker) {
            let now = <pallet_timestamp::Module<T>>::get();
            let ticker_reg = Self::ticker_registration(ticker);
            if ticker_reg.owner == did {
                if let Some(expiry) = ticker_reg.expiry {
                    if now > expiry {
                        return false;
                    }
                } else {
                    return true;
                }
                return true;
            }
        }
        false
    }

    /// Returns 0 if ticker is registered to someone else.
    /// 1 if ticker is available for registry.
    /// 2 if ticker is already registered to provided did.
    pub fn is_ticker_available_or_registered_to(
        ticker: &Ticker,
        did: IdentityId,
    ) -> TickerRegistrationStatus {
        // Assumes uppercase ticker
        if <Tickers<T>>::contains_key(ticker) {
            let ticker_reg = Self::ticker_registration(*ticker);
            if let Some(expiry) = ticker_reg.expiry {
                let now = <pallet_timestamp::Module<T>>::get();
                if now > expiry {
                    // ticker registered to someone but expired and can be registered again
                    return TickerRegistrationStatus::Available;
                } else if ticker_reg.owner == did {
                    // ticker is already registered to provided did (but may expire in future)
                    return TickerRegistrationStatus::RegisteredByDid;
                }
            } else if ticker_reg.owner == did {
                // ticker is already registered to provided did (and will never expire)
                return TickerRegistrationStatus::RegisteredByDid;
            }
            // ticker registered to someone else
            return TickerRegistrationStatus::RegisteredByOther;
        }
        // Ticker not registered yet
        TickerRegistrationStatus::Available
    }

    fn _register_ticker(
        ticker: &Ticker,
        signer: &Signatory,
        to_did: IdentityId,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        <<T as IdentityTrait>::ProtocolFee>::charge_fee(&signer, ProtocolOp::AssetRegisterTicker)?;

        if <Tickers<T>>::contains_key(ticker) {
            let ticker_details = <Tickers<T>>::get(ticker);
            <identity::Module<T>>::remove_link(
                Signatory::from(ticker_details.owner),
                ticker_details.link_id,
            );
        }

        let link = <identity::Module<T>>::add_link(
            Signatory::from(to_did),
            LinkData::TickerOwned(*ticker),
            expiry,
        );

        let ticker_registration = TickerRegistration {
            owner: to_did,
            expiry,
            link_id: link,
        };

        // Store ticker registration details
        <Tickers<T>>::insert(ticker, ticker_registration);

        Self::deposit_event(RawEvent::TickerRegistered(to_did, *ticker, expiry));
        Ok(())
    }

    /// Get the asset `id` balance of `who`.
    pub fn balance(ticker: &Ticker, did: &IdentityId) -> T::Balance {
        Self::balance_of(ticker, did)
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
            return Self::balance(&ticker, &did);
        }

        if <UserCheckpoints>::contains_key(&ticker_did) {
            let user_checkpoints = Self::user_checkpoints(&ticker_did);
            if at > *user_checkpoints.last().unwrap_or(&0) {
                // Using unwrap_or to be defensive.
                // or part should never be triggered due to the check on 2 lines above
                // User has not transacted after checkpoint creation.
                // This means their current balance = their balance at that cp.
                return Self::balance(&ticker, &did);
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
        Self::balance(&ticker, &did)
    }

    fn find_ceiling(arr: &Vec<u64>, key: u64) -> u64 {
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

    fn _is_valid_transfer(
        ticker: &Ticker,
        extension_caller: T::AccountId,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        value: T::Balance,
    ) -> StdResult<u8, &'static str> {
        if Self::frozen(ticker) {
            return Ok(ERC1400_TRANSFERS_HALTED);
        }
        let general_status_code =
            T::ComplianceManager::verify_restriction(ticker, from_did, to_did, value)?;
        Ok(if general_status_code != ERC1400_TRANSFER_SUCCESS {
            COMPLIANCE_MANAGER_FAILURE
        } else {
            let mut final_result = true;
            let mut is_valid = false;
            let mut is_invalid = false;
            let mut force_valid = false;
            let tms = Self::extensions((ticker, SmartExtensionType::TransferManager))
                .into_iter()
                .filter(|tm| !Self::extension_details((ticker, tm)).is_archive)
                .collect::<Vec<T::AccountId>>();
            if !tms.is_empty() {
                for tm in tms.into_iter() {
                    let result = Self::verify_restriction(
                        ticker,
                        extension_caller.clone(),
                        from_did,
                        to_did,
                        value,
                        tm,
                    );
                    match result {
                        RestrictionResult::Valid => is_valid = true,
                        RestrictionResult::Invalid => is_invalid = true,
                        RestrictionResult::ForceValid => force_valid = true,
                    }
                }
                //is_valid = force_valid ? true : (is_invalid ? false : is_valid);
                final_result = force_valid || !is_invalid && is_valid;
            }
            if final_result {
                return Ok(ERC1400_TRANSFER_SUCCESS);
            } else {
                return Ok(SMART_EXTENSION_FAILURE);
            }
        })
    }

    // The SimpleToken standard transfer function
    // internal
    fn _transfer(
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
            Error::<T>::NotAAssetHolder
        );
        let sender_balance = Self::balance(ticker, &from_did);
        ensure!(sender_balance >= value, Error::<T>::InsufficientBalance);

        let updated_from_balance = sender_balance
            .checked_sub(&value)
            .ok_or(Error::<T>::BalanceOverflow)?;
        let receiver_balance = Self::balance(ticker, &to_did);
        let updated_to_balance = receiver_balance
            .checked_add(&value)
            .ok_or(Error::<T>::BalanceOverflow)?;

        Self::_update_checkpoint(ticker, from_did, sender_balance);
        Self::_update_checkpoint(ticker, to_did, receiver_balance);
        // reduce sender's balance
        <BalanceOf<T>>::insert(ticker, &from_did, updated_from_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert(ticker, &to_did, updated_to_balance);

        // Update statistic info.
        <statistics::Module<T>>::update_transfer_stats(
            ticker,
            Some(updated_from_balance),
            Some(updated_to_balance),
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
        protocol_fee_data: Option<(&Signatory, ProtocolOp)>,
    ) -> DispatchResult {
        // Granularity check
        ensure!(
            Self::check_granularity(ticker, value),
            Error::<T>::InvalidGranularity
        );
        //Increase receiver balance
        let current_to_balance = Self::balance(ticker, &to_did);
        let updated_to_balance = current_to_balance
            .checked_add(&value)
            .ok_or(Error::<T>::BalanceOverflow)?;
        // verify transfer check
        ensure!(
            Self::_is_valid_transfer(ticker, caller.clone(), None, Some(to_did), value)?
                == ERC1400_TRANSFER_SUCCESS,
            Error::<T>::InvalidTransfer
        );

        // Read the token details
        let mut token = Self::token_details(ticker);
        let updated_total_supply = token
            .total_supply
            .checked_add(&value)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;
        ensure!(
            updated_total_supply <= MAX_SUPPLY.into(),
            Error::<T>::TotalSupplyAboveLimit
        );
        // Increase total supply
        token.total_supply = updated_total_supply;

        // Charge the given fee.
        if let Some((payee, op)) = protocol_fee_data {
            <<T as IdentityTrait>::ProtocolFee>::charge_fee(payee, op)?;
        }
        Self::_update_checkpoint(ticker, to_did, current_to_balance);

        <BalanceOf<T>>::insert(ticker, &to_did, updated_to_balance);
        <Tokens<T>>::insert(ticker, token);
        let round = Self::funding_round(ticker);
        let ticker_round = (*ticker, round.clone());
        let issued_in_this_round = Self::issued_in_funding_round(&ticker_round)
            .checked_add(&value)
            .ok_or(Error::<T>::FundingRoundTotalOverflow)?;
        <IssuedInFundingRound<T>>::insert(&ticker_round, issued_in_this_round);
        Self::deposit_event(RawEvent::Issued(
            Context::current_identity_or::<Identity<T>>(&AccountKey::try_from(caller.encode())?)?,
            *ticker,
            to_did,
            value,
            round,
            issued_in_this_round,
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
        let remaining_balance = Self::balance(&ticker, &holder_did)
            .checked_sub(&value)
            .ok_or(Error::<T>::BalanceUnderflow)?;
        ensure!(
            remaining_balance >= Self::total_custody_allowance(&(*ticker, holder_did)),
            Error::<T>::InsufficientBalance
        );
        Ok(())
    }

    fn _increase_custody_allowance(
        caller_did: IdentityId,
        ticker: Ticker,
        holder_did: IdentityId,
        custodian_did: IdentityId,
        value: T::Balance,
    ) -> DispatchResult {
        let new_custody_allowance = Self::total_custody_allowance((ticker, holder_did))
            .checked_add(&value)
            .ok_or(Error::<T>::TotalAllowanceOverflow)?;
        // Ensure that balance of the token holder should greater than or equal to the total custody allowance + value
        ensure!(
            Self::balance(&ticker, &holder_did) >= new_custody_allowance,
            Error::<T>::InsufficientBalance
        );
        // Ensure the valid DID
        ensure!(
            <identity::DidRecords>::contains_key(custodian_did),
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
            Signatory::from(ticker_details.owner),
            Signatory::from(to_did),
            auth_id,
        )?;

        <identity::Module<T>>::remove_link(
            Signatory::from(ticker_details.owner),
            ticker_details.link_id,
        );

        let link = <identity::Module<T>>::add_link(
            Signatory::from(to_did),
            LinkData::TickerOwned(ticker),
            ticker_details.expiry,
        );

        <Tickers<T>>::mutate(&ticker, |tr| {
            tr.owner = to_did;
            tr.link_id = link;
        });

        Self::deposit_event(RawEvent::TickerTransferred(
            to_did,
            ticker,
            ticker_details.owner,
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
            Signatory::from(token_details.owner_did),
            Signatory::from(to_did),
            auth_id,
        )?;

        <identity::Module<T>>::remove_link(
            Signatory::from(ticker_details.owner),
            ticker_details.link_id,
        );
        <identity::Module<T>>::remove_link(
            Signatory::from(token_details.owner_did),
            token_details.link_id,
        );

        let ticker_link = <identity::Module<T>>::add_link(
            Signatory::from(to_did),
            LinkData::TickerOwned(ticker),
            None,
        );
        let token_link = <identity::Module<T>>::add_link(
            Signatory::from(to_did),
            LinkData::AssetOwned(ticker),
            None,
        );

        <Tickers<T>>::mutate(&ticker, |tr| {
            tr.owner = to_did;
            tr.link_id = ticker_link;
        });
        <Tokens<T>>::mutate(&ticker, |tr| {
            tr.owner_did = to_did;
            tr.link_id = token_link;
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

        // Creation of the encoded data for the verifyTransfer function of the extension
        // i.e fn verify_transfer(
        //        from: Option<IdentityId>,
        //        to: Option<IdentityId>,
        //        value: Balance,
        //        balance_from: Balance,
        //        balance_to: Balance,
        //        total_supply: Balance
        //    ) -> RestrictionResult { }

        let encoded_data = [
            &selector[..],
            &encoded_from[..],
            &encoded_to[..],
            &encoded_value[..],
            &balance_from[..],
            &balance_to[..],
            &total_supply[..],
        ]
        .concat();

        // Calling extension to verify the compliance rule
        // native currency value should be `0` as no funds need to transfer to the smart extension
        // We are passing arbitrary high `gas_limit` value to make sure extension's function execute successfully
        // TODO: Once gas estimate function will be introduced, arbitrary gas value will be replaced by the estimated gas
        let is_allowed =
            Self::call_extension(extension_caller, dest, 0.into(), 5_000_000, encoded_data);
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
        // Non-Issuance case check
        if let Some(from_id) = from_did {
            if Identity::<T>::has_valid_cdd(from_id) {
                let balance = Self::balance(&ticker, &from_id);
                if balance < amount
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
                .unwrap_or(ERC1400_TRANSFER_FAILURE),
        )
    }
}
