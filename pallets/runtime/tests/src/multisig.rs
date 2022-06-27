use super::{
    next_block,
    storage::{
        add_secondary_key, get_last_auth_id, get_primary_key, get_secondary_keys,
        register_keyring_account, set_curr_did, Call, TestStorage, User,
    },
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok};
use pallet_multisig as multisig;
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_primitives::{AccountId, AuthorizationData, Permissions, SecondaryKey, Signatory};
use test_client::AccountKeyring;

type Balances = pallet_balances::Module<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type MultiSig = pallet_multisig::Module<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::Origin;
type IdError = pallet_identity::Error<TestStorage>;
type Error = pallet_multisig::Error<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Scheduler = pallet_scheduler::Pallet<TestStorage>;

#[test]
fn create_multisig() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let ms_address = MultiSig::get_next_multisig_address(alice.acc());

        let signers = || vec![Signatory::from(alice.did), Signatory::from(bob.did)];
        let create = |signers, nsigs| MultiSig::create_multisig(alice.origin(), signers, nsigs);

        assert_ok!(create(signers(), 1));
        assert_eq!(MultiSig::ms_signs_required(ms_address), 1);

        assert_noop!(create(vec![], 10), Error::NoSigners);
        assert_noop!(create(signers(), 0), Error::RequiredSignaturesOutOfBounds);
        assert_noop!(create(signers(), 10), Error::RequiredSignaturesOutOfBounds);
    });
}

#[test]
fn join_multisig() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = Signatory::Account(AccountKeyring::Bob.to_account_id());
        let charlie = User::new(AccountKeyring::Charlie);

        // Add dave's key as a secondary key of charlie.
        let dave = User::new_with(charlie.did, AccountKeyring::Dave);
        add_secondary_key(charlie.did, dave.acc());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), bob_signer.clone()],
            1,
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(alice_did)),
            false
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        let alice_auth_id = get_last_auth_id(&Signatory::from(alice_did));
        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        let bob_auth_id = get_last_auth_id(&bob_signer);
        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(alice_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            true
        );

        set_curr_did(None);
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), bob_signer.clone()],
            1,
        ));

        let bob_auth_id2 = get_last_auth_id(&bob_signer);
        set_curr_did(Some(alice_did));
        assert_eq!(
            MultiSig::accept_multisig_signer_as_key(bob.clone(), bob_auth_id2),
            Err(Error::SignerAlreadyLinkedToMultisig.into()),
        );

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), dave.signatory_acc()],
            1,
        ));

        // Testing signer key that is already a secondary key on another identity.
        let dave_auth_id = get_last_auth_id(&dave.signatory_acc());
        set_curr_did(Some(alice_did));
        assert_eq!(
            MultiSig::accept_multisig_signer_as_key(dave.origin(), dave_auth_id),
            Err(Error::SignerAlreadyLinkedToIdentity.into()),
        );

        set_curr_did(None);
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::Account(ms_address.clone())],
            1,
        ));

        // Testing that a multisig can't add itself as a signer.
        let ms_auth_id = Identity::add_auth(
            alice_did,
            Signatory::Account(ms_address.clone()),
            AuthorizationData::AddMultiSigSigner(ms_address.clone()),
            None,
        );

        assert_eq!(
            MultiSig::accept_multisig_signer_as_key(Origin::signed(ms_address.clone()), ms_auth_id),
            Err(Error::MultisigNotAllowedToLinkToItself.into()),
        );
    });
}

#[test]
fn change_multisig_sigs_required() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = Signatory::Account(AccountKeyring::Bob.to_account_id());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), bob_signer.clone()],
            2,
        ));

        let alice_auth_id = get_last_auth_id(&Signatory::from(alice_did));
        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        let bob_auth_id = get_last_auth_id(&bob_signer);

        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(alice_did)),
            true
        );

        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), true);

        let call = Box::new(Call::MultiSig(multisig::Call::change_sigs_required {
            sigs_required: 1,
        }));

        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::create_proposal_as_key(
            bob.clone(),
            ms_address.clone(),
            call,
            None,
            false
        ));

        assert_eq!(MultiSig::ms_signs_required(ms_address.clone()), 2);

        let proposal = (ms_address.clone(), 0);
        let proposal_details = MultiSig::proposal_detail(&proposal);
        assert_eq!(
            proposal_details.status,
            multisig::ProposalStatus::ActiveOrExpired
        );

        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::approve_as_identity(
            alice.clone(),
            ms_address.clone(),
            0
        ));
        next_block();
        assert_eq!(MultiSig::ms_signs_required(ms_address), 1);
    });
}

