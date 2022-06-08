// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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

///! Checked increments of types.

/// Types which can be incremented.
pub trait CheckedInc: Sized {
    /// Increment `self`, or fail if out of bounds.
    fn checked_inc(&self) -> Option<Self>;
}

impl CheckedInc for u32 {
    fn checked_inc(&self) -> Option<Self> {
        self.checked_add(1)
    }
}

impl CheckedInc for u64 {
    fn checked_inc(&self) -> Option<Self> {
        self.checked_add(1)
    }
}

/// Implement `CheckedInc` for the passed type.
#[macro_export]
macro_rules! impl_checked_inc {
    ($typ:ty) => {
        impl $crate::checked_inc::CheckedInc for $typ {
            fn checked_inc(&self) -> Option<Self> {
                self.0.checked_inc().map(Self)
            }
        }
    };
}
