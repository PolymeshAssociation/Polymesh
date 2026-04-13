use super::{
    asset_pallet::setup::create_and_issue_sample_asset,
    asset_test::{max_len, max_len_bytes, set_timestamp},
    committee_test::gc_vmo,
    ext_builder::PROTOCOL_OP_BASE_FEE,
    multisig::{create_multisig_default_perms, create_signers},
    storage::{
        account_from, add_secondary_key, add_secondary_key_with_perms, get_identity_id,
        get_last_auth_id, get_primary_key, get_secondary_keys, register_keyring_account,
        register_keyring_account_with_balance, GovernanceCommittee, TestStorage, User,
    },
    ExtBuilder,
};
use codec::Encode;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchResult, traits::Currency};
use pallet_balances as balances;
use pallet_identity::{
    Authorizations, CurrentAuthId, CustomClaimIdSequence, CustomClaims, CustomClaimsInverse,
    OffChainAuthorizationNonce,
};
use pallet_identity::{Config as IdentityConfig, Event};
use polymesh_primitives::identity::SecondaryKeyWithAuth;
use polymesh_primitives::{
    asset::AssetId,
    crypto::{ChainScopedMessage, IDENTITY_ADD_SECONDARY_KEY_LABEL},
};
use polymesh_primitives::{
    constants::currency::POLY, crypto::BytesWrapped, traits::group::GroupTrait,
    traits::CurrentFeePayer, AccountId, AssetPermissions, AuthorizationData, AuthorizationType,
    Claim, ClaimType, CustomClaimTypeId, ExtrinsicName, ExtrinsicPermissions, IdentityClaim,
    IdentityId, KeyRecord, PalletName, PalletPermissions, Permissions, PortfolioId,
    PortfolioNumber, Scope, SecondaryKey, Signatory, SubsetRestriction, SystematicIssuers, Ticker,
    GC_DID,
};
use polymesh_runtime_develop::runtime::{RuntimeCall, TxFeeHandler};
use sp_core::H512;
use sp_keyring::Sr25519Keyring;
use sp_std::iter;
use std::convert::From;

type AuthorizationsGiven = pallet_identity::AuthorizationsGiven<TestStorage>;
type Asset = pallet_asset::Pallet<TestStorage>;
type Balances = balances::Pallet<TestStorage>;
type BaseError = pallet_base::Error<TestStorage>;
type Identity = pallet_identity::Pallet<TestStorage>;
type MultiSig = pallet_multisig::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type DidRegistrars = <TestStorage as IdentityConfig>::DidRegistrars;
type Error = pallet_identity::Error<TestStorage>;
type PError = pallet_permissions::Error<TestStorage>;

const MAX_ASSETS: u64 = 2000;
const MAX_PORTFOLIOS: u64 = 2000;

// Identity Test Helper functions
// =======================================

/// Utility function to fetch *only* systematic CDD claims.
///
/// We have 2 systematic CDD claims issuers:
/// * Committee group (used for upgrade, technical & GC members)
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

fn scoped_target_id_auth(user: &User) -> (ChainScopedMessage<TestStorage, IdentityId>, u64) {
    let expires_at = 100u64;
    let nonce = OffChainAuthorizationNonce::<TestStorage>::get(user.did);
    let message = ChainScopedMessage::new_unchecked(
        nonce,
        IDENTITY_ADD_SECONDARY_KEY_LABEL,
        expires_at,
        user.did,
    );
    (message, expires_at)
}

macro_rules! assert_add_cdd_claim {
    ($signer:expr, $target:expr) => {
        assert_ok!(Identity::add_claim(
            $signer,
            $target,
            Claim::CustomerDueDiligence(Default::default()),
            None
        ));
    };
}

// Tests
// ======================================
#[test]
fn only_primary_or_secondary_keys_can_authenticate_as_an_identity() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);

        let bob = User::new(Sr25519Keyring::Bob);

        // Add charlie's key as a secondary key of bob.
        let charlie = User::new_with(bob.did, Sr25519Keyring::Charlie);
        add_secondary_key(bob.did, charlie.acc());

        // Check primary key.
        assert!(Identity::is_key_authorized(alice.did, &alice.acc()));
        // Check secondary_keys.
        assert!(Identity::is_key_authorized(bob.did, &charlie.acc()));

        // Remove charlie's key from the secondary keys of bob.
        assert_ok!(Identity::remove_secondary_keys(
            bob.origin(),
            vec![charlie.acc()]
        ));

        // Verify the secondary key was removed.
        assert_eq!(Identity::is_key_authorized(bob.did, &charlie.acc()), false);
    });
}

#[test]
fn gc_add_remove_cdd_claim() {
    ExtBuilder::default().build().execute_with(|| {
        let target = User::new(Sr25519Keyring::Charlie);
        let fetch =
            || Identity::fetch_claim(target.did, ClaimType::CustomerDueDiligence, GC_DID, None);

        assert_ok!(Identity::gc_add_cdd_claim(gc_vmo(), target.did));

        assert_eq!(
            fetch(),
            Some(IdentityClaim {
                claim_issuer: GC_DID,
                issuance_date: 0,
                last_update_date: 0,
                expiry: None,
                claim: Claim::CustomerDueDiligence(Default::default())
            })
        );

        assert_ok!(Identity::gc_revoke_cdd_claim(gc_vmo(), target.did));
        assert_eq!(fetch(), None);
    });
}

#[test]
fn revoking_claims() {
    ExtBuilder::default().build().execute_with(|| {
        let _owner = User::new(Sr25519Keyring::Alice);
        let _issuer = User::new(Sr25519Keyring::Bob);
        let claim_issuer = User::new(Sr25519Keyring::Charlie);
        let scope = Scope::from(IdentityId::from(0));

        let fetch = || {
            Identity::fetch_claim(
                claim_issuer.did,
                ClaimType::Accredited,
                claim_issuer.did,
                Some(scope.clone()),
            )
        };
        assert_ok!(Identity::add_claim(
            claim_issuer.origin(),
            claim_issuer.did,
            Claim::Accredited(scope.clone()),
            Some(100u64),
        ));
        assert!(fetch().is_some());

        assert_ok!(Identity::revoke_claim(
            claim_issuer.origin(),
            claim_issuer.did,
            Claim::Accredited(scope.clone()),
        ));
        assert!(fetch().is_none());
    });
}

