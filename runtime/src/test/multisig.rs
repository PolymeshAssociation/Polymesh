use crate::{
    identity, multisig,
    test::storage::{build_ext, register_keyring_account, Call, TestStorage},
};
use codec::Encode;
use frame_support::{assert_err, assert_ok};
use primitives::{AccountKey, Signatory};
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type MultiSig = multisig::Module<TestStorage>;

type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn create_multisig() {
    build_ext().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), Signatory::from(bob_did)],
            1,
        ));

        assert_eq!(MultiSig::ms_signs_required(musig_address), 1);

        assert_err!(
            MultiSig::create_multisig(alice.clone(), vec![], 10,),
            "No signers provided"
        );

        assert_err!(
            MultiSig::create_multisig(
                alice.clone(),
                vec![Signatory::from(alice_did), Signatory::from(bob_did)],
                0,
            ),
            "Sigs required out of bounds"
        );

        assert_err!(
            MultiSig::create_multisig(
                alice.clone(),
                vec![Signatory::from(alice_did), Signatory::from(bob_did)],
                10,
            ),
            "Sigs required out of bounds"
        );
    });
}

#[test]
fn join_multisig() {
    build_ext().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let bob_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), bob_signer],
            1,
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), Signatory::from(alice_did))),
            false
        );

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            false
        );

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            Identity::last_authorization(Signatory::from(alice_did))
        ));

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            Identity::last_authorization(bob_signer)
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), Signatory::from(alice_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            true
        );
    });
}

#[test]
fn change_multisig_sigs_required() {
    build_ext().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let bob_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), bob_signer],
            2,
        ));

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            Identity::last_authorization(Signatory::from(alice_did))
        ));

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            Identity::last_authorization(bob_signer)
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), Signatory::from(alice_did))),
            true
        );

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            true
        );

        let call = Box::new(Call::MultiSig(multisig::Call::change_sigs_required(1)));

        assert_ok!(MultiSig::create_proposal_as_key(
            bob.clone(),
            musig_address.clone(),
            call
        ));

        assert_eq!(MultiSig::ms_signs_required(musig_address.clone()), 2);

        assert_ok!(MultiSig::approve_as_identity(
            alice.clone(),
            musig_address.clone(),
            0
        ));

        assert_eq!(MultiSig::ms_signs_required(musig_address), 1);
    });
}

#[test]
fn remove_multisig_signer() {
    build_ext().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let bob_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), bob_signer],
            1,
        ));

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            Identity::last_authorization(Signatory::from(alice_did))
        ));

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            Identity::last_authorization(bob_signer)
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), Signatory::from(alice_did))),
            true
        );

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            true
        );

        let call = Box::new(Call::MultiSig(multisig::Call::remove_multisig_signer(
            bob_signer,
        )));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), Signatory::from(alice_did))),
            true
        );

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            false
        );
    });
}

#[test]
fn add_multisig_signer() {
    build_ext().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let bob_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did)],
            1,
        ));

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            Identity::last_authorization(Signatory::from(alice_did))
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), Signatory::from(alice_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            false
        );

        let call = Box::new(Call::MultiSig(multisig::Call::add_multisig_signer(
            bob_signer,
        )));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), Signatory::from(alice_did))),
            true
        );

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            false
        );

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            Identity::last_authorization(bob_signer)
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            true
        );
    });
}
