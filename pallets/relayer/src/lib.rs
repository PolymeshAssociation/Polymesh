// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

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
//! The Simple Relayer module provides extrinsics for subsidising of
//! both transaction fees as well as protocol fees.
//!
//! ## Overview
//!
//! The Simple Relayer module provides functions for:
//!
//! - Adding or removing a subsidiser for another user's key.
//! - Managing how much POLYX can be used by a user key to pay
//!   transaction/protocol fees.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `approve_subsidy` approves a subsidy for a `user_key`.
//! - `accept_subsidy` accepts a subsidy from a `paying_key`.
//! - `remove_subsidy` removes a subsidy between a `user_key` and a `paying_key`.
//! - `update_polyx_limit` updates the available POLYX for a `user_key`.
//! - `increase_polyx_limit` increases the available POLYX for a `user_key`.
//! - `decrease_polyx_limit` decreases the available POLYX for a `user_key`.
//! - `relay_tx` relays a transaction on behalf of a target account.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::{extract_actual_weight, DispatchResult, GetDispatchInfo};
use frame_support::pallet_prelude::{DispatchError, DispatchResultWithPostInfo};
use frame_support::traits::{Contains, GetCallMetadata};
use frame_support::weights::Weight;
use frame_support::{ensure, fail};
use frame_system::{ensure_signed, RawOrigin};
use scale_info::TypeInfo;
use sp_runtime::transaction_validity::InvalidTransaction;
use sp_std::prelude::*;

use polymesh_primitives::crypto::{ChainScopedMessage, RELAY_TX_LABEL};
use polymesh_primitives::identity::AuthorizationNonce;
use polymesh_primitives::traits::SubsidiserTrait;
use polymesh_primitives::{Balance, TransactionError};

pub trait WeightInfo {
    fn approve_subsidy() -> Weight;
    fn accept_subsidy() -> Weight;
    fn revoke_subsidy() -> Weight;
    fn remove_subsidy() -> Weight;
    fn update_polyx_limit() -> Weight;
    fn increase_polyx_limit() -> Weight;
    fn decrease_polyx_limit() -> Weight;

