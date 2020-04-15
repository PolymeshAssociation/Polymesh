#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use polymesh_primitives::IdentityId;
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::{
    constants::did::TREASURY_ID,
    traits::{balances::Trait as BalancesTrait, CommonTrait},
};

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement},
};
use frame_system::{self as system, ensure_root};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

pub trait Trait: frame_system::Trait + CommonTrait + BalancesTrait {
    // The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Treasury {
        ///
        pub TreasuryAccount get(fn treasury_account) /*build(|_| Self::make_treasury_account())*/ : T::AccountId;
        ///
        pub TreasuryId get(fn treasury_id) /*build(|_| Self::make_treasury_id())*/ : IdentityId;
    }
    /*
    add_extra_genesis {
        config(treasury_balance): T::Balance;
        build( |config: &GenesisConfig<T>| {
            balances::<Module<T>>::AccountStore::insert(
                Self::make_treasury_account(),
                AccountData {
                    free: treasury_balance,
                    ..Default::default()
                });
        });
    }*/
}

decl_event!(
    pub enum Event<T> where
    <T as CommonTrait>::Balance
    {
        NoneE(Balance),
    }
);

decl_error! {
    /// Error for the treasury module.
    pub enum Error for Module<T: Trait> {
        /// Proposer's balance is too low.
        InsufficientBalance,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        pub fn disbursement(origin, target: T::AccountId, amount: T::Balance
            ) -> DispatchResult
        {
            ensure_root(origin)?;

            // Ensure treasury has enough balance.
            ensure!(
                Self::balance() >= amount,
                Error::<T>::InsufficientBalance);

            let src = Self::treasury_account();
            balances::Module::<T>::transfer_core(
                &src,
                &target,
                amount,
                None,
                ExistenceRequirement::AllowDeath)
        }

        pub fn reimbursement(origin, amount: T::Balance) -> DispatchResult {
            ensure_root(origin)?;
            let acc = Self::treasury_account();

            let _ = balances::Module::<T>::deposit_into_existing( &acc, amount)?;
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // Add public immutables and private mutables.

    /// Return the amount of money in the pot.
    // The existential deposit is not part of the pot so treasury account never gets deleted.
    pub fn balance() -> T::Balance {
        let acc = Self::treasury_account();
        balances::Module::<T>::free_balance(acc)
        // T::free_balance(acc)
    }

    fn make_treasury_account() -> T::AccountId {
        let h = T::Hashing::hash(TREASURY_ID);
        T::AccountId::decode(&mut &h.encode()[..]).unwrap_or_default()
    }

    fn make_treasury_id() -> IdentityId {
        IdentityId::from(*TREASURY_ID)
    }
}
