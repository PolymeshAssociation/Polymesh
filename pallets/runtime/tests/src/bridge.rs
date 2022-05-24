use super::{
    fast_forward_blocks, next_block,
    storage::{Call, TestStorage, User},
    ExtBuilder,
};

use frame_support::{
    assert_noop, assert_ok, storage::IterableStorageDoubleMap, traits::Currency, weights::Weight,
};
use pallet_bridge::{
    self as bridge, BridgeTx as GBridgeTx, BridgeTxDetail as GBridgeTxDetail, BridgeTxStatus,
};
use polymesh_primitives::AccountId;
use test_client::AccountKeyring::*;

type Bridge = bridge::Module<TestStorage>;
type BridgeGenesis = bridge::GenesisConfig<TestStorage>;
type Error = bridge::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;
type MultiSig = pallet_multisig::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::Origin;
type System = frame_system::Pallet<TestStorage>;
type Scheduler = pallet_scheduler::Pallet<TestStorage>;

type BridgeTx = GBridgeTx<AccountId>;
type BridgeTxDetail = GBridgeTxDetail<u32>;

const AMOUNT: u128 = 1_000_000_000;
const AMOUNT_OVER_LIMIT: u128 = 1_000_000_000_000_000_000_000;
const WEIGHT_EXPECTED_1: u64 = 766606000;
const WEIGHT_EXPECTED_2: u64 = 1556059000;
const MIN_SIGNS_REQUIRED: u64 = 2;

fn test_with_controller(test: &dyn Fn(&[AccountId])) {
    let (admin, bob, charlie, dave) = (
        Alice.to_account_id(),
        Bob.to_account_id(),
        Charlie.to_account_id(),
        Dave.to_account_id(),
    );

    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .add_regular_users_from_accounts(&[
            admin.clone(),
            Eve.to_account_id(),
            Ferdie.to_account_id(),
        ])
        .set_bridge_controller(
            admin,
            [bob.clone(), charlie.clone(), dave.clone()].into(),
            MIN_SIGNS_REQUIRED,
        )
        .build()
        .execute_with(|| test(&[bob, charlie, dave]));
}

fn signed_admin() -> Origin {
    Origin::signed(Bridge::admin())
}

fn make_bridge_tx(recipient: AccountId, amount: u128) -> BridgeTx {
    BridgeTx {
        nonce: 1,
        recipient,
        amount,
        tx_hash: Default::default(),
    }
}

fn alice_bridge_tx(amount: u128) -> BridgeTx {
    make_bridge_tx(Alice.to_account_id(), amount)
}

fn bridge_tx_to_proposal(tx: &BridgeTx) -> Call {
    Call::Bridge(bridge::Call::handle_bridge_tx {
        bridge_tx: tx.clone(),
    })
}

fn alice_make_bridge_tx(amount: u128) -> BridgeTx {
    make_bridge_tx(Alice.to_account_id(), amount)
}

fn alice_balance() -> u128 {
    Balances::total_balance(&Alice.to_account_id())
}

fn alice_tx_details(tx_id: u32) -> BridgeTxDetail {
    Bridge::bridge_tx_details(Alice.to_account_id(), tx_id)
}

fn make_bridge_txs(amount: u128) -> [BridgeTx; 3] {
    [Alice, Eve, Ferdie].map(|acc| make_bridge_tx(acc.to_account_id(), amount))
}

fn signers_approve_bridge_tx(tx: BridgeTx, signers: &[AccountId]) -> BridgeTx {
    let controller = Bridge::controller();
    let proposal = bridge_tx_to_proposal(&tx);
    let mut proposal_id = None;

    // Use minimun number of signs to approve it.
    for i in 0..(MIN_SIGNS_REQUIRED as usize) {
        assert_ok!(Bridge::propose_bridge_tx(
            Origin::signed(signers[i].clone()),
            tx.clone()
        ));

        // Verify approvals.
        // Fetch proposal ID if unknown.
        let p_id = proposal_id
            .get_or_insert_with(|| {
                MultiSig::proposal_ids(&controller.clone(), proposal.clone()).unwrap_or_default()
            })
            .clone();
        assert_eq!(
            MultiSig::proposal_detail(&(controller.clone(), p_id)).approvals,
            (i + 1) as u64
        );
    }
    tx
}

fn advance_block_and_verify_alice_balance(offset: u32, expected_balance: u128) -> Weight {
    (0..=offset)
        .map(|_| {
            assert_eq!(alice_balance(), expected_balance);
            next_block()
        })
        .sum()
}

fn next_unlock_block_number() -> u32 {
    System::block_number() + 1 + Bridge::timelock()
}

