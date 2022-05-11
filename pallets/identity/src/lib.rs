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

//! # Identity module
//!
//! This module is used to manage identity concept.
//!
//!  - [Module](./struct.Module.html)
//!  - [Trait](./trait.Trait.html)
//!
//! ## Overview :
//!
//! Identity concept groups different account (keys) in one place, and it allows each key to
//! make operations based on the constraint that each account (permissions and key types).
//!
//! Any account can create and manage one and only one identity, using
//! [register_did](./struct.Module.html#method.register_did). Other accounts can be added to a
//! target identity as secondary key, where we also define the type of account (`External`,
//! `MultiSign`, etc.) and/or its permission.
//!
//! ## Identity information
//!
//! Identity contains the following data:
//!  - `primary_key`. It is the administrator account of the identity.
//!  - `secondary_keys`. List of keys and their capabilities (type of key and its permissions) .
//!
//! ## Freeze secondary keys
//!
//! It is an *emergency action* to block all secondary keys of an identity and it can only be performed
//! by its administrator.
//!
//! see [freeze_secondary_keys](./struct.Module.html#method.freeze_secondary_keys)
//! see [unfreeze_secondary_keys](./struct.Module.html#method.unfreeze_secondary_keys)
//!
//! ## Claim Unique Index
//!
//! Each claim is identified by a unique index, which is composed by two keys in order to optimise
//! the posterior use of them:
//! - Claim First Key, which have two fields:
//!    - A target DID, which is the user that receive that claim.
//!    - The type of the claim.
//! - Claim Second Key contains:
//!     - An issuer of the claim, who generated/added that claim.
//!     - An optional scope, it could limit the scope of this claim to specific assets,
//!     identities, or any other custom label.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `invalidate_cdd_claims` - Invalidates any claim generated by `cdd` from `disable_from` timestamps.
//! - `remove_secondary_keys` - Removes specified secondary keys of a DID if present.
//! - `accept_primary_key` - Accept authorization to become the new primary key of an identity.
//! - `change_cdd_requirement_for_mk_rotation` - Sets if CDD authorization is required for updating primary key of an identity.
//! - `join_identity_as_key` - Join an identity as a secondary key.
//! - `add_claim` - Adds a new claim record or edits an existing one.
//! - `revoke_claim` - Marks the specified claim as revoked.
//! - `revoke_claim_by_index` - Revoke a claim identified by its index.
//! - `set_secondary_key_permissions` - Sets permissions for a secondary key.
//! - `freeze_secondary_keys` - Disables all secondary keys at `did` identity.
//! - `unfreeze_secondary_keys` - Re-enables all secondary keys of the caller's identity.
//! - `add_authorization` - Adds an authorization.
//! - `remove_authorization` - Removes an authorization.
//! - `add_secondary_keys_with_authorization` - Adds secondary keys to target identity `id`.
//! - `add_investor_uniqueness_claim` - Adds InvestorUniqueness claim for a given target identity.
//! - `add_investor_uniqueness_claim_v2` - Adds InvestorUniqueness claim V2 for a given target identity.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
#![feature(const_option, option_result_contains, crate_visibility_modifier)]

mod auth;
mod claims;
mod keys;

pub mod types;
pub use types::{Claim1stKey, Claim2ndKey, DidStatus, PermissionedCallOriginData, RpcDidRecords};

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use confidential_identity::ScopeClaimProof;
use core::convert::From;
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::DispatchResult,
    traits::{ChangeMembers, Currency, EnsureOrigin, Get, InitializeMembers},
    weights::{
        DispatchClass::{Normal, Operational},
        Pays, Weight,
    },
};
use frame_system::ensure_root;
pub use polymesh_common_utilities::traits::identity::WeightInfo;
use polymesh_common_utilities::{
    constants::did::SECURITY_TOKEN,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    traits::identity::{
        AuthorizationNonce, Config, IdentityFnTrait, RawEvent, SecondaryKeyWithAuth,
    },
    SystematicIssuers, GC_DID,
};
use polymesh_primitives::{
    investor_zkproof_data::v1::InvestorZKProofData, storage_migrate_on, storage_migration_ver,
    Authorization, AuthorizationData, AuthorizationType, CddId, Claim, ClaimType, DidRecord,
    IdentityClaim, IdentityId, KeyRecord, Permissions, Scope, SecondaryKey, Signatory, Ticker,
};
use sp_runtime::traits::Hash;
use sp_std::{convert::TryFrom, prelude::*};

