use super::{
    storage::{register_keyring_account, register_keyring_account_with_balance, Call, TestStorage},
    ExtBuilder,
};

use frame_support::{assert_err, assert_ok, traits::Currency, StorageDoubleMap};
use pallet_balances as balances;
use pallet_identity as identity;
use pallet_multisig as multisig;
use polymesh_primitives::{AccountKey, Signatory};
use polymesh_runtime_common::bridge::{self, BridgeTx, BridgeTxStatus};
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
            MultiSig::proposal_detail(&($address, $proposal_id)).approvals,
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
    let alice = Origin::signed(AccountKeyring::Alice.public());

    let bob_key = AccountKey::from(AccountKeyring::Bob.public());
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public());
    let dave_key = AccountKey::from(AccountKeyring::Dave.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());
    let dave = Origin::signed(AccountKeyring::Dave.public());

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        555
    ));

    let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![
            Signatory::from(bob_key),
            Signatory::from(charlie_key),
            Signatory::from(dave_key)
        ],
        2,
    ));

    assert_eq!(MultiSig::ms_signs_required(controller), 2);
    let last_authorization = |account: AccountKey| {
        <Authorizations>::iter_prefix(Signatory::from(account))
            .next()
            .unwrap()
            .auth_id
    };

    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        bob.clone(),
        last_authorization(bob_key)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        charlie.clone(),
        last_authorization(charlie_key)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        dave.clone(),
        last_authorization(dave_key)
    ));

    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(bob_key)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(charlie_key)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(dave_key)),
        true
    );

    let admin = frame_system::RawOrigin::Signed(Default::default());
    assert_ok!(Bridge::change_bridge_limit(
        Origin::system(admin.clone()),
        1_000_000_000_000_000_000_000_000,
        1
    ));
    assert_ok!(Bridge::change_controller(
        Origin::system(admin.clone()),
        controller
    ));
    assert_eq!(Bridge::controller(), controller);
    let amount = 1_000_000_000_000_000_000_000;
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: AccountKeyring::Alice.public(),
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
    let alices_balance = Balances::total_balance(&AccountKeyring::Alice.public());

    assert_eq!(
        MultiSig::proposal_ids(&controller, proposal.clone()),
        Some(0)
    );
    assert_ok!(Bridge::propose_bridge_tx(charlie, bridge_tx.clone()));
    assert_tx_approvals!(controller, 0, 2);
    assert_tx_approvals!(controller, 1, 0);

    assert_eq!(MultiSig::proposal_ids(&controller, proposal), Some(0));
    let new_alices_balance = Balances::total_balance(&AccountKeyring::Alice.public());
    assert_eq!(new_alices_balance, alices_balance + amount);
    // Attempt to handle the same transaction again.
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Handled
    );
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
        let alice = Origin::signed(AccountKeyring::Alice.public());

        let bob_key = AccountKey::from(AccountKeyring::Bob.public());
        let charlie_key = AccountKey::from(AccountKeyring::Charlie.public());
        let dave_key = AccountKey::from(AccountKeyring::Dave.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let charlie = Origin::signed(AccountKeyring::Charlie.public());
        let dave = Origin::signed(AccountKeyring::Dave.public());

        assert_ok!(Balances::top_up_identity_balance(
            alice.clone(),
            alice_did,
            555
        ));

        let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![
                Signatory::from(bob_key),
                Signatory::from(charlie_key),
                Signatory::from(dave_key)
            ],
            2,
        ));

        assert_eq!(MultiSig::ms_signs_required(controller), 2);
        let last_authorization = |account: AccountKey| {
            <Authorizations>::iter_prefix(Signatory::from(account))
                .next()
                .unwrap()
                .auth_id
        };

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            last_authorization(bob_key)
        ));
        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            charlie.clone(),
            last_authorization(charlie_key)
        ));
        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            dave.clone(),
            last_authorization(dave_key)
        ));

        assert_eq!(
            MultiSig::ms_signers(controller, Signatory::from(bob_key)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(controller, Signatory::from(charlie_key)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(controller, Signatory::from(dave_key)),
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
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let amount = 1_000_000;
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: AccountKeyring::Alice.public(),
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
            recipient: AccountKeyring::Alice.public(),
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
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());

    let bob_key = AccountKey::from(AccountKeyring::Bob.public());
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        555
    ));

    let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(bob_key), Signatory::from(charlie_key),],
        2,
    ));

    assert_eq!(MultiSig::ms_signs_required(controller), 2);
    let last_authorization = |account: AccountKey| {
        <Authorizations>::iter_prefix(Signatory::from(account))
            .next()
            .unwrap()
            .auth_id
    };

    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        bob.clone(),
        last_authorization(bob_key)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        charlie.clone(),
        last_authorization(charlie_key)
    ));

    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(bob_key)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(charlie_key)),
        true
    );
    let admin = Origin::system(frame_system::RawOrigin::Signed(Default::default()));
    assert_ok!(Bridge::change_controller(admin.clone(), controller));
    assert_eq!(Bridge::controller(), controller);
    let amount = 1_000_000_000_000_000_000_000;
    assert_ok!(Bridge::change_bridge_limit(
        admin.clone(),
        1_000_000_000_000_000_000_000_000,
        1
    ));
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: AccountKeyring::Alice.public(),
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
    let alices_balance = || Balances::total_balance(&AccountKeyring::Alice.public());
    let starting_alices_balance = alices_balance();
    assert_eq!(
        MultiSig::proposal_ids(&controller, proposal.clone()),
        Some(0)
    );
    // Approve the transaction bypassing the bridge API. The transaction will be handled but scheduled for later
    assert_ok!(MultiSig::approve_as_key(charlie, controller, 0));
    assert_tx_approvals!(controller, 0, 2);
    assert_eq!(MultiSig::proposal_ids(&controller, proposal), Some(0));
    // The tokens were not issued because the transaction is frozen.
    assert_eq!(alices_balance(), starting_alices_balance);
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    // Unfreeze the bridge.
    assert_ok!(Bridge::unfreeze(admin.clone()));
    assert!(!Bridge::frozen());
    // Still no issue. The transaction needs to be processed.
    assert_eq!(alices_balance(), starting_alices_balance);
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    next_block();
    next_block();
    next_block();
    next_block();
    // Now the tokens are issued.
    assert_eq!(alices_balance(), starting_alices_balance + amount);
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Handled
    );
    // Attempt to handle the same transaction again.
    assert_err!(
        Bridge::handle_bridge_tx(Origin::signed(controller), bridge_tx),
        Error::ProposalAlreadyHandled
    );
}