fn ensure_tx_status(recipient: AccountId, nonce: u32, expected_status: BridgeTxStatus) -> u32 {
    let tx_details = Bridge::bridge_tx_details(recipient, nonce);
    assert_eq!(tx_details.status, expected_status);
    tx_details.execution_block
}

#[test]
fn can_issue_to_identity() {
    test_with_controller(&|signers| {
        let tx = alice_make_bridge_tx(AMOUNT);
        let tx = signers_approve_bridge_tx(tx, signers);

        // Wait for timelock, and transaction should be handled.
        fast_forward_blocks(Bridge::timelock() + 1);
        assert_eq!(alice_tx_details(1).status, BridgeTxStatus::Handled);

        let controller = Origin::signed(Bridge::controller());
        assert_noop!(
            Bridge::handle_bridge_tx(controller, tx),
            Error::ProposalAlreadyHandled
        );
    });
}

#[test]
fn can_change_controller() {
    test_with_controller(&|_signers| {
        let controller = Bob.to_account_id();

        assert_ok!(Bridge::change_controller(
            signed_admin(),
            controller.clone()
        ));
        assert_eq!(Bridge::controller(), controller);
    });
}

#[test]
fn cannot_propose_without_controller() {
    let alice = Alice.to_account_id();

    ExtBuilder::default()
        .add_regular_users_from_accounts(&[alice.clone()])
        .build()
        .execute_with(|| {
            let bridge_tx = alice_bridge_tx(1_000_000);
            assert_noop!(
                Bridge::propose_bridge_tx(Origin::signed(alice), bridge_tx),
                Error::ControllerNotSet
            );
        });
}

#[test]
fn cannot_call_bridge_callback_extrinsics() {
    test_with_controller(&|_signers| {
        let controller = Bridge::controller();
        let no_admin = Origin::signed(Bob.to_account_id());
        assert_noop!(
            Bridge::change_controller(no_admin.clone(), controller),
            Error::BadAdmin
        );

        let bridge_tx = alice_bridge_tx(1_000_000);
        assert_noop!(
            Bridge::handle_bridge_tx(no_admin, bridge_tx),
            Error::BadCaller
        );
    });
}

#[test]
fn can_admin_freeze_and_unfreeze_bridge() {
    test_with_controller(&do_admin_freeze_and_unfreeze_bridge)
}

fn do_admin_freeze_and_unfreeze_bridge(signers: &[AccountId]) {
    let alice = Alice.to_account_id();
    let admin = signed_admin();
    let tx = alice_make_bridge_tx(AMOUNT);
    let timelock = Bridge::timelock();

    // Freeze the bridge with the transaction still in flight.
    assert_ok!(Bridge::freeze(admin.clone()));
    assert!(Bridge::frozen());

    let starting_alices_balance = alice_balance();
    signers_approve_bridge_tx(tx, signers);

    ensure_tx_status(alice.clone(), 1, BridgeTxStatus::Absent);
    fast_forward_blocks(timelock);

    // Weight calculation when bridge is freezed.
    ensure_tx_status(alice.clone(), 1, BridgeTxStatus::Timelocked);
    assert_eq!(next_block(), WEIGHT_EXPECTED_1);

    // Unfreeze the bridge.
    assert_ok!(Bridge::unfreeze(admin));
    assert!(!Bridge::frozen());
    next_block();

    // Still no issue. The transaction needs to be processed.
    assert_eq!(alice_balance(), starting_alices_balance);
    let execution_block = ensure_tx_status(alice.clone(), 1, BridgeTxStatus::Pending(1));

    // It will be 0 as txn has to wait for X more block to execute.
    assert_eq!(
        advance_block_and_verify_alice_balance(
            execution_block - System::block_number() - 1,
            starting_alices_balance
        ),
        WEIGHT_EXPECTED_2
    );

    // Now the tokens are issued.
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT);
    ensure_tx_status(alice, 1, BridgeTxStatus::Handled);
}

#[test]
fn test_freeze_admins() {
    test_with_controller(&do_test_freeze_admins)
}