pub type Event<T> = polymesh_common_utilities::traits::identity::Event<T>;

storage_migration_ver!(1);

decl_storage! {
    trait Store for Module<T: Config> as Identity {

        /// DID -> identity info
        pub DidRecords get(fn did_records):
            map hasher(identity) IdentityId => Option<DidRecord<T::AccountId>>;

        /// DID -> bool that indicates if secondary keys are frozen.
        pub IsDidFrozen get(fn is_did_frozen): map hasher(identity) IdentityId => bool;

        /// It stores the current identity for current transaction.
        pub CurrentDid: Option<IdentityId>;

        /// It stores the current gas fee payer for the current transaction
        pub CurrentPayer: Option<T::AccountId>;

        /// (Target ID, claim type) (issuer,scope) -> Associated claims
        pub Claims: double_map hasher(twox_64_concat) Claim1stKey, hasher(blake2_128_concat) Claim2ndKey => IdentityClaim;

        /// Map from AccountId to `KeyRecord` that holds the key's identity and permissions.
        pub KeyRecords get(fn key_records):
            map hasher(twox_64_concat) T::AccountId => Option<KeyRecord<T::AccountId>>;

        /// A reverse double map to allow finding all keys for an identity.
        pub DidKeys get(fn did_keys):
            double_map hasher(identity) IdentityId, hasher(twox_64_concat) T::AccountId => bool;

        /// Nonce to ensure unique actions. starts from 1.
        pub MultiPurposeNonce get(fn multi_purpose_nonce) build(|_| 1u64): u64;

        /// Authorization nonce per Identity. Initially is 0.
        pub OffChainAuthorizationNonce get(fn offchain_authorization_nonce): map hasher(identity) IdentityId => AuthorizationNonce;

        /// All authorizations that an identity/key has
        pub Authorizations get(fn authorizations): double_map hasher(blake2_128_concat)
            Signatory<T::AccountId>, hasher(twox_64_concat) u64 => Option<Authorization<T::AccountId, T::Moment>>;

        /// All authorizations that an identity has given. (Authorizer, auth_id -> authorized)
        pub AuthorizationsGiven: double_map hasher(identity)
            IdentityId, hasher(twox_64_concat) u64 => Signatory<T::AccountId>;

        /// Obsoleted storage variable superceded by `CddAuthForPrimaryKeyRotation`. It is kept here
        /// for the purpose of storage migration.
        pub CddAuthForMasterKeyRotation get(fn cdd_auth_for_master_key_rotation): bool;

        /// A config flag that, if set, instructs an authorization from a CDD provider in order to
        /// change the primary key of an identity.
        pub CddAuthForPrimaryKeyRotation get(fn cdd_auth_for_primary_key_rotation): bool;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1).unwrap()): Version;

        /// How many "strong" references to the account key.
        ///
        /// Strong references will block a key from leaving it's identity.
        ///
        /// Pallets using "strong" references to account keys:
        /// * Relayer: For `user_key` and `paying_key`
        ///
        pub AccountKeyRefCount get(fn account_key_ref_count):
            map hasher(blake2_128_concat) T::AccountId => u64;
    }
    add_extra_genesis {
        // Identities at genesis.
        config(identities): Vec<polymesh_primitives::identity_id::GenesisIdentityRecord<T::AccountId>>;
        // Secondary keys of identities at genesis. `identities` have to be initialised.
        config(secondary_keys): Vec<(T::AccountId, IdentityId)>;
        build(|config: &GenesisConfig<T>| {
            polymesh_common_utilities::SYSTEMATIC_ISSUERS
                .iter()
                .copied()
                .for_each(<Module<T>>::register_systematic_id);

            // Add CDD claims to Treasury & BRR
            let sys_issuers_with_cdd = [SystematicIssuers::Treasury, SystematicIssuers::BlockRewardReserve, SystematicIssuers::Settlement, SystematicIssuers::Rewards];
            let id_with_cdd = sys_issuers_with_cdd.iter()
                .map(|iss| iss.as_id())
                .collect::<Vec<_>>();

            <Module<T>>::add_systematic_cdd_claims(&id_with_cdd, SystematicIssuers::CDDProvider);

            //  Other
            for gen_id in &config.identities {
                let cdd_claim = Claim::CustomerDueDiligence(CddId::new_v1(gen_id.did, gen_id.investor));
                // Direct storage change for registering the DID and providing the claim
                <Module<T>>::ensure_no_id_record(gen_id.did).unwrap();
                <MultiPurposeNonce>::mutate(|n| *n += 1_u64);
                let expiry = gen_id.cdd_claim_expiry.iter().map(|m| T::Moment::from(*m as u32)).next();
                <Module<T>>::do_register_id(gen_id.primary_key.clone(), gen_id.did, gen_id.secondary_keys.clone());
                for issuer in &gen_id.issuers {
                    <Module<T>>::base_add_claim(gen_id.did, cdd_claim.clone(), issuer.clone(), expiry);
                }
            }

            for &(ref secondary_account_id, did) in &config.secondary_keys {
                // Direct storage change for attaching some secondary keys to identities
                <Module<T>>::ensure_id_record_exists(did).unwrap();
                assert!(
                    <Module<T>>::can_add_key_record(secondary_account_id),
                    "Secondary key already linked"
                );
                <MultiPurposeNonce>::mutate(|n| *n += 1_u64);
                let sk = SecondaryKey::from_account_id(secondary_account_id.clone());
                <Module<T>>::add_key_record(secondary_account_id, sk.make_key_record(did));
                <Module<T>>::deposit_event(RawEvent::SecondaryKeysAdded(did, vec![sk.into()]));
            }
        });
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        const InitialPOLYX: <T::Balances as Currency<T::AccountId>>::Balance = T::InitialPOLYX::get().into();

        fn on_runtime_upgrade() -> Weight {
            storage_migrate_on!(StorageVersion::get(), 1, {
                migration::migrate_v1::<T>();
            });

            0
        }

        /// Register `target_account` with a new Identity.
        ///
        /// # Failure
        /// - `origin` has to be a active CDD provider. Inactive CDD providers cannot add new
        /// claims.
        /// - `target_account` (primary key of the new Identity) can be linked to just one and only
        /// one identity.
        /// - External secondary keys can be linked to just one identity.
        ///
        /// # Weight
        /// `7_000_000_000 + 600_000 * secondary_keys.len()`
        #[weight = <T as Config>::WeightInfo::cdd_register_did(secondary_keys.len() as u32)]
        pub fn cdd_register_did(
            origin,
            target_account: T::AccountId,
            secondary_keys: Vec<SecondaryKey<T::AccountId>>
        ) {
            Self::base_cdd_register_did(origin, target_account, secondary_keys)?;
        }

        /// Invalidates any claim generated by `cdd` from `disable_from` timestamps.
        ///
        /// You can also define an expiration time,
        /// which will invalidate all claims generated by that `cdd` and remove it as CDD member group.
        #[weight = (<T as Config>::WeightInfo::invalidate_cdd_claims(), Operational, Pays::Yes)]
        pub fn invalidate_cdd_claims(
            origin,
            cdd: IdentityId,
            disable_from: T::Moment,
            expiry: Option<T::Moment>,
        ) {
            Self::base_invalidate_cdd_claims(origin, cdd, disable_from, expiry)?;
        }

        /// Removes specified secondary keys of a DID if present.
        ///
        /// # Failure
        /// It can only called by primary key owner.
        ///
        /// # Weight
        /// `950_000_000 + 60_000 * keys_to_remove.len()`
        #[weight = <T as Config>::WeightInfo::remove_secondary_keys(keys_to_remove.len() as u32)]
        pub fn remove_secondary_keys(origin, keys_to_remove: Vec<T::AccountId>) {
            Self::base_remove_secondary_keys(origin, keys_to_remove)?;
        }

        /// Call this with the new primary key. By invoking this method, caller accepts authorization
        /// to become the new primary key of the issuing identity. If a CDD service provider approved
        /// this change (or this is not required), primary key of the DID is updated.
        ///
        /// The caller (new primary key) must be either a secondary key of the issuing identity, or
        /// unlinked to any identity.
        ///
        /// Differs from rotate_primary_key_to_secondary in that it will unlink the old primary key
        /// instead of leaving it as a secondary key.
        ///
        /// # Arguments
        /// * `owner_auth_id` Authorization from the owner who initiated the change
        /// * `cdd_auth_id` Authorization from a CDD service provider
        #[weight = <T as Config>::WeightInfo::accept_primary_key()]
        pub fn accept_primary_key(origin, rotation_auth_id: u64, optional_cdd_auth_id: Option<u64>) -> DispatchResult {
            Self::accept_primary_key_rotation(origin, rotation_auth_id, optional_cdd_auth_id)
        }

        /// Set if CDD authorization is required for updating primary key of an identity.
        /// Callable via root (governance)
        ///
        /// # Arguments
        /// * `auth_required` CDD Authorization required or not
        #[weight = (<T as Config>::WeightInfo::change_cdd_requirement_for_mk_rotation(), Operational, Pays::Yes)]
        pub fn change_cdd_requirement_for_mk_rotation(origin, auth_required: bool) {
            ensure_root(origin)?;
            CddAuthForPrimaryKeyRotation::put(auth_required);
            Self::deposit_event(RawEvent::CddRequirementForPrimaryKeyUpdated(auth_required));
        }

        /// Join an identity as a secondary key.
        #[weight = <T as Config>::WeightInfo::join_identity_as_key()]
        pub fn join_identity_as_key(origin, auth_id: u64) -> DispatchResult {
            Self::join_identity(origin, auth_id)
        }

        /// Leave the secondary key's identity.
        #[weight = <T as Config>::WeightInfo::leave_identity_as_key()]
        pub fn leave_identity_as_key(origin) -> DispatchResult {
            Self::leave_identity(origin)
        }

        /// Adds a new claim record or edits an existing one.
        ///
        /// Only called by did_issuer's secondary key.
        #[weight = <T as Config>::WeightInfo::add_claim()]
        pub fn add_claim(
            origin,
            target: IdentityId,
            claim: Claim,
            expiry: Option<T::Moment>,
        ) -> DispatchResult {
            let issuer = Self::ensure_signed_and_validate_claim_target(origin, target)?;

            match &claim {
                Claim::CustomerDueDiligence(..) => Self::base_add_cdd_claim(target, claim, issuer, expiry),
                Claim::InvestorUniqueness(..) | Claim::InvestorUniquenessV2(..) => Err(Error::<T>::ClaimVariantNotAllowed.into()),
                _ => {
                    Self::ensure_custom_scopes_limited(&claim)?;
                    T::ProtocolFee::charge_fee(ProtocolOp::IdentityAddClaim)?;
                    Self::base_add_claim(target, claim, issuer, expiry);
                    Ok(())
                }
            }
        }

        /// Marks the specified claim as revoked.
        #[weight = (<T as Config>::WeightInfo::revoke_claim(), revoke_claim_class(claim.claim_type()))]
        pub fn revoke_claim(origin, target: IdentityId, claim: Claim) -> DispatchResult {
            let issuer = Self::ensure_perms(origin)?;
            let claim_type = claim.claim_type();
            let scope = claim.as_scope().cloned();
            Self::base_revoke_claim(target, claim_type, issuer, scope)
        }

        /// Sets permissions for an specific `target_key` key.
        ///
        /// Only the primary key of an identity is able to set secondary key permissions.
        #[weight = <T as Config>::WeightInfo::set_secondary_key_permissions_full(&perms)]
        pub fn set_secondary_key_permissions(origin, key: T::AccountId, perms: Permissions) {
            Self::base_set_secondary_key_permissions(origin, key, perms)?;
        }

        /// It disables all secondary keys at `did` identity.
        ///
        /// # Errors
        ///
        #[weight = <T as Config>::WeightInfo::freeze_secondary_keys()]
        pub fn freeze_secondary_keys(origin) -> DispatchResult {
            Self::set_frozen_secondary_key_flags(origin, true)
        }

        /// Re-enables all secondary keys of the caller's identity.
        #[weight = <T as Config>::WeightInfo::unfreeze_secondary_keys()]
        pub fn unfreeze_secondary_keys(origin) -> DispatchResult {
            Self::set_frozen_secondary_key_flags(origin, false)
        }

        // Manage generic authorizations
        /// Adds an authorization.
        #[weight = <T as Config>::WeightInfo::add_authorization_full::<T::AccountId>(&data)]
        pub fn add_authorization(
            origin,
            target: Signatory<T::AccountId>,
            data: AuthorizationData<T::AccountId>,
            expiry: Option<T::Moment>
        ) {
            Self::base_add_authorization(origin, target, data, expiry)?;
        }

        /// Removes an authorization.
        /// _auth_issuer_pays determines whether the issuer of the authorisation pays the transaction fee
        #[weight = <T as Config>::WeightInfo::remove_authorization()]
        pub fn remove_authorization(
            origin,
            target: Signatory<T::AccountId>,
            auth_id: u64,
            _auth_issuer_pays: bool,
        ) {
            Self::base_remove_authorization(origin, target, auth_id)?;
        }

        /// It adds secondary keys to target identity `id`.
        /// Keys are directly added to identity because each of them has an authorization.
        ///
        /// Arguments:
        ///     - `origin` Primary key of `id` identity.
        ///     - `id` Identity where new secondary keys will be added.
        ///     - `additional_keys` New secondary items (and their authorization data) to add to target
        ///     identity.
        ///
        /// Failure
        ///     - It can only called by primary key owner.
        ///     - Keys should be able to linked to any identity.
        #[weight = <T as Config>::WeightInfo::add_secondary_keys_full::<T::AccountId>(&additional_keys)]
        pub fn add_secondary_keys_with_authorization(
            origin,
            additional_keys: Vec<SecondaryKeyWithAuth<T::AccountId>>,
            expires_at: T::Moment
        ) {
            Self::base_add_secondary_keys_with_authorization(origin, additional_keys, expires_at)?;
        }

        /// Add `Claim::InvestorUniqueness` claim for a given target identity.
        ///
        /// # <weight>
        ///  Weight of the this extrinsic is depend on the computation that used to validate
        ///  the proof of claim, which will be a constant independent of user inputs.
        /// # </weight>
        ///
        /// # Arguments
        /// * origin - Who provides the claim to the user? In this case, it's the user's account id as the user provides.
        /// * target - `IdentityId` to which the claim gets assigned.
        /// * claim - `InvestorUniqueness` claim details.
        /// * proof - To validate the self attestation.
        /// * expiry - Expiry of claim.
        ///
        /// # Errors
        /// * `DidMustAlreadyExist` Target should already been a part of the ecosystem.
        /// * `ClaimVariantNotAllowed` When origin trying to pass claim variant other than `InvestorUniqueness`.
        /// * `ConfidentialScopeClaimNotAllowed` When issuer is different from target or CDD_ID is invalid for given user.
        /// * `InvalidScopeClaim When proof is invalid.
        /// * `InvalidCDDId` when you are not the owner of that CDD_ID.
        #[weight = <T as Config>::WeightInfo::add_investor_uniqueness_claim()]
        pub fn add_investor_uniqueness_claim(origin, target: IdentityId, claim: Claim, proof: InvestorZKProofData, expiry: Option<T::Moment>) -> DispatchResult {
            Self::base_add_investor_uniqueness_claim(origin, target, claim, None, proof.into(), expiry)
        }

        /// Assuming this is executed by the GC voting majority, adds a new cdd claim record.
        #[weight = (<T as Config>::WeightInfo::add_claim(), Operational, Pays::Yes)]
        pub fn gc_add_cdd_claim(
            origin,
            target: IdentityId,
        ) {
            T::GCVotingMajorityOrigin::ensure_origin(origin)?;
            Self::add_systematic_cdd_claims(&[target], SystematicIssuers::Committee);
        }

        /// Assuming this is executed by the GC voting majority, removes an existing cdd claim record.
        #[weight = (<T as Config>::WeightInfo::add_claim(), Operational, Pays::Yes)]
        pub fn gc_revoke_cdd_claim(origin, target: IdentityId) -> DispatchResult {
            T::GCVotingMajorityOrigin::ensure_origin(origin)?;
            Self::base_revoke_claim(target, ClaimType::CustomerDueDiligence, GC_DID, None)
        }

        #[weight = <T as Config>::WeightInfo::add_investor_uniqueness_claim_v2()]
        pub fn add_investor_uniqueness_claim_v2(origin, target: IdentityId, scope: Scope, claim: Claim, proof: ScopeClaimProof, expiry: Option<T::Moment>) -> DispatchResult {
            Self::base_add_investor_uniqueness_claim(origin, target, claim, Some(scope), proof.into(), expiry)
        }

        /// Revokes a specific claim using its [Claim Unique Index](/pallet_identity/index.html#claim-unique-index) composed by `target`,
        /// `claim_type`, and `scope`.
        ///
        /// Please note that `origin` must be the issuer of the target claim.
        ///
        /// # Errors
        /// - `TargetHasNonZeroBalanceAtScopeId` when you try to revoke a `InvestorUniqueness*`
        /// claim, and `target` identity still have any balance on the given `scope`.
        #[weight = (<T as Config>::WeightInfo::revoke_claim_by_index(), revoke_claim_class(*claim_type))]
        pub fn revoke_claim_by_index(origin, target: IdentityId, claim_type: ClaimType, scope: Option<Scope>) -> DispatchResult {
            let issuer = Self::ensure_perms(origin)?;
            Self::base_revoke_claim(target, claim_type, issuer, scope)
        }

        /// Call this with the new primary key. By invoking this method, caller accepts authorization
        /// to become the new primary key of the issuing identity. If a CDD service provider approved
        /// this change, (or this is not required), primary key of the DID is updated.
        ///
        /// The caller (new primary key) must be either a secondary key of the issuing identity, or
        /// unlinked to any identity.
        ///
        /// Differs from accept_primary_key in that it will leave the old primary key as a secondary
        /// key with the permissions specified in the corresponding RotatePrimaryKeyToSecondary authorization
        /// instead of unlinking the old primary key.
        ///
        /// # Arguments
        /// * `owner_auth_id` Authorization from the owner who initiated the change
        /// * `cdd_auth_id` Authorization from a CDD service provider
        #[weight = <T as Config>::WeightInfo::rotate_primary_key_to_secondary()]
        pub fn rotate_primary_key_to_secondary(origin, auth_id:u64, optional_cdd_auth_id: Option<u64>) -> DispatchResult {
            Self::base_rotate_primary_key_to_secondary(origin, auth_id, optional_cdd_auth_id)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// One secondary or primary key can only belong to one DID
        AlreadyLinked,
        /// Missing current identity on the transaction
        MissingCurrentIdentity,
        /// Signatory is not pre authorized by the identity
        Unauthorized,
        /// Account Id cannot be extracted from signer
        InvalidAccountKey,
        /// Only CDD service providers are allowed.
        UnAuthorizedCddProvider,
        /// An invalid authorization from the owner.
        InvalidAuthorizationFromOwner,
        /// An invalid authorization from the CDD provider.
        InvalidAuthorizationFromCddProvider,
        /// Attestation was not by a CDD service provider.
        NotCddProviderAttestation,
        /// Authorizations are not for the same DID.
        AuthorizationsNotForSameDids,
        /// The DID must already exist.
        DidMustAlreadyExist,
        /// Current identity cannot be forwarded, it is not a secondary key of target identity.
        CurrentIdentityCannotBeForwarded,
        /// The offchain authorization has expired.
        AuthorizationExpired,
        /// The target DID has no valid CDD.
        TargetHasNoCdd,
        /// Authorization has been explicitly revoked.
        AuthorizationHasBeenRevoked,
        /// An invalid authorization signature.
        InvalidAuthorizationSignature,
        /// This key is not allowed to execute a given operation.
        KeyNotAllowed,
        /// Only the primary key is allowed to revoke an Identity Signatory off-chain authorization.
        NotPrimaryKey,
        /// The DID does not exist.
        DidDoesNotExist,
        /// The DID already exists.
        DidAlreadyExists,
        /// The secondary keys contain the primary key.
        SecondaryKeysContainPrimaryKey,
        /// Couldn't charge fee for the transaction.
        FailedToChargeFee,
        /// Signer is not a secondary key of the provided identity
        NotASigner,
        /// Cannot convert a `T::AccountId` to `AnySignature::Signer::AccountId`.
        CannotDecodeSignerAccountId,
        /// Multisig can not be unlinked from an identity while it still holds POLYX
        MultiSigHasBalance,
        /// Confidential Scope claims can be added by an Identity to it-self.
        ConfidentialScopeClaimNotAllowed,
        /// Addition of a new scope claim gets invalidated.
        InvalidScopeClaim,
        /// Try to add a claim variant using un-designated extrinsic.
        ClaimVariantNotAllowed,
        /// Try to delete the IU claim even when the user has non zero balance at given scopeId.
        TargetHasNonZeroBalanceAtScopeId,
        /// CDDId should be unique & same within all cdd claims possessed by a DID.
        CDDIdNotUniqueForIdentity,
        /// Non systematic CDD providers can not create default cdd_id claims.
        InvalidCDDId,
        /// Claim and Proof versions are different.
        ClaimAndProofVersionsDoNotMatch,
        /// The account key is being used, it can't be unlinked.
        AccountKeyIsBeingUsed,
        /// A custom scope is too long.
        /// It can at most be `32` characters long.
        CustomScopeTooLong,
    }
}