fn next_block() {
    let block_number = System::block_number() + 1;
    System::set_block_number(block_number);
    // Call the timelocked tx handler.
    Bridge::on_initialize(block_number);
}

#[test]
fn can_timelock_txs() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .build()
        .execute_with(do_timelock_txs);
}

fn do_timelock_txs() {
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());

    let bob_key = AccountKey::from(AccountKeyring::Bob.public());
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        555
    ));

    let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(bob_key), Signatory::from(charlie_key),],
        1,
    ));

    assert_eq!(MultiSig::ms_signs_required(controller), 1);
    let last_authorization = |account: AccountKey| {
        <Authorizations>::iter_prefix(Signatory::from(account))
            .next()
            .unwrap()
            .auth_id
    };

    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        bob.clone(),
        last_authorization(bob_key)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        charlie.clone(),
        last_authorization(charlie_key)
    ));

    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(bob_key)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(charlie_key)),
        true
    );
    let admin = Origin::system(frame_system::RawOrigin::Signed(Default::default()));
    assert_ok!(Bridge::change_controller(admin.clone(), controller));
    assert_eq!(Bridge::controller(), controller);
    assert_ok!(Bridge::change_bridge_limit(
        admin.clone(),
        1_000_000_000_000_000_000_000_000,
        1
    ));
    let timelock = 3;
    assert_ok!(Bridge::change_timelock(admin.clone(), timelock));
    let amount = 1_000_000_000_000_000_000_000;
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: AccountKeyring::Alice.public(),
        amount,
        tx_hash: Default::default(),
    };
    let proposal = Box::new(Call::Bridge(bridge::Call::handle_bridge_tx(
        bridge_tx.clone(),
    )));
    let alices_balance = || Balances::total_balance(&AccountKeyring::Alice.public());
    let starting_alices_balance = alices_balance();
    assert_eq!(MultiSig::proposal_ids(&controller, proposal.clone()), None);
    assert_tx_approvals!(controller, 0, 0);
    assert_ok!(Bridge::propose_bridge_tx(bob.clone(), bridge_tx.clone()));
    assert_tx_approvals!(controller, 0, 1);
    let first_block_number = System::block_number();
    let unlock_block_number = first_block_number + timelock + 1;
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    assert_eq!(
        Bridge::timelocked_txs(unlock_block_number),
        vec![bridge_tx.clone()]
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).execution_block,
        unlock_block_number
    );
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(System::block_number(), unlock_block_number);
    assert!(Bridge::timelocked_txs(unlock_block_number).is_empty());
    assert_eq!(alices_balance(), starting_alices_balance + amount);
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).execution_block,
        unlock_block_number
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Handled
    );
}

