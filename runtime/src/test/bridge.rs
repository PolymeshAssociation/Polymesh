use crate::{
    balances,
    bridge::{self, BridgeTx, IssueRecipient, PendingTx},
    identity, multisig,
    test::storage::{build_ext, register_keyring_account, Call, TestStorage},
};
use frame_support::{assert_err, assert_ok};
use primitives::Signatory;
use test_client::AccountKeyring;

type Bridge = bridge::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Identity = identity::Module<TestStorage>;
type MultiSig = multisig::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn can_issue_to_identity() {
    build_ext().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let charlie = Origin::signed(AccountKeyring::Charlie.public());
        let validator_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        let bobs_balance = Balances::identity_balance(bob_did);
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
        assert_ok!(Bridge::propose_change_validators(alice, validator_address));
        assert_eq!(Bridge::validators(), validator_address);
        let value = 1_000_000;
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Identity(bob_did),
            value,
            tx_hash: Default::default(),
        };
        assert_ok!(Bridge::propose_bridge_tx(bob, bridge_tx.clone()));
        assert_ok!(Bridge::propose_bridge_tx(charlie, bridge_tx.clone()));
        let new_bobs_balance = Balances::identity_balance(bob_did);
        assert_eq!(new_bobs_balance, bobs_balance + value);
    });
}

#[test]
fn cannot_propose_without_validators() {
    build_ext().execute_with(|| {
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
    build_ext().execute_with(|| {
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
