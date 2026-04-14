use frame_support::dispatch::{DispatchInfo, Pays, PostDispatchInfo};
use frame_support::weights::Weight;
use frame_support::{assert_noop, assert_ok};
use frame_system;
use polymesh_primitives::crypto::{ChainScopedMessage, RELAY_TX_LABEL};
use sp_keyring::Sr25519Keyring;
use sp_runtime::traits::{Dispatchable, TransactionExtension, TxBaseImplication};
use sp_runtime::transaction_validity::TransactionSource;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};
use sp_runtime::MultiAddress;
use sp_runtime::MultiSignature;

use pallet_balances::Call as BalancesCall;
use pallet_relayer::{RelayTxNonces, Subsidy};
use polymesh_primitives::constants::currency::POLY;
use polymesh_primitives::protocol_fee::ProtocolOp;
use polymesh_primitives::traits::CurrentFeePayer;
use polymesh_primitives::transaction_payment::CallPaymentInfo;
use polymesh_primitives::{AccountId, Balance, Ticker, TransactionError};
use polymesh_runtime_develop::runtime::{RuntimeCall as DevRuntimeCall, TxFeeHandler};
use polymesh_transaction_payment::Val;

use super::asset_test::set_timestamp;
use super::pips_test::assert_balance;
use super::storage::{
    register_keyring_account_with_balance, EventTest, RuntimeCall, System, TestStorage, User,
};
use super::ExtBuilder;

type Relayer = pallet_relayer::Pallet<TestStorage>;
type Subsidies = pallet_relayer::Subsidies<TestStorage>;
type Identity = pallet_identity::Pallet<TestStorage>;
type AccountKeyRefCount = pallet_identity::AccountKeyRefCount<TestStorage>;
type Balances = pallet_balances::Pallet<TestStorage>;
type ProtocolFee = pallet_protocol_fee::Pallet<TestStorage>;
type TransactionPayment = pallet_transaction_payment::Pallet<TestStorage>;
type ChargeTransactionPayment = polymesh_transaction_payment::ChargeTransactionPayment<TestStorage>;
type Error = pallet_relayer::Error<TestStorage>;
type IdentityError = pallet_identity::Error<TestStorage>;
type RuntimeOrigin = <TestStorage as frame_system::Config>::RuntimeOrigin;

// Relayer Test Helper functions
// =======================================

fn call_balance_transfer(val: Balance) -> RuntimeCall {
    RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
        dest: MultiAddress::Id(Sr25519Keyring::Alice.to_account_id()),
        value: val,
    })
}

fn call_system_remark(size: usize) -> RuntimeCall {
    RuntimeCall::System(frame_system::Call::remark {
        remark: vec![0; size],
    })
}

fn call_utility_batch(calls: Vec<RuntimeCall>) -> RuntimeCall {
    RuntimeCall::Utility(pallet_utility::Call::batch { calls })
}

fn call_utility_batch_all(calls: Vec<RuntimeCall>) -> RuntimeCall {
    RuntimeCall::Utility(pallet_utility::Call::batch_all { calls })
}

fn call_asset_register_ticker(name: &[u8]) -> RuntimeCall {
    let ticker = Ticker::from_slice_truncated(name);
    RuntimeCall::Asset(pallet_asset::Call::register_unique_ticker { ticker })
}

fn call_relayer_remove_subsidy(user_key: AccountId, paying_key: AccountId) -> RuntimeCall {
    RuntimeCall::Relayer(pallet_relayer::Call::remove_subsidy {
        user_key,
        paying_key,
    })
}

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: u64) -> DispatchInfo {
    DispatchInfo {
        call_weight: Weight::from_parts(w, 0),
        ..Default::default()
    }
}

fn post_info_from_weight(w: u64) -> PostDispatchInfo {
    PostDispatchInfo {
        actual_weight: Some(Weight::from_parts(w, 0)),
        pays_fee: Pays::Yes,
    }
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

fn assert_invalid_subsidy_call(caller: &AccountId, call: &RuntimeCall) {
    let origin = RuntimeOrigin::signed(caller.clone());

    let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
        TransactionError::PalletNotSubsidised as u8,
    ));

    let pre_err = ChargeTransactionPayment::from(0)
        .validate(
            origin.clone(),
            call,
            &info_from_weight(5),
            10,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .map(|_| ())
        .unwrap_err();
    assert_eq!(pre_err, expected_err);

    let val_charge = Val::Charge {
        tip: 0,
        who: caller.clone(),
        fee_with_tip: 1,
        subsidiser: None,
    };
    let pre_err = ChargeTransactionPayment::from(0)
        .prepare(val_charge, &origin, call, &info_from_weight(5), 10)
        .map(|_| ())
        .unwrap_err();
    assert_eq!(pre_err, expected_err);
}

