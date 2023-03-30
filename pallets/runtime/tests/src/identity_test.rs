use super::{
    asset_test::{an_asset, basic_asset, max_len, max_len_bytes, set_timestamp, token},
    committee_test::gc_vmo,
    exec_noop, exec_ok,
    ext_builder::PROTOCOL_OP_BASE_FEE,
    storage::{
        account_from, add_secondary_key, add_secondary_key_with_perms,
        create_cdd_id_and_investor_uid, get_identity_id, get_last_auth_id, get_primary_key,
        get_secondary_keys, provide_scope_claim, register_keyring_account,
        register_keyring_account_with_balance, GovernanceCommittee, TestStorage, User,
    },
    ExtBuilder,
};
use codec::Encode;
use confidential_identity_v1::mocked::make_investor_uid;
use frame_support::{
    assert_noop, assert_ok, dispatch::DispatchResult, traits::Currency, StorageDoubleMap,
    StorageMap, StorageValue,
};
use pallet_asset::SecurityToken;
use pallet_balances as balances;
use pallet_identity::{CustomClaimIdSequence, CustomClaims, CustomClaimsInverse};
use polymesh_common_utilities::{
    asset::AssetSubTrait,
    constants::currency::POLY,
    protocol_fee::ProtocolOp,
    traits::{
        group::GroupTrait,
        identity::{
            Config as IdentityConfig, CreateChildIdentityWithAuth, RawEvent, SecondaryKeyWithAuth,
            TargetIdAuthorization,
        },
        transaction_payment::CddAndFeeDetails,
    },
    SystematicIssuers, GC_DID,
};
use polymesh_primitives::{
    investor_zkproof_data::v2, AccountId, AssetPermissions, AuthorizationData, AuthorizationType,
    CddId, Claim, ClaimType, CustomClaimTypeId, DispatchableName, ExtrinsicPermissions,
    IdentityClaim, IdentityId, InvestorUid, KeyRecord, PalletName, PalletPermissions, Permissions,
    PortfolioId, PortfolioNumber, Scope, SecondaryKey, Signatory, SubsetRestriction, Ticker,
    TransactionError,
};
use polymesh_runtime_develop::runtime::{CddHandler, RuntimeCall};
use sp_core::H512;
use sp_runtime::transaction_validity::InvalidTransaction;
use std::convert::From;
use test_client::AccountKeyring;

type AuthorizationsGiven = pallet_identity::AuthorizationsGiven<TestStorage>;
type Asset = pallet_asset::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type BaseError = pallet_base::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type ParentDid = pallet_identity::ParentDid;
type MultiSig = pallet_multisig::Module<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type CddServiceProviders = <TestStorage as IdentityConfig>::CddServiceProviders;
type Error = pallet_identity::Error<TestStorage>;
type PError = pallet_permissions::Error<TestStorage>;

const MAX_ASSETS: u64 = 2000;
const MAX_PORTFOLIOS: u64 = 2000;

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

fn target_id_auth(user: User) -> (TargetIdAuthorization<u64>, u64) {
    let expires_at = 100u64;
    (
        TargetIdAuthorization {
            target_id: user.did,
            nonce: Identity::offchain_authorization_nonce(user.did),
            expires_at,
        },
        expires_at,
    )
}

fn create_new_token(name: &[u8], owner: User) -> (Ticker, SecurityToken) {
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
#[test]
fn only_primary_or_secondary_keys_can_authenticate_as_an_identity() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);

        let bob = User::new(AccountKeyring::Bob);

        // Add charlie's key as a secondary key of bob.
        let charlie = User::new_with(bob.did, AccountKeyring::Charlie);
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
        let target = User::new(AccountKeyring::Charlie);
        let fetch =
            || Identity::fetch_claim(target.did, ClaimType::CustomerDueDiligence, GC_DID, None);

        assert_ok!(Identity::gc_add_cdd_claim(gc_vmo(), target.did));

        let cdd_id = CddId::new_v1(target.did, InvestorUid::from(target.did.as_ref()));
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

        assert_ok!(Identity::gc_revoke_cdd_claim(gc_vmo(), target.did));
        assert_eq!(fetch(), None);
    });
}

