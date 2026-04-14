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

use crate::{
    types, AccountKeyRefCount, Config, CurrentAuthId, DidKeys, DidRecords, Error, Event,
    IsDidFrozen, KeyAssetPermissions, KeyExtrinsicPermissions, KeyPortfolioPermissions, KeyRecords,
    MultiPurposeNonce, OffChainAuthorizationNonce, OutdatedAuthorizations, Pallet,
    PermissionedCallOriginData, RpcDidRecords,
};
use codec::Encode as _;
use frame_support::dispatch::DispatchResult;
use frame_support::ensure;
use frame_support::traits::{Currency as _, Get as _, Randomness as _};
use frame_system::ensure_signed;
use pallet_base::{ensure_custom_length_ok, ensure_custom_string_limited};
use pallet_permissions::{AccountCallPermissionsData, CheckAccountCallPermissions};
use polymesh_primitives::asset::AssetHolder;
use polymesh_primitives::constants::did::USER;
use polymesh_primitives::crypto::{ChainScopedMessage, IDENTITY_ADD_SECONDARY_KEY_LABEL};
use polymesh_primitives::identity::limits::{
    MAX_ASSETS, MAX_EXTRINSICS, MAX_PALLETS, MAX_PORTFOLIOS,
};
use polymesh_primitives::identity::SecondaryKeyWithAuth;
use polymesh_primitives::protocol_fee::{ChargeProtocolFee as _, ProtocolOp};
use polymesh_primitives::SystematicIssuers;
use polymesh_primitives::{
    extract_auth, AuthorizationData, DidRecord, ExtrinsicName, ExtrinsicPermissions, IdentityId,
    KeyRecord, PalletName, Permissions, SecondaryKey, Signatory,
};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::AccountIdConversion as _;
use sp_runtime::DispatchError;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::{vec, vec::Vec};

// Maximum secondary keys to return from RPC `identity_getDidRecords`.
const RPC_MAX_KEYS: usize = 200;
const MAX_NAME_LEN: usize = 60;

// Limit the maximum memory/cpu cost of a key's permissions.
const MAX_PERMISSION_COMPLEXITY: usize = 1_000_000;

impl<T: Config> Pallet<T> {
    pub fn ensure_no_id_record(id: IdentityId) -> DispatchResult {
        ensure!(!Self::is_did_active(id), Error::<T>::DidAlreadyExists);
        Ok(())
    }

    /// Returns `Err(DidDoesNotExist)` unless `id` has an associated record.
    pub fn ensure_id_record_exists(id: IdentityId) -> DispatchResult {
        ensure!(Self::is_did_active(id), Error::<T>::DidDoesNotExist);
        Ok(())
    }

    /// Returns the DID associated with `key`, if any,
    /// assuming it is either the primary key or isn't frozen.
    pub fn get_identity(key: &T::AccountId) -> Option<IdentityId> {
        match KeyRecords::<T>::get(key)? {
            KeyRecord::PrimaryKey(did) => Some(did),
            KeyRecord::SecondaryKey(did) if !IsDidFrozen::<T>::get(did) => Some(did),
            // Is a multisig signer, or frozen secondary key.
            _ => None,
        }
    }

    /// It checks if `key` is a secondary key of `did` identity.
    /// # IMPORTANT
    /// If secondary keys are frozen this function always returns false.
    /// A primary key cannot be frozen.
    pub fn is_key_authorized(did: IdentityId, key: &T::AccountId) -> bool {
        // `key_did` will be `None` if the key is frozen.
        let key_did = Self::get_identity(key);

        // Make sure the key's identity matches.
        key_did == Some(did)
    }

    /// It checks if `key` is a secondary key of `did` identity.
    pub fn is_secondary_key(did: IdentityId, key: &T::AccountId) -> bool {
        Self::ensure_secondary_key(did, key).is_ok()
    }

    /// Get the identity's primary key.
    pub fn get_primary_key(did: IdentityId) -> Option<T::AccountId> {
        DidRecords::<T>::get(did).and_then(|d| d.primary_key)
    }

    /// Use `did` as reference.
    pub fn is_primary_key(did: &IdentityId, key: &T::AccountId) -> bool {
        let primary_key = DidRecords::<T>::get(did).and_then(|d| d.primary_key);
        primary_key.as_ref() == Some(key)
    }

