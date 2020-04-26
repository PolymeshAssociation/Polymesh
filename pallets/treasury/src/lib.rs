#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use pallet_balances as balances;
use pallet_identity as identity;
use polymesh_common_utilities::{
    traits::{
        balances::Trait as BalancesTrait, identity::Trait as IdentityTrait, CommonTrait,
        NegativeImbalance, PositiveImbalance,
    },
    Context,
};
use polymesh_primitives::{AccountKey, Beneficiary, IdentityId};

use codec::Encode;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement, Imbalance, OnUnbalanced, WithdrawReason},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::traits::Saturating;
use sp_std::{convert::TryFrom, prelude::*};

pub type ProposalIndex = u32;

type Identity<T> = identity::Module<T>;

pub trait Trait: frame_system::Trait + CommonTrait + BalancesTrait + IdentityTrait {
    // The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

pub trait TreasuryTrait<Balance> {
    fn disbursement(target: IdentityId, amount: Balance);
    fn balance() -> Balance;
}

decl_storage! {
    trait Store for Module<T: Trait> as Treasury {
        pub Balance get(fn balance) config(): T::Balance;
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

        /// It transfers balances from treasury to each of beneficiaries and the specific amount
        /// for each of them.
        ///
        /// # Error
        /// * `BadOrigin`: Only root can execute transaction.
        /// * `InsufficientBalance`: If treasury balances is not enough to cover all beneficiaries.
        pub fn disbursement(origin, beneficiaries: Vec< Beneficiary<T::Balance>>) -> DispatchResult
        {
            ensure_root(origin)?;

            // Ensure treasury has enough balance.
            let total_amount = beneficiaries.iter().fold( 0.into(), |acc,b| b.amount.saturating_add(acc));
            ensure!(
                Self::balance() >= total_amount,
                Error::<T>::InsufficientBalance);

            beneficiaries.into_iter().for_each( |b| {
                Self::unsafe_disbursement(b.id, b.amount);
                Self::deposit_event(RawEvent::TreasuryDisbursement(b.id, b.amount));
            });
            Ok(())
        }

        /// It transfers the specific `amount` from `origin` account into treasury.
        ///
        /// Only accounts which are associated to an identity can make a donation to treasury.
        pub fn reimbursement(origin, amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let _did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            let _ = balances::Module::<T>::withdraw(
                &sender,
                amount,
                WithdrawReason::Transfer.into(),
                ExistenceRequirement::AllowDeath,
            )?;

            Self::unsafe_reimbursement(amount);
            Self::deposit_event(RawEvent::TreasuryReimbursement(amount));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn unsafe_disbursement(target: IdentityId, amount: T::Balance) {
        // Update treasury and total issuance.
        let new_treasury_balance = Self::balance() - amount;
        <Balance<T>>::put(new_treasury_balance);

        // Top up target identity balance.
        balances::Module::<T>::unsafe_top_up_identity_balance(&target, amount);
    }

    fn unsafe_reimbursement(amount: T::Balance) {
        // Update treasury balance.
        let old_balance = Self::balance();
        let new_balance = old_balance.saturating_add(amount);
        debug::info!(
            "Treasury reimbursement from {:?} to {:?}",
            old_balance,
            new_balance
        );
        <Balance<T>>::put(new_balance);

        // Update total issuance when that positive imbalance is dropped.
        let _ = PositiveImbalance::<T>::new(amount);
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
    /// It is called when fees are sent to treasury.
    fn on_nonzero_unbalanced(amount: NegativeImbalance<T>) {
        let abs_amount = amount.peek();
        Self::unsafe_reimbursement(abs_amount);
    }
}
