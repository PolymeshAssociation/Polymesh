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
//! - `set_paying_key` creates an authorization to allow a `user_key`
//!   to accept a `paying_key` as their subsidiser.
//! - `accept_paying_key` accepts a `paying_key` authorization.
//! - `remove_paying_key` removes the `paying_key` from a `user_key`.
//! - `update_polyx_limit` updates the available POLYX for a `user_key`.
//! - `increase_polyx_limit` increases the available POLYX for a `user_key`.
//! - `decrease_polyx_limit` decreases the available POLYX for a `user_key`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure, fail,
};
use frame_system::ensure_signed;
use pallet_identity::PermissionedCallOriginData;
pub use polymesh_common_utilities::traits::relayer::{
    Config, Event, RawEvent, SubsidiserTrait, WeightInfo,
};
use polymesh_primitives::{
    extract_auth, AuthorizationData, Balance, IdentityId, Signatory, TransactionError,
};
use sp_runtime::transaction_validity::InvalidTransaction;

type Identity<T> = pallet_identity::Module<T>;

/// A Subsidy for transaction and protocol fees.
///
/// This holds the subsidiser's paying key and the remaining POLYX balance
/// available for subsidising transaction and protocol fees.
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subsidy<Acc> {
    /// The subsidiser's paying key.
    pub paying_key: Acc,
    /// How much POLYX is remaining for subsidising transaction and protocol fees.
    pub remaining: Balance,
}

/// Update action for subsidy POLYX limit.
enum UpdateAction {
    /// Set the current subsidy limit to `amount`.
    Set,
    /// Add `amount` to the current subsidy limit.
    Add,
    /// Subtract `amount` from the current subsidy limit.
    Sub,
}

