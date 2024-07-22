use frame_support::{
    assert_err_ignore_postinfo, assert_noop, assert_ok, assert_storage_noop, dispatch::Weight,
};

use pallet_multisig::{
    self as multisig, LostCreatorPrivileges, ProposalStates, ProposalVoteCounts, Votes,
};
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_primitives::multisig::ProposalState;
use polymesh_primitives::{AccountId, AuthorizationData, Permissions, SecondaryKey, Signatory};
use sp_keyring::AccountKeyring;

use super::asset_test::set_timestamp;
use super::next_block;
use super::storage::{
    add_secondary_key, get_primary_key, get_secondary_keys, register_keyring_account, RuntimeCall,
    TestStorage, User,
};
use super::ExtBuilder;

type Balances = pallet_balances::Module<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type MultiSig = pallet_multisig::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type IdError = pallet_identity::Error<TestStorage>;
type Error = pallet_multisig::Error<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Scheduler = pallet_scheduler::Pallet<TestStorage>;

fn get_last_auth_id(account: &AccountId) -> u64 {
    let signatory = Signatory::Account(account.clone());
    super::storage::get_last_auth_id(&signatory)
}

#[test]
fn create_multisig() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let eve_signer = AccountKeyring::Eve.to_account_id();

        let ms_address = MultiSig::get_next_multisig_address(alice.acc()).expect("Next MS");

        let signers = || vec![eve_signer.clone(), bob_signer.clone()];
        let create = |signers, nsigs| MultiSig::create_multisig(alice.origin(), signers, nsigs);

        assert_ok!(create(signers(), 1));
        assert_eq!(MultiSig::ms_signs_required(ms_address), 1);

        assert_noop!(create(vec![], 10), Error::NotEnoughSigners);
        assert_noop!(create(signers(), 0), Error::RequiredSignersIsZero);
        assert_noop!(create(signers(), 10), Error::NotEnoughSigners);
    });
}

#[test]
fn join_multisig() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());

        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());
        let ferdie_signer = ferdie_key;

        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = User::new(AccountKeyring::Charlie);

        // Add dave's key as a secondary key of charlie.
        let dave = User::new_with(charlie.did, AccountKeyring::Dave);
        add_secondary_key(charlie.did, dave.acc());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer.clone(), bob_signer.clone()],
            1,
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), ferdie_signer.clone()),
            false
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        let ferdie_auth_id = get_last_auth_id(&ferdie_signer);
        assert_ok!(MultiSig::accept_multisig_signer(
            ferdie.clone(),
            ferdie_auth_id
        ));

        let bob_auth_id = get_last_auth_id(&bob_signer);
        assert_ok!(MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), ferdie_signer.clone()),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            true
        );

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer.clone(), bob_signer.clone()],
            1,
        ));

        let bob_auth_id2 = get_last_auth_id(&bob_signer);
        assert_eq!(
            MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id2),
            Err(Error::SignerAlreadyLinkedToMultisig.into()),
        );

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer.clone(), dave.acc()],
            1,
        ));

        // Testing signer key that is already a secondary key on another identity.
        let dave_auth_id = get_last_auth_id(&dave.acc());
        assert_eq!(
            MultiSig::accept_multisig_signer(dave.origin(), dave_auth_id),
            Err(Error::SignerAlreadyLinkedToIdentity.into()),
        );

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![ms_address.clone()],
            1,
        ));

        // Testing that a multisig can't add itself as a signer.
        let ms_auth_id = Identity::add_auth(
            alice_did,
            Signatory::Account(ms_address.clone()),
            AuthorizationData::AddMultiSigSigner(ms_address.clone()),
            None,
        )
        .unwrap();

        assert_eq!(
            MultiSig::accept_multisig_signer(Origin::signed(ms_address.clone()), ms_auth_id),
            Err(Error::MultisigNotAllowedToLinkToItself.into()),
        );
    });
}

