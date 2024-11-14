use frame_support::{
    assert_err, assert_err_ignore_postinfo, assert_noop, assert_ok, assert_storage_noop,
    dispatch::DispatchResult, BoundedVec,
};

use pallet_multisig::{
    self as multisig, AdminDid, LastInvalidProposal, ProposalStates, ProposalVoteCounts, Votes,
};
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_primitives::multisig::ProposalState;
use polymesh_primitives::{AccountId, AuthorizationData, Permissions, SecondaryKey, Signatory};
use sp_keyring::AccountKeyring;

use super::asset_test::set_timestamp;
use super::next_block;
use super::storage::{
    add_secondary_key, get_primary_key, get_secondary_keys, RuntimeCall, TestStorage, User,
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

pub fn create_signers(
    signers: Vec<AccountId>,
) -> BoundedVec<AccountId, <TestStorage as pallet_multisig::Config>::MaxSigners> {
    signers.try_into().unwrap()
}

fn make_multisig_primary(primary: User, ms_address: AccountId) -> DispatchResult {
    let auth_id = Identity::add_auth(
        primary.did,
        Signatory::Account(ms_address.clone()),
        AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
        None,
    )?;
    let ms_origin = Origin::signed(ms_address);
    Identity::rotate_primary_key_to_secondary(ms_origin, auth_id, None)?;
    Ok(())
}

#[track_caller]
pub fn create_multisig_default_perms(
    caller: AccountId,
    signers: BoundedVec<AccountId, <TestStorage as pallet_multisig::Config>::MaxSigners>,
    num_sigs: u64,
) -> AccountId {
    let origin = Origin::signed(caller.clone());
    let did = Identity::get_identity(&caller).expect("User missing identity");
    let ms_address = MultiSig::get_next_multisig_address(caller).expect("Next MS");
    let perms = Permissions::default();
    assert_ok!(MultiSig::create_multisig(
        origin,
        signers,
        num_sigs,
        Some(perms)
    ));
    assert_ok!(MultiSig::add_admin(Origin::signed(ms_address.clone()), did,));
    ms_address
}

fn create_multisig_result(
    caller: AccountId,
    signers: BoundedVec<AccountId, <TestStorage as pallet_multisig::Config>::MaxSigners>,
    num_sigs: u64,
) -> DispatchResult {
    let origin = Origin::signed(caller);
    MultiSig::create_multisig(origin, signers, num_sigs, None).map_err(|e| e.error)?;
    Ok(())
}

#[test]
fn create_multisig_required_signers() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let eve_signer = AccountKeyring::Eve.to_account_id();

        let ms_address = MultiSig::get_next_multisig_address(alice.acc()).expect("Next MS");

        let signers = || create_signers(vec![eve_signer.clone(), bob_signer.clone()]);
        let create = |signers, nsigs| create_multisig_result(alice.acc(), signers, nsigs);

        assert_ok!(create(signers(), 1));
        assert_eq!(MultiSig::ms_signs_required(ms_address), 1);

        assert_noop!(create(create_signers(vec![]), 10), Error::NotEnoughSigners);
        assert_noop!(create(signers(), 0), Error::RequiredSignersIsZero);
        assert_noop!(create(signers(), 10), Error::NotEnoughSigners);
    });
}

#[test]
fn join_multisig() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);

        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());
        let ferdie_signer = ferdie_key;

        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = User::new(AccountKeyring::Charlie);

        // Add dave's key as a secondary key of charlie.
        let dave = User::new_with(charlie.did, AccountKeyring::Dave);
        add_secondary_key(charlie.did, dave.acc());

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer.clone(), bob_signer.clone()]),
            1,
        );

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

        create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer.clone(), bob_signer.clone()]),
            1,
        );

        let bob_auth_id2 = get_last_auth_id(&bob_signer);
        assert_eq!(
            MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id2),
            Err(Error::SignerAlreadyLinkedToMultisig.into()),
        );

        create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer.clone(), dave.acc()]),
            1,
        );

        // Testing signer key that is already a secondary key on another identity.
        let dave_auth_id = get_last_auth_id(&dave.acc());
        assert_eq!(
            MultiSig::accept_multisig_signer(dave.origin(), dave_auth_id),
            Err(Error::SignerAlreadyLinkedToIdentity.into()),
        );

        let ms_address =
            create_multisig_default_perms(alice.acc(), create_signers(vec![ms_address.clone()]), 1);

        // Testing that a multisig can't add itself as a signer.
        let ms_auth_id = Identity::add_auth(
            alice.did,
            Signatory::Account(ms_address.clone()),
            AuthorizationData::AddMultiSigSigner(ms_address.clone()),
            None,
        )
        .unwrap();

        assert_eq!(
            MultiSig::accept_multisig_signer(Origin::signed(ms_address.clone()), ms_auth_id),
            Err(Error::NestingNotAllowed.into()),
        );
    });
}