    fn relay_tx() -> Weight;
}

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_identity::Config + pallet_utility::Config
    {
        /// Subsidy pallet weights.
        type WeightInfo: WeightInfo;
        /// Subsidy call filter.
        type SubsidyCallFilter: frame_support::traits::Contains<
            <Self as frame_system::Config>::RuntimeCall,
        >;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A `paying_key` has approved subsidy for a `user_key`.
        ApprovedSubsidy {
            /// The user key to be subsidised.
            user_key: T::AccountId,
            /// The paying key that will subsidise the `user_key`.
            paying_key: T::AccountId,
            /// The initial POLYX limit for this subsidy.
            initial_polyx_limit: Balance,
        },

        /// Accepted subsidy.
        AcceptedSubsidy {
            /// The user key being subsidised.
            user_key: T::AccountId,
            /// The paying key that is subsidising the `user_key`.
            paying_key: T::AccountId,
            /// The initial POLYX limit for this subsidy.
            initial_polyx_limit: Balance,
        },

        /// Removed subsidy.
        RemovedSubsidy {
            /// The user key that was being subsidised.
            user_key: T::AccountId,
            /// The paying key that was subsidising the `user_key`.
            paying_key: T::AccountId,
            /// The remaining POLYX for this subsidy.
            remaining: Balance,
        },

        /// Removed pending subsidy.
        RemovedPendingSubsidy {
            /// The user key that was being subsidised.
            user_key: T::AccountId,
            /// The paying key that was subsidising the `user_key`.
            paying_key: T::AccountId,
            /// The initial POLYX limit for this subsidy.
            initial_polyx_limit: Balance,
        },

        /// Updated polyx limit.
        UpdatedPolyxLimit {
            /// The user key being subsidised.
            user_key: T::AccountId,
            /// The paying key that is subsidising the `user_key`.
            paying_key: T::AccountId,
            /// The new remaining POLYX for this subsidy.
            remaining: Balance,
            /// The old remaining POLYX for this subsidy.
            old_remaining: Balance,
        },

        /// The subsidy fee has been used to pay for a transaction or protocol fee.
        SubsidyDebited {
            /// The user key being subsidised.
            user_key: T::AccountId,
            /// The paying key that is subsidising the `user_key`.
            paying_key: T::AccountId,
            /// The amount of POLYX debited.
            amount: Balance,
        },

        /// Relayed transaction.
        RelayedTx {
            caller: T::AccountId,
            target: T::AccountId,
            result: DispatchResult,
        },
    }

    /// A Subsidy for transaction and protocol fees.
    ///
    /// This holds the subsidiser's paying key and the remaining POLYX balance
    /// available for subsidising transaction and protocol fees.
    #[derive(
        Encode,
        Decode,
        TypeInfo,
        Clone,
        Debug,
        Default,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        MaxEncodedLen
    )]
    pub struct Subsidy<Acc> {
        /// The subsidiser's paying key.
        pub paying_key: Acc,
        /// How much POLYX is remaining for subsidising transaction and protocol fees.
        pub remaining: Balance,
    }

    /// Update action for subsidy POLYX limit.
    #[derive(Copy, Clone)]
    pub enum UpdateAction {
        /// Set the current subsidy limit to `amount`.
        Set,
        /// Add `amount` to the current subsidy limit.
        Add,
        /// Subtract `amount` from the current subsidy limit.
        Sub,
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// The subsidy for a `user_key` if they are being subsidised,
    /// as a map `user_key` => `Subsidy`.
    ///
    /// A key can only have one subsidy at a time.  Accepting a new subsidy
    /// will replace any existing subsidy.
    #[pallet::storage]
    pub type Subsidies<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Subsidy<T::AccountId>, OptionQuery>;

    /// Pending subsidies for a `user_key` from a `paying_key`.
    ///
    /// This is used to track subsidies that have been authorised but not yet accepted.
    ///
    /// The paying key can update or cancel the subsidy before it is accepted.
    #[pallet::storage]
    pub type PendingSubsidies<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // user_key
        Blake2_128Concat,
        T::AccountId, // paying_key
        Balance,      // initial POLYX limit
        OptionQuery,
    >;

    /// Nonce for `relay_tx`.
    #[pallet::storage]
    pub type RelayTxNonces<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, AuthorizationNonce, ValueQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            use polymesh_primitives::{RocksDbWeight as DbWeight, Weight};
            if Pallet::<T>::on_chain_storage_version() < Pallet::<T>::in_code_storage_version() {
                let mut count = 0u64;
                // Remove account key ref counts for all paying keys and user keys in Subsidies.
                for (user_key, subsidy) in Subsidies::<T>::iter() {
                    count += 1;
                    // Decrease paying key usage.
                    pallet_identity::Pallet::<T>::remove_account_key_ref_count(&subsidy.paying_key);
                    // Decrease user key usage.
                    pallet_identity::Pallet::<T>::remove_account_key_ref_count(&user_key);
                }

                log::info!(
                    target: "relayer",
                    "Relayer storage migration: removed {} account key refs",
                    count * 2
                );
                STORAGE_VERSION.put::<Pallet<T>>();
                let db_weight = DbWeight::get();
                db_weight
                    .reads(count * 3)
                    .saturating_add(db_weight.writes(count * 2))
            } else {
                Weight::zero()
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Approve a subsidy for a `user_key`.
        ///
        /// # Arguments
        /// - `user_key` the user key to subsidise.
        /// - `polyx_limit` the initial POLYX limit for this subsidy.
        ///
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::approve_subsidy())]
        pub fn approve_subsidy(
            origin: OriginFor<T>,
            user_key: T::AccountId,
            polyx_limit: Balance,
        ) -> DispatchResult {
            Self::base_approve_subsidy(origin, user_key, polyx_limit)
        }

        /// Revoke a previously approved subsidy.
        ///
        /// # Arguments
        /// - `user_key` the user key to revoke the subsidy for.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::revoke_subsidy())]
        pub fn revoke_subsidy(origin: OriginFor<T>, user_key: T::AccountId) -> DispatchResult {
            Self::base_revoke_subsidy(origin, user_key)
        }

        /// Accepts a subsidy from a `paying_key`.
        ///
        /// # Arguments
        /// - `paying_key` the paying key that is subsidising the caller's `user_key`.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::accept_subsidy())]
        pub fn accept_subsidy(origin: OriginFor<T>, paying_key: T::AccountId) -> DispatchResult {
            Self::base_accept_subsidy(origin, paying_key)
        }

        /// Removes the `paying_key` from a `user_key`.
        ///
        /// This can only be called by either the `user_key` or the `paying_key`.
        ///
        /// # Arguments
        /// - `user_key` the user key to remove the subsidy from.
        /// - `paying_key` the paying key that was subsidising the `user_key`.
        ///
        /// # Errors
        /// - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
        /// - `NotPayingKey` if the `paying_key` doesn't match the current `paying_key`.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_subsidy())]
        pub fn remove_subsidy(
            origin: OriginFor<T>,
            user_key: T::AccountId,
            paying_key: T::AccountId,
        ) -> DispatchResult {
            Self::base_remove_subsidy(origin, user_key, paying_key)
        }

        /// Updates the available POLYX for a `user_key`.
        ///
        /// # Arguments
        /// - `user_key` the user key of the subsidy to update the available POLYX.
        /// - `polyx_limit` the amount of POLYX available for subsidising the `user_key`.
        ///
        /// # Errors
        /// - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
        /// - `NotPayingKey` if `origin` doesn't match the current `paying_key`.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::update_polyx_limit())]
        pub fn update_polyx_limit(
            origin: OriginFor<T>,
            user_key: T::AccountId,
            polyx_limit: Balance,
        ) -> DispatchResult {
            Self::base_update_polyx_limit(origin, user_key, UpdateAction::Set, polyx_limit)
        }

        /// Increase the available POLYX for a `user_key`.
        ///
        /// # Arguments
        /// - `user_key` the user key of the subsidy to update the available POLYX.
        /// - `amount` the amount of POLYX to add to the subsidy of `user_key`.
        ///
        /// # Errors
        /// - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
        /// - `NotPayingKey` if `origin` doesn't match the current `paying_key`.
        /// - `Overlow` if the subsidy's remaining POLYX would have overflowed `u128::MAX`.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::increase_polyx_limit())]
        pub fn increase_polyx_limit(
            origin: OriginFor<T>,
            user_key: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            Self::base_update_polyx_limit(origin, user_key, UpdateAction::Add, amount)
        }

        /// Decrease the available POLYX for a `user_key`.
        ///
        /// # Arguments
        /// - `user_key` the user key of the subsidy to update the available POLYX.
        /// - `amount` the amount of POLYX to remove from the subsidy of `user_key`.
        ///
        /// # Errors
        /// - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
        /// - `NotPayingKey` if `origin` doesn't match the current `paying_key`.
        /// - `Overlow` if the subsidy has less then `amount` POLYX remaining.
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::decrease_polyx_limit())]
        pub fn decrease_polyx_limit(
            origin: OriginFor<T>,
            user_key: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            Self::base_update_polyx_limit(origin, user_key, UpdateAction::Sub, amount)
        }

        /// Relay a call for a target from an origin
        ///
        /// Relaying in this context refers to the ability of origin to make a call on behalf of
        /// target.
        ///
        /// Transaction and protocol fees are charged to origin.
        ///
        /// # Parameters
        /// - `target`: Account to be relayed
        /// - `signature`: Signature from target authorizing the relay
        /// - `call`: Call to be relayed on behalf of target
        #[pallet::call_index(7)]
        #[pallet::weight({
                let dispatch_info = call.get_dispatch_info();
                (
                    <T as Config>::WeightInfo::relay_tx()
                        .saturating_add(dispatch_info.call_weight),
                    dispatch_info.class,
                )
            })]
        pub fn relay_tx(
            origin: OriginFor<T>,
            target: T::AccountId,
            signature: T::OffChainSignature,
            call: Box<<T as pallet_utility::Config>::RuntimeCall>,
            expires_at: T::Moment,
        ) -> DispatchResultWithPostInfo {
            let caller = ensure_signed(origin)?;

            // Get the current nonce for the target and increment it.
            let nonce = RelayTxNonces::<T>::mutate(&target, |nonce| {
                let current_nonce = *nonce;
                *nonce = current_nonce + 1;
                current_nonce
            });
            // Create the chain scoped message that the target needs to have signed.
            let scoped_call =
                ChainScopedMessage::<T, _>::new(nonce, RELAY_TX_LABEL, expires_at, &call)
                    .ok_or(Error::<T>::ExpiredRelayTx)?;

            ensure!(
                scoped_call.verify_signature(&target, &signature),
                Error::<T>::InvalidSignature
            );

            let info = call.get_dispatch_info();
            // Dispatch the call with the `target` as the signed origin.
            let result = pallet_utility::Pallet::<T>::dispatch_call(
                RawOrigin::Signed(target.clone()).into(),
                false,
                *call,
            );
            // Get the actual weight of this call.
            let weight = extract_actual_weight(&result, &info);

            Self::deposit_event(Event::<T>::RelayedTx {
                caller,
                target,
                result: result.map(|_| ()).map_err(|e| e.error),
            });

            let base_weight = <T as Config>::WeightInfo::relay_tx();
            Ok(Some(base_weight.saturating_add(weight)).into())
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The `user_key` doesn't have a `paying_key`.
        NoPayingKey,
        /// The `user_key` has a different `paying_key`.
        NotPayingKey,
        /// The signer is not the `paying_key` or the `user_key`.
        NotAuthorized,
        /// The remaining POLYX for `user_key` overflowed.
        Overflow,
        /// There is no pending subsidy from the `paying_key` for the `user_key`.
        NoPendingSubsidy,
        /// Offchain signature is invalid
        InvalidSignature,
        /// The relay transaction has expired.
        ExpiredRelayTx,
    }
}

