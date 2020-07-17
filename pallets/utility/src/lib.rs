// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.
//
// Modified by Polymath Inc - 2020
// This module is inspired from the `pallet-utility`.
// https://github.com/paritytech/substrate/tree/a439a7aa5a9a3df2a42d9b25ea04288d3a0866e8/frame/utility
//
// Polymesh changes:
// - Pseudonymal dispatch has been removed.
// - Multisig dispatch has been removed.

//! # Utility Module
//! A module with helpers for dispatch management.
//!
//! - [`utility::Trait`](./trait.Trait.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//!
//! This module contains three basic pieces of functionality, two of which are stateless:
//! - Batch dispatch: A stateless operation, allowing any origin to execute multiple calls in a
//!   single dispatch. This can be useful to amalgamate proposals, combining `set_code` with
//!   corresponding `set_storage`s, for efficient multiple payouts with just a single signature
//!   verify, or in combination with one of the other two dispatch functionality.
//! - Pseudonymal dispatch: A stateless operation, allowing a signed origin to execute a call from
//!   an alternative signed origin. Each account has 2**16 possible "pseudonyms" (alternative
//!   account IDs) and these can be stacked. This can be useful as a key management tool, where you
//!   need multiple distinct accounts (e.g. as controllers for many staking accounts), but where
//!   it's perfectly fine to have each of them controlled by the same underlying keypair.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For batch dispatch
//! * `batch` - Dispatch multiple calls from the sender's origin.
//!
//! [`Call`]: ./enum.Call.html
//! [`Trait`]: ./trait.Trait.html

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::{convert::TryFrom};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::PostDispatchInfo,
    traits::UnfilteredDispatchable,
    weights::{DispatchClass, GetDispatchInfo, Weight},
    Parameter,
};
use frame_system as system;
use frame_system::{ensure_signed, RawOrigin};
use polymesh_common_utilities::{
    balances::CheckCdd, identity::AuthorizationNonce, identity::Trait as IdentityModule,
};
use polymesh_primitives::AccountKey;
use sp_runtime::{
    traits::Dispatchable, traits::Verify, DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::prelude::*;

/// Configuration trait.
pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    /// The overarching call type.
    type Call: Parameter
        + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
        + GetDispatchInfo
        + From<frame_system::Call<Self>>
        + UnfilteredDispatchable<Origin = Self::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Utility { }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Offchain signature is invalid
        InvalidSignature,
        /// Target does not have a valid CDD
        TargetCddMissing,
        /// Origin does not have a valid CDD
        OriginCddMissing,
        /// Provided nonce was invalid
        /// If the provided nonce < current nonce, the call was already executed
        /// If the provided nonce > current nonce, the call(s) before the current failed to execute
        InvalidNonce,
    }
}

decl_event! {
    /// Events type.
    pub enum Event
    {
        /// Batch of dispatches did not complete fully. Index of first failing dispatch given, as
        /// well as the error.
        BatchInterrupted(u32, DispatchError),
        /// Batch of dispatches completed fully with no error.
        BatchCompleted,
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct UniqueCall<C> {
    nonce: AuthorizationNonce,
    inner: C,
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// Deposit one of this module's events by using the default implementation.
        fn deposit_event() = default;

        /// Send a batch of dispatch calls.
        ///
        /// This will execute until the first one fails and then stop.
        ///
        /// May be called from any origin.
        ///
        /// - `calls`: The calls to be dispatched from the same origin.
        ///
        /// # <weight>
        /// - The sum of the weights of the `calls`.
        /// - One event.
        /// # </weight>
        ///
        /// This will return `Ok` in all circumstances. To determine the success of the batch, an
        /// event is deposited. If a call failed and the batch was interrupted, then the
        /// `BatchInterrupted` event is deposited, along with the number of successful calls made
        /// and the error of the failed call. If all were successful, then the `BatchCompleted`
        /// event is deposited.
        #[weight = (
            calls.iter()
                .map(|call| call.get_dispatch_info().weight)
                .fold(10_000, |a: Weight, n| a.saturating_add(n)),
            {
                let all_operational = calls.iter()
                    .map(|call| call.get_dispatch_info().class)
                    .all(|class| class == DispatchClass::Operational);
                if all_operational {
                    DispatchClass::Operational
                } else {
                    DispatchClass::Normal
                }
            },
        )]
        pub fn batch(origin, calls: Vec<<T as Trait>::Call>) -> DispatchResult {
            let is_root = ensure_root(origin.clone()).is_ok();
            for (index, call) in calls.into_iter().enumerate() {
                let result = if is_root {
                    call.dispatch_bypass_filter(origin.clone())
                } else {
                    call.dispatch(origin.clone())
                };
                if let Err(e) = result {
                    Self::deposit_event(Event::BatchInterrupted(index as u32, e.error));
                    return Ok(());
                }
            }
            Self::deposit_event(Event::BatchCompleted);

            Ok(())
        }

        /// TODO docs
        #[weight = (
            call.inner.get_dispatch_info().weight.saturating_add(250_000),
            call.inner.get_dispatch_info().class,
        )]
        pub fn relay_tx(
            origin,
            target: T::AccountId,
            signature: <T as IdentityModule>::OffChainSignature,
            call: UniqueCall<<T as Trait>::Call>
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;

           let origin_key = AccountKey::try_from(origin.encode())?;
           let target_key = AccountKey::try_from(target.encode())?;

           let target_nonce = <Nonces<T>>::get(&target);

            ensure!(
                target_nonce == call.nonce,
                Error::<T>::InvalidNonce
            );

            ensure!(
                signature.verify(call.encode().as_slice(), &target),
                Error::<T>::InvalidSignature
            );

            ensure!(
                T::CddChecker::check_key_cdd(&origin_key),
                Error::<T>::OriginCddMissing
            );

            ensure!(
                T::CddChecker::check_key_cdd(&target_key),
                Error::<T>::TargetCddMissing
            );

            <Nonces<T>>::insert(target.clone(), target_nonce + 1);

            call.inner.dispatch(RawOrigin::Signed(target).into())
        }
    }
}
