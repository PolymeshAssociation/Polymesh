mod common;
use common::{
    storage::{make_account, make_account_with_balance, EventTest, TestStorage},
    ExtBuilder,
};
use polymesh_runtime::{runtime, Runtime};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::balances::{Memo, RawEvent as BalancesRawEvent};
use polymesh_runtime_identity as identity;

use frame_support::{
    assert_err, assert_ok,
    traits::{Currency, ExistenceRequirement},
    weights::{DispatchInfo, Weight},
};
use frame_system::{EventRecord, Phase};
use pallet_transaction_payment::ChargeTransactionPayment;
use polymesh_primitives::traits::BlockRewardsReserveCurrency;
use sp_runtime::traits::SignedExtension;
use test_client::AccountKeyring;

pub type Balances = balances::Module<TestStorage>;
pub type System = frame_system::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Error = balances::Error<TestStorage>;

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
    DispatchInfo {
        weight: w,
        ..Default::default()
    }
}

#[test]
#[ignore]
fn signed_extension_charge_transaction_payment_work() {
    ExtBuilder::default()
        .existential_deposit(10)
        .transaction_fees(10, 1, 5)
        .monied(true)
        .build()
        .execute_with(|| {
            let len = 10;
            let alice_pub = AccountKeyring::Alice.public();
            let alice_id = AccountKeyring::Alice.to_account_id();

            let call = runtime::Call::Identity(identity::Call::register_did(vec![]));

            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0),
                    &alice_id,
                    &call,
                    info_from_weight(5),
                    len
                )
                .is_ok()
            );
            assert_eq!(Balances::free_balance(&alice_pub), 100 - 20 - 25);
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0 /* 0 tip */),
                    &alice_id,
                    &call,
                    info_from_weight(3),
                    len
                )
                .is_ok()
            );
            assert_eq!(Balances::free_balance(&alice_pub), 100 - 20 - 25 - 20 - 15);
        });
}

#[test]
fn tipping_fails() {
    ExtBuilder::default()
        .existential_deposit(10)
        .transaction_fees(10, 1, 5)
        .monied(true)
        .build()
        .execute_with(|| {
            let call = runtime::Call::Identity(identity::Call::register_did(vec![]));
            let len = 10;
            let alice_id = AccountKeyring::Alice.to_account_id();
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(5 /* 5 tip */),
                    &alice_id,
                    &call,
                    info_from_weight(3),
                    len
                )
                .is_err()
            );
        });
}
#[test]
fn mint_subsidy_works() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Ferdie.public()])
        .build()
        .execute_with(|| {
            let brr = Balances::block_rewards_reserve();
            assert_eq!(Balances::free_balance(&brr), 0);
            let mut ti = Balances::total_issuance();
            let alice = AccountKeyring::Alice.public();
            let mut balance_alice = Balances::free_balance(&alice);

            // When there is no balance in BRR, minting should increase total supply
            assert_ok!(Balances::deposit_into_existing(&alice, 10).map(drop));
            assert_eq!(Balances::free_balance(&alice), balance_alice + 10);
            assert_eq!(Balances::total_issuance(), ti + 10);
            ti = ti + 10;
            balance_alice = balance_alice + 10;

            // Funding BRR
            let eve_signed = Origin::signed(AccountKeyring::Eve.public());
            assert_ok!(Balances::top_up_brr_balance(eve_signed, 500,));
            assert_eq!(Balances::free_balance(&brr), 500);
            assert_eq!(Balances::total_issuance(), ti);

            // When BRR has enough funds to subsidize a mint fully, it should subsidize it.
            assert_ok!(Balances::deposit_into_existing(&alice, 100).map(drop));
            assert_eq!(Balances::free_balance(&brr), 400);
            assert_eq!(Balances::free_balance(&alice), balance_alice + 100);
            assert_eq!(Balances::total_issuance(), ti);
            balance_alice = balance_alice + 100;

            // When BRR has funds to subsidize a mint partially, it should subsidize it and rest should be minted.
            assert_ok!(Balances::deposit_into_existing(&alice, 1000).map(drop));
            assert_eq!(Balances::free_balance(&brr), 0);
            assert_eq!(Balances::free_balance(&alice), balance_alice + 1000);
            // 400 subsidized, 600 minted.
            assert_eq!(Balances::total_issuance(), ti + 600);
            ti = ti + 600;
            balance_alice = balance_alice + 1000;

            // When BRR has no funds to subsidize a mint, it should be fully minted.
            assert_ok!(Balances::deposit_into_existing(&alice, 100).map(drop));
            assert_eq!(Balances::free_balance(&brr), 0);
            assert_eq!(Balances::free_balance(&alice), balance_alice + 100);
            assert_eq!(Balances::total_issuance(), ti + 100);
        });
}

