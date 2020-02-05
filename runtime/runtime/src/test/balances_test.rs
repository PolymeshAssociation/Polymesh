use crate::{
    runtime,
    test::{
        storage::{make_account, TestStorage},
        ExtBuilder,
    },
    Runtime,
};
use polymesh_runtime_balances as balances;
use polymesh_runtime_identity as identity;

use frame_support::{
    assert_err, assert_ok,
    weights::{DispatchInfo, Weight},
};
use pallet_transaction_payment::ChargeTransactionPayment;
use sp_runtime::traits::SignedExtension;
use test_client::AccountKeyring;

pub type Balances = balances::Module<TestStorage>;

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
