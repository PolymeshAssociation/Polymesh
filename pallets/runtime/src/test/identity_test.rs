use crate::test::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};

use polymesh_primitives::{
    AccountKey, AuthorizationData, IdentityClaimData, LinkData, Permission, Signatory,
    SignatoryType, SigningItem, Ticker,
};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::identity::{SigningItemWithAuth, TargetIdAuthorization};
use polymesh_runtime_identity::{self as identity, Error};

use codec::Encode;
use frame_support::{assert_err, assert_ok, traits::Currency, StorageDoubleMap};
use sp_core::H512;
use test_client::AccountKeyring;

use std::convert::TryFrom;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type System = frame_system::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;

type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn add_claims_batch() {
    ExtBuilder::default().build().execute_with(|| {
        let _owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let _issuer_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let _issuer = AccountKeyring::Bob.public();
        let claim_issuer_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let claim_issuer = AccountKeyring::Charlie.public();

        let claim_records = vec![
            (claim_issuer_did.clone(), 100u64, IdentityClaimData::NoData),
            (
                claim_issuer_did.clone(),
                200u64,
                IdentityClaimData::CustomerDueDiligence,
            ),
        ];
        assert_ok!(Identity::add_claims_batch(
            Origin::signed(claim_issuer.clone()),
            claim_records,
        ));

        let claim1 = Identity::fetch_valid_claim(
            claim_issuer_did,
            IdentityClaimData::NoData,
            claim_issuer_did,
        )
        .unwrap();
        let claim2 = Identity::fetch_valid_claim(
            claim_issuer_did,
            IdentityClaimData::CustomerDueDiligence,
            claim_issuer_did,
        )
        .unwrap();

        assert_eq!(claim1.expiry, 100u64);
        assert_eq!(claim2.expiry, 200u64);

        assert_eq!(claim1.claim, IdentityClaimData::NoData);

        assert_eq!(claim2.claim, IdentityClaimData::CustomerDueDiligence);
    });
}

/// TODO Add `Signatory::Identity(..)` test.
#[test]
fn only_master_or_signing_keys_can_authenticate_as_an_identity() {
    ExtBuilder::default().build().execute_with(|| {
        let owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let owner_signer =
            Signatory::AccountKey(AccountKey::from(AccountKeyring::Alice.public().0));

        let a_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let a = Origin::signed(AccountKeyring::Bob.public());
        let b_did = register_keyring_account(AccountKeyring::Dave).unwrap();

        let charlie_key = AccountKey::from(AccountKeyring::Charlie.public().0);
        let charlie_signer = Signatory::AccountKey(charlie_key);
        let charlie_signing_item =
            SigningItem::new(charlie_signer.clone(), vec![Permission::Admin]);

        assert_ok!(Identity::add_signing_items(
            a.clone(),
            vec![charlie_signing_item]
        ));
        assert_ok!(Identity::authorize_join_to_identity(
            Origin::signed(AccountKeyring::Charlie.public()),
            a_did
        ));

        // Check master key on master and signing_keys.
        assert!(Identity::is_signer_authorized(owner_did, &owner_signer));
        assert!(Identity::is_signer_authorized(a_did, &charlie_signer));

        assert!(Identity::is_signer_authorized(b_did, &charlie_signer) == false);

        // ... and remove that key.
        assert_ok!(Identity::remove_signing_items(
            a.clone(),
            vec![charlie_signer.clone()]
        ));
        assert!(Identity::is_signer_authorized(a_did, &charlie_signer) == false);
    });
}