    /// Get the full permissions of a key.
    pub fn get_key_permissions(key: &T::AccountId) -> Permissions {
        Permissions {
            asset: KeyAssetPermissions::<T>::get(key).unwrap_or_default(),
            extrinsic: KeyExtrinsicPermissions::<T>::get(key).unwrap_or_default(),
            portfolio: KeyPortfolioPermissions::<T>::get(key).unwrap_or_default(),
        }
    }

    /// RPC call to fetch some aggregate account data for fewer round trips.
    pub fn get_key_identity_data(acc: T::AccountId) -> Option<types::KeyIdentityData<IdentityId>> {
        let (identity, permissions) = match KeyRecords::<T>::get(&acc)? {
            KeyRecord::PrimaryKey(did) => Some((did, None)),
            KeyRecord::SecondaryKey(did) => {
                let perms = Self::get_key_permissions(&acc);
                Some((did, Some(perms)))
            }
            // Is a multisig signer.
            _ => None,
        }?;
        Some(types::KeyIdentityData {
            identity,
            permissions,
        })
    }

    /// Check if the key is linked to an identity or MultiSig.
    /// (linked_to_did, linked_to_multsig)
    pub fn is_key_linked(acc: &T::AccountId) -> (bool, bool) {
        match KeyRecords::<T>::get(acc) {
            // Linked to an identity.
            Some(KeyRecord::PrimaryKey(_)) | Some(KeyRecord::SecondaryKey(_)) => (true, false),
            // Is a multisig signer.
            Some(KeyRecord::MultiSigSignerKey(_)) => (false, true),
            None => (false, false),
        }
    }