#[test]
fn revoking_claims() {
    ExtBuilder::default().build().execute_with(|| {
        let _owner = User::new(AccountKeyring::Alice);
        let _issuer = User::new(AccountKeyring::Bob);
        let claim_issuer = User::new(AccountKeyring::Charlie);
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
        let _owner = User::new(AccountKeyring::Alice);
        let _issuer = User::new(AccountKeyring::Bob);
        let claim_issuer = User::new(AccountKeyring::Charlie);
        let scope = Scope::from(IdentityId::from(0));

        let add = |claim, scope| {
            Identity::add_claim(claim_issuer.origin(), claim_issuer.did, claim, scope)
        };
        let fetch =
            |claim, scope| Identity::fetch_claim(claim_issuer.did, claim, claim_issuer.did, scope);

        let revoke = |claim| Identity::revoke_claim(claim_issuer.origin(), claim_issuer.did, claim);

        assert_ok!(add(Claim::Accredited(scope.clone()), Some(100u64)));
        assert_ok!(add(Claim::NoData, None));
        assert!(fetch(ClaimType::Accredited, Some(scope.clone())).is_some());

        assert!(fetch(ClaimType::NoType, None).is_some());
        assert!(fetch(ClaimType::Accredited, Some(scope.clone())).is_some());

        assert_ok!(revoke(Claim::Accredited(scope.clone())));

        assert_ok!(revoke(Claim::NoData));

        assert!(fetch(ClaimType::Accredited, Some(scope.clone())).is_none());

        assert!(fetch(ClaimType::NoType, None).is_none());

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
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);
    let charlie = User::new_with(alice.did, AccountKeyring::Charlie);

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
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);

    // Add bob with default permissions.
    add_secondary_key(alice.did, bob.acc());

    // Create some tokens.
    let max_tokens = 20;
    let tokens: Vec<Ticker> = (0..max_tokens)
        .map(|i| {
            let name = format!("TOKEN{}", i);
            let (ticker, _) = create_new_token(name.as_bytes(), alice);
            ticker
        })
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
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new_with(alice.did, AccountKeyring::Bob);
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
    let alice = User::new(AccountKeyring::Alice);
    let [bob, charlie, dave] = [
        AccountKeyring::Bob,
        AccountKeyring::Charlie,
        AccountKeyring::Dave,
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
    // commenting this because `default_identity` feature is not allowing to access None identity.
    // assert_noop!(
    //     Identity::unfreeze_secondary_keys(bob.clone()),
    //     DispatchError::Other("Current identity is none and key is not linked to any identity")
    // );
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
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);
    let charlie = User::new_with(alice.did, AccountKeyring::Charlie);

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
    let alice = AccountKeyring::Alice.to_account_id();
    let bob = AccountKeyring::Bob.to_account_id();
    let charlie = AccountKeyring::Charlie.to_account_id();
    TestStorage::set_payer_context(Some(alice.clone()));
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    TestStorage::set_payer_context(Some(charlie.clone()));
    let _charlie_id = register_keyring_account_with_balance(AccountKeyring::Charlie, 100).unwrap();
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

    // 4. Bob should NOT transfer any amount. SE is simulated.
    // Balances::transfer_with_memo(Origin::signed(bob), charlie, 1_000, None),
    let payer = CddHandler::get_valid_payer(
        &RuntimeCall::Balances(balances::Call::transfer_with_memo {
            dest: AccountKeyring::Charlie.to_account_id().into(),
            value: 1_000,
            memo: None,
        }),
        &AccountKeyring::Bob.to_account_id(),
    );
    assert_noop!(
        payer,
        InvalidTransaction::Custom(TransactionError::MissingIdentity as u8)
    );

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
    let alice = User::new(AccountKeyring::Alice);

    let perm1 = Permissions::empty();
    let perm2 = Permissions::from_pallet_permissions(vec![PalletPermissions::entire_pallet(
        b"identity".into(),
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
        let bob = User::new_with(alice.did, AccountKeyring::Bob);
        let data = AuthorizationData::JoinIdentity(perms);
        let auth_id = Identity::add_auth(alice.did, bob.signatory_acc(), data, None);
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
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);

    // Add bob with default permissions
    add_secondary_key(alice.did, bob.acc());

    let permissions = Permissions::from_pallet_permissions(vec![PalletPermissions::entire_pallet(
        b"identity".into(),
    )]);
    // Try adding bob again with custom permissions
    let auth_id = Identity::add_auth(
        alice.did,
        bob.signatory_acc(),
        AuthorizationData::JoinIdentity(permissions.clone()),
        None,
    );
    assert_eq!(
        Identity::join_identity(bob.origin(), auth_id),
        Err(Error::AlreadyLinked.into()),
    );

    // Try addind the same secondary_key using `add_secondary_keys_with_authorization`
    let (authorization, expires_at) = target_id_auth(alice);
    let auth_encoded = authorization.encode();
    let auth_signature = H512::from(alice.ring.sign(&auth_encoded));

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
    TestStorage::set_current_identity(&alice.did);
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
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);
    let dave = User::new_with(alice.did, AccountKeyring::Dave);

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
    TestStorage::set_current_identity(&alice.did);
    let remove_sk = |u: User| Identity::remove_secondary_keys(alice.origin(), vec![u.acc()]);
    assert_ok!(remove_sk(bob));

    // Try changing the permissions for bob's key.
    // This should fail.
    let result = Identity::set_secondary_key_permissions(
        alice.origin(),
        bob.acc(),
        Permissions::from_pallet_permissions(vec![PalletPermissions::entire_pallet(
            b"identity".into(),
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
    let bob_key = AccountKeyring::Bob.to_account_id();
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);
    let charlie = User::new(AccountKeyring::Charlie);
    let dave_key = AccountKeyring::Dave.to_account_id();

    let ms_address = MultiSig::get_next_multisig_address(alice.acc());

    assert_ok!(MultiSig::create_multisig(
        alice.origin(),
        vec![
            Signatory::from(alice.did),
            Signatory::Account(dave_key.clone())
        ],
        1,
    ));
    let auth_id = get_last_auth_id(&Signatory::Account(dave_key.clone()));
    assert_ok!(MultiSig::unsafe_accept_multisig_signer(
        Signatory::Account(dave_key.clone()),
        auth_id
    ));

    add_secondary_key(alice.did, bob.acc());

    add_secondary_key(alice.did, ms_address.clone());

    // Fund the multisig
    assert_ok!(Balances::transfer(
        alice.origin(),
        ms_address.clone().into(),
        2 * POLY
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));
    assert_eq!(Identity::get_identity(&bob_key), Some(alice.did));

    // Try removing bob using charlie
    TestStorage::set_current_identity(&charlie.did);
    assert_noop!(
        Identity::remove_secondary_keys(charlie.origin(), vec![bob.acc()]),
        Error::NotASigner
    );

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));
    assert_eq!(Identity::get_identity(&bob_key), Some(alice.did));

    // Try remove bob using alice
    TestStorage::set_current_identity(&alice.did);
    assert_ok!(Identity::remove_secondary_keys(
        alice.origin(),
        vec![bob.acc()]
    ));

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Try removing multisig while it has funds
    assert_noop!(
        Identity::remove_secondary_keys(alice.origin(), vec![ms_address.clone()]),
        Error::MultiSigHasBalance
    );

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));
    assert_eq!(Identity::get_identity(&bob_key), None);

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(ms_address.clone(), Signatory::Account(dave_key.clone())),
        true
    );

    // Transfer funds back to Alice
    assert_ok!(Balances::transfer(
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
    assert_eq!(
        MultiSig::ms_signers(ms_address.clone(), Signatory::Account(dave_key)),
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
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);

    let bob_sk = SecondaryKey::new(bob.acc(), Permissions::empty());
    let dave_key = AccountKeyring::Dave.to_account_id();

    let ms_address = MultiSig::get_next_multisig_address(alice.acc());

    assert_ok!(MultiSig::create_multisig(
        alice.origin(),
        vec![
            Signatory::from(alice.did),
            Signatory::Account(dave_key.clone())
        ],
        1,
    ));
    let auth_id = get_last_auth_id(&Signatory::Account(dave_key.clone()));
    assert_ok!(MultiSig::unsafe_accept_multisig_signer(
        Signatory::Account(dave_key.clone()),
        auth_id
    ));

    add_secondary_key_with_perms(alice.did, bob.acc(), Permissions::empty());

    // Check DidRecord.
    assert_eq!(get_secondary_keys(alice.did), vec![bob_sk]);
    assert_eq!(Identity::get_identity(&bob.acc()), Some(alice.did));

    // Bob leaves
    assert_ok!(Identity::leave_identity_as_key(bob.origin()));

    // Check DidRecord.
    assert_eq!(get_secondary_keys(alice.did).len(), 0);
    assert_eq!(Identity::get_identity(&bob.acc()), None);
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), None);

    add_secondary_key_with_perms(alice.did, ms_address.clone(), Permissions::empty());
    // send funds to multisig
    assert_ok!(Balances::transfer(
        alice.origin(),
        ms_address.clone().into(),
        2 * POLY
    ));
    // multisig tries leaving identity while it has funds
    assert_noop!(
        Identity::leave_identity_as_key(Origin::signed(ms_address.clone())),
        Error::MultiSigHasBalance
    );

    // Check DidRecord.
    assert_eq!(Identity::get_identity(&dave_key), None);
    assert_eq!(Identity::get_identity(&ms_address), Some(alice.did));

    // Check multisig's signer
    assert_eq!(
        MultiSig::ms_signers(ms_address.clone(), Signatory::Account(dave_key.clone())),
        true
    );

    // send funds back to alice from multisig
    assert_ok!(Balances::transfer(
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
    assert_eq!(
        MultiSig::ms_signers(ms_address.clone(), Signatory::Account(dave_key)),
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
    // Register identities.
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);

    // Check external signed key uniqueness.
    let charlie = User::new_with(alice.did, AccountKeyring::Charlie);
    add_secondary_key(alice.did, charlie.acc());
    let auth_id = Identity::add_auth(
        alice.did,
        bob.signatory_acc(),
        AuthorizationData::JoinIdentity(Permissions::empty()),
        None,
    );
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
    let a = User::new(AccountKeyring::Alice);

    let (authorization, expires_at) = target_id_auth(a);
    let auth_encoded = authorization.encode();

    let keys = [
        AccountKeyring::Bob,
        AccountKeyring::Charlie,
        AccountKeyring::Dave,
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

    let f = User::new(AccountKeyring::Ferdie);
    let (ferdie_auth, _) = target_id_auth(a);
    let ferdie_secondary_key_with_auth = SecondaryKeyWithAuth {
        secondary_key: SecondaryKey::from_account_id(f.acc()).into(),
        auth_signature: H512::from(AccountKeyring::Eve.sign(ferdie_auth.encode().as_slice())),
    };

    assert_noop!(
        add(f, vec![ferdie_secondary_key_with_auth]),
        Error::AuthorizationExpired
    );
}

pub(crate) fn test_with_bad_ext_perms(test: impl Fn(ExtrinsicPermissions)) {
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

pub(crate) fn test_with_bad_perms(did: IdentityId, test: impl Fn(Permissions)) {
    test(Permissions {
        asset: SubsetRestriction::elems((0..=MAX_ASSETS).map(Ticker::generate_into)),
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

#[test]
fn add_secondary_keys_with_authorization_too_many_sks() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(AccountKeyring::Alice);
        let bob = User::new_with(user.did, AccountKeyring::Bob);

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
        test_with_bad_perms(user.did, |perms| {
            let auth_encoded = auth();
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
        let key_record = KeyRecord::SecondaryKey(user.did, Permissions::empty());
        for idx in 0..max_len() {
            let key = account_from(idx as u64 + 100);
            Identity::add_key_record(&key, key_record.clone());
        }

        // No Secondary key limit.
        let auth_encoded = auth();
        assert_ok!(Identity::add_secondary_keys_with_authorization(
            user.origin(),
            secondary_keys_with_auth(&[bob], &auth_encoded),
            expires_at
        ));
    });
}

#[test]
fn secondary_key_with_bad_permissions() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .cdd_providers(vec![
            AccountKeyring::Eve.to_account_id(),
            AccountKeyring::Ferdie.to_account_id(),
        ])
        .build()
        .execute_with(|| {
            let cdd1 = AccountKeyring::Eve.to_account_id();
            let alice = User::new(AccountKeyring::Alice);
            let bob = User::new_with(alice.did, AccountKeyring::Bob);

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
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let ticker50 = Ticker::from_slice_truncated(&[0x50][..]);
        let mut auth_id = Identity::add_auth(
            alice.did,
            bob.signatory_did(),
            AuthorizationData::TransferTicker(ticker50),
            None,
        );
        assert_eq!(
            AuthorizationsGiven::get(alice.did, auth_id),
            bob.signatory_did(),
        );
        let mut auth =
            Identity::authorizations(&bob.signatory_did(), auth_id).expect("Missing authorization");
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
        );
        assert_eq!(
            AuthorizationsGiven::get(alice.did, auth_id),
            bob.signatory_did()
        );
        auth =
            Identity::authorizations(&bob.signatory_did(), auth_id).expect("Missing authorization");
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
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let ticker50 = Ticker::from_slice_truncated(&[0x50][..]);
        let auth_id = Identity::add_auth(
            alice.did,
            bob.signatory_did(),
            AuthorizationData::TransferTicker(ticker50),
            None,
        );
        assert_eq!(
            AuthorizationsGiven::get(alice.did, auth_id),
            bob.signatory_did()
        );
        let auth =
            Identity::authorizations(&bob.signatory_did(), auth_id).expect("Missing authorization");
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
        assert!(
            !<pallet_identity::Authorizations<TestStorage>>::contains_key(
                bob.signatory_did(),
                auth_id
            )
        );
    });
}

#[test]
fn changing_primary_key() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(changing_primary_key_we);
}

fn changing_primary_key_we() {
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);

    // Primary key matches Alice's key
    let alice_pk = || get_primary_key(alice.did);
    assert_eq!(alice_pk(), alice.acc());

    let add = |ring: AccountKeyring| {
        Identity::add_auth(
            alice.did,
            Signatory::Account(ring.to_account_id()),
            AuthorizationData::RotatePrimaryKey,
            None,
        )
    };
    let accept = |ring: AccountKeyring, auth| {
        Identity::accept_primary_key(Origin::signed(ring.to_account_id()), auth, None)
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
    let charlie = AccountKeyring::Charlie;
    assert_ok!(accept(charlie, add(charlie)));
    assert_eq!(alice_pk(), charlie.to_account_id());
    assert_ok!(Identity::ensure_key_did_unlinked(&alice.acc()));

    // Add Alice's old primary key as a secondary key to the identity..
    let join_auth = Identity::add_auth(
        alice.did,
        Signatory::Account(alice.acc()),
        AuthorizationData::JoinIdentity(Permissions::default()),
        None,
    );
    assert_ok!(Identity::join_identity(alice.origin(), join_auth));

    // Do it again but now making the secondary key the new primary key.
    assert_ok!(accept(alice.ring, add(alice.ring)));
    assert_eq!(alice_pk(), alice.acc());
    assert_ok!(Identity::ensure_key_did_unlinked(&charlie.to_account_id()));
}

#[test]
fn changing_primary_key_with_cdd_auth() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(|| changing_primary_key_with_cdd_auth_we());
}

fn changing_primary_key_with_cdd_auth_we() {
    let alice = User::new(AccountKeyring::Alice);
    let alice_pk = || get_primary_key(alice.did);
    let new = User::new_with(alice.did, AccountKeyring::Bob);

    let cdd_did = get_identity_id(AccountKeyring::Eve).unwrap();

    // Primary key matches Alice's key
    assert_eq!(alice_pk(), alice.acc());

    // Alice triggers change of primary key
    let owner_auth_id = Identity::add_auth(
        alice.did,
        new.signatory_acc(),
        AuthorizationData::RotatePrimaryKey,
        None,
    );

    let cdd_auth_id = Identity::add_auth(
        cdd_did,
        new.signatory_acc(),
        AuthorizationData::AttestPrimaryKeyRotation(alice.did),
        None,
    );

    assert_ok!(Identity::change_cdd_requirement_for_mk_rotation(
        frame_system::RawOrigin::Root.into(),
        true
    ));

    assert!(Identity::accept_primary_key(new.origin(), owner_auth_id.clone(), None).is_err());

    let owner_auth_id2 = Identity::add_auth(
        alice.did,
        new.signatory_acc(),
        AuthorizationData::RotatePrimaryKey,
        None,
    );

    // Accept the authorization with the new key
    assert_ok!(Identity::accept_primary_key(
        new.origin(),
        owner_auth_id2,
        Some(cdd_auth_id)
    ));

    // Alice's primary key is now Bob's
    assert_eq!(alice_pk(), AccountKeyring::Bob.to_account_id());
}
#[test]
fn rotating_primary_key_to_secondary() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(rotating_primary_key_to_secondary_we);
}

