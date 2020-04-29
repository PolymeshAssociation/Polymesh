//! # Simple Token Module
//!
//! The Simple Token module provides functionality for issuing and managing tokens which do not have transfer restrictions.
//!
//! ## Overview
//!
//! The Simple Token module provides functions for:
//!
//! - Creating a simple token with an inital balance
//! - Transfering simple tokens between identities
//! - Approving simple tokens to be transferred on your behalf by another identity
//!
//! ### Use case
//!
//! In some cases the asset module may be unnecessary. For example a token representing USD may not need transfer restrictions
//! that are typically associated with securities.
//!
//! In other cases a simple token may be used to represent a wrapped asset that originates on a different chain, for example BTC,
//! which by its nature does not need transfer restrictions.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `create_token` - Creates a new simple token and mints a balance to the issuer
//! - `approve` - Approves another identity to transfer tokens on behalf of the caller
//! - `transfer` - Transfers simple tokens to another identity
//! - `transfer_from` - Transfers simple tokens to another identity using the approval process
//!
//! ### Public Functions
//!
//! - `balance_of` - Returns the simple token balance associated with an identity

use pallet_identity as identity;
use polymesh_common_utilities::{
    balances::Trait as BalancesTrait, constants::currency::MAX_SUPPLY,
    identity::Trait as IdentityTrait, CommonTrait, Context,
};
use polymesh_primitives::{AccountKey, IdentityId, Signatory, Ticker};

use codec::Encode;
use sp_std::{convert::TryFrom, prelude::*};

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{CheckedAdd, CheckedSub};

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + BalancesTrait + IdentityTrait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

/// Struct to store the details of each simple token
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct SimpleTokenRecord<U> {
    pub ticker: Ticker,
    pub total_supply: U,
    pub owner_did: IdentityId,
}

decl_storage! {
    trait Store for Module<T: Trait> as SimpleToken {
        /// Mapping from (ticker, owner DID, spender DID) to allowance amount
        Allowance get(fn allowance): map hasher(blake2_128_concat) (Ticker, IdentityId, IdentityId) => T::Balance;
        /// Mapping from (ticker, owner DID) to their balance
        pub BalanceOf get(fn balance_of): map hasher(blake2_128_concat) (Ticker, IdentityId) => T::Balance;
        /// The details associated with each simple token
        Tokens get(fn tokens): map hasher(blake2_128_concat) Ticker => SimpleTokenRecord<T::Balance>;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The sender must be a signing key for the DID.
        SenderMustBeSigningKeyForDid,
        /// A ticker with this name already exists.
        TickerAlreadyExists,
        /// The total supply is above the limit.
        TotalSupplyAboveLimit,
        /// The sender is not a token owner.
        NotAnOwner,
        /// An overflow while calculating the allowance.
        AllowanceOverflow,
        /// No such allowance.
        NoSuchAllowance,
        /// Insufficient allowance.
        InsufficientAllowance,
        /// Sender balance underflow.
        BalanceUnderflow,
        /// Recipient balance overflow.
        BalanceOverflow,
        /// Insufficient balance.
        InsufficientBalance,
    }
}

