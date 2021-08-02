use super::{
    storage::{get_last_auth_id, make_account_without_cdd, Call, TestStorage, User},
    ExtBuilder,
};
use frame_support::{
    assert_noop, assert_ok,
    weights::{DispatchInfo, Pays, PostDispatchInfo, Weight},
    StorageMap,
};
use pallet_relayer::Subsidy;
use polymesh_common_utilities::{
    constants::currency::POLY, protocol_fee::ProtocolOp,
    traits::transaction_payment::CddAndFeeDetails,
};
use polymesh_primitives::{AccountId, Balance, Signatory, Ticker, TransactionError};
use polymesh_runtime_develop::{fee_details::CddHandler, runtime::Call as DevCall};
use sp_runtime::{
    traits::{Dispatchable, SignedExtension},
    transaction_validity::{InvalidTransaction, TransactionValidityError},
    MultiAddress,
};
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Relayer = pallet_relayer::Module<TestStorage>;
type Subsidies = pallet_relayer::Subsidies<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type AccountKeyRefCount = pallet_identity::AccountKeyRefCount<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;
type ProtocolFee = pallet_protocol_fee::Module<TestStorage>;
type TransactionPayment = pallet_transaction_payment::Module<TestStorage>;
type ChargeTransactionPayment = pallet_transaction_payment::ChargeTransactionPayment<TestStorage>;
type Error = pallet_relayer::Error<TestStorage>;

// Relayer Test Helper functions
// =======================================

fn call_balance_transfer(val: Balance) -> <TestStorage as frame_system::Config>::Call {
    Call::Balances(pallet_balances::Call::transfer(
        MultiAddress::Id(AccountKeyring::Alice.to_account_id()),
        val,
    ))
}

fn call_asset_register_ticker(name: &[u8]) -> <TestStorage as frame_system::Config>::Call {
    let ticker = Ticker::try_from(name).unwrap();
    Call::Asset(pallet_asset::Call::register_ticker(ticker))
}

fn call_relayer_remove_paying_key(
    user_key: AccountId,
    paying_key: AccountId,
) -> <TestStorage as frame_system::Config>::Call {
    Call::Relayer(pallet_relayer::Call::remove_paying_key(
        user_key, paying_key,
    ))
}

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
    DispatchInfo {
        weight: w,
        ..Default::default()
    }
}