impl<T: Config> Module<T> {
    /// Only used by `create_asset` since `AssetDidRegistered` is defined here instead of there.
    pub fn commit_token_did(did: IdentityId, ticker: Ticker) {
        DidRecords::<T>::insert(did, DidRecord::default());
        Self::deposit_event(RawEvent::AssetDidRegistered(did, ticker));
    }

    /// IMPORTANT: No state change is allowed in this function
    /// because this function is used within the RPC calls
    /// It is a helper function that can be used to get did for any asset
    pub fn get_token_did(ticker: &Ticker) -> Result<IdentityId, &'static str> {
        let mut buf = SECURITY_TOKEN.encode();
        buf.append(&mut ticker.encode());
        IdentityId::try_from(T::Hashing::hash(&buf[..]).as_ref())
    }

    pub fn get_did_status(dids: Vec<IdentityId>) -> Vec<DidStatus> {
        dids.into_iter()
            .map(|did| {
                // Does DID exist in the ecosystem?
                if !DidRecords::<T>::contains_key(did) {
                    DidStatus::Unknown
                }
                // DID exists, but does it have a valid CDD?
                else if Self::has_valid_cdd(did) {
                    DidStatus::CddVerified
                } else {
                    DidStatus::Exists
                }
            })
            .collect()
    }

    #[cfg(feature = "runtime-benchmarks")]
    /// Links a did with an identity
    pub fn link_did(account: T::AccountId, did: IdentityId) {
        Self::add_key_record(&account, KeyRecord::PrimaryKey(did));
    }

    #[cfg(feature = "runtime-benchmarks")]
    /// Sets the current did in the context
    pub fn set_context_did(did: Option<IdentityId>) {
        polymesh_common_utilities::Context::set_current_identity::<Self>(did);
    }
}

