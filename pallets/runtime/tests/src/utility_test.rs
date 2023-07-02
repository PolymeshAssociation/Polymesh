use super::storage::example::Call as ExampleCall;
use super::{
    assert_event_doesnt_exist, assert_event_exists, assert_last_event,
    committee_test::set_members,
    pips_test::{assert_balance, assert_state, committee_proposal, community_proposal},
    storage::{
        add_secondary_key, get_secondary_keys, next_block, register_keyring_account_with_balance,
        EventTest, Identity, Portfolio, RuntimeCall, RuntimeOrigin, System, TestBaseCallFilter,
        TestStorage, User, Utility,
    },
    ExtBuilder,
};
use codec::Encode;
use frame_support::{
    assert_err_ignore_postinfo, assert_noop, assert_ok, assert_storage_noop,
    dispatch::{
        extract_actual_weight, DispatchError, DispatchErrorWithPostInfo, Dispatchable,
        GetDispatchInfo, Pays, PostDispatchInfo, Weight,
    },
    error::BadOrigin,
    storage,
    traits::Contains,
};
use frame_system::{Call as SystemCall, EventRecord};
use pallet_timestamp::Call as TimestampCall;

use pallet_balances::Call as BalancesCall;
use pallet_pips::{ProposalState, SnapshotResult};
use pallet_portfolio::Call as PortfolioCall;
use pallet_utility::{
    self as utility, Call as UtilityCall, Config as UtilityConfig, Event, UniqueCall, WeightInfo,
};
use polymesh_common_utilities::traits::transaction_payment::CddAndFeeDetails;
use polymesh_primitives::{
    AccountId, Balance, PalletPermissions, Permissions, PortfolioName, PortfolioNumber,
    SubsetRestriction,
};
use sp_core::sr25519::Signature;
use test_client::AccountKeyring;

type Error = utility::Error<TestStorage>;

type Balances = pallet_balances::Module<TestStorage>;
type Pips = pallet_pips::Module<TestStorage>;
type Committee = pallet_committee::Module<TestStorage, pallet_committee::Instance1>;

fn consensus_call(call: RuntimeCall, signers: &[User]) {
    let call = Box::new(call);
    for signer in signers {
        assert_ok!(Committee::vote_or_propose(
            signer.origin(),
            true,
            call.clone()
        ));
    }
}

fn transfer(to: AccountId, amount: Balance) -> RuntimeCall {
    RuntimeCall::Balances(BalancesCall::transfer {
        dest: to.into(),
        value: amount,
    })
}

const ERROR: DispatchError = DispatchError::Module(sp_runtime::ModuleError {
    index: 5,
    error: [2, 0, 0, 0],
    message: None,
});

#[track_caller]
fn assert_event(event: Event<TestStorage>) {
    assert_eq!(
        System::events().pop().unwrap().event,
        EventTest::Utility(event)
    )
}

fn batch_test(test: impl FnOnce(AccountId, AccountId)) {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let alice = AccountKeyring::Alice.to_account_id();
        TestStorage::set_payer_context(Some(alice.clone()));
        let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

        let bob = AccountKeyring::Bob.to_account_id();
        TestStorage::set_payer_context(Some(bob.clone()));
        let _ = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

        assert_balance(alice.clone(), 1000, 0);
        assert_balance(bob.clone(), 1000, 0);

        test(alice, bob)
    });
}

#[test]
fn batch_with_signed_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob.clone(), 400), transfer(bob.clone(), 400)];
        assert_ok!(Utility::batch(RuntimeOrigin::signed(alice.clone()), calls));
        assert_balance(alice, 200, 0);
        assert_balance(bob, 1000 + 400 + 400, 0);
        assert_event(Event::BatchCompleted);
    });
}

#[test]
fn batch_early_exit_works() {
    batch_test(|alice, bob| {
        let calls = vec![
            transfer(bob.clone(), 400),
            transfer(bob.clone(), 900),
            transfer(bob.clone(), 400),
        ];
        assert_ok!(Utility::batch(RuntimeOrigin::signed(alice.clone()), calls));
        assert_balance(alice, 600, 0);
        assert_balance(bob, 1000 + 400, 0);
        assert_event(Event::BatchInterrupted {
            index: 1,
            error: ERROR,
        });
    })
}

