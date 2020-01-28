use self::imbalances::NegativeImbalance;
use crate::traits::{identity::IdentityTrait, CommonTrait};

use frame_support::{
    decl_event,
    traits::{Get, OnFreeBalanceZero, OnUnbalanced},
};
use frame_system::{self as system, OnNewAccount};

/// Tag a type as an instance of a module.
///
/// Defines storage prefixes, they must be unique.
#[allow(non_upper_case_globals)]
pub trait Instance: 'static {
    /// The prefix used by any storage entry of an instance.
    const PREFIX: &'static str;
    const PREFIX_FOR_TotalIssuance: &'static str;
    const PREFIX_FOR_Vesting: &'static str;
    const PREFIX_FOR_FreeBalance: &'static str;
    const PREFIX_FOR_ReservedBalance: &'static str;
    const PREFIX_FOR_Locks: &'static str;
    const PREFIX_FOR_IdentityBalance: &'static str;
    const PREFIX_FOR_ChargeDid: &'static str;
}

pub struct DefaultInstance;

#[allow(non_upper_case_globals)]
impl Instance for DefaultInstance {
    const PREFIX: &'static str = "Balances";
    const PREFIX_FOR_TotalIssuance: &'static str = "Balances TotalIssuance";
    const PREFIX_FOR_Vesting: &'static str = "Balances Vesting";
    const PREFIX_FOR_FreeBalance: &'static str = "Balances FreeBalance";
    const PREFIX_FOR_ReservedBalance: &'static str = "Balances ReservedBalance";
    const PREFIX_FOR_Locks: &'static str = "Balances Locks";
    const PREFIX_FOR_IdentityBalance: &'static str = "Balances IdentityBalance";
    const PREFIX_FOR_ChargeDid: &'static str = "Balances ChargeDid";
}

pub trait Subtrait<I: Instance = DefaultInstance>: CommonTrait {
    /// This type is no longer needed but kept for compatibility reasons.
    /// A function that is invoked when the free-balance has fallen below the existential deposit and
    /// has been reduced to zero.
    ///
    /// Gives a chance to clean up resources associated with the given account.
    type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

    /// Handler for when a new account is created.
    type OnNewAccount: OnNewAccount<Self::AccountId>;

    /// This type is no longer needed but kept for compatibility reasons.
    /// The minimum amount required to keep an account open.
    type ExistentialDeposit: Get<Self::Balance>;

    /// The fee required to make a transfer.
    type TransferFee: Get<Self::Balance>;

    /// The fee required to create an account.
    type CreationFee: Get<Self::Balance>;

    /// The fee to be paid for making a transaction; the base.
    type TransactionBaseFee: Get<Self::Balance>;

    /// The fee to be paid for making a transaction; the per-byte portion.
    type TransactionByteFee: Get<Self::Balance>;

    /// Convert a weight value into a deductible fee based on the currency type.
    // type WeightToFee: Convert<Weight, Self::Balance>;

    /// Used to charge fee to identity rather than user directly
    type Identity: IdentityTrait<Self::Balance>;
}

decl_event!(
    pub enum Event<T, I: Instance = DefaultInstance> where
    <T as system::Trait>::AccountId,
    <T as CommonTrait>::Balance
    {
        /// A new account was created.
        NewAccount(AccountId, Balance),
        /// An account was reaped.
        ReapedAccount(AccountId),
        /// Transfer succeeded (from, to, value, fees).
        Transfer(AccountId, AccountId, Balance, Balance),
    }
);

pub trait Trait<I: Instance = DefaultInstance>: Subtrait<I> {
    /// Handler for the unbalanced reduction when taking transaction fees.
    type TransactionPayment: OnUnbalanced<NegativeImbalance<Self>>;

    /// Handler for the unbalanced reduction when taking fees associated with balance
    /// transfer (which may also include account creation).
    type TransferPayment: OnUnbalanced<NegativeImbalance<Self>>;

    /// Handler for the unbalanced reduction when removing a dust account.
    type DustRemoval: OnUnbalanced<NegativeImbalance<Self>>;

    // / The overarching event type.
    type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;
}

// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.
pub mod imbalances {
    use crate::traits::CommonTrait;

    use frame_support::traits::{Imbalance, TryDrop};
    use sp_runtime::traits::{Saturating, Zero};
    use sp_std::{mem, result};

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been created without any equal and opposite accounting.
    #[must_use]
    pub struct PositiveImbalance<T: CommonTrait>(T::Balance);

    impl<T: CommonTrait> PositiveImbalance<T> {
        /// Create a new positive imbalance from a balance.
        pub fn new(amount: T::Balance) -> Self {
            PositiveImbalance(amount)
        }
    }

    impl<T: CommonTrait> TryDrop for PositiveImbalance<T> {
        fn try_drop(self) -> result::Result<(), Self> {
            self.drop_zero()
        }
    }

    impl<T: CommonTrait> Imbalance<T::Balance> for PositiveImbalance<T> {
        type Opposite = NegativeImbalance<T>;

        fn zero() -> Self {
            Self(Zero::zero())
        }
        fn drop_zero(self) -> result::Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }
        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;

            mem::forget(self);
            (Self(first), Self(second))
        }
        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);

            self
        }
        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
        }
        fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            mem::forget((self, other));

            if a >= b {
                Ok(Self(a - b))
            } else {
                Err(NegativeImbalance::new(b - a))
            }
        }
        fn peek(&self) -> T::Balance {
            self.0.clone()
        }
    }

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been destroyed without any equal and opposite accounting.
    #[must_use]
    pub struct NegativeImbalance<T: CommonTrait>(T::Balance);

    impl<T: CommonTrait> NegativeImbalance<T> {
        /// Create a new negative imbalance from a balance.
        pub fn new(amount: T::Balance) -> Self {
            NegativeImbalance(amount)
        }
    }

    impl<T: CommonTrait> TryDrop for NegativeImbalance<T> {
        fn try_drop(self) -> result::Result<(), Self> {
            self.drop_zero()
        }
    }

    impl<T: CommonTrait> Imbalance<T::Balance> for NegativeImbalance<T> {
        type Opposite = PositiveImbalance<T>;

        fn zero() -> Self {
            Self(Zero::zero())
        }
        fn drop_zero(self) -> result::Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }
        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;

            mem::forget(self);
            (Self(first), Self(second))
        }
        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);

            self
        }
        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
        }
        fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            mem::forget((self, other));

            if a >= b {
                Ok(Self(a - b))
            } else {
                Err(PositiveImbalance::new(b - a))
            }
        }
        fn peek(&self) -> T::Balance {
            self.0.clone()
        }
    }
}
