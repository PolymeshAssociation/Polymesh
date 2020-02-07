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
        let charlie = Origin::signed(AccountKeyring::Bob.public());
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
        assert_ok!(Bridge::propose_change_validators(alice, validator_address));
        let value = 1_000_000;
        let bridge_tx = BridgeTx {
            nonce: 1,
            recipient: IssueRecipient::Identity(bob_did),
            value,
            tx_hash: Default::default(),
        };
        assert_ok!(Bridge::propose_bridge_tx(bob, bridge_tx.clone()));
        assert_ok!(Bridge::propose_bridge_tx(charlie, bridge_tx));
        let new_bobs_balance = Balances::identity_balance(bob_did);
        assert_eq!(new_bobs_balance, bobs_balance + value);
    });
}
