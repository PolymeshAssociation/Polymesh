// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::traits::CommonConfig;
use core::marker::PhantomData;
use frame_support::traits::{Imbalance, TryDrop};
use polymesh_primitives::traits::BlockRewardsReserveCurrency;
use polymesh_primitives::Balance;
use sp_arithmetic::traits::Zero;
use sp_std::{mem, result};

// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.

/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been created without any equal and opposite accounting.
#[must_use]
pub struct PositiveImbalance<T: CommonConfig>(Balance, PhantomData<T>);

impl<T: CommonConfig> PositiveImbalance<T> {
    /// Create a new positive imbalance from a balance.
    pub fn new(amount: Balance) -> Self {
        PositiveImbalance(amount, PhantomData)
    }
}

impl<T: CommonConfig> TryDrop for PositiveImbalance<T> {
    fn try_drop(self) -> result::Result<(), Self> {
        self.drop_zero()
    }
}

impl<T: CommonConfig> Imbalance<Balance> for PositiveImbalance<T> {
    type Opposite = NegativeImbalance<T>;

    fn zero() -> Self {
        Self::new(Zero::zero())
    }

    fn drop_zero(self) -> result::Result<(), Self> {
        if self.0.is_zero() {
            Ok(())
        } else {
            Err(self)
        }
    }

    fn split(self, amount: Balance) -> (Self, Self) {
        let first = self.0.min(amount);
        let second = self.0 - first;

        mem::forget(self);
        (Self::new(first), Self::new(second))
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
            Ok(Self::new(a - b))
        } else {
            Err(NegativeImbalance::new(b - a))
        }
    }

    fn peek(&self) -> Balance {
        self.0
    }
}

impl<T: CommonConfig> Drop for PositiveImbalance<T> {
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {
        T::BlockRewardsReserve::drop_positive_imbalance(self.0);
    }
}

/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been destroyed without any equal and opposite accounting.
#[must_use]
pub struct NegativeImbalance<T: CommonConfig>(Balance, PhantomData<T>);

impl<T: CommonConfig> NegativeImbalance<T> {
    /// Create a new negative imbalance from a balance.
    pub fn new(amount: Balance) -> Self {
        NegativeImbalance(amount, PhantomData)
    }
}

impl<T: CommonConfig> TryDrop for NegativeImbalance<T> {
    fn try_drop(self) -> result::Result<(), Self> {
        self.drop_zero()
    }
}

impl<T: CommonConfig> Imbalance<Balance> for NegativeImbalance<T> {
    type Opposite = PositiveImbalance<T>;

    fn zero() -> Self {
        Self::new(Zero::zero())
    }

    fn drop_zero(self) -> result::Result<(), Self> {
        if self.0.is_zero() {
            Ok(())
        } else {
            Err(self)
        }
    }

    fn split(self, amount: Balance) -> (Self, Self) {
        let first = self.0.min(amount);
        let second = self.0 - first;

        mem::forget(self);
        (Self::new(first), Self::new(second))
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
            Ok(Self::new(a - b))
        } else {
            Err(PositiveImbalance::new(b - a))
        }
    }

    fn peek(&self) -> Balance {
        self.0
    }
}

impl<T: CommonConfig> Drop for NegativeImbalance<T> {
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {
        T::BlockRewardsReserve::drop_negative_imbalance(self.0);
    }
}
