use crate::{
    committee::{self, PolymeshVotes, ProportionMatch},
    test::{
        storage::{make_account, Call, TestStorage},
        ExtBuilder,
    },
};
use polymesh_primitives::IdentityId;
use polymesh_runtime_group::{self as group};
use polymesh_runtime_identity as identity;

use frame_support::{assert_err, assert_noop, assert_ok, Hashable};
use test_client::AccountKeyring;

use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};

type Committee = committee::Module<TestStorage, committee::Instance1>;
type CommitteeGroup = group::Module<TestStorage, group::Instance1>;
type System = frame_system::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn motions_basic_environment_works() {
    let committee = (1..=3).map(IdentityId::from).collect::<Vec<_>>();
    ExtBuilder::default()
        .committee_members(committee)
        .build()
        .execute_with(motions_basic_environment_works_we);
}

fn motions_basic_environment_works_we() {
    System::set_block_number(1);

    let committee = (1..=3).map(IdentityId::from).collect::<Vec<_>>();
    assert_eq!(Committee::members(), committee);
    assert_eq!(Committee::proposals(), vec![]);
}

fn make_proposal(value: u64) -> Call {
    // Call::System(frame_system::Call::remark(value.encode()))
    Call::Identity(identity::Call::accept_master_key(value, value))
}

#[test]
fn propose_works() {
    ExtBuilder::default().build().execute_with(propose_works_we);
}

fn propose_works_we() {
    System::set_block_number(1);

    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account(alice_acc).unwrap();

    let root = Origin::system(frame_system::RawOrigin::Root);
    CommitteeGroup::reset_members(root, vec![alice_did]).unwrap();

    let proposal = make_proposal(42);
    let hash = proposal.blake2_256().into();
    assert_ok!(Committee::propose(
        alice_signer.clone(),
        Box::new(proposal.clone())
    ));
    assert_eq!(Committee::proposals(), vec![hash]);
    assert_eq!(Committee::proposal_of(&hash), Some(proposal));
    assert_eq!(
        Committee::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did],
            nays: vec![]
        })
    );
}

#[test]
fn preventing_motions_from_non_members_works() {
    ExtBuilder::default()
        .build()
        .execute_with(preventing_motions_from_non_members_works_we);
}

fn preventing_motions_from_non_members_works_we() {
    System::set_block_number(1);

    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, _) = make_account(alice_acc).unwrap();

    let proposal = make_proposal(42);
    assert_noop!(
        Committee::propose(alice_signer, Box::new(proposal.clone())),
        committee::Error::<TestStorage, committee::Instance1>::NotACommitteeMember
    );
}

#[test]
fn preventing_voting_from_non_members_works() {
    ExtBuilder::default()
        .build()
        .execute_with(preventing_voting_from_non_members_works_we);
}

fn preventing_voting_from_non_members_works_we() {
    System::set_block_number(1);

    let root = Origin::system(frame_system::RawOrigin::Root);
    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account(alice_acc).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (bob_signer, _) = make_account(bob_acc).unwrap();

    CommitteeGroup::reset_members(root, vec![alice_did]).unwrap();

    let proposal = make_proposal(42);
    let hash: H256 = proposal.blake2_256().into();
    assert_ok!(Committee::propose(
        alice_signer.clone(),
        Box::new(proposal.clone())
    ));
    assert_noop!(
        Committee::vote(bob_signer, hash.clone(), 0, true),
        committee::Error::<TestStorage, committee::Instance1>::NotACommitteeMember
    );
}

#[test]
fn motions_ignoring_bad_index_vote_works() {
    ExtBuilder::default()
        .build()
        .execute_with(motions_ignoring_bad_index_vote_works_we);
}

fn motions_ignoring_bad_index_vote_works_we() {
    System::set_block_number(3);

    let root = Origin::system(frame_system::RawOrigin::Root);
    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account(alice_acc).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (bob_signer, bob_did) = make_account(bob_acc).unwrap();

    CommitteeGroup::reset_members(root, vec![alice_did, bob_did]).unwrap();

    let proposal = make_proposal(42);
    let hash: H256 = proposal.blake2_256().into();
    assert_ok!(Committee::propose(
        alice_signer.clone(),
        Box::new(proposal.clone())
    ));
    assert_noop!(
        Committee::vote(bob_signer, hash.clone(), 1, true),
        committee::Error::<TestStorage, committee::Instance1>::MismatchedVotingIndex
    );
}

#[test]
fn motions_revoting_works() {
    ExtBuilder::default()
        .build()
        .execute_with(motions_revoting_works_we);
}

fn motions_revoting_works_we() {
    System::set_block_number(1);

    let root = Origin::system(frame_system::RawOrigin::Root);
    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account(alice_acc).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (_bob_signer, bob_did) = make_account(bob_acc).unwrap();
    let charlie_acc = AccountKeyring::Charlie.public();
    let (_charlie_signer, charlie_did) = make_account(charlie_acc).unwrap();

    CommitteeGroup::reset_members(root, vec![alice_did, bob_did, charlie_did]).unwrap();

    let proposal = make_proposal(42);
    let hash: H256 = proposal.blake2_256().into();
    assert_ok!(Committee::propose(
        alice_signer.clone(),
        Box::new(proposal.clone())
    ));
    assert_eq!(
        Committee::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did],
            nays: vec![]
        })
    );
    assert_noop!(
        Committee::vote(alice_signer.clone(), hash.clone(), 0, true),
        committee::Error::<TestStorage, committee::Instance1>::DuplicateVote
    );
    assert_ok!(Committee::vote(
        alice_signer.clone(),
        hash.clone(),
        0,
        false
    ));
    assert_eq!(
        Committee::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![],
            nays: vec![alice_did]
        })
    );
    assert_noop!(
        Committee::vote(alice_signer, hash, 0, false),
        committee::Error::<TestStorage, committee::Instance1>::DuplicateVote
    );
}

