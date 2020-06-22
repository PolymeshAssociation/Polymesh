use super::{
    ext_builder::PROTOCOL_OP_BASE_FEE,
    storage::{register_keyring_account, Call, TestStorage},
    ExtBuilder,
};

use pallet_balances as balances;
use pallet_identity as identity;
use pallet_multisig as multisig;
use polymesh_common_utilities::Context;
use polymesh_primitives::{AccountKey, IdentityId, Signatory};

use codec::Encode;
use frame_support::{assert_err, assert_ok, StorageDoubleMap};
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Balances = balances::Module<TestStorage>;
type Identity = identity::Module<TestStorage>;
type MultiSig = multisig::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;
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
        assert_eq!(
            Identity::get_identity(&AccountKey::try_from(musig_address.encode()).unwrap()),
            Some(alice_did)
        );

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
        Context::set_current_identity::<Identity>(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        let bob_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer)
            .next()
            .unwrap()
            .auth_id;
        Context::set_current_identity::<Identity>(Some(alice_did));
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
        assert_eq!(
            Identity::get_identity(
                &AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap()
            ),
            Some(alice_did)
        );

        Context::set_current_identity::<Identity>(None);
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), bob_signer],
            1,
        ));

        let bob_auth_id2 = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer)
            .next()
            .unwrap()
            .auth_id;
        Context::set_current_identity::<Identity>(Some(alice_did));
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
        Context::set_current_identity::<Identity>(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        let bob_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer)
            .next()
            .unwrap()
            .auth_id;

        Context::set_current_identity::<Identity>(Some(alice_did));
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

        Context::set_current_identity::<Identity>(Some(alice_did));
        assert_ok!(MultiSig::create_proposal_as_key(
            bob.clone(),
            musig_address.clone(),
            call,
            None,
            false
        ));

        assert_eq!(MultiSig::ms_signs_required(musig_address.clone()), 2);

        Context::set_current_identity::<Identity>(Some(alice_did));
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

        Context::set_current_identity::<Identity>(Some(alice_did));
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
            call.clone(),
            None,
            false
        ));
        assert_eq!(MultiSig::ms_signs_required(musig_address.clone()), 2);
        assert_ok!(MultiSig::create_or_approve_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call,
            None,
            false
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

        Context::set_current_identity::<Identity>(Some(alice_did));
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

        assert_eq!(
            Identity::get_identity(
                &AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap()
            ),
            Some(alice_did)
        );

        let call = Box::new(Call::MultiSig(multisig::Call::remove_multisig_signer(
            bob_signer,
        )));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call,
            None,
            false
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

        assert_eq!(
            Identity::get_identity(
                &AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap()
            ),
            None
        );

        Context::set_current_identity::<Identity>(None);

        let remove_alice = Box::new(Call::MultiSig(multisig::Call::remove_multisig_signer(
            alice_signer,
        )));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            remove_alice,
            None,
            false
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

        Context::set_current_identity::<Identity>(Some(alice_did));
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
            call,
            None,
            false
        ));

        let call2 = Box::new(Call::MultiSig(multisig::Call::add_multisig_signer(
            charlie_signer,
        )));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call2,
            None,
            false
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

        assert_ok!(MultiSig::make_multisig_master(
            alice.clone(),
            musig_address.clone(),
            None
        ));

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            charlie.clone(),
            charlie_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), charlie_signer),
            true
        );
        Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
        assert!(Identity::change_cdd_requirement_for_mk_rotation(root.clone(), true).is_ok());

        Context::set_current_identity::<Identity>(Some(alice_did));
        assert_err!(
            MultiSig::accept_multisig_signer_as_key(bob.clone(), bob_auth_id),
            Error::ChangeNotAllowed
        );
        Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
        assert!(Identity::change_cdd_requirement_for_mk_rotation(root.clone(), false).is_ok());

        Context::set_current_identity::<Identity>(Some(alice_did));
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
            call,
            None,
            false
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

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            charlie,
            charlie_auth_id
        ));

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
            Error::NotMasterKey
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

