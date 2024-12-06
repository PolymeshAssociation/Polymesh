// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Modified by Polymesh Association, original from:
// https://github.com/PolymeshAssociation/polkadot-sdk/substrate/frame/utility
//
// Polymesh changes:
// - Add permissions checks.

//! # Utility Pallet
//! A stateless pallet with helpers for dispatch management which does no re-authentication.
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! This pallet contains two basic pieces of functionality:
//! - Batch dispatch: A stateless operation, allowing any origin to execute multiple calls in a
//!   single dispatch. This can be useful to amalgamate proposals, combining `set_code` with
//!   corresponding `set_storage`s, for efficient multiple payouts with just a single signature
//!   verify, or in combination with one of the other two dispatch functionality.
//! - Pseudonymal dispatch: A stateless operation, allowing a signed origin to execute a call from
//!   an alternative signed origin. Each account has 2 * 2**16 possible "pseudonyms" (alternative
//!   account IDs) and these can be stacked. This can be useful as a key management tool, where you
//!   need multiple distinct accounts (e.g. as controllers for many staking accounts), but where
//!   it's perfectly fine to have each of them controlled by the same underlying keypair. Derivative
//!   accounts are, for the purposes of proxy filtering considered exactly the same as the origin
//!   and are thus hampered with the origin's filters.
//!
//! Since proxy filters are respected in all dispatches of this pallet, it should never need to be
//! filtered by any proxy.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For batch dispatch
//! * `batch` - Dispatch multiple calls from the sender's origin, stopping on the first error.
//! * `batch_all` - Send a batch of dispatch calls and atomically execute them.
//!   The whole transaction will rollback and fail if any of the calls failed.
//! * `force_batch` - Send a batch of dispatch calls. Unlike `batch`, it allows errors and
//!   won't interrupt.
//! * `dispatch_as` - Dispatches a function call with a provided origin.
//! * `with_weight` - Dispatch a function call with a specified weight.
//!
//! ## POLYMESH
//! * Removed `as_derivative`.
//! * Added `relay_tx`.
//! * Added as deprecated: `batch_old`, `batch_atomic`, `batch_optimistic`.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchClass;
use frame_support::dispatch::{extract_actual_weight, GetDispatchInfo, PostDispatchInfo};
use frame_support::dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, Weight};
use frame_support::ensure;
use frame_support::traits::GetCallMetadata;
use frame_support::traits::{IsSubType, OriginTrait, UnfilteredDispatchable};
use frame_system::{ensure_root, ensure_signed, RawOrigin};
use scale_info::TypeInfo;
use sp_core::Get;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::TrailingZeroInput;
use sp_runtime::traits::{BadOrigin, Dispatchable};
use sp_runtime::{traits::Verify, DispatchError, RuntimeDebug};
use sp_std::prelude::*;

use pallet_permissions::with_call_metadata;
use polymesh_common_utilities::balances::{CheckCdd, Config as BalancesConfig};
use polymesh_common_utilities::identity::{AuthorizationNonce, Config as IdentityConfig};
use polymesh_common_utilities::Context;
use polymesh_primitives::IdentityId;

type Identity<T> = pallet_identity::Module<T>;

pub trait WeightInfo {
    fn batch(c: u32) -> Weight;
    fn batch_all(c: u32) -> Weight;
    fn dispatch_as() -> Weight;
    fn force_batch(c: u32) -> Weight;

    // POLYMESH:
    fn ensure_root() -> Weight;
    fn relay_tx() -> Weight;
    fn as_derivative() -> Weight;
}

// POLYMESH:
pub const MIN_WEIGHT: Weight = Weight::from_ref_time(1_000_000);

// POLYMESH: Used for permission checks.
type CallPermissions<T> = pallet_permissions::Module<T>;

/// POLYMESH: type for our events.
pub type EventCounts = Vec<u32>;
/// POLYMESH: type for our events.
pub type ErrorAt = (u32, DispatchError);

