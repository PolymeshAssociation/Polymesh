use super::{
    storage::{
        fast_forward_to_block, get_identity_id, make_account, register_keyring_account, Call,
        EventTest, TestStorage,
    },
    ExtBuilder,
};
use frame_support::{assert_err, assert_noop, assert_ok, dispatch::DispatchError, Hashable};
use frame_system::{EventRecord, Phase};
use pallet_committee::{self as committee, PolymeshVotes, RawEvent as CommitteeRawEvent};
use pallet_group::{self as group};
use pallet_identity as identity;
use pallet_pips::{
    self as pips, Pip, PipDescription, ProposalState, Referendum, ReferendumState, ReferendumType,
    Url,
};
use polymesh_common_utilities::Context;
use polymesh_primitives::IdentityId;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Committee = committee::Module<TestStorage, committee::Instance1>;
type CommitteeGroup = group::Module<TestStorage, group::Instance1>;
type System = frame_system::Module<TestStorage>;
type Identity = identity::Module<TestStorage>;
type Pips = pips::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn motions_basic_environment_works() {
    let committee = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();
    ExtBuilder::default()
        .governance_committee(committee)
        .build()
        .execute_with(motions_basic_environment_works_we);
}

fn motions_basic_environment_works_we() {
    let committee = [AccountKeyring::Alice, AccountKeyring::Bob]
        .iter()
        .map(|key| get_identity_id(*key).unwrap())
        .collect::<Vec<_>>();

    System::set_block_number(1);
    assert_eq!(Committee::members(), committee);
    assert_eq!(Committee::proposals(), vec![]);
}

fn make_proposal(value: u64) -> Call {
    Call::Identity(identity::Call::accept_master_key(value, Some(value)))
}

#[test]
fn propose_works() {
    ExtBuilder::default().build().execute_with(propose_works_we);
}

fn propose_works_we() {
    System::set_block_number(1);

    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account(alice_acc).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (_, bob_did) = make_account(bob_acc).unwrap();

    let root = Origin::system(frame_system::RawOrigin::Root);
    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    CommitteeGroup::reset_members(root, vec![alice_did, bob_did]).unwrap();
    Context::set_current_identity::<Identity>(None);

    let proposal = make_proposal(42);
    let hash = proposal.blake2_256().into();

    assert_ok!(Committee::propose(
        alice_signer.clone(),
        Box::new(proposal.clone())
    ));
    let block_number = System::block_number();
    assert_eq!(Committee::proposals(), vec![hash]);
    assert_eq!(Committee::proposal_of(&hash), Some(proposal));
    assert_eq!(
        Committee::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did],
            nays: vec![],
            end: block_number
        })
    );
}

#[test]
fn single_member_committee_works() {
    ExtBuilder::default()
        .build()
        .execute_with(single_member_committee_works_we);
}

fn single_member_committee_works_we() {
    System::set_block_number(1);

    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account(alice_acc).unwrap();

    let root = Origin::system(frame_system::RawOrigin::Root);
    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    CommitteeGroup::reset_members(root, vec![alice_did]).unwrap();
    Context::set_current_identity::<Identity>(None);

    // Proposal is executed if committee is comprised of a single member
    let proposal = make_proposal(42);
    let hash: sp_core::H256 = proposal.blake2_256().into();
    assert_ok!(Committee::propose(
        alice_signer.clone(),
        Box::new(proposal.clone())
    ));
    assert_eq!(Committee::proposals(), vec![]);

    let expected_event = EventRecord {
        phase: Phase::ApplyExtrinsic(0),
        event: EventTest::committee_Instance1(CommitteeRawEvent::Executed(
            alice_did,
            hash.clone(),
            false,
        )),
        topics: vec![],
    };

    assert_eq!(System::events().contains(&expected_event), true);
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
        committee::Error::<TestStorage, committee::Instance1>::BadOrigin
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

    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    CommitteeGroup::reset_members(root, vec![alice_did]).unwrap();
    Context::set_current_identity::<Identity>(None);

    let proposal = make_proposal(42);
    let hash: H256 = proposal.blake2_256().into();
    assert_ok!(Committee::propose(
        alice_signer.clone(),
        Box::new(proposal.clone())
    ));
    assert_noop!(
        Committee::vote(bob_signer, hash.clone(), 0, true),
        committee::Error::<TestStorage, committee::Instance1>::BadOrigin
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

    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    CommitteeGroup::reset_members(root, vec![alice_did, bob_did]).unwrap();
    Context::set_current_identity::<Identity>(None);

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

    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    CommitteeGroup::reset_members(root, vec![alice_did, bob_did, charlie_did]).unwrap();
    Context::set_current_identity::<Identity>(None);

    let proposal = make_proposal(42);
    let hash: H256 = proposal.blake2_256().into();
    assert_ok!(Committee::propose(
        alice_signer.clone(),
        Box::new(proposal.clone())
    ));
    let block_number = System::block_number();
    assert_eq!(
        Committee::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did],
            nays: vec![],
            end: block_number
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
    let block_number = System::block_number();
    assert_eq!(
        Committee::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![],
            nays: vec![alice_did],
            end: block_number
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

    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    CommitteeGroup::reset_members(root, vec![alice_did, bob_did, charlie_did]).unwrap();
    Context::set_current_identity::<Identity>(None);

    let proposal = make_proposal(69);
    let hash = BlakeTwo256::hash_of(&proposal);
    assert_ok!(Committee::propose(
        charlie_signer.clone(),
        Box::new(proposal.clone())
    ));
    let block_number = System::block_number();
    assert_eq!(
        Committee::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![charlie_did],
            nays: vec![],
            end: block_number
        })
    );
    assert_ok!(Committee::vote(bob_signer.clone(), hash.clone(), 0, false));
    let block_number = System::block_number();
    assert_eq!(
        Committee::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![charlie_did],
            nays: vec![bob_did],
            end: block_number
        })
    );
}