#[test]
fn change_multisig_sigs_required() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![charlie_signer.clone(), bob_signer.clone()]),
            2,
        );

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
            None
        ));
        next_block();
        assert_eq!(MultiSig::ms_signs_required(ms_address), 1);
    });
}

#[test]
fn remove_multisig_signers() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![charlie_signer.clone(), bob_signer.clone()]),
            1,
        );

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

        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::remove_multisig_signers {
                signers: create_signers(vec![bob_signer.clone()]),
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
                signers: create_signers(vec![charlie_signer.clone()]),
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
        let alice = User::new(AccountKeyring::Alice);
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = Origin::signed(AccountKeyring::Dave.to_account_id());
        let dave_signer = AccountKeyring::Dave.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![dave_signer.clone()]),
            1,
        );

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
                signers: create_signers(vec![bob_signer.clone()]),
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
                signers: create_signers(vec![charlie_signer.clone()]),
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

        assert_ok!(make_multisig_primary(alice, ms_address.clone()));

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
fn multisig_as_primary() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let ferdie_key = AccountKeyring::Ferdie.to_account_id();

        let ms_address =
            create_multisig_default_perms(alice.acc(), create_signers(vec![ferdie_key]), 1);

        assert_eq!(
            get_primary_key(alice.did),
            AccountKeyring::Alice.to_account_id()
        );

        assert_err!(
            make_multisig_primary(bob, ms_address.clone()),
            IdError::AlreadyLinked
        );

        assert_ok!(make_multisig_primary(alice, ms_address.clone()));

        assert_eq!(get_primary_key(alice.did), ms_address);
    });
}

#[test]
fn rotate_multisig_primary_key_with_balance() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let dave_key = AccountKeyring::Dave.to_account_id();
        let charlie_key = AccountKeyring::Charlie.to_account_id();

        let ms_address =
            create_multisig_default_perms(alice.acc(), create_signers(vec![dave_key]), 1);

        // Alice's primary key hasn't changed.
        assert_eq!(get_primary_key(alice.did), alice.acc());

        // Bob can't make the MultiSig account his primary key.
        assert_err!(
            make_multisig_primary(bob, ms_address.clone()),
            IdError::AlreadyLinked
        );

        // Make the MultiSig account Alice's primary key.
        assert_ok!(make_multisig_primary(alice, ms_address.clone()));

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

        // Succeeds
        assert_ok!(Identity::accept_primary_key(
            Origin::signed(charlie_key.clone()),
            auth_id,
            None
        ));
    });
}

#[test]
fn multisig_as_secondary_key_default_perms() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let ferdie_key = AccountKeyring::Ferdie.to_account_id();

        let ms_address =
            create_multisig_default_perms(alice.acc(), create_signers(vec![ferdie_key]), 1);
        // The desired secondary key record.
        let multisig_secondary = SecondaryKey::new(ms_address.clone(), Permissions::default());
        assert!(get_secondary_keys(alice.did).contains(&multisig_secondary));
    });
}