fn post_info_from_weight(w: Weight) -> PostDispatchInfo {
    PostDispatchInfo {
        actual_weight: Some(w),
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

/// Setup a subsidy with the `payer` paying for the `user`.
#[track_caller]
fn setup_subsidy(user: User, payer: User, limit: Balance) {
    // Add authorization for using `payer` as the paying key for `user`.
    assert_ok!(Relayer::set_paying_key(payer.origin(), user.acc(), limit));

    // No subsidy yet.
    assert_subsidy(user, None);

    // `user` accepts the paying key.
    TestStorage::set_current_identity(&user.did);
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
    let bob = User::new(AccountKeyring::Bob);
    let alice = User::new(AccountKeyring::Alice);
    let dave = User::new(AccountKeyring::Dave);

    // Add authorization for using Alice as the paying key for Bob.
    assert_ok!(Relayer::set_paying_key(alice.origin(), bob.acc(), 10u128));

    // The keys are not used yet.
    assert_key_usage(alice, 0);
    assert_key_usage(dave, 0);
    assert_key_usage(bob, 0);

    // No subsidy yet.
    assert_subsidy(bob, None);

    // Bob accepts the paying key.
    TestStorage::set_current_identity(&bob.did);
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
    TestStorage::set_current_identity(&alice.did);
    assert_ok!(Relayer::update_polyx_limit(
        alice.origin(),
        bob.acc(),
        10_000u128
    ));

    // Bob now has a subsidy of 10,000 POLYX.
    assert_subsidy(bob, Some((alice, 10_000u128)));

    // Dave tries to remove the paying key from Bob's key.  Not allowed.
    TestStorage::set_current_identity(&dave.did);
    assert_noop!(
        Relayer::remove_paying_key(dave.origin(), bob.acc(), alice.acc()),
        Error::NotAuthorizedForUserKey
    );

    // Dave tries to remove the wrong paying key from Bob's key.  Not allowed.
    TestStorage::set_current_identity(&dave.did);
    assert_noop!(
        Relayer::remove_paying_key(dave.origin(), bob.acc(), dave.acc()),
        Error::NotPayingKey
    );

    // Alice tries to remove the paying key from Bob's key.  Allowed.
    TestStorage::set_current_identity(&alice.did);
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
    let bob = User::new(AccountKeyring::Bob);
    let alice = User::new(AccountKeyring::Alice);

    let assert_limit = |limit| {
        assert_subsidy(bob, Some((alice, limit)));
    };

    let mut limit = 10u128;
    setup_subsidy(bob, alice, limit);

    // Bob tries to update his Polyx limit.  Not allowed
    assert_noop!(
        Relayer::update_polyx_limit(bob.origin(), bob.acc(), 1_000_000u128),
        Error::NotPayingKey
    );
    assert_limit(limit);
    assert_noop!(
        Relayer::increase_polyx_limit(bob.origin(), bob.acc(), 100u128),
        Error::NotPayingKey
    );
    assert_limit(limit);
    assert_noop!(
        Relayer::decrease_polyx_limit(bob.origin(), bob.acc(), 100u128),
        Error::NotPayingKey
    );
    assert_limit(limit);

    // Alice updates the Polyx limit for Bob.  Allowed
    TestStorage::set_current_identity(&alice.did);
    limit = 10_000u128;
    assert_ok!(Relayer::update_polyx_limit(
        alice.origin(),
        bob.acc(),
        limit,
    ));
    assert_limit(limit);

    // Alice increases the limit.
    assert_ok!(Relayer::increase_polyx_limit(
        alice.origin(),
        bob.acc(),
        100u128,
    ));
    limit += 100u128;
    assert_limit(limit);

    // Alice decreases the limit.
    assert_ok!(Relayer::decrease_polyx_limit(
        alice.origin(),
        bob.acc(),
        100u128,
    ));
    limit -= 100u128;
    assert_limit(limit);

    // Test `Overflow` error.
    assert_noop!(
        Relayer::increase_polyx_limit(alice.origin(), bob.acc(), u128::MAX),
        Error::Overflow
    );
    assert_limit(limit);
    assert_noop!(
        Relayer::decrease_polyx_limit(alice.origin(), bob.acc(), limit + 100u128),
        Error::Overflow
    );
    assert_limit(limit);
}

#[test]
fn accept_new_paying_key_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_accept_new_paying_key_test);
}
fn do_accept_new_paying_key_test() {
    let bob = User::new(AccountKeyring::Bob);
    let alice = User::new(AccountKeyring::Alice);
    let dave = User::new(AccountKeyring::Dave);

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
    TestStorage::set_current_identity(&dave.did);
    assert_ok!(Relayer::set_paying_key(dave.origin(), bob.acc(), 200u128));

    // Bob accepts Dave as his new subsidiser replacing Alice as the subsidiser.
    TestStorage::set_current_identity(&bob.did);
    let auth_id = get_last_auth_id(&Signatory::Account(bob.acc()));
    assert_ok!(Relayer::accept_paying_key(bob.origin(), auth_id));

    assert_usages(1, 0, 1);
    // Bob now has a subsidy of 200 POLYX from Dave.
    assert_subsidy(bob, Some((dave, 200u128)));

    // Alice tries to remove the paying key from Bob's key.  Not allowed.
    TestStorage::set_current_identity(&alice.did);
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
    let bob = User::new(AccountKeyring::Bob);
    let alice = User::new(AccountKeyring::Alice);

    setup_subsidy(bob, alice, 2000);

    // Bob (user key) tries to remove the paying key from Bob's key.  Allowed.
    TestStorage::set_current_identity(&bob.did);
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
fn relayer_user_key_missing_cdd_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_relayer_user_key_missing_cdd_test);
}
fn do_relayer_user_key_missing_cdd_test() {
    let alice = User::new(AccountKeyring::Alice);
    let bob_acc = AccountKeyring::Bob.to_account_id();
    let (bob_sign, bob_did) = make_account_without_cdd(bob_acc.clone()).unwrap();

    // Add authorization for using Alice as the paying key for Bob.
    assert_ok!(Relayer::set_paying_key(
        alice.origin(),
        bob_acc.clone(),
        10u128
    ));

    // Bob tries to accept the paying key, without having a CDD.
    TestStorage::set_current_identity(&bob_did);
    let auth_id = get_last_auth_id(&Signatory::Account(bob_acc.clone()));
    assert_noop!(
        Relayer::accept_paying_key(bob_sign, auth_id),
        Error::UserKeyCddMissing
    );
}

