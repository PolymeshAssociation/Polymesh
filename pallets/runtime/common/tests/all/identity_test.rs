use super::{
    ext_builder::PROTOCOL_OP_BASE_FEE,
    storage::{
        add_signing_item, authorizations_to, get_identity_id, register_keyring_account,
        register_keyring_account_with_balance, GovernanceCommittee, TestStorage,
    },
    ExtBuilder,
};

use pallet_identity_rpc_runtime_api::LinkType;
use polymesh_common_utilities::{
    traits::{
        group::GroupTrait,
        identity::{
            LinkedKeyInfo, SigningItemWithAuth, TargetIdAuthorization, Trait as IdentityTrait,
        },
    },
    SystematicIssuers,
};
use polymesh_primitives::{
    AccountKey, AuthorizationData, AuthorizationError, Claim, ClaimType, Document, IdentityClaim,
    IdentityId, JoinIdentityData, LinkData, Permission, Scope, Signatory, SignatoryType,
    SigningItem, Ticker, TransactionError,
};
use polymesh_runtime_develop::{fee_details::CddHandler, runtime::Call};

use pallet_balances as balances;
use pallet_identity::{self as identity, BatchAddClaimItem, BatchRevokeClaimItem, Error};
use pallet_transaction_payment::CddAndFeeDetails;

use codec::Encode;
use frame_support::{assert_err, assert_ok, traits::Currency, StorageDoubleMap};
use sp_core::H512;
use sp_runtime::transaction_validity::InvalidTransaction;
use test_client::AccountKeyring;

use std::convert::{From, TryFrom};

type Balances = balances::Module<TestStorage>;
// type BalancesCall = <balances::Module<TestStorage> as BTrait>::Call;

type Identity = identity::Module<TestStorage>;
type MultiSig = pallet_multisig::Module<TestStorage>;
type System = frame_system::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;

type Origin = <TestStorage as frame_system::Trait>::Origin;
type CddServiceProviders = <TestStorage as IdentityTrait>::CddServiceProviders;

// Identity Test Helper functions
// =======================================

/// Utility function to fetch *only* systematic CDD claims.
///
/// We have 2 systematic CDD claims issuers:
/// * Governance Committee group.
/// * CDD providers group.
fn fetch_systematic_cdd(target: IdentityId) -> Option<IdentityClaim> {
    let claim_type = ClaimType::CustomerDueDiligence;
    let gc_id = SystematicIssuers::Committee.as_id();

    Identity::fetch_claim(target, claim_type, gc_id, None).or_else(|| {
        let cdd_id = SystematicIssuers::CDDProvider.as_id();
        Identity::fetch_claim(target, claim_type, cdd_id, None)
    })
}

// Tests
// =======================================

#[test]
fn add_claims_batch_test() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .cdd_providers(vec![
            AccountKeyring::Eve.public(),
            AccountKeyring::Ferdie.public(),
        ])
        .build()
        .execute_with(|| add_claims_batch());
}

fn add_claims_batch() {
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let bob_issuer = AccountKeyring::Bob.public();
    let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
    let cdd_claim_issuer = AccountKeyring::Eve.public();
    let cdd_claim_did = get_identity_id(AccountKeyring::Eve).unwrap();

    let scope = Scope::from(0);

    let claim_records = vec![
        BatchAddClaimItem {
            target: alice_did,
            claim: Claim::CustomerDueDiligence,
            expiry: None,
        },
        BatchAddClaimItem {
            target: alice_did,
            claim: Claim::Affiliate(scope),
            expiry: None,
        },
    ];

    assert_ok!(Identity::batch_add_claim(
        Origin::signed(cdd_claim_issuer),
        claim_records.clone(),
    ));

    // Using Bob as the singer who is not a CDD Provider to check if the transaction fails
    assert_err!(
        Identity::batch_add_claim(Origin::signed(bob_issuer), claim_records.clone(),),
        Error::<TestStorage>::UnAuthorizedCddProvider
    );

    let claim1 = Identity::fetch_claim(
        alice_did,
        ClaimType::CustomerDueDiligence,
        cdd_claim_did,
        None,
    )
    .unwrap();

    let claim2 =
        Identity::fetch_claim(alice_did, ClaimType::Affiliate, cdd_claim_did, Some(scope)).unwrap();

    assert_eq!(claim1.expiry, None);
    assert_eq!(claim2.expiry, None);

    assert_eq!(claim1.claim, Claim::CustomerDueDiligence);
    assert_eq!(claim2.claim, Claim::Affiliate(scope));
}

/// TODO Add `Signatory::Identity(..)` test.
#[test]
fn only_master_or_signing_keys_can_authenticate_as_an_identity() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let owner_signer =
            Signatory::AccountKey(AccountKey::from(AccountKeyring::Alice.public().0));

        let a_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let a = Origin::signed(AccountKeyring::Bob.public());
        let b_did = register_keyring_account(AccountKeyring::Dave).unwrap();

        let charlie_key = AccountKey::from(AccountKeyring::Charlie.public().0);
        let charlie_signer = Signatory::AccountKey(charlie_key);

        assert_ok!(Balances::top_up_identity_balance(
            a.clone(),
            a_did,
            PROTOCOL_OP_BASE_FEE
        ));
        add_signing_item(a_did, charlie_signer);

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
        let scope = Scope::from(0);

        assert_ok!(Identity::add_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            Claim::Accredited(scope),
            Some(100u64),
        ));
        assert!(Identity::fetch_claim(
            claim_issuer_did,
            ClaimType::Accredited,
            claim_issuer_did,
            Some(scope)
        )
        .is_some());

        assert_ok!(Identity::revoke_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            Claim::Accredited(scope),
        ));
        assert!(Identity::fetch_claim(
            claim_issuer_did,
            ClaimType::Accredited,
            claim_issuer_did,
            Some(scope)
        )
        .is_none());
    });
}