#[test]
fn revoking_claims() {
    ExtBuilder::default().build().execute_with(|| {
        let _owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let _issuer_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let _issuer = Origin::signed(AccountKeyring::Bob.public());
        let claim_issuer_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let claim_issuer = Origin::signed(AccountKeyring::Charlie.public());

        assert_ok!(Identity::add_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            IdentityClaimData::NoData,
            100u64,
        ));
        assert!(Identity::fetch_valid_claim(
            claim_issuer_did,
            IdentityClaimData::NoData,
            claim_issuer_did
        )
        .is_some());

        assert_ok!(Identity::revoke_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            IdentityClaimData::NoData
        ));
        assert!(Identity::fetch_valid_claim(
            claim_issuer_did,
            IdentityClaimData::NoData,
            claim_issuer_did
        )
        .is_none());
    });
}

#[test]
fn only_master_key_can_add_signing_key_permissions() {
    ExtBuilder::default()
        .build()
        .execute_with(&only_master_key_can_add_signing_key_permissions_with_externalities);
}

fn only_master_key_can_add_signing_key_permissions_with_externalities() {
    let bob_key = AccountKey::from(AccountKeyring::Bob.public().0);
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public().0);
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());

    assert_ok!(Identity::add_signing_items(
        alice.clone(),
        vec![SigningItem::from(bob_key), SigningItem::from(charlie_key)]
    ));
    assert_ok!(Identity::authorize_join_to_identity(bob.clone(), alice_did));
    assert_ok!(Identity::authorize_join_to_identity(charlie, alice_did));

    // Only `alice` is able to update `bob`'s permissions and `charlie`'s permissions.
    assert_ok!(Identity::set_permission_to_signer(
        alice.clone(),
        Signatory::AccountKey(bob_key),
        vec![Permission::Operator]
    ));
    assert_ok!(Identity::set_permission_to_signer(
        alice.clone(),
        Signatory::AccountKey(charlie_key),
        vec![Permission::Admin, Permission::Operator]
    ));

    // Bob tries to get better permission by himself at `alice` Identity.
    assert_err!(
        Identity::set_permission_to_signer(
            bob.clone(),
            Signatory::AccountKey(bob_key),
            vec![Permission::Full]
        ),
        Error::<TestStorage>::KeyNotAllowed
    );

    // Bob tries to remove Charlie's permissions at `alice` Identity.
    assert_err!(
        Identity::set_permission_to_signer(bob, Signatory::AccountKey(charlie_key), vec![]),
        Error::<TestStorage>::KeyNotAllowed
    );

    // Alice over-write some permissions.
    assert_ok!(Identity::set_permission_to_signer(
        alice,
        Signatory::AccountKey(bob_key),
        vec![]
    ));
}

#[test]
fn add_signing_keys_with_specific_type() {
    ExtBuilder::default()
        .build()
        .execute_with(&add_signing_keys_with_specific_type_with_externalities);
}

/// It tests that signing key can be added using non-default key type
/// (`SignatoryType::External`).
fn add_signing_keys_with_specific_type_with_externalities() {
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public().0);
    let dave_key = AccountKey::from(AccountKeyring::Dave.public().0);

    // Create keys using non-default type.
    let charlie_signing_key = SigningItem {
        signer: Signatory::AccountKey(charlie_key),
        signer_type: SignatoryType::Relayer,
        permissions: vec![],
    };
    let dave_signing_key = SigningItem {
        signer: Signatory::AccountKey(dave_key),
        signer_type: SignatoryType::MultiSig,
        permissions: vec![],
    };

    // Add signing keys with non-default type.
    let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    assert_ok!(Identity::add_signing_items(
        alice,
        vec![charlie_signing_key, dave_signing_key.clone()]
    ));

    // Register did with non-default type.
    let bob = AccountKeyring::Bob.public();
    Balances::make_free_balance_be(&bob, 5_000);
    assert_ok!(Identity::register_did(
        Origin::signed(bob),
        vec![dave_signing_key]
    ));
}

/// It verifies that frozen keys are recovered after `unfreeze` call.
#[test]
fn freeze_signing_keys_test() {
    ExtBuilder::default()
        .build()
        .execute_with(&freeze_signing_keys_with_externalities);
}

