//! # Asset Module
//!
//! The Asset module is one place to create the security tokens on the Polymesh blockchain.
//! It consist every required functionality related to securityToken and every function
//! execution can be differentiate at the token level by providing the ticker of the token.
//! In ethereum analogy every token has different smart contract address which act as the unique identity
//! of the token while here token lives at low-level where token ticker act as the differentiator
//!
//! ## Overview
//!
//! The Asset module provides functions for:
//!
//! - Creating the tokens
//! - Creation of checkpoints on the token level
//! - Management of the token (Document mgt etc)
//! - Transfer/redeem functionality of the token
//! - Custodian functionality
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `register_ticker` - Used to either register a new ticker or extend registration of an existing ticker
//! - `accept_ticker_transfer` - Used to accept a ticker transfer authorization
//! - `create_token` - Initializes a new security token
//! - `transfer` - Transfer tokens from one DID to another DID as tokens are stored/managed on the DID level
//! - `controller_transfer` - Forces a transfer between two DIDs.
//! - `approve` - Approve token transfer from one DID to DID
//! - `transfer_from` - If sufficient allowance provided, transfer from a DID to another DID without token owner's signature.
//! - `create_checkpoint` - Function used to create the checkpoint
//! - `issue` - Function is used to issue(or mint) new tokens for the given DID
//! - `batch_issue` - Batch version of issue function
//! - `redeem` - Used to redeem the security tokens
//! - `redeem_from` - Used to redeem the security tokens by some other DID who has approval
//! - `controller_redeem` - Forces a redemption of an DID's tokens. Can only be called by token owner
//! - `make_divisible` - Change the divisibility of the token to divisible. Only called by the token owner
//! - `can_transfer` - Checks whether a transaction with given parameters can take place or not
//! - `transfer_with_data` - This function can be used by the exchanges of other third parties to dynamically validate the transaction by passing the data blob
//! - `transfer_from_with_data` - This function can be used by the exchanges of other third parties to dynamically validate the transaction by passing the data blob
//! - `is_issuable` - Used to know whether the given token will issue new tokens or not
//! - `get_document` - Used to get the documents details attach with the token
//! - `set_document` - Used to set the details of the document, Only be called by the token owner
//! - `remove_document` - Used to remove the document details for the given token, Only be called by the token owner
//! - `increase_custody_allowance` - Used to increase the allowance for a given custodian
//! - `increase_custody_allowance_of` - Used to increase the allowance for a given custodian by providing the off chain signature
//! - `transfer_by_custodian` - Used to transfer the tokens by the approved custodian
//!
//! ### Public Functions
//!
//! - `is_ticker_available` - Returns if ticker is available to register
//! - `is_ticker_registry_valid` - Returns if ticker is registered to a particular did
//! - `token_details` - Returns details of the token
//! - `balance_of` - Returns the balance of the DID corresponds to the ticker
//! - `total_checkpoints_of` - Returns the checkpoint Id
//! - `total_supply_at` - Returns the total supply at a given checkpoint
//! - `custodian_allowance`- Returns the allowance provided to a custodian for a given ticker and token holder
//! - `total_custody_allowance` - Returns the total allowance approved by the token holder.

use crate::{balances, constants::*, general_tm, identity, percentage_tm, utils};
use codec::Encode;
use core::result::Result as StdResult;
use currency::*;
use primitives::{AuthorizationData, AuthorizationError, IdentityId, Key, Signer, Ticker};
use rstd::{convert::TryFrom, prelude::*};
use session;
use sr_primitives::traits::{CheckedAdd, CheckedSub, Verify};
#[cfg(feature = "std")]
use sr_primitives::{Deserialize, Serialize};
use srml_support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, WithdrawReason},
};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait:
    system::Trait
    + general_tm::Trait
    + percentage_tm::Trait
    + utils::Trait
    + balances::Trait
    + identity::Trait
    + session::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
}

/// The type of an asset represented by a token.
#[derive(codec::Encode, codec::Decode, Clone, Debug, PartialEq, Eq)]
pub enum AssetType {
    Equity,
    Debt,
    Commodity,
    StructuredProduct,
    Custom(Vec<u8>),
}

impl Default for AssetType {
    fn default() -> Self {
        AssetType::Custom(b"undefined".to_vec())
    }
}

/// The type of an identifier associated with a token.
#[derive(codec::Encode, codec::Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifierType {
    Isin,
    Cusip,
    Custom(Vec<u8>),
}

impl Default for IdentifierType {
    fn default() -> Self {
        IdentifierType::Custom(b"undefined".to_vec())
    }
}

/// struct to store the token details
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct SecurityToken<U> {
    pub name: Vec<u8>,
    pub total_supply: U,
    pub owner_did: IdentityId,
    pub divisible: bool,
    pub asset_type: AssetType,
}

/// struct to store the signed data
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct SignData<U> {
    custodian_did: IdentityId,
    holder_did: IdentityId,
    ticker: Ticker,
    value: U,
    nonce: u16,
}

/// struct to store the ticker registration details
#[derive(codec::Encode, codec::Decode, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistration<U> {
    owner: IdentityId,
    expiry: Option<U>,
}

/// struct to store the ticker registration config
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(codec::Encode, codec::Decode, Clone, Default, PartialEq, Debug)]
pub struct TickerRegistrationConfig<U> {
    pub max_ticker_length: u8,
    pub registration_length: Option<U>,
}

/// Enum that represents the current status of a ticker
#[derive(codec::Encode, codec::Decode, Clone, Eq, PartialEq, Debug)]
pub enum TickerRegistrationStatus {
    RegisteredByOther,
    Available,
    RegisteredByDid,
}

decl_storage! {
    trait Store for Module<T: Trait> as Asset {
        /// The DID of the fee collector
        FeeCollector get(fee_collector) config(): T::AccountId;
        /// Ticker registration details
        /// (ticker) -> TickerRegistration
        pub Tickers get(ticker_registration): map Ticker => TickerRegistration<T::Moment>;
        /// Ticker registration config
        /// (ticker) -> TickerRegistrationConfig
        pub TickerConfig get(ticker_registration_config) config(): TickerRegistrationConfig<T::Moment>;
        /// details of the token corresponding to the token ticker
        /// (ticker) -> SecurityToken details [returns SecurityToken struct]
        pub Tokens get(token_details): map Ticker => SecurityToken<T::Balance>;
        /// Used to store the securityToken balance corresponds to ticker and Identity
        /// (ticker, DID) -> balance
        pub BalanceOf get(balance_of): map (Ticker, IdentityId) => T::Balance;
        /// A map of asset identifiers whose keys are pairs of a ticker name and an `IdentifierType`
        /// and whose values are byte vectors.
        pub Identifiers get(identifiers): map (Ticker, IdentifierType) => Vec<u8>;
        /// (ticker, sender (DID), spender(DID)) -> allowance amount
        Allowance get(allowance): map (Ticker, IdentityId, IdentityId) => T::Balance;
        /// cost in base currency to create a token
        AssetCreationFee get(asset_creation_fee) config(): T::Balance;
        /// cost in base currency to register a ticker
        TickerRegistrationFee get(ticker_registration_fee) config(): T::Balance;
        /// Checkpoints created per token
        /// (ticker) -> no. of checkpoints
        pub TotalCheckpoints get(total_checkpoints_of): map Ticker => u64;
        /// Total supply of the token at the checkpoint
        /// (ticker, checkpointId) -> total supply at given checkpoint
        pub CheckpointTotalSupply get(total_supply_at): map (Ticker, u64) => T::Balance;
        /// Balance of a DID at a checkpoint
        /// (ticker, DID, checkpoint ID) -> Balance of a DID at a checkpoint
        CheckpointBalance get(balance_at_checkpoint): map (Ticker, IdentityId, u64) => T::Balance;
        /// Last checkpoint updated for a DID's balance
        /// (ticker, DID) -> List of checkpoints where user balance changed
        UserCheckpoints get(user_checkpoints): map (Ticker, IdentityId) => Vec<u64>;
        /// The documents attached to the tokens
        /// (ticker, document name) -> (URI, document hash)
        Documents get(documents): map (Ticker, Vec<u8>) => (Vec<u8>, Vec<u8>, T::Moment);
        /// Allowance provided to the custodian
        /// (ticker, token holder, custodian) -> balance
        pub CustodianAllowance get(custodian_allowance): map(Ticker, IdentityId, IdentityId) => T::Balance;
        /// Total custodian allowance for a given token holder
        /// (ticker, token holder) -> balance
        pub TotalCustodyAllowance get(total_custody_allowance): map(Ticker, IdentityId) => T::Balance;
        /// Store the nonce for off chain signature to increase the custody allowance
        /// (ticker, token holder, nonce) -> bool
        AuthenticationNonce get(authentication_nonce): map(Ticker, IdentityId, u16) => bool;
        /// The name of the current funding round.
        /// ticker -> funding round
        FundingRound get(funding_round): map Ticker => Vec<u8>;
        /// The total balances of tokens issued in all recorded funding rounds.
        /// (ticker, funding round) -> balance
        IssuedInFundingRound get(issued_in_funding_round): map (Ticker, Vec<u8>) => T::Balance;
    }
}

