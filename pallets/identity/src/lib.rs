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
//! `MuliSign`, etc.) and/or its permission.
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
//! # TODO
//!  - KYC is mocked: see [has_valid_kyc](./struct.Module.html#method.has_valid_kyc)

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use polymesh_primitives::{
    AccountKey, AuthIdentifier, Authorization, AuthorizationData, AuthorizationError,
    Identity as DidRecord, IdentityId, Link, LinkData, Permission, PreAuthorizedKeyInfo, Signatory,
    SignatoryType, SigningItem, Ticker,
};
use polymesh_runtime_common::{
    constants::{
        did::{SECURITY_TOKEN, USER},
        KYC_EXPIRY_CLAIM_KEY,
    },
    traits::{
        asset::AcceptTransfer,
        balances::BalancesTrait,
        group::GroupTrait,
        identity::{
            AuthorizationNonce, Claim, ClaimMetaData, ClaimRecord, ClaimValue, LinkedKeyInfo,
            RawEvent, SigningItemWithAuth, TargetIdAuthorization,
        },
        multisig::AddSignerMultiSig,
    },
    BatchDispatchInfo,
};

use codec::Encode;
use core::{
    convert::{From, TryInto},
    result::Result as StdResult,
};

use sp_core::sr25519::{Public, Signature};
use sp_io::hashing::blake2_256;
use sp_runtime::{
    traits::{Dispatchable, Hash, SaturatedConversion, Verify},
    AnySignature,
};
use sp_std::{convert::TryFrom, mem::swap, prelude::*, vec};

use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{ExistenceRequirement, WithdrawReason},
    weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_signed};

pub use polymesh_runtime_common::traits::identity::{IdentityTrait, Trait};
pub type Event<T> = polymesh_runtime_common::traits::identity::Event<T>;