#[test]
fn remove_multisig_signers_via_admin() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = User::new(AccountKeyring::Dave);

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![charlie_signer.clone(), bob_signer.clone()]),
            1,
        );

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
            MultiSig::remove_multisig_signers_via_admin(
                bob.clone(),
                ms_address.clone(),
                create_signers(vec![bob_signer.clone()])
            ),
            IdError::KeyNotAllowed
        );

        assert_noop!(
            MultiSig::remove_multisig_signers_via_admin(
                dave.origin(),
                ms_address.clone(),
                create_signers(vec![bob_signer.clone()])
            ),
            Error::IdentityNotAdmin
        );

        assert_ok!(MultiSig::remove_multisig_signers_via_admin(
            alice.origin(),
            ms_address.clone(),
            create_signers(vec![bob_signer.clone()])
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
            MultiSig::remove_multisig_signers_via_admin(
                alice.origin(),
                ms_address.clone(),
                create_signers(vec![charlie_signer.clone()])
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
fn add_multisig_signers_via_admin() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = User::new(AccountKeyring::Dave);

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![charlie_signer.clone()]),
            1,
        );

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
            MultiSig::add_multisig_signers_via_admin(
                bob.clone(),
                ms_address.clone(),
                create_signers(vec![bob_signer.clone()])
            ),
            pallet_permissions::Error::<TestStorage>::UnauthorizedCaller
        );

        assert_noop!(
            MultiSig::add_multisig_signers_via_admin(
                dave.origin(),
                ms_address.clone(),
                create_signers(vec![bob_signer.clone()])
            ),
            Error::IdentityNotAdmin
        );

        assert_ok!(MultiSig::add_multisig_signers_via_admin(
            alice.origin(),
            ms_address.clone(),
            create_signers(vec![bob_signer.clone()])
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
        let alice = User::new(AccountKeyring::Alice);
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = Origin::signed(AccountKeyring::Dave.to_account_id());
        let dave_signer = AccountKeyring::Dave.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![charlie_signer.clone(), dave_signer.clone()]),
            1,
        );
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
                signers: create_signers(vec![bob_signer.clone()]),
            },
        ));
        assert_ok!(MultiSig::create_proposal(
            charlie.clone(),
            ms_address.clone(),
            call,
            None,
        ));
        next_block();
        let proposal_id = MultiSig::next_proposal_id(ms_address.clone()) - 1;
        let bob_auth_id = get_last_auth_id(&bob_signer.clone());
        let multi_purpose_nonce = Identity::multi_purpose_nonce();

        assert_storage_noop!(assert_err_ignore_postinfo!(
            MultiSig::approve(dave.clone(), ms_address.clone(), proposal_id, None),
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
        let alice = User::new(AccountKeyring::Alice);

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

        let ms_address = setup_multisig(
            alice.acc(),
            3,
            create_signers(vec![ferdie_key, bob_key, charlie_key, dave_key, eve_key]),
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
        let proposal_id1 = MultiSig::next_proposal_id(ms_address.clone()) - 1;
        assert_ok!(MultiSig::create_proposal(
            ferdie.clone(),
            ms_address.clone(),
            call2,
            None,
        ));
        let proposal_id2 = MultiSig::next_proposal_id(ms_address.clone()) - 1;

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
            MultiSig::approve(dave.clone(), ms_address.clone(), proposal_id1, None),
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
            MultiSig::approve(dave.clone(), ms_address.clone(), proposal_id2, None),
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
fn add_signers_via_admin_removed_controls() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer]),
            2,
        );
        MultiSig::remove_admin_via_admin(alice.origin(), ms_address.clone()).unwrap();

        assert_noop!(
            MultiSig::add_multisig_signers_via_admin(
                alice.origin(),
                ms_address,
                create_signers(vec![charlie_signer])
            ),
            Error::IdentityNotAdmin
        );
    });
}

#[test]
fn remove_signers_via_admin_removed_controls() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer, charlie_signer.clone()]),
            2,
        );
        MultiSig::remove_admin_via_admin(alice.origin(), ms_address.clone()).unwrap();

        assert_noop!(
            MultiSig::add_multisig_signers_via_admin(
                alice.origin(),
                ms_address,
                create_signers(vec![charlie_signer])
            ),
            Error::IdentityNotAdmin
        );
    });
}

#[test]
fn change_sigs_required_via_admin_id_not_creator() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = User::new(AccountKeyring::Dave);

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer, charlie_signer]),
            2,
        );

        assert_noop!(
            MultiSig::change_sigs_required_via_admin(bob.clone(), ms_address.clone(), 2),
            pallet_permissions::Error::<TestStorage>::UnauthorizedCaller
        );

        assert_noop!(
            MultiSig::change_sigs_required_via_admin(dave.origin(), ms_address, 2),
            Error::IdentityNotAdmin
        );
    });
}