fn rotating_primary_key_to_secondary_we() {
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);
    let charlie = AccountKeyring::Charlie;
    let charlie_origin = Origin::signed(charlie.to_account_id());

    // Primary key matches Alice's key
    let alice_pk = || get_primary_key(alice.did);
    assert_eq!(alice_pk(), alice.acc());

    let rotate =
        |from: Origin, auth_id: u64| Identity::rotate_primary_key_to_secondary(from, auth_id, None);

    assert!(rotate(charlie_origin.clone(), 0).is_err());

    let bob_rotate_auth = Identity::add_auth(
        alice.did,
        bob.signatory_acc(),
        AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
        None,
    );
    assert_eq!(
        rotate(bob.origin(), bob_rotate_auth),
        Err(Error::AlreadyLinked.into())
    );

    let charlie_rotate_auth = Identity::add_auth(
        alice.did,
        Signatory::Account(charlie.to_account_id()),
        AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
        None,
    );
    assert_ok!(rotate(charlie_origin.clone(), charlie_rotate_auth));
    assert_eq!(alice_pk(), charlie.to_account_id());
    assert!(Identity::is_secondary_key(alice.did, &alice.acc()));

    let alice_rotate_auth = Identity::add_auth(
        alice.did,
        alice.signatory_acc(),
        AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
        None,
    );
    assert_ok!(rotate(alice.origin(), alice_rotate_auth));
    assert_eq!(alice_pk(), alice.acc());
    assert!(Identity::is_secondary_key(
        alice.did,
        &charlie.to_account_id()
    ));
}

