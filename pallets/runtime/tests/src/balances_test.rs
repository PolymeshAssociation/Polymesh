use frame_support::assert_ok;
use frame_support::dispatch::DispatchInfo;
use frame_support::traits::Currency;
use frame_support::weights::Weight;
use sp_keyring::Sr25519Keyring;
use sp_runtime::traits::{TransactionExtension, TxBaseImplication};
use sp_runtime::transaction_validity::TransactionSource;

use pallet_balances::{self as balances, Event as BalancesRawEvent};
use pallet_identity as identity;
use pallet_transaction_payment::{ChargeTransactionPayment, Val};
use polymesh_primitives::Memo;

use super::storage::{RuntimeCall as StorageRuntimeCall, TestStorage};
use super::ExtBuilder;

pub type Balances = balances::Pallet<TestStorage>;
pub type System = frame_system::Pallet<TestStorage>;
type Identity = identity::Pallet<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type Error = balances::Error<TestStorage>;

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: u64) -> DispatchInfo {
    DispatchInfo {
        call_weight: Weight::from_parts(w, 0),
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
            let alice_acc = Sr25519Keyring::Alice.to_account_id();
            let charge_tx_payment = ChargeTransactionPayment::<TestStorage>::from(0);
            let call = StorageRuntimeCall::System(frame_system::Call::remark { remark: vec![] });

            assert!(charge_tx_payment
                .clone()
                .prepare(
                    Val::NoCharge,
                    &Origin::signed(alice_acc.clone()),
                    &call,
                    &info_from_weight(5),
                    10
                )
                .is_ok());

            assert_eq!(Balances::free_balance(&alice_acc), 100 - 20 - 25);

            assert!(charge_tx_payment
                .prepare(
                    Val::NoCharge,
                    &Origin::signed(alice_acc.clone()),
                    &call,
                    &info_from_weight(3),
                    10
                )
                .is_ok());

            assert_eq!(Balances::free_balance(&alice_acc), 100 - 20 - 25 - 20 - 15);
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
            assert!(ChargeTransactionPayment::<TestStorage>::from(5)
                .validate(
                    Origin::signed(Sr25519Keyring::Alice.to_account_id()),
                    &StorageRuntimeCall::System(frame_system::Call::remark { remark: vec![] }),
                    &info_from_weight(3),
                    10,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .is_err());
        });
}

#[test]
fn transfer_with_memo() {
    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .cdd_providers(vec![Sr25519Keyring::Ferdie.to_account_id()])
        .build()
        .execute_with(transfer_with_memo_we);
}

fn transfer_with_memo_we() {
    let alice = Sr25519Keyring::Alice.to_account_id();
    let bob = Sr25519Keyring::Bob.to_account_id();

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
        crate::storage::RuntimeEvent::Balances(BalancesRawEvent::TransferWithMemo {
            from: Sr25519Keyring::Alice.to_account_id(),
            to: Sr25519Keyring::Bob.to_account_id(),
            amount: 300,
            memo: None
        })
    );
    assert_eq!(
        system_events.pop().unwrap().event,
        crate::storage::RuntimeEvent::Balances(BalancesRawEvent::Transfer {
            from: Sr25519Keyring::Alice.to_account_id(),
            to: Sr25519Keyring::Bob.to_account_id(),
            amount: 300,
        })
    );
    assert_eq!(
        system_events.pop().unwrap().event,
        crate::storage::RuntimeEvent::Balances(BalancesRawEvent::TransferWithMemo {
            from: Sr25519Keyring::Alice.to_account_id(),
            to: Sr25519Keyring::Bob.to_account_id(),
            amount: 200,
            memo: memo_2
        })
    );
    assert_eq!(
        system_events.pop().unwrap().event,
        crate::storage::RuntimeEvent::Balances(BalancesRawEvent::Transfer {
            from: Sr25519Keyring::Alice.to_account_id(),
            to: Sr25519Keyring::Bob.to_account_id(),
            amount: 200,
        })
    );
}
