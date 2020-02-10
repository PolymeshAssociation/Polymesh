use crate::traits::{BlockRewardsReserveTrait, CommonTrait};

use frame_support::traits::{Imbalance, TryDrop};
use sp_arithmetic::traits::{Saturating, Zero};
use sp_std::{mem, result};

// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.

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

    fn offset(self, other: NegativeImbalance<T>) -> result::Result<Self, NegativeImbalance<T>> {
        let (a, b) = (self.0, other.0);
        mem::forget((self, other));

        if a >= b {
            Ok(Self(a - b))
        } else {
            Err(NegativeImbalance::new(b - a))
        }
    }

    fn peek(&self) -> T::Balance {
        self.0
    }
}

impl<T: CommonTrait> Drop for PositiveImbalance<T> {
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {
        T::BlockRewardsReserve::drop_positive_imbalance(self.0);
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

    fn offset(self, other: PositiveImbalance<T>) -> result::Result<Self, PositiveImbalance<T>> {
        let (a, b) = (self.0, other.0);
        mem::forget((self, other));

        if a >= b {
            Ok(Self(a - b))
        } else {
            Err(PositiveImbalance::new(b - a))
        }
    }

    fn peek(&self) -> T::Balance {
        self.0
    }
}

impl<T: CommonTrait> Drop for NegativeImbalance<T> {
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {
        T::BlockRewardsReserve::drop_negative_imbalance(self.0);
    }
}
