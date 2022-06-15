// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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

use crate::{
    AuthorizationType, Authorizations, AuthorizationsGiven, Config, Error, KeyRecords, Module,
    MultiPurposeNonce, RawEvent,
};
use frame_support::dispatch::DispatchResult;
use frame_support::{ensure, StorageDoubleMap, StorageMap, StorageValue};
use frame_system::ensure_signed;
use polymesh_common_utilities::Context;
use polymesh_primitives::{
    Authorization, AuthorizationData, AuthorizationError, IdentityId, Signatory,
};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

impl<T: Config> Module<T> {
    /// Adds an authorization.
    crate fn base_add_authorization(
        origin: T::Origin,
        target: Signatory<T::AccountId>,
        authorization_data: AuthorizationData<T::AccountId>,
        expiry: Option<T::Moment>,
    ) -> Result<u64, DispatchError> {
        let from_did = Self::ensure_perms(origin)?;
        if let AuthorizationData::JoinIdentity(perms)
        | AuthorizationData::RotatePrimaryKeyToSecondary(perms) = &authorization_data
        {
            Self::ensure_perms_length_limited(perms)?;
        }
        Ok(Self::add_auth(from_did, target, authorization_data, expiry))
    }

    /// Adds an authorization.
    pub fn add_auth(
        from: IdentityId,
        target: Signatory<T::AccountId>,
        authorization_data: AuthorizationData<T::AccountId>,
        expiry: Option<T::Moment>,
    ) -> u64 {
        let new_nonce = Self::multi_purpose_nonce() + 1u64;
        MultiPurposeNonce::put(&new_nonce);

        let auth = Authorization {
            authorization_data: authorization_data.clone(),
            authorized_by: from,
            expiry,
            auth_id: new_nonce,
            count: 50,
        };

        <Authorizations<T>>::insert(target.clone(), new_nonce, auth);
        <AuthorizationsGiven<T>>::insert(from, new_nonce, target.clone());

        // This event is split in order to help the event harvesters.
        Self::deposit_event(RawEvent::AuthorizationAdded(
            from,
            target.as_identity().cloned(),
            target.as_account().cloned(),
            new_nonce,
            authorization_data,
            expiry,
        ));

        new_nonce
    }

    /// Removes an authorization.
    crate fn base_remove_authorization(
        origin: T::Origin,
        target: Signatory<T::AccountId>,
        auth_id: u64,
    ) -> DispatchResult {
        let sender = ensure_signed(origin)?;
        let from_did = if <KeyRecords<T>>::contains_key(&sender) {
            // If the sender is linked to an identity, ensure that it has relevant permissions
            pallet_permissions::Module::<T>::ensure_call_permissions(&sender)?.primary_did
        } else {
            Context::current_identity_or::<Self>(&sender)?
        };

        let auth = Self::ensure_authorization(&target, auth_id)?;
        let revoked = auth.authorized_by == from_did;
        ensure!(
            revoked || target.eq_either(&from_did, &sender),
            Error::<T>::Unauthorized
        );
        Self::unsafe_remove_auth(&target, auth_id, &auth.authorized_by, revoked);
        Ok(())
    }

    /// Removes any authorization. No questions asked.
    /// NB: Please do all the required checks before calling this function.
    crate fn unsafe_remove_auth(
        target: &Signatory<T::AccountId>,
        auth_id: u64,
        authorizer: &IdentityId,
        revoked: bool,
    ) {
        <Authorizations<T>>::remove(target, auth_id);
        <AuthorizationsGiven<T>>::remove(authorizer, auth_id);
        let id = target.as_identity().cloned();
        let acc = target.as_account().cloned();
        let event = if revoked {
            RawEvent::AuthorizationRevoked
        } else {
            RawEvent::AuthorizationRejected
        };
        Self::deposit_event(event(id, acc, auth_id))
    }

    /// Returns an iterator over all authorizations belonging to `signer` and authorized by `did`.
    crate fn auths_of(
        signer: &Signatory<T::AccountId>,
        did: IdentityId,
    ) -> impl Iterator<Item = u64> {
        <Authorizations<T>>::iter_prefix_values(signer)
            .filter_map(move |auth| (auth.authorized_by == did).then_some(auth.auth_id))
    }

