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
//! - `batch_create_token` - Use to create the multiple security tokens in a single transaction.
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
//! - `token_details` - Returns details of the token
//! - `balance_of` - Returns the balance of the DID corresponds to the ticker
//! - `total_checkpoints_of` - Returns the checkpoint Id
//! - `total_supply_at` - Returns the total supply at a given checkpoint
//! - `custodian_allowance`- Returns the allowance provided to a custodian for a given ticker and token holder
//! - `total_custody_allowance` - Returns the total allowance approved by the token holder.

use crate::{
    balances,
    constants::*,
    general_tm, identity, percentage_tm,
    registry::{self, RegistryEntry, TokenType},
    utils,
};
use codec::Encode;
use core::result::Result as StdResult;
use currency::*;
use primitives::{IdentityId, Key};
use rstd::{convert::TryFrom, prelude::*};
use session;
use sr_primitives::traits::{CheckedAdd, CheckedSub, Verify};
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
    + registry::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
}

/// struct to store the token details
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct SecurityToken<U> {
    pub name: Vec<u8>,
    pub total_supply: U,
    pub owner_did: IdentityId,
    pub divisible: bool,
}

/// struct to store the token details
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct SignData<U> {
    custodian_did: IdentityId,
    holder_did: IdentityId,
    ticker: Vec<u8>,
    value: U,
    nonce: u16,
}

decl_storage! {
    trait Store for Module<T: Trait> as Asset {
        /// The DID of the fee collector
        FeeCollector get(fee_collector) config(): T::AccountId;
        /// details of the token corresponding to the token ticker
        /// (ticker) -> SecurityToken details [returns SecurityToken struct]
        pub Tokens get(token_details): map Vec<u8> => SecurityToken<T::Balance>;
        /// Used to store the securityToken balance corresponds to ticker and Identity
        /// (ticker, DID) -> balance
        pub BalanceOf get(balance_of): map (Vec<u8>, IdentityId) => T::Balance;
        /// (ticker, sender (DID), spender(DID)) -> allowance amount
        Allowance get(allowance): map (Vec<u8>, IdentityId, IdentityId) => T::Balance;
        /// cost in base currency to create a token
        AssetCreationFee get(asset_creation_fee) config(): T::Balance;
        /// Checkpoints created per token
        /// (ticker) -> no. of checkpoints
        pub TotalCheckpoints get(total_checkpoints_of): map (Vec<u8>) => u64;
        /// Total supply of the token at the checkpoint
        /// (ticker, checkpointId) -> total supply at given checkpoint
        pub CheckpointTotalSupply get(total_supply_at): map (Vec<u8>, u64) => T::Balance;
        /// Balance of a DID at a checkpoint
        /// (ticker, DID, checkpoint ID) -> Balance of a DID at a checkpoint
        CheckpointBalance get(balance_at_checkpoint): map (Vec<u8>, IdentityId, u64) => T::Balance;
        /// Last checkpoint updated for a DID's balance
        /// (ticker, DID) -> List of checkpoints where user balance changed
        UserCheckpoints get(user_checkpoints): map (Vec<u8>, IdentityId) => Vec<u64>;
        /// The documents attached to the tokens
        /// (ticker, document name) -> (URI, document hash)
        Documents get(documents): map (Vec<u8>, Vec<u8>) => (Vec<u8>, Vec<u8>, T::Moment);
        /// Allowance provided to the custodian
        /// (ticker, token holder, custodian) -> balance
        pub CustodianAllowance get(custodian_allowance): map(Vec<u8>, IdentityId, IdentityId) => T::Balance;
        /// Total custodian allowance for a given token holder
        /// (ticker, token holder) -> balance
        pub TotalCustodyAllowance get(total_custody_allowance): map(Vec<u8>, IdentityId) => T::Balance;
        /// Store the nonce for off chain signature to increase the custody allowance
        /// (ticker, token holder, nonce) -> bool
        AuthenticationNonce get(authentication_nonce): map(Vec<u8>, IdentityId, u16) => bool;
    }
}

