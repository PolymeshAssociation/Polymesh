use super::{
    assert_event_doesnt_exist, assert_event_exists, assert_last_event,
    pips_test::assert_balance,
    storage::{
        add_secondary_key, register_keyring_account_with_balance, Call, EventTest, Identity,
        Origin, Portfolio, System, TestStorage, User, Utility,
    },
    ExtBuilder,
};
use codec::Encode;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use frame_system::EventRecord;
use pallet_balances::Call as BalancesCall;
use pallet_portfolio::Call as PortfolioCall;
use pallet_utility::{self as utility, Event, UniqueCall};
use polymesh_common_utilities::traits::transaction_payment::CddAndFeeDetails;
use polymesh_primitives::{
    AccountId, PalletPermissions, Permissions, PortfolioName, PortfolioNumber, Signatory,
    SubsetRestriction,
};
use sp_core::sr25519::Signature;
use test_client::AccountKeyring;

type Error = utility::Error<TestStorage>;

fn transfer(to: AccountId, amount: u128) -> Call {
    Call::Balances(BalancesCall::transfer(to.into(), amount))
}

const ERROR: DispatchError = DispatchError::Module {
    index: 4,
    error: 2,
    message: None,
};

fn assert_event(event: Event) {
    assert_eq!(
        System::events().pop().unwrap().event,
        EventTest::pallet_utility(event)
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
        assert_ok!(Utility::batch(Origin::signed(alice.clone()), calls));
        assert_balance(alice, 200, 0);
        assert_balance(bob, 1000 + 400 + 400, 0);
        assert_event(Event::BatchCompleted(vec![1, 1]));
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
        assert_ok!(Utility::batch(Origin::signed(alice.clone()), calls));
        assert_balance(alice, 600, 0);
        assert_balance(bob, 1000 + 400, 0);
        assert_event(Event::BatchInterrupted(vec![1, 0], (1, ERROR)));
    })
}

#[test]
fn batch_optimistic_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob.clone(), 401), transfer(bob.clone(), 402)];
        assert_ok!(Utility::batch_optimistic(
            Origin::signed(alice.clone()),
            calls
        ));
        assert_event(Event::BatchCompleted(vec![1, 1]));
        assert_balance(alice, 1000 - 401 - 402, 0);
        assert_balance(bob, 1000 + 401 + 402, 0);
    });
}

#[test]
fn batch_optimistic_failures_listed() {
    batch_test(|alice, bob| {
        assert_ok!(Utility::batch_optimistic(
            Origin::signed(alice.clone()),
            vec![
                transfer(bob.clone(), 401), // YAY.
                transfer(bob.clone(), 900), // NAY.
                transfer(bob.clone(), 800), // NAY.
                transfer(bob.clone(), 402), // YAY.
                transfer(bob.clone(), 403), // NAY.
            ]
        ));
        assert_event(Event::BatchOptimisticFailed(
            vec![1, 0, 0, 1, 0],
            vec![(1, ERROR), (2, ERROR), (4, ERROR)],
        ));
        assert_balance(alice, 1000 - 401 - 402, 0);
        assert_balance(bob, 1000 + 401 + 402, 0);
    });
}

#[test]
fn batch_atomic_works() {
    batch_test(|alice, bob| {
        let calls = vec![transfer(bob.clone(), 401), transfer(bob.clone(), 402)];
        assert_ok!(Utility::batch_atomic(Origin::signed(alice.clone()), calls));
        assert_event(Event::BatchCompleted(vec![1, 1]));
        assert_balance(alice, 1000 - 401 - 402, 0);
        assert_balance(bob, 1000 + 401 + 402, 0);
    });
}

#[test]
fn batch_atomic_early_exit_works() {
    batch_test(|alice, bob| {
        let trans = |x| transfer(bob.clone(), x);
        let calls = vec![trans(400), trans(900), trans(400)];
        assert_ok!(Utility::batch_atomic(Origin::signed(alice.clone()), calls));
        assert_balance(alice, 1000, 0);
        assert_balance(bob, 1000, 0);
        assert_event(Event::BatchInterrupted(vec![1, 0], (1, ERROR)));
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

    let origin = Origin::signed(alice);
    let transaction = UniqueCall::new(
        Utility::nonce(bob.clone()),
        Call::Balances(BalancesCall::transfer(charlie.clone().into(), 50)),
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

    let origin = Origin::signed(alice);
    let transaction = UniqueCall::new(
        Utility::nonce(bob.clone()),
        Call::Balances(BalancesCall::transfer(charlie.clone().into(), 59)),
    );

    assert_noop!(
        Utility::relay_tx(
            origin.clone(),
            bob.clone(),
            Signature::default().into(),
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
        Call::Balances(BalancesCall::transfer(charlie.into(), 59)),
    );

    assert_noop!(
        Utility::relay_tx(
            origin.clone(),
            bob,
            Signature::default().into(),
            transaction
        ),
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
    let bob_key = AccountKeyring::Bob.to_account_id();
    let bob_origin = Origin::signed(bob_key.clone());
    let bob_signer = Signatory::Account(bob_key);
    let check_name = |name| {
        assert_eq!(Portfolio::portfolios(&alice.did, &PortfolioNumber(1)), name);
    };

    // Add Bob.
    add_secondary_key(alice.did, bob_signer.clone());
    let low_risk_name: PortfolioName = b"low risk".into();
    assert_ok!(Portfolio::create_portfolio(
        bob_origin.clone(),
        low_risk_name.clone()
    ));
    assert_last_event!(EventTest::pallet_portfolio(
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
    assert_ok!(Identity::set_permission_to_signer(
        alice.origin(),
        bob_signer,
        bob_permissions,
    ));
    let bob_secondary_key = &Identity::did_records(&alice.did).secondary_keys[0];
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
        Portfolio::create_portfolio(bob_origin.clone(), high_risk_name.clone()),
        pallet_permissions::Error::<TestStorage>::UnauthorizedCaller
    );

    // Call one disallowed and one allowed extrinsic in a batch.
    let calls = vec![
        Call::Portfolio(PortfolioCall::create_portfolio(high_risk_name.clone())),
        Call::Portfolio(PortfolioCall::rename_portfolio(
            1u64.into(),
            high_risk_name.clone(),
        )),
    ];
    assert_ok!(Utility::batch(bob_origin.clone(), calls.clone()));
    assert_event_doesnt_exist!(EventTest::pallet_utility(Event::BatchCompleted(_)));
    assert_event_exists!(
        EventTest::pallet_utility(Event::BatchInterrupted(events, err)),
        events == &[0] && err.0 == 0
    );
    check_name(low_risk_name);

    // Call the same extrinsics optimistically.
    assert_ok!(Utility::batch_optimistic(bob_origin, calls));
    assert_event_exists!(
        EventTest::pallet_utility(Event::BatchOptimisticFailed(events, errors)),
        events == &[0, 1] && errors.len() == 1 && errors[0].0 == 0
    );
    check_name(high_risk_name);
}
