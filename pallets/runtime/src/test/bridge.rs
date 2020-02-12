use crate::test::storage::{
    register_keyring_account, register_keyring_account_with_balance, Call, TestStorage,
};
use crate::test::ExtBuilder;
use crate::{
    bridge::{self, BridgeTx, IssueRecipient},
    multisig,
};
use frame_support::{assert_err, assert_ok, StorageDoubleMap};
use polymesh_primitives::{IdentityId, Signatory};
use polymesh_runtime_balances as balances;
use polymesh_runtime_identity as identity;
use test_client::AccountKeyring;

type Bridge = bridge::Module<TestStorage>;
type Error = bridge::Error<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Authorizations = identity::Authorizations<TestStorage>;
type MultiSig = multisig::Module<TestStorage>;
type MultiSigError = multisig::Error<TestStorage>;
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
        let bridge_signers = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did)
            ],
            2,
        ));
        assert_eq!(MultiSig::ms_signs_required(bridge_signers), 2);
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
            MultiSig::ms_signers((bridge_signers, Signatory::from(alice_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((bridge_signers, Signatory::from(bob_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((bridge_signers, Signatory::from(charlie_did))),
            true
        );
        assert_ok!(Bridge::change_bridge_signers(
            Origin::system(frame_system::RawOrigin::Root),
            bridge_signers
        ));
        assert_eq!(Bridge::bridge_signers(), bridge_signers);
        let value = 1_000_000_000_000_000_000_000;
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Identity(bob_did),
            value,
            tx_hash: Default::default(),
        };
        assert_eq!(Bridge::bridge_signers(), bridge_signers);
        let proposal_id = || Bridge::bridge_tx_proposals(&bridge_tx);
        assert_eq!(proposal_id(), None);
        assert_tx_approvals!(bridge_signers, 0, 0);
        assert_tx_approvals!(bridge_signers, 1, 0);
        assert_ok!(Bridge::propose_bridge_tx(bob, bridge_tx.clone()));
        assert_tx_approvals!(bridge_signers, 0, 1);
        assert_tx_approvals!(bridge_signers, 1, 0);
        let bobs_balance = Balances::identity_balance(bob_did);
        assert_eq!(proposal_id(), Some(0));
        assert_ok!(Bridge::propose_bridge_tx(charlie, bridge_tx.clone()));
        assert_tx_approvals!(bridge_signers, 0, 2);
        assert_tx_approvals!(bridge_signers, 1, 0);
        assert_eq!(proposal_id(), Some(0));
        let new_bobs_balance = Balances::identity_balance(bob_did);
        assert_eq!(new_bobs_balance, bobs_balance + value);
    });
}

#[test]
fn can_change_bridge_signers() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did =
            register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
        let bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 1_000).unwrap();
        let charlie_did =
            register_keyring_account_with_balance(AccountKeyring::Charlie, 1_000).unwrap();
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
        let bridge_signers = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did)
            ],
            2,
        ));
        assert_eq!(MultiSig::ms_signs_required(bridge_signers), 2);
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
            MultiSig::ms_signers((bridge_signers, Signatory::from(alice_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((bridge_signers, Signatory::from(bob_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((bridge_signers, Signatory::from(charlie_did))),
            true
        );
        assert_ok!(Bridge::change_bridge_signers(
            Origin::system(frame_system::RawOrigin::Root),
            bridge_signers
        ));
        assert_eq!(Bridge::bridge_signers(), bridge_signers);
        let new_bridge_signers = MultiSig::get_next_multisig_address(AccountKeyring::Bob.public());
        assert_ok!(MultiSig::create_multisig(
            bob.clone(),
            vec![Signatory::from(bob_did), Signatory::from(charlie_did)],
            1,
        ));
        assert_eq!(MultiSig::ms_signs_required(new_bridge_signers), 1);
        let call = Box::new(Call::Bridge(bridge::Call::handle_bridge_signers(
            new_bridge_signers,
        )));
        assert_tx_approvals!(bridge_signers, 0, 0);
        assert_tx_approvals!(new_bridge_signers, 0, 0);
        assert!(MultiSig::create_proposal(bridge_signers, call, Signatory::from(bob_did)).is_ok());
        assert_tx_approvals!(bridge_signers, 0, 1);
        assert_tx_approvals!(new_bridge_signers, 0, 0);
        assert_eq!(Bridge::bridge_signers(), bridge_signers);
        assert_err!(
            MultiSig::approve_as_identity(dave.clone(), bridge_signers, 0),
            MultiSigError::IdentityMissing
        );
        assert_tx_approvals!(bridge_signers, 0, 1);
        assert_tx_approvals!(new_bridge_signers, 0, 0);
        assert_ok!(MultiSig::approve_as_identity(
            charlie.clone(),
            bridge_signers,
            0
        ));
        assert_tx_approvals!(bridge_signers, 0, 2);
        assert_tx_approvals!(new_bridge_signers, 0, 0);
        assert_eq!(Bridge::bridge_signers(), new_bridge_signers);
    });
}

#[test]
fn cannot_propose_without_bridge_signers() {
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
            Error::BridgeSignersNotSet,
        );
    });
}

#[test]
fn cannot_call_bridge_callback_extrinsics() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_account = AccountKeyring::Alice.public();
        let alice = Origin::signed(alice_account);
        assert_err!(
            Bridge::handle_bridge_signers(alice.clone(), alice_account),
            Error::BadCaller,
        );
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Account(alice_account),
            value: 1_000_000,
            tx_hash: Default::default(),
        };
        assert_err!(Bridge::handle_bridge_tx(alice, bridge_tx), Error::BadCaller);
    });
}
