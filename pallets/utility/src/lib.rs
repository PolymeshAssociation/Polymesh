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
// https://github.com/PolymathNetwork/substrate/tree/a439a7aa5a9a3df2a42d9b25ea04288d3a0866e8/frame/utility
//
// Polymesh changes:
// - Pseudonymal dispatch has been removed.
// - Multisig dispatch has been removed.

//! # Utility Module
//! A module with helpers for dispatch management.
//!
//! ## Overview
//! This module contains the following functionality:
//! - [Batch dispatch]\: A stateless operation, allowing any origin to execute multiple calls in a
//!   single dispatch. This can be useful to amalgamate proposals, combining `set_code` with
//!   corresponding `set_storage`s, for efficient multiple payouts with just a single signature
//!   verify, or in combination with one of the other dispatch functionality.
//! - [Relayed dispatch]\: A stateful operation, allowing a signed origin to execute calls on
//!   behalf of another account. This is useful when a transaction's fee needs to be paid by a third party.
//!   Relaying dispatch requires the dispatched call to be unique as to avoid replay attacks.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `batch` - Dispatch multiple calls from the sender's origin.
//! - `relay_tx` - Relay a call for a target from an origin.
//!
//! [Batch dispatch]: ./struct.Module.html#method.batch
//! [Relayed dispatch]: ./struct.Module.html#method.relay_tx

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, PostDispatchInfo},
    ensure,
    traits::{GetCallMetadata, UnfilteredDispatchable},
    weights::{GetDispatchInfo, Weight},
    Parameter,
};
use frame_system::{ensure_root, ensure_signed, Module as System, RawOrigin};
use pallet_balances::{self as balances};
use pallet_permissions::with_call_metadata;
use polymesh_common_utilities::{
    balances::{CheckCdd, Config as BalancesConfig},
    identity::{AuthorizationNonce, Config as IdentityConfig},
    with_transaction,
};
use sp_runtime::{traits::Dispatchable, traits::Verify, DispatchError, RuntimeDebug};
use sp_std::prelude::*;

type CallPermissions<T> = pallet_permissions::Module<T>;

pub type EventCounts = Vec<u32>;
pub type ErrorAt = (u32, DispatchError);

/// Configuration trait.
pub trait Config: frame_system::Config + IdentityConfig + BalancesConfig {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    /// The overarching call type.
    type Call: Parameter
        + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
        + GetCallMetadata
        + GetDispatchInfo
        + From<frame_system::Call<Self>>
        + From<balances::Call<Self>>
        + UnfilteredDispatchable<Origin = Self::Origin>;

    type WeightInfo: WeightInfo;
}

pub trait WeightInfo {
    fn batch(calls: &[impl GetDispatchInfo]) -> Weight;
    fn batch_atomic(calls: &[impl GetDispatchInfo]) -> Weight;
    fn batch_optimistic(calls: &[impl GetDispatchInfo]) -> Weight;
    fn relay_tx(call: &impl GetDispatchInfo) -> Weight;
}

decl_storage! {
    trait Store for Module<T: Config> as Utility {
        Nonces get(fn nonce): map hasher(twox_64_concat) T::AccountId => AuthorizationNonce;
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Offchain signature is invalid
        InvalidSignature,
        /// Target does not have a valid CDD
        TargetCddMissing,
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
        /// Batch of dispatches did not complete fully.
        /// Includes a vector of event counts for each dispatch and
        /// the index of the first failing dispatch as well as the error.
        BatchInterrupted(EventCounts, ErrorAt),
        /// Batch of dispatches did not complete fully.
        /// Includes a vector of event counts for each call and
        /// a vector of any failed dispatches with their indices and associated error.
        BatchOptimisticFailed(EventCounts, Vec<ErrorAt>),
        /// Batch of dispatches completed fully with no error.
        /// Includes a vector of event counts for each dispatch.
        BatchCompleted(EventCounts),
    }
}

/// Wraps a `Call` and provides uniqueness through a nonce
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct UniqueCall<C> {
    nonce: AuthorizationNonce,
    call: Box<C>,
}