#[test]
fn revoking_batch_claims() {
    ExtBuilder::default().build().execute_with(|| {
        let _owner = User::new(Sr25519Keyring::Alice);
        let _issuer = User::new(Sr25519Keyring::Bob);
        let claim_issuer = User::new(Sr25519Keyring::Charlie);
        let scope = Scope::from(IdentityId::from(0));

        let add = |claim, scope| {
            Identity::add_claim(claim_issuer.origin(), claim_issuer.did, claim, scope)
        };
        let fetch =
            |claim, scope| Identity::fetch_claim(claim_issuer.did, claim, claim_issuer.did, scope);

        let revoke = |claim| Identity::revoke_claim(claim_issuer.origin(), claim_issuer.did, claim);

        assert_ok!(add(Claim::Accredited(scope.clone()), Some(100u64)));
        assert_ok!(add(Claim::Blocked(scope.clone()), None));
        assert!(fetch(ClaimType::Accredited, Some(scope.clone())).is_some());

        assert!(fetch(ClaimType::Blocked, Some(scope.clone())).is_some());
        assert!(fetch(ClaimType::Accredited, Some(scope.clone())).is_some());

        assert_ok!(revoke(Claim::Accredited(scope.clone())));

        assert_ok!(revoke(Claim::Blocked(scope.clone())));

        assert!(fetch(ClaimType::Accredited, Some(scope.clone())).is_none());

        assert!(fetch(ClaimType::Blocked, Some(scope.clone())).is_none());

        assert!(fetch(ClaimType::Accredited, Some(scope.clone())).is_none());
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
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new_with(alice.did, Sr25519Keyring::Bob);
    let charlie = User::new_with(alice.did, Sr25519Keyring::Charlie);

    add_secondary_key(alice.did, charlie.acc());
    add_secondary_key(alice.did, bob.acc());

    let set = |by: User, to: User, perms| {
        Identity::set_secondary_key_permissions(by.origin(), to.acc(), perms)
    };

    // Only `alice` is able to update `bob`'s permissions and `charlie`'s permissions.
    assert_ok!(set(alice, bob, Permissions::empty()));
    assert_ok!(set(alice, charlie, Permissions::empty()));

    // Bob tries to get better permission by himself at `alice` Identity.
    assert_noop!(set(bob, bob, Permissions::default()), Error::KeyNotAllowed);

    // Bob tries to remove Charlie's permissions at `alice` Identity.
    assert_noop!(
        set(bob, charlie, Permissions::empty()),
        Error::KeyNotAllowed
    );

    // Alice overwrites some permissions.
    assert_ok!(set(alice, bob, Permissions::empty()));
}

#[test]
fn add_permissions_to_multiple_tokens() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_add_permissions_to_multiple_tokens);
}
fn do_add_permissions_to_multiple_tokens() {
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new_with(alice.did, Sr25519Keyring::Bob);

    // Add bob with default permissions.
    add_secondary_key(alice.did, bob.acc());

    // Create some tokens.
    let max_tokens = 20;
    let tokens: Vec<AssetId> = (0..max_tokens)
        .map(|_| create_and_issue_sample_asset(&alice))
        .collect();

    let test_set_perms = |asset| {
        assert_ok!(Identity::set_secondary_key_permissions(
            alice.origin(),
            bob.acc(),
            Permissions {
                asset,
                ..Default::default()
            },
        ));
    };

    // add one-by-one.
    for num in 0..max_tokens {
        test_set_perms(AssetPermissions::elems(tokens[0..num].into_iter().cloned()));
    }

    // remove all permissions.
    test_set_perms(AssetPermissions::empty());

    // bulk add in reverse order.
    test_set_perms(AssetPermissions::elems(
        tokens[0..max_tokens].into_iter().rev().cloned(),
    ));
}

#[test]
fn set_secondary_key_permissions_with_bad_perms() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new_with(alice.did, Sr25519Keyring::Bob);
        add_secondary_key(alice.did, bob.acc());
        test_with_bad_perms(alice.did, |perms| {
            assert_too_long!(Identity::set_secondary_key_permissions(
                alice.origin(),
                bob.acc(),
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
    let alice = User::new(Sr25519Keyring::Alice);
    let [bob, charlie, dave] = [
        Sr25519Keyring::Bob,
        Sr25519Keyring::Charlie,
        Sr25519Keyring::Dave,
    ]
    .map(|r| User::new_with(alice.did, r));

    let is_auth = |key| Identity::is_key_authorized(alice.did, &key);

    // Add secondary keys.
    add_secondary_key(alice.did, bob.acc());
    add_secondary_key(alice.did, charlie.acc());

    assert_eq!(is_auth(bob.acc()), true);

    // Freeze secondary keys: bob & charlie.
    assert_noop!(
        Identity::freeze_secondary_keys(bob.origin()),
        Error::KeyNotAllowed
    );
    assert_ok!(Identity::freeze_secondary_keys(alice.origin()));

    assert_eq!(is_auth(bob.acc()), false);

    add_secondary_key(alice.did, dave.acc());

    // update permission of frozen keys.
    assert_ok!(Identity::set_secondary_key_permissions(
        alice.origin(),
        bob.acc(),
        Permissions::default().into(),
    ));

    // unfreeze all
    assert_noop!(
        Identity::unfreeze_secondary_keys(bob.origin()),
        Error::KeyNotAllowed
    );
    assert_ok!(Identity::unfreeze_secondary_keys(alice.origin()));

    assert_eq!(is_auth(dave.acc()), true);
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
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new_with(alice.did, Sr25519Keyring::Bob);
    let charlie = User::new_with(alice.did, Sr25519Keyring::Charlie);

    // Add secondary keys.
    add_secondary_key(alice.did, bob.acc());
    add_secondary_key(alice.did, charlie.acc());

    // Freeze all secondary keys
    assert_ok!(Identity::freeze_secondary_keys(alice.origin()));

    // Remove Bob's key.
    assert_ok!(Identity::remove_secondary_keys(
        alice.origin(),
        vec![bob.acc()]
    ));
    // Check DidRecord.
    assert_eq!(
        get_secondary_keys(alice.did),
        vec![SecondaryKey::new(charlie.acc(), Permissions::default())]
    );
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
    let alice = Sr25519Keyring::Alice.to_account_id();
    let bob = Sr25519Keyring::Bob.to_account_id();
    let charlie = Sr25519Keyring::Charlie.to_account_id();
    TestStorage::set_payer_context(Some(alice.clone()));
    let alice_id = register_keyring_account(Sr25519Keyring::Alice).unwrap();
    TestStorage::set_payer_context(Some(charlie.clone()));
    let _charlie_id = register_keyring_account_with_balance(Sr25519Keyring::Charlie, 100).unwrap();
    assert_eq!(Balances::free_balance(charlie.clone()), 100);

    // 1. Add Bob as signatory to Alice ID.
    TestStorage::set_payer_context(Some(alice.clone()));
    add_secondary_key(alice_id, bob.clone());
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice.clone()),
        bob.clone().into(),
        25_000,
        None
    ));
    assert_eq!(Balances::free_balance(bob.clone()), 25_000);

    // 2. Bob can transfer some funds to Charlie ID.
    TestStorage::set_payer_context(Some(bob.clone()));
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(bob.clone()),
        charlie.clone().into(),
        1_000,
        None
    ));
    assert_eq!(Balances::free_balance(charlie.clone()), 1100);

    // 3. Alice freezes her secondary keys.
    assert_ok!(Identity::freeze_secondary_keys(Origin::signed(
        alice.clone()
    )));

    // 4. Bob should be able to transfer any amount. SE is simulated.
    // Balances::transfer_with_memo(Origin::signed(bob), charlie, 1_000, None),
    let payer = TxFeeHandler::get_valid_payer(
        &RuntimeCall::Balances(balances::Call::transfer_with_memo {
            dest: Sr25519Keyring::Charlie.to_account_id().into(),
            value: 1_000,
            memo: None,
        }),
        Sr25519Keyring::Bob.to_account_id(),
    );
    assert!(payer.is_ok());

    assert_eq!(Balances::free_balance(charlie.clone()), 1100);

    // 5. Alice still can make transfers.
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice.clone()),
        charlie.clone().into(),
        1_000,
        None
    ));
    assert_eq!(Balances::free_balance(charlie.clone()), 2100);

    // 6. Unfreeze signatory keys, and Bob should be able to transfer again.
    assert_ok!(Identity::unfreeze_secondary_keys(Origin::signed(alice)));
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(bob),
        charlie.clone().into(),
        1_000,
        None
    ));
    assert_eq!(Balances::free_balance(charlie), 3100);
}