/// Wraps a `Call` and provides uniqueness through a nonce
/// POLYMESH: used for `relay_tx`
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
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

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    ///
    /// POLYMESH: Add `IdentityConfig` and `BalancesConfig`.
    #[pallet::config]
    pub trait Config: frame_system::Config + IdentityConfig + BalancesConfig {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
            + GetCallMetadata
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// The caller origin, overarching type of all pallets origins.
        type PalletsOrigin: Parameter +
            Into<<Self as frame_system::Config>::RuntimeOrigin> +
            IsType<<<Self as frame_system::Config>::RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Batch of dispatches did not complete fully. Index of first failing dispatch given, as
        /// well as the error.
        BatchInterrupted { index: u32, error: DispatchError },
        /// Batch of dispatches completed fully with no error.
        BatchCompleted,
        /// Batch of dispatches completed but has errors.
        BatchCompletedWithErrors,
        /// A single item within a Batch of dispatches has completed with no error.
        ItemCompleted,
        /// A single item within a Batch of dispatches has completed with error.
        ItemFailed { error: DispatchError },
        /// A call was dispatched.
        DispatchedAs { result: DispatchResult },
        /// Relayed transaction.
        /// POLYMESH: event.
        RelayedTx {
            caller_did: IdentityId,
            target: T::AccountId,
            result: DispatchResult,
        },
    }

    // Align the call size to 1KB. As we are currently compiling the runtime for native/wasm
    // the `size_of` of the `Call` can be different. To ensure that this don't leads to
    // mismatches between native/wasm or to different metadata for the same runtime, we
    // algin the call size. The value is chosen big enough to hopefully never reach it.
    const CALL_ALIGN: u32 = 1024;

    #[pallet::extra_constants]
    impl<T: Config> Pallet<T> {
        /// The limit on the number of batched calls.
        fn batched_calls_limit() -> u32 {
            let allocator_limit = sp_core::MAX_POSSIBLE_ALLOCATION;
            let call_size =
                ((sp_std::mem::size_of::<<T as Config>::RuntimeCall>() as u32 + CALL_ALIGN - 1)
                    / CALL_ALIGN)
                    * CALL_ALIGN;
            // The margin to take into account vec doubling capacity.
            let margin_factor = 3;

            allocator_limit / margin_factor / call_size
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn integrity_test() {
            // If you hit this error, you need to try to `Box` big dispatchable parameters.
            assert!(
                sp_std::mem::size_of::<<T as Config>::RuntimeCall>() as u32 <= CALL_ALIGN,
                "Call enum size should be smaller than {} bytes.",
                CALL_ALIGN,
            );
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Too many calls batched.
        TooManyCalls,
        /// Offchain signature is invalid
        /// POLYMESH error
        InvalidSignature,
        /// Target does not have a valid CDD
        /// POLYMESH error
        TargetCddMissing,
        /// Provided nonce was invalid
        /// If the provided nonce < current nonce, the call was already executed
        /// If the provided nonce > current nonce, the call(s) before the current failed to execute
        /// POLYMESH error
        InvalidNonce,
        /// Decoding derivative account Id failed.
        UnableToDeriveAccountId,
    }

    /// Nonce for `relay_tx`.
    /// POLYMESH: added.
    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub(super) type Nonces<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, AuthorizationNonce, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Send a batch of dispatch calls.
        ///
        /// May be called from any origin except `None`.
        ///
        /// - `calls`: The calls to be dispatched from the same origin. The number of call must not
        ///   exceed the constant: `batched_calls_limit` (available in constant metadata).
        ///
        /// If origin is root then the calls are dispatched without checking origin filter. (This
        /// includes bypassing `frame_system::Config::BaseCallFilter`).
        ///
        /// ## Complexity
        /// - O(C) where C is the number of calls to be batched.
        ///
        /// This will return `Ok` in all circumstances. To determine the success of the batch, an
        /// event is deposited. If a call failed and the batch was interrupted, then the
        /// `BatchInterrupted` event is deposited, along with the number of successful calls made
        /// and the error of the failed call. If all were successful, then the `BatchCompleted`
        /// event is deposited.
        #[pallet::call_index(0)]
        #[pallet::weight({
            let (dispatch_weight, dispatch_class) = Pallet::<T>::weight_and_dispatch_class(&calls);
            let dispatch_weight = dispatch_weight.saturating_add(<T as Config>::WeightInfo::batch(calls.len() as u32));
            (dispatch_weight, dispatch_class)
        })]
        pub fn batch(
            origin: OriginFor<T>,
            calls: Vec<<T as Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            // Do not allow the `None` origin.
            if ensure_none(origin.clone()).is_ok() {
                return Err(BadOrigin.into());
            }

            let is_root = ensure_root(origin.clone()).is_ok();
            let calls_len = calls.len();
            ensure!(
                calls_len <= Self::batched_calls_limit() as usize,
                Error::<T>::TooManyCalls
            );

            // Track the actual weight of each of the batch calls.
            let mut weight = Weight::zero();
            for (index, call) in calls.into_iter().enumerate() {
                let info = call.get_dispatch_info();
                // If origin is root, don't apply any dispatch filters; root can call anything.
                let result = Self::dispatch_call(origin.clone(), is_root, call);
                // Add the weight of this call.
                weight = weight.saturating_add(extract_actual_weight(&result, &info));
                if let Err(e) = result {
                    Self::deposit_event(Event::<T>::BatchInterrupted {
                        index: index as u32,
                        error: e.error,
                    });
                    // Take the weight of this function itself into account.
                    let base_weight =
                        <T as Config>::WeightInfo::batch(index.saturating_add(1) as u32);
                    // Return the actual used weight + base_weight of this call.
                    return Ok(Some(base_weight.saturating_add(weight)).into());
                }
                Self::deposit_event(Event::<T>::ItemCompleted);
            }
            Self::deposit_event(Event::<T>::BatchCompleted);
            let base_weight = <T as Config>::WeightInfo::batch(calls_len as u32);
            Ok(Some(base_weight.saturating_add(weight)).into())
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
        /// POLYMESH: added.
        #[pallet::call_index(1)]
        #[pallet::weight({
                let dispatch_info = call.call.get_dispatch_info();
                (
                    <T as Config>::WeightInfo::relay_tx()
                        .saturating_add(dispatch_info.weight),
                    dispatch_info.class,
                )
            })]
        pub fn relay_tx(
            origin: OriginFor<T>,
            target: T::AccountId,
            signature: T::OffChainSignature,
            call: UniqueCall<<T as Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let caller_did = CallPermissions::<T>::ensure_call_permissions(&sender)?.primary_did;

            let target_nonce = <Nonces<T>>::get(&target);

            ensure!(target_nonce == call.nonce, Error::<T>::InvalidNonce);

            ensure!(
                signature.verify(call.encode().as_slice(), &target),
                Error::<T>::InvalidSignature
            );

            ensure!(
                T::CddChecker::check_key_cdd(&target),
                Error::<T>::TargetCddMissing
            );

            <Nonces<T>>::insert(target.clone(), target_nonce + 1);

            let info = call.call.get_dispatch_info();
            // Dispatch the call with the `target` as the signed origin.
            let result =
                Self::dispatch_call(RawOrigin::Signed(target.clone()).into(), false, *call.call);
            // Get the actual weight of this call.
            let weight = extract_actual_weight(&result, &info);

            Self::deposit_event(Event::<T>::RelayedTx {
                caller_did,
                target,
                result: result.map(|_| ()).map_err(|e| e.error),
            });

            let base_weight = <T as Config>::WeightInfo::relay_tx();
            Ok(Some(base_weight.saturating_add(weight)).into())
        }

        /// Send a batch of dispatch calls and atomically execute them.
        /// The whole transaction will rollback and fail if any of the calls failed.
        ///
        /// May be called from any origin except `None`.
        ///
        /// - `calls`: The calls to be dispatched from the same origin. The number of call must not
        ///   exceed the constant: `batched_calls_limit` (available in constant metadata).
        ///
        /// If origin is root then the calls are dispatched without checking origin filter. (This
        /// includes bypassing `frame_system::Config::BaseCallFilter`).
        ///
        /// ## Complexity
        /// - O(C) where C is the number of calls to be batched.
        #[pallet::call_index(2)]
        #[pallet::weight({
            let (dispatch_weight, dispatch_class) = Pallet::<T>::weight_and_dispatch_class(&calls);
            let dispatch_weight = dispatch_weight.saturating_add(<T as Config>::WeightInfo::batch_all(calls.len() as u32));
            (dispatch_weight, dispatch_class)
        })]
        pub fn batch_all(
            origin: OriginFor<T>,
            calls: Vec<<T as Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            // Do not allow the `None` origin.
            if ensure_none(origin.clone()).is_ok() {
                return Err(BadOrigin.into());
            }

            let is_root = ensure_root(origin.clone()).is_ok();
            let calls_len = calls.len();
            ensure!(
                calls_len <= Self::batched_calls_limit() as usize,
                Error::<T>::TooManyCalls
            );

            // POLYMESH: Create filtered origin here.
            let filtered_origin = if is_root {
                origin
            } else {
                let mut filtered_origin = origin;
                // Don't allow users to nest `batch_all` calls.
                filtered_origin.add_filter(move |c: &<T as frame_system::Config>::RuntimeCall| {
                    let c = <T as Config>::RuntimeCall::from_ref(c);
                    !matches!(c.is_sub_type(), Some(Call::batch_all { .. }))
                });
                filtered_origin
            };

            // Track the actual weight of each of the batch calls.
            let mut weight = Weight::zero();
            for (index, call) in calls.into_iter().enumerate() {
                let info = call.get_dispatch_info();
                // If origin is root, bypass any dispatch filter; root can call anything.
                let result = Self::dispatch_call(filtered_origin.clone(), is_root, call);
                // Add the weight of this call.
                weight = weight.saturating_add(extract_actual_weight(&result, &info));
                result.map_err(|mut err| {
                    // Take the weight of this function itself into account.
                    let base_weight =
                        <T as Config>::WeightInfo::batch_all(index.saturating_add(1) as u32);
                    // Return the actual used weight + base_weight of this call.
                    err.post_info = Some(base_weight.saturating_add(weight)).into();
                    err
                })?;
                Self::deposit_event(Event::<T>::ItemCompleted);
            }
            Self::deposit_event(Event::<T>::BatchCompleted);
            let base_weight = <T as Config>::WeightInfo::batch_all(calls_len as u32);
            Ok(Some(base_weight.saturating_add(weight)).into())
        }

        /// Dispatches a function call with a provided origin.
        ///
        /// The dispatch origin for this call must be _Root_.
        ///
        /// ## Complexity
        /// - O(1).
        #[pallet::call_index(3)]
        #[pallet::weight({
                let dispatch_info = call.get_dispatch_info();
                (
                    <T as Config>::WeightInfo::dispatch_as()
                        .saturating_add(dispatch_info.weight),
                    dispatch_info.class,
                )
            })]
        pub fn dispatch_as(
            origin: OriginFor<T>,
            as_origin: Box<T::PalletsOrigin>,
            call: Box<<T as Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            Self::base_dispatch_as(origin, as_origin, call)
        }

        /// Send a batch of dispatch calls.
        /// Unlike `batch`, it allows errors and won't interrupt.
        ///
        /// May be called from any origin except `None`.
        ///
        /// - `calls`: The calls to be dispatched from the same origin. The number of call must not
        ///   exceed the constant: `batched_calls_limit` (available in constant metadata).
        ///
        /// If origin is root then the calls are dispatch without checking origin filter. (This
        /// includes bypassing `frame_system::Config::BaseCallFilter`).
        ///
        /// ## Complexity
        /// - O(C) where C is the number of calls to be batched.
        #[pallet::call_index(4)]
        #[pallet::weight({
            let (dispatch_weight, dispatch_class) = Pallet::<T>::weight_and_dispatch_class(&calls);
            let dispatch_weight = dispatch_weight.saturating_add(<T as Config>::WeightInfo::force_batch(calls.len() as u32));
            (dispatch_weight, dispatch_class)
        })]
        pub fn force_batch(
            origin: OriginFor<T>,
            calls: Vec<<T as Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            // Do not allow the `None` origin.
            if ensure_none(origin.clone()).is_ok() {
                return Err(BadOrigin.into());
            }

            let is_root = ensure_root(origin.clone()).is_ok();
            let calls_len = calls.len();
            ensure!(
                calls_len <= Self::batched_calls_limit() as usize,
                Error::<T>::TooManyCalls
            );

            // Track the actual weight of each of the batch calls.
            let mut weight = Weight::zero();
            // Track failed dispatch occur.
            let mut has_error: bool = false;
            for call in calls.into_iter() {
                let info = call.get_dispatch_info();
                // If origin is root, don't apply any dispatch filters; root can call anything.
                let result = Self::dispatch_call(origin.clone(), is_root, call);
                // Add the weight of this call.
                weight = weight.saturating_add(extract_actual_weight(&result, &info));
                if let Err(e) = result {
                    has_error = true;
                    Self::deposit_event(Event::<T>::ItemFailed { error: e.error });
                } else {
                    Self::deposit_event(Event::<T>::ItemCompleted);
                }
            }
            if has_error {
                Self::deposit_event(Event::<T>::BatchCompletedWithErrors);
            } else {
                Self::deposit_event(Event::<T>::BatchCompleted);
            }
            let base_weight = <T as Config>::WeightInfo::batch(calls_len as u32);
            Ok(Some(base_weight.saturating_add(weight)).into())
        }

        /// Dispatch a function call with a specified weight.
        ///
        /// This function does not check the weight of the call, and instead allows the
        /// Root origin to specify the weight of the call.
        ///
        /// The dispatch origin for this call must be _Root_.
        #[pallet::call_index(5)]
        #[pallet::weight((*_weight, call.get_dispatch_info().class))]
        pub fn with_weight(
            origin: OriginFor<T>,
            call: Box<<T as Config>::RuntimeCall>,
            _weight: Weight,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_root(origin)?;
            call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into())
                .map_err(|e| e.error)?;
            Ok(().into())
        }

        /// Send a call through an indexed pseudonym of the sender.
        ///
        /// Filter from origin are passed along. The call will be dispatched with an origin which
        /// use the same filter as the origin of this call.
        ///
        /// The dispatch origin for this call must be _Signed_.
        #[pallet::call_index(9)]
        #[pallet::weight({
            let dispatch_info = call.get_dispatch_info();
			(
				<T as Config>::WeightInfo::as_derivative()
					.saturating_add(T::DbWeight::get().reads_writes(1, 1))
					.saturating_add(dispatch_info.weight),
				dispatch_info.class,
			)
        })]
        pub fn as_derivative(
            origin: OriginFor<T>,
            index: u16,
            call: Box<<T as Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            Self::base_as_derivative(origin, index, call)
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Get the accumulated `weight` and the dispatch class for the given `calls`.
    fn weight_and_dispatch_class(calls: &[<T as Config>::RuntimeCall]) -> (Weight, DispatchClass) {
        let dispatch_infos = calls.iter().map(|call| call.get_dispatch_info());
        let (dispatch_weight, dispatch_class) = dispatch_infos.fold(
            (Weight::zero(), DispatchClass::Operational),
            |(total_weight, dispatch_class): (Weight, DispatchClass), di| {
                (
                    total_weight.saturating_add(di.weight),
                    // If not all are `Operational`, we want to use `DispatchClass::Normal`.
                    if di.class == DispatchClass::Normal {
                        di.class
                    } else {
                        dispatch_class
                    },
                )
            },
        );

        (dispatch_weight, dispatch_class)
    }
}