fn do_test_freeze_admins(_signers: &[AccountId]) {
    let eve = User::existing(Eve);
    let ferdie = User::existing(Ferdie);
    let admin = signed_admin();

    let test_freeze = |user: User, can_freeze: bool| {
        // Make sure we start with the bridge unfrozen.
        assert!(!Bridge::frozen());

        if can_freeze {
            // User is allowed to freeze the bridge.
            assert_ok!(Bridge::freeze(user.origin()));
            assert!(Bridge::frozen());
        } else {
            // User is not allowed to freeze the bridge.
            assert_noop!(Bridge::freeze(user.origin()), Error::BadAdmin);
            assert!(!Bridge::frozen());

            // Use admin to freeze the bridge.
            assert_ok!(Bridge::freeze(admin.clone()));
            assert!(Bridge::frozen());
        }

        // User is not allowed to unfreeze the bridge.
        assert_noop!(Bridge::unfreeze(user.origin()), Error::BadAdmin);
        assert!(Bridge::frozen());

        // Use admin to unfreeze the bridge.
        assert_ok!(Bridge::unfreeze(admin.clone()));
        assert!(!Bridge::frozen());
    };
    let add_freeze_admin = |user: User| {
        // Use admin to add a freeze admin.
        assert_ok!(Bridge::add_freeze_admin(admin.clone(), user.acc()));
        assert!(Bridge::freeze_admins(user.acc()));

        // Check that they can freeze/unfreeze the bridge.
        test_freeze(user, true);
    };
    let remove_freeze_admin = |user: User| {
        // Use admin to remove a freeze admin.
        assert_ok!(Bridge::remove_freeze_admin(admin.clone(), user.acc()));
        assert!(!Bridge::freeze_admins(user.acc()));

        // Check that they cannot freeze/unfreeze the bridge.
        test_freeze(user, false);
    };

    // Eve and Ferdie are not freeze admins.
    test_freeze(eve, false);
    test_freeze(ferdie, false);

    // Add Eve as a freeze admin.
    add_freeze_admin(eve);

    // Ferdie is still denied.
    test_freeze(ferdie, false);

    // Add Ferdie as a freeze admin.
    add_freeze_admin(ferdie);

    // Test that Eve can still freeze/unfreeze the bridge.
    test_freeze(eve, true);

    // Remove Eve from freeze admins.
    remove_freeze_admin(eve);

    // Test that Ferdie can still freeze/unfreeze the bridge.
    test_freeze(ferdie, true);

    // Remove Ferdie from freeze admins.
    remove_freeze_admin(ferdie);

    // Both Eve and Ferdie are no longer freeze admins.
    test_freeze(eve, false);
    test_freeze(ferdie, false);
}

#[test]
fn can_timelock_txs() {
    test_with_controller(&do_timelock_txs)
}

fn do_timelock_txs(signers: &[AccountId]) {
    let alice = Alice.to_account_id();
    let tx = alice_make_bridge_tx(AMOUNT);
    let starting_alices_balance = alice_balance();

    // Approve the transaction by `signers`.
    signers_approve_bridge_tx(tx, signers);
    next_block();
    let unlock_block_number = next_unlock_block_number();

    // Tx should be timelocked.
    let execution_block = ensure_tx_status(alice.clone(), 1, BridgeTxStatus::Timelocked);
    assert_eq!(execution_block, unlock_block_number);

    // Alice's balance should not change until `unlock_block_number`.
    advance_block_and_verify_alice_balance(Bridge::timelock(), starting_alices_balance);
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT);

    // Tx was handled.
    let execution_block = ensure_tx_status(alice, 1, BridgeTxStatus::Handled);
    assert_eq!(execution_block, unlock_block_number);
}

#[test]
fn can_rate_limit() {
    test_with_controller(&do_rate_limit);
}

fn do_rate_limit(signers: &[AccountId]) {
    let rate_limit = 1_000_000_000;
    let alice = Alice.to_account_id();
    let admin = signed_admin();
    let tx = alice_make_bridge_tx(AMOUNT_OVER_LIMIT);
    let starting_alices_balance = alice_balance();

    // Set up limit and timeclock.
    assert_ok!(Bridge::change_bridge_limit(admin.clone(), rate_limit, 1));

    // Propose the transaction... and it should not issue due to the current rate_limit.
    signers_approve_bridge_tx(tx, signers);
    advance_block_and_verify_alice_balance(Bridge::timelock(), starting_alices_balance);

    // Still no issue, rate limit reached.
    assert_eq!(alice_balance(), starting_alices_balance);
    assert_ok!(Bridge::change_bridge_limit(admin, AMOUNT_OVER_LIMIT + 1, 1));

    // Mint successful after limit is increased.
    next_block();
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT_OVER_LIMIT);
    ensure_tx_status(alice, 1, BridgeTxStatus::Handled);
}

#[test]
fn is_exempted() {
    test_with_controller(&do_exempted)
}

