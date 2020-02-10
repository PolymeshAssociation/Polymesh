use crate::test::storage::{
    register_keyring_account, register_keyring_account_with_balance, Call, TestStorage,
};
use crate::test::ExtBuilder;
use crate::{
    bridge::{self, BridgeTx, IssueRecipient},
    multisig,
};
use polymesh_runtime_balances as balances;
use polymesh_runtime_identity as identity;

use frame_support::{assert_err, assert_ok};
use polymesh_primitives::Signatory;
use test_client::AccountKeyring;

type Bridge = bridge::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Identity = identity::Module<TestStorage>;
type MultiSig = multisig::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

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
        let validator_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did)
            ],
            2,
        ));
        assert_eq!(MultiSig::ms_signs_required(validator_address), 2);
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            Identity::last_authorization(Signatory::from(alice_did))
        ));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            bob.clone(),
            Identity::last_authorization(Signatory::from(bob_did))
        ));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            charlie.clone(),
            Identity::last_authorization(Signatory::from(charlie_did))
        ));
        assert_eq!(
            MultiSig::ms_signers((validator_address, Signatory::from(alice_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((validator_address, Signatory::from(bob_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((validator_address, Signatory::from(charlie_did))),
            true
        );
        assert_ok!(Bridge::propose_change_validators(
            alice.clone(),
            validator_address
        ));
        assert_eq!(Bridge::validators(), validator_address);
        let value = 1_000_000_000_000_000_000_000;
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Identity(bob_did),
            value,
            tx_hash: Default::default(),
        };
        assert_eq!(Bridge::validators(), validator_address);
        let proposal_id = || Bridge::bridge_tx_proposals(bridge_tx.clone());
        assert_eq!(proposal_id(), None);
        assert_tx_approvals!(validator_address, 0, 0);
        assert_tx_approvals!(validator_address, 1, 0);
        assert_ok!(Bridge::propose_bridge_tx(bob, bridge_tx.clone()));
        assert_tx_approvals!(validator_address, 0, 1);
        assert_tx_approvals!(validator_address, 1, 0);
        let bobs_balance = Balances::identity_balance(bob_did);
        assert_eq!(proposal_id(), Some(0));
        assert_ok!(Bridge::propose_bridge_tx(charlie, bridge_tx.clone()));
        assert_tx_approvals!(validator_address, 0, 2);
        assert_tx_approvals!(validator_address, 1, 0);
        assert_eq!(proposal_id(), None);
        let new_bobs_balance = Balances::identity_balance(bob_did);
        assert_eq!(new_bobs_balance, bobs_balance + value);
    });
}

#[test]
fn can_change_validators() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did =
            register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
        let bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();
        let charlie_did =
            register_keyring_account_with_balance(AccountKeyring::Charlie, 1_000).unwrap();
        let dave_did = register_keyring_account_with_balance(AccountKeyring::Dave, 1_000).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let charlie = Origin::signed(AccountKeyring::Charlie.public());
        let dave = Origin::signed(AccountKeyring::Dave.public());
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
        let validator_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did)
            ],
            2,
        ));
        assert_eq!(MultiSig::ms_signs_required(validator_address), 2);
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            Identity::last_authorization(Signatory::from(alice_did))
        ));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            bob.clone(),
            Identity::last_authorization(Signatory::from(bob_did))
        ));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            charlie.clone(),
            Identity::last_authorization(Signatory::from(charlie_did))
        ));
        assert_eq!(
            MultiSig::ms_signers((validator_address, Signatory::from(alice_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((validator_address, Signatory::from(bob_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((validator_address, Signatory::from(charlie_did))),
            true
        );
        assert_ok!(Bridge::propose_change_validators(
            alice.clone(),
            validator_address
        ));
        assert_eq!(Bridge::validators(), validator_address);
        let new_validator_address =
            MultiSig::get_next_multisig_address(AccountKeyring::Bob.public());
        assert_ok!(MultiSig::create_multisig(
            bob.clone(),
            vec![Signatory::from(bob_did), Signatory::from(charlie_did)],
            1,
        ));
        assert_eq!(MultiSig::ms_signs_required(new_validator_address), 1);
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            bob.clone(),
            Identity::last_authorization(Signatory::from(bob_did))
        ));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            charlie.clone(),
            Identity::last_authorization(Signatory::from(charlie_did))
        ));
        assert_eq!(
            MultiSig::ms_signers((new_validator_address, Signatory::from(alice_did))),
            false
        );
        assert_eq!(
            MultiSig::ms_signers((new_validator_address, Signatory::from(bob_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((new_validator_address, Signatory::from(charlie_did))),
            true
        );
        assert_tx_approvals!(validator_address, 0, 0);
        assert_tx_approvals!(new_validator_address, 0, 0);
        assert_ok!(Bridge::propose_change_validators(
            bob.clone(),
            new_validator_address
        ));
        assert_tx_approvals!(validator_address, 0, 1);
        assert_tx_approvals!(new_validator_address, 0, 0);
        assert_eq!(Bridge::validators(), validator_address);
        assert_err!(
            Bridge::propose_change_validators(dave.clone(), new_validator_address),
            "not a signer to approve as identity"
        );
        assert_tx_approvals!(validator_address, 0, 1);
        assert_tx_approvals!(new_validator_address, 0, 0);
        assert_ok!(Bridge::propose_change_validators(
            charlie.clone(),
            new_validator_address
        ));
        assert_tx_approvals!(validator_address, 0, 2);
        assert_tx_approvals!(new_validator_address, 0, 0);
        assert_eq!(Bridge::validators(), new_validator_address);
    });
}

#[test]
fn cannot_propose_without_validators() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let value = 1_000_000;
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Identity(alice_did),
            value,
            tx_hash: Default::default(),
        };
        assert_err!(
            Bridge::propose_bridge_tx(alice, bridge_tx),
            "bridge validators not set"
        );
    });
}

#[test]
fn cannot_call_validator_callback_extrinsics() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_account = AccountKeyring::Alice.public();
        let alice = Origin::signed(alice_account);
        assert_err!(
            Bridge::handle_change_validators(alice.clone(), alice_account),
            "should be called by the validator set account"
        );
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Account(alice_account),
            value: 1_000_000,
            tx_hash: Default::default(),
        };
        assert_err!(
            Bridge::handle_bridge_tx(alice, bridge_tx),
            "should be called by the validator set account"
        );
    });
}