#[test]
fn can_rate_limit() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .build()
        .execute_with(do_rate_limit);
}

fn do_rate_limit() {
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());

    let bob_key = AccountKey::from(AccountKeyring::Bob.public());
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        555
    ));

    let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(bob_key), Signatory::from(charlie_key),],
        1,
    ));

    assert_eq!(MultiSig::ms_signs_required(controller), 1);
    let last_authorization = |account: AccountKey| {
        <Authorizations>::iter_prefix(Signatory::from(account))
            .next()
            .unwrap()
            .auth_id
    };

    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        bob.clone(),
        last_authorization(bob_key)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        charlie.clone(),
        last_authorization(charlie_key)
    ));

    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(bob_key)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(charlie_key)),
        true
    );
    let admin = Origin::system(frame_system::RawOrigin::Signed(Default::default()));
    assert_ok!(Bridge::change_controller(admin.clone(), controller));
    assert_eq!(Bridge::controller(), controller);
    assert_ok!(Bridge::change_bridge_limit(admin.clone(), 1_000_000_000, 1));
    let timelock = 3;
    assert_ok!(Bridge::change_timelock(admin.clone(), timelock));
    let amount = 1_000_000_000_000_000_000_000;
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: AccountKeyring::Alice.public(),
        amount,
        tx_hash: Default::default(),
    };
    let proposal = Box::new(Call::Bridge(bridge::Call::handle_bridge_tx(
        bridge_tx.clone(),
    )));
    let alices_balance = || Balances::total_balance(&AccountKeyring::Alice.public());
    let starting_alices_balance = alices_balance();
    assert_eq!(MultiSig::proposal_ids(&controller, proposal.clone()), None);
    assert_tx_approvals!(controller, 0, 0);
    assert_ok!(Bridge::propose_bridge_tx(bob.clone(), bridge_tx.clone()));
    assert_tx_approvals!(controller, 0, 1);
    let first_block_number = System::block_number();
    let unlock_block_number = first_block_number + timelock + 1;
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    assert_eq!(
        Bridge::timelocked_txs(unlock_block_number),
        vec![bridge_tx.clone()]
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).execution_block,
        unlock_block_number
    );
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(System::block_number(), unlock_block_number);
    // Still no issue, rate limit reached
    assert_eq!(alices_balance(), starting_alices_balance);
    assert_ok!(Bridge::change_bridge_limit(
        admin.clone(),
        1_000_000_000_000_000_000_000_000_000,
        1
    ));
    next_block();
    next_block();
    next_block();
    next_block();
    next_block();
    next_block();
    // Mint successful after limit is increased
    assert!(Bridge::timelocked_txs(unlock_block_number).is_empty());
    assert_eq!(alices_balance(), starting_alices_balance + amount);
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Handled
    );
}

#[test]
fn is_exempted() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .build()
        .execute_with(do_exempted);
}

fn do_exempted() {
    let admin = Origin::system(frame_system::RawOrigin::Signed(Default::default()));
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());

    let bob_key = AccountKey::from(AccountKeyring::Bob.public());
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        555
    ));

    let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(bob_key), Signatory::from(charlie_key),],
        1,
    ));

    assert_eq!(MultiSig::ms_signs_required(controller), 1);
    let last_authorization = |account: AccountKey| {
        <Authorizations>::iter_prefix(Signatory::from(account))
            .next()
            .unwrap()
            .auth_id
    };

    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        bob.clone(),
        last_authorization(bob_key)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        charlie.clone(),
        last_authorization(charlie_key)
    ));
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(bob_key)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(charlie_key)),
        true
    );
    assert_ok!(Bridge::change_controller(admin.clone(), controller));
    assert_eq!(Bridge::controller(), controller);
    assert_ok!(Bridge::change_bridge_limit(admin.clone(), 1_000_000_000, 1));
    let timelock = 3;
    assert_ok!(Bridge::change_timelock(admin.clone(), timelock));
    let amount = 1_000_000_000_000_000_000_000;
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: AccountKeyring::Alice.public(),
        amount,
        tx_hash: Default::default(),
    };
    let proposal = Box::new(Call::Bridge(bridge::Call::handle_bridge_tx(
        bridge_tx.clone(),
    )));
    let alices_balance = || Balances::total_balance(&AccountKeyring::Alice.public());
    let starting_alices_balance = alices_balance();
    assert_eq!(MultiSig::proposal_ids(&controller, proposal.clone()), None);
    assert_tx_approvals!(controller, 0, 0);
    assert_ok!(Bridge::propose_bridge_tx(bob.clone(), bridge_tx.clone()));
    assert_tx_approvals!(controller, 0, 1);
    let first_block_number = System::block_number();
    let unlock_block_number = first_block_number + timelock + 1;
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    assert_eq!(
        Bridge::timelocked_txs(unlock_block_number),
        vec![bridge_tx.clone()]
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).execution_block,
        unlock_block_number
    );
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(System::block_number(), unlock_block_number);
    // Still no issue, rate limit reached
    assert_eq!(alices_balance(), starting_alices_balance);
    assert_ok!(Bridge::change_bridge_exempted(
        admin.clone(),
        vec![(alice_did, true)]
    ));
    next_block();
    next_block();
    next_block();
    next_block();
    next_block();
    next_block();
    // Mint successful after exemption
    assert!(Bridge::timelocked_txs(unlock_block_number).is_empty());
    assert_eq!(alices_balance(), starting_alices_balance + amount);
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Handled
    );
}