// public interface for this runtime module
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// initialize the default event for this module
        fn deposit_event() = default;

        /// This function is used to either register a new ticker or extend validity of an exisitng ticker
        /// NB Ticker validity does not get carryforward when renewing ticker
        ///
        /// # Arguments
        /// * `origin` It contains the signing key of the caller (i.e who signed the transaction to execute this function)
        /// * `ticker` ticker to register
        pub fn register_ticker(origin, ticker: Ticker) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from(sender.encode())?;
            let signer = Signer::Key( sender_key.clone());
            let to_did =  match <identity::Module<T>>::current_did() {
                Some(x) => x,
                None => {
                    if let Some(did) = <identity::Module<T>>::get_identity(&sender_key) {
                        did
                    } else {
                        return Err("did not found");
                    }
                }
            };

            ticker.canonize();
            ensure!(<identity::Module<T>>::is_signer_authorized(to_did, &signer), "sender must be a signing key for DID");

            ensure!(!<Tokens<T>>::exists(&ticker), "token already created");

            let ticker_config = Self::ticker_registration_config();

            ensure!(ticker.len() <= usize::try_from(ticker_config.max_ticker_length).unwrap_or_default(), "ticker length over the limit");

            // Ensure that the ticker is not registered by someone else
            ensure!(
                Self::is_ticker_available_or_registered_to(&ticker, to_did) != TickerRegistrationStatus::RegisteredByOther,
                "ticker registered to someone else"
            );

            let now = <timestamp::Module<T>>::get();
            let expiry = if let Some(exp) = ticker_config.registration_length { Some(now + exp) } else { None };

            Self::_register_ticker(&ticker, sender, to_did, expiry);

            Ok(())
        }

        /// This function is used to accept a ticker transfer
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the signing key of the caller (i.e who signed the transaction to execute this function)
        /// * `auth_id` Authorization ID of ticker transfer authorization
        pub fn accept_ticker_transfer(origin, auth_id: u64) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from(sender.encode())?;
            let to_did =  match <identity::Module<T>>::current_did() {
                Some(x) => x,
                None => {
                    if let Some(did) = <identity::Module<T>>::get_identity(&sender_key) {
                        did
                    } else {
                        return Err("did not found");
                    }
                }
            };
            Self::_accept_ticker_transfer(to_did, auth_id)
        }

        /// This function is used to accept a token ownership transfer
        /// NB: To reject the transfer, call remove auth function in identity module.
        ///
        /// # Arguments
        /// * `origin` It contains the signing key of the caller (i.e who signed the transaction to execute this function)
        /// * `auth_id` Authorization ID of the token ownership transfer authorization
        pub fn accept_token_ownership_transfer(origin, auth_id: u64) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from(sender.encode())?;
            let to_did =  match <identity::Module<T>>::current_did() {
                Some(x) => x,
                None => {
                    if let Some(did) = <identity::Module<T>>::get_identity(&sender_key) {
                        did
                    } else {
                        return Err("did not found");
                    }
                }
            };
            Self::_accept_token_ownership_transfer(to_did, auth_id)
        }

        /// Initializes a new security token
        /// makes the initiating account the owner of the security token
        /// & the balance of the owner is set to total supply
        ///
        /// # Arguments
        /// * `origin` It contains the signing key of the caller (i.e who signed the transaction to execute this function)
        /// * `did` DID of the creator of the token or the owner of the token
        /// * `name` Name of the token
        /// * `ticker` Symbol of the token
        /// * `total_supply` Total supply of the token
        /// * `divisible` boolean to identify the divisibility status of the token.
        pub fn create_token(
            origin,
            did: IdentityId,
            name: Vec<u8>,
            ticker: Ticker,
            total_supply: T::Balance,
            divisible: bool,
            asset_type: AssetType,
            identifiers: Vec<(IdentifierType, Vec<u8>)>
        ) -> Result {
            let sender = ensure_signed(origin)?;
            let signer = Signer::Key(Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");
            ticker.canonize();
            ensure!(!<Tokens<T>>::exists(&ticker), "token already created");

            let ticker_config = Self::ticker_registration_config();

            ensure!(ticker.len() <= usize::try_from(ticker_config.max_ticker_length).unwrap_or_default(), "ticker length over the limit");

            // checking max size for name and ticker
            // byte arrays (vecs) with no max size should be avoided
            ensure!(name.len() <= 64, "token name cannot exceed 64 bytes");

            let is_ticker_available_or_registered_to = Self::is_ticker_available_or_registered_to(&ticker, did);

            ensure!(is_ticker_available_or_registered_to != TickerRegistrationStatus::RegisteredByOther, "Ticker registered to someone else");

            if !divisible {
                ensure!(total_supply % ONE_UNIT.into() == 0.into(), "Invalid Total supply");
            }

            ensure!(total_supply <= MAX_SUPPLY.into(), "Total supply above the limit");

            // Alternative way to take a fee - fee is proportionaly paid to the validators and dust is burned
            let validators = <session::Module<T>>::validators();
            let fee = Self::asset_creation_fee();
            let validator_len:T::Balance;
            if validators.len() < 1 {
                validator_len = T::Balance::from(1 as u32);
            } else {
                validator_len = T::Balance::from(validators.len() as u32);
            }
            let proportional_fee = fee / validator_len;
            for v in validators {
                <balances::Module<T> as Currency<_>>::transfer(
                    &sender,
                    &<T as utils::Trait>::validator_id_to_account_id(v),
                    proportional_fee
                )?;
            }
            let remainder_fee = fee - (proportional_fee * validator_len);
            let _withdraw_result = <balances::Module<T>>::withdraw(&sender, remainder_fee, WithdrawReason::Fee, ExistenceRequirement::KeepAlive)?;
            <identity::Module<T>>::register_asset_did(&ticker)?;
            if is_ticker_available_or_registered_to == TickerRegistrationStatus::Available {
                // ticker not registered by anyone (or registry expired). we can charge fee and register this ticker
                Self::_register_ticker(&ticker, sender, did, None);
            } else {
                // Ticker already registered by the user
                <Tickers<T>>::mutate(&ticker, |tr| tr.expiry = None);
            }

            let token = SecurityToken {
                name,
                total_supply,
                owner_did: did,
                divisible,
                asset_type: asset_type.clone(),
            };
            <Tokens<T>>::insert(&ticker, token);
            <BalanceOf<T>>::insert((ticker, did), total_supply);
            Self::deposit_event(RawEvent::IssuedToken(
                ticker,
                total_supply,
                did,
                divisible,
                asset_type,
            ));
            for (typ, val) in &identifiers {
                <Identifiers>::insert((ticker, typ.clone()), val.clone());
            }
            Self::deposit_event(RawEvent::IdentifiersUpdated(ticker, identifiers));

            Ok(())
        }

        /// Renames a given token.
        ///
        /// # Arguments
        /// * `origin` - the signing key of the sender
        /// * `ticker` - the ticker of the token
        /// * `name` - the new name of the token
        pub fn rename_token(origin, ticker: Ticker, name: Vec<u8>) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;
            let signer = Signer::Key(Key::try_from(sender.encode())?);
            ensure!(<Tokens<T>>::exists(&ticker), "token doesn't exist");
            let token = <Tokens<T>>::get(&ticker);
            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(token.owner_did, &signer), "sender must be a signing key for the token owner DID");
            <Tokens<T>>::mutate(&ticker, |token| token.name = name.clone());
            Self::deposit_event(RawEvent::TokenRenamed(ticker, name));
            Ok(())
        }

        /// Transfer tokens from one DID to another DID as tokens are stored/managed on the DID level
        ///
        /// # Arguments
        /// * `_origin` signing key of the sender
        /// * `did` DID of the `from` token holder, from whom tokens needs to transferred
        /// * `ticker` Ticker of the token
        /// * `to_did` DID of the `to` token holder, to whom token needs to transferred
        /// * `value` Value that needs to transferred
        pub fn transfer(_origin, did: IdentityId, ticker: Ticker, to_did: IdentityId, value: T::Balance) -> Result {
            ticker.canonize();
            let sender = ensure_signed(_origin)?;
            let signer = Signer::Key( Key::try_from(sender.encode())?);


            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&ticker, did, value)?;
            ensure!(Self::_is_valid_transfer(&ticker, Some(did), Some(to_did), value)? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");

            Self::_transfer(&ticker, did, to_did, value)
        }

        /// Forces a transfer between two DIDs & This can only be called by security token owner.
        /// This function doesn't validate any type of restriction beside a valid KYC check
        ///
        /// # Arguments
        /// * `_origin` signing key of the token owner DID.
        /// * `did` Token owner DID.
        /// * `ticker` symbol of the token
        /// * `from_did` DID of the token holder from whom balance token will be transferred.
        /// * `to_did` DID of token holder to whom token balance will be transferred.
        /// * `value` Amount of tokens.
        /// * `data` Some off chain data to validate the restriction.
        /// * `operator_data` It is a string which describes the reason of this control transfer call.
        pub fn controller_transfer(_origin, did: IdentityId, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, value: T::Balance, data: Vec<u8>, operator_data: Vec<u8>) -> Result {
            ticker.canonize();
            let sender = ensure_signed(_origin)?;
            let signer = Signer::Key( Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            ensure!(Self::is_owner(&ticker, did), "user is not authorized");

            Self::_transfer(&ticker, from_did, to_did, value.clone())?;

            Self::deposit_event(RawEvent::ControllerTransfer(ticker, did, from_did, to_did, value, data, operator_data));

            Ok(())
        }

        /// approve token transfer from one DID to DID
        /// once this is done, transfer_from can be called with corresponding values
        ///
        /// # Arguments
        /// * `_origin` Signing key of the token owner (i.e sender)
        /// * `did` DID of the sender
        /// * `spender_did` DID of the spender
        /// * `value` Amount of the tokens approved
        fn approve(_origin, did: IdentityId, ticker: Ticker, spender_did: IdentityId, value: T::Balance) -> Result {
            ticker.canonize();
            let sender = ensure_signed(_origin)?;
            let signer = Signer::Key(Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            ensure!(<BalanceOf<T>>::exists((ticker, did)), "Account does not own this token");

            let allowance = Self::allowance((ticker, did, spender_did));
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((ticker, did, spender_did), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker, did, spender_did, value));

            Ok(())
        }

        /// If sufficient allowance provided, transfer from a DID to another DID without token owner's signature.
        ///
        /// # Arguments
        /// * `_origin` Signing key of spender
        /// * `did` DID of the spender
        /// * `_ticker` Ticker of the token
        /// * `from_did` DID from whom token is being transferred
        /// * `to_did` DID to whom token is being transferred
        /// * `value` Amount of the token for transfer
        pub fn transfer_from(origin, did: IdentityId, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, value: T::Balance) -> Result {
            let spender = Signer::Key(Key::try_from(ensure_signed(origin)?.encode())?);

            // Check that spender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &spender), "sender must be a signing key for DID");

            ticker.canonize();
            let ticker_from_did_did = (ticker, from_did, did);
            ensure!(<Allowance<T>>::exists(&ticker_from_did_did), "Allowance does not exist");
            let allowance = Self::allowance(&ticker_from_did_did);
            ensure!(allowance >= value, "Not enough allowance");

            // using checked_sub (safe math) to avoid overflow
            let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;
            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&ticker, from_did, value)?;

            ensure!(Self::_is_valid_transfer(&ticker, Some(from_did), Some(to_did), value)? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");
            Self::_transfer(&ticker, from_did, to_did, value)?;

            // Change allowance afterwards
            <Allowance<T>>::insert(&ticker_from_did_did, updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker, from_did, did, value));
            Ok(())
        }

        /// Function used to create the checkpoint
        ///
        /// # Arguments
        /// * `_origin` Signing key of the token owner. (Only token owner can call this function).
        /// * `did` DID of the token owner
        /// * `_ticker` Ticker of the token
        pub fn create_checkpoint(_origin, did: IdentityId, ticker: Ticker) -> Result {
            ticker.canonize();
            let sender = ensure_signed(_origin)?;
            let signer = Signer::Key( Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            ensure!(Self::is_owner(&ticker, did), "user is not authorized");
            Self::_create_checkpoint(&ticker)
        }

        /// Function is used to issue(or mint) new tokens for the given DID
        /// can only be executed by the token owner
        ///
        /// # Arguments
        /// * `origin` Signing key of token owner
        /// * `did` DID of the token owner
        /// * `ticker` Ticker of the token
        /// * `to_did` DID of the token holder to whom new tokens get issued.
        /// * `value` Amount of tokens that get issued
        pub fn issue(origin, did: IdentityId, ticker: Ticker, to_did: IdentityId, value: T::Balance, _data: Vec<u8>) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;
            let signer = Signer::Key( Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            ensure!(Self::is_owner(&ticker, did), "user is not authorized");
            Self::_mint(&ticker, to_did, value)
        }

        /// Function is used issue(or mint) new tokens for the given DIDs
        /// can only be executed by the token owner
        ///
        /// # Arguments
        /// * `origin` Signing key of token owner
        /// * `did` DID of the token owner
        /// * `ticker` Ticker of the token
        /// * `investor_dids` Array of the DID of the token holders to whom new tokens get issued.
        /// * `values` Array of the Amount of tokens that get issued
        pub fn batch_issue(origin, did: IdentityId, ticker: Ticker, investor_dids: Vec<IdentityId>, values: Vec<T::Balance>) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;
            let signer = Signer::Key(Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            ensure!(investor_dids.len() == values.len(), "Investor/amount list length inconsistent");

            ensure!(Self::is_owner(&ticker, did), "user is not authorized");


            // A helper vec for calculated new investor balances
            let mut updated_balances = Vec::with_capacity(investor_dids.len());

            // A helper vec for calculated new investor balances
            let mut current_balances = Vec::with_capacity(investor_dids.len());

            // Get current token details for supply update
            let mut token = Self::token_details(ticker);

            // A round of per-investor checks
            for i in 0..investor_dids.len() {
                ensure!(
                    Self::check_granularity(&ticker, values[i]),
                    "Invalid granularity"
                );
                let updated_total_supply = token
                    .total_supply
                    .checked_add(&values[i])
                    .ok_or("overflow in calculating total supply")?;
                ensure!(updated_total_supply <= MAX_SUPPLY.into(), "Total supply above the limit");

                current_balances.push(Self::balance_of((ticker, investor_dids[i].clone())));
                updated_balances.push(current_balances[i]
                    .checked_add(&values[i])
                    .ok_or("overflow in calculating balance")?);

                // verify transfer check
                ensure!(Self::_is_valid_transfer(&ticker, None, Some(investor_dids[i]), values[i])? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");

                // New total supply must be valid
                token.total_supply = updated_total_supply;
            }
            let round = Self::funding_round(&ticker);
            let ticker_round = (ticker, round.clone());
            // Update the total token balance issued in this funding round.
            let mut issued_in_this_round = Self::issued_in_funding_round(&ticker_round);
            for v in &values {
                issued_in_this_round = issued_in_this_round
                    .checked_add(v)
                    .ok_or("current funding round total overflowed")?;
            }
            <IssuedInFundingRound<T>>::insert(&ticker_round, issued_in_this_round);
            // Update investor balances and emit events quoting the updated total token balance issued.
            for i in 0..investor_dids.len() {
                Self::_update_checkpoint(&ticker, investor_dids[i], current_balances[i]);
                <BalanceOf<T>>::insert((ticker, investor_dids[i]), updated_balances[i]);
                Self::deposit_event(RawEvent::Issued(
                    ticker,
                    investor_dids[i],
                    values[i],
                    round.clone(),
                    issued_in_this_round
                ));
            }
            <Tokens<T>>::insert(ticker, token);

            Ok(())
        }

        /// Used to redeem the security tokens
        ///
        /// # Arguments
        /// * `_origin` Signing key of the token holder who wants to redeem the tokens
        /// * `did` DID of the token holder
        /// * `ticker` Ticker of the token
        /// * `value` Amount of the tokens needs to redeem
        /// * `_data` An off chain data blob used to validate the redeem functionality.
        pub fn redeem(_origin, did: IdentityId, ticker: Ticker, value: T::Balance, _data: Vec<u8>) -> Result {
            ticker.canonize();
            let sender = ensure_signed(_origin)?;
            let signer = Signer::Key( Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            // Granularity check
            ensure!(
                Self::check_granularity(&ticker, value),
                "Invalid granularity"
                );
            let ticker_did = (ticker, did);
            ensure!(<BalanceOf<T>>::exists(&ticker_did), "Account does not own this token");
            let burner_balance = Self::balance_of(&ticker_did);
            ensure!(burner_balance >= value, "Not enough balance.");

            // Reduce sender's balance
            let updated_burner_balance = burner_balance
                .checked_sub(&value)
                .ok_or("overflow in calculating balance")?;
            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&ticker, did, value)?;

            // verify transfer check
            ensure!(Self::_is_valid_transfer(&ticker, Some(did), None, value)? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");

            //Decrease total supply
            let mut token = Self::token_details(&ticker);
            token.total_supply = token.total_supply.checked_sub(&value).ok_or("overflow in calculating balance")?;

            Self::_update_checkpoint(&ticker, did, burner_balance);

            <BalanceOf<T>>::insert((ticker, did), updated_burner_balance);
            <Tokens<T>>::insert(&ticker, token);

            Self::deposit_event(RawEvent::Redeemed(ticker, did, value));

            Ok(())

        }

        /// Used to redeem the security tokens by some other DID who has approval
        ///
        /// # Arguments
        /// * `_origin` Signing key of the spender who has valid approval to redeem the tokens
        /// * `did` DID of the spender
        /// * `ticker` Ticker of the token
        /// * `from_did` DID from whom balance get reduced
        /// * `value` Amount of the tokens needs to redeem
        /// * `_data` An off chain data blob used to validate the redeem functionality.
        pub fn redeem_from(_origin, did: IdentityId, ticker: Ticker, from_did: IdentityId, value: T::Balance, _data: Vec<u8>) -> Result {
            ticker.canonize();
            let sender = ensure_signed(_origin)?;
            let signer = Signer::Key( Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            // Granularity check
            ensure!(
                Self::check_granularity(&ticker, value),
                "Invalid granularity"
                );
            let ticker_did = (ticker, did);
            ensure!(<BalanceOf<T>>::exists(&ticker_did), "Account does not own this token");
            let burner_balance = Self::balance_of(&ticker_did);
            ensure!(burner_balance >= value, "Not enough balance.");

            // Reduce sender's balance
            let updated_burner_balance = burner_balance
                .checked_sub(&value)
                .ok_or("overflow in calculating balance")?;

            let ticker_from_did_did = (ticker, from_did, did);
            ensure!(<Allowance<T>>::exists(&ticker_from_did_did), "Allowance does not exist");
            let allowance = Self::allowance(&ticker_from_did_did);
            ensure!(allowance >= value, "Not enough allowance");
            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&ticker, did, value)?;
            ensure!(Self::_is_valid_transfer(&ticker, Some(from_did), None, value)? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");

            let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;

            //Decrease total suply
            let mut token = Self::token_details(&ticker);
            token.total_supply = token.total_supply.checked_sub(&value).ok_or("overflow in calculating balance")?;

            Self::_update_checkpoint(&ticker, did, burner_balance);

            <Allowance<T>>::insert(&ticker_from_did_did, updated_allowance);
            <BalanceOf<T>>::insert(&ticker_did, updated_burner_balance);
            <Tokens<T>>::insert(&ticker, token);

            Self::deposit_event(RawEvent::Redeemed(ticker, did, value));
            Self::deposit_event(RawEvent::Approval(ticker, from_did, did, value));

            Ok(())
        }

        /// Forces a redemption of an DID's tokens. Can only be called by token owner
        ///
        /// # Arguments
        /// * `_origin` Signing key of the token owner
        /// * `did` DID of the token holder
        /// * `ticker` Ticker of the token
        /// * `token_holder_did` DID from whom balance get reduced
        /// * `value` Amount of the tokens needs to redeem
        /// * `data` An off chain data blob used to validate the redeem functionality.
        /// * `operator_data` Any data blob that defines the reason behind the force redeem.
        pub fn controller_redeem(origin, did: IdentityId, ticker: Ticker, token_holder_did: IdentityId, value: T::Balance, data: Vec<u8>, operator_data: Vec<u8>) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;
            let signer = Signer::Key( Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");
            ensure!(Self::is_owner(&ticker, did), "user is not token owner");

            // Granularity check
            ensure!(
                Self::check_granularity(&ticker, value),
                "Invalid granularity"
                );
            let ticker_token_holder_did = (ticker, token_holder_did);
            ensure!(<BalanceOf<T>>::exists(&ticker_token_holder_did), "Account does not own this token");
            let burner_balance = Self::balance_of(&ticker_token_holder_did);
            ensure!(burner_balance >= value, "Not enough balance.");

            // Reduce sender's balance
            let updated_burner_balance = burner_balance
                .checked_sub(&value)
                .ok_or("overflow in calculating balance")?;

            //Decrease total suply
            let mut token = Self::token_details(&ticker);
            token.total_supply = token.total_supply.checked_sub(&value).ok_or("overflow in calculating balance")?;

            Self::_update_checkpoint(&ticker, token_holder_did, burner_balance);

            <BalanceOf<T>>::insert(&ticker_token_holder_did, updated_burner_balance);
            <Tokens<T>>::insert(&ticker, token);

            Self::deposit_event(RawEvent::ControllerRedemption(ticker, did, token_holder_did, value, data, operator_data));

            Ok(())
        }

        /// Makes an indivisible token divisible. Only called by the token owner
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner.
        /// * `did` DID of the token owner
        /// * `ticker` Ticker of the token
        pub fn make_divisible(origin, did: IdentityId, ticker: Ticker) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;
            let sender_signer = Signer::Key(Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender_signer), "sender must be a signing key for DID");

            ensure!(Self::is_owner(&ticker, did), "user is not authorized");
            // Read the token details
            let mut token = Self::token_details(&ticker);
            ensure!(!token.divisible, "token already divisible");
            token.divisible = true;
            <Tokens<T>>::insert(&ticker, token);
            Self::deposit_event(RawEvent::DivisibilityChanged(ticker, true));
            Ok(())
        }

        /// Checks whether a transaction with given parameters can take place or not
        /// This function is state less function and used to validate the transfer before actual transfer call.
        ///
        /// # Arguments
        /// * `_origin` Signing Key of the caller
        /// * `ticker` Ticker of the token
        /// * `from_did` DID from whom tokens will be transferred
        /// * `to_did` DID to whom tokens will be transferred
        /// * `value` Amount of the tokens
        /// * `data` Off chain data blob to validate the transfer.
        pub fn can_transfer(_origin, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, value: T::Balance, data: Vec<u8>) {
            ticker.canonize();
            let mut current_balance: T::Balance = Self::balance_of((ticker, from_did));
            if current_balance < value {
                current_balance = 0.into();
            } else {
                current_balance = current_balance - value;
            }
            if current_balance < Self::total_custody_allowance((ticker, from_did)) {
                sr_primitives::print("Insufficient balance");
                Self::deposit_event(RawEvent::CanTransfer(ticker, from_did, to_did, value, data, ERC1400_INSUFFICIENT_BALANCE as u32));
            } else {
                match Self::_is_valid_transfer(&ticker, Some(from_did), Some(to_did), value) {
                    Ok(code) =>
                    {
                        Self::deposit_event(RawEvent::CanTransfer(ticker, from_did, to_did, value, data, code as u32));
                    },
                    Err(msg) => {
                        // We emit a generic error with the event whenever there's an internal issue - i.e. captured
                        // in a string error and not using the status codes
                        sr_primitives::print(msg);
                        Self::deposit_event(RawEvent::CanTransfer(ticker, from_did, to_did, value, data, ERC1400_TRANSFER_FAILURE as u32));
                    }
                }
            }
        }

        /// An ERC1594 transfer with data
        /// This function can be used by the exchanges of other third parties to dynamically validate the transaction
        /// by passing the data blob
        ///
        /// # Arguments
        /// * `origin` Signing key of the sender
        /// * `did` DID from whom tokens will be transferred
        /// * `ticker` Ticker of the token
        /// * `to_did` DID to whom tokens will be transferred
        /// * `value` Amount of the tokens
        /// * `data` Off chain data blob to validate the transfer.
        pub fn transfer_with_data(origin, did: IdentityId, ticker: Ticker, to_did: IdentityId, value: T::Balance, data: Vec<u8>) -> Result {
            ticker.canonize();
            Self::transfer(origin, did, ticker, to_did, value)?;
            Self::deposit_event(RawEvent::TransferWithData(ticker, did, to_did, value, data));
            Ok(())
        }

        /// An ERC1594 transfer_from with data
        /// This function can be used by the exchanges of other third parties to dynamically validate the transaction
        /// by passing the data blob
        ///
        /// # Arguments
        /// * `origin` Signing key of the spender
        /// * `did` DID of spender
        /// * `ticker` Ticker of the token
        /// * `from_did` DID from whom tokens will be transferred
        /// * `to_did` DID to whom tokens will be transferred
        /// * `value` Amount of the tokens
        /// * `data` Off chain data blob to validate the transfer.
        pub fn transfer_from_with_data(origin, did: IdentityId, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, value: T::Balance, data: Vec<u8>) -> Result {
            ticker.canonize();
            Self::transfer_from(origin, did, ticker, from_did,  to_did, value)?;
            Self::deposit_event(RawEvent::TransferWithData(ticker, from_did, to_did, value, data));
            Ok(())
        }

        /// Used to know whether the given token will issue new tokens or not
        ///
        /// # Arguments
        /// * `_origin` Signing key
        /// * `ticker` Ticker of the token whose issuance status need to know
        pub fn is_issuable(_origin, ticker:Ticker) {
            ticker.canonize();
            Self::deposit_event(RawEvent::IsIssuable(ticker, true));
        }

        /// Used to get the documents details attach with the token
        ///
        /// # Arguments
        /// * `_origin` Caller signing key
        /// * `ticker` Ticker of the token
        /// * `name` Name of the document
        pub fn get_document(_origin, ticker: Ticker, name: Vec<u8>) -> Result {
            ticker.canonize();
            let record = <Documents<T>>::get((ticker, name.clone()));
            Self::deposit_event(RawEvent::GetDocument(ticker, name, record.0, record.1, record.2));
            Ok(())
        }

        /// Used to set the details of the document, Only be called by the token owner
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner
        /// * `did` DID of the token owner
        /// * `ticker` Ticker of the token
        /// * `name` Name of the document
        /// * `uri` Off chain URL of the document
        /// * `document_hash` Hash of the document to proof the incorruptibility of the document
        pub fn set_document(origin, did: IdentityId, ticker: Ticker, name: Vec<u8>, uri: Vec<u8>, document_hash: Vec<u8>) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;
            let sender_signer = Signer::Key( Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender_signer), "sender must be a signing key for DID");
            ensure!(Self::is_owner(&ticker, did), "user is not authorized");

            <Documents<T>>::insert((ticker, name), (uri, document_hash, <timestamp::Module<T>>::get()));
            Ok(())
        }

        /// Used to remove the document details for the given token, Only be called by the token owner
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner
        /// * `did` DID of the token owner
        /// * `ticker` Ticker of the token
        /// * `name` Name of the document
        pub fn remove_document(origin, did: IdentityId, ticker: Ticker, name: Vec<u8>) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;
            let sender_signer = Signer::Key( Key::try_from(sender.encode())?);


            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender_signer), "sender must be a signing key for DID");
            ensure!(Self::is_owner(&ticker, did), "user is not authorized");

            <Documents<T>>::remove((ticker, name));
            Ok(())
        }

        /// ERC-2258 Implementation

        /// Used to increase the allowance for a given custodian
        /// Any investor/token holder can add a custodian and transfer the token transfer ownership to the custodian
        /// Through that investor balance will remain the same but the given token are only transfer by the custodian.
        /// This implementation make sure to have an accurate investor count from omnibus wallets.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token holder
        /// * `ticker` Ticker of the token
        /// * `holder_did` DID of the token holder (i.e who wants to increase the custody allowance)
        /// * `custodian_did` DID of the custodian (i.e whom allowance provided)
        /// * `value` Allowance amount
        pub fn increase_custody_allowance(origin, ticker: Ticker, holder_did: IdentityId, custodian_did: IdentityId, value: T::Balance) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;
            let sender_signer = Signer::Key( Key::try_from(sender.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(holder_did, &sender_signer),
                "sender must be a signing key for DID"
            );
            Self::_increase_custody_allowance(ticker, holder_did, custodian_did, value)?;
            Ok(())
        }

        /// Used to increase the allowance for a given custodian by providing the off chain signature
        ///
        /// # Arguments
        /// * `origin` Signing key of a DID who posses off chain signature
        /// * `ticker` Ticker of the token
        /// * `holder_did` DID of the token holder (i.e who wants to increase the custody allowance)
        /// * `holder_account_id` Signing key which signs the off chain data blob.
        /// * `custodian_did` DID of the custodian (i.e whom allowance provided)
        /// * `caller_did` DID of the caller
        /// * `value` Allowance amount
        /// * `nonce` A u16 number which avoid the replay attack
        /// * `signature` Signature provided by the holder_did
        pub fn increase_custody_allowance_of(
            origin,
            ticker: Ticker,
            holder_did: IdentityId,
            holder_account_id: T::AccountId,
            custodian_did: IdentityId,
            caller_did: IdentityId,
            value: T::Balance,
            nonce: u16,
            signature: T::OffChainSignature
        ) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;

            ensure!(!Self::authentication_nonce((ticker, holder_did, nonce)), "Signature already used");

            let msg = SignData {
                custodian_did: custodian_did,
                holder_did: holder_did,
                ticker,
                value,
                nonce
            };
            // holder_account_id should be a part of the holder_did
            ensure!(signature.verify(&msg.encode()[..], &holder_account_id), "Invalid signature");
            let sender_signer = Signer::Key(Key::try_from(sender.encode())?);
            ensure!(
                <identity::Module<T>>::is_signer_authorized(caller_did, &sender_signer),
                "sender must be a signing key for DID"
            );
            // Validate the holder signing key
            let holder_signer = Signer::Key(Key::try_from(holder_account_id.encode())?);
            ensure!(
                <identity::Module<T>>::is_signer_authorized(holder_did, &holder_signer),
                "holder signing key must be a signing key for holder DID"
            );
            Self::_increase_custody_allowance(ticker, holder_did, custodian_did, value)?;
            <AuthenticationNonce>::insert((ticker, holder_did, nonce), true);
            Ok(())
        }

        /// Used to transfer the tokens by the approved custodian
        ///
        /// # Arguments
        /// * `origin` Signing key of the custodian
        /// * `ticker` Ticker of the token
        /// * `holder_did` DID of the token holder (i.e whom balance get reduced)
        /// * `custodian_did` DID of the custodian (i.e who has the valid approved allowance)
        /// * `receiver_did` DID of the receiver
        /// * `value` Amount of tokens need to transfer
        pub fn transfer_by_custodian(
            origin,
            ticker: Ticker,
            holder_did: IdentityId,
            custodian_did: IdentityId,
            receiver_did: IdentityId,
            value: T::Balance
        ) -> Result {
            ticker.canonize();
            let sender = ensure_signed(origin)?;
            let sender_signer = Signer::Key( Key::try_from(sender.encode())?);
            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(custodian_did, &sender_signer),
                "sender must be a signing key for DID"
            );
            let mut custodian_allowance = Self::custodian_allowance((ticker, holder_did, custodian_did));
            // Check whether the custodian has enough allowance or not
            ensure!(custodian_allowance >= value, "Insufficient allowance");
            // using checked_sub (safe math) to avoid underflow
            custodian_allowance = custodian_allowance.checked_sub(&value).ok_or("underflow in calculating allowance")?;
            // using checked_sub (safe math) to avoid underflow
            let new_total_allowance = Self::total_custody_allowance((ticker, holder_did))
                .checked_sub(&value)
                .ok_or("underflow in calculating the total allowance")?;
            // Validate the transfer
            ensure!(Self::_is_valid_transfer(&ticker, Some(holder_did), Some(receiver_did), value)? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");
            Self::_transfer(&ticker, holder_did, receiver_did, value)?;
            // Update Storage of allowance
            <CustodianAllowance<T>>::insert((ticker, custodian_did, holder_did), &custodian_allowance);
            <TotalCustodyAllowance<T>>::insert((ticker, holder_did), new_total_allowance);
            Self::deposit_event(RawEvent::CustodyTransfer(ticker, custodian_did, holder_did, receiver_did, value));
            Ok(())
        }

        /// Sets the name of the current funding round.
        ///
        /// # Arguments
        /// * `origin` - the signing key of the token owner DID.
        /// * `did` - the token owner DID.
        /// * `ticker` - the ticker of the token.
        /// * `name` - the desired name of the current funding round.
        pub fn set_funding_round(origin, did: IdentityId, ticker: Vec<u8>, name: Vec<u8>) -> Result {
            let ticker = utils::bytes_to_upper(ticker.as_slice());
            let sender = ensure_signed(origin)?;
            let signer = Signer::Key(Key::try_from(sender.encode())?);
            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer),
                    "sender must be a signing key for DID");
            ensure!(Self::is_owner(&ticker, did), "DID is not of the asset owner");
            <FundingRound>::insert(ticker.clone(), name.clone());
            Self::deposit_event(RawEvent::FundingRound(ticker, name));
            Ok(())
        }

        /// Updates the asset identifiers. Can only be called by the token owner.
        ///
        /// # Arguments
        /// * `origin` - the signing key of the token owner
        /// * `did` - the DID of the token owner
        /// * `ticker` - the ticker of the token
        /// * `identifiers` - the asset identifiers to be updated in the form of a vector of pairs
        ///    of `IdentifierType` and `Vec<u8>` value.
        pub fn update_identifiers(
            origin,
            did: IdentityId,
            ticker: Ticker,
            identifiers: Vec<(IdentifierType, Vec<u8>)>
        ) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_signer = Signer::Key(Key::try_from(sender.encode())?);
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender_signer),
                    "sender must be a signing key for DID");
            ticker.canonize();
            ensure!(Self::is_owner(&ticker, did), "user is not authorized");
            for (typ, val) in &identifiers {
                <Identifiers>::insert((ticker, typ.clone()), val.clone());
            }
            Self::deposit_event(RawEvent::IdentifiersUpdated(ticker, identifiers));
            Ok(())
        }
    }
}

