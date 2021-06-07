use super::{
    asset_test::{an_asset, basic_asset, max_len, max_len_bytes, token},
    committee_test::gc_vmo,
    ext_builder::PROTOCOL_OP_BASE_FEE,
    storage::{
        add_secondary_key, create_cdd_id_and_investor_uid, get_identity_id, get_last_auth_id,
        provide_scope_claim, register_keyring_account, register_keyring_account_with_balance,
        AccountId, GovernanceCommittee, TestStorage, User,
    },
    ExtBuilder,
};
use codec::Encode;
use confidential_identity::mocked::make_investor_uid as make_investor_uid_v2;
use core::iter;
use frame_support::{
    assert_noop, assert_ok, dispatch::DispatchResult, traits::Currency, StorageDoubleMap,
    StorageMap,
};
use pallet_asset::SecurityToken;
use pallet_balances as balances;
use pallet_identity::types::DidRecords as RpcDidRecords;
use pallet_identity::{self as identity, DidRecords};
use polymesh_common_utilities::{
    protocol_fee::ProtocolOp,
    traits::{
        group::GroupTrait,
        identity::{SecondaryKeyWithAuth, TargetIdAuthorization, Trait as IdentityTrait},
        transaction_payment::CddAndFeeDetails,
    },
    SystematicIssuers, GC_DID,
};
use polymesh_primitives::{
    investor_zkproof_data::v2, AssetPermissions, AuthorizationData, AuthorizationError,
    AuthorizationType, CddId, Claim, ClaimType, DispatchableName, ExtrinsicPermissions,
    IdentityClaim, IdentityId, InvestorUid, PalletName, PalletPermissions, Permissions,
    PortfolioId, PortfolioNumber, Scope, SecondaryKey, Signatory, SubsetRestriction, Ticker,
    TransactionError,
};
use polymesh_runtime_develop::{fee_details::CddHandler, runtime::Call};
use sp_core::{crypto::AccountId32, sr25519::Public, H512};
use sp_runtime::transaction_validity::InvalidTransaction;
use std::convert::{From, TryFrom};
use test_client::AccountKeyring;

type AuthorizationsGiven = identity::AuthorizationsGiven<TestStorage>;
type Asset = pallet_asset::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Identity = identity::Module<TestStorage>;
type MultiSig = pallet_multisig::Module<TestStorage>;
type System = frame_system::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;

type Origin = <TestStorage as frame_system::Trait>::Origin;
type CddServiceProviders = <TestStorage as IdentityTrait>::CddServiceProviders;
type Error = pallet_identity::Error<TestStorage>;
type PError = pallet_permissions::Error<TestStorage>;

// Identity Test Helper functions
// =======================================

/// Utility function to fetch *only* systematic CDD claims.
///
/// We have 2 systematic CDD claims issuers:
/// * Governance Committee group.
/// * CDD providers group.
fn fetch_systematic_claim(target: IdentityId) -> Option<IdentityClaim> {
    fetch_systematic_gc(target).or_else(|| fetch_systematic_cdd(target))
}

fn fetch_systematic_gc(target: IdentityId) -> Option<IdentityClaim> {
    Identity::fetch_claim(target, ClaimType::CustomerDueDiligence, GC_DID, None)
}

fn fetch_systematic_cdd(target: IdentityId) -> Option<IdentityClaim> {
    Identity::fetch_claim(
        target,
        ClaimType::CustomerDueDiligence,
        SystematicIssuers::CDDProvider.as_id(),
        None,
    )
}

fn get_secondary_keys(target: IdentityId) -> Vec<SecondaryKey<AccountId>> {
    match Identity::get_did_records(target) {
        RpcDidRecords::Success { secondary_keys, .. } => secondary_keys,
        _ => vec![],
    }
}

fn create_new_token(name: &[u8], owner: User) -> (Ticker, SecurityToken<u128>) {
    let r = token(name, owner.did);
    assert_ok!(basic_asset(owner, r.0, &r.1));
    r
}

macro_rules! assert_add_cdd_claim {
    ($signer:expr, $target:expr) => {
        assert_ok!(Identity::add_claim(
            $signer,
            $target,
            Claim::CustomerDueDiligence(create_cdd_id_and_investor_uid($target).0),
            None
        ));
    };
}

// Tests
// ======================================
/// TODO Add `Signatory::Identity(..)` test.
#[test]
fn only_primary_or_secondary_keys_can_authenticate_as_an_identity() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let owner_signer = Signatory::Account(AccountKeyring::Alice.public());

        let a_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let a = Origin::signed(AccountKeyring::Bob.public());
        let b_did = register_keyring_account(AccountKeyring::Dave).unwrap();

        let charlie_key = AccountKeyring::Charlie.public();
        let charlie_signer = Signatory::Account(charlie_key);

        add_secondary_key(a_did, charlie_signer);

        // Check primary key on primary and secondary_keys.
        assert!(Identity::is_signer_authorized(owner_did, &owner_signer));
        assert!(Identity::is_signer_authorized(a_did, &charlie_signer));

        assert!(Identity::is_signer_authorized(b_did, &charlie_signer) == false);

        // ... and remove that key.
        assert_ok!(Identity::remove_secondary_keys(
            a.clone(),
            vec![charlie_signer.clone()]
        ));
        assert!(Identity::is_signer_authorized(a_did, &charlie_signer) == false);
    });
}

#[test]
fn gc_add_remove_cdd_claim() {
    ExtBuilder::default().build().execute_with(|| {
        let target_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let fetch =
            || Identity::fetch_claim(target_did, ClaimType::CustomerDueDiligence, GC_DID, None);

        assert_ok!(Identity::gc_add_cdd_claim(gc_vmo(), target_did));

        let cdd_id = CddId::new_v1(target_did, InvestorUid::from(target_did.as_ref()));
        assert_eq!(
            fetch(),
            Some(IdentityClaim {
                claim_issuer: GC_DID,
                issuance_date: 0,
                last_update_date: 0,
                expiry: None,
                claim: Claim::CustomerDueDiligence(cdd_id)
            })
        );

        assert_ok!(Identity::gc_revoke_cdd_claim(gc_vmo(), target_did));
        assert_eq!(fetch(), None);
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
        let scope = Scope::from(IdentityId::from(0));

        assert_ok!(Identity::add_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            Claim::Accredited(scope.clone()),
            Some(100u64),
        ));
        assert!(Identity::fetch_claim(
            claim_issuer_did,
            ClaimType::Accredited,
            claim_issuer_did,
            Some(scope.clone())
        )
        .is_some());

        assert_ok!(Identity::revoke_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            Claim::Accredited(scope.clone()),
        ));
        assert!(Identity::fetch_claim(
            claim_issuer_did,
            ClaimType::Accredited,
            claim_issuer_did,
            Some(scope.clone())
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
        let scope = Scope::from(IdentityId::from(0));

        assert_ok!(Identity::add_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            Claim::Accredited(scope.clone()),
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
            Some(scope.clone())
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
            Some(scope.clone()),
        )
        .is_some());

        assert_ok!(Identity::revoke_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            Claim::Accredited(scope.clone()),
        ));

        assert_ok!(Identity::revoke_claim(
            claim_issuer.clone(),
            claim_issuer_did,
            Claim::NoData,
        ));

        assert!(Identity::fetch_claim(
            claim_issuer_did,
            ClaimType::Accredited,
            claim_issuer_did,
            Some(scope.clone())
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
            Some(scope.clone()),
        )
        .is_none());
    });
}

