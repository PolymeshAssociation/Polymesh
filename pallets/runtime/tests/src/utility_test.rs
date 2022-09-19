use super::{
    assert_event_doesnt_exist, assert_event_exists, assert_last_event,
    pips_test::assert_balance,
    storage::{
        add_secondary_key, get_secondary_keys, Call, EventTest, Identity, Portfolio, System,
        TestStorage, User, Utility,
    },
    ExtBuilder,
};
use codec::Encode;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use frame_system::EventRecord;
use pallet_balances::Call as BalancesCall;
use pallet_portfolio::Call as PortfolioCall;
use pallet_utility::{self as utility, Event, UniqueCall};
use polymesh_primitives::{
    PalletPermissions, Permissions, PortfolioName, PortfolioNumber, SubsetRestriction,
};
use sp_core::sr25519::Signature;
use test_client::AccountKeyring;

type Balances = pallet_balances::Module<TestStorage>;
type Error = utility::Error<TestStorage>;

fn transfer(to: User, amount: u128) -> Call {
    Call::Balances(BalancesCall::transfer {
        dest: to.acc().into(),
        value: amount,
    })
}

const ERROR: DispatchError = DispatchError::Module(sp_runtime::ModuleError {
    index: 4,
    error: [2, 0, 0, 0],
    message: None,
});

fn assert_event(event: Event) {
    assert_eq!(
        System::events().pop().unwrap().event,
        EventTest::Utility(event)
    )
}

fn batch_test(test: impl FnOnce(User, User)) {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);

        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        test(alice, bob)
    });
}

#[test]
fn batch_with_signed_works() {
    batch_test(|alice, bob| {
        let prev_alice_balance = Balances::free_balance(&alice.acc());
        let prev_bob_balance = Balances::free_balance(&bob.acc());
        let calls = vec![transfer(bob, 400), transfer(bob, 400)];
        assert_ok!(Utility::batch(alice.origin(), calls));
        assert_balance(alice.acc(), prev_alice_balance - 400 - 400, 0);
        assert_balance(bob.acc(), prev_bob_balance + 400 + 400, 0);
        assert_event(Event::BatchCompleted(vec![1, 1]));
    });
}

#[test]
fn batch_early_exit_works() {
    batch_test(|alice, bob| {
        let prev_alice_balance = Balances::free_balance(&alice.acc());
        let prev_bob_balance = Balances::free_balance(&bob.acc());
        let calls = vec![
            transfer(bob, 400),
            transfer(bob, prev_alice_balance + 900), // early exit here.
            transfer(bob, 400),
        ];
        assert_ok!(Utility::batch(alice.origin(), calls));
        assert_balance(alice.acc(), prev_alice_balance - 400, 0);
        assert_balance(bob.acc(), prev_bob_balance + 400, 0);
        assert_event(Event::BatchInterrupted(vec![1, 0], (1, ERROR)));
    })
}

#[test]
fn batch_optimistic_works() {
    batch_test(|alice, bob| {
        let prev_alice_balance = Balances::free_balance(&alice.acc());
        let prev_bob_balance = Balances::free_balance(&bob.acc());
        let calls = vec![transfer(bob, 401), transfer(bob, 402)];
        assert_ok!(Utility::batch_optimistic(alice.origin(), calls));
        assert_event(Event::BatchCompleted(vec![1, 1]));
        assert_balance(alice.acc(), prev_alice_balance - 401 - 402, 0);
        assert_balance(bob.acc(), prev_bob_balance + 401 + 402, 0);
    });
}

#[test]
fn batch_optimistic_failures_listed() {
    batch_test(|alice, bob| {
        let prev_alice_balance = Balances::free_balance(&alice.acc());
        let prev_bob_balance = Balances::free_balance(&bob.acc());
        assert_ok!(Utility::batch_optimistic(
            alice.origin(),
            vec![
                transfer(bob, 401),                      // YAY.
                transfer(bob, prev_alice_balance + 900), // NAY.
                transfer(bob, prev_alice_balance + 800), // NAY.
                transfer(bob, 402),                      // YAY.
                transfer(bob, prev_alice_balance + 403), // NAY.
            ]
        ));
        assert_event(Event::BatchOptimisticFailed(
            vec![1, 0, 0, 1, 0],
            vec![(1, ERROR), (2, ERROR), (4, ERROR)],
        ));
        assert_balance(alice.acc(), prev_alice_balance - 401 - 402, 0);
        assert_balance(bob.acc(), prev_bob_balance + 401 + 402, 0);
    });
}

