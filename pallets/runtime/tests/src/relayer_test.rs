use super::{
    storage::{get_last_auth_id, make_account_without_cdd, TestStorage, User},
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok, StorageMap};
use pallet_relayer::Subsidy;
use polymesh_common_utilities::traits::transaction_payment::CddAndFeeDetails;
use polymesh_primitives::{AccountId, Balance, Signatory};
use polymesh_runtime_develop::{fee_details::CddHandler, runtime::Call};
use test_client::AccountKeyring;

type Relayer = pallet_relayer::Module<TestStorage>;
type Subsidies = pallet_relayer::Subsidies<TestStorage>;

type Identity = pallet_identity::Module<TestStorage>;
type AccountKeyRefCount = pallet_identity::AccountKeyRefCount<TestStorage>;

type Error = pallet_relayer::Error<TestStorage>;

// Relayer Test Helper functions
// =======================================

#[track_caller]
fn assert_key_usage(user: User, usage: u64) {
    assert_eq!(AccountKeyRefCount::get(&user.acc()), usage);
}

fn get_subsidy(user: User) -> Option<Subsidy<AccountId>> {
    Subsidies::get(&user.acc())
}

#[track_caller]
fn assert_subsidy(user: User, subsidy: Option<(User, Balance)>) {
    assert_eq!(
        get_subsidy(user).map(|s| (s.paying_key, s.remaining)),
        subsidy.map(|s| (s.0.acc(), s.1))
    );
}

#[test]
fn basic_relayer_paying_key_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_basic_relayer_paying_key_test);
}
fn do_basic_relayer_paying_key_test() {
    let bob = User::new(AccountKeyring::Bob);
    let alice = User::new(AccountKeyring::Alice);
    let dave = User::new(AccountKeyring::Dave);

    // Add authorization for using Alice as the paying key for Bob.
    assert_ok!(Relayer::set_paying_key(alice.origin(), bob.acc(), 10u128));

    // The keys are not used yet.
    assert_key_usage(alice, 0);
    assert_key_usage(dave, 0);
    assert_key_usage(bob, 0);

    // No subsidy yet.
    assert_subsidy(bob, None);

    // Bob accept's the paying key.
    TestStorage::set_current_identity(&bob.did);
    let auth_id = get_last_auth_id(&Signatory::Account(bob.acc()));
    assert_ok!(Relayer::accept_paying_key(bob.origin(), auth_id));

    // Bob now has a subsidy of 10 POLYX.
    assert_subsidy(bob, Some((alice, 10u128)));

    // Alice's and Bob's keys are now used, but Dave's key is still unused.
    assert_key_usage(alice, 1);
    assert_key_usage(bob, 1);
    assert_key_usage(dave, 0);

    // Bob tries to increase his Polyx limit.  Not allowed
    assert_noop!(
        Relayer::update_polyx_limit(bob.origin(), bob.acc(), 12345u128),
        Error::NotPayingKey
    );

    // Alice updates the Polyx limit for Bob.  Allowed
    TestStorage::set_current_identity(&alice.did);
    assert_ok!(Relayer::update_polyx_limit(
        alice.origin(),
        bob.acc(),
        10_000u128
    ));

    // Bob now has a subsidy of 10,000 POLYX.
    assert_subsidy(bob, Some((alice, 10_000u128)));

    // Dave tries to remove the paying key from Bob's key.  Not allowed.
    TestStorage::set_current_identity(&dave.did);
    assert_noop!(
        Relayer::remove_paying_key(dave.origin(), bob.acc(), alice.acc()),
        Error::NotAuthorizedForUserKey
    );

    // Dave tries to remove the wrong paying key from Bob's key.  Not allowed.
    TestStorage::set_current_identity(&dave.did);
    assert_noop!(
        Relayer::remove_paying_key(dave.origin(), bob.acc(), dave.acc()),
        Error::NotPayingKey
    );

    // Add authorization for using Dave as the paying key for Bob.
    assert_ok!(Relayer::set_paying_key(dave.origin(), bob.acc(), 0u128));

    // Bob tries to accept the new paying key, but he already has a paying key.
    TestStorage::set_current_identity(&bob.did);
    let auth_id = get_last_auth_id(&Signatory::Account(bob.acc()));
    assert_noop!(
        Relayer::accept_paying_key(bob.origin(), auth_id),
        Error::AlreadyHasPayingKey
    );

    // Alice tries to remove the paying key from Bob's key.  Allowed.
    TestStorage::set_current_identity(&alice.did);
    assert_ok!(Relayer::remove_paying_key(
        alice.origin(),
        bob.acc(),
        alice.acc(),
    ));

    // Bob no longer has a subsidy.
    assert_subsidy(bob, None);

    // Check alice's key is not used any more.
    assert_key_usage(alice, 0);

    // Alice tries to update the poly limit for Bob,
    // but Bob no longer has a subsiy.
    assert_noop!(
        Relayer::update_polyx_limit(alice.origin(), bob.acc(), 42u128),
        Error::NoPayingKey
    );

    // Alice tries to remove the paying key a second time,
    // but Bob no longer has a subsiy.
    assert_noop!(
        Relayer::remove_paying_key(alice.origin(), bob.acc(), alice.acc()),
        Error::NoPayingKey
    );
}