#[test]
fn only_primary_key_can_add_secondary_key_permissions() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&only_primary_key_can_add_secondary_key_permissions_with_externalities);
}
fn only_primary_key_can_add_secondary_key_permissions_with_externalities() {
    let bob_key = AccountKeyring::Bob.public();
    let charlie_key = AccountKeyring::Charlie.public();
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());

    add_secondary_key(alice_did, Signatory::Account(charlie_key));
    add_secondary_key(alice_did, Signatory::Account(bob_key));

    // Only `alice` is able to update `bob`'s permissions and `charlie`'s permissions.
    assert_ok!(Identity::set_permission_to_signer(
        alice.clone(),
        Signatory::Account(bob_key),
        Permissions::empty().into(),
    ));
    assert_ok!(Identity::set_permission_to_signer(
        alice.clone(),
        Signatory::Account(charlie_key),
        Permissions::empty().into(),
    ));

    // Bob tries to get better permission by himself at `alice` Identity.
    assert_noop!(
        Identity::set_permission_to_signer(
            bob.clone(),
            Signatory::Account(bob_key),
            Permissions::default().into()
        ),
        PError::UnauthorizedCaller
    );

    // Bob tries to remove Charlie's permissions at `alice` Identity.
    assert_noop!(
        Identity::set_permission_to_signer(
            bob,
            Signatory::Account(charlie_key),
            Permissions::empty().into()
        ),
        PError::UnauthorizedCaller
    );

    // Alice over-write some permissions.
    assert_ok!(Identity::set_permission_to_signer(
        alice,
        Signatory::Account(bob_key),
        Permissions::empty().into()
    ));
}

#[test]
fn add_permissions_to_multiple_tokens() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_add_permissions_to_multiple_tokens);
}
fn do_add_permissions_to_multiple_tokens() {
    let bob_key = AccountKeyring::Bob.public();
    let bob_signer = Signatory::Account(bob_key);
    let alice = User::new(AccountKeyring::Alice);

    // Add bob with default permissions
    add_secondary_key(alice.did, bob_signer);

    // Create some tokens
    let max_tokens = 30;
    let tokens: Vec<Ticker> = (0..max_tokens)
        .map(|i| {
            let name = format!("TOKEN_{}", i);
            let (ticker, _) = create_new_token(name.as_bytes(), alice);
            ticker
        })
        .collect();

    let set_bob_asset_permissions = |num_token| {
        assert_ok!(Identity::set_permission_to_signer(
            alice.origin(),
            bob_signer,
            Permissions {
                asset: AssetPermissions::elems(tokens[0..num_token].into_iter().map(|t| t.clone())),
                ..Default::default()
            },
        ));
    };

    // add one-by-one
    for num in 0..max_tokens {
        set_bob_asset_permissions(num);
    }

    // remove all permissions
    set_bob_asset_permissions(0);

    // bulk add
    set_bob_asset_permissions(max_tokens);
}

#[test]
fn set_permission_to_signer_with_bad_perms() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = AccountKeyring::Bob.public();
        add_secondary_key(alice.did, Signatory::Account(bob));
        test_with_bad_perms(alice.did, |perms| {
            assert_too_long!(Identity::set_permission_to_signer(
                alice.origin(),
                Signatory::Account(bob),
                perms,
            ));
        });
    });
}

/// It verifies that frozen keys are recovered after `unfreeze` call.
#[test]
fn freeze_secondary_keys_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&freeze_secondary_keys_with_externalities);
}

fn freeze_secondary_keys_with_externalities() {
    let (bob_key, charlie_key, dave_key) = (
        AccountKeyring::Bob.public(),
        AccountKeyring::Charlie.public(),
        AccountKeyring::Dave.public(),
    );
    // Add secondary keys.
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());

    add_secondary_key(alice_did, Signatory::Account(bob_key));
    add_secondary_key(alice_did, Signatory::Account(charlie_key));

    assert_eq!(
        Identity::is_signer_authorized(alice_did, &Signatory::Account(bob_key)),
        true
    );

    // Freeze secondary keys: bob & charlie.
    assert_noop!(
        Identity::freeze_secondary_keys(bob.clone()),
        Error::KeyNotAllowed
    );
    assert_ok!(Identity::freeze_secondary_keys(alice.clone()));

    assert_eq!(
        Identity::is_signer_authorized(alice_did, &Signatory::Account(bob_key)),
        false
    );

    add_secondary_key(alice_did, Signatory::Account(dave_key));

    // update permission of frozen keys.
    assert_ok!(Identity::set_permission_to_signer(
        alice.clone(),
        Signatory::Account(bob_key),
        Permissions::default().into(),
    ));

    // unfreeze all
    // commenting this because `default_identity` feature is not allowing to access None identity.
    // assert_noop!(
    //     Identity::unfreeze_secondary_keys(bob.clone()),
    //     DispatchError::Other("Current identity is none and key is not linked to any identity")
    // );
    assert_ok!(Identity::unfreeze_secondary_keys(alice.clone()));

    assert_eq!(
        Identity::is_signer_authorized(alice_did, &Signatory::Account(dave_key)),
        true
    );
}

/// It double-checks that frozen keys are removed too.
#[test]
fn remove_frozen_secondary_keys_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&remove_frozen_secondary_keys_with_externalities);
}

fn remove_frozen_secondary_keys_with_externalities() {
    let (bob_key, charlie_key) = (
        AccountKeyring::Bob.public(),
        AccountKeyring::Charlie.public(),
    );

    let charlie_secondary_key =
        SecondaryKey::new(Signatory::Account(charlie_key), Permissions::default());

    // Add secondary keys.
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());

    add_secondary_key(alice_did, Signatory::Account(bob_key));
    add_secondary_key(alice_did, Signatory::Account(charlie_key));

    // Freeze all secondary keys
    assert_ok!(Identity::freeze_secondary_keys(alice.clone()));

    // Remove Bob's key.
    assert_ok!(Identity::remove_secondary_keys(
        alice.clone(),
        vec![Signatory::Account(bob_key)]
    ));
    // Check DidRecord.
    let did_rec = Identity::did_records(alice_did);
    assert_eq!(did_rec.secondary_keys, vec![charlie_secondary_key]);
}

/// It double-checks that frozen keys are removed too.
#[test]
fn frozen_secondary_keys_cdd_verification_test() {
    ExtBuilder::default()
        .build()
        .execute_with(&frozen_secondary_keys_cdd_verification_test_we);
}

fn frozen_secondary_keys_cdd_verification_test_we() {
    // 0. Create identity for Alice and secondary key from Bob.
    let alice = AccountKeyring::Alice.public();
    let bob = AccountKeyring::Bob.public();
    let charlie = AccountKeyring::Charlie.public();
    TestStorage::set_payer_context(Some(alice));
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    TestStorage::set_payer_context(Some(charlie));
    let _charlie_id = register_keyring_account_with_balance(AccountKeyring::Charlie, 100).unwrap();
    assert_eq!(Balances::free_balance(charlie), 100);

    // 1. Add Bob as signatory to Alice ID.
    let bob_signatory = Signatory::Account(AccountKeyring::Bob.public());
    TestStorage::set_payer_context(Some(alice));

    add_secondary_key(alice_id, bob_signatory);
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice),
        bob,
        25_000,
        None
    ));
    assert_eq!(Balances::free_balance(bob), 25_000);

    // 2. Bob can transfer some funds to Charlie ID.
    TestStorage::set_payer_context(Some(bob));
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(bob),
        charlie,
        1_000,
        None
    ));
    assert_eq!(Balances::free_balance(charlie), 1100);

    // 3. Alice freezes her secondary keys.
    assert_ok!(Identity::freeze_secondary_keys(Origin::signed(alice)));

    // 4. Bob should NOT transfer any amount. SE is simulated.
    // Balances::transfer_with_memo(Origin::signed(bob), charlie, 1_000, None),
    let payer = CddHandler::get_valid_payer(
        &Call::Balances(balances::Call::transfer_with_memo(
            AccountKeyring::Charlie.to_account_id().into(),
            1_000,
            None,
        )),
        &AccountId32::from(AccountKeyring::Bob.public().0),
    );
    assert_noop!(
        payer,
        InvalidTransaction::Custom(TransactionError::MissingIdentity as u8)
    );

    assert_eq!(Balances::free_balance(charlie), 1100);

    // 5. Alice still can make transfers.
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice),
        charlie,
        1_000,
        None
    ));
    assert_eq!(Balances::free_balance(charlie), 2100);

    // 6. Unfreeze signatory keys, and Bob should be able to transfer again.
    assert_ok!(Identity::unfreeze_secondary_keys(Origin::signed(alice)));
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(bob),
        charlie,
        1_000,
        None
    ));
    assert_eq!(Balances::free_balance(charlie), 3100);
}

