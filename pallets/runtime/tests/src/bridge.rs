use super::{
    storage::{AccountId, Call, TestStorage},
    ExtBuilder,
};

use frame_support::{
    assert_err, assert_ok,
    storage::IterableStorageDoubleMap,
    traits::{Currency, OnInitialize},
    weights::Weight,
};
use pallet_bridge::{self as bridge, BridgeTx, BridgeTxDetail, BridgeTxStatus};
use sp_core::sr25519::Public;
use test_client::AccountKeyring;

type Bridge = bridge::Module<TestStorage>;
type BridgeGenesis = bridge::GenesisConfig<TestStorage>;
type Error = bridge::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;
type MultiSig = pallet_multisig::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type System = frame_system::Module<TestStorage>;
type Scheduler = pallet_scheduler::Module<TestStorage>;

const AMOUNT: u128 = 1_000_000_000;
const AMOUNT_OVER_LIMIT: u128 = 1_000_000_000_000_000_000_000;
const WEIGHT_EXPECTED: u64 = 500000210u64;

fn test_with_controller(test: &dyn Fn(&[Public], usize)) {
    let admin = AccountKeyring::Alice.public();
    let bob = AccountKeyring::Bob.public();
    let charlie = AccountKeyring::Charlie.public();
    let dave = AccountKeyring::Dave.public();
    let min_signs_required = 2;

    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .add_regular_users_from_accounts(&[admin])
        .set_bridge_controller(admin, [bob, charlie, dave].into(), min_signs_required)
        .build()
        .execute_with(|| {
            let signer_accounts = [bob, charlie, dave];
            test(&signer_accounts, min_signs_required as usize)
        });
}

fn alice_bridge_tx(amount: u128) -> BridgeTx<Public, u128> {
    BridgeTx {
        nonce: 1,
        recipient: AccountKeyring::Alice.public(),
        amount,
        tx_hash: Default::default(),
    }
}

fn alice_proposal_tx(amount: u128) -> Box<Call> {
    let tx = alice_bridge_tx(amount);
    let call = bridge::Call::handle_bridge_tx(tx);

    box (Call::Bridge(call))
}

fn alice_balance() -> u128 {
    Balances::total_balance(&AccountKeyring::Alice.public())
}

fn alice_tx_details(tx_id: u32) -> BridgeTxDetail<u128, u64> {
    Bridge::bridge_tx_details(AccountKeyring::Alice.public(), tx_id)
}

fn signers_approve_proposal(
    proposal: Call,
    signers: &[Public],
    min_signs_required: usize,
) -> BridgeTx<Public, u128> {
    let controller = Bridge::controller();
    match proposal {
        Call::Bridge(bridge::Call::handle_bridge_tx(ref tx)) => {
            // Use minimun number of signs to approve it.
            for i in 0..min_signs_required {
                assert_ok!(Bridge::propose_bridge_tx(
                    Origin::signed(signers[i]),
                    tx.clone()
                ));
                assert_tx_approvals_and_next_block(controller, i + 1);
                assert_eq!(
                    MultiSig::proposal_ids(&controller, proposal.clone()),
                    Some(0)
                );
            }
            return tx.clone();
        }
        _ => panic!("Invalid call"),
    }
}

/// Advances the system `block_number` and run any scheduled task.
fn next_block() -> Weight {
    let block_number = System::block_number() + 1;
    System::set_block_number(block_number);
    // Call the timelocked tx handler.
    Scheduler::on_initialize(block_number)
}

fn advance_block(offset: u64) -> Weight {
    (0..=offset).map(|_| next_block()).sum()
}

fn advance_block_and_verify_alice_balance(offset: u64, expected_balance: u128) -> Weight {
    (0..=offset)
        .map(|_| {
            assert_eq!(alice_balance(), expected_balance);
            next_block()
        })
        .sum()
}

fn next_unlock_block_number() -> u64 {
    System::block_number() + 1 + Bridge::timelock()
}

fn assert_tx_approvals_and_next_block(controller: Public, num_approvals: usize) {
    assert_eq!(
        MultiSig::proposal_detail(&(controller, 0)).approvals,
        num_approvals as u64
    );
}