fn run_add_secondary_key_with_perm_test(
    add_key_with_perms: impl Fn(User, Permissions) -> DispatchResult,
) {
    let alice = User::new(Sr25519Keyring::Alice);

    let perm1 = Permissions::empty();
    let perm2 = Permissions::from_pallet_permissions(vec![PalletPermissions::entire_pallet(
        "identity".into(),
    )]);

    // Count alice's secondary keys.
    let count_keys = || get_secondary_keys(alice.did).len();

    // Add bob's identity signatory with empty permissions
    let res = add_key_with_perms(alice, perm1.clone());
    assert_ok!(res);
    assert_eq!(count_keys(), 1);

    // Add bob's identity signatory again with non-empty permissions
    let res = add_key_with_perms(alice, perm2.clone());
    assert_noop!(res, Error::AlreadyLinked);
    assert_eq!(count_keys(), 1);

    // Add bob's identity signatory again.
    let res = add_key_with_perms(alice, perm1.clone());
    assert_noop!(res, Error::AlreadyLinked);
    assert_eq!(count_keys(), 1);
}

#[test]
fn join_identity_as_key_with_perm_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_join_identity_as_key_with_perm_test);
}

fn do_join_identity_as_key_with_perm_test() {
    // Use `add_auth` and `join_identity_as_key` to add a secondary key.
    run_add_secondary_key_with_perm_test(move |alice, perms| {
        let bob = User::new_with(alice.did, Sr25519Keyring::Bob);
        let data = AuthorizationData::JoinIdentity(perms);
        let auth_id = Identity::add_auth(alice.did, bob.signatory_acc(), data, None).unwrap();
        Identity::join_identity_as_key(bob.origin(), auth_id)
    })
}

#[test]
fn add_secondary_keys_with_permissions_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_add_secondary_keys_with_permissions_test);
}

fn do_add_secondary_keys_with_permissions_test() {
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new_with(alice.did, Sr25519Keyring::Bob);

    // Add bob with default permissions
    add_secondary_key(alice.did, bob.acc());

    let permissions = Permissions::from_pallet_permissions(vec![PalletPermissions::entire_pallet(
        "identity".into(),
    )]);
    // Try adding bob again with custom permissions
    let auth_id = Identity::add_auth(
        alice.did,
        bob.signatory_acc(),
        AuthorizationData::JoinIdentity(permissions.clone()),
        None,
    )
    .unwrap();
    assert_eq!(
        Identity::join_identity(bob.origin(), auth_id),
        Err(Error::AlreadyLinked.into()),
    );

    // Try addind the same secondary_key using `add_secondary_keys_with_authorization`
    let (authorization, expires_at) = scoped_target_id_auth(&alice);
    let signature = authorization
        .sign(&alice.ring)
        .expect("Signing should not fail");
    let auth_signature = H512::from(signature.as_ref());

    let key_with_auth = SecondaryKeyWithAuth {
        auth_signature,
        secondary_key: SecondaryKey::new(bob.acc(), permissions).into(),
    };
    assert_noop!(
        Identity::add_secondary_keys_with_authorization(
            alice.origin(),
            vec![key_with_auth],
            expires_at
        ),
        Error::AlreadyLinked
    );

    // Check KeyRecords map
    assert_eq!(Identity::get_identity(&bob.acc()), Some(alice.did));

    // Check DidRecords
    let keys = get_secondary_keys(alice.did);
    assert_eq!(keys.len(), 1);

    // Try remove bob using alice
    assert_ok!(Identity::remove_secondary_keys(
        alice.origin(),
        vec![bob.acc()]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&bob.acc()), None);

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
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new_with(alice.did, Sr25519Keyring::Bob);
    let dave = User::new_with(alice.did, Sr25519Keyring::Dave);

    add_secondary_key(alice.did, bob.acc());
    add_secondary_key(alice.did, dave.acc());

    let did_of = |u: User| Identity::get_identity(&u.acc());

    // Check KeyRecords map
    assert_eq!(did_of(bob), Some(alice.did));
    assert_eq!(did_of(dave), Some(alice.did));

    // Check DidRecords
    let keys = get_secondary_keys(alice.did);
    assert_eq!(keys.len(), 2);

    // Try removing bob using alice.
    let remove_sk = |u: User| Identity::remove_secondary_keys(alice.origin(), vec![u.acc()]);
    assert_ok!(remove_sk(bob));

    // Try changing the permissions for bob's key.
    // This should fail.
    let result = Identity::set_secondary_key_permissions(
        alice.origin(),
        bob.acc(),
        Permissions::from_pallet_permissions(vec![PalletPermissions::entire_pallet(
            "identity".into(),
        )])
        .into(),
    );
    assert_noop!(result, Error::NotASigner);

    // Check DidRecords.
    let keys = get_secondary_keys(alice.did);
    assert_eq!(keys.len(), 1);

    // Check the identity map.
    assert_eq!(did_of(bob), None);
    assert_eq!(did_of(dave), Some(alice.did));

    // Try re-adding bob's key.
    add_secondary_key(alice.did, bob.acc());

    // Check identity map
    assert_eq!(did_of(bob), Some(alice.did));

    // Remove bob's key again.
    assert_ok!(remove_sk(bob));

    // Try removing dave using alice.
    assert_ok!(remove_sk(dave));

    // Check the identity map.
    assert_eq!(did_of(dave), None);

    // Check DidRecords.
    let keys = get_secondary_keys(alice.did);
    assert_eq!(keys.len(), 0);
}

#[test]
fn remove_secondary_keys_test_with_externalities() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_remove_secondary_keys_test_with_externalities);
}

fn do_remove_secondary_keys_test_with_externalities() {
    let bob_key = Sr25519Keyring::Bob.to_account_id();
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new_with(alice.did, Sr25519Keyring::Bob);
    let charlie = User::new(Sr25519Keyring::Charlie);
    let dave_key = Sr25519Keyring::Dave.to_account_id();
    let ferdie_key = Sr25519Keyring::Ferdie.to_account_id();

    let ms_address = create_multisig_default_perms(
        alice.acc(),
        create_signers(vec![ferdie_key.clone(), dave_key.clone()]),
        1,
    );
    let auth_id = get_last_auth_id(&Signatory::Account(dave_key.clone()));
    assert_ok!(MultiSig::accept_multisig_signer(
        Origin::signed(dave_key.clone()),
        auth_id
    ));

    add_secondary_key(alice.did, bob.acc());

    // Fund the multisig
    assert_ok!(Balances::transfer_allow_death(
        alice.origin(),
        ms_address.clone().into(),
        2 * POLY
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));
    assert_eq!(Identity::get_identity(&bob_key), Some(alice.did));

    // Try removing bob using charlie
    assert_noop!(
        Identity::remove_secondary_keys(charlie.origin(), vec![bob.acc()]),
        Error::NotASigner
    );

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));
    assert_eq!(Identity::get_identity(&bob_key), Some(alice.did));

    // Try remove bob using alice
    assert_ok!(Identity::remove_secondary_keys(
        alice.origin(),
        vec![bob.acc()]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(ms_address.clone(), dave_key.clone()),
        true
    );

    // Transfer funds back to Alice
    assert_ok!(Balances::transfer_allow_death(
        Origin::signed(ms_address.clone()),
        alice.acc().into(),
        2 * POLY
    ));

    // Empty multisig's funds and remove as signer
    assert_ok!(Identity::remove_secondary_keys(
        alice.origin(),
        vec![ms_address.clone()]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), None);
    assert_eq!(Identity::get_identity(&bob.acc()), None);

    // Check multisig's signer
    assert_eq!(MultiSig::ms_signers(ms_address.clone(), dave_key), true);
}

#[test]
fn leave_identity_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&leave_identity_test_with_externalities);
}

