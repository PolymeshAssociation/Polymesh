use super::{
    storage::{
        get_last_auth_id, register_keyring_account, register_keyring_account_with_balance, Call,
        TestStorage,
    },
    ExtBuilder,
};

use frame_support::{
    assert_err, assert_ok,
    traits::{Currency, OnInitialize},
    weights::Weight,
};
use frame_system::RawOrigin;
use pallet_bridge::{self as bridge, BridgeTx, BridgeTxStatus, BridgeTxDetail, Trait};
use polymesh_primitives::Signatory;
use sp_core::sr25519::Public;
use test_client::AccountKeyring;

type Bridge = bridge::Module<TestStorage>;
type Error = bridge::Error<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;
type MultiSig = pallet_multisig::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type System = frame_system::Module<TestStorage>;
type Scheduler = pallet_scheduler::Module<TestStorage>;

fn assert_tx_approvals_and_next_block(controller: Public, num_approvals: usize) {
    assert_eq!(MultiSig::proposal_detail(&(controller, 0)).approvals, num_approvals as u64);
    next_block();
}

fn test_with_controller(test: &dyn Fn(&[Public], usize))
{
    let admin = AccountKeyring::Alice.public();
    let bob = AccountKeyring::Bob.public();
    let charlie = AccountKeyring::Charlie.public();
    let dave = AccountKeyring::Dave.public();
    let min_signs_required = 2;

    ExtBuilder::default()
        .balance_factor(1_000)
        .monied(true)
        .regular_users(vec![admin])
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

    box(Call::Bridge(call))
}

fn alice_balance() -> u128 {
    Balances::total_balance(&AccountKeyring::Alice.public())
}

fn alice_tx_details(tx_id: u32) -> BridgeTxDetail<u128, u64> {
    Bridge::bridge_tx_details(AccountKeyring::Alice.public(), tx_id)
}

fn signers_approve_proposal(proposal: Call, signers: &[Public], min_signs_required: usize) ->  BridgeTx<Public, u128>
{
    let controller = Bridge::controller();
    match proposal {
        Call::Bridge(bridge::Call::handle_bridge_tx(ref tx)) => {
            for i in 0..min_signs_required {
                assert_ok!(Bridge::propose_bridge_tx(Origin::signed(signers[i]), tx.clone()));
                assert_tx_approvals_and_next_block(controller, i + 1);
                assert_eq!(MultiSig::proposal_ids(&controller, proposal.clone()), Some(0));
            }
            next_block();
            return tx.clone()
        },
        _ => panic!("Invalid call"),
   }
}

fn next_block() -> Weight {
    let block_number = System::block_number() + 1;
    System::set_block_number(block_number);
    // Call the timelocked tx handler.
    Scheduler::on_initialize(block_number)
}

/*
fn next_unlock_block_number() -> u64 {
    System::block_number() + Bridge::timelock() + 1
}*/



fn advance_block_until_and_verify_alice_balance(until: u64, expected_balance: u128) {
    assert!( until <= System::block_number());
    while until != System::block_number() {
        assert_eq!(alice_balance(), expected_balance);
        next_block();
    }
}

#[test]
fn can_issue_to_identity(){
    test_with_controller(&can_issue_to_identity_we);
}

fn can_issue_to_identity_we(signers: &[Public], min_signs_required: usize) { 
    let amount = 1_000_000_000_000;
    let proposal = alice_proposal_tx(amount);
    let tx = signers_approve_proposal(*proposal, signers, min_signs_required);

    // Attempt to handle the same transaction again.
    assert_eq!(alice_tx_details(1).status, BridgeTxStatus::Handled);

    let controller = Origin::signed(Bridge::controller());
    assert_err!(Bridge::handle_bridge_tx(controller, tx), Error::ProposalAlreadyHandled);
}