fn do_exempted(signers: &[AccountId]) {
    let alice = Alice.to_account_id();
    let alice_did = Identity::get_identity(&alice).expect("Alice missing identity");
    let tx = alice_make_bridge_tx(AMOUNT_OVER_LIMIT);
    let starting_alices_balance = alice_balance();

    // Send and approve the transaction.
    signers_approve_bridge_tx(tx, signers);
    next_block();

    let execution_block = ensure_tx_status(alice.clone(), 1, BridgeTxStatus::Timelocked);
    assert_eq!(execution_block, next_unlock_block_number());

    advance_block_and_verify_alice_balance(Bridge::timelock(), starting_alices_balance);

    // Still no issue, rate limit reached.
    assert_eq!(alice_balance(), starting_alices_balance);
    assert_ok!(Bridge::change_bridge_exempted(
        signed_admin(),
        vec![(alice_did, true)]
    ));
    next_block();

    // Mint successful after exemption.
    advance_block_and_verify_alice_balance(Bridge::timelock(), starting_alices_balance);
    ensure_tx_status(alice, 1, BridgeTxStatus::Handled);
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT_OVER_LIMIT);
}

#[test]
fn can_force_mint() {
    test_with_controller(&do_force_mint);
}

fn do_force_mint(signers: &[AccountId]) {
    let alice = Alice.to_account_id();
    let tx = alice_make_bridge_tx(AMOUNT_OVER_LIMIT);
    let starting_alices_balance = alice_balance();
    let timelock = Bridge::timelock();

    let tx = signers_approve_bridge_tx(tx, signers);
    next_block();
    let unlock_block_number = next_unlock_block_number();

    let execution_block = ensure_tx_status(alice.clone(), 1, BridgeTxStatus::Timelocked);
    assert_eq!(execution_block, unlock_block_number);

    advance_block_and_verify_alice_balance(timelock, starting_alices_balance);

    // Still no issue, rate limit reached.
    assert_eq!(alice_balance(), starting_alices_balance);
    assert_ok!(Bridge::force_handle_bridge_tx(signed_admin(), tx));

    // Mint successful after force handle.
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT_OVER_LIMIT);
    let execution_block = ensure_tx_status(alice, 1, BridgeTxStatus::Handled);
    assert_eq!(execution_block, unlock_block_number);
}

#[test]
fn change_admin() {
    test_with_controller(&|signers| {
        let new_admin = signers[0].clone();
        assert_ne!(new_admin, Bridge::admin());

        assert_noop!(
            Bridge::change_admin(Origin::signed(new_admin.clone()), new_admin.clone()),
            Error::BadAdmin
        );
        assert_ok!(Bridge::change_admin(signed_admin(), new_admin.clone()));
        assert_eq!(Bridge::admin(), new_admin);
    });
}

#[test]
fn change_timelock() {
    test_with_controller(&|signers| {
        let no_admin = Origin::signed(signers[0].clone());
        let new_timelock = Bridge::timelock() * 2;

        assert_noop!(
            Bridge::change_timelock(no_admin, new_timelock),
            Error::BadAdmin
        );
        assert_ok!(Bridge::change_timelock(signed_admin(), new_timelock));
        assert_eq!(Bridge::timelock(), new_timelock);
    });
}

#[test]
fn freeze_txs() {
    test_with_controller(&do_freeze_txs);
}

fn do_freeze_txs(signers: &[AccountId]) {
    let no_admin = Origin::signed(signers[0].clone());

    // Create some txs and register the recipients' balance.
    let txs = make_bridge_txs(AMOUNT).map(|tx| signers_approve_bridge_tx(tx, signers));
    let init_balances = txs
        .iter()
        .map(|tx| Balances::total_balance(&tx.recipient))
        .collect::<Vec<_>>();

    fast_forward_blocks(Bridge::timelock());

    // Freeze all txs except the first one.
    let frozen_txs = txs.iter().skip(1).cloned().collect::<Vec<_>>();
    assert!(!frozen_txs.is_empty());
    assert_noop!(
        Bridge::freeze_txs(no_admin.clone(), frozen_txs.clone()),
        Error::BadAdmin
    );
    assert_ok!(Bridge::freeze_txs(signed_admin(), frozen_txs.clone()));
    next_block();

    // Double check that first TX is done, and any other is frozen.
    let tx = txs.iter().next().unwrap();
    ensure_tx_status(tx.recipient.clone(), tx.nonce, BridgeTxStatus::Handled);

    frozen_txs.iter().for_each(|tx| {
        ensure_tx_status(tx.recipient.clone(), tx.nonce, BridgeTxStatus::Frozen);
    });

    // Unfreeze frozen TXs.
    assert_noop!(
        Bridge::unfreeze_txs(no_admin, frozen_txs.clone()),
        Error::BadAdmin
    );
    assert_ok!(Bridge::unfreeze_txs(signed_admin(), frozen_txs));

    // Verify that all TXs are done and balances of owner are updated.
    txs.iter()
        .zip(init_balances.iter())
        .for_each(|(tx, init_balance)| {
            ensure_tx_status(tx.recipient.clone(), tx.nonce, BridgeTxStatus::Handled);
            assert_eq!(
                Balances::total_balance(&tx.recipient),
                init_balance + AMOUNT
            );
        });
}

