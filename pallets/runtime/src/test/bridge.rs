use crate::test::storage::{
    register_keyring_account, register_keyring_account_with_balance, TestStorage,
};
use crate::test::ExtBuilder;
use crate::{
    bridge::{self, BridgeTx, IssueRecipient, ValidatorSet},
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
        let validator_account = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did)
            ],
            2,
        ));
        assert_eq!(MultiSig::ms_signs_required(validator_account), 2);
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
            MultiSig::ms_signers((validator_account, Signatory::from(alice_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((validator_account, Signatory::from(bob_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((validator_account, Signatory::from(charlie_did))),
            true
        );
        assert_ok!(Bridge::change_validator_set_account(
            Origin::system(frame_system::RawOrigin::Root),
            validator_account
        ));
        assert_eq!(Bridge::validators(), validator_account);
        let value = 1_000_000_000_000_000_000_000;
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Identity(bob_did),
            value,
            tx_hash: Default::default(),
        };
        assert_eq!(Bridge::validators(), validator_account);
        let proposal_id = || Bridge::bridge_tx_proposals(&bridge_tx);
        assert_eq!(proposal_id(), None);
        assert_tx_approvals!(validator_account, 0, 0);
        assert_tx_approvals!(validator_account, 1, 0);
        assert_ok!(Bridge::propose_bridge_tx(bob, bridge_tx.clone()));
        assert_tx_approvals!(validator_account, 0, 1);
        assert_tx_approvals!(validator_account, 1, 0);
        let bobs_balance = Balances::identity_balance(bob_did);
        assert_eq!(proposal_id(), Some(0));
        assert_ok!(Bridge::propose_bridge_tx(charlie, bridge_tx.clone()));
        assert_tx_approvals!(validator_account, 0, 2);
        assert_tx_approvals!(validator_account, 1, 0);
        assert_eq!(proposal_id(), Some(0));
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
        let validator_account = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did)
            ],
            2,
        ));
        assert_eq!(MultiSig::ms_signs_required(validator_account), 2);
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
            MultiSig::ms_signers((validator_account, Signatory::from(alice_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((validator_account, Signatory::from(bob_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((validator_account, Signatory::from(charlie_did))),
            true
        );
        assert_ok!(Bridge::change_validator_set_account(
            Origin::system(frame_system::RawOrigin::Root),
            validator_account
        ));
        assert_eq!(Bridge::validators(), validator_account);
        let new_validator_account = MultiSig::get_next_multisig_address(validator_account);
        let new_validator_set = ValidatorSet {
            nonce: 1,
            signers: vec![Signatory::from(bob_did), Signatory::from(charlie_did)],
            signatures_required: 1,
        };
        assert_tx_approvals!(validator_account, 0, 0);
        assert_tx_approvals!(new_validator_account, 0, 0);
        assert_ok!(Bridge::propose_validator_set(
            bob.clone(),
            new_validator_set.clone()
        ));
        assert_tx_approvals!(validator_account, 0, 1);
        assert_tx_approvals!(new_validator_account, 0, 0);
        assert_eq!(Bridge::validators(), validator_account);
        assert_err!(
            Bridge::propose_validator_set(dave.clone(), new_validator_set.clone()),
            Error::IdentityMissing
        );
        assert_tx_approvals!(validator_account, 0, 1);
        assert_tx_approvals!(new_validator_account, 0, 0);
        assert_ok!(Bridge::propose_validator_set(
            charlie.clone(),
            new_validator_set
        ));
        assert_tx_approvals!(validator_account, 0, 2);
        assert_tx_approvals!(new_validator_account, 0, 0);
        assert_eq!(Bridge::validators(), new_validator_account);
        assert_eq!(MultiSig::ms_signs_required(new_validator_account), 1);
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
            Error::ValidatorsNotSet,
        );
    });
}

#[test]
fn cannot_call_validator_callback_extrinsics() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did =
            register_keyring_account_with_balance(AccountKeyring::Alice, 1_000).unwrap();
        let alice_account = AccountKeyring::Alice.public();
        let alice = Origin::signed(alice_account);
        assert_err!(
            Bridge::handle_validator_set(
                alice.clone(),
                ValidatorSet {
                    nonce: 1,
                    signers: vec![Signatory::from(alice_did)],
                    signatures_required: 1,
                }
            ),
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