#[test]
fn change_multisig_sigs_required() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![charlie_signer.clone(), bob_signer.clone()],
            2,
        ));

        let charlie_auth_id = get_last_auth_id(&charlie_signer);
        assert_ok!(MultiSig::accept_multisig_signer(
            charlie.clone(),
            charlie_auth_id
        ));

        let bob_auth_id = get_last_auth_id(&bob_signer);
        assert_ok!(MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer),
            true
        );

        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), true);

        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 1 },
        ));

        assert_ok!(MultiSig::create_proposal(
            bob.clone(),
            ms_address.clone(),
            call,
            None,
        ));

        assert_eq!(MultiSig::ms_signs_required(ms_address.clone()), 2);

        let proposal_state = ProposalStates::<TestStorage>::get(&ms_address, 0).unwrap();
        assert_eq!(proposal_state, ProposalState::Active { until: None });

        assert_ok!(MultiSig::approve(
            charlie.clone(),
            ms_address.clone(),
            0,
            Weight::MAX
        ));
        next_block();
        assert_eq!(MultiSig::ms_signs_required(ms_address), 1);
    });
}

#[test]
fn create_or_approve_change_multisig_sigs_required() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![charlie_signer.clone(), bob_signer.clone()],
            2,
        ));

        let charlie_auth_id = get_last_auth_id(&charlie_signer);
        assert_ok!(MultiSig::accept_multisig_signer(
            charlie.clone(),
            charlie_auth_id
        ));

        let bob_auth_id = get_last_auth_id(&bob_signer);
        assert_ok!(MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id));
        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer),
            true
        );
        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), true);
        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 1 },
        ));
        assert_ok!(MultiSig::create_or_approve_proposal(
            bob.clone(),
            ms_address.clone(),
            call.clone(),
            None,
        ));
        next_block();
        assert_eq!(MultiSig::ms_signs_required(ms_address.clone()), 2);
        assert_ok!(MultiSig::create_or_approve_proposal(
            charlie.clone(),
            ms_address.clone(),
            call,
            None,
        ));
        next_block();
        assert_eq!(MultiSig::ms_signs_required(ms_address), 1);
    });
}

#[test]
fn remove_multisig_signers() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![charlie_signer.clone(), bob_signer.clone()],
            1,
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 0);

        let charlie_auth_id = get_last_auth_id(&charlie_signer);
        assert_ok!(MultiSig::accept_multisig_signer(
            charlie.clone(),
            charlie_auth_id
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 1);

        let bob_auth_id = get_last_auth_id(&bob_signer);

        assert_ok!(MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 2);

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer.clone()),
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

        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::remove_multisig_signers {
                signers: vec![bob_signer.clone()],
            },
        ));

        assert_ok!(MultiSig::create_proposal(
            charlie.clone(),
            ms_address.clone(),
            call,
            None,
        ));

        next_block();

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 1);

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer.clone()),
            true
        );

        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), false);

        assert_eq!(
            Identity::get_identity(&AccountKeyring::Bob.to_account_id()),
            None
        );

        let remove_alice = Box::new(RuntimeCall::MultiSig(
            multisig::Call::remove_multisig_signers {
                signers: vec![charlie_signer.clone()],
            },
        ));

        assert_ok!(MultiSig::create_proposal(
            charlie.clone(),
            ms_address.clone(),
            remove_alice,
            None,
        ));

        next_block();

        // Alice not removed since that would've broken the multi sig.
        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer),
            true
        );
    });
}

