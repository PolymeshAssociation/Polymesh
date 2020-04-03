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
//! target identity as signing key, where we also define the type of account (`External`,
//! `MultiSign`, etc.) and/or its permission.
//!
//! Some operations at identity level are only allowed to its administrator account, like
//! [set_master_key](./struct.Module.html#method.set_master_key) or
//!
//! ## Identity information
//!
//! Identity contains the following data:
//!  - `master_key`. It is the administrator account of the identity.
//!  - `signing_keys`. List of keys and their capabilities (type of key and its permissions) .
//!
//! ## Freeze signing keys
//!
//! It is an *emergency action* to block all signing keys of an identity and it can only be performed
//! by its administrator.
//!
//! see [freeze_signing_keys](./struct.Module.html#method.freeze_signing_keys)
//! see [unfreeze_signing_keys](./struct.Module.html#method.unfreeze_signing_keys)
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use polymesh_primitives::{
    AccountKey, AuthIdentifier, Authorization, AuthorizationData, AuthorizationError, Claim,
    ClaimType, Identity as DidRecord, IdentityClaim, IdentityId, Link, LinkData, Permission,
    PreAuthorizedKeyInfo, Scope, Signatory, SignatoryType, SigningItem, Ticker,
};
use polymesh_runtime_common::{
    constants::did::{CDD_PROVIDERS_ID, GOVERNANCE_COMMITTEE_ID, SECURITY_TOKEN, USER},
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    traits::{
        asset::AcceptTransfer,
        group::{GroupTrait, InactiveMember},
        identity::{
            AuthorizationNonce, LinkedKeyInfo, RawEvent, SigningItemWithAuth, TargetIdAuthorization,
        },
        multisig::AddSignerMultiSig,
    },
    Context, SystematicIssuers,
};

use codec::{Decode, Encode};
use core::{
    convert::{From, TryInto},
    result::Result as StdResult,
};
use sp_core::sr25519::{Public, Signature};
use sp_io::hashing::blake2_256;
use sp_runtime::{
    traits::{CheckedAdd, Dispatchable, Hash, SaturatedConversion, Verify, Zero},
    AnySignature,
};
use sp_std::{convert::TryFrom, mem::swap, prelude::*, vec};

use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{ChangeMembers, InitializeMembers},
    weights::{GetDispatchInfo, SimpleDispatchInfo},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use pallet_transaction_payment::{CddAndFeeDetails, ChargeTxFee};
use polymesh_runtime_identity_rpc_runtime_api::DidRecords as RpcDidRecords;

pub use polymesh_runtime_common::traits::identity::{IdentityTrait, Trait};

pub type Event<T> = polymesh_runtime_common::traits::identity::Event<T>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Claim1stKey {
    pub target: IdentityId,
    pub claim_type: ClaimType,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Claim2ndKey {
    pub issuer: IdentityId,
    pub scope: Option<Scope>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Default)]
pub struct BatchAddClaimItem<M> {
    pub target: IdentityId,
    pub claim: Claim,
    pub expiry: Option<M>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Default)]
pub struct BatchRevokeClaimItem {
    pub target: IdentityId,
    pub claim: Claim,
}