decl_storage! {
    trait Store for Module<T: Trait> as identity {

        /// Module owner.
        Owner get(fn owner) config(): T::AccountId;

        /// DID -> identity info
        pub DidRecords get(fn did_records): map IdentityId => DidRecord;

        /// DID -> bool that indicates if signing keys are frozen.
        pub IsDidFrozen get(fn is_did_frozen): map IdentityId => bool;

        /// It stores the current identity for current transaction.
        pub CurrentDid get(fn current_did): Option<IdentityId>;

        /// (DID, claim_key, claim_issuer) -> Associated claims
        pub Claims get(fn claims): map(IdentityId, ClaimMetaData) => Claim<T::Moment>;

        /// DID -> array of (claim_key and claim_issuer)
        pub ClaimKeys get(fn claim_keys): map IdentityId => Vec<ClaimMetaData>;

        // Account => DID
        pub KeyToIdentityIds get(fn key_to_identity_ids): map AccountKey => Option<LinkedKeyInfo>;

        /// How much does creating a DID cost
        pub DidCreationFee get(fn did_creation_fee) config(): T::Balance;

        /// It stores validated identities by any KYC.
        pub KYCValidation get(fn has_valid_kyc): map IdentityId => bool;

        /// Nonce to ensure unique actions. starts from 1.
        pub MultiPurposeNonce get(fn multi_purpose_nonce) build(|_| 1u64): u64;

        /// Pre-authorize join to Identity.
        pub PreAuthorizedJoinDid get(fn pre_authorized_join_did): map Signatory => Vec<PreAuthorizedKeyInfo>;

        /// Authorization nonce per Identity. Initially is 0.
        pub OffChainAuthorizationNonce get(fn offchain_authorization_nonce): map IdentityId => AuthorizationNonce;

        /// Inmediate revoke of any off-chain authorization.
        pub RevokeOffChainAuthorization get(fn is_offchain_authorization_revoked): map (Signatory, TargetIdAuthorization<T::Moment>) => bool;

        /// All authorizations that an identity/key has
        pub Authorizations: double_map hasher(blake2_256) Signatory, blake2_256(u64) => Authorization<T::Moment>;

        /// All links that an identity/key has
        pub Links: double_map hasher(blake2_256) Signatory, blake2_256(u64) => Link<T::Moment>;

        /// All authorizations that an identity/key has given. (Authorizer, auth_id -> authorized)
        pub AuthorizationsGiven: double_map hasher(blake2_256) Signatory, blake2_256(u64) => Signatory;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        /// Register signing keys for a new DID. Uses origin key as the master key.
        ///
        /// # TODO
        /// Signing keys should authorize its use in this identity.
        ///
        /// # Failure
        /// - Master key (administrator) can be linked to just one identity.
        /// - External signing keys can be linked to just one identity.
        pub fn register_did(origin, signing_items: Vec<SigningItem>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            // TODO: Subtract proper fee.
            let _imbalance = <T::Balances>::withdraw(
                &sender,
                Self::did_creation_fee(),
                WithdrawReason::Fee.into(),
                ExistenceRequirement::KeepAlive,
            )?;
            Self::_register_did(sender, signing_items)
        }

        /// Adds new signing keys for a DID. Only called by master key owner.
        ///
        /// # Failure
        ///  - It can only called by master key owner.
        ///  - If any signing key is already linked to any identity, it will fail.
        ///  - If any signing key is already
        pub fn add_signing_items(origin, did: IdentityId, signing_items: Vec<SigningItem>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            // Check constraint 1-to-1 in relation key-identity.
            for s_item in &signing_items{
                if let Signatory::AccountKey(ref key) = s_item.signer {
                    if !Self::can_key_be_linked_to_did( key, s_item.signer_type) {
                        return Err(Error::<T>::AlreadyLinked.into());
                    }
                }
            }

            // Ignore any key which is already valid in that identity.
            let authorized_signing_items = Self::did_records( did).signing_items;
            signing_items.iter()
                .filter( |si| authorized_signing_items.contains(si) == false)
                .for_each( |si| Self::add_pre_join_identity( si, did));

            Self::deposit_event(RawEvent::NewSigningItems(did, signing_items));
            Ok(())
        }

        /// Removes specified signing keys of a DID if present.
        ///
        /// # Failure
        /// It can only called by master key owner.
        pub fn remove_signing_items(origin, did: IdentityId, signers_to_remove: Vec<Signatory>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
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
        fn set_master_key(origin, did: IdentityId, new_key: AccountKey) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from( sender.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            ensure!( Self::can_key_be_linked_to_did(&new_key, SignatoryType::External), "Master key can only belong to one DID");

            <DidRecords>::mutate(did,
            |record| {
                (*record).master_key = new_key.clone();
            });

            Self::deposit_event(RawEvent::NewMasterKey(did, sender, new_key));
            Ok(())
        }

        /// Call this with the new master key. By invoking this method, caller accepts authorization
        /// with the new master key. If a KYC service provider approved this change, master key of
        /// the DID is updated.
        ///
        /// # Arguments
        /// * `owner_auth_id` Authorization from the owner who initiated the change
        /// * `kyc_auth_id` Authorization from a KYC service provider
        pub fn accept_master_key(origin, rotation_auth_id: u64, kyc_auth_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let signer = Signatory::from(sender_key);

            // When both authorizations are present...
            ensure!(<Authorizations<T>>::exists(signer, rotation_auth_id), "Invalid authorization from owner");
            ensure!(<Authorizations<T>>::exists(signer, kyc_auth_id), "Invalid authorization from KYC service provider");

            // Accept authorization from the owner
            let rotation_auth = <Authorizations<T>>::get(signer, rotation_auth_id);
            if let AuthorizationData::RotateMasterKey(rotation_for_did) = rotation_auth.authorization_data {
                // Ensure the request was made by the owner of master key
                match rotation_auth.authorized_by {
                    Signatory::AccountKey(key) =>  {
                        let master_key = <DidRecords>::get(rotation_for_did).master_key;
                        ensure!(key == master_key, "Authorization to change key was not from the owner of master key");
                    },
                    _ => return Err(Error::<T>::UnknownAuthorization.into())
                };

                // Aceept authorization from KYC service provider
                let kyc_auth = <Authorizations<T>>::get(signer, kyc_auth_id);
                if let AuthorizationData::AttestMasterKeyRotation(attestation_for_did) = kyc_auth.authorization_data {
                    // Attestor must be a KYC service provider
                    let kyc_provider_did = match kyc_auth.authorized_by {
                        Signatory::AccountKey(ref key) =>  Self::get_identity(key),
                        Signatory::Identity(id)  => Some(id),
                    };

                    if let Some(id) = kyc_provider_did {
                        ensure!(T::KycServiceProviders::is_member(&id), "Attestation was not by a KYC service provider");
                    } else {
                        return Err(Error::<T>::NoDIDFound.into());
                    }

                    // Make sure authorizations are for the same DID
                    ensure!(rotation_for_did == attestation_for_did, "Authorizations are not for the same DID");

                    // remove owner's authorization
                    Self::consume_auth(rotation_auth.authorized_by, signer, rotation_auth_id)?;

                    // remove KYC service provider's authorization
                    Self::consume_auth(kyc_auth.authorized_by, signer, kyc_auth_id)?;

                    // Replace master key of the owner that initiated key rotation
                    <DidRecords>::mutate(rotation_for_did, |record| {
                        (*record).master_key = sender_key.clone();
                    });

                    Self::deposit_event(RawEvent::MasterKeyChanged(rotation_for_did, sender_key));
                } else {
                    return Err(Error::<T>::UnknownAuthorization.into());
                }
            } else {
                return Err(Error::<T>::UnknownAuthorization.into());
            }

            Ok(())
        }

        /// Adds new claim record or edits an existing one. Only called by did_issuer's signing key
        #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        pub fn add_claim(
            origin,
            did: IdentityId,
            claim_key: Vec<u8>,
            did_issuer: IdentityId,
            expiry: <T as pallet_timestamp::Trait>::Moment,
            claim_value: ClaimValue
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords>::exists(did), "DID must already exist");
            ensure!(<DidRecords>::exists(did_issuer), "claim issuer DID must already exist");

            let sender_key = AccountKey::try_from(sender.encode())?;

            // Verify that sender key is one of did_issuer's signing keys
            let sender_signer = Signatory::AccountKey(sender_key);
            ensure!(Self::is_signer_authorized(did_issuer, &sender_signer), "Sender must hold a claim issuer's signing key");

            let claim_meta_data = ClaimMetaData {
                claim_key: claim_key,
                claim_issuer: did_issuer,
            };

            let now = <pallet_timestamp::Module<T>>::get();

            let claim = Claim {
                issuance_date: now,
                expiry: expiry,
                claim_value: claim_value,
            };

            <Claims<T>>::insert((did.clone(), claim_meta_data.clone()), claim.clone());

            <ClaimKeys>::mutate(&did, |old_claim_data| {
                if !old_claim_data.contains(&claim_meta_data) {
                    old_claim_data.push(claim_meta_data.clone());
                }
            });

            Self::deposit_event(RawEvent::NewClaims(did, claim_meta_data, claim));

            Ok(())
        }

        /// Adds a new batch of claim records or edits an existing one. Only called by
        /// `did_issuer`'s signing key.
        #[weight = BatchDispatchInfo::new_normal(3_000, 10_000)]
        pub fn add_claims_batch(
            origin,
            did_issuer: IdentityId,
            claims: Vec<ClaimRecord<<T as pallet_timestamp::Trait>::Moment>>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<DidRecords>::exists(did_issuer), "claim issuer DID must already exist");
            let sender_key = AccountKey::try_from(sender.encode())?;
            // Verify that sender key is one of did_issuer's signing keys
            let sender_signer = Signatory::AccountKey(sender_key);
            ensure!(Self::is_signer_authorized(did_issuer, &sender_signer),
                    "Sender must hold a claim issuer's signing key");
            // Claims that successfully passed all required checks. Unless all claims pass those
            // checks, the whole operation fails.
            let mut checked_claims = Vec::new();
            // Check input claims.
            for ClaimRecord {
                did,
                claim_key,
                expiry,
                claim_value,
            } in claims {
                ensure!(<DidRecords>::exists(did), "DID must already exist");
                let claim_meta_data = ClaimMetaData {
                    claim_key: claim_key.clone(),
                    claim_issuer: did_issuer.clone(),
                };
                let now = <pallet_timestamp::Module<T>>::get();
                let claim = Claim {
                    issuance_date: now,
                    expiry: expiry.clone(),
                    claim_value: claim_value.clone(),
                };
                checked_claims.push((did.clone(), claim_meta_data, claim));
            }
            // Register the claims.
            for (did, claim_meta_data, claim) in checked_claims {
                <Claims<T>>::insert((did.clone(), claim_meta_data.clone()), claim.clone());
                <ClaimKeys>::mutate(&did, |old_claim_data| {
                    if !old_claim_data.contains(&claim_meta_data) {
                        old_claim_data.push(claim_meta_data.clone());
                    }
                });
                Self::deposit_event(RawEvent::NewClaims(did, claim_meta_data, claim));
            }
            Ok(())
        }

        fn forwarded_call(origin, target_did: IdentityId, proposal: Box<T::Proposal>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // 1. Constraints.
            // 1.1. A valid current identity.
            if let Some(current_did) = <CurrentDid>::get() {
                // 1.2. Check that current_did is a signing key of target_did
                ensure!( Self::is_signer_authorized(current_did, &Signatory::Identity(target_did)),
                    "Current identity cannot be forwarded, it is not a signing key of target identity");
            } else {
                return Err(Error::<T>::MissingCurrentIdentity.into());
            }

            // 1.3. Check that target_did has a KYC.
            // Please keep in mind that `current_did` is double-checked:
            //  - by `SignedExtension` (`update_did_signed_extension`) on 0 level nested call, or
            //  - by next code, as `target_did`, on N-level nested call, where N is equal or greater that 1.
            ensure!(Self::has_valid_kyc(target_did), "Invalid KYC validation on target did");

            // 2. Actions
            <CurrentDid>::put(target_did);

            // Also set current_did roles when acting as a signing key for target_did
            // Re-dispatch call - e.g. to asset::doSomething...
            let new_origin = frame_system::RawOrigin::Signed(sender).into();

            let _res = match proposal.dispatch(new_origin) {
                Ok(_) => true,
                Err(e) => {
                    let e: DispatchError = e.into();
                    sp_runtime::print(e);
                    false
                }
            };

            Ok(())
        }

        /// Marks the specified claim as revoked
        pub fn revoke_claim(origin, did: IdentityId, claim_key: Vec<u8>, did_issuer: IdentityId) -> DispatchResult {
            let sender = Signatory::AccountKey( AccountKey::try_from( ensure_signed(origin)?.encode())?);

            ensure!(<DidRecords>::exists(&did), "DID must already exist");
            ensure!(<DidRecords>::exists(&did_issuer), "claim issuer DID must already exist");

            // Verify that sender key is one of did_issuer's signing keys
            ensure!(Self::is_signer_authorized(did_issuer, &sender), "Sender must hold a claim issuer's signing key");

            let claim_meta_data = ClaimMetaData {
                claim_key: claim_key,
                claim_issuer: did_issuer,
            };

            <Claims<T>>::remove((did, claim_meta_data.clone()));

            <ClaimKeys>::mutate(&did, |old_claim_metadata| {
                *old_claim_metadata = old_claim_metadata
                    .iter()
                    .filter(|&metadata| *metadata != claim_meta_data)
                    .cloned()
                    .collect();
            });

            Self::deposit_event(RawEvent::RevokedClaim(did, claim_meta_data));

            Ok(())
        }

        /// It sets permissions for an specific `target_key` key.
        /// Only the master key of an identity is able to set signing key permissions.
        pub fn set_permission_to_signer(origin, did: IdentityId, signer: Signatory, permissions: Vec<Permission>) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;
            let record = Self::grant_check_only_master_key( &sender_key, did)?;

            // You are trying to add a permission to did's master key. It is not needed.
            if let Signatory::AccountKey(ref key) = signer {
                if record.master_key == *key {
                    return Ok(());
                }
            }

            // Find key in `DidRecord::signing_keys`
            if record.signing_items.iter().find(|&si| si.signer == signer).is_some() {
                Self::update_signing_item_permissions(did, &signer, permissions)
            } else {
                Err(Error::<T>::InvalidSender.into())
            }
        }

        /// It disables all signing keys at `did` identity.
        ///
        /// # Errors
        ///
        pub fn freeze_signing_keys(origin, did: IdentityId) -> DispatchResult {
            Self::set_frozen_signing_key_flags( origin, did, true)
        }

        pub fn unfreeze_signing_keys(origin, did: IdentityId) -> DispatchResult {
            Self::set_frozen_signing_key_flags( origin, did, false)
        }

        pub fn get_my_did(origin) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            if let Some(did) = Self::get_identity(&sender_key) {
                Self::deposit_event(RawEvent::DidQuery(sender_key, did));
                sp_runtime::print(did);
                Ok(())
            } else {
                Err(Error::<T>::NoDIDFound.into())
            }
        }

        pub fn get_asset_did(origin, ticker: Ticker) -> DispatchResult {
            ensure_signed(origin)?;
            let did = Self::get_token_did(&ticker)?;
            Self::deposit_event(RawEvent::AssetDid(ticker, did));
            sp_runtime::print(did);
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
            let from_did =  match Self::current_did() {
                Some(x) => x,
                None => {
                    if let Some(did) = Self::get_identity(&sender_key) {
                        did
                    } else {
                        return Err(Error::<T>::NoDIDFound.into());
                    }
                }
            };

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
            let from_did =  match Self::current_did() {
                Some(x) => x,
                None => {
                    if let Some(did) = Self::get_identity(&sender_key) {
                        did
                    } else {
                        return Err(Error::<T>::NoDIDFound.into());
                    }
                }
            };

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
            let from_did =  match Self::current_did() {
                Some(x) => x,
                None => {
                    if let Some(did) = Self::get_identity(&sender_key) {
                        did
                    } else {
                        return Err(Error::<T>::NoDIDFound.into());
                    }
                }
            };

            ensure!(<Authorizations<T>>::exists(target, auth_id), "Invalid auth");

            let auth = <Authorizations<T>>::get(target, auth_id);

            ensure!(auth.authorized_by.eq_either(&from_did, &sender_key) || target.eq_either(&from_did, &sender_key) , "Unauthorized");

            Self::remove_auth(target, auth_id, auth.authorized_by);

            Ok(())
        }

        /// Removes an array of authorizations
        pub fn batch_remove_authorization(
            origin,
            // Vec<(target_did, auth_id)>
            auth_identifiers: Vec<AuthIdentifier>
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let from_did =  match Self::current_did() {
                Some(x) => x,
                None => {
                    if let Some(did) = Self::get_identity(&sender_key) {
                        did
                    } else {
                        return Err(Error::<T>::NoDIDFound.into());
                    }
                }
            };

            let mut auths = Vec::with_capacity(auth_identifiers.len());
            for i in 0..auth_identifiers.len() {
                let auth_identifier = &auth_identifiers[i];
                ensure!(<Authorizations<T>>::exists(&auth_identifier.0, &auth_identifier.1), "Invalid auth");

                auths.push(<Authorizations<T>>::get(&auth_identifier.0, &auth_identifier.1));
                ensure!(auths[i].authorized_by.eq_either(&from_did, &sender_key) || auth_identifier.0.eq_either(&from_did, &sender_key) , "Unauthorized");
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
            let signer = match Self::current_did() {
                Some(x) => Signatory::from(x),
                None => {
                    if let Some(did) = Self::get_identity(&sender_key) {
                        Signatory::from(did)
                    } else {
                        Signatory::from(sender_key)
                    }
                }
            };

            ensure!(<Authorizations<T>>::exists(signer, auth_id), "Invalid auth");
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
                        _ => return Err(Error::<T>::UnknownAuthorization.into())
                    }
                },
                Signatory::AccountKey(key) => {
                    match auth.authorization_data {
                        AuthorizationData::AddMultiSigSigner =>
                            T::AddSignerMultiSigTarget::accept_multisig_signer(Signatory::from(key), auth_id),
                        _ => return Err(Error::<T>::UnknownAuthorization.into())
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
            let signer = match Self::current_did() {
                Some(x) => Signatory::from(x),
                None => {
                    if let Some(did) = Self::get_identity(&sender_key) {
                        Signatory::from(did)
                    } else {
                        Signatory::from(sender_key)
                    }
                }
            };

            match signer {
                Signatory::Identity(did) => {
                    for auth_id in auth_ids {
                        // NB: Even if an auth is invalid (due to any reason), this batch function does NOT return an error.
                        // It will just skip that particular authorization.
                        if <Authorizations<T>>::exists(signer, auth_id) {
                            let auth = <Authorizations<T>>::get(signer, auth_id);
                            // NB: Result is not handled, invalid auths are just ignored to let the batch function continue.
                            let _result = match auth.authorization_data {
                                AuthorizationData::TransferTicker(_) =>
                                    T::AcceptTransferTarget::accept_ticker_transfer(did, auth_id),
                                AuthorizationData::TransferTokenOwnership(_) =>
                                    T::AcceptTransferTarget::accept_token_ownership_transfer(did, auth_id),
                                AuthorizationData::AddMultiSigSigner =>
                                    T::AddSignerMultiSigTarget::accept_multisig_signer(Signatory::from(did), auth_id),
                                _ => Err(Error::<T>::UnknownAuthorization.into())
                            };
                        }
                    }
                },
                Signatory::AccountKey(key) => {
                    for auth_id in auth_ids {
                        // NB: Even if an auth is invalid (due to any reason), this batch function does NOT return an error.
                        // It will just skip that particular authorization.
                        if <Authorizations<T>>::exists(signer, auth_id) {
                            let auth = <Authorizations<T>>::get(signer, auth_id);
                            //NB: Result is not handled, invalid auths are just ignored to let the batch function continue.
                            let _result = match auth.authorization_data {
                                AuthorizationData::AddMultiSigSigner =>
                                    T::AddSignerMultiSigTarget::accept_multisig_signer(Signatory::from(key), auth_id),
                                _ => Err(Error::<T>::UnknownAuthorization.into())
                            };
                        }
                    }
                }
            }

            Ok(())
        }

        // Manage Authorizations to join to an Identity
        // ================================================

        /// The key designated by `origin` accepts the authorization to join to `target_id`
        /// Identity.
        ///
        /// # Errors
        ///  - AccountKey should be authorized previously to join to that target identity.
        ///  - AccountKey is not linked to any other identity.
        pub fn authorize_join_to_identity(origin, target_id: IdentityId) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;
            let signer_from_key = Signatory::AccountKey( sender_key.clone());
            let signer_id_found = Self::key_to_identity_ids(sender_key);

            // Double check that `origin` (its key or identity) has been pre-authorize.
            let valid_signer = if <PreAuthorizedJoinDid>::exists(&signer_from_key) {
                // Sender key is valid.
                // Verify 1-to-1 relation between key and identity.
                if signer_id_found.is_some() {
                    return Err(Error::<T>::AlreadyLinked.into());
                }
                Some( signer_from_key)
            } else {
                // Otherwise, sender's identity (only master key) should be pre-authorize.
                match signer_id_found {
                    Some( LinkedKeyInfo::Unique(sender_id)) if Self::is_master_key(sender_id, &sender_key) => {
                        let signer_from_id = Signatory::Identity(sender_id);
                        if <PreAuthorizedJoinDid>::exists(&signer_from_id) {
                            Some(signer_from_id)
                        } else {
                            None
                        }
                    },
                    _ => None
                }
            };

            // Only works with a valid signer.
            if let Some(signer) = valid_signer {
                if let Some(pre_auth) = Self::pre_authorized_join_did( signer.clone())
                        .iter()
                        .find( |pre_auth_item| pre_auth_item.target_id == target_id) {
                    // Remove pre-auth, link key to identity and update identity record.
                    Self::remove_pre_join_identity(&signer, target_id);
                    if let Signatory::AccountKey(key) = signer {
                        Self::link_key_to_did( &key, pre_auth.signing_item.signer_type, target_id);
                    }
                    <DidRecords>::mutate( target_id, |identity| {
                        identity.add_signing_items( &[pre_auth.signing_item.clone()]);
                    });
                    Self::deposit_event( RawEvent::SignerJoinedToIdentityApproved( signer, target_id));
                    Ok(())
                } else {
                    Err(Error::<T>::Unauthorized.into())
                }
            } else {
                Err(Error::<T>::Unauthorized.into())
            }
        }

        /// Identity's master key or target key are allowed to reject a pre authorization to join.
        /// It only affects the authorization: if key accepted it previously, then this transaction
        /// shall have no effect.
        pub fn unauthorized_join_to_identity(origin, signer: Signatory, target_id: IdentityId) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;

            let mut is_remove_allowed = Self::is_master_key( target_id, &sender_key);

            if !is_remove_allowed {
                is_remove_allowed = match signer {
                    Signatory::AccountKey(ref key) => sender_key == *key,
                    Signatory::Identity(id) => Self::is_master_key(id, &sender_key)
                }
            }

            if is_remove_allowed {
                Self::remove_pre_join_identity( &signer, target_id);
                Ok(())
            } else {
                Err(Error::<T>::Unauthorized.into())
            }
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
        pub fn add_signing_items_with_authorization( origin,
                id: IdentityId,
                expires_at: T::Moment,
                additional_keys: Vec<SigningItemWithAuth>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, id)?;

            // 0. Check expiration
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!( now < expires_at, "Offchain authorization has expired");
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
                    Signatory::Identity(ref id) if <DidRecords>::exists(id) => {
                        let master_key = <DidRecords>::get(id).master_key;
                        Public::try_from( master_key.as_slice()).ok()
                    },
                    _ => None
                };

                if let Some(account_id) = account_id_found {
                    if let Signatory::AccountKey(ref key) = si.signer {
                        // 1.1. Constraint 1-to-1 account to DID
                        ensure!( Self::can_key_be_linked_to_did( key, si.signer_type),
                        "One signing key can only belong to one identity");
                    }

                    // 1.2. Offchain authorization is not revoked explicitly.
                    ensure!( Self::is_offchain_authorization_revoked((si.signer.clone(), authorization.clone())) == false,
                        "Authorization has been explicitly revoked");

                    // 1.3. Verify the signature.
                    let signature = AnySignature::from( Signature::from_h512(si_with_auth.auth_signature));
                    ensure!( signature.verify( auth_encoded.as_slice(), &account_id),
                        "Invalid Authorization signature");
                } else {
                    return Err(Error::<T>::InvalidAccountKey.into());
                }
            }

            // 2.1. Link keys to identity
            additional_keys.iter().for_each( |si_with_auth| {
                let si = & si_with_auth.signing_item;
                if let Signatory::AccountKey(ref key) = si.signer {
                    Self::link_key_to_did( key, si.signer_type, id);
                }
            });

            // 2.2. Update that identity information and its offchain authorization nonce.
            <DidRecords>::mutate( id, |record| {
                let keys = additional_keys.iter().map( |si_with_auth| si_with_auth.signing_item.clone())
                    .collect::<Vec<_>>();
                (*record).add_signing_items( &keys[..]);
            });
            <OffChainAuthorizationNonce>::mutate( id, |offchain_nonce| {
                *offchain_nonce = authorization.nonce + 1;
            });

            Ok(())
        }

        /// It revokes the `auth` off-chain authorization of `signer`. It only takes effect if
        /// the authorized transaction is not yet executed.
        pub fn revoke_offchain_authorization(origin, signer: Signatory, auth: TargetIdAuthorization<T::Moment>) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;

            match signer {
                Signatory::AccountKey(ref key) => ensure!( sender_key == *key, "This key is not allowed to revoke this off-chain authorization"),
                Signatory::Identity(id) => ensure!( Self::is_master_key(id, &sender_key), "Only master key is allowed to revoke an Identity Signatory off-chain authorization"),
            }

            <RevokeOffChainAuthorization<T>>::insert( (signer,auth), true);
            Ok(())
        }

        /// Query whether given signer identity has valid KYC or not
        ///
        /// # Arguments
        /// * `origin` Signatory whose identity get checked
        /// * `buffer_time` Buffer time corresponds to which kyc expiry need to check
        pub fn is_my_identity_has_valid_kyc(origin, buffer_time: u64) ->  DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let my_did =  match Self::current_did() {
                Some(x) => x,
                None => {
                    if let Some(did) = Self::get_identity(&sender_key) {
                        did
                    } else {
                        return Err(Error::<T>::NoDIDFound.into());
                    }
                }
            };
            let (is_kyced, kyc_provider) = Self::is_identity_has_valid_kyc(my_did, buffer_time);
            Self::deposit_event(RawEvent::MyKycStatus(my_did, is_kyced, kyc_provider));
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
    }
}