#[test]
fn check_for_approval_closure() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let eve_did = register_keyring_account(AccountKeyring::Eve).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let eve = Origin::signed(AccountKeyring::Eve.public());
        let bob_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Bob.public().encode()).unwrap());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), Signatory::from(eve_did)],
            1,
        ));

        let alice_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(alice_did))
                .next()
                .unwrap()
                .auth_id;

        Context::set_current_identity::<Identity>(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(alice_did)),
            true
        );

        let eve_auth_id =
            <identity::Authorizations<TestStorage>>::iter_prefix(Signatory::from(eve_did))
                .next()
                .unwrap()
                .auth_id;

        Context::set_current_identity::<Identity>(Some(eve_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            eve.clone(),
            eve_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), Signatory::from(eve_did)),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(musig_address.clone(), bob_signer),
            false
        );

        let call = Box::new(Call::MultiSig(multisig::Call::add_multisig_signer(
            bob_signer,
        )));
        Context::set_current_identity::<Identity>(Some(alice_did));
        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call.clone(),
            None,
            false
        ));
        let proposal_id = MultiSig::proposal_ids(musig_address.clone(), call).unwrap();
        let mut auth = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer);
        let bob_auth_id = auth.next().unwrap().auth_id;
        let multi_purpose_nonce = Identity::multi_purpose_nonce();

        Context::set_current_identity::<Identity>(Some(eve_did));
        assert_ok!(MultiSig::approve_as_identity(
            eve.clone(),
            musig_address.clone(),
            proposal_id
        ));
        auth = <identity::Authorizations<TestStorage>>::iter_prefix(bob_signer);
        let after_extra_approval_auth_id = auth.next().unwrap().auth_id;
        let after_extra_approval_multi_purpose_nonce = Identity::multi_purpose_nonce();
        // To validate that no new auth is created
        assert_eq!(bob_auth_id, after_extra_approval_auth_id);
        assert_eq!(
            multi_purpose_nonce,
            after_extra_approval_multi_purpose_nonce
        );
        assert_eq!(
            MultiSig::proposal_detail(&(musig_address.clone(), proposal_id)).approvals,
            2
        );
    });
}

#[test]
fn reject_proposals() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());

        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let bob = Origin::signed(AccountKeyring::Bob.public());

        let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let charlie = Origin::signed(AccountKeyring::Charlie.public());

        let dave_did = register_keyring_account(AccountKeyring::Dave).unwrap();
        let dave = Origin::signed(AccountKeyring::Dave.public());

        let eve_key = AccountKey::try_from(AccountKeyring::Eve.public().encode()).unwrap();
        let eve = Origin::signed(AccountKeyring::Eve.public());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        setup_multisig(
            alice.clone(),
            3,
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did),
                Signatory::from(dave_did),
                Signatory::from(eve_key),
            ],
        );

        let call1 = Box::new(Call::MultiSig(multisig::Call::change_sigs_required(4)));
        let call2 = Box::new(Call::MultiSig(multisig::Call::change_sigs_required(5)));
        Context::set_current_identity::<Identity>(Some(alice_did));
        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call1.clone(),
            None,
            false
        ));
        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call2.clone(),
            None,
            true
        ));

        let proposal_id1 = MultiSig::proposal_ids(musig_address.clone(), call1).unwrap();
        let proposal_id2 = MultiSig::proposal_ids(musig_address.clone(), call2).unwrap();

        // Proposal with auto close disabled can be voted on even after rejection.
        Context::set_current_identity::<Identity>(Some(bob_did));
        assert_ok!(MultiSig::reject_as_identity(
            bob.clone(),
            musig_address.clone(),
            proposal_id1
        ));
        Context::set_current_identity::<Identity>(Some(charlie_did));
        assert_ok!(MultiSig::reject_as_identity(
            charlie.clone(),
            musig_address.clone(),
            proposal_id1
        ));
        assert_ok!(MultiSig::reject_as_key(
            eve.clone(),
            musig_address.clone(),
            proposal_id1
        ));
        Context::set_current_identity::<Identity>(Some(dave_did));
        assert_ok!(MultiSig::approve_as_identity(
            dave.clone(),
            musig_address.clone(),
            proposal_id1
        ));
        let proposal_details1 = MultiSig::proposal_detail(&(musig_address.clone(), proposal_id1));
        assert_eq!(proposal_details1.approvals, 2);
        assert_eq!(proposal_details1.rejections, 3);
        assert_eq!(
            proposal_details1.status,
            multisig::ProposalStatus::ActiveOrExpired
        );
        assert_eq!(proposal_details1.auto_close, false);

        // Proposal with auto close enabled can not be voted on after rejection.
        Context::set_current_identity::<Identity>(Some(bob_did));
        assert_ok!(MultiSig::reject_as_identity(
            bob.clone(),
            musig_address.clone(),
            proposal_id2
        ));
        Context::set_current_identity::<Identity>(Some(charlie_did));
        assert_ok!(MultiSig::reject_as_identity(
            charlie.clone(),
            musig_address.clone(),
            proposal_id2
        ));
        assert_ok!(MultiSig::reject_as_key(
            eve.clone(),
            musig_address.clone(),
            proposal_id2
        ));
        Context::set_current_identity::<Identity>(Some(dave_did));
        assert_err!(
            MultiSig::approve_as_identity(dave.clone(), musig_address.clone(), proposal_id2),
            Error::ProposalAlreadyRejected
        );

        let proposal_details2 = MultiSig::proposal_detail(&(musig_address.clone(), proposal_id2));
        assert_eq!(proposal_details2.approvals, 1);
        assert_eq!(proposal_details2.rejections, 3);
        assert_eq!(proposal_details2.status, multisig::ProposalStatus::Rejected);
        assert_eq!(proposal_details2.auto_close, true);
    });
}

