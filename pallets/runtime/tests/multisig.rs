mod common;
use common::{
    ext_builder::PROTOCOL_OP_BASE_FEE,
    storage::{register_keyring_account, Call, TestStorage},
    ExtBuilder,
};

use polymesh_primitives::{AccountKey, Signatory};
use polymesh_runtime::multisig;
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::Context;
use polymesh_runtime_identity as identity;

use codec::Encode;
use frame_support::{assert_err, assert_ok, StorageDoubleMap};
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Balances = balances::Module<TestStorage>;
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

        assert_eq!(MultiSig::ms_signs_required(musig_address), 1);
        assert_eq!(MultiSig::ms_creator(musig_address), alice_did);

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

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), bob_signer],
            1,
        ));

        let bob_auth_id2 = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer)
            .next()
            .unwrap()
            .auth_id;

        assert_err!(
            MultiSig::accept_multisig_signer_as_key(bob.clone(), bob_auth_id2),
            Error::SignerAlreadyLinked
        );
    });
}

#[test]
fn change_multisig_sigs_required() {
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
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let bob_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap());
        let charlie = Origin::signed(AccountKeyring::Charlie.public());
        let charlie_signer = Signatory::from(
            AccountKey::try_from(AccountKeyring::Charlie.public().encode()).unwrap(),
        );

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did)],
            1,
        ));

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

        let call2 = Box::new(Call::MultiSig(multisig::Call::add_multisig_signer(
            charlie_signer,
        )));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call2
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

        let charlie_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(charlie_signer)
            .next()
            .unwrap()
            .auth_id;

        let root = Origin::system(frame_system::RawOrigin::Root);

        assert!(Identity::change_cdd_requirement_for_mk_rotation(root.clone(), true).is_ok());
        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            charlie.clone(),
            charlie_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), charlie_signer),
            true
        );

        assert!(Identity::_register_did(musig_address.clone(), vec![], None).is_ok());

        assert_err!(
            MultiSig::accept_multisig_signer_as_key(bob.clone(), bob_auth_id),
            Error::ChangeNotAllowed
        );

        assert!(Identity::change_cdd_requirement_for_mk_rotation(root.clone(), false).is_ok());

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

#[test]
fn should_change_all_signers_and_sigs_required() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());

        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let bob = Origin::signed(AccountKeyring::Bob.public());

        let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let charlie = Origin::signed(AccountKeyring::Charlie.public());

        let dave_did = register_keyring_account(AccountKeyring::Dave).unwrap();
        let dave = Origin::signed(AccountKeyring::Dave.public());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), Signatory::from(bob_did)],
            1,
        ));

        let alice_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(alice_did))
                .next()
                .unwrap()
                .auth_id;

        let bob_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(bob_did))
                .next()
                .unwrap()
                .auth_id;

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            true
        );

        let call = Box::new(Call::MultiSig(
            multisig::Call::change_all_signers_and_sigs_required(
                vec![Signatory::from(charlie_did), Signatory::from(dave_did)],
                2,
            ),
        ));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            false
        );

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(bob_did)),
            false
        );

        let charlie_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(charlie_did))
                .next()
                .unwrap()
                .auth_id;

        let dave_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(dave_did))
                .next()
                .unwrap()
                .auth_id;

        let _by =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(charlie_did))
                .next()
                .unwrap()
                .authorized_by;

        Context::set_current_identity::<Identity>(None);
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            charlie,
            charlie_auth_id
        ));

        Context::set_current_identity::<Identity>(None);
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            dave,
            dave_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(dave_did)),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(charlie_did)),
            true
        );

        assert_eq!(MultiSig::number_of_signers(musig_address.clone()), 2);
    })
}

#[test]
fn make_multisig_master() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did)],
            1,
        ));

        assert_eq!(
            Identity::did_records(alice_did).master_key,
            AccountKey::try_from(AccountKeyring::Alice.public().encode()).unwrap()
        );

        assert_err!(
            MultiSig::make_multisig_master(bob.clone(), musig_address.clone(), None),
            Error::IdentityNotCreator
        );

        assert_ok!(MultiSig::make_multisig_master(
            alice.clone(),
            musig_address.clone(),
            None
        ));

        assert_eq!(
            Identity::did_records(alice_did).master_key,
            AccountKey::try_from(musig_address.encode()).unwrap()
        );
    });
}

#[test]
fn make_multisig_signer() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let bob = Origin::signed(AccountKeyring::Bob.public());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());
        let ms_key = AccountKey::try_from(musig_address.encode()).unwrap();

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did)],
            1,
        ));

        let signing_items = Identity::did_records(alice_did).signing_items;
        assert!(signing_items.iter().find(|si| **si == ms_key).is_none());

        assert_err!(
            MultiSig::make_multisig_signer(bob.clone(), musig_address.clone()),
            Error::IdentityNotCreator
        );

        assert_ok!(Balances::top_up_identity_balance(
            alice.clone(),
            alice_did,
            PROTOCOL_OP_BASE_FEE
        ));
        assert_ok!(MultiSig::make_multisig_signer(
            alice.clone(),
            musig_address.clone(),
        ));

        let signing_items2 = Identity::did_records(alice_did).signing_items;
        assert!(signing_items2.iter().find(|si| **si == ms_key).is_some());
    });
}

#[test]
fn remove_multisig_signers_via_creator() {
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

        assert_err!(
            MultiSig::remove_multisig_signers_via_creator(
                bob.clone(),
                musig_address.clone(),
                vec![bob_signer]
            ),
            Error::IdentityNotCreator
        );

        assert_ok!(MultiSig::remove_multisig_signers_via_creator(
            alice.clone(),
            musig_address.clone(),
            vec![bob_signer]
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

        assert_err!(
            MultiSig::remove_multisig_signers_via_creator(
                alice.clone(),
                musig_address.clone(),
                vec![alice_signer]
            ),
            Error::NotEnoughSigners
        );

        // Alice not removed since that would've broken the multi sig.
        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), alice_signer),
            true
        );
    });
}

#[test]
fn add_multisig_signers_via_creator() {
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

        assert_err!(
            MultiSig::add_multisig_signers_via_creator(
                bob.clone(),
                musig_address.clone(),
                vec![bob_signer]
            ),
            Error::IdentityNotCreator
        );

        assert_ok!(MultiSig::add_multisig_signers_via_creator(
            alice.clone(),
            musig_address.clone(),
            vec![bob_signer]
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
