use super::{
    ext_builder::MockProtocolBaseFees,
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
    traits::SignedExtension,
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

    // `user` accept's the paying key.
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

    // Bob accept's the paying key.
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

    // Add authorization for using Dave as the paying key for Bob.
    assert_ok!(Relayer::set_paying_key(dave.origin(), bob.acc(), 0u128));

    // Bob tries to accept the new paying key, but he already has a paying key.
    TestStorage::set_current_identity(&bob.did);
    let auth_id = get_last_auth_id(&Signatory::Account(bob.acc()));
    assert_noop!(
        Relayer::accept_paying_key(bob.origin(), auth_id),
        Error::AlreadyHasPayingKey
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
    // but Bob no longer has a subsiy.
    assert_noop!(
        Relayer::update_polyx_limit(alice.origin(), bob.acc(), 42u128),
        Error::NoPayingKey
    );

    // Alice tries to remove the paying key a second time,
    // but Bob no longer has a subsiy.
    assert_noop!(
        Relayer::remove_paying_key(alice.origin(), bob.acc(), alice.acc()),
        Error::NoPayingKey
    );
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
fn relayer_transaction_fees_test() {
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::AssetRegisterTicker, 500)]);
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .set_protocol_base_fees(protocol_fee)
        .build()
        .execute_with(&do_relayer_transaction_fees_test);
}
fn do_relayer_transaction_fees_test() {
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

    let pre = ChargeTransactionPayment::from(0)
        .pre_dispatch(
            &bob.acc(),
            &call_asset_register_ticker(b"A"),
            &info_from_weight(100),
            len,
        )
        .unwrap();

    assert!(ChargeTransactionPayment::post_dispatch(
        pre,
        &info_from_weight(100),
        &post_info_from_weight(50),
        len,
        &Ok(())
    )
    .is_ok());
    assert_eq!(diff_balance(), (30999, 30999));
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