impl<T: Config> IdentityFnTrait<T::AccountId> for Module<T> {
    /// Fetches identity of a key.
    fn get_identity(key: &T::AccountId) -> Option<IdentityId> {
        Self::get_identity(key)
    }

    /// Fetches the caller's identity from the context.
    fn current_identity() -> Option<IdentityId> {
        CurrentDid::get()
    }

    /// Sets the caller's identity in the context.
    fn set_current_identity(id: Option<IdentityId>) {
        if let Some(id) = id {
            CurrentDid::put(id);
        } else {
            CurrentDid::kill();
        }
    }

    /// Fetches the fee payer from the context.
    fn current_payer() -> Option<T::AccountId> {
        <CurrentPayer<T>>::get()
    }

    /// Sets the fee payer in the context.
    fn set_current_payer(payer: Option<T::AccountId>) {
        if let Some(payer) = payer {
            <CurrentPayer<T>>::put(payer);
        } else {
            <CurrentPayer<T>>::kill();
        }
    }

    /// Provides the DID status for the given DID
    fn has_valid_cdd(target_did: IdentityId) -> bool {
        Self::has_valid_cdd(target_did)
    }
}

impl<T: Config> ChangeMembers<IdentityId> for Module<T> {
    /// Updates systematic CDDs of members of a group.
    fn change_members_sorted(
        incoming: &[IdentityId],
        outgoing: &[IdentityId],
        _new: &[IdentityId],
    ) {
        // Add/remove Systematic CDD claims for new/removed members.
        let issuer = SystematicIssuers::CDDProvider;
        Self::add_systematic_cdd_claims(incoming, issuer);
        Self::revoke_systematic_cdd_claims(outgoing, issuer);
    }
}