#[test]
fn add_secondary_keys_with_permissions_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_add_secondary_keys_with_permissions_test);
}

fn do_add_secondary_keys_with_permissions_test() {
    let bob_key = AccountKeyring::Bob.public();
    let bob_signer = Signatory::Account(bob_key);
    let alice = User::new(AccountKeyring::Alice);

    // Add bob with default permissions
    add_secondary_key(alice.did, bob_signer);

    let permissions = Permissions::from_pallet_permissions(vec![PalletPermissions::entire_pallet(
        b"identity".into(),
    )]);
    // Try adding bob again with custom permissions
    let auth_id = Identity::add_auth(
        alice.did,
        bob_signer,
        AuthorizationData::JoinIdentity(permissions.clone()),
        None,
    );
    assert_noop!(
        Identity::join_identity(bob_signer, auth_id),
        Error::AlreadyLinked
    );

    // Try addind the same secondary_key using `add_secondary_keys_with_authorization`
    let expires_at = 100u64;
    let target_id_auth = |user: User| TargetIdAuthorization {
        target_id: user.did,
        nonce: Identity::offchain_authorization_nonce(user.did),
        expires_at,
    };
    let authorization = target_id_auth(alice);
    let auth_encoded = authorization.encode();
    let auth_signature = H512::from(alice.ring.sign(&auth_encoded));

    let bob_key_2 = SecondaryKey::new(bob_signer, permissions);
    let key_with_auth = SecondaryKeyWithAuth {
        auth_signature,
        secondary_key: bob_key_2.into(),
    };

    assert_noop!(
        Identity::add_secondary_keys_with_authorization(
            alice.origin(),
            vec![key_with_auth],
            expires_at
        ),
        Error::AlreadyLinked
    );

    // Check KeyToIdentityIds map
    assert_eq!(Identity::get_identity(&bob_key), Some(alice.did));

    // Check DidRecords
    let keys = get_secondary_keys(alice.did);
    assert_eq!(keys.len(), 1);

    // Try remove bob using alice
    TestStorage::set_current_identity(&alice.did);
    assert_ok!(Identity::remove_secondary_keys(
        alice.origin(),
        vec![Signatory::Account(bob_key)]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Check DidRecords
    let keys = get_secondary_keys(alice.did);
    assert_eq!(keys.len(), 0);
}

#[test]
fn remove_secondary_keys_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_remove_secondary_keys_test);
}

fn do_remove_secondary_keys_test() {
    let bob_key = AccountKeyring::Bob.public();
    let dave_key = AccountKeyring::Dave.public();
    let alice = User::new(AccountKeyring::Alice);

    add_secondary_key(alice.did, Signatory::Account(bob_key));
    add_secondary_key(alice.did, Signatory::Account(dave_key));

    // Check KeyToIdentityIds map
    assert_eq!(Identity::get_identity(&bob_key), Some(alice.did));
    assert_eq!(Identity::get_identity(&dave_key), Some(alice.did));

    // Check DidRecords
    let keys = get_secondary_keys(alice.did);
    assert_eq!(keys.len(), 2);

    // Try remove bob using alice
    TestStorage::set_current_identity(&alice.did);
    assert_ok!(Identity::remove_secondary_keys(
        alice.origin(),
        vec![Signatory::Account(bob_key)]
    ));

    // try changing the permissions for bob's key
    // This should fail.
    let result = Identity::set_permission_to_signer(
        alice.origin(),
        Signatory::Account(bob_key),
        Permissions::from_pallet_permissions(vec![PalletPermissions::entire_pallet(
            b"identity".into(),
        )])
        .into(),
    );
    assert_ok!(result);
    // FIXME: use this after the fix.
    //assert_noop!(result, Error::NotASigner);

    // Check DidRecords
    let keys = get_secondary_keys(alice.did);
    assert_eq!(keys.len(), 2);
    // FIXME: used this after the fix
    //assert_eq!(keys.len(), 1);

    // Check identity map
    assert_eq!(Identity::get_identity(&bob_key), None);
    assert_eq!(Identity::get_identity(&dave_key), Some(alice.did));

    // try re-adding bob's key
    add_secondary_key(alice.did, Signatory::Account(bob_key));

    // Check identity map
    assert_eq!(Identity::get_identity(&bob_key), Some(alice.did));

    // remove bob's key again
    assert_ok!(Identity::remove_secondary_keys(
        alice.origin(),
        vec![Signatory::Account(bob_key)]
    ));

    // Try remove dave using alice
    assert_ok!(Identity::remove_secondary_keys(
        alice.origin(),
        vec![Signatory::Account(dave_key)]
    ));

    // Check identity map
    assert_eq!(Identity::get_identity(&dave_key), None);

    // Check DidRecords
    let keys = get_secondary_keys(alice.did);
    // 2 x bob_key
    assert_eq!(keys.len(), 2);
    // FIXME: used this after the fix
    //assert_eq!(keys.len(), 0);
}

#[test]
fn remove_secondary_keys_test_with_externalities() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_remove_secondary_keys_test_with_externalities);
}