#[test]
fn batch_optimistic_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob.clone(), 401), transfer(bob.clone(), 402)];
        assert_ok!(Utility::force_batch(
            RuntimeOrigin::signed(alice.clone()),
            calls
        ));
        assert_event(Event::BatchCompleted);
        assert_balance(alice, 1000 - 401 - 402, 0);
        assert_balance(bob, 1000 + 401 + 402, 0);
    });
}

#[test]
fn batch_optimistic_failures_listed() {
    batch_test(|alice, bob| {
        assert_ok!(Utility::force_batch(
            RuntimeOrigin::signed(alice.clone()),
            vec![
                transfer(bob.clone(), 401), // YAY.
                transfer(bob.clone(), 900), // NAY.
                transfer(bob.clone(), 800), // NAY.
                transfer(bob.clone(), 402), // YAY.
                transfer(bob.clone(), 403), // NAY.
            ]
        ));
        let mut events = System::events();
        assert_eq!(
            events.pop().unwrap().event,
            EventTest::Utility(Event::BatchCompletedWithErrors)
        );
        assert_eq!(
            events.pop().unwrap().event,
            EventTest::Utility(Event::ItemFailed { error: ERROR })
        );
        assert_eq!(
            events.pop().unwrap().event,
            EventTest::Utility(Event::ItemCompleted)
        );
        // skip Balances::Transfer event.
        events.pop().unwrap();
        assert_eq!(
            events.pop().unwrap().event,
            EventTest::Utility(Event::ItemFailed { error: ERROR })
        );
        assert_eq!(
            events.pop().unwrap().event,
            EventTest::Utility(Event::ItemFailed { error: ERROR })
        );
        assert_eq!(
            events.pop().unwrap().event,
            EventTest::Utility(Event::ItemCompleted)
        );
        assert_balance(alice, 1000 - 401 - 402, 0);
        assert_balance(bob, 1000 + 401 + 402, 0);
    });
}

#[test]
fn batch_atomic_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob.clone(), 401), transfer(bob.clone(), 402)];
        assert_ok!(Utility::batch_all(
            RuntimeOrigin::signed(alice.clone()),
            calls
        ));
        assert_event(Event::BatchCompleted);
        assert_balance(alice, 1000 - 401 - 402, 0);
        assert_balance(bob, 1000 + 401 + 402, 0);
    });
}

#[test]
fn batch_atomic_early_exit_works() {
    batch_test(|alice, bob| {
        let trans = |x| transfer(bob.clone(), x);
        let calls = vec![trans(400), trans(900), trans(400)];
        assert_storage_noop!(assert_err_ignore_postinfo!(
            Utility::batch_all(RuntimeOrigin::signed(alice.clone()), calls),
            pallet_balances::Error::<TestStorage>::InsufficientBalance
        ));
        assert_balance(alice, 1000, 0);
        assert_balance(bob, 1000, 0);
    })
}

#[test]
fn relay_happy_case() {
    ExtBuilder::default()
        .build()
        .execute_with(_relay_happy_case);
}