#[test]
fn issue_must_work() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Ferdie.public()])
        .build()
        .execute_with(|| {
            let init_total_issuance = Balances::total_issuance();
            let imbalance = Balances::burn(10);
            assert_eq!(Balances::total_issuance(), init_total_issuance - 10);
            drop(imbalance);
            assert_eq!(Balances::total_issuance(), init_total_issuance);

            let brr = Balances::block_rewards_reserve();
            assert_eq!(Balances::free_balance(&brr), 0);
            let mut ti = Balances::total_issuance();
            let _alice = AccountKeyring::Alice.public();

            // When there is no balance in BRR, issuance should increase total supply
            // NOTE: dropping negative imbalance is equivalent to burning. It will decrease total supply.
            let imbalance = Balances::issue_using_block_rewards_reserve(10);
            assert_eq!(Balances::total_issuance(), ti + 10);
            drop(imbalance);
            assert_eq!(Balances::total_issuance(), ti);

            // Funding BRR
            let eve = AccountKeyring::Eve.public();
            assert_err!(
                <Balances as Currency<_>>::transfer(
                    &eve,
                    &brr,
                    500,
                    ExistenceRequirement::AllowDeath
                ),
                Error::ReceiverCddMissing
            );
            let eve_signed = Origin::signed(AccountKeyring::Eve.public());
            assert_ok!(Balances::top_up_brr_balance(eve_signed, 500,));
            assert_eq!(Balances::free_balance(&brr), 500);
            assert_eq!(Balances::total_issuance(), ti);

            // When BRR has enough funds to subsidize a mint fully, it should subsidize it.
            let imbalance2 = Balances::issue_using_block_rewards_reserve(100);
            assert_eq!(Balances::total_issuance(), ti);
            assert_eq!(Balances::free_balance(&brr), 400);
            drop(imbalance2);
            assert_eq!(Balances::total_issuance(), ti - 100);
            ti = ti - 100;

            // When BRR has funds to subsidize a mint partially, it should subsidize it and rest should be minted.
            let imbalance3 = Balances::issue_using_block_rewards_reserve(1000);
            assert_eq!(Balances::total_issuance(), ti + 600);
            assert_eq!(Balances::free_balance(&brr), 0);
            drop(imbalance3);
            // NOTE: Since burned Poly reduces total supply rather than increasing BRR balance,
            // the new total supply is 1000 less after dropping.
            assert_eq!(Balances::total_issuance(), ti - 400);
            ti = ti - 400;

            // When BRR has no funds to subsidize a mint, it should be fully minted.
            let imbalance4 = Balances::issue_using_block_rewards_reserve(100);
            assert_eq!(Balances::total_issuance(), ti + 100);
            drop(imbalance4);
            assert_eq!(Balances::total_issuance(), ti);
        });
}

#[test]
#[ignore]
fn should_charge_identity() {
    ExtBuilder::default()
        .existential_deposit(10)
        .transaction_fees(10, 1, 5)
        .monied(true)
        .build()
        .execute_with(|| {
            let call = runtime::Call::Identity(identity::Call::register_did(vec![]));
            let dave_pub = AccountKeyring::Dave.public();
            let dave_id = AccountKeyring::Dave.to_account_id();
            let (signed_acc_id, acc_did) = make_account(dave_pub).unwrap();
            let len = 10;
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0 /* 0 tip */),
                    &dave_id,
                    &call,
                    info_from_weight(3),
                    len
                )
                .is_ok()
            );

            assert_ok!(Balances::change_charge_did_flag(
                signed_acc_id.clone(),
                true
            ));
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0 /* 0 tip */),
                    &dave_id,
                    &call,
                    info_from_weight(3),
                    len
                )
                .is_err()
            ); // no balance in identity
            assert_eq!(Balances::free_balance(&dave_pub), 365);
            assert_ok!(Balances::top_up_identity_balance(
                signed_acc_id.clone(),
                acc_did,
                300
            ));
            assert_eq!(Balances::free_balance(&dave_pub), 65);
            assert_eq!(Balances::identity_balance(acc_did), 300);
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0 /* 0 tip */),
                    &dave_id,
                    &call,
                    info_from_weight(3),
                    len
                )
                .is_ok()
            );
            assert_ok!(Balances::reclaim_identity_balance(
                signed_acc_id.clone(),
                acc_did,
                230
            ));
            assert_err!(
                Balances::reclaim_identity_balance(signed_acc_id, acc_did, 230),
                "too few free funds in account"
            );
            assert_eq!(Balances::free_balance(&dave_pub), 295);
            assert_eq!(Balances::identity_balance(acc_did), 35);
        });
}

#[test]
fn transfer_with_memo() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Ferdie.public()])
        .build()
        .execute_with(transfer_with_memo_we);
}

fn transfer_with_memo_we() {
    let alice = AccountKeyring::Alice.public();
    let bob = AccountKeyring::Bob.public();
    let memo_1 = Some(Memo([7u8; 32]));
    assert!(Balances::transfer_with_memo(Origin::signed(alice), bob, 100, memo_1.clone()).is_err());
    let _ = make_account_with_balance(bob, 0);
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice),
        bob,
        100,
        memo_1.clone()
    ));
    System::set_block_number(2);
    let memo_2 = Some(Memo([42u8; 32]));
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice),
        bob,
        200,
        memo_2.clone()
    ));

    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice),
        bob,
        300,
        None
    ));

    let expected_events = vec![
        EventRecord {
            phase: Phase::ApplyExtrinsic(0),
            event: EventTest::balances(BalancesRawEvent::TransferWithMemo(
                alice.clone(),
                bob.clone(),
                100,
                memo_1.unwrap(),
            )),
            topics: vec![],
        },
        EventRecord {
            phase: Phase::ApplyExtrinsic(0),
            event: EventTest::balances(BalancesRawEvent::TransferWithMemo(
                alice,
                bob,
                200,
                memo_2.unwrap(),
            )),
            topics: vec![],
        },
        EventRecord {
            phase: Phase::ApplyExtrinsic(0),
            event: EventTest::balances(BalancesRawEvent::Transfer(alice, bob, 300)),
            topics: vec![],
        },
    ];
    // Ignoring `frame_system` events
    let system_events = System::events();
    let emitted_events = vec![
        system_events[12].clone(),
        system_events[13].clone(),
        system_events[14].clone(),
    ];
    assert_eq!(emitted_events, expected_events);
}
