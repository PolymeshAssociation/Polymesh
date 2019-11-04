use rstd::{convert::TryFrom, prelude::*};

pub static DID_PREFIX: &'static str = "did:poly:";
use crate::balances;

use primitives::{DidRecord, Key, KeyRole, SigningKey};

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
    topic: u32,
    schema: u32,
    bytes: Vec<u8>,
    expiry: U,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct ClaimRecord<U> {
    claim: Claim<U>,
    revoked: bool,
    /// issuer DID
    issued_by: Vec<u8>,
    attestation: Vec<u8>,
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
        pub DidRecords get(did_records): map Vec<u8> => DidRecord<T::Balance>;

        /// DID -> DID claim issuers
        pub ClaimIssuers get(claim_issuers): map Vec<u8> => Vec<Vec<u8>>;

        /// DID -> Associated claims
        pub Claims get(claims): map Vec<u8> => Vec<ClaimRecord<T::Moment>>;

        // Signing key => DID
        pub SigningKeyDid get(signing_key_did): map Key => Vec<u8>;

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
        pub fn register_did(origin, did: Vec<u8>, signing_keys: Vec<SigningKey>) -> Result {
            let sender = ensure_signed(origin)?;
            let master_key = Key::try_from( sender.encode())?;

            // Make sure caller specified a correct DID
            validate_did(did.as_slice())?;

            // Make sure there's no pre-existing entry for the DID
            ensure!(!<DidRecords<T>>::exists(&did), "DID must be unique");

            // TODO: Subtract the fee
            let _imbalance = <balances::Module<T> as Currency<_>>::withdraw(
                &sender,
                Self::did_creation_fee(),
                WithdrawReason::Fee,
                ExistenceRequirement::KeepAlive
                )?;

            for roled_key in &signing_keys {
                let key = &roled_key.key;
                if <SigningKeyDid>::exists(key) {
                    ensure!(<SigningKeyDid>::get(key) == did, "One signing key can only belong to one DID");
                }
            }

            for roled_key in &signing_keys {
                <SigningKeyDid>::insert( &roled_key.key, did.clone());
            }

            let record = DidRecord {
                signing_keys: signing_keys.clone(),
                master_key,
                ..Default::default()
            };

            <DidRecords<T>>::insert(&did, record);

            Self::deposit_event(RawEvent::NewDid(did, sender, signing_keys));

            Ok(())
        }

        /// Adds new signing keys for a DID. Only called by master key owner.
        pub fn add_signing_keys(origin, did: Vec<u8>, additional_keys: Vec<SigningKey>) -> Result {
            let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, &did)?;

            for skey in &additional_keys {
                if <SigningKeyDid>::exists(&skey.key) {
                    ensure!(<SigningKeyDid>::get(&skey.key) == did, "One signing key can only belong to one DID");
                }
            }

            for skey in &additional_keys {
                <SigningKeyDid>::insert(&skey.key, did.clone());
            }

            <DidRecords<T>>::mutate(&did,
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
        fn remove_signing_keys(origin, did: Vec<u8>, keys_to_remove: Vec<Key>) -> Result {
            let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, &did)?;

            for key in &keys_to_remove {
                if <SigningKeyDid>::exists(key) {
                    ensure!(<SigningKeyDid>::get(key) == did, "Signing key does not belong to this DID");
                }
            }

            for key in &keys_to_remove {
                <SigningKeyDid>::remove(key);
            }

            <DidRecords<T>>::mutate(&did,
            |record| {
                let not_in_keys_to_remove = |skey: &SigningKey| keys_to_remove.iter()
                        .find(|&rk| skey == rk)
                        .is_none();

                (*record).signing_keys.retain( |skey| not_in_keys_to_remove(&skey));
                (*record).frozen_signing_keys.retain( |skey| not_in_keys_to_remove(&skey));
            });

            Self::deposit_event(RawEvent::SigningKeysRemoved(did, keys_to_remove));

            Ok(())
        }

        /// Sets a new master key for a DID. Only called by master key owner.
        fn set_master_key(origin, did: Vec<u8>, new_key: Key) -> Result {
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from( sender.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, &did)?;

            <DidRecords<T>>::mutate(&did,
            |record| {
                (*record).master_key = new_key.clone();
            });

            Self::deposit_event(RawEvent::NewMasterKey(did, sender, new_key));
            Ok(())
        }

        /// Adds funds to a DID.
        pub fn fund_poly(origin, did: Vec<u8>, amount: <T as balances::Trait>::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(&did), "DID must already exist");

            let record = <DidRecords<T>>::get(&did);

            // We must know that new balance is valid without creating side effects
            let new_record_balance = record.balance.checked_add(&amount).ok_or("overflow occured when increasing DID balance")?;

            let _imbalance = <balances::Module<T> as Currency<_>>::withdraw(
                &sender,
                amount,
                WithdrawReason::Fee,
                ExistenceRequirement::KeepAlive
                )?;

            <DidRecords<T>>::mutate(&did, |record| {
                (*record).balance = new_record_balance;
            });

            Self::deposit_event(RawEvent::PolyDepositedInDid(did, sender, amount));

            Ok(())
        }

        /// Withdraws funds from a DID. Only called by master key owner.
        fn withdrawy_poly(origin, did: Vec<u8>, amount: <T as balances::Trait>::Balance) -> Result {
            let sender = ensure_signed(origin)?;
            let record = Self::grant_check_only_master_key( &Key::try_from( sender.encode())?, &did)?;

            // We must know that new balance is valid without creating side effects
            let new_record_balance = record.balance.checked_sub(&amount).ok_or("underflow occured when decreasing DID balance")?;

            let _imbalance = <balances::Module<T> as Currency<_>>::deposit_into_existing(&sender, amount)?;

            <DidRecords<T>>::mutate(&did, |record| {
                (*record).balance = new_record_balance;
            });

            Self::deposit_event(RawEvent::PolyWithdrawnFromDid(did, sender, amount));

            Ok(())
        }

        /// Transfers funds between DIDs.
        fn transfer_poly(origin, did: Vec<u8>, to_did: Vec<u8>, amount: <T as balances::Trait>::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(Self::is_signing_key(&did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let from_record = <DidRecords<T>>::get(did.clone());
            let to_record = <DidRecords<T>>::get(to_did.clone());

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
        fn add_claim_issuer(origin, did: Vec<u8>, did_issuer: Vec<u8>) -> Result {
            let sender_key = Key::try_from( ensure_signed(origin)?.encode())?;
            let _grant_checked = Self::grant_check_only_master_key( &sender_key, &did)?;

            <ClaimIssuers>::mutate(did.clone(), |old_claim_issuers| {
                if !old_claim_issuers.contains(&did_issuer) {
                    old_claim_issuers.push(did_issuer.clone());
                }
            });

            Self::deposit_event(RawEvent::NewClaimIssuer(did, did_issuer));
            Ok(())
        }

        /// Removes a claim issuer DID. Only called by master key owner.
        fn remove_claim_issuer(origin, did: Vec<u8>, did_issuer: Vec<u8>) -> Result {
            let sender_key = Key::try_from( ensure_signed(origin)?.encode())?;
            let _grant_checked = Self::grant_check_only_master_key( &sender_key, &did)?;

            ensure!(<DidRecords<T>>::exists(&did_issuer), "claim issuer DID must already exist");

            <ClaimIssuers>::mutate(&did, |old_claim_issuers| {
                *old_claim_issuers = old_claim_issuers
                    .iter()
                    .filter(|&issuer| *issuer != did_issuer)
                    .cloned()
                    .collect();
            });

            Self::deposit_event(RawEvent::RemovedClaimIssuer(did, did_issuer));
            Ok(())
        }

        /// Adds new claim records. Only called by did_issuer's signing key
        fn add_claim(origin, did: Vec<u8>, did_issuer: Vec<u8>, claims: Vec<Claim<T::Moment>>) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(&did), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(&did_issuer), "claim issuer DID must already exist");

            let sender_key = Key::try_from( sender.encode())?;
            ensure!(Self::is_claim_issuer(&did, &did_issuer) || Self::is_master_key(&did, &sender_key), "did_issuer must be a claim issuer or master key for DID");

            // Verify that sender key is one of did_issuer's signing keys
            ensure!(Self::is_signing_key(&did_issuer, &sender_key), "Sender must hold a claim issuer's signing key");

            <Claims<T>>::mutate(&did, |claim_records| {
                let mut new_records = claims
                    .iter()
                    .cloned()
                    .map(|claim| ClaimRecord {
                        claim,
                        revoked: false,
                        issued_by: did_issuer.clone(),
                        attestation: Vec::new(),
                    })
                    .collect();

                claim_records.append(&mut new_records);
            });

            Self::deposit_event(RawEvent::NewClaims(did, did_issuer, claims));

            Ok(())
        }

        /// Adds new claim records with an attestation. Only called by issuer signing keys
        fn add_claim_with_attestation(origin, did: Vec<u8>, did_issuer: Vec<u8>, claims: Vec<Claim<T::Moment>>, attestation: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(&did), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(&did_issuer), "claim issuer DID must already exist");

            let sender_key = Key::try_from( sender.encode())?;
            ensure!(Self::is_claim_issuer(&did, &did_issuer) || Self::is_master_key(&did, &sender_key), "did_issuer must be a claim issuer or master key for DID");

            // Verify that sender key is one of did_issuer's signing keys
            ensure!(Self::is_signing_key(&did_issuer, &sender_key), "Sender must hold a claim issuer's signing key");

            <Claims<T>>::mutate(&did, |claim_records| {
                let mut new_records = claims
                    .iter()
                    .cloned()
                    .map(|claim| ClaimRecord {
                        claim,
                        revoked: false,
                        issued_by: did_issuer.clone(),
                        attestation: attestation.clone(),
                    })
                    .collect();

                claim_records.append(&mut new_records);
            });

            Self::deposit_event(RawEvent::NewClaimsWithAttestation(did, did_issuer, claims, attestation));

            Ok(())
        }

        /// Marks the specified claim as revoked
        fn revoke_claim(origin, did: Vec<u8>, did_issuer: Vec<u8>, claim: Claim<T::Moment>) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(&did), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(&did_issuer), "claim issuer DID must already exist");
            ensure!(Self::is_claim_issuer(&did, &did_issuer), "did_issuer must be a claim issuer for DID");

            // Verify that sender key is one of did_issuer's signing keys
            let sender_key = Key::try_from( sender.encode())?;
            ensure!(Self::is_signing_key(&did_issuer, &sender_key), "Sender must hold a claim issuer's signing key");

            <Claims<T>>::mutate(&did, |claim_records| {
                claim_records
                    .iter_mut()
                    .for_each(|record| if record.issued_by == did_issuer && record.claim == claim {
                        (*record).revoked = true;
                })
            });

            Self::deposit_event(RawEvent::RevokedClaim(did, did_issuer, claim));

            Ok(())
        }

        /// Marks all claims of an issuer as revoked
        fn revoke_all(origin, did: Vec<u8>, did_issuer: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(did_issuer.clone()), "claim issuer DID must already exist");
            ensure!(Self::is_claim_issuer(&did, &did_issuer), "did_issuer must be a claim issuer or master key for DID");

            // Verify that sender key is one of did_issuer's signing keys
            let sender_key = Key::try_from( sender.encode())?;
            ensure!(Self::is_signing_key(&did_issuer, &sender_key), "Sender must hold a claim issuer's signing key");

            <Claims<T>>::mutate(did.clone(), |claim_records| {

                claim_records
                    .iter_mut()
                    .for_each(|record| if record.issued_by == did_issuer {
                        (*record).revoked = true;
                })
            });

            Self::deposit_event(RawEvent::RevokedAllClaims(did, did_issuer));

            Ok(())
        }

        /// It sets roles for an specific `target_key` key.
        /// Only the master key of an identity is able to set signing key roles.
        fn set_role_to_signing_key(origin, did: Vec<u8>, target_key: Key, roles: Vec<KeyRole>) -> Result {
            let sender_key = Key::try_from( ensure_signed(origin)?.encode())?;
            let record = Self::grant_check_only_master_key( &sender_key, &did)?;

            // You are trying to add a role to did's master key. It is not needed.
            if record.master_key == target_key {
                return Ok(());
            }

            // Target did has sender's master key in its signing keys.
            ensure!(
                record.signing_keys.iter().find(|&rk| rk == &target_key).is_some(),
                "Sender is not part of did's signing keys"
            );

            // Get current roles of `key` at `investor_did`.
            let mut new_roles = match record.signing_keys.iter().find(|&rk| rk == &target_key) {
                Some(ref rk) => rk.roles.iter().chain( roles.iter()).cloned().collect(),
                None => roles.clone()
            };

            // Sort result and remove duplicates.
            new_roles.sort();
            new_roles.dedup();

            Self::update_roles(&did, &target_key, new_roles)
        }

        fn freeze_signing_keys(origin, did: Vec<u8>) -> Result {
            let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, &did)?;

            <DidRecords<T>>::mutate(&did,
                |record| {
                    (*record).frozen_signing_keys.append( &mut record.signing_keys);
                });
            Ok(())
        }

        fn unfreeze_signing_keys(origin, did: Vec<u8>) -> Result {
            let sender_key = Key::try_from(ensure_signed(origin)?.encode())?;
            let _grants_checked = Self::grant_check_only_master_key(&sender_key, &did)?;

            <DidRecords<T>>::mutate(&did,
                |record| {
                    (*record).signing_keys.append( &mut  record.frozen_signing_keys);
                });
            Ok(())
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
        NewDid(Vec<u8>, AccountId, Vec<SigningKey>),

        /// DID, new keys
        SigningKeysAdded(Vec<u8>, Vec<SigningKey>),

        /// DID, the keys that got removed
        SigningKeysRemoved(Vec<u8>, Vec<Key>),

        /// DID, old master key account ID, new key
        NewMasterKey(Vec<u8>, AccountId, Key),

        /// beneficiary DID, sender, amount
        PolyDepositedInDid(Vec<u8>, AccountId, Balance),

        /// DID, beneficiary, amount
        PolyWithdrawnFromDid(Vec<u8>, AccountId, Balance),

        /// DID, amount
        PolyChargedFromDid(Vec<u8>, Balance),

        /// DID from, DID to, amount
        PolyTransfer(Vec<u8>, Vec<u8>, Balance),

        /// DID, claim issuer DID
        NewClaimIssuer(Vec<u8>, Vec<u8>),

        /// DID, removed claim issuer DID
        RemovedClaimIssuer(Vec<u8>, Vec<u8>),

        /// DID, claim issuer DID, claims
        NewClaims(Vec<u8>, Vec<u8>, Vec<Claim<Moment>>),

        /// DID, claim issuer DID, claims, attestation
        NewClaimsWithAttestation(Vec<u8>, Vec<u8>, Vec<Claim<Moment>>, Vec<u8>),

        /// DID, claim issuer DID, claim
        RevokedClaim(Vec<u8>, Vec<u8>, Claim<Moment>),

        /// DID, claim issuer DID
        RevokedAllClaims(Vec<u8>, Vec<u8>),

        /// DID
        NewIssuer(Vec<u8>),
    }
);