    /// Retrieve DidRecords for `did`
    ///
    /// Results limited to `RPC_MAX_KEYS` secondary keys.
    pub fn get_did_records(did: IdentityId) -> RpcDidRecords<T::AccountId> {
        if let Some(record) = DidRecords::<T>::get(&did) {
            let secondary_keys = DidKeys::<T>::iter_key_prefix(&did)
                .take(RPC_MAX_KEYS)
                .filter_map(|key| {
                    // Lookup the key's permissions and convert that into a `SecondaryKey` type.
                    KeyRecords::<T>::get(&key).and_then(|r| {
                        if r.is_secondary_key().is_some() {
                            Some(SecondaryKey {
                                permissions: Self::get_key_permissions(&key),
                                key,
                            })
                        } else {
                            None
                        }
                    })
                })
                .collect();
            RpcDidRecords::Success {
                primary_key: record.primary_key.unwrap_or_else(types::zero_account_id),
                secondary_keys,
            }
        } else {
            RpcDidRecords::IdNotFound
        }
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
    fn ensure_key_unlinkable_from_did(key: &T::AccountId) -> DispatchResult {
        ensure!(
            <AccountKeyRefCount<T>>::get(key) == 0,
            Error::<T>::AccountKeyIsBeingUsed
        );
        Ok(())
    }

    /// Ensure `key` isn't linked to a DID.
    pub fn ensure_key_did_unlinked(key: &T::AccountId) -> DispatchResult {
        ensure!(Self::can_add_key_record(key), Error::<T>::AlreadyLinked);
        Ok(())
    }

    /// Checks that a key doesn't already exists (i.e. not linked to an Identity or a MultiSig).
    pub fn can_add_key_record(key: &T::AccountId) -> bool {
        !KeyRecords::<T>::contains_key(key)
    }

    pub fn set_key_permissions(key: &T::AccountId, permissions: &Permissions) {
        // Update secondary key's permissions.
        KeyAssetPermissions::<T>::insert(key, &permissions.asset);
        KeyExtrinsicPermissions::<T>::insert(key, &permissions.extrinsic);
        KeyPortfolioPermissions::<T>::insert(key, &permissions.portfolio);
    }

    pub fn remove_key_permissions(key: &T::AccountId) {
        // Remove the key's permissions.
        KeyAssetPermissions::<T>::remove(key);
        KeyExtrinsicPermissions::<T>::remove(key);
        KeyPortfolioPermissions::<T>::remove(key);
    }

    /// Add a `KeyRecord` for an `AccountId` key, if it doesn't exist.
    ///
    /// The `key` can be:
    /// * An Identity's Primary key.  (The identity can only have one)
    /// * A Secondary key linked to an Identity.  (Can have multiple)
    /// * A signer key for a MultiSig account.
    ///
    /// This function applies the change if `can_add_key_record` returns `true`.
    /// Otherwise, it does nothing.
    pub fn add_key_record(key: &T::AccountId, record: KeyRecord<T::AccountId>) {
        if !KeyRecords::<T>::contains_key(key) {
            // `key` is not yet linked to any identity, so no constraints.
            KeyRecords::<T>::insert(key, &record);
            // For primary/secondary keys add to `DidKeys`.
            if let Some((did, is_primary_key)) = record.get_did_key_type() {
                DidKeys::<T>::insert(did, key, true);
                // For primary keys also set the DID record.
                if is_primary_key {
                    DidRecords::<T>::insert(did, DidRecord::new(key.clone()));
                }
            }
        }
    }

    /// Remove a key's record if the `did` matches.
    pub fn remove_key_record(key: &T::AccountId, did: Option<IdentityId>) {
        let remove_key = match KeyRecords::<T>::get(key) {
            Some(KeyRecord::PrimaryKey(did1)) if Some(did1) == did => {
                // `did` must match the key's `did`.
                DidRecords::<T>::mutate(did1, |d| {
                    match d {
                        Some(ref mut d) if d.primary_key.as_ref() == Some(key) => {
                            // Only clear the Identities primary key if it matches.
                            d.primary_key = None;
                        }
                        _ => (),
                    }
                });
                // Remove the key from the Identity's list of keys.
                DidKeys::<T>::remove(did1, key);
                true
            }
            Some(KeyRecord::SecondaryKey(did1)) if Some(did1) == did => {
                // Remove the secondary key's permissions.
                Self::remove_key_permissions(key);
                // `did` must match the key's `did`.
                // Remove the key from the Identity's list of keys.
                DidKeys::<T>::remove(did1, key);
                true
            }
            Some(KeyRecord::MultiSigSignerKey(_)) if did.is_none() => {
                // `did` must be `None` when removing a MultiSig signer key.
                true
            }
            Some(_) | None => false,
        };
        if remove_key {
            KeyRecords::<T>::remove(key);
        }
    }

    /// Accepts a primary key rotation.
    pub(crate) fn accept_primary_key_rotation(
        origin: T::RuntimeOrigin,
        rotation_auth_id: u64,
    ) -> DispatchResult {
        let sender = ensure_signed(origin)?;
        let signer = Signatory::Account(sender.clone());
        Self::accept_auth_with(&signer, rotation_auth_id, |data, target_did| {
            // Ensure Authorization is a `RotatePrimaryKey`.
            extract_auth!(data, RotatePrimaryKey);
            Self::common_rotate_primary_key(target_did, sender, None)
        })
    }

    // Sets the new primary key and optionally removes it as a secondary key if it is one.
    // Old primary key will be added as a secondary key if `new_permissions` is not None
    // New primary key must either be unlinked, or linked to the `target_did`
    pub fn common_rotate_primary_key(
        target_did: IdentityId,
        new_primary_key: T::AccountId,
        new_permissions: Option<Permissions>,
    ) -> DispatchResult {
        let old_primary_key =
            Self::get_primary_key(target_did).ok_or(Error::<T>::InvalidAccountKey)?;

        let key_record = KeyRecords::<T>::get(&new_primary_key);
        let (is_linked, is_secondary_key) = match key_record {
            Some(KeyRecord::PrimaryKey(_)) => {
                // Already linked as a primary key.
                (true, false)
            }
            Some(KeyRecord::SecondaryKey(did)) => {
                // Only allow if it is a secondary key of the `target_did`
                (true, did == target_did)
            }
            Some(KeyRecord::MultiSigSignerKey(_)) => {
                // MultiSig signer key can't be linked.
                (true, false)
            }
            None => {
                // Key is not linked.
                (false, false)
            }
        };
        ensure!((!is_linked || is_secondary_key), Error::<T>::AlreadyLinked);

        if new_permissions.is_none() {
            Self::ensure_key_unlinkable_from_did(&old_primary_key)?;
        }

        // Replace primary key of the owner that initiated key rotation.
        let key_record = KeyRecord::PrimaryKey(target_did);
        if is_secondary_key {
            // Convert secondary key to primary key.
            KeyRecords::<T>::insert(&new_primary_key, key_record);
            DidRecords::<T>::insert(target_did, DidRecord::new(new_primary_key.clone()));

            let removed_keys = vec![new_primary_key.clone()];
            Self::deposit_event(Event::SecondaryKeysRemoved(target_did, removed_keys));
        } else {
            Self::add_key_record(&new_primary_key, key_record);
        }
        Self::deposit_event(Event::PrimaryKeyUpdated(
            target_did,
            old_primary_key.clone(),
            new_primary_key,
        ));

        if let Some(perms) = new_permissions {
            // Convert old primary key to secondary key.
            KeyRecords::<T>::insert(&old_primary_key, KeyRecord::SecondaryKey(target_did));
            Self::set_key_permissions(&old_primary_key, &perms);

            let sk = SecondaryKey::new(old_primary_key, perms);
            Self::deposit_event(Event::SecondaryKeysAdded(target_did, vec![sk]));
        } else {
            Self::remove_key_record(&old_primary_key, Some(target_did));
        }
        Ok(())
    }

    /// Accepts a primary key rotation.
    /// Differs from accept_primary_key_rotation in that it will leave the old primary key as a
    /// secondary key with the permissions specified in the corresponding RotatePrimaryKeyToSecondary authorization
    /// instead of unlinking the primary key.
    pub(crate) fn base_rotate_primary_key_to_secondary(
        origin: T::RuntimeOrigin,
        rotation_auth_id: u64,
    ) -> DispatchResult {
        let new_primary_key = ensure_signed(origin)?;
        let new_primary_key_signer = Signatory::Account(new_primary_key.clone());
        Self::accept_auth_with(
            &new_primary_key_signer,
            rotation_auth_id,
            |data, target_did| {
                let perms = extract_auth!(data, RotatePrimaryKeyToSecondary(p));

                Self::common_rotate_primary_key(target_did, new_primary_key, Some(perms))
            },
        )
    }

    /// Set permissions for the specific `key`.
    /// Only the primary key of an identity is able to set secondary key permissions.
    pub(crate) fn base_set_secondary_key_permissions(
        origin: T::RuntimeOrigin,
        key: T::AccountId,
        permissions: Permissions,
    ) -> DispatchResult {
        let (_, did) = Self::ensure_primary_key(origin)?;

        // Ensure that the `key` is a secondary key of the caller's Identity
        Self::ensure_secondary_key(did, &key)?;

        Self::ensure_perms_length_limited(&permissions)?;

        // Get old permissions.
        let old_perms = Self::get_key_permissions(&key);
        // Update secondary key's permissions.
        Self::set_key_permissions(&key, &permissions);

        Self::deposit_event(Event::SecondaryKeyPermissionsUpdated(
            did,
            key.clone(),
            old_perms,
            permissions,
        ));
        Ok(())
    }

    /// Removes specified secondary keys of a DID if present.
    pub(crate) fn base_remove_secondary_keys(
        origin: T::RuntimeOrigin,
        keys: Vec<T::AccountId>,
    ) -> DispatchResult {
        let (_, did) = Self::ensure_primary_key(origin)?;

        // Ensure that it is safe to unlink the secondary keys from the did.
        for key in &keys {
            // Ensure that the key is a secondary key.
            Self::ensure_secondary_key(did, &key)?;
            // Ensure that the key can be unlinked.
            Self::ensure_key_unlinkable_from_did(key)?;
        }

        // Remove links and get all authorization IDs per signer.
        for key in &keys {
            // Unlink the secondary account key.
            Self::remove_key_record(key, Some(did));
            // Sets all authorizations for key as outdated (these will be deleted on_intialize)
            Self::set_outdated_autorizations(Signatory::Account(key.clone()));
        }

        Self::deposit_event(Event::SecondaryKeysRemoved(did, keys));
        Ok(())
    }

    /// Sets all authorizations with auth_id less or equal to the current id as invalid for the
    /// `signatory_account`.
    fn set_outdated_autorizations(signatory_account: Signatory<T::AccountId>) {
        let current_auth_id = CurrentAuthId::<T>::get();
        OutdatedAuthorizations::<T>::insert(signatory_account, current_auth_id);
    }

    /// Adds secondary keys to target identity `id`.
    /// Keys are directly added to identity because each of them has an authorization.
    pub(crate) fn base_add_secondary_keys_with_authorization(
        origin: T::RuntimeOrigin,
        keys: Vec<SecondaryKeyWithAuth<T::AccountId>>,
        expires_at: T::Moment,
    ) -> DispatchResult {
        let (_, did) = Self::ensure_primary_key(origin)?;

        // Charge the fee.
        T::ProtocolFee::batch_charge_fee(
            ProtocolOp::IdentityAddSecondaryKeysWithAuthorization,
            keys.len(),
        )?;

        // Update that identity's offchain authorization nonce.
        let nonce = OffChainAuthorizationNonce::<T>::mutate(did, |nonce| {
            let auth_nonce = *nonce;
            *nonce = auth_nonce + 1;
            auth_nonce
        });

        // Create authorization data that the keys need to sign.
        let authorization = ChainScopedMessage::<T, _>::new(
            nonce,
            IDENTITY_ADD_SECONDARY_KEY_LABEL,
            expires_at,
            did,
        )
        .ok_or(Error::<T>::AuthorizationExpired)?;

        // Verify signatures.
        let mut additional_keys_si = Vec::with_capacity(keys.len());
        let mut seen = BTreeSet::new();
        for si_with_auth in keys {
            let SecondaryKeyWithAuth {
                secondary_key,
                auth_signature,
            } = si_with_auth;

            // Check for duplicate keys.
            ensure!(!seen.contains(&secondary_key.key), Error::<T>::DuplicateKey);
            seen.insert(secondary_key.key.clone());

            Self::ensure_perms_length_limited(&secondary_key.permissions)?;

            // Constraint 1-to-1 account to DID.
            Self::ensure_key_did_unlinked(&secondary_key.key)?;

            // Verify the signature.
            ensure!(
                authorization.verify_any_signature(&secondary_key.key, auth_signature),
                Error::<T>::InvalidAuthorizationSignature
            );

            additional_keys_si.push(secondary_key);
        }

        // Link keys to identity
        for sk in &additional_keys_si {
            Self::add_key_record(&sk.key, KeyRecord::SecondaryKey(did));
            Self::set_key_permissions(&sk.key, &sk.permissions);
        }

        Self::deposit_event(Event::SecondaryKeysAdded(did, additional_keys_si));
        Ok(())
    }

    /// Accepts an auth to join an identity as a signer
    pub fn join_identity(origin: T::RuntimeOrigin, auth_id: u64) -> DispatchResult {
        let key = ensure_signed(origin)?;
        let signer = Signatory::Account(key.clone());
        Self::accept_auth_with(&signer, auth_id, |data, target_did| {
            let permissions = extract_auth!(data, JoinIdentity(p));
            // Not really needed unless we allow identities to be deleted.
            Self::ensure_id_record_exists(target_did)?;

            // Ensure that the key is unlinked.
            Self::ensure_key_did_unlinked(&key)?;

            ensure!(
                !Self::is_did_locked(target_did),
                Error::<T>::TargetDidInactive
            );

            // Charge the protocol fee after all checks.
            T::ProtocolFee::charge_fee(ProtocolOp::IdentityAddSecondaryKeysWithAuthorization)?;

            Self::unsafe_join_identity(target_did, permissions, key);
            Ok(())
        })
    }

    /// Joins a DID as an account based secondary key.
    pub fn unsafe_join_identity(
        target_did: IdentityId,
        permissions: Permissions,
        key: T::AccountId,
    ) {
        // Link the secondary key.
        Self::add_key_record(&key, KeyRecord::SecondaryKey(target_did));
        Self::set_key_permissions(&key, &permissions);

        let sk = SecondaryKey { key, permissions };
        Self::deposit_event(Event::SecondaryKeysAdded(target_did, vec![sk]));
    }

    pub(crate) fn leave_identity(origin: T::RuntimeOrigin) -> DispatchResult {
        let (key, did) = Self::ensure_did(origin)?;

        // Ensure that the caller is a secondary key.
        Self::ensure_secondary_key(did, &key)?;

        // Ensure that it is safe to unlink the account key from the did.
        Self::ensure_key_unlinkable_from_did(&key)?;

        // Unlink secondary key from the identity.
        Self::remove_key_record(&key, Some(did));

        Self::deposit_event(Event::SecondaryKeyLeftIdentity(did, key));
        Ok(())
    }

    /// Freezes/unfreezes the target `did` identity.
    ///
    /// # Errors
    /// Only primary key can freeze/unfreeze an identity.
    pub(crate) fn set_frozen_secondary_key_flags(
        origin: T::RuntimeOrigin,
        freeze: bool,
    ) -> DispatchResult {
        let (_, did) = Self::ensure_primary_key(origin)?;
        if freeze {
            IsDidFrozen::<T>::insert(&did, true);
            Self::deposit_event(Event::SecondaryKeysFrozen(did))
        } else {
            IsDidFrozen::<T>::remove(&did);
            Self::deposit_event(Event::SecondaryKeysUnfrozen(did));
        }
        Ok(())
    }

    /// Create a new DID from BABE randomness and a `nonce`.
    fn make_did() -> Result<IdentityId, DispatchError> {
        let nonce = MultiPurposeNonce::<T>::get() + 7u64;
        // Even if this transaction fails, nonce should be increased for added unpredictability of dids
        MultiPurposeNonce::<T>::put(&nonce);

        let (randomness, _) = T::Randomness::random(&nonce.encode());
        let did = IdentityId(blake2_256(&(USER, randomness, nonce).encode()));

        // Make sure there's no pre-existing entry for the DID
        // This should never happen but just being defensive here
        Self::ensure_no_id_record(did)?;

        Ok(did)
    }

    /// Registers a did without adding a CDD claim for it.
    pub fn register_did_without_cdd(
        sender: T::AccountId,
        secondary_keys: Vec<SecondaryKey<T::AccountId>>,
        protocol_fee_data: Option<ProtocolOp>,
    ) -> Result<IdentityId, DispatchError> {
        // Ensure primary key is not linked to any identity.
        Self::ensure_key_did_unlinked(&sender)?;
        // Check for duplicate secondary keys and ensure they are not the primary key.
        let mut seen = BTreeSet::new();
        for sk in &secondary_keys {
            // Ensure the key is not the primary key.
            ensure!(sk.key != sender, Error::<T>::SecondaryKeysContainPrimaryKey);
            // Ensure the key is not duplicated.
            ensure!(!seen.contains(&sk.key), Error::<T>::DuplicateKey);
            seen.insert(sk.key.clone());
        }

        // Create a new identity.
        let did = Self::make_did()?;

        // Charge the given fee.
        if let Some(op) = protocol_fee_data {
            T::ProtocolFee::charge_fee(op)?;
        }

        // Link the primary key.
        Self::add_key_record(&sender, KeyRecord::PrimaryKey(did));

        // Give `InitialPOLYX` to the primary key for testing.
        let _ = T::Balances::deposit_creating(&sender, T::InitialPOLYX::get());
        Self::deposit_event(Event::DidCreated(did, sender, secondary_keys.clone()));

        // Add join identity authorizations for secondary keys.
        for sk in secondary_keys {
            let signer = Signatory::Account(sk.key.clone());
            let data = AuthorizationData::JoinIdentity(sk.permissions.clone());
            Self::add_auth(did, signer, data, None)?;
        }
        Ok(did)
    }

    /// For testing/benchmarking only.
    /// Registers a DID.
    //#[cfg(feature = "runtime-benchmarks")]
    pub fn testing_register_did(sender: T::AccountId) -> Result<IdentityId, DispatchError> {
        Self::register_did_without_cdd(sender, vec![], None)
    }

    /// Registers the systematic issuer with its DID.
    pub(crate) fn register_systematic_id(issuer: SystematicIssuers) {
        let acc = issuer.as_pallet_id().into_account_truncating();
        let id = issuer.as_id();
        log::info!(
            "Register Systematic id {} with account {:?} as {}",
            issuer,
            acc,
            id
        );

        Self::do_register_id(acc, id, vec![]);
    }

    /// Registers `primary_key` as `id` identity.
    pub(crate) fn do_register_id(
        primary_key: T::AccountId,
        id: IdentityId,
        secondary_keys: Vec<SecondaryKey<T::AccountId>>,
    ) {
        // Link primary key.
        <Pallet<T>>::add_key_record(&primary_key, KeyRecord::PrimaryKey(id));
        // Link secondary keys.
        for sk in &secondary_keys {
            Self::add_key_record(&sk.key, KeyRecord::SecondaryKey(id));
            Self::set_key_permissions(&sk.key, &sk.permissions);
        }

        Self::deposit_event(Event::DidCreated(id, primary_key, secondary_keys));
    }

    /// Ensure the `key` is a secondary key of the identity `did`.
    fn ensure_secondary_key(did: IdentityId, key: &T::AccountId) -> DispatchResult {
        let key_did = KeyRecords::<T>::get(key).and_then(|rec| rec.is_secondary_key());
        ensure!(key_did == Some(did), Error::<T>::NotASigner);
        Ok(())
    }

    /// Ensures that `origin`'s key is the primary key of a DID that exists.
    /// Returns the caller's account and DID.
    pub fn ensure_primary_key(
        origin: T::RuntimeOrigin,
    ) -> Result<(T::AccountId, IdentityId), DispatchError> {
        let sender = ensure_signed(origin)?;
        let key_rec = KeyRecords::<T>::get(&sender)
            .ok_or(pallet_permissions::Error::<T>::UnauthorizedCaller)?;
        let did = key_rec.is_primary_key().ok_or(Error::<T>::KeyNotAllowed)?;
        ensure!(
            !Self::is_did_locked(did),
            Error::<T>::UnauthorizedCallerDidInactive
        );
        Ok((sender, did))
    }

    /// Ensures that `origin`'s key is linked to a DID that exists.
    /// Returns the caller's account and DID.
    pub fn ensure_did(
        origin: T::RuntimeOrigin,
    ) -> Result<(T::AccountId, IdentityId), DispatchError> {
        let sender = ensure_signed(origin)?;
        let did = Self::get_identity(&sender).ok_or(Error::<T>::MissingIdentity)?;
        ensure!(
            !Self::is_did_locked(did),
            Error::<T>::UnauthorizedCallerDidInactive
        );
        Ok((sender, did))
    }

    /// Checks call permissions and, if successful, returns the caller's account, primary and secondary identities.
    pub fn ensure_origin_call_permissions(
        origin: T::RuntimeOrigin,
    ) -> Result<PermissionedCallOriginData<T::AccountId>, DispatchError> {
        let sender = ensure_signed(origin)?;
        let AccountCallPermissionsData {
            primary_did,
            secondary_key,
        } = pallet_permissions::Pallet::<T>::ensure_call_permissions(&sender)?;
        Ok(PermissionedCallOriginData {
            sender,
            primary_did,
            secondary_key,
        })
    }

    /// Ensure `origin` is signed and permissioned for this call, returning its DID.
    pub fn ensure_perms(origin: T::RuntimeOrigin) -> Result<IdentityId, DispatchError> {
        Self::ensure_origin_call_permissions(origin).map(|x| x.primary_did)
    }

    /// Ensures length limits are enforced in `perms`.
    pub fn ensure_perms_length_limited(perms: &Permissions) -> DispatchResult {
        ensure_custom_length_ok::<T>(perms.complexity(), MAX_PERMISSION_COMPLEXITY)?;
        ensure_custom_length_ok::<T>(perms.asset.complexity(), MAX_ASSETS)?;
        ensure_custom_length_ok::<T>(perms.portfolio.complexity(), MAX_PORTFOLIOS)?;
        Self::ensure_no_except_perms(&perms.extrinsic)?;
        Self::ensure_extrinsic_perms_length_limited(&perms.extrinsic)
    }

    // Ensures that extrinsic permissions do not use the Except variant
    // This is considered unsafe since extrinsic names can change or be replaced with newer versions
    pub fn ensure_no_except_perms(perms: &ExtrinsicPermissions) -> DispatchResult {
        if !perms.check_no_except_perms() {
            return Err(Error::<T>::ExceptNotAllowedForExtrinsics.into());
        }
        Ok(())
    }

    /// Ensures length limits are enforced in `perms`.
    pub fn ensure_extrinsic_perms_length_limited(perms: &ExtrinsicPermissions) -> DispatchResult {
        if let Some(set) = perms.inner() {
            ensure_custom_length_ok::<T>(set.len(), MAX_PALLETS)?;
            for (name, elem) in set {
                ensure_custom_string_limited::<T>(name.as_bytes(), MAX_NAME_LEN)?;
                if let Some(set) = elem.extrinsics.inner() {
                    ensure_custom_length_ok::<T>(set.len(), MAX_EXTRINSICS)?;
                    for elem in set {
                        ensure_custom_string_limited::<T>(elem.as_bytes(), MAX_NAME_LEN)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Checks if the caller is permissioned to call the current extrinsic skipping CDD checks.
    /// If `must_be_primary_key` ensures that the caller is a primary key.
    pub fn ensure_valid_origin(
        origin: T::RuntimeOrigin,
        must_be_primary_key: bool,
    ) -> Result<(T::AccountId, IdentityId), DispatchError> {
        let caller_acc = ensure_signed(origin)?;
        let account_data = pallet_permissions::Pallet::<T>::ensure_valid_origin_permissions(
            &caller_acc,
            must_be_primary_key,
        )?;
        Ok((caller_acc, account_data.primary_did))
    }

    /// If [`AssetHolder::Portfolio`], returns [`IdentityId`] directly from [`PortfolioId::did`].
    /// If [`AssetHolder::Account`], looks up the account's key record and returns the linked IdentityId if it exists.
    pub fn asset_holder_did(asset_holder: &AssetHolder) -> Result<IdentityId, DispatchError> {
        match asset_holder {
            AssetHolder::Portfolio(portfolio_id) => Ok(portfolio_id.did),
            AssetHolder::Account(acc_id) => {
                if let Ok(acc_id) = pallet_base::pallet_account_id::<T>(acc_id) {
                    if let Some(key_record) = KeyRecords::<T>::get(acc_id) {
                        if let Some(did) = key_record.as_did() {
                            return Ok(did);
                        }
                    }
                }
                return Err(Error::<T>::IdentityNotFoundForAccountPortfolio.into());
            }
        }
    }
}

impl<T: Config> CheckAccountCallPermissions<T::AccountId> for Pallet<T> {
    fn check_account_call_permissions(
        who: &T::AccountId,
        pallet_name: impl FnOnce() -> PalletName,
        function_name: impl FnOnce() -> ExtrinsicName,
    ) -> Result<AccountCallPermissionsData<T::AccountId>, DispatchError> {
        let account_call_permissions_data =
            Self::ensure_valid_origin_permissions(who, false, pallet_name, function_name)?;

        ensure!(
            !Self::is_did_locked(account_call_permissions_data.primary_did),
            Error::<T>::UnauthorizedCallerDidInactive
        );

        Ok(account_call_permissions_data)
    }

    fn ensure_valid_origin_permissions(
        caller_acc: &T::AccountId,
        must_be_primary_key: bool,
        pallet_name: impl FnOnce() -> PalletName,
        function_name: impl FnOnce() -> ExtrinsicName,
    ) -> Result<AccountCallPermissionsData<T::AccountId>, DispatchError> {
        let key_record = KeyRecords::<T>::get(&caller_acc).ok_or(Error::<T>::MissingIdentity)?;

        if must_be_primary_key {
            let did = key_record
                .is_primary_key()
                .ok_or(Error::<T>::KeyNotAllowed)?;
            return Ok(AccountCallPermissionsData::new(did, None));
        }

        if let KeyRecord::PrimaryKey(did) = key_record {
            return Ok(AccountCallPermissionsData::new(did, None));
        }

        let did = key_record
            .is_secondary_key()
            .ok_or(Error::<T>::KeyNotAllowed)?;

        ensure!(
            !IsDidFrozen::<T>::get(&did),
            Error::<T>::UnauthorizedCallerFrozenDid
        );

        let permissions = Self::get_key_permissions(&caller_acc);
        ensure!(
            permissions
                .extrinsic
                .sufficient_for(&pallet_name(), &function_name()),
            Error::<T>::UnauthorizedCallerMissingPermissions
        );
        let sk = SecondaryKey::new(caller_acc.clone(), permissions);

        Ok(AccountCallPermissionsData::new(did, Some(sk)))
    }
}
