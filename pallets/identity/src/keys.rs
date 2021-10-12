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

use crate::{
    types, AccountKeyRefCount, Config, DidRecord, DidRecords, Error, KeyToIdentityIds, Module,
    PermissionedCallOriginData, RpcDidRecords,
};
use frame_support::dispatch::DispatchResult;
use frame_support::{ensure, StorageMap as _};
use polymesh_common_utilities::multisig::MultiSigSubTrait as _;
use polymesh_primitives::{IdentityId, Permissions, SecondaryKey, Signatory};
use sp_runtime::DispatchError;

impl<T: Config> Module<T> {
    /// It checks if `key` is a secondary key of `did` identity.
    /// # IMPORTANT
    /// If secondary keys are frozen this function always returns false.
    /// A primary key cannot be frozen.
    pub fn is_key_authorized(did: IdentityId, key: &T::AccountId) -> bool {
        let record = <DidRecords<T>>::get(did);

        // Check primary id or key.
        &record.primary_key == key
            // Check secondary items if DID is not frozen.
            || !Self::is_did_frozen(did) && record.secondary_keys.iter().any(|si| si.signer.as_account().contains(&key))
    }

    /// It checks if `key` is a secondary key of `did` identity.
    pub fn is_signer(did: IdentityId, signer: &Signatory<T::AccountId>) -> bool {
        let record = <DidRecords<T>>::get(did);
        record.secondary_keys.iter().any(|si| si.signer == *signer)
    }

    /// Use `did` as reference.
    pub fn is_primary_key(did: &IdentityId, key: &T::AccountId) -> bool {
        key == &<DidRecords<T>>::get(did).primary_key
    }

    /// RPC call to fetch some aggregate account data for fewer round trips.
    pub fn get_key_identity_data(acc: T::AccountId) -> Option<types::KeyIdentityData<IdentityId>> {
        let identity = Self::get_identity(&acc)?;
        let record = <DidRecords<T>>::get(identity);
        let permissions = if acc == record.primary_key {
            None
        } else {
            Some(record.secondary_keys.into_iter().find_map(|sk| {
                sk.signer.as_account().filter(|&a| a == &acc)?;
                Some(sk.permissions)
            })?)
        };
        Some(types::KeyIdentityData {
            identity,
            permissions,
        })
    }

    /// Retrieve DidRecords for `did`
    pub fn get_did_records(
        did: IdentityId,
    ) -> RpcDidRecords<T::AccountId, SecondaryKey<T::AccountId>> {
        if let Some(record) = Self::identity_record_of(did) {
            RpcDidRecords::Success {
                primary_key: record.primary_key,
                secondary_keys: record.secondary_keys,
            }
        } else {
            RpcDidRecords::IdNotFound
        }
    }

    /// Return the record of `did` and ensure that `sender` is the primary key of it.
    crate fn grant_check_only_primary_key(
        sender: &T::AccountId,
        did: IdentityId,
    ) -> Result<DidRecord<T::AccountId>, DispatchError> {
        Self::ensure_id_record_exists(did)?;
        let record = <DidRecords<T>>::get(did);
        ensure!(*sender == record.primary_key, Error::<T>::KeyNotAllowed);
        Ok(record)
    }

    /// Increment the reference counter for `key`.
    pub fn add_account_key_ref_count(key: &T::AccountId) {
        <AccountKeyRefCount<T>>::mutate(key, |n| *n = n.saturating_add(1_u64));
    }

    /// Decrement the reference counter for `key`.
    pub fn remove_account_key_ref_count(key: &T::AccountId) {
        <AccountKeyRefCount<T>>::mutate(key, |n| *n = n.saturating_sub(1_u64));
    }

    /// Ensure that the account key is safe to unlink from it's identity.
    crate fn ensure_key_unlinkable_from_did(key: &T::AccountId) -> DispatchResult {
        ensure!(
            <AccountKeyRefCount<T>>::get(key) == 0,
            Error::<T>::AccountKeyIsBeingUsed
        );
        Ok(())
    }

    /// Ensure `key` isn't linked to a DID.
    pub fn ensure_key_did_unlinked(key: &T::AccountId) -> DispatchResult {
        ensure!(
            Self::can_link_account_key_to_did(key),
            Error::<T>::AlreadyLinked
        );
        Ok(())
    }

    /// Checks that a key is not linked to any identity or multisig.
    pub fn can_link_account_key_to_did(key: &T::AccountId) -> bool {
        !<KeyToIdentityIds<T>>::contains_key(key) && !T::MultiSig::is_signer(key)
    }

    /// Links a primary or secondary `AccountId` key `key` to an identity `did`.
    ///
    /// This function applies the change if `can_link_account_key_to_did` returns `true`.
    /// Otherwise, it does nothing.
    pub fn link_account_key_to_did(key: &T::AccountId, did: IdentityId) {
        if !<KeyToIdentityIds<T>>::contains_key(key) {
            // `key` is not yet linked to any identity, so no constraints.
            <KeyToIdentityIds<T>>::insert(key, did);
        }
    }

    /// Unlinks an `AccountId` key `key` from an identity `did`.
    crate fn unlink_account_key_from_did(key: &T::AccountId, did: IdentityId) {
        if <KeyToIdentityIds<T>>::contains_key(key) && <KeyToIdentityIds<T>>::get(key) == did {
            <KeyToIdentityIds<T>>::remove(key)
        }
    }

    /// Set permissions for the specific `target_key`.
    /// Only the primary key of an identity is able to set secondary key permissions.
    crate fn base_set_permission_to_signer(
        origin: T::Origin,
        signer: Signatory<T::AccountId>,
        permissions: Permissions,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            sender,
            primary_did: did,
            ..
        } = Self::ensure_origin_call_permissions(origin)?;
        let record = Self::grant_check_only_primary_key(&sender, did)?;

        // Ensure that the signer is a secondary key of the caller's Identity
        ensure!(
            record.secondary_keys.iter().any(|si| si.signer == signer),
            Error::<T>::NotASigner
        );
        Self::update_secondary_key_permissions(did, &signer, permissions)
    }
}