impl<T: Config> InitializeMembers<IdentityId> for Module<T> {
    /// Initializes members of a group by adding systematic claims for them.
    fn initialize_members(members: &[IdentityId]) {
        Self::add_systematic_cdd_claims(members, SystematicIssuers::CDDProvider);
    }
}

/// A `revoke_claim` or `revoke_claim_by_index` TX is operational iff `claim_type` is a `Claim::CustomerDueDiligence`.
/// Otherwise, it will be a normal transaction.
fn revoke_claim_class(claim_type: ClaimType) -> frame_support::weights::DispatchClass {
    match claim_type {
        ClaimType::CustomerDueDiligence => Operational,
        _ => Normal,
    }
}

pub mod migration {
    use super::*;

    mod v1 {
        use super::*;
        use polymesh_primitives::secondary_key::v1;
        use scale_info::TypeInfo;

        /// Old v1 Identity information.
        #[derive(Encode, Decode, TypeInfo)]
        #[derive(Clone, Default, PartialEq)]
        pub struct IdentityRecord<AccountId> {
            pub primary_key: AccountId,
            pub secondary_keys: Vec<v1::SecondaryKey<AccountId>>,
        }

        decl_storage! {
            trait Store for Module<T: Config> as Identity {
                pub DidRecords get(fn did_records): map hasher(identity) IdentityId => IdentityRecord<T::AccountId>;
                pub KeyToIdentityIds get(fn key_to_identity_dids):
                    map hasher(twox_64_concat) T::AccountId => IdentityId;
            }
        }