#[test]
fn revoking_batch_claims() {
    ExtBuilder::default().build().execute_with(|| {
        let _owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let _issuer_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let _issuer = Origin::signed(AccountKeyring::Bob.public());
        let claim_issuer_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let claim_issuer = Origin::signed(AccountKeyring::Charlie.public());
        let scope = Scope::from(0);

        assert_ok!(Identity::add_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            Claim::Accredited(scope),
            Some(100u64),
        ));

        assert_ok!(Identity::add_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            Claim::NoData,
            None,
        ));
        assert!(Identity::fetch_claim(
            claim_issuer_did,
            ClaimType::Accredited,
            claim_issuer_did,
            Some(scope)
        )
        .is_some());

        assert!(
            Identity::fetch_claim(claim_issuer_did, ClaimType::NoType, claim_issuer_did, None,)
                .is_some()
        );

        assert!(Identity::fetch_claim(
            claim_issuer_did,
            ClaimType::Accredited,
            claim_issuer_did,
            Some(scope),
        )
        .is_some());

        assert_ok!(Identity::batch_revoke_claim(
            claim_issuer.clone(),
            vec![
                BatchRevokeClaimItem {
                    target: claim_issuer_did,
                    claim: Claim::Accredited(scope),
                },
                BatchRevokeClaimItem {
                    target: claim_issuer_did,
                    claim: Claim::NoData,
                }
            ]
        ));
        assert!(Identity::fetch_claim(
            claim_issuer_did,
            ClaimType::Accredited,
            claim_issuer_did,
            Some(scope)
        )
        .is_none());

        assert!(
            Identity::fetch_claim(claim_issuer_did, ClaimType::NoType, claim_issuer_did, None)
                .is_none()
        );

        assert!(Identity::fetch_claim(
            claim_issuer_did,
            ClaimType::Accredited,
            claim_issuer_did,
            Some(scope),
        )
        .is_none());
    });
}

#[test]
fn only_master_key_can_add_signing_key_permissions() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&only_master_key_can_add_signing_key_permissions_with_externalities);
}
fn only_master_key_can_add_signing_key_permissions_with_externalities() {
    let bob_key = AccountKey::from(AccountKeyring::Bob.public().0);
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public().0);
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        PROTOCOL_OP_BASE_FEE * 2
    ));
    add_signing_item(alice_did, Signatory::from(charlie_key));
    add_signing_item(alice_did, Signatory::from(bob_key));

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

/// It verifies that frozen keys are recovered after `unfreeze` call.
#[test]
fn freeze_signing_keys_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&freeze_signing_keys_with_externalities);
}

fn freeze_signing_keys_with_externalities() {
    let (bob_key, charlie_key, dave_key) = (
        AccountKey::from(AccountKeyring::Bob.public().0),
        AccountKey::from(AccountKeyring::Charlie.public().0),
        AccountKey::from(AccountKeyring::Dave.public().0),
    );
    // Add signing keys.
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        PROTOCOL_OP_BASE_FEE * 2
    ));
    add_signing_item(alice_did, Signatory::from(bob_key));
    add_signing_item(alice_did, Signatory::from(charlie_key));

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
    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        PROTOCOL_OP_BASE_FEE
    ));
    add_signing_item(alice_did, Signatory::from(dave_key));

    // update permission of frozen keys.
    assert_ok!(Identity::set_permission_to_signer(
        alice.clone(),
        Signatory::AccountKey(bob_key),
        vec![Permission::Operator]
    ));

    // unfreeze all
    // commenting this because `default_identity` feature is not allowing to access None identity.
    // assert_err!(
    //     Identity::unfreeze_signing_keys(bob.clone()),
    //     DispatchError::Other("Current identity is none and key is not linked to any identity")
    // );
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
        .monied(true)
        .build()
        .execute_with(&remove_frozen_signing_keys_with_externalities);
}

fn remove_frozen_signing_keys_with_externalities() {
    let (bob_key, charlie_key) = (
        AccountKey::from(AccountKeyring::Bob.public().0),
        AccountKey::from(AccountKeyring::Charlie.public().0),
    );

    let charlie_signing_key = SigningItem::new(Signatory::AccountKey(charlie_key), vec![]);

    // Add signing keys.
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        PROTOCOL_OP_BASE_FEE
    ));
    add_signing_item(alice_did, Signatory::from(bob_key));
    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        PROTOCOL_OP_BASE_FEE
    ));
    add_signing_item(alice_did, Signatory::from(charlie_key));

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

/// It double-checks that frozen keys are removed too.
#[test]
fn frozen_signing_keys_cdd_verification_test() {
    ExtBuilder::default()
        .build()
        .execute_with(&frozen_signing_keys_cdd_verification_test_we);
}

