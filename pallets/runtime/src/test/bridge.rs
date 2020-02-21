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
type Identity = identity::Module<TestStorage>;
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
    ExtBuilder::default()
        .existential_deposit(1_000)
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Ferdie.public()])
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
    let relayers = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
    assert_ok!(MultiSig::create_multisig(
        alice.clone(),
        vec![
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            Signatory::from(charlie_did)
        ],
        2,
    ));
    assert_eq!(
        Identity::_register_did(relayers.clone(), vec![]).is_ok(),
        true
    );
    assert_eq!(MultiSig::ms_signs_required(relayers), 2);
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
        MultiSig::ms_signers(relayers, Signatory::from(alice_did)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(relayers, Signatory::from(bob_did)),
        true
    );
    assert_eq!(
        MultiSig::ms_signers(relayers, Signatory::from(charlie_did)),
        true
    );
    assert_ok!(Bridge::change_relayers(
        Origin::system(frame_system::RawOrigin::Root),
        relayers
    ));
    assert_eq!(Bridge::relayers(), relayers);
    let amount = 1_000_000_000_000_000_000_000;
    let bridge_tx = BridgeTx {
        nonce: 1,
        recipient: IssueRecipient::Identity(bob_did),
        amount,
        tx_hash: Default::default(),
    };
    assert_eq!(Bridge::relayers(), relayers);
    assert_eq!(Bridge::bridge_tx_proposals(&bridge_tx), None);
    assert_tx_approvals!(relayers, 0, 0);
    assert_tx_approvals!(relayers, 1, 0);

    assert_ok!(Bridge::propose_bridge_tx(bob.clone(), bridge_tx.clone()));
    assert_tx_approvals!(relayers, 0, 1);
    assert_tx_approvals!(relayers, 1, 0);
    let bobs_balance = Balances::identity_balance(bob_did);

    assert_eq!(Bridge::bridge_tx_proposals(&bridge_tx), Some(0));
    assert_ok!(Bridge::propose_bridge_tx(charlie, bridge_tx.clone()));
    assert_eq!(MultiSig::tx_approvals(&(relayers, 0)), 2);
    assert_eq!(MultiSig::tx_approvals(&(relayers, 1)), 0);

    assert_eq!(Bridge::bridge_tx_proposals(&bridge_tx), Some(0));
    assert_ok!(Bridge::finalize_pending(bob, bob_did));

    let new_bobs_balance = Balances::identity_balance(bob_did);
    assert_eq!(new_bobs_balance, bobs_balance + amount);
}

#[test]
fn can_change_relayers() {
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
        let relayers = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did)
            ],
            2,
        ));
        assert_eq!(
            Identity::_register_did(relayers.clone(), vec![]).is_ok(),
            true
        );
        assert_eq!(MultiSig::ms_signs_required(relayers), 2);
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
            MultiSig::ms_signers(relayers, Signatory::from(alice_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(relayers, Signatory::from(bob_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(relayers, Signatory::from(charlie_did)),
            true
        );
        assert_ok!(Bridge::change_relayers(
            Origin::system(frame_system::RawOrigin::Root),
            relayers
        ));
        assert_eq!(Bridge::relayers(), relayers);
        let new_relayers = MultiSig::get_next_multisig_address(AccountKeyring::Bob.public());
        assert_ok!(MultiSig::create_multisig(
            bob.clone(),
            vec![Signatory::from(bob_did), Signatory::from(charlie_did)],
            1,
        ));
        assert_eq!(
            Identity::_register_did(new_relayers.clone(), vec![]).is_ok(),
            true
        );
        assert_eq!(MultiSig::ms_signs_required(new_relayers), 1);
        let call = Box::new(Call::Bridge(bridge::Call::handle_relayers(new_relayers)));
        assert_tx_approvals!(relayers, 0, 0);
        assert_tx_approvals!(new_relayers, 0, 0);
        assert!(MultiSig::create_proposal(relayers, Signatory::from(bob_did), call).is_ok());
        assert_tx_approvals!(relayers, 0, 1);
        assert_tx_approvals!(new_relayers, 0, 0);
        assert_eq!(Bridge::relayers(), relayers);
        assert_err!(
            MultiSig::approve_as_identity(dave.clone(), relayers, 0),
            "Current identity is none and key is not linked to any identity"
        );
        assert_tx_approvals!(relayers, 0, 1);
        assert_tx_approvals!(new_relayers, 0, 0);
        assert_ok!(MultiSig::approve_as_identity(charlie.clone(), relayers, 0));
        assert_tx_approvals!(relayers, 0, 2);
        assert_tx_approvals!(new_relayers, 0, 0);
        assert_eq!(Bridge::relayers(), new_relayers);
    });
}

#[test]
fn cannot_propose_without_relayers() {
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
            Error::RelayersNotSet,
        );
    });
}

#[test]
fn cannot_call_bridge_callback_extrinsics() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_account = AccountKeyring::Alice.public();
        let alice = Origin::signed(alice_account);
        assert_err!(
            Bridge::handle_relayers(alice.clone(), alice_account),
            Error::BadCaller,
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
