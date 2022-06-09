// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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

use core::mem;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::{Get, StorageInfo, StorageInfoTrait};
use frame_support::{decl_error, decl_module, ensure};
pub use polymesh_common_utilities::traits::base::{Config, Event};
use polymesh_primitives::checked_inc::CheckedInc;
use sp_std::vec::Vec;

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;
        const MaxLen: u32 = T::MaxLen::get();
    }
}

/// Emit an unexpected error event that should be investigated manually
pub fn emit_unexpected_error<T: Config>(error: Option<DispatchError>) {
    Module::<T>::deposit_event(Event::UnexpectedError(error));
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Exceeded a generic length limit.
        /// The limit could be for any sort of lists of things, including a string.
        TooLong,
        /// The sequence counter for something overflowed.
        ///
        /// When this happens depends on e.g., the capacity of the identifier type.
        /// For example, we might have `pub struct PipId(u32);`, with `u32::MAX` capacity.
        /// In practice, these errors will never happen but no code path should result in a panic,
        /// so these corner cases need to be covered with an error variant.
        CounterOverflow,
    }
}

/// Ensure that the `len` provided is within the generic length limit.
pub fn ensure_length_ok<T: Config>(len: usize) -> DispatchResult {
    ensure!(len <= T::MaxLen::get() as usize, Error::<T>::TooLong);
    Ok(())
}

/// Ensure that `s.len()` is within the generic length limit.
pub fn ensure_string_limited<T: Config>(s: &[u8]) -> DispatchResult {
    ensure_length_ok::<T>(s.len())
}

/// Ensure that the `len` provided is within the custom length limit.
pub fn ensure_custom_length_ok<T: Config>(len: usize, limit: usize) -> DispatchResult {
    ensure!(len <= limit, Error::<T>::TooLong);
    Ok(())
}

/// Ensure that `s.len()` is within the custom length limit.
pub fn ensure_custom_string_limited<T: Config>(s: &[u8], limit: usize) -> DispatchResult {
    ensure_custom_length_ok::<T>(s.len(), limit)
}

/// Ensure that given `Some(s)`, `s.len()` is within the generic length limit.
pub fn ensure_opt_string_limited<T: Config>(s: Option<&[u8]>) -> DispatchResult {
    match s {
        None => Ok(()),
        Some(s) => ensure_string_limited::<T>(s),
    }
}

/// Try to pre-increment the counter `seq` and return the next number/ID to use.
pub fn try_next_pre<T: Config, I: CheckedInc + Clone>(seq: &mut I) -> Result<I, DispatchError> {
    let id = seq.checked_inc().ok_or(Error::<T>::CounterOverflow)?;
    *seq = id.clone();
    Ok(id)
}

/// Try to post-increment the counter `seq` and return the next number/ID to use.
pub fn try_next_post<T: Config, I: CheckedInc>(seq: &mut I) -> Result<I, DispatchError> {
    seq.checked_inc()
        .map(|x| mem::replace(seq, x))
        .ok_or_else(|| Error::<T>::CounterOverflow.into())
}

impl<T: Config> frame_support::traits::IntegrityTest for Module<T> {}

impl<T: Config> StorageInfoTrait for Module<T> {
    fn storage_info() -> Vec<StorageInfo> {
        Vec::new()
    }
}