fn frozen_signing_keys_cdd_verification_test_we() {
    // 0. Create identity for Alice and signing key from Bob.
    let alice = AccountKeyring::Alice.public();
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let charlie = AccountKeyring::Charlie.public();
    let _charlie_id = register_keyring_account_with_balance(AccountKeyring::Charlie, 100).unwrap();
    let bob = AccountKeyring::Bob.public();
    assert_eq!(Balances::free_balance(charlie), 59);

    // 1. Add Bob as signatory to Alice ID.
    let bob_signatory = Signatory::from(AccountKey::from(AccountKeyring::Bob.public().0));
    assert_ok!(Balances::top_up_identity_balance(
        Origin::signed(alice),
        alice_id,
        100_000
    ));
    add_signing_item(alice_id, bob_signatory);
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice),
        bob,
        25_000,
        None
    ));
    assert_eq!(Balances::free_balance(bob), 25_000);

    // 2. Bob can transfer some funds to Charlie ID.
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(bob),
        charlie,
        1_000,
        None
    ));
    assert_eq!(Balances::free_balance(charlie), 1_059);

    // 3. Alice freezes her signing keys.
    assert_ok!(Identity::freeze_signing_keys(Origin::signed(alice)));

    // 4. Bob should NOT transfer any amount. SE is simulated.
    // Balances::transfer_with_memo(Origin::signed(bob), charlie, 1_000, None),
    let payer = CddHandler::get_valid_payer(
        &Call::Balances(balances::Call::transfer_with_memo(
            AccountKeyring::Charlie.to_account_id().into(),
            1_000,
            None,
        )),
        &Signatory::from(AccountKey::from(bob.0)),
    );
    assert_err!(
        payer,
        InvalidTransaction::Custom(TransactionError::MissingIdentity as u8)
    );

    assert_eq!(Balances::free_balance(charlie), 1_059);

    // 5. Alice still can make transfers.
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice),
        charlie,
        1_000,
        None
    ));
    assert_eq!(Balances::free_balance(charlie), 2_059);

    // 6. Unfreeze signatory keys, and Bob should be able to transfer again.
    assert_ok!(Identity::unfreeze_signing_keys(Origin::signed(alice)));
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(bob),
        charlie,
        1_000,
        None
    ));
    assert_eq!(Balances::free_balance(charlie), 3_059);
}

#[test]
fn remove_signing_keys_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&remove_signing_keys_test_with_externalities);
}

fn remove_signing_keys_test_with_externalities() {
    let bob_key = AccountKey::from(AccountKeyring::Bob.public().0);

    let bob_signing_key = SigningItem::new(Signatory::AccountKey(bob_key), vec![]);

    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());
    let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();

    let dave_key = AccountKey::from(AccountKeyring::Dave.public().0);

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        PROTOCOL_OP_BASE_FEE
    ));

    let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    let musig_account_key = AccountKey::from(musig_address.clone());

    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(alice_did), Signatory::from(dave_key)],
        1,
    ));
    let auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(dave_key))
        .next()
        .unwrap()
        .auth_id;
    assert_ok!(MultiSig::unsafe_accept_multisig_signer(
        Signatory::from(dave_key),
        auth_id
    ));

    add_signing_item(alice_did, Signatory::from(bob_key));
    add_signing_item(alice_did, Signatory::from(musig_account_key));

    // Fund the multisig
    assert_ok!(Balances::transfer(alice.clone(), musig_address.clone(), 1));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&musig_account_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&bob_key), Some(alice_did));

    // Try removing bob using charlie
    assert_ok!(Identity::remove_signing_items(
        charlie.clone(),
        vec![Signatory::AccountKey(bob_key)]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&musig_account_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&bob_key), Some(alice_did));

    // Try remove bob using alice
    assert_ok!(Identity::remove_signing_items(
        alice.clone(),
        vec![Signatory::AccountKey(bob_key)]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&musig_account_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Try removing multisig while it has funds
    assert_ok!(Identity::remove_signing_items(
        alice.clone(),
        vec![Signatory::AccountKey(musig_account_key)]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&musig_account_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(musig_address.clone(), Signatory::from(dave_key)),
        true
    );

    // Empty multisig's funds and remove as signer
    assert_ok!(Balances::top_up_identity_balance(
        Origin::signed(musig_address.clone()),
        alice_did,
        1
    ));
    assert_ok!(Identity::remove_signing_items(
        alice.clone(),
        vec![Signatory::AccountKey(musig_account_key)]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_account_key), None);
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(musig_address.clone(), Signatory::from(dave_key)),
        true
    );
}

#[test]
fn leave_identity_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&leave_identity_test_with_externalities);
}