decl_storage! {
    trait Store for Module<T: Trait> as identity {

        /// Module owner.
        Owner get(fn owner) config(): T::AccountId;

        /// DID -> identity info
        pub DidRecords get(fn did_records) config(): map hasher(blake2_256) IdentityId => DidRecord;

        /// DID -> bool that indicates if signing keys are frozen.
        pub IsDidFrozen get(fn is_did_frozen): map hasher(blake2_256) IdentityId => bool;

        /// It stores the current identity for current transaction.
        pub CurrentDid: Option<IdentityId>;

        /// It stores the current gas fee payer for the current transaction
        pub CurrentPayer: Option<Signatory>;

        /// (Target ID, claim type) (issuer,scope) -> Associated claims
        pub Claims: double_map hasher(blake2_256) Claim1stKey, hasher(blake2_256) Claim2ndKey => IdentityClaim;

        // Account => DID
        pub KeyToIdentityIds get(fn key_to_identity_ids) config(): map hasher(blake2_256) AccountKey => Option<LinkedKeyInfo>;

        /// Nonce to ensure unique actions. starts from 1.
        pub MultiPurposeNonce get(fn multi_purpose_nonce) build(|_| 1u64): u64;

        /// Pre-authorize join to Identity.
        pub PreAuthorizedJoinDid get(fn pre_authorized_join_did): map hasher(blake2_256) Signatory => Vec<PreAuthorizedKeyInfo>;

        /// Authorization nonce per Identity. Initially is 0.
        pub OffChainAuthorizationNonce get(fn offchain_authorization_nonce): map hasher(blake2_256) IdentityId => AuthorizationNonce;

        /// Inmediate revoke of any off-chain authorization.
        pub RevokeOffChainAuthorization get(fn is_offchain_authorization_revoked): map hasher(blake2_256) (Signatory, TargetIdAuthorization<T::Moment>) => bool;

        /// All authorizations that an identity/key has
        pub Authorizations: double_map hasher(blake2_256) Signatory, hasher(blake2_256) u64 => Authorization<T::Moment>;

        /// All links that an identity/key has
        pub Links: double_map hasher(blake2_256) Signatory, hasher(blake2_256) u64 => Link<T::Moment>;

        /// All authorizations that an identity/key has given. (Authorizer, auth_id -> authorized)
        pub AuthorizationsGiven: double_map hasher(blake2_256) Signatory, hasher(blake2_256) u64 => Signatory;

        /// It defines if authorization from a CDD provider is needed to change master key of an identity
        pub CddAuthForMasterKeyRotation get(fn cdd_auth_for_master_key_rotation): bool;
    }
    add_extra_genesis {
        config(identities): Vec<(T::AccountId, IdentityId, IdentityId, Option<u64>)>;
        build(|config: &GenesisConfig<T>| {
            // Add System DID: Governance committee && CDD providers
            [GOVERNANCE_COMMITTEE_ID, CDD_PROVIDERS_ID].iter()
                .for_each(|raw_id| {
                    let id = IdentityId::from(**raw_id);
                    let master_key = AccountKey::from(**raw_id);

                    <DidRecords>::insert( id, DidRecord {
                        master_key,
                        ..Default::default()
                    });
                });

            //  Other
            for &(ref master_account_id, issuer, did, expiry) in &config.identities {
                // Direct storage change for registering the DID and providing the claim
                let master_key = AccountKey::try_from(master_account_id.encode()).unwrap();
                assert!(!<DidRecords>::contains_key(did), "Identity already exist");
                <MultiPurposeNonce>::mutate(|n| *n += 1_u64);
                <Module<T>>::link_key_to_did(&master_key, SignatoryType::External, did);
                let record = DidRecord {
                    master_key,
                    ..Default::default()
                };
                <DidRecords>::insert(&did, record);

                // Add the claim data for the CustomerDueDiligence type claim
                let claim_type = ClaimType::CustomerDueDiligence;
                let pk = Claim1stKey{ target: did, claim_type };
                let sk = Claim2ndKey{ issuer, scope: None };
                let id_claim = IdentityClaim {
                    claim_issuer: issuer,
                    issuance_date: 0_u64,
                    last_update_date: 0_u64,
                    expiry: expiry,
                    claim: Claim::CustomerDueDiligence,
                };

                <Claims>::insert(&pk, &sk, id_claim);
            }
            // TODO: Generate CDD for BRR
        });
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        // TODO: Remove this function. cdd_register_did should be used instead.
        pub fn register_did(origin, signing_items: Vec<SigningItem>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let signer = Signatory::from(AccountKey::try_from(sender.encode())?);
            let new_id = Self::_register_did(
                sender,
                signing_items,
                Some((&signer, ProtocolOp::IdentityRegisterDid))
            )?;
            // Added for easier testing. To be removed before production
            let cdd_providers = T::CddServiceProviders::get_members();
            if cdd_providers.len() > 0 {
                Self::unsafe_add_claim(new_id, Claim::CustomerDueDiligence, cdd_providers[0], None);
            }
            Ok(())
        }

        /// Register `target_account` with a new Identity.
        ///
        /// # Failure
        /// - `origin` has to be a active CDD provider. Inactive CDD providers cannot add new
        /// claims.
        /// - `target_account` (master key of the new Identity) can be linked to just one and only
        /// one identity.
        /// - External signing keys can be linked to just one identity.
        ///
        /// # TODO
        /// - Imbalance: Since we are not handling the imbalance here, this will leave a hold in
        ///     the total supply. We are reducing someone's balance but not increasing anyone's
        ///     else balance or decreasing total supply. This will mean that the sum of all
        ///     balances will become less than the total supply.
        pub fn cdd_register_did(
            origin,
            target_account: T::AccountId,
            cdd_claim_expiry: Option<T::Moment>,
            signing_items: Vec<SigningItem>
        ) -> DispatchResult {
            // Sender has to be part of CDDProviders
            let cdd_sender = ensure_signed(origin)?;
            let cdd_key = AccountKey::try_from(cdd_sender.encode())?;
            let cdd_id = Context::current_identity_or::<Self>(&cdd_key)?;

            let cdd_providers = T::CddServiceProviders::get_members();
            ensure!(cdd_providers.contains(&cdd_id), Error::<T>::UnAuthorizedCddProvider);
            // Register Identity and add claim.
            let new_id = Self::_register_did(
                target_account,
                signing_items,
                Some((&Signatory::AccountKey(cdd_key), ProtocolOp::IdentityCddRegisterDid))
            )?;
            Self::unsafe_add_claim(new_id, Claim::CustomerDueDiligence, cdd_id, cdd_claim_expiry);
            Ok(())
        }

        /// It invalidates any claim generated by `cdd` from `disable_from` timestamps.
        /// You can also define an expiration time, which will invalidate all claims generated by
        /// that `cdd` and remove it as CDD member group.
        pub fn invalidate_cdd_claims(
            origin,
            cdd: IdentityId,
            disable_from: T::Moment,
            expiry: Option<T::Moment>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(
                T::CddServiceProviders::get_valid_members_at(now).contains(&cdd),
                Error::<T>::UnAuthorizedCddProvider);

            T::CddServiceProviders::disable_member( cdd, expiry, Some(disable_from))
        }

        /// Removes specified signing keys of a DID if present.
        ///
        /// # Failure
        /// It can only called by master key owner.
        pub fn remove_signing_items(origin, signers_to_remove: Vec<Signatory>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Self>(&sender_key)?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            // Remove any Pre-Authentication & link
            signers_to_remove.iter().for_each( |signer| {
                Self::remove_pre_join_identity( signer, did);
                if let Signatory::AccountKey(ref key) = signer {
                    Self::unlink_key_to_did(key, did);
                }
            });

            // Update signing keys at Identity.
            <DidRecords>::mutate(did, |record| {
                (*record).remove_signing_items( &signers_to_remove);
            });

            Self::deposit_event(RawEvent::RevokedSigningItems(did, signers_to_remove));
            Ok(())
        }

        /// Sets a new master key for a DID.
        ///
        /// # Failure
        /// Only called by master key owner.
        fn set_master_key(origin, new_key: AccountKey) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from( sender.encode())?;
            let did = Context::current_identity_or::<Self>(&sender_key)?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            ensure!(
                Self::can_key_be_linked_to_did(&new_key, SignatoryType::External),
                Error::<T>::AlreadyLinked
            );
            T::ProtocolFee::charge_fee(
                &Signatory::AccountKey(sender_key),
                ProtocolOp::IdentitySetMasterKey
            )?;
            <DidRecords>::mutate(did,
            |record| {
                (*record).master_key = new_key;
            });

            Self::deposit_event(RawEvent::NewMasterKey(did, sender, new_key));
            Ok(())
        }

        /// Call this with the new master key. By invoking this method, caller accepts authorization
        /// with the new master key. If a CDD service provider approved this change, master key of
        /// the DID is updated.
        ///
        /// # Arguments
        /// * `owner_auth_id` Authorization from the owner who initiated the change
        /// * `cdd_auth_id` Authorization from a CDD service provider
        pub fn accept_master_key(origin, rotation_auth_id: u64, optional_cdd_auth_id: Option<u64>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            Self::accept_master_key_rotation(sender_key, rotation_auth_id, optional_cdd_auth_id)
        }

        /// Set if CDD authorization is required for updating master key of an identity.
        /// Callable via root (governance)
        ///
        /// # Arguments
        /// * `auth_required` CDD Authorization required or not
        pub fn change_cdd_requirement_for_mk_rotation(
            origin,
            auth_required: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <CddAuthForMasterKeyRotation>::put(auth_required);
            Self::deposit_event(RawEvent::CddRequirementForMasterKeyUpdated(auth_required));
            Ok(())
        }

        /// Join an identity as a signing key
        pub fn join_identity_as_key(origin, auth_id: u64) -> DispatchResult {
            let signer = Signatory::from(AccountKey::try_from(ensure_signed(origin)?.encode())?);
            Self::join_identity(signer, auth_id)
        }

        /// Join an identity as a signing identity
        pub fn join_identity_as_identity(origin, auth_id: u64) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let sender_did = Context::current_identity_or::<Self>(&sender_key)?;
            Self::join_identity(Signatory::from(sender_did), auth_id)
        }

        /// Adds new claim record or edits an existing one. Only called by did_issuer's signing key
        #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        pub fn add_claim(
            origin,
            target: IdentityId,
            claim: Claim,
            expiry: Option<T::Moment>,
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let issuer = Context::current_identity_or::<Self>(&sender_key)?;
            ensure!(<DidRecords>::contains_key(target), Error::<T>::DidMustAlreadyExist);

            match claim {
                Claim::CustomerDueDiligence => Self::unsafe_add_cdd_claim(target, claim, issuer, expiry)?,
                _ => {
                    T::ProtocolFee::charge_fee(
                    &Signatory::AccountKey(sender_key),
                    ProtocolOp::IdentityAddClaim
                    )?;
                    Self::unsafe_add_claim(target, claim, issuer, expiry)
                }
            };
            Ok(())
        }

        /// Adds a new batch of claim records or edits an existing one. Only called by
        /// `did_issuer`'s signing key.
        // TODO: fix #[weight = BatchDispatchInfo::new_normal(3_000, 10_000)]
        pub fn add_claims_batch(
            origin,
            claims: Vec<BatchAddClaimItem<T::Moment>>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let issuer = Context::current_identity_or::<Self>(&sender_key)?;
            // Check input claims.
            ensure!( claims.iter().all(
                |batch_claim_item| <DidRecords>::contains_key(batch_claim_item.target)),
                Error::<T>::DidMustAlreadyExist);

            T::ProtocolFee::charge_fee_batch(
                &Signatory::AccountKey(sender_key),
                ProtocolOp::IdentityAddClaim,
                claims.len()
            )?;
            claims
                .into_iter()
                .for_each(|bci| {
                    Self::unsafe_add_claim(bci.target, bci.claim, issuer, bci.expiry)
                });
            Ok(())
        }

        fn forwarded_call(origin, target_did: IdentityId, proposal: Box<T::Proposal>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // 1. Constraints.
            // 1.1. A valid current identity.
            if let Some(current_did) = Context::current_identity::<Self>() {
                // 1.2. Check that current_did is a signing key of target_did
                ensure!(
                    Self::is_signer_authorized(current_did, &Signatory::Identity(target_did)),
                    Error::<T>::CurrentIdentityCannotBeForwarded
                );
            } else {
                return Err(Error::<T>::MissingCurrentIdentity.into());
            }

            // 1.3. Check that target_did has a CDD.
            ensure!(Self::has_valid_cdd(target_did), Error::<T>::TargetHasNoCdd);

            // 1.4 charge fee
            ensure!(
                T::ChargeTxFeeTarget::charge_fee(
                    proposal.encode().len().try_into().unwrap_or_default(),
                    proposal.get_dispatch_info(),
                )
                .is_ok(),
                Error::<T>::FailedToChargeFee
            );

            // 2. Actions
            T::CddHandler::set_current_identity(&target_did);

            // Also set current_did roles when acting as a signing key for target_did
            // Re-dispatch call - e.g. to asset::doSomething...
            let new_origin = frame_system::RawOrigin::Signed(sender).into();

            let _res = match proposal.dispatch(new_origin) {
                Ok(_) => true,
                Err(e) => {
                    let e: DispatchError = e;
                    sp_runtime::print(e);
                    false
                }
            };

            Ok(())
        }

        /// Marks the specified claim as revoked
        pub fn revoke_claim(origin,
            target: IdentityId,
            claim: Claim,
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;
            let issuer = Context::current_identity_or::<Self>(&sender_key)?;
            let claim_type = claim.claim_type();
            let scope = claim.as_scope().cloned();

            Self::unsafe_revoke_claim(target, claim_type, issuer, scope);
            Ok(())
        }

        /// Revoke multiple claims in a batch
        ///
        /// # Arguments
        /// * origin - did issuer
        /// * did_and_claim_data - Vector of the identities & the corresponding claim data whom claim needs to be revoked
        pub fn revoke_claims_batch(origin,
            claims: Vec<BatchRevokeClaimItem>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;
            let issuer = Context::current_identity_or::<Self>(&sender_key)?;

            claims.into_iter()
                .for_each( |bci| {
                    let claim_type = bci.claim.claim_type();
                    let scope = bci.claim.as_scope().cloned();
                    Self::unsafe_revoke_claim(bci.target, claim_type, issuer, scope)
                });
            Ok(())
        }

        /// It sets permissions for an specific `target_key` key.
        /// Only the master key of an identity is able to set signing key permissions.
        pub fn set_permission_to_signer(origin, signer: Signatory, permissions: Vec<Permission>) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Self>(&sender_key)?;
            let record = Self::grant_check_only_master_key( &sender_key, did)?;

            // You are trying to add a permission to did's master key. It is not needed.
            if let Signatory::AccountKey(ref key) = signer {
                if record.master_key == *key {
                    return Ok(());
                }
            }

            // Find key in `DidRecord::signing_keys`
            if record.signing_items.iter().any(|si| si.signer == signer) {
                Self::update_signing_item_permissions(did, &signer, permissions)
            } else {
                Err(Error::<T>::InvalidSender.into())
            }
        }

        /// It disables all signing keys at `did` identity.
        ///
        /// # Errors
        ///
        pub fn freeze_signing_keys(origin) -> DispatchResult {
            Self::set_frozen_signing_key_flags(origin, true)
        }

        pub fn unfreeze_signing_keys(origin) -> DispatchResult {
            Self::set_frozen_signing_key_flags(origin, false)
        }

        pub fn get_my_did(origin) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Self>(&sender_key)?;

            Self::deposit_event(RawEvent::DidQuery(sender_key, did));
            Ok(())
        }

        pub fn get_cdd_of(_origin, of: T::AccountId) -> DispatchResult {
            let key = AccountKey::try_from(of.encode())?;
            if let Some(did) = Self::get_identity(&key) {
                let cdd = Self::has_valid_cdd(did);
                Self::deposit_event(RawEvent::CddQuery(key, did, cdd));
            }
            Ok(())
        }

        // Manage generic authorizations
        /// Adds an authorization
        pub fn add_authorization(
            origin,
            target: Signatory,
            authorization_data: AuthorizationData,
            expiry: Option<T::Moment>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let from_did = Context::current_identity_or::<Self>(&sender_key)?;

            Self::add_auth(Signatory::from(from_did), target, authorization_data, expiry);

            Ok(())
        }

        /// Adds an authorization as a key.
        /// To be used by signing keys that don't have an identity
        pub fn add_authorization_as_key(
            origin,
            target: Signatory,
            authorization_data: AuthorizationData,
            expiry: Option<T::Moment>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;

            Self::add_auth(Signatory::from(sender_key), target, authorization_data, expiry);

            Ok(())
        }

        // Manage generic authorizations
        /// Adds an array of authorization
        pub fn batch_add_authorization(
            origin,
            // Vec<(target_did, auth_data, expiry)>
            auths: Vec<(Signatory, AuthorizationData, Option<T::Moment>)>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let from_did = Context::current_identity_or::<Self>(&sender_key)?;

            for auth in auths {
                Self::add_auth(Signatory::from(from_did), auth.0, auth.1, auth.2);
            }

            Ok(())
        }

        /// Removes an authorization
        pub fn remove_authorization(
            origin,
            target: Signatory,
            auth_id: u64
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let from_did = Context::current_identity_or::<Self>(&sender_key)?;

            ensure!(
                <Authorizations<T>>::contains_key(target, auth_id),
                Error::<T>::AuthorizationDoesNotExist
            );
            let auth = <Authorizations<T>>::get(target, auth_id);
            ensure!(
                auth.authorized_by.eq_either(&from_did, &sender_key) ||
                    target.eq_either(&from_did, &sender_key),
                Error::<T>::Unauthorized
            );
            Self::remove_auth(target, auth_id, auth.authorized_by);

            Ok(())
        }

        /// Removes an array of authorizations
        pub fn batch_remove_authorization(
            origin,
            auth_identifiers: Vec<AuthIdentifier>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let from_did = Context::current_identity_or::<Self>(&sender_key)?;
            let mut auths = Vec::with_capacity(auth_identifiers.len());
            for i in 0..auth_identifiers.len() {
                let auth_identifier = &auth_identifiers[i];
                ensure!(
                    <Authorizations<T>>::contains_key(&auth_identifier.0, &auth_identifier.1),
                    Error::<T>::AuthorizationDoesNotExist
                );
                auths.push(<Authorizations<T>>::get(&auth_identifier.0, &auth_identifier.1));
                ensure!(
                    auths[i].authorized_by.eq_either(&from_did, &sender_key) ||
                        auth_identifier.0.eq_either(&from_did, &sender_key),
                    Error::<T>::Unauthorized
                );
            }

            for i in 0..auth_identifiers.len() {
                let auth_identifier = &auth_identifiers[i];
                Self::remove_auth(auth_identifier.0, auth_identifier.1, auths[i].authorized_by);

            }

            Ok(())
        }

        /// Accepts an authorization
        pub fn accept_authorization(
            origin,
            auth_id: u64
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let signer = Context::current_identity_or::<Self>(&sender_key)
                .map_or_else(
                    |_error| Signatory::from(sender_key),
                    Signatory::from);
            ensure!(
                <Authorizations<T>>::contains_key(signer, auth_id),
                Error::<T>::AuthorizationDoesNotExist
            );
            let auth = <Authorizations<T>>::get(signer, auth_id);
            match signer {
                Signatory::Identity(did) => {
                    match auth.authorization_data {
                        AuthorizationData::TransferTicker(_) =>
                            T::AcceptTransferTarget::accept_ticker_transfer(did, auth_id),
                        AuthorizationData::TransferTokenOwnership(_) =>
                            T::AcceptTransferTarget::accept_token_ownership_transfer(did, auth_id),
                        AuthorizationData::AddMultiSigSigner =>
                            T::AddSignerMultiSigTarget::accept_multisig_signer(Signatory::from(did), auth_id),
                        AuthorizationData::JoinIdentity(_) =>
                            Self::join_identity(Signatory::from(did), auth_id),
                        _ => Err(Error::<T>::UnknownAuthorization.into())
                    }
                },
                Signatory::AccountKey(key) => {
                    match auth.authorization_data {
                        AuthorizationData::AddMultiSigSigner =>
                            T::AddSignerMultiSigTarget::accept_multisig_signer(Signatory::from(key), auth_id),
                        AuthorizationData::RotateMasterKey(_identityid) =>
                            Self::accept_master_key_rotation(key , auth_id, None),
                        AuthorizationData::JoinIdentity(_) =>
                            Self::join_identity(Signatory::from(key), auth_id),
                        _ => Err(Error::<T>::UnknownAuthorization.into())
                    }
                }
            }
        }

        /// Accepts an array of authorizations
        pub fn batch_accept_authorization(
            origin,
            auth_ids: Vec<u64>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let signer = Context::current_identity_or::<Self>(&sender_key)
                .map_or_else(
                    |_error| Signatory::from(sender_key),
                    Signatory::from);

            match signer {
                Signatory::Identity(did) => {
                    for auth_id in auth_ids {
                        // NB: Even if an auth is invalid (due to any reason), this batch function does NOT return an error.
                        // It will just skip that particular authorization.

                        if <Authorizations<T>>::contains_key(signer, auth_id) {
                            let auth = <Authorizations<T>>::get(signer, auth_id);

                            // NB: Result is not handled, invalid auths are just ignored to let the batch function continue.
                            let _result = match auth.authorization_data {
                                AuthorizationData::TransferTicker(_) =>
                                    T::AcceptTransferTarget::accept_ticker_transfer(did, auth_id),
                                AuthorizationData::TransferTokenOwnership(_) =>
                                    T::AcceptTransferTarget::accept_token_ownership_transfer(did, auth_id),
                                AuthorizationData::AddMultiSigSigner =>
                                    T::AddSignerMultiSigTarget::accept_multisig_signer(Signatory::from(did), auth_id),
                                AuthorizationData::JoinIdentity(_) =>
                                    Self::join_identity(Signatory::from(did), auth_id),
                                _ => Err(Error::<T>::UnknownAuthorization.into())
                            };
                        }
                    }
                },
                Signatory::AccountKey(key) => {
                    for auth_id in auth_ids {
                        // NB: Even if an auth is invalid (due to any reason), this batch function does NOT return an error.
                        // It will just skip that particular authorization.

                        if <Authorizations<T>>::contains_key(signer, auth_id) {
                            let auth = <Authorizations<T>>::get(signer, auth_id);

                            //NB: Result is not handled, invalid auths are just ignored to let the batch function continue.
                            let _result = match auth.authorization_data {
                                AuthorizationData::AddMultiSigSigner =>
                                    T::AddSignerMultiSigTarget::accept_multisig_signer(Signatory::from(key), auth_id),
                                AuthorizationData::RotateMasterKey(_identityid) =>
                                    Self::accept_master_key_rotation(key , auth_id, None),
                                AuthorizationData::JoinIdentity(_) =>
                                    Self::join_identity(Signatory::from(key), auth_id),
                                _ => Err(Error::<T>::UnknownAuthorization.into())
                            };
                        }
                    }
                }
            }

            Ok(())
        }

        /// It adds signing keys to target identity `id`.
        /// Keys are directly added to identity because each of them has an authorization.
        ///
        /// Arguments:
        ///     - `origin` Master key of `id` identity.
        ///     - `id` Identity where new signing keys will be added.
        ///     - `additional_keys` New signing items (and their authorization data) to add to target
        ///     identity.
        ///
        /// Failure
        ///     - It can only called by master key owner.
        ///     - Keys should be able to linked to any identity.
        pub fn add_signing_items_with_authorization(
            origin,
            expires_at: T::Moment,
            additional_keys: Vec<SigningItemWithAuth>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let id = Context::current_identity_or::<Self>(&sender_key)?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, id)?;

            // 0. Check expiration
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(now < expires_at, Error::<T>::AuthorizationExpired);
            let authorization = TargetIdAuthorization {
                target_id: id,
                nonce: Self::offchain_authorization_nonce(id),
                expires_at
            };
            let auth_encoded= authorization.encode();

            // 1. Verify signatures.
            for si_with_auth in additional_keys.iter() {
                let si = &si_with_auth.signing_item;

                // Get account_id from signer
                let account_id_found = match si.signer {
                    Signatory::AccountKey(ref key) =>  Public::try_from(key.as_slice()).ok(),
                    Signatory::Identity(ref id) if <DidRecords>::contains_key(id) => {
                        let master_key = <DidRecords>::get(id).master_key;
                        Public::try_from( master_key.as_slice()).ok()
                    },
                    _ => None
                };

                if let Some(account_id) = account_id_found {
                    if let Signatory::AccountKey(ref key) = si.signer {
                        // 1.1. Constraint 1-to-1 account to DID
                        ensure!(
                            Self::can_key_be_linked_to_did(key, si.signer_type),
                            Error::<T>::AlreadyLinked
                        );
                    }
                    // 1.2. Offchain authorization is not revoked explicitly.
                    let si_signer_authorization = &(si.signer, authorization.clone());
                    ensure!(
                        !Self::is_offchain_authorization_revoked(si_signer_authorization),
                        Error::<T>::AuthorizationHasBeenRevoked
                    );
                    // 1.3. Verify the signature.
                    let signature = AnySignature::from( Signature::from_h512(si_with_auth.auth_signature));
                    ensure!(
                        signature.verify(auth_encoded.as_slice(), &account_id),
                        Error::<T>::InvalidAuthorizationSignature
                    );
                } else {
                    return Err(Error::<T>::InvalidAccountKey.into());
                }
            }
            // 1.999. Charge the fee.
            T::ProtocolFee::charge_fee_batch(
                &Signatory::AccountKey(sender_key),
                ProtocolOp::IdentityAddSigningItem,
                additional_keys.len()
            )?;
            // 2.1. Link keys to identity
            additional_keys.iter().for_each( |si_with_auth| {
                let si = &si_with_auth.signing_item;
                if let Signatory::AccountKey(ref key) = si.signer {
                    Self::link_key_to_did( key, si.signer_type, id);
                }
            });
            // 2.2. Update that identity information and its offchain authorization nonce.
            <DidRecords>::mutate(id, |record| {
                let keys = additional_keys
                    .iter()
                    .map(|si_with_auth| si_with_auth.signing_item.clone())
                    .collect::<Vec<_>>();
                (*record).add_signing_items(&keys[..]);
            });
            <OffChainAuthorizationNonce>::mutate(id, |offchain_nonce| {
                *offchain_nonce = authorization.nonce + 1;
            });

            Ok(())
        }

        /// It revokes the `auth` off-chain authorization of `signer`. It only takes effect if
        /// the authorized transaction is not yet executed.
        pub fn revoke_offchain_authorization(origin, signer: Signatory, auth: TargetIdAuthorization<T::Moment>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;

            match signer {
                Signatory::AccountKey(ref key) => {
                    ensure!(sender_key == *key, Error::<T>::KeyNotAllowed);
                }
                Signatory::Identity(id) => {
                    ensure!(Self::is_master_key(id, &sender_key), Error::<T>::NotMasterKey);
                }
            }

            <RevokeOffChainAuthorization<T>>::insert((signer,auth), true);
            Ok(())
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// One signing key can only belong to one DID
        AlreadyLinked,
        /// Missing current identity on the transaction
        MissingCurrentIdentity,
        /// Sender is not part of did's signing keys
        InvalidSender,
        /// No did linked to the user
        NoDIDFound,
        /// Signatory is not pre authorized by the identity
        Unauthorized,
        /// Given authorization is not pre-known
        UnknownAuthorization,
        /// Account Id cannot be extracted from signer
        InvalidAccountKey,
        /// Only CDD service providers are allowed.
        UnAuthorizedCddProvider,
        /// An invalid authorization from the owner.
        InvalidAuthorizationFromOwner,
        /// An invalid authorization from the CDD provider.
        InvalidAuthorizationFromCddProvider,
        /// The authorization to change the key was not from the owner of the master key.
        KeyChangeUnauthorized,
        /// Attestation was not by a CDD service provider.
        NotCddProviderAttestation,
        /// Authorizations are not for the same DID.
        AuthorizationsNotForSameDids,
        /// The DID must already exist.
        DidMustAlreadyExist,
        /// The Claim issuer DID must already exist.
        ClaimIssuerDidMustAlreadyExist,
        /// Sender must hold a claim issuer's signing key.
        SenderMustHoldClaimIssuerKey,
        /// Current identity cannot be forwarded, it is not a signing key of target identity.
        CurrentIdentityCannotBeForwarded,
        /// The authorization does not exist.
        AuthorizationDoesNotExist,
        /// The offchain authorization has expired.
        AuthorizationExpired,
        /// The master key is already linked to an identity.
        MasterKeyAlreadyLinked,
        /// The target DID has no valid CDD.
        TargetHasNoCdd,
        /// Authorization has been explicitly revoked.
        AuthorizationHasBeenRevoked,
        /// An invalid authorization signature.
        InvalidAuthorizationSignature,
        /// This key is not allowed to execute a given operation.
        KeyNotAllowed,
        /// Only the master key is allowed to revoke an Identity Signatory off-chain authorization.
        NotMasterKey,
        /// The DID does not exist.
        DidDoesNotExist,
        /// The DID already exists.
        DidAlreadyExists,
        /// The signing keys contain the master key.
        SigningKeysContainMasterKey,
        /// Couldn't charge fee for the transaction
        FailedToChargeFee,
    }
}

