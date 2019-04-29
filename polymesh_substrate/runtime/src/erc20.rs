//! ERC20
//!
//! This module implements a simple ERC20 API on top of Polymesh.
use crate::asset;
use crate::identity;
use crate::utils;

use rstd::prelude::*;

use runtime_primitives::traits::{CheckedAdd, CheckedSub};
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, OnUnbalanced, WithdrawReason},
    StorageMap, StorageValue,
};
use system::ensure_signed;

type FeeOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait + utils::Trait + identity::Trait {
    // TODO: Add other types and constants required configure this module.
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type TokenFeeCharge: OnUnbalanced<NegativeImbalanceOf<Self>>;
}

// struct to store the token details
#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct ERC20Token<U, V> {
    ticker: Vec<u8>,
    total_supply: U,
    pub owner: V,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ERC20 {
        // Just a dummy storage item.
        // Here we are declaring a StorageValue, `Something` as a Option<u32>
        // `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
        //
        // balances mapping for an account and its tokens
        Tokens get(tokens): map Vec<u8> => ERC20Token<T::TokenBalance, T::AccountId>;
        BalanceOf get(balance_of): map (Vec<u8>, T::AccountId) => T::TokenBalance;
        // How much creating a new ERC20 token costs in base currency
        CreationFee get(creation_fee) config(): FeeOf<T>;
    }
}

// The module's dispatchable functions.
decl_module! {
/// The module declaration.
pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event<T>() = default;

    fn create_token(origin, ticker: Vec<u8>, total_supply: T::TokenBalance) -> Result {
        let sender = ensure_signed(origin)?;
        ensure!(!<Tokens<T>>::exists(ticker.clone()), "Ticker with this name already exists");
        ensure!(<identity::Module<T>>::is_issuer(sender.clone()), "Sender is not an issuer");

        // Charge the creation fee
        let imbalance = T::Currency::withdraw(
            &sender,
            Self::creation_fee(),
            WithdrawReason::Fee,
            ExistenceRequirement::KeepAlive
            )?;
        T::TokenFeeCharge::on_unbalanced(imbalance);

        ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");

        let new_token = ERC20Token {
            ticker: ticker.clone(),
            total_supply: total_supply.clone(),
            owner: sender.clone(),
        };

        <Tokens<T>>::insert(ticker.clone(), new_token);
        // Let the owner to distribute the whole supply of the token
        <BalanceOf<T>>::insert((ticker.clone(), sender.clone()), total_supply);

        runtime_io::print("Initialized a new token");

        Self::deposit_event(RawEvent::TokenCreated(ticker.clone(), sender, total_supply));

        Ok(())
    }

    fn transfer(origin, ticker: Vec<u8>, to: T::AccountId, amount: T::TokenBalance) -> Result {
        let sender = ensure_signed(origin)?;

        Self::_transfer(ticker.clone(), sender, to, amount)
    }
}
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        TokenBalance = <T as utils::Trait>::TokenBalance,
    {
        // Balance lookup
        Balance(Vec<u8>, AccountId, TokenBalance),
        TokenCreated(Vec<u8>, AccountId, TokenBalance),
        Transfer(Vec<u8>, AccountId, AccountId, TokenBalance),
    }
);

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
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl Trait for Test {
        type Event = ();
    }
    type ERC20 = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            assert_ok!(ERC20::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            assert_eq!(ERC20::something(), Some(42));
        });
    }
}