fn _relay_happy_case() {
    let alice = AccountKeyring::Alice.to_account_id();
    let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

    let bob = AccountKeyring::Bob.to_account_id();
    let _ = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

    let charlie = AccountKeyring::Charlie.to_account_id();
    let _ = register_keyring_account_with_balance(AccountKeyring::Charlie, 1_000).unwrap();

    // 41 Extra for registering a DID
    assert_balance(bob.clone(), 1041, 0);
    assert_balance(charlie.clone(), 1041, 0);

    let origin = RuntimeOrigin::signed(alice);
    let transaction = UniqueCall::new(
        Utility::nonce(bob.clone()),
        RuntimeCall::Balances(BalancesCall::transfer {
            dest: charlie.clone().into(),
            value: 50,
        }),
    );

    assert_ok!(Utility::relay_tx(
        origin,
        bob.clone(),
        AccountKeyring::Bob.sign(&transaction.encode()).into(),
        transaction
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
    let alice = AccountKeyring::Alice.to_account_id();
    let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();

    let bob = AccountKeyring::Bob.to_account_id();

    let charlie = AccountKeyring::Charlie.to_account_id();

    let origin = RuntimeOrigin::signed(alice);
    let transaction = UniqueCall::new(
        Utility::nonce(bob.clone()),
        RuntimeCall::Balances(BalancesCall::transfer {
            dest: charlie.clone().into(),
            value: 59,
        }),
    );

    assert_noop!(
        Utility::relay_tx(
            origin.clone(),
            bob.clone(),
            Signature([0; 64]).into(),
            transaction.clone()
        ),
        Error::InvalidSignature
    );

    assert_noop!(
        Utility::relay_tx(
            origin.clone(),
            bob.clone(),
            AccountKeyring::Bob.sign(&transaction.encode()).into(),
            transaction.clone()
        ),
        Error::TargetCddMissing
    );

    let _ = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();

    let transaction = UniqueCall::new(
        Utility::nonce(bob.clone()) + 1,
        RuntimeCall::Balances(BalancesCall::transfer {
            dest: charlie.into(),
            value: 59,
        }),
    );

    assert_noop!(
        Utility::relay_tx(origin.clone(), bob, Signature([0; 64]).into(), transaction),
        Error::InvalidNonce
    );
}

#[test]
fn batch_secondary_with_permissions_works() {
    ExtBuilder::default()
        .build()
        .execute_with(batch_secondary_with_permissions);
}

fn batch_secondary_with_permissions() {
    System::set_block_number(1);
    let alice = User::new(AccountKeyring::Alice).balance(1_000);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);
    let check_name = |name| {
        assert_eq!(
            Portfolio::portfolios(&alice.did, &PortfolioNumber(1)),
            Some(name)
        );
    };

    // Add Bob.
    add_secondary_key(alice.did, bob.acc());
    let low_risk_name: PortfolioName = b"low risk".into();
    assert_ok!(Portfolio::create_portfolio(
        bob.origin(),
        low_risk_name.clone()
    ));
    assert_last_event!(EventTest::Portfolio(
        pallet_portfolio::Event::PortfolioCreated(_, _, _)
    ));
    check_name(low_risk_name.clone());

    // Set and check Bob's permissions.
    let bob_pallet_permissions = vec![
        PalletPermissions::new(b"Identity".into(), SubsetRestriction::Whole),
        PalletPermissions::new(
            b"Portfolio".into(),
            SubsetRestriction::elems(vec![
                b"move_portfolio_funds".into(),
                b"rename_portfolio".into(),
            ]),
        ),
    ];
    let bob_permissions = Permissions {
        extrinsic: SubsetRestriction::These(bob_pallet_permissions.into_iter().collect()),
        ..Permissions::default()
    };
    assert_ok!(Identity::set_secondary_key_permissions(
        alice.origin(),
        bob.acc(),
        bob_permissions,
    ));
    let bob_secondary_key = &get_secondary_keys(alice.did)[0];
    let check_permission = |name: &[u8], t| {
        assert_eq!(
            t,
            bob_secondary_key.has_extrinsic_permission(&b"Portfolio".into(), &name.into())
        );
    };
    check_permission(b"rename_portfolio", true);
    check_permission(b"create_portfolio", false);

    // Call a disallowed extrinsic.
    let high_risk_name: PortfolioName = b"high risk".into();
    assert_noop!(
        Portfolio::create_portfolio(bob.origin(), high_risk_name.clone()),
        pallet_permissions::Error::<TestStorage>::UnauthorizedCaller
    );

    // Call one disallowed and one allowed extrinsic in a batch.
    let calls = vec![
        RuntimeCall::Portfolio(PortfolioCall::create_portfolio {
            name: high_risk_name.clone(),
        }),
        RuntimeCall::Portfolio(PortfolioCall::rename_portfolio {
            num: 1u64.into(),
            to_name: high_risk_name.clone(),
        }),
    ];
    let expected_error: DispatchError =
        pallet_permissions::Error::<TestStorage>::UnauthorizedCaller.into();
    assert_ok!(Utility::batch(bob.origin(), calls.clone()));
    assert_event_doesnt_exist!(EventTest::Utility(Event::BatchCompleted));
    assert_event_exists!(
        EventTest::Utility(Event::BatchInterrupted { index, error }),
        *index == 0u32 && error == &expected_error
    );
    check_name(low_risk_name);

    // Call the same extrinsics optimistically.
    assert_ok!(Utility::force_batch(bob.origin(), calls));
    assert_event_exists!(
        EventTest::Utility(Event::ItemFailed { error }),
        error == &expected_error
    );
    check_name(high_risk_name);
}

///
/// Tests ported from `substrate/frame/utility/src/tests.rs`.
///

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| next_block());
    ext
}

