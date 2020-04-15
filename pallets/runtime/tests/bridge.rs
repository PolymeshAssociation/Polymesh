mod common;
use common::{
    storage::{register_keyring_account, register_keyring_account_with_balance, Call, TestStorage},
    ExtBuilder,
};

use frame_support::{assert_err, assert_ok, StorageDoubleMap};
use polymesh_primitives::{IdentityId, Signatory};
use polymesh_runtime::{
    bridge::{self, BridgeTx, IssueRecipient},
    multisig,
};
use polymesh_runtime_balances as balances;
use polymesh_runtime_identity as identity;
use sp_runtime::traits::OnInitialize;
use test_client::AccountKeyring;

type Bridge = bridge::Module<TestStorage>;
type Error = bridge::Error<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Authorizations = identity::Authorizations<TestStorage>;
type MultiSig = multisig::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type System = frame_system::Module<TestStorage>;

macro_rules! assert_tx_approvals {
    ($address:expr, $proposal_id:expr, $num_approvals:expr) => {{
        assert_eq!(
            MultiSig::tx_approvals(&($address, $proposal_id)),
            $num_approvals
        );
    }};
}

#[test]
fn can_issue_to_identity() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .build()
        .execute_with(can_issue_to_identity_we);
}

fn can_issue_to_identity_we() {
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
    let bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();
    let charlie_did =
        register_keyring_account_with_balance(AccountKeyring::Charlie, 1_000).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());
    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        555
    ));
    assert_ok!(Balances::top_up_identity_balance(bob.clone(), bob_did, 555));
    assert_ok!(Balances::top_up_identity_balance(
        charlie.clone(),
        charlie_did,
        555
    ));
    let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            Signatory::from(charlie_did)
        ],
        2,
    ));
    assert_eq!(MultiSig::ms_signs_required(controller), 2);
    let last_authorization = |did: IdentityId| {
        <Authorizations>::iter_prefix(Signatory::from(did))
            .next()
            .unwrap()
            .auth_id
    };
    assert_ok!(MultiSig::accept_multisig_signer_as_identity(
        alice.clone(),
        last_authorization(alice_did)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_identity(
        bob.clone(),
        last_authorization(bob_did)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_identity(
        charlie.clone(),
        last_authorization(charlie_did)
    ));
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(alice_did)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(bob_did)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(charlie_did)),
        true
    );
    let admin = frame_system::RawOrigin::Signed(Default::default());
    assert_ok!(Bridge::change_controller(Origin::system(admin), controller));
    assert_eq!(Bridge::controller(), controller);
    let amount = 1_000_000_000_000_000_000_000;
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: IssueRecipient::Identity(bob_did),
        amount,
        tx_hash: Default::default(),
    };
    let proposal = Box::new(Call::Bridge(bridge::Call::handle_bridge_tx(
        bridge_tx.clone(),
    )));
    assert_eq!(MultiSig::proposal_ids(&controller, proposal.clone()), None);
    assert_tx_approvals!(controller, 0, 0);
    assert_tx_approvals!(controller, 1, 0);

    assert_ok!(Bridge::propose_bridge_tx(bob.clone(), bridge_tx.clone()));
    assert_tx_approvals!(controller, 0, 1);
    assert_tx_approvals!(controller, 1, 0);
    let bobs_balance = Balances::identity_balance(bob_did);

    assert_eq!(
        MultiSig::proposal_ids(&controller, proposal.clone()),
        Some(0)
    );
    assert_ok!(Bridge::propose_bridge_tx(charlie, bridge_tx.clone()));
    assert_eq!(MultiSig::tx_approvals(&(controller, 0)), 2);
    assert_eq!(MultiSig::tx_approvals(&(controller, 1)), 0);

    assert_eq!(MultiSig::proposal_ids(&controller, proposal), Some(0));
    let new_bobs_balance = Balances::identity_balance(bob_did);
    assert_eq!(new_bobs_balance, bobs_balance + amount);
    // Attempt to handle the same transaction again.
    assert!(Bridge::handled_txs(&bridge_tx));
    assert_err!(
        Bridge::handle_bridge_tx(Origin::signed(controller), bridge_tx),
        Error::ProposalAlreadyHandled
    );
}