fn freeze_signing_keys_with_externalities() {
    let (bob_key, charlie_key, dave_key) = (
        AccountKey::from(AccountKeyring::Bob.public().0),
        AccountKey::from(AccountKeyring::Charlie.public().0),
        AccountKey::from(AccountKeyring::Dave.public().0),
    );
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());
    let dave = Origin::signed(AccountKeyring::Dave.public());

    let bob_signing_key = SigningItem::new(Signatory::AccountKey(bob_key), vec![Permission::Admin]);
    let charlie_signing_key = SigningItem::new(
        Signatory::AccountKey(charlie_key),
        vec![Permission::Operator],
    );
    let dave_signing_key = SigningItem::from(dave_key);

    // Add signing keys.
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());

    let signing_keys_v1 = vec![bob_signing_key.clone(), charlie_signing_key];
    assert_ok!(Identity::add_signing_items(
        alice.clone(),
        signing_keys_v1.clone()
    ));
    assert_ok!(Identity::authorize_join_to_identity(bob.clone(), alice_did));
    assert_ok!(Identity::authorize_join_to_identity(
        charlie.clone(),
        alice_did
    ));

    assert_eq!(
        Identity::is_signer_authorized(alice_did, &Signatory::AccountKey(bob_key)),
        true
    );

    // Freeze signing keys: bob & charlie.
    assert_err!(
        Identity::freeze_signing_keys(bob.clone()),
        Error::<TestStorage>::KeyNotAllowed
    );
    assert_ok!(Identity::freeze_signing_keys(alice.clone()));

    assert_eq!(
        Identity::is_signer_authorized(alice_did, &Signatory::AccountKey(bob_key)),
        false
    );

    // Add new signing keys.
    let signing_keys_v2 = vec![dave_signing_key.clone()];
    assert_ok!(Identity::add_signing_items(
        alice.clone(),
        signing_keys_v2.clone()
    ));
    assert_ok!(Identity::authorize_join_to_identity(dave, alice_did));
    assert_eq!(
        Identity::is_signer_authorized(alice_did, &Signatory::AccountKey(dave_key)),
        false
    );

    // update permission of frozen keys.
    assert_ok!(Identity::set_permission_to_signer(
        alice.clone(),
        Signatory::AccountKey(bob_key),
        vec![Permission::Operator]
    ));

    // unfreeze all
    assert_err!(
        Identity::unfreeze_signing_keys(bob.clone()),
        Error::<TestStorage>::KeyNotAllowed
    );
    assert_ok!(Identity::unfreeze_signing_keys(alice.clone()));

    assert_eq!(
        Identity::is_signer_authorized(alice_did, &Signatory::AccountKey(dave_key)),
        true
    );
}

/// It double-checks that frozen keys are removed too.
#[test]
fn remove_frozen_signing_keys_test() {
    ExtBuilder::default()
        .build()
        .execute_with(&remove_frozen_signing_keys_with_externalities);
}

fn remove_frozen_signing_keys_with_externalities() {
    let (bob_key, charlie_key) = (
        AccountKey::from(AccountKeyring::Bob.public().0),
        AccountKey::from(AccountKeyring::Charlie.public().0),
    );

    let bob_signing_key = SigningItem::new(Signatory::AccountKey(bob_key), vec![Permission::Admin]);
    let charlie_signing_key = SigningItem::new(
        Signatory::AccountKey(charlie_key),
        vec![Permission::Operator],
    );

    // Add signing keys.
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());

    let signing_keys_v1 = vec![bob_signing_key, charlie_signing_key.clone()];
    assert_ok!(Identity::add_signing_items(
        alice.clone(),
        signing_keys_v1.clone()
    ));
    assert_ok!(Identity::authorize_join_to_identity(
        Origin::signed(AccountKeyring::Bob.public()),
        alice_did
    ));
    assert_ok!(Identity::authorize_join_to_identity(
        Origin::signed(AccountKeyring::Charlie.public()),
        alice_did
    ));

    // Freeze all signing keys
    assert_ok!(Identity::freeze_signing_keys(alice.clone()));

    // Remove Bob's key.
    assert_ok!(Identity::remove_signing_items(
        alice.clone(),
        vec![Signatory::AccountKey(bob_key)]
    ));
    // Check DidRecord.
    let did_rec = Identity::did_records(alice_did);
    assert_eq!(did_rec.signing_items, vec![charlie_signing_key]);
}