#[test]
fn relayer_user_key_missing_cdd_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_relayer_user_key_missing_cdd_test);
}
fn do_relayer_user_key_missing_cdd_test() {
    let alice = User::new(AccountKeyring::Alice);
    let bob_acc = AccountKeyring::Bob.to_account_id();
    let (bob_sign, bob_did) = make_account_without_cdd(bob_acc.clone()).unwrap();

    // Add authorization for using Alice as the paying key for Bob.
    assert_ok!(Relayer::set_paying_key(
        alice.origin(),
        bob_acc.clone(),
        10u128
    ));

    // Bob tries to accept the paying key, without having a CDD.
    TestStorage::set_current_identity(&bob_did);
    let auth_id = get_last_auth_id(&Signatory::Account(bob_acc.clone()));
    assert_noop!(
        Relayer::accept_paying_key(bob_sign, auth_id),
        Error::UserKeyCddMissing
    );
}

#[test]
fn relayer_paying_key_missing_cdd_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_relayer_paying_key_missing_cdd_test);
}
fn do_relayer_paying_key_missing_cdd_test() {
    let alice = User::new(AccountKeyring::Alice);
    let bob_acc = AccountKeyring::Bob.to_account_id();
    let (bob_sign, bob_did) = make_account_without_cdd(bob_acc.clone()).unwrap();

    // Add authorization for using Bob as the paying key for Alice.
    assert_ok!(Relayer::set_paying_key(bob_sign, alice.acc(), 10u128));

    // Alice tries to accept the paying key, but the paying key
    // is without a CDD.
    TestStorage::set_current_identity(&alice.did);
    let auth_id = get_last_auth_id(&Signatory::Account(alice.acc()));
    assert_noop!(
        Relayer::accept_paying_key(alice.origin(), auth_id),
        Error::PayingKeyCddMissing
    );
}

/// TODO: Add tests for subsidiser interface.

#[test]
fn relayer_accept_cdd_and_fees_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_relayer_accept_cdd_and_fees_test);
}
fn do_relayer_accept_cdd_and_fees_test() {
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);
    let bob_sign = Signatory::Account(bob.acc());

    // Alice creates authoration to subsidise for Bob.
    assert_ok!(Relayer::set_paying_key(alice.origin(), bob.acc(), 0u128));
    let bob_auth_id = get_last_auth_id(&bob_sign);

    // Check that Bob can accept the subsidy with Alice paying for the transaction.
    assert_eq!(
        CddHandler::get_valid_payer(
            &Call::Relayer(pallet_relayer::Call::accept_paying_key(bob_auth_id)),
            &bob.acc()
        ),
        Ok(Some(alice.acc()))
    );
}