#[test]
fn changing_vote_threshold_works() {
    ExtBuilder::default()
        .governance_committee_vote_threshold((1, 1))
        .build()
        .execute_with(changing_vote_threshold_works_we);
}

fn changing_vote_threshold_works_we() {
    assert_eq!(Committee::vote_threshold(), (1, 1));
    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    assert_ok!(Committee::set_vote_threshold(
        Origin::system(frame_system::RawOrigin::Root),
        4,
        17
    ));
    assert_eq!(Committee::vote_threshold(), (4, 17));
}

#[test]
fn rage_quit() {
    ExtBuilder::default()
        .governance_committee_vote_threshold((2, 3))
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
    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    CommitteeGroup::reset_members(root.clone(), committee).unwrap();
    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(None);

    assert_ok!(u32::try_from((Committee::members()).len()), 4);
    // Ferdie is NOT a member
    assert_eq!(Committee::is_member(&ferdie_did), false);
    Context::set_current_identity::<Identity>(Some(ferdie_did));
    assert_err!(
        CommitteeGroup::abdicate_membership(ferdie_signer),
        group::Error::<TestStorage, group::Instance1>::NoSuchMember
    );
    Context::set_current_identity::<Identity>(None);

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
    let block_number = System::block_number();
    assert_eq!(
        Committee::voting(&proposal_hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did, bob_did],
            nays: vec![charlie_did],
            end: block_number
        })
    );

    // Bob quits, its vote should be removed.
    assert_ok!(u32::try_from((Committee::members()).len()), 4);
    assert_eq!(Committee::is_member(&bob_did), true);
    Context::set_current_identity::<Identity>(Some(bob_did));
    assert_ok!(CommitteeGroup::abdicate_membership(bob_signer.clone()));
    Context::set_current_identity::<Identity>(None);
    assert_eq!(Committee::is_member(&bob_did), false);
    assert_ok!(u32::try_from((Committee::members()).len()), 3);
    let block_number = System::block_number();
    assert_eq!(
        Committee::voting(&proposal_hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did],
            nays: vec![charlie_did],
            end: block_number
        })
    );

    // Charlie quits, its vote should be removed and
    // propose should be accepted.
    assert_eq!(Committee::is_member(&charlie_did), true);
    Context::set_current_identity::<Identity>(Some(charlie_did));
    assert_ok!(CommitteeGroup::abdicate_membership(charlie_signer.clone()));
    Context::set_current_identity::<Identity>(None);
    assert_ok!(u32::try_from((Committee::members()).len()), 2);
    assert_eq!(Committee::is_member(&charlie_did), false);
    // TODO: Only one member, voting should be approved.
    let block_number = System::block_number();
    assert_eq!(
        Committee::voting(&proposal_hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![alice_did],
            nays: vec![],
            end: block_number
        })
    );

    // Assigning random DID but in Production root will have DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    let committee = vec![alice_did, bob_did, charlie_did];
    CommitteeGroup::reset_members(root, committee).unwrap();
    Context::set_current_identity::<Identity>(Some(bob_did));
    assert_ok!(Committee::vote(bob_signer.clone(), proposal_hash, 0, true));
    Context::set_current_identity::<Identity>(None);
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

#[test]
fn release_coordinator() {
    let committee = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();
    ExtBuilder::default()
        .governance_committee(committee)
        .governance_committee_vote_threshold((2, 3))
        .build()
        .execute_with(release_coordinator_we);
}