fn leave_identity_test_with_externalities() {
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new_with(alice.did, Sr25519Keyring::Bob);

    let bob_sk = SecondaryKey::new(bob.acc(), Permissions::empty());
    let dave_key = Sr25519Keyring::Dave.to_account_id();
    let ferdie_key = Sr25519Keyring::Ferdie.to_account_id();

    let ms_address = create_multisig_default_perms(
        alice.acc(),
        create_signers(vec![ferdie_key.clone(), dave_key.clone()]),
        1,
    );
    let ms_sk = SecondaryKey::new(ms_address.clone(), Permissions::default());
    let auth_id = get_last_auth_id(&Signatory::Account(dave_key.clone()));
    assert_ok!(MultiSig::accept_multisig_signer(
        Origin::signed(dave_key.clone()),
        auth_id
    ));

    add_secondary_key_with_perms(alice.did, bob.acc(), Permissions::empty());

    // Check DidRecord.
    assert_eq!(get_secondary_keys(alice.did), vec![bob_sk, ms_sk]);
    assert_eq!(Identity::get_identity(&bob.acc()), Some(alice.did));

    // Bob leaves
    assert_ok!(Identity::leave_identity_as_key(bob.origin()));

    // Check DidRecord.
    assert_eq!(get_secondary_keys(alice.did).len(), 1);
    assert_eq!(Identity::get_identity(&bob.acc()), None);
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));

    // send funds to multisig
    assert_ok!(Balances::transfer_allow_death(
        alice.origin(),
        ms_address.clone().into(),
        2 * POLY
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(ms_address.clone(), dave_key.clone()),
        true
    );

    // send funds back to alice from multisig
    assert_ok!(Balances::transfer_allow_death(
        Origin::signed(ms_address.clone()),
        alice.acc().into(),
        2 * POLY
    ));

    // Empty multisig's funds and remove as signer
    assert_ok!(Identity::leave_identity_as_key(Origin::signed(
        ms_address.clone()
    )));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), None);

    // Check multisig's signer
    assert_eq!(MultiSig::ms_signers(ms_address.clone(), dave_key), true);
}

#[test]
fn enforce_uniqueness_keys_in_identity_tests() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&enforce_uniqueness_keys_in_identity);
}

fn enforce_uniqueness_keys_in_identity() {
    // Register identities.
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new(Sr25519Keyring::Bob);

    // Check external signed key uniqueness.
    let charlie = User::new_with(alice.did, Sr25519Keyring::Charlie);
    add_secondary_key(alice.did, charlie.acc());
    let auth_id = Identity::add_auth(
        alice.did,
        bob.signatory_acc(),
        AuthorizationData::JoinIdentity(Permissions::empty()),
        None,
    )
    .unwrap();
    assert_eq!(
        Identity::join_identity(bob.origin(), auth_id),
        Err(Error::AlreadyLinked.into()),
    );
}

fn secondary_keys_with_auth(
    users: &[User],
    auth_encoded: &[u8],
) -> Vec<SecondaryKeyWithAuth<AccountId>> {
    users
        .iter()
        .map(|user| SecondaryKeyWithAuth {
            auth_signature: H512::from(user.ring.sign(&auth_encoded)),
            secondary_key: SecondaryKey::from_account_id(user.acc()).into(),
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
    let a = User::new(Sr25519Keyring::Alice);

    let (scoped_auth, expires_at) = scoped_target_id_auth(&a);
    let auth_encoded = BytesWrapped(&scoped_auth).encode();

    let keys = [
        Sr25519Keyring::Bob,
        Sr25519Keyring::Charlie,
        Sr25519Keyring::Dave,
    ];
    let users @ [b, c, _] = keys.map(|r| User::new_with(a.did, r));
    let secondary_keys_with_auth = secondary_keys_with_auth(&users, &auth_encoded);

    let add = |user: User, auth| {
        Identity::add_secondary_keys_with_authorization(user.origin(), auth, expires_at)
    };

    assert_ok!(add(a, secondary_keys_with_auth[..2].to_owned()));

    let secondary_keys = get_secondary_keys(a.did);
    let contains = |u: User| secondary_keys.iter().any(|si| si.key == u.acc());
    assert_eq!(contains(b), true);
    assert_eq!(contains(c), true);

    // Check reply attack. Alice's nonce is different now.
    // NOTE: We need to force the increment of account's nonce manually.
    System::inc_account_nonce(a.acc());

    assert_noop!(
        add(a, secondary_keys_with_auth[2..].to_owned()),
        Error::InvalidAuthorizationSignature
    );

    // Check expire
    System::inc_account_nonce(a.acc());
    set_timestamp(expires_at);

    let f = User::new(Sr25519Keyring::Ferdie);
    let (ferdie_auth, _) = scoped_target_id_auth(&a);
    let signature = ferdie_auth
        .sign(&f.ring)
        .expect("Failed to sign the authorization");
    let ferdie_secondary_key_with_auth = SecondaryKeyWithAuth {
        secondary_key: SecondaryKey::from_account_id(f.acc()).into(),
        auth_signature: H512::from(signature.as_ref()),
    };

    assert_noop!(
        add(f, vec![ferdie_secondary_key_with_auth]),
        Error::AuthorizationExpired
    );
}

pub(crate) fn max_len_string<R: From<String>>(add: u32) -> R {
    iter::repeat('A')
        .take((max_len() + add) as usize)
        .collect::<String>()
        .into()
}

pub(crate) fn test_with_bad_ext_perms(test: impl Fn(ExtrinsicPermissions)) {
    test(ExtrinsicPermissions::these(
        (0..=max_len() as u64)
            .map(PalletName::generate)
            .map(PalletPermissions::entire_pallet),
    ));
    test(ExtrinsicPermissions::these([
        PalletPermissions::entire_pallet(max_len_string(1)),
    ]));
    test(ExtrinsicPermissions::these([PalletPermissions::new(
        "".into(),
        SubsetRestriction::elems((0..=max_len() as u64).map(ExtrinsicName::generate)),
    )]));
    test(ExtrinsicPermissions::these([PalletPermissions::new(
        "".into(),
        SubsetRestriction::elem(max_len_string(1)),
    )]));
}

pub(crate) fn test_with_bad_perms(did: IdentityId, test: impl Fn(Permissions)) {
    test(Permissions {
        asset: SubsetRestriction::elems(
            (0..=MAX_ASSETS)
                .map(|i| AssetId::new([i.to_le_bytes(), [0; 8]].concat().try_into().unwrap())),
        ),
        ..<_>::default()
    });
    test(Permissions {
        portfolio: SubsetRestriction::elems(
            (0..=MAX_PORTFOLIOS)
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

// Test for DuplicateKey error in `add_secondary_keys_with_authorization`.
#[test]
fn add_secondary_keys_with_authorization_duplicate_keys() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(Sr25519Keyring::Alice);
        let bob = User::new_with(user.did, Sr25519Keyring::Bob);

        let expires_at = 100;
        let nonce = OffChainAuthorizationNonce::<TestStorage>::get(user.did);
        let scoped = ChainScopedMessage::<TestStorage, _>::new_unchecked(
            nonce,
            IDENTITY_ADD_SECONDARY_KEY_LABEL,
            expires_at,
            &user.did,
        );

        let signature = scoped
            .sign(&bob.ring)
            .expect("Failed to sign the authorization");
        let auth_signature = H512::from(signature);

        let secondary_key = SecondaryKey::new(bob.acc(), Permissions::default());
        let auths = vec![
            SecondaryKeyWithAuth {
                auth_signature,
                secondary_key: secondary_key.clone(),
            },
            SecondaryKeyWithAuth {
                auth_signature,
                secondary_key,
            },
        ];
        assert_noop!(
            Identity::add_secondary_keys_with_authorization(user.origin(), auths, expires_at),
            Error::DuplicateKey
        );
    });
}

#[test]
fn add_secondary_keys_with_authorization_too_many_sks() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(Sr25519Keyring::Alice);
        let bob = User::new_with(user.did, Sr25519Keyring::Bob);

        let (scoped_auth, expires_at) = scoped_target_id_auth(&user);
        let auth_encoded = BytesWrapped(&scoped_auth).encode();

        // Test various length problems in `Permissions`.
        test_with_bad_perms(user.did, |perms| {
            let auth_signature = H512::from(bob.ring.sign(&auth_encoded));

            let secondary_key = SecondaryKey::new(bob.acc(), perms);
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

        // Populate with MAX_LEN secondary keys.
        let key_record = KeyRecord::SecondaryKey(user.did);
        let perms = Permissions::empty();
        for idx in 0..max_len() {
            let key = account_from(idx as u64 + 100);
            Identity::add_key_record(&key, key_record.clone());
            Identity::set_key_permissions(&key, &perms);
        }

        // No Secondary key limit.
        assert_ok!(Identity::add_secondary_keys_with_authorization(
            user.origin(),
            secondary_keys_with_auth(&[bob], &auth_encoded),
            expires_at
        ));
    });
}

#[test]
#[allow(deprecated)]
fn secondary_key_with_bad_permissions() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .did_registrars(vec![
            Sr25519Keyring::Eve.to_account_id(),
            Sr25519Keyring::Ferdie.to_account_id(),
        ])
        .build()
        .execute_with(|| {
            let cdd1 = Sr25519Keyring::Eve.to_account_id();
            let alice = User::new(Sr25519Keyring::Alice);
            let bob = User::new_with(alice.did, Sr25519Keyring::Bob);

            test_with_bad_perms(bob.did, |perms| {
                let bob_sk = SecondaryKey::new(bob.acc(), perms);
                assert_noop!(
                    Identity::cdd_register_did(
                        Origin::signed(cdd1.clone()),
                        alice.acc(),
                        vec![bob_sk]
                    ),
                    pallet_base::Error::<TestStorage>::TooLong
                );
            });
        });
}

#[test]
fn adding_authorizations_bad_perms() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(Sr25519Keyring::Alice);
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
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let ticker50 = Ticker::from_slice_truncated(&[0x50][..]);
        let mut auth_id = Identity::add_auth(
            alice.did,
            bob.signatory_did(),
            AuthorizationData::TransferTicker(ticker50),
            None,
        )
        .unwrap();
        assert_eq!(
            AuthorizationsGiven::get(alice.did, auth_id),
            bob.signatory_did(),
        );
        let mut auth = Authorizations::<TestStorage>::get(&bob.signatory_did(), auth_id)
            .expect("Missing authorization");
        assert_eq!(auth.authorized_by, alice.did);
        assert_eq!(auth.expiry, None);
        assert_eq!(
            auth.authorization_data,
            AuthorizationData::TransferTicker(ticker50)
        );
        auth_id = Identity::add_auth(
            alice.did,
            bob.signatory_did(),
            AuthorizationData::TransferTicker(ticker50),
            Some(100),
        )
        .unwrap();
        assert_eq!(
            AuthorizationsGiven::get(alice.did, auth_id),
            bob.signatory_did()
        );
        auth = Authorizations::<TestStorage>::get(&bob.signatory_did(), auth_id)
            .expect("Missing authorization");
        assert_eq!(auth.authorized_by, alice.did);
        assert_eq!(auth.expiry, Some(100));
        assert_eq!(
            auth.authorization_data,
            AuthorizationData::TransferTicker(ticker50)
        );

        // Testing the list of filtered authorizations
        set_timestamp(120);

        // Getting expired and non-expired both
        let mut authorizations = Identity::get_filtered_authorizations(
            bob.signatory_did(),
            true,
            Some(AuthorizationType::TransferTicker),
        );
        assert_eq!(authorizations.len(), 2);
        authorizations = Identity::get_filtered_authorizations(
            bob.signatory_did(),
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
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let ticker50 = Ticker::from_slice_truncated(&[0x50][..]);
        let auth_id = Identity::add_auth(
            alice.did,
            bob.signatory_did(),
            AuthorizationData::TransferTicker(ticker50),
            None,
        )
        .unwrap();
        assert_eq!(
            AuthorizationsGiven::get(alice.did, auth_id),
            bob.signatory_did()
        );
        let auth = Authorizations::<TestStorage>::get(&bob.signatory_did(), auth_id)
            .expect("Missing authorization");
        assert_eq!(
            auth.authorization_data,
            AuthorizationData::TransferTicker(ticker50)
        );
        assert_ok!(Identity::remove_authorization(
            alice.origin(),
            bob.signatory_did(),
            auth_id,
            false,
        ));
        assert!(!AuthorizationsGiven::contains_key(alice.did, auth_id));
        assert!(!Authorizations::<TestStorage>::contains_key(
            bob.signatory_did(),
            auth_id
        ));
    });
}