fn do_remove_secondary_keys_test_with_externalities() {
    let bob_key = AccountKeyring::Bob.public();
    let alice_key = AccountKeyring::Alice.public();
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());
    let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let dave_key = AccountKeyring::Dave.public();

    let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(alice_did), Signatory::Account(dave_key)],
        1,
    ));
    let auth_id = get_last_auth_id(&Signatory::Account(dave_key));
    assert_ok!(MultiSig::unsafe_accept_multisig_signer(
        Signatory::Account(dave_key),
        auth_id
    ));

    add_secondary_key(alice_did, Signatory::Account(bob_key));

    add_secondary_key(alice_did, Signatory::Account(musig_address));

    // Fund the multisig
    assert_ok!(Balances::transfer(alice.clone(), musig_address.clone(), 1));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_address), Some(alice_did));
    assert_eq!(Identity::get_identity(&bob_key), Some(alice_did));

    // Try removing bob using charlie
    TestStorage::set_current_identity(&charlie_did);
    assert_ok!(Identity::remove_secondary_keys(
        charlie.clone(),
        vec![Signatory::Account(bob_key)]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_address), Some(alice_did));
    assert_eq!(Identity::get_identity(&bob_key), Some(alice_did));

    // Try remove bob using alice
    TestStorage::set_current_identity(&alice_did);
    assert_ok!(Identity::remove_secondary_keys(
        alice.clone(),
        vec![Signatory::Account(bob_key)]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_address), Some(alice_did));
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Try removing multisig while it has funds
    assert_ok!(Identity::remove_secondary_keys(
        alice.clone(),
        vec![Signatory::Account(musig_address.clone())]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_address), Some(alice_did));
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(musig_address.clone(), Signatory::Account(dave_key)),
        true
    );

    // Transfer funds back to Alice
    assert_ok!(Balances::transfer(
        Origin::signed(musig_address.clone()),
        alice_key.clone(),
        1
    ));

    // Empty multisig's funds and remove as signer
    assert_ok!(Identity::remove_secondary_keys(
        alice.clone(),
        vec![Signatory::Account(musig_address.clone())]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_address), None);
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(musig_address.clone(), Signatory::Account(dave_key)),
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
    let bob_key = AccountKeyring::Bob.public();
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let alice_key = AccountKeyring::Alice.public();
    let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let charlie = Origin::signed(AccountKeyring::Charlie.public());
    let bob_secondary_key = SecondaryKey::new(Signatory::Account(bob_key), Permissions::default());
    let charlie_secondary_key =
        SecondaryKey::new(Signatory::Identity(charlie_did), Permissions::default());
    let alice_secondary_keys = vec![bob_secondary_key, charlie_secondary_key.clone()];
    let dave_key = AccountKeyring::Dave.public();

    let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(alice_did), Signatory::Account(dave_key)],
        1,
    ));
    let auth_id = get_last_auth_id(&Signatory::Account(dave_key));
    assert_ok!(MultiSig::unsafe_accept_multisig_signer(
        Signatory::Account(dave_key),
        auth_id
    ));

    add_secondary_key(alice_did, Signatory::Account(bob_key));
    add_secondary_key(alice_did, Signatory::from(charlie_did));

    // Check DidRecord.
    let did_rec = Identity::did_records(alice_did);
    assert_eq!(did_rec.secondary_keys, alice_secondary_keys);
    assert_eq!(Identity::get_identity(&bob_key), Some(alice_did));

    // Bob leaves
    assert_ok!(Identity::leave_identity_as_key(bob));

    // Check DidRecord.
    let did_rec = Identity::did_records(alice_did);
    assert_eq!(did_rec.secondary_keys, vec![charlie_secondary_key]);
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Charlie leaves
    TestStorage::set_current_identity(&charlie_did);
    assert_ok!(Identity::leave_identity_as_identity(charlie, alice_did));

    // Check DidRecord.
    let did_rec = Identity::did_records(alice_did);
    assert_eq!(did_rec.secondary_keys.len(), 0);
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_address), None);

    add_secondary_key(alice_did, Signatory::Account(musig_address));
    // send funds to multisig
    assert_ok!(Balances::transfer(alice.clone(), musig_address.clone(), 1));
    // multisig tries leaving identity while it has funds
    assert_noop!(
        Identity::leave_identity_as_key(Origin::signed(musig_address.clone())),
        Error::MultiSigHasBalance
    );

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_address), Some(alice_did));

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(musig_address.clone(), Signatory::Account(dave_key)),
        true
    );

    // send funds back to alice from multisig
    assert_ok!(Balances::transfer(
        Origin::signed(musig_address.clone()),
        alice_key.clone(),
        1
    ));

    // Empty multisig's funds and remove as signer
    assert_ok!(Identity::leave_identity_as_key(Origin::signed(
        musig_address.clone()
    )));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&musig_address), None);

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(musig_address.clone(), Signatory::Account(dave_key)),
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
    let _ = register_keyring_account(AccountKeyring::Bob).unwrap();

    // Check external signed key uniqueness.
    let charlie_key = AccountKeyring::Charlie.public();
    add_secondary_key(alice_id, Signatory::Account(charlie_key));
    let auth_id = Identity::add_auth(
        alice_id,
        Signatory::Account(AccountKeyring::Bob.public()),
        AuthorizationData::JoinIdentity(Permissions::empty()),
        None,
    );
    assert_noop!(
        Identity::join_identity(Signatory::Account(AccountKeyring::Bob.public()), auth_id),
        Error::AlreadyLinked
    );
}

#[test]
fn add_remove_secondary_identities() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&add_remove_secondary_identities_with_externalities);
}