impl<T: Trait> Module<T> {
    /// Private and not sanitized function. It is designed to be used internally by
    /// others sanitezed functions.
    fn update_roles(target_did: &Vec<u8>, key: &Key, roles: Vec<KeyRole>) -> Result {
        <DidRecords<T>>::mutate(target_did, |record| {
            // First filter avoids duplication of key.
            let mut signing_keys = record
                .signing_keys
                .iter()
                .filter(|&rk| rk != key)
                .cloned()
                .collect::<Vec<_>>();

            signing_keys.push(SigningKey::new(key.clone(), roles));
            (*record).signing_keys = signing_keys;
        });
        Ok(())
    }

    pub fn is_claim_issuer(did: &Vec<u8>, issuer_did: &Vec<u8>) -> bool {
        <ClaimIssuers>::get(did).contains(issuer_did)
    }

    pub fn is_signing_key(did: &Vec<u8>, key: &Key) -> bool {
        let record = <DidRecords<T>>::get(did);
        record.signing_keys.iter().find(|&rk| rk == key).is_some() || record.master_key == *key
    }

    /// Use `did` as reference.
    pub fn is_master_key(did: &Vec<u8>, key: &Key) -> bool {
        key == &<DidRecords<T>>::get(did).master_key
    }

    /// Withdraws funds from a DID balance
    pub fn charge_poly(did: Vec<u8>, amount: T::Balance) -> bool {
        if !<DidRecords<T>>::exists(did.clone()) {
            return false;
        }

        let record = <DidRecords<T>>::get(did.clone());

        if record.balance < amount {
            return false;
        }

        <DidRecords<T>>::mutate(did.clone(), |record| {
            (*record).balance = record.balance - amount;
        });

        Self::deposit_event(RawEvent::PolyChargedFromDid(did, amount));

        return true;
    }