decl_event! {
    pub enum Event<T>
        where
        Balance = <T as balances::Trait>::Balance,
        Moment = <T as timestamp::Trait>::Moment,
    {
        /// event for transfer of tokens
        /// ticker, from DID, to DID, value
        Transfer(Ticker, IdentityId, IdentityId, Balance),
        /// event when an approval is made
        /// ticker, owner DID, spender DID, value
        Approval(Ticker, IdentityId, IdentityId, Balance),
        /// emit when tokens get issued
        /// ticker, beneficiary DID, value, funding round, total issued in this funding round
        Issued(Ticker, IdentityId, Balance, Vec<u8>, Balance),
        /// emit when tokens get redeemed
        /// ticker, DID, value
        Redeemed(Ticker, IdentityId, Balance),
        /// event for forced transfer of tokens
        /// ticker, controller DID, from DID, to DID, value, data, operator data
        ControllerTransfer(Ticker, IdentityId, IdentityId, IdentityId, Balance, Vec<u8>, Vec<u8>),
        /// event for when a forced redemption takes place
        /// ticker, controller DID, token holder DID, value, data, operator data
        ControllerRedemption(Ticker, IdentityId, IdentityId, Balance, Vec<u8>, Vec<u8>),
        /// Event for creation of the asset
        /// ticker, total supply, owner DID, divisibility, asset type
        IssuedToken(Ticker, Balance, IdentityId, bool, AssetType),
        /// Event emitted when a token identifiers are updated.
        /// ticker, a vector of (identifier type, identifier value)
        IdentifiersUpdated(Ticker, Vec<(IdentifierType, Vec<u8>)>),
        /// Event for change in divisibility
        /// ticker, divisibility
        DivisibilityChanged(Ticker, bool),
        /// can_transfer() output
        /// ticker, from_did, to_did, value, data, ERC1066 status
        /// 0 - OK
        /// 1,2... - Error, meanings TBD
        CanTransfer(Ticker, IdentityId, IdentityId, Balance, Vec<u8>, u32),
        /// An additional event to Transfer; emitted when transfer_with_data is called; similar to
        /// Transfer with data added at the end.
        /// ticker, from DID, to DID, value, data
        TransferWithData(Ticker, IdentityId, IdentityId, Balance, Vec<u8>),
        /// is_issuable() output
        /// ticker, return value (true if issuable)
        IsIssuable(Ticker, bool),
        /// get_document() output
        /// ticker, name, uri, hash, last modification date
        GetDocument(Ticker, Vec<u8>, Vec<u8>, Vec<u8>, Moment),
        /// emit when tokens transferred by the custodian
        /// ticker, custodian did, holder/from did, to did, amount
        CustodyTransfer(Ticker, IdentityId, IdentityId, IdentityId, Balance),
        /// emit when allowance get increased
        /// ticker, holder did, custodian did, oldAllowance, newAllowance
        CustodyAllowanceChanged(Ticker, IdentityId, IdentityId, Balance, Balance),
        /// emit when ticker is registered
        /// ticker, ticker owner, expiry
        TickerRegistered(Ticker, IdentityId, Option<Moment>),
        /// emit when ticker is transferred
        /// ticker, from, to
        TickerTransferred(Ticker, IdentityId, IdentityId),
        /// emit when token ownership is transferred
        /// ticker, from, to
        TokenOwnershipTransferred(Ticker, IdentityId, IdentityId),
        /// emit when ticker is registered
        /// ticker, current owner, approved owner
        TickerTransferApproval(Ticker, IdentityId, IdentityId),
        /// ticker transfer approval withdrawal
        /// ticker, approved did
        TickerTransferApprovalWithdrawal(Ticker, IdentityId),
        /// An event emitted when a token is renamed.
        /// Parameters: ticker, new token name.
        TokenRenamed(Ticker, Vec<u8>),
        /// An event carrying the name of the current funding round of a ticker.
        /// Parameters: ticker, funding round name.
        FundingRound(Ticker, Vec<u8>),
    }
}