fn add_remove_secondary_identities_with_externalities() {
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();

    add_secondary_key(alice_id, Signatory::from(bob_id));
    add_secondary_key(alice_id, Signatory::from(charlie_id));

    assert_ok!(Identity::remove_secondary_keys(
        alice.clone(),
        vec![Signatory::Identity(bob_id)]
    ));

    let alice_rec = Identity::did_records(alice_id);
    let mut charlie_sk = SecondaryKey::from(charlie_id);
    // Correct the permissions to ones set by `add_secondary_key`.
    charlie_sk.permissions = Permissions::default();
    assert_eq!(alice_rec.secondary_keys, vec![charlie_sk]);

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

fn secondary_keys_with_auth(
    keys: &[AccountKeyring],
    ids: &[IdentityId],
    auth_encoded: &[u8],
) -> Vec<SecondaryKeyWithAuth<Public>> {
    keys.iter()
        .map(|acc| H512::from(acc.sign(&auth_encoded)))
        .zip(ids.iter().map(|&id| SecondaryKey::from(id).into()))
        .map(|(auth_signature, secondary_key)| SecondaryKeyWithAuth {
            auth_signature,
            secondary_key,
        })
        .collect()
}

#[test]
fn one_step_join_id() {
    ExtBuilder::default()
        .build()
        .execute_with(&one_step_join_id_with_ext);
}

fn one_step_join_id_with_ext() {
    let a = User::new(AccountKeyring::Alice);

    let expires_at = 100u64;
    let target_id_auth = |user: User| TargetIdAuthorization {
        target_id: user.did,
        nonce: Identity::offchain_authorization_nonce(user.did),
        expires_at,
    };
    let authorization = target_id_auth(a);
    let auth_encoded = authorization.encode();

    let keys = [
        AccountKeyring::Bob,
        AccountKeyring::Charlie,
        AccountKeyring::Dave,
    ];
    let users @ [b, c, _] = keys.map(User::new);
    let secondary_keys_with_auth =
        secondary_keys_with_auth(&keys, &users.map(|u| u.did), &auth_encoded);

    let add = |user: User, auth| {
        Identity::add_secondary_keys_with_authorization(user.origin(), auth, expires_at)
    };

    assert_ok!(add(a, secondary_keys_with_auth[..2].to_owned()));

    let secondary_keys = Identity::did_records(a.did).secondary_keys;
    let contains = |u: User| secondary_keys.iter().find(|si| **si == u.did).is_some();
    assert_eq!(contains(b), true);
    assert_eq!(contains(c), true);

    // Check reply attack. Alice's nonce is different now.
    // NOTE: We need to force the increment of account's nonce manually.
    System::inc_account_nonce(a.acc());

    assert_noop!(
        add(a, secondary_keys_with_auth[2..].to_owned()),
        Error::InvalidAuthorizationSignature
    );

    // Check revoke off-chain authorization.
    let e = User::new(AccountKeyring::Eve);
    let eve_auth = target_id_auth(a);
    assert_ne!(authorization.nonce, eve_auth.nonce);

    let eve_secondary_key_with_auth = SecondaryKeyWithAuth {
        secondary_key: SecondaryKey::from(e.did).into(),
        auth_signature: H512::from(AccountKeyring::Eve.sign(eve_auth.encode().as_slice())),
    };

    assert_ok!(Identity::revoke_offchain_authorization(
        e.origin(),
        Signatory::Identity(e.did),
        eve_auth
    ));
    assert_noop!(
        add(a, vec![eve_secondary_key_with_auth]),
        Error::AuthorizationHasBeenRevoked
    );

    // Check expire
    System::inc_account_nonce(a.acc());
    Timestamp::set_timestamp(expires_at);

    let f = User::new(AccountKeyring::Ferdie);
    let ferdie_auth = target_id_auth(a);
    let ferdie_secondary_key_with_auth = SecondaryKeyWithAuth {
        secondary_key: SecondaryKey::from(f.did).into(),
        auth_signature: H512::from(AccountKeyring::Eve.sign(ferdie_auth.encode().as_slice())),
    };

    assert_noop!(
        add(f, vec![ferdie_secondary_key_with_auth]),
        Error::AuthorizationExpired
    );
}

crate fn test_with_bad_ext_perms(test: impl Fn(ExtrinsicPermissions)) {
    test(SubsetRestriction::elems(
        (0..=max_len() as u64)
            .map(Ticker::generate)
            .map(PalletName::from)
            .map(PalletPermissions::entire_pallet),
    ));
    test(SubsetRestriction::elem(PalletPermissions::entire_pallet(
        max_len_bytes(1),
    )));
    test(SubsetRestriction::elem(PalletPermissions::new(
        "".into(),
        SubsetRestriction::elems(
            (0..=max_len() as u64)
                .map(Ticker::generate)
                .map(DispatchableName::from),
        ),
    )));
    test(SubsetRestriction::elem(PalletPermissions::new(
        "".into(),
        SubsetRestriction::elem(max_len_bytes(1)),
    )));
}

crate fn test_with_bad_perms(did: IdentityId, test: impl Fn(Permissions)) {
    test(Permissions {
        asset: SubsetRestriction::elems((0..=max_len() as u64).map(Ticker::generate_into)),
        ..<_>::default()
    });
    test(Permissions {
        portfolio: SubsetRestriction::elems(
            (0..=max_len() as u64)
                .map(|n| PortfolioId::user_portfolio(did, PortfolioNumber::from(n))),
        ),
        ..<_>::default()
    });
    test_with_bad_ext_perms(|extrinsic| {
        test(Permissions {
            extrinsic,
            ..<_>::default()
        })
    });
}

#[test]
fn add_secondary_keys_with_authorization_too_many_sks() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let expires_at = 100;
        let auth = || {
            let auth = TargetIdAuthorization {
                target_id: user.did,
                nonce: Identity::offchain_authorization_nonce(user.did),
                expires_at,
            };
            auth.encode()
        };

        // Test various length problems in `Permissions`.
        test_with_bad_perms(bob.did, |perms| {
            let auth_encoded = auth();
            let auth_signature = H512::from(bob.ring.sign(&auth_encoded));

            let secondary_key = SecondaryKey::new(bob.did.into(), perms).into();
            let auths = vec![SecondaryKeyWithAuth {
                auth_signature,
                secondary_key,
            }];
            assert_too_long!(Identity::add_secondary_keys_with_authorization(
                user.origin(),
                auths,
                expires_at
            ));
        });

        // Populate with MAX SKs.
        DidRecords::<TestStorage>::mutate(user.did, |rec| {
            let sk = SecondaryKey {
                signer: Signatory::Account(rec.primary_key),
                permissions: Permissions::empty(),
            };
            rec.secondary_keys = iter::repeat(sk).take(max_len() as usize).collect();
        });

        // Fail at adding the {MAX + 1}th SK.
        let auth_encoded = auth();
        let auths = secondary_keys_with_auth(&[bob.ring], &[bob.did], &auth_encoded);
        assert_too_long!(Identity::add_secondary_keys_with_authorization(
            user.origin(),
            auths,
            expires_at
        ));
    });
}

#[test]
fn adding_authorizations_bad_perms() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(AccountKeyring::Alice);
        test_with_bad_perms(user.did, |perms| {
            assert_too_long!(Identity::add_authorization(
                user.origin(),
                user.did.into(),
                AuthorizationData::JoinIdentity(perms),
                None
            ));
        });
    });
}

#[test]
fn adding_authorizations() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_did = Signatory::from(register_keyring_account(AccountKeyring::Bob).unwrap());
        let ticker50 = Ticker::try_from(&[0x50][..]).unwrap();
        let mut auth_id = Identity::add_auth(
            alice_did,
            bob_did,
            AuthorizationData::TransferTicker(ticker50),
            None,
        );
        assert_eq!(<AuthorizationsGiven>::get(alice_did, auth_id), bob_did);
        let mut auth = Identity::authorizations(&bob_did, auth_id);
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
        assert_eq!(<AuthorizationsGiven>::get(alice_did, auth_id), bob_did);
        auth = Identity::authorizations(&bob_did, auth_id);
        assert_eq!(auth.authorized_by, alice_did);
        assert_eq!(auth.expiry, Some(100));
        assert_eq!(
            auth.authorization_data,
            AuthorizationData::TransferTicker(ticker50)
        );

        // Testing the list of filtered authorizations
        Timestamp::set_timestamp(120);

        // Getting expired and non-expired both
        let mut authorizations = Identity::get_filtered_authorizations(
            bob_did,
            true,
            Some(AuthorizationType::TransferTicker),
        );
        assert_eq!(authorizations.len(), 2);
        authorizations = Identity::get_filtered_authorizations(
            bob_did,
            false,
            Some(AuthorizationType::TransferTicker),
        );
        // One authorization is expired
        assert_eq!(authorizations.len(), 1);
    });
}

#[test]
fn removing_authorizations() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob_did = Signatory::from(register_keyring_account(AccountKeyring::Bob).unwrap());
        let ticker50 = Ticker::try_from(&[0x50][..]).unwrap();
        let auth_id = Identity::add_auth(
            alice_did,
            bob_did,
            AuthorizationData::TransferTicker(ticker50),
            None,
        );
        assert_eq!(<AuthorizationsGiven>::get(alice_did, auth_id), bob_did);
        let auth = Identity::authorizations(&bob_did, auth_id);
        assert_eq!(
            auth.authorization_data,
            AuthorizationData::TransferTicker(ticker50)
        );
        assert_ok!(Identity::remove_authorization(
            alice.clone(),
            bob_did,
            auth_id,
            false,
        ));
        assert!(!<AuthorizationsGiven>::contains_key(alice_did, auth_id));
        assert!(!<identity::Authorizations<TestStorage>>::contains_key(
            bob_did, auth_id
        ));
    });
}

#[test]
fn changing_primary_key() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .build()
        .execute_with(changing_primary_key_we);
}

fn changing_primary_key_we() {
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);

    // Primary key matches Alice's key
    let alice_pk = || Identity::did_records(alice.did).primary_key;
    assert_eq!(alice_pk(), alice.acc());

    let add = |ring: AccountKeyring| {
        Identity::add_auth(
            alice.did,
            Signatory::Account(ring.public()),
            AuthorizationData::RotatePrimaryKey(alice.did),
            None,
        )
    };
    let accept = |ring: AccountKeyring, auth| {
        Identity::accept_primary_key(Origin::signed(ring.public()), auth, None)
    };

    // In the case of a key belong to key DID for which we're rotating, we don't allow rotation.
    let auth = add(alice.ring);
    assert_noop!(accept(alice.ring, auth), Error::AlreadyLinked);

    // Add and accept auth with new key to become new primary key.
    // Unfortunately, the new key is still linked to Bob's DID.
    let auth = add(bob.ring);
    assert_noop!(accept(bob.ring, auth), Error::AlreadyLinked);

    // Do it again, but for Charlie, who isn't attached to a DID.
    // Alice's primary key will be Charlie's.'
    let charlie = AccountKeyring::Charlie;
    assert_ok!(accept(charlie, add(charlie)));
    assert_eq!(alice_pk(), charlie.public());
}