impl<T: Trait> Module<T> {
    /// Accepts an auth to join an identity as a signer
    pub fn join_identity(signer: Signatory, auth_id: u64) -> DispatchResult {
        ensure!(
            <Authorizations<T>>::contains_key(signer, auth_id),
            AuthorizationError::Invalid
        );

        let auth = <Authorizations<T>>::get(signer, auth_id);

        let identity_to_join = match auth.authorization_data {
            AuthorizationData::JoinIdentity(identity) => Ok(identity),
            _ => Err(AuthorizationError::Invalid),
        }?;

        ensure!(
            <DidRecords>::contains_key(&identity_to_join),
            "Identity does not exist"
        );

        let master = Self::did_records(&identity_to_join).master_key;

        Self::consume_auth(Signatory::from(master), signer, auth_id)?;

        Self::unsafe_join_identity(identity_to_join, signer)
    }

    /// Joins an identity as signer
    pub fn unsafe_join_identity(identity_to_join: IdentityId, signer: Signatory) -> DispatchResult {
        if let Signatory::AccountKey(key) = signer {
            ensure!(
                Self::can_key_be_linked_to_did(&key, SignatoryType::External),
                Error::<T>::AlreadyLinked
            );
            T::ProtocolFee::charge_fee(
                &Signatory::Identity(identity_to_join),
                ProtocolOp::IdentityAddSigningItem,
            )?;
            Self::link_key_to_did(&key, SignatoryType::External, identity_to_join);
        } else {
            T::ProtocolFee::charge_fee(
                &Signatory::Identity(identity_to_join),
                ProtocolOp::IdentityAddSigningItem,
            )?;
        }
        <DidRecords>::mutate(identity_to_join, |identity| {
            identity.add_signing_items(&[SigningItem::new(signer, vec![])]);
        });

        Self::deposit_event(RawEvent::NewSigningItems(
            identity_to_join,
            [SigningItem::new(signer, vec![])].to_vec(),
        ));

        Ok(())
    }