#[test]
fn changing_primary_key() {
    ExtBuilder::default()
        .monied(true)
        .did_registrars(vec![Sr25519Keyring::Eve.to_account_id()])
        .build()
        .execute_with(changing_primary_key_we);
}

fn changing_primary_key_we() {
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new(Sr25519Keyring::Bob);

    // Primary key matches Alice's key
    let alice_pk = || get_primary_key(alice.did);
    assert_eq!(alice_pk(), alice.acc());

    let add = |ring: Sr25519Keyring| {
        Identity::add_auth(
            alice.did,
            Signatory::Account(ring.to_account_id()),
            AuthorizationData::RotatePrimaryKey,
            None,
        )
        .unwrap()
    };
    let accept = |ring: Sr25519Keyring, auth| {
        Identity::accept_primary_key(Origin::signed(ring.to_account_id()), auth)
    };

    // In the case of a key belong to key DID for which we're rotating, we don't allow rotation.
    let auth = add(alice.ring);
    assert_eq!(accept(alice.ring, auth), Err(Error::AlreadyLinked.into()));

    // Add and accept auth with new key to become new primary key.
    // Unfortunately, the new key is still linked to Bob's DID.
    let auth = add(bob.ring);
    assert_eq!(accept(bob.ring, auth), Err(Error::AlreadyLinked.into()));

    // Do it again, but for Charlie, who isn't attached to a DID.
    // Alice's primary key will be Charlie's.'
    let charlie = Sr25519Keyring::Charlie;
    assert_ok!(accept(charlie, add(charlie)));
    assert_eq!(alice_pk(), charlie.to_account_id());
    assert_ok!(Identity::ensure_key_did_unlinked(&alice.acc()));

    // Add Alice's old primary key as a secondary key to the identity..
    let join_auth = Identity::add_auth(
        alice.did,
        Signatory::Account(alice.acc()),
        AuthorizationData::JoinIdentity(Permissions::default()),
        None,
    )
    .unwrap();
    assert_ok!(Identity::join_identity(alice.origin(), join_auth));

    // Do it again but now making the secondary key the new primary key.
    assert_ok!(accept(alice.ring, add(alice.ring)));
    assert_eq!(alice_pk(), alice.acc());
    assert_ok!(Identity::ensure_key_did_unlinked(&charlie.to_account_id()));
}

#[test]
fn rotating_primary_key_to_secondary() {
    ExtBuilder::default()
        .monied(true)
        .did_registrars(vec![Sr25519Keyring::Eve.to_account_id()])
        .build()
        .execute_with(rotating_primary_key_to_secondary_we);
}