    /// Use to get the filtered authorization data for a given signatory
    /// - if auth_type is None then return authorizations data on the basis of the `allow_expired` boolean
    /// - if auth_type is Some(value) then return filtered authorizations on the value basis type in conjunction
    ///   with `allow_expired` boolean condition
    pub fn get_filtered_authorizations(
        signatory: Signatory<T::AccountId>,
        allow_expired: bool,
        auth_type: Option<AuthorizationType>,
    ) -> Vec<Authorization<T::AccountId, T::Moment>> {
        let now = <pallet_timestamp::Pallet<T>>::get();
        let auths = <Authorizations<T>>::iter_prefix_values(signatory)
            .filter(|auth| allow_expired || auth.expiry.filter(|&e| e < now).is_none());
        if let Some(auth_type) = auth_type {
            auths
                .filter(|auth| auth.authorization_data.auth_type() == auth_type)
                .collect()
        } else {
            auths.collect()
        }
    }

    /// Returns an auth id if it is present and not expired.
    pub fn get_non_expired_auth(
        target: &Signatory<T::AccountId>,
        auth_id: &u64,
    ) -> Option<Authorization<T::AccountId, T::Moment>> {
        Self::authorizations(target, *auth_id).filter(|auth| {
            auth.expiry
                .filter(|&expiry| <pallet_timestamp::Pallet<T>>::get() > expiry)
                .is_none()
        })
    }

    /// Given that `auth_by` is the DID that issued an authorization,
    /// ensure that it matches `from`, or otherwise return an error.
    pub fn ensure_auth_by(auth_by: IdentityId, from: IdentityId) -> DispatchResult {
        ensure!(auth_by == from, AuthorizationError::Unauthorized);
        Ok(())
    }

    /// Accepts an authorization `auth_id` as `target`,
    /// executing `accepter` for case-specific additional validation and storage changes.
    pub fn accept_auth_with(
        target: &Signatory<T::AccountId>,
        auth_id: u64,
        accepter: impl FnOnce(AuthorizationData<T::AccountId>, IdentityId) -> DispatchResult,
    ) -> DispatchResult {
        // Extract authorization.
        let mut auth = Self::ensure_authorization(target, auth_id)?;

        // Ensure that `auth.expiry`, if provided, is in the future.
        if let Some(expiry) = auth.expiry {
            let now = <pallet_timestamp::Pallet<T>>::get();
            ensure!(expiry > now, AuthorizationError::Expired);
        }

        // Run custom per-type validation and updates.
        let res = accepter(auth.authorization_data.clone(), auth.authorized_by.clone());

        if res.is_err() {
            // decrement
            auth.count = auth.count.saturating_sub(1);

            // check if count is zero
            if auth.count == 0 {
                <Authorizations<T>>::remove(&target, auth_id);
                <AuthorizationsGiven<T>>::remove(auth.authorized_by.clone(), auth_id);
                Self::deposit_event(RawEvent::AuthorizationRetryLimitReached(
                    target.as_identity().cloned(),
                    target.as_account().cloned(),
                    auth_id,
                ));
            } else {
                // update authorization
                <Authorizations<T>>::insert(&target, auth_id, auth);
            }

            // return error
            return res;
        }

        // Remove authorization from storage and emit event.
        <Authorizations<T>>::remove(&target, auth_id);
        <AuthorizationsGiven<T>>::remove(auth.authorized_by, auth_id);
        Self::deposit_event(RawEvent::AuthorizationConsumed(
            target.as_identity().cloned(),
            target.as_account().cloned(),
            auth_id,
        ));
        Ok(())
    }

    /// Return and ensure that there's an authorization `auth_id` for `target`.
    fn ensure_authorization(
        target: &Signatory<T::AccountId>,
        auth_id: u64,
    ) -> Result<Authorization<T::AccountId, T::Moment>, DispatchError> {
        Self::authorizations(target, auth_id).ok_or_else(|| AuthorizationError::Invalid.into())
    }
}