#[test]
fn rotating_primary_key_to_secondary_with_cdd_auth() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(|| rotating_primary_key_to_secondary_with_cdd_auth_we());
}

fn rotating_primary_key_to_secondary_with_cdd_auth_we() {
    let alice = User::new(AccountKeyring::Alice);
    let alice_pk = || get_primary_key(alice.did);
    let charlie = AccountKeyring::Charlie;
    let charlie_origin = Origin::signed(charlie.to_account_id());

    let rotate = |from: Origin, auth_id: u64, cdd: Option<u64>| {
        Identity::rotate_primary_key_to_secondary(from, auth_id, cdd)
    };

    let cdd_did = get_identity_id(AccountKeyring::Eve).unwrap();

    let rotate_auth = Identity::add_auth(
        alice.did,
        Signatory::Account(charlie.to_account_id()),
        AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
        None,
    );
    let join_auth = Identity::add_auth(
        alice.did,
        Signatory::Account(charlie.to_account_id()),
        AuthorizationData::JoinIdentity(Permissions::default()),
        None,
    );
    assert_ok!(Identity::join_identity(charlie_origin.clone(), join_auth));

    // Primary key matches Alice's key
    assert_eq!(alice_pk(), alice.acc());

    assert_ok!(Identity::change_cdd_requirement_for_mk_rotation(
        frame_system::RawOrigin::Root.into(),
        true
    ));

    assert!(rotate(charlie_origin.clone(), rotate_auth, None).is_err());

    let cdd_auth_id = Identity::add_auth(
        cdd_did,
        Signatory::Account(charlie.to_account_id()),
        AuthorizationData::AttestPrimaryKeyRotation(alice.did),
        None,
    );

    assert_ok!(rotate(charlie_origin, rotate_auth, Some(cdd_auth_id)));

    // Alice's primary key is now Bob's
    assert_eq!(alice_pk(), charlie.to_account_id());
}