#[test]
fn voting_works() {
    ExtBuilder::default().build().execute_with(voting_works_we);
}

fn voting_works_we() {
    System::set_block_number(1);

    let root = Origin::system(frame_system::RawOrigin::Root);
    let alice_acc = AccountKeyring::Alice.public();
    let (_alice_signer, alice_did) = make_account(alice_acc).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (bob_signer, bob_did) = make_account(bob_acc).unwrap();
    let charlie_acc = AccountKeyring::Charlie.public();
    let (charlie_signer, charlie_did) = make_account(charlie_acc).unwrap();

    CommitteeGroup::reset_members(root, vec![alice_did, bob_did, charlie_did]).unwrap();

    let proposal = make_proposal(69);
    let hash = BlakeTwo256::hash_of(&proposal);
    assert_ok!(Committee::propose(
        charlie_signer.clone(),
        Box::new(proposal.clone())
    ));
    assert_ok!(Committee::vote(bob_signer.clone(), hash.clone(), 0, false));
    assert_eq!(
        Committee::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![charlie_did],
            nays: vec![bob_did]
        })
    );
}

#[test]
fn changing_vote_threshold_works() {
    ExtBuilder::default()
        .committee_vote_threshold((ProportionMatch::AtLeast, 1, 1))
        .build()
        .execute_with(changing_vote_threshold_works_we);
}

fn changing_vote_threshold_works_we() {
    assert_eq!(
        Committee::vote_threshold(),
        (ProportionMatch::AtLeast, 1, 1)
    );
    assert_ok!(Committee::set_vote_threshold(
        Origin::signed(AccountKeyring::Alice.public()),
        ProportionMatch::AtLeast,
        4,
        17
    ));
    assert_eq!(
        Committee::vote_threshold(),
        (ProportionMatch::AtLeast, 4, 17)
    );
}

#[test]
fn rage_quit() {
    ExtBuilder::default()
        .committee_vote_threshold((ProportionMatch::AtLeast, 2, 3))
        .build()
        .execute_with(rage_quit_we);
}

fn rage_quit_we() {
    // 1. Add members to committee
    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account(alice_acc).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (bob_signer, bob_did) = make_account(bob_acc).unwrap();
    let charlie_acc = AccountKeyring::Charlie.public();
    let (charlie_signer, charlie_did) = make_account(charlie_acc).unwrap();
    let dave_acc = AccountKeyring::Dave.public();
    let (_, dave_did) = make_account(dave_acc).unwrap();
    let ferdie_acc = AccountKeyring::Ferdie.public();
    let (ferdie_signer, ferdie_did) = make_account(ferdie_acc).unwrap();
    let committee = vec![alice_did, bob_did, charlie_did, dave_did];

    let root = Origin::system(frame_system::RawOrigin::Root);
    CommitteeGroup::reset_members(root.clone(), committee).unwrap();

    // Ferdie is NOT a member
    assert_eq!(Committee::is_member(&ferdie_did), false);
    assert_err!(
        CommitteeGroup::abdicate_membership(ferdie_signer),
        group::Error::<TestStorage, group::Instance1>::MemberNotFound
    );

    // Make a proposal... only Alice & Bob approve it.
    let proposal = make_proposal(42);
    let proposal_hash = BlakeTwo256::hash_of(&proposal);
    assert_ok!(Committee::propose(alice_signer.clone(), Box::new(proposal)));
    assert_ok!(Committee::vote(bob_signer.clone(), proposal_hash, 0, true));
    assert_ok!(Committee::vote(
        charlie_signer.clone(),
        proposal_hash,
        0,
        false
    ));
    assert_eq!(
        Committee::voting(&proposal_hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did, bob_did],
            nays: vec![charlie_did]
        })
    );

    // Bob quits, its vote should be removed.
    assert_eq!(Committee::is_member(&bob_did), true);
    assert_ok!(CommitteeGroup::abdicate_membership(bob_signer.clone()));
    assert_eq!(Committee::is_member(&bob_did), false);
    assert_eq!(
        Committee::voting(&proposal_hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did],
            nays: vec![charlie_did]
        })
    );

    // Charlie quits, its vote should be removed and
    // propose should be accepted.
    assert_eq!(Committee::is_member(&charlie_did), true);
    assert_ok!(CommitteeGroup::abdicate_membership(charlie_signer.clone()));
    assert_eq!(Committee::is_member(&charlie_did), false);
    // TODO: Only one member, voting should be approved.
    assert_eq!(
        Committee::voting(&proposal_hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did],
            nays: vec![]
        })
    );

    let committee = vec![alice_did, bob_did, charlie_did];
    CommitteeGroup::reset_members(root, committee).unwrap();
    assert_ok!(Committee::vote(bob_signer.clone(), proposal_hash, 0, true));
    assert_eq!(Committee::voting(&proposal_hash), None);

    // Alice should not quit because she is the last member.
    assert_ok!(CommitteeGroup::abdicate_membership(charlie_signer));
    assert_ok!(CommitteeGroup::abdicate_membership(bob_signer));
    assert_eq!(Committee::is_member(&alice_did), true);
    assert_err!(
        CommitteeGroup::abdicate_membership(alice_signer),
        group::Error::<TestStorage, group::Instance1>::LastMemberCannotQuit
    );
    assert_eq!(Committee::is_member(&alice_did), true);
}