/// This trait assumes `ticker` converted to the canonical notation.
pub trait AssetTrait<V> {
    fn total_supply(ticker: &Ticker) -> V;
    fn balance(ticker: &Ticker, did: IdentityId) -> V;
    fn _mint_from_sto(ticker: &Ticker, sender_did: IdentityId, tokens_purchased: V) -> Result;
    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool;
    fn get_balance_at(ticker: &Ticker, did: IdentityId, at: u64) -> V;
}

impl<T: Trait> AssetTrait<T::Balance> for Module<T> {
    fn _mint_from_sto(ticker: &Ticker, sender: IdentityId, tokens_purchased: T::Balance) -> Result {
        Self::_mint(ticker, sender, tokens_purchased)
    }

    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool {
        Self::_is_owner(ticker, did)
    }

    /// Get the asset `id` balance of `who`.
    fn balance(ticker: &Ticker, who: IdentityId) -> T::Balance {
        Self::balance_of((*ticker, who))
    }

    // Get the total supply of an asset `id`
    fn total_supply(ticker: &Ticker) -> T::Balance {
        Self::token_details(ticker).total_supply
    }

    fn get_balance_at(ticker: &Ticker, did: IdentityId, at: u64) -> T::Balance {
        Self::get_balance_at(*ticker, did, at)
    }
}

/// This trait is used to call functions that accept transfer of a ticker or token ownership
pub trait AcceptTransfer {
    /// Accept and process a ticker transfer
    ///
    /// # Arguments
    /// * `to_did` did of the receiver
    /// * `auth_id` Authorization id of the authorization created by current ticker owner
    fn accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> Result;
    /// Accept and process a token ownership transfer
    ///
    /// # Arguments
    /// * `to_did` did of the receiver
    /// * `auth_id` Authorization id of the authorization created by current token owner
    fn accept_token_ownership_transfer(to_did: IdentityId, auth_id: u64) -> Result;
}

impl<T: Trait> AcceptTransfer for Module<T> {
    fn accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> Result {
        Self::_accept_ticker_transfer(to_did, auth_id)
    }

    fn accept_token_ownership_transfer(to_did: IdentityId, auth_id: u64) -> Result {
        Self::_accept_token_ownership_transfer(to_did, auth_id)
    }
}

/// All functions in the decl_module macro become part of the public interface of the module
/// If they are there, they are accessible via extrinsics calls whether they are public or not
/// However, in the impl module section (this, below) the functions can be public and private
/// Private functions are internal to this module e.g.: _transfer
/// Public functions can be called from other modules e.g.: lock and unlock (being called from the tcr module)
/// All functions in the impl module section are not part of public interface because they are not part of the Call enum
impl<T: Trait> Module<T> {
    // Public immutables
    pub fn _is_owner(ticker: &Ticker, did: IdentityId) -> bool {
        let token = Self::token_details(ticker);
        token.owner_did == did
    }

    pub fn is_ticker_available(ticker: &Ticker) -> bool {
        // Assumes uppercase ticker
        if <Tickers<T>>::exists(ticker) {
            let now = <timestamp::Module<T>>::get();
            if let Some(expiry) = Self::ticker_registration(*ticker).expiry {
                if now <= expiry {
                    return false;
                }
            } else {
                return false;
            }
        }
        return true;
    }

