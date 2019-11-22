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

use crate::{balances, identity, utils};
use primitives::{IdentityId, Key};

use codec::Encode;
use rstd::{convert::TryFrom, prelude::*};

use crate::constants::currency::MAX_SUPPLY;
use sr_primitives::traits::{CheckedAdd, CheckedSub};
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait + utils::Trait + identity::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// Struct to store the details of each simple token
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct SimpleTokenRecord<U> {
    pub ticker: Vec<u8>,
    pub total_supply: U,
    pub owner_did: IdentityId,
}

decl_storage! {
    trait Store for Module<T: Trait> as SimpleToken {
        /// Mapping from (ticker, owner DID, spender DID) to allowance amount
        Allowance get(allowance): map (Vec<u8>, IdentityId, IdentityId) => T::Balance;
        /// Mapping from (ticker, owner DID) to their balance
        pub BalanceOf get(balance_of): map (Vec<u8>, IdentityId) => T::Balance;
        /// The cost to create a new simple token
        CreationFee get(creation_fee) config(): T::Balance;
        /// The details associated with each simple token
        Tokens get(tokens): map Vec<u8> => SimpleTokenRecord<T::Balance>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new token and mint a balance to the issuing identity
        pub fn create_token(origin, did: IdentityId, ticker: Vec<u8>, total_supply: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            ensure!(!<Tokens<T>>::exists(&ticker), "Ticker with this name already exists");
            ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");
            ensure!(total_supply <= MAX_SUPPLY.into(), "Total supply above the limit");

            // TODO Charge proper fee
            // <identity::DidRecords<T>>::mutate( did, |record| -> Result {
            //     record.balance = record.balance.checked_sub(&Self::creation_fee()).ok_or("Could not charge for token issuance")?;
            //     Ok(())
            // })?;

            let new_token = SimpleTokenRecord {
                ticker: ticker.clone(),
                total_supply: total_supply.clone(),
                owner_did: did.clone(),
            };

            <Tokens<T>>::insert(&ticker, new_token);
            // Let the owner distribute the whole supply of the token
            <BalanceOf<T>>::insert((ticker.clone(), did.clone()), total_supply);

            sr_primitives::print("Initialized a new token");

            Self::deposit_event(RawEvent::TokenCreated(ticker, did, total_supply));

            Ok(())
        }

        /// Approve another identity to transfer tokens on behalf of the caller
        fn approve(origin, did: IdentityId, ticker: Vec<u8>, spender_did: IdentityId, value: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker_did = (ticker.clone(), did.clone());
            ensure!(<BalanceOf<T>>::exists(&ticker_did), "Account does not own this token");

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let ticker_did_spender_did = (ticker.clone(), did, spender_did);
            let allowance = Self::allowance(&ticker_did_spender_did);
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert(&ticker_did_spender_did, updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker, did, spender_did, value));

            Ok(())
        }

        /// Transfer tokens to another identity
        pub fn transfer(origin, did: IdentityId, ticker: Vec<u8>, to_did: IdentityId, amount: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            Self::_transfer(&ticker, did, to_did, amount)
        }

        /// Transfer tokens to another identity using the approval mechanic
        fn transfer_from(origin, did: IdentityId, ticker: Vec<u8>, from_did: IdentityId, to_did: IdentityId, amount: T::Balance) -> Result {
            let spender = ensure_signed(origin)?;

            // Check that spender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(spender.encode())?), "spender must be a signing key for DID");

            let ticker_from_did_did = (ticker.clone(), from_did, did);
            ensure!(<Allowance<T>>::exists(&ticker_from_did_did), "Allowance does not exist.");
            let allowance = Self::allowance(&ticker_from_did_did);
            ensure!(allowance >= amount, "Not enough allowance.");

            // Needs to happen before allowance subtraction so that the from balance is checked in _transfer
            Self::_transfer(&ticker, from_did, to_did, amount)?;

            // using checked_sub (safe math) to avoid overflow
            let updated_allowance = allowance.checked_sub(&amount).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((ticker.clone(), from_did.clone(), did.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker, from_did, did, updated_allowance));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as balances::Trait>::Balance,
    {
        /// ticker, from DID, spender DID, amount
        Approval(Vec<u8>, IdentityId, IdentityId, Balance),
        /// ticker, owner DID, supply
        TokenCreated(Vec<u8>, IdentityId, Balance),
        /// ticker, from DID, to DID, amount
        Transfer(Vec<u8>, IdentityId, IdentityId, Balance),
    }
);

pub trait SimpleTokenTrait<V> {
    /// Tranfers tokens between two identities
    fn transfer(sender_did: IdentityId, ticker: &Vec<u8>, to_did: IdentityId, amount: V) -> Result;
    /// Returns the balance associated with an identity and ticker
    fn balance_of(ticker: Vec<u8>, owner_did: IdentityId) -> V;
}

impl<T: Trait> SimpleTokenTrait<T::Balance> for Module<T> {
    /// Tranfers tokens between two identities
    fn transfer(
        sender_did: IdentityId,
        ticker: &Vec<u8>,
        to_did: IdentityId,
        amount: T::Balance,
    ) -> Result {
        Self::_transfer(ticker, sender_did, to_did, amount)
    }
    /// Returns the balance associated with an identity and ticker
    fn balance_of(ticker: Vec<u8>, owner_did: IdentityId) -> T::Balance {
        Self::balance_of((ticker, owner_did))
    }
}

impl<T: Trait> Module<T> {
    fn _transfer(
        ticker: &Vec<u8>,
        from_did: IdentityId,
        to_did: IdentityId,
        amount: T::Balance,
    ) -> Result {
        let ticker_from_did = (ticker.clone(), from_did.clone());
        ensure!(
            <BalanceOf<T>>::exists(&ticker_from_did),
            "Sender doesn't own this token"
        );
        let from_balance = Self::balance_of(&ticker_from_did);
        ensure!(from_balance >= amount, "Insufficient balance");

        let new_from_balance = from_balance
            .checked_sub(&amount)
            .ok_or("overflow in calculating from balance")?;
        let ticker_to_did = (ticker.clone(), to_did.clone());
        let to_balance = Self::balance_of(&ticker_to_did);
        let new_to_balance = to_balance
            .checked_add(&amount)
            .ok_or("overflow in calculating to balanc")?;

        <BalanceOf<T>>::insert(&ticker_from_did, new_from_balance);
        <BalanceOf<T>>::insert(&ticker_to_did, new_to_balance);

        Self::deposit_event(RawEvent::Transfer(ticker.clone(), from_did, to_did, amount));
        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    /*
     *    use super::*;
     *
     *    use substrate_primitives::{Blake2Hasher, H256};
     *    use sr_io::with_externalities;
     *    use sr_primitives::{
     *        testing::{Digest, DigestItem, Header},
     *        traits::{BlakeTwo256, IdentityLookup},
     *        BuildStorage,
     *    };
     *    use srml_support::{assert_ok, impl_outer_origin};
     *
     *    impl_outer_origin! {
     *        pub enum Origin for Test {}
     *    }
     *
     *    // For testing the module, we construct most of a mock runtime. This means
     *    // first constructing a configuration type (`Test`) which `impl`s each of the
     *    // configuration traits of modules we want to use.
     *    #[derive(Clone, Eq, PartialEq)]
     *    pub struct Test;
     *    impl system::Trait for Test {
     *        type Origin = Origin;
     *        type Index = u64;
     *        type BlockNumber = u64;
     *        type Hash = H256;
     *        type Hashing = BlakeTwo256;
     *        type Digest = H256;
     *        type AccountId = u64;
     *        type Lookup = IdentityLookup<Self::AccountId>;
     *        type Header = Header;
     *        type Event = ();
     *        type Log = DigestItem;
     *    }
     *    impl Trait for Test {
     *        type Event = ();
     *    }
     *    type SimpleToken = Module<Test>;
     *
     *    // This function basically just builds a genesis storage key/value store according to
     *    // our desired mockup.
     *    fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
     *        system::GenesisConfig::default()
     *            .build_storage()
     *            .unwrap()
     *            .0
     *            .into()
     *    }
     *
     *    #[test]
     *    fn it_works_for_default_value() {
     *        with_externalities(&mut new_test_ext(), || {
     *            // Just a dummy test for the dummy funtion `do_something`
     *            // calling the `do_something` function with a value 42
     *            assert_ok!(SimpleToken::do_something(Origin::signed(1), 42));
     *            // asserting that the stored value is equal to what we stored
     *            assert_eq!(SimpleToken::something(), Some(42));
     *        });
     *    }
     */
}