#[test]
fn change_sigs_required_via_admin_removed_controls() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer, charlie_signer]),
            2,
        );
        MultiSig::remove_admin_via_admin(alice.origin(), ms_address.clone()).unwrap();

        assert_noop!(
            MultiSig::change_sigs_required_via_admin(alice.origin(), ms_address, 2),
            Error::IdentityNotAdmin
        );
    });
}

#[test]
fn change_sigs_required_via_admin_not_enough_signers() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer.clone(), charlie_signer]),
            2,
        );

        // Signers must accept to be added to the multisig account
        let bob_auth_id = get_last_auth_id(&bob_signer);
        MultiSig::accept_multisig_signer(bob, bob_auth_id).unwrap();

        assert_noop!(
            MultiSig::change_sigs_required_via_admin(alice.origin(), ms_address, 4),
            Error::NotEnoughSigners
        );
    });
}

#[test]
fn change_sigs_required_via_admin_successfully() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer.clone(), charlie_signer]),
            2,
        );
        // Signers must accept to be added to the multisig account
        let bob_auth_id = get_last_auth_id(&bob_signer);
        MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id).unwrap();

        assert_ok!(MultiSig::change_sigs_required_via_admin(
            alice.origin(),
            ms_address,
            1
        ));
    });
}

#[test]
fn remove_admin_via_admin_id_not_creator() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();
        let dave = User::new(AccountKeyring::Dave);

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer.clone(), charlie_signer]),
            2,
        );

        assert_noop!(
            MultiSig::remove_admin_via_admin(bob.clone(), ms_address.clone()),
            pallet_permissions::Error::<TestStorage>::UnauthorizedCaller
        );

        assert_noop!(
            MultiSig::remove_admin_via_admin(dave.origin(), ms_address),
            Error::IdentityNotAdmin
        );
    });
}

#[test]
fn remove_admin_via_admin_successfully() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer.clone(), charlie_signer]),
            2,
        );

        assert_ok!(MultiSig::remove_admin_via_admin(
            alice.origin(),
            ms_address.clone()
        ));
        assert!(AdminDid::<TestStorage>::get(ms_address).is_none())
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
        let ms_address = setup_multisig(
            alice.acc(),
            3,
            create_signers(vec![ferdie_key.clone(), bob_key, dave_key, eve_key]),
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
        let proposal_id = MultiSig::next_proposal_id(ms_address.clone()) - 1;

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
        let ms_address = setup_multisig(
            alice.acc(),
            3,
            create_signers(vec![ferdie_key.clone(), bob_key.clone(), dave_key, eve_key]),
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
        let proposal_id = MultiSig::next_proposal_id(ms_address.clone()) - 1;

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

#[test]
fn remove_admin_successfully() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer.clone(), charlie_signer]),
            2,
        );
        assert!(AdminDid::<TestStorage>::get(&ms_address).is_some());

        assert_ok!(MultiSig::remove_admin(Origin::signed(ms_address.clone())));
        assert!(AdminDid::<TestStorage>::get(ms_address).is_none())
    });
}

#[test]
fn remove_admin_not_multsig() {
    ExtBuilder::default().build().execute_with(|| {
        // Multisig creator
        let alice = User::new(AccountKeyring::Alice);
        // Multisig signers
        let ferdie_signer = AccountKeyring::Ferdie.to_account_id();
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ferdie_signer, bob_signer.clone(), charlie_signer]),
            2,
        );

        assert_noop!(
            MultiSig::remove_admin(alice.origin()),
            Error::NoSuchMultisig
        );
    });
}

fn expired_proposals() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);

        let bob_key = AccountKeyring::Bob.to_account_id();
        let bob = Origin::signed(bob_key.clone());

        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());

        let charlie_key = AccountKeyring::Charlie.to_account_id();
        let charlie = Origin::signed(charlie_key.clone());

        let ms_address = setup_multisig(
            alice.acc(),
            3,
            create_signers(vec![ferdie_key, bob_key, charlie_key]),
        );

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

        let proposal_id = MultiSig::next_proposal_id(ms_address.clone()) - 1;
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
            None
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
            MultiSig::approve(charlie.clone(), ms_address.clone(), proposal_id, None),
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
            None
        ));

        vote_count = ProposalVoteCounts::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        proposal_state = ProposalStates::<TestStorage>::get(&ms_address, proposal_id).unwrap();
        assert_eq!(vote_count.approvals, 3);
        assert_eq!(proposal_state, ProposalState::ExecutionSuccessful);
    });
}