        decl_module! {
            pub struct Module<T: Config> for enum Call where origin: T::Origin { }
        }
    }

    pub fn migrate_v1_key<T: Config>(key: T::AccountId, record: KeyRecord<T::AccountId>) {
        // Add key to record mapping.
        KeyRecords::<T>::insert(&key, &record);
        // For primary/secondary keys add to `DidKeys`.
        if let Some((did, is_primary_key)) = record.get_did_key_type() {
            DidKeys::<T>::insert(did, &key, true);
            // For primary keys also set the DID record.
            if is_primary_key {
                DidRecords::<T>::insert(did, DidRecord::new(key));
            }
        }
    }

    pub fn migrate_v1<T: Config>() {
        sp_runtime::runtime_logger::RuntimeLogger::init();

        log::info!(" >>> Updating Identity storage. Migrating DidRecords...");
        let (total_dids, total_sks) = v1::DidRecords::<T>::drain().fold(
            (0usize, 0usize),
            |(total_dids, total_sks), (did, mut record)| {
                // Migrate primary key.
                if record.primary_key == Default::default() {
                    // Asset identities don't have primary keys.
                    DidRecords::<T>::insert(did, DidRecord::default());
                } else {
                    migrate_v1_key::<T>(record.primary_key, KeyRecord::PrimaryKey(did));
                }

                // Migrate secondary keys.
                let sk_count = record.secondary_keys.drain(..).fold(0usize, |total, sk| {
                    if let Some((key, key_record)) = sk.into_key_record(did) {
                        migrate_v1_key::<T>(key, key_record);
                    }
                    total + 1
                });

                (total_dids + 1, total_sks + sk_count)
            },
        );
        log::info!(
            " >>> Migrated {} Identities and {} secondary keys.",
            total_dids,
            total_sks
        );

        log::info!(" >>> Removing KeyToIdentityIds...");
        use frame_support::storage::child::KillStorageResult::*;
        let (num_removed, all_removed) = match v1::KeyToIdentityIds::<T>::remove_all(None) {
            AllRemoved(removed) => (removed, true),
            SomeRemaining(removed) => (removed, false),
        };
        log::info!(
            " >>> Removed {} KeyToIdentityIds, removed all: {}.",
            num_removed,
            all_removed
        );
    }
}