fn call_foobar(err: bool, start_weight: Weight, end_weight: Option<Weight>) -> RuntimeCall {
    RuntimeCall::Example(ExampleCall::foobar {
        err,
        start_weight,
        end_weight,
    })
}

#[test]
fn sub_batch_with_root_works() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        let ferdie = User::new(AccountKeyring::Ferdie).balance(10);
        let k = b"a".to_vec();
        let call = RuntimeCall::System(frame_system::Call::set_storage {
            items: vec![(k.clone(), k.clone())],
        });
        assert!(!TestBaseCallFilter::contains(&call));
        assert_eq!(Balances::free_balance(charlie.acc()), 10);
        assert_eq!(Balances::free_balance(ferdie.acc()), 10);
        assert_ok!(Utility::batch(
            RuntimeOrigin::root(),
            vec![
                RuntimeCall::Balances(BalancesCall::force_transfer {
                    source: charlie.acc().into(),
                    dest: ferdie.acc().into(),
                    value: 5
                }),
                RuntimeCall::Balances(BalancesCall::force_transfer {
                    source: charlie.acc().into(),
                    dest: ferdie.acc().into(),
                    value: 5
                }),
                call, // Check filters are correctly bypassed
            ]
        ));
        assert_eq!(Balances::free_balance(charlie.acc()), 0);
        assert_eq!(Balances::free_balance(ferdie.acc()), 20);
        assert_eq!(storage::unhashed::get_raw(&k), Some(k));
    });
}

#[test]
fn sub_batch_with_signed_works() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        let ferdie = User::new(AccountKeyring::Ferdie).balance(10);
        assert_eq!(Balances::free_balance(charlie.acc()), 10);
        assert_eq!(Balances::free_balance(ferdie.acc()), 10);
        assert_ok!(Utility::batch(
            charlie.origin(),
            vec![transfer(ferdie.acc(), 5), transfer(ferdie.acc(), 5)]
        ),);
        assert_eq!(Balances::free_balance(charlie.acc()), 0);
        assert_eq!(Balances::free_balance(ferdie.acc()), 20);
    });
}

#[test]
fn sub_batch_with_signed_filters() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie);
        assert_ok!(Utility::batch(
            charlie.origin(),
            vec![RuntimeCall::Example(ExampleCall::noop2 {})]
        ),);
        System::assert_last_event(
            utility::Event::BatchInterrupted {
                index: 0,
                error: frame_system::Error::<TestStorage>::CallFiltered.into(),
            }
            .into(),
        );
    });
}

