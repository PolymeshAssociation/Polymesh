use rstd::prelude::*;
//use codec::Codec;

pub static DID_PREFIX: &'static str = "did:poly:";
use crate::balances;

use codec::Encode;
use sr_primitives::traits::{CheckedAdd, CheckedSub};
use srml_support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, WithdrawReason},
    StorageMap,
};
use system::{self, ensure_signed};

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Issuer<U> {
    account: U,
    access_level: u16,
    active: bool,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Investor<U> {
    pub account: U,
    pub access_level: u16,
    pub active: bool,
    pub jurisdiction: u16,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct DidRecord<U> {
    pub master_key: Vec<u8>,
    pub signing_keys: Vec<Vec<u8>>,
    pub balance: U,
}

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

        ERC20IssuerList get(erc20_issuer_list): map T::AccountId => Issuer<T::AccountId>;
        IssuerList get(issuer_list): map T::AccountId => Issuer<T::AccountId>;
        pub InvestorList get(investor_list): map T::AccountId => Investor<T::AccountId>;

        /// DID -> identity info
        pub DidRecords get(did_records): map Vec<u8> => DidRecord<T::Balance>;

        /// DID -> DID claim issuers
        pub ClaimIssuers get(claim_issuers): map Vec<u8> => Vec<Vec<u8>>;

        /// DID -> Associated claims
        pub Claims get(claims): map Vec<u8> => Vec<ClaimRecord<T::Moment>>;

        // Signing key => DID
        pub SigningKeyDid get(signing_key_did): map Vec<u8> => Vec<u8>;

        // Signing key => Charge Fee to did?. Default is false i.e. the fee will be charged from user balance
        pub ChargeDid get(charge_did): map Vec<u8> => bool;

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        fn create_issuer(origin,_issuer: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::owner() == sender,"Sender must be the identity module owner");

            Self::do_create_issuer(_issuer)
        }

        fn create_erc20_issuer(origin,_issuer: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::owner() == sender,"Sender must be the identity module owner");

            Self::do_create_erc20_issuer(_issuer)
        }

        fn create_investor(origin,_investor: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::owner() == sender,"Sender must be the identity module owner");

            Self::do_create_investor(_investor)
        }

        fn set_charge_did(origin, charge_did: bool) -> Result {
            let sender = ensure_signed(origin)?;
            <ChargeDid>::insert(sender.encode(), charge_did);
            Ok(())
        }

        /// Register signing keys for a new DID. Uses origin key as the master key
        fn register(origin, did: Vec<u8>, signing_keys: Vec<Vec<u8>>) -> Result {

            let sender = ensure_signed(origin)?;

            let master_key = sender.encode();

            // Make sure there's no pre-existing entry for the DID
            ensure!(!<DidRecords<T>>::exists(did.clone()), "DID must be unique");

            // Make sure caller specified a correct DID
            validate_did(did.as_slice())?;

            for key in &signing_keys {
                if <SigningKeyDid>::exists(key.clone()) {
                    ensure!(<SigningKeyDid>::get(key) == did.clone(), "One signing key can only belong to one DID");
                }
            }

            for key in &signing_keys {
                <SigningKeyDid>::insert(key, did.clone());
            }

            let record = DidRecord {
                signing_keys: signing_keys.clone(),
                master_key,
                ..Default::default()
            };

            <DidRecords<T>>::insert(did.clone(), record);

            Self::deposit_event(RawEvent::NewDid(did, sender, signing_keys));

            Ok(())
        }

        /// Adds new signing keys for a DID. Only called by master key owner.
        fn add_signing_keys(origin, did: Vec<u8>, additional_keys: Vec<Vec<u8>>) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");

            // Verify that sender key is current master key
            let sender_key = sender.encode();
            let record = <DidRecords<T>>::get(did.clone());
            ensure!(sender_key == record.master_key, "Sender must hold the master key");

            for key in &additional_keys {
                if <SigningKeyDid>::exists(key.clone()) {
                    ensure!(<SigningKeyDid>::get(key) == did.clone(), "One signing key can only belong to one DID");
                }
            }

            for key in &additional_keys {
                <SigningKeyDid>::insert(key, did.clone());
            }

            <DidRecords<T>>::mutate(did.clone(),
            |record| {
                // Concatenate new keys while making sure the key set is
                // unique
                let mut new_keys: Vec<Vec<u8>> = record.signing_keys
                    .iter()
                    .cloned()
                    .filter(|key| {
                        !additional_keys.contains(key)
                    }).collect();
                new_keys.append(&mut additional_keys.clone());

                (*record).signing_keys = new_keys;
            });

            Self::deposit_event(RawEvent::SigningKeysAdded(did, additional_keys));

            Ok(())
        }

        /// Removes specified signing keys of a DID if present. Only called by master key owner.
        fn remove_signing_keys(origin, did: Vec<u8>, keys_to_remove: Vec<Vec<u8>>) -> Result {
            let sender = ensure_signed(origin)?;

            // Verify that sender key is current master key
            let sender_key = sender.encode();
            let record = <DidRecords<T>>::get(did.clone());
            ensure!(sender_key == record.master_key, "Sender must hold the master key");

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");

            for key in &keys_to_remove {
                if <SigningKeyDid>::exists(key.clone()) {
                    ensure!(<SigningKeyDid>::get(key) == did.clone(), "Signing key does not belong to this DID");
                }
            }

            for key in &keys_to_remove {
                <SigningKeyDid>::remove(key);
            }

            <DidRecords<T>>::mutate(did.clone(),
            |record| {
                // Filter out keys meant for deletion
                (*record).signing_keys = record.signing_keys
                    .iter()
                    .cloned()
                    .filter(|key| {
                        !keys_to_remove.contains(key)
                    }).collect();
            });

            Self::deposit_event(RawEvent::SigningKeysRemoved(did, keys_to_remove));

            Ok(())
        }

        /// Sets a new master key for a DID. Only called by master key owner.
        fn set_master_key(origin, did: Vec<u8>, new_key: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            // Verify that sender key is current master key
            let sender_key = sender.encode();
            let record = <DidRecords<T>>::get(did.clone());
            ensure!(sender_key == record.master_key, "Sender must hold the master key");

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");

            <DidRecords<T>>::mutate(did.clone(),
            |record| {
                (*record).master_key = new_key.clone();
            });

            Self::deposit_event(RawEvent::NewMasterKey(did, sender, new_key));

            Ok(())
        }

        /// Adds funds to a DID.
        fn fund_poly(origin, did: Vec<u8>, amount: <T as balances::Trait>::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");

            let record = <DidRecords<T>>::get(did.clone());

            // We must know that new balance is valid without creating side effects
            let new_record_balance = record.balance.checked_add(&amount).ok_or("overflow occured when increasing DID balance")?;

            let _imbalance = <balances::Module<T> as Currency<_>>::withdraw(
                &sender,
                amount,
                WithdrawReason::Transfer,
                ExistenceRequirement::KeepAlive
                )?;

            <DidRecords<T>>::mutate(did.clone(), |record| {
                (*record).balance = new_record_balance;
            });

            Self::deposit_event(RawEvent::PolyDepositedInDid(did, sender, amount));

            Ok(())
        }

        /// Withdraws funds from a DID. Only called by master key owner.
        fn withdrawy_poly(origin, did: Vec<u8>, amount: <T as balances::Trait>::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // Verify that sender key is current master key
            let sender_key = sender.encode();
            let record = <DidRecords<T>>::get(did.clone());
            ensure!(sender_key == record.master_key, "Sender must hold the master key");

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");

            let record = <DidRecords<T>>::get(did.clone());

            // We must know that new balance is valid without creating side effects
            let new_record_balance = record.balance.checked_sub(&amount).ok_or("underflow occured when decreasing DID balance")?;

            let _imbalance = <balances::Module<T> as Currency<_>>::deposit_into_existing(&sender, amount)?;

            <DidRecords<T>>::mutate(did.clone(), |record| {
                (*record).balance = new_record_balance;
            });

            Self::deposit_event(RawEvent::PolyWithdrawnFromDid(did, sender, amount));

            Ok(())
        }

        /// Appends a claim issuer DID to a DID. Only called by master key owner.
        fn add_claim_issuer(origin, did: Vec<u8>, did_issuer: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            // Verify that sender key is current master key
            let sender_key = sender.encode();
            let record = <DidRecords<T>>::get(did.clone());
            ensure!(sender_key == record.master_key, "Sender must hold the master key");

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(did_issuer.clone()), "claim issuer DID must already exist");

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
            let sender = ensure_signed(origin)?;

            // Verify that sender key is current master key
            let sender_key = sender.encode();
            let record = <DidRecords<T>>::get(did.clone());
            ensure!(sender_key == record.master_key, "Sender must hold the master key");

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(did_issuer.clone()), "claim issuer DID must already exist");

            <ClaimIssuers>::mutate(did.clone(), |old_claim_issuers| {
                *old_claim_issuers = old_claim_issuers
                    .iter()
                    .cloned()
                    .filter(|issuer| *issuer != did_issuer)
                    .collect();
            });

            Self::deposit_event(RawEvent::RemovedClaimIssuer(did, did_issuer));

            Ok(())
        }

        /// Adds new claim records. Only called by did_issuer's signing key
        fn add_claim(origin, did: Vec<u8>, did_issuer: Vec<u8>, claims: Vec<Claim<T::Moment>>) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(did_issuer.clone()), "claim issuer DID must already exist");

            // Verify that sender key is one of did_issuer's signing keys
            let sender_key = sender.encode();
            ensure!(Self::is_signing_key(did_issuer.clone(), &sender_key), "Sender must hold a claim issuer's signing key");

            <Claims<T>>::mutate(did.clone(), |claim_records| {
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

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(did_issuer.clone()), "claim issuer DID must already exist");
            ensure!(Self::is_claim_issuer(did.clone(), &did_issuer), "did_issuer must be a claim issuer for DID");

            // Verify that sender key is one of did_issuer's signing keys
            let sender_key = sender.encode();
            ensure!(Self::is_signing_key(did_issuer.clone(), &sender_key), "Sender must hold a claim issuer's signing key");

            <Claims<T>>::mutate(did.clone(), |claim_records| {
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

            ensure!(<DidRecords<T>>::exists(did.clone()), "DID must already exist");
            ensure!(<DidRecords<T>>::exists(did_issuer.clone()), "claim issuer DID must already exist");
            ensure!(Self::is_claim_issuer(did.clone(), &did_issuer), "did_issuer must be a claim issuer for DID");

            // Verify that sender key is one of did_issuer's signing keys
            let sender_key = sender.encode();
            ensure!(Self::is_signing_key(did_issuer.clone(), &sender_key), "Sender must hold a claim issuer's signing key");

            <Claims<T>>::mutate(did.clone(), |claim_records| {

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
            ensure!(Self::is_claim_issuer(did.clone(), &did_issuer), "did_issuer must be a claim issuer for DID");

            // Verify that sender key is one of did_issuer's signing keys
            let sender_key = sender.encode();
            ensure!(Self::is_signing_key(did_issuer.clone(), &sender_key), "Sender must hold a claim issuer's signing key");

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
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as balances::Trait>::Balance,
        Moment = <T as timestamp::Trait>::Moment,
    {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit funtion, from our runtime funtions
        SomethingStored(u32, AccountId),

        /// DID, master key account ID, signing keys
        NewDid(Vec<u8>, AccountId, Vec<Vec<u8>>),

        /// DID, new keys
        SigningKeysAdded(Vec<u8>, Vec<Vec<u8>>),

        /// DID, the keys that got removed
        SigningKeysRemoved(Vec<u8>, Vec<Vec<u8>>),

        /// DID, old master key account ID, new key
        NewMasterKey(Vec<u8>, AccountId, Vec<u8>),

        /// beneficiary DID, sender, amount
        PolyDepositedInDid(Vec<u8>, AccountId, Balance),

        /// DID, beneficiary, amount
        PolyWithdrawnFromDid(Vec<u8>, AccountId, Balance),

        /// DID, amount
        PolyChargedFromDid(Vec<u8>, Balance),

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
    }
);

impl<T: Trait> Module<T> {
    /// Add a new issuer. Warning: No identity module ownership checks are performed
    pub fn do_create_issuer(_issuer: T::AccountId) -> Result {
        let new_issuer = Issuer {
            account: _issuer.clone(),
            access_level: 1,
            active: true,
        };

        <IssuerList<T>>::insert(_issuer, new_issuer);
        Ok(())
    }

    /// Add a new ERC20 issuer. Warning: No identity module ownership checks are performed
    pub fn do_create_erc20_issuer(_erc20_issuer: T::AccountId) -> Result {
        let new_erc20_issuer = Issuer {
            account: _erc20_issuer.clone(),
            access_level: 1,
            active: true,
        };

        <ERC20IssuerList<T>>::insert(_erc20_issuer, new_erc20_issuer);
        Ok(())
    }

    /// Add a new investor. Warning: No identity module ownership checks are performed
    pub fn do_create_investor(_investor: T::AccountId) -> Result {
        let new_investor = Investor {
            account: _investor.clone(),
            access_level: 1,
            active: true,
            jurisdiction: 1,
        };

        <InvestorList<T>>::insert(_investor, new_investor);
        Ok(())
    }

    pub fn is_issuer(_user: T::AccountId) -> bool {
        let user = Self::issuer_list(_user.clone());
        user.account == _user && user.access_level == 1 && user.active
    }

    pub fn is_erc20_issuer(_user: T::AccountId) -> bool {
        let user = Self::erc20_issuer_list(_user.clone());
        user.account == _user && user.access_level == 1 && user.active
    }

    pub fn is_investor(_user: T::AccountId) -> bool {
        let user = Self::investor_list(_user.clone());
        user.account == _user && user.access_level == 1 && user.active
    }

    pub fn is_claim_issuer(did: Vec<u8>, did_issuer: &Vec<u8>) -> bool {
        <ClaimIssuers>::get(did).contains(&did_issuer)
    }

    pub fn is_signing_key(did: Vec<u8>, key: &Vec<u8>) -> bool {
        <DidRecords<T>>::get(did).signing_keys.contains(key)
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
    fn signing_key_charge_did(signing_key: Vec<u8>) -> bool;
    fn charge_poly(did: Vec<u8>, amount: T) -> bool;
}

impl<T: Trait> IdentityTrait<T::Balance> for Module<T> {
    fn charge_poly(signing_key: Vec<u8>, amount: T::Balance) -> bool {
        Self::charge_poly(<SigningKeyDid>::get(signing_key), amount)
    }

    fn signing_key_charge_did(signing_key: Vec<u8>) -> bool {
        if <SigningKeyDid>::exists(signing_key.clone()) {
            if Self::is_signing_key(<SigningKeyDid>::get(signing_key.clone()), &signing_key) {
                if <ChargeDid>::exists(signing_key.clone()) {
                    return <ChargeDid>::get(signing_key.clone());
                }
            }
        }
        return false;
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    /*
     *    use super::*;
     *
     *    use primitives::{Blake2Hasher, H256};
     *    use sr_io::with_externalities;
     *    use sr_primitives::{
     *        testing::{Digest, DigestItem, Header},
     *        traits::{BlakeTwo256, IdentityLookup},
     *        BuildStorage,
     *    };
     *    use srml_support::{assert_ok, impl_outer_origin};
     *
     *    impl_outer_origin! {
     *        pub enum Origin for Test {}
     *    }
     *
     *    // For testing the module, we construct most of a mock runtime. This means
     *    // first constructing a configuration type (`Test`) which `impl`s each of the
     *    // configuration traits of modules we want to use.
     *    #[derive(Clone, Eq, PartialEq)]
     *    pub struct Test;
     *    impl system::Trait for Test {
     *        type Origin = Origin;
     *        type Index = u64;
     *        type BlockNumber = u64;
     *        type Hash = H256;
     *        type Hashing = BlakeTwo256;
     *        type Digest = Digest;
     *        type AccountId = u64;
     *        type Lookup = IdentityLookup<Self::AccountId>;
     *        type Header = Header;
     *        type Event = ();
     *        type Log = DigestItem;
     *    }
     *    impl Trait for Test {
     *        type Event = ();
     *    }
     *    type identity = Module<Test>;
     *
     *    // This function basically just builds a genesis storage key/value store according to
     *    // our desired mockup.
     *    fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
     *        system::GenesisConfig::<Test>::default()
     *            .build_storage()
     *            .unwrap()
     *            .0
     *            .into()
     *    }
     *
     *    #[test]
     *    fn it_works_for_default_value() {
     *        with_externalities(&mut new_test_ext(), || {
     *            // Just a dummy test for the dummy funtion `do_something`
     *            // calling the `do_something` function with a value 42
     *            assert_ok!(identity::do_something(Origin::signed(1), 42));
     *            // asserting that the stored value is equal to what we stored
     *            assert_eq!(identity::something(), Some(42));
     *        });
     *    }
     */
}