impl<T: Trait> Module<T> {
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
            expiry: expiry,
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
        if !<Authorizations<T>>::exists(target, auth_id) {
            // Auth does not exist
            return Err(AuthorizationError::Invalid.into());
        }
        let auth = <Authorizations<T>>::get(target, auth_id);
        if auth.authorized_by != from {
            // Not authorized to revoke this authorization
            return Err(AuthorizationError::Unauthorized.into());
        }
        if let Some(expiry) = auth.expiry {
            let now = <pallet_timestamp::Module<T>>::get();
            if expiry <= now {
                return Err(AuthorizationError::Expired.into());
            }
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
            expiry: expiry,
            link_id: new_nonce,
        };

        <Links<T>>::insert(target, new_nonce, link);

        Self::deposit_event(RawEvent::NewLink(new_nonce, target, link_data, expiry));
        new_nonce
    }

    /// Remove a link (if it exists) from a key or identity
    /// NB: Please do all the required checks before calling this function.
    pub fn remove_link(target: Signatory, link_id: u64) {
        if <Links<T>>::exists(target, link_id) {
            <Links<T>>::remove(target, link_id);
            Self::deposit_event(RawEvent::LinkRemoved(link_id, target));
        }
    }

    /// Update link data (if it exists) from a key or identity
    /// NB: Please do all the required checks before calling this function.
    pub fn update_link(target: Signatory, link_id: u64, link_data: LinkData) {
        if <Links<T>>::exists(target, link_id) {
            <Links<T>>::mutate(target, link_id, |link| link.link_data = link_data);

            Self::deposit_event(RawEvent::LinkUpdated(link_id, target));
        }
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

    pub fn fetch_claim_value(
        did: IdentityId,
        claim_key: Vec<u8>,
        claim_issuer: IdentityId,
    ) -> Option<ClaimValue> {
        let claim_meta_data = ClaimMetaData {
            claim_key,
            claim_issuer,
        };
        if <Claims<T>>::exists((did, claim_meta_data.clone())) {
            let now = <pallet_timestamp::Module<T>>::get();
            let claim = <Claims<T>>::get((did, claim_meta_data));
            if claim.expiry > now {
                return Some(claim.claim_value);
            }
        }
        None
    }

    pub fn fetch_claim_value_multiple_issuers(
        did: IdentityId,
        claim_key: Vec<u8>,
        claim_issuers: Vec<IdentityId>,
    ) -> Option<ClaimValue> {
        for claim_issuer in claim_issuers {
            let claim_value = Self::fetch_claim_value(did, claim_key.clone(), claim_issuer);
            if claim_value.is_some() {
                return claim_value;
            }
        }
        None
    }

    pub fn is_identity_has_valid_kyc(
        claim_for: IdentityId,
        buffer: u64,
    ) -> (bool, Option<IdentityId>) {
        let trusted_kyc_providers = T::KycServiceProviders::get_members();
        if trusted_kyc_providers.len() > 0 {
            for trusted_kyc_provider in trusted_kyc_providers {
                if let Some(claim) = Self::fetch_claim_value(
                    claim_for,
                    KYC_EXPIRY_CLAIM_KEY.to_vec(),
                    trusted_kyc_provider,
                ) {
                    if let Ok(value) = claim.value.as_slice().try_into() {
                        if let Some(threshold) = ((<pallet_timestamp::Module<T>>::get())
                            .saturated_into::<u64>())
                        .checked_add(buffer)
                        {
                            if u64::from_be_bytes(value) > threshold {
                                return (true, Some(trusted_kyc_provider));
                            }
                        }
                    }
                }
            }
        }
        return (false, None);
    }

    /// It checks that `sender_key` is the master key of `did` Identifier and that
    /// did exists.
    /// # Return
    /// A result object containing the `DidRecord` of `did`.
    pub fn grant_check_only_master_key(
        sender_key: &AccountKey,
        did: IdentityId,
    ) -> sp_std::result::Result<DidRecord, &'static str> {
        ensure!(<DidRecords>::exists(did), "DID does not exist");
        let record = <DidRecords>::get(did);
        ensure!(
            *sender_key == record.master_key,
            "Only master key of an identity is able to execute this operation"
        );

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
        return None;
    }

    /// It freezes/unfreezes the target `did` identity.
    ///
    /// # Errors
    /// Only master key can freeze/unfreeze an identity.
    fn set_frozen_signing_key_flags(
        origin: T::Origin,
        did: IdentityId,
        freeze: bool,
    ) -> DispatchResult {
        let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
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
            match linked_key_info {
                LinkedKeyInfo::Group(mut dids) => {
                    if !dids.contains(&did) && key_type != SignatoryType::External {
                        dids.push(did);
                        dids.sort();

                        <KeyToIdentityIds>::insert(key, LinkedKeyInfo::Group(dids));
                    }
                }
                _ => {
                    // This case is protected by `can_key_be_linked_to_did`.
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

    /// It set/reset the current identity.
    pub fn set_current_did(did_opt: Option<IdentityId>) {
        if let Some(did) = did_opt {
            <CurrentDid>::put(did);
        } else {
            <CurrentDid>::kill();
        }
    }
    /// It adds `signing_item` to pre authorized items for `id` identity.
    fn add_pre_join_identity(signing_item: &SigningItem, id: IdentityId) {
        let signer = &signing_item.signer;
        let new_pre_auth = PreAuthorizedKeyInfo::new(signing_item.clone(), id);

        if !<PreAuthorizedJoinDid>::exists(signer) {
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
        ensure!(!<DidRecords>::exists(did), "DID must be unique");
        <DidRecords>::insert(did, DidRecord::default());
        Ok(())
    }

    /// It is a helper function that can be used to get did for any asset
    pub fn get_token_did(ticker: &Ticker) -> StdResult<IdentityId, &'static str> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&SECURITY_TOKEN.encode());
        buf.extend_from_slice(&ticker.encode());
        IdentityId::try_from(T::Hashing::hash(&buf[..]).as_ref())
    }

    pub fn _register_did(sender: T::AccountId, signing_items: Vec<SigningItem>) -> DispatchResult {
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
            "Master key already belong to one DID"
        );
        // 1.2. Master key is not part of signing keys.
        ensure!(
            signing_items.iter().find(|sk| **sk == master_key).is_none(),
            "Signing keys contains the master key"
        );

        let block_hash = <system::Module<T>>::block_hash(<system::Module<T>>::block_number());

        let did = IdentityId::from(blake2_256(&(USER, block_hash, new_nonce).encode()));

        // 1.3. Make sure there's no pre-existing entry for the DID
        // This should never happen but just being defensive here
        ensure!(!<DidRecords>::exists(did), "DID must be unique");
        // 1.4. Signing keys can be linked to the new identity.
        for s_item in &signing_items {
            if let Signatory::AccountKey(ref key) = s_item.signer {
                if !Self::can_key_be_linked_to_did(key, s_item.signer_type) {
                    return Err(Error::<T>::AlreadyLinked.into());
                }
            }
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
        <DidRecords>::insert(did, record);

        // TODO KYC is valid by default.
        KYCValidation::insert(did, true);

        Self::deposit_event(RawEvent::NewDid(did, sender, signing_items));
        Ok(())
    }
}

impl<T: Trait> IdentityTrait for Module<T> {
    fn get_identity(key: &AccountKey) -> Option<IdentityId> {
        Self::get_identity(&key)
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
}