fn leave_identity_test_with_externalities() {
    let bob_key = AccountKey::from(AccountKeyring::Bob.public().0);
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let charlie = Origin::signed(AccountKeyring::Charlie.public());

    let bob_signing_key = SigningItem::new(Signatory::AccountKey(bob_key), vec![]);
    let charlie_signing_key = SigningItem::new(Signatory::Identity(charlie_did), vec![]);
    let alice_signing_items = vec![bob_signing_key, charlie_signing_key.clone()];

    let dave_key = AccountKey::from(AccountKeyring::Dave.public().0);

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        2 * PROTOCOL_OP_BASE_FEE
    ));

    let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    let musig_account_key = AccountKey::from(musig_address.clone());

    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(alice_did), Signatory::from(dave_key)],
        1,
    ));
    let auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(dave_key))
        .next()
        .unwrap()
        .auth_id;
    assert_ok!(MultiSig::unsafe_accept_multisig_signer(
        Signatory::from(dave_key),
        auth_id
    ));

    add_signing_item(alice_did, Signatory::from(bob_key));
    add_signing_item(alice_did, Signatory::from(charlie_did));

    // Fund the multisig
    assert_ok!(Balances::transfer(alice.clone(), musig_address.clone(), 1));

    // Check DidRecord.
    let did_rec = Identity::did_records(alice_did);
    assert_eq!(did_rec.signing_items, alice_signing_items);
    assert_eq!(Identity::get_identity(&bob_key), Some(alice_did));

    // Bob leaves
    assert_ok!(Identity::leave_identity_as_key(bob));

    // Check DidRecord.
    let did_rec = Identity::did_records(alice_did);
    assert_eq!(did_rec.signing_items, vec![charlie_signing_key]);
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Charlie leaves
    assert_ok!(Identity::leave_identity_as_identity(charlie, alice_did));

    // Check DidRecord.
    let did_rec = Identity::did_records(alice_did);
    assert_eq!(did_rec.signing_items.len(), 0);
    assert_eq!(Identity::get_identity(&bob_key), None);

    add_signing_item(alice_did, Signatory::from(musig_account_key));
    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&musig_account_key), Some(alice_did));

    // multisig tries leaving identity while it has funds
    assert_err!(
        Identity::leave_identity_as_key(Origin::signed(musig_address.clone())),
        Error::<TestStorage>::MultiSigHasBalance
    );

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), Some(alice_did));
    assert_eq!(Identity::get_identity(&musig_account_key), Some(alice_did));

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(musig_address.clone(), Signatory::from(dave_key)),
        true
    );

    // Empty multisig's funds and remove as signer
    assert_ok!(Balances::top_up_identity_balance(
        Origin::signed(musig_address.clone()),
        alice_did,
        1
    ));
    assert_ok!(Identity::leave_identity_as_key(Origin::signed(
        musig_address.clone()
    )));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_account_key), None);

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(musig_address.clone(), Signatory::from(dave_key)),
        true
    );
}

#[test]
fn enforce_uniqueness_keys_in_identity_tests() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&enforce_uniqueness_keys_in_identity);
}

fn enforce_uniqueness_keys_in_identity() {
    // Register identities
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let _bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();

    // Check external signed key uniqueness.
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public().0);
    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_id,
        PROTOCOL_OP_BASE_FEE
    ));
    add_signing_item(alice_id, Signatory::from(charlie_key));
    let auth_id = Identity::add_auth(
        Signatory::from(AccountKey::from(AccountKeyring::Alice.public().0)),
        Signatory::from(AccountKey::from(AccountKeyring::Bob.public().0)),
        AuthorizationData::JoinIdentity(JoinIdentityData::new(alice_id, vec![])),
        None,
    );
    assert_err!(
        Identity::join_identity(
            Signatory::from(AccountKey::from(AccountKeyring::Bob.public().0)),
            auth_id
        ),
        Error::<TestStorage>::AlreadyLinked
    );
}

#[test]
fn add_remove_signing_identities() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&add_remove_signing_identities_with_externalities);
}

fn add_remove_signing_identities_with_externalities() {
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_id,
        PROTOCOL_OP_BASE_FEE * 2
    ));
    add_signing_item(alice_id, Signatory::from(bob_id));
    add_signing_item(alice_id, Signatory::from(charlie_id));

    assert_ok!(Identity::remove_signing_items(
        alice.clone(),
        vec![Signatory::Identity(bob_id)]
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

    assert_ok!(Identity::batch_add_signing_item_with_authorization(
        a.clone(),
        signing_items_with_auth[..2].to_owned(),
        expires_at
    ));

    let signing_items = Identity::did_records(a_id).signing_items;
    assert_eq!(signing_items.iter().find(|si| **si == b_id).is_some(), true);
    assert_eq!(signing_items.iter().find(|si| **si == c_id).is_some(), true);

    // Check reply attack. Alice's nonce is different now.
    // NOTE: We need to force the increment of account's nonce manually.
    System::inc_account_nonce(&a_pub);

    assert_err!(
        Identity::batch_add_signing_item_with_authorization(
            a.clone(),
            signing_items_with_auth[2..].to_owned(),
            expires_at
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
        Identity::batch_add_signing_item_with_authorization(
            a,
            vec![eve_signing_item_with_auth],
            expires_at
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
        Identity::batch_add_signing_item_with_authorization(
            f,
            vec![ferdie_signing_item_with_auth],
            expires_at
        ),
        Error::<TestStorage>::AuthorizationExpired
    );
}

#[test]
fn adding_authorizations() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = Signatory::from(register_keyring_account(AccountKeyring::Alice).unwrap());
        let bob_did = Signatory::from(register_keyring_account(AccountKeyring::Bob).unwrap());
        let ticker50 = Ticker::try_from(&[0x50][..]).unwrap();
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
        let ticker50 = Ticker::try_from(&[0x50][..]).unwrap();
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
        assert!(!<identity::AuthorizationsGiven>::contains_key(
            alice_did, auth_id
        ));
        assert!(!<identity::Authorizations<TestStorage>>::contains_key(
            bob_did, auth_id
        ));
    });
}

