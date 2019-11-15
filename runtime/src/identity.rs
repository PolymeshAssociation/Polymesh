use rstd::{convert::TryFrom, prelude::*};

use crate::balances;

use primitives::{DidRecord, IdentityId, Key, KeyRole, KeyType, SigningKey};

use codec::Encode;
use sr_primitives::traits::{CheckedAdd, CheckedSub};
use srml_support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, WithdrawReason},
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
/// Master key or extenal type signing key are restricted to be linked to just one identity.
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
}

decl_storage! {
    trait Store for Module<T: Trait> as identity {

        Owner get(owner) config(): T::AccountId;

        /// DID -> identity info
        pub DidRecords get(did_records): map IdentityId => DidRecord<T::Balance>;

        /// DID -> bool that indicates if signing keys are frozen.
        pub IsDidFrozen get(is_did_frozen): map IdentityId => bool;

        /// DID -> DID claim issuers
        pub ClaimIssuers get(claim_issuers): map IdentityId => Vec<IdentityId>;

        /// (DID, claim_key, claim_issuer) -> Associated claims
        pub Claims get(claims): map(IdentityId, ClaimMetaData) => Claim<T::Moment>;

        /// DID -> array of (claim_key and claim_issuer)
        pub ClaimKeys get(claim_keys): map IdentityId => Vec<ClaimMetaData>;

        // Account => DID
        pub KeyToIdentityIds get(key_to_identity_ids): map Key => Option<LinkedKeyInfo>;

        // Signing key => Charge Fee to did?. Default is false i.e. the fee will be charged from user balance
        pub ChargeDid get(charge_did): map Key => bool;

        /// How much does creating a DID cost
        pub DidCreationFee get(did_creation_fee) config(): T::Balance;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        fn set_charge_did(origin, charge_did: bool) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from( sender.encode())?;
            <ChargeDid>::insert(sender_key, charge_did);
            Ok(())
        }

        /// Register signing keys for a new DID. Uses origin key as the master key
        pub fn register_did(origin, did: IdentityId, signing_keys: Vec<SigningKey>) -> Result {
            let sender = ensure_signed(origin)?;
            let master_key = Key::try_from( sender.encode())?;
            ensure!( Self::can_key_be_linked_to_did( &master_key, KeyType::External),
                "Master key already belong to one DID");

            // Make sure there's no pre-existing entry for the DID
            ensure!(!<DidRecords<T>>::exists(did), "DID must be unique");

            // TODO: Subtract the fee
            let _imbalance = <balances::Module<T> as Currency<_>>::withdraw(
                &sender,
                Self::did_creation_fee(),
                WithdrawReason::Fee,
                ExistenceRequirement::KeepAlive
                )?;

            for sig_key in &signing_keys {
                if !Self::can_key_be_linked_to_did( &sig_key.key, sig_key.key_type){
                    return Err("One signing key can only belong to one DID");
                }
            }

            // Link  master key and signing keys.
            Self::link_key_to_did( &master_key, KeyType::External, did);
            for sig_key in &signing_keys {
                Self::link_key_to_did( &sig_key.key, sig_key.key_type, did);
            }

            let record = DidRecord {
                signing_keys: signing_keys.clone(),
                master_key,
                ..Default::default()
            };

            <DidRecords<T>>::insert(did, record);

            Self::deposit_event(RawEvent::NewDid(did, sender, signing_keys));
            Ok(())
        }

        /// Adds new signing keys for a DID. Only called by master key owner.
        pub fn add_signing_keys(origin, did: IdentityId, additional_keys: Vec<SigningKey>) -> Result {
            let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            for skey in &additional_keys {
                if !Self::can_key_be_linked_to_did( &skey.key, skey.key_type) {
                    return Err( "One signing key can only belong to one DID");
                }
            }

            additional_keys.iter()
                .for_each( |skey| Self::link_key_to_did( &skey.key, skey.key_type, did));

            <DidRecords<T>>::mutate( did,
            |record| {
                // Concatenate new keys while making sure the key set is
                // unique
                let mut new_roled_keys = additional_keys.iter()
                    .filter( |&add_key| {
                        record.signing_keys.iter()
                        .find( |&rk| rk == add_key)
                        .is_none()
                    })
                    .cloned()
                    .collect::<Vec<_>>();

                (*record).signing_keys.append( &mut new_roled_keys);
            });

            Self::deposit_event(RawEvent::SigningKeysAdded(did, additional_keys));

            Ok(())
        }

        /// Removes specified signing keys of a DID if present. Only called by master key owner.
        fn remove_signing_keys(origin, did: IdentityId, keys_to_remove: Vec<Key>) -> Result {
            let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            // Check that key is linked to that DID.
            for key in &keys_to_remove {
                if let Some(linked_key_info) = <KeyToIdentityIds>::get(key) {
                    let error_msg = "Signing key does not belong to this DID";

                    match linked_key_info {
                        LinkedKeyInfo::Unique(link_did) => if did != link_did {
                            return Err(error_msg);
                        },
                        LinkedKeyInfo::Group(link_dids) => if link_dids.into_iter().find( |id| did == *id).is_none() {
                            return Err(error_msg);
                        }
                    }
                }
            }

            // Remove links between keys and DID
            keys_to_remove.iter().for_each( |key| Self::unlink_key_to_did(key, did));

            // Remove signing keys from DID records.
            <DidRecords<T>>::mutate(did,
            |record| {
                (*record).signing_keys.retain( |skey| keys_to_remove.iter()
                        .find(|&rk| skey == rk)
                        .is_none());
            });

            Self::deposit_event(RawEvent::SigningKeysRemoved(did, keys_to_remove));
            Ok(())
        }

        /// Sets a new master key for a DID. Only called by master key owner.
        fn set_master_key(origin, did: IdentityId, new_key: Key) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from( sender.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, did)?;

            ensure!( Self::can_key_be_linked_to_did(&new_key, KeyType::External), "Master key can only belong to one DID");

            <DidRecords<T>>::mutate(did,
            |record| {
                (*record).master_key = new_key.clone();
            });

            Self::deposit_event(RawEvent::NewMasterKey(did, sender, new_key));
            Ok(())
        }

        /// Adds funds to a DID.
        pub fn fund_poly(origin, did: IdentityId, amount: <T as balances::Trait>::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(did), "DID must already exist");

            let record = <DidRecords<T>>::get(did);

            // We must know that new balance is valid without creating side effects
            let new_record_balance = record.balance.checked_add(&amount).ok_or("overflow occured when increasing DID balance")?;

            let _imbalance = <balances::Module<T> as Currency<_>>::withdraw(
                &sender,
                amount,
                WithdrawReason::Fee,
                ExistenceRequirement::KeepAlive
                )?;

            <DidRecords<T>>::mutate(did, |record| {
                (*record).balance = new_record_balance;
            });

            Self::deposit_event(RawEvent::PolyDepositedInDid(did, sender, amount));

            Ok(())
        }

        /// Withdraws funds from a DID. Only called by master key owner.
        fn withdrawy_poly(origin, did: IdentityId, amount: <T as balances::Trait>::Balance) -> Result {
            let sender = ensure_signed(origin)?;
            let record = Self::grant_check_only_master_key( &Key::try_from( sender.encode())?, did)?;

            // We must know that new balance is valid without creating side effects
            let new_record_balance = record.balance.checked_sub(&amount).ok_or("underflow occured when decreasing DID balance")?;

            let _imbalance = <balances::Module<T> as Currency<_>>::deposit_into_existing(&sender, amount)?;

            <DidRecords<T>>::mutate(did, |record| {
                (*record).balance = new_record_balance;
            });

            Self::deposit_event(RawEvent::PolyWithdrawnFromDid(did, sender, amount));

            Ok(())
        }

        /// Transfers funds between DIDs.
        fn transfer_poly(origin, did: IdentityId, to_did: IdentityId, amount: <T as balances::Trait>::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(Self::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let from_record = <DidRecords<T>>::get(did);
            let to_record = <DidRecords<T>>::get(to_did);

            // Same for `from`
            let new_from_balance = from_record.balance.checked_sub(&amount).ok_or("Sender must have sufficient funds")?;

            // Compute new `to_did` balance and check that beneficiary's balance can be increased
            let new_to_balance = to_record.balance.checked_add(&amount).ok_or("Failed to increase to_did balance")?;

            // Alter from record
            <DidRecords<T>>::mutate(did, |record| {
                record.balance = new_from_balance;
            });

            // Alter to record
            <DidRecords<T>>::mutate(to_did, |record| {
                record.balance = new_to_balance;
            });

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

            ensure!(<DidRecords<T>>::exists(did_issuer), "claim issuer DID must already exist");

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

        /// Adds new claim record or edits an exisitng one. Only called by did_issuer's signing key
        pub fn add_claim(
            origin,
            did: IdentityId,
            claim_key: Vec<u8>,
            did_issuer: IdentityId,
            expiry: <T as timestamp::Trait>::Moment,
            claim_value: ClaimValue
        ) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(did), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(did_issuer), "claim issuer DID must already exist");

            let sender_key = Key::try_from( sender.encode())?;
            ensure!(Self::is_claim_issuer(did, did_issuer) || Self::is_master_key(did, &sender_key), "did_issuer must be a claim issuer or master key for DID");

            // Verify that sender key is one of did_issuer's signing keys
            ensure!(Self::is_authorized_key(did_issuer, &sender_key), "Sender must hold a claim issuer's signing key");

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

        /// Marks the specified claim as revoked
        pub fn revoke_claim(origin, did: IdentityId, claim_key: Vec<u8>, did_issuer: IdentityId) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(&did), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(&did_issuer), "claim issuer DID must already exist");

            // Verify that sender key is one of did_issuer's signing keys
            let sender_key = Key::try_from( sender.encode())?;
            ensure!(Self::is_authorized_key(did_issuer, &sender_key), "Sender must hold a claim issuer's signing key");

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

        /// It sets roles for an specific `target_key` key.
        /// Only the master key of an identity is able to set signing key roles.
        fn set_role_to_signing_key(origin, did: IdentityId, target_key: Key, roles: Vec<KeyRole>) -> Result {
            let sender_key = Key::try_from( ensure_signed(origin)?.encode())?;
            let record = Self::grant_check_only_master_key( &sender_key, did)?;

            // You are trying to add a role to did's master key. It is not needed.
            if record.master_key == target_key {
                return Ok(());
            }

            // Find key in `DidRecord::signing_keys` or in `DidRecord::frozen_signing_keys`.
            if let Some(ref _signing_key) = record.signing_keys.iter().find(|&sk| sk == &target_key) {
                Self::update_signing_key_roles(did, &target_key, roles)
            } else {
                Err( "Sender is not part of did's signing keys")
            }
        }

        fn freeze_signing_keys(origin, did: IdentityId) -> Result {
            Self::set_frozen_signing_key_flags( origin, did, true)
        }

        fn unfreeze_signing_keys(origin, did: IdentityId) -> Result {
            Self::set_frozen_signing_key_flags( origin, did, false)
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as balances::Trait>::Balance,
        Moment = <T as timestamp::Trait>::Moment,
    {
        /// DID, master key account ID, signing keys
        NewDid(IdentityId, AccountId, Vec<SigningKey>),

        /// DID, new keys
        SigningKeysAdded(IdentityId, Vec<SigningKey>),

        /// DID, the keys that got removed
        SigningKeysRemoved(IdentityId, Vec<Key>),

        /// DID, updated signing key, previous roles
        SigningKeyRolesUpdated(IdentityId, SigningKey, Vec<KeyRole>),

        /// DID, old master key account ID, new key
        NewMasterKey(IdentityId, AccountId, Key),

        /// beneficiary DID, sender, amount
        PolyDepositedInDid(IdentityId, AccountId, Balance),

        /// DID, beneficiary, amount
        PolyWithdrawnFromDid(IdentityId, AccountId, Balance),

        /// DID, amount
        PolyChargedFromDid(IdentityId, Balance),

        /// DID from, DID to, amount
        PolyTransfer(IdentityId, IdentityId, Balance),

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
    }
);