#[test]
fn sub_batch_handles_weight_refund() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie);
        let start_weight = Weight::from_ref_time(100);
        let end_weight = Weight::from_ref_time(75);
        let diff = start_weight - end_weight;
        let batch_len = 4;

        // Full weight when ok
        let inner_call = call_foobar(false, start_weight, None);
        let batch_calls = vec![inner_call; batch_len as usize];
        let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_ok!(result);
        assert_eq!(extract_actual_weight(&result, &info), info.weight);

        // Refund weight when ok
        let inner_call = call_foobar(false, start_weight, Some(end_weight));
        let batch_calls = vec![inner_call; batch_len as usize];
        let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_ok!(result);
        // Diff is refunded
        assert_eq!(
            extract_actual_weight(&result, &info),
            info.weight - diff * batch_len
        );

        // Full weight when err
        let good_call = call_foobar(false, start_weight, None);
        let bad_call = call_foobar(true, start_weight, None);
        let batch_calls = vec![good_call, bad_call];
        let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_ok!(result);
        System::assert_last_event(
            utility::Event::BatchInterrupted {
                index: 1,
                error: DispatchError::Other(""),
            }
            .into(),
        );
        // No weight is refunded
        assert_eq!(extract_actual_weight(&result, &info), info.weight);

        // Refund weight when err
        let good_call = call_foobar(false, start_weight, Some(end_weight));
        let bad_call = call_foobar(true, start_weight, Some(end_weight));
        let batch_calls = vec![good_call, bad_call];
        let batch_len = batch_calls.len() as u64;
        let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_ok!(result);
        System::assert_last_event(
            utility::Event::BatchInterrupted {
                index: 1,
                error: DispatchError::Other(""),
            }
            .into(),
        );
        assert_eq!(
            extract_actual_weight(&result, &info),
            info.weight - diff * batch_len
        );

        // Partial batch completion
        let good_call = call_foobar(false, start_weight, Some(end_weight));
        let bad_call = call_foobar(true, start_weight, Some(end_weight));
        let batch_calls = vec![good_call, bad_call.clone(), bad_call];
        let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_ok!(result);
        System::assert_last_event(
            utility::Event::BatchInterrupted {
                index: 1,
                error: DispatchError::Other(""),
            }
            .into(),
        );
        assert_eq!(
            extract_actual_weight(&result, &info),
            // Real weight is 2 calls at end_weight
            <TestStorage as UtilityConfig>::WeightInfo::batch(2) + end_weight * 2,
        );
    });
}

#[test]
fn sub_batch_all_works() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        let ferdie = User::new(AccountKeyring::Ferdie).balance(10);
        assert_eq!(Balances::free_balance(charlie.acc()), 10);
        assert_eq!(Balances::free_balance(ferdie.acc()), 10);
        assert_ok!(Utility::batch_all(
            charlie.origin(),
            vec![transfer(ferdie.acc(), 5), transfer(ferdie.acc(), 5)]
        ),);
        assert_eq!(Balances::free_balance(charlie.acc()), 0);
        assert_eq!(Balances::free_balance(ferdie.acc()), 20);
    });
}

#[test]
fn sub_batch_all_revert() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        let ferdie = User::new(AccountKeyring::Ferdie).balance(10);
        let call = transfer(ferdie.acc(), 5);
        let info = call.get_dispatch_info();

        assert_eq!(Balances::free_balance(charlie.acc()), 10);
        assert_eq!(Balances::free_balance(ferdie.acc()), 10);
        let batch_all_calls = RuntimeCall::Utility(UtilityCall::<TestStorage>::batch_all {
            calls: vec![
                transfer(ferdie.acc(), 5),
                transfer(ferdie.acc(), 10),
                transfer(ferdie.acc(), 5),
            ],
        });
        assert_noop!(
            batch_all_calls.dispatch(charlie.origin()),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: Some(
                        <TestStorage as UtilityConfig>::WeightInfo::batch_all(2) + info.weight * 2
                    ),
                    pays_fee: Pays::Yes
                },
                error: pallet_balances::Error::<TestStorage>::InsufficientBalance.into()
            }
        );
        assert_eq!(Balances::free_balance(charlie.acc()), 10);
        assert_eq!(Balances::free_balance(ferdie.acc()), 10);
    });
}