#[test]
fn adding_links() {
    ExtBuilder::default().build().execute_with(|| {
        let bob_did = Signatory::from(register_keyring_account(AccountKeyring::Bob).unwrap());
        let ticker50 = Ticker::try_from(&[0x50][..]).unwrap();
        let ticker51 = Ticker::try_from(&[0x51][..]).unwrap();
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

        // Testing the list of filtered links
        Timestamp::set_timestamp(120);

        // Getting expired and non-expired both
        let mut links =
            Identity::get_filtered_links(bob_did, true, Some(LinkType::TickerOwnership));
        assert_eq!(links.len(), 4);
        links = Identity::get_filtered_links(bob_did, false, Some(LinkType::TickerOwnership));
        // Two links are expired
        assert_eq!(links.len(), 2);
        // Add other type of link
        // 1.1 : Add document type
        let doc = Document {
            name: b"D".into(),
            uri: b"www.d.com".into(),
            content_hash: b"0x4".into(),
        };

        Identity::add_link(bob_did, LinkData::DocumentOwned(doc.clone()), None);
        // 1.2 : Add AssetOwned type
        Identity::add_link(bob_did, LinkData::AssetOwned(ticker51), None);
        Identity::add_link(bob_did, LinkData::AssetOwned(ticker50), Some(200));

        // Query DocumentOwnership type link
        links = Identity::get_filtered_links(bob_did, true, Some(LinkType::DocumentOwnership));
        assert_eq!(links.len(), 1);

        // Query AssetOwnership type
        links = Identity::get_filtered_links(bob_did, true, Some(LinkType::AssetOwnership));
        assert_eq!(links.len(), 2);

        // Increase time
        Timestamp::set_timestamp(220);
        links = Identity::get_filtered_links(bob_did, false, Some(LinkType::AssetOwnership));
        assert_eq!(links.len(), 1);

        // Query all links without providing link type and allow expired ones as well
        links = Identity::get_filtered_links(bob_did, true, None);
        assert_eq!(links.len(), 7);

        // Query all links without providing link type and not allow the expired ones
        links = Identity::get_filtered_links(bob_did, false, None);
        assert_eq!(links.len(), 4);
    });
}

