//! SimpleToken
//!
//! This module implements a simple SimpleToken API on top of Polymesh.

use crate::balances;
use crate::identity;
use crate::utils;
use codec::Encode;
use rstd::prelude::*;

use sr_primitives::traits::{CheckedAdd, CheckedSub};
use srml_support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, WithdrawReason},
    StorageMap,
};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait + utils::Trait + identity::Trait {
    // TODO: Add other types and constants required configure this module.
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// struct to store the token details
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct SimpleTokenRecord<U> {
    pub ticker: Vec<u8>,
    pub total_supply: U,
    pub owner_did: Vec<u8>,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as SimpleToken {
        // ticker, owner DID, spender DID -> allowance amount
        Allowance get(allowance): map (Vec<u8>, Vec<u8>, Vec<u8>) => T::TokenBalance;
        // ticker, DID
        pub BalanceOf get(balance_of): map (Vec<u8>, Vec<u8>) => T::TokenBalance;
        // How much creating a new SimpleToken token costs in base currency
        CreationFee get(creation_fee) config(): T::Balance;
        // Token Details
        Tokens get(tokens): map Vec<u8> => SimpleTokenRecord<T::TokenBalance>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        pub fn create_token(origin, did: Vec<u8>, ticker: Vec<u8>, total_supply: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(did.clone(), &sender.encode()), "sender must be a signing key for DID");

            ensure!(!<Tokens<T>>::exists(ticker.clone()), "Ticker with this name already exists");
            ensure!(<identity::Module<T>>::is_simple_token_issuer(did.clone()), "Sender is not an issuer");
            ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");

            <identity::DidRecords<T>>::mutate(did.clone(), |record| -> Result {
                record.balance = record.balance.checked_sub(&Self::creation_fee()).ok_or("Could not charge for token issuance")?;
                Ok(())
            })?;

            let new_token = SimpleTokenRecord {
                ticker: ticker.clone(),
                total_supply: total_supply.clone(),
                owner_did: did.clone(),
            };

            <Tokens<T>>::insert(ticker.clone(), new_token);
            // Let the owner distribute the whole supply of the token
            <BalanceOf<T>>::insert((ticker.clone(), did.clone()), total_supply);

            sr_primitives::print("Initialized a new token");

            Self::deposit_event(RawEvent::TokenCreated(ticker.clone(), did, total_supply));

            Ok(())
        }

        fn approve(origin, did: Vec<u8>, ticker: Vec<u8>, spender_did: Vec<u8>, value: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(<BalanceOf<T>>::exists((ticker.clone(), did.clone())), "Account does not own this token");

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(did.clone(), &sender.encode()), "sender must be a signing key for DID");

            let allowance = Self::allowance((ticker.clone(), did.clone(), spender_did.clone()));
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((ticker.clone(), did.clone(), spender_did.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker.clone(), did.clone(), spender_did.clone(), value));

            Ok(())
        }

        pub fn transfer(origin, did: Vec<u8>, ticker: Vec<u8>, to_did: Vec<u8>, amount: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(did.clone(), &sender.encode()), "sender must be a signing key for DID");

            Self::_transfer(ticker.clone(), did, to_did, amount)
        }

        fn transfer_from(origin, did: Vec<u8>, ticker: Vec<u8>, from_did: Vec<u8>, to_did: Vec<u8>, amount: T::TokenBalance) -> Result {
            let spender = ensure_signed(origin)?;

            // Check that spender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(did.clone(), &spender.encode()), "spender must be a signing key for DID");

            ensure!(<Allowance<T>>::exists((ticker.clone(), from_did.clone(), did.clone())), "Allowance does not exist.");
            let allowance = Self::allowance((ticker.clone(), from_did.clone(), did.clone()));
            ensure!(allowance >= amount, "Not enough allowance.");

            // Needs to happen before allowance subtraction so that the from balance is checked in _transfer
            Self::_transfer(ticker.clone(), from_did.clone(), to_did, amount)?;

            // using checked_sub (safe math) to avoid overflow
            let updated_allowance = allowance.checked_sub(&amount).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((ticker.clone(), from_did.clone(), did.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker.clone(), from_did.clone(), did.clone(), updated_allowance));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        TokenBalance = <T as utils::Trait>::TokenBalance,
    {
        // ticker, from DID, spender DID, amount
        Approval(Vec<u8>, Vec<u8>, Vec<u8>, TokenBalance),
        // ticker, owner DID, supply
        TokenCreated(Vec<u8>, Vec<u8>, TokenBalance),
        // ticker, from DID, to DID, amount
        Transfer(Vec<u8>, Vec<u8>, Vec<u8>, TokenBalance),
    }
);

pub trait SimpleTokenTrait<V> {
    fn transfer(sender_did: Vec<u8>, ticker: Vec<u8>, to_did: Vec<u8>, amount: V) -> Result;

    fn balanceOf(ticker: Vec<u8>, owner_did: Vec<u8>) -> V;
}

impl<T: Trait> SimpleTokenTrait<T::TokenBalance> for Module<T> {
    fn transfer(
        sender_did: Vec<u8>,
        ticker: Vec<u8>,
        to_did: Vec<u8>,
        amount: T::TokenBalance,
    ) -> Result {
        Self::_transfer(ticker.clone(), sender_did, to_did, amount)
    }

    fn balanceOf(ticker: Vec<u8>, owner_did: Vec<u8>) -> T::TokenBalance {
        Self::balance_of((ticker, owner_did))
    }
}

impl<T: Trait> Module<T> {
    fn _transfer(
        ticker: Vec<u8>,
        from_did: Vec<u8>,
        to_did: Vec<u8>,
        amount: T::TokenBalance,
    ) -> Result {
        ensure!(
            <BalanceOf<T>>::exists((ticker.clone(), from_did.clone())),
            "Sender doesn't own this token"
        );
        let from_balance = Self::balance_of((ticker.clone(), from_did.clone()));
        ensure!(from_balance >= amount, "Insufficient balance");

        let new_from_balance = from_balance
            .checked_sub(&amount)
            .ok_or("overflow in calculating from balance")?;
        let to_balance = Self::balance_of((ticker.clone(), to_did.clone()));
        let new_to_balance = to_balance
            .checked_add(&amount)
            .ok_or("overflow in calculating to balanc")?;

        <BalanceOf<T>>::insert((ticker.clone(), from_did.clone()), new_from_balance);
        <BalanceOf<T>>::insert((ticker.clone(), to_did.clone()), new_to_balance);

        Self::deposit_event(RawEvent::Transfer(ticker, from_did, to_did, amount));
        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    /*
     *    use super::*;
     *
     *    use primitives::{Blake2Hasher, H256};
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
     *        type Digest = Digest;
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
     *        system::GenesisConfig::<Test>::default()
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