#[test]
fn enforce_uniqueness_keys_in_identity_tests() {
    ExtBuilder::default()
        .build()
        .execute_with(&enforce_uniqueness_keys_in_identity);
}

fn enforce_uniqueness_keys_in_identity() {
    // Register identities
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let _bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let bob = Origin::signed(AccountKeyring::Bob.public());

    // Check external signed key uniqueness.
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public().0);
    let charlie_sk = SigningItem::new(
        Signatory::AccountKey(charlie_key),
        vec![Permission::Operator],
    );
    assert_ok!(Identity::add_signing_items(
        alice.clone(),
        vec![charlie_sk.clone()]
    ));
    assert_ok!(Identity::authorize_join_to_identity(
        Origin::signed(AccountKeyring::Charlie.public()),
        alice_id
    ));

    assert_err!(
        Identity::add_signing_items(bob.clone(), vec![charlie_sk]),
        Error::<TestStorage>::AlreadyLinked
    );

    // Check non-external signed key non-uniqueness.
    let dave_key = AccountKey::from(AccountKeyring::Dave.public().0);
    let dave_sk = SigningItem {
        signer: Signatory::AccountKey(dave_key),
        signer_type: SignatoryType::MultiSig,
        permissions: vec![Permission::Operator],
    };
    assert_ok!(Identity::add_signing_items(
        alice.clone(),
        vec![dave_sk.clone()]
    ));
    assert_ok!(Identity::add_signing_items(bob.clone(), vec![dave_sk]));

    // Check that master key acts like external signed key.
    let bob_key = AccountKey::from(AccountKeyring::Bob.public().0);
    let bob_sk_as_mutisig = SigningItem {
        signer: Signatory::AccountKey(bob_key),
        signer_type: SignatoryType::MultiSig,
        permissions: vec![Permission::Operator],
    };
    assert_err!(
        Identity::add_signing_items(alice.clone(), vec![bob_sk_as_mutisig]),
        Error::<TestStorage>::AlreadyLinked
    );

    let bob_sk = SigningItem::new(Signatory::AccountKey(bob_key), vec![Permission::Admin]);
    assert_err!(
        Identity::add_signing_items(alice.clone(), vec![bob_sk]),
        Error::<TestStorage>::AlreadyLinked
    );
}

#[test]
fn add_remove_signing_identities() {
    ExtBuilder::default()
        .build()
        .execute_with(&add_remove_signing_identities_with_externalities);
}

fn add_remove_signing_identities_with_externalities() {
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let bob = Origin::signed(AccountKeyring::Bob.public());

    let charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let charlie = Origin::signed(AccountKeyring::Charlie.public());
    let dave_id = register_keyring_account(AccountKeyring::Dave).unwrap();

    assert_ok!(Identity::add_signing_items(
        alice.clone(),
        vec![SigningItem::from(bob_id), SigningItem::from(charlie_id)]
    ));
    assert_ok!(Identity::authorize_join_to_identity(bob, alice_id));
    assert_ok!(Identity::authorize_join_to_identity(charlie, alice_id));
    assert_eq!(
        Identity::is_signer_authorized(alice_id, &Signatory::Identity(bob_id)),
        true
    );

    assert_ok!(Identity::remove_signing_items(
        alice.clone(),
        vec![Signatory::Identity(bob_id), Signatory::Identity(dave_id)]
    ));

    let alice_rec = Identity::did_records(alice_id);
    assert_eq!(alice_rec.signing_items, vec![SigningItem::from(charlie_id)]);

    // Check is_authorized_identity
    assert_eq!(
        Identity::is_signer_authorized(alice_id, &Signatory::Identity(charlie_id)),
        true
    );
    assert_eq!(
        Identity::is_signer_authorized(alice_id, &Signatory::Identity(bob_id)),
        false
    );
}