#[test]
fn add_multisig_signers() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = Origin::signed(AccountKeyring::Dave.to_account_id());
        let dave_signer = AccountKeyring::Dave.to_account_id();

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![dave_signer.clone()],
            1,
        ));

        let dave_auth_id = get_last_auth_id(&dave_signer);

        assert_ok!(MultiSig::accept_multisig_signer(dave.clone(), dave_auth_id));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), dave_signer.clone()),
            true
        );
        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::add_multisig_signers {
                signers: vec![bob_signer.clone()],
            },
        ));

        assert_ok!(MultiSig::create_proposal(
            dave.clone(),
            ms_address.clone(),
            call,
            None,
        ));

        next_block();

        let call2 = Box::new(RuntimeCall::MultiSig(
            multisig::Call::add_multisig_signers {
                signers: vec![charlie_signer.clone()],
            },
        ));

        assert_ok!(MultiSig::create_proposal(
            dave.clone(),
            ms_address.clone(),
            call2,
            None,
        ));

        next_block();

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), dave_signer.clone()),
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

        assert_ok!(MultiSig::accept_multisig_signer(
            charlie.clone(),
            charlie_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer),
            true
        );
        assert!(Identity::change_cdd_requirement_for_mk_rotation(root.clone(), true).is_ok());

        assert_eq!(
            MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id),
            Err(Error::ChangeNotAllowed.into()),
        );
        assert!(Identity::change_cdd_requirement_for_mk_rotation(root.clone(), false).is_ok());

        assert_ok!(MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id));

        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), true);
    });
}

#[test]
fn make_multisig_primary() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = User::new(AccountKeyring::Bob);
        let ferdie_key = AccountKeyring::Ferdie.to_account_id();

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_key],
            1,
        ));

        assert_eq!(
            get_primary_key(alice_did),
            AccountKeyring::Alice.to_account_id()
        );

        assert_noop!(
            MultiSig::make_multisig_primary(bob.origin(), ms_address.clone(), None),
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

        let ms_address = MultiSig::get_next_multisig_address(alice.acc()).expect("Next MS");

        assert_ok!(MultiSig::create_multisig(alice.origin(), vec![dave_key], 1,));

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
        )
        .unwrap();
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
        let ferdie_key = AccountKeyring::Ferdie.to_account_id();

        let multisig = MultiSig::get_next_multisig_address(alice.acc()).expect("Next MS");

        assert_ok!(MultiSig::create_multisig(
            alice.origin(),
            vec![ferdie_key],
            1,
        ));
        // The desired secondary key record.
        let multisig_secondary = SecondaryKey::new(multisig.clone(), Permissions::empty());

        let has_ms_sk = || get_secondary_keys(alice.did).contains(&multisig_secondary);
        assert!(!has_ms_sk());

        let mk_ms_signer =
            |u: User| MultiSig::make_multisig_secondary(u.origin(), multisig.clone(), None);
        assert_noop!(mk_ms_signer(bob), Error::IdentityNotCreator);

        assert_ok!(mk_ms_signer(alice));
        assert!(has_ms_sk());

        assert_noop!(mk_ms_signer(alice), IdError::AlreadyLinked);
    });
}

#[test]
fn remove_multisig_signers_via_creator() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = User::new(AccountKeyring::Dave);

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![charlie_signer.clone(), bob_signer.clone()],
            1,
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 0);

        let charlie_auth_id = get_last_auth_id(&charlie_signer.clone());

        assert_ok!(MultiSig::accept_multisig_signer(
            charlie.clone(),
            charlie_auth_id
        ));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 1);

        let bob_auth_id = get_last_auth_id(&bob_signer.clone());

        assert_ok!(MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id));

        assert_eq!(MultiSig::number_of_signers(ms_address.clone()), 2);

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer.clone()),
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
            IdError::KeyNotAllowed
        );

        assert_noop!(
            MultiSig::remove_multisig_signers_via_creator(
                dave.origin(),
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
            MultiSig::ms_signers(ms_address.clone(), charlie_signer.clone()),
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
                vec![charlie_signer.clone()]
            ),
            Error::NotEnoughSigners
        );

        // Alice not removed since that would've broken the multi sig.
        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer),
            true
        );
    });
}

