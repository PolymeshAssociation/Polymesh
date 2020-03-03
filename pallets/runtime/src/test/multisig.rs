use crate::{
    multisig,
    test::{
        storage::{register_keyring_account, Call, TestStorage},
        ExtBuilder,
    },
};

use polymesh_primitives::{AccountKey, Signatory};
use polymesh_runtime_common::Context;
use polymesh_runtime_identity as identity;

use codec::Encode;
use frame_support::{assert_err, assert_ok, StorageDoubleMap};
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type MultiSig = multisig::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Error = multisig::Error<TestStorage>;

#[test]
fn create_multisig() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), Signatory::from(bob_did)],
            1,
        ));

        assert!(Identity::_register_did(musig_address.clone(), vec![],).is_ok());

        assert_eq!(MultiSig::ms_signs_required(musig_address), 1);

        assert_err!(
            MultiSig::create_multisig(alice.clone(), vec![], 10,),
            Error::NoSigners
        );

        assert_err!(
            MultiSig::create_multisig(
                alice.clone(),
                vec![Signatory::from(alice_did), Signatory::from(bob_did)],
                0,
            ),
            Error::RequiredSignaturesOutOfBounds
        );

        assert_err!(
            MultiSig::create_multisig(
                alice.clone(),
                vec![Signatory::from(alice_did), Signatory::from(bob_did)],
                10,
            ),
            Error::RequiredSignaturesOutOfBounds
        );
    });
}

#[test]
fn join_multisig() {
    ExtBuilder::default().build().execute_with(|| {
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

        assert!(Identity::_register_did(musig_address.clone(), vec![],).is_ok());

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            false
        );

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
            false
        );

        let alice_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(alice_did))
                .next()
                .unwrap()
                .auth_id;

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        let bob_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer)
            .next()
            .unwrap()
            .auth_id;

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
            true
        );
    });
}

#[test]
fn change_multisig_sigs_required() {
    ExtBuilder::default().build().execute_with(|| {
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

        assert!(Identity::_register_did(musig_address.clone(), vec![],).is_ok());

        let alice_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(alice_did))
                .next()
                .unwrap()
                .auth_id;

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        let bob_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer)
            .next()
            .unwrap()
            .auth_id;

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
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
fn create_or_approve_change_multisig_sigs_required() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
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
        assert!(Identity::_register_did(musig_address.clone(), vec![]).is_ok());
        let alice_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(alice_did))
                .next()
                .unwrap()
                .auth_id;
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));
        let bob_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer)
            .next()
            .unwrap()
            .auth_id;
        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));
        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
            true
        );
        let call = Box::new(Call::MultiSig(multisig::Call::change_sigs_required(1)));
        assert_ok!(MultiSig::create_or_approve_proposal_as_key(
            bob.clone(),
            musig_address.clone(),
            call.clone()
        ));
        assert_eq!(MultiSig::ms_signs_required(musig_address.clone()), 2);
        assert_ok!(MultiSig::create_or_approve_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call
        ));
        assert_eq!(MultiSig::ms_signs_required(musig_address), 1);
    });
}

#[test]
fn remove_multisig_signer() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let alice_signer = Signatory::from(alice_did);
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let bob_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![alice_signer, bob_signer],
            1,
        ));

        assert!(Identity::_register_did(musig_address.clone(), vec![],).is_ok());

        assert_eq!(MultiSig::number_of_signers(musig_address.clone()), 0);

        let alice_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(alice_signer)
            .next()
            .unwrap()
            .auth_id;

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        assert_eq!(MultiSig::number_of_signers(musig_address.clone()), 1);

        let bob_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer)
            .next()
            .unwrap()
            .auth_id;

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(MultiSig::number_of_signers(musig_address.clone()), 2);

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), alice_signer),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
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

        assert_eq!(MultiSig::number_of_signers(musig_address.clone()), 1);

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), alice_signer),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
            false
        );

        Context::set_current_identity::<Identity>(None);

        let remove_alice = Box::new(Call::MultiSig(multisig::Call::remove_multisig_signer(
            alice_signer,
        )));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            remove_alice
        ));

        // Alice not removed since that would've broken the multi sig.
        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), alice_signer),
            true
        );
    });
}

#[test]
fn add_multisig_signer() {
    ExtBuilder::default().build().execute_with(|| {
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

        assert!(Identity::_register_did(musig_address.clone(), vec![],).is_ok());

        let alice_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(alice_did))
                .next()
                .unwrap()
                .auth_id;

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
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
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
            false
        );

        let bob_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer)
            .next()
            .unwrap()
            .auth_id;

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
            true
        );
    });
}
