use super::{
    storage::{
        get_last_auth_id,
        TestStorage, User,
    },
    ExtBuilder,
};
use frame_support::{
    assert_noop, assert_ok,
};
use polymesh_common_utilities::{
    traits::{
        transaction_payment::CddAndFeeDetails,
    },
};
use polymesh_primitives::{
    Signatory,
};
use test_client::AccountKeyring;

type Relayer = pallet_relayer::Module<TestStorage>;

type Error = pallet_relayer::Error<TestStorage>;

// Relayer Test Helper functions
// =======================================

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

    // add authorization for using Alice as the paying key for bob
    assert_ok!(Relayer::set_paying_key(
        alice.origin(),
        bob.acc(),
    ));

    // Bob accept's the paying key
    TestStorage::set_current_identity(&bob.did);
    let auth_id = get_last_auth_id(&Signatory::Account(bob.acc()));
    assert_ok!(Relayer::accept_paying_key(
        bob.origin(),
        auth_id
    ));

    // Bob tries to increase his Polyx limit.  Not allowed
    TestStorage::set_current_identity(&bob.did);
    assert_noop!(
        Relayer::update_polyx_limit(
            bob.origin(),
            bob.acc(),
            12345u128
        ),
        Error::NotPayingKey
    );

    // Alice updates the Polyx limit for Bob.  Allowed
    TestStorage::set_current_identity(&alice.did);
    assert_ok!(Relayer::update_polyx_limit(
        alice.origin(),
        bob.acc(),
        10_000u128
    ));

    // Dave tries to remove the paying key from Bob's key.  Not allowed.
    TestStorage::set_current_identity(&dave.did);
    assert_noop!(
        Relayer::remove_paying_key(
            dave.origin(),
            bob.acc(),
            alice.acc(),
        ),
        Error::NotAuthorizedForUserKey
    );

    // add authorization for using Dave as the paying key for bob
    TestStorage::set_current_identity(&dave.did);
    assert_ok!(Relayer::set_paying_key(
        dave.origin(),
        bob.acc(),
    ));

    // Bob tries to accept the new paying key, but he already has a paying key
    TestStorage::set_current_identity(&bob.did);
    let auth_id = get_last_auth_id(&Signatory::Account(bob.acc()));
    assert_noop!(
        Relayer::accept_paying_key(
            bob.origin(),
            auth_id
        ),
        Error::AlreadyHasPayingKey
    );

    // Alice tries to remove the paying key from Bob's key.  Allowed.
    TestStorage::set_current_identity(&alice.did);
    assert_ok!(Relayer::remove_paying_key(
        alice.origin(),
        bob.acc(),
        alice.acc(),
    ));
}

