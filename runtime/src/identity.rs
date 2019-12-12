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
//! [add_claim_issuer](./struct.Module.html#method.add_claim_issuer).
//!
//! ## Identity information
//!
//! Identity contains the following data:
//!  - `master_key`. It is the administrator account of the identity.
//!  - `signing_keys`. List of keys and their capabilities (type of key and its permissions) .
//!
//! ## Claim Issuers
//!
//! The administrator of the entity can add/remove claim issuers (see
//! [add_claim_issuer](./struct.Module.html#method.add_claim_issuer) ). Only these claim issuers
//! are able to add claims to that identity.
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

use rstd::{convert::TryFrom, prelude::*};

use crate::balances;
use crate::constants::did::USER;
use primitives::{
    Identity as DidRecord, IdentityId, Key, Permission, PreAuthorizedKeyInfo, Signer, SignerType,
    SigningItem,
};

use codec::Encode;
use sr_io::blake2_256;
use sr_primitives::{traits::Dispatchable, DispatchError};
use srml_support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, WithdrawReason},
    Parameter,
};
use system::{self, ensure_signed};

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Claim<U> {
    issuance_date: U,
    expiry: U,
    claim_value: ClaimValue,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct ClaimMetaData {
    claim_key: Vec<u8>,
    claim_issuer: IdentityId,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct ClaimValue {
    pub data_type: DataTypes,
    pub value: Vec<u8>,
}

#[derive(codec::Encode, codec::Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum DataTypes {
    U8,
    U16,
    U32,
    U64,
    U128,
    Bool,
    VecU8,
}

impl Default for DataTypes {
    fn default() -> Self {
        DataTypes::VecU8
    }
}

/// Keys could be linked to several identities (`IdentityId`) as master key or signing key.
/// Master key or external type signing key are restricted to be linked to just one identity.
/// Other types of signing key could be associated with more that one identity.
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub enum LinkedKeyInfo {
    Unique(IdentityId),
    Group(Vec<IdentityId>),
}

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait + timestamp::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// An extrinsic call.
    type Proposal: Parameter + Dispatchable<Origin = Self::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as identity {

        /// Module owner.
        Owner get(owner) config(): T::AccountId;

        /// DID -> identity info
        pub DidRecords get(did_records): map IdentityId => DidRecord;

        /// DID -> bool that indicates if signing keys are frozen.
        pub IsDidFrozen get(is_did_frozen): map IdentityId => bool;

        /// DID -> DID claim issuers
        pub ClaimIssuers get(claim_issuers): map IdentityId => Vec<IdentityId>;

        /// It stores the current identity for current transaction.
        pub CurrentDid get(current_did): Option<IdentityId>;

        /// (DID, claim_key, claim_issuer) -> Associated claims
        pub Claims get(claims): map(IdentityId, ClaimMetaData) => Claim<T::Moment>;

        /// DID -> array of (claim_key and claim_issuer)
        pub ClaimKeys get(claim_keys): map IdentityId => Vec<ClaimMetaData>;

        // Account => DID
        pub KeyToIdentityIds get(key_to_identity_ids): map Key => Option<LinkedKeyInfo>;

        /// How much does creating a DID cost
        pub DidCreationFee get(did_creation_fee) config(): T::Balance;

        /// It stores validated identities by any KYC.
        pub KYCValidation get(has_valid_kyc): map IdentityId => bool;

        /// Nonce to ensure unique DIDs are generated. starts from 1.
        pub DidNonce get(did_nonce) build(|_| 1u128): u128;

        /// Pre-authorize join to Identity.
        pub PreAuthorizedJoinDid get( pre_authorized_join_did): map Signer => Vec<PreAuthorizedKeyInfo>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
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
        pub fn register_did(origin, signing_items: Vec<SigningItem>) -> Result {
            let sender = ensure_signed(origin)?;
            // Adding extrensic count to did nonce for some unpredictability
            // NB: this does not guarantee randomness
            let new_nonce = Self::did_nonce() + u128::from(<system::Module<T>>::extrinsic_count()) + 7u128;
            // Even if this transaction fails, nonce should be increased for added unpredictability of dids
            <DidNonce>::put(&new_nonce);

            let master_key = Key::try_from( sender.encode())?;

            // 1 Check constraints.
            // 1.1. Master key is not linked to any identity.
            ensure!( Self::can_key_be_linked_to_did( &master_key, SignerType::External),
                "Master key already belong to one DID");
            // 1.2. Master key is not part of signing keys.
            ensure!( signing_items.iter().find( |sk| **sk == master_key).is_none(),
                "Signing keys contains the master key");

            let block_hash = <system::Module<T>>::block_hash(<system::Module<T>>::block_number());

            let did = IdentityId::from(
                blake2_256(
                    &(USER, block_hash, new_nonce).encode()
                )
            );

            // 1.3. Make sure there's no pre-existing entry for the DID
            // This should never happen but just being defensive here
            ensure!(!<DidRecords>::exists(did), "DID must be unique");
            // 1.4. Signing keys can be linked to the new identity.
            for s_item in &signing_items {
                if let Signer::Key(ref key) = s_item.signer {
                    if !Self::can_key_be_linked_to_did( key, s_item.signer_type){
                        return Err("One signing key can only belong to one DID");
                    }
                }
            }

            // 2. Apply changes to our extrinsics.
            // TODO: Subtract the fee
            let _imbalance = <balances::Module<T> as Currency<_>>::withdraw(
                &sender,
                Self::did_creation_fee(),
                WithdrawReason::Fee,
                ExistenceRequirement::KeepAlive
                )?;

            // 2.1. Link  master key and add pre-authorized signing keys
            Self::link_key_to_did( &master_key, SignerType::External, did);
            signing_items.iter().for_each( |s_item| Self::add_pre_join_identity( s_item, did));

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

        /// Adds new signing keys for a DID. Only called by master key owner.
        ///
        /// # Failure
        ///  - It can only called by master key owner.
        ///  - If any signing key is already linked to any identity, it will fail.
        ///  - If any signing key is already
        pub fn add_signing_items(origin, did: IdentityId, signing_items: Vec<SigningItem>) -> Result {
            let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            // Check constraint 1-to-1 in relation key-identity.
            for s_item in &signing_items{
                if let Signer::Key(ref key) = s_item.signer {
                    if !Self::can_key_be_linked_to_did( key, s_item.signer_type) {
                        return Err( "One signing key can only belong to one DID");
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
        fn remove_signing_items(origin, did: IdentityId, signers_to_remove: Vec<Signer>) -> Result {
            let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            // Remove any Pre-Authentication & link
            signers_to_remove.iter().for_each( |signer| {
                Self::remove_pre_join_identity( signer, did);
                if let Signer::Key(ref key) = signer {
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
        fn set_master_key(origin, did: IdentityId, new_key: Key) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from( sender.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            ensure!( Self::can_key_be_linked_to_did(&new_key, SignerType::External), "Master key can only belong to one DID");

            <DidRecords>::mutate(did,
            |record| {
                (*record).master_key = new_key.clone();
            });

            Self::deposit_event(RawEvent::NewMasterKey(did, sender, new_key));
            Ok(())
        }

        /// Appends a claim issuer DID to a DID. Only called by master key owner.
        pub fn add_claim_issuer(origin, did: IdentityId, claim_issuer_did: IdentityId) -> Result {
            let sender_key = Key::try_from( ensure_signed(origin)?.encode())?;
            let _grant_checked = Self::grant_check_only_master_key( &sender_key, did)?;

            // Master key shouldn't be added itself as claim issuer.
            ensure!( did != claim_issuer_did, "Master key cannot add itself as claim issuer");

            <ClaimIssuers>::mutate(did, |old_claim_issuers| {
                if !old_claim_issuers.contains(&claim_issuer_did) {
                    old_claim_issuers.push(claim_issuer_did);
                }
            });

            Self::deposit_event(RawEvent::NewClaimIssuer(did, claim_issuer_did));
            Ok(())
        }

        /// Removes a claim issuer DID. Only called by master key owner.
        fn remove_claim_issuer(origin, did: IdentityId, did_issuer: IdentityId) -> Result {
            let sender_key = Key::try_from( ensure_signed(origin)?.encode())?;
            let _grant_checked = Self::grant_check_only_master_key( &sender_key, did)?;

            ensure!(<DidRecords>::exists(did_issuer), "claim issuer DID must already exist");

            <ClaimIssuers>::mutate(did, |old_claim_issuers| {
                *old_claim_issuers = old_claim_issuers
                    .iter()
                    .filter(|&issuer| *issuer != did_issuer)
                    .cloned()
                    .collect();
            });

            Self::deposit_event(RawEvent::RemovedClaimIssuer(did, did_issuer));
            Ok(())
        }

        /// Adds new claim record or edits an existing one. Only called by did_issuer's signing key
        pub fn add_claim(
            origin,
            did: IdentityId,
            claim_key: Vec<u8>,
            did_issuer: IdentityId,
            expiry: <T as timestamp::Trait>::Moment,
            claim_value: ClaimValue
        ) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords>::exists(did), "DID must already exist");
            ensure!(<DidRecords>::exists(did_issuer), "claim issuer DID must already exist");

            let sender_key = Key::try_from( sender.encode())?;
            ensure!(Self::is_claim_issuer(did, did_issuer) || Self::is_master_key(did, &sender_key), "did_issuer must be a claim issuer or master key for DID");

            // Verify that sender key is one of did_issuer's signing keys
            let sender_signer = Signer::Key( sender_key);
            ensure!(Self::is_signer_authorized(did_issuer, &sender_signer), "Sender must hold a claim issuer's signing key");

            let claim_meta_data = ClaimMetaData {
                claim_key: claim_key,
                claim_issuer: did_issuer,
            };

            let now = <timestamp::Module<T>>::get();

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

        fn forwarded_call(origin, target_did: IdentityId, proposal: Box<T::Proposal>) -> Result {
            let sender = ensure_signed(origin)?;

            // 1. Constraints.
            // 1.1. A valid current identity.
            if let Some(current_did) = <CurrentDid>::get() {
                // 1.2. Check that current_did is a signing key of target_did
                ensure!( Self::is_signer_authorized(current_did, &Signer::Identity(target_did)),
                    "Current identity cannot be forwarded, it is not a signing key of target identity");
            } else {
                return Err("Missing current identity on the transaction");
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
            let new_origin = system::RawOrigin::Signed(sender).into();

            let _res = match proposal.dispatch(new_origin) {
                Ok(_) => true,
                Err(e) => {
                    let e: DispatchError = e.into();
                    sr_primitives::print(e);
                    false
                }
            };

            Ok(())
        }

        /// Marks the specified claim as revoked
        pub fn revoke_claim(origin, did: IdentityId, claim_key: Vec<u8>, did_issuer: IdentityId) -> Result {
            let sender = Signer::Key( Key::try_from( ensure_signed(origin)?.encode())?);

            ensure!(<DidRecords>::exists(&did), "DID must already exist");
            ensure!(<DidRecords>::exists(&did_issuer), "claim issuer DID must already exist");

            // Verify that sender key is one of did_issuer's signing keys
            ensure!(Self::is_signer_authorized(did_issuer, &sender), "Sender must hold a claim issuer's signing key");

            let claim_meta_data = ClaimMetaData {
                claim_key: claim_key,
                claim_issuer: did_issuer,
            };

            <Claims<T>>::remove((did.clone(), claim_meta_data.clone()));

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
        fn set_permission_to_signer(origin, did: IdentityId, signer: Signer, permissions: Vec<Permission>) -> Result {
            let sender_key = Key::try_from( ensure_signed(origin)?.encode())?;
            let record = Self::grant_check_only_master_key( &sender_key, did)?;

            // You are trying to add a permission to did's master key. It is not needed.
            if let Signer::Key(ref key) = signer {
                if record.master_key == *key {
                    return Ok(());
                }
            }

            // Find key in `DidRecord::signing_keys`
            if record.signing_items.iter().find(|&si| si.signer == signer).is_some() {
                Self::update_signing_item_permissions(did, &signer, permissions)
            } else {
                Err( "Sender is not part of did's signing keys")
            }
        }

        /// It disables all signing keys at `did` identity.
        ///
        /// # Errors
        ///
        fn freeze_signing_keys(origin, did: IdentityId) -> Result {
            Self::set_frozen_signing_key_flags( origin, did, true)
        }

        fn unfreeze_signing_keys(origin, did: IdentityId) -> Result {
            Self::set_frozen_signing_key_flags( origin, did, false)
        }

        pub fn get_my_did(origin) -> Result {
            let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
            if let Some(did) = Self::get_identity(&sender_key) {
                Self::deposit_event(RawEvent::DidQuery(sender_key, did));
                sr_primitives::print(did);
                Ok(())
            } else {
                Err("No did linked to the user")
            }
        }


        // Manage Authorizations to join to an Identity
        // ================================================

        /// The key designated by `origin` accepts the authorization to join to `target_id`
        /// Identity.
        ///
        /// # Errors
        ///  - Key should be authorized previously to join to that target identity.
        ///  - Key is not linked to any other identity.
        pub fn authorize_join_to_identity(origin, target_id: IdentityId) -> Result {
            let sender_key = Key::try_from( ensure_signed(origin)?.encode())?;
            let signer_from_key = Signer::Key( sender_key.clone());
            let signer_id_found = Self::key_to_identity_ids(sender_key);

            // Double check that `origin` (its key or identity) has been pre-authorize.
            let valid_signer = if <PreAuthorizedJoinDid>::exists(&signer_from_key) {
                // Sender key is valid.
                // Verify 1-to-1 relation between key and identity.
                if signer_id_found.is_some() {
                    return Err("Key is already linked to an identity");
                }
                Some( signer_from_key)
            } else {
                // Otherwise, sender's identity (only master key) should be pre-authorize.
                match signer_id_found {
                    Some( LinkedKeyInfo::Unique(sender_id)) if Self::is_master_key(sender_id, &sender_key) => {
                        let signer_from_id = Signer::Identity(sender_id);
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
                    if let Signer::Key(key) = signer {
                        Self::link_key_to_did( &key, pre_auth.signing_item.signer_type, target_id);
                    }
                    <DidRecords>::mutate( target_id, |identity| {
                        identity.add_signing_items( &[pre_auth.signing_item.clone()]);
                    });
                    Ok(())
                } else {
                    Err( "Signer is not pre authorized by the identity")
                }
            } else {
                Err( "Signer is not pre authorized by any identity")
            }
        }

        /// Identity's master key or target key are allowed to reject a pre authorization to join.
        /// It only affects the authorization: if key accepted it previously, then this transaction
        /// shall have no effect.
        pub fn unauthorized_join_to_identity(origin, signer: Signer, target_id: IdentityId) -> Result {
            let sender_key = Key::try_from( ensure_signed(origin)?.encode())?;

            let mut is_remove_allowed = Self::is_master_key( target_id, &sender_key);

            if !is_remove_allowed {
                is_remove_allowed = match signer {
                    Signer::Key(ref key) => sender_key == *key,
                    Signer::Identity(id) => Self::is_master_key(id, &sender_key)
                }
            }

            if is_remove_allowed {
                Self::remove_pre_join_identity( &signer, target_id);
                Ok(())
            } else {
                Err("Account cannot remove this authorization")
            }
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Moment = <T as timestamp::Trait>::Moment,
    {
        /// DID, master key account ID, signing keys
        NewDid(IdentityId, AccountId, Vec<SigningItem>),

        /// DID, new keys
        NewSigningItems(IdentityId, Vec<SigningItem>),

        /// DID, the keys that got removed
        RevokedSigningItems(IdentityId, Vec<Signer>),

        /// DID, updated signing key, previous permissions
        SigningPermissionsUpdated(IdentityId, SigningItem, Vec<Permission>),

        /// DID, old master key account ID, new key
        NewMasterKey(IdentityId, AccountId, Key),

        /// DID, claim issuer DID
        NewClaimIssuer(IdentityId, IdentityId),

        /// DID, removed claim issuer DID
        RemovedClaimIssuer(IdentityId, IdentityId),

        /// DID, claim issuer DID, claims
        NewClaims(IdentityId, ClaimMetaData, Claim<Moment>),

        /// DID, claim issuer DID, claim
        RevokedClaim(IdentityId, ClaimMetaData),

        /// DID
        NewIssuer(IdentityId),

        /// DID queried
        DidQuery(Key, IdentityId),
    }
);

impl<T: Trait> Module<T> {
    /// Private and not sanitized function. It is designed to be used internally by
    /// others sanitezed functions.
    fn update_signing_item_permissions(
        target_did: IdentityId,
        signer: &Signer,
        mut permissions: Vec<Permission>,
    ) -> Result {
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
                rstd::mem::swap(&mut signing_item.permissions, &mut permissions);
                (*record).signing_items.retain(|si| si.signer != *signer);
                (*record).signing_items.push(signing_item.clone());
                new_s_item = Some(signing_item);
            }
        });

        if let Some(s) = new_s_item {
            Self::deposit_event(RawEvent::SigningPermissionsUpdated(
                target_did,
                s.clone(),
                permissions,
            ));
        }
        Ok(())
    }

    pub fn is_claim_issuer(did: IdentityId, issuer_did: IdentityId) -> bool {
        <ClaimIssuers>::get(did).contains(&issuer_did)
    }

    /// It checks if `key` is a signing key of `did` identity.
    /// # IMPORTANT
    /// If signing keys are frozen this function always returns false.
    /// Master key cannot be frozen.
    pub fn is_signer_authorized(did: IdentityId, signer: &Signer) -> bool {
        let record = <DidRecords>::get(did);

        // Check master id or key
        match signer {
            Signer::Key(ref signer_key) if record.master_key == *signer_key => true,
            Signer::Identity(ref signer_id) if did == *signer_id => true,
            _ => {
                // Check signing items if DID is not frozen.
                !Self::is_did_frozen(did)
                    && record
                        .signing_items
                        .iter()
                        .find(|&si| si.signer == *signer)
                        .is_some()
            }
        }
    }

    fn is_signer_authorized_with_permissions(
        did: IdentityId,
        signer: &Signer,
        permissions: Vec<Permission>,
    ) -> bool {
        let record = <DidRecords>::get(did);

        match signer {
            Signer::Key(ref signer_key) if record.master_key == *signer_key => true,
            Signer::Identity(ref signer_id) if did == *signer_id => true,
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
                // Signer is not part of signing items of `did`, or
                // Did is frozen.
                false
            }
        }
    }

    /// Use `did` as reference.
    pub fn is_master_key(did: IdentityId, key: &Key) -> bool {
        key == &<DidRecords>::get(did).master_key
    }

    pub fn fetch_claim_value(
        did: IdentityId,
        claim_key: Vec<u8>,
        claim_issuer: IdentityId,
    ) -> Option<ClaimValue> {
        let claim_meta_data = ClaimMetaData {
            claim_key: claim_key,
            claim_issuer: claim_issuer,
        };
        if <Claims<T>>::exists((did.clone(), claim_meta_data.clone())) {
            let now = <timestamp::Module<T>>::get();
            let claim = <Claims<T>>::get((did, claim_meta_data));
            if claim.expiry > now {
                return Some(claim.claim_value);
            }
        }
        return None;
    }

    pub fn fetch_claim_value_multiple_issuers(
        did: IdentityId,
        claim_key: Vec<u8>,
        claim_issuers: Vec<IdentityId>,
    ) -> Option<ClaimValue> {
        for claim_issuer in claim_issuers {
            let claim_value = Self::fetch_claim_value(did.clone(), claim_key.clone(), claim_issuer);
            if claim_value.is_some() {
                return claim_value;
            }
        }
        return None;
    }

    /// It checks that `sender_key` is the master key of `did` Identifier and that
    /// did exists.
    /// # Return
    /// A result object containing the `DidRecord` of `did`.
    pub fn grant_check_only_master_key(
        sender_key: &Key,
        did: IdentityId,
    ) -> rstd::result::Result<DidRecord, &'static str> {
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
    pub fn get_identity(key: &Key) -> Option<IdentityId> {
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
    fn set_frozen_signing_key_flags(origin: T::Origin, did: IdentityId, freeze: bool) -> Result {
        let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
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
    pub fn can_key_be_linked_to_did(key: &Key, signer_type: SignerType) -> bool {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(key) {
            match linked_key_info {
                LinkedKeyInfo::Unique(..) => false,
                LinkedKeyInfo::Group(..) => signer_type != SignerType::External,
            }
        } else {
            true
        }
    }

    /// It links `key` key to `did` identity as a `key_type` type.
    /// # Errors
    /// This function can be used if `can_key_be_linked_to_did` returns true. Otherwise, it will do
    /// nothing.
    fn link_key_to_did(key: &Key, key_type: SignerType, did: IdentityId) {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(key) {
            match linked_key_info {
                LinkedKeyInfo::Group(mut dids) => {
                    if !dids.contains(&did) && key_type != SignerType::External {
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
            // Key is not yet linked to any identity, so no constraints.
            let linked_key_info = match key_type {
                SignerType::External => LinkedKeyInfo::Unique(did),
                _ => LinkedKeyInfo::Group(vec![did]),
            };
            <KeyToIdentityIds>::insert(key, linked_key_info);
        }
    }

    /// It unlinks the `key` key from `did`.
    /// If there is no more associated identities, its full entry is removed.
    fn unlink_key_to_did(key: &Key, did: IdentityId) {
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
    pub(super) fn set_current_did(did_opt: Option<IdentityId>) {
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
    fn remove_pre_join_identity(signer: &Signer, id: IdentityId) {
        let mut is_pre_auth_list_empty = false;
        <PreAuthorizedJoinDid>::mutate(signer, |pre_auth_list| {
            pre_auth_list.retain(|pre_auth| pre_auth.target_id != id);
            is_pre_auth_list_empty = pre_auth_list.is_empty();
        });

        if is_pre_auth_list_empty {
            <PreAuthorizedJoinDid>::remove(signer);
        }
    }
}

pub trait IdentityTrait<T> {
    fn get_identity(key: &Key) -> Option<IdentityId>;
    fn is_signer_authorized(did: IdentityId, signer: &Signer) -> bool;
    fn is_signer_authorized_with_permissions(
        did: IdentityId,
        signer: &Signer,
        permissions: Vec<Permission>,
    ) -> bool;
    fn is_master_key(did: IdentityId, key: &Key) -> bool;
}

impl<T: Trait> IdentityTrait<T::Balance> for Module<T> {
    fn get_identity(key: &Key) -> Option<IdentityId> {
        Self::get_identity(&key)
    }

    fn is_signer_authorized(did: IdentityId, signer: &Signer) -> bool {
        Self::is_signer_authorized(did, signer)
    }

    fn is_master_key(did: IdentityId, key: &Key) -> bool {
        Self::is_master_key(did, &key)
    }

    fn is_signer_authorized_with_permissions(
        did: IdentityId,
        signer: &Signer,
        permissions: Vec<Permission>,
    ) -> bool {
        Self::is_signer_authorized_with_permissions(did, signer, permissions)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use primitives::SignerType;

    use sr_io::{with_externalities, TestExternalities};
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, ConvertInto, IdentityLookup},
        Perbill,
    };
    use srml_support::{
        assert_err, assert_ok,
        dispatch::{DispatchError, DispatchResult},
        impl_outer_origin, parameter_types,
    };
    use std::result::Result;
    use substrate_primitives::{Blake2Hasher, H256};

    impl_outer_origin! {
        pub enum Origin for IdentityTest {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct IdentityTest;

    parameter_types! {
        pub const BlockHashCount: u32 = 250;
        pub const MaximumBlockWeight: u32 = 4096;
        pub const MaximumBlockLength: u32 = 4096;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }

    impl system::Trait for IdentityTest {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();

        type Call = ();
        type WeightMultiplierUpdate = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }

    parameter_types! {
        pub const ExistentialDeposit: u64 = 0;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
        pub const TransactionBaseFee: u64 = 0;
        pub const TransactionByteFee: u64 = 0;
    }

    impl balances::Trait for IdentityTest {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type DustRemoval = ();
        type TransferPayment = ();

        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type TransactionBaseFee = TransactionBaseFee;
        type TransactionByteFee = TransactionByteFee;
        type WeightToFee = ConvertInto;
        type Identity = super::Module<IdentityTest>;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl timestamp::Trait for IdentityTest {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    #[derive(codec::Encode, codec::Decode, Debug, Clone, Eq, PartialEq)]
    pub struct IdentityProposal {
        pub dummy: u8,
    }

    impl sr_primitives::traits::Dispatchable for IdentityProposal {
        type Origin = Origin;
        type Trait = IdentityTest;
        type Error = DispatchError;

        fn dispatch(self, _origin: Self::Origin) -> DispatchResult<Self::Error> {
            Ok(())
        }
    }

    impl super::Trait for IdentityTest {
        type Event = ();
        type Proposal = IdentityProposal;
    }

    type Identity = super::Module<IdentityTest>;

    /// Create externalities
    fn build_ext() -> TestExternalities<Blake2Hasher> {
        system::GenesisConfig::default()
            .build_storage::<IdentityTest>()
            .unwrap()
            .into()
    }

    /// It creates an Account and registers its DID.
    fn make_account(
        id: &u64,
    ) -> Result<(<IdentityTest as system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(id.clone());
        Identity::register_did(signed_id.clone(), vec![])?;
        let did = Identity::get_identity(&Key::try_from(id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn only_claim_issuers_can_add_claims() {
        with_externalities(&mut build_ext(), || {
            let (_owner, owner_did) = make_account(&Identity::owner()).unwrap();
            let (issuer, issuer_did) = make_account(&2).unwrap();
            let (claim_issuer, claim_issuer_did) = make_account(&3).unwrap();

            let claim_value = ClaimValue {
                data_type: DataTypes::VecU8,
                value: "some_value".as_bytes().to_vec(),
            };

            assert_ok!(Identity::add_claim(
                claim_issuer.clone(),
                claim_issuer_did,
                "some_key".as_bytes().to_vec(),
                claim_issuer_did.clone(),
                100u64,
                claim_value.clone()
            ));

            assert_err!(
                Identity::add_claim(
                    claim_issuer.clone(),
                    owner_did.clone(),
                    "some_key".as_bytes().to_vec(),
                    issuer_did.clone(),
                    100u64,
                    claim_value.clone()
                ),
                "did_issuer must be a claim issuer or master key for DID"
            );
            assert_err!(
                Identity::add_claim(
                    issuer.clone(),
                    issuer_did.clone(),
                    "some_key".as_bytes().to_vec(),
                    claim_issuer_did.clone(),
                    100u64,
                    claim_value.clone()
                ),
                "Sender must hold a claim issuer\'s signing key"
            );
        });
    }

    /// TODO Add `Signer::Identity(..)` test.
    #[test]
    fn only_master_or_signing_keys_can_authenticate_as_an_identity() {
        with_externalities(&mut build_ext(), || {
            let owner_id = Identity::owner();
            let owner_signer = Signer::Key(Key::try_from(owner_id.encode()).unwrap());
            let (_owner, owner_did) = make_account(&owner_id).unwrap();
            let (a, a_did) = make_account(&2).unwrap();
            let (_b, b_did) = make_account(&3).unwrap();
            let charlie_acc = 4u64;
            let charlie_key = Key::try_from(charlie_acc.encode()).unwrap();
            let charlie_signer = Signer::Key(charlie_key);
            let charlie_signing_item =
                SigningItem::new(charlie_signer.clone(), vec![Permission::Admin]);

            assert_ok!(Identity::add_signing_items(
                a.clone(),
                a_did,
                vec![charlie_signing_item]
            ));
            assert_ok!(Identity::authorize_join_to_identity(
                Origin::signed(charlie_acc),
                a_did
            ));

            // Check master key on master and signing_keys.
            assert!(Identity::is_signer_authorized(owner_did, &owner_signer));
            assert!(Identity::is_signer_authorized(a_did, &charlie_signer));

            assert!(Identity::is_signer_authorized(b_did, &charlie_signer) == false);

            // ... and remove that key.
            assert_ok!(Identity::remove_signing_items(
                a.clone(),
                a_did.clone(),
                vec![charlie_signer.clone()]
            ));
            assert!(Identity::is_signer_authorized(a_did, &charlie_signer) == false);
        });
    }

    #[test]
    fn revoking_claims() {
        with_externalities(&mut build_ext(), || {
            let (owner, owner_did) = make_account(&Identity::owner()).unwrap();
            let (issuer, issuer_did) = make_account(&2).unwrap();
            let (claim_issuer, claim_issuer_did) = make_account(&3).unwrap();

            assert_ok!(Identity::add_claim_issuer(
                owner.clone(),
                owner_did,
                claim_issuer_did
            ));

            let claim_value = ClaimValue {
                data_type: DataTypes::VecU8,
                value: "some_value".as_bytes().to_vec(),
            };

            assert_ok!(Identity::add_claim(
                claim_issuer.clone(),
                claim_issuer_did,
                "some_key".as_bytes().to_vec(),
                claim_issuer_did,
                100u64,
                claim_value.clone()
            ));

            assert_err!(
                Identity::revoke_claim(
                    issuer.clone(),
                    issuer_did,
                    "some_key".as_bytes().to_vec(),
                    claim_issuer_did
                ),
                "Sender must hold a claim issuer\'s signing key"
            );

            assert_ok!(Identity::revoke_claim(
                claim_issuer.clone(),
                owner_did,
                "some_key".as_bytes().to_vec(),
                claim_issuer_did
            ));
        });
    }

    #[test]
    fn only_master_key_can_add_signing_key_permissions() {
        with_externalities(
            &mut build_ext(),
            &only_master_key_can_add_signing_key_permissions_with_externalities,
        );
    }

    fn only_master_key_can_add_signing_key_permissions_with_externalities() {
        let (alice_acc, bob_acc, charlie_acc) = (1u64, 2u64, 3u64);
        let (bob_key, charlie_key) = (
            Key::try_from(bob_acc.encode()).unwrap(),
            Key::try_from(charlie_acc.encode()).unwrap(),
        );
        let (alice, alice_did) = make_account(&alice_acc).unwrap();

        assert_ok!(Identity::add_signing_items(
            alice.clone(),
            alice_did,
            vec![SigningItem::from(bob_key), SigningItem::from(charlie_key)]
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(bob_acc),
            alice_did
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(charlie_acc),
            alice_did
        ));

        // Only `alice` is able to update `bob`'s permissions and `charlie`'s permissions.
        assert_ok!(Identity::set_permission_to_signer(
            alice.clone(),
            alice_did,
            Signer::Key(bob_key),
            vec![Permission::Operator]
        ));
        assert_ok!(Identity::set_permission_to_signer(
            alice.clone(),
            alice_did,
            Signer::Key(charlie_key),
            vec![Permission::Admin, Permission::Operator]
        ));

        // Bob tries to get better permission by himself at `alice` Identity.
        assert_err!(
            Identity::set_permission_to_signer(
                Origin::signed(bob_acc),
                alice_did,
                Signer::Key(bob_key),
                vec![Permission::Full]
            ),
            "Only master key of an identity is able to execute this operation"
        );

        // Bob tries to remove Charlie's permissions at `alice` Identity.
        assert_err!(
            Identity::set_permission_to_signer(
                Origin::signed(bob_acc),
                alice_did,
                Signer::Key(charlie_key),
                vec![]
            ),
            "Only master key of an identity is able to execute this operation"
        );

        // Alice over-write some permissions.
        assert_ok!(Identity::set_permission_to_signer(
            alice.clone(),
            alice_did,
            Signer::Key(bob_key),
            vec![]
        ));
    }

    #[test]
    fn add_signing_keys_with_specific_type() {
        with_externalities(
            &mut build_ext(),
            &add_signing_keys_with_specific_type_with_externalities,
        );
    }

    /// It tests that signing key can be added using non-default key type
    /// (`SignerType::External`).
    fn add_signing_keys_with_specific_type_with_externalities() {
        let (alice_acc, bob_acc, charlie_acc, dave_acc) = (1u64, 2u64, 3u64, 4u64);
        let (charlie_key, dave_key) = (
            Key::try_from(charlie_acc.encode()).unwrap(),
            Key::try_from(dave_acc.encode()).unwrap(),
        );

        // Create keys using non-default type.
        let charlie_signing_key = SigningItem {
            signer: Signer::Key(charlie_key),
            signer_type: SignerType::Relayer,
            permissions: vec![],
        };
        let dave_signing_key = SigningItem {
            signer: Signer::Key(dave_key),
            signer_type: SignerType::Multisig,
            permissions: vec![],
        };

        // Add signing keys with non-default type.
        let (alice, alice_did) = make_account(&alice_acc).unwrap();
        assert_ok!(Identity::add_signing_items(
            alice,
            alice_did,
            vec![charlie_signing_key, dave_signing_key.clone()]
        ));

        // Register did with non-default type.
        assert_ok!(Identity::register_did(
            Origin::signed(bob_acc),
            vec![dave_signing_key]
        ));
    }

    /// It verifies that frozen keys are recovered after `unfreeze` call.
    #[test]
    fn freeze_signing_keys_test() {
        with_externalities(&mut build_ext(), &freeze_signing_keys_with_externalities);
    }

    fn freeze_signing_keys_with_externalities() {
        let (alice_acc, bob_acc, charlie_acc, dave_acc) = (1u64, 2u64, 3u64, 4u64);
        let (bob_key, charlie_key, dave_key) = (
            Key::try_from(bob_acc.encode()).unwrap(),
            Key::try_from(charlie_acc.encode()).unwrap(),
            Key::try_from(dave_acc.encode()).unwrap(),
        );

        let bob_signing_key = SigningItem::new(Signer::Key(bob_key), vec![Permission::Admin]);
        let charlie_signing_key =
            SigningItem::new(Signer::Key(charlie_key), vec![Permission::Operator]);
        let dave_signing_key = SigningItem::from(dave_key);

        // Add signing keys.
        let (alice, alice_did) = make_account(&alice_acc).unwrap();
        let signing_keys_v1 = vec![bob_signing_key.clone(), charlie_signing_key];
        assert_ok!(Identity::add_signing_items(
            alice.clone(),
            alice_did,
            signing_keys_v1.clone()
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(bob_acc),
            alice_did
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(charlie_acc),
            alice_did
        ));

        assert_eq!(
            Identity::is_signer_authorized(alice_did, &Signer::Key(bob_key)),
            true
        );

        // Freeze signing keys: bob & charlie.
        assert_err!(
            Identity::freeze_signing_keys(Origin::signed(bob_acc), alice_did.clone()),
            "Only master key of an identity is able to execute this operation"
        );
        assert_ok!(Identity::freeze_signing_keys(
            alice.clone(),
            alice_did.clone()
        ));

        assert_eq!(
            Identity::is_signer_authorized(alice_did, &Signer::Key(bob_key)),
            false
        );

        // Add new signing keys.
        let signing_keys_v2 = vec![dave_signing_key.clone()];
        assert_ok!(Identity::add_signing_items(
            alice.clone(),
            alice_did.clone(),
            signing_keys_v2.clone()
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(dave_acc),
            alice_did
        ));
        assert_eq!(
            Identity::is_signer_authorized(alice_did, &Signer::Key(dave_key)),
            false
        );

        // update permission of frozen keys.
        assert_ok!(Identity::set_permission_to_signer(
            alice.clone(),
            alice_did.clone(),
            Signer::Key(bob_key),
            vec![Permission::Operator]
        ));

        // unfreeze all
        assert_err!(
            Identity::unfreeze_signing_keys(Origin::signed(bob_acc), alice_did.clone()),
            "Only master key of an identity is able to execute this operation"
        );
        assert_ok!(Identity::unfreeze_signing_keys(
            alice.clone(),
            alice_did.clone()
        ));

        assert_eq!(
            Identity::is_signer_authorized(alice_did, &Signer::Key(dave_key)),
            true
        );
    }

    /// It double-checks that frozen keys are removed too.
    #[test]
    fn remove_frozen_signing_keys_test() {
        with_externalities(
            &mut build_ext(),
            &remove_frozen_signing_keys_with_externalities,
        );
    }

    fn remove_frozen_signing_keys_with_externalities() {
        let (alice_acc, bob_acc, charlie_acc) = (1u64, 2u64, 3u64);
        let (bob_key, charlie_key) = (
            Key::try_from(bob_acc.encode()).unwrap(),
            Key::try_from(charlie_acc.encode()).unwrap(),
        );

        let bob_signing_key = SigningItem::new(Signer::Key(bob_key), vec![Permission::Admin]);
        let charlie_signing_key =
            SigningItem::new(Signer::Key(charlie_key), vec![Permission::Operator]);

        // Add signing keys.
        let (alice, alice_did) = make_account(&alice_acc).unwrap();
        let signing_keys_v1 = vec![bob_signing_key, charlie_signing_key.clone()];
        assert_ok!(Identity::add_signing_items(
            alice.clone(),
            alice_did,
            signing_keys_v1.clone()
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(bob_acc),
            alice_did
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(charlie_acc),
            alice_did
        ));

        // Freeze all signing keys
        assert_ok!(Identity::freeze_signing_keys(alice.clone(), alice_did));

        // Remove Bob's key.
        assert_ok!(Identity::remove_signing_items(
            alice.clone(),
            alice_did,
            vec![Signer::Key(bob_key)]
        ));
        // Check DidRecord.
        let did_rec = Identity::did_records(alice_did);
        assert_eq!(did_rec.signing_items, vec![charlie_signing_key]);
    }

    #[test]
    fn add_claim_issuer_tests() {
        with_externalities(&mut build_ext(), &add_claim_issuer_tests_with_externalities);
    }

    fn add_claim_issuer_tests_with_externalities() {
        // Register identities
        let (alice_acc, bob_acc, charlie_acc) = (1u64, 2u64, 3u64);
        let (alice, alice_did) = make_account(&alice_acc).unwrap();
        let (_bob, bob_did) = make_account(&bob_acc).unwrap();

        // Check `add_claim_issuer` constraints.
        assert_ok!(Identity::add_claim_issuer(
            alice.clone(),
            alice_did.clone(),
            bob_did.clone()
        ));
        assert_err!(
            Identity::add_claim_issuer(
                Origin::signed(charlie_acc),
                alice_did.clone(),
                bob_did.clone()
            ),
            "Only master key of an identity is able to execute this operation"
        );
        assert_err!(
            Identity::add_claim_issuer(alice, alice_did.clone(), alice_did),
            "Master key cannot add itself as claim issuer"
        );
    }

    #[test]
    fn enforce_uniqueness_keys_in_identity_tests() {
        with_externalities(&mut build_ext(), &enforce_uniqueness_keys_in_identity);
    }

    fn enforce_uniqueness_keys_in_identity() {
        let unique_error = "One signing key can only belong to one DID";
        // Register identities
        let (a_acc, b_acc, c_acc, d_acc) = (1u64, 2u64, 3u64, 4u64);
        let (alice, alice_id) = make_account(&a_acc).unwrap();
        let (bob, bob_id) = make_account(&b_acc).unwrap();

        // Check external signed key uniqueness.
        let charlie_key = Key::try_from(c_acc.encode()).unwrap();
        let charlie_sk = SigningItem::new(Signer::Key(charlie_key), vec![Permission::Operator]);
        assert_ok!(Identity::add_signing_items(
            alice.clone(),
            alice_id,
            vec![charlie_sk.clone()]
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(c_acc),
            alice_id
        ));

        assert_err!(
            Identity::add_signing_items(bob.clone(), bob_id, vec![charlie_sk]),
            unique_error
        );

        // Check non-external signed key non-uniqueness.
        let dave_key = Key::try_from(d_acc.encode()).unwrap();
        let dave_sk = SigningItem {
            signer: Signer::Key(dave_key),
            signer_type: SignerType::Multisig,
            permissions: vec![Permission::Operator],
        };
        assert_ok!(Identity::add_signing_items(
            alice.clone(),
            alice_id,
            vec![dave_sk.clone()]
        ));
        assert_ok!(Identity::add_signing_items(
            bob.clone(),
            bob_id,
            vec![dave_sk]
        ));

        // Check that master key acts like external signed key.
        let bob_key = Key::try_from(b_acc.encode()).unwrap();
        let bob_sk_as_mutisig = SigningItem {
            signer: Signer::Key(bob_key),
            signer_type: SignerType::Multisig,
            permissions: vec![Permission::Operator],
        };
        assert_err!(
            Identity::add_signing_items(alice.clone(), alice_id, vec![bob_sk_as_mutisig]),
            unique_error
        );

        let bob_sk = SigningItem::new(Signer::Key(bob_key), vec![Permission::Admin]);
        assert_err!(
            Identity::add_signing_items(alice.clone(), alice_id, vec![bob_sk]),
            unique_error
        );
    }

    #[test]
    fn add_remove_signing_identities() {
        with_externalities(
            &mut build_ext(),
            &add_remove_signing_identities_with_externalities,
        );
    }

    fn add_remove_signing_identities_with_externalities() {
        let (a_acc, b_acc, c_acc, d_acc) = (1u64, 2u64, 3u64, 4u64);
        let (alice, alice_id) = make_account(&a_acc).unwrap();
        let (_bob, bob_id) = make_account(&b_acc).unwrap();
        let (_charlie, charlie_id) = make_account(&c_acc).unwrap();
        let (_dave, dave_id) = make_account(&d_acc).unwrap();

        assert_ok!(Identity::add_signing_items(
            alice.clone(),
            alice_id,
            vec![SigningItem::from(bob_id), SigningItem::from(charlie_id)]
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(b_acc),
            alice_id
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(c_acc),
            alice_id
        ));
        assert_eq!(
            Identity::is_signer_authorized(alice_id, &Signer::Identity(bob_id)),
            true
        );

        assert_ok!(Identity::remove_signing_items(
            alice.clone(),
            alice_id,
            vec![Signer::Identity(bob_id), Signer::Identity(dave_id)]
        ));

        let alice_rec = Identity::did_records(alice_id);
        assert_eq!(alice_rec.signing_items, vec![SigningItem::from(charlie_id)]);

        // Check is_authorized_identity
        assert_eq!(
            Identity::is_signer_authorized(alice_id, &Signer::Identity(charlie_id)),
            true
        );
        assert_eq!(
            Identity::is_signer_authorized(alice_id, &Signer::Identity(bob_id)),
            false
        );
    }

    #[test]
    fn two_step_join_id() {
        with_externalities(&mut build_ext(), &two_step_join_id_with_ext);
    }

    fn two_step_join_id_with_ext() {
        let (a_acc, b_acc, c_acc, d_acc, e_acc) = (1u64, 2u64, 3u64, 4u64, 5u64);
        let (a, a_id) = make_account(&a_acc).unwrap();
        let (b, b_id) = make_account(&b_acc).unwrap();

        let c_sk = SigningItem::new(
            Signer::Key(Key::try_from(c_acc.encode()).unwrap()),
            vec![Permission::Operator],
        );
        let d_sk = SigningItem::new(
            Signer::Key(Key::try_from(d_acc.encode()).unwrap()),
            vec![Permission::Full],
        );
        let e_sk = SigningItem::new(
            Signer::Key(Key::try_from(e_acc.encode()).unwrap()),
            vec![Permission::Full],
        );

        // Check 1-to-1 relation between key and identity.
        let signing_keys = vec![c_sk.clone(), d_sk.clone(), e_sk.clone()];
        assert_ok!(Identity::add_signing_items(
            a.clone(),
            a_id,
            signing_keys.clone()
        ));
        assert_ok!(Identity::add_signing_items(b, b_id, signing_keys));
        assert_eq!(Identity::is_signer_authorized(a_id, &c_sk.signer), false);

        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(c_acc),
            a_id
        ));
        assert_eq!(Identity::is_signer_authorized(a_id, &c_sk.signer), true);

        assert_err!(
            Identity::authorize_join_to_identity(Origin::signed(c_acc), b_id),
            "Key is already linked to an identity"
        );
        assert_eq!(Identity::is_signer_authorized(b_id, &c_sk.signer), false);

        // Check after remove a signing key.
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(d_acc),
            a_id
        ));
        assert_eq!(Identity::is_signer_authorized(a_id, &d_sk.signer), true);
        assert_ok!(Identity::remove_signing_items(
            a.clone(),
            a_id,
            vec![d_sk.signer.clone()]
        ));
        assert_eq!(Identity::is_signer_authorized(a_id, &d_sk.signer), false);

        // Check remove pre-authorization from master and itself.
        assert_err!(
            Identity::unauthorized_join_to_identity(a.clone(), e_sk.signer.clone(), b_id),
            "Account cannot remove this authorization"
        );
        assert_ok!(Identity::unauthorized_join_to_identity(
            a,
            e_sk.signer.clone(),
            a_id
        ));
        assert_ok!(Identity::unauthorized_join_to_identity(
            Origin::signed(e_acc),
            e_sk.signer,
            b_id
        ));
    }
}