#[test]
fn changing_primary_key_with_cdd_auth() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .build()
        .execute_with(|| changing_primary_key_with_cdd_auth_we());
}

fn changing_primary_key_with_cdd_auth_we() {
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice_key = AccountKeyring::Alice.public();

    let new_key = AccountKeyring::Bob.public();
    let new_key_origin = Origin::signed(AccountKeyring::Bob.public());

    let cdd_did = get_identity_id(AccountKeyring::Eve).unwrap();

    // Primary key matches Alice's key
    assert_eq!(Identity::did_records(alice_did).primary_key, alice_key);

    // Alice triggers change of primary key
    let owner_auth_id = Identity::add_auth(
        alice_did,
        Signatory::Account(new_key),
        AuthorizationData::RotatePrimaryKey(alice_did),
        None,
    );

    let cdd_auth_id = Identity::add_auth(
        cdd_did,
        Signatory::Account(new_key),
        AuthorizationData::AttestPrimaryKeyRotation(alice_did),
        None,
    );

    assert_ok!(Identity::change_cdd_requirement_for_mk_rotation(
        frame_system::RawOrigin::Root.into(),
        true
    ));

    assert!(
        Identity::accept_primary_key(new_key_origin.clone(), owner_auth_id.clone(), None).is_err()
    );

    let owner_auth_id2 = Identity::add_auth(
        alice_did,
        Signatory::Account(new_key),
        AuthorizationData::RotatePrimaryKey(alice_did),
        None,
    );

    // Accept the authorization with the new key
    assert_ok!(Identity::accept_primary_key(
        new_key_origin.clone(),
        owner_auth_id2,
        Some(cdd_auth_id)
    ));

    // Alice's primary key is now Bob's
    assert_eq!(
        Identity::did_records(alice_did).primary_key,
        AccountKeyring::Bob.public()
    );
}

#[test]
fn cdd_register_did_test() {
    ExtBuilder::default()
        .balance_factor(1_000)
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
    assert_ok!(Identity::cdd_register_did(cdd1.clone(), alice, vec![]));
    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    assert_add_cdd_claim!(cdd1.clone(), alice_id);

    // Check that Alice's ID is attested by CDD 1.
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Error case: Try account without ID.
    assert!(Identity::cdd_register_did(non_id, bob_acc, vec![]).is_err(),);
    // Error case: Try account with ID but it is not part of CDD providers.
    assert!(Identity::cdd_register_did(Origin::signed(alice), bob_acc, vec![]).is_err());

    // CDD 2 registers properly Bob's ID.
    assert_ok!(Identity::cdd_register_did(cdd2.clone(), bob_acc, vec![]));
    let bob_id = get_identity_id(AccountKeyring::Bob).unwrap();
    assert_add_cdd_claim!(cdd2, bob_id);

    assert_eq!(Identity::has_valid_cdd(bob_id), true);

    // Register with secondary_keys
    // ==============================================
    // Register Charlie with secondary keys.
    let charlie = AccountKeyring::Charlie.public();
    let dave = AccountKeyring::Dave.public();
    let dave_si = SecondaryKey::from_account_id(dave.clone());
    let alice_si = SecondaryKey::from(alice_id);
    let secondary_keys = vec![dave_si.clone().into(), alice_si.clone().into()];
    assert_ok!(Identity::cdd_register_did(
        cdd1.clone(),
        charlie,
        secondary_keys
    ));
    let charlie_id = get_identity_id(AccountKeyring::Charlie).unwrap();
    assert_add_cdd_claim!(cdd1.clone(), charlie_id);

    Balances::make_free_balance_be(&charlie, 10_000_000_000);
    assert_eq!(Identity::has_valid_cdd(charlie_id), true);
    assert_eq!(
        Identity::did_records(charlie_id).secondary_keys.is_empty(),
        true
    );

    let dave_auth_id = get_last_auth_id(&dave_si.signer);

    assert_ok!(Identity::accept_authorization(
        Origin::signed(dave),
        dave_auth_id
    ));
    assert_eq!(
        Identity::did_records(charlie_id).secondary_keys,
        vec![dave_si.clone()]
    );

    let alice_auth_id = get_last_auth_id(&alice_si.signer);

    TestStorage::set_current_identity(&alice_id);
    assert_ok!(Identity::accept_authorization(
        Origin::signed(alice),
        alice_auth_id
    ));
    assert_eq!(
        Identity::did_records(charlie_id).secondary_keys,
        vec![dave_si, alice_si]
    );
}

#[test]
fn add_identity_signers() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let _alice_acc_signer = Signatory::Account(AccountKeyring::Alice.public());
        let bob_identity_signer = Signatory::from(bob_did);
        let _charlie_acc_signer = Signatory::Account(AccountKeyring::Charlie.public());
        let dave_acc_signer = Signatory::Account(AccountKeyring::Dave.public());

        let auth_id_for_acc_to_id = Identity::add_auth(
            alice_did,
            bob_identity_signer,
            AuthorizationData::JoinIdentity(Permissions::default()),
            None,
        );

        assert_ok!(Identity::join_identity(
            bob_identity_signer,
            auth_id_for_acc_to_id
        ));

        let auth_id_for_acc2_to_id = Identity::add_auth(
            charlie_did,
            bob_identity_signer,
            AuthorizationData::JoinIdentity(Permissions::default()),
            None,
        );

        // Getting expired and non-expired both
        let authorizations = Identity::get_filtered_authorizations(
            bob_identity_signer,
            true,
            Some(AuthorizationType::JoinIdentity),
        );
        assert_eq!(authorizations.len(), 1);

        assert_ok!(Identity::join_identity(
            bob_identity_signer,
            auth_id_for_acc2_to_id
        ));

        let auth_id_for_acc1_to_acc = Identity::add_auth(
            alice_did,
            dave_acc_signer,
            AuthorizationData::JoinIdentity(Permissions::default()),
            None,
        );

        assert_ok!(Identity::join_identity(
            dave_acc_signer,
            auth_id_for_acc1_to_acc
        ));

        let auth_id_for_acc2_to_acc = Identity::add_auth(
            charlie_did,
            dave_acc_signer,
            AuthorizationData::JoinIdentity(Permissions::default()),
            None,
        );

        assert_noop!(
            Identity::join_identity(dave_acc_signer, auth_id_for_acc2_to_acc),
            Error::AlreadyLinked
        );

        let alice_secondary_keys = Identity::did_records(alice_did).secondary_keys;
        let charlie_secondary_keys = Identity::did_records(charlie_did).secondary_keys;
        let mut dave_sk = SecondaryKey::from_account_id(AccountKeyring::Dave.public());
        // Correct the permissions to ones set by `add_secondary_key`.
        dave_sk.permissions = Permissions::default();
        assert!(alice_secondary_keys
            .iter()
            .find(|si| **si == bob_did)
            .is_some());
        assert!(charlie_secondary_keys
            .iter()
            .find(|si| **si == bob_did)
            .is_some());
        assert!(alice_secondary_keys
            .iter()
            .find(|si| **si == dave_sk)
            .is_some());
        assert!(charlie_secondary_keys
            .iter()
            .find(|si| **si == dave_sk)
            .is_none());
    });
}

#[test]
fn invalidate_cdd_claims() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .cdd_providers(vec![
            AccountKeyring::Eve.public(),
            AccountKeyring::Ferdie.public(),
        ])
        .build()
        .execute_with(invalidate_cdd_claims_we);
}

