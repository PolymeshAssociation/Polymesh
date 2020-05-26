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
// TODO

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
//! #### For pseudonymal dispatch
//! * `as_sub` - Dispatch a call from a secondary ("sub") signed origin.
//!
//! #### For multisig dispatch
//! * `as_multi` - Approve and if possible dispatch a call from a composite origin formed from a
//!   number of signed origins.
//! * `approve_as_multi` - Approve a call from a composite origin.
//! * `cancel_as_multi` - Cancel a call from a composite origin.
//!
//! [`Call`]: ./enum.Call.html
//! [`Trait`]: ./trait.Trait.html

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    weights::{DispatchClass, FunctionOf, GetDispatchInfo},
    Parameter,
};
use frame_system as system;
use sp_core::TypeId;
use sp_runtime::{traits::Dispatchable, DispatchError, DispatchResult};
use sp_std::prelude::*;

/// Configuration trait.
pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    /// The overarching call type.
    type Call: Parameter + Dispatchable<Origin = Self::Origin> + GetDispatchInfo;
}

decl_storage! {
    trait Store for Module<T: Trait> as Utility { }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Threshold is too low (zero).
        ZeroThreshold,
        /// Call is already approved by this signatory.
        AlreadyApproved,
        /// Call doesn't need any (more) approvals.
        NoApprovalsNeeded,
        /// There are too few signatories in the list.
        TooFewSignatories,
        /// There are too many signatories in the list.
        TooManySignatories,
        /// The signatories were provided out of order; they should be ordered.
        SignatoriesOutOfOrder,
        /// The sender was contained in the other signatories; it shouldn't be.
        SenderInSignatories,
        /// Multisig operation not found when attempting to cancel.
        NotFound,
        /// Only the account that originally created the multisig is able to cancel it.
        NotOwner,
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

/// A module identifier. These are per module and should be stored in a registry somewhere.
#[derive(Clone, Copy, Eq, PartialEq, Encode, Decode)]
struct IndexedUtilityModuleId(u16);

impl TypeId for IndexedUtilityModuleId {
    const TYPE_ID: [u8; 4] = *b"suba";
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
        #[weight = FunctionOf(
            |args: (&Vec<<T as Trait>::Call>,)| {
                args.0.iter()
                    .map(|call| call.get_dispatch_info().weight)
                    .fold(10_000, |a, n| a + n)
            },
            |args: (&Vec<<T as Trait>::Call>,)| {
                let all_operational = args.0.iter()
                    .map(|call| call.get_dispatch_info().class)
                    .all(|class| class == DispatchClass::Operational);
                if all_operational {
                    DispatchClass::Operational
                } else {
                    DispatchClass::Normal
                }
            },
            true
        )]
        pub fn batch(origin, calls: Vec<<T as Trait>::Call>) -> DispatchResult {
            for (index, call) in calls.into_iter().enumerate() {
                let result = call.dispatch(origin.clone());
                if let Err(e) = result {
                    Self::deposit_event(Event::BatchInterrupted(index as u32, e));
                    return Ok(());
                }
            }
            Self::deposit_event(Event::BatchCompleted);

            Ok(())
        }
    }
}