    pub fn add_auth(
        from: Signatory,
        target: Signatory,
        authorization_data: AuthorizationData,
        expiry: Option<T::Moment>,
    ) -> u64 {
        let new_nonce = Self::multi_purpose_nonce() + 1u64;
        <MultiPurposeNonce>::put(&new_nonce);

        let auth = Authorization {
            authorization_data: authorization_data.clone(),
            authorized_by: from,
            expiry,
            auth_id: new_nonce,
        };

        <Authorizations<T>>::insert(target, new_nonce, auth);
        <AuthorizationsGiven>::insert(from, new_nonce, target);

        Self::deposit_event(RawEvent::NewAuthorization(
            new_nonce,
            from,
            target,
            authorization_data,
            expiry,
        ));

        new_nonce
    }

    /// Remove any authorization. No questions asked.
    /// NB: Please do all the required checks before calling this function.
    pub fn remove_auth(target: Signatory, auth_id: u64, authorizer: Signatory) {
        <Authorizations<T>>::remove(target, auth_id);
        <AuthorizationsGiven>::remove(authorizer, auth_id);

        Self::deposit_event(RawEvent::AuthorizationRemoved(auth_id, target));
    }

    /// Consumes an authorization.
    /// Checks if the auth has not expired and the caller is authorized to consume this auth.
    pub fn consume_auth(from: Signatory, target: Signatory, auth_id: u64) -> DispatchResult {
        ensure!(
            <Authorizations<T>>::contains_key(target, auth_id),
            AuthorizationError::Invalid
        );
        let auth = <Authorizations<T>>::get(target, auth_id);
        ensure!(auth.authorized_by == from, AuthorizationError::Unauthorized);
        if let Some(expiry) = auth.expiry {
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(expiry > now, AuthorizationError::Expired);
        }

        Self::remove_auth(target, auth_id, auth.authorized_by);
        Ok(())
    }