#[test]
fn two_step_join_id() {
    ExtBuilder::default()
        .build()
        .execute_with(&two_step_join_id_with_ext);
}

fn two_step_join_id_with_ext() {
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let bob = Origin::signed(AccountKeyring::Bob.public());

    let c_sk = SigningItem::new(
        Signatory::AccountKey(AccountKey::from(AccountKeyring::Charlie.public().0)),
        vec![Permission::Operator],
    );
    let d_sk = SigningItem::new(
        Signatory::AccountKey(AccountKey::from(AccountKeyring::Dave.public().0)),
        vec![Permission::Full],
    );
    let e_sk = SigningItem::new(
        Signatory::AccountKey(AccountKey::from(AccountKeyring::Eve.public().0)),
        vec![Permission::Full],
    );

    // Check 1-to-1 relation between key and identity.
    let signing_keys = vec![c_sk.clone(), d_sk.clone(), e_sk.clone()];
    assert_ok!(Identity::add_signing_items(
        alice.clone(),
        signing_keys.clone()
    ));
    assert_ok!(Identity::add_signing_items(bob.clone(), signing_keys));
    assert_eq!(
        Identity::is_signer_authorized(alice_id, &c_sk.signer),
        false
    );

    let charlie = Origin::signed(AccountKeyring::Charlie.public());
    assert_ok!(Identity::authorize_join_to_identity(
        charlie.clone(),
        alice_id
    ));
    assert_eq!(Identity::is_signer_authorized(alice_id, &c_sk.signer), true);

    assert_err!(
        Identity::authorize_join_to_identity(charlie, bob_id),
        Error::<TestStorage>::AlreadyLinked
    );
    assert_eq!(Identity::is_signer_authorized(bob_id, &c_sk.signer), false);

    // Check after remove a signing key.
    let dave = Origin::signed(AccountKeyring::Dave.public());
    assert_ok!(Identity::authorize_join_to_identity(dave, alice_id));
    assert_eq!(Identity::is_signer_authorized(alice_id, &d_sk.signer), true);
    assert_ok!(Identity::remove_signing_items(
        alice.clone(),
        vec![d_sk.signer.clone()]
    ));
    assert_eq!(
        Identity::is_signer_authorized(alice_id, &d_sk.signer),
        false
    );

    // Check remove pre-authorization from master and itself.
    assert_err!(
        Identity::unauthorized_join_to_identity(alice.clone(), e_sk.signer.clone(), bob_id),
        Error::<TestStorage>::Unauthorized
    );
    assert_ok!(Identity::unauthorized_join_to_identity(
        alice,
        e_sk.signer.clone(),
        alice_id
    ));

    let eve = Origin::signed(AccountKeyring::Eve.public());
    assert_ok!(Identity::unauthorized_join_to_identity(
        eve,
        e_sk.signer,
        bob_id
    ));
}

#[test]
fn one_step_join_id() {
    ExtBuilder::default()
        .build()
        .execute_with(&one_step_join_id_with_ext);
}