#[test]
fn add_multisig_signers_via_creator() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = User::new(AccountKeyring::Dave);

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![charlie_signer.clone()],
            1,
        ));

        let charlie_auth_id = get_last_auth_id(&charlie_signer);

        assert_ok!(MultiSig::accept_multisig_signer(
            charlie.clone(),
            charlie_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer.clone()),
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
            pallet_permissions::Error::<TestStorage>::UnauthorizedCaller
        );

        assert_noop!(
            MultiSig::add_multisig_signers_via_creator(
                dave.origin(),
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
            MultiSig::ms_signers(ms_address.clone(), charlie_signer.clone()),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        let bob_auth_id = get_last_auth_id(&bob_signer.clone());

        assert_ok!(MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id));

        assert_eq!(MultiSig::ms_signers(ms_address.clone(), bob_signer), true);
    });
}

#[test]
fn check_for_approval_closure() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = Origin::signed(AccountKeyring::Dave.to_account_id());
        let dave_signer = AccountKeyring::Dave.to_account_id();

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![charlie_signer.clone(), dave_signer.clone()],
            1,
        ));
        let charlie_auth_id = get_last_auth_id(&charlie_signer.clone());
        assert_ok!(MultiSig::accept_multisig_signer(
            charlie.clone(),
            charlie_auth_id
        ));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), charlie_signer.clone()),
            true
        );

        let dave_auth_id = get_last_auth_id(&dave_signer.clone());

        assert_ok!(MultiSig::accept_multisig_signer(dave.clone(), dave_auth_id));

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), dave_signer.clone()),
            true
        );

        assert_eq!(
            MultiSig::ms_signers(ms_address.clone(), bob_signer.clone()),
            false
        );

        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::add_multisig_signers {
                signers: vec![bob_signer.clone()],
            },
        ));
        assert_ok!(MultiSig::create_proposal(
            charlie.clone(),
            ms_address.clone(),
            call,
            None,
        ));
        next_block();
        let proposal_id = MultiSig::ms_tx_done(ms_address.clone()) - 1;
        let bob_auth_id = get_last_auth_id(&bob_signer.clone());
        let multi_purpose_nonce = Identity::multi_purpose_nonce();

        assert_storage_noop!(assert_err_ignore_postinfo!(
            MultiSig::approve(dave.clone(), ms_address.clone(), proposal_id, Weight::MAX),
            Error::ProposalAlreadyExecuted
        ));

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
            ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id)
                .unwrap()
                .approvals,
            1
        );
    });
}