#[test]
fn can_issue_to_identity() {
    test_with_controller(&|signers, min_signs_required| {
        let proposal = alice_proposal_tx(AMOUNT);
        let tx = signers_approve_proposal(*proposal, signers, min_signs_required);

        // Wait for timelock, and proposal should be handled.
        advance_block(Bridge::timelock() + 1);
        assert_eq!(alice_tx_details(1).status, BridgeTxStatus::Handled);

        let controller = Origin::signed(Bridge::controller());
        assert_err!(
            Bridge::handle_bridge_tx(controller, tx),
            Error::ProposalAlreadyHandled
        );
    });
}

#[test]
fn can_change_controller() {
    test_with_controller(&|_signers, _mins_signs_required| {
        let admin = Origin::signed(Bridge::admin());
        let controller = AccountKeyring::Bob.public();

        assert_ok!(Bridge::change_controller(admin, controller));
        assert_eq!(Bridge::controller(), controller);
    });
}

#[test]
fn cannot_propose_without_controller() {
    let alice = AccountKeyring::Alice.public();

    ExtBuilder::default()
        .add_regular_users_from_accounts(&[alice])
        .build()
        .execute_with(|| {
            let bridge_tx = alice_bridge_tx(1_000_000);
            assert_err!(
                Bridge::propose_bridge_tx(Origin::signed(alice), bridge_tx),
                Error::ControllerNotSet
            );
        });
}

#[test]
fn cannot_call_bridge_callback_extrinsics() {
    test_with_controller(&|_signers, _min_signs_required| {
        let controller = Bridge::controller();
        let no_admin = Origin::signed(AccountKeyring::Bob.public());
        assert_err!(
            Bridge::change_controller(no_admin.clone(), controller),
            Error::BadAdmin
        );

        let bridge_tx = alice_bridge_tx(1_000_000);
        assert_err!(
            Bridge::handle_bridge_tx(no_admin, bridge_tx),
            Error::BadCaller
        );
    });
}

#[test]
fn can_freeze_and_unfreeze_bridge() {
    test_with_controller(&do_freeze_and_unfreeze_bridge)
}

fn do_freeze_and_unfreeze_bridge(signers: &[Public], min_signs_required: usize) {
    let alice = AccountKeyring::Alice.public();
    let admin = Origin::signed(Bridge::admin());
    let proposal = alice_proposal_tx(AMOUNT);
    let timelock = Bridge::timelock();

    // Freeze the bridge with the transaction still in flight.
    assert_ok!(Bridge::freeze(admin.clone()));
    assert!(Bridge::frozen());

    let starting_alices_balance = alice_balance();
    signers_approve_proposal(*proposal, signers, min_signs_required);
    assert_eq!(
        Bridge::bridge_tx_details(alice, &1).status,
        BridgeTxStatus::Absent
    );
    advance_block(timelock);

    // Weight calculation when bridge is freezed
    assert_eq!(
        Bridge::bridge_tx_details(alice, &1).status,
        BridgeTxStatus::Timelocked
    );
    assert_eq!(next_block(), WEIGHT_EXPECTED);

    // Unfreeze the bridge.
    assert_ok!(Bridge::unfreeze(admin));
    assert!(!Bridge::frozen());
    next_block();

    // Still no issue. The transaction needs to be processed.
    assert_eq!(alice_balance(), starting_alices_balance);
    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.status, BridgeTxStatus::Pending(1));

    // It will be 0 as txn has to wait for X more block to execute.
    assert_eq!(
        advance_block_and_verify_alice_balance(
            tx_details.execution_block - System::block_number() - 1,
            starting_alices_balance
        ),
        WEIGHT_EXPECTED
    );

    // Now the tokens are issued.
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT);
    assert_eq!(
        Bridge::bridge_tx_details(alice, &1).status,
        BridgeTxStatus::Handled
    );
}

#[test]
fn can_timelock_txs() {
    test_with_controller(&do_timelock_txs)
}

fn do_timelock_txs(signers: &[Public], min_signs_required: usize) {
    let alice = AccountKeyring::Alice.public();
    let proposal = alice_proposal_tx(AMOUNT);
    let starting_alices_balance = alice_balance();

    // Approves the `proposal` by `signers`.
    signers_approve_proposal(*proposal, signers, min_signs_required);
    next_block();
    let unlock_block_number = next_unlock_block_number();

    // Tx should be timelocked
    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.status, BridgeTxStatus::Timelocked);
    assert_eq!(tx_details.execution_block, unlock_block_number);

    // Alice's banlance should not change until `unlock_block_number`.
    advance_block_and_verify_alice_balance(Bridge::timelock(), starting_alices_balance);
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT);

    // Tx was handled.
    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.execution_block, unlock_block_number);
    assert_eq!(tx_details.status, BridgeTxStatus::Handled);
}