#[test]
fn create_or_approve_change_multisig_sigs_required() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = Signatory::Account(AccountKeyring::Bob.to_account_id());
        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), bob_signer.clone()],
            2,
        ));
        let alice_auth_id = get_last_auth_id(&Signatory::from(alice_did));

        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));
        let bob_auth_id = get_last_auth_id(&bob_signer);
        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));
        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(alice_did)),
            true
        );
        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), true);
        let call = Box::new(Call::MultiSig(multisig::Call::change_sigs_required {
            sigs_required: 1,
        }));
        assert_ok!(MultiSig::create_or_approve_proposal_as_key(
            bob.clone(),
            ms_address.clone(),
            call.clone(),
            None,
            false
        ));
        next_block();
        assert_eq!(MultiSig::ms_signs_required(ms_address.clone()), 2);
        assert_ok!(MultiSig::create_or_approve_proposal_as_identity(
            alice.clone(),
            ms_address.clone(),
            call,
            None,
            false
        ));
        next_block();
        assert_eq!(MultiSig::ms_signs_required(ms_address), 1);
    });
}

#[test]
fn remove_multisig_signer() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let alice_signer = Signatory::from(alice_did);
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = Signatory::Account(AccountKeyring::Bob.to_account_id());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![alice_signer.clone(), bob_signer.clone()],
            1,
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 0);

        let alice_auth_id = get_last_auth_id(&alice_signer);

        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 1);

        let bob_auth_id = get_last_auth_id(&bob_signer);

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 2);

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), alice_signer.clone()),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            true
        );

        // No direct identity for Bob as he is only a signer
        assert_eq!(
            Identity::get_identity(&AccountKeyring::Bob.to_account_id()),
            None
        );
        // No identity as multisig has not been set as a secondary / primary key
        assert_eq!(Identity::get_identity(&ms_address), None);

        let call = Box::new(Call::MultiSig(multisig::Call::remove_multisig_signer {
            signer: bob_signer.clone(),
        }));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            ms_address.clone(),
            call,
            None,
            false
        ));

        next_block();

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 1);

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), alice_signer.clone()),
            true
        );

        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), false);

        assert_eq!(
            Identity::get_identity(&AccountKeyring::Bob.to_account_id()),
            None
        );

        set_curr_did(None);

        let remove_alice = Box::new(Call::MultiSig(multisig::Call::remove_multisig_signer {
            signer: alice_signer.clone(),
        }));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            ms_address.clone(),
            remove_alice,
            None,
            false
        ));

        next_block();

        // Alice not removed since that would've broken the multi sig.
        assert_eq!(MultiSig::ms_signers(ms_address.clone(), alice_signer), true);
    });
}

#[test]
fn add_multisig_signer() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = Signatory::Account(AccountKeyring::Bob.to_account_id());
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = Signatory::Account(AccountKeyring::Charlie.to_account_id());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did)],
            1,
        ));

        let alice_auth_id = get_last_auth_id(&Signatory::from(alice_did));

        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(alice_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        let call = Box::new(Call::MultiSig(multisig::Call::add_multisig_signer {
            signer: bob_signer.clone(),
        }));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            ms_address.clone(),
            call,
            None,
            false
        ));

        next_block();

        let call2 = Box::new(Call::MultiSig(multisig::Call::add_multisig_signer {
            signer: charlie_signer.clone(),
        }));

        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            ms_address.clone(),
            call2,
            None,
            false
        ));

        next_block();

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(alice_did)),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        let bob_auth_id = get_last_auth_id(&bob_signer);
        let charlie_auth_id = get_last_auth_id(&charlie_signer);

        let root = Origin::from(frame_system::RawOrigin::Root);

        assert_ok!(MultiSig::make_multisig_primary(
            alice.clone(),
            ms_address.clone(),
            None
        ));

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            charlie.clone(),
            charlie_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer),
            true
        );
        assert!(Identity::change_cdd_requirement_for_mk_rotation(root.clone(), true).is_ok());

        set_curr_did(Some(alice_did));
        assert_eq!(
            MultiSig::accept_multisig_signer_as_key(bob.clone(), bob_auth_id),
            Err(Error::ChangeNotAllowed.into()),
        );
        assert!(Identity::change_cdd_requirement_for_mk_rotation(root.clone(), false).is_ok());

        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), true);
    });
}