    pub fn get_authorization(target: Signatory, auth_id: u64) -> Authorization<T::Moment> {
        <Authorizations<T>>::get(target, auth_id)
    }

    pub fn get_link(target: Signatory, link_id: u64) -> Link<T::Moment> {
        <Links<T>>::get(target, link_id)
    }

    /// Adds a link to a key or an identity
    /// NB: Please do all the required checks before calling this function.
    pub fn add_link(target: Signatory, link_data: LinkData, expiry: Option<T::Moment>) -> u64 {
        let new_nonce = Self::multi_purpose_nonce() + 1u64;
        <MultiPurposeNonce>::put(&new_nonce);

        let link = Link {
            link_data: link_data.clone(),
            expiry,
            link_id: new_nonce,
        };

        <Links<T>>::insert(target, new_nonce, link);

        Self::deposit_event(RawEvent::NewLink(new_nonce, target, link_data, expiry));
        new_nonce
    }

    /// Remove a link (if it exists) from a key or identity
    /// NB: Please do all the required checks before calling this function.
    pub fn remove_link(target: Signatory, link_id: u64) {
        if <Links<T>>::contains_key(target, link_id) {
            <Links<T>>::remove(target, link_id);
            Self::deposit_event(RawEvent::LinkRemoved(link_id, target));
        }
    }

    /// Update link data (if it exists) from a key or identity
    /// NB: Please do all the required checks before calling this function.
    pub fn update_link(target: Signatory, link_id: u64, link_data: LinkData) {
        if <Links<T>>::contains_key(target, link_id) {
            <Links<T>>::mutate(target, link_id, |link| link.link_data = link_data);
            Self::deposit_event(RawEvent::LinkUpdated(link_id, target));
        }
    }

    /// Accepts a master key rotation
    fn accept_master_key_rotation(
        sender_key: AccountKey,
        rotation_auth_id: u64,
        optional_cdd_auth_id: Option<u64>,
    ) -> DispatchResult {
        let signer = Signatory::from(sender_key);
        // ensure authorization is present
        ensure!(
            <Authorizations<T>>::contains_key(signer, rotation_auth_id),
            Error::<T>::InvalidAuthorizationFromOwner
        );

        // Accept authorization from the owner
        let rotation_auth = <Authorizations<T>>::get(signer, rotation_auth_id);

        if let AuthorizationData::RotateMasterKey(rotation_for_did) =
            rotation_auth.authorization_data
        {
            // Ensure the request was made by the owner of master key
            match rotation_auth.authorized_by {
                Signatory::AccountKey(key) => {
                    let master_key = <DidRecords>::get(rotation_for_did).master_key;
                    ensure!(key == master_key, Error::<T>::KeyChangeUnauthorized);
                }
                _ => return Err(Error::<T>::UnknownAuthorization.into()),
            };
            // consume owner's authorization
            Self::consume_auth(rotation_auth.authorized_by, signer, rotation_auth_id)?;
            Self::unsafe_master_key_rotation(sender_key, rotation_for_did, optional_cdd_auth_id)
        } else {
            Err(Error::<T>::UnknownAuthorization.into())
        }
    }