#[test]
fn sub_batch_all_handles_weight_refund() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        let start_weight = Weight::from_ref_time(100);
        let end_weight = Weight::from_ref_time(75);
        let diff = start_weight - end_weight;
        let batch_len = 4;

        // Full weight when ok
        let inner_call = call_foobar(false, start_weight, None);
        let batch_calls = vec![inner_call; batch_len as usize];
        let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_ok!(result);
        assert_eq!(extract_actual_weight(&result, &info), info.weight);

        // Refund weight when ok
        let inner_call = call_foobar(false, start_weight, Some(end_weight));
        let batch_calls = vec![inner_call; batch_len as usize];
        let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_ok!(result);
        // Diff is refunded
        assert_eq!(
            extract_actual_weight(&result, &info),
            info.weight - diff * batch_len
        );

        // Full weight when err
        let good_call = call_foobar(false, start_weight, None);
        let bad_call = call_foobar(true, start_weight, None);
        let batch_calls = vec![good_call, bad_call];
        let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_err_ignore_postinfo!(result, "The cake is a lie.");
        // No weight is refunded
        assert_eq!(extract_actual_weight(&result, &info), info.weight);

        // Refund weight when err
        let good_call = call_foobar(false, start_weight, Some(end_weight));
        let bad_call = call_foobar(true, start_weight, Some(end_weight));
        let batch_calls = vec![good_call, bad_call];
        let batch_len = batch_calls.len() as u64;
        let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_err_ignore_postinfo!(result, "The cake is a lie.");
        assert_eq!(
            extract_actual_weight(&result, &info),
            info.weight - diff * batch_len
        );

        // Partial batch completion
        let good_call = call_foobar(false, start_weight, Some(end_weight));
        let bad_call = call_foobar(true, start_weight, Some(end_weight));
        let batch_calls = vec![good_call, bad_call.clone(), bad_call];
        let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
        let info = call.get_dispatch_info();
        let result = call.dispatch(charlie.origin());
        assert_err_ignore_postinfo!(result, "The cake is a lie.");
        assert_eq!(
            extract_actual_weight(&result, &info),
            // Real weight is 2 calls at end_weight
            <TestStorage as UtilityConfig>::WeightInfo::batch_all(2) + end_weight * 2,
        );
    });
}

#[test]
fn sub_batch_all_does_not_nest() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        let ferdie = User::new(AccountKeyring::Ferdie).balance(10);
        let batch_all = RuntimeCall::Utility(UtilityCall::batch_all {
            calls: vec![
                transfer(ferdie.acc(), 1),
                transfer(ferdie.acc(), 1),
                transfer(ferdie.acc(), 1),
            ],
        });

        let info = batch_all.get_dispatch_info();

        assert_eq!(Balances::free_balance(charlie.acc()), 10);
        assert_eq!(Balances::free_balance(ferdie.acc()), 10);
        // A nested batch_all call will not pass the filter, and fail with `CallFiltered`.
        assert_noop!(
            Utility::batch_all(charlie.origin(), vec![batch_all.clone()]),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: Some(
                        <TestStorage as UtilityConfig>::WeightInfo::batch_all(1) + info.weight
                    ),
                    pays_fee: Pays::Yes
                },
                error: frame_system::Error::<TestStorage>::CallFiltered.into(),
            }
        );

        // And for those who want to get a little fancy, we check that the filter persists across
        // other kinds of dispatch wrapping functions... in this case
        // `batch_all(batch(batch_all(..)))`
        let batch_nested = RuntimeCall::Utility(UtilityCall::batch {
            calls: vec![batch_all],
        });
        // Batch will end with `Ok`, but does not actually execute as we can see from the event
        // and balances.
        assert_ok!(Utility::batch_all(charlie.origin(), vec![batch_nested]));
        System::assert_has_event(
            utility::Event::BatchInterrupted {
                index: 0,
                error: frame_system::Error::<TestStorage>::CallFiltered.into(),
            }
            .into(),
        );
        assert_eq!(Balances::free_balance(charlie.acc()), 10);
        assert_eq!(Balances::free_balance(ferdie.acc()), 10);
    });
}

#[test]
fn sub_batch_limit() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        let calls = vec![RuntimeCall::System(SystemCall::remark { remark: vec![] }); 40_000];
        assert_noop!(
            Utility::batch(charlie.origin(), calls.clone()),
            pallet_utility::Error::<TestStorage>::TooManyCalls
        );
        assert_noop!(
            Utility::batch_all(charlie.origin(), calls),
            pallet_utility::Error::<TestStorage>::TooManyCalls
        );
    });
}