impl<T: Config> Pallet<T> {
    fn base_approve_subsidy(
        origin: T::RuntimeOrigin,
        user_key: T::AccountId,
        initial_polyx_limit: Balance,
    ) -> DispatchResult {
        let paying_key = ensure_signed(origin)?;

        // Set or update pending subsidy.
        PendingSubsidies::<T>::insert(&user_key, &paying_key, initial_polyx_limit);

        Self::deposit_event(Event::ApprovedSubsidy {
            user_key,
            paying_key,
            initial_polyx_limit,
        });
        Ok(())
    }

    fn base_revoke_subsidy(origin: T::RuntimeOrigin, user_key: T::AccountId) -> DispatchResult {
        let paying_key = ensure_signed(origin)?;

        // Remove pending subsidy.
        let initial_polyx_limit = PendingSubsidies::<T>::take(&user_key, &paying_key)
            .ok_or(Error::<T>::NoPendingSubsidy)?;

        // Emit event.
        Self::deposit_event(Event::RemovedPendingSubsidy {
            user_key,
            paying_key,
            initial_polyx_limit,
        });
        Ok(())
    }

    fn base_accept_subsidy(origin: T::RuntimeOrigin, paying_key: T::AccountId) -> DispatchResult {
        let user_key = ensure_signed(origin)?;

        // Get the pending subsidy.
        let initial_polyx_limit = PendingSubsidies::<T>::take(&user_key, &paying_key)
            .ok_or(Error::<T>::NoPendingSubsidy)?;

        // Remove existing subsidy for the user_key, if it exists.
        if let Some(subsidy) = Subsidies::<T>::get(&user_key) {
            Self::deposit_event(Event::RemovedSubsidy {
                user_key: user_key.clone(),
                paying_key: subsidy.paying_key,
                remaining: subsidy.remaining,
            });
        }

        // All checks passed.
        Subsidies::<T>::insert(
            &user_key,
            Subsidy {
                paying_key: paying_key.clone(),
                remaining: initial_polyx_limit,
            },
        );

        Self::deposit_event(Event::AcceptedSubsidy {
            user_key,
            paying_key,
            initial_polyx_limit,
        });

        Ok(())
    }