/*
#[test]
fn can_change_controller() {
    test_with_controller( |_signers, _mins_signs_required| {
        let admin = Bridge::admin().into();
        let controller = AccountKeyring::Bob.public();

        assert_ok!(Bridge::change_controller(admin, controller));
        assert_eq!(Bridge::controller(), controller);
    });
}

#[test]
fn cannot_propose_without_controller() {
    let alice = AccountKeyring::Alice.public();

    ExtBuilder::default()
        .regular_users(vec![alice])
        .build()
        .execute_with(|| {
            let bridge_tx = alice_bridge_tx(1_000_000);
            assert_err!(
                Bridge::propose_bridge_tx(alice, bridge_tx),
                Error::ControllerNotSet);
        });
}

#[test]
fn cannot_call_bridge_callback_extrinsics() {
    test_with_controller(|_signers, _min_signs_required| {
        let controller = Bridge::controller();
        let no_admin = Origin::signed(AccountKeyring::Bob.public());
        assert_err!(Bridge::change_controller(no_admin, controller), Error::BadAdmin);

        let bridge_tx = alice_bridge_tx(1_000_000); 
        assert_err!(Bridge::handle_bridge_tx(no_admin, bridge_tx), Error::BadCaller);
    });
}

#[test]
fn can_freeze_and_unfreeze_bridge() {
    test_with_controller(do_freeze_and_unfreeze_bridge)
}

fn do_freeze_and_unfreeze_bridge( signers: &[Public], min_signs_required: u64) {
    let amount = 1_000_000_000;
    let weight_expected = 500000210;
    let admin = Bridge::admin();
    let proposal = alice_proposal_tx(amount);

    // Freeze the bridge with the transaction still in flight.
    assert_ok!(Bridge::freeze(admin));
    assert!(Bridge::frozen());

    let starting_alices_balance = alices_balance();
    signers_approve_proposal(proposal, signers, min_signs_required);
    assert_eq!( Bridge::bridge_tx_details(alice, &1).status, BridgeTxStatus::Timelocked);

    // Weight calculation when bridge is freezed
    assert_eq!(next_block(), weight_expected);
   
    // Unfreeze the bridge.
    assert_ok!(Bridge::unfreeze(creator.clone()));
    assert!(!Bridge::frozen());
   
    // Still no issue. The transaction needs to be processed.
    assert_eq!(alices_balance(), starting_alices_balance);
    assert_eq!( Bridge::bridge_tx_details(alice, &1).status, BridgeTxStatus::Pending(1));

    // It will be 0 as txn has to wait for 1 more block to execute.
    assert_eq!(next_block(), 0);
    assert_eq!(next_block(), weight_expected);

    // Now the tokens are issued.
    assert_eq!(alices_balance(), starting_alices_balance + amount);
    assert_eq!(Bridge::bridge_tx_details(alice, &1).status, BridgeTxStatus::Handled);
}

#[test]
fn can_timelock_txs() {
    test_with_controller(do_timelock_txs)
}

fn do_timelock_txs(signers: &[Public], min_signs_required: u64) {
    let admin = Bridge::admin();
    let amount = 1_000_000_000;
    let proposal = alice_proposal_tx(amount);
    let starting_alices_balance = alices_balance();
    let unlock_block_number = next_unlock_block_number(); 

    // Approves the `proposal` by `signers`.
    signers_approve_proposal(proposal, signers, min_signs_required);

    // Tx should be timelocked
    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.status, BridgeTxStatus::Timelocked);
    assert_eq!(tx_details.execution_block, unlock_block_number);

    // Alice's banlance should not change until `unlock_block_number`.
    advance_block_until_and_verify_alice_balance(unlock_block_number, starting_alices_balance);
    assert_eq!(alices_balance(), starting_alices_balance + amount);

    // Tx was handled.
    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.execution_block, unlock_block_number);
    assert_eq!(tx_details.status, BridgeTxStatus::Handled);
}

#[test]
fn can_rate_limit() {
    test_with_controller(do_rate_limit);
}

fn do_rate_limit(signers: [&Public], min_signs_required: u64) {
    let amount = 1_000_000_000_000_000_000_000;
    let rate_limit = 1_000_000_000;
    let unlock_block_number = next_unlock_block_number(); 

    let admin = Bridge::admin();
    let proposal = alice_proposal_tx(amount);
    let starting_alices_balance = alices_balance();
   
    // Set up limit and timeclock.
    assert_ok!(Bridge::change_bridge_limit(admin.clone(), rate_limit, 1));
   
    // Send the proposal... and it should not issue due to the current rate_limit.
    signers_approve_proposal(proposal, signers, min_signs_required);
    advance_block_until_and_verify_alice_balance(unlock_block_number, starting_alices_balance);

    // Still no issue, rate limit reached
    assert_eq!(alices_balance(), starting_alices_balance);
    assert_ok!(Bridge::change_bridge_limit(admin.clone(), amount +1, 1));

    // Mint successful after limit is increased
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance + amount);
    assert_eq!(
        Bridge::bridge_tx_details(alice, &1).status,
        BridgeTxStatus::Handled
    );
}

#[test]
fn is_exempted() {
    test_with_controller(do_exempted)
}

fn do_exempted(signers: &[Public], min_signs_required: u64) {
    let alice = AccountKeyring::Alice.public();
    let alice_did = Identity::key_to_identity_dids(alice);
    let amount = 1_000_000_000_000_000_000_000;
    let proposal = alice_proposal_tx(amount);
    let starting_alices_balance = alices_balance();
    let unlock_block_number = next_unlock_block_number();

    // Send and approve the proposal.
    signers_approve_proposal(proposal, signers, min_signs_required);
 
    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.status, BridgeTxStatus::Timelocked);
    assert_eq!(tx_details.execution_block, unlock_block_number);

    advance_block_until_and_verify_alice_balance(unlock_block_number, starting_alices_balance);

    // Still no issue, rate limit reached
    assert_eq!(alices_balance(), starting_alices_balance);
    assert_ok!(Bridge::change_bridge_exempted(
        admin.clone(),
        vec![(alice_did, true)]
    ));
    next_block();

    // Mint successful after exemption
    assert_eq!(alices_balance(), starting_alices_balance + amount);
    assert_eq!(
        Bridge::bridge_tx_details(alice, &1).status,
        BridgeTxStatus::Handled
    );
}

#[test]
fn can_force_mint() {
    test_with_controller(do_force_mint)
}

fn do_force_mint(signers: &[Public], min_signs_required: u64) {
    let amount = 1_000_000_000_000_000_000_000;
    let alice = AccountKeyring::Alice.public();
    let proposal = alice_proposal_tx(amount); 
    let starting_alices_balance = alices_balance();
    let unlock_block_number = next_unlock_block_number(); 

    signers_approve_proposal(proposal, signers, min_signs_required);

    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.status, BridgeTxStatus::Timelocked);
    assert_eq!(tx_details.execution_block, unlock_block_number);

    advance_block_until_and_verify_alice_balance(unlock_block_number, starting_alices_balance);

    // Still no issue, rate limit reached
    assert_eq!(alices_balance(), starting_alices_balance);
    assert_ok!(Bridge::force_handle_bridge_tx(admin.clone(), bridge_tx.clone()));

    // Mint successful after force handle
    assert_eq!(alices_balance(), starting_alices_balance + amount);
    let tx_details = Bridge::bridge_tx_details(alice, &1);
    assert_eq!(tx_details.execution_block, unlock_block_number);
    assert_eq!(tx_details.status, BridgeTxStatus::Handled);
}
*/