#[test]
fn cdd_register_did_test() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .cdd_providers(vec![
            AccountKeyring::Eve.to_account_id(),
            AccountKeyring::Ferdie.to_account_id(),
        ])
        .build()
        .execute_with(|| cdd_register_did_test_we());
}

fn cdd_register_did_test_we() {
    let cdd1 = Origin::signed(AccountKeyring::Eve.to_account_id());
    let cdd2 = Origin::signed(AccountKeyring::Ferdie.to_account_id());
    let non_id = Origin::signed(AccountKeyring::Charlie.to_account_id());

    let alice = AccountKeyring::Alice.to_account_id();
    let bob_acc = AccountKeyring::Bob.to_account_id();

    // CDD 1 registers correctly the Alice's ID.
    assert_ok!(Identity::cdd_register_did(
        cdd1.clone(),
        alice.clone(),
        vec![]
    ));
    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    assert_add_cdd_claim!(cdd1.clone(), alice_id);

    // Check that Alice's ID is attested by CDD 1.
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Error case: Try account without ID.
    assert!(Identity::cdd_register_did(non_id, bob_acc.clone(), vec![]).is_err(),);
    // Error case: Try account with ID but it is not part of CDD providers.
    assert!(
        Identity::cdd_register_did(Origin::signed(alice.clone()), bob_acc.clone(), vec![]).is_err()
    );

    // CDD 2 registers properly Bob's ID.
    assert_ok!(Identity::cdd_register_did(cdd2.clone(), bob_acc, vec![]));
    let bob_id = get_identity_id(AccountKeyring::Bob).unwrap();
    assert_add_cdd_claim!(cdd2, bob_id);

    assert_eq!(Identity::has_valid_cdd(bob_id), true);

    // Register with secondary_keys
    // ==============================================
    // Register Charlie with secondary keys.
    let charlie = AccountKeyring::Charlie.to_account_id();
    let dave = AccountKeyring::Dave.to_account_id();
    let dave_si = SecondaryKey::from_account_id(dave.clone());
    let secondary_keys = vec![dave_si.clone().into()];
    assert_ok!(Identity::cdd_register_did(
        cdd1.clone(),
        charlie.clone(),
        secondary_keys
    ));
    let charlie_id = get_identity_id(AccountKeyring::Charlie).unwrap();
    assert_add_cdd_claim!(cdd1.clone(), charlie_id);

    Balances::make_free_balance_be(&charlie, 10_000_000_000);
    assert_eq!(Identity::has_valid_cdd(charlie_id), true);
    assert_eq!(get_secondary_keys(charlie_id).is_empty(), true);

    let dave_auth_id = get_last_auth_id(&Signatory::Account(dave_si.key.clone()));

    assert_ok!(Identity::join_identity_as_key(
        Origin::signed(dave),
        dave_auth_id
    ));
    assert_eq!(get_secondary_keys(charlie_id), vec![dave_si.clone()]);
}

#[test]
fn add_identity_signers() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new_with(alice.did, AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);
        let dave = User::new_with(alice.did, AccountKeyring::Dave);

        let add_auth = |from: User, to| {
            let data = AuthorizationData::JoinIdentity(Permissions::default());
            Identity::add_auth(from.did, to, data, None)
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
        let mut dave_sk = SecondaryKey::from_account_id(AccountKeyring::Dave.to_account_id());
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
fn invalidate_cdd_claims() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .cdd_providers(vec![
            AccountKeyring::Eve.to_account_id(),
            AccountKeyring::Ferdie.to_account_id(),
        ])
        .build()
        .execute_with(invalidate_cdd_claims_we);
}