fn rotating_primary_key_to_secondary_we() {
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new(Sr25519Keyring::Bob);
    let charlie = Sr25519Keyring::Charlie;
    let charlie_origin = Origin::signed(charlie.to_account_id());

    // Primary key matches Alice's key
    let alice_pk = || get_primary_key(alice.did);
    assert_eq!(alice_pk(), alice.acc());

    let rotate =
        |from: Origin, auth_id: u64| Identity::rotate_primary_key_to_secondary(from, auth_id);

    assert!(rotate(charlie_origin.clone(), 0).is_err());

    let bob_rotate_auth = Identity::add_auth(
        alice.did,
        bob.signatory_acc(),
        AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
        None,
    )
    .unwrap();
    assert_eq!(
        rotate(bob.origin(), bob_rotate_auth),
        Err(Error::AlreadyLinked.into())
    );

    let charlie_rotate_auth = Identity::add_auth(
        alice.did,
        Signatory::Account(charlie.to_account_id()),
        AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
        None,
    )
    .unwrap();
    assert_ok!(rotate(charlie_origin.clone(), charlie_rotate_auth));
    assert_eq!(alice_pk(), charlie.to_account_id());
    assert!(Identity::is_secondary_key(alice.did, &alice.acc()));

    let alice_rotate_auth = Identity::add_auth(
        alice.did,
        alice.signatory_acc(),
        AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
        None,
    )
    .unwrap();
    assert_ok!(rotate(alice.origin(), alice_rotate_auth));
    assert_eq!(alice_pk(), alice.acc());
    assert!(Identity::is_secondary_key(
        alice.did,
        &charlie.to_account_id()
    ));
}

#[test]
fn cdd_register_did_test() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .did_registrars(vec![
            Sr25519Keyring::Eve.to_account_id(),
            Sr25519Keyring::Ferdie.to_account_id(),
        ])
        .build()
        .execute_with(|| cdd_register_did_test_we());
}

#[allow(deprecated)]
fn cdd_register_did_test_we() {
    let cdd1 = Origin::signed(Sr25519Keyring::Eve.to_account_id());
    let cdd2 = Origin::signed(Sr25519Keyring::Ferdie.to_account_id());
    let non_id = Origin::signed(Sr25519Keyring::Charlie.to_account_id());

    let alice = Sr25519Keyring::Alice.to_account_id();
    let bob_acc = Sr25519Keyring::Bob.to_account_id();

    // CDD 1 registers correctly the Alice's ID.
    assert_ok!(Identity::cdd_register_did(
        cdd1.clone(),
        alice.clone(),
        vec![]
    ));
    let alice_id = get_identity_id(Sr25519Keyring::Alice).unwrap();
    assert_add_cdd_claim!(cdd1.clone(), alice_id);

    // Check that Alice's ID is active DID.
    assert_eq!(Identity::is_did_active(alice_id), true);

    // Error case: Try account without ID.
    assert!(Identity::cdd_register_did(non_id, bob_acc.clone(), vec![]).is_err(),);
    // Error case: Try account with ID but it is not part of CDD providers.
    assert!(
        Identity::cdd_register_did(Origin::signed(alice.clone()), bob_acc.clone(), vec![]).is_err()
    );

    // CDD 2 registers properly Bob's ID.
    assert_ok!(Identity::cdd_register_did(cdd2.clone(), bob_acc, vec![]));
    let bob_id = get_identity_id(Sr25519Keyring::Bob).unwrap();
    assert_add_cdd_claim!(cdd2, bob_id);

    assert_eq!(Identity::is_did_active(bob_id), true);

    // Register with secondary_keys
    // ==============================================
    // Register Charlie with secondary keys.
    let charlie = Sr25519Keyring::Charlie.to_account_id();
    let dave = Sr25519Keyring::Dave.to_account_id();
    let dave_si = SecondaryKey::from_account_id(dave.clone());
    let secondary_keys = vec![dave_si.clone().into()];
    assert_ok!(Identity::cdd_register_did(
        cdd1.clone(),
        charlie.clone(),
        secondary_keys
    ));
    let charlie_id = get_identity_id(Sr25519Keyring::Charlie).unwrap();
    assert_add_cdd_claim!(cdd1.clone(), charlie_id);

    Balances::make_free_balance_be(&charlie, 10_000_000_000);
    assert_eq!(Identity::is_did_active(charlie_id), true);
    assert_eq!(get_secondary_keys(charlie_id).is_empty(), true);

    let dave_auth_id = get_last_auth_id(&Signatory::Account(dave_si.key.clone()));

    assert_ok!(Identity::join_identity_as_key(
        Origin::signed(dave),
        dave_auth_id
    ));
    assert_eq!(get_secondary_keys(charlie_id), vec![dave_si.clone()]);
}

// Test for the DuplicateKey error in `cdd_register_did`.
#[test]
#[allow(deprecated)]
fn cdd_register_did_duplicate_keys_test() {
    ExtBuilder::default()
        .did_registrars(vec![Sr25519Keyring::Eve.to_account_id()])
        .build()
        .execute_with(|| {
            let cdd = Origin::signed(Sr25519Keyring::Eve.to_account_id());
            let alice = Sr25519Keyring::Alice.to_account_id();
            let bob_acc = Sr25519Keyring::Bob.to_account_id();

            let secondary_key = SecondaryKey::new(bob_acc, Permissions::default());
            let secondary_keys = vec![secondary_key.clone().into(), secondary_key.into()];
            assert_noop!(
                Identity::cdd_register_did(cdd, alice, secondary_keys),
                Error::DuplicateKey
            );
        });
}

/// Test the new `register_did` extrinsic for registering DIDs.
#[test]
fn register_did_test() {
    ExtBuilder::default()
        .did_registrars(vec![Sr25519Keyring::Eve.to_account_id()])
        .build()
        .execute_with(|| {
            System::set_block_number(1);

            let registrar = Origin::signed(Sr25519Keyring::Eve.to_account_id());
            let alice = Sr25519Keyring::Alice.to_account_id();
            let bob = Sr25519Keyring::Bob.to_account_id();
            let charlie = Sr25519Keyring::Charlie.to_account_id();

            // Success: DID registrar registers Alice's DID
            assert_ok!(Identity::register_did(registrar.clone(), alice.clone()));
            let alice_id = get_identity_id(Sr25519Keyring::Alice).unwrap();

            // Verify DID is active and primary key is set correctly
            assert!(Identity::is_did_active(alice_id));
            assert_eq!(get_primary_key(alice_id), alice.clone());

            // Verify DidCreated event was emitted
            System::assert_has_event(Event::DidCreated(alice_id, alice.clone(), vec![]).into());

            // Error: Cannot register DID for account that already has one
            assert_noop!(
                Identity::register_did(registrar.clone(), alice.clone()),
                Error::AlreadyLinked
            );

            // Error: Account without DID cannot register others
            assert_noop!(
                Identity::register_did(Origin::signed(charlie.clone()), bob.clone()),
                Error::MissingIdentity
            );

            // Error: Account with DID but not in DidRegistrars group cannot register
            assert_noop!(
                Identity::register_did(Origin::signed(alice.clone()), bob.clone()),
                Error::UnAuthorizedDidRegistrar
            );
        });
}