impl<T: Trait> Module<T> {
    /// Private and not sanitized function. It is designed to be used internally by
    /// others sanitezed functions.
    fn update_signing_key_roles(
        target_did: IdentityId,
        key: &Key,
        mut roles: Vec<KeyRole>,
    ) -> Result {
        // Remove duplicates.
        roles.sort();
        roles.dedup();

        let mut new_sk: Option<SigningKey> = None;

        <DidRecords<T>>::mutate(target_did, |record| {
            if let Some(mut sk) = (*record).signing_keys.iter().find(|sk| *sk == key).cloned() {
                rstd::mem::swap(&mut sk.roles, &mut roles);
                (*record).signing_keys.retain(|sk| sk != key);
                (*record).signing_keys.push(sk.clone());
                new_sk = Some(sk);
            }
        });

        Self::deposit_event(RawEvent::SigningKeyRolesUpdated(
            target_did,
            new_sk.unwrap_or_else(|| SigningKey::default()),
            roles,
        ));
        Ok(())
    }

    pub fn is_claim_issuer(did: IdentityId, issuer_did: IdentityId) -> bool {
        <ClaimIssuers>::get(did).contains(&issuer_did)
    }

    /// It checks if `key` is a signing key of `did` identity.
    /// # IMPORTANT
    /// If signing keys are frozen this function always returns false.
    /// Master key cannot be frozen.
    pub fn is_authorized_key(did: IdentityId, key: &Key) -> bool {
        let record = <DidRecords<T>>::get(did);
        if record.master_key == *key {
            return true;
        }

        if !Self::is_did_frozen(did) {
            return record.signing_keys.iter().find(|&rk| rk == key).is_some();
        }

        return false;
    }

