// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2021 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! # Base Module
//!
//! The Base module provides utility and framework-like functionality atop of substrate.
//!
//! Currently, the module contains:
//! - `Error::TooLong` and `ensure_*` functions to conveniently check data types for length.
//! - `Event::UnexpectedError` and `emit_unexpected_error` to embed an error for manual inspection.
//!
//! There are currently no extrinsics or storage items in the base module.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::Get;
use frame_support::{decl_error, decl_module, decl_storage, ensure};

pub use polymesh_common_utilities::traits::base::{Event, Trait};

decl_storage! {
    trait Store for Module<T: Trait> as base {
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;
        const MaxLen: u32 = T::MaxLen::get();
    }
}

/// Emit an unexpected error event that should be investigated manually
pub fn emit_unexpected_error<T: Trait>(error: Option<DispatchError>) {
    Module::<T>::deposit_event(Event::UnexpectedError(error));
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Exceeded a generic length limit.
        /// The limit could be for any sort of lists of things, including a string.
        TooLong,
    }
}

/// Ensure that the `len` provided is within the generic length limit.
pub fn ensure_length_ok<T: Trait>(len: usize) -> DispatchResult {
    ensure!(len <= T::MaxLen::get() as usize, Error::<T>::TooLong);
    Ok(())
}

/// Ensure that `s.len()` is within the generic length limit.
pub fn ensure_string_limited<T: Trait>(s: &[u8]) -> DispatchResult {
    ensure_length_ok::<T>(s.len())
}

/// Ensure that given `Some(s)`, `s.len()` is within the generic length limit.
pub fn ensure_opt_string_limited<T: Trait>(s: Option<&[u8]>) -> DispatchResult {
    match s {
        None => Ok(()),
        Some(s) => ensure_string_limited::<T>(s),
    }
}

impl<T: Trait> frame_support::traits::IntegrityTest for Module<T> {}