#[test]
fn multisig_nesting_not_allowed() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let dave = Origin::signed(AccountKeyring::Dave.to_account_id());
        let dave_signer = AccountKeyring::Dave.to_account_id();

        // Created the first top-level multisig.
        let ms1_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![dave_signer.clone()]),
            1,
        );
        let auth_id = get_last_auth_id(&dave_signer);
        assert_ok!(MultiSig::accept_multisig_signer(dave.clone(), auth_id));

        assert_eq!(
            MultiSig::ms_signers(ms1_address.clone(), dave_signer.clone()),
            true
        );

        // Create another multisig with `ms1_address` as a signer.
        let ms2_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![ms1_address.clone()]),
            1,
        );

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

        // Ensure that `ms1_address` is not a signer for `ms2_address`.
        assert_eq!(
            MultiSig::ms_signers(ms2_address.clone(), ms1_address.clone()),
            false
        );
    });
}

#[test]
fn create_expired_proposal() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob_key = AccountKeyring::Bob.to_account_id();
        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());
        let charlie_key = AccountKeyring::Charlie.to_account_id();

        let ms_address = setup_multisig(
            alice.acc(),
            3,
            create_signers(vec![ferdie_key, bob_key, charlie_key]),
        );

        let expires_at = 100u64;
        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 2 },
        ));

        set_timestamp(expires_at);

        assert_eq!(
            MultiSig::create_proposal(ferdie.clone(), ms_address.clone(), call, Some(100u64),)
                .unwrap_err()
                .error,
            Error::InvalidExpiryDate.into()
        )
    });
}

#[test]
fn invalidate_proposals_change_sigs_required() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob_key = AccountKeyring::Bob.to_account_id();
        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());
        let charlie_key = AccountKeyring::Charlie.to_account_id();

        let ms_address = setup_multisig(
            alice.acc(),
            3,
            create_signers(vec![ferdie_key, bob_key, charlie_key]),
        );

        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 2 },
        ));

        assert_ok!(MultiSig::create_proposal(
            ferdie.clone(),
            ms_address.clone(),
            call,
            None
        ));

        assert_ok!(MultiSig::change_sigs_required_via_admin(
            alice.origin(),
            ms_address.clone(),
            2
        ));

        assert_eq!(
            LastInvalidProposal::<TestStorage>::get(&ms_address),
            Some(0)
        );

        assert_eq!(
            MultiSig::approve(ferdie.clone(), ms_address.clone(), 0, None)
                .unwrap_err()
                .error,
            Error::InvalidatedProposal.into()
        );
        assert_eq!(
            MultiSig::reject(ferdie, ms_address, 0).unwrap_err().error,
            Error::InvalidatedProposal.into()
        );
    });
}

#[test]
fn invalidate_proposals_add_signer() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob_key = AccountKeyring::Bob.to_account_id();
        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_key = AccountKeyring::Charlie.to_account_id();

        let ms_address = setup_multisig(alice.acc(), 2, create_signers(vec![ferdie_key, bob_key]));

        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 2 },
        ));

        assert_ok!(MultiSig::create_proposal(
            ferdie.clone(),
            ms_address.clone(),
            call,
            None
        ));

        assert_ok!(MultiSig::add_multisig_signers_via_admin(
            alice.origin(),
            ms_address.clone(),
            create_signers(vec![charlie_key.clone()])
        ));

        assert_eq!(LastInvalidProposal::<TestStorage>::get(&ms_address), None);

        let charlie_auth_id = get_last_auth_id(&charlie_key);
        assert_ok!(MultiSig::accept_multisig_signer(
            charlie.clone(),
            charlie_auth_id
        ));

        assert_eq!(
            LastInvalidProposal::<TestStorage>::get(&ms_address),
            Some(0)
        );

        assert_eq!(
            MultiSig::approve(ferdie.clone(), ms_address.clone(), 0, None)
                .unwrap_err()
                .error,
            Error::InvalidatedProposal.into()
        );
        assert_eq!(
            MultiSig::reject(ferdie, ms_address, 0).unwrap_err().error,
            Error::InvalidatedProposal.into()
        );
    });
}

