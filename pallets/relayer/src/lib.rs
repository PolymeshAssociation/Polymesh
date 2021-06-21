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
//! TODO: Add PolyLimit
//! TODO: Add support for `AuthorizationData::AddRelayerPayingKey` to `CddAndFeeDetails` in `pallets/runtime/*/src/fee_details.rs`

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::{decl_error, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use pallet_identity::{self as identity, PermissionedCallOriginData};
pub use polymesh_common_utilities::traits::relayer::{
    Config, Event, IdentityToRelayer, RawEvent, WeightInfo,
};
use polymesh_primitives::{AuthorizationData, IdentityId, Signatory};

type Identity<T> = identity::Module<T>;

decl_storage! {
    trait Store for Module<T: Config> as Relayer {
        /// map `user_key` to `paying_key`
        pub PayingKeys get(fn paying_keys):
            map hasher(blake2_128_concat) T::AccountId => T::AccountId;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Set paying key for `user_key`
        /// TODO: Add docs.
        #[weight = <T as Config>::WeightInfo::set_paying_key()]
        pub fn set_paying_key(origin, user_key: T::AccountId) -> DispatchResult {
            Self::base_set_paying_key(origin, user_key)
        }

        /// Accept paying key for `origin = user_key`
        /// TODO: Add docs.
        #[weight = <T as Config>::WeightInfo::accept_paying_key()]
        pub fn accept_paying_key(origin, auth_id: u64) -> DispatchResult {
            Self::base_accept_paying_key(origin, auth_id)
        }

        /// Remove paying key for `user_key`
        /// TODO: Add docs.
        #[weight = <T as Config>::WeightInfo::remove_paying_key()]
        pub fn remove_paying_key(origin, user_key: T::AccountId, paying_key: T::AccountId) -> DispatchResult {
            Self::base_remove_paying_key(origin, user_key, paying_key)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The `user_key` is not attached to a CDD'd identity.
        UserKeyCddMissing,
        /// The `user_key` is not attached to a CDD'd identity.
        PayingKeyCddMissing,
        /// The `user_key` already has a `paying_key`.
        AlreadyHasPayingKey,
        /// The `user_key` doesn't have a `paying_key`.
        NoPayingKey,
        /// The `user_key` has a different `paying_key`.
        NotPayingKey,
        /// Not a relayer authorization.
        NotARelayerAuth,
        /// The signer not authorized for `paying_key`.
        NotAuthorizedForPayingKey,
        /// The signer not authorized for `user_key`.
        NotAuthorizedForUserKey,
    }
}

impl<T: Config> Module<T> {
    fn base_set_paying_key(origin: T::Origin, user_key: T::AccountId) -> DispatchResult {
        let PermissionedCallOriginData {
            sender: paying_key,
            primary_did: paying_did,
            ..
        } = <Identity<T>>::ensure_origin_call_permissions(origin)?;

        // Create authorization for setting the `paying_key` to the `user_key`
        Self::unsafe_add_auth_for_paying_key(paying_did, user_key, paying_key);
        Ok(())
    }

    fn base_accept_paying_key(origin: T::Origin, auth_id: u64) -> DispatchResult {
        let PermissionedCallOriginData {
            sender: user_key,
            ..
        } = <Identity<T>>::ensure_origin_call_permissions(origin)?;
        let signer = Signatory::Account(user_key);

        <Identity<T>>::accept_auth_with(&signer, auth_id, |data, auth_by| -> DispatchResult {
            let (user_key, paying_key) = match data {
                AuthorizationData::AddRelayerPayingKey(user_key, paying_key) => {
                    Ok((user_key, paying_key))
                }
                _ => Err(Error::<T>::NotARelayerAuth),
            }?;

            Self::auth_accept_paying_key(signer.clone(), auth_by, user_key, paying_key)
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

        // allow: `origin == user_key` or `origin == paying_key`
        if sender != user_key && sender != paying_key {
          // allow: `origin == primary key of user_key's identity`
          ensure!(
              <Identity<T>>::get_identity(&user_key) == Some(sender_did),
              Error::<T>::NotAuthorizedForUserKey
          );
        }

        // Check if there is a paying key.
        ensure!(
            <PayingKeys<T>>::contains_key(&user_key),
            Error::<T>::NoPayingKey
        );

        // Check if the current paying key matches.
        ensure!(
            <PayingKeys<T>>::get(&user_key) == paying_key,
            Error::<T>::NotPayingKey
        );

        // Remove paying key for user key.
        <PayingKeys<T>>::remove(&user_key);

        Ok(())
    }

    /// Adds an authorization to add a `paying_key` to the `user_key`
    pub fn unsafe_add_auth_for_paying_key(
        from: IdentityId,
        user_key: T::AccountId,
        paying_key: T::AccountId,
    ) -> u64 {
        let auth_id = <Identity<T>>::add_auth(
            from,
            Signatory::Account(user_key.clone()),
            AuthorizationData::AddRelayerPayingKey(user_key.clone(), paying_key.clone()),
            None,
        );
        Self::deposit_event(RawEvent::PayingKeyAuthorized(
            from.for_event(),
            user_key,
            paying_key,
            auth_id,
        ));
        auth_id
    }

    fn key_has_valid_cdd(key: &T::AccountId) -> bool {
        if let Some(did) = <Identity<T>>::get_identity(key) {
            <Identity<T>>::has_valid_cdd(did)
        } else {
            false
        }
    }

    fn ensure_set_paying_key(
        from: IdentityId,
        user_key: &T::AccountId,
        paying_key: &T::AccountId,
    ) -> DispatchResult {
        ensure!(
            <Identity<T>>::get_identity(paying_key) == Some(from),
            Error::<T>::NotAuthorizedForPayingKey
        );

        ensure!(
            Self::key_has_valid_cdd(user_key),
            Error::<T>::UserKeyCddMissing
        );
        ensure!(
            Self::key_has_valid_cdd(paying_key),
            Error::<T>::PayingKeyCddMissing
        );
        ensure!(
            !<PayingKeys<T>>::contains_key(user_key),
            Error::<T>::AlreadyHasPayingKey
        );

        Ok(())
    }
}

impl<T: Config> IdentityToRelayer<T::AccountId> for Module<T> {
    fn auth_accept_paying_key(
        signer: Signatory<T::AccountId>,
        from: IdentityId,
        user_key: T::AccountId,
        paying_key: T::AccountId,
    ) -> DispatchResult {
        // Check `signer` is DID/Key of `user_key`
        ensure!(
            match signer {
                Signatory::Account(signer_key) => (signer_key == user_key),
                Signatory::Identity(signer_did) => {
                    <Identity<T>>::get_identity(&user_key) == Some(signer_did)
                }
            },
            Error::<T>::NotAuthorizedForUserKey
        );

        // ensure that we can still set the paying key
        Self::ensure_set_paying_key(from, &user_key, &paying_key)?;

        ensure!(
            !<PayingKeys<T>>::contains_key(&user_key),
            Error::<T>::AlreadyHasPayingKey
        );

        <PayingKeys<T>>::insert(user_key, paying_key);
        Ok(())
    }
}
