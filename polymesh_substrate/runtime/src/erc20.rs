//! ERC20
//!
//! This module implements a simple ERC20 API on top of Polymesh.

use crate::balances;
use crate::identity;
use crate::utils;

use rstd::prelude::*;

use runtime_primitives::traits::{CheckedAdd, CheckedSub};
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, WithdrawReason},
    StorageMap,
};
use system::ensure_signed;

type FeeOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait + utils::Trait + identity::Trait {
    // TODO: Add other types and constants required configure this module.
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
}

// struct to store the token details
#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct ERC20Token<U, V> {
    pub ticker: Vec<u8>,
    pub total_supply: U,
    pub owner: V,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ERC20 {
        // How much Alice (first address) allows Bob (second address) to spend from her account
        Allowance get(allowance): map (Vec<u8>, T::AccountId, T::AccountId) => T::TokenBalance;
        pub BalanceOf get(balance_of): map (Vec<u8>, T::AccountId) => T::TokenBalance;
        // How much creating a new ERC20 token costs in base currency
        CreationFee get(creation_fee) config(): FeeOf<T>;
        // Token Details
        Tokens get(tokens): map Vec<u8> => ERC20Token<T::TokenBalance, T::AccountId>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event<T>() = default;

        pub fn create_token(origin, ticker: Vec<u8>, total_supply: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(!<Tokens<T>>::exists(ticker.clone()), "Ticker with this name already exists");
            ensure!(<identity::Module<T>>::is_erc20_issuer(sender.clone()), "Sender is not an issuer");
            ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");


            // Charge the creation fee
            let _imbalance = T::Currency::withdraw(
                &sender,
                Self::creation_fee(),
                WithdrawReason::Fee,
                ExistenceRequirement::KeepAlive
                )?;

            let new_token = ERC20Token {
                ticker: ticker.clone(),
                total_supply: total_supply.clone(),
                owner: sender.clone(),
            };

            <Tokens<T>>::insert(ticker.clone(), new_token);
            // Let the owner distribute the whole supply of the token
            <BalanceOf<T>>::insert((ticker.clone(), sender.clone()), total_supply);

            runtime_io::print("Initialized a new token");

            Self::deposit_event(RawEvent::TokenCreated(ticker.clone(), sender, total_supply));

            Ok(())
        }

        fn approve(origin, ticker: Vec<u8>, spender: T::AccountId, value: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(<BalanceOf<T>>::exists((ticker.clone(), sender.clone())), "Account does not own this token");

            let allowance = Self::allowance((ticker.clone(), sender.clone(), spender.clone()));
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((ticker.clone(), sender.clone(), spender.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker.clone(), sender.clone(), spender.clone(), value));

            Ok(())
        }

        pub fn transfer(origin, ticker: Vec<u8>, to: T::AccountId, amount: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;

            Self::_transfer(ticker.clone(), sender, to, amount)
        }

        fn transfer_from(origin, ticker: Vec<u8>, from: T::AccountId, to: T::AccountId, amount: T::TokenBalance) -> Result {
            let spender = ensure_signed(origin)?;
            ensure!(<Allowance<T>>::exists((ticker.clone(), from.clone(), spender.clone())), "Allowance does not exist.");
            let allowance = Self::allowance((ticker.clone(), from.clone(), spender.clone()));
            ensure!(allowance >= amount, "Not enough allowance.");

            // Needs to happen before allowance subtraction so that the from balance is checked in _transfer
            Self::_transfer(ticker.clone(), from.clone(), to, amount)?;

            // using checked_sub (safe math) to avoid overflow
            let updated_allowance = allowance.checked_sub(&amount).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((ticker.clone(), from.clone(), spender.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker.clone(), from.clone(), spender.clone(), updated_allowance));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        TokenBalance = <T as utils::Trait>::TokenBalance,
    {
        // ticker, from, spender, amount
        Approval(Vec<u8>, AccountId, AccountId, TokenBalance),
        // ticker, owner, supply
        TokenCreated(Vec<u8>, AccountId, TokenBalance),
        // ticker, from, to, amount
        Transfer(Vec<u8>, AccountId, AccountId, TokenBalance),
    }
);

pub trait ERC20Trait<T, V> {
    //pub fn approve(sender: T, ticker: Vec<u8>, spender: T, value: V) -> Result ;

    fn transfer(sender: T, ticker: Vec<u8>, to: T, amount: V) -> Result;

    fn balanceOf(ticker: Vec<u8>, owner: T) -> V;

    //pub fn transfer_from(sender: T, ticker: Vec<u8>, from: T, to: T, amount: V) -> Result;
}

impl<T: Trait> ERC20Trait<T::AccountId, T::TokenBalance> for Module<T> {
    fn transfer(
        sender: T::AccountId,
        ticker: Vec<u8>,
        to: T::AccountId,
        amount: T::TokenBalance,
    ) -> Result {
        Self::_transfer(ticker.clone(), sender, to, amount)
    }

    fn balanceOf(ticker: Vec<u8>, owner: T::AccountId) -> T::TokenBalance {
        Self::balance_of((ticker, owner))
    }
}

impl<T: Trait> Module<T> {
    fn _transfer(
        ticker: Vec<u8>,
        from: T::AccountId,
        to: T::AccountId,
        amount: T::TokenBalance,
    ) -> Result {
        ensure!(
            <BalanceOf<T>>::exists((ticker.clone(), from.clone())),
            "Sender doesn't own this token"
        );
        let from_balance = Self::balance_of((ticker.clone(), from.clone()));
        ensure!(from_balance >= amount, "Insufficient balance");

        let new_from_balance = from_balance
            .checked_sub(&amount)
            .ok_or("overflow in calculating from balance")?;
        let to_balance = Self::balance_of((ticker.clone(), to.clone()));
        let new_to_balance = to_balance
            .checked_add(&amount)
            .ok_or("overflow in calculating to balanc")?;

        <BalanceOf<T>>::insert((ticker.clone(), from.clone()), new_from_balance);
        <BalanceOf<T>>::insert((ticker.clone(), to.clone()), new_to_balance);

        Self::deposit_event(RawEvent::Transfer(ticker, from, to, amount));
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
     *    use runtime_io::with_externalities;
     *    use runtime_primitives::{
     *        testing::{Digest, DigestItem, Header},
     *        traits::{BlakeTwo256, IdentityLookup},
     *        BuildStorage,
     *    };
     *    use support::{assert_ok, impl_outer_origin};
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
     *    type ERC20 = Module<Test>;
     *
     *    // This function basically just builds a genesis storage key/value store according to
     *    // our desired mockup.
     *    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
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
     *            assert_ok!(ERC20::do_something(Origin::signed(1), 42));
     *            // asserting that the stored value is equal to what we stored
     *            assert_eq!(ERC20::something(), Some(42));
     *        });
     *    }
     */
}