decl_storage! {
    trait Store for Module<T: Config> as Relayer {
        /// The subsidy for a `user_key` if they are being subsidised,
        /// as a map `user_key` => `Subsidy`.
        ///
        /// A key can only have one subsidy at a time.  To change subsidisers
        /// a key needs to call `remove_paying_key` to remove the current subsidy,
        /// before they can accept a new subsidiser.
        pub Subsidies get(fn subsidies):
            map hasher(blake2_128_concat) T::AccountId => Option<Subsidy<T::AccountId>>;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Creates an authorization to allow `user_key` to accept the caller (`origin == paying_key`) as their subsidiser.
        ///
        /// # Arguments
        /// - `user_key` the user key to subsidise.
        /// - `polyx_limit` the initial POLYX limit for this subsidy.
        ///
        /// # Errors
        /// - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
        #[weight = <T as Config>::WeightInfo::set_paying_key()]
        pub fn set_paying_key(origin, user_key: T::AccountId, polyx_limit: Balance) -> DispatchResult {
            Self::base_set_paying_key(origin, user_key, polyx_limit)
        }

        /// Accepts a `paying_key` authorization.
        ///
        /// # Arguments
        /// - `auth_id` the authorization id to accept a `paying_key`.
        ///
        /// # Errors
        /// - `AuthorizationError::Invalid` if `auth_id` does not exist for the given caller.
        /// - `AuthorizationError::Expired` if `auth_id` the authorization has expired.
        /// - `AuthorizationError::BadType` if `auth_id` was not a `AddRelayerPayingKey` authorization.
        /// - `NotAuthorizedForUserKey` if `origin` is not authorized to accept the authorization for the `user_key`.
        /// - `NotAuthorizedForPayingKey` if the authorization was created by a signer that isn't authorized by the `paying_key`.
        /// - `UserKeyCddMissing` if the `user_key` is not attached to a CDD'd identity.
        /// - `PayingKeyCddMissing` if the `paying_key` is not attached to a CDD'd identity.
        /// - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
        #[weight = <T as Config>::WeightInfo::accept_paying_key()]
        pub fn accept_paying_key(origin, auth_id: u64) -> DispatchResult {
            Self::base_accept_paying_key(origin, auth_id)
        }

        /// Removes the `paying_key` from a `user_key`.
        ///
        /// # Arguments
        /// - `user_key` the user key to remove the subsidy from.
        /// - `paying_key` the paying key that was subsidising the `user_key`.
        ///
        /// # Errors
        /// - `NotAuthorizedForUserKey` if `origin` is not authorized to remove the subsidy for the `user_key`.
        /// - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
        /// - `NotPayingKey` if the `paying_key` doesn't match the current `paying_key`.
        /// - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
        #[weight = <T as Config>::WeightInfo::remove_paying_key()]
        pub fn remove_paying_key(origin, user_key: T::AccountId, paying_key: T::AccountId) -> DispatchResult {
            Self::base_remove_paying_key(origin, user_key, paying_key)
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
        /// - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
        #[weight = <T as Config>::WeightInfo::update_polyx_limit()]
        pub fn update_polyx_limit(origin, user_key: T::AccountId, polyx_limit: Balance) -> DispatchResult {
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
        /// - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
        /// - `Overlow` if the subsidy's remaining POLYX would have overflowed `u128::MAX`.
        #[weight = <T as Config>::WeightInfo::increase_polyx_limit()]
        pub fn increase_polyx_limit(origin, user_key: T::AccountId, amount: Balance) -> DispatchResult {
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
        /// - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
        /// - `Overlow` if the subsidy has less then `amount` POLYX remaining.
        #[weight = <T as Config>::WeightInfo::decrease_polyx_limit()]
        pub fn decrease_polyx_limit(origin, user_key: T::AccountId, amount: Balance) -> DispatchResult {
            Self::base_update_polyx_limit(origin, user_key, UpdateAction::Sub, amount)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The `user_key` is not attached to a CDD'd identity.
        UserKeyCddMissing,
        /// The `user_key` is not attached to a CDD'd identity.
        PayingKeyCddMissing,
        /// The `user_key` doesn't have a `paying_key`.
        NoPayingKey,
        /// The `user_key` has a different `paying_key`.
        NotPayingKey,
        /// The signer is not authorized for `paying_key`.
        NotAuthorizedForPayingKey,
        /// The signer is not authorized for `user_key`.
        NotAuthorizedForUserKey,
        /// The remaining POLYX for `user_key` overflowed.
        Overflow,
    }
}

impl<T: Config> Module<T> {
    fn base_set_paying_key(
        origin: T::Origin,
        user_key: T::AccountId,
        polyx_limit: Balance,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            sender: paying_key,
            primary_did: paying_did,
            ..
        } = <Identity<T>>::ensure_origin_call_permissions(origin)?;

        // Create authorization for `paying_key` to subsidise the `user_key`, with `polyx_limit` POLYX.
        Self::unverified_add_auth_for_paying_key(paying_did, user_key, paying_key, polyx_limit);
        Ok(())
    }

    fn base_accept_paying_key(origin: T::Origin, auth_id: u64) -> DispatchResult {
        let user_key = ensure_signed(origin)?;
        let user_did =
            <Identity<T>>::get_identity(&user_key).ok_or(Error::<T>::UserKeyCddMissing)?;
        let signer = Signatory::Account(user_key);

        <Identity<T>>::accept_auth_with(&signer, auth_id, |data, auth_by| -> DispatchResult {
            let (user_key, paying_key, polyx_limit) =
                extract_auth!(data, AddRelayerPayingKey(user_key, paying_key, polyx_limit));

            Self::auth_accept_paying_key(
                user_did,
                auth_by,
                user_key.clone(),
                paying_key.clone(),
                polyx_limit,
            )?;

            Self::deposit_event(RawEvent::AcceptedPayingKey(
                user_did.for_event(),
                user_key,
                paying_key,
            ));

            Ok(())
        })
    }

    fn base_remove_paying_key(
        origin: T::Origin,
        user_key: T::AccountId,
        paying_key: T::AccountId,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            sender,
            primary_did: sender_did,
            ..
        } = <Identity<T>>::ensure_origin_call_permissions(origin)?;

        // Allow: `origin == user_key` or `origin == paying_key`.
        if sender != user_key && sender != paying_key {
            // Allow: `origin == primary key of user_key's identity`.
            ensure!(
                <Identity<T>>::get_identity(&user_key) == Some(sender_did),
                Error::<T>::NotAuthorizedForUserKey
            );
        }

        // Check if the current paying key matches.
        Self::ensure_is_paying_key(&user_key, &paying_key)?;

        // Decrease paying key usage.
        <Identity<T>>::remove_account_key_ref_count(&paying_key);
        // Decrease user key usage.
        <Identity<T>>::remove_account_key_ref_count(&user_key);

        // Remove paying key for user key.
        <Subsidies<T>>::remove(&user_key);

        Self::deposit_event(RawEvent::RemovedPayingKey(
            sender_did.for_event(),
            user_key,
            paying_key,
        ));
        Ok(())
    }

    fn base_update_polyx_limit(
        origin: T::Origin,
        user_key: T::AccountId,
        action: UpdateAction,
        amount: Balance,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            sender: paying_key,
            primary_did: paying_did,
            ..
        } = <Identity<T>>::ensure_origin_call_permissions(origin)?;

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
        <Subsidies<T>>::insert(&user_key, subsidy);

        Self::deposit_event(RawEvent::UpdatedPolyxLimit(
            paying_did.for_event(),
            user_key,
            paying_key,
            new_remaining,
            old_remaining,
        ));
        Ok(())
    }

    /// Adds an authorization to add a `paying_key` to the `user_key`.
    pub fn unverified_add_auth_for_paying_key(
        from: IdentityId,
        user_key: T::AccountId,
        paying_key: T::AccountId,
        polyx_limit: Balance,
    ) -> u64 {
        let auth_id = <Identity<T>>::add_auth(
            from,
            Signatory::Account(user_key.clone()),
            AuthorizationData::AddRelayerPayingKey(
                user_key.clone(),
                paying_key.clone(),
                polyx_limit,
            ),
            None,
        );
        Self::deposit_event(RawEvent::AuthorizedPayingKey(
            from.for_event(),
            user_key,
            paying_key,
            polyx_limit,
            auth_id,
        ));
        auth_id
    }

    /// Check if the `key` has a valid CDD.
    fn key_has_valid_cdd(key: &T::AccountId) -> bool {
        if let Some(did) = <Identity<T>>::get_identity(key) {
            <Identity<T>>::has_valid_cdd(did)
        } else {
            false
        }
    }

    /// Ensure that `paying_key` is the paying key for `user_key`.
    fn ensure_is_paying_key(
        user_key: &T::AccountId,
        paying_key: &T::AccountId,
    ) -> Result<Subsidy<T::AccountId>, DispatchError> {
        // Check if the current paying key matches.
        match <Subsidies<T>>::get(user_key) {
            // There was no subsidy.
            None => fail!(Error::<T>::NoPayingKey),
            // Paying key doesn't match.
            Some(s) if s.paying_key != *paying_key => fail!(Error::<T>::NotPayingKey),
            Some(s) => Ok(s),
        }
    }

    /// Validate and accept a `paying_key` for the `user_key`.
    fn auth_accept_paying_key(
        user_did: IdentityId,
        from: IdentityId,
        user_key: T::AccountId,
        paying_key: T::AccountId,
        polyx_limit: Balance,
    ) -> DispatchResult {
        // Ensure that the authorization came from the DID of the paying_key.
        ensure!(
            <Identity<T>>::get_identity(&paying_key) == Some(from),
            Error::<T>::NotAuthorizedForPayingKey
        );

        // Ensure both user_key and paying_key have valid CDD.
        ensure!(
            <Identity<T>>::has_valid_cdd(user_did),
            Error::<T>::UserKeyCddMissing
        );
        ensure!(
            Self::key_has_valid_cdd(&paying_key),
            Error::<T>::PayingKeyCddMissing
        );

        // Remove existing subsidy for the user_key, if it exists.
        if let Some(subsidy) = <Subsidies<T>>::get(&user_key) {
            // Decrease old paying key usage.
            <Identity<T>>::remove_account_key_ref_count(&subsidy.paying_key);

            Self::deposit_event(RawEvent::RemovedPayingKey(
                user_did.for_event(),
                user_key.clone(),
                subsidy.paying_key,
            ));
        } else {
            // Increase user key usage.
            <Identity<T>>::add_account_key_ref_count(&user_key);
        }

        // Increase paying key usage.
        <Identity<T>>::add_account_key_ref_count(&paying_key);

        // All checks passed.
        <Subsidies<T>>::insert(
            user_key,
            Subsidy {
                paying_key,
                remaining: polyx_limit,
            },
        );

        Ok(())
    }

    fn ensure_pallet_is_subsidised(pallet: &[u8]) -> Result<Option<()>, InvalidTransaction> {
        match pallet {
            // These pallets are subsidised by the paying key.
            b"Asset" | b"ComplianceManager" | b"CorporateAction" | b"ExternalAgents"
            | b"Permissions" | b"Portfolio" | b"Settlement" | b"Statistics" | b"Sto" => {
                Ok(Some(()))
            }
            // The user key needs to pay for `remove_paying_key` call.
            b"Relayer" => Ok(None),
            // Reject all other pallets.
            _ => fail!(InvalidTransaction::Custom(
                TransactionError::PalletNotSubsidised as u8
            )),
        }
    }

    fn get_subsidy(
        user_key: &T::AccountId,
        fee: Balance,
    ) -> Result<Option<Subsidy<T::AccountId>>, InvalidTransaction> {
        // Get the Subsidy for `user_key`.
        match <Subsidies<T>>::get(user_key) {
            // There was no subsidy.
            None => Ok(None),
            // Has subsidy, but not enough remaining POLYX.
            Some(s) if s.remaining < fee => fail!(InvalidTransaction::Payment),
            // Has subsidy and enough POLYX.
            Some(s) => Ok(Some(s)),
        }
    }
}

impl<T: Config> SubsidiserTrait<T::AccountId> for Module<T> {
    fn check_subsidy(
        user_key: &T::AccountId,
        fee: Balance,
        pallet: Option<&[u8]>,
    ) -> Result<Option<T::AccountId>, InvalidTransaction> {
        match (Self::get_subsidy(user_key, fee)?, pallet) {
            (Some(s), Some(pallet)) => {
                // Ensure that the current pallet can be subsidised.
                Ok(Self::ensure_pallet_is_subsidised(pallet)?.map(|_| s.paying_key))
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
            <Subsidies<T>>::insert(user_key, subsidy);
            Ok(Some(paying_key))
        } else {
            Ok(None)
        }
    }
}