#[test]
fn removing_links() {
    ExtBuilder::default().build().execute_with(|| {
        let bob_did = Signatory::from(register_keyring_account(AccountKeyring::Bob).unwrap());
        let ticker50 = Ticker::try_from(&[0x50][..]).unwrap();
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

    // Master key matches Alice's key
    assert_eq!(Identity::did_records(alice_did).master_key, alice_key);

    // Alice triggers change of master key
    let owner_auth_id = Identity::add_auth(
        Signatory::AccountKey(alice_key),
        Signatory::AccountKey(new_key),
        AuthorizationData::RotateMasterKey(alice_did),
        None,
    );

    // Accept the authorization with the new key
    assert_ok!(Identity::accept_master_key(
        new_key_origin.clone(),
        owner_auth_id.clone(),
        None
    ));

    // Alice's master key is now Bob's
    assert_eq!(
        Identity::did_records(alice_did).master_key,
        AccountKey::from(AccountKeyring::Bob.public().0)
    );
}

#[test]
fn changing_master_key_with_cdd_auth() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .build()
        .execute_with(|| changing_master_key_with_cdd_auth_we());
}

fn changing_master_key_with_cdd_auth_we() {
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice_key = AccountKey::from(AccountKeyring::Alice.public().0);

    let _target_did = register_keyring_account(AccountKeyring::Bob).unwrap();
    let new_key = AccountKey::from(AccountKeyring::Bob.public().0);
    let new_key_origin = Origin::signed(AccountKeyring::Bob.public());

    let cdd_did = get_identity_id(AccountKeyring::Eve).unwrap();

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

    assert_ok!(Identity::change_cdd_requirement_for_mk_rotation(
        frame_system::RawOrigin::Root.into(),
        true
    ));

    assert!(
        Identity::accept_master_key(new_key_origin.clone(), owner_auth_id.clone(), None).is_err()
    );

    let owner_auth_id2 = Identity::add_auth(
        Signatory::AccountKey(alice_key),
        Signatory::AccountKey(new_key),
        AuthorizationData::RotateMasterKey(alice_did),
        None,
    );

    // Accept the authorization with the new key
    assert_ok!(Identity::accept_master_key(
        new_key_origin.clone(),
        owner_auth_id2,
        Some(cdd_auth_id)
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
    let cdd1 = Origin::signed(AccountKeyring::Eve.public());
    let cdd2 = Origin::signed(AccountKeyring::Ferdie.public());
    let non_id = Origin::signed(AccountKeyring::Charlie.public());

    let alice = AccountKeyring::Alice.public();
    let bob_acc = AccountKeyring::Bob.public();

    // CDD 1 registers correctly the Alice's ID.
    assert_ok!(Identity::cdd_register_did(
        cdd1.clone(),
        alice,
        Some(10),
        vec![]
    ));

    // Check that Alice's ID is attested by CDD 1.
    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Error case: Try account without ID.
    assert!(Identity::cdd_register_did(non_id, bob_acc, Some(10), vec![]).is_err(),);
    // Error case: Try account with ID but it is not part of CDD providers.
    assert!(Identity::cdd_register_did(Origin::signed(alice), bob_acc, Some(10), vec![]).is_err());

    // CDD 2 registers properly Bob's ID.
    assert_ok!(Identity::cdd_register_did(cdd2, bob_acc, Some(10), vec![]));
    let bob_id = get_identity_id(AccountKeyring::Bob).unwrap();
    assert_eq!(Identity::has_valid_cdd(bob_id), true);

    // Register with signing_keys
    // ==============================================
    // Register Charlie with signing keys.
    let charlie = AccountKeyring::Charlie.public();
    let dave = AccountKeyring::Dave.public();
    let dave_si = SigningItem::from(AccountKey::from(dave.clone()));
    let alice_si = SigningItem::from(alice_id);
    let signing_keys = vec![dave_si.clone(), alice_si.clone()];
    assert_ok!(Identity::cdd_register_did(
        cdd1.clone(),
        charlie,
        Some(10),
        signing_keys
    ));

    Balances::make_free_balance_be(&charlie, 10_000_000_000);
    let charlie_id = get_identity_id(AccountKeyring::Charlie).unwrap();
    assert_eq!(Identity::has_valid_cdd(charlie_id), true);
    assert_eq!(
        Identity::did_records(charlie_id).signing_items.is_empty(),
        true
    );
    assert_ok!(Balances::top_up_identity_balance(
        Origin::signed(charlie),
        charlie_id,
        10_000_000
    ));

    // Dave authorizes to be joined to Charlie.
    let dave_auth_list = authorizations_to(&dave_si.signer);
    let dave_auth_id = dave_auth_list
        .iter()
        .map(|auth| auth.auth_id)
        .next()
        .unwrap();

    assert_ok!(Identity::accept_authorization(
        Origin::signed(dave),
        dave_auth_id
    ));
    assert_eq!(
        Identity::did_records(charlie_id).signing_items,
        vec![dave_si.clone()]
    );

    let alice_auth_list = authorizations_to(&alice_si.signer);
    let alice_auth_id = alice_auth_list
        .iter()
        .map(|auth| auth.auth_id)
        .next()
        .unwrap();

    assert_ok!(Identity::accept_authorization(
        Origin::signed(alice),
        alice_auth_id
    ));
    assert_eq!(
        Identity::did_records(charlie_id).signing_items,
        vec![dave_si, alice_si]
    );
}

#[test]
fn add_identity_signers() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let charlie = Origin::signed(AccountKeyring::Charlie.public());
        let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let alice_identity_signer = Signatory::from(alice_did);
        let alice_acc_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Alice.public().encode()).unwrap());
        let bob_identity_signer = Signatory::from(bob_did);
        let charlie_acc_signer = Signatory::from(
            AccountKey::try_from(AccountKeyring::Charlie.public().encode()).unwrap(),
        );
        let dave_acc_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Dave.public().encode()).unwrap());

        let auth_id_for_id_to_id = Identity::add_auth(
            alice_identity_signer,
            bob_identity_signer,
            AuthorizationData::JoinIdentity(JoinIdentityData::new(alice_did, vec![])),
            None,
        );

        assert_err!(
            Identity::join_identity(bob_identity_signer, auth_id_for_id_to_id),
            AuthorizationError::Unauthorized
        );

        let auth_id_for_acc_to_id = Identity::add_auth(
            alice_acc_signer,
            bob_identity_signer,
            AuthorizationData::JoinIdentity(JoinIdentityData::new(alice_did, vec![])),
            None,
        );

        assert_ok!(Balances::top_up_identity_balance(
            alice.clone(),
            alice_did,
            PROTOCOL_OP_BASE_FEE
        ));
        assert_ok!(Identity::join_identity(
            bob_identity_signer,
            auth_id_for_acc_to_id
        ));

        let auth_id_for_acc2_to_id = Identity::add_auth(
            charlie_acc_signer,
            bob_identity_signer,
            AuthorizationData::JoinIdentity(JoinIdentityData::new(charlie_did, vec![])),
            None,
        );

        assert_ok!(Balances::top_up_identity_balance(
            charlie.clone(),
            charlie_did,
            PROTOCOL_OP_BASE_FEE
        ));
        assert_ok!(Identity::join_identity(
            bob_identity_signer,
            auth_id_for_acc2_to_id
        ));

        let auth_id_for_acc1_to_acc = Identity::add_auth(
            alice_acc_signer,
            dave_acc_signer,
            AuthorizationData::JoinIdentity(JoinIdentityData::new(alice_did, vec![])),
            None,
        );

        assert_ok!(Balances::top_up_identity_balance(
            alice.clone(),
            alice_did,
            PROTOCOL_OP_BASE_FEE
        ));
        assert_ok!(Identity::join_identity(
            dave_acc_signer,
            auth_id_for_acc1_to_acc
        ));

        let auth_id_for_acc2_to_acc = Identity::add_auth(
            charlie_acc_signer,
            dave_acc_signer,
            AuthorizationData::JoinIdentity(JoinIdentityData::new(charlie_did, vec![])),
            None,
        );

        assert_err!(
            Identity::join_identity(dave_acc_signer, auth_id_for_acc2_to_acc),
            Error::<TestStorage>::AlreadyLinked
        );

        let alice_signing_items = Identity::did_records(alice_did).signing_items;
        let charlie_signing_items = Identity::did_records(charlie_did).signing_items;
        assert!(alice_signing_items
            .iter()
            .find(|si| **si == bob_did)
            .is_some());
        assert!(charlie_signing_items
            .iter()
            .find(|si| **si == bob_did)
            .is_some());
        assert!(
            alice_signing_items
                .iter()
                .find(|si| **si
                    == AccountKey::try_from(AccountKeyring::Dave.public().encode()).unwrap())
                .is_some()
        );
        assert!(
            charlie_signing_items
                .iter()
                .find(|si| **si
                    == AccountKey::try_from(AccountKeyring::Dave.public().encode()).unwrap())
                .is_none()
        );
    });
}

#[test]
fn invalidate_cdd_claims() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .cdd_providers(vec![
            AccountKeyring::Eve.public(),
            AccountKeyring::Ferdie.public(),
        ])
        .build()
        .execute_with(invalidate_cdd_claims_we);
}