fn one_step_join_id_with_ext() {
    let a_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let a_pub = AccountKeyring::Alice.public();
    let a = Origin::signed(a_pub.clone());
    let b_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let c_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let d_id = register_keyring_account(AccountKeyring::Dave).unwrap();

    let expires_at = 100u64;
    let authorization = TargetIdAuthorization {
        target_id: a_id.clone(),
        nonce: Identity::offchain_authorization_nonce(a_id),
        expires_at,
    };
    let auth_encoded = authorization.encode();

    let signatures = [
        AccountKeyring::Bob,
        AccountKeyring::Charlie,
        AccountKeyring::Dave,
    ]
    .iter()
    .map(|acc| H512::from(acc.sign(&auth_encoded)))
    .collect::<Vec<_>>();

    let signing_items_with_auth = vec![
        SigningItemWithAuth {
            signing_item: SigningItem::from(b_id.clone()),
            auth_signature: signatures[0].clone(),
        },
        SigningItemWithAuth {
            signing_item: SigningItem::from(c_id.clone()),
            auth_signature: signatures[1].clone(),
        },
        SigningItemWithAuth {
            signing_item: SigningItem::from(d_id.clone()),
            auth_signature: signatures[2].clone(),
        },
    ];

    assert_ok!(Identity::add_signing_items_with_authorization(
        a.clone(),
        expires_at,
        signing_items_with_auth[..2].to_owned()
    ));

    let signing_items = Identity::did_records(a_id).signing_items;
    assert_eq!(signing_items.iter().find(|si| **si == b_id).is_some(), true);
    assert_eq!(signing_items.iter().find(|si| **si == c_id).is_some(), true);

    // Check reply atack. Alice's nonce is different now.
    // NOTE: We need to force the increment of account's nonce manually.
    System::inc_account_nonce(&a_pub);

    assert_err!(
        Identity::add_signing_items_with_authorization(
            a.clone(),
            expires_at,
            signing_items_with_auth[2..].to_owned()
        ),
        Error::<TestStorage>::InvalidAuthorizationSignature
    );

    // Check revoke off-chain authorization.
    let e = Origin::signed(AccountKeyring::Eve.public());
    let e_id = register_keyring_account(AccountKeyring::Eve).unwrap();
    let eve_auth = TargetIdAuthorization {
        target_id: a_id.clone(),
        nonce: Identity::offchain_authorization_nonce(a_id),
        expires_at,
    };
    assert_ne!(authorization.nonce, eve_auth.nonce);

    let eve_signing_item_with_auth = SigningItemWithAuth {
        signing_item: SigningItem::from(e_id),
        auth_signature: H512::from(AccountKeyring::Eve.sign(eve_auth.encode().as_slice())),
    };

    assert_ok!(Identity::revoke_offchain_authorization(
        e,
        Signatory::Identity(e_id),
        eve_auth
    ));
    assert_err!(
        Identity::add_signing_items_with_authorization(
            a,
            expires_at,
            vec![eve_signing_item_with_auth]
        ),
        Error::<TestStorage>::AuthorizationHasBeenRevoked
    );

    // Check expire
    System::inc_account_nonce(&a_pub);
    Timestamp::set_timestamp(expires_at);

    let f = Origin::signed(AccountKeyring::Ferdie.public());
    let f_id = register_keyring_account(AccountKeyring::Ferdie).unwrap();
    let ferdie_auth = TargetIdAuthorization {
        target_id: a_id.clone(),
        nonce: Identity::offchain_authorization_nonce(a_id),
        expires_at,
    };
    let ferdie_signing_item_with_auth = SigningItemWithAuth {
        signing_item: SigningItem::from(f_id.clone()),
        auth_signature: H512::from(AccountKeyring::Eve.sign(ferdie_auth.encode().as_slice())),
    };

    assert_err!(
        Identity::add_signing_items_with_authorization(
            f,
            expires_at,
            vec![ferdie_signing_item_with_auth]
        ),
        Error::<TestStorage>::AuthorizationExpired
    );
}