impl<C> UniqueCall<C> {
    pub fn new(nonce: AuthorizationNonce, call: C) -> Self {
        Self {
            nonce,
            call: Box::new(call),
        }
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// Deposit one of this module's events by using the default implementation.
        fn deposit_event() = default;

        /// Dispatch multiple calls from the sender's origin.
        ///
        /// This will execute until the first one fails and then stop.
        ///
        /// May be called from root or a signed origin.
        ///
        ///# Parameters
        /// - `calls`: The calls to be dispatched from the same origin.
        ///
        /// # Weight
        /// - The sum of the weights of the `calls`.
        /// - One event.
        ///
        /// This will return `Ok` in all circumstances except an unsigned origin. To determine the success of the batch, an
        /// event is deposited. If a call failed and the batch was interrupted, then the
        /// `BatchInterrupted` event is deposited, along with the number of successful calls made
        /// and the error of the failed call. If all were successful, then the `BatchCompleted`
        /// event is deposited.
        #[weight = <T as Config>::WeightInfo::batch(&calls)]
        pub fn batch(origin, calls: Vec<<T as Config>::Call>) {
            let is_root = Self::ensure_root_or_signed(origin.clone())?;

            // Run batch
            Self::deposit_event(Self::run_batch(origin.clone(), is_root, calls, true));
        }

        /// Dispatch multiple calls from the sender's origin.
        ///
        /// This will execute all calls, in order, stopping at the first failure,
        /// in which case the state changes are rolled back.
        /// On failure, an event `BatchInterrupted(failure_idx, error)` is deposited.
        ///
        /// May be called from root or a signed origin.
        ///
        ///# Parameters
        /// - `calls`: The calls to be dispatched from the same origin.
        ///
        /// # Weight
        /// - The sum of the weights of the `calls`.
        /// - One event.
        ///
        /// This will return `Ok` in all circumstances except an unsigned origin.
        /// To determine the success of the batch, an event is deposited.
        /// If any call failed, then `BatchInterrupted` is deposited.
        /// If all were successful, then the `BatchCompleted` event is deposited.
        #[weight = <T as Config>::WeightInfo::batch_atomic(&calls)]
        pub fn batch_atomic(origin, calls: Vec<<T as Config>::Call>) {
            let is_root = Self::ensure_root_or_signed(origin.clone())?;

            // Run batch inside a transaction
            Self::deposit_event(match with_transaction(|| {
                // Run batch
                match Self::run_batch(origin.clone(), is_root, calls, true) {
                    Event::BatchCompleted(counts) => Ok(Event::BatchCompleted(counts)),
                    ev => {
                        // Batch didn't complete.  Abort transaction
                        Err(ev)
                    }
                }
            }) {
                Ok(ev) => ev,
                Err(ev) => ev
            });
        }

        /// Dispatch multiple calls from the sender's origin.
        ///
        /// This will execute all calls, in order, irrespective of failures.
        /// Any failures will be available in a `BatchOptimisticFailed` event.
        ///
        /// May be called from root or a signed origin.
        ///
        ///# Parameters
        /// - `calls`: The calls to be dispatched from the same origin.
        ///
        ///
        /// # Weight
        /// - The sum of the weights of the `calls`.
        /// - One event.
        ///
        /// This will return `Ok` in all circumstances except an unsigned origin.
        /// To determine the success of the batch, an event is deposited.
        /// If any call failed, then `BatchOptimisticFailed` is deposited,
        /// with a vector of event counts for each call as well as a vector
        /// of errors.
        /// If all were successful, then the `BatchCompleted` event is deposited.
        #[weight = <T as Config>::WeightInfo::batch_optimistic(&calls)]
        pub fn batch_optimistic(origin, calls: Vec<<T as Config>::Call>) {
            let is_root = Self::ensure_root_or_signed(origin.clone())?;

            // Optimistically (hey, it's in the function name, :wink:) assume no errors.
            Self::deposit_event(Self::run_batch(origin.clone(), is_root, calls, false));
        }

        /// Relay a call for a target from an origin
        ///
        /// Relaying in this context refers to the ability of origin to make a call on behalf of
        /// target.
        ///
        /// Fees are charged to origin
        ///
        /// # Parameters
        /// - `target`: Account to be relayed
        /// - `signature`: Signature from target authorizing the relay
        /// - `call`: Call to be relayed on behalf of target
        ///
        #[weight = <T as Config>::WeightInfo::relay_tx(&*call.call)]
        pub fn relay_tx(
            origin,
            target: T::AccountId,
            signature: T::OffChainSignature,
            call: UniqueCall<<T as Config>::Call>
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            CallPermissions::<T>::ensure_call_permissions(&sender)?;

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
                T::CddChecker::check_key_cdd(&target),
                Error::<T>::TargetCddMissing
            );

            <Nonces<T>>::insert(target.clone(), target_nonce + 1);

            Self::dispatch_call(RawOrigin::Signed(target).into(), false, *call.call)
                .map(|info| info
                     .actual_weight
                     .map(|w| w.saturating_add(90_000_000))
                     .into())
                .map_err(|e| DispatchErrorWithPostInfo {
                    error: e.error,
                    post_info: e
                        .post_info
                        .actual_weight
                        .map(|w| w.saturating_add(90_000_000))
                        .into()
                })
        }
    }
}

impl<T: Config> Module<T> {
    fn dispatch_call(
        origin: T::Origin,
        is_root: bool,
        call: <T as Config>::Call,
    ) -> DispatchResultWithPostInfo {
        with_call_metadata(call.get_call_metadata(), || {
            if is_root {
                call.dispatch_bypass_filter(origin)
            } else {
                call.dispatch(origin)
            }
        })
    }

    fn run_batch(
        origin: T::Origin,
        is_root: bool,
        calls: Vec<<T as Config>::Call>,
        stop_on_errors: bool,
    ) -> Event {
        let mut counts = EventCounts::with_capacity(calls.len());
        let mut errors = Vec::new();
        for (index, call) in calls.into_iter().enumerate() {
            let count = System::<T>::event_count();

            // Dispatch the call in a modified metadata context.
            let res = Self::dispatch_call(origin.clone(), is_root, call);
            counts.push(System::<T>::event_count().saturating_sub(count));

            // Handle call error.
            if let Err(e) = res {
                let err_at = (index as u32, e.error);
                if stop_on_errors {
                    // Interrupt the batch on first error.
                    return Event::BatchInterrupted(counts, err_at);
                }
                errors.push(err_at);
            }
        }
        if errors.is_empty() {
            Event::BatchCompleted(counts)
        } else {
            Event::BatchOptimisticFailed(counts, errors)
        }
    }

    fn ensure_root_or_signed(origin: T::Origin) -> Result<bool, DispatchError> {
        let is_root = ensure_root(origin.clone()).is_ok();
        if !is_root {
            ensure_signed(origin)?;
        }
        Ok(is_root)
    }
}