// POLYMESH:
impl<T: Config> Pallet<T> {
    fn dispatch_call(
        origin: T::RuntimeOrigin,
        is_root: bool,
        call: <T as Config>::RuntimeCall,
    ) -> DispatchResultWithPostInfo {
        with_call_metadata(call.get_call_metadata(), || {
            // If origin is root, don't apply any dispatch filters; root can call anything.
            if is_root {
                call.dispatch_bypass_filter(origin)
            } else {
                call.dispatch(origin)
            }
        })
    }

    /// Ensure `origin` is Root, if not return a fix small weight.
    pub(crate) fn ensure_root(origin: T::RuntimeOrigin) -> DispatchResultWithPostInfo {
        // Ensure the origin is Root.
        if ensure_root(origin).is_err() {
            // Return a minimal weight.
            return Err(DispatchErrorWithPostInfo {
                post_info: Some(<T as Config>::WeightInfo::ensure_root()).into(),
                error: DispatchError::BadOrigin,
            });
        }
        Ok(().into())
    }

    fn base_dispatch_as(
        origin: T::RuntimeOrigin,
        as_origin: Box<T::PalletsOrigin>,
        call: Box<<T as Config>::RuntimeCall>,
    ) -> DispatchResultWithPostInfo {
        Self::ensure_root(origin)?;

        let as_origin: T::RuntimeOrigin = (*as_origin).into();

        let behalf_account_id = {
            match as_origin.clone().into() {
                Ok(RawOrigin::Signed(account_id)) => Some(account_id.clone()),
                _ => None,
            }
        };

        let dispatch_info = call.get_dispatch_info();
        let call_result = Self::run_with_temporary_payer(as_origin, behalf_account_id, call, true);
        // Get the actual weight of this call.
        let weight = extract_actual_weight(&call_result, &dispatch_info);

        Self::deposit_event(Event::<T>::DispatchedAs {
            result: call_result.map(|_| ()).map_err(|e| e.error),
        });
        // POLYMESH: return the actual weight of the call.
        let base_weight = <T as Config>::WeightInfo::dispatch_as();
        Ok(Some(base_weight.saturating_add(weight)).into())
    }