#[test]
fn adding_authorizations() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = Signatory::from(register_keyring_account(AccountKeyring::Alice).unwrap());
        let bob_did = Signatory::from(register_keyring_account(AccountKeyring::Bob).unwrap());
        let ticker50 = Ticker::from(&[0x50][..]);
        let mut auth_id = Identity::add_auth(
            alice_did,
            bob_did,
            AuthorizationData::TransferTicker(ticker50),
            None,
        );
        assert_eq!(
            <identity::AuthorizationsGiven>::get(alice_did, auth_id),
            bob_did
        );
        let mut auth = Identity::get_authorization(bob_did, auth_id);
        assert_eq!(auth.authorized_by, alice_did);
        assert_eq!(auth.expiry, None);
        assert_eq!(
            auth.authorization_data,
            AuthorizationData::TransferTicker(ticker50)
        );
        auth_id = Identity::add_auth(
            alice_did,
            bob_did,
            AuthorizationData::TransferTicker(ticker50),
            Some(100),
        );
        assert_eq!(
            <identity::AuthorizationsGiven>::get(alice_did, auth_id),
            bob_did
        );
        auth = Identity::get_authorization(bob_did, auth_id);
        assert_eq!(auth.authorized_by, alice_did);
        assert_eq!(auth.expiry, Some(100));
        assert_eq!(
            auth.authorization_data,
            AuthorizationData::TransferTicker(ticker50)
        );
    });
}

#[test]
fn removing_authorizations() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = Signatory::from(register_keyring_account(AccountKeyring::Alice).unwrap());
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob_did = Signatory::from(register_keyring_account(AccountKeyring::Bob).unwrap());
        let ticker50 = Ticker::from(&[0x50][..]);
        let auth_id = Identity::add_auth(
            alice_did,
            bob_did,
            AuthorizationData::TransferTicker(ticker50),
            None,
        );
        assert_eq!(
            <identity::AuthorizationsGiven>::get(alice_did, auth_id),
            bob_did
        );
        let auth = Identity::get_authorization(bob_did, auth_id);
        assert_eq!(
            auth.authorization_data,
            AuthorizationData::TransferTicker(ticker50)
        );
        assert_ok!(Identity::remove_authorization(
            alice.clone(),
            bob_did,
            auth_id
        ));
        assert!(!<identity::AuthorizationsGiven>::exists(alice_did, auth_id));
        assert!(!<identity::Authorizations<TestStorage>>::exists(
            bob_did, auth_id
        ));
    });
}

#[test]
fn adding_links() {
    ExtBuilder::default().build().execute_with(|| {
        let bob_did = Signatory::from(register_keyring_account(AccountKeyring::Bob).unwrap());
        let ticker50 = Ticker::from(&[0x50][..]);
        let ticker51 = Ticker::from(&[0x51][..]);
        let mut link_id = Identity::add_link(bob_did, LinkData::TickerOwned(ticker50), None);
        let mut link = Identity::get_link(bob_did, link_id);
        assert_eq!(link.expiry, None);
        assert_eq!(link.link_data, LinkData::TickerOwned(ticker50));
        link_id = Identity::add_link(bob_did, LinkData::TickerOwned(ticker51), None);
        link = Identity::get_link(bob_did, link_id);
        assert_eq!(link.expiry, None);
        assert_eq!(link.link_data, LinkData::TickerOwned(ticker51));
        link_id = Identity::add_link(bob_did, LinkData::TickerOwned(ticker50), Some(100));
        link = Identity::get_link(bob_did, link_id);
        assert_eq!(link.expiry, Some(100));
        assert_eq!(link.link_data, LinkData::TickerOwned(ticker50));
        link_id = Identity::add_link(bob_did, LinkData::TickerOwned(ticker50), Some(100));
        link = Identity::get_link(bob_did, link_id);
        assert_eq!(link.expiry, Some(100));
        assert_eq!(link.link_data, LinkData::TickerOwned(ticker50));
    });
}

#[test]
fn removing_links() {
    ExtBuilder::default().build().execute_with(|| {
        let bob_did = Signatory::from(register_keyring_account(AccountKeyring::Bob).unwrap());
        let ticker50 = Ticker::from(&[0x50][..]);
        let link_id = Identity::add_link(bob_did, LinkData::TickerOwned(ticker50), None);
        let link = Identity::get_link(bob_did, link_id);
        assert_eq!(link.link_data, LinkData::TickerOwned(ticker50));
        Identity::remove_link(bob_did, link_id);
        let removed_link = Identity::get_link(bob_did, link_id);
        assert_eq!(removed_link.link_data, LinkData::NoData);
    });
}