#[test]
fn can_rate_limit() {
    test_with_controller(&do_rate_limit);
}

fn do_rate_limit(signers: &[Public], min_signs_required: usize) {
    let rate_limit = 1_000_000_000;
    let alice = AccountKeyring::Alice.public();
    let admin = Origin::signed(Bridge::admin());
    let proposal = alice_proposal_tx(AMOUNT_OVER_LIMIT);
    let starting_alices_balance = alice_balance();

    // Set up limit and timeclock.
    assert_ok!(Bridge::change_bridge_limit(admin.clone(), rate_limit, 1));

    // Send the proposal... and it should not issue due to the current rate_limit.
    signers_approve_proposal(*proposal, signers, min_signs_required);
    advance_block_and_verify_alice_balance(Bridge::timelock(), starting_alices_balance);

    // Still no issue, rate limit reached
    assert_eq!(alice_balance(), starting_alices_balance);
    assert_ok!(Bridge::change_bridge_limit(admin, AMOUNT_OVER_LIMIT + 1, 1));

    // Mint successful after limit is increased
    next_block();
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT_OVER_LIMIT);
    assert_eq!(
        Bridge::bridge_tx_details(alice, &1).status,
        BridgeTxStatus::Handled
    );
}

#[test]
fn is_exempted() {
    test_with_controller(&do_exempted)
}

fn do_exempted(signers: &[Public], min_signs_required: usize) {
    let alice = AccountKeyring::Alice.public();
    let alice_did = Identity::key_to_identity_dids(alice);
    let proposal = alice_proposal_tx(AMOUNT_OVER_LIMIT);
    let starting_alices_balance = alice_balance();

    // Send and approve the proposal.
    signers_approve_proposal(*proposal, signers, min_signs_required);
    next_block();

    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.status, BridgeTxStatus::Timelocked);
    assert_eq!(tx_details.execution_block, next_unlock_block_number());

    advance_block_and_verify_alice_balance(Bridge::timelock(), starting_alices_balance);

    // Still no issue, rate limit reached
    let admin = Origin::signed(Bridge::admin());
    assert_eq!(alice_balance(), starting_alices_balance);
    assert_ok!(Bridge::change_bridge_exempted(
        admin,
        vec![(alice_did, true)]
    ));
    next_block();

    // Mint successful after exemption
    advance_block_and_verify_alice_balance(Bridge::timelock(), starting_alices_balance);
    assert_eq!(
        Bridge::bridge_tx_details(alice, &1).status,
        BridgeTxStatus::Handled
    );
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT_OVER_LIMIT);
}

#[test]
fn can_force_mint() {
    test_with_controller(&do_force_mint);
}

fn do_force_mint(signers: &[Public], min_signs_required: usize) {
    let alice = AccountKeyring::Alice.public();
    let proposal = alice_proposal_tx(AMOUNT_OVER_LIMIT);
    let starting_alices_balance = alice_balance();
    let timelock = Bridge::timelock();
    let admin = Bridge::admin();

    let tx = signers_approve_proposal(*proposal, signers, min_signs_required);
    next_block();
    let unlock_block_number = next_unlock_block_number();

    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.status, BridgeTxStatus::Timelocked);
    assert_eq!(tx_details.execution_block, unlock_block_number);

    advance_block_and_verify_alice_balance(timelock, starting_alices_balance);

    // Still no issue, rate limit reached
    assert_eq!(alice_balance(), starting_alices_balance);
    assert_ok!(Bridge::force_handle_bridge_tx(Origin::signed(admin), tx));

    // Mint successful after force handle
    assert_eq!(alice_balance(), starting_alices_balance + AMOUNT_OVER_LIMIT);
    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.execution_block, unlock_block_number);
    assert_eq!(tx_details.status, BridgeTxStatus::Handled);
}

#[test]
fn genesis_txs() {
    let alice = AccountKeyring::Alice.public();
    let bob = AccountKeyring::Bob.public();
    let charlie = AccountKeyring::Charlie.public();
    let complete_txs = vec![
        BridgeTx {
            nonce: 1,
            recipient: alice,
            amount: 111,
            tx_hash: Default::default(),
        },
        BridgeTx {
            nonce: 2,
            recipient: bob,
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

fn check_genesis_txs(txs: impl Iterator<Item = BridgeTx<AccountId, u128>>) {
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