#[test]
fn batch_atomic_works() {
    batch_test(|alice, bob| {
        let prev_alice_balance = Balances::free_balance(&alice.acc());
        let prev_bob_balance = Balances::free_balance(&bob.acc());
        let calls = vec![transfer(bob, 401), transfer(bob, 402)];
        assert_ok!(Utility::batch_atomic(alice.origin(), calls));
        assert_event(Event::BatchCompleted(vec![1, 1]));
        assert_balance(alice.acc(), prev_alice_balance - 401 - 402, 0);
        assert_balance(bob.acc(), prev_bob_balance + 401 + 402, 0);
    });
}

#[test]
fn batch_atomic_early_exit_works() {
    batch_test(|alice, bob| {
        let prev_alice_balance = Balances::free_balance(&alice.acc());
        let prev_bob_balance = Balances::free_balance(&bob.acc());
        let trans = |x| transfer(bob, x);
        let calls = vec![
            trans(400),
            trans(prev_alice_balance + 900), // This call aborts the `atomic` batch.
            trans(400),
        ];
        assert_ok!(Utility::batch_atomic(alice.origin(), calls));
        assert_balance(alice.acc(), prev_alice_balance, 0);
        assert_balance(bob.acc(), prev_bob_balance, 0);
        assert_event(Event::BatchInterrupted(vec![1, 0], (1, ERROR)));
    })
}

#[test]
fn relay_happy_case() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(_relay_happy_case);
}

fn _relay_happy_case() {
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);
    let charlie = User::new(AccountKeyring::Charlie);

    let prev_bob_balance = Balances::free_balance(&bob.acc());
    let prev_charlie_balance = Balances::free_balance(&charlie.acc());

    let transaction = UniqueCall::new(
        Utility::nonce(bob.acc()),
        Call::Balances(BalancesCall::transfer {
            dest: charlie.acc().into(),
            value: 50,
        }),
    );

    assert_ok!(Utility::relay_tx(
        alice.origin(),
        bob.acc(),
        AccountKeyring::Bob.sign(&transaction.encode()).into(),
        transaction
    ));

    assert_balance(bob.acc(), prev_bob_balance - 50, 0);
    assert_balance(charlie.acc(), prev_charlie_balance + 50, 0);
}

#[test]
fn relay_unhappy_cases() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(_relay_unhappy_cases);
}

fn _relay_unhappy_cases() {
    let alice = User::new(AccountKeyring::Alice);

    let bob = AccountKeyring::Bob.to_account_id();

    let charlie = AccountKeyring::Charlie.to_account_id();

    let transaction = UniqueCall::new(
        Utility::nonce(&bob),
        Call::Balances(BalancesCall::transfer {
            dest: charlie.clone().into(),
            value: 59,
        }),
    );

    assert_noop!(
        Utility::relay_tx(
            alice.origin(),
            bob.clone(),
            Signature([0; 64]).into(),
            transaction.clone()
        ),
        Error::InvalidSignature
    );

    assert_noop!(
        Utility::relay_tx(
            alice.origin(),
            bob,
            AccountKeyring::Bob.sign(&transaction.encode()).into(),
            transaction.clone()
        ),
        Error::TargetCddMissing
    );

    let bob = User::new(AccountKeyring::Bob);
    let transaction = UniqueCall::new(
        Utility::nonce(bob.acc()) + 1,
        Call::Balances(BalancesCall::transfer {
            dest: charlie.into(),
            value: 59,
        }),
    );

    assert_noop!(
        Utility::relay_tx(
            alice.origin(),
            bob.acc(),
            Signature([0; 64]).into(),
            transaction
        ),
        Error::InvalidNonce
    );
}

#[test]
fn batch_secondary_with_permissions_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(batch_secondary_with_permissions);
}

fn batch_secondary_with_permissions() {
    System::set_block_number(1);
    let alice = User::new(AccountKeyring::Alice).balance(1_000);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);
    let check_name = |name| {
        assert_eq!(Portfolio::portfolios(&alice.did, &PortfolioNumber(1)), name);
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
        Call::Portfolio(PortfolioCall::create_portfolio {
            name: high_risk_name.clone(),
        }),
        Call::Portfolio(PortfolioCall::rename_portfolio {
            num: 1u64.into(),
            to_name: high_risk_name.clone(),
        }),
    ];
    assert_ok!(Utility::batch(bob.origin(), calls.clone()));
    assert_event_doesnt_exist!(EventTest::Utility(Event::BatchCompleted(_)));
    assert_event_exists!(
        EventTest::Utility(Event::BatchInterrupted(events, err)),
        events == &[0] && err.0 == 0
    );
    check_name(low_risk_name);

    // Call the same extrinsics optimistically.
    assert_ok!(Utility::batch_optimistic(bob.origin(), calls));
    assert_event_exists!(
        EventTest::Utility(Event::BatchOptimisticFailed(events, errors)),
        events == &[0, 1] && errors.len() == 1 && errors[0].0 == 0
    );
    check_name(high_risk_name);
}
