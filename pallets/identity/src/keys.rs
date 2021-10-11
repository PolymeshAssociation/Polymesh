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

use crate::{AccountKeyRefCount, Config, DidRecords, Error, KeyToIdentityIds, Module};
use frame_support::dispatch::DispatchResult;
use frame_support::{ensure, StorageMap as _};
use polymesh_common_utilities::multisig::MultiSigSubTrait as _;
use polymesh_primitives::{IdentityId, Signatory};

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
}