/// Setup a subsidy with the `payer` paying for the `user`.
#[track_caller]
fn setup_subsidy(user: User, payer: User, limit: Balance) {
    // Add a subsidy for `user` with `payer` as the paying key.
    assert_ok!(Relayer::approve_subsidy(payer.origin(), user.acc(), limit));

    // No subsidy yet.
    assert_subsidy(user, None);

    // `user` accepts the paying key.
    assert_ok!(Relayer::accept_subsidy(user.origin(), payer.acc()));

    // `user` now has a subsidy of `limit` POLYX.
    assert_subsidy(user, Some((payer, limit)));
}

#[test]
fn basic_relayer_subsidy_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_basic_relayer_subsidy_test);
}
fn do_basic_relayer_subsidy_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);
    let dave = User::new(Sr25519Keyring::Dave);

    // Add authorization for using Alice as the paying key for Bob.
    assert_ok!(Relayer::approve_subsidy(alice.origin(), bob.acc(), 10u128));

    // No subsidy yet.
    assert_subsidy(bob, None);

    // Bob accepts the paying key.
    assert_ok!(Relayer::accept_subsidy(bob.origin(), alice.acc()));

    // Bob now has a subsidy of 10 POLYX.
    assert_subsidy(bob, Some((alice, 10u128)));

    // Bob tries to increase his Polyx limit.  Not allowed
    assert_noop!(
        Relayer::update_polyx_limit(bob.origin(), bob.acc(), 12345u128),
        Error::NotPayingKey
    );

    // Alice updates the Polyx limit for Bob.  Allowed
    assert_ok!(Relayer::update_polyx_limit(
        alice.origin(),
        bob.acc(),
        10_000u128
    ));

    // Bob now has a subsidy of 10,000 POLYX.
    assert_subsidy(bob, Some((alice, 10_000u128)));

    // Dave tries to remove the paying key from Bob's key.  Not allowed.
    assert_noop!(
        Relayer::remove_subsidy(dave.origin(), bob.acc(), alice.acc()),
        Error::NotAuthorized
    );

    // Dave tries to remove the wrong paying key from Bob's key.  Not allowed.
    assert_noop!(
        Relayer::remove_subsidy(dave.origin(), bob.acc(), dave.acc()),
        Error::NotPayingKey
    );

    // Alice tries to remove the paying key from Bob's key.  Allowed.
    assert_ok!(Relayer::remove_subsidy(
        alice.origin(),
        bob.acc(),
        alice.acc(),
    ));

    // Bob no longer has a subsidy.
    assert_subsidy(bob, None);

    // Alice tries to update the poly limit for Bob,
    // but Bob no longer has a subsidy.
    assert_noop!(
        Relayer::update_polyx_limit(alice.origin(), bob.acc(), 42u128),
        Error::NoPayingKey
    );

    // Alice tries to remove the paying key a second time,
    // but Bob no longer has a subsidy.
    assert_noop!(
        Relayer::remove_subsidy(alice.origin(), bob.acc(), alice.acc()),
        Error::NoPayingKey
    );
}

