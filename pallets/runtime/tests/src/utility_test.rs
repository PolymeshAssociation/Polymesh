use super::{
    storage::{register_keyring_account_with_balance, Call, TestStorage},
    ExtBuilder,
};
use pallet_balances::{self as balances, Call as BalancesCall};
use pallet_utility as utility;
use polymesh_common_utilities::traits::transaction_payment::CddAndFeeDetails;

use codec::Encode;
use frame_support::{assert_err, assert_ok};
use pallet_utility::UniqueCall;
use polymesh_primitives::Signatory;
use sp_core::sr25519::Signature;
use test_client::AccountKeyring;

type Balances = balances::Module<TestStorage>;
type Utility = utility::Module<TestStorage>;
type Error = utility::Error<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn batch_with_signed_works() {
    ExtBuilder::default()
        .build()
        .execute_with(batch_with_signed_works_we);
}

fn batch_with_signed_works_we() {
    let alice = AccountKeyring::Alice.public();
    TestStorage::set_payer_context(Some(Signatory::Account(alice)));
    let _alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

    let bob = AccountKeyring::Bob.public();
    TestStorage::set_payer_context(Some(Signatory::Account(bob)));
    let _bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

    assert_eq!(Balances::free_balance(alice), 959);
    assert_eq!(Balances::free_balance(bob), 959);

    let batched_calls = vec![
        Call::Balances(BalancesCall::transfer(bob, 400)),
        Call::Balances(BalancesCall::transfer(bob, 400)),
    ];

    assert_ok!(Utility::batch(Origin::signed(alice), batched_calls));
    assert_eq!(Balances::free_balance(alice), 159);
    assert_eq!(Balances::free_balance(bob), 959 + 400 + 400);
}

#[test]
fn batch_early_exit_works() {
    ExtBuilder::default()
        .build()
        .execute_with(batch_early_exit_works_we);
}

fn batch_early_exit_works_we() {
    let alice = AccountKeyring::Alice.public();
    TestStorage::set_payer_context(Some(Signatory::Account(alice)));
    let _alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

    let bob = AccountKeyring::Bob.public();
    TestStorage::set_payer_context(Some(Signatory::Account(bob)));
    let _bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

    assert_eq!(Balances::free_balance(alice), 959);
    assert_eq!(Balances::free_balance(bob), 959);

    let batched_calls = vec![
        Call::Balances(BalancesCall::transfer(bob, 400)),
        Call::Balances(BalancesCall::transfer(bob, 900)),
        Call::Balances(BalancesCall::transfer(bob, 400)),
    ];

    assert_ok!(Utility::batch(Origin::signed(alice), batched_calls));
    assert_eq!(Balances::free_balance(alice), 559);
    assert_eq!(Balances::free_balance(bob), 959 + 400);
}

#[test]
fn relay_happy_case() {
    ExtBuilder::default()
        .build()
        .execute_with(_relay_happy_case);
}

fn _relay_happy_case() {
    let alice = AccountKeyring::Alice.public();
    let _alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

    let bob = AccountKeyring::Bob.public();
    let _bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

    let charlie = AccountKeyring::Charlie.public();
    let _charlie_did =
        register_keyring_account_with_balance(AccountKeyring::Charlie, 1_000).unwrap();

    assert_eq!(Balances::free_balance(bob), 1000);
    assert_eq!(Balances::free_balance(charlie), 1000);

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

    assert_eq!(Balances::free_balance(bob), 950);
    assert_eq!(Balances::free_balance(charlie), 1_050);
}

#[test]
fn relay_unhappy_cases() {
    ExtBuilder::default()
        .build()
        .execute_with(_relay_unhappy_cases);
}

fn _relay_unhappy_cases() {
    let alice = AccountKeyring::Alice.public();
    let _alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

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

    let _bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

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