#[test]
fn can_change_controller() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did =
            register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
        let bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();
        let charlie_did =
            register_keyring_account_with_balance(AccountKeyring::Charlie, 1_000).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let charlie = Origin::signed(AccountKeyring::Charlie.public());
        assert_ok!(Balances::top_up_identity_balance(
            alice.clone(),
            alice_did,
            555
        ));
        assert_ok!(Balances::top_up_identity_balance(bob.clone(), bob_did, 555));
        assert_ok!(Balances::top_up_identity_balance(
            charlie.clone(),
            charlie_did,
            555
        ));
        let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did)
            ],
            2,
        ));
        assert_eq!(MultiSig::ms_signs_required(controller), 2);
        let last_authorization = |did: IdentityId| {
            <Authorizations>::iter_prefix(Signatory::from(did))
                .next()
                .unwrap()
                .auth_id
        };
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            last_authorization(alice_did)
        ));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            bob.clone(),
            last_authorization(bob_did)
        ));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            charlie.clone(),
            last_authorization(charlie_did)
        ));
        assert_eq!(
            MultiSig::ms_signers(controller, Signatory::from(alice_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(controller, Signatory::from(bob_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(controller, Signatory::from(charlie_did)),
            true
        );
        let admin = frame_system::RawOrigin::Signed(Default::default());
        assert_ok!(Bridge::change_controller(Origin::system(admin), controller));
        assert_eq!(Bridge::controller(), controller);
    });
}

#[test]
fn cannot_propose_without_controller() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let amount = 1_000_000;
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Identity(alice_did),
            amount,
            tx_hash: Default::default(),
        };
        assert_err!(
            Bridge::propose_bridge_tx(alice, bridge_tx),
            Error::ControllerNotSet,
        );
    });
}

#[test]
fn cannot_call_bridge_callback_extrinsics() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_account = AccountKeyring::Alice.public();
        let alice = Origin::signed(alice_account);
        assert_err!(
            Bridge::change_controller(alice.clone(), alice_account),
            Error::BadAdmin,
        );
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Account(alice_account),
            amount: 1_000_000,
            tx_hash: Default::default(),
        };
        assert_err!(Bridge::handle_bridge_tx(alice, bridge_tx), Error::BadCaller);
    });
}

#[test]
fn can_freeze_and_unfreeze_bridge() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .build()
        .execute_with(do_freeze_and_unfreeze_bridge);
}

fn do_freeze_and_unfreeze_bridge() {
    let admin = Origin::system(frame_system::RawOrigin::Signed(Default::default()));
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
    let bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();
    let charlie_did =
        register_keyring_account_with_balance(AccountKeyring::Charlie, 1_000).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());
    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        555
    ));
    assert_ok!(Balances::top_up_identity_balance(bob.clone(), bob_did, 555));
    assert_ok!(Balances::top_up_identity_balance(
        charlie.clone(),
        charlie_did,
        555
    ));
    let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            Signatory::from(charlie_did)
        ],
        2,
    ));
    assert_eq!(MultiSig::ms_signs_required(controller), 2);
    let last_authorization = |did: IdentityId| {
        <Authorizations>::iter_prefix(Signatory::from(did))
            .next()
            .unwrap()
            .auth_id
    };
    assert_ok!(MultiSig::accept_multisig_signer_as_identity(
        alice.clone(),
        last_authorization(alice_did)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_identity(
        bob.clone(),
        last_authorization(bob_did)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_identity(
        charlie.clone(),
        last_authorization(charlie_did)
    ));
    assert_ok!(Bridge::change_controller(admin.clone(), controller));
    assert_eq!(Bridge::controller(), controller);
    let amount = 1_000_000_000_000_000_000_000;
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: IssueRecipient::Identity(bob_did),
        amount,
        tx_hash: Default::default(),
    };
    let proposal = Box::new(Call::Bridge(bridge::Call::handle_bridge_tx(
        bridge_tx.clone(),
    )));
    assert_eq!(MultiSig::proposal_ids(&controller, proposal.clone()), None);
    assert_tx_approvals!(controller, 0, 0);
    // First propose the transaction using the bridge API.
    assert_ok!(Bridge::propose_bridge_tx(bob.clone(), bridge_tx.clone()));
    // Freeze the bridge with the transaction still in flight.
    assert_ok!(Bridge::freeze(admin.clone()));
    assert!(Bridge::frozen());
    assert_tx_approvals!(controller, 0, 1);
    let bobs_balance = || Balances::identity_balance(bob_did);
    let starting_bobs_balance = bobs_balance();
    assert_eq!(
        MultiSig::proposal_ids(&controller, proposal.clone()),
        Some(0)
    );
    // Approve the transaction bypassing the bridge API. The transaction will be handled but will itself be
    // frozen.
    assert_ok!(MultiSig::approve_as_identity(charlie, controller, 0));
    assert_eq!(MultiSig::tx_approvals(&(controller, 0)), 2);
    assert_eq!(MultiSig::proposal_ids(&controller, proposal), Some(0));
    // The tokens were not issued because the transaction is frozen.
    assert_eq!(bobs_balance(), starting_bobs_balance);
    assert!(!Bridge::handled_txs(&bridge_tx));
    assert!(Bridge::frozen_txs(&bridge_tx));
    // Unfreeze the bridge.
    assert_ok!(Bridge::unfreeze(admin.clone()));
    assert!(!Bridge::frozen());
    // Still no issue. The transaction needs to be unfrozen.
    assert_eq!(bobs_balance(), starting_bobs_balance);
    assert!(!Bridge::handled_txs(bridge_tx.clone()));
    assert_ok!(Bridge::unfreeze_txs(admin.clone(), vec![bridge_tx.clone()]));
    // Now the tokens are issued.
    assert_eq!(bobs_balance(), starting_bobs_balance + amount);
    assert!(!Bridge::frozen_txs(&bridge_tx));
    assert!(!Bridge::pending_txs(bob_did).contains(&bridge_tx));
    // Attempt to handle the same transaction again.
    assert!(Bridge::handled_txs(&bridge_tx));
    assert_err!(
        Bridge::handle_bridge_tx(Origin::signed(controller), bridge_tx),
        Error::ProposalAlreadyHandled
    );
}