#[test]
fn reject_proposals() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());

        let bob_key = AccountKeyring::Bob.to_account_id();
        let bob = Origin::signed(bob_key.clone());

        let charlie_key = AccountKeyring::Charlie.to_account_id();
        let charlie = Origin::signed(charlie_key.clone());

        let dave_key = AccountKeyring::Dave.to_account_id();
        let dave = Origin::signed(dave_key.clone());

        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());

        let eve_key = AccountKeyring::Eve.to_account_id();
        let eve = Origin::signed(eve_key.clone());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        setup_multisig(
            alice.clone(),
            3,
            vec![ferdie_key, bob_key, charlie_key, dave_key, eve_key],
        );

        let call1 = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 4 },
        ));
        let call2 = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 5 },
        ));
        assert_ok!(MultiSig::create_proposal(
            ferdie.clone(),
            ms_address.clone(),
            call1,
            None,
        ));
        let proposal_id1 = MultiSig::ms_tx_done(ms_address.clone()) - 1;
        assert_ok!(MultiSig::create_proposal(
            ferdie.clone(),
            ms_address.clone(),
            call2,
            None,
        ));
        let proposal_id2 = MultiSig::ms_tx_done(ms_address.clone()) - 1;

        // Proposals can't be voted on even after rejection.
        assert_ok!(MultiSig::reject(
            bob.clone(),
            ms_address.clone(),
            proposal_id1
        ));
        assert_ok!(MultiSig::reject(
            charlie.clone(),
            ms_address.clone(),
            proposal_id1
        ));
        assert_ok!(MultiSig::reject(
            eve.clone(),
            ms_address.clone(),
            proposal_id1
        ));
        assert_storage_noop!(assert_err_ignore_postinfo!(
            MultiSig::approve(dave.clone(), ms_address.clone(), proposal_id1, Weight::MAX),
            Error::ProposalAlreadyRejected
        ));
        let vote_count1 =
            ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id1).unwrap();
        let proposal_state1 =
            ProposalStates::<TestStorage>::get(&ms_address, proposal_id1).unwrap();
        assert_eq!(vote_count1.approvals, 1);
        assert_eq!(vote_count1.rejections, 3);
        assert_eq!(proposal_state1, ProposalState::Rejected);

        // Proposal can't be voted on after rejection.
        assert_ok!(MultiSig::reject(
            bob.clone(),
            ms_address.clone(),
            proposal_id2
        ));
        assert_ok!(MultiSig::reject(
            charlie.clone(),
            ms_address.clone(),
            proposal_id2
        ));
        assert_ok!(MultiSig::reject(
            eve.clone(),
            ms_address.clone(),
            proposal_id2
        ));
        assert_storage_noop!(assert_err_ignore_postinfo!(
            MultiSig::approve(dave.clone(), ms_address.clone(), proposal_id2, Weight::MAX),
            Error::ProposalAlreadyRejected
        ));

        let vote_count2 =
            ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id2).unwrap();
        let proposal_state2 =
            ProposalStates::<TestStorage>::get(&ms_address, proposal_id2).unwrap();
        next_block();
        assert_eq!(vote_count2.approvals, 1);
        assert_eq!(vote_count2.rejections, 3);
        assert_eq!(proposal_state2, ProposalState::Rejected);
    });
}

#[test]
fn add_signers_via_creator_removed_controls() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let multisig_account_id =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();

        MultiSig::create_multisig(alice.clone(), vec![ferdie_signer, bob_signer], 2).unwrap();
        MultiSig::remove_creator_controls(alice.clone(), multisig_account_id.clone()).unwrap();

        assert_noop!(
            MultiSig::add_multisig_signers_via_creator(
                alice.clone(),
                multisig_account_id,
                vec![charlie_signer]
            ),
            Error::CreatorControlsHaveBeenRemoved
        );
    });
}

#[test]
fn remove_signers_via_creator_removed_controls() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let multisig_account_id =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();

        MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer, bob_signer, charlie_signer.clone()],
            2,
        )
        .unwrap();
        MultiSig::remove_creator_controls(alice.clone(), multisig_account_id.clone()).unwrap();

        assert_noop!(
            MultiSig::add_multisig_signers_via_creator(
                alice.clone(),
                multisig_account_id,
                vec![charlie_signer]
            ),
            Error::CreatorControlsHaveBeenRemoved
        );
    });
}

#[test]
fn change_sigs_required_via_creator_id_not_creator() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = User::new(AccountKeyring::Dave);

        let multisig_account_id =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();

        MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer, bob_signer, charlie_signer],
            2,
        )
        .unwrap();

        assert_noop!(
            MultiSig::change_sigs_required_via_creator(bob.clone(), multisig_account_id.clone(), 2),
            pallet_permissions::Error::<TestStorage>::UnauthorizedCaller
        );

        assert_noop!(
            MultiSig::change_sigs_required_via_creator(dave.origin(), multisig_account_id, 2),
            Error::IdentityNotCreator
        );
    });
}

#[test]
fn change_sigs_required_via_creator_removed_controls() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let multisig_account_id =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();

        MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer, bob_signer, charlie_signer],
            2,
        )
        .unwrap();
        MultiSig::remove_creator_controls(alice.clone(), multisig_account_id.clone()).unwrap();

        assert_noop!(
            MultiSig::change_sigs_required_via_creator(alice.clone(), multisig_account_id, 2),
            Error::CreatorControlsHaveBeenRemoved
        );
    });
}