    /// Processes master key rotation
    pub fn unsafe_master_key_rotation(
        sender_key: AccountKey,
        rotation_for_did: IdentityId,
        optional_cdd_auth_id: Option<u64>,
    ) -> DispatchResult {
        // Aceept authorization from CDD service provider
        if Self::cdd_auth_for_master_key_rotation() {
            if let Some(cdd_auth_id) = optional_cdd_auth_id {
                let signer = Signatory::from(sender_key);
                ensure!(
                    <Authorizations<T>>::contains_key(signer, cdd_auth_id),
                    Error::<T>::InvalidAuthorizationFromCddProvider
                );
                let cdd_auth = <Authorizations<T>>::get(signer, cdd_auth_id);

                if let AuthorizationData::AttestMasterKeyRotation(attestation_for_did) =
                    cdd_auth.authorization_data
                {
                    // Attestor must be a CDD service provider
                    let cdd_provider_did = match cdd_auth.authorized_by {
                        Signatory::AccountKey(ref key) => Self::get_identity(key),
                        Signatory::Identity(id) => Some(id),
                    };

                    if let Some(id) = cdd_provider_did {
                        ensure!(
                            T::CddServiceProviders::is_member(&id),
                            Error::<T>::NotCddProviderAttestation
                        );
                    } else {
                        return Err(Error::<T>::NoDIDFound.into());
                    }

                    // Make sure authorizations are for the same DID
                    ensure!(
                        rotation_for_did == attestation_for_did,
                        Error::<T>::AuthorizationsNotForSameDids
                    );

                    // consume CDD service provider's authorization
                    Self::consume_auth(cdd_auth.authorized_by, signer, cdd_auth_id)?;
                } else {
                    return Err(Error::<T>::UnknownAuthorization.into());
                }
            } else {
                return Err(Error::<T>::InvalidAuthorizationFromCddProvider.into());
            }
        }

        // Replace master key of the owner that initiated key rotation
        <DidRecords>::mutate(rotation_for_did, |record| {
            Self::unlink_key_to_did(&(*record).master_key, rotation_for_did);
            (*record).master_key = sender_key;
        });

        Self::deposit_event(RawEvent::MasterKeyChanged(rotation_for_did, sender_key));
        Ok(())
    }

    /// Private and not sanitized function. It is designed to be used internally by
    /// others sanitezed functions.
    fn update_signing_item_permissions(
        target_did: IdentityId,
        signer: &Signatory,
        mut permissions: Vec<Permission>,
    ) -> DispatchResult {
        // Remove duplicates.
        permissions.sort();
        permissions.dedup();

        let mut new_s_item: Option<SigningItem> = None;

        <DidRecords>::mutate(target_did, |record| {
            if let Some(mut signing_item) = (*record)
                .signing_items
                .iter()
                .find(|si| si.signer == *signer)
                .cloned()
            {
                swap(&mut signing_item.permissions, &mut permissions);
                (*record).signing_items.retain(|si| si.signer != *signer);
                (*record).signing_items.push(signing_item.clone());
                new_s_item = Some(signing_item);
            }
        });

        if let Some(s) = new_s_item {
            Self::deposit_event(RawEvent::SigningPermissionsUpdated(
                target_did,
                s,
                permissions,
            ));
        }
        Ok(())
    }

    /// It checks if `key` is a signing key of `did` identity.
    /// # IMPORTANT
    /// If signing keys are frozen this function always returns false.
    /// Master key cannot be frozen.
    pub fn is_signer_authorized(did: IdentityId, signer: &Signatory) -> bool {
        let record = <DidRecords>::get(did);

        // Check master id or key
        match signer {
            Signatory::AccountKey(ref signer_key) if record.master_key == *signer_key => true,
            Signatory::Identity(ref signer_id) if did == *signer_id => true,
            _ => {
                // Check signing items if DID is not frozen.
                !Self::is_did_frozen(did)
                    && record.signing_items.iter().any(|si| si.signer == *signer)
            }
        }
    }

    fn is_signer_authorized_with_permissions(
        did: IdentityId,
        signer: &Signatory,
        permissions: Vec<Permission>,
    ) -> bool {
        let record = <DidRecords>::get(did);

        match signer {
            Signatory::AccountKey(ref signer_key) if record.master_key == *signer_key => true,
            Signatory::Identity(ref signer_id) if did == *signer_id => true,
            _ => {
                if !Self::is_did_frozen(did) {
                    if let Some(signing_item) =
                        record.signing_items.iter().find(|&si| &si.signer == signer)
                    {
                        // It retruns true if all requested permission are in this signing item.
                        return permissions.iter().all(|required_permission| {
                            signing_item.has_permission(*required_permission)
                        });
                    }
                }
                // Signatory is not part of signing items of `did`, or
                // Did is frozen.
                false
            }
        }
    }

    /// Use `did` as reference.
    pub fn is_master_key(did: IdentityId, key: &AccountKey) -> bool {
        key == &<DidRecords>::get(did).master_key
    }

    /// It returns true if `id_claim` is not expired at `moment`.
    #[inline]
    fn is_identity_claim_not_expired_at(id_claim: &IdentityClaim, moment: T::Moment) -> bool {
        if let Some(expiry) = id_claim.expiry {
            expiry > moment.saturated_into::<u64>()
        } else {
            true
        }
    }

    /// It fetches an specific `claim_type` claim type for target identity `id`, which was issued
    /// by `issuer`.
    /// It only returns non-expired claims.
    pub fn fetch_claim(
        id: IdentityId,
        claim_type: ClaimType,
        issuer: IdentityId,
        scope: Option<Scope>,
    ) -> Option<IdentityClaim> {
        let now = <pallet_timestamp::Module<T>>::get();

        Self::fetch_base_claim_with_issuer(id, claim_type, issuer, scope)
            .into_iter()
            .filter(|c| Self::is_identity_claim_not_expired_at(c, now))
            .next()
    }

    /// See `Self::fetch_cdd`.
    #[inline]
    pub fn has_valid_cdd(claim_for: IdentityId) -> bool {
        let trusted_cdd_providers = T::CddServiceProviders::get_members();
        // It will never happen in production but helpful during testing.
        // TODO: Remove this condition
        if trusted_cdd_providers.len() == 0 {
            return true;
        }

        Self::fetch_cdd(claim_for, T::Moment::zero()).is_some()
    }

    /// It returns the CDD identity which issued the current valid CDD claim for `claim_for`
    /// identity.
    /// # Parameters
    /// * `leeway` : This leeway is added to now() before check if claim is expired.
    ///
    /// # Safety
    ///
    /// No state change is allowed in this function because this function is used within the RPC
    /// calls.
    pub fn fetch_cdd(claim_for: IdentityId, leeway: T::Moment) -> Option<IdentityId> {
        let exp_with_leeway = <pallet_timestamp::Module<T>>::get()
            .checked_add(&leeway)
            .unwrap_or_default();

        let active_cdds = T::CddServiceProviders::get_active_members();
        let inactive_not_expired_cdds = T::CddServiceProviders::get_inactive_members()
            .into_iter()
            .filter(|cdd| !T::CddServiceProviders::is_member_expired(cdd, exp_with_leeway))
            .collect::<Vec<_>>();

        Self::fetch_base_claims(claim_for, ClaimType::CustomerDueDiligence)
            .filter(|id_claim| {
                Self::is_identity_cdd_claim_valid(
                    id_claim,
                    exp_with_leeway,
                    &active_cdds,
                    &inactive_not_expired_cdds,
                )
            })
            .map(|id_claim| id_claim.claim_issuer)
            .next()
    }

    /// A CDD claims is considered valid if:
    /// * Claim is not expired at `exp_with_leeway` moment.
    /// * Its issuer is valid, that means:
    ///   * Issuer is an active CDD provider, or
    ///   * Issuer is an inactive CDD provider but claim was updated/created before that it was
    ///   deactivated.
    fn is_identity_cdd_claim_valid(
        id_claim: &IdentityClaim,
        exp_with_leeway: T::Moment,
        active_cdds: &[IdentityId],
        inactive_not_expired_cdds: &[InactiveMember<T::Moment>],
    ) -> bool {
        Self::is_identity_claim_not_expired_at(id_claim, exp_with_leeway)
            && (active_cdds.contains(&id_claim.claim_issuer)
                || inactive_not_expired_cdds
                    .iter()
                    .filter(|cdd| cdd.id == id_claim.claim_issuer)
                    .any(|cdd| {
                        id_claim.last_update_date < cdd.deactivated_at.saturated_into::<u64>()
                    }))
    }