#[test]
fn make_multisig_primary() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did)],
            1,
        ));

        assert_eq!(
            get_primary_key(alice_did),
            AccountKeyring::Alice.to_account_id()
        );

        assert_noop!(
            MultiSig::make_multisig_primary(bob.clone(), ms_address.clone(), None),
            Error::IdentityNotCreator
        );

        assert_ok!(MultiSig::make_multisig_primary(
            alice.clone(),
            ms_address.clone(),
            None
        ));

        assert_eq!(get_primary_key(alice_did), ms_address);
    });
}

#[test]
fn rotate_multisig_primary_key_with_balance() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let dave_key = AccountKeyring::Dave.to_account_id();
        let charlie_key = AccountKeyring::Charlie.to_account_id();

        let ms_address = MultiSig::get_next_multisig_address(alice.acc());

        assert_ok!(MultiSig::create_multisig(
            alice.origin(),
            vec![Signatory::Account(dave_key)],
            1,
        ));

        // Alice's primary key hasn't changed.
        assert_eq!(get_primary_key(alice.did), alice.acc());

        // Bob can't make the MultiSig account his primary key.
        assert_noop!(
            MultiSig::make_multisig_primary(bob.origin(), ms_address.clone(), None),
            Error::IdentityNotCreator
        );

        // Make the MultiSig account Alice's primary key.
        assert_ok!(MultiSig::make_multisig_primary(
            alice.origin(),
            ms_address.clone(),
            None
        ));

        // Alice's primary key is now the MultiSig.
        assert_eq!(get_primary_key(alice.did), ms_address);

        // Fund the MultiSig.
        assert_ok!(Balances::transfer(
            bob.origin(),
            ms_address.clone().into(),
            2 * POLY
        ));

        // Add RotatePrimaryKey authorization for charlie_key to become the primary_key for Alice.
        let auth_id = Identity::add_auth(
            alice.did,
            Signatory::Account(charlie_key.clone()),
            AuthorizationData::RotatePrimaryKey,
            None,
        );
        // Fails because the current MultiSig primary_key has a balance.
        assert_eq!(
            Identity::accept_primary_key(Origin::signed(charlie_key.clone()), auth_id, None),
            Err(IdError::MultiSigHasBalance.into()),
        );
    });
}

#[test]
fn make_multisig_secondary_key() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let multisig = MultiSig::get_next_multisig_address(alice.acc());

        assert_ok!(MultiSig::create_multisig(
            alice.origin(),
            vec![Signatory::from(alice.did)],
            1,
        ));
        // The desired secondary key record.
        let multisig_secondary = SecondaryKey::new(multisig.clone(), Permissions::empty());

        let has_ms_sk = || get_secondary_keys(alice.did).contains(&multisig_secondary);
        assert!(!has_ms_sk());

        let mk_ms_signer =
            |u: User| MultiSig::make_multisig_secondary(u.origin(), multisig.clone());
        assert_noop!(mk_ms_signer(bob), Error::IdentityNotCreator);

        assert_ok!(mk_ms_signer(alice));
        assert!(has_ms_sk());

        assert_noop!(mk_ms_signer(alice), IdError::AlreadyLinked);
    });
}

#[test]
fn remove_multisig_signers_via_creator() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let alice_signer = Signatory::from(alice_did);
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = Signatory::Account(AccountKeyring::Bob.to_account_id());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![alice_signer.clone(), bob_signer.clone()],
            1,
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 0);

        let alice_auth_id = get_last_auth_id(&alice_signer.clone());

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 1);

        let bob_auth_id = get_last_auth_id(&bob_signer.clone());

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 2);

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), alice_signer.clone()),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            true
        );

        assert_noop!(
            MultiSig::remove_multisig_signers_via_creator(
                bob.clone(),
                ms_address.clone(),
                vec![bob_signer.clone()]
            ),
            Error::IdentityNotCreator
        );

        assert_ok!(MultiSig::remove_multisig_signers_via_creator(
            alice.clone(),
            ms_address.clone(),
            vec![bob_signer.clone()]
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 1);

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), alice_signer.clone()),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        assert_noop!(
            MultiSig::remove_multisig_signers_via_creator(
                alice.clone(),
                ms_address.clone(),
                vec![alice_signer.clone()]
            ),
            Error::NotEnoughSigners
        );

        // Alice not removed since that would've broken the multi sig.
        assert_eq!(MultiSig::ms_signers(ms_address.clone(), alice_signer), true);
    });
}