fn invalidate_cdd_claims_we() {
    let root = Origin::from(frame_system::RawOrigin::Root);
    let cdd = AccountKeyring::Eve.to_account_id();
    let alice_acc = AccountKeyring::Alice.to_account_id();
    let bob_acc = AccountKeyring::Bob.to_account_id();
    assert_ok!(Identity::cdd_register_did(
        Origin::signed(cdd.clone()),
        alice_acc,
        vec![]
    ));
    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    assert_add_cdd_claim!(Origin::signed(cdd.clone()), alice_id);

    // Check that Alice's ID is attested by CDD 1.
    let cdd_1_id = Identity::get_identity(&cdd).unwrap();
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Disable CDD 1.
    assert_ok!(Identity::invalidate_cdd_claims(root, cdd_1_id, 5, Some(10)));
    assert_eq!(Identity::has_valid_cdd(alice_id), true);

    // Move to time 8... CDD_1 is inactive: Its claims are valid.
    set_timestamp(8);
    assert_eq!(Identity::has_valid_cdd(alice_id), true);
    assert_noop!(
        Identity::cdd_register_did(Origin::signed(cdd.clone()), bob_acc.clone(), vec![]),
        Error::UnAuthorizedCddProvider
    );

    // Move to time 11 ... CDD_1 is expired: Its claims are invalid.
    set_timestamp(11);
    assert_eq!(Identity::has_valid_cdd(alice_id), false);
    assert_noop!(
        Identity::cdd_register_did(Origin::signed(cdd), bob_acc, vec![]),
        Error::UnAuthorizedCddProvider
    );
}

#[test]
fn cdd_provider_with_systematic_cdd_claims() {
    let cdd_providers = [
        AccountKeyring::Alice.to_account_id(),
        AccountKeyring::Bob.to_account_id(),
    ]
    .to_vec();

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
    let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
    let charlie_acc = AccountKeyring::Charlie.to_account_id();

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
    let cdd_providers = [
        AccountKeyring::Alice.to_account_id(),
        AccountKeyring::Bob.to_account_id(),
    ]
    .to_vec();
    let governance_committee = [
        AccountKeyring::Charlie.to_account_id(),
        AccountKeyring::Dave.to_account_id(),
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
    let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
    let ferdie_acc = AccountKeyring::Ferdie.to_account_id();

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
    let gc_and_cdd_providers = [
        AccountKeyring::Alice.to_account_id(),
        AccountKeyring::Bob.to_account_id(),
    ]
    .to_vec();

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
            AccountKeyring::Eve.to_account_id(),
            AccountKeyring::Ferdie.to_account_id(),
        ])
        .build()
        .execute_with(|| {
            let cdd_1_acc = AccountKeyring::Eve.to_account_id();
            let alice_acc = AccountKeyring::Alice.to_account_id();
            let bob_acc = AccountKeyring::Bob.to_account_id();
            let charlie_acc = AccountKeyring::Charlie.to_account_id();

            // Add secondary keys.
            let sk = |acc: &AccountId| SecondaryKey {
                key: acc.clone(),
                permissions: Permissions::empty(),
            };
            let sks = vec![sk(&bob_acc), sk(&charlie_acc)];

            assert_ok!(Identity::cdd_register_did(
                Origin::signed(cdd_1_acc.clone()),
                alice_acc.clone(),
                sks.clone(),
            ));
            let alice_did = Identity::get_identity(&alice_acc).unwrap();
            assert_add_cdd_claim!(Origin::signed(cdd_1_acc), alice_did);

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

#[ignore]
#[test]
fn add_investor_uniqueness_claim() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Charlie.to_account_id()])
        .build()
        .execute_with(do_add_investor_uniqueness_claim);
}