fn invalidate_cdd_claims_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let cdd_1_acc = AccountKeyring::Eve.public();
    let cdd_1_key = AccountKey::try_from(cdd_1_acc.0).unwrap();
    let alice_acc = AccountKeyring::Alice.public();
    let bob_acc = AccountKeyring::Bob.public();
    assert_ok!(Identity::cdd_register_did(
        Origin::signed(cdd_1_acc),
        alice_acc,
        Some(10),
        vec![]
    ));

    // Check that Alice's ID is attested by CDD 1.
    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    let cdd_1_id = Identity::get_identity(&cdd_1_key).unwrap();
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Disable CDD 1.
    assert_ok!(Identity::invalidate_cdd_claims(root, cdd_1_id, 5, Some(10)));
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Move to time 8... CDD_1 is inactive: Its claims are valid.
    Timestamp::set_timestamp(8);
    assert_eq!(Identity::has_valid_cdd(alice_id), true);
    assert_err!(
        Identity::cdd_register_did(Origin::signed(cdd_1_acc), bob_acc, Some(10), vec![]),
        Error::<TestStorage>::UnAuthorizedCddProvider
    );

    // Move to time 11 ... CDD_1 is expired: Its claims are invalid.
    Timestamp::set_timestamp(11);
    assert_eq!(Identity::has_valid_cdd(alice_id), false);
    assert_err!(
        Identity::cdd_register_did(Origin::signed(cdd_1_acc), bob_acc, Some(20), vec![]),
        Error::<TestStorage>::UnAuthorizedCddProvider
    );
}

#[test]
fn cdd_provider_with_systematic_cdd_claims() {
    let cdd_providers = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();

    ExtBuilder::default()
        .monied(true)
        .cdd_providers(cdd_providers)
        .build()
        .execute_with(cdd_provider_with_systematic_cdd_claims_we);
}

fn cdd_provider_with_systematic_cdd_claims_we() {
    // 0. Get Bob & Alice IDs.
    let root = Origin::system(frame_system::RawOrigin::Root);
    let bob_id = get_identity_id(AccountKeyring::Bob).expect("Bob should be one of CDD providers");
    let alice_id =
        get_identity_id(AccountKeyring::Alice).expect("Bob should be one of CDD providers");

    // 1. Each CDD provider has a *systematic* CDD claim.
    let cdd_providers = CddServiceProviders::get_members();
    assert_eq!(
        cdd_providers
            .iter()
            .all(|cdd| fetch_systematic_cdd(*cdd).is_some()),
        true
    );

    // 2. Remove one member from CDD provider and double-check that systematic CDD claim was
    //    removed too.
    assert_ok!(CddServiceProviders::remove_member(root.clone(), bob_id));
    assert_eq!(fetch_systematic_cdd(bob_id).is_none(), true);
    assert_eq!(fetch_systematic_cdd(alice_id).is_some(), true);

    // 3. Add DID with CDD claim to CDD providers, and check that systematic CDD claim was added.
    // Then remove that DID from CDD provides, it should keep its previous CDD claim.
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let charlie_acc = AccountKeyring::Charlie.public();

    // 3.1. Add CDD claim to Charlie, by Alice.
    assert_ok!(Identity::cdd_register_did(
        alice,
        charlie_acc.clone(),
        None,
        vec![]
    ));
    let charlie_id =
        get_identity_id(AccountKeyring::Charlie).expect("Charlie should have an Identity Id");
    let charlie_cdd_claim =
        Identity::fetch_cdd(charlie_id, 0).expect("Charlie should have a CDD claim by Alice");

    // 3.2. Add Charlie as trusted CDD providers, and check its new systematic CDD claim.
    assert_ok!(CddServiceProviders::add_member(root.clone(), charlie_id));
    assert_eq!(fetch_systematic_cdd(charlie_id).is_some(), true);

    // 3.3. Remove Charlie from trusted CDD providers, and verify that systematic CDD claim was
    //   removed and previous CDD claim works.
    assert_ok!(CddServiceProviders::remove_member(root, charlie_id));
    assert_eq!(fetch_systematic_cdd(charlie_id).is_none(), true);
    assert_eq!(Identity::fetch_cdd(charlie_id, 0), Some(charlie_cdd_claim));
}

#[test]
fn gc_with_systematic_cdd_claims() {
    let cdd_providers = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();
    let governance_committee = [
        AccountKeyring::Charlie.public(),
        AccountKeyring::Dave.public(),
    ]
    .to_vec();

    ExtBuilder::default()
        .monied(true)
        .cdd_providers(cdd_providers)
        .governance_committee(governance_committee)
        .build()
        .execute_with(gc_with_systematic_cdd_claims_we);
}