#[test]
fn sub_force_batch_works() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        let ferdie = User::new(AccountKeyring::Ferdie).balance(10);
        assert_eq!(Balances::free_balance(charlie.acc()), 10);
        assert_eq!(Balances::free_balance(ferdie.acc()), 10);
        assert_ok!(Utility::force_batch(
            charlie.origin(),
            vec![
                transfer(ferdie.acc(), 5),
                call_foobar(true, Weight::from_ref_time(75), None),
                transfer(ferdie.acc(), 10),
                transfer(ferdie.acc(), 5),
            ]
        ));
        System::assert_last_event(utility::Event::BatchCompletedWithErrors.into());
        System::assert_has_event(
            utility::Event::ItemFailed {
                error: DispatchError::Other(""),
            }
            .into(),
        );
        assert_eq!(Balances::free_balance(charlie.acc()), 0);
        assert_eq!(Balances::free_balance(ferdie.acc()), 20);

        assert_ok!(Utility::force_batch(
            ferdie.origin(),
            vec![transfer(charlie.acc(), 5), transfer(charlie.acc(), 5),]
        ));
        System::assert_last_event(utility::Event::BatchCompleted.into());

        assert_ok!(Utility::force_batch(
            charlie.origin(),
            vec![transfer(ferdie.acc(), 50),]
        ),);
        System::assert_last_event(utility::Event::BatchCompletedWithErrors.into());
    });
}

#[test]
fn sub_none_origin_does_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Utility::force_batch(RuntimeOrigin::none(), vec![]),
            BadOrigin
        );
        assert_noop!(Utility::batch(RuntimeOrigin::none(), vec![]), BadOrigin);
        assert_noop!(Utility::batch_all(RuntimeOrigin::none(), vec![]), BadOrigin);
    })
}

#[test]
fn sub_batch_doesnt_work_with_inherents() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        // fails because inherents expect the origin to be none.
        assert_ok!(Utility::batch(
            charlie.origin(),
            vec![RuntimeCall::Timestamp(TimestampCall::set { now: 42 }),]
        ));
        System::assert_last_event(
            utility::Event::BatchInterrupted {
                index: 0,
                error: BadOrigin.into(),
            }
            .into(),
        );
    })
}

#[test]
fn sub_force_batch_doesnt_work_with_inherents() {
    new_test_ext().execute_with(|| {
        // fails because inherents expect the origin to be none.
        assert_ok!(Utility::force_batch(
            RuntimeOrigin::root(),
            vec![RuntimeCall::Timestamp(TimestampCall::set { now: 42 }),]
        ));
        System::assert_last_event(utility::Event::BatchCompletedWithErrors.into());
    })
}

#[test]
fn sub_batch_all_doesnt_work_with_inherents() {
    new_test_ext().execute_with(|| {
        let charlie = User::new(AccountKeyring::Charlie).balance(10);
        let batch_all = RuntimeCall::Utility(UtilityCall::batch_all {
            calls: vec![RuntimeCall::Timestamp(TimestampCall::set { now: 42 })],
        });
        let info = batch_all.get_dispatch_info();

        // fails because inherents expect the origin to be none.
        assert_noop!(
            batch_all.dispatch(charlie.origin()),
            DispatchErrorWithPostInfo {
                post_info: PostDispatchInfo {
                    actual_weight: Some(info.weight),
                    pays_fee: Pays::Yes
                },
                error: BadOrigin.into(),
            }
        );
    })
}

#[test]
fn batch_works_with_committee_origin() {
    new_test_ext().execute_with(|| {
        let proposer = User::new(AccountKeyring::Dave);

        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);
        set_members(vec![bob.did, charlie.did]);

        assert_ok!(Pips::set_min_proposal_deposit(RuntimeOrigin::root(), 10));

        let consensus_batch = |calls| {
            consensus_call(
                RuntimeCall::Utility(UtilityCall::batch { calls }),
                &[bob, charlie],
            )
        };

        let id_committee = Pips::pip_id_sequence();
        assert_ok!(committee_proposal(0));
        let id_snapshot = Pips::pip_id_sequence();
        assert_ok!(community_proposal(proposer, 10));
        assert_ok!(Pips::snapshot(bob.origin()));
        consensus_batch(vec![
            RuntimeCall::Pips(pallet_pips::Call::approve_committee_proposal { id: id_committee }),
            RuntimeCall::Pips(pallet_pips::Call::enact_snapshot_results {
                results: vec![(id_snapshot, SnapshotResult::Approve)],
            }),
        ]);
        assert_state(id_committee, false, ProposalState::Scheduled);
        assert_state(id_snapshot, false, ProposalState::Scheduled);
    })
}