#[test]
fn add_multisig_signers_via_creator() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = Signatory::Account(AccountKeyring::Bob.to_account_id());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did)],
            1,
        ));

        let alice_auth_id = get_last_auth_id(&Signatory::from(alice_did));

        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(alice_did)),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        assert_noop!(
            MultiSig::add_multisig_signers_via_creator(
                bob.clone(),
                ms_address.clone(),
                vec![bob_signer.clone()]
            ),
            Error::IdentityNotCreator
        );

        assert_ok!(MultiSig::add_multisig_signers_via_creator(
            alice.clone(),
            ms_address.clone(),
            vec![bob_signer.clone()]
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(alice_did)),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        let bob_auth_id = get_last_auth_id(&bob_signer.clone());

        assert_ok!(MultiSig::accept_multisig_signer_as_key(
            bob.clone(),
            bob_auth_id
        ));

        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), true);
    });
}

#[test]
fn check_for_approval_closure() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let eve_did = register_keyring_account(AccountKeyring::Eve).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let eve = Origin::signed(AccountKeyring::Eve.to_account_id());
        let bob_signer = Signatory::Account(AccountKeyring::Bob.to_account_id());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![Signatory::from(alice_did), Signatory::from(eve_did)],
            1,
        ));

        let alice_auth_id = get_last_auth_id(&Signatory::from(alice_did));

        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            alice.clone(),
            alice_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(alice_did)),
            true
        );

        let eve_auth_id = get_last_auth_id(&Signatory::from(eve_did));

        set_curr_did(Some(eve_did));
        assert_ok!(MultiSig::accept_multisig_signer_as_identity(
            eve.clone(),
            eve_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), Signatory::from(eve_did)),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        let call = Box::new(Call::MultiSig(multisig::Call::add_multisig_signer {
            signer: bob_signer.clone(),
        }));
        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            ms_address.clone(),
            call,
            None,
            false
        ));
        next_block();
        let proposal_id = MultiSig::ms_tx_done(ms_address.clone()) - 1;
        let bob_auth_id = get_last_auth_id(&bob_signer.clone());
        let multi_purpose_nonce = Identity::multi_purpose_nonce();

        set_curr_did(Some(eve_did));

        assert_noop!(
            MultiSig::approve_as_identity(eve.clone(), ms_address.clone(), proposal_id),
            Error::ProposalAlreadyExecuted
        );

        next_block();
        let after_extra_approval_auth_id = get_last_auth_id(&bob_signer.clone());
        let after_extra_approval_multi_purpose_nonce = Identity::multi_purpose_nonce();
        // To validate that no new auth is created
        assert_eq!(bob_auth_id, after_extra_approval_auth_id);
        assert_eq!(
            multi_purpose_nonce,
            after_extra_approval_multi_purpose_nonce
        );
        assert_eq!(
            MultiSig::proposal_detail(&(ms_address.clone(), proposal_id)).approvals,
            1
        );
    });
}