    /// It iterates over all claims of type `claim_type` for target `id` identity.
    /// Please note that it could return expired claims.
    fn fetch_base_claims<'a>(
        target: IdentityId,
        claim_type: ClaimType,
    ) -> impl Iterator<Item = IdentityClaim> + 'a {
        let pk = Claim1stKey { target, claim_type };
        <Claims>::iter_prefix(pk)
    }

    /// It fetches an specific `claim_type` claim type for target identity `id`, which was issued
    /// by `issuer`.
    fn fetch_base_claim_with_issuer(
        target: IdentityId,
        claim_type: ClaimType,
        issuer: IdentityId,
        scope: Option<Scope>,
    ) -> Option<IdentityClaim> {
        let pk = Claim1stKey { target, claim_type };
        let sk = Claim2ndKey { issuer, scope };

        if <Claims>::contains_key(&pk, &sk) {
            Some(<Claims>::get(&pk, &sk))
        } else {
            None
        }
    }

    /// It checks that `sender_key` is the master key of `did` Identifier and that
    /// did exists.
    /// # Return
    /// A result object containing the `DidRecord` of `did`.
    pub fn grant_check_only_master_key(
        sender_key: &AccountKey,
        did: IdentityId,
    ) -> sp_std::result::Result<DidRecord, Error<T>> {
        ensure!(<DidRecords>::contains_key(did), Error::<T>::DidDoesNotExist);
        let record = <DidRecords>::get(did);
        ensure!(*sender_key == record.master_key, Error::<T>::KeyNotAllowed);
        Ok(record)
    }

    /// It checks if `key` is the master key or signing key of any did
    /// # Return
    /// An Option object containing the `did` that belongs to the key.
    pub fn get_identity(key: &AccountKey) -> Option<IdentityId> {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(key) {
            if let LinkedKeyInfo::Unique(linked_id) = linked_key_info {
                return Some(linked_id);
            }
        }
        None
    }

    /// It freezes/unfreezes the target `did` identity.
    ///
    /// # Errors
    /// Only master key can freeze/unfreeze an identity.
    fn set_frozen_signing_key_flags(origin: T::Origin, freeze: bool) -> DispatchResult {
        let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
        let did = Context::current_identity_or::<Self>(&sender_key)?;
        let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

        if freeze {
            <IsDidFrozen>::insert(did, true);
        } else {
            <IsDidFrozen>::remove(did);
        }
        Ok(())
    }

    /// It checks that any sternal account can only be associated with at most one.
    /// Master keys are considered as external accounts.
    pub fn can_key_be_linked_to_did(key: &AccountKey, signer_type: SignatoryType) -> bool {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(key) {
            match linked_key_info {
                LinkedKeyInfo::Unique(..) => false,
                LinkedKeyInfo::Group(..) => signer_type != SignatoryType::External,
            }
        } else {
            true
        }
    }

    /// It links `key` key to `did` identity as a `key_type` type.
    /// # Errors
    /// This function can be used if `can_key_be_linked_to_did` returns true. Otherwise, it will do
    /// nothing.
    fn link_key_to_did(key: &AccountKey, key_type: SignatoryType, did: IdentityId) {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(key) {
            if let LinkedKeyInfo::Group(mut dids) = linked_key_info {
                if !dids.contains(&did) && key_type != SignatoryType::External {
                    dids.push(did);
                    dids.sort();

                    <KeyToIdentityIds>::insert(key, LinkedKeyInfo::Group(dids));
                }
            }
        } else {
            // AccountKey is not yet linked to any identity, so no constraints.
            let linked_key_info = match key_type {
                SignatoryType::External => LinkedKeyInfo::Unique(did),
                _ => LinkedKeyInfo::Group(vec![did]),
            };
            <KeyToIdentityIds>::insert(key, linked_key_info);
        }
    }

    /// It unlinks the `key` key from `did`.
    /// If there is no more associated identities, its full entry is removed.
    fn unlink_key_to_did(key: &AccountKey, did: IdentityId) {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(key) {
            match linked_key_info {
                LinkedKeyInfo::Unique(..) => <KeyToIdentityIds>::remove(key),
                LinkedKeyInfo::Group(mut dids) => {
                    dids.retain(|ref_did| *ref_did != did);
                    if dids.is_empty() {
                        <KeyToIdentityIds>::remove(key);
                    } else {
                        <KeyToIdentityIds>::insert(key, LinkedKeyInfo::Group(dids));
                    }
                }
            }
        }
    }

    /// It adds `signing_item` to pre authorized items for `id` identity.
    fn add_pre_join_identity(signing_item: &SigningItem, id: IdentityId) {
        let signer = &signing_item.signer;
        let new_pre_auth = PreAuthorizedKeyInfo::new(signing_item.clone(), id);

        if !<PreAuthorizedJoinDid>::contains_key(signer) {
            <PreAuthorizedJoinDid>::insert(signer, vec![new_pre_auth]);
        } else {
            <PreAuthorizedJoinDid>::mutate(signer, |pre_auth_list| {
                pre_auth_list.retain(|pre_auth| *pre_auth != id);
                pre_auth_list.push(new_pre_auth);
            });
        }
    }

    /// It removes `signing_item` to pre authorized items for `id` identity.
    fn remove_pre_join_identity(signer: &Signatory, id: IdentityId) {
        let mut is_pre_auth_list_empty = false;
        <PreAuthorizedJoinDid>::mutate(signer, |pre_auth_list| {
            pre_auth_list.retain(|pre_auth| pre_auth.target_id != id);
            is_pre_auth_list_empty = pre_auth_list.is_empty();
        });

        if is_pre_auth_list_empty {
            <PreAuthorizedJoinDid>::remove(signer);
        }
    }

    /// It registers a did for a new asset. Only called by create_token function.
    pub fn register_asset_did(ticker: &Ticker) -> DispatchResult {
        let did = Self::get_token_did(ticker)?;
        Self::deposit_event(RawEvent::AssetDid(*ticker, did));
        // Making sure there's no pre-existing entry for the DID
        // This should never happen but just being defensive here
        ensure!(
            !<DidRecords>::contains_key(did),
            Error::<T>::DidAlreadyExists
        );
        <DidRecords>::insert(did, DidRecord::default());
        Ok(())
    }

    /// IMPORTANT: No state change is allowed in this function
    /// because this function is used within the RPC calls
    /// It is a helper function that can be used to get did for any asset
    pub fn get_token_did(ticker: &Ticker) -> StdResult<IdentityId, &'static str> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&SECURITY_TOKEN.encode());
        buf.extend_from_slice(&ticker.encode());
        IdentityId::try_from(T::Hashing::hash(&buf[..]).as_ref())
    }

    pub fn _register_did(
        sender: T::AccountId,
        signing_items: Vec<SigningItem>,
        protocol_fee_data: Option<(&Signatory, ProtocolOp)>,
    ) -> Result<IdentityId, DispatchError> {
        // Adding extrensic count to did nonce for some unpredictability
        // NB: this does not guarantee randomness
        let new_nonce =
            Self::multi_purpose_nonce() + u64::from(<system::Module<T>>::extrinsic_count()) + 7u64;
        // Even if this transaction fails, nonce should be increased for added unpredictability of dids
        <MultiPurposeNonce>::put(&new_nonce);

        let master_key = AccountKey::try_from(sender.encode())?;

        // 1 Check constraints.
        // 1.1. Master key is not linked to any identity.
        ensure!(
            Self::can_key_be_linked_to_did(&master_key, SignatoryType::External),
            Error::<T>::MasterKeyAlreadyLinked
        );
        // 1.2. Master key is not part of signing keys.
        ensure!(
            signing_items.iter().find(|sk| **sk == master_key).is_none(),
            Error::<T>::SigningKeysContainMasterKey
        );

        let block_hash = <system::Module<T>>::block_hash(<system::Module<T>>::block_number());

        let did = IdentityId::from(blake2_256(&(USER, block_hash, new_nonce).encode()));

        // 1.3. Make sure there's no pre-existing entry for the DID
        // This should never happen but just being defensive here
        ensure!(
            !<DidRecords>::contains_key(did),
            Error::<T>::DidAlreadyExists
        );
        // 1.4. Signing keys can be linked to the new identity.
        for s_item in &signing_items {
            if let Signatory::AccountKey(ref key) = s_item.signer {
                ensure!(
                    Self::can_key_be_linked_to_did(key, s_item.signer_type),
                    Error::<T>::AlreadyLinked
                );
            }
        }

        // 1.999. Charge the given fee.
        if let Some((payee, op)) = protocol_fee_data {
            T::ProtocolFee::charge_fee(payee, op)?;
        }

        // 2. Apply changes to our extrinsics.
        // 2.1. Link  master key and add pre-authorized signing keys
        Self::link_key_to_did(&master_key, SignatoryType::External, did);
        signing_items
            .iter()
            .for_each(|s_item| Self::add_pre_join_identity(s_item, did));

        // 2.2. Create a new identity record.
        let record = DidRecord {
            master_key,
            ..Default::default()
        };
        <DidRecords>::insert(&did, record);

        Self::deposit_event(RawEvent::NewDid(did, sender, signing_items));
        Ok(did)
    }

    /// It adds a new claim without any previous security check.
    fn unsafe_add_claim(
        target: IdentityId,
        claim: Claim,
        issuer: IdentityId,
        expiry: Option<T::Moment>,
    ) {
        let claim_type = claim.claim_type();
        let scope = claim.as_scope().cloned();
        let last_update_date = <pallet_timestamp::Module<T>>::get().saturated_into::<u64>();
        let issuance_date = Self::fetch_claim(target, claim_type, issuer, scope)
            .map_or(last_update_date, |id_claim| id_claim.issuance_date);

        let expiry = expiry.into_iter().map(|m| m.saturated_into::<u64>()).next();
        let pk = Claim1stKey { target, claim_type };
        let sk = Claim2ndKey { issuer, scope };
        let id_claim = IdentityClaim {
            claim_issuer: issuer,
            issuance_date,
            last_update_date,
            expiry,
            claim,
        };

        <Claims>::insert(&pk, &sk, id_claim.clone());
        Self::deposit_event(RawEvent::NewClaims(target, id_claim));
    }

    /// It ensures that CDD claim issuer is a valid CDD provider before add the claim.
    ///
    /// # Errors
    /// - 'UnAuthorizedCddProvider' is returned if `issuer` is not a CDD provider.
    fn unsafe_add_cdd_claim(
        target: IdentityId,
        claim: Claim,
        issuer: IdentityId,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        let cdd_providers = T::CddServiceProviders::get_members();
        ensure!(
            cdd_providers.contains(&issuer),
            Error::<T>::UnAuthorizedCddProvider
        );

        Self::unsafe_add_claim(target, claim, issuer, expiry);
        Ok(())
    }

    pub fn is_identity_exists(did: &IdentityId) -> bool {
        <DidRecords>::contains_key(did)
    }

    /// It removes a claim from `target` which was issued by `issuer` without any security check.
    fn unsafe_revoke_claim(
        target: IdentityId,
        claim_type: ClaimType,
        issuer: IdentityId,
        scope: Option<Scope>,
    ) {
        let pk = Claim1stKey { target, claim_type };
        let sk = Claim2ndKey { scope, issuer };

        <Claims>::remove(&pk, &sk);
        Self::deposit_event(RawEvent::RevokedClaim(target, claim_type, issuer));
    }

    /// Returns an auth id if it is present and not expired.
    pub fn get_non_expired_auth(
        target: &Signatory,
        auth_id: &u64,
    ) -> Option<Authorization<T::Moment>> {
        if !<Authorizations<T>>::contains_key(target, auth_id) {
            return None;
        }
        let auth = <Authorizations<T>>::get(target, auth_id);
        if let Some(expiry) = auth.expiry {
            let now = <pallet_timestamp::Module<T>>::get();
            if expiry > now {
                return None;
            }
        }
        Some(auth)
    }

    /// Returns identity of a signatory
    pub fn get_identity_of_signatory(signer: &Signatory) -> Option<IdentityId> {
        match signer {
            Signatory::AccountKey(key) => Self::get_identity(&key),
            Signatory::Identity(did) => Some(*did),
        }
    }
}