#[test]
fn change_sigs_required_via_creator_not_enough_signers() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let multisig_account_id =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();

        MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer, bob_signer.clone(), charlie_signer],
            2,
        )
        .unwrap();

        // Signers must accept to be added to the multisig account
        let bob_auth_id = get_last_auth_id(&bob_signer);
        MultiSig::accept_multisig_signer(bob, bob_auth_id).unwrap();

        assert_noop!(
            MultiSig::change_sigs_required_via_creator(alice.clone(), multisig_account_id, 4),
            Error::NotEnoughSigners
        );
    });
}

#[test]
fn change_sigs_required_via_creator_successfully() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let multisig_account_id =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();

        MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer, bob_signer.clone(), charlie_signer],
            2,
        )
        .unwrap();
        // Signers must accept to be added to the multisig account
        let bob_auth_id = get_last_auth_id(&bob_signer);
        MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id).unwrap();

        assert_ok!(MultiSig::change_sigs_required_via_creator(
            alice.clone(),
            multisig_account_id,
            1
        ));
    });
}

#[test]
fn remove_creator_controls_id_not_creator() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = User::new(AccountKeyring::Dave);

        let multisig_account_id =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();

        MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer, bob_signer.clone(), charlie_signer],
            2,
        )
        .unwrap();

        assert_noop!(
            MultiSig::remove_creator_controls(bob.clone(), multisig_account_id.clone()),
            pallet_permissions::Error::<TestStorage>::UnauthorizedCaller
        );

        assert_noop!(
            MultiSig::remove_creator_controls(dave.origin(), multisig_account_id),
            Error::IdentityNotCreator
        );
    });
}

#[test]
fn remove_creator_controls_successfully() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let multisig_account_id =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();

        MultiSig::create_multisig(
            alice.clone(),
            vec![ferdie_signer, bob_signer.clone(), charlie_signer],
            2,
        )
        .unwrap();

        assert_ok!(MultiSig::remove_creator_controls(
            alice.clone(),
            multisig_account_id
        ));
        assert!(LostCreatorPrivileges::<TestStorage>::get(alice_did))
    });
}

#[test]
fn proposal_owner_rejection() {
    ExtBuilder::default().build().execute_with(|| {
        let alice: User = User::new(AccountKeyring::Alice);

        let bob_key = AccountKeyring::Bob.to_account_id();
        let bob = Origin::signed(bob_key.clone());

        let dave_key = AccountKeyring::Dave.to_account_id();

        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());

        let eve_key = AccountKeyring::Eve.to_account_id();

        // Creates a multi-signature
        let ms_address =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();
        setup_multisig(
            alice.origin(),
            3,
            vec![ferdie_key.clone(), bob_key, dave_key, eve_key],
        );

        // Creates a proposal
        let call1 = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 4 },
        ));
        assert_ok!(MultiSig::create_proposal(
            ferdie.clone(),
            ms_address.clone(),
            call1,
            None,
        ));
        let proposal_id = MultiSig::ms_tx_done(ms_address.clone()) - 1;

        // The owner of the proposal should be able to reject it if no one else has voted
        assert_ok!(MultiSig::reject(
            ferdie.clone(),
            ms_address.clone(),
            proposal_id
        ));

        // The proposal status must be set to rejected
        let vote_count = ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        let proposal_state = ProposalStates::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        assert_eq!(proposal_state, ProposalState::Rejected);
        assert_eq!(vote_count.approvals, 0);
        assert_eq!(vote_count.rejections, 1);
        assert_eq!(
            Votes::<TestStorage>::get((&ms_address, proposal_id), ferdie_key),
            true
        );

        // The owner shouldn't be able to change their vote again
        assert_noop!(
            MultiSig::reject(ferdie, ms_address.clone(), proposal_id),
            Error::ProposalAlreadyRejected
        );

        // No other votes are allowed, since the proposal has been rejected
        assert_noop!(
            MultiSig::reject(bob, ms_address.clone(), proposal_id),
            Error::ProposalAlreadyRejected
        );
    });
}