fn release_coordinator_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let alice_id = get_identity_id(AccountKeyring::Alice).expect("Alice is part of the committee");
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let bob_id = get_identity_id(AccountKeyring::Bob).expect("Bob is part of the committee");
    let charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();

    assert_eq!(
        Committee::release_coordinator(),
        Some(IdentityId::from(999))
    );

    Context::set_current_identity::<Identity>(Some(alice_id));
    assert_err!(
        Committee::set_release_coordinator(alice.clone(), bob_id),
        DispatchError::BadOrigin
    );
    Context::set_current_identity::<Identity>(None);

    // Assigning the random DID to the Root, In Production Root has valid DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    assert_err!(
        Committee::set_release_coordinator(root.clone(), charlie_id),
        committee::Error::<TestStorage, committee::Instance1>::MemberNotFound
    );

    assert_ok!(Committee::set_release_coordinator(root.clone(), bob_id));
    assert_eq!(Committee::release_coordinator(), Some(bob_id));

    // Bob abdicates
    Context::set_current_identity::<Identity>(Some(bob_id));
    assert_ok!(CommitteeGroup::abdicate_membership(bob));
    assert_eq!(Committee::release_coordinator(), None);

    // Assigning the random DID to the Root, In Production Root has valid DID
    Context::set_current_identity::<Identity>(Some(IdentityId::from(999)));
    assert_ok!(Committee::set_release_coordinator(root.clone(), alice_id));
    assert_eq!(Committee::release_coordinator(), Some(alice_id));
}

#[test]
fn enact_referendum() {
    let committee = vec![
        AccountKeyring::Alice.public(),
        AccountKeyring::Bob.public(),
        AccountKeyring::Charlie.public(),
    ];
    ExtBuilder::default()
        .governance_committee(committee)
        .build()
        .execute_with(enact_referendum_we);
}

fn enact_referendum_we() {
    System::set_block_number(1);

    let proposal = make_proposal(42);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    let alice = AccountKeyring::Alice.public();
    let _alice_id = register_keyring_account(AccountKeyring::Alice);
    let bob = AccountKeyring::Bob.public();
    let _bob_id = register_keyring_account(AccountKeyring::Bob);
    let charlie = AccountKeyring::Charlie.public();
    let _charlie_id = register_keyring_account(AccountKeyring::Charlie);
    let dave = AccountKeyring::Dave.public();
    let _dave_id = register_keyring_account(AccountKeyring::Dave);

    // 1. Create the PIP.
    assert_ok!(Pips::propose(
        Origin::signed(alice),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url.clone()),
        Some(proposal_desc.clone()),
        None
    ));
    assert_eq!(
        Pips::proposals(0),
        Some(Pip {
            id: 0,
            proposal: proposal.clone(),
            state: ProposalState::Pending,
            beneficiaries: None,
        })
    );
    assert_ok!(Pips::fast_track_proposal(Origin::signed(alice), 0));

    // 2. Alice and Bob vote to enact that pip, they are 2/3 of committee.
    assert_ok!(Committee::vote_enact_referendum(Origin::signed(alice), 0));
    assert_err!(
        Committee::vote_enact_referendum(Origin::signed(dave), 0),
        committee::Error::<TestStorage, committee::Instance1>::BadOrigin,
    );
    assert_ok!(Committee::vote_enact_referendum(Origin::signed(bob), 0));
    assert_eq!(
        Pips::proposals(0),
        Some(Pip {
            id: 0,
            proposal,
            state: ProposalState::Referendum,
            beneficiaries: None,
        })
    );
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Scheduled,
            referendum_type: ReferendumType::FastTracked,
            enactment_period: 101,
        })
    );
    // Execute referendum
    fast_forward_to_block(102);
    /*assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Executed,
            referendum_type: ReferendumType::FastTracked,
            enactment_period: 101,
        })
    );*/

    // 3. Invalid referendum.
    assert_err!(
        Committee::vote_enact_referendum(Origin::signed(alice), 1),
        committee::Error::<TestStorage, committee::Instance1>::NoSuchProposal,
    );

    // 4. Reject referendums.
    // Bob and Chalie reject the referendum.
    let proposal_rej = make_proposal(1);
    assert_ok!(Pips::propose(
        Origin::signed(alice),
        Box::new(proposal_rej.clone()),
        50,
        Some(proposal_url),
        Some(proposal_desc),
        None
    ));
    assert_ok!(Pips::fast_track_proposal(Origin::signed(alice), 1));

    assert_ok!(Committee::vote_reject_referendum(Origin::signed(bob), 1));
    assert_ok!(Committee::vote_reject_referendum(
        Origin::signed(charlie),
        1
    ));
    assert_eq!(
        Pips::proposals(1),
        Some(Pip {
            id: 1,
            proposal: proposal_rej,
            state: ProposalState::Referendum,
            beneficiaries: None
        })
    );
    assert_eq!(
        Pips::referendums(1),
        Some(Referendum {
            id: 1,
            state: ReferendumState::Rejected,
            referendum_type: ReferendumType::FastTracked,
            enactment_period: 0,
        })
    );

    assert_err!(
        Committee::vote_enact_referendum(Origin::signed(dave), 1),
        committee::Error::<TestStorage, committee::Instance1>::BadOrigin,
    );
}
