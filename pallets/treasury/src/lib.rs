#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use polymesh_primitives::IdentityId;
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::{
    balances::Trait as BalancesTrait, CommonTrait, NegativeImbalance,
};

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Imbalance, OnUnbalanced},
};
use frame_system::{self as system, ensure_root};
use sp_runtime::traits::Saturating;
use sp_std::prelude::*;

pub type ProposalIndex = u32;

type BalanceOf<T> = <T as CommonTrait>::Balance;

pub trait Trait: frame_system::Trait + CommonTrait + BalancesTrait {
    // The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

pub trait TreasuryTrait<Balance> {
    fn disbursement(target: IdentityId, amount: Balance);
    fn balance() -> Balance;
}

decl_storage! {
    trait Store for Module<T: Trait> as Treasury {
        pub Balance get(fn balance) config(): BalanceOf<T>;
    }
}

decl_event!(
    pub enum Event<T> where
    <T as CommonTrait>::Balance
    {
        /// Disbursement to a target Identity.
        /// (target identity, amount)
        TreasuryDisbursement(IdentityId, Balance),

        /// Treasury reimbursement.
        TreasuryReimbursement(Balance),
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

        pub fn disbursement(origin, target: IdentityId, amount: T::Balance
            ) -> DispatchResult
        {
            ensure_root(origin)?;

            // Ensure treasury has enough balance.
            ensure!(
                Self::balance() >= amount,
                Error::<T>::InsufficientBalance);

            Self::unsafe_disbursement(target, amount);
            Self::deposit_event(RawEvent::TreasuryDisbursement(target, amount));
            Ok(())
        }

        pub fn reimbursement(origin, amount: T::Balance) -> DispatchResult {
            ensure_root(origin)?;

            Self::unsafe_reimbursement(amount);
            Self::deposit_event(RawEvent::TreasuryReimbursement(amount));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn unsafe_disbursement(target: IdentityId, amount: T::Balance) {
        let new_treasury_balance = Self::balance() - amount;
        <Balance<T>>::put(new_treasury_balance);

        balances::Module::<T>::unsafe_top_up_identity_balance(&target, amount);
    }

    fn unsafe_reimbursement(amount: T::Balance) {
        let new_balance = Self::balance().saturating_add(amount);
        <Balance<T>>::put(new_balance);
    }
}

impl<T: Trait> TreasuryTrait<T::Balance> for Module<T> {
    #[inline]
    fn disbursement(target: IdentityId, amount: T::Balance) {
        Self::unsafe_disbursement(target, amount);
    }

    #[inline]
    fn balance() -> T::Balance {
        Self::balance()
    }
}

impl<T: Trait> OnUnbalanced<NegativeImbalance<T>> for Module<T> {
    fn on_nonzero_unbalanced(amount: NegativeImbalance<T>) {
        let abs_amount = amount.peek();
        Self::unsafe_reimbursement(abs_amount);
    }
}