#[test]
fn update_polyx_limit_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_update_polyx_limit_test);
}
fn do_update_polyx_limit_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);

    enum Action {
        Set,
        Add,
        Sub,
    }
    use Action::*;
    let test_update = |action, p: User, x, expected_res: Result<(), Error>, expected_limit| {
        let ext = match action {
            Set => Relayer::update_polyx_limit,
            Add => Relayer::increase_polyx_limit,
            Sub => Relayer::decrease_polyx_limit,
        };
        if let Err(e) = expected_res {
            assert_noop!(ext(p.origin(), bob.acc(), x), e);
        } else {
            assert_ok!(ext(p.origin(), bob.acc(), x));
        }
        assert_subsidy(bob, Some((alice, expected_limit)));
    };

    let mut limit = 10;
    setup_subsidy(bob, alice, limit);

    // Bob tries to update his Polyx limit.  Not allowed
    test_update(Set, bob, 1_000_000, Err(Error::NotPayingKey), limit);
    test_update(Add, bob, 100, Err(Error::NotPayingKey), limit);
    test_update(Sub, bob, 100, Err(Error::NotPayingKey), limit);

    // Alice updates the Polyx limit for Bob.  Allowed
    limit = 10_000;
    test_update(Set, alice, limit, Ok(()), limit);

    // Alice increases the limit.
    limit += 100;
    test_update(Add, alice, 100, Ok(()), limit);

    // Alice decreases the limit.
    limit -= 100;
    test_update(Sub, alice, 100, Ok(()), limit);

    // Test `Overflow` error.
    test_update(Add, alice, u128::MAX, Err(Error::Overflow), limit);
    test_update(Sub, alice, limit + 100, Err(Error::Overflow), limit);
}

#[test]
fn accept_new_subsidy_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_accept_new_subsidy_test);
}
fn do_accept_new_subsidy_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);
    let dave = User::new(Sr25519Keyring::Dave);

    setup_subsidy(bob, alice, 10);

    // Bob now has a subsidy of 10 POLYX from Alice.
    assert_subsidy(bob, Some((alice, 10u128)));

    // Add authorization for using Dave as the paying key for Bob.
    assert_ok!(Relayer::approve_subsidy(dave.origin(), bob.acc(), 200u128));

    // Bob accepts Dave as his new subsidiser replacing Alice as the subsidiser.
    assert_ok!(Relayer::accept_subsidy(bob.origin(), dave.acc()));

    // Bob now has a subsidy of 200 POLYX from Dave.
    assert_subsidy(bob, Some((dave, 200u128)));

    // Alice tries to remove the paying key from Bob's key.  Not allowed.
    assert_noop!(
        Relayer::remove_subsidy(alice.origin(), bob.acc(), dave.acc()),
        Error::NotAuthorized
    );
}

#[test]
fn user_remove_subsidy_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_user_remove_subsidy_test);
}
fn do_user_remove_subsidy_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);

    setup_subsidy(bob, alice, 2000);

    // Bob (user key) tries to remove the paying key from Bob's key.  Allowed.
    assert_ok!(Relayer::remove_subsidy(
        bob.origin(),
        bob.acc(),
        alice.acc(),
    ));

    // Bob no longer has a subsidy.
    assert_subsidy(bob, None);
}

#[test]
fn user_remove_subsidy_transaction_fee_test() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(&do_user_remove_subsidy_transaction_fee_test);
}
fn do_user_remove_subsidy_transaction_fee_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);

    let prev_alice_balance = Balances::free_balance(&alice.acc());
    let prev_bob_balance = Balances::free_balance(&bob.acc());
    let remaining = 100 * POLY;

    setup_subsidy(bob, alice, remaining);

    let diff_balances = || {
        let curr_alice_balance = Balances::free_balance(&alice.acc());
        let curr_bob_balance = Balances::free_balance(&bob.acc());
        (
            prev_alice_balance - curr_alice_balance,
            prev_bob_balance - curr_bob_balance,
        )
    };

    let len = 10;
    //
    // Bob removes alice's key from the subsidy.
    //
    let call = call_relayer_remove_subsidy(bob.acc(), alice.acc());
    let call_info = info_from_weight(5);
    // 0. Calculate fees for registering an asset ticker.
    let transaction_fee = TransactionPayment::compute_fee(len as u32, &call_info, 0);

    // 1. Call `pre_dispatch`.
    let val = ChargeTransactionPayment::from(0)
        .validate(
            RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .unwrap();

    let pre = ChargeTransactionPayment::from(0)
        .prepare(
            val.1,
            &RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
        )
        .unwrap();

    // 2. Execute extrinsic.
    assert_ok!(call.dispatch(bob.origin()));

    // 3. Call `post_dispatch`.
    assert!(ChargeTransactionPayment::post_dispatch(
        pre,
        &call_info,
        &mut post_info_from_weight(5),
        len,
        &Ok(())
    )
    .is_ok());

    // Verify that Bob paid for the transaction fee and not Alice.
    assert_eq!(diff_balances(), (0, transaction_fee));
}

