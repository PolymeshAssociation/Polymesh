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
    types, AccountKeyRefCount, Config, DidKeys, DidRecords, Error, IsDidFrozen, KeyRecords, Module,
    MultiPurposeNonce, OffChainAuthorizationNonce, PermissionedCallOriginData, RawEvent,
    RpcDidRecords,
};
use codec::{Decode, Encode as _};
use core::mem;
use frame_support::dispatch::DispatchResult;
use frame_support::traits::{Currency as _, Get as _};
use frame_support::{
    ensure, IterableStorageDoubleMap, StorageDoubleMap, StorageMap as _, StorageValue as _,
};
use frame_system::ensure_signed;
use pallet_base::{ensure_custom_length_ok, ensure_custom_string_limited};
use polymesh_common_utilities::constants::did::USER;
use polymesh_common_utilities::group::GroupTrait;
use polymesh_common_utilities::identity::{SecondaryKeyWithAuth, TargetIdAuthorization};
use polymesh_common_utilities::multisig::MultiSigSubTrait as _;
use polymesh_common_utilities::protocol_fee::{ChargeProtocolFee as _, ProtocolOp};
use polymesh_common_utilities::traits::{
    AccountCallPermissionsData, CddAndFeeDetails, CheckAccountCallPermissions,
};
use polymesh_common_utilities::{Context, SystematicIssuers};
use polymesh_primitives::{
    extract_auth, AuthorizationData, DidRecord, DispatchableName, ExtrinsicPermissions, IdentityId,
    KeyRecord, PalletName, Permissions, SecondaryKey, Signatory,
};
use sp_core::sr25519::Signature;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{AccountIdConversion as _, IdentifyAccount, Verify};
use sp_runtime::{AnySignature, DispatchError};
use sp_std::{vec, vec::Vec};

// Maximum secondary keys to return from RPC `identity_getDidRecords`.
const RPC_MAX_KEYS: usize = 200;

const MAX_ASSETS: usize = 2000;
const MAX_PORTFOLIOS: usize = 2000;
const MAX_PALLETS: usize = 80;
const MAX_EXTRINSICS: usize = 80;
const MAX_NAME_LEN: usize = 60;

// Limit the maximum memory/cpu cost of a key's permissions.
const MAX_PERMISSION_COMPLEXITY: usize = 1_000_000;

type System<T> = frame_system::Pallet<T>;

impl<T: Config> Module<T> {
    /// Does the identity given by `did` exist?
    pub fn is_identity_exists(did: &IdentityId) -> bool {
        DidRecords::<T>::contains_key(did)
    }

    pub fn ensure_no_id_record(id: IdentityId) -> DispatchResult {
        ensure!(!Self::is_identity_exists(&id), Error::<T>::DidAlreadyExists);
        Ok(())
    }

    /// Returns `Err(DidDoesNotExist)` unless `id` has an associated record.
    crate fn ensure_id_record_exists(id: IdentityId) -> DispatchResult {
        ensure!(Self::is_identity_exists(&id), Error::<T>::DidDoesNotExist);
        Ok(())
    }