#[test]
fn invalidate_proposals_remove_signer() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob_key = AccountKeyring::Bob.to_account_id();
        let ferdie_key = AccountKeyring::Ferdie.to_account_id();
        let ferdie = Origin::signed(ferdie_key.clone());
        let charlie_key = AccountKeyring::Charlie.to_account_id();

        let ms_address = setup_multisig(
            alice.acc(),
            2,
            create_signers(vec![ferdie_key, bob_key, charlie_key.clone()]),
        );

        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 2 },
        ));

        assert_ok!(MultiSig::create_proposal(
            ferdie.clone(),
            ms_address.clone(),
            call,
            None
        ));

        assert_ok!(MultiSig::remove_multisig_signers_via_admin(
            alice.origin(),
            ms_address.clone(),
            create_signers(vec![charlie_key])
        ));

        assert_eq!(
            LastInvalidProposal::<TestStorage>::get(&ms_address),
            Some(0)
        );

        assert_eq!(
            MultiSig::approve(ferdie.clone(), ms_address.clone(), 0, None)
                .unwrap_err()
                .error,
            Error::InvalidatedProposal.into()
        );
        assert_eq!(
            MultiSig::reject(ferdie, ms_address, 0).unwrap_err().error,
            Error::InvalidatedProposal.into()
        );
    });
}

#[test]
fn invalidate_proposals_via_executed_proposal() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = Origin::signed(AccountKeyring::Bob.to_account_id());
        let bob_signer = AccountKeyring::Bob.to_account_id();
        let charlie = Origin::signed(AccountKeyring::Charlie.to_account_id());
        let charlie_signer = AccountKeyring::Charlie.to_account_id();

        let ms_address = create_multisig_default_perms(
            alice.acc(),
            create_signers(vec![charlie_signer.clone(), bob_signer.clone()]),
            2,
        );

        let charlie_auth_id = get_last_auth_id(&charlie_signer);
        assert_ok!(MultiSig::accept_multisig_signer(
            charlie.clone(),
            charlie_auth_id
        ));

        let bob_auth_id = get_last_auth_id(&bob_signer);
        assert_ok!(MultiSig::accept_multisig_signer(bob.clone(), bob_auth_id));

        let call = Box::new(RuntimeCall::MultiSig(
            multisig::Call::change_sigs_required { sigs_required: 1 },
        ));
        assert_ok!(MultiSig::create_proposal(
            bob.clone(),
            ms_address.clone(),
            call.clone(),
            None,
        ));

        assert_ok!(MultiSig::create_proposal(
            bob.clone(),
            ms_address.clone(),
            call.clone(),
            None,
        ));

        assert_ok!(MultiSig::create_proposal(
            bob.clone(),
            ms_address.clone(),
            call,
            None,
        ));

        assert_ok!(MultiSig::approve(
            charlie.clone(),
            ms_address.clone(),
            0,
            None
        ));

        // At this point the proposal of id 0 executes
        next_block();

        // All other proposals must have been invalidated
        assert_eq!(
            LastInvalidProposal::<TestStorage>::get(&ms_address),
            Some(2)
        );
        assert_eq!(
            MultiSig::approve(charlie.clone(), ms_address.clone(), 1, None)
                .unwrap_err()
                .error,
            Error::InvalidatedProposal.into()
        );
        assert_eq!(
            MultiSig::reject(charlie.clone(), ms_address, 2)
                .unwrap_err()
                .error,
            Error::InvalidatedProposal.into()
        );
    });
}

fn setup_multisig(
    creator: AccountId,
    sigs_required: u64,
    signers: BoundedVec<AccountId, <TestStorage as pallet_multisig::Config>::MaxSigners>,
) -> AccountId {
    let ms_address = create_multisig_default_perms(creator, signers.clone(), sigs_required);

    for signer in signers {
        let auth_id = get_last_auth_id(&signer);
        assert_ok!(MultiSig::accept_multisig_signer(
            Origin::signed(signer),
            auth_id
        ));
    }
    ms_address
}