    /// It checks that `sender_key` is the master key of `did` Identifier and that
    /// did exists.
    /// # Return
    /// A result object containing the `DidRecord` of `did`.
    pub fn grant_check_only_master_key(
        sender_key: &Key,
        did: &Vec<u8>,
    ) -> rstd::result::Result<DidRecord<<T as balances::Trait>::Balance>, &'static str> {
        ensure!(<DidRecords<T>>::exists(did), "DID does not exist");
        let record = <DidRecords<T>>::get(did);
        ensure!(
            *sender_key == record.master_key,
            "Only master key of an identity is able to execute this operation"
        );

        Ok(record)
    }
}

/// Make sure the supplied slice is a valid Polymesh DID
pub fn validate_did(did: &[u8]) -> Result {
    // TODO: Also check length after prefix,
    if did.starts_with(DID_PREFIX.as_bytes()) {
        Ok(())
    } else {
        Err("DID has no valid prefix")
    }
}

pub trait IdentityTrait<T> {
    fn signing_key_charge_did(signing_key: &Key) -> bool;
    fn charge_poly(signing_key: &Key, amount: T) -> bool;
}

impl<T: Trait> IdentityTrait<T::Balance> for Module<T> {
    fn charge_poly(signing_key: &Key, amount: T::Balance) -> bool {
        Self::charge_poly(<SigningKeyDid>::get(signing_key), amount)
    }

