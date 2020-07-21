use super::{
    storage::{register_keyring_account_with_balance, Call, TestStorage},
    ExtBuilder,
};
use frame_support::assert_ok;
use pallet_balances::{self as balances, Call as BalancesCall};
use pallet_utility as utility;
use polymesh_common_utilities::traits::transaction_payment::CddAndFeeDetails;
use polymesh_primitives::Signatory;
use test_client::AccountKeyring;

type Balances = balances::Module<TestStorage>;
type Utility = utility::Module<TestStorage>;
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
