use super::{storage::TestStorage, ExtBuilder};
use pallet_balances::{self as balances, Event as BalancesRawEvent};
use pallet_identity as identity;
use polymesh_runtime_develop::{runtime, Runtime};

use frame_support::{
    assert_ok,
    dispatch::{DispatchInfo, Weight},
    traits::Currency,
};
use pallet_transaction_payment::ChargeTransactionPayment;
use polymesh_primitives::Memo;
use sp_keyring::AccountKeyring;
use sp_runtime::traits::SignedExtension;

pub type Balances = balances::Pallet<TestStorage>;
pub type System = frame_system::Pallet<TestStorage>;
type Identity = identity::Pallet<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type Error = balances::Error<TestStorage>;

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: u64) -> DispatchInfo {
    DispatchInfo {
        weight: Weight::from_parts(w, 0),
        ..Default::default()
    }
}

#[test]
#[ignore]
fn signed_extension_charge_transaction_payment_work() {
    ExtBuilder::default()
        .balance_factor(10)
        .transaction_fees(0, 1, 5)
        .monied(true)
        .build()
        .execute_with(|| {
            let len = 10;
            let alice_id = AccountKeyring::Alice.to_account_id();

            let call = runtime::RuntimeCall::System(frame_system::Call::remark { remark: vec![] });

            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0),
                    &alice_id,
                    &call,
                    &info_from_weight(5),
                    len
                )
                .is_ok()
            );
            assert_eq!(Balances::free_balance(&alice_id), 100 - 20 - 25);
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0 /* 0 tip */),
                    &alice_id,
                    &call,
                    &info_from_weight(3),
                    len
                )
                .is_ok()
            );
            assert_eq!(Balances::free_balance(&alice_id), 100 - 20 - 25 - 20 - 15);
        });
}

#[test]
fn tipping_fails() {
    ExtBuilder::default()
        .balance_factor(10)
        .transaction_fees(0, 1, 5)
        .monied(true)
        .build()
        .execute_with(|| {
            let alice_id = AccountKeyring::Alice.to_account_id();
            let call = runtime::RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
            let len = 10;
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(5 /* 5 tip */),
                    &alice_id,
                    &call,
                    &info_from_weight(3),
                    len
                )
                .is_err()
            );
        });
}

#[test]
fn transfer_with_memo() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Ferdie.to_account_id()])
        .build()
        .execute_with(transfer_with_memo_we);
}

fn transfer_with_memo_we() {
    let alice = AccountKeyring::Alice.to_account_id();
    let bob = AccountKeyring::Bob.to_account_id();

    let memo_1 = Some(Memo([7u8; 32]));
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice.clone()),
        bob.clone().into(),
        100,
        memo_1.clone()
    ),);
    Balances::make_free_balance_be(&bob, 0);
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice.clone()),
        bob.clone().into(),
        100,
        memo_1.clone()
    ));
    System::set_block_number(2);
    let memo_2 = Some(Memo([42u8; 32]));
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice.clone()),
        bob.clone().into(),
        200,
        memo_2.clone()
    ));

    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice.clone()),
        bob.clone().into(),
        300,
        None
    ));

    //// Ignoring `frame_system` events
    let mut system_events = System::events();
    assert_eq!(
        system_events.pop().unwrap().event,
        crate::storage::RuntimeEvent::Balances(BalancesRawEvent::TransferMemo {
            from: AccountKeyring::Alice.to_account_id(),
            to: AccountKeyring::Bob.to_account_id(),
            amount: 300,
            memo: None
        })
    );
    assert_eq!(
        system_events.pop().unwrap().event,
        crate::storage::RuntimeEvent::Balances(BalancesRawEvent::Transfer {
            from: AccountKeyring::Alice.to_account_id(),
            to: AccountKeyring::Bob.to_account_id(),
            amount: 300,
        })
    );
    assert_eq!(
        system_events.pop().unwrap().event,
        crate::storage::RuntimeEvent::Balances(BalancesRawEvent::TransferMemo {
            from: AccountKeyring::Alice.to_account_id(),
            to: AccountKeyring::Bob.to_account_id(),
            amount: 200,
            memo: memo_2
        })
    );
    assert_eq!(
        system_events.pop().unwrap().event,
        crate::storage::RuntimeEvent::Balances(BalancesRawEvent::Transfer {
            from: AccountKeyring::Alice.to_account_id(),
            to: AccountKeyring::Bob.to_account_id(),
            amount: 200,
        })
    );
}