#[test]
fn can_force_mint() {
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .build()
        .execute_with(do_force_mint);
}

fn do_force_mint() {
    let admin = Origin::system(frame_system::RawOrigin::Signed(Default::default()));
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
    let alice = Origin::signed(AccountKeyring::Alice.public());

    let bob_key = AccountKey::from(AccountKeyring::Bob.public());
    let charlie_key = AccountKey::from(AccountKeyring::Charlie.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let charlie = Origin::signed(AccountKeyring::Charlie.public());

    assert_ok!(Balances::top_up_identity_balance(
        alice.clone(),
        alice_did,
        555
    ));

    let controller = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![Signatory::from(bob_key), Signatory::from(charlie_key),],
        1,
    ));

    assert_eq!(MultiSig::ms_signs_required(controller), 1);
    let last_authorization = |account: AccountKey| {
        <Authorizations>::iter_prefix(Signatory::from(account))
            .next()
            .unwrap()
            .auth_id
    };

    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        bob.clone(),
        last_authorization(bob_key)
    ));
    assert_ok!(MultiSig::accept_multisig_signer_as_key(
        charlie.clone(),
        last_authorization(charlie_key)
    ));

    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(bob_key)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(controller, Signatory::from(charlie_key)),
        true
    );

    assert_ok!(Bridge::change_controller(admin.clone(), controller));
    assert_eq!(Bridge::controller(), controller);
    assert_ok!(Bridge::change_bridge_limit(admin.clone(), 1_000_000_000, 1));
    let timelock = 3;
    assert_ok!(Bridge::change_timelock(admin.clone(), timelock));
    let amount = 1_000_000_000_000_000_000_000;
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: AccountKeyring::Alice.public(),
        amount,
        tx_hash: Default::default(),
    };
    let proposal = Box::new(Call::Bridge(bridge::Call::handle_bridge_tx(
        bridge_tx.clone(),
    )));
    let alices_balance = || Balances::total_balance(&AccountKeyring::Alice.public());
    let starting_alices_balance = alices_balance();
    assert_eq!(MultiSig::proposal_ids(&controller, proposal.clone()), None);
    assert_tx_approvals!(controller, 0, 0);
    assert_ok!(Bridge::propose_bridge_tx(bob.clone(), bridge_tx.clone()));
    assert_tx_approvals!(controller, 0, 1);
    let first_block_number = System::block_number();
    let unlock_block_number = first_block_number + timelock + 1;
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    assert_eq!(
        Bridge::timelocked_txs(unlock_block_number),
        vec![bridge_tx.clone()]
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Timelocked
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).execution_block,
        unlock_block_number
    );
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(alices_balance(), starting_alices_balance);
    next_block();
    assert_eq!(System::block_number(), unlock_block_number);
    // Still no issue, rate limit reached
    assert_eq!(alices_balance(), starting_alices_balance);
    assert_ok!(Bridge::force_handle_bridge_tx(
        admin.clone(),
        bridge_tx.clone()
    ));
    // Mint successful after force handle
    assert!(Bridge::timelocked_txs(unlock_block_number).is_empty());
    assert_eq!(alices_balance(), starting_alices_balance + amount);
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).execution_block,
        unlock_block_number
    );
    assert_eq!(
        Bridge::bridge_tx_details(AccountKeyring::Alice.public(), &1).status,
        BridgeTxStatus::Handled
    );
}
