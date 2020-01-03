use crate::traits::identity::IdentityTrait;
use self::imbalances::{ NegativeImbalance };

use codec::Codec;

use system::{ self, Event, OnNewAccount };
use srml_support::{
    traits::{ OnFreeBalanceZero, OnUnbalanced, Get },
    Parameter
};
use runtime_primitives::{
    weights::Weight,
    traits::{ Member, SimpleArithmetic, MaybeSerializeDebug, Convert, }
 };

/// Tag a type as an instance of a module.
///
/// Defines storage prefixes, they must be unique.
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

pub trait Trait<I: Instance>: system::Trait {
    /// The balance of an account.
    type Balance: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerializeDebug
        + From<u128>
        + From<Self::BlockNumber>;

    /// This type is no longer needed but kept for compatibility reasons.
    /// A function that is invoked when the free-balance has fallen below the existential deposit and
    /// has been reduced to zero.
    ///
    /// Gives a chance to clean up resources associated with the given account.
    type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

    /// Handler for when a new account is created.
    type OnNewAccount: OnNewAccount<Self::AccountId>;

    /// Handler for the unbalanced reduction when taking transaction fees.
    type TransactionPayment: OnUnbalanced<NegativeImbalance<Self, I>>;

    /// Handler for the unbalanced reduction when taking fees associated with balance
    /// transfer (which may also include account creation).
    type TransferPayment: OnUnbalanced<NegativeImbalance<Self, I>>;

    /// Handler for the unbalanced reduction when removing a dust account.
    type DustRemoval: OnUnbalanced<NegativeImbalance<Self, I>>;

    /// The overarching event type.
    type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;

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
    type WeightToFee: Convert<Weight, Self::Balance>;

    /// Used to charge fee to identity rather than user directly
    type Identity: IdentityTrait<Self::Balance>;
}

pub trait Subtrait<I: Instance = DefaultInstance>: system::Trait {
    /// The balance of an account.
    type Balance: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerializeDebug
        + From<u128>
        + From<Self::BlockNumber>;

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
    type WeightToFee: Convert<Weight, Self::Balance>;

    /// Used to charge fee to identity rather than user directly
    type Identity: IdentityTrait<Self::Balance>;
}


// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.
mod imbalances {
    use super::{ Instance, DefaultInstance, Subtrait, Trait };

    use runtime_primitives::traits::{ Zero };
    use srml_support::traits::Imbalance;
    use rstd::{ mem, result };

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been created without any equal and opposite accounting.
    #[must_use]
    pub struct PositiveImbalance<T: Subtrait<I>, I: Instance = DefaultInstance>(T::Balance);

    impl<T: Subtrait<I>, I: Instance> PositiveImbalance<T, I> {
        /// Create a new positive imbalance from a balance.
        pub fn new(amount: T::Balance) -> Self {
            PositiveImbalance(amount)
        }
    }

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been destroyed without any equal and opposite accounting.
    #[must_use]
    pub struct NegativeImbalance<T: Subtrait<I>, I: Instance = DefaultInstance>(T::Balance);

    impl<T: Subtrait<I>, I: Instance> NegativeImbalance<T, I> {
        /// Create a new negative imbalance from a balance.
        pub fn new(amount: T::Balance) -> Self {
            NegativeImbalance(amount)
        }
    }

    impl<T: Trait<I>, I: Instance> Imbalance<T::Balance> for PositiveImbalance<T, I> {
        type Opposite = NegativeImbalance<T, I>;

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

    impl<T: Trait<I>, I: Instance> Imbalance<T::Balance> for NegativeImbalance<T, I> {
        type Opposite = PositiveImbalance<T, I>;

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

    impl<T: Subtrait<I>, I: Instance> Drop for PositiveImbalance<T, I> {
        /// Basic drop handler will just square up the total issuance.
        fn drop(&mut self) {
            <super::TotalIssuance<super::ElevatedTrait<T, I>, I>>::mutate(|v| {
                *v = v.saturating_add(self.0)
            });
        }
    }

    impl<T: Subtrait<I>, I: Instance> Drop for NegativeImbalance<T, I> {
        /// Basic drop handler will just square up the total issuance.
        fn drop(&mut self) {
            <super::TotalIssuance<super::ElevatedTrait<T, I>, I>>::mutate(|v| {
                *v = v.saturating_sub(self.0)
            });
        }
    }
}