#[test]
fn relayer_transaction_and_protocol_fees_test() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(&do_relayer_transaction_and_protocol_fees_test);
}
fn do_relayer_transaction_and_protocol_fees_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);

    let prev_balance = Balances::free_balance(&alice.acc());
    let remaining = 2_000 * POLY;

    setup_subsidy(bob, alice, remaining);

    let diff_balance = || {
        let curr_balance = Balances::free_balance(&alice.acc());
        let curr_remaining = get_subsidy(bob).map(|s| s.remaining).unwrap();
        (prev_balance - curr_balance, remaining - curr_remaining)
    };

    let len = 10;
    let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
        TransactionError::PalletNotSubsidised as u8,
    ));

    // Pallet System is not subsidised.
    // test `validate`
    let pre_err = ChargeTransactionPayment::from(0)
        .validate(
            RuntimeOrigin::signed(bob.acc()),
            &call_system_remark(42),
            &info_from_weight(5),
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .map(|_| ())
        .unwrap_err();
    assert_eq!(pre_err, expected_err);

    // test `pre_dispatch`
    let pre_err = ChargeTransactionPayment::from(0)
        .prepare(
            Val::Charge {
                tip: 0,
                who: bob.acc(),
                fee_with_tip: 1,
                subsidiser: Some(alice.acc()),
            },
            &RuntimeOrigin::signed(bob.acc()),
            &call_system_remark(42),
            &info_from_weight(5),
            len,
        )
        .map(|_| ())
        .unwrap_err();
    assert_eq!(pre_err, expected_err);

    // No charge to subsidiser balance or subsidy remaining POLYX.
    assert_eq!(diff_balance(), (0, 0));

    //
    // Bob registers an asset ticker with the transaction and protocol fees paid by subsidiser.
    //
    let call = call_asset_register_ticker(b"A");
    let call_info = info_from_weight(100);
    // 0. Calculate fees for registering an asset ticker.
    let transaction_fee = TransactionPayment::compute_fee(len as u32, &call_info, 0);
    assert!(transaction_fee > 0);
    let protocol_fee = ProtocolFee::compute_fee(&[ProtocolOp::AssetRegisterTicker]);
    assert!(protocol_fee > 0);
    let total_fee = transaction_fee + protocol_fee;

    // 1. Call `pre_dispatch`.
    let val = ChargeTransactionPayment::from(0)
        .validate(
            RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .unwrap();

    let pre = ChargeTransactionPayment::from(0)
        .prepare(
            val.1,
            &RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
        )
        .unwrap();

    // 2. Execute extrinsic.
    assert_ok!(call.dispatch(bob.origin()));

    // 3. Call `post_dispatch`.
    assert!(ChargeTransactionPayment::post_dispatch(
        pre,
        &call_info,
        &mut post_info_from_weight(50),
        len,
        &Ok(())
    )
    .is_ok());

    // Verify that the correct fee was deducted from alice's balance
    // and Bob's subsidy's remaining POLYX.
    assert_eq!(diff_balance(), (total_fee, total_fee));
}

#[test]
fn relayer_batched_subsidy_calls_test() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(&do_relayer_batched_subsidy_calls_test);
}
fn do_relayer_batched_subsidy_calls_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);

    let prev_balance = Balances::free_balance(&alice.acc());
    let remaining = 2_000 * POLY;

    setup_subsidy(bob, alice, remaining);

    let diff_balance = || {
        let curr_balance = Balances::free_balance(&alice.acc());
        let curr_remaining = get_subsidy(bob).map(|s| s.remaining).unwrap();
        (prev_balance - curr_balance, remaining - curr_remaining)
    };

    let len = 10;

    // Pallet System is not subsidised.
    let call = call_utility_batch(vec![call_system_remark(42)]);
    assert_invalid_subsidy_call(&bob.acc(), &call);

    // No charge to subsidiser balance or subsidy remaining POLYX.
    assert_eq!(diff_balance(), (0, 0));

    // Large batches are not allowed.
    let call = call_utility_batch(vec![
        call_asset_register_ticker(b"A"),
        call_asset_register_ticker(b"B"),
        call_asset_register_ticker(b"C"),
        call_asset_register_ticker(b"D"),
        call_asset_register_ticker(b"E"),
        call_asset_register_ticker(b"F"),
        call_asset_register_ticker(b"G"),
        // Too many calls.
        call_asset_register_ticker(b"H"),
    ]);
    assert_invalid_subsidy_call(&bob.acc(), &call);

    // No charge to subsidiser balance or subsidy remaining POLYX.
    assert_eq!(diff_balance(), (0, 0));

    // Nested batches are not allowed.
    let call = call_utility_batch(vec![call_utility_batch(vec![call_asset_register_ticker(
        b"A",
    )])]);
    assert_invalid_subsidy_call(&bob.acc(), &call);

    // No charge to subsidiser balance or subsidy remaining POLYX.
    assert_eq!(diff_balance(), (0, 0));

    //
    // Bob registers a some tickers with the transaction and protocol fees paid by subsidiser.
    //
    let call = call_utility_batch_all(vec![
        call_asset_register_ticker(b"A"),
        call_asset_register_ticker(b"B"),
        call_asset_register_ticker(b"C"),
        call_asset_register_ticker(b"D"),
        call_asset_register_ticker(b"E"),
    ]);
    let call_info = info_from_weight(100);
    // 0. Calculate fees for registering an asset ticker.
    let transaction_fee = TransactionPayment::compute_fee(len as u32, &call_info, 0);
    assert!(transaction_fee > 0);
    let protocol_fee = ProtocolFee::compute_fee(&[
        ProtocolOp::AssetRegisterTicker,
        ProtocolOp::AssetRegisterTicker,
        ProtocolOp::AssetRegisterTicker,
        ProtocolOp::AssetRegisterTicker,
        ProtocolOp::AssetRegisterTicker,
    ]);
    assert!(protocol_fee > 0);
    let total_fee = transaction_fee + protocol_fee;

    // 1. Call `pre_dispatch`.
    let val = ChargeTransactionPayment::from(0)
        .validate(
            RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .unwrap();

    let pre = ChargeTransactionPayment::from(0)
        .prepare(
            val.1,
            &RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
        )
        .unwrap();

    // 2. Execute extrinsic.
    assert_ok!(call.dispatch(bob.origin()));

    // 3. Call `post_dispatch`.
    assert!(ChargeTransactionPayment::post_dispatch(
        pre,
        &call_info,
        &mut post_info_from_weight(50),
        len,
        &Ok(())
    )
    .is_ok());

    // Verify that the correct fee was deducted from alice's balance
    // and Bob's subsidy's remaining POLYX.
    assert_eq!(diff_balance(), (total_fee, total_fee));
}