    pub fn is_ticker_registry_valid(ticker: &Ticker, did: IdentityId) -> bool {
        // Assumes uppercase ticker
        if <Tickers<T>>::exists(ticker) {
            let now = <timestamp::Module<T>>::get();
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
        return false;
    }

    /// Returns 0 if ticker is registered to someone else
    /// 1 if ticker is available for registry
    /// 2 if ticker is already registered to provided did
    pub fn is_ticker_available_or_registered_to(
        ticker: &Ticker,
        did: IdentityId,
    ) -> TickerRegistrationStatus {
        // Assumes uppercase ticker
        if <Tickers<T>>::exists(ticker) {
            let ticker_reg = Self::ticker_registration(*ticker);
            if let Some(expiry) = ticker_reg.expiry {
                let now = <timestamp::Module<T>>::get();
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
        return TickerRegistrationStatus::Available;
    }

    fn _register_ticker(
        ticker: &Ticker,
        sender: T::AccountId,
        to_did: IdentityId,
        expiry: Option<T::Moment>,
    ) {
        // charge fee
        Self::charge_ticker_registration_fee(ticker, sender.clone(), to_did);

        let ticker_registration = TickerRegistration {
            owner: to_did,
            expiry: expiry.clone(),
        };

        // Store ticker registration details
        <Tickers<T>>::insert(ticker, ticker_registration);

        Self::deposit_event(RawEvent::TickerRegistered(*ticker, to_did, expiry));
    }

    fn charge_ticker_registration_fee(_ticker: &Ticker, _sender: T::AccountId, _did: IdentityId) {
        //TODO: Charge fee
    }

    /// Get the asset `id` balance of `who`.
    pub fn balance(ticker: Ticker, did: IdentityId) -> T::Balance {
        ticker.canonize();
        Self::balance_of((ticker, did))
    }

    // Get the total supply of an asset `id`
    pub fn total_supply(ticker: Ticker) -> T::Balance {
        ticker.canonize();
        Self::token_details(ticker).total_supply
    }

    pub fn get_balance_at(ticker: Ticker, did: IdentityId, at: u64) -> T::Balance {
        ticker.canonize();
        let ticker_did = (ticker, did);
        if !<TotalCheckpoints>::exists(ticker) ||
            at == 0 || //checkpoints start from 1
            at > Self::total_checkpoints_of(&ticker)
        {
            // No checkpoints data exist
            return Self::balance_of(&ticker_did);
        }

        if <UserCheckpoints>::exists(&ticker_did) {
            let user_checkpoints = Self::user_checkpoints(&ticker_did);
            if at > *user_checkpoints.last().unwrap_or(&0) {
                // Using unwrap_or to be defensive.
                // or part should never be triggered due to the check on 2 lines above
                // User has not transacted after checkpoint creation.
                // This means their current balance = their balance at that cp.
                return Self::balance_of(&ticker_did);
            }
            // Uses the first checkpoint that was created after target checpoint
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
        return Self::balance_of(&ticker_did);
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
        return arr[0];
    }

    fn _is_valid_transfer(
        ticker: &Ticker,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        value: T::Balance,
    ) -> StdResult<u8, &'static str> {
        let general_status_code =
            <general_tm::Module<T>>::verify_restriction(ticker, from_did, to_did, value)?;
        Ok(if general_status_code != ERC1400_TRANSFER_SUCCESS {
            general_status_code
        } else {
            <percentage_tm::Module<T>>::verify_restriction(ticker, from_did, to_did, value)?
        })
    }

    // the SimpleToken standard transfer function
    // internal
    fn _transfer(
        ticker: &Ticker,
        from_did: IdentityId,
        to_did: IdentityId,
        value: T::Balance,
    ) -> Result {
        // Granularity check
        ensure!(
            Self::check_granularity(ticker, value),
            "Invalid granularity"
        );
        let ticker_from_did = (*ticker, from_did);
        ensure!(
            <BalanceOf<T>>::exists(&ticker_from_did),
            "Account does not own this token"
        );
        let sender_balance = Self::balance_of(&ticker_from_did);
        ensure!(sender_balance >= value, "Not enough balance.");

        let updated_from_balance = sender_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating balance")?;
        let ticker_to_did = (*ticker, to_did);
        let receiver_balance = Self::balance_of(ticker_to_did);
        let updated_to_balance = receiver_balance
            .checked_add(&value)
            .ok_or("overflow in calculating balance")?;

        Self::_update_checkpoint(ticker, from_did, sender_balance);
        Self::_update_checkpoint(ticker, to_did, receiver_balance);
        // reduce sender's balance
        <BalanceOf<T>>::insert(&ticker_from_did, updated_from_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert(ticker_to_did, updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(*ticker, from_did, to_did, value));
        Ok(())
    }

    pub fn _create_checkpoint(ticker: &Ticker) -> Result {
        if <TotalCheckpoints>::exists(ticker) {
            let mut checkpoint_count = Self::total_checkpoints_of(ticker);
            checkpoint_count = checkpoint_count
                .checked_add(1)
                .ok_or("overflow in adding checkpoint")?;
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
        if <TotalCheckpoints>::exists(ticker) {
            let checkpoint_count = Self::total_checkpoints_of(ticker);
            let ticker_user_did_checkpont = (*ticker, user_did, checkpoint_count);
            if !<CheckpointBalance<T>>::exists(&ticker_user_did_checkpont) {
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

    pub fn _mint(ticker: &Ticker, to_did: IdentityId, value: T::Balance) -> Result {
        // Granularity check
        ensure!(
            Self::check_granularity(ticker, value),
            "Invalid granularity"
        );
        //Increase receiver balance
        let ticker_to_did = (*ticker, to_did);
        let current_to_balance = Self::balance_of(&ticker_to_did);
        let updated_to_balance = current_to_balance
            .checked_add(&value)
            .ok_or("overflow in calculating balance")?;
        // verify transfer check
        ensure!(
            Self::_is_valid_transfer(ticker, None, Some(to_did), value)?
                == ERC1400_TRANSFER_SUCCESS,
            "Transfer restrictions failed"
        );

        // Read the token details
        let mut token = Self::token_details(ticker);
        let updated_total_supply = token
            .total_supply
            .checked_add(&value)
            .ok_or("overflow in calculating total supply")?;
        ensure!(
            updated_total_supply <= MAX_SUPPLY.into(),
            "Total supply above the limit"
        );
        //Increase total suply
        token.total_supply = updated_total_supply;

        Self::_update_checkpoint(ticker, to_did, current_to_balance);

        <BalanceOf<T>>::insert(&ticker_to_did, updated_to_balance);
        <Tokens<T>>::insert(ticker, token);
        let round = Self::funding_round(ticker);
        let ticker_round = (*ticker, round.clone());
        let issued_in_this_round = Self::issued_in_funding_round(&ticker_round)
            .checked_add(&value)
            .ok_or("current funding round total overflowed")?;
        <IssuedInFundingRound<T>>::insert(&ticker_round, issued_in_this_round);
        Self::deposit_event(RawEvent::Issued(
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
    ) -> Result {
        let remaining_balance = Self::balance_of(&(*ticker, holder_did))
            .checked_sub(&value)
            .ok_or("underflow in balance deduction")?;
        ensure!(
            remaining_balance >= Self::total_custody_allowance(&(*ticker, holder_did)),
            "Insufficient balance for transfer"
        );
        Ok(())
    }

    fn _increase_custody_allowance(
        ticker: Ticker,
        holder_did: IdentityId,
        custodian_did: IdentityId,
        value: T::Balance,
    ) -> Result {
        let new_custody_allowance = Self::total_custody_allowance((ticker, holder_did))
            .checked_add(&value)
            .ok_or("total custody allowance get overflowed")?;
        // Ensure that balance of the token holder should greater than or equal to the total custody allowance + value
        ensure!(
            Self::balance_of((ticker, holder_did)) >= new_custody_allowance,
            "Insufficient balance of holder did"
        );
        // Ensure the valid DID
        ensure!(
            <identity::DidRecords>::exists(custodian_did),
            "Invalid custodian DID"
        );

        let old_allowance = Self::custodian_allowance((ticker, holder_did, custodian_did));
        let new_current_allowance = old_allowance
            .checked_add(&value)
            .ok_or("allowance get overflowed")?;
        // Update Storage
        <CustodianAllowance<T>>::insert(
            (ticker, holder_did, custodian_did),
            &new_current_allowance,
        );
        <TotalCustodyAllowance<T>>::insert((ticker, holder_did), new_custody_allowance);
        Self::deposit_event(RawEvent::CustodyAllowanceChanged(
            ticker,
            holder_did,
            custodian_did,
            old_allowance,
            new_current_allowance,
        ));
        Ok(())
    }

    /// Accept and process a ticker transfer
    pub fn _accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> Result {
        ensure!(
            <identity::Authorizations<T>>::exists((Signer::from(to_did), auth_id)),
            AuthorizationError::Invalid.into()
        );

        let auth = <identity::Module<T>>::authorizations((Signer::from(to_did), auth_id));

        let ticker = match auth.authorization_data {
            AuthorizationData::TransferTicker(ticker) => {
                ticker.canonize();
                ticker
            }
            _ => return Err("Not a ticker transfer auth"),
        };

        ensure!(!<Tokens<T>>::exists(&ticker), "token already created");

        let current_owner = Self::ticker_registration(&ticker).owner;

        <identity::Module<T>>::consume_auth(
            Signer::from(current_owner),
            Signer::from(to_did),
            auth_id,
        )?;

        <Tickers<T>>::mutate(&ticker, |tr| tr.owner = to_did);

        Self::deposit_event(RawEvent::TickerTransferred(ticker, current_owner, to_did));

        Ok(())
    }

    /// Accept and process a token ownership transfer
    pub fn _accept_token_ownership_transfer(to_did: IdentityId, auth_id: u64) -> Result {
        ensure!(
            <identity::Authorizations<T>>::exists((Signer::from(to_did), auth_id)),
            AuthorizationError::Invalid.into()
        );

        let auth = <identity::Module<T>>::authorizations((Signer::from(to_did), auth_id));

        let ticker = match auth.authorization_data {
            AuthorizationData::TransferTokenOwnership(ticker) => {
                ticker.canonize();
                ticker
            }
            _ => return Err("Not a token ownership transfer auth"),
        };

        ensure!(<Tokens<T>>::exists(&ticker), "Token does not exist");

        let current_owner = Self::token_details(&ticker).owner_did;

        <identity::Module<T>>::consume_auth(
            Signer::from(current_owner),
            Signer::from(to_did),
            auth_id,
        )?;

        <Tokens<T>>::mutate(&ticker, |t| t.owner_did = to_did);
        <Tickers<T>>::mutate(&ticker, |t| t.owner = to_did);

        Self::deposit_event(RawEvent::TokenOwnershipTransferred(
            ticker,
            current_owner,
            to_did,
        ));

        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{exemption, group, identity};
    use primitives::{IdentityId, Key};
    use rand::Rng;

    use chrono::prelude::*;
    use lazy_static::lazy_static;
    use sr_io::with_externalities;
    use sr_primitives::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys},
        AnySignature, Perbill,
    };
    use srml_support::{
        assert_err, assert_noop, assert_ok,
        dispatch::{DispatchError, DispatchResult},
        impl_outer_origin, parameter_types,
    };
    use std::sync::{Arc, Mutex};
    use substrate_primitives::{Blake2Hasher, H256};
    use system::EnsureSignedBy;
    use test_client::{self, AccountKeyring};

    type SessionIndex = u32;
    type AuthorityId = <AnySignature as Verify>::Signer;
    type BlockNumber = u64;
    type AccountId = <AnySignature as Verify>::Signer;
    type OffChainSignature = AnySignature;

    pub struct TestOnSessionEnding;
    impl session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
        fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
            None
        }
    }

    pub struct TestSessionHandler;
    impl session::SessionHandler<AuthorityId> for TestSessionHandler {
        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            _validators: &[(AuthorityId, Ks)],
            _queued_validators: &[(AuthorityId, Ks)],
        ) {
        }

        fn on_disabled(_validator_index: usize) {}

        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}
    }

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const Period: BlockNumber = 1;
        pub const Offset: BlockNumber = 0;
        pub const BlockHashCount: u32 = 250;
        pub const MaximumBlockWeight: u32 = 4 * 1024 * 1024;
        pub const MaximumBlockLength: u32 = 4 * 1024 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = BlockNumber;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        //type AccountId = u64;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<AccountId>;
        type WeightMultiplierUpdate = ();
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type AvailableBlockRatio = AvailableBlockRatio;
        type MaximumBlockLength = MaximumBlockLength;
        type Version = ();
    }

    parameter_types! {
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    }

    impl session::Trait for Test {
        type OnSessionEnding = TestOnSessionEnding;
        type Keys = UintAuthorityId;
        type ShouldEndSession = session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AuthorityId;
        type ValidatorIdOf = ConvertInto;
        type SelectInitialValidators = ();
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl session::historical::Trait for Test {
        type FullIdentification = ();
        type FullIdentificationOf = ();
    }

    parameter_types! {
        pub const ExistentialDeposit: u64 = 0;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
        pub const TransactionBaseFee: u64 = 0;
        pub const TransactionByteFee: u64 = 0;
    }

    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type TransactionBaseFee = TransactionBaseFee;
        type TransactionByteFee = TransactionByteFee;
        type WeightToFee = ConvertInto;
        type Identity = identity::Module<Test>;
    }

    impl general_tm::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }

    parameter_types! {
        pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
    }

    impl group::Trait<group::Instance1> for Test {
        type Event = ();
        type AddOrigin = EnsureSignedBy<One, AccountId>;
        type RemoveOrigin = EnsureSignedBy<Two, AccountId>;
        type SwapOrigin = EnsureSignedBy<Three, AccountId>;
        type ResetOrigin = EnsureSignedBy<Four, AccountId>;
        type MembershipInitialized = ();
        type MembershipChanged = ();
    }

    #[derive(codec::Encode, codec::Decode, Debug, Clone, Eq, PartialEq)]
    pub struct IdentityProposal {
        pub dummy: u8,
    }

    impl sr_primitives::traits::Dispatchable for IdentityProposal {
        type Origin = Origin;
        type Trait = Test;
        type Error = DispatchError;

        fn dispatch(self, _origin: Self::Origin) -> DispatchResult<Self::Error> {
            Ok(())
        }
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = IdentityProposal;
        type AcceptTransferTarget = Module<Test>;
    }
    impl percentage_tm::Trait for Test {
        type Event = ();
    }

    impl exemption::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl utils::Trait for Test {
        type OffChainSignature = OffChainSignature;
        fn validator_id_to_account_id(v: <Self as session::Trait>::ValidatorId) -> Self::AccountId {
            v
        }
    }

    impl Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }
    type Asset = Module<Test>;
    type Balances = balances::Module<Test>;
    type Identity = identity::Module<Test>;
    type GeneralTM = general_tm::Module<Test>;

    lazy_static! {
        static ref INVESTOR_MAP_OUTER_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
    }

    /// Build a genesis identity instance owned by account No. 1
    fn identity_owned_by_alice() -> sr_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        identity::GenesisConfig::<Test> {
            owner: AccountKeyring::Alice.public().into(),
            did_creation_fee: 250,
        }
        .assimilate_storage(&mut t)
        .unwrap();
        self::GenesisConfig::<Test> {
            asset_creation_fee: 0,
            ticker_registration_fee: 0,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 8,
                registration_length: Some(10000),
            },
            fee_collector: AccountKeyring::Dave.public().into(),
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sr_io::TestExternalities::new(t)
    }

    fn make_account(
        account_id: &AccountId,
    ) -> StdResult<(<Test as system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        Balances::make_free_balance_be(&account_id, 1_000_000);
        Identity::register_did(signed_id.clone(), vec![])?;
        let did = Identity::get_identity(&Key::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn issuers_can_create_and_rename_tokens() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();
            // Raise the owner's base currency balance
            Balances::make_free_balance_be(&owner_acc, 1_000_000);

            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01],
                owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            assert!(!<identity::DidRecords>::exists(
                Identity::get_token_did(&ticker).unwrap()
            ));
            let identifiers = vec![(IdentifierType::default(), b"undefined".to_vec())];
            let ticker = Ticker::from_slice(token.name.as_slice());
            assert_err!(
                Asset::create_token(
                    owner_signed.clone(),
                    owner_did,
                    token.name.clone(),
                    ticker,
                    1_000_000_000_000_000_000_000_000, // Total supply over the limit
                    true,
                    token.asset_type.clone(),
                    identifiers.clone(),
                ),
                "Total supply above the limit"
            );

            // Issuance is successful
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                identifiers.clone(),
            ));