#[test]
fn changing_master_key() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .build()
        .execute_with(|| changing_master_key_we());
}

fn changing_master_key_we() {
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice_key = AccountKey::from(AccountKeyring::Alice.public().0);

    let _target_did = register_keyring_account(AccountKeyring::Bob).unwrap();
    let new_key = AccountKey::from(AccountKeyring::Bob.public().0);
    let new_key_origin = Origin::signed(AccountKeyring::Bob.public());

    let cdd_acc = AccountKey::from(AccountKeyring::Eve.public().0);
    let cdd_did = Identity::get_identity(&cdd_acc).unwrap();

    // Master key matches Alice's key
    assert_eq!(Identity::did_records(alice_did).master_key, alice_key);

    // Alice triggers change of master key
    let owner_auth_id = Identity::add_auth(
        Signatory::AccountKey(alice_key),
        Signatory::AccountKey(new_key),
        AuthorizationData::RotateMasterKey(alice_did),
        None,
    );

    let cdd_auth_id = Identity::add_auth(
        Signatory::Identity(cdd_did),
        Signatory::AccountKey(new_key),
        AuthorizationData::AttestMasterKeyRotation(alice_did),
        None,
    );

    // Accept the authorization with the new key
    assert_ok!(Identity::accept_master_key(
        new_key_origin.clone(),
        owner_auth_id.clone(),
        cdd_auth_id.clone()
    ));

    // Alice's master key is now Bob's
    assert_eq!(
        Identity::did_records(alice_did).master_key,
        AccountKey::from(AccountKeyring::Bob.public().0)
    );
}

#[test]
fn cdd_register_did_test() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .cdd_providers(vec![
            AccountKeyring::Eve.public(),
            AccountKeyring::Ferdie.public(),
        ])
        .build()
        .execute_with(|| cdd_register_did_test_we());
}

fn cdd_register_did_test_we() {
    let cdd_1_acc = AccountKeyring::Eve.public();
    let cdd_1_key = AccountKey::try_from(cdd_1_acc.0).unwrap();
    let cdd_2_acc = AccountKeyring::Ferdie.public();
    let cdd_2_key = AccountKey::try_from(cdd_2_acc.0).unwrap();
    let non_id = Origin::signed(AccountKeyring::Charlie.public());

    let alice_acc = AccountKeyring::Alice.public();
    let alice_key = AccountKey::try_from(alice_acc.0).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let bob_key = AccountKey::try_from(bob_acc.0).unwrap();

    // CDD 1 registers correctly the Alice's ID.
    assert_ok!(Identity::cdd_register_did(
        Origin::signed(cdd_1_acc),
        alice_acc,
        10,
        vec![]
    ));

    // Check that Alice's ID is attested by CDD 1.
    let alice_id = Identity::get_identity(&alice_key).unwrap();
    let _cdd_1_id = Identity::get_identity(&cdd_1_key).unwrap();
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Error case: Try account without ID.
    assert!(Identity::cdd_register_did(non_id, bob_acc, 10, vec![]).is_err(),);
    // Error case: Try account with ID but it is not part of CDD providers.
    assert!(Identity::cdd_register_did(Origin::signed(alice_acc), bob_acc, 10, vec![]).is_err());

    // CDD 2 registers properly Bob's ID.
    assert_ok!(Identity::cdd_register_did(
        Origin::signed(cdd_2_acc),
        bob_acc,
        10,
        vec![]
    ));
    let bob_id = Identity::get_identity(&bob_key).unwrap();
    let _cdd_2_id = Identity::get_identity(&cdd_2_key).unwrap();
    assert_eq!(Identity::has_valid_cdd(bob_id), true);
}