#[test]
fn relayer_accept_cdd_and_fees_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_relayer_accept_cdd_and_fees_test);
}
fn do_relayer_accept_cdd_and_fees_test() {
    let alice = User::new(Sr25519Keyring::Alice);
    let bob = User::new(Sr25519Keyring::Bob);

    // Alice creates a subsidy for Bob.
    assert_ok!(Relayer::approve_subsidy(alice.origin(), bob.acc(), 0u128));

    // Check that Bob can accept the subsidy with Alice paying for the transaction.
    assert_eq!(
        TxFeeHandler::call_payment_info(
            &DevRuntimeCall::Relayer(pallet_relayer::Call::accept_subsidy {
                paying_key: alice.acc()
            }),
            bob.acc()
        ),
        Ok(CallPaymentInfo::new(alice.acc(), None, None))
    );
}

#[test]
fn subsidy_reserve_and_settle_test() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(&do_subsidy_reserve_and_settle_test);
}

fn do_subsidy_reserve_and_settle_test() {
    System::set_block_number(1);
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);

    let len = 10;
    let call_info = info_from_weight(100);
    let tx_fee = TransactionPayment::compute_fee(len as u32, &call_info, 0);

    // --- Part 1: tight budget, protocol fee fails, imbalance settled ---
    let call = call_asset_register_ticker(b"A");
    setup_subsidy(bob, alice, tx_fee);

    let val = ChargeTransactionPayment::from(0)
        .validate(
            RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .unwrap();

    let pre = ChargeTransactionPayment::from(0)
        .prepare(
            val.1,
            &RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
        )
        .unwrap();

    assert_eq!(get_subsidy(bob).unwrap().remaining, 0);

    let dispatch_result = call.dispatch(bob.origin());
    assert!(dispatch_result.is_err());

    let mut actual_post_info = post_info_from_weight(100);
    assert_ok!(ChargeTransactionPayment::post_dispatch(
        pre,
        &call_info,
        &mut actual_post_info,
        len,
        &dispatch_result.map(|_| ()).map_err(|e| e.error),
    ));

    // --- Part 2: subsidy removed mid-dispatch, event still emitted ---
    let call = call_asset_register_ticker(b"B");
    // Re-create subsidy with enough budget.
    Subsidies::remove(&bob.acc());
    setup_subsidy(bob, alice, tx_fee * 2);

    let val = ChargeTransactionPayment::from(0)
        .validate(
            RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .unwrap();

    let pre = ChargeTransactionPayment::from(0)
        .prepare(
            val.1,
            &RuntimeOrigin::signed(bob.acc()),
            &call,
            &call_info,
            len,
        )
        .unwrap();

    // Simulate subsidy removal during dispatch (e.g. last call in a batch).
    Subsidies::remove(&bob.acc());
    assert!(get_subsidy(bob).is_none());

    let mut actual_post_info = post_info_from_weight(100);
    assert_ok!(ChargeTransactionPayment::post_dispatch(
        pre,
        &call_info,
        &mut actual_post_info,
        len,
        &Ok(()),
    ));

    assert!(System::events().iter().any(|e| matches!(
        &e.event,
        EventTest::Relayer(pallet_relayer::Event::SubsidyDebited {
            paying_key,
            ..
        }) if *paying_key == alice.acc()
    )));
}

#[test]
fn relay_happy_case() {
    ExtBuilder::default()
        .build()
        .execute_with(_relay_happy_case);
}

fn _relay_happy_case() {
    let alice = Sr25519Keyring::Alice.to_account_id();
    let _ = register_keyring_account_with_balance(Sr25519Keyring::Alice, 1_000).unwrap();

    let bob = Sr25519Keyring::Bob.to_account_id();
    let _ = register_keyring_account_with_balance(Sr25519Keyring::Bob, 1_000).unwrap();

    let charlie = Sr25519Keyring::Charlie.to_account_id();
    let _ = register_keyring_account_with_balance(Sr25519Keyring::Charlie, 1_000).unwrap();

    // 41 Extra for registering a DID
    assert_balance(bob.clone(), 1041, 0);
    assert_balance(charlie.clone(), 1041, 0);

    let origin = RuntimeOrigin::signed(alice);
    let nonce = RelayTxNonces::<TestStorage>::get(bob.clone());
    let call = Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
        dest: charlie.clone().into(),
        value: 50,
    }));
    let expires_at = 100u64;
    let scoped_call =
        ChainScopedMessage::<TestStorage, _>::new(nonce, RELAY_TX_LABEL, expires_at, &call)
            .expect("Shouldn't be expired");
    let signature = scoped_call.sign(&Sr25519Keyring::Bob).expect("Should sign");

    assert_ok!(Relayer::relay_tx(
        origin,
        bob.clone(),
        signature.into(),
        call,
        expires_at
    ));

    assert_balance(bob, 991, 0);
    assert_balance(charlie, 1_091, 0);
}

#[test]
fn relay_unhappy_cases() {
    ExtBuilder::default()
        .build()
        .execute_with(_relay_unhappy_cases);
}

fn _relay_unhappy_cases() {
    let alice = Sr25519Keyring::Alice.to_account_id();
    let _ = register_keyring_account_with_balance(Sr25519Keyring::Alice, 1_000).unwrap();

    let bob = Sr25519Keyring::Bob.to_account_id();

    let charlie = Sr25519Keyring::Charlie.to_account_id();

    let origin = RuntimeOrigin::signed(alice);
    let call = Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
        dest: charlie.clone().into(),
        value: 59,
    }));
    let expires_at = 100u64;
    let signature = MultiSignature::Sr25519(Default::default());
    assert_noop!(
        Relayer::relay_tx(
            origin.clone(),
            bob.clone(),
            signature.clone(),
            call.clone(),
            expires_at,
        ),
        Error::InvalidSignature
    );

    let _ = register_keyring_account_with_balance(Sr25519Keyring::Bob, 1_000).unwrap();

    set_timestamp(expires_at);

    assert_noop!(
        Relayer::relay_tx(origin.clone(), bob, signature, call, expires_at),
        Error::ExpiredRelayTx
    );
}
