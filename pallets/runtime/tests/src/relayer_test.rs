use frame_support::dispatch::{DispatchInfo, Pays, PostDispatchInfo};
use frame_support::weights::Weight;
use frame_support::{assert_noop, assert_ok};
use frame_system;
use sp_keyring::Sr25519Keyring;
use sp_runtime::traits::{Dispatchable, TransactionExtension, TxBaseImplication};
use sp_runtime::transaction_validity::TransactionSource;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};
use sp_runtime::MultiAddress;

use pallet_relayer::Subsidy;
use polymesh_primitives::constants::currency::POLY;
use polymesh_primitives::protocol_fee::ProtocolOp;
use polymesh_primitives::traits::CddAndFeeDetails;
use polymesh_primitives::{AccountId, Balance, Signatory, Ticker, TransactionError};
use polymesh_runtime_develop::runtime::{CddHandler, RuntimeCall as DevRuntimeCall};
use polymesh_transaction_payment::Val;

use super::storage::{get_last_auth_id, make_account_without_cdd, RuntimeCall, TestStorage, User};
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

fn call_relayer_remove_paying_key(user_key: AccountId, paying_key: AccountId) -> RuntimeCall {
    RuntimeCall::Relayer(pallet_relayer::Call::remove_paying_key {
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
    // Add authorization for using `payer` as the paying key for `user`.
    assert_ok!(Relayer::set_paying_key(payer.origin(), user.acc(), limit));

    // No subsidy yet.
    assert_subsidy(user, None);

    // `user` accepts the paying key.
    let auth_id = get_last_auth_id(&Signatory::Account(user.acc()));
    assert_ok!(Relayer::accept_paying_key(user.origin(), auth_id));

    // `user` now has a subsidy of `limit` POLYX.
    assert_subsidy(user, Some((payer, limit)));
}

#[test]
fn basic_relayer_paying_key_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_basic_relayer_paying_key_test);
}
fn do_basic_relayer_paying_key_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);
    let dave = User::new(Sr25519Keyring::Dave);

    // Add authorization for using Alice as the paying key for Bob.
    assert_ok!(Relayer::set_paying_key(alice.origin(), bob.acc(), 10u128));

    // The keys are not used yet.
    assert_key_usage(alice, 0);
    assert_key_usage(dave, 0);
    assert_key_usage(bob, 0);

    // No subsidy yet.
    assert_subsidy(bob, None);

    // Bob accepts the paying key.
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
    assert_ok!(Relayer::update_polyx_limit(
        alice.origin(),
        bob.acc(),
        10_000u128
    ));

    // Bob now has a subsidy of 10,000 POLYX.
    assert_subsidy(bob, Some((alice, 10_000u128)));

    // Dave tries to remove the paying key from Bob's key.  Not allowed.
    assert_noop!(
        Relayer::remove_paying_key(dave.origin(), bob.acc(), alice.acc()),
        Error::NotAuthorizedForUserKey
    );

    // Dave tries to remove the wrong paying key from Bob's key.  Not allowed.
    assert_noop!(
        Relayer::remove_paying_key(dave.origin(), bob.acc(), dave.acc()),
        Error::NotPayingKey
    );

    // Alice tries to remove the paying key from Bob's key.  Allowed.
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
    // but Bob no longer has a subsidy.
    assert_noop!(
        Relayer::update_polyx_limit(alice.origin(), bob.acc(), 42u128),
        Error::NoPayingKey
    );

    // Alice tries to remove the paying key a second time,
    // but Bob no longer has a subsidy.
    assert_noop!(
        Relayer::remove_paying_key(alice.origin(), bob.acc(), alice.acc()),
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
fn accept_new_paying_key_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_accept_new_paying_key_test);
}
fn do_accept_new_paying_key_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);
    let dave = User::new(Sr25519Keyring::Dave);

    let assert_usages = |bob_cnt, alice_cnt, dave_cnt| {
        assert_key_usage(bob, bob_cnt);
        assert_key_usage(alice, alice_cnt);
        assert_key_usage(dave, dave_cnt);
    };

    setup_subsidy(bob, alice, 10);
    assert_usages(1, 1, 0);

    // Bob now has a subsidy of 10 POLYX from Alice.
    assert_subsidy(bob, Some((alice, 10u128)));

    // Add authorization for using Dave as the paying key for Bob.
    assert_ok!(Relayer::set_paying_key(dave.origin(), bob.acc(), 200u128));

    // Bob accepts Dave as his new subsidiser replacing Alice as the subsidiser.
    let auth_id = get_last_auth_id(&Signatory::Account(bob.acc()));
    assert_ok!(Relayer::accept_paying_key(bob.origin(), auth_id));

    assert_usages(1, 0, 1);
    // Bob now has a subsidy of 200 POLYX from Dave.
    assert_subsidy(bob, Some((dave, 200u128)));

    // Alice tries to remove the paying key from Bob's key.  Not allowed.
    assert_noop!(
        Relayer::remove_paying_key(alice.origin(), bob.acc(), dave.acc()),
        Error::NotAuthorizedForUserKey
    );
}