#[test]
fn reject_proposals() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());

        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());

        let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());

        let dave_did = register_keyring_account(AccountKeyring::Dave).unwrap();
        let dave = Origin::signed(AccountKeyring::Dave.to_account_id());

        let eve_key = AccountKeyring::Eve.to_account_id();
        let eve = Origin::signed(AccountKeyring::Eve.to_account_id());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

        setup_multisig(
            alice.clone(),
            3,
            vec![
                Signatory::from(alice_did),
                Signatory::from(bob_did),
                Signatory::from(charlie_did),
                Signatory::from(dave_did),
                Signatory::Account(eve_key),
            ],
        );

        let call1 = Box::new(Call::MultiSig(multisig::Call::change_sigs_required {
            sigs_required: 4,
        }));
        let call2 = Box::new(Call::MultiSig(multisig::Call::change_sigs_required {
            sigs_required: 5,
        }));
        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            ms_address.clone(),
            call1,
            None,
            false
        ));
        let proposal_id1 = MultiSig::ms_tx_done(ms_address.clone()) - 1;
        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            ms_address.clone(),
            call2,
            None,
            true
        ));
        let proposal_id2 = MultiSig::ms_tx_done(ms_address.clone()) - 1;

        // Proposal with auto close disabled can be voted on even after rejection.
        set_curr_did(Some(bob_did));
        assert_ok!(MultiSig::reject_as_identity(
            bob.clone(),
            ms_address.clone(),
            proposal_id1
        ));
        set_curr_did(Some(charlie_did));
        assert_ok!(MultiSig::reject_as_identity(
            charlie.clone(),
            ms_address.clone(),
            proposal_id1
        ));
        assert_ok!(MultiSig::reject_as_key(
            eve.clone(),
            ms_address.clone(),
            proposal_id1
        ));
        set_curr_did(Some(dave_did));
        assert_ok!(MultiSig::approve_as_identity(
            dave.clone(),
            ms_address.clone(),
            proposal_id1
        ));
        let proposal_details1 = MultiSig::proposal_detail(&(ms_address.clone(), proposal_id1));
        assert_eq!(proposal_details1.approvals, 2);
        assert_eq!(proposal_details1.rejections, 3);
        assert_eq!(
            proposal_details1.status,
            multisig::ProposalStatus::ActiveOrExpired
        );
        assert_eq!(proposal_details1.auto_close, false);

        // Proposal with auto close enabled can not be voted on after rejection.
        set_curr_did(Some(bob_did));
        assert_ok!(MultiSig::reject_as_identity(
            bob.clone(),
            ms_address.clone(),
            proposal_id2
        ));
        set_curr_did(Some(charlie_did));
        assert_ok!(MultiSig::reject_as_identity(
            charlie.clone(),
            ms_address.clone(),
            proposal_id2
        ));
        assert_ok!(MultiSig::reject_as_key(
            eve.clone(),
            ms_address.clone(),
            proposal_id2
        ));
        set_curr_did(Some(dave_did));
        assert_noop!(
            MultiSig::approve_as_identity(dave.clone(), ms_address.clone(), proposal_id2),
            Error::ProposalAlreadyRejected
        );

        let proposal_details2 = MultiSig::proposal_detail(&(ms_address.clone(), proposal_id2));
        next_block();
        assert_eq!(proposal_details2.approvals, 1);
        assert_eq!(proposal_details2.rejections, 3);
        assert_eq!(proposal_details2.status, multisig::ProposalStatus::Rejected);
        assert_eq!(proposal_details2.auto_close, true);
    });
}

fn expired_proposals() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());

        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());

        let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id());

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
        let call = Box::new(Call::MultiSig(multisig::Call::change_sigs_required {
            sigs_required: 2,
        }));

        set_curr_did(Some(alice_did));
        assert_ok!(MultiSig::create_proposal_as_identity(
            alice.clone(),
            ms_address.clone(),
            call,
            Some(100u64),
            false
        ));

        let proposal_id = MultiSig::ms_tx_done(ms_address.clone()) - 1;
        let mut proposal_details = MultiSig::proposal_detail(&(ms_address.clone(), proposal_id));
        assert_eq!(proposal_details.approvals, 1);
        assert_eq!(
            proposal_details.status,
            multisig::ProposalStatus::ActiveOrExpired
        );

        set_curr_did(Some(bob_did));
        assert_ok!(MultiSig::approve_as_identity(
            bob.clone(),
            ms_address.clone(),
            proposal_id
        ));

        proposal_details = MultiSig::proposal_detail(&(ms_address.clone(), proposal_id));
        assert_eq!(proposal_details.approvals, 2);
        assert_eq!(
            proposal_details.status,
            multisig::ProposalStatus::ActiveOrExpired
        );

        // Approval fails when proposal has expired
        Timestamp::set_timestamp(expires_at);
        set_curr_did(Some(charlie_did));
        assert_noop!(
            MultiSig::approve_as_identity(charlie.clone(), ms_address.clone(), proposal_id),
            Error::ProposalExpired
        );

        proposal_details = MultiSig::proposal_detail(&(ms_address.clone(), proposal_id));
        assert_eq!(proposal_details.approvals, 2);
        assert_eq!(
            proposal_details.status,
            multisig::ProposalStatus::ActiveOrExpired
        );

        // Approval works when time is expiry - 1
        Timestamp::set_timestamp(expires_at - 1);
        assert_ok!(MultiSig::approve_as_identity(
            charlie.clone(),
            ms_address.clone(),
            proposal_id
        ));

        proposal_details = MultiSig::proposal_detail(&(ms_address.clone(), proposal_id));
        assert_eq!(proposal_details.approvals, 3);
        assert_eq!(
            proposal_details.status,
            multisig::ProposalStatus::ExecutionSuccessful
        );
    });
}

fn setup_multisig(creator_origin: Origin, sigs_required: u64, signers: Vec<Signatory<AccountId>>) {
    assert_ok!(MultiSig::create_multisig(
        creator_origin,
        signers.clone(),
        sigs_required,
    ));

    for signer in signers {
        let auth_id = get_last_auth_id(&signer);
        assert_ok!(MultiSig::unsafe_accept_multisig_signer(signer, auth_id));
    }
}