#[test]
fn add_identity_signers() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new_with(alice.did, Sr25519Keyring::Bob);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let dave = User::new_with(alice.did, Sr25519Keyring::Dave);

        let add_auth = |from: User, to| {
            let data = AuthorizationData::JoinIdentity(Permissions::default());
            Identity::add_auth(from.did, to, data, None).unwrap()
        };

        let auth_id_for_acc_to_id = add_auth(alice, bob.signatory_acc());

        assert_ok!(Identity::join_identity(bob.origin(), auth_id_for_acc_to_id));

        let auth_id_for_acc2_to_id = add_auth(charlie, bob.signatory_acc());

        // Getting expired and non-expired both
        let authorizations = Identity::get_filtered_authorizations(
            bob.signatory_acc(),
            true,
            Some(AuthorizationType::JoinIdentity),
        );
        assert_eq!(authorizations.len(), 1);

        assert_eq!(
            Identity::join_identity(bob.origin(), auth_id_for_acc2_to_id),
            Err(Error::AlreadyLinked.into()),
        );

        let auth_id_for_acc1_to_acc = add_auth(alice, dave.signatory_acc());

        assert_ok!(Identity::join_identity(
            dave.origin(),
            auth_id_for_acc1_to_acc
        ));

        let auth_id_for_acc2_to_acc = add_auth(charlie, dave.signatory_acc());

        assert_eq!(
            Identity::join_identity(dave.origin(), auth_id_for_acc2_to_acc),
            Err(Error::AlreadyLinked.into()),
        );

        let alice_secondary_keys = get_secondary_keys(alice.did);
        let charlie_secondary_keys = get_secondary_keys(charlie.did);
        let mut dave_sk = SecondaryKey::from_account_id(Sr25519Keyring::Dave.to_account_id());
        // Correct the permissions to ones set by `add_secondary_key`.
        dave_sk.permissions = Permissions::default();
        assert!(alice_secondary_keys
            .iter()
            .find(|si| si.key == bob.acc())
            .is_some());
        assert!(charlie_secondary_keys
            .iter()
            .find(|si| si.key == bob.acc())
            .is_none());
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
fn did_registrar_with_systematic_cdd_claims() {
    let did_registrars = [
        Sr25519Keyring::Alice.to_account_id(),
        Sr25519Keyring::Bob.to_account_id(),
    ]
    .to_vec();

    ExtBuilder::default()
        .monied(true)
        .did_registrars(did_registrars)
        .build()
        .execute_with(did_registrar_with_systematic_cdd_claims_we);
}

fn did_registrar_with_systematic_cdd_claims_we() {
    // 0. Get Bob & Alice IDs.
    let root = Origin::from(frame_system::RawOrigin::Root);
    let bob_id =
        get_identity_id(Sr25519Keyring::Bob).expect("Bob should be one of the DID registrars");
    let alice_id =
        get_identity_id(Sr25519Keyring::Alice).expect("Alice should be one of the DID registrars");

    // 1. Each DID registrar has a *systematic* CDD claim.
    let did_registrars = DidRegistrars::get_members();
    assert_eq!(
        did_registrars
            .iter()
            .all(|did| fetch_systematic_claim(*did).is_some()),
        true
    );

    // 2. Remove one member from DID registrars and double-check that systematic CDD claim was
    //    removed too.
    assert_ok!(DidRegistrars::remove_member(root.clone(), bob_id));
    assert_eq!(fetch_systematic_claim(bob_id).is_none(), true);
    assert_eq!(fetch_systematic_claim(alice_id).is_some(), true);

    // 3. Add DID with CDD claim to DID registrars, and check that systematic CDD claim was added.
    // Then remove that DID from DID registrars, it should keep its previous CDD claim.
    let alice = Origin::signed(Sr25519Keyring::Alice.to_account_id());
    let charlie_acc = Sr25519Keyring::Charlie.to_account_id();

    // 3.1. Add CDD claim to Charlie, by Alice (a DID registrar).
    assert_ok!(Identity::register_did(alice.clone(), charlie_acc.clone()));
    let charlie_id =
        get_identity_id(Sr25519Keyring::Charlie).expect("Charlie should have an Identity Id");
    assert_add_cdd_claim!(alice, charlie_id);

    let charlie_cdd_claim =
        Identity::fetch_claim(charlie_id, ClaimType::CustomerDueDiligence, alice_id, None)
            .expect("Charlie should have a CDD claim by Alice");

    // 3.2. Add Charlie as trusted DID registrar, and check its new systematic CDD claim.
    assert_ok!(DidRegistrars::add_member(root.clone(), charlie_id));
    assert_eq!(fetch_systematic_claim(charlie_id).is_some(), true);

    // 3.3. Remove Charlie from trusted DID registrars, and verify that systematic CDD claim was
    //   removed and previous CDD claim works.
    assert_ok!(DidRegistrars::remove_member(root, charlie_id));
    assert_eq!(fetch_systematic_claim(charlie_id).is_none(), true);
    assert_eq!(
        Identity::fetch_claim(charlie_id, ClaimType::CustomerDueDiligence, alice_id, None),
        Some(charlie_cdd_claim)
    );
}

#[test]
fn gc_with_systematic_cdd_claims() {
    let did_registrars = [
        Sr25519Keyring::Alice.to_account_id(),
        Sr25519Keyring::Bob.to_account_id(),
    ]
    .to_vec();
    let governance_committee = [
        Sr25519Keyring::Charlie.to_account_id(),
        Sr25519Keyring::Dave.to_account_id(),
    ]
    .to_vec();

    ExtBuilder::default()
        .monied(true)
        .did_registrars(did_registrars)
        .governance_committee(governance_committee)
        .build()
        .execute_with(gc_with_systematic_cdd_claims_we);
}

fn gc_with_systematic_cdd_claims_we() {
    // 0.
    let root = Origin::from(frame_system::RawOrigin::Root);
    let charlie_id = get_identity_id(Sr25519Keyring::Charlie)
        .expect("Charlie should be a Governance Committee member");
    let dave_id = get_identity_id(Sr25519Keyring::Dave)
        .expect("Dave should be a Governance Committee member");
    let alice_id = get_identity_id(Sr25519Keyring::Alice).expect("Alice should be a DID registrar");

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

    // 3. Add DID with CDD claim to DID registrars, and check that systematic CDD claim was added.
    // Then remove that DID from DID registrars, it should keep its previous CDD claim.
    let alice = Origin::signed(Sr25519Keyring::Alice.to_account_id());
    let ferdie_acc = Sr25519Keyring::Ferdie.to_account_id();

    // 3.1. Add CDD claim to Ferdie, by Alice.
    assert_ok!(Identity::register_did(alice.clone(), ferdie_acc.clone()));
    let ferdie_id =
        get_identity_id(Sr25519Keyring::Ferdie).expect("Ferdie should have an Identity Id");
    assert_add_cdd_claim!(alice, ferdie_id);

    let ferdie_cdd_claim =
        Identity::fetch_claim(ferdie_id, ClaimType::CustomerDueDiligence, alice_id, None)
            .expect("Ferdie should have a CDD claim by Alice");

    // 3.2. Add Ferdie to GC, and check its new systematic CDD claim.
    assert_ok!(GovernanceCommittee::add_member(root.clone(), ferdie_id));
    assert_eq!(fetch_systematic_claim(ferdie_id).is_some(), true);

    // 3.3. Remove Ferdie from GC, and verify that systematic CDD claim was
    //   removed and previous CDD claim works.
    assert_ok!(GovernanceCommittee::remove_member(root, ferdie_id));
    assert_eq!(fetch_systematic_claim(ferdie_id).is_none(), true);
    assert_eq!(
        Identity::fetch_claim(ferdie_id, ClaimType::CustomerDueDiligence, alice_id, None),
        Some(ferdie_cdd_claim)
    );
}

#[test]
fn gc_and_registrar_with_systematic_cdd_claims() {
    let gc_and_registrars = [
        Sr25519Keyring::Alice.to_account_id(),
        Sr25519Keyring::Bob.to_account_id(),
    ]
    .to_vec();

    ExtBuilder::default()
        .monied(true)
        .did_registrars(gc_and_registrars.clone())
        .governance_committee(gc_and_registrars.clone())
        .build()
        .execute_with(gc_and_registrar_with_systematic_cdd_claims_we);
}