#[test]
fn can_timelock_txs() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .build()
        .execute_with(do_timelock_txs);
}

fn next_block() {
    let block_number = System::block_number() + 1;
    System::set_block_number(block_number);
    // Call the timelocked tx handler.
    Bridge::on_initialize(block_number);
}

fn do_timelock_txs() {
    let admin = Origin::system(frame_system::RawOrigin::Signed(Default::default()));
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
    let bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(alice_did), Signatory::from(bob_did)],
        1,
    ));
    assert_eq!(MultiSig::ms_signs_required(controller), 1);
    let last_authorization = |did: IdentityId| {
        <Authorizations>::iter_prefix(Signatory::from(did))
            .next()
            .unwrap()
            .auth_id
    };
    assert_ok!(MultiSig::accept_multisig_signer_as_identity(
        alice.clone(),
        last_authorization(alice_did)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_identity(
        bob.clone(),
        last_authorization(bob_did)
    ));
    assert_ok!(Bridge::change_controller(admin.clone(), controller));
    assert_eq!(Bridge::controller(), controller);
    let timelock = 3;
    assert_ok!(Bridge::change_timelock(admin, timelock));
    let amount = 1_000_000_000_000_000_000_000;
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: IssueRecipient::Identity(bob_did),
        amount,
        tx_hash: Default::default(),
    };
    let proposal = Box::new(Call::Bridge(bridge::Call::handle_bridge_tx(
        bridge_tx.clone(),
    )));
    let bobs_balance = || Balances::identity_balance(bob_did);
    let starting_bobs_balance = bobs_balance();
    assert_eq!(MultiSig::proposal_ids(&controller, proposal.clone()), None);
    assert_tx_approvals!(controller, 0, 0);
    assert_ok!(Bridge::propose_bridge_tx(bob.clone(), bridge_tx.clone()));
    assert_tx_approvals!(controller, 0, 1);
    let first_block_number = System::block_number();
    let unlock_block_number = first_block_number + timelock;
    assert_eq!(
        Bridge::timelocked_txs(unlock_block_number),
        vec![bridge_tx.clone()]
    );
    next_block();
    assert_eq!(bobs_balance(), starting_bobs_balance);
    next_block();
    assert_eq!(bobs_balance(), starting_bobs_balance);
    next_block();
    assert_eq!(System::block_number(), unlock_block_number);
    assert!(Bridge::timelocked_txs(unlock_block_number).is_empty());
    assert_eq!(bobs_balance(), starting_bobs_balance + amount);
    assert!(Bridge::handled_txs(&bridge_tx));
}
