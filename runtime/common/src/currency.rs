use crate::traits::{BalanceLock, CommonTrait};

pub trait CurrencyModule<T>
where
    T: CommonTrait,
{
    fn currency_reserved_balance(who: &T::AccountId) -> T::Balance;
    fn set_reserved_balance(who: &T::AccountId, amount: T::Balance);
    fn currency_total_issuance() -> T::Balance;
    fn currency_free_balance(who: &T::AccountId) -> T::Balance;
    fn set_free_balance(who: &T::AccountId, amount: T::Balance) -> T::Balance;
    fn currency_burn(amount: T::Balance);
    fn currency_issue(amount: T::Balance);
    fn currency_vesting_balance(who: &T::AccountId) -> T::Balance;
    fn currency_locks(who: &T::AccountId) -> Vec<BalanceLock<T::Balance, T::BlockNumber>>;
    fn new_account(who: &T::AccountId, amount: T::Balance);

    fn free_balance_exists(who: &T::AccountId) -> bool;
}

#[macro_export]
macro_rules! impl_currency {
    () => {
        // impl<T: Trait, I: Instance> Currency<T::AccountId> for Module<T, I>
        impl<T: Trait> Currency<T::AccountId> for Module<T>
        where
            T::Balance: MaybeSerializeDeserialize + Debug,
            Module<T>: CurrencyModule<T>,
        {
            type Balance = T::Balance;
            type PositiveImbalance = PositiveImbalance<T>;
            type NegativeImbalance = NegativeImbalance<T>;

            fn total_balance(who: &T::AccountId) -> Self::Balance {
                Self::free_balance(who) + Self::currency_reserved_balance(who)
            }

            fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
                Self::free_balance(who) >= value
            }

            fn total_issuance() -> Self::Balance {
                // <TotalIssuance<T, I>>::get()
                Self::currency_total_issuance()
            }

            fn minimum_balance() -> Self::Balance {
                0u128.into()
            }

            fn free_balance(who: &T::AccountId) -> Self::Balance {
                // <FreeBalance<T, I>>::get(who)
                Self::currency_free_balance(who)
            }

            fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
                /*
                <TotalIssuance<T, I>>::mutate(|issued| {
                    *issued = issued.checked_sub(&amount).unwrap_or_else(|| {
                        amount = *issued;
                        Zero::zero()
                    });
                });*/
                Self::currency_burn(amount);
                PositiveImbalance::new(amount)
            }

            fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
                /*<TotalIssuance<T, I>>::mutate(|issued| {
                    *issued = issued.checked_add(&amount).unwrap_or_else(|| {
                        amount = Self::Balance::max_value() - *issued;
                        Self::Balance::max_value()
                    })
                });*/
                Self::currency_issue(amount);
                NegativeImbalance::new(amount)
            }

            // # <weight>
            // Despite iterating over a list of locks, they are limited by the number of
            // lock IDs, which means the number of runtime modules that intend to use and create locks.
            // # </weight>
            fn ensure_can_withdraw(
                who: &T::AccountId,
                _amount: T::Balance,
                reasons: WithdrawReasons,
                new_balance: T::Balance,
            ) -> DispatchResult {
                if reasons.intersects(WithdrawReason::Reserve | WithdrawReason::Transfer)
                    && Self::vesting_balance(who) > new_balance
                {
                    Err(Error::<T>::VestingBalance)?
                }
                let locks = Self::locks(who);
                if locks.is_empty() {
                    return Ok(());
                }

                let now = <frame_system::Module<T>>::block_number();
                if locks.into_iter().all(|l| {
                    now >= l.until || new_balance >= l.amount || !l.reasons.intersects(reasons)
                }) {
                    Ok(())
                } else {
                    Err(Error::<T>::LiquidityRestrictions.into())
                }
            }

            fn transfer(
                _transactor: &T::AccountId,
                _dest: &T::AccountId,
                _value: Self::Balance,
                _existence_requirement: ExistenceRequirement,
            ) -> DispatchResult {
                /*
                let from_balance = Self::free_balance(transactor);
                let to_balance = Self::free_balance(dest);
                let would_create = to_balance.is_zero();
                let fee = if would_create {
                    T::CreationFee::get()
                } else {
                    T::TransferFee::get()
                };
                let liability = value.checked_add(&fee).ok_or(Error::<T, I>::Overflow)?;
                let new_from_balance = from_balance
                    .checked_sub(&liability)
                    .ok_or(Error::<T, I>::InsufficientBalance)?;

                Self::ensure_can_withdraw(
                    transactor,
                    value,
                    WithdrawReason::Transfer.into(),
                    new_from_balance,
                )?;

                // NOTE: total stake being stored in the same type means that this could never overflow
                // but better to be safe than sorry.
                let new_to_balance = to_balance
                    .checked_add(&value)
                    .ok_or(Error::<T, I>::Overflow)?;

                if transactor != dest {
                    Self::set_free_balance(transactor, new_from_balance);
                    if !<FreeBalance<T, I>>::exists(dest) {
                        Self::new_account(dest, new_to_balance);
                    }

                    Self::set_free_balance(dest, new_to_balance);
                    T::TransferPayment::on_unbalanced(NegativeImbalance::new(fee));

                    Self::deposit_event(RawEvent::Transfer(
                            transactor.clone(),
                            dest.clone(),
                            value,
                            fee,
                    ));
                }*/

                Ok(())
            }

            fn withdraw(
                who: &T::AccountId,
                value: Self::Balance,
                reasons: WithdrawReasons,
                _liveness: ExistenceRequirement,
            ) -> result::Result<Self::NegativeImbalance, DispatchError> {
                if let Some(new_balance) = Self::free_balance(who).checked_sub(&value) {
                    Self::ensure_can_withdraw(who, value, reasons, new_balance)?;
                    Self::set_free_balance(who, new_balance);
                    Ok(NegativeImbalance::new(value))
                } else {
                    Err(Error::<T>::InsufficientBalance)?
                }
            }

            fn slash(
                who: &T::AccountId,
                value: Self::Balance,
            ) -> (Self::NegativeImbalance, Self::Balance) {
                let free_balance = Self::free_balance(who);
                let free_slash = sp_std::cmp::min(free_balance, value);
                Self::set_free_balance(who, free_balance - free_slash);
                let remaining_slash = value - free_slash;
                // NOTE: `slash()` prefers free balance, but assumes that reserve balance can be drawn
                // from in extreme circumstances. `can_slash()` should be used prior to `slash()` to avoid having
                // to draw from reserved funds, however we err on the side of punishment if things are inconsistent
                // or `can_slash` wasn't used appropriately.
                if !remaining_slash.is_zero() {
                    let reserved_balance = Self::currency_reserved_balance(who);
                    let reserved_slash = sp_std::cmp::min(reserved_balance, remaining_slash);
                    Self::set_reserved_balance(who, reserved_balance - reserved_slash);
                    (
                        NegativeImbalance::new(free_slash + reserved_slash),
                        remaining_slash - reserved_slash,
                    )
                } else {
                    (NegativeImbalance::new(value), Zero::zero())
                }
            }

            fn deposit_into_existing(
                who: &T::AccountId,
                value: Self::Balance,
            ) -> sp_std::result::Result<Self::PositiveImbalance, &'static str> {
                if Self::total_balance(who).is_zero() {
                    return Err("beneficiary account must pre-exist");
                }
                Self::set_free_balance(who, Self::free_balance(who) + value);
                Ok(PositiveImbalance::new(value))
            }

            fn deposit_creating(
                who: &T::AccountId,
                value: Self::Balance,
            ) -> Self::PositiveImbalance {
                let (imbalance, _) =
                    Self::make_free_balance_be(who, Self::free_balance(who) + value);
                if let SignedImbalance::Positive(p) = imbalance {
                    p
                } else {
                    // Impossible, but be defensive.
                    Self::PositiveImbalance::zero()
                }
            }

            fn make_free_balance_be(
                who: &T::AccountId,
                balance: Self::Balance,
            ) -> (
                SignedImbalance<Self::Balance, Self::PositiveImbalance>,
                UpdateBalanceOutcome,
            ) {
                let original = Self::free_balance(who);
                let imbalance = if original <= balance {
                    SignedImbalance::Positive(PositiveImbalance::new(balance - original))
                } else {
                    SignedImbalance::Negative(NegativeImbalance::new(original - balance))
                };
                // if !<FreeBalance<T, I>>::exists(who) {
                if !Self::free_balance_exists(who) {
                    Self::new_account(&who, balance);
                }
                Self::set_free_balance(who, balance);
                (imbalance, UpdateBalanceOutcome::Updated)
            }
        }
    };
}