fn gc_and_registrar_with_systematic_cdd_claims_we() {
    // 0. Accounts
    let root = Origin::from(frame_system::RawOrigin::Root);
    let alice_id = get_identity_id(Sr25519Keyring::Alice)
        .expect("Alice should be a Governance Committee member");

    // 1. Alice should have 2 systematic CDD claims: One as GC member & another one as DID
    //    registrar.
    assert_eq!(fetch_systematic_gc(alice_id).is_some(), true);
    assert_eq!(fetch_systematic_cdd(alice_id).is_some(), true);

    // 2. Remove Alice from DID registrars.
    assert_ok!(DidRegistrars::remove_member(root.clone(), alice_id));
    assert_eq!(fetch_systematic_gc(alice_id).is_some(), true);
    assert_eq!(fetch_systematic_cdd(alice_id).is_none(), true);

    // 3. Remove Alice from GC.
    assert_ok!(GovernanceCommittee::remove_member(root, alice_id));
    assert_eq!(fetch_systematic_gc(alice_id).is_none(), true);
    assert_eq!(fetch_systematic_cdd(alice_id).is_none(), true);
}

#[test]
#[allow(deprecated)]
fn add_permission_with_secondary_key() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .did_registrars(vec![
            Sr25519Keyring::Eve.to_account_id(),
            Sr25519Keyring::Ferdie.to_account_id(),
        ])
        .build()
        .execute_with(|| {
            let registrar_acc = Sr25519Keyring::Eve.to_account_id();
            let alice_acc = Sr25519Keyring::Alice.to_account_id();
            let bob_acc = Sr25519Keyring::Bob.to_account_id();
            let charlie_acc = Sr25519Keyring::Charlie.to_account_id();

            // Add secondary keys.
            let sk = |acc: &AccountId| SecondaryKey {
                key: acc.clone(),
                permissions: Permissions::empty(),
            };
            let sks = vec![sk(&bob_acc), sk(&charlie_acc)];

            assert_ok!(Identity::cdd_register_did(
                Origin::signed(registrar_acc.clone()),
                alice_acc.clone(),
                sks.clone(),
            ));
            let alice_did = Identity::get_identity(&alice_acc).unwrap();
            assert_add_cdd_claim!(Origin::signed(registrar_acc), alice_did);

            let bob_auth_id = get_last_auth_id(&Signatory::Account(bob_acc.clone()));
            let charlie_auth_id = get_last_auth_id(&Signatory::Account(charlie_acc.clone()));

            println!("Print the protocol base fee: {:?}", PROTOCOL_OP_BASE_FEE);

            // accept the auth_id
            assert_ok!(Identity::join_identity_as_key(
                Origin::signed(bob_acc),
                bob_auth_id
            ));

            // accept the auth_id
            assert_ok!(Identity::join_identity_as_key(
                Origin::signed(charlie_acc),
                charlie_auth_id
            ));

            // check for permissions.
            assert_eq!(get_secondary_keys(alice_did), sks);
        });
}

#[test]
fn ensure_custom_scopes_limited() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(Sr25519Keyring::Alice);
        let data = |n| Scope::Custom([b'1'].repeat(n));
        let add = |n| Identity::add_claim(user.origin(), user.did, Claim::Affiliate(data(n)), None);
        // Check <= 32.
        assert_ok!(add(31));
        assert_ok!(add(32));
        // Check 33.
        assert_noop!(add(33), Error::CustomScopeTooLong);
    });
}

#[test]
fn custom_claim_type_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(Sr25519Keyring::Alice);
        let case = |add| Identity::register_custom_claim_type(user.origin(), max_len_bytes(add));
        assert_too_long!(case(1));
        assert_ok!(case(0));
    });
}

#[test]
fn custom_claim_type_works() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(Sr25519Keyring::Alice);
        let register = |ty: &str| Identity::register_custom_claim_type(user.origin(), ty.into());
        let seq_is = |num| {
            assert_eq!(CustomClaimIdSequence::<TestStorage>::get().0, num);
        };
        let slot_has = |id, data: &str| {
            seq_is(id);
            let id = CustomClaimTypeId(id);
            let data = data.as_bytes().to_vec();
            assert_eq!(CustomClaims::<TestStorage>::get(id).as_ref(), Some(&data));
            assert_eq!(CustomClaimsInverse::<TestStorage>::get(data), Some(id));
        };

        // Nothing so far. Generator (G) at 0.
        seq_is(0);

        // Register first type. G -> 1.
        assert_ok!(register("foo"));
        slot_has(1, "foo");

        // Register same type. G unmoved.
        assert_noop!(register("foo"), Error::CustomClaimTypeAlreadyExists);
        slot_has(1, "foo");

        // Register different type. G -> 2.
        assert_ok!(register("bar"));
        slot_has(2, "bar");

        // Register same type. G unmoved.
        assert_noop!(register("bar"), Error::CustomClaimTypeAlreadyExists);
        slot_has(2, "bar");

        // Register different type. G -> 3.
        assert_ok!(register("foobar"));
        slot_has(3, "foobar");

        // Set G to max. Next registration fails.
        CustomClaimIdSequence::<TestStorage>::put(CustomClaimTypeId(u32::MAX));
        assert_noop!(register("qux"), BaseError::CounterOverflow);
    });
}

#[test]
fn invalid_custom_claim_type() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        assert_noop!(
            Identity::base_add_claim(
                alice.did,
                Claim::Custom(CustomClaimTypeId(1_000_000), None),
                alice.did,
                None
            ),
            Error::CustomClaimTypeDoesNotExist
        );
    });
}

#[test]
#[allow(deprecated)]
fn cdd_register_did_events() {
    ExtBuilder::default()
        .did_registrars(vec![Sr25519Keyring::Eve.to_account_id()])
        .build()
        .execute_with(|| {
            System::set_block_number(1);
            // Register an Identity for alice with two secondary keys
            let registrar = Origin::signed(Sr25519Keyring::Eve.to_account_id());
            let alice_account_id = Sr25519Keyring::Alice.to_account_id();
            let alice_secondary_keys = vec![
                SecondaryKey::from_account_id(Sr25519Keyring::Dave.to_account_id()),
                SecondaryKey::from_account_id(Sr25519Keyring::Charlie.to_account_id()),
            ];
            assert_ok!(Identity::cdd_register_did(
                registrar,
                alice_account_id.clone(),
                alice_secondary_keys.clone()
            ));
            let alice_did = get_identity_id(Sr25519Keyring::Alice).unwrap();
            // Make sure one Authorization event was sent for each secondary key
            let mut system_events = System::events();
            assert_eq!(
                system_events.pop().unwrap().event,
                super::storage::EventTest::Identity(Event::AuthorizationAdded(
                    alice_did,
                    None,
                    Some(Sr25519Keyring::Charlie.to_account_id()),
                    CurrentAuthId::<TestStorage>::get(),
                    AuthorizationData::JoinIdentity(alice_secondary_keys[1].permissions.clone()),
                    None,
                ))
            );
            assert_eq!(
                system_events.pop().unwrap().event,
                super::storage::EventTest::Identity(Event::AuthorizationAdded(
                    alice_did,
                    None,
                    Some(Sr25519Keyring::Dave.to_account_id()),
                    CurrentAuthId::<TestStorage>::get() - 1,
                    AuthorizationData::JoinIdentity(alice_secondary_keys[0].permissions.clone()),
                    None,
                ))
            );
            // Make sure a Did Created event was sent
            assert_eq!(
                system_events.pop().unwrap().event,
                super::storage::EventTest::Identity(Event::DidCreated(
                    alice_did,
                    alice_account_id,
                    alice_secondary_keys.clone()
                ))
            );
        });
}