    /// Returns the DID associated with `key`, if any,
    /// assuming it is either the primary key or isn't frozen.
    pub fn get_identity(key: &T::AccountId) -> Option<IdentityId> {
        match KeyRecords::<T>::get(key)? {
            KeyRecord::PrimaryKey(did) => Some(did),
            KeyRecord::SecondaryKey(did, _) if !Self::is_did_frozen(did) => Some(did),
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

    /// RPC call to fetch some aggregate account data for fewer round trips.
    pub fn get_key_identity_data(acc: T::AccountId) -> Option<types::KeyIdentityData<IdentityId>> {
        let (identity, permissions) = match KeyRecords::<T>::get(acc)? {
            KeyRecord::PrimaryKey(did) => Some((did, None)),
            KeyRecord::SecondaryKey(did, perms) => Some((did, Some(perms))),
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
            Some(KeyRecord::PrimaryKey(_)) | Some(KeyRecord::SecondaryKey(_, _)) => (true, false),
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
                    KeyRecords::<T>::get(&key).and_then(|r| r.into_secondary_key(key))
                })
                .collect();
            RpcDidRecords::Success {
                primary_key: record.primary_key.unwrap_or_default(),
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
        // Do not allow unlinking MultiSig keys with balance >= 1 POLYX.
        if T::MultiSig::is_multisig(key) {
            ensure!(
                T::Balances::total_balance(key) < T::MultiSigBalanceLimit::get().into(),
                Error::<T>::MultiSigHasBalance
            );
        }
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
        if Self::can_add_key_record(key) {
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
            Some(KeyRecord::SecondaryKey(did1, _)) if Some(did1) == did => {
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
    crate fn accept_primary_key_rotation(
        origin: T::Origin,
        rotation_auth_id: u64,
        optional_cdd_auth_id: Option<u64>,
    ) -> DispatchResult {
        let sender = ensure_signed(origin)?;
        let signer = Signatory::Account(sender.clone());
        Self::accept_auth_with(&signer, rotation_auth_id, |data, target_did| {
            // Ensure Authorization is a `RotatePrimaryKey`.
            extract_auth!(data, RotatePrimaryKey);
            Self::common_rotate_primary_key(target_did, sender, None, optional_cdd_auth_id)
        })
    }

    // Sets the new primary key and optionally removes it as a secondary key if it is one.
    // Checks the cdd auth if this is required.
    // Old primary key will be added as a secondary key if `new_permissions` is not None
    // New primary key must either be unlinked, or linked to the `target_did`
    pub fn common_rotate_primary_key(
        target_did: IdentityId,
        new_primary_key: T::AccountId,
        new_permissions: Option<Permissions>,
        optional_cdd_auth_id: Option<u64>,
    ) -> DispatchResult {
        let old_primary_key = Self::get_primary_key(target_did).unwrap_or_default();

        let key_record = KeyRecords::<T>::get(&new_primary_key);
        let (is_linked, is_secondary_key) = match key_record {
            Some(KeyRecord::PrimaryKey(_)) => {
                // Already linked as a primary key.
                (true, false)
            }
            Some(KeyRecord::SecondaryKey(did, _)) => {
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

        let signer = Signatory::Account(new_primary_key.clone());

        // Accept authorization from CDD service provider.
        if Self::cdd_auth_for_primary_key_rotation() {
            let auth_id = optional_cdd_auth_id
                .ok_or_else(|| Error::<T>::InvalidAuthorizationFromCddProvider)?;

            Self::accept_auth_with(&signer, auth_id, |data, auth_by| {
                let attestation_for_did = extract_auth!(data, AttestPrimaryKeyRotation(a));
                // Attestor must be a CDD service provider.
                ensure!(
                    T::CddServiceProviders::is_member(&auth_by),
                    Error::<T>::NotCddProviderAttestation
                );
                // Ensure authorizations are for the same DID.
                ensure!(
                    target_did == attestation_for_did,
                    Error::<T>::AuthorizationsNotForSameDids
                );
                Ok(())
            })?;
        }

        // Replace primary key of the owner that initiated key rotation.
        let key_record = KeyRecord::PrimaryKey(target_did);
        if is_secondary_key {
            // Convert secondary key to primary key.
            KeyRecords::<T>::insert(&new_primary_key, key_record);
            DidRecords::<T>::insert(target_did, DidRecord::new(new_primary_key.clone()));

            let removed_keys = vec![new_primary_key.clone()];
            Self::deposit_event(RawEvent::SecondaryKeysRemoved(target_did, removed_keys));
        } else {
            Self::add_key_record(&new_primary_key, key_record);
        }
        Self::deposit_event(RawEvent::PrimaryKeyUpdated(
            target_did,
            old_primary_key.clone(),
            new_primary_key,
        ));

        if let Some(perms) = new_permissions {
            // Convert old primary key to secondary key.
            KeyRecords::<T>::insert(
                &old_primary_key,
                KeyRecord::SecondaryKey(target_did, perms.clone()),
            );

            let sk = SecondaryKey::new(old_primary_key, perms);
            Self::deposit_event(RawEvent::SecondaryKeysAdded(target_did, vec![sk.into()]));
        } else {
            Self::remove_key_record(&old_primary_key, Some(target_did));
        }
        Ok(())
    }

    /// Accepts a primary key rotation.
    /// Differs from accept_primary_key_rotation in that it will leave the old primary key as a
    /// secondary key with the permissions specified in the corresponding RotatePrimaryKeyToSecondary authorization
    /// instead of unlinking the primary key.
    crate fn base_rotate_primary_key_to_secondary(
        origin: T::Origin,
        rotation_auth_id: u64,
        optional_cdd_auth_id: Option<u64>,
    ) -> DispatchResult {
        let new_primary_key = ensure_signed(origin)?;
        let new_primary_key_signer = Signatory::Account(new_primary_key.clone());
        Self::accept_auth_with(
            &new_primary_key_signer,
            rotation_auth_id,
            |data, target_did| {
                let perms = extract_auth!(data, RotatePrimaryKeyToSecondary(p));

                Self::common_rotate_primary_key(
                    target_did,
                    new_primary_key,
                    Some(perms),
                    optional_cdd_auth_id,
                )
            },
        )
    }

    /// Set permissions for the specific `key`.
    /// Only the primary key of an identity is able to set secondary key permissions.
    crate fn base_set_secondary_key_permissions(
        origin: T::Origin,
        key: T::AccountId,
        permissions: Permissions,
    ) -> DispatchResult {
        let (_, did) = Self::ensure_primary_key(origin)?;

        // Ensure that the `key` is a secondary key of the caller's Identity
        Self::ensure_secondary_key(did, &key)?;

        Self::ensure_perms_length_limited(&permissions)?;

        // Update secondary key's permissions.
        KeyRecords::<T>::mutate(&key, |record| {
            if let Some(KeyRecord::SecondaryKey(_, perms)) = record {
                let old_perms = mem::replace(perms, permissions.clone());
                Self::deposit_event(RawEvent::SecondaryKeyPermissionsUpdated(
                    did,
                    key.clone(),
                    old_perms,
                    permissions,
                ));
            }
        });
        Ok(())
    }

    /// Removes specified secondary keys of a DID if present.
    crate fn base_remove_secondary_keys(
        origin: T::Origin,
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

            // All `auth_id`s for `signer` authorized by `did`.
            let signer = Signatory::Account(key.clone());
            for auth_id in Self::auths_of(&signer, did) {
                // Remove authorizations.
                Self::unsafe_remove_auth(&signer, auth_id, &did, true);
            }
        }

        Self::deposit_event(RawEvent::SecondaryKeysRemoved(did, keys));
        Ok(())
    }

    /// Adds secondary keys to target identity `id`.
    /// Keys are directly added to identity because each of them has an authorization.
    crate fn base_add_secondary_keys_with_authorization(
        origin: T::Origin,
        keys: Vec<SecondaryKeyWithAuth<T::AccountId>>,
        expires_at: T::Moment,
    ) -> DispatchResult {
        let (_, did) = Self::ensure_primary_key(origin)?;

        // 0. Check expiration
        let now = <pallet_timestamp::Pallet<T>>::get();
        ensure!(now < expires_at, Error::<T>::AuthorizationExpired);
        let authorization = TargetIdAuthorization {
            target_id: did,
            nonce: Self::offchain_authorization_nonce(did),
            expires_at,
        };
        let auth_encoded = authorization.encode();

        // 1. Verify signatures.
        for si_with_auth in keys.iter() {
            let si: SecondaryKey<T::AccountId> = si_with_auth.secondary_key.clone().into();

            Self::ensure_perms_length_limited(&si.permissions)?;

            // 1.1. Constraint 1-to-1 account to DID.
            Self::ensure_key_did_unlinked(&si.key)?;

            // 1.2. Verify the signature.
            let signature = AnySignature::from(Signature::from_h512(si_with_auth.auth_signature));
            let signer: <<AnySignature as Verify>::Signer as IdentifyAccount>::AccountId =
                Decode::decode(&mut &si.key.encode()[..])
                    .map_err(|_| Error::<T>::CannotDecodeSignerAccountId)?;
            ensure!(
                signature.verify(auth_encoded.as_slice(), &signer),
                Error::<T>::InvalidAuthorizationSignature
            );
        }
        // 1.999. Charge the fee.
        T::ProtocolFee::batch_charge_fee(
            ProtocolOp::IdentityAddSecondaryKeysWithAuthorization,
            keys.len(),
        )?;
        // 2.1. Link keys to identity
        let additional_keys_si: Vec<_> = keys
            .into_iter()
            .map(|si_with_auth| si_with_auth.secondary_key)
            .collect();

        additional_keys_si.iter().for_each(|sk| {
            Self::add_key_record(
                &sk.key,
                KeyRecord::SecondaryKey(did, sk.permissions.clone()),
            );
        });
        // 2.2. Update that identity's offchain authorization nonce.
        OffChainAuthorizationNonce::mutate(did, |nonce| *nonce = authorization.nonce + 1);

        Self::deposit_event(RawEvent::SecondaryKeysAdded(did, additional_keys_si));
        Ok(())
    }

    /// Accepts an auth to join an identity as a signer
    pub fn join_identity(origin: T::Origin, auth_id: u64) -> DispatchResult {
        let key = ensure_signed(origin)?;
        let signer = Signatory::Account(key.clone());
        Self::accept_auth_with(&signer, auth_id, |data, target_did| {
            let permissions = extract_auth!(data, JoinIdentity(p));
            // Not really needed unless we allow identities to be deleted.
            Self::ensure_id_record_exists(target_did)?;

            // Ensure that the key is unlinked.
            Self::ensure_key_did_unlinked(&key)?;

            // Check that the new Identity has a valid CDD claim.
            ensure!(Self::has_valid_cdd(target_did), Error::<T>::TargetHasNoCdd);
            // Charge the protocol fee after all checks.
            T::ProtocolFee::charge_fee(ProtocolOp::IdentityAddSecondaryKeysWithAuthorization)?;
            // Update current did of the transaction to the newly joined did.
            // This comes handy when someone uses a batch transaction to leave their identity,
            // join another identity, and then do something as the new identity.
            T::CddHandler::set_current_identity(&target_did);

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
        Self::add_key_record(
            &key,
            KeyRecord::SecondaryKey(target_did, permissions.clone()),
        );

        let sk = SecondaryKey { key, permissions };
        Self::deposit_event(RawEvent::SecondaryKeysAdded(target_did, vec![sk]));
    }

    crate fn leave_identity(origin: T::Origin) -> DispatchResult {
        let (key, did) = Self::ensure_did(origin)?;

        // Ensure that the caller is a secondary key.
        Self::ensure_secondary_key(did, &key)?;

        // Ensure that it is safe to unlink the account key from the did.
        Self::ensure_key_unlinkable_from_did(&key)?;

        // Unlink secondary key from the identity.
        Self::remove_key_record(&key, Some(did));

        Self::deposit_event(RawEvent::SecondaryKeyLeftIdentity(did, key));
        Ok(())
    }

    /// Freezes/unfreezes the target `did` identity.
    ///
    /// # Errors
    /// Only primary key can freeze/unfreeze an identity.
    crate fn set_frozen_secondary_key_flags(origin: T::Origin, freeze: bool) -> DispatchResult {
        let (_, did) = Self::ensure_primary_key(origin)?;
        if freeze {
            IsDidFrozen::insert(&did, true);
            Self::deposit_event(RawEvent::SecondaryKeysFrozen(did))
        } else {
            IsDidFrozen::remove(&did);
            Self::deposit_event(RawEvent::SecondaryKeysUnfrozen(did));
        }
        Ok(())
    }

    /// Create a new DID out of the parent block hash and a `nonce`.
    fn make_did(nonce: u64) -> IdentityId {
        // TODO: Look into getting randomness from `pallet_babe`.
        // NB: We can't get the current block's hash while processing
        // an extrinsic, so we use parent hash here.
        let parent_hash = System::<T>::parent_hash();
        IdentityId(blake2_256(&(USER, parent_hash, nonce).encode()))
    }

    /// Registers a did without adding a CDD claim for it.
    pub fn _register_did(
        sender: T::AccountId,
        secondary_keys: Vec<SecondaryKey<T::AccountId>>,
        protocol_fee_data: Option<ProtocolOp>,
    ) -> Result<IdentityId, DispatchError> {
        let new_nonce = Self::multi_purpose_nonce() + 7u64;
        // Even if this transaction fails, nonce should be increased for added unpredictability of dids
        MultiPurposeNonce::put(&new_nonce);

        // 1 Check constraints.
        // Primary key is not linked to any identity.
        Self::ensure_key_did_unlinked(&sender)?;
        // Primary key is not part of secondary keys.
        ensure!(
            !secondary_keys.iter().any(|sk| sk.key == sender),
            Error::<T>::SecondaryKeysContainPrimaryKey
        );

        let did = Self::make_did(new_nonce);

        // Make sure there's no pre-existing entry for the DID
        // This should never happen but just being defensive here
        Self::ensure_no_id_record(did)?;

        // Secondary keys can be linked to the new identity.
        for sk in &secondary_keys {
            Self::ensure_key_did_unlinked(&sk.key)?;
        }

        // Charge the given fee.
        if let Some(op) = protocol_fee_data {
            T::ProtocolFee::charge_fee(op)?;
        }

        // 2. Apply changes to our extrinsic.
        // 2.1. Create a new identity record and link the primary key.
        Self::add_key_record(&sender, KeyRecord::PrimaryKey(did));
        // 2.2. add pre-authorized secondary keys.
        secondary_keys.iter().for_each(|sk| {
            let signer = Signatory::Account(sk.key.clone());
            let data = AuthorizationData::JoinIdentity(sk.permissions.clone());
            Self::add_auth(did, signer, data, None);
        });

        // 2.3. Give `InitialPOLYX` to the primary key for testing.
        T::Balances::deposit_creating(&sender, T::InitialPOLYX::get().into());

        Self::deposit_event(RawEvent::DidCreated(did, sender, secondary_keys));
        Ok(did)
    }

    /// Registers the systematic issuer with its DID.
    #[allow(dead_code)]
    crate fn register_systematic_id(issuer: SystematicIssuers)
    where
        T::AccountId: core::fmt::Display,
    {
        let acc = issuer.as_pallet_id().into_account();
        let id = issuer.as_id();
        log::info!(
            "Register Systematic id {} with account {} as {}",
            issuer,
            acc,
            id
        );

        Self::do_register_id(acc, id, vec![]);
    }

    /// Registers `primary_key` as `id` identity.
    #[allow(dead_code)]
    crate fn do_register_id(
        primary_key: T::AccountId,
        id: IdentityId,
        secondary_keys: Vec<SecondaryKey<T::AccountId>>,
    ) {
        // Link primary key.
        <Module<T>>::add_key_record(&primary_key, KeyRecord::PrimaryKey(id));
        // Link secondary keys.
        for sk in &secondary_keys {
            Self::add_key_record(&sk.key, KeyRecord::SecondaryKey(id, sk.permissions.clone()));
        }

        Self::deposit_event(RawEvent::DidCreated(id, primary_key, secondary_keys));
    }

    /// Ensure the `key` is a secondary key of the identity `did`.
    fn ensure_secondary_key(did: IdentityId, key: &T::AccountId) -> DispatchResult {
        let key_did = Self::key_records(key).and_then(|rec| rec.is_secondary_key());
        ensure!(key_did == Some(did), Error::<T>::NotASigner);
        Ok(())
    }

    /// Ensures that `origin`'s key is the primary key of a DID.
    fn ensure_primary_key(origin: T::Origin) -> Result<(T::AccountId, IdentityId), DispatchError> {
        let sender = ensure_signed(origin)?;
        let key_rec =
            Self::key_records(&sender).ok_or(pallet_permissions::Error::<T>::UnauthorizedCaller)?;
        let did = key_rec.is_primary_key().ok_or(Error::<T>::KeyNotAllowed)?;
        Ok((sender, did))
    }

    /// Ensures that `origin`'s key is linked to a DID and returns both.
    pub fn ensure_did(origin: T::Origin) -> Result<(T::AccountId, IdentityId), DispatchError> {
        let sender = ensure_signed(origin)?;
        let did = Context::current_identity_or::<Self>(&sender)?;
        Ok((sender, did))
    }

    /// Checks call permissions and, if successful, returns the caller's account, primary and secondary identities.
    pub fn ensure_origin_call_permissions(
        origin: T::Origin,
    ) -> Result<PermissionedCallOriginData<T::AccountId>, DispatchError> {
        let sender = ensure_signed(origin)?;
        let AccountCallPermissionsData {
            primary_did,
            secondary_key,
        } = pallet_permissions::Module::<T>::ensure_call_permissions(&sender)?;
        Ok(PermissionedCallOriginData {
            sender,
            primary_did,
            secondary_key,
        })
    }

    /// Ensure `origin` is signed and permissioned for this call, returning its DID.
    pub fn ensure_perms(origin: T::Origin) -> Result<IdentityId, DispatchError> {
        Self::ensure_origin_call_permissions(origin).map(|x| x.primary_did)
    }

    /// Ensures length limits are enforced in `perms`.
    pub fn ensure_perms_length_limited(perms: &Permissions) -> DispatchResult {
        ensure_custom_length_ok::<T>(perms.complexity(), MAX_PERMISSION_COMPLEXITY)?;
        ensure_custom_length_ok::<T>(perms.asset.complexity(), MAX_ASSETS)?;
        ensure_custom_length_ok::<T>(perms.portfolio.complexity(), MAX_PORTFOLIOS)?;
        Self::ensure_extrinsic_perms_length_limited(&perms.extrinsic)
    }

    /// Ensures length limits are enforced in `perms`.
    pub fn ensure_extrinsic_perms_length_limited(perms: &ExtrinsicPermissions) -> DispatchResult {
        if let Some(set) = perms.inner() {
            ensure_custom_length_ok::<T>(set.len(), MAX_PALLETS)?;
            for elem in set {
                ensure_custom_string_limited::<T>(&elem.pallet_name, MAX_NAME_LEN)?;
                if let Some(set) = elem.dispatchable_names.inner() {
                    ensure_custom_length_ok::<T>(set.len(), MAX_EXTRINSICS)?;
                    for elem in set {
                        ensure_custom_string_limited::<T>(elem, MAX_NAME_LEN)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl<T: Config> CheckAccountCallPermissions<T::AccountId> for Module<T> {
    // For weighting purposes, the function reads 4 storage values.
    fn check_account_call_permissions(
        who: &T::AccountId,
        pallet_name: impl FnOnce() -> PalletName,
        function_name: impl FnOnce() -> DispatchableName,
    ) -> Option<AccountCallPermissionsData<T::AccountId>> {
        let data = |did, secondary_key| AccountCallPermissionsData {
            primary_did: did,
            secondary_key,
        };

        match KeyRecords::<T>::get(who)? {
            // Primary keys do not have / require further permission checks.
            KeyRecord::PrimaryKey(did) => Some(data(did, None)),
            // Secondary Key. Ensure DID isn't frozen + key has sufficient permissions.
            KeyRecord::SecondaryKey(did, permissions) if !Self::is_did_frozen(&did) => {
                let sk = SecondaryKey {
                    key: who.clone(),
                    permissions,
                };
                sk.has_extrinsic_permission(&pallet_name(), &function_name())
                    .then(|| data(did, Some(sk)))
            }
            // DIDs with frozen secondary keys, AKA frozen DIDs, are not permitted to call extrinsics.
            _ => None,
        }
    }
}