#[test]
fn proposal_owner_rejection_denied() {
    ExtBuilder::default().build().execute_with(|| {
        let alice: User = User::new(AccountKeyring::Alice);

        let bob_key = AccountKeyring::Bob.to_account_id();
        let bob = Origin::signed(bob_key.clone());

        let dave_key = AccountKeyring::Dave.to_account_id();

        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());

        let eve_key = AccountKeyring::Eve.to_account_id();

        // Creates a multi-signature
        let ms_address =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id()).unwrap();
        setup_multisig(
            alice.origin(),
            3,
            vec![ferdie_key.clone(), bob_key.clone(), dave_key, eve_key],
        );

        // Creates a proposal
        let call1 = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 4 },
        ));
        assert_ok!(MultiSig::create_proposal(
            ferdie.clone(),
            ms_address.clone(),
            call1,
            None,
        ));
        let proposal_id = MultiSig::ms_tx_done(ms_address.clone()) - 1;

        // The owner of the proposal shouldn't be able to reject it since bob has already voted
        assert_ok!(MultiSig::reject(
            bob.clone(),
            ms_address.clone(),
            proposal_id
        ));
        assert_noop!(
            MultiSig::reject(ferdie, ms_address.clone(), proposal_id),
            Error::AlreadyVoted
        );

        // The proposal status must be set to Active
        let vote_count = ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        let proposal_state = ProposalStates::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        assert_eq!(proposal_state, ProposalState::Active { until: None });
        assert_eq!(vote_count.approvals, 1);
        assert_eq!(vote_count.rejections, 1);
        assert_eq!(
            Votes::<TestStorage>::get((&ms_address, proposal_id), ferdie_key),
            true
        );
        assert_eq!(
            Votes::<TestStorage>::get((&ms_address, proposal_id), bob_key),
            true
        );

        // No user should be able to change their vote
        assert_noop!(
            MultiSig::reject(bob, ms_address.clone(), proposal_id),
            Error::AlreadyVoted
        );
    });
}

fn expired_proposals() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());

        let bob_key = AccountKeyring::Bob.to_account_id();
        let bob = Origin::signed(bob_key.clone());

        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());

        let charlie_key = AccountKeyring::Charlie.to_account_id();
        let charlie = Origin::signed(charlie_key.clone());

        let ms_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
            .expect("Next MS");

        setup_multisig(alice.clone(), 3, vec![ferdie_key, bob_key, charlie_key]);

        let expires_at = 100u64;
        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 2 },
        ));

        assert_ok!(MultiSig::create_proposal(
            ferdie.clone(),
            ms_address.clone(),
            call,
            Some(100u64),
        ));

        let proposal_id = MultiSig::ms_tx_done(ms_address.clone()) - 1;
        let mut vote_count =
            ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        let mut proposal_state =
            ProposalStates::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        assert_eq!(vote_count.approvals, 1);
        assert_eq!(
            proposal_state,
            ProposalState::Active {
                until: Some(100u64)
            }
        );

        assert_ok!(MultiSig::approve(
            bob.clone(),
            ms_address.clone(),
            proposal_id,
            Weight::MAX
        ));

        vote_count = ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        proposal_state = ProposalStates::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        assert_eq!(vote_count.approvals, 2);
        assert_eq!(
            proposal_state,
            ProposalState::Active {
                until: Some(100u64)
            }
        );

        // Approval fails when proposal has expired
        set_timestamp(expires_at);
        assert_noop!(
            MultiSig::approve(
                charlie.clone(),
                ms_address.clone(),
                proposal_id,
                Weight::MAX
            ),
            Error::ProposalExpired
        );

        vote_count = ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        proposal_state = ProposalStates::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        assert_eq!(vote_count.approvals, 2);
        assert_eq!(
            proposal_state,
            ProposalState::Active {
                until: Some(100u64)
            }
        );

        // Approval works when time is expiry - 1
        set_timestamp(expires_at - 1);
        assert_ok!(MultiSig::approve(
            charlie,
            ms_address.clone(),
            proposal_id,
            Weight::MAX
        ));

        vote_count = ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        proposal_state = ProposalStates::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        assert_eq!(vote_count.approvals, 3);
        assert_eq!(proposal_state, ProposalState::ExecutionSuccessful);
    });
}