fn expired_proposals() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());

        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let bob = Origin::signed(AccountKeyring::Bob.public());

        let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let charlie = Origin::signed(AccountKeyring::Charlie.public());

        let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

        setup_multisig(
            alice.clone(),
            3,
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did),
            ],
        );

        let expires_at = 100u64;
        let call = Box::new(Call::MultiSig(multisig::Call::change_sigs_required(2)));

        Context::set_current_identity::<Identity>(Some(alice_did));
        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            musig_address.clone(),
            call.clone(),
            Some(100u64),
            false
        ));

        let proposal_id = MultiSig::proposal_ids(musig_address.clone(), call).unwrap();
        let mut proposal_details = MultiSig::proposal_detail(&(musig_address.clone(), proposal_id));
        assert_eq!(proposal_details.approvals, 1);
        assert_eq!(
            proposal_details.status,
            multisig::ProposalStatus::ActiveOrExpired
        );

        Context::set_current_identity::<Identity>(Some(bob_did));
        assert_ok!(MultiSig::approve_as_identity(
            bob.clone(),
            musig_address.clone(),
            proposal_id
        ));

        proposal_details = MultiSig::proposal_detail(&(musig_address.clone(), proposal_id));
        assert_eq!(proposal_details.approvals, 2);
        assert_eq!(
            proposal_details.status,
            multisig::ProposalStatus::ActiveOrExpired
        );

        // Approval fails when proposal has expired
        Timestamp::set_timestamp(expires_at);
        Context::set_current_identity::<Identity>(Some(charlie_did));
        assert_err!(
            MultiSig::approve_as_identity(charlie.clone(), musig_address.clone(), proposal_id),
            Error::ProposalExpired
        );

        proposal_details = MultiSig::proposal_detail(&(musig_address.clone(), proposal_id));
        assert_eq!(proposal_details.approvals, 2);
        assert_eq!(
            proposal_details.status,
            multisig::ProposalStatus::ActiveOrExpired
        );

        // Approval works when time is expiry - 1
        Timestamp::set_timestamp(expires_at - 1);
        assert_ok!(MultiSig::approve_as_identity(
            charlie.clone(),
            musig_address.clone(),
            proposal_id
        ));

        proposal_details = MultiSig::proposal_detail(&(musig_address.clone(), proposal_id));
        assert_eq!(proposal_details.approvals, 3);
        assert_eq!(
            proposal_details.status,
            multisig::ProposalStatus::ExecutionSuccessful
        );
    });
}

fn setup_multisig(creator_origin: Origin, sigs_required: u64, signers: Vec<Signatory>) {
    assert_ok!(MultiSig::create_multisig(
        creator_origin,
        signers.clone(),
        sigs_required,
    ));

    for signer in signers {
        let auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(signer)
            .next()
            .unwrap()
            .auth_id;
        assert_ok!(MultiSig::unsafe_accept_multisig_signer(signer, auth_id));
    }
}