    /// Use `did` as reference.
    pub fn is_master_key(did: IdentityId, key: &Key) -> bool {
        key == &<DidRecords<T>>::get(did).master_key
    }

    /// Withdraws funds from a DID balance
    pub fn charge_poly(id: IdentityId, amount: T::Balance) -> bool {
        if !<DidRecords<T>>::exists(&id) {
            return false;
        }

        let record = <DidRecords<T>>::get(&id);

        if record.balance < amount {
            return false;
        }

        <DidRecords<T>>::mutate(&id, |record| {
            (*record).balance = record.balance - amount;
        });

        Self::deposit_event(RawEvent::PolyChargedFromDid(id, amount));

        return true;
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
    ) -> rstd::result::Result<DidRecord<<T as balances::Trait>::Balance>, &'static str> {
        ensure!(<DidRecords<T>>::exists(did), "DID does not exist");
        let record = <DidRecords<T>>::get(did);
        ensure!(
            *sender_key == record.master_key,
            "Only master key of an identity is able to execute this operation"
        );

        Ok(record)
    }

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
    pub fn can_key_be_linked_to_did(key: &Key, key_type: KeyType) -> bool {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(key) {
            match linked_key_info {
                LinkedKeyInfo::Unique(..) => false,
                LinkedKeyInfo::Group(..) => key_type != KeyType::External,
            }
        } else {
            true
        }
    }

    /// It links `key` key to `did` identity as a `key_type` type.
    /// # Errors
    /// This function can be used if `can_key_be_linked_to_did` returns true. Otherwise, it will do
    /// nothing.
    fn link_key_to_did(key: &Key, key_type: KeyType, did: IdentityId) {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(key) {
            match linked_key_info {
                LinkedKeyInfo::Group(mut dids) => {
                    if !dids.contains(&did) && key_type != KeyType::External {
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
                KeyType::External => LinkedKeyInfo::Unique(did),
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
}

pub trait IdentityTrait<T> {
    fn signing_key_charge_did(signing_key: &Key) -> bool;
    fn charge_poly(signing_key: &Key, amount: T) -> bool;
}

impl<T: Trait> IdentityTrait<T::Balance> for Module<T> {
    /// Only signing keys with one-to-one relation with Identity are allowed to charge poly.
    fn charge_poly(signing_key: &Key, amount: T::Balance) -> bool {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(signing_key) {
            if let LinkedKeyInfo::Unique(linked_id) = linked_key_info {
                return Self::charge_poly(linked_id, amount);
            }
        }

        false
    }

    fn signing_key_charge_did(signing_key: &Key) -> bool {
        if let Some(linked_key_info) = <KeyToIdentityIds>::get(signing_key) {
            if let LinkedKeyInfo::Unique(identity) = linked_key_info {
                if Self::is_authorized_key(identity, signing_key)
                    && <ChargeDid>::exists(signing_key)
                {
                    return <ChargeDid>::get(signing_key);
                }
            }
        }

        false
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use primitives::KeyType;

    use sr_io::{with_externalities, TestExternalities};
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, ConvertInto, IdentityLookup},
        Perbill,
    };
    use srml_support::{assert_err, assert_ok, impl_outer_origin, parameter_types};
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

    impl super::Trait for IdentityTest {
        type Event = ();
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
        id: u64,
    ) -> Result<(<IdentityTest as system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(id);
        let did = IdentityId::from(id as u128);

        Identity::register_did(signed_id.clone(), did, vec![])?;
        Ok((signed_id, did))
    }

    #[test]
    fn dids_are_unique() {
        with_externalities(&mut build_ext(), || {
            let did_1 = IdentityId::from(1);
            let did_2 = IdentityId::from(2);

            assert_ok!(Identity::register_did(Origin::signed(1), did_1, vec![]));

            assert_ok!(Identity::register_did(Origin::signed(2), did_2, vec![]));

            assert_err!(
                Identity::register_did(Origin::signed(3), did_1, vec![]),
                "DID must be unique"
            );
        });
    }

    #[test]
    fn only_claim_issuers_can_add_claims() {
        with_externalities(&mut build_ext(), || {
            let (_owner, owner_did) = make_account(Identity::owner()).unwrap();
            let (issuer, issuer_did) = make_account(2).unwrap();
            let (claim_issuer, claim_issuer_did) = make_account(3).unwrap();

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

    #[test]
    fn only_master_or_signing_keys_can_authenticate_as_an_identity() {
        with_externalities(&mut build_ext(), || {
            let owner_id = Identity::owner();
            let owner_key = Key::try_from(owner_id.encode()).unwrap();
            let (_owner, owner_did) = make_account(owner_id).unwrap();
            let (a, a_did) = make_account(2).unwrap();
            let (_b, b_did) = make_account(3).unwrap();
            let charlie_sig_key =
                SigningKey::new(Key::try_from(4u64.encode()).unwrap(), vec![KeyRole::Admin]);

            assert_ok!(Identity::add_signing_keys(
                a.clone(),
                a_did,
                vec![charlie_sig_key.clone()]
            ));

            // Check master key on master and signing_keys.
            assert!(Identity::is_authorized_key(owner_did, &owner_key));
            assert!(Identity::is_authorized_key(a_did, &charlie_sig_key.key));

            assert!(Identity::is_authorized_key(b_did, &charlie_sig_key.key) == false);

            // ... and remove that key.
            assert_ok!(Identity::remove_signing_keys(
                a.clone(),
                a_did.clone(),
                vec![charlie_sig_key.key.clone()]
            ));
            assert!(Identity::is_authorized_key(a_did, &charlie_sig_key.key) == false);
        });
    }

    #[test]
    fn revoking_claims() {
        with_externalities(&mut build_ext(), || {
            let (owner, owner_did) = make_account(Identity::owner()).unwrap();
            let (issuer, issuer_did) = make_account(2).unwrap();
            let (claim_issuer, claim_issuer_did) = make_account(3).unwrap();

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
    fn only_master_key_can_add_signing_key_roles() {
        with_externalities(
            &mut build_ext(),
            &only_master_key_can_add_signing_key_roles_with_externalities,
        );
    }

    fn only_master_key_can_add_signing_key_roles_with_externalities() {
        let (alice_acc, bob_acc, charlie_acc) = (1u64, 2u64, 3u64);
        let (bob_key, charlie_key) = (
            Key::try_from(bob_acc.encode()).unwrap(),
            Key::try_from(charlie_acc.encode()).unwrap(),
        );
        let (alice, alice_did) = make_account(alice_acc).unwrap();

        assert_ok!(Identity::add_signing_keys(
            alice.clone(),
            alice_did,
            vec![
                SigningKey::from(bob_key.clone()),
                SigningKey::from(charlie_key.clone())
            ]
        ));

        // Only `alice` is able to update `bob`'s roles and `charlie`'s roles.
        assert_ok!(Identity::set_role_to_signing_key(
            alice.clone(),
            alice_did,
            bob_key.clone(),
            vec![KeyRole::Operator]
        ));
        assert_ok!(Identity::set_role_to_signing_key(
            alice.clone(),
            alice_did,
            charlie_key.clone(),
            vec![KeyRole::Admin, KeyRole::Operator]
        ));

        // Bob tries to get better role by himself at `alice` Identity.
        assert_err!(
            Identity::set_role_to_signing_key(
                Origin::signed(bob_acc),
                alice_did,
                bob_key.clone(),
                vec![KeyRole::Full]
            ),
            "Only master key of an identity is able to execute this operation"
        );

        // Bob tries to remove Charlie's roles at `alice` Identity.
        assert_err!(
            Identity::set_role_to_signing_key(
                Origin::signed(bob_acc),
                alice_did,
                charlie_key,
                vec![]
            ),
            "Only master key of an identity is able to execute this operation"
        );

        // Alice over-write some roles.
        assert_ok!(Identity::set_role_to_signing_key(
            alice.clone(),
            alice_did,
            bob_key,
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
    /// (`KeyType::External`).
    fn add_signing_keys_with_specific_type_with_externalities() {
        let (alice_acc, bob_acc, charlie_acc, dave_acc) = (1u64, 2u64, 3u64, 4u64);
        let (charlie_key, dave_key) = (
            Key::try_from(charlie_acc.encode()).unwrap(),
            Key::try_from(dave_acc.encode()).unwrap(),
        );

        // Create keys using non-default type.
        let charlie_signing_key = SigningKey {
            key: charlie_key,
            key_type: KeyType::Relayer,
            roles: vec![],
        };
        let dave_signing_key = SigningKey {
            key: dave_key,
            key_type: KeyType::Multisig,
            roles: vec![],
        };

        // Add signing keys with non-default type.
        let (alice, alice_did) = make_account(alice_acc).unwrap();
        assert_ok!(Identity::add_signing_keys(
            alice,
            alice_did,
            vec![charlie_signing_key, dave_signing_key.clone()]
        ));

        // Register did with non-default type.
        let bob_did = IdentityId::from(bob_acc as u128);
        assert_ok!(Identity::register_did(
            Origin::signed(bob_acc),
            bob_did,
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

        let bob_signing_key = SigningKey::new(bob_key.clone(), vec![KeyRole::Admin]);
        let charlie_signing_key = SigningKey::new(charlie_key, vec![KeyRole::Operator]);
        let dave_signing_key = SigningKey::new(dave_key.clone(), vec![]);

        // Add signing keys.
        let (alice, alice_did) = make_account(alice_acc).unwrap();
        let signing_keys_v1 = vec![bob_signing_key.clone(), charlie_signing_key];
        assert_ok!(Identity::add_signing_keys(
            alice.clone(),
            alice_did.clone(),
            signing_keys_v1.clone()
        ));

        assert_eq!(Identity::is_authorized_key(alice_did, &bob_key), true);

        // Freeze signing keys: bob & charlie.
        assert_err!(
            Identity::freeze_signing_keys(Origin::signed(bob_acc), alice_did.clone()),
            "Only master key of an identity is able to execute this operation"
        );
        assert_ok!(Identity::freeze_signing_keys(
            alice.clone(),
            alice_did.clone()
        ));

        assert_eq!(Identity::is_authorized_key(alice_did, &bob_key), false);

        // Add new signing keys.
        let signing_keys_v2 = vec![dave_signing_key.clone()];
        assert_ok!(Identity::add_signing_keys(
            alice.clone(),
            alice_did.clone(),
            signing_keys_v2.clone()
        ));
        assert_eq!(Identity::is_authorized_key(alice_did, &dave_key), false);

        // update role of frozen keys.
        assert_ok!(Identity::set_role_to_signing_key(
            alice.clone(),
            alice_did.clone(),
            bob_key.clone(),
            vec![KeyRole::Operator]
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

        assert_eq!(Identity::is_authorized_key(alice_did, &dave_key), true);
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

        let bob_signing_key = SigningKey::new(bob_key.clone(), vec![KeyRole::Admin]);
        let charlie_signing_key = SigningKey::new(charlie_key, vec![KeyRole::Operator]);

        // Add signing keys.
        let (alice, alice_did) = make_account(alice_acc).unwrap();
        let signing_keys_v1 = vec![bob_signing_key, charlie_signing_key.clone()];
        assert_ok!(Identity::add_signing_keys(
            alice.clone(),
            alice_did,
            signing_keys_v1.clone()
        ));

        // Freeze all signing keys
        assert_ok!(Identity::freeze_signing_keys(alice.clone(), alice_did));

        // Remove Bob's key.
        assert_ok!(Identity::remove_signing_keys(
            alice.clone(),
            alice_did,
            vec![bob_key.clone()]
        ));
        // Check DidRecord.
        let did_rec = Identity::did_records(alice_did);
        assert_eq!(did_rec.signing_keys, vec![charlie_signing_key]);
    }

    #[test]
    fn add_claim_issuer_tests() {
        with_externalities(&mut build_ext(), &add_claim_issuer_tests_with_externalities);
    }

    fn add_claim_issuer_tests_with_externalities() {
        // Register identities
        let (alice_acc, bob_acc, charlie_acc) = (1u64, 2u64, 3u64);
        let (alice, alice_did) = make_account(alice_acc).unwrap();
        let (_bob, bob_did) = make_account(bob_acc).unwrap();

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
        let (alice, alice_id) = make_account(a_acc).unwrap();
        let (bob, bob_id) = make_account(b_acc).unwrap();

        // Check external signed key uniqueness.
        let charlie_key = Key::try_from(c_acc.encode()).unwrap();
        let charlie_sk = SigningKey::new(charlie_key, vec![KeyRole::Operator]);
        assert_ok!(Identity::add_signing_keys(
            alice.clone(),
            alice_id,
            vec![charlie_sk.clone()]
        ));

        assert_err!(
            Identity::add_signing_keys(bob.clone(), bob_id, vec![charlie_sk]),
            unique_error
        );

        // Check non-external signed key non-uniqueness.
        let dave_key = Key::try_from(d_acc.encode()).unwrap();
        let dave_sk = SigningKey {
            key: dave_key,
            key_type: KeyType::Multisig,
            roles: vec![KeyRole::Operator],
        };
        assert_ok!(Identity::add_signing_keys(
            alice.clone(),
            alice_id,
            vec![dave_sk.clone()]
        ));
        assert_ok!(Identity::add_signing_keys(
            bob.clone(),
            bob_id,
            vec![dave_sk]
        ));

        // Check that master key acts like external signed key.
        let bob_key = Key::try_from(b_acc.encode()).unwrap();
        let bob_sk_as_mutisig = SigningKey {
            key: bob_key.clone(),
            key_type: KeyType::Multisig,
            roles: vec![KeyRole::Operator],
        };
        assert_err!(
            Identity::add_signing_keys(alice.clone(), alice_id, vec![bob_sk_as_mutisig]),
            unique_error
        );

        let bob_sk = SigningKey::new(bob_key, vec![KeyRole::Admin]);
        assert_err!(
            Identity::add_signing_keys(alice.clone(), alice_id, vec![bob_sk]),
            unique_error
        );
    }
}