#[test]
fn force_batch_works_with_committee_origin() {
    new_test_ext().execute_with(|| {
        let proposer = User::new(AccountKeyring::Dave);

        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);
        set_members(vec![bob.did, charlie.did]);

        assert_ok!(Pips::set_min_proposal_deposit(RuntimeOrigin::root(), 10));

        let consensus_batch = |calls| {
            consensus_call(
                RuntimeCall::Utility(UtilityCall::force_batch { calls }),
                &[bob, charlie],
            )
        };

        let id_committee = Pips::pip_id_sequence();
        assert_ok!(committee_proposal(0));
        let id_snapshot = Pips::pip_id_sequence();
        assert_ok!(community_proposal(proposer, 10));
        assert_ok!(Pips::snapshot(bob.origin()));
        consensus_batch(vec![
            RuntimeCall::Pips(pallet_pips::Call::approve_committee_proposal { id: id_committee }),
            RuntimeCall::Pips(pallet_pips::Call::enact_snapshot_results {
                results: vec![(id_snapshot, SnapshotResult::Approve)],
            }),
        ]);
        assert_state(id_committee, false, ProposalState::Scheduled);
        assert_state(id_snapshot, false, ProposalState::Scheduled);
    })
}

#[test]
fn batch_all_works_with_committee_origin() {
    new_test_ext().execute_with(|| {
        let proposer = User::new(AccountKeyring::Dave);

        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);
        set_members(vec![bob.did, charlie.did]);

        assert_ok!(Pips::set_min_proposal_deposit(RuntimeOrigin::root(), 10));

        let consensus_batch = |calls| {
            consensus_call(
                RuntimeCall::Utility(UtilityCall::batch_all { calls }),
                &[bob, charlie],
            )
        };

        let id_committee = Pips::pip_id_sequence();
        assert_ok!(committee_proposal(0));
        let id_snapshot = Pips::pip_id_sequence();
        assert_ok!(community_proposal(proposer, 10));
        assert_ok!(Pips::snapshot(bob.origin()));
        consensus_batch(vec![
            RuntimeCall::Pips(pallet_pips::Call::approve_committee_proposal { id: id_committee }),
            RuntimeCall::Pips(pallet_pips::Call::enact_snapshot_results {
                results: vec![(id_snapshot, SnapshotResult::Approve)],
            }),
        ]);
        assert_state(id_committee, false, ProposalState::Scheduled);
        assert_state(id_snapshot, false, ProposalState::Scheduled);
    })
}

#[test]
fn sub_with_weight_works() {
    new_test_ext().execute_with(|| {
        let weights = <TestStorage as frame_system::Config>::BlockWeights::get();
        let upgrade_code_call = Box::new(RuntimeCall::System(
            frame_system::Call::set_code_without_checks { code: vec![] },
        ));
        // Weight before is max.
        assert_eq!(
            upgrade_code_call.get_dispatch_info().weight,
            weights.max_block
        );
        assert_eq!(
            upgrade_code_call.get_dispatch_info().class,
            frame_support::dispatch::DispatchClass::Operational
        );

        let with_weight_call = UtilityCall::<TestStorage>::with_weight {
            call: upgrade_code_call,
            weight: Weight::from_parts(123, 456),
        };
        // Weight after is set by Root.
        assert_eq!(
            with_weight_call.get_dispatch_info().weight,
            Weight::from_parts(123, 456)
        );
        assert_eq!(
            with_weight_call.get_dispatch_info().class,
            frame_support::dispatch::DispatchClass::Operational
        );
    })
}
