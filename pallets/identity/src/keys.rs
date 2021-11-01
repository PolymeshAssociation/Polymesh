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
    types, AccountKeyRefCount, Config, DidRecord, DidRecords, Error, IsDidFrozen, KeyToIdentityIds,
    Module, MultiPurposeNonce, OffChainAuthorizationNonce, PermissionedCallOriginData, RawEvent,
    RpcDidRecords,
};
use codec::{Decode, Encode as _};
use core::{iter, mem};
use frame_support::dispatch::DispatchResult;
use frame_support::traits::{Currency as _, Get as _};
use frame_support::{debug, ensure, StorageMap as _, StorageValue as _};
use frame_system::ensure_signed;
use pallet_base::{ensure_length_ok, ensure_string_limited};
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
    extract_auth, secondary_key, AuthorizationData, DispatchableName, ExtrinsicPermissions,
    IdentityId, PalletName, Permissions, SecondaryKey, Signatory,
};
use sp_core::sr25519::Signature;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{AccountIdConversion as _, IdentifyAccount, Verify, Zero as _};
use sp_runtime::{AnySignature, DispatchError};
use sp_std::{vec, vec::Vec};

type System<T> = frame_system::Module<T>;

impl<T: Config> Module<T> {
    /// Does the identity given by `did` exist?
    pub fn is_identity_exists(did: &IdentityId) -> bool {
        <DidRecords<T>>::contains_key(did)
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
        <KeyToIdentityIds<T>>::try_get(key)
            .ok()
            .filter(|did| !Self::is_did_frozen(did) || Self::is_primary_key(&did, key))
    }

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
        if let Ok(record) = <DidRecords<T>>::try_get(did) {
            RpcDidRecords::Success {
                primary_key: record.primary_key,
                secondary_keys: record.secondary_keys,
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
    fn unlink_account_key_from_did(key: &T::AccountId, did: IdentityId) {
        if <KeyToIdentityIds<T>>::contains_key(key) && <KeyToIdentityIds<T>>::get(key) == did {
            <KeyToIdentityIds<T>>::remove(key)
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
            Self::unsafe_primary_key_rotation(sender, target_did, optional_cdd_auth_id)
        })
    }

    /// Processes primary key rotation.
    pub fn unsafe_primary_key_rotation(
        sender: T::AccountId,
        rotation_for_did: IdentityId,
        optional_cdd_auth_id: Option<u64>,
    ) -> DispatchResult {
        // Accept authorization from CDD service provider.
        if Self::cdd_auth_for_primary_key_rotation() {
            let auth_id = optional_cdd_auth_id
                .ok_or_else(|| Error::<T>::InvalidAuthorizationFromCddProvider)?;

            let signer = Signatory::Account(sender.clone());
            Self::accept_auth_with(&signer, auth_id, |data, auth_by| {
                let attestation_for_did = extract_auth!(data, AttestPrimaryKeyRotation(a));
                // Attestor must be a CDD service provider.
                ensure!(
                    T::CddServiceProviders::is_member(&auth_by),
                    Error::<T>::NotCddProviderAttestation
                );
                // Ensure authorizations are for the same DID.
                ensure!(
                    rotation_for_did == attestation_for_did,
                    Error::<T>::AuthorizationsNotForSameDids
                );
                Ok(())
            })?;
        }

        Self::ensure_key_did_unlinked(&sender)?;

        // Get the current DidRecord.
        let mut record = Self::did_records(&rotation_for_did);
        let old_primary_key = record.primary_key;

        // Ensure that it is safe to unlink the primary key from the did.
        Self::ensure_key_unlinkable_from_did(&old_primary_key)?;

        // Replace primary key of the owner that initiated key rotation.
        Self::unlink_account_key_from_did(&old_primary_key, rotation_for_did);
        record.primary_key = sender.clone();
        Self::link_account_key_to_did(&sender, rotation_for_did);
        <DidRecords<T>>::insert(&rotation_for_did, record);

        Self::deposit_event(RawEvent::PrimaryKeyUpdated(
            rotation_for_did,
            old_primary_key,
            sender,
        ));
        Ok(())
    }

    /// Set permissions for the specific `target_key`.
    /// Only the primary key of an identity is able to set secondary key permissions.
    crate fn base_set_permission_to_signer(
        origin: T::Origin,
        signer: Signatory<T::AccountId>,
        permissions: Permissions,
    ) -> DispatchResult {
        let (_, did, record) = Self::ensure_primary_key(origin)?;

        // Ensure that the signer is a secondary key of the caller's Identity
        ensure!(
            record.secondary_keys.iter().any(|si| si.signer == signer),
            Error::<T>::NotASigner
        );

        Self::ensure_perms_length_limited(&permissions)?;

        <DidRecords<T>>::mutate(did, |record| {
            if let Some(secondary_key) = record
                .secondary_keys
                .iter_mut()
                .find(|si| si.signer == signer)
            {
                let old_perms = mem::replace(&mut secondary_key.permissions, permissions.clone());
                Self::deposit_event(RawEvent::SecondaryKeyPermissionsUpdated(
                    did,
                    secondary_key.clone().into(),
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
        signers: Vec<Signatory<T::AccountId>>,
    ) -> DispatchResult {
        let (_, did, _) = Self::ensure_primary_key(origin)?;

        // Ensure that it is safe to unlink the secondary keys from the did.
        for signer in &signers {
            if let Signatory::Account(key) = &signer {
                Self::ensure_key_unlinkable_from_did(key)?;
            }
        }

        // Remove links and get all authorization IDs per signer.
        signers
            .iter()
            .flat_map(|signer| {
                use either::Either::{Left, Right};

                // Unlink each of the given secondary keys from `did`.
                if let Signatory::Account(key) = &signer {
                    // Unlink multisig signers.
                    if T::MultiSig::is_multisig(key) {
                        if !T::Balances::total_balance(key).is_zero() {
                            return Left(iter::empty());
                        }
                        // Unlink multisig signers from the identity.
                        Self::unlink_multisig_signers_from_did(
                            T::MultiSig::get_key_signers(key),
                            did,
                        );
                    }
                    // Unlink the secondary account key.
                    Self::unlink_account_key_from_did(key, did);
                }

                // All `auth_id`s for `signer` authorized by `did`.
                Right(Self::auths_of(signer, did))
            })
            // Remove authorizations.
            .for_each(|(signer, auth_id)| Self::unsafe_remove_auth(signer, auth_id, &did, true));

        // Update secondary keys at Identity.
        <DidRecords<T>>::mutate(did, |record| {
            record.remove_secondary_keys(&signers);
        });

        Self::deposit_event(RawEvent::SecondaryKeysRemoved(did, signers));
        Ok(())
    }

    /// Adds secondary keys to target identity `id`.
    /// Keys are directly added to identity because each of them has an authorization.
    crate fn base_add_secondary_keys_with_authorization(
        origin: T::Origin,
        keys: Vec<SecondaryKeyWithAuth<T::AccountId>>,
        expires_at: T::Moment,
    ) -> DispatchResult {
        let (_, did, _) = Self::ensure_primary_key(origin)?;

        // 0. Check expiration
        let now = <pallet_timestamp::Module<T>>::get();
        ensure!(now < expires_at, Error::<T>::AuthorizationExpired);
        let authorization = TargetIdAuthorization {
            target_id: did,
            nonce: Self::offchain_authorization_nonce(did),
            expires_at,
        };
        let auth_encoded = authorization.encode();

        let mut record = <DidRecords<T>>::get(did);

        // Ensure we won't have too many keys.
        ensure_length_ok::<T>(record.secondary_keys.len().saturating_add(keys.len()))?;

        // 1. Verify signatures.
        for si_with_auth in keys.iter() {
            let si: SecondaryKey<T::AccountId> = si_with_auth.secondary_key.clone().into();

            Self::ensure_perms_length_limited(&si.permissions)?;

            // Get account_id from signer.
            let account_id = si
                .signer
                .as_account()
                .ok_or(Error::<T>::InvalidAccountKey)?;

            // 1.1. Constraint 1-to-1 account to DID.
            ensure!(
                Self::can_link_account_key_to_did(account_id),
                Error::<T>::AlreadyLinked
            );

            // 1.2. Verify the signature.
            let signature = AnySignature::from(Signature::from_h512(si_with_auth.auth_signature));
            let signer: <<AnySignature as Verify>::Signer as IdentifyAccount>::AccountId =
                Decode::decode(&mut &account_id.encode()[..])
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
            if let Signatory::Account(key) = &sk.signer {
                Self::link_account_key_to_did(key, did);
            }
        });
        // 2.2. Update that identity information and its offchain authorization nonce.
        record.add_secondary_keys(additional_keys_si.iter().map(|sk| sk.clone().into()));
        <DidRecords<T>>::insert(did, record);
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

            // Link the secondary key.
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
        Self::link_account_key_to_did(&key, target_did);

        // Link the secondary key.
        let sk = SecondaryKey::new(Signatory::Account(key), permissions);
        <DidRecords<T>>::mutate(target_did, |identity| {
            identity.add_secondary_keys(iter::once(sk.clone()));
        });
        Self::deposit_event(RawEvent::SecondaryKeysAdded(target_did, vec![sk.into()]));
    }

    crate fn leave_identity(origin: T::Origin) -> DispatchResult {
        let (key, did) = Self::ensure_did(origin)?;
        let signer = Signatory::Account(key.clone());
        ensure!(Self::is_signer(did, &signer), Error::<T>::NotASigner);

        // Ensure that it is safe to unlink the account key from the did.
        Self::ensure_key_unlinkable_from_did(&key)?;

        // Unlink multisig signers.
        if T::MultiSig::is_multisig(&key) {
            ensure!(
                T::Balances::total_balance(&key).is_zero(),
                Error::<T>::MultiSigHasBalance
            );
            // Unlink multisig signers from the identity.
            Self::unlink_multisig_signers_from_did(T::MultiSig::get_key_signers(&key), did);
        }
        Self::unlink_account_key_from_did(&key, did);

        // Update secondary keys at Identity.
        <DidRecords<T>>::mutate(did, |record| {
            record.remove_secondary_keys(&[signer.clone()]);
        });
        Self::deposit_event(RawEvent::SignerLeft(did, signer));
        Ok(())
    }

    fn unlink_multisig_signers_from_did(signers: Vec<T::AccountId>, did: IdentityId) {
        for signer in signers {
            Self::unlink_account_key_from_did(&signer, did)
        }
    }

    /// Freezes/unfreezes the target `did` identity.
    ///
    /// # Errors
    /// Only primary key can freeze/unfreeze an identity.
    crate fn set_frozen_secondary_key_flags(origin: T::Origin, freeze: bool) -> DispatchResult {
        let (_, did, _) = Self::ensure_primary_key(origin)?;
        if freeze {
            IsDidFrozen::insert(&did, true);
            Self::deposit_event(RawEvent::SecondaryKeysFrozen(did))
        } else {
            IsDidFrozen::remove(&did);
            Self::deposit_event(RawEvent::SecondaryKeysUnfrozen(did));
        }
        Ok(())
    }

    /// Create a new DID out of the current block hash and a `nonce`.
    fn make_did(nonce: u64) -> IdentityId {
        // TODO: This currently always returns 0x000...000.
        // See https://polymath.atlassian.net/browse/MESH-1546
        let block_hash = System::<T>::block_hash(System::<T>::block_number());
        IdentityId(blake2_256(&(USER, block_hash, nonce).encode()))
    }

    /// Registers a did without adding a CDD claim for it.
    pub fn _register_did(
        sender: T::AccountId,
        secondary_keys: Vec<SecondaryKey<T::AccountId>>,
        protocol_fee_data: Option<ProtocolOp>,
    ) -> Result<IdentityId, DispatchError> {
        // Adding extrensic count to did nonce for some unpredictability
        // NB: this does not guarantee randomness
        let new_nonce =
            Self::multi_purpose_nonce() + u64::from(System::<T>::extrinsic_count()) + 7u64;
        // Even if this transaction fails, nonce should be increased for added unpredictability of dids
        MultiPurposeNonce::put(&new_nonce);

        // 1 Check constraints.
        // Primary key is not linked to any identity.
        Self::ensure_key_did_unlinked(&sender)?;
        // Primary key is not part of secondary keys.
        ensure!(
            !secondary_keys
                .iter()
                .any(|sk| sk.signer.as_account() == Some(&sender)),
            Error::<T>::SecondaryKeysContainPrimaryKey
        );

        let did = Self::make_did(new_nonce);

        // Make sure there's no pre-existing entry for the DID
        // This should never happen but just being defensive here
        Self::ensure_no_id_record(did)?;

        // Secondary keys can be linked to the new identity.
        for sk in &secondary_keys {
            if let Signatory::Account(ref key) = sk.signer {
                Self::ensure_key_did_unlinked(key)?;
            }
        }

        // Charge the given fee.
        if let Some(op) = protocol_fee_data {
            T::ProtocolFee::charge_fee(op)?;
        }

        // 2. Apply changes to our extrinsic.
        // 2.1. Link primary key and add pre-authorized secondary keys.
        Self::link_account_key_to_did(&sender, did);
        secondary_keys.iter().for_each(|sk| {
            let data = AuthorizationData::JoinIdentity(sk.permissions.clone().into());
            Self::add_auth(did, sk.signer.clone(), data, None);
        });

        // 2.2. Create a new identity record.
        let record = DidRecord {
            primary_key: sender.clone(),
            ..Default::default()
        };
        <DidRecords<T>>::insert(&did, record);

        // 2.3. Give `InitialPOLYX` to the primary key for testing.
        T::Balances::deposit_creating(&sender, T::InitialPOLYX::get().into());

        Self::deposit_event(RawEvent::DidCreated(
            did,
            sender,
            secondary_keys
                .into_iter()
                .map(secondary_key::api::SecondaryKey::from)
                .collect(),
        ));
        Ok(did)
    }

    /// Registers the systematic issuer with its DID.
    #[allow(dead_code)]
    crate fn register_systematic_id(issuer: SystematicIssuers)
    where
        T::AccountId: core::fmt::Display,
    {
        let acc = issuer.as_module_id().into_account();
        let id = issuer.as_id();
        debug::info!(
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
        <Module<T>>::link_account_key_to_did(&primary_key, id);
        for sk in &secondary_keys {
            if let Signatory::Account(key) = &sk.signer {
                Self::link_account_key_to_did(key, id);
            }
        }

        let record = DidRecord {
            primary_key: primary_key.clone(),
            secondary_keys,
            ..Default::default()
        };
        <DidRecords<T>>::insert(&id, record);

        Self::deposit_event(RawEvent::DidCreated(id, primary_key, vec![]));
    }

    /// Ensures that `origin`'s key is the primary key of a DID.
    fn ensure_primary_key(
        origin: T::Origin,
    ) -> Result<(T::AccountId, IdentityId, DidRecord<T::AccountId>), DispatchError> {
        let sender = ensure_signed(origin)?;
        let (did, record) = Self::did_record_of(&sender)
            .ok_or(pallet_permissions::Error::<T>::UnauthorizedCaller)?;
        ensure!(sender == record.primary_key, Error::<T>::KeyNotAllowed);
        Ok((sender, did, record))
    }

    /// Returns `Some((did, record))` if the DID record is present for the DID of `who`
    fn did_record_of(who: &T::AccountId) -> Option<(IdentityId, DidRecord<T::AccountId>)> {
        let did = <KeyToIdentityIds<T>>::try_get(who).ok()?;
        let record = <DidRecords<T>>::try_get(&did).ok()?;
        Some((did, record))
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
    crate fn ensure_perms_length_limited(perms: &Permissions) -> DispatchResult {
        ensure_length_ok::<T>(perms.asset.complexity())?;
        ensure_length_ok::<T>(perms.portfolio.complexity())?;
        Self::ensure_extrinsic_perms_length_limited(&perms.extrinsic)
    }

    /// Ensures length limits are enforced in `perms`.
    pub fn ensure_extrinsic_perms_length_limited(perms: &ExtrinsicPermissions) -> DispatchResult {
        if let Some(set) = perms.inner() {
            ensure_length_ok::<T>(set.len())?;
            for elem in set {
                ensure_string_limited::<T>(&elem.pallet_name)?;
                if let Some(set) = elem.dispatchable_names.inner() {
                    ensure_length_ok::<T>(set.len())?;
                    for elem in set {
                        ensure_string_limited::<T>(elem)?;
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
        let (did, record) = Self::did_record_of(who)?;
        let data = |secondary_key| AccountCallPermissionsData {
            primary_did: did,
            secondary_key,
        };

        if who == &record.primary_key {
            // It is a direct call and `who` is the primary key.
            return Some(data(None));
        }

        // DIDs with frozen secondary keys, AKA frozen DIDs, are not permitted to call extrinsics.
        if Self::is_did_frozen(&did) {
            return None;
        }

        // Find the secondary key matching `who` and ensure it has sufficient permissions.
        record
            .secondary_keys
            .into_iter()
            .find(|sk| sk.signer.as_account().contains(&who))
            .filter(|sk| sk.has_extrinsic_permission(&pallet_name(), &function_name()))
            .map(|sk| data(Some(sk)))
    }
}
