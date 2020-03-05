use crate::test::{
    storage::{make_account, TestStorage},
    ExtBuilder,
};
use polymesh_primitives::IdentityId;
use polymesh_runtime_common::traits::group::GroupTrait;
use polymesh_runtime_group::{self as group};

use frame_support::{assert_err, assert_noop, assert_ok};
use test_client::AccountKeyring;

type CommitteeGroup = group::Module<TestStorage, group::Instance1>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn query_membership_works() {
    let committee = (1..3).map(IdentityId::from).collect::<Vec<_>>();

    ExtBuilder::default()
        .committee_members(committee.clone())
        .build()
        .execute_with(|| {
            assert_eq!(CommitteeGroup::members(), committee);
        });
}

#[test]
fn add_member_works() {
    ExtBuilder::default()
        .committee_members(vec![IdentityId::from(3)])
        .build()
        .execute_with(add_member_works_we)
}

fn add_member_works_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Bob.public());

    assert_noop!(
        CommitteeGroup::add_member(non_root, IdentityId::from(3)),
        group::Error::<TestStorage, group::Instance1>::BadOrigin
    );
    assert_noop!(
        CommitteeGroup::add_member(root.clone(), IdentityId::from(3)),
        group::Error::<TestStorage, group::Instance1>::DuplicateMember
    );
    assert_ok!(CommitteeGroup::add_member(root, IdentityId::from(4)));
    assert_eq!(
        CommitteeGroup::members(),
        vec![IdentityId::from(3), IdentityId::from(4)]
    );
}

#[test]
fn remove_member_works() {
    let committee = (1..=3).map(IdentityId::from).collect::<Vec<_>>();

    ExtBuilder::default()
        .committee_members(committee)
        .build()
        .execute_with(remove_member_works_we)
}

fn remove_member_works_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Bob.public());

    assert_noop!(
        CommitteeGroup::remove_member(non_root, IdentityId::from(3)),
        group::Error::<TestStorage, group::Instance1>::BadOrigin
    );
    assert_noop!(
        CommitteeGroup::remove_member(root.clone(), IdentityId::from(5)),
        group::Error::<TestStorage, group::Instance1>::NoSuchMember
    );
    assert_ok!(CommitteeGroup::remove_member(root, IdentityId::from(3)));
    assert_eq!(
        CommitteeGroup::members(),
        vec![IdentityId::from(1), IdentityId::from(2),]
    );
}

#[test]
fn swap_member_works() {
    let committee = (1..=3).map(IdentityId::from).collect::<Vec<_>>();

    ExtBuilder::default()
        .committee_members(committee)
        .build()
        .execute_with(swap_member_works_we);
}

fn swap_member_works_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Bob.public());

    assert_noop!(
        CommitteeGroup::swap_member(non_root, IdentityId::from(1), IdentityId::from(5)),
        group::Error::<TestStorage, group::Instance1>::BadOrigin
    );
    assert_noop!(
        CommitteeGroup::swap_member(root.clone(), IdentityId::from(5), IdentityId::from(6)),
        group::Error::<TestStorage, group::Instance1>::NoSuchMember
    );
    assert_noop!(
        CommitteeGroup::swap_member(root.clone(), IdentityId::from(1), IdentityId::from(3)),
        group::Error::<TestStorage, group::Instance1>::DuplicateMember
    );
    assert_ok!(CommitteeGroup::swap_member(
        root.clone(),
        IdentityId::from(2),
        IdentityId::from(2)
    ));
    assert_eq!(
        CommitteeGroup::members(),
        (1..=3).map(IdentityId::from).collect::<Vec<_>>()
    );
    assert_ok!(CommitteeGroup::swap_member(
        root.clone(),
        IdentityId::from(1),
        IdentityId::from(6)
    ));
    assert_eq!(
        CommitteeGroup::members(),
        [
            IdentityId::from(2),
            IdentityId::from(3),
            IdentityId::from(6)
        ]
        .to_vec()
    );
}

#[test]
fn reset_members_works() {
    let committee = (1..=3).map(IdentityId::from).collect::<Vec<_>>();
    ExtBuilder::default()
        .committee_members(committee)
        .build()
        .execute_with(reset_members_works_we);
}

fn reset_members_works_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Bob.public());
    let new_committee = (4..=6).map(IdentityId::from).collect::<Vec<_>>();

    assert_noop!(
        CommitteeGroup::reset_members(non_root, new_committee.clone()),
        group::Error::<TestStorage, group::Instance1>::BadOrigin
    );
    assert_ok!(CommitteeGroup::reset_members(root, new_committee.clone()));
    assert_eq!(CommitteeGroup::members(), new_committee);
}

#[test]
fn rage_quit() {
    ExtBuilder::default().build().execute_with(rage_quit_we);
}

fn rage_quit_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);

    // 1. Add members to committee
    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account(alice_acc).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (bob_signer, bob_did) = make_account(bob_acc).unwrap();
    let charlie_acc = AccountKeyring::Charlie.public();
    let (charlie_signer, charlie_did) = make_account(charlie_acc).unwrap();
    let ferdie_acc = AccountKeyring::Ferdie.public();
    let (ferdie_signer, ferdie_did) = make_account(ferdie_acc).unwrap();

    // 0. Threshold is 2/3
    let committee = vec![alice_did, bob_did, charlie_did];
    assert_ok!(CommitteeGroup::reset_members(root.clone(), committee));

    // Ferdie is NOT a member
    assert_eq!(CommitteeGroup::is_member(&ferdie_did), false);
    assert_err!(
        CommitteeGroup::abdicate_membership(ferdie_signer),
        group::Error::<TestStorage, group::Instance1>::NoSuchMember
    );

    // Bob quits, its vote should be removed.
    assert_eq!(CommitteeGroup::is_member(&bob_did), true);
    assert_ok!(CommitteeGroup::abdicate_membership(bob_signer.clone()));
    assert_eq!(CommitteeGroup::is_member(&bob_did), false);

    // Charlie quits, its vote should be removed and
    // propose should be accepted.
    assert_eq!(CommitteeGroup::is_member(&charlie_did), true);
    assert_ok!(CommitteeGroup::abdicate_membership(charlie_signer.clone()));
    assert_eq!(CommitteeGroup::is_member(&charlie_did), false);

    // Alice should not quit because she is the last member.
    assert_eq!(CommitteeGroup::is_member(&alice_did), true);
    assert_err!(
        CommitteeGroup::abdicate_membership(alice_signer),
        group::Error::<TestStorage, group::Instance1>::LastMemberCannotQuit
    );
    assert_eq!(CommitteeGroup::is_member(&alice_did), true);
}