    fn base_as_derivative(
        origin: T::RuntimeOrigin,
        index: u16,
        call: Box<<T as Config>::RuntimeCall>,
    ) -> DispatchResultWithPostInfo {
        let origin_account_id = ensure_signed(origin.clone())?;

        // Sets the caller to be the derivative of the caller's account
        let derivative_account_id = Self::derivative_account_id(origin_account_id, index)?;
        let mut origin = origin;
        origin.set_caller_from(frame_system::RawOrigin::Signed(
            derivative_account_id.clone(),
        ));

        let dispatch_info = call.get_dispatch_info();
        let call_result =
            Self::run_with_temporary_payer(origin, Some(derivative_account_id), call, false);

        // Always take into account the base weight of this call and add the real weight of the dispatch.
        let mut weight = <T as Config>::WeightInfo::as_derivative()
            .saturating_add(T::DbWeight::get().reads_writes(1, 1));
        weight = weight.saturating_add(extract_actual_weight(&call_result, &dispatch_info));

        call_result
            .map_err(|mut err| {
                err.post_info = Some(weight).into();
                err
            })
            .map(|_| Some(weight).into())
    }

    /// Derive a derivative account ID from the owner account and the index.
    pub fn derivative_account_id(
        origin_account_id: T::AccountId,
        index: u16,
    ) -> Result<T::AccountId, DispatchError> {
        let entropy = (b"modlpy/utilisuba", origin_account_id, index).using_encoded(blake2_256);
        Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .map_err(|_| Error::<T>::UnableToDeriveAccountId.into())
    }

    /// Dispatches `call` Setting CurrentPayer to `account_id`.
    /// The value is reset once the call is done.
    fn run_with_temporary_payer(
        origin: T::RuntimeOrigin,
        account_id: Option<T::AccountId>,
        call: Box<<T as Config>::RuntimeCall>,
        bypass_filter: bool,
    ) -> Result<PostDispatchInfo, DispatchErrorWithPostInfo> {
        // Hold the original value for payer.
        let original_payer = Context::current_payer::<Identity<T>>();
        // Temporarily change payer
        Context::set_current_payer::<Identity<T>>(account_id);
        // dispatch the call
        let call_result = {
            with_call_metadata(call.get_call_metadata(), || {
                if bypass_filter {
                    call.dispatch_bypass_filter(origin)
                } else {
                    call.dispatch(origin)
                }
            })
        };
        // Restore the original payer
        Context::set_current_payer::<Identity<T>>(original_payer);
        call_result
    }
}