fn gc_with_systematic_cdd_claims_we() {
    // 0.
    let root = Origin::system(frame_system::RawOrigin::Root);
    let charlie_id = get_identity_id(AccountKeyring::Charlie)
        .expect("Charlie should be a Governance Committee member");
    let dave_id = get_identity_id(AccountKeyring::Dave)
        .expect("Dave should be a Governance Committee member");

    // 1. Each GC member has a *systematic* CDD claim.
    let governance_committee = GovernanceCommittee::get_members();
    assert_eq!(
        governance_committee
            .iter()
            .all(|gc_member| fetch_systematic_cdd(*gc_member).is_some()),
        true
    );

    // 2. Remove one member from GC and double-check that systematic CDD claim was
    //    removed too.
    assert_ok!(GovernanceCommittee::remove_member(root.clone(), charlie_id));
    assert_eq!(fetch_systematic_cdd(charlie_id).is_none(), true);
    assert_eq!(fetch_systematic_cdd(dave_id).is_some(), true);

    // 3. Add DID with CDD claim to CDD providers, and check that systematic CDD claim was added.
    // Then remove that DID from CDD provides, it should keep its previous CDD claim.
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let ferdie_acc = AccountKeyring::Ferdie.public();

    // 3.1. Add CDD claim to Ferdie, by Alice.
    assert_ok!(Identity::cdd_register_did(
        alice,
        ferdie_acc.clone(),
        None,
        vec![]
    ));
    let ferdie_id =
        get_identity_id(AccountKeyring::Ferdie).expect("Ferdie should have an Identity Id");
    let ferdie_cdd_claim =
        Identity::fetch_cdd(ferdie_id, 0).expect("Ferdie should have a CDD claim by Alice");

    // 3.2. Add Ferdie to GC, and check its new systematic CDD claim.
    assert_ok!(GovernanceCommittee::add_member(root.clone(), ferdie_id));
    assert_eq!(fetch_systematic_cdd(ferdie_id).is_some(), true);

    // 3.3. Remove Ferdie from GC, and verify that systematic CDD claim was
    //   removed and previous CDD claim works.
    assert_ok!(GovernanceCommittee::remove_member(root, ferdie_id));
    assert_eq!(fetch_systematic_cdd(ferdie_id).is_none(), true);
    assert_eq!(Identity::fetch_cdd(ferdie_id, 0), Some(ferdie_cdd_claim));
}

#[test]
fn gc_and_cdd_with_systematic_cdd_claims() {
    let gc_and_cdd_providers =
        [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();

    ExtBuilder::default()
        .monied(true)
        .cdd_providers(gc_and_cdd_providers.clone())
        .governance_committee(gc_and_cdd_providers.clone())
        .build()
        .execute_with(gc_and_cdd_with_systematic_cdd_claims_we);
}

fn gc_and_cdd_with_systematic_cdd_claims_we() {
    // 0. Accounts
    let root = Origin::system(frame_system::RawOrigin::Root);
    let alice_id = get_identity_id(AccountKeyring::Alice)
        .expect("Charlie should be a Governance Committee member");

    // 1. Alice should have 2 systematic CDD claims: One as GC member & another one as CDD
    //    provider.
    assert_eq!(fetch_systematic_cdd(alice_id).is_some(), true);

    // 2. Remove Alice from CDD providers.
    assert_ok!(CddServiceProviders::remove_member(root.clone(), alice_id));
    assert_eq!(fetch_systematic_cdd(alice_id).is_some(), true);

    // 3. Remove Alice from GC.
    assert_ok!(GovernanceCommittee::remove_member(root, alice_id));
    assert_eq!(fetch_systematic_cdd(alice_id).is_none(), true);
}

#[test]
fn add_permission_with_signing_item() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .cdd_providers(vec![
            AccountKeyring::Eve.public(),
            AccountKeyring::Ferdie.public(),
        ])
        .build()
        .execute_with(|| {
            let cdd_1_acc = AccountKeyring::Eve.public();
            let alice_acc = AccountKeyring::Alice.public();
            let bob_acc = AccountKeyring::Bob.public();
            let charlie_acc = AccountKeyring::Charlie.public();

            // SigningItem added
            let sig_1 = SigningItem {
                signer: Signatory::from(AccountKey::try_from(bob_acc.encode()).unwrap()),
                signer_type: SignatoryType::External,
                permissions: vec![Permission::Admin, Permission::Operator],
            };

            let sig_2 = SigningItem {
                signer: Signatory::from(AccountKey::try_from(charlie_acc.encode()).unwrap()),
                signer_type: SignatoryType::External,
                permissions: vec![Permission::Full],
            };

            assert_ok!(Identity::cdd_register_did(
                Origin::signed(cdd_1_acc),
                alice_acc,
                Some(10),
                vec![sig_1.clone(), sig_2.clone()]
            ));

            let did = match Identity::key_to_identity_ids(
                AccountKey::try_from(alice_acc.encode()).unwrap(),
            )
            .unwrap()
            {
                LinkedKeyInfo::Unique(did) => did,
                _ => Default::default(),
            };

            let bob_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(
                Signatory::from(AccountKey::try_from(bob_acc.encode()).unwrap()),
            )
            .next()
            .unwrap()
            .auth_id;
            let charlie_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(
                Signatory::from(AccountKey::try_from(charlie_acc.encode()).unwrap()),
            )
            .next()
            .unwrap()
            .auth_id;

            println!("Print the protocol base fee: {:?}", PROTOCOL_OP_BASE_FEE);

            // Fund the identity
            assert_ok!(Balances::top_up_identity_balance(
                Origin::signed(alice_acc),
                did,
                PROTOCOL_OP_BASE_FEE * 3
            ));

            // accept the auth_id
            assert_ok!(Identity::accept_authorization(
                Origin::signed(bob_acc),
                bob_auth_id
            ));

            // accept the auth_id
            assert_ok!(Identity::accept_authorization(
                Origin::signed(charlie_acc),
                charlie_auth_id
            ));

            // check for permissions
            let sig_items = (Identity::did_records(did)).signing_items;
            assert_eq!(sig_items[0], sig_1);
            assert_eq!(sig_items[1], sig_2);
        });
}