fn do_add_investor_uniqueness_claim() {
    let alice = User::new(AccountKeyring::Alice);
    let cdd_provider = AccountKeyring::Charlie.to_account_id();
    let ticker = an_asset(alice, true);
    let initial_balance = Asset::balance_of(ticker, alice.did);
    let add_iu_claim = |investor_uid| {
        provide_scope_claim(
            alice.did,
            ticker,
            investor_uid,
            cdd_provider.clone(),
            Some(1),
        )
    };
    let no_balance_at_scope = |scope_id| {
        assert_eq!(
            false,
            pallet_asset::BalanceOfAtScope::contains_key(scope_id, alice.did)
        );
    };
    let balance_at_scope = |scope_id, balance| {
        assert_eq!(balance, Asset::balance_of_at_scope(scope_id, alice.did));
    };
    let scope_id_of = |scope_id| {
        assert_eq!(scope_id, Asset::scope_id(&ticker, &alice.did));
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
        Origin::signed(cdd_provider.clone()),
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

#[ignore]
#[test]
fn add_investor_uniqueness_claim_v2() {
    let user = AccountKeyring::Alice.to_account_id();
    let user_no_cdd_id = AccountKeyring::Bob.to_account_id();

    ExtBuilder::default()
        .add_regular_users_from_accounts(&[user.clone()])
        .build()
        .execute_with(|| {
            // Create an DID without CDD_id.
            assert_ok!(Identity::_register_did(
                user_no_cdd_id.clone(),
                vec![],
                Some(ProtocolOp::IdentityCddRegisterDid)
            ));

            // Load test cases and run them.
            let test_data = add_investor_uniqueness_claim_v2_data(user.clone(), user_no_cdd_id);
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
    user: AccountId,
    user_no_cdd_id: AccountId,
) -> Vec<(
    (AccountId, Scope, Claim, v2::InvestorZKProofData),
    DispatchResult,
)> {
    let ticker = Ticker::default();
    let did = Identity::get_identity(&user).unwrap();
    let investor: InvestorUid = make_investor_uid(did.as_bytes()).into();
    let cdd_id = CddId::new_v2(did, investor.clone());
    let proof = v2::InvestorZKProofData::new(&did, &investor, &ticker);
    let claim = Claim::InvestorUniquenessV2(cdd_id);
    let scope = Scope::Ticker(ticker);
    let invalid_ticker = Ticker::from_slice_truncated(&b"1"[..]);
    let invalid_version_claim =
        Claim::InvestorUniqueness(Scope::Ticker(ticker), IdentityId::from(42u128), cdd_id);
    let invalid_proof = v2::InvestorZKProofData::new(&did, &investor, &invalid_ticker);

    vec![
        // Invalid claim.
        (
            (
                user.clone(),
                Scope::Ticker(invalid_ticker),
                claim.clone(),
                proof,
            ),
            Err(Error::InvalidScopeClaim.into()),
        ),
        // Valid ZKProof v2
        ((user.clone(), scope.clone(), claim.clone(), proof), Ok(())),
        // Not allowed claim.
        (
            (user.clone(), scope.clone(), Claim::NoData, proof),
            Err(Error::ClaimVariantNotAllowed.into()),
        ),
        // Missing CDD id.
        (
            (user_no_cdd_id, scope.clone(), claim.clone(), proof),
            Err(Error::InvalidCDDId.into()),
        ),
        // Invalid ZKProof
        (
            (user.clone(), scope.clone(), claim, invalid_proof),
            Err(Error::InvalidScopeClaim.into()),
        ),
        // Claim version does NOT match.
        (
            (user, scope.clone(), invalid_version_claim, proof),
            Err(Error::ClaimAndProofVersionsDoNotMatch.into()),
        ),
    ]
}

#[test]
fn ensure_custom_scopes_limited() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(AccountKeyring::Alice);
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
        let user = User::new(AccountKeyring::Alice);
        let case = |add| Identity::register_custom_claim_type(user.origin(), max_len_bytes(add));
        assert_too_long!(case(1));
        assert_ok!(case(0));
    });
}

#[test]
fn custom_claim_type_works() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(AccountKeyring::Alice);
        let register = |ty: &str| Identity::register_custom_claim_type(user.origin(), ty.into());
        let seq_is = |num| {
            assert_eq!(CustomClaimIdSequence::get().0, num);
        };
        let slot_has = |id, data: &str| {
            seq_is(id);
            let id = CustomClaimTypeId(id);
            let data = data.as_bytes();
            assert_eq!(CustomClaims::get(id), data);
            assert_eq!(CustomClaimsInverse::get(data), id);
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
        CustomClaimIdSequence::put(CustomClaimTypeId(u32::MAX));
        assert_noop!(register("qux"), BaseError::CounterOverflow);
    });
}

#[test]
fn invalid_custom_claim_type() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
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
fn cdd_register_did_events() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(|| {
            System::set_block_number(1);
            // Register an Identity for alice with two secundary keys
            let cdd_provider = Origin::signed(AccountKeyring::Eve.to_account_id());
            let alice_account_id = AccountKeyring::Alice.to_account_id();
            let alice_secundary_keys = vec![
                SecondaryKey::from_account_id(AccountKeyring::Dave.to_account_id()),
                SecondaryKey::from_account_id(AccountKeyring::Charlie.to_account_id()),
            ];
            assert_ok!(Identity::cdd_register_did(
                cdd_provider,
                alice_account_id.clone(),
                alice_secundary_keys.clone()
            ));
            let alice_did = get_identity_id(AccountKeyring::Alice).unwrap();
            // Make sure one Authorization event was sent for each secundary key
            let mut system_events = System::events();
            assert_eq!(
                system_events.pop().unwrap().event,
                super::storage::EventTest::Identity(RawEvent::AuthorizationAdded(
                    alice_did,
                    None,
                    Some(AccountKeyring::Charlie.to_account_id()),
                    Identity::multi_purpose_nonce(),
                    AuthorizationData::JoinIdentity(alice_secundary_keys[1].permissions.clone()),
                    None,
                ))
            );
            assert_eq!(
                system_events.pop().unwrap().event,
                super::storage::EventTest::Identity(RawEvent::AuthorizationAdded(
                    alice_did,
                    None,
                    Some(AccountKeyring::Dave.to_account_id()),
                    Identity::multi_purpose_nonce() - 1,
                    AuthorizationData::JoinIdentity(alice_secundary_keys[0].permissions.clone()),
                    None,
                ))
            );
            // Make sure a Did Created event was sent
            assert_eq!(
                system_events.pop().unwrap().event,
                super::storage::EventTest::Identity(RawEvent::DidCreated(
                    alice_did,
                    alice_account_id,
                    alice_secundary_keys.clone()
                ))
            );
        });
}

#[test]
fn child_identity_test() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(&do_child_identity_test);
}