impl<T: Trait> Module<T> {
    /// RPC call to know whether the given did has valid cdd claim or not
    pub fn is_identity_has_valid_cdd(
        target: IdentityId,
        leeway: Option<T::Moment>,
    ) -> Option<IdentityId> {
        Self::fetch_cdd(target, leeway.unwrap_or_default())
    }

    /// RPC call to query the given ticker did
    pub fn get_asset_did(ticker: Ticker) -> Result<IdentityId, &'static str> {
        Self::get_token_did(&ticker)
    }

    /// Retrieve DidRecords for `did`
    pub fn get_did_records(did: IdentityId) -> RpcDidRecords<AccountKey, SigningItem> {
        if <DidRecords>::contains_key(did) {
            let record = <DidRecords>::get(did);
            RpcDidRecords::Success {
                master_key: record.master_key,
                signing_items: record.signing_items,
            }
        } else {
            RpcDidRecords::IdNotFound
        }
    }
}

impl<T: Trait> IdentityTrait for Module<T> {
    fn get_identity(key: &AccountKey) -> Option<IdentityId> {
        Self::get_identity(&key)
    }

    fn current_identity() -> Option<IdentityId> {
        <CurrentDid>::get()
    }

    fn set_current_identity(id: Option<IdentityId>) {
        if let Some(id) = id {
            <CurrentDid>::put(id);
        } else {
            <CurrentDid>::kill();
        }
    }

    fn current_payer() -> Option<Signatory> {
        <CurrentPayer>::get()
    }

    fn set_current_payer(payer: Option<Signatory>) {
        if let Some(payer) = payer {
            <CurrentPayer>::put(payer);
        } else {
            <CurrentPayer>::kill();
        }
    }

    fn is_signer_authorized(did: IdentityId, signer: &Signatory) -> bool {
        Self::is_signer_authorized(did, signer)
    }

    fn is_master_key(did: IdentityId, key: &AccountKey) -> bool {
        Self::is_master_key(did, &key)
    }

    fn is_signer_authorized_with_permissions(
        did: IdentityId,
        signer: &Signatory,
        permissions: Vec<Permission>,
    ) -> bool {
        Self::is_signer_authorized_with_permissions(did, signer, permissions)
    }

    fn unsafe_add_systematic_cdd_claims(targets: &[IdentityId], issuer: SystematicIssuers) {
        targets.iter().for_each(|new_member| {
            Self::unsafe_add_claim(
                *new_member,
                Claim::CustomerDueDiligence,
                issuer.as_id(),
                None,
            )
        });
    }

    fn unsafe_revoke_systematic_cdd_claims(targets: &[IdentityId], issuer: SystematicIssuers) {
        targets.iter().for_each(|removed_member| {
            Self::unsafe_revoke_claim(
                *removed_member,
                ClaimType::CustomerDueDiligence,
                issuer.as_id(),
                None,
            )
        });
    }
}

impl<T: Trait> ChangeMembers<IdentityId> for Module<T> {
    fn change_members_sorted(
        incoming: &[IdentityId],
        outgoing: &[IdentityId],
        _new: &[IdentityId],
    ) {
        // Add/remove Systematic CDD claims for new/removed members.
        let issuer = SystematicIssuers::CDDProvider;
        Self::unsafe_add_systematic_cdd_claims(incoming, issuer);
        Self::unsafe_revoke_systematic_cdd_claims(outgoing, issuer);
    }
}

impl<T: Trait> InitializeMembers<IdentityId> for Module<T> {
    fn initialize_members(members: &[IdentityId]) {
        Self::unsafe_add_systematic_cdd_claims(members, SystematicIssuers::CDDProvider);
    }
}