#[test]
fn multisig_proposal_nesting_not_allowed() {
    ExtBuilder::default().build().execute_with(|| {
        let _alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let dave = Origin::signed(AccountKeyring::Dave.to_account_id());
        let dave_signer = AccountKeyring::Dave.to_account_id();

        // Created the first top-level multisig.
        let ms1_address =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
                .expect("Next MS");
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![dave_signer.clone()],
            1,
        ));
        let auth_id = get_last_auth_id(&dave_signer);
        assert_ok!(MultiSig::accept_multisig_signer(dave.clone(), auth_id));

        assert_eq!(
            MultiSig::ms_signers(ms1_address.clone(), dave_signer.clone()),
            true
        );

        // Create another multisig with `ms1_address` as a signer.
        let ms2_address =
            MultiSig::get_next_multisig_address(AccountKeyring::Alice.to_account_id())
                .expect("Next MS");
        assert_ok!(MultiSig::create_multisig(
            alice.clone(),
            vec![ms1_address.clone()],
            1,
        ));

        let auth_id = get_last_auth_id(&ms1_address);
        let call_accept_auth = Box::new(RuntimeCall::MultiSig(
            multisig::Call::accept_multisig_signer { auth_id },
        ));

        assert_ok!(MultiSig::create_proposal(
            dave.clone(),
            ms1_address.clone(),
            call_accept_auth,
            None,
        ));

        // Check that `ms1_address` is now a signer for `ms2_address`.
        assert_eq!(
            MultiSig::ms_signers(ms2_address.clone(), ms1_address.clone()),
            true
        );

        // Try to nest a proposal execution.
        let nested_call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::add_multisig_signers {
                signers: vec![bob_signer.clone()],
            },
        ));
        let call_create_proposal =
            Box::new(RuntimeCall::MultiSig(multisig::Call::create_proposal {
                multisig: ms2_address.clone(),
                proposal: nested_call,
                expiry: None,
            }));

        assert_ok!(MultiSig::create_proposal(
            dave.clone(),
            ms1_address.clone(),
            call_create_proposal,
            None,
        ));
        // The top-level proposal should execute ok.
        let proposal_id = MultiSig::ms_tx_done(ms1_address.clone()) - 1;
        let proposal_state = ProposalStates::<TestStorage>::get(&ms1_address, proposal_id).unwrap();
        assert_eq!(proposal_state, ProposalState::ExecutionSuccessful);

        // The nested proposal should fail.
        let proposal_id = MultiSig::ms_tx_done(ms2_address.clone()) - 1;
        let proposal_state = ProposalStates::<TestStorage>::get(&ms2_address, proposal_id).unwrap();
        assert_eq!(proposal_state, ProposalState::ExecutionFailed);
    });
}

fn setup_multisig(creator_origin: Origin, sigs_required: u64, signers: Vec<AccountId>) {
    assert_ok!(MultiSig::create_multisig(
        creator_origin,
        signers.clone(),
        sigs_required,
    ));

    for signer in signers {
        let auth_id = get_last_auth_id(&signer);
        assert_ok!(MultiSig::base_accept_multisig_signer(signer, auth_id));
    }
}