    fn base_remove_subsidy(
        origin: T::RuntimeOrigin,
        user_key: T::AccountId,
        paying_key: T::AccountId,
    ) -> DispatchResult {
        let caller_key = ensure_signed(origin)?;

        // Allow: `origin == user_key` or `origin == paying_key`.
        ensure!(
            caller_key == user_key || caller_key == paying_key,
            Error::<T>::NotAuthorized
        );

        // Check if the current paying key matches.
        let subsidy = Self::ensure_is_paying_key(&user_key, &paying_key)?;

        // Remove paying key for user key.
        Subsidies::<T>::remove(&user_key);

        Self::deposit_event(Event::RemovedSubsidy {
            user_key,
            paying_key,
            remaining: subsidy.remaining,
        });
        Ok(())
    }

    fn base_update_polyx_limit(
        origin: T::RuntimeOrigin,
        user_key: T::AccountId,
        action: UpdateAction,
        amount: Balance,
    ) -> DispatchResult {
        let paying_key = ensure_signed(origin)?;

        // Check if the current paying key matches.
        let mut subsidy = Self::ensure_is_paying_key(&user_key, &paying_key)?;

        // Update polyx limit.
        let old_remaining = subsidy.remaining;
        let new_remaining = match action {
            UpdateAction::Set => amount,
            UpdateAction::Add => old_remaining
                .checked_add(amount)
                .ok_or(Error::<T>::Overflow)?,
            UpdateAction::Sub => old_remaining
                .checked_sub(amount)
                .ok_or(Error::<T>::Overflow)?,
        };
        subsidy.remaining = new_remaining;
        Subsidies::<T>::insert(&user_key, subsidy);

        Self::deposit_event(Event::UpdatedPolyxLimit {
            user_key,
            paying_key,
            remaining: new_remaining,
            old_remaining,
        });
        Ok(())
    }

