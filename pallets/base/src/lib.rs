// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2021 Polymesh Association

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
//!
//! There are currently no extrinsics or storage items in the base module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::mem;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::ensure;
use frame_support::traits::Get;
use polymesh_primitives::checked_inc::CheckedInc;
use polymesh_primitives::AccountId as AccountId32;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The maximum length governing `TooLong`.
        ///
        /// How lengths are computed to compare against this value is situation based.
        /// For example, you could halve it, double it, compute a sum for some tree of strings, etc.
        #[pallet::constant]
        type MaxLen: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::event]
    pub enum Event {
        /// An unexpected error happened that should be investigated.
        /// TODO: Unused, remove it.
        UnexpectedError(Option<DispatchError>),
    }

    #[pallet::error]
    pub enum Error<T> {
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
        /// Invalid account ID.
        InvalidAccountId,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
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

/// Returns account ID of type `T::AccountId` from an `AccountId32`.
pub fn pallet_account_id<T: Config>(acc_id: &AccountId32) -> Result<T::AccountId, DispatchError> {
    let account_id =
        Decode::decode(&mut &acc_id.encode()[..]).map_err(|_| Error::<T>::InvalidAccountId)?;
    Ok(account_id)
}
