use core::marker::PhantomData;
use frame_support::traits::fungible::{Balanced, Credit, Debt, Inspect};
use frame_support::traits::tokens::{Precision, WithdrawConsequence};
use frame_support::traits::{Imbalance, OnUnbalanced};
use frame_support::unsigned::TransactionValidityError;
use sp_runtime::traits::{DispatchInfoOf, PostDispatchInfoOf, Saturating, Zero};
use sp_runtime::transaction_validity::InvalidTransaction;

use crate::pallet::Config;

/// Handle withdrawing, refunding and depositing of transaction fees.
pub trait OnChargeTransaction<T: Config> {
    /// The underlying integer type in which fees are calculated.
    type Balance: frame_support::traits::tokens::Balance;

    type LiquidityInfo: Default;

    /// Before the transaction is executed the payment of the transaction fees
    /// need to be secured.
    ///
    /// Note: The `fee` already includes the `tip`.
    fn withdraw_fee(
        who: &T::AccountId,
        call: &T::RuntimeCall,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        fee: Self::Balance,
        tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError>;

    /// Check if the predicted fee from the transaction origin can be withdrawn.
    ///
    /// Note: The `fee` already includes the `tip`.
    fn can_withdraw_fee(
        who: &T::AccountId,
        call: &T::RuntimeCall,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        fee: Self::Balance,
        tip: Self::Balance,
    ) -> Result<(), TransactionValidityError>;

    /// After the transaction was executed the actual fee can be calculated.
    /// This function should refund any overpaid fees and optionally deposit
    /// the corrected amount.
    ///
    /// Note: The `fee` already includes the `tip`.
    fn correct_and_deposit_fee(
        who: &T::AccountId,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        corrected_fee: Self::Balance,
        tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError>;

    #[cfg(feature = "runtime-benchmarks")]
    fn endow_account(who: &T::AccountId, amount: Self::Balance);

    #[cfg(feature = "runtime-benchmarks")]
    fn minimum_balance() -> Self::Balance;

    // Polymesh change
    // -----------------------------------------------------------------
    fn charge_fee(who: &T::AccountId, fee: Self::Balance) -> Result<(), TransactionValidityError>;
    // -----------------------------------------------------------------
}

/// Implements transaction payment for a pallet implementing the [`frame_support::traits::fungible`]
/// trait (eg. pallet_balances) using an unbalance handler (implementing
/// [`OnUnbalanced`]).
///
/// The unbalance handler is given 2 unbalanceds in [`OnUnbalanced::on_unbalanceds`]: `fee` and
/// then `tip`.
pub struct FungibleAdapter<F, OU>(PhantomData<(F, OU)>);

impl<T, F, OU> OnChargeTransaction<T> for FungibleAdapter<F, OU>
where
    T: Config,
    F: Balanced<T::AccountId>,
    OU: OnUnbalanced<Credit<T::AccountId, F>>,
{
    type LiquidityInfo = Option<Credit<T::AccountId, F>>;
    type Balance = <F as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    fn withdraw_fee(
        who: &<T>::AccountId,
        _call: &<T>::RuntimeCall,
        _dispatch_info: &DispatchInfoOf<<T>::RuntimeCall>,
        fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        if fee.is_zero() {
            return Ok(None);
        }

        match F::withdraw(
            who,
            fee,
            Precision::Exact,
            frame_support::traits::tokens::Preservation::Preserve,
            frame_support::traits::tokens::Fortitude::Polite,
        ) {
            Ok(imbalance) => Ok(Some(imbalance)),
            Err(_) => Err(InvalidTransaction::Payment.into()),
        }
    }

    fn can_withdraw_fee(
        who: &T::AccountId,
        _call: &T::RuntimeCall,
        _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<(), TransactionValidityError> {
        if fee.is_zero() {
            return Ok(());
        }

        match F::can_withdraw(who, fee) {
            WithdrawConsequence::Success => Ok(()),
            _ => Err(InvalidTransaction::Payment.into()),
        }
    }

    fn correct_and_deposit_fee(
        who: &<T>::AccountId,
        _dispatch_info: &DispatchInfoOf<<T>::RuntimeCall>,
        _post_info: &PostDispatchInfoOf<<T>::RuntimeCall>,
        corrected_fee: Self::Balance,
        tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        if let Some(paid) = already_withdrawn {
            // Calculate how much refund we should return
            let refund_amount = paid.peek().saturating_sub(corrected_fee);
            // Refund to the the account that paid the fees if it exists & refund is non-zero.
            // Otherwise, don't refund anything.
            let refund_imbalance =
                if refund_amount > Zero::zero() && F::total_balance(who) > F::Balance::zero() {
                    F::deposit(who, refund_amount, Precision::BestEffort)
                        .unwrap_or_else(|_| Debt::<T::AccountId, F>::zero())
                } else {
                    Debt::<T::AccountId, F>::zero()
                };
            // merge the imbalance caused by paying the fees and refunding parts of it again.
            let adjusted_paid: Credit<T::AccountId, F> = paid
                .offset(refund_imbalance)
                .same()
                .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
            // Call someone else to handle the imbalance (fee and tip separately)
            let (tip, fee) = adjusted_paid.split(tip);
            OU::on_unbalanceds(Some(fee).into_iter().chain(Some(tip)));
        }

        Ok(())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn endow_account(who: &T::AccountId, amount: Self::Balance) {
        let _ = F::deposit(who, amount, Precision::BestEffort);
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn minimum_balance() -> Self::Balance {
        F::minimum_balance()
    }

    // Polymesh change
    // -----------------------------------------------------------------
    fn charge_fee(
        _who: &T::AccountId,
        _fee: Self::Balance,
    ) -> Result<(), TransactionValidityError> {
        unimplemented!("");
    }
    // -----------------------------------------------------------------
}