    /// Ensure that `paying_key` is the paying key for `user_key`.
    fn ensure_is_paying_key(
        user_key: &T::AccountId,
        paying_key: &T::AccountId,
    ) -> Result<Subsidy<T::AccountId>, DispatchError> {
        // Check if the current paying key matches.
        match Subsidies::<T>::get(user_key) {
            // There was no subsidy.
            None => fail!(Error::<T>::NoPayingKey),
            // Paying key doesn't match.
            Some(s) if s.paying_key != *paying_key => fail!(Error::<T>::NotPayingKey),
            Some(s) => Ok(s),
        }
    }

    fn get_subsidy(
        user_key: &T::AccountId,
        fee: Balance,
    ) -> Result<Option<Subsidy<T::AccountId>>, InvalidTransaction> {
        // Get the Subsidy for `user_key`.
        match Subsidies::<T>::get(user_key) {
            // There was no subsidy.
            None => Ok(None),
            // Has subsidy, but not enough remaining POLYX.
            Some(s) if s.remaining < fee => fail!(InvalidTransaction::Payment),
            // Has subsidy and enough POLYX.
            Some(s) => Ok(Some(s)),
        }
    }

    /// Check if the `user_key` has a pending subsidy from `paying_key`.
    pub fn has_pending_subsidy(user_key: &T::AccountId, paying_key: &T::AccountId) -> bool {
        PendingSubsidies::<T>::contains_key(user_key, paying_key)
    }
}