            // A correct entry is added
            assert_eq!(Asset::token_details(ticker), token);
            //assert!(Identity::is_existing_identity(Identity::get_token_did(&token.name).unwrap()));
            assert!(<identity::DidRecords>::exists(
                Identity::get_token_did(&ticker).unwrap()
            ));
            assert_eq!(Asset::token_details(ticker), token);

            // Unauthorized identities cannot rename the token.
            let eve_acc = AccountId::from(AccountKeyring::Eve);
            let (eve_signed, _eve_did) = make_account(&eve_acc).unwrap();
            assert_err!(
                Asset::rename_token(eve_signed, ticker, vec![0xde, 0xad, 0xbe, 0xef]),
                "sender must be a signing key for the token owner DID"
            );
            // The token should remain unchanged in storage.
            assert_eq!(Asset::token_details(ticker), token);
            // Rename the token and check storage has been updated.
            let renamed_token = SecurityToken {
                name: vec![0x42],
                owner_did: token.owner_did,
                total_supply: token.total_supply,
                divisible: token.divisible,
                asset_type: token.asset_type.clone(),
            };
            assert_ok!(Asset::rename_token(
                owner_signed.clone(),
                ticker,
                renamed_token.name.clone()
            ));
            assert_eq!(Asset::token_details(ticker), renamed_token);
            for (typ, val) in identifiers {
                assert_eq!(Asset::identifiers((ticker, typ)), val);
            }
        });
    }

    /// # TODO
    /// It should be re-enable once issuer claim is re-enabled.
    #[test]
    #[ignore]
    fn non_issuers_cant_create_tokens() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (_, owner_did) = make_account(&owner_acc).unwrap();

            // Expected token entry
            let _ = SecurityToken {
                name: vec![0x01],
                owner_did: owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };

            let wrong_acc = AccountId::from(AccountKeyring::Bob);

            Balances::make_free_balance_be(&wrong_acc, 1_000_000);

            let wrong_did = IdentityId::try_from("did:poly:wrong");
            assert!(wrong_did.is_err());
        });
    }

    #[test]
    fn valid_transfers_pass() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            Balances::make_free_balance_be(&owner_acc, 1_000_000);

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (_, alice_did) = make_account(&alice_acc).unwrap();

            Balances::make_free_balance_be(&alice_acc, 1_000_000);

            // Issuance is successful
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
            ));

            // A correct entry is added
            assert_eq!(Asset::token_details(ticker), token);

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                owner_signed.clone(),
                owner_did,
                ticker,
                asset_rule
            ));

            assert_ok!(Asset::transfer(
                owner_signed.clone(),
                owner_did,
                ticker,
                alice_did,
                500
            ));
        })
    }

    #[test]
    fn valid_custodian_allowance() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            Balances::make_free_balance_be(&owner_acc, 1_000_000);

            let investor1_acc = AccountId::from(AccountKeyring::Bob);
            let (investor1_signed, investor1_did) = make_account(&investor1_acc).unwrap();

            Balances::make_free_balance_be(&investor1_acc, 1_000_000);

            let investor2_acc = AccountId::from(AccountKeyring::Charlie);
            let (investor2_signed, investor2_did) = make_account(&investor2_acc).unwrap();

            Balances::make_free_balance_be(&investor2_acc, 1_000_000);

            let custodian_acc = AccountId::from(AccountKeyring::Eve);
            let (custodian_signed, custodian_did) = make_account(&custodian_acc).unwrap();

            Balances::make_free_balance_be(&custodian_acc, 1_000_000);

            // Issuance is successful
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
            ));

            assert_eq!(
                Asset::balance_of((ticker, token.owner_did)),
                token.total_supply
            );

            assert_eq!(Asset::token_details(ticker), token);

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                owner_signed.clone(),
                owner_did,
                ticker,
                asset_rule
            ));
            let funding_round1 = b"Round One".to_vec();
            assert_ok!(Asset::set_funding_round(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                funding_round1.clone()
            ));
            // Mint some tokens to investor1
            let num_tokens1: u128 = 2_000_000;
            assert_ok!(Asset::issue(
                owner_signed.clone(),
                owner_did,
                ticker,
                investor1_did,
                num_tokens1,
                vec![0x0]
            ));
            assert_eq!(Asset::funding_round(&token.name), funding_round1.clone());
            assert_eq!(
                Asset::issued_in_funding_round((token.name.clone(), funding_round1.clone())),
                num_tokens1
            );
            // Check the expected default behaviour of the map.
            assert_eq!(
                Asset::issued_in_funding_round((token.name.clone(), b"No such round".to_vec())),
                0
            );
            assert_eq!(Asset::balance_of((ticker, investor1_did)), num_tokens1,);

            // Failed to add custodian because of insufficient balance
            assert_noop!(
                Asset::increase_custody_allowance(
                    investor1_signed.clone(),
                    ticker,
                    investor1_did,
                    custodian_did,
                    250_00_00 as u128
                ),
                "Insufficient balance of holder did"
            );

            // Failed to add/increase the custodian allowance because of Invalid custodian did
            let custodian_did_not_register = IdentityId::from(5u128);
            assert_noop!(
                Asset::increase_custody_allowance(
                    investor1_signed.clone(),
                    ticker,
                    investor1_did,
                    custodian_did_not_register,
                    50_00_00 as u128
                ),
                "Invalid custodian DID"
            );

            // Add custodian
            assert_ok!(Asset::increase_custody_allowance(
                investor1_signed.clone(),
                ticker,
                investor1_did,
                custodian_did,
                50_00_00 as u128
            ));

            assert_eq!(
                Asset::custodian_allowance((ticker, investor1_did, custodian_did)),
                50_00_00 as u128
            );

            assert_eq!(
                Asset::total_custody_allowance((ticker, investor1_did)),
                50_00_00 as u128
            );

            // Transfer the token upto the limit
            assert_ok!(Asset::transfer(
                investor1_signed.clone(),
                investor1_did,
                ticker,
                investor2_did,
                140_00_00 as u128
            ));

            assert_eq!(
                Asset::balance_of((ticker, investor2_did)),
                140_00_00 as u128
            );

            // Try to Transfer the tokens beyond the limit
            assert_noop!(
                Asset::transfer(
                    investor1_signed.clone(),
                    investor1_did,
                    ticker,
                    investor2_did,
                    50_00_00 as u128
                ),
                "Insufficient balance for transfer"
            );

            // Should fail to transfer the token by the custodian because of invalid signing key
            assert_noop!(
                Asset::transfer_by_custodian(
                    investor2_signed.clone(),
                    ticker,
                    investor1_did,
                    custodian_did,
                    investor2_did,
                    45_00_00 as u128
                ),
                "sender must be a signing key for DID"
            );

            // Should fail to transfer the token by the custodian because of insufficient allowance
            assert_noop!(
                Asset::transfer_by_custodian(
                    custodian_signed.clone(),
                    ticker,
                    investor1_did,
                    custodian_did,
                    investor2_did,
                    55_00_00 as u128
                ),
                "Insufficient allowance"
            );

            // Successfully transfer by the custodian
            assert_ok!(Asset::transfer_by_custodian(
                custodian_signed.clone(),
                ticker,
                investor1_did,
                custodian_did,
                investor2_did,
                45_00_00 as u128
            ));
        });
    }

    #[test]
    fn valid_custodian_allowance_of() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            Balances::make_free_balance_be(&owner_acc, 1_000_000);

            let investor1_acc = AccountId::from(AccountKeyring::Bob);
            let (investor1_signed, investor1_did) = make_account(&investor1_acc).unwrap();

            Balances::make_free_balance_be(&investor1_acc, 1_000_000);

            let investor2_acc = AccountId::from(AccountKeyring::Charlie);
            let (investor2_signed, investor2_did) = make_account(&investor2_acc).unwrap();

            Balances::make_free_balance_be(&investor2_acc, 1_000_000);

            let custodian_acc = AccountId::from(AccountKeyring::Eve);
            let (custodian_signed, custodian_did) = make_account(&custodian_acc).unwrap();

            Balances::make_free_balance_be(&custodian_acc, 1_000_000);

            // Issuance is successful
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
            ));

            assert_eq!(
                Asset::balance_of((ticker, token.owner_did)),
                token.total_supply
            );

            assert_eq!(Asset::token_details(ticker), token);

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                owner_signed.clone(),
                owner_did,
                ticker,
                asset_rule
            ));

            // Mint some tokens to investor1
            assert_ok!(Asset::issue(
                owner_signed.clone(),
                owner_did,
                ticker,
                investor1_did,
                200_00_00 as u128,
                vec![0x0]
            ));

            assert_eq!(
                Asset::balance_of((ticker, investor1_did)),
                200_00_00 as u128
            );

            let msg = SignData {
                custodian_did: custodian_did,
                holder_did: investor1_did,
                ticker,
                value: 50_00_00 as u128,
                nonce: 1,
            };

            let investor1_key = AccountKeyring::Bob;

            // Add custodian
            assert_ok!(Asset::increase_custody_allowance_of(
                investor2_signed.clone(),
                ticker,
                investor1_did,
                investor1_acc.clone(),
                custodian_did,
                investor2_did,
                50_00_00 as u128,
                1,
                OffChainSignature::from(investor1_key.sign(&msg.encode()))
            ));

            assert_eq!(
                Asset::custodian_allowance((ticker, investor1_did, custodian_did)),
                50_00_00 as u128
            );

            assert_eq!(
                Asset::total_custody_allowance((ticker, investor1_did)),
                50_00_00 as u128
            );

            // use the same signature with the same nonce should fail
            assert_noop!(
                Asset::increase_custody_allowance_of(
                    investor2_signed.clone(),
                    ticker,
                    investor1_did,
                    investor1_acc.clone(),
                    custodian_did,
                    investor2_did,
                    50_00_00 as u128,
                    1,
                    OffChainSignature::from(investor1_key.sign(&msg.encode()))
                ),
                "Signature already used"
            );

            // use the same signature with the different nonce should fail
            assert_noop!(
                Asset::increase_custody_allowance_of(
                    investor2_signed.clone(),
                    ticker,
                    investor1_did,
                    investor1_acc.clone(),
                    custodian_did,
                    investor2_did,
                    50_00_00 as u128,
                    3,
                    OffChainSignature::from(investor1_key.sign(&msg.encode()))
                ),
                "Invalid signature"
            );

            // Transfer the token upto the limit
            assert_ok!(Asset::transfer(
                investor1_signed.clone(),
                investor1_did,
                ticker,
                investor2_did,
                140_00_00 as u128
            ));

            assert_eq!(
                Asset::balance_of((ticker, investor2_did)),
                140_00_00 as u128
            );

            // Try to Transfer the tokens beyond the limit
            assert_noop!(
                Asset::transfer(
                    investor1_signed.clone(),
                    investor1_did,
                    ticker,
                    investor2_did,
                    50_00_00 as u128
                ),
                "Insufficient balance for transfer"
            );

            // Should fail to transfer the token by the custodian because of invalid signing key
            assert_noop!(
                Asset::transfer_by_custodian(
                    investor2_signed.clone(),
                    ticker,
                    investor1_did,
                    custodian_did,
                    investor2_did,
                    45_00_00 as u128
                ),
                "sender must be a signing key for DID"
            );

            // Should fail to transfer the token by the custodian because of insufficient allowance
            assert_noop!(
                Asset::transfer_by_custodian(
                    custodian_signed.clone(),
                    ticker,
                    investor1_did,
                    custodian_did,
                    investor2_did,
                    55_00_00 as u128
                ),
                "Insufficient allowance"
            );

            // Successfully transfer by the custodian
            assert_ok!(Asset::transfer_by_custodian(
                custodian_signed.clone(),
                ticker,
                investor1_did,
                custodian_did,
                investor2_did,
                45_00_00 as u128
            ));
        });
    }

    #[test]
    fn checkpoints_fuzz_test() {
        println!("Starting");
        for _ in 0..10 {
            // When fuzzing in local, feel free to bump this number to add more fuzz runs.
            with_externalities(&mut identity_owned_by_alice(), || {
                let now = Utc::now();
                <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

                let owner_acc = AccountId::from(AccountKeyring::Dave);
                let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

                // Expected token entry
                let token = SecurityToken {
                    name: vec![0x01],
                    owner_did: owner_did,
                    total_supply: 1_000_000,
                    divisible: true,
                    asset_type: AssetType::default(),
                };
                let ticker = Ticker::from_slice(token.name.as_slice());
                let bob_acc = AccountId::from(AccountKeyring::Bob);
                let (_, bob_did) = make_account(&bob_acc).unwrap();

                // Issuance is successful
                assert_ok!(Asset::create_token(
                    owner_signed.clone(),
                    owner_did,
                    token.name.clone(),
                    ticker,
                    token.total_supply,
                    true,
                    token.asset_type.clone(),
                    vec![],
                ));

                let asset_rule = general_tm::AssetRule {
                    sender_rules: vec![],
                    receiver_rules: vec![],
                };

                // Allow all transfers
                assert_ok!(GeneralTM::add_active_rule(
                    owner_signed.clone(),
                    owner_did,
                    ticker,
                    asset_rule
                ));

                let mut owner_balance: [u128; 100] = [1_000_000; 100];
                let mut bob_balance: [u128; 100] = [0; 100];
                let mut rng = rand::thread_rng();
                for j in 1..100 {
                    let transfers = rng.gen_range(0, 10);
                    owner_balance[j] = owner_balance[j - 1];
                    bob_balance[j] = bob_balance[j - 1];
                    for _k in 0..transfers {
                        if j == 1 {
                            owner_balance[0] -= 1;
                            bob_balance[0] += 1;
                        }
                        owner_balance[j] -= 1;
                        bob_balance[j] += 1;
                        assert_ok!(Asset::transfer(
                            owner_signed.clone(),
                            owner_did,
                            ticker,
                            bob_did,
                            1
                        ));
                    }
                    assert_ok!(Asset::create_checkpoint(
                        owner_signed.clone(),
                        owner_did,
                        ticker,
                    ));
                    let x: u64 = u64::try_from(j).unwrap();
                    assert_eq!(
                        Asset::get_balance_at(ticker, owner_did, 0),
                        owner_balance[j]
                    );
                    assert_eq!(Asset::get_balance_at(ticker, bob_did, 0), bob_balance[j]);
                    assert_eq!(
                        Asset::get_balance_at(ticker, owner_did, 1),
                        owner_balance[1]
                    );
                    assert_eq!(Asset::get_balance_at(ticker, bob_did, 1), bob_balance[1]);
                    assert_eq!(
                        Asset::get_balance_at(ticker, owner_did, x - 1),
                        owner_balance[j - 1]
                    );
                    assert_eq!(
                        Asset::get_balance_at(ticker, bob_did, x - 1),
                        bob_balance[j - 1]
                    );
                    assert_eq!(
                        Asset::get_balance_at(ticker, owner_did, x),
                        owner_balance[j]
                    );
                    assert_eq!(Asset::get_balance_at(ticker, bob_did, x), bob_balance[j]);
                    assert_eq!(
                        Asset::get_balance_at(ticker, owner_did, x + 1),
                        owner_balance[j]
                    );
                    assert_eq!(
                        Asset::get_balance_at(ticker, bob_did, x + 1),
                        bob_balance[j]
                    );
                    assert_eq!(
                        Asset::get_balance_at(ticker, owner_did, 1000),
                        owner_balance[j]
                    );
                    assert_eq!(Asset::get_balance_at(ticker, bob_did, 1000), bob_balance[j]);
                }
            });
        }
    }

    #[test]
    fn register_ticker() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            Balances::make_free_balance_be(&owner_acc, 1_000_000);

            let token = SecurityToken {
                name: vec![0x01],
                owner_did: owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            let identifiers = vec![(IdentifierType::Custom(b"check".to_vec()), b"me".to_vec())];
            let ticker = Ticker::from_slice(token.name.as_slice());
            // Issuance is successful
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                identifiers.clone(),
            ));

            assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), true);
            assert_eq!(Asset::is_ticker_available(&ticker), false);
            let stored_token = <Module<Test>>::token_details(&ticker);
            assert_eq!(stored_token.asset_type, token.asset_type);
            for (typ, val) in identifiers {
                assert_eq!(Asset::identifiers((ticker, typ)), val);
            }

            assert_err!(
                Asset::register_ticker(owner_signed.clone(), Ticker::from_slice(&[0x01])),
                "token already created"
            );

            assert_err!(
                Asset::register_ticker(
                    owner_signed.clone(),
                    Ticker::from_slice(&[0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01])
                ),
                "ticker length over the limit"
            );

            let ticker = Ticker::from_slice(&[0x01, 0x01]);

            assert_eq!(Asset::is_ticker_available(&ticker), true);

            assert_ok!(Asset::register_ticker(owner_signed.clone(), ticker));

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (alice_signed, _) = make_account(&alice_acc).unwrap();

            Balances::make_free_balance_be(&alice_acc, 1_000_000);

            assert_err!(
                Asset::register_ticker(alice_signed.clone(), ticker),
                "ticker registered to someone else"
            );

            assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), true);
            assert_eq!(Asset::is_ticker_available(&ticker), false);

            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64 + 10001);

            assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), false);
            assert_eq!(Asset::is_ticker_available(&ticker), true);
        })
    }

    #[test]
    fn transfer_ticker() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (alice_signed, alice_did) = make_account(&alice_acc).unwrap();

            let bob_acc = AccountId::from(AccountKeyring::Bob);
            let (bob_signed, bob_did) = make_account(&bob_acc).unwrap();

            let ticker = Ticker::from_slice(&[0x01, 0x01]);

            assert_eq!(Asset::is_ticker_available(&ticker), true);
            assert_ok!(Asset::register_ticker(owner_signed.clone(), ticker));

            Identity::add_auth(
                Signer::from(owner_did),
                Signer::from(alice_did),
                AuthorizationData::TransferTicker(ticker),
                None,
            );

            Identity::add_auth(
                Signer::from(owner_did),
                Signer::from(bob_did),
                AuthorizationData::TransferTicker(ticker),
                None,
            );

            assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), true);
            assert_eq!(Asset::is_ticker_registry_valid(&ticker, alice_did), false);
            assert_eq!(Asset::is_ticker_available(&ticker), false);

            let mut auth_id = Identity::last_authorization(Signer::from(alice_did));

            assert_err!(
                Asset::accept_ticker_transfer(alice_signed.clone(), auth_id + 1),
                "Authorization does not exist"
            );

            assert_ok!(Asset::accept_ticker_transfer(alice_signed.clone(), auth_id));

            auth_id = Identity::last_authorization(Signer::from(bob_did));
            assert_err!(
                Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
                "Illegal use of Authorization"
            );

            Identity::add_auth(
                Signer::from(alice_did),
                Signer::from(bob_did),
                AuthorizationData::TransferTicker(ticker),
                Some(now.timestamp() as u64 - 100),
            );
            auth_id = Identity::last_authorization(Signer::from(bob_did));
            assert_err!(
                Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
                "Authorization expired"
            );

            Identity::add_auth(
                Signer::from(alice_did),
                Signer::from(bob_did),
                AuthorizationData::Custom(ticker),
                Some(now.timestamp() as u64 + 100),
            );
            auth_id = Identity::last_authorization(Signer::from(bob_did));
            assert_err!(
                Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
                "Not a ticker transfer auth"
            );

            Identity::add_auth(
                Signer::from(alice_did),
                Signer::from(bob_did),
                AuthorizationData::TransferTicker(ticker),
                Some(now.timestamp() as u64 + 100),
            );
            auth_id = Identity::last_authorization(Signer::from(bob_did));
            assert_ok!(Asset::accept_ticker_transfer(bob_signed.clone(), auth_id));

            assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), false);
            assert_eq!(Asset::is_ticker_registry_valid(&ticker, alice_did), false);
            assert_eq!(Asset::is_ticker_registry_valid(&ticker, bob_did), true);
            assert_eq!(Asset::is_ticker_available(&ticker), false);
        })
    }

    #[test]
    fn transfer_token_ownership() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (alice_signed, alice_did) = make_account(&alice_acc).unwrap();

            let bob_acc = AccountId::from(AccountKeyring::Bob);
            let (bob_signed, bob_did) = make_account(&bob_acc).unwrap();

            let token_name = vec![0x01, 0x01];
            let ticker = Ticker::from_slice(token_name.as_slice());
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token_name.clone(),
                ticker,
                1_000_000,
                true,
                AssetType::default(),
                vec![],
            ));

            Identity::add_auth(
                Signer::from(owner_did),
                Signer::from(alice_did),
                AuthorizationData::TransferTokenOwnership(ticker),
                None,
            );

            Identity::add_auth(
                Signer::from(owner_did),
                Signer::from(bob_did),
                AuthorizationData::TransferTokenOwnership(ticker),
                None,
            );

            assert_eq!(Asset::token_details(&ticker).owner_did, owner_did);

            let mut auth_id = Identity::last_authorization(Signer::from(alice_did));

            assert_err!(
                Asset::accept_token_ownership_transfer(alice_signed.clone(), auth_id + 1),
                "Authorization does not exist"
            );

            assert_ok!(Asset::accept_token_ownership_transfer(
                alice_signed.clone(),
                auth_id
            ));
            assert_eq!(Asset::token_details(&ticker).owner_did, alice_did);

            auth_id = Identity::last_authorization(Signer::from(bob_did));
            assert_err!(
                Asset::accept_token_ownership_transfer(bob_signed.clone(), auth_id),
                "Illegal use of Authorization"
            );

            Identity::add_auth(
                Signer::from(alice_did),
                Signer::from(bob_did),
                AuthorizationData::TransferTokenOwnership(ticker),
                Some(now.timestamp() as u64 - 100),
            );
            auth_id = Identity::last_authorization(Signer::from(bob_did));
            assert_err!(
                Asset::accept_token_ownership_transfer(bob_signed.clone(), auth_id),
                "Authorization expired"
            );

            Identity::add_auth(
                Signer::from(alice_did),
                Signer::from(bob_did),
                AuthorizationData::Custom(ticker),
                Some(now.timestamp() as u64 + 100),
            );
            auth_id = Identity::last_authorization(Signer::from(bob_did));
            assert_err!(
                Asset::accept_token_ownership_transfer(bob_signed.clone(), auth_id),
                "Not a token ownership transfer auth"
            );

            Identity::add_auth(
                Signer::from(alice_did),
                Signer::from(bob_did),
                AuthorizationData::TransferTokenOwnership(Ticker::from_slice(&[0x50])),
                Some(now.timestamp() as u64 + 100),
            );
            auth_id = Identity::last_authorization(Signer::from(bob_did));
            assert_err!(
                Asset::accept_token_ownership_transfer(bob_signed.clone(), auth_id),
                "Token does not exist"
            );

            Identity::add_auth(
                Signer::from(alice_did),
                Signer::from(bob_did),
                AuthorizationData::TransferTokenOwnership(ticker),
                Some(now.timestamp() as u64 + 100),
            );
            auth_id = Identity::last_authorization(Signer::from(bob_did));
            assert_ok!(Asset::accept_token_ownership_transfer(
                bob_signed.clone(),
                auth_id
            ));
            assert_eq!(Asset::token_details(&ticker).owner_did, bob_did);
        })
    }

    #[test]
    fn update_identifiers() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();
            // Raise the owner's base currency balance
            Balances::make_free_balance_be(&owner_acc, 1_000_000);
            // Expected token entry
            let token = SecurityToken {
                name: b"TEST".to_vec(),
                owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            assert!(!<identity::DidRecords>::exists(
                Identity::get_token_did(&token.name).unwrap()
            ));
            let identifier_value1 = b"ABC123";
            let identifiers = vec![(IdentifierType::Cusip, identifier_value1.to_vec())];
            let ticker = token.name.clone();
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                identifiers.clone(),
            ));
            // A correct entry was added
            assert_eq!(Asset::token_details(ticker), token);
            assert_eq!(
                Asset::identifiers((ticker, IdentifierType::Cusip)),
                identifier_value1.to_vec()
            );
            let identifier_value2 = b"XYZ555";
            let updated_identifiers = vec![
                (IdentifierType::Cusip, Default::default()),
                (IdentifierType::Isin, identifier_value2.to_vec()),
            ];
            assert_ok!(Asset::update_identifiers(
                owner_signed.clone(),
                owner_did,
                ticker,
                updated_identifiers.clone(),
            ));
            for (typ, val) in updated_identifiers {
                assert_eq!(Asset::identifiers((ticker, typ)), val);
            }
        });
    }

    /*
     *    #[test]
     *    /// This test loads up a YAML of testcases and checks each of them
     *    fn transfer_scenarios_external() {
     *        let mut yaml_path_buf = PathBuf::new();
     *        yaml_path_buf.push(env!("CARGO_MANIFEST_DIR")); // This package's root
     *        yaml_path_buf.push("tests/asset_transfers.yml");
     *
     *        println!("Loading YAML from {:?}", yaml_path_buf);
     *
     *        let yaml_string = read_to_string(yaml_path_buf.as_path())
     *            .expect("Could not load the YAML file to a string");
     *
     *        // Parse the YAML
     *        let yaml = YamlLoader::load_from_str(&yaml_string).expect("Could not parse the YAML file");
     *
     *        let yaml = &yaml[0];
     *
     *        let now = Utc::now();
     *
     *        for case in yaml["test_cases"]
     *            .as_vec()
     *            .expect("Could not reach test_cases")
     *        {
     *            println!("Case: {:#?}", case);
     *
     *            let accounts = case["named_accounts"]
     *                .as_hash()
     *                .expect("Could not view named_accounts as a hashmap");
     *
     *            let mut externalities = if let Some(identity_owner) =
     *                accounts.get(&Yaml::String("identity-owner".to_owned()))
     *            {
     *                identity_owned_by(
     *                    identity_owner["id"]
     *                        .as_i64()
     *                        .expect("Could not get identity owner's ID") as u64,
     *                )
     *            } else {
     *                system::GenesisConfig::default()
     *                    .build_storage()
     *                    .unwrap()
     *                    .0
     *                    .into()
     *            };
     *
     *            with_externalities(&mut externalities, || {
     *                // Instantiate accounts
     *                for (name, account) in accounts {
     *                    <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);
     *                    let name = name
     *                        .as_str()
     *                        .expect("Could not take named_accounts key as string");
     *                    let id = account["id"].as_i64().expect("id is not a number") as u64;
     *                    let balance = account["balance"]
     *                        .as_i64()
     *                        .expect("balance is not a number");
     *
     *                    println!("Preparing account {}", name);
     *
     *                    Balances::make_free_balance_be(&id, balance.clone() as u128);
     *                    println!("{}: gets {} initial balance", name, balance);
     *                    if account["issuer"]
     *                        .as_bool()
     *                        .expect("Could not check if account is an issuer")
     *                    {
     *                        assert_ok!(identity::Module::<Test>::do_create_issuer(id));
     *                        println!("{}: becomes issuer", name);
     *                    }
     *                    if account["investor"]
     *                        .as_bool()
     *                        .expect("Could not check if account is an investor")
     *                    {
     *                        assert_ok!(identity::Module::<Test>::do_create_investor(id));
     *                        println!("{}: becomes investor", name);
     *                    }
     *                }
     *
     *                // Issue tokens
     *                let tokens = case["tokens"]
     *                    .as_hash()
     *                    .expect("Could not view tokens as a hashmap");
     *
     *                for (ticker, token) in tokens {
     *                    let ticker = ticker.as_str().expect("Can't parse ticker as string");
     *                    println!("Preparing token {}:", ticker);
     *
     *                    let owner = token["owner"]
     *                        .as_str()
     *                        .expect("Can't parse owner as string");
     *
     *                    let owner_id = accounts
     *                        .get(&Yaml::String(owner.to_owned()))
     *                        .expect("Can't get owner record")["id"]
     *                        .as_i64()
     *                        .expect("Can't parse owner id as i64")
     *                        as u64;
     *                    let total_supply = token["total_supply"]
     *                        .as_i64()
     *                        .expect("Can't parse the total supply as i64")
     *                        as u128;
     *
     *                    let token_struct = SecurityToken {
     *                        name: *ticker.into_bytes(),
     *                        owner: owner_id,
     *                        total_supply,
     *                        divisible: true,
     *                    };
     *                    println!("{:#?}", token_struct);
     *
     *                    // Check that issuing succeeds/fails as expected
     *                    if token["issuance_succeeds"]
     *                        .as_bool()
     *                        .expect("Could not check if issuance should succeed")
     *                    {
     *                        assert_ok!(Asset::create_token(
     *                            Origin::signed(token_struct.owner),
     *                            token_struct.name.clone(),
     *                            token_struct.name.clone(),
     *                            token_struct.total_supply,
     *                            true
     *                        ));
     *
     *                        // Also check that the new token matches what we asked to create
     *                        assert_eq!(
     *                            Asset::token_details(token_struct.name.clone()),
     *                            token_struct
     *                        );
     *
     *                        // Check that the issuer's balance corresponds to total supply
     *                        assert_eq!(
     *                            Asset::balance_of((token_struct.name, token_struct.owner)),
     *                            token_struct.total_supply
     *                        );
     *
     *                        // Add specified whitelist entries
     *                        let whitelists = token["whitelist_entries"]
     *                            .as_vec()
     *                            .expect("Could not view token whitelist entries as vec");
     *
     *                        for wl_entry in whitelists {
     *                            let investor = wl_entry["investor"]
     *                                .as_str()
     *                                .expect("Can't parse investor as string");
     *                            let investor_id = accounts
     *                                .get(&Yaml::String(investor.to_owned()))
     *                                .expect("Can't get investor account record")["id"]
     *                                .as_i64()
     *                                .expect("Can't parse investor id as i64")
     *                                as u64;
     *
     *                            let expiry = wl_entry["expiry"]
     *                                .as_i64()
     *                                .expect("Can't parse expiry as i64");
     *
     *                            let wl_id = wl_entry["whitelist_id"]
     *                                .as_i64()
     *                                .expect("Could not parse whitelist_id as i64")
     *                                as u32;
     *
     *                            println!(
     *                                "Token {}: processing whitelist entry for {}",
     *                                ticker, investor
     *                            );
     *
     *                            general_tm::Module::<Test>::add_to_whitelist(
     *                                Origin::signed(owner_id),
     *                                *ticker.into_bytes(),
     *                                wl_id,
     *                                investor_id,
     *                                (now + Duration::hours(expiry)).timestamp() as u64,
     *                            )
     *                            .expect("Could not create whitelist entry");
     *                        }
     *                    } else {
     *                        assert!(Asset::create_token(
     *                            Origin::signed(token_struct.owner),
     *                            token_struct.name.clone(),
     *                            token_struct.name.clone(),
     *                            token_struct.total_supply,
     *                            true
     *                        )
     *                        .is_err());
     *                    }
     *                }
     *
     *                // Set up allowances
     *                let allowances = case["allowances"]
     *                    .as_vec()
     *                    .expect("Could not view allowances as a vec");
     *
     *                for allowance in allowances {
     *                    let sender = allowance["sender"]
     *                        .as_str()
     *                        .expect("Could not view sender as str");
     *                    let sender_id = case["named_accounts"][sender]["id"]
     *                        .as_i64()
     *                        .expect("Could not view sender id as i64")
     *                        as u64;
     *                    let spender = allowance["spender"]
     *                        .as_str()
     *                        .expect("Could not view spender as str");
     *                    let spender_id = case["named_accounts"][spender]["id"]
     *                        .as_i64()
     *                        .expect("Could not view sender id as i64")
     *                        as u64;
     *                    let amount = allowance["amount"]
     *                        .as_i64()
     *                        .expect("Could not view amount as i64")
     *                        as u128;
     *                    let ticker = allowance["ticker"]
     *                        .as_str()
     *                        .expect("Could not view ticker as str");
     *                    let succeeds = allowance["succeeds"]
     *                        .as_bool()
     *                        .expect("Could not determine if allowance should succeed");
     *
     *                    if succeeds {
     *                        assert_ok!(Asset::approve(
     *                            Origin::signed(sender_id),
     *                            *ticker.into_bytes(),
     *                            spender_id,
     *                            amount,
     *                        ));
     *                    } else {
     *                        assert!(Asset::approve(
     *                            Origin::signed(sender_id),
     *                            *ticker.into_bytes(),
     *                            spender_id,
     *                            amount,
     *                        )
     *                        .is_err())
     *                    }
     *                }
     *
     *                println!("Transfers:");
     *                // Perform regular transfers
     *                let transfers = case["transfers"]
     *                    .as_vec()
     *                    .expect("Could not view transfers as vec");
     *                for transfer in transfers {
     *                    let from = transfer["from"]
     *                        .as_str()
     *                        .expect("Could not view from as str");
     *                    let from_id = case["named_accounts"][from]["id"]
     *                        .as_i64()
     *                        .expect("Could not view from_id as i64")
     *                        as u64;
     *                    let to = transfer["to"].as_str().expect("Could not view to as str");
     *                    let to_id = case["named_accounts"][to]["id"]
     *                        .as_i64()
     *                        .expect("Could not view to_id as i64")
     *                        as u64;
     *                    let amount = transfer["amount"]
     *                        .as_i64()
     *                        .expect("Could not view amount as i64")
     *                        as u128;
     *                    let ticker = transfer["ticker"]
     *                        .as_str()
     *                        .expect("Coule not view ticker as str")
     *                        .to_owned();
     *                    let succeeds = transfer["succeeds"]
     *                        .as_bool()
     *                        .expect("Could not view succeeds as bool");
     *
     *                    println!("{} of token {} from {} to {}", amount, ticker, from, to);
     *                    let ticker = ticker.into_bytes();
     *
     *                    // Get sender's investor data
     *                    let investor_data = <InvestorList<Test>>::get(from_id);
     *
     *                    println!("{}'s investor data: {:#?}", from, investor_data);
     *
     *                    if succeeds {
     *                        assert_ok!(Asset::transfer(
     *                            Origin::signed(from_id),
     *                            ticker,
     *                            to_id,
     *                            amount
     *                        ));
     *                    } else {
     *                        assert!(
     *                            Asset::transfer(Origin::signed(from_id), ticker, to_id, amount)
     *                                .is_err()
     *                        );
     *                    }
     *                }
     *
     *                println!("Approval-based transfers:");
     *                // Perform allowance transfers
     *                let transfer_froms = case["transfer_froms"]
     *                    .as_vec()
     *                    .expect("Could not view transfer_froms as vec");
     *                for transfer_from in transfer_froms {
     *                    let from = transfer_from["from"]
     *                        .as_str()
     *                        .expect("Could not view from as str");
     *                    let from_id = case["named_accounts"][from]["id"]
     *                        .as_i64()
     *                        .expect("Could not view from_id as i64")
     *                        as u64;
     *                    let spender = transfer_from["spender"]
     *                        .as_str()
     *                        .expect("Could not view spender as str");
     *                    let spender_id = case["named_accounts"][spender]["id"]
     *                        .as_i64()
     *                        .expect("Could not view spender_id as i64")
     *                        as u64;
     *                    let to = transfer_from["to"]
     *                        .as_str()
     *                        .expect("Could not view to as str");
     *                    let to_id = case["named_accounts"][to]["id"]
     *                        .as_i64()
     *                        .expect("Could not view to_id as i64")
     *                        as u64;
     *                    let amount = transfer_from["amount"]
     *                        .as_i64()
     *                        .expect("Could not view amount as i64")
     *                        as u128;
     *                    let ticker = transfer_from["ticker"]
     *                        .as_str()
     *                        .expect("Coule not view ticker as str")
     *                        .to_owned();
     *                    let succeeds = transfer_from["succeeds"]
     *                        .as_bool()
     *                        .expect("Could not view succeeds as bool");
     *
     *                    println!(
     *                        "{} of token {} from {} to {} spent by {}",
     *                        amount, ticker, from, to, spender
     *                    );
     *                    let ticker = ticker.into_bytes();
     *
     *                    // Get sender's investor data
     *                    let investor_data = <InvestorList<Test>>::get(spender_id);
     *
     *                    println!("{}'s investor data: {:#?}", from, investor_data);
     *
     *                    if succeeds {
     *                        assert_ok!(Asset::transfer_from(
     *                            Origin::signed(spender_id),
     *                            ticker,
     *                            from_id,
     *                            to_id,
     *                            amount
     *                        ));
     *                    } else {
     *                        assert!(Asset::transfer_from(
     *                            Origin::signed(from_id),
     *                            ticker,
     *                            from_id,
     *                            to_id,
     *                            amount
     *                        )
     *                        .is_err());
     *                    }
     *                }
     *            });
     *        }
     *    }
     */
}