#[test]
fn user_remove_paying_key_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_user_remove_paying_key_test);
}
fn do_user_remove_paying_key_test() {
    let bob = User::new(Sr25519Keyring::Bob);
    let alice = User::new(Sr25519Keyring::Alice);

    setup_subsidy(bob, alice, 2000);

    // Bob (user key) tries to remove the paying key from Bob's key.  Allowed.
    assert_ok!(Relayer::remove_paying_key(
        bob.origin(),
        bob.acc(),
        alice.acc(),
    ));

    // Bob no longer has a subsidy.
    assert_subsidy(bob, None);

    // Check alice's key is not used any more.
    assert_key_usage(alice, 0);
    // Check bob's key is not used any more.
    assert_key_usage(bob, 0);
}

#[test]
fn relayer_user_key_without_cdd_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_relayer_user_key_without_cdd_test);
}
fn do_relayer_user_key_without_cdd_test() {
    let alice = User::new(Sr25519Keyring::Alice);
    let bob_acc = Sr25519Keyring::Bob.to_account_id();
    let (bob_sign, _) = make_account_without_cdd(bob_acc.clone()).unwrap();

    // Add authorization for using Alice as the paying key for Bob.
    assert_ok!(Relayer::set_paying_key(
        alice.origin(),
        bob_acc.clone(),
        10u128
    ));

    // Bob tries to accept the paying key, without having a DID.
    let auth_id = get_last_auth_id(&Signatory::Account(bob_acc.clone()));
    assert_ok!(Relayer::accept_paying_key(bob_sign, auth_id),);
}

#[test]
fn relayer_paying_key_without_cdd_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_relayer_paying_key_without_cdd_test);
}
fn do_relayer_paying_key_without_cdd_test() {
    let alice = User::new(Sr25519Keyring::Alice);
    let bob_acc = Sr25519Keyring::Bob.to_account_id();
    let (bob_sign, _) = make_account_without_cdd(bob_acc.clone()).unwrap();

    // Add authorization for using Bob as the paying key for Alice.
    assert_ok!(Relayer::set_paying_key(bob_sign, alice.acc(), 10u128));

    // Alice tries to accept the paying key, but the paying key
    // is without a DID.
    let auth_id = get_last_auth_id(&Signatory::Account(alice.acc()));
    assert_ok!(Relayer::accept_paying_key(alice.origin(), auth_id),);
}

#[test]
fn user_remove_paying_key_transaction_fee_test() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(&do_user_remove_paying_key_transaction_fee_test);
}
fn do_user_remove_paying_key_transaction_fee_test() {
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
    let call = call_relayer_remove_paying_key(bob.acc(), alice.acc());
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
    let bob_sign = Signatory::Account(bob.acc());

    // Alice creates authoration to subsidise for Bob.
    assert_ok!(Relayer::set_paying_key(alice.origin(), bob.acc(), 0u128));
    let auth_id = get_last_auth_id(&bob_sign);

    // Check that Bob can accept the subsidy with Alice paying for the transaction.
    assert_eq!(
        CddHandler::get_valid_payer(
            &DevRuntimeCall::Relayer(pallet_relayer::Call::accept_paying_key { auth_id }),
            bob.acc()
        ),
        Ok(Some(alice.acc()))
    );
}