type Identity<T> = identity::Module<T>;

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        /// Create a new token and mint a balance to the issuing identity
        #[weight = SimpleDispatchInfo::FixedNormal(200_000)]
        pub fn create_token(origin, ticker: Ticker, total_supply: T::Balance) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            ensure!(!<Tokens<T>>::contains_key(&ticker), Error::<T>::TickerAlreadyExists);
            ensure!(total_supply <= MAX_SUPPLY.into(), Error::<T>::TotalSupplyAboveLimit);

            let new_token = SimpleTokenRecord {
                ticker: ticker,
                total_supply: total_supply,
                owner_did: did,
            };

            <Tokens<T>>::insert(&ticker, new_token);
            // Let the owner distribute the whole supply of the token
            <BalanceOf<T>>::insert((ticker, did), total_supply);

            sp_runtime::print("Initialized a new token");

            Self::deposit_event(RawEvent::TokenCreated(ticker, did, total_supply));

            Ok(())
        }

        /// Approve another identity to transfer tokens on behalf of the caller
        #[weight = SimpleDispatchInfo::FixedNormal(150_000)]
        pub fn approve(origin, ticker: Ticker, spender_did: IdentityId, value: T::Balance) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            let ticker_did = (ticker, did);
            ensure!(<BalanceOf<T>>::contains_key(&ticker_did), Error::<T>::NotAnOwner);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );

            let ticker_did_spender_did = (ticker, did, spender_did);
            let allowance = Self::allowance(&ticker_did_spender_did);
            let updated_allowance = allowance.checked_add(&value).ok_or(Error::<T>::AllowanceOverflow)?;
            <Allowance<T>>::insert(&ticker_did_spender_did, updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker, did, spender_did, value));

            Ok(())
        }

        /// Transfer tokens to another identity
        #[weight = SimpleDispatchInfo::FixedNormal(300_000)]
        pub fn transfer(origin, ticker: Ticker, to_did: IdentityId, amount: T::Balance) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );

            Self::_transfer(&ticker, did, to_did, amount)
        }

        /// Transfer tokens to another identity using the approval mechanic
        #[weight = SimpleDispatchInfo::FixedNormal(400_000)]
        pub fn transfer_from(origin, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, amount: T::Balance) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let spender = Signatory::AccountKey(sender_key);

            // Check that spender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &spender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            let ticker_from_did_did = (ticker, from_did, did);
            ensure!(<Allowance<T>>::contains_key(&ticker_from_did_did), Error::<T>::NoSuchAllowance);
            let allowance = Self::allowance(&ticker_from_did_did);
            ensure!(allowance >= amount, Error::<T>::InsufficientAllowance);

            // Needs to happen before allowance subtraction so that the from balance is checked in _transfer
            Self::_transfer(&ticker, from_did, to_did, amount)?;

            // using checked_sub (safe math) to avoid overflow
            let updated_allowance = allowance.checked_sub(&amount)
                .ok_or(Error::<T>::AllowanceOverflow)?;
            <Allowance<T>>::insert((ticker, from_did, did), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker, from_did, did, updated_allowance));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
    {
        /// ticker, from DID, spender DID, amount
        Approval(Ticker, IdentityId, IdentityId, Balance),
        /// ticker, owner DID, supply
        TokenCreated(Ticker, IdentityId, Balance),
        /// ticker, from DID, to DID, amount
        Transfer(Ticker, IdentityId, IdentityId, Balance),
    }
);

pub trait SimpleTokenTrait<V> {
    /// Tranfers tokens between two identities
    fn transfer(
        sender_did: IdentityId,
        ticker: &Ticker,
        to_did: IdentityId,
        amount: V,
    ) -> DispatchResult;
    /// Returns the balance associated with an identity and ticker
    fn balance_of(ticker: Ticker, owner_did: IdentityId) -> V;
}

impl<T: Trait> SimpleTokenTrait<T::Balance> for Module<T> {
    /// Tranfers tokens between two identities
    fn transfer(
        sender_did: IdentityId,
        ticker: &Ticker,
        to_did: IdentityId,
        amount: T::Balance,
    ) -> DispatchResult {
        Self::_transfer(ticker, sender_did, to_did, amount)
    }
    /// Returns the balance associated with an identity and ticker
    fn balance_of(ticker: Ticker, owner_did: IdentityId) -> T::Balance {
        Self::balance_of((ticker, owner_did))
    }
}

impl<T: Trait> Module<T> {
    fn _transfer(
        ticker: &Ticker,
        from_did: IdentityId,
        to_did: IdentityId,
        amount: T::Balance,
    ) -> DispatchResult {
        let ticker_from_did = (*ticker, from_did);
        ensure!(
            <BalanceOf<T>>::contains_key(&ticker_from_did),
            Error::<T>::NotAnOwner
        );
        let from_balance = Self::balance_of(&ticker_from_did);
        ensure!(from_balance >= amount, Error::<T>::InsufficientBalance);

        let new_from_balance = from_balance
            .checked_sub(&amount)
            .ok_or(Error::<T>::BalanceUnderflow)?;
        let ticker_to_did = (*ticker, to_did);
        let to_balance = Self::balance_of(&ticker_to_did);
        let new_to_balance = to_balance
            .checked_add(&amount)
            .ok_or(Error::<T>::BalanceOverflow)?;

        <BalanceOf<T>>::insert(&ticker_from_did, new_from_balance);
        <BalanceOf<T>>::insert(&ticker_to_did, new_to_balance);

        Self::deposit_event(RawEvent::Transfer(*ticker, from_did, to_did, amount));
        Ok(())
    }
}