fn invalidate_cdd_claims_we() {
    let root = Origin::from(frame_system::RawOrigin::Root);
    let cdd = AccountKeyring::Eve.public();
    let alice_acc = AccountKeyring::Alice.public();
    let bob_acc = AccountKeyring::Bob.public();
    assert_ok!(Identity::cdd_register_did(
        Origin::signed(cdd),
        alice_acc,
        vec![]
    ));
    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    assert_add_cdd_claim!(Origin::signed(cdd), alice_id);

    // Check that Alice's ID is attested by CDD 1.
    let cdd_1_id = Identity::get_identity(&cdd).unwrap();
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Disable CDD 1.
    assert_ok!(Identity::invalidate_cdd_claims(root, cdd_1_id, 5, Some(10)));
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Move to time 8... CDD_1 is inactive: Its claims are valid.
    Timestamp::set_timestamp(8);
    assert_eq!(Identity::has_valid_cdd(alice_id), true);
    assert_noop!(
        Identity::cdd_register_did(Origin::signed(cdd), bob_acc, vec![]),
        Error::UnAuthorizedCddProvider
    );

    // Move to time 11 ... CDD_1 is expired: Its claims are invalid.
    Timestamp::set_timestamp(11);
    assert_eq!(Identity::has_valid_cdd(alice_id), false);
    assert_noop!(
        Identity::cdd_register_did(Origin::signed(cdd), bob_acc, vec![]),
        Error::UnAuthorizedCddProvider
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
    let root = Origin::from(frame_system::RawOrigin::Root);
    let bob_id = get_identity_id(AccountKeyring::Bob).expect("Bob should be one of CDD providers");
    let alice_id =
        get_identity_id(AccountKeyring::Alice).expect("Bob should be one of CDD providers");

    // 1. Each CDD provider has a *systematic* CDD claim.
    let cdd_providers = CddServiceProviders::get_members();
    assert_eq!(
        cdd_providers
            .iter()
            .all(|cdd| fetch_systematic_claim(*cdd).is_some()),
        true
    );

    // 2. Remove one member from CDD provider and double-check that systematic CDD claim was
    //    removed too.
    assert_ok!(CddServiceProviders::remove_member(root.clone(), bob_id));
    assert_eq!(fetch_systematic_claim(bob_id).is_none(), true);
    assert_eq!(fetch_systematic_claim(alice_id).is_some(), true);

    // 3. Add DID with CDD claim to CDD providers, and check that systematic CDD claim was added.
    // Then remove that DID from CDD provides, it should keep its previous CDD claim.
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let charlie_acc = AccountKeyring::Charlie.public();

    // 3.1. Add CDD claim to Charlie, by Alice.
    assert_ok!(Identity::cdd_register_did(
        alice.clone(),
        charlie_acc.clone(),
        vec![]
    ));
    let charlie_id =
        get_identity_id(AccountKeyring::Charlie).expect("Charlie should have an Identity Id");
    assert_add_cdd_claim!(alice, charlie_id);

    let charlie_cdd_claim =
        Identity::fetch_cdd(charlie_id, 0).expect("Charlie should have a CDD claim by Alice");

    // 3.2. Add Charlie as trusted CDD providers, and check its new systematic CDD claim.
    assert_ok!(CddServiceProviders::add_member(root.clone(), charlie_id));
    assert_eq!(fetch_systematic_claim(charlie_id).is_some(), true);

    // 3.3. Remove Charlie from trusted CDD providers, and verify that systematic CDD claim was
    //   removed and previous CDD claim works.
    assert_ok!(CddServiceProviders::remove_member(root, charlie_id));
    assert_eq!(fetch_systematic_claim(charlie_id).is_none(), true);
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
    let root = Origin::from(frame_system::RawOrigin::Root);
    let charlie_id = get_identity_id(AccountKeyring::Charlie)
        .expect("Charlie should be a Governance Committee member");
    let dave_id = get_identity_id(AccountKeyring::Dave)
        .expect("Dave should be a Governance Committee member");

    // 1. Each GC member has a *systematic* CDD claim.
    let governance_committee = GovernanceCommittee::get_members();
    assert_eq!(
        governance_committee
            .iter()
            .all(|gc_member| fetch_systematic_claim(*gc_member).is_some()),
        true
    );

    // 2. Remove one member from GC and double-check that systematic CDD claim was
    //    removed too.
    assert_ok!(GovernanceCommittee::remove_member(root.clone(), charlie_id));
    assert_eq!(fetch_systematic_claim(charlie_id).is_none(), true);
    assert_eq!(fetch_systematic_claim(dave_id).is_some(), true);

    // 3. Add DID with CDD claim to CDD providers, and check that systematic CDD claim was added.
    // Then remove that DID from CDD provides, it should keep its previous CDD claim.
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let ferdie_acc = AccountKeyring::Ferdie.public();

    // 3.1. Add CDD claim to Ferdie, by Alice.
    assert_ok!(Identity::cdd_register_did(
        alice.clone(),
        ferdie_acc.clone(),
        vec![]
    ));
    let ferdie_id =
        get_identity_id(AccountKeyring::Ferdie).expect("Ferdie should have an Identity Id");
    assert_add_cdd_claim!(alice, ferdie_id);

    let ferdie_cdd_claim =
        Identity::fetch_cdd(ferdie_id, 0).expect("Ferdie should have a CDD claim by Alice");

    // 3.2. Add Ferdie to GC, and check its new systematic CDD claim.
    assert_ok!(GovernanceCommittee::add_member(root.clone(), ferdie_id));
    assert_eq!(fetch_systematic_claim(ferdie_id).is_some(), true);

    // 3.3. Remove Ferdie from GC, and verify that systematic CDD claim was
    //   removed and previous CDD claim works.
    assert_ok!(GovernanceCommittee::remove_member(root, ferdie_id));
    assert_eq!(fetch_systematic_claim(ferdie_id).is_none(), true);
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
    let root = Origin::from(frame_system::RawOrigin::Root);
    let alice_id = get_identity_id(AccountKeyring::Alice)
        .expect("Alice should be a Governance Committee member");

    // 1. Alice should have 2 systematic CDD claims: One as GC member & another one as CDD
    //    provider.
    assert_eq!(fetch_systematic_gc(alice_id).is_some(), true);
    assert_eq!(fetch_systematic_cdd(alice_id).is_some(), true);

    // 2. Remove Alice from CDD providers.
    assert_ok!(CddServiceProviders::remove_member(root.clone(), alice_id));
    assert_eq!(fetch_systematic_gc(alice_id).is_some(), true);

    // 3. Remove Alice from GC.
    assert_ok!(GovernanceCommittee::remove_member(root, alice_id));
    assert_eq!(fetch_systematic_gc(alice_id).is_none(), true);
}

#[test]
fn add_permission_with_secondary_key() {
    ExtBuilder::default()
        .balance_factor(1_000)
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

            // SecondaryKey added
            let sig_1 = SecondaryKey {
                signer: Signatory::Account(bob_acc),
                permissions: Permissions::empty(),
            };

            let sig_2 = SecondaryKey {
                signer: Signatory::Account(charlie_acc),
                permissions: Permissions::empty(),
            };

            assert_ok!(Identity::cdd_register_did(
                Origin::signed(cdd_1_acc),
                alice_acc,
                vec![sig_1.clone().into(), sig_2.clone().into()]
            ));
            let alice_did = Identity::get_identity(&alice_acc).unwrap();
            assert_add_cdd_claim!(Origin::signed(cdd_1_acc), alice_did);

            let bob_auth_id = get_last_auth_id(&Signatory::Account(bob_acc));
            let charlie_auth_id = get_last_auth_id(&Signatory::Account(charlie_acc));

            println!("Print the protocol base fee: {:?}", PROTOCOL_OP_BASE_FEE);

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
            let sig_items = (Identity::did_records(alice_did)).secondary_keys;
            assert_eq!(sig_items[0], sig_1);
            assert_eq!(sig_items[1], sig_2);
        });
}

