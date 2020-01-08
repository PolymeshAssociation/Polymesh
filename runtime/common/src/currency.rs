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
            T::Balance: MaybeSerializeDebug,
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
                reason: WithdrawReason,
                new_balance: T::Balance,
            ) -> srml_support::dispatch::Result {
                match reason {
                    WithdrawReason::Reserve | WithdrawReason::Transfer
                        if Self::currency_vesting_balance(who) > new_balance =>
                    {
                        return Err("vesting balance too high to send value")
                    }
                    _ => {}
                }
                let locks = Self::currency_locks(who);
                if locks.is_empty() {
                    return Ok(());
                }

                let now = <system::Module<T>>::block_number();
                if locks.into_iter().all(|l| {
                    now >= l.until || new_balance >= l.amount || !l.reasons.contains(reason)
                }) {
                    Ok(())
                } else {
                    Err("account liquidity restrictions prevent withdrawal")
                }
            }

            fn transfer(
                transactor: &T::AccountId,
                dest: &T::AccountId,
                value: Self::Balance,
            ) -> srml_support::dispatch::Result {
                /*
                let from_balance = Self::free_balance(transactor);
                let to_balance = Self::free_balance(dest);
                let would_create = to_balance.is_zero();
                let fee = if would_create {
                    T::CreationFee::get()
                } else {
                    T::TransferFee::get()
                };
                let liability = match value.checked_add(&fee) {
                    Some(l) => l,
                    None => return Err("got overflow after adding a fee to value"),
                };

                let new_from_balance = match from_balance.checked_sub(&liability) {
                    None => return Err("balance too low to send value"),
                    Some(b) => b,
                };

                Self::ensure_can_withdraw(
                    transactor,
                    value,
                    WithdrawReason::Transfer,
                    new_from_balance,
                )?;

                // NOTE: total stake being stored in the same type means that this could never overflow
                // but better to be safe than sorry.
                let new_to_balance = match to_balance.checked_add(&value) {
                    Some(b) => b,
                    None => return Err("destination balance too high to receive value"),
                };

                if transactor != dest {
                    Self::set_free_balance(transactor, new_from_balance);
                    // if !<FreeBalance<T, I>>::exists(dest) {
                    if ! T::free_balance_exists(dest) {
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
                reason: WithdrawReason,
                _liveness: ExistenceRequirement,
            ) -> rstd::result::Result<Self::NegativeImbalance, &'static str> {
                if let Some(new_balance) = Self::free_balance(who).checked_sub(&value) {
                    Self::ensure_can_withdraw(who, value, reason, new_balance)?;
                    Self::set_free_balance(who, new_balance);
                    Ok(NegativeImbalance::new(value))
                } else {
                    Err("too few free funds in account")
                }
            }

            fn slash(
                who: &T::AccountId,
                value: Self::Balance,
            ) -> (Self::NegativeImbalance, Self::Balance) {
                let free_balance = Self::free_balance(who);
                let free_slash = rstd::cmp::min(free_balance, value);
                Self::set_free_balance(who, free_balance - free_slash);
                let remaining_slash = value - free_slash;
                // NOTE: `slash()` prefers free balance, but assumes that reserve balance can be drawn
                // from in extreme circumstances. `can_slash()` should be used prior to `slash()` to avoid having
                // to draw from reserved funds, however we err on the side of punishment if things are inconsistent
                // or `can_slash` wasn't used appropriately.
                if !remaining_slash.is_zero() {
                    let reserved_balance = Self::currency_reserved_balance(who);
                    let reserved_slash = rstd::cmp::min(reserved_balance, remaining_slash);
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
            ) -> rstd::result::Result<Self::PositiveImbalance, &'static str> {
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