// public interface for this runtime module
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// initialize the default event for this module
        fn deposit_event() = default;

        /// This function is use to create the multiple security tokens in a single transaction.
        /// This function can be used for token migrations from one blockchain to another or can be used by any
        /// whitelabler who wants to issue multiple tokens for their clients.
        ///
        /// # Arguments
        /// * `origin` It consist the signing key of the caller (i.e who signed the transaction to execute this function)
        /// * `did` DID of the creator of the tokens
        /// * `names` Array of the names of the tokens
        /// * `tickers` Array of symbols of the tokens
        /// * `total_supply_values` Array of total supply value that will be initial supply of the token
        /// * `divisible_values` Array of booleans to identify the divisibility status of the token.
        pub fn batch_create_token(origin, did: IdentityId, names: Vec<Vec<u8>>, tickers: Vec<Vec<u8>>, total_supply_values: Vec<T::Balance>, divisible_values: Vec<bool>) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from( sender.encode())?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &sender_key), "sender must be a signing key for DID");

            // Ensure we get a complete set of parameters for every token
            ensure!((names.len() == tickers.len()) == (total_supply_values.len() == divisible_values.len()), "Inconsistent token param vector lengths");

            // bytes_to_upper() all tickers
            let mut tickers = tickers;
            tickers.iter_mut().for_each(|ticker| {
                *ticker = utils::bytes_to_upper(ticker.as_slice());
            });

            // A helper vec for duplicate ticker detection
            let mut seen_tickers = Vec::new();

            let n_tokens = names.len();

            // Perform per-token checks beforehand
            for i in 0..n_tokens {
                // checking max size for name and ticker
                // byte arrays (vecs) with no max size should be avoided
                ensure!(names[i].len() <= 64, "token name cannot exceed 64 bytes");
                ensure!(tickers[i].len() <= 32, "token ticker cannot exceed 32 bytes");

                ensure!(!seen_tickers.contains(&tickers[i]), "Duplicate tickers in token batch");
                seen_tickers.push(tickers[i].clone());

                if !divisible_values[i] {
                    ensure!(total_supply_values[i] % ONE_UNIT.into() == 0.into(), "Invalid Total supply");
                }

                ensure!(total_supply_values[i] <= MAX_SUPPLY.into(), "Total supply above the limit");

                // Ensure the uniqueness of the ticker
                ensure!(!<Tokens<T>>::exists(tickers[i].clone()), "Ticker is already issued");
            }
            // TODO: Fix fee withdrawal
            // Withdraw n_tokens * Self::asset_creation_fee() from sender DID
            // let validators = <session::Module<T>>::validators();
            // let fee = Self::asset_creation_fee().checked_mul(&<FeeOf<T> as As<usize>>::sa(n_tokens)).ok_or("asset_creation_fee() * n_tokens overflows")?;
            // let validator_len;
            // if validators.len() < 1 {
            //     validator_len = <FeeOf<T> as As<usize>>::sa(1);
            // } else {
            //     validator_len = <FeeOf<T> as As<usize>>::sa(validators.len());
            // }
            // let proportional_fee = fee / validator_len;
            // let proportional_fee_in_balance = <T::CurrencyToBalance as Convert<FeeOf<T>, T::Balance>>::convert(proportional_fee);
            // for v in &validators {
            //     <balances::Module<T> as Currency<_>>::transfer(&sender, v, proportional_fee_in_balance)?;
            // }
            // let remainder_fee = fee - (proportional_fee * validator_len);
            // let remainder_fee_balance = <T::CurrencyToBalance as Convert<FeeOf<T>, T::Balance>>::convert(proportional_fee);
            // <identity::DidRecords>::mutate(did, |record| -> Result {
            //     record.balance = record.balance.checked_sub(&remainder_fee_balance).ok_or("Could not charge for token issuance")?;
            //     Ok(())
            // })?;

            // Perform per-ticker issuance
            for i in 0..n_tokens {
                let token = SecurityToken {
                    name: names[i].clone(),
                    total_supply: total_supply_values[i],
                    owner_did: did,
                    divisible: divisible_values[i]
                };

                let reg_entry = RegistryEntry { token_type: TokenType::AssetToken as u32, owner_did: did };

                <registry::Module<T>>::put(&tickers[i], &reg_entry)?;

                <Tokens<T>>::insert(&tickers[i], token);
                <BalanceOf<T>>::insert((tickers[i].clone(), did), total_supply_values[i]);
                Self::deposit_event(RawEvent::IssuedToken(tickers[i].clone(), total_supply_values[i], did, divisible_values[i]));
                sr_primitives::print("Batch token initialized");
            }

            Ok(())
        }

        /// Initializes a new security token
        /// makes the initiating account the owner of the security token
        /// & the balance of the owner is set to total supply
        ///
        /// # Arguments
        /// * `origin` It consist the signing key of the caller (i.e who signed the transaction to execute this function)
        /// * `did` DID of the creator of the token or the owner of the token
        /// * `name` Name of the token
        /// * `_ticker` Symbol of the token
        /// * `total_supply` Total supply of the token
        /// * `divisible` boolean to identify the divisibility status of the token.
        pub fn create_token(origin, did: IdentityId, name: Vec<u8>, _ticker: Vec<u8>, total_supply: T::Balance, divisible: bool) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from(sender.encode())?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &sender_key), "sender must be a signing key for DID");

            // checking max size for name and ticker
            // byte arrays (vecs) with no max size should be avoided
            ensure!(name.len() <= 64, "token name cannot exceed 64 bytes");
            ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");

            if !divisible {
                ensure!(total_supply % ONE_UNIT.into() == 0.into(), "Invalid Total supply");
            }

            ensure!(total_supply <= MAX_SUPPLY.into(), "Total supply above the limit");

            ensure!(<registry::Module<T>>::get(&ticker).is_none(), "Ticker is already taken");

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

            let token = SecurityToken {
                name,
                total_supply,
                owner_did: did,
                divisible: divisible
            };

            let reg_entry = RegistryEntry { token_type: TokenType::AssetToken as u32, owner_did: did };

            <registry::Module<T>>::put(&ticker, &reg_entry)?;

            <Tokens<T>>::insert(&ticker, token);
            <BalanceOf<T>>::insert((ticker.clone(), did), total_supply);
            Self::deposit_event(RawEvent::IssuedToken(ticker, total_supply, did, divisible));
            sr_primitives::print("Initialized!!!");

            Ok(())
        }

        /// Transfer tokens from one DID to another DID as tokens are stored/managed on the DID level
        ///
        /// # Arguments
        /// * `_origin` signing key of the sender
        /// * `did` DID of the `from` token holder, from whom tokens needs to transferred
        /// * `_ticker` Ticker of the token
        /// * `to_did` DID of the `to` token holder, to whom token needs to transferred
        /// * `value` Value that needs to transferred
        pub fn transfer(_origin, did: IdentityId, _ticker: Vec<u8>, to_did: IdentityId, value: T::Balance) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, & Key::try_from(sender.encode())?), "sender must be a signing key for DID");

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
        /// * `_ticker` symbol of the token
        /// * `from_did` DID of the token holder from whom balance token will be transferred.
        /// * `to_did` DID of token holder to whom token balance will be transferred.
        /// * `value` Amount of tokens.
        /// * `data` Some off chain data to validate the restriction.
        /// * `operator_data` It is a string which describes the reason of this control transfer call.
        pub fn controller_transfer(_origin, did: IdentityId, _ticker: Vec<u8>, from_did: IdentityId, to_did: IdentityId, value: T::Balance, data: Vec<u8>, operator_data: Vec<u8>) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, & Key::try_from( sender.encode())?), "sender must be a signing key for DID");

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
        fn approve(_origin, did: IdentityId, _ticker: Vec<u8>, spender_did: IdentityId, value: T::Balance) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, & Key::try_from( sender.encode())?), "sender must be a signing key for DID");

            ensure!(<BalanceOf<T>>::exists((ticker.clone(), did)), "Account does not own this token");

            let allowance = Self::allowance((ticker.clone(), did, spender_did));
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((ticker.clone(), did, spender_did), updated_allowance);

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
        pub fn transfer_from(_origin, did: IdentityId, _ticker: Vec<u8>, from_did: IdentityId, to_did: IdentityId, value: T::Balance) -> Result {
            let spender = ensure_signed(_origin)?;

            // Check that spender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, & Key::try_from( spender.encode())?), "sender must be a signing key for DID");

            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let ticker_from_did_did = (ticker.clone(), from_did, did);
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
        pub fn create_checkpoint(_origin, did: IdentityId, _ticker: Vec<u8>) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, & Key::try_from( sender.encode())?), "sender must be a signing key for DID");

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
        pub fn issue(origin, did: IdentityId, ticker: Vec<u8>, to_did: IdentityId, value: T::Balance, _data: Vec<u8>) -> Result {
            let upper_ticker = utils::bytes_to_upper(&ticker);
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, & Key::try_from( sender.encode())?), "sender must be a signing key for DID");

            ensure!(Self::is_owner(&upper_ticker, did), "user is not authorized");
            Self::_mint(&upper_ticker, to_did, value)
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
        pub fn batch_issue(origin, did: IdentityId, ticker: Vec<u8>, investor_dids: Vec<IdentityId>, values: Vec<T::Balance>) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from( sender.encode())?), "sender must be a signing key for DID");

            ensure!(investor_dids.len() == values.len(), "Investor/amount list length inconsistent");

            ensure!(Self::is_owner(&ticker, did), "user is not authorized");


            // A helper vec for calculated new investor balances
            let mut updated_balances = Vec::with_capacity(investor_dids.len());

            // A helper vec for calculated new investor balances
            let mut current_balances = Vec::with_capacity(investor_dids.len());

            // Get current token details for supply update
            let mut token = Self::token_details(ticker.clone());

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

                current_balances.push(Self::balance_of((ticker.clone(), investor_dids[i].clone())));
                updated_balances.push(current_balances[i]
                    .checked_add(&values[i])
                    .ok_or("overflow in calculating balance")?);

                // verify transfer check
                ensure!(Self::_is_valid_transfer(&ticker, None, Some(investor_dids[i]), values[i])? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");

                // New total supply must be valid
                token.total_supply = updated_total_supply;
            }

            // After checks are ensured introduce side effects
            for i in 0..investor_dids.len() {
                Self::_update_checkpoint(&ticker, investor_dids[i], current_balances[i]);

                <BalanceOf<T>>::insert((ticker.clone(), investor_dids[i]), updated_balances[i]);

                Self::deposit_event(RawEvent::Issued(ticker.clone(), investor_dids[i], values[i]));
            }
            <Tokens<T>>::insert(ticker.clone(), token);

            Ok(())
        }

        /// Used to redeem the security tokens
        ///
        /// # Arguments
        /// * `_origin` Signing key of the token holder who wants to redeem the tokens
        /// * `did` DID of the token holder
        /// * `_ticker` Ticker of the token
        /// * `value` Amount of the tokens needs to redeem
        /// * `_data` An off chain data blob used to validate the redeem functionality.
        pub fn redeem(_origin, did: IdentityId, _ticker: Vec<u8>, value: T::Balance, _data: Vec<u8>) -> Result {
            let upper_ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            // Granularity check
            ensure!(
                Self::check_granularity(&upper_ticker, value),
                "Invalid granularity"
                );
            let ticker_did = (upper_ticker.clone(), did);
            ensure!(<BalanceOf<T>>::exists(&ticker_did), "Account does not own this token");
            let burner_balance = Self::balance_of(&ticker_did);
            ensure!(burner_balance >= value, "Not enough balance.");

            // Reduce sender's balance
            let updated_burner_balance = burner_balance
                .checked_sub(&value)
                .ok_or("overflow in calculating balance")?;
            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&upper_ticker, did, value)?;

            // verify transfer check
            ensure!(Self::_is_valid_transfer(&upper_ticker, Some(did), None, value)? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");

            //Decrease total supply
            let mut token = Self::token_details(&upper_ticker);
            token.total_supply = token.total_supply.checked_sub(&value).ok_or("overflow in calculating balance")?;

            Self::_update_checkpoint(&upper_ticker, did, burner_balance);

            <BalanceOf<T>>::insert((upper_ticker.clone(), did), updated_burner_balance);
            <Tokens<T>>::insert(&upper_ticker, token);

            Self::deposit_event(RawEvent::Redeemed(upper_ticker, did, value));

            Ok(())

        }

        /// Used to redeem the security tokens by some other DID who has approval
        ///
        /// # Arguments
        /// * `_origin` Signing key of the spender who has valid approval to redeem the tokens
        /// * `did` DID of the spender
        /// * `_ticker` Ticker of the token
        /// * `from_did` DID from whom balance get reduced
        /// * `value` Amount of the tokens needs to redeem
        /// * `_data` An off chain data blob used to validate the redeem functionality.
        pub fn redeem_from(_origin, did: IdentityId, _ticker: Vec<u8>, from_did: IdentityId, value: T::Balance, _data: Vec<u8>) -> Result {
            let upper_ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            // Granularity check
            ensure!(
                Self::check_granularity(&upper_ticker, value),
                "Invalid granularity"
                );
            let ticker_did = (upper_ticker.clone(), did);
            ensure!(<BalanceOf<T>>::exists(&ticker_did), "Account does not own this token");
            let burner_balance = Self::balance_of(&ticker_did);
            ensure!(burner_balance >= value, "Not enough balance.");

            // Reduce sender's balance
            let updated_burner_balance = burner_balance
                .checked_sub(&value)
                .ok_or("overflow in calculating balance")?;

            let ticker_from_did_did = (upper_ticker.clone(), from_did, did);
            ensure!(<Allowance<T>>::exists(&ticker_from_did_did), "Allowance does not exist");
            let allowance = Self::allowance(&ticker_from_did_did);
            ensure!(allowance >= value, "Not enough allowance");
            // Check whether the custody allowance remain intact or not
            Self::_check_custody_allowance(&upper_ticker, did, value)?;
            ensure!(Self::_is_valid_transfer( &upper_ticker, Some(from_did), None, value)? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");

            let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;

            //Decrease total suply
            let mut token = Self::token_details(&upper_ticker);
            token.total_supply = token.total_supply.checked_sub(&value).ok_or("overflow in calculating balance")?;

            Self::_update_checkpoint(&upper_ticker, did, burner_balance);

            <Allowance<T>>::insert(&ticker_from_did_did, updated_allowance);
            <BalanceOf<T>>::insert(&ticker_did, updated_burner_balance);
            <Tokens<T>>::insert(&upper_ticker, token);

            Self::deposit_event(RawEvent::Redeemed(upper_ticker.clone(), did, value));
            Self::deposit_event(RawEvent::Approval(upper_ticker, from_did, did, value));

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
        pub fn controller_redeem(origin, did: IdentityId, ticker: Vec<u8>, token_holder_did: IdentityId, value: T::Balance, data: Vec<u8>, operator_data: Vec<u8>) -> Result {
            let ticker = utils::bytes_to_upper(ticker.as_slice());
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");
            ensure!(Self::is_owner(&ticker, did), "user is not token owner");

            // Granularity check
            ensure!(
                Self::check_granularity(&ticker, value),
                "Invalid granularity"
                );
            let ticker_token_holder_did = (ticker.clone(), token_holder_did);
            ensure!(<BalanceOf<T>>::exists( &ticker_token_holder_did), "Account does not own this token");
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
        pub fn make_divisible(origin, did: IdentityId, ticker: Vec<u8>) -> Result {
            let ticker = utils::bytes_to_upper(ticker.as_slice());
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

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
        pub fn can_transfer(_origin, ticker: Vec<u8>, from_did: IdentityId, to_did: IdentityId, value: T::Balance, data: Vec<u8>) {
            let mut current_balance: T::Balance = Self::balance_of((ticker.clone(), from_did));
            if current_balance < value {
                current_balance = 0.into();
            } else {
                current_balance = current_balance - value;
            }
            if current_balance < Self::total_custody_allowance((ticker.clone(), from_did)) {
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
        pub fn transfer_with_data(origin, did: IdentityId, ticker: Vec<u8>, to_did: IdentityId, value: T::Balance, data: Vec<u8>) -> Result {
            Self::transfer(origin, did, ticker.clone(), to_did, value)?;
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
        pub fn transfer_from_with_data(origin, did: IdentityId, ticker: Vec<u8>, from_did: IdentityId, to_did: IdentityId, value: T::Balance, data: Vec<u8>) -> Result {
            Self::transfer_from(origin, did, ticker.clone(), from_did,  to_did, value)?;
            Self::deposit_event(RawEvent::TransferWithData(ticker, from_did, to_did, value, data));
            Ok(())
        }

        /// Used to know whether the given token will issue new tokens or not
        ///
        /// # Arguments
        /// * `_origin` Signing key
        /// * `ticker` Ticker of the token whose issuance status need to know
        pub fn is_issuable(_origin, ticker: Vec<u8>) {
            Self::deposit_event(RawEvent::IsIssuable(ticker, true));
        }

        /// Used to get the documents details attach with the token
        ///
        /// # Arguments
        /// * `_origin` Caller signing key
        /// * `ticker` Ticker of the token
        /// * `name` Name of the document
        pub fn get_document(_origin, ticker: Vec<u8>, name: Vec<u8>) -> Result {
            let record = <Documents<T>>::get((ticker.clone(), name.clone()));
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
        pub fn set_document(origin, did: IdentityId, ticker: Vec<u8>, name: Vec<u8>, uri: Vec<u8>, document_hash: Vec<u8>) -> Result {
            let ticker = utils::bytes_to_upper(ticker.as_slice());
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");
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
        pub fn remove_document(origin, did: IdentityId, ticker: Vec<u8>, name: Vec<u8>) -> Result {
            let ticker = utils::bytes_to_upper(ticker.as_slice());
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");
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
        pub fn increase_custody_allowance(origin, ticker: Vec<u8>, holder_did: IdentityId, custodian_did: IdentityId, value: T::Balance) -> Result {
            let ticker = utils::bytes_to_upper(ticker.as_slice());
            let sender = ensure_signed(origin)?;
            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_authorized_key(holder_did, &Key::try_from(sender.encode())?),
                "sender must be a signing key for DID"
            );
            Self::_increase_custody_allowance(ticker.clone(), holder_did, custodian_did, value)?;
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
        pub fn increase_custody_allowance_of(origin, ticker: Vec<u8>, holder_did: IdentityId, holder_account_id: T::AccountId, custodian_did: IdentityId, caller_did: IdentityId,  value: T::Balance, nonce: u16, signature: T::OffChainSignature) -> Result {
            let ticker = utils::bytes_to_upper(ticker.as_slice());
            let sender = ensure_signed(origin)?;

            ensure!(!Self::authentication_nonce((ticker.clone(), holder_did, nonce)), "Signature already used");

            let msg = SignData {
                custodian_did: custodian_did,
                holder_did: holder_did,
                ticker: ticker.clone(),
                value,
                nonce
            };
            // holder_account_id should be a part of the holder_did
            ensure!(signature.verify(&msg.encode()[..], &holder_account_id), "Invalid signature");
            ensure!(
                <identity::Module<T>>::is_authorized_key(caller_did, &Key::try_from(sender.encode())?),
                "sender must be a signing key for DID"
            );
            // Validate the holder signing key
            ensure!(
                <identity::Module<T>>::is_authorized_key(holder_did, &Key::try_from(holder_account_id.encode())?),
                "holder signing key must be a signing key for holder DID"
            );
            Self::_increase_custody_allowance(ticker.clone(), holder_did, custodian_did, value)?;
            <AuthenticationNonce>::insert((ticker.clone(), holder_did, nonce), true);
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
        pub fn transfer_by_custodian(origin, ticker: Vec<u8>, holder_did: IdentityId, custodian_did: IdentityId, receiver_did: IdentityId, value: T::Balance) -> Result {
            let ticker = utils::bytes_to_upper(ticker.as_slice());
            let sender = ensure_signed(origin)?;
            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_authorized_key(custodian_did, &Key::try_from(sender.encode())?),
                "sender must be a signing key for DID"
            );
            let mut custodian_allowance = Self::custodian_allowance((ticker.clone(), holder_did, custodian_did));
            // Check whether the custodian has enough allowance or not
            ensure!(custodian_allowance >= value, "Insufficient allowance");
            // using checked_sub (safe math) to avoid underflow
            custodian_allowance = custodian_allowance.checked_sub(&value).ok_or("underflow in calculating allowance")?;
            // using checked_sub (safe math) to avoid underflow
            let new_total_allowance = Self::total_custody_allowance((ticker.clone(), holder_did))
                .checked_sub(&value)
                .ok_or("underflow in calculating the total allowance")?;
            // Validate the transfer
            ensure!(Self::_is_valid_transfer(&ticker, Some(holder_did), Some(receiver_did), value)? == ERC1400_TRANSFER_SUCCESS, "Transfer restrictions failed");
            Self::_transfer(&ticker, holder_did, receiver_did, value)?;
            // Update Storage of allowance
            <CustodianAllowance<T>>::insert((ticker.clone(), custodian_did, holder_did), &custodian_allowance);
            <TotalCustodyAllowance<T>>::insert((ticker.clone(), holder_did), new_total_allowance);
            Self::deposit_event(RawEvent::CustodyTransfer(ticker.clone(), custodian_did, holder_did, receiver_did, value));
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
            Transfer(Vec<u8>, IdentityId, IdentityId, Balance),
            /// event when an approval is made
            /// ticker, owner DID, spender DID, value
            Approval(Vec<u8>, IdentityId, IdentityId, Balance),
            /// emit when tokens get issued
            /// ticker, beneficiary DID, value
            Issued(Vec<u8>, IdentityId, Balance),
            /// emit when tokens get redeemed
            /// ticker, DID, value
            Redeemed(Vec<u8>, IdentityId, Balance),
            /// event for forced transfer of tokens
            /// ticker, controller DID, from DID, to DID, value, data, operator data
            ControllerTransfer(Vec<u8>, IdentityId, IdentityId, IdentityId, Balance, Vec<u8>, Vec<u8>),
            /// event for when a forced redemption takes place
            /// ticker, controller DID, token holder DID, value, data, operator data
            ControllerRedemption(Vec<u8>, IdentityId, IdentityId, Balance, Vec<u8>, Vec<u8>),
            /// Event for creation of the asset
            /// ticker, total supply, owner DID, divisibility
            IssuedToken(Vec<u8>, Balance, IdentityId, bool),
            /// Event for change in divisibility
            /// ticker, divisibility
            DivisibilityChanged(Vec<u8>, bool),
            /// can_transfer() output
            /// ticker, from_did, to_did, value, data, ERC1066 status
            /// 0 - OK
            /// 1,2... - Error, meanings TBD
            CanTransfer(Vec<u8>, IdentityId, IdentityId, Balance, Vec<u8>, u32),
            /// An additional event to Transfer; emitted when transfer_with_data is called; similar to
            /// Transfer with data added at the end.
            /// ticker, from DID, to DID, value, data
            TransferWithData(Vec<u8>, IdentityId, IdentityId, Balance, Vec<u8>),
            /// is_issuable() output
            /// ticker, return value (true if issuable)
            IsIssuable(Vec<u8>, bool),
            /// get_document() output
            /// ticker, name, uri, hash, last modification date
            GetDocument(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Moment),
            /// emit when tokens transferred by the custodian
            /// ticker, custodian did, holder/from did, to did, amount
            CustodyTransfer(Vec<u8>, IdentityId, IdentityId, IdentityId, Balance),
            /// emit when allowance get increased
            /// ticker, holder did, custodian did, oldAllowance, newAllowance
            CustodyAllowanceChanged(Vec<u8>, IdentityId, IdentityId, Balance, Balance),
        }
}

pub trait AssetTrait<V> {
    fn total_supply(ticker: &[u8]) -> V;
    fn balance(ticker: &[u8], did: IdentityId) -> V;
    fn _mint_from_sto(ticker: &[u8], sender_did: IdentityId, tokens_purchased: V) -> Result;
    fn is_owner(ticker: &Vec<u8>, did: IdentityId) -> bool;
    fn get_balance_at(ticker: &Vec<u8>, did: IdentityId, at: u64) -> V;
}

impl<T: Trait> AssetTrait<T::Balance> for Module<T> {
    fn _mint_from_sto(ticker: &[u8], sender: IdentityId, tokens_purchased: T::Balance) -> Result {
        let upper_ticker = utils::bytes_to_upper(ticker);
        Self::_mint(&upper_ticker, sender, tokens_purchased)
    }

    fn is_owner(ticker: &Vec<u8>, did: IdentityId) -> bool {
        Self::_is_owner(ticker, did)
    }

    /// Get the asset `id` balance of `who`.
    fn balance(ticker: &[u8], who: IdentityId) -> T::Balance {
        let upper_ticker = utils::bytes_to_upper(ticker);
        return Self::balance_of((upper_ticker, who));
    }

    // Get the total supply of an asset `id`
    fn total_supply(ticker: &[u8]) -> T::Balance {
        let upper_ticker = utils::bytes_to_upper(ticker);
        return Self::token_details(upper_ticker).total_supply;
    }

    fn get_balance_at(ticker: &Vec<u8>, did: IdentityId, at: u64) -> T::Balance {
        let upper_ticker = utils::bytes_to_upper(ticker);
        return Self::get_balance_at(&upper_ticker, did, at);
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
    pub fn _is_owner(ticker: &Vec<u8>, did: IdentityId) -> bool {
        let token = Self::token_details(ticker);
        token.owner_did == did
    }

    /// Get the asset `id` balance of `who`.
    pub fn balance(ticker: &Vec<u8>, did: IdentityId) -> T::Balance {
        let upper_ticker = utils::bytes_to_upper(ticker);
        Self::balance_of((upper_ticker, did))
    }

    // Get the total supply of an asset `id`
    pub fn total_supply(ticker: &[u8]) -> T::Balance {
        let upper_ticker = utils::bytes_to_upper(ticker);
        Self::token_details(upper_ticker).total_supply
    }

    pub fn get_balance_at(ticker: &Vec<u8>, did: IdentityId, at: u64) -> T::Balance {
        let upper_ticker = utils::bytes_to_upper(ticker);
        let ticker_did = (upper_ticker.clone(), did);
        if !<TotalCheckpoints>::exists(upper_ticker.clone()) ||
            at == 0 || //checkpoints start from 1
            at > Self::total_checkpoints_of(&upper_ticker)
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
                upper_ticker.clone(),
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
        ticker: &Vec<u8>,
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
        ticker: &Vec<u8>,
        from_did: IdentityId,
        to_did: IdentityId,
        value: T::Balance,
    ) -> Result {
        // Granularity check
        ensure!(
            Self::check_granularity(ticker, value),
            "Invalid granularity"
        );
        let ticket_from_did = (ticker.clone(), from_did);
        ensure!(
            <BalanceOf<T>>::exists(&ticket_from_did),
            "Account does not own this token"
        );
        let sender_balance = Self::balance_of(&ticket_from_did);
        ensure!(sender_balance >= value, "Not enough balance.");

        let updated_from_balance = sender_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating balance")?;
        let ticket_to_did = (ticker.clone(), to_did);
        let receiver_balance = Self::balance_of(&ticket_to_did);
        let updated_to_balance = receiver_balance
            .checked_add(&value)
            .ok_or("overflow in calculating balance")?;

        Self::_update_checkpoint(ticker, from_did, sender_balance);
        Self::_update_checkpoint(ticker, to_did, receiver_balance);
        // reduce sender's balance
        <BalanceOf<T>>::insert(ticket_from_did, updated_from_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert(ticket_to_did, updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(ticker.clone(), from_did, to_did, value));
        Ok(())
    }

    pub fn _create_checkpoint(ticker: &Vec<u8>) -> Result {
        if <TotalCheckpoints>::exists(ticker) {
            let mut checkpoint_count = Self::total_checkpoints_of(ticker);
            checkpoint_count = checkpoint_count
                .checked_add(1)
                .ok_or("overflow in adding checkpoint")?;
            <TotalCheckpoints>::insert(ticker, checkpoint_count);
            <CheckpointTotalSupply<T>>::insert(
                (ticker.clone(), checkpoint_count),
                Self::token_details(ticker).total_supply,
            );
        } else {
            <TotalCheckpoints>::insert(ticker, 1);
            <CheckpointTotalSupply<T>>::insert(
                (ticker.clone(), 1),
                Self::token_details(ticker).total_supply,
            );
        }
        Ok(())
    }

    fn _update_checkpoint(ticker: &Vec<u8>, user_did: IdentityId, user_balance: T::Balance) {
        if <TotalCheckpoints>::exists(ticker) {
            let checkpoint_count = Self::total_checkpoints_of(ticker);
            let ticker_user_did_checkpont = (ticker.clone(), user_did, checkpoint_count);
            if !<CheckpointBalance<T>>::exists(&ticker_user_did_checkpont) {
                <CheckpointBalance<T>>::insert(&ticker_user_did_checkpont, user_balance);
                <UserCheckpoints>::mutate((ticker.clone(), user_did), |user_checkpoints| {
                    user_checkpoints.push(checkpoint_count);
                });
            }
        }
    }

    fn is_owner(ticker: &Vec<u8>, did: IdentityId) -> bool {
        Self::_is_owner(ticker, did)
    }

    pub fn _mint(ticker: &Vec<u8>, to_did: IdentityId, value: T::Balance) -> Result {
        // Granularity check
        ensure!(
            Self::check_granularity(ticker, value),
            "Invalid granularity"
        );
        //Increase receiver balance
        let ticker_to_did = (ticker.clone(), to_did);
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

        Self::deposit_event(RawEvent::Issued(ticker.clone(), to_did, value));

        Ok(())
    }

    fn check_granularity(ticker: &Vec<u8>, value: T::Balance) -> bool {
        // Read the token details
        let token = Self::token_details(ticker);
        token.divisible || value % ONE_UNIT.into() == 0.into()
    }

    fn _check_custody_allowance(
        ticker: &Vec<u8>,
        holder_did: IdentityId,
        value: T::Balance,
    ) -> Result {
        let remaining_balance = Self::balance_of((ticker.clone(), holder_did))
            .checked_sub(&value)
            .ok_or("underflow in balance deduction")?;
        ensure!(
            remaining_balance >= Self::total_custody_allowance((ticker.clone(), holder_did)),
            "Insufficient balance for transfer"
        );
        Ok(())
    }

    fn _increase_custody_allowance(
        ticker: Vec<u8>,
        holder_did: IdentityId,
        custodian_did: IdentityId,
        value: T::Balance,
    ) -> Result {
        let new_custody_allowance = Self::total_custody_allowance((ticker.clone(), holder_did))
            .checked_add(&value)
            .ok_or("total custody allowance get overflowed")?;
        // Ensure that balance of the token holder should greater than or equal to the total custody allowance + value
        ensure!(
            Self::balance_of((ticker.clone(), holder_did)) >= new_custody_allowance,
            "Insufficient balance of holder did"
        );
        // Ensure the valid DID
        ensure!(
            <identity::DidRecords>::exists(custodian_did),
            "Invalid custodian DID"
        );

        let old_allowance = Self::custodian_allowance((ticker.clone(), holder_did, custodian_did));
        let new_current_allowance = old_allowance
            .checked_add(&value)
            .ok_or("allowance get overflowed")?;
        // Update Storage
        <CustodianAllowance<T>>::insert(
            (ticker.clone(), holder_did, custodian_did),
            &new_current_allowance,
        );
        <TotalCustodyAllowance<T>>::insert((ticker.clone(), holder_did), new_custody_allowance);
        Self::deposit_event(RawEvent::CustodyAllowanceChanged(
            ticker.clone(),
            holder_did,
            custodian_did,
            old_allowance,
            new_current_allowance,
        ));
        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{exemption, identity};
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

    impl registry::Trait for Test {}
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
    fn issuers_can_create_tokens() {
        with_externalities(&mut identity_owned_by_alice(), || {
            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();
            // Raise the owner's base currency balance
            Balances::make_free_balance_be(&owner_acc, 1_000_000);

            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: owner_did,
                total_supply: 1_000_000,
                divisible: true,
            };

            assert_err!(
                Asset::create_token(
                    owner_signed.clone(),
                    owner_did,
                    token.name.clone(),
                    token.name.clone(),
                    1_000_000_000_000_000_000_000_000, // Total supply over the limit
                    true
                ),
                "Total supply above the limit"
            );

            // Issuance is successful
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                token.name.clone(),
                token.total_supply,
                true
            ));

            // A correct entry is added
            assert_eq!(Asset::token_details(token.name.clone()), token);
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
            };

            Balances::make_free_balance_be(&owner_acc, 1_000_000);

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (_, alice_did) = make_account(&alice_acc).unwrap();

            Balances::make_free_balance_be(&alice_acc, 1_000_000);

            // Issuance is successful
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                token.name.clone(),
                token.total_supply,
                true
            ));

            // A correct entry is added
            assert_eq!(Asset::token_details(token.name.clone()), token);

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                asset_rule
            ));

            assert_ok!(Asset::transfer(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
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
            };

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
                token.name.clone(),
                token.total_supply,
                true
            ));

            assert_eq!(
                Asset::balance_of((token.name.clone(), token.owner_did)),
                token.total_supply
            );

            assert_eq!(Asset::token_details(token.name.clone()), token);

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                asset_rule
            ));

            // Mint some tokens to investor1
            assert_ok!(Asset::issue(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                investor1_did,
                200_00_00 as u128,
                vec![0x0]
            ));

            assert_eq!(
                Asset::balance_of((token.name.clone(), investor1_did)),
                200_00_00 as u128
            );

            // Failed to add custodian because of insufficient balance
            assert_noop!(
                Asset::increase_custody_allowance(
                    investor1_signed.clone(),
                    token.name.clone(),
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
                    token.name.clone(),
                    investor1_did,
                    custodian_did_not_register,
                    50_00_00 as u128
                ),
                "Invalid custodian DID"
            );

            // Add custodian
            assert_ok!(Asset::increase_custody_allowance(
                investor1_signed.clone(),
                token.name.clone(),
                investor1_did,
                custodian_did,
                50_00_00 as u128
            ));

            assert_eq!(
                Asset::custodian_allowance((token.name.clone(), investor1_did, custodian_did)),
                50_00_00 as u128
            );

            assert_eq!(
                Asset::total_custody_allowance((token.name.clone(), investor1_did)),
                50_00_00 as u128
            );

            // Transfer the token upto the limit
            assert_ok!(Asset::transfer(
                investor1_signed.clone(),
                investor1_did,
                token.name.clone(),
                investor2_did,
                140_00_00 as u128
            ));

            assert_eq!(
                Asset::balance_of((token.name.clone(), investor2_did)),
                140_00_00 as u128
            );

            // Try to Transfer the tokens beyond the limit
            assert_noop!(
                Asset::transfer(
                    investor1_signed.clone(),
                    investor1_did,
                    token.name.clone(),
                    investor2_did,
                    50_00_00 as u128
                ),
                "Insufficient balance for transfer"
            );

            // Should fail to transfer the token by the custodian because of invalid signing key
            assert_noop!(
                Asset::transfer_by_custodian(
                    investor2_signed.clone(),
                    token.name.clone(),
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
                    token.name.clone(),
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
                token.name.clone(),
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
            };

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
                token.name.clone(),
                token.total_supply,
                true
            ));

            assert_eq!(
                Asset::balance_of((token.name.clone(), token.owner_did)),
                token.total_supply
            );

            assert_eq!(Asset::token_details(token.name.clone()), token);

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                asset_rule
            ));

            // Mint some tokens to investor1
            assert_ok!(Asset::issue(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                investor1_did,
                200_00_00 as u128,
                vec![0x0]
            ));

            assert_eq!(
                Asset::balance_of((token.name.clone(), investor1_did)),
                200_00_00 as u128
            );

            let msg = SignData {
                custodian_did: custodian_did,
                holder_did: investor1_did,
                ticker: token.name.clone(),
                value: 50_00_00 as u128,
                nonce: 1,
            };

            let investor1_key = AccountKeyring::Bob;

            // Add custodian
            assert_ok!(Asset::increase_custody_allowance_of(
                investor2_signed.clone(),
                token.name.clone(),
                investor1_did,
                investor1_acc.clone(),
                custodian_did,
                investor2_did,
                50_00_00 as u128,
                1,
                OffChainSignature::from(investor1_key.sign(&msg.encode()))
            ));

            assert_eq!(
                Asset::custodian_allowance((token.name.clone(), investor1_did, custodian_did)),
                50_00_00 as u128
            );

            assert_eq!(
                Asset::total_custody_allowance((token.name.clone(), investor1_did)),
                50_00_00 as u128
            );

            // use the same signature with the same nonce should fail
            assert_noop!(
                Asset::increase_custody_allowance_of(
                    investor2_signed.clone(),
                    token.name.clone(),
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
                    token.name.clone(),
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
                token.name.clone(),
                investor2_did,
                140_00_00 as u128
            ));

            assert_eq!(
                Asset::balance_of((token.name.clone(), investor2_did)),
                140_00_00 as u128
            );

            // Try to Transfer the tokens beyond the limit
            assert_noop!(
                Asset::transfer(
                    investor1_signed.clone(),
                    investor1_did,
                    token.name.clone(),
                    investor2_did,
                    50_00_00 as u128
                ),
                "Insufficient balance for transfer"
            );

            // Should fail to transfer the token by the custodian because of invalid signing key
            assert_noop!(
                Asset::transfer_by_custodian(
                    investor2_signed.clone(),
                    token.name.clone(),
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
                    token.name.clone(),
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
                token.name.clone(),
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
        for i in 0..10 {
            // When fuzzing in local, feel free to bump this number to add more fuzz runs.
            with_externalities(&mut identity_owned_by_alice(), || {
                let now = Utc::now();
                <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

                let owner_acc = AccountId::from(AccountKeyring::Dave);
                let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

                // Expected token entry
                let token = SecurityToken {
                    name: vec![0x01],
                    owner_did: owner_did.clone(),
                    total_supply: 1_000_000,
                    divisible: true,
                };

                let bob_acc = AccountId::from(AccountKeyring::Bob);
                let (_, bob_did) = make_account(&bob_acc).unwrap();

                // Issuance is successful
                assert_ok!(Asset::create_token(
                    owner_signed.clone(),
                    owner_did,
                    token.name.clone(),
                    token.name.clone(),
                    token.total_supply,
                    true
                ));

                let asset_rule = general_tm::AssetRule {
                    sender_rules: vec![],
                    receiver_rules: vec![],
                };

                // Allow all transfers
                assert_ok!(GeneralTM::add_active_rule(
                    owner_signed.clone(),
                    owner_did,
                    token.name.clone(),
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
                            owner_did.clone(),
                            token.name.clone(),
                            bob_did.clone(),
                            1
                        ));
                    }
                    assert_ok!(Asset::create_checkpoint(
                        owner_signed.clone(),
                        owner_did.clone(),
                        token.name.clone(),
                    ));
                    let x: u64 = u64::try_from(j).unwrap();
                    assert_eq!(
                        Asset::get_balance_at(&token.name, owner_did, 0),
                        owner_balance[j]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, bob_did, 0),
                        bob_balance[j]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, owner_did, 1),
                        owner_balance[1]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, bob_did, 1),
                        bob_balance[1]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, owner_did, x - 1),
                        owner_balance[j - 1]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, bob_did, x - 1),
                        bob_balance[j - 1]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, owner_did, x),
                        owner_balance[j]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, bob_did, x),
                        bob_balance[j]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, owner_did, x + 1),
                        owner_balance[j]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, bob_did, x + 1),
                        bob_balance[j]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, owner_did, 1000),
                        owner_balance[j]
                    );
                    assert_eq!(
                        Asset::get_balance_at(&token.name, bob_did, 1000),
                        bob_balance[j]
                    );
                }
            });
            println!("Instance {} done", i);
        }
        println!("Done");
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
     *                        name: ticker.to_owned().into_bytes(),
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
     *                                ticker.to_owned().into_bytes(),
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
     *                            ticker.to_owned().into_bytes(),
     *                            spender_id,
     *                            amount,
     *                        ));
     *                    } else {
     *                        assert!(Asset::approve(
     *                            Origin::signed(sender_id),
     *                            ticker.to_owned().into_bytes(),
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