#[test]
fn add_investor_uniqueness_claim() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Charlie.public()])
        .build()
        .execute_with(do_add_investor_uniqueness_claim);
}

fn do_add_investor_uniqueness_claim() {
    let alice = User::new(AccountKeyring::Alice);
    let cdd_provider = AccountKeyring::Charlie.public();
    let ticker = an_asset(alice, true);
    let initial_balance = Asset::balance_of(ticker, alice.did);
    let add_iu_claim =
        |investor_uid| provide_scope_claim(alice.did, ticker, investor_uid, cdd_provider, Some(1));
    let no_balance_at_scope = |scope_id| {
        assert_eq!(
            false,
            <pallet_asset::BalanceOfAtScope<TestStorage>>::contains_key(scope_id, alice.did)
        );
    };
    let balance_at_scope = |scope_id, balance| {
        assert_eq!(balance, Asset::balance_of_at_scope(scope_id, alice.did));
    };
    let scope_id_of = |scope_id| {
        assert_eq!(scope_id, Asset::scope_id_of(ticker, alice.did));
    };
    let aggregate_balance = |scope_id, balance| {
        assert_eq!(balance, Asset::aggregate_balance_of(ticker, scope_id));
    };

    // Get some tokens for Alice in case the default initial balance changes to 0 in simple_token.
    let amount = 10_000;
    assert_ok!(Asset::issue(alice.origin(), ticker, amount));
    let asset_balance = initial_balance + amount;

    // Make a claim with a scope ID.
    let (scope_id, cdd_id) = add_iu_claim(alice.uid());
    balance_at_scope(scope_id, asset_balance);
    scope_id_of(scope_id);
    aggregate_balance(scope_id, asset_balance);

    // Revoke the first CDD claim in order to issue another one.
    assert_ok!(Identity::revoke_claim(
        Origin::signed(cdd_provider),
        alice.did,
        Claim::CustomerDueDiligence(cdd_id)
    ));

    // Make another claim with a different scope ID.
    let new_uid = InvestorUid::from("ALICE-2");
    // Adding a claim is possible thanks to the expiration of the previous CDD claim.
    let new_scope_id = add_iu_claim(new_uid).0;
    no_balance_at_scope(scope_id);
    balance_at_scope(new_scope_id, asset_balance);
    scope_id_of(new_scope_id);
    aggregate_balance(scope_id, 0);
    aggregate_balance(new_scope_id, asset_balance);
}

#[test]
fn add_investor_uniqueness_claim_v2() {
    let user = AccountKeyring::Alice.public();
    let user_no_cdd_id = AccountKeyring::Bob.public();

    ExtBuilder::default()
        .add_regular_users_from_accounts(&[user])
        .build()
        .execute_with(|| {
            // Create an DID without CDD_id.
            assert_ok!(Identity::_register_did(
                user_no_cdd_id,
                vec![],
                Some(ProtocolOp::IdentityRegisterDid)
            ));

            // Load test cases and run them.
            let test_data = add_investor_uniqueness_claim_v2_data(user, user_no_cdd_id);
            for (idx, (input, expect)) in test_data.into_iter().enumerate() {
                let (user, scope, claim, proof) = input;
                let did = Identity::get_identity(&user).unwrap_or_default();
                let origin = Origin::signed(user);
                let output = Identity::add_investor_uniqueness_claim_v2(
                    origin, did, scope, claim, proof.0, None,
                );
                assert_eq!(
                    output, expect,
                    "Unexpected output at index {}: output: {:?}, expected: {:?}",
                    idx, output, expect
                );
            }
        });
}

/// Creates a data set as an input for `do_add_investor_uniqueness_claim_v2`.
fn add_investor_uniqueness_claim_v2_data(
    user: Public,
    user_no_cdd_id: Public,
) -> Vec<(
    (Public, Scope, Claim, v2::InvestorZKProofData),
    DispatchResult,
)> {
    let ticker = Ticker::default();
    let did = Identity::get_identity(&user).unwrap();
    let investor: InvestorUid = make_investor_uid_v2(did.as_bytes()).into();
    let cdd_id = CddId::new_v2(did, investor.clone());
    let proof = v2::InvestorZKProofData::new(&did, &investor, &ticker);
    let claim = Claim::InvestorUniquenessV2(cdd_id);
    let scope = Scope::Ticker(ticker);
    let invalid_ticker = Ticker::try_from(&b"1"[..]).unwrap();
    let invalid_version_claim =
        Claim::InvestorUniqueness(Scope::Ticker(ticker), IdentityId::from(42u128), cdd_id);
    let invalid_proof = v2::InvestorZKProofData::new(&did, &investor, &invalid_ticker);

    vec![
        // Invalid claim.
        (
            (user, Scope::Ticker(invalid_ticker), claim.clone(), proof),
            Err(Error::InvalidScopeClaim.into()),
        ),
        // Valid ZKProof v2
        ((user, scope.clone(), claim.clone(), proof), Ok(())),
        // Not allowed claim.
        (
            (user, scope.clone(), Claim::NoData, proof),
            Err(Error::ClaimVariantNotAllowed.into()),
        ),
        // Missing CDD id.
        (
            (user_no_cdd_id, scope.clone(), claim.clone(), proof),
            Err(Error::ConfidentialScopeClaimNotAllowed.into()),
        ),
        // Invalid ZKProof
        (
            (user, scope.clone(), claim, invalid_proof),
            Err(Error::InvalidScopeClaim.into()),
        ),
        // Claim version does NOT match.
        (
            (user, scope.clone(), invalid_version_claim, proof),
            Err(Error::ClaimAndProofVersionsDoNotMatch.into()),
        ),
    ]
}

fn setup_join_identity(source: &User, target: &User) {
    assert_ok!(Identity::add_authorization(
        source.origin(),
        target.did.into(),
        AuthorizationData::JoinIdentity(Permissions::default()),
        None
    ));
    let auth_id = get_last_auth_id(&target.did.into());
    assert_ok!(Identity::join_identity(target.did.into(), auth_id));
}

#[test]
fn ext_join_identity_as_identity() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        // Check non-exist authorization
        assert_noop!(
            Identity::join_identity_as_identity(bob.origin(), 0),
            "Authorization does not exist"
        );

        // Add add authorization to later accept
        assert_ok!(Identity::add_authorization(
            alice.origin(),
            bob.did.into(),
            AuthorizationData::Custom(Ticker::default()),
            None
        ));

        // Try joining with wrong authorization type
        let auth_id = get_last_auth_id(&bob.did.into());
        assert_noop!(
            Identity::join_identity_as_identity(bob.origin(), auth_id),
            AuthorizationError::Invalid
        );

        setup_join_identity(&alice, &bob);
    });
}
#[test]
fn ext_leave_identity_as_identity() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);

        setup_join_identity(&alice, &bob);

        let leave = |u: User| Identity::leave_identity_as_identity(u.origin(), alice.did);
        // Try to leave own identity
        assert_noop!(leave(alice), Error::NotASigner);
        // Try to leave a identity that has no signers
        assert_noop!(leave(charlie), Error::NotASigner);
        assert_ok!(leave(bob));
    });
}
