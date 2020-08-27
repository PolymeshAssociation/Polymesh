use super::{
    pips_test::assert_balance,
    storage::{register_keyring_account_with_balance, Call, EventTest, TestStorage},
    ExtBuilder,
};
use pallet_balances::{self as balances, Call as BalancesCall};
use pallet_utility::{self as utility, Event};
use polymesh_common_utilities::traits::transaction_payment::CddAndFeeDetails;

use codec::Encode;
use frame_support::{assert_err, assert_ok, dispatch::DispatchError};
use pallet_utility::UniqueCall;
use sp_core::sr25519::Signature;
use test_client::AccountKeyring;

type Balances = balances::Module<TestStorage>;
type Utility = utility::Module<TestStorage>;
type Error = utility::Error<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type System = frame_system::Module<TestStorage>;

fn transfer(to: Public, amount: u128) -> Call {
    Call::Balances(BalancesCall::transfer(to, amount))
}

const ERROR: DispatchError = DispatchError::Module {
    index: 0,
    error: 2,
    message: None,
};

fn assert_event(event: Event) {
    assert_eq!(
        System::events().pop().unwrap().event,
        EventTest::pallet_utility(event)
    )
}

fn batch_test(test: impl FnOnce(Public, Public)) {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let alice = AccountKeyring::Alice.public();
        TestStorage::set_payer_context(Some(alice));
        let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

        let bob = AccountKeyring::Bob.public();
        TestStorage::set_payer_context(Some(bob));
        let _ = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

        assert_balance(alice, 959, 0);
        assert_balance(bob, 959, 0);

        test(alice, bob)
    });
}

#[test]
fn batch_with_signed_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob, 400), transfer(bob, 400)];
        assert_ok!(Utility::batch(Origin::signed(alice), calls));
        assert_balance(alice, 159, 0);
        assert_balance(bob, 959 + 400 + 400, 0);
        assert_event(Event::BatchCompleted);
    });
}

#[test]
fn batch_early_exit_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob, 400), transfer(bob, 900), transfer(bob, 400)];
        assert_ok!(Utility::batch(Origin::signed(alice), calls));
        assert_balance(alice, 559, 0);
        assert_balance(bob, 959 + 400, 0);
        assert_event(Event::BatchInterrupted(1, ERROR));
    })
}

#[test]
fn batch_optimistic_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob, 401), transfer(bob, 402)];
        assert_ok!(Utility::batch_optimistic(Origin::signed(alice), calls));
        assert_event(Event::BatchCompleted);
        assert_balance(alice, 959 - 401 - 402, 0);
        assert_balance(bob, 959 + 401 + 402, 0);
    });
}

#[test]
fn batch_optimistic_failures_listed() {
    batch_test(|alice, bob| {
        assert_ok!(Utility::batch_optimistic(
            Origin::signed(alice),
            vec![
                transfer(bob, 401), // YAY.
                transfer(bob, 900), // NAY.
                transfer(bob, 800), // NAY.
                transfer(bob, 402), // YAY.
                transfer(bob, 403), // NAY.
            ]
        ));
        assert_event(Event::BatchOptimisticFailed(vec![
            (1, ERROR),
            (2, ERROR),
            (4, ERROR),
        ]));
        assert_balance(alice, 959 - 401 - 402, 0);
        assert_balance(bob, 959 + 401 + 402, 0);
    });
}

#[test]
fn batch_atomic_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob, 401), transfer(bob, 402)];
        assert_ok!(Utility::batch_atomic(Origin::signed(alice), calls));
        assert_event(Event::BatchCompleted);
        assert_balance(alice, 959 - 401 - 402, 0);
        assert_balance(bob, 959 + 401 + 402, 0);
    });
}

#[test]
fn batch_atomic_early_exit_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob, 400), transfer(bob, 900), transfer(bob, 400)];
        assert_ok!(Utility::batch_atomic(Origin::signed(alice), calls));
        assert_balance(alice, 959, 0);
        assert_balance(bob, 959, 0);
        assert_event(Event::BatchInterrupted(1, ERROR));
    })
}

#[test]
fn relay_happy_case() {
    ExtBuilder::default()
        .build()
        .execute_with(_relay_happy_case);
}

fn _relay_happy_case() {
    let alice = AccountKeyring::Alice.public();
    let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

    let bob = AccountKeyring::Bob.public();
    let _ = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

    let charlie = AccountKeyring::Charlie.public();
    let _ = register_keyring_account_with_balance(AccountKeyring::Charlie, 1_000).unwrap();

    assert_balance(bob, 1000, 0);
    assert_balance(charlie, 1000, 0);

    let origin = Origin::signed(alice);
    let transaction = UniqueCall::new(
        Utility::nonce(bob),
        Call::Balances(BalancesCall::transfer(charlie, 50)),
    );

    assert_ok!(Utility::relay_tx(
        origin,
        bob,
        AccountKeyring::Bob.sign(&transaction.encode()).into(),
        transaction
    ));

    assert_balance(bob, 950, 0);
    assert_balance(charlie, 1_050, 0);
}

#[test]
fn relay_unhappy_cases() {
    ExtBuilder::default()
        .build()
        .execute_with(_relay_unhappy_cases);
}

fn _relay_unhappy_cases() {
    let alice = AccountKeyring::Alice.public();
    let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

    let bob = AccountKeyring::Bob.public();

    let charlie = AccountKeyring::Charlie.public();

    let origin = Origin::signed(alice);
    let transaction = UniqueCall::new(
        Utility::nonce(bob),
        Call::Balances(BalancesCall::transfer(charlie, 59)),
    );

    assert_err!(
        Utility::relay_tx(
            origin.clone(),
            bob,
            Signature::default().into(),
            transaction.clone()
        ),
        Error::InvalidSignature
    );

    assert_err!(
        Utility::relay_tx(
            origin.clone(),
            bob,
            AccountKeyring::Bob.sign(&transaction.encode()).into(),
            transaction.clone()
        ),
        Error::TargetCddMissing
    );

    let _ = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

    let transaction = UniqueCall::new(
        Utility::nonce(bob) + 1,
        Call::Balances(BalancesCall::transfer(charlie, 59)),
    );

    assert_err!(
        Utility::relay_tx(
            origin.clone(),
            bob,
            Signature::default().into(),
            transaction
        ),
        Error::InvalidNonce
    );
}