#[test]
fn batch_propose_bridge_tx() {
    test_with_controller(&do_batch_propose_bridge_tx);
}

fn do_batch_propose_bridge_tx(signers: &[AccountId]) {
    let alice = Origin::signed(Alice.to_account_id());
    let txs = make_bridge_txs(AMOUNT).to_vec();
    let ensure_txs_status = |txs: &[BridgeTx], status: BridgeTxStatus| {
        txs.iter().for_each(|tx| {
            ensure_tx_status(tx.recipient.clone(), tx.nonce, status);
        });
    };

    assert_ok!(Bridge::batch_propose_bridge_tx(alice, txs.clone()));

    // Transactions should be `Absent`.
    ensure_txs_status(&txs, BridgeTxStatus::Absent);
    txs.iter().for_each(|tx| {
        signers_approve_bridge_tx(tx.clone(), signers);
    });

    // Advance block
    fast_forward_blocks(Bridge::timelock());
    ensure_txs_status(&txs, BridgeTxStatus::Timelocked);

    // Now transactions should be `Handled`.
    let last_block = txs
        .iter()
        .map(|tx| Bridge::bridge_tx_details(tx.recipient.clone(), tx.nonce).execution_block)
        .max()
        .unwrap_or_default();
    let offset = last_block - System::block_number();
    fast_forward_blocks(offset);
    ensure_txs_status(&txs, BridgeTxStatus::Handled);
}

#[test]
fn genesis_txs() {
    let [alice, bob, charlie] = [Alice, Bob, Charlie].map(|k| k.to_account_id());
    let complete_txs = vec![
        BridgeTx {
            nonce: 1,
            recipient: alice.clone(),
            amount: 111,
            tx_hash: Default::default(),
        },
        BridgeTx {
            nonce: 2,
            recipient: bob.clone(),
            amount: 222,
            tx_hash: Default::default(),
        },
    ];

    let regular_users = vec![alice, bob];
    ExtBuilder::default()
        .cdd_providers(vec![charlie])
        .add_regular_users_from_accounts(&regular_users)
        .set_bridge_complete_tx(complete_txs.clone())
        .build()
        .execute_with(|| check_genesis_txs(complete_txs.into_iter()));
}

fn check_genesis_txs(txs: impl Iterator<Item = BridgeTx>) {
    let mut txs: Vec<_> = txs
        .map(|tx| {
            (
                tx.recipient,
                tx.nonce,
                BridgeTxDetail {
                    amount: tx.amount,
                    status: BridgeTxStatus::Handled,
                    execution_block: 0,
                    tx_hash: tx.tx_hash,
                },
            )
        })
        .collect();
    txs.sort();
    for tx in &txs {
        assert_eq!(tx.2.amount, Balances::total_balance(&tx.0));
    }
    assert_eq!(
        <bridge::BridgeTxDetails<TestStorage>>::iter().collect::<Vec<_>>(),
        txs
    );
}

#[test]
fn remove_txs() {
    test_with_controller(&do_remove_txs);
}

fn do_remove_txs(signers: &[AccountId]) {
    let no_admin = Origin::signed(signers[0].clone());

    // Create some txs and register the recipients' balance.
    let txs = make_bridge_txs(AMOUNT).map(|tx| signers_approve_bridge_tx(tx, signers));
    fast_forward_blocks(Bridge::timelock());

    // Freeze all txs except the first one.
    let frozen_txs = txs.iter().skip(1).cloned().collect::<Vec<_>>();
    assert!(!frozen_txs.is_empty());
    assert_ok!(Bridge::freeze_txs(signed_admin(), frozen_txs.clone()));

    assert_noop!(
        Bridge::remove_txs(no_admin, txs.clone().into()),
        Error::BadAdmin
    );
    assert_noop!(
        Bridge::remove_txs(signed_admin(), txs.clone().into()),
        Error::NotFrozen
    );

    assert_ok!(Bridge::remove_txs(signed_admin(), frozen_txs.clone()));
    assert!(frozen_txs
        .iter()
        .map(|tx| Bridge::get_tx_details(&tx))
        .all(|tx| tx.status == BridgeTxStatus::Absent));
    assert!(Bridge::get_tx_details(&txs[0]).status == BridgeTxStatus::Timelocked);
}