fn do_child_identity_test() {
    let cdd = Origin::signed(AccountKeyring::Eve.to_account_id());
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);
    let dave = User::new_with(alice.did, AccountKeyring::Dave);

    let charlie = User::new(AccountKeyring::Charlie);

    // Helper functions.
    let did_of = |u: User| Identity::get_identity(&u.acc());
    let valid_cdd = |u: User| did_of(u).map(Identity::has_valid_cdd).unwrap_or_default();
    let inc_acc_ref = |u: User| Identity::add_account_key_ref_count(&u.acc());
    let rejoin_parent = |parent: User, child: User| {
        ParentDid::insert(child.did, parent.did);
    };

    // Create some secondary keys.
    add_secondary_key(alice.did, bob.acc());
    add_secondary_key(alice.did, dave.acc());

    // Check KeyRecords map
    assert_eq!(did_of(bob), Some(alice.did));
    assert_eq!(did_of(dave), Some(alice.did));
    assert!(valid_cdd(alice));
    assert!(valid_cdd(bob));
    assert!(valid_cdd(dave));

    // The new child identity's primary key must be a secondary key.
    exec_noop!(
        Identity::create_child_identity(alice.origin(), charlie.acc()),
        Error::NotASigner,
    );

    // The new child identity's primary key must be unlinkable (no account references).
    inc_acc_ref(dave);
    exec_noop!(
        Identity::create_child_identity(alice.origin(), dave.acc()),
        Error::AccountKeyIsBeingUsed,
    );

    // Only a secondary key from the parent identity can be used as the primary key for the child identity.
    exec_noop!(
        Identity::create_child_identity(alice.origin(), alice.acc()),
        Error::NotASigner,
    );

    // Only the primary key can create a child identity.
    exec_noop!(
        Identity::create_child_identity(bob.origin(), bob.acc()),
        Error::KeyNotAllowed,
    );

    // Create child identity with Bob as the primary key.
    exec_ok!(Identity::create_child_identity(alice.origin(), bob.acc()));
    // Update bob's identity.
    let bob_did = did_of(bob).expect("Bob's new identity");
    let bob = User::new_with(bob_did, AccountKeyring::Bob);

    // Ensure bob has a new identity.
    assert!(valid_cdd(bob));
    assert_ne!(bob.did, alice.did);
    let (bob_cdd_id, _bob_uid) = create_cdd_id_and_investor_uid(bob_did);

    // Attach secondary key to child identity.
    let ferdie = User::new_with(bob.did, AccountKeyring::Ferdie);
    add_secondary_key(bob.did, ferdie.acc());

    // Child identity can't create a child identity.
    exec_noop!(
        Identity::create_child_identity(bob.origin(), ferdie.acc()),
        Error::IsChildIdentity,
    );

    // Parent can force unlinking of child identity without CDD claim.
    exec_ok!(Identity::unlink_child_identity(alice.origin(), bob_did));
    rejoin_parent(alice, bob);

    // Child can force unlinking from parent identity without CDD claim.
    exec_ok!(Identity::unlink_child_identity(bob.origin(), bob_did));
    rejoin_parent(alice, bob);

    // Child identity can receive a CDD Claim.
    assert_ok!(Identity::add_claim(
        cdd.clone(),
        bob_did,
        Claim::CustomerDueDiligence(bob_cdd_id),
        None
    ));

    // Parent can unlink child identity (has CDD claim).
    exec_ok!(Identity::unlink_child_identity(alice.origin(), bob_did));
    rejoin_parent(alice, bob);

    // Child can unlink from parent identity.
    exec_ok!(Identity::unlink_child_identity(bob.origin(), bob_did));

    // Bob's identity doesn't have a parent.
    exec_noop!(
        Identity::unlink_child_identity(alice.origin(), bob_did),
        Error::NoParentIdentity,
    );

    // Rejoin parent identity for testing.
    rejoin_parent(alice, bob);

    // The caller's identity must be the parent or child.
    exec_noop!(
        Identity::unlink_child_identity(charlie.origin(), bob_did),
        Error::NotParentOrChildIdentity,
    );

    // Only the parent's primary key can unlink a child identity.
    exec_noop!(
        Identity::unlink_child_identity(dave.origin(), bob_did),
        Error::KeyNotAllowed,
    );

    // Only the child's primary key can unlink a child identity.
    exec_noop!(
        Identity::unlink_child_identity(ferdie.origin(), bob_did),
        Error::KeyNotAllowed,
    );

    // Unlink child from parent again.
    exec_ok!(Identity::unlink_child_identity(bob.origin(), bob_did));

    assert!(valid_cdd(bob));

    // Bob's identity is no longer a child identity.  It can create child identities.
    exec_ok!(Identity::create_child_identity(bob.origin(), ferdie.acc()));
    // Update ferdie's identity.
    let ferdie_did = did_of(ferdie).expect("Ferdie's new identity");
    let ferdie = User::new_with(ferdie_did, AccountKeyring::Ferdie);
    assert!(valid_cdd(ferdie));
}

#[test]
fn create_child_identities_with_auth_test() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(&do_create_child_identities_with_auth_test);
}

fn do_create_child_identities_with_auth_test() {
    let alice = User::new(AccountKeyring::Alice);
    let charlie = User::new_with(alice.did, AccountKeyring::Charlie);
    // Create some secondary keys.
    add_secondary_key(alice.did, charlie.acc());

    let expires_at = 100;
    let auth = || {
        let auth = TargetIdAuthorization {
            target_id: alice.did,
            nonce: Identity::offchain_authorization_nonce(alice.did),
            expires_at,
        };
        auth.encode()
    };
    let create_with_auth = |child: AccountKeyring| {
        let auth_encoded = auth();
        CreateChildIdentityWithAuth {
            key: child.to_account_id(),
            auth_signature: H512::from(child.sign(&auth_encoded)),
        }
    };

    // Try creating a child identity with a key already linked to another identity.
    let child_keys = vec![create_with_auth(charlie.ring)];
    assert_noop!(
        Identity::create_child_identities(alice.origin(), child_keys, expires_at),
        Error::AlreadyLinked
    );

    // Repeat a key.
    let child_keys = vec![
        create_with_auth(AccountKeyring::Bob),
        create_with_auth(AccountKeyring::Dave),
        create_with_auth(AccountKeyring::Ferdie),
        create_with_auth(AccountKeyring::Bob), // Duplicate key.
    ];
    assert_noop!(
        Identity::create_child_identities(alice.origin(), child_keys, expires_at),
        Error::DuplicateKey
    );

    // Create multiple child identities from unlinked keys.
    let child_keys = vec![
        create_with_auth(AccountKeyring::Bob),
        create_with_auth(AccountKeyring::Dave),
        create_with_auth(AccountKeyring::Ferdie),
    ];
    assert_ok!(Identity::create_child_identities(
        alice.origin(),
        child_keys,
        expires_at
    ));
}