impl<T: Config> Pallet<T>
where
    <T as frame_system::Config>::RuntimeCall: GetCallMetadata,
{
    fn ensure_subsidy_call(
        call: &<T as frame_system::Config>::RuntimeCall,
    ) -> Result<bool, InvalidTransaction> {
        // Check if the call is allowed to be subsidised.
        if T::SubsidyCallFilter::contains(call) {
            Ok(true)
        } else {
            let metadata = call.get_call_metadata();
            if metadata.pallet_name == "Relayer" {
                // The user key needs to pay for `remove_subsidy` call.
                Ok(false)
            } else {
                Err(InvalidTransaction::Custom(
                    TransactionError::PalletNotSubsidised as u8,
                ))
            }
        }
    }
}

impl<T: Config> SubsidiserTrait<T::AccountId, <T as frame_system::Config>::RuntimeCall>
    for Pallet<T>
where
    <T as frame_system::Config>::RuntimeCall: GetCallMetadata,
{
    fn check_subsidy(
        user_key: &T::AccountId,
        fee: Balance,
        call: Option<&<T as frame_system::Config>::RuntimeCall>,
    ) -> Result<Option<T::AccountId>, InvalidTransaction> {
        match (Self::get_subsidy(user_key, fee)?, call) {
            (Some(s), Some(call)) => {
                // Ensure the call can be subsidised.
                if !Self::ensure_subsidy_call(call)? {
                    // The caller must pay for the transaction themselves.
                    return Ok(None);
                }
                Ok(Some(s.paying_key))
            }
            (Some(s), None) => {
                // No pallet restriction applied (protocol fees).
                Ok(Some(s.paying_key))
            }
            (None, _) => Ok(None),
        }
    }

    fn debit_subsidy(
        user_key: &T::AccountId,
        fee: Balance,
    ) -> Result<Option<T::AccountId>, InvalidTransaction> {
        if let Some(mut subsidy) = Self::get_subsidy(user_key, fee)? {
            let paying_key = subsidy.paying_key.clone();
            // Debit the fee from the remaining POLYX of subsidy.
            subsidy.remaining = subsidy.remaining.saturating_sub(fee);
            Subsidies::<T>::insert(user_key, subsidy);

            // Emit event.
            Self::deposit_event(Event::SubsidyDebited {
                user_key: user_key.clone(),
                paying_key: paying_key.clone(),
                amount: fee,
            });

            Ok(Some(paying_key))
        } else {
            Ok(None)
        }
    }

    fn reserve_subsidy(
        user_key: &T::AccountId,
        fee: Balance,
    ) -> Result<Option<T::AccountId>, InvalidTransaction> {
        if let Some(mut subsidy) = Self::get_subsidy(user_key, fee)? {
            let paying_key = subsidy.paying_key.clone();
            subsidy.remaining = subsidy
                .remaining
                .checked_sub(fee)
                .ok_or(InvalidTransaction::Payment)?;
            Subsidies::<T>::insert(user_key, subsidy);
            Ok(Some(paying_key))
        } else {
            Ok(None)
        }
    }

    fn settle_subsidy(
        user_key: &T::AccountId,
        paying_key: &T::AccountId,
        reserved: Balance,
        actual: Balance,
    ) {
        if let Some(mut subsidy) = Subsidies::<T>::get(user_key) {
            // Only refund to the subsidy if the paying_key still matches.
            // The user may have switched to a different subsidiser during dispatch.
            if subsidy.paying_key == *paying_key {
                let refund = reserved.saturating_sub(actual);
                subsidy.remaining = subsidy.remaining.saturating_add(refund);
                Subsidies::<T>::insert(user_key, subsidy);
            }
        }

        // Always emit the event using the original paying_key from prepare,
        // even if the subsidy was removed or changed during dispatch.
        Self::deposit_event(Event::SubsidyDebited {
            user_key: user_key.clone(),
            paying_key: paying_key.clone(),
            amount: actual,
        });
    }
}