    fn signing_key_charge_did(signing_key: &Key) -> bool {
        if <SigningKeyDid>::exists(signing_key) {
            if Self::is_signing_key(&<SigningKeyDid>::get(signing_key), signing_key) {
                if <ChargeDid>::exists(signing_key) {
                    return <ChargeDid>::get(signing_key);
                }
            }
        }
        return false;
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use primitives::SigningKeyType;

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
    ) -> Result<(<IdentityTest as system::Trait>::Origin, Vec<u8>), &'static str> {
        let signed_id = Origin::signed(id);
        let did = format!("did:poly:{}", id).as_bytes().to_vec();

        Identity::register_did(signed_id.clone(), did.clone(), vec![])?;
        Ok((signed_id, did))
    }

    #[test]
    fn dids_are_unique() {
        with_externalities(&mut build_ext(), || {
            let did_1 = "did:poly:1".as_bytes().to_vec();

            assert_ok!(Identity::register_did(
                Origin::signed(1),
                did_1.clone(),
                vec![]
            ));

            assert_ok!(Identity::register_did(
                Origin::signed(2),
                "did:poly:2".as_bytes().to_vec(),
                vec![]
            ));

            assert_err!(
                Identity::register_did(Origin::signed(3), did_1, vec![]),
                "DID must be unique"
            );
        });
    }

    #[test]
    fn only_claim_issuers_can_add_claims() {
        with_externalities(&mut build_ext(), || {
            let owner_id = Identity::owner();
            let owner_key = Key::try_from(owner_id.encode()).unwrap();
            let (owner, owner_did) = make_account(owner_id).unwrap();

            let (issuer, issuer_did) = make_account(2).unwrap();
            let (claim_issuer, claim_issuer_did) = make_account(3).unwrap();

            assert_ok!(Identity::add_signing_keys(
                claim_issuer.clone(),
                claim_issuer_did.clone(),
                vec![SigningKey::from(owner_key.clone())]
            ));

            // Add Claims by master & claim_issuer
            let claims = vec![Claim {
                topic: 1,
                schema: 1,
                bytes: vec![],
                expiry: 10,
            }];

            assert_ok!(Identity::add_claim(
                owner.clone(),
                owner_did.clone(),
                claim_issuer_did.clone(),
                claims.clone()
            ));

            assert_err!(
                Identity::add_claim(
                    claim_issuer.clone(),
                    owner_did.clone(),
                    issuer_did.clone(),
                    claims.clone()
                ),
                "did_issuer must be a claim issuer or master key for DID"
            );
            assert_err!(
                Identity::add_claim(
                    issuer.clone(),
                    issuer_did.clone(),
                    claim_issuer_did.clone(),
                    claims.clone()
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

            assert_ok!(Identity::add_signing_keys(
                a.clone(),
                a_did.clone(),
                vec![SigningKey::from(owner_key.clone())]
            ));

            // Check master key on master and signing_keys.
            assert!(Identity::is_signing_key(&owner_did, &owner_key));
            assert!(Identity::is_signing_key(&a_did, &owner_key));

            assert!(Identity::is_signing_key(&b_did, &owner_key) == false);

            // ... and remove that key.
            assert_ok!(Identity::remove_signing_keys(
                a.clone(),
                a_did.clone(),
                vec![owner_key.clone()]
            ));
            assert!(Identity::is_signing_key(&a_did, &owner_key) == false);
        });
    }

    #[test]
    fn revoking_claims() {
        with_externalities(&mut build_ext(), || {
            let owner_id = Identity::owner();
            let owner_key = Key::try_from(owner_id.encode()).unwrap();
            let (owner, owner_did) = make_account(Identity::owner()).unwrap();
            let (issuer, issuer_did) = make_account(2).unwrap();

            let (claim_issuer, claim_issuer_did) = make_account(3).unwrap();
            assert_ok!(Identity::add_signing_keys(
                claim_issuer.clone(),
                claim_issuer_did.clone(),
                vec![SigningKey::from(owner_key)]
            ));

            assert_ok!(Identity::add_claim_issuer(
                owner.clone(),
                owner_did.clone(),
                claim_issuer_did.clone()
            ));

            // Add Claims by master & claim_issuer
            let claim = Claim {
                topic: 1,
                schema: 1,
                bytes: vec![],
                expiry: 10,
            };

            assert_ok!(Identity::add_claim(
                owner.clone(),
                owner_did.clone(),
                claim_issuer_did.clone(),
                vec![claim.clone()]
            ));

            assert_err!(
                Identity::revoke_claim(
                    issuer.clone(),
                    issuer_did.clone(),
                    claim_issuer_did.clone(),
                    claim.clone()
                ),
                "did_issuer must be a claim issuer for DID"
            );
            // TODO Should this fail?
            assert_err!(
                Identity::revoke_claim(
                    claim_issuer.clone(),
                    claim_issuer_did.clone(),
                    claim_issuer_did.clone(),
                    claim.clone()
                ),
                "did_issuer must be a claim issuer for DID"
            );

            assert_ok!(Identity::revoke_claim(
                owner.clone(),
                owner_did.clone(),
                claim_issuer_did.clone(),
                claim.clone()
            ));
            // TODO Revoke claim twice??
            assert_ok!(Identity::revoke_claim(
                owner,
                owner_did,
                claim_issuer_did,
                claim
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
            alice_did.clone(),
            vec![
                SigningKey::from(bob_key.clone()),
                SigningKey::from(charlie_key.clone())
            ]
        ));

        // Only `alice` is able to update `bob`'s roles and `charlie`'s roles.
        assert_ok!(Identity::set_role_to_signing_key(
            alice.clone(),
            alice_did.clone(),
            bob_key.clone(),
            vec![KeyRole::Operator]
        ));
        assert_ok!(Identity::set_role_to_signing_key(
            alice.clone(),
            alice_did.clone(),
            charlie_key.clone(),
            vec![KeyRole::Admin, KeyRole::Operator]
        ));

        // Bob tries to get better role by himself at `alice` Identity.
        assert_err!(
            Identity::set_role_to_signing_key(
                Origin::signed(bob_acc),
                alice_did.clone(),
                bob_key.clone(),
                vec![KeyRole::Full]
            ),
            "Only master key of an identity is able to execute this operation"
        );

        // Bob tries to remove Charlie's roles at `alice` Identity.
        assert_err!(
            Identity::set_role_to_signing_key(
                Origin::signed(bob_acc),
                alice_did.clone(),
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
    /// (`SigningKeyType::External`).
    fn add_signing_keys_with_specific_type_with_externalities() {
        let (alice_acc, bob_acc, charlie_acc, dave_acc) = (1u64, 2u64, 3u64, 4u64);
        let (bob_key, charlie_key, dave_key) = (
            Key::try_from(bob_acc.encode()).unwrap(),
            Key::try_from(charlie_acc.encode()).unwrap(),
            Key::try_from(dave_acc.encode()).unwrap(),
        );

        // Create keys using non-default type.
        let bob_signing_key = SigningKey {
            key: bob_key,
            roles: vec![],
            key_type: SigningKeyType::Identity,
        };
        let charlie_signing_key = SigningKey {
            key: charlie_key,
            key_type: SigningKeyType::Relayer,
            roles: vec![],
        };
        let dave_signing_key = SigningKey {
            key: dave_key,
            key_type: SigningKeyType::Multisig,
            roles: vec![],
        };

        // Add signing keys with non-default type.
        let (alice, alice_did) = make_account(alice_acc).unwrap();
        assert_ok!(Identity::add_signing_keys(
            alice,
            alice_did,
            vec![bob_signing_key, charlie_signing_key]
        ));

        // Register did with non-default type.
        let bob_did = format!("did:poly:{}", bob_acc).as_bytes().to_vec();
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

        let bob_signing_key = SigningKey::new(bob_key, vec![KeyRole::Admin]);
        let charlie_signing_key = SigningKey::new(charlie_key, vec![KeyRole::Operator]);
        let dave_signing_key = SigningKey::new(dave_key, vec![]);

        // Add signing keys.
        let (alice, alice_did) = make_account(alice_acc).unwrap();
        let signing_keys_v1 = vec![bob_signing_key, charlie_signing_key];
        assert_ok!(Identity::add_signing_keys(
            alice.clone(),
            alice_did.clone(),
            signing_keys_v1.clone()
        ));

        // Freeze signing keys: bob & charlie.
        assert_err!(
            Identity::freeze_signing_keys(Origin::signed(bob_acc), alice_did.clone()),
            "Only master key of an identity is able to execute this operation"
        );
        assert_ok!(Identity::freeze_signing_keys(
            alice.clone(),
            alice_did.clone()
        ));

        let did_rec_1 = Identity::did_records(alice_did.clone());
        assert_eq!(did_rec_1.signing_keys.len(), 0);
        assert_eq!(did_rec_1.frozen_signing_keys, signing_keys_v1);

        // Add new signing keys.
        let signing_keys_v2 = vec![dave_signing_key];
        assert_ok!(Identity::add_signing_keys(
            alice.clone(),
            alice_did.clone(),
            signing_keys_v2.clone()
        ));
        let did_rec_2 = Identity::did_records(alice_did.clone());
        assert_eq!(did_rec_2.signing_keys, signing_keys_v2);
        assert_eq!(did_rec_2.frozen_signing_keys, signing_keys_v1);

        // 2nd freeze
        let all_signing_keys = signing_keys_v1
            .iter()
            .chain(signing_keys_v2.iter())
            .cloned()
            .collect::<Vec<_>>();
        assert_ok!(Identity::freeze_signing_keys(
            alice.clone(),
            alice_did.clone()
        ));
        let did_rec_3 = Identity::did_records(alice_did.clone());
        assert_eq!(did_rec_3.signing_keys, Vec::<SigningKey>::new());
        assert_eq!(did_rec_3.frozen_signing_keys, all_signing_keys);

        // unfreeze all
        assert_err!(
            Identity::unfreeze_signing_keys(Origin::signed(bob_acc), alice_did.clone()),
            "Only master key of an identity is able to execute this operation"
        );
        assert_ok!(Identity::unfreeze_signing_keys(
            alice.clone(),
            alice_did.clone()
        ));

        let did_rec_4 = Identity::did_records(alice_did.clone());
        assert_eq!(did_rec_4.signing_keys, all_signing_keys);
        assert_eq!(did_rec_4.frozen_signing_keys, Vec::<SigningKey>::new());
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
            alice_did.clone(),
            signing_keys_v1.clone()
        ));

        // Freeze all signing keys
        assert_ok!(Identity::freeze_signing_keys(
            alice.clone(),
            alice_did.clone()
        ));

        // Remove Bob's key.
        assert_ok!(Identity::remove_signing_keys(
            alice.clone(),
            alice_did.clone(),
            vec![bob_key.clone()]
        ));

        // Check DidRecord.
        let did_rec = Identity::did_records(alice_did.clone());
        assert_eq!(did_rec.frozen_signing_keys, vec![charlie_signing_key]);
        assert_eq!(did_rec.signing_keys.len(), 0);
    }
}
