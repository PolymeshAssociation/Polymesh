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

//! # Simple Relayer Module
//!
//! TODO: Add description.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::{
    debug, decl_error, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;

pub use polymesh_common_utilities::traits::relayer::{Event, Trait, WeightInfo};

decl_storage! {
    trait Store for Module<T: Trait> as Relayer {
        /// map user key to paying key
        pub PayingKeys get(fn paying_keys):
            map hasher(blake2_128_concat) T::AccountId => T::AccountId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Set paying key for `userKey`
        #[weight = <T as Trait>::WeightInfo::set_paying_key()]
        pub fn set_paying_key(origin, user_key: T::AccountId) -> DispatchResult {
            Self::base_set_paying_key(origin, user_key)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The `userKey` already has a `payingKey`
        AlreadyHasPayingKey,
    }
}

impl<T: Trait> Module<T> {
    fn base_set_paying_key(origin: T::Origin, user_key: T::AccountId) -> DispatchResult {
        debug::info!("set_paying_key(): user_key={:?}", user_key);
        let sender = ensure_signed(origin)?;

        ensure!(
            <PayingKeys<T>>::contains_key(&user_key),
            Error::<T>::AlreadyHasPayingKey
        );

        // TODO: generate Nonce for `auth_id` and store auth data.

        // TODO: move this to `accept_paying_key`
        <PayingKeys<T>>::insert(sender, user_key);

        Ok(())
    }
}