#[test]
fn relayer_paying_key_missing_cdd_test() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(&do_relayer_paying_key_missing_cdd_test);
}
fn do_relayer_paying_key_missing_cdd_test() {
    let alice = User::new(AccountKeyring::Alice);
    let bob_acc = AccountKeyring::Bob.to_account_id();
    let (bob_sign, _) = make_account_without_cdd(bob_acc.clone()).unwrap();

    // Add authorization for using Bob as the paying key for Alice.
    assert_ok!(Relayer::set_paying_key(bob_sign, alice.acc(), 10u128));

    // Alice tries to accept the paying key, but the paying key
    // is without a CDD.
    TestStorage::set_current_identity(&alice.did);
    let auth_id = get_last_auth_id(&Signatory::Account(alice.acc()));
    assert_noop!(
        Relayer::accept_paying_key(alice.origin(), auth_id),
        Error::PayingKeyCddMissing
    );
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
    let bob = User::new(AccountKeyring::Bob);
    let alice = User::new(AccountKeyring::Alice);

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
    let pre = ChargeTransactionPayment::from(0)
        .pre_dispatch(&bob.acc(), &call, &call_info, len)
        .unwrap();

    // 2. Execute extrinsic.
    assert_ok!(call.dispatch(bob.origin()));

    // 3. Call `post_dispatch`.
    assert!(ChargeTransactionPayment::post_dispatch(
        pre,
        &call_info,
        &post_info_from_weight(5),
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
    let bob = User::new(AccountKeyring::Bob);
    let alice = User::new(AccountKeyring::Alice);

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

    // Pallet Balance is not subsidised.
    // test `validate`
    let pre_err = ChargeTransactionPayment::from(0)
        .validate(
            &bob.acc(),
            &call_balance_transfer(42),
            &info_from_weight(5),
            len,
        )
        .map(|_| ())
        .unwrap_err();
    assert_eq!(pre_err, expected_err);

    // test `pre_dispatch`
    let pre_err = ChargeTransactionPayment::from(0)
        .pre_dispatch(
            &bob.acc(),
            &call_balance_transfer(42),
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
    let pre = ChargeTransactionPayment::from(0)
        .pre_dispatch(&bob.acc(), &call, &call_info, len)
        .unwrap();

    // 2. Execute extrinsic.
    assert_ok!(call.dispatch(bob.origin()));

    // 3. Call `post_dispatch`.
    assert!(ChargeTransactionPayment::post_dispatch(
        pre,
        &call_info,
        &post_info_from_weight(50),
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
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);
    let bob_sign = Signatory::Account(bob.acc());

    // Alice creates authoration to subsidise for Bob.
    assert_ok!(Relayer::set_paying_key(alice.origin(), bob.acc(), 0u128));
    let bob_auth_id = get_last_auth_id(&bob_sign);

    // Check that Bob can accept the subsidy with Alice paying for the transaction.
    assert_eq!(
        CddHandler::get_valid_payer(
            &DevCall::Relayer(pallet_relayer::Call::accept_paying_key(bob_auth_id)),
            &bob.acc()
        ),
        Ok(Some(alice.acc()))
    );
}
