mod common;
use common::{
    storage::{get_identity_id, make_account, register_keyring_account, TestStorage},
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
    let committee = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();

    ExtBuilder::default()
        .governance_committee(committee)
        .build()
        .execute_with(|| {
            let committee = [
                get_identity_id(AccountKeyring::Bob).unwrap(),
                get_identity_id(AccountKeyring::Alice).unwrap(),
            ]
            .to_vec();

            assert_eq!(CommitteeGroup::get_members(), committee);
        });
}

#[test]
fn add_member_works() {
    let committee = [AccountKeyring::Alice.public()].to_vec();
    ExtBuilder::default()
        .governance_committee(committee)
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

    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    assert_noop!(
        CommitteeGroup::add_member(root.clone(), alice_id),
        group::Error::<TestStorage, group::Instance1>::DuplicateMember
    );
    assert_ok!(CommitteeGroup::add_member(root, IdentityId::from(4)));
    assert_eq!(
        CommitteeGroup::get_members(),
        vec![alice_id, IdentityId::from(4)]
    );
}

#[test]
fn remove_member_works() {
    let committee = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();

    ExtBuilder::default()
        .governance_committee(committee)
        .build()
        .execute_with(remove_member_works_we)
}

fn remove_member_works_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Charlie.public());

    assert_noop!(
        CommitteeGroup::remove_member(non_root, IdentityId::from(3)),
        group::Error::<TestStorage, group::Instance1>::BadOrigin
    );
    assert_noop!(
        CommitteeGroup::remove_member(root.clone(), IdentityId::from(5)),
        group::Error::<TestStorage, group::Instance1>::NoSuchMember
    );
    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    let bob_id = get_identity_id(AccountKeyring::Bob).unwrap();
    assert_ok!(CommitteeGroup::remove_member(root, alice_id));
    assert_eq!(CommitteeGroup::get_members(), [bob_id].to_vec());
}

#[test]
fn swap_member_works() {
    let committee = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();

    ExtBuilder::default()
        .governance_committee(committee)
        .build()
        .execute_with(swap_member_works_we);
}

fn swap_member_works_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Charlie.public());
    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    let bob_id = get_identity_id(AccountKeyring::Bob).unwrap();
    let charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();

    assert_noop!(
        CommitteeGroup::swap_member(non_root, alice_id, IdentityId::from(5)),
        group::Error::<TestStorage, group::Instance1>::BadOrigin
    );
    assert_noop!(
        CommitteeGroup::swap_member(root.clone(), IdentityId::from(5), IdentityId::from(6)),
        group::Error::<TestStorage, group::Instance1>::NoSuchMember
    );
    assert_noop!(
        CommitteeGroup::swap_member(root.clone(), alice_id, bob_id),
        group::Error::<TestStorage, group::Instance1>::DuplicateMember
    );
    assert_ok!(CommitteeGroup::swap_member(root.clone(), bob_id, bob_id));
    assert_eq!(CommitteeGroup::get_members(), [bob_id, alice_id].to_vec());
    assert_ok!(CommitteeGroup::swap_member(
        root.clone(),
        alice_id,
        charlie_id
    ));
    assert_eq!(CommitteeGroup::get_members(), [bob_id, charlie_id].to_vec());
}

#[test]
fn reset_members_works() {
    let committee = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();
    ExtBuilder::default()
        .governance_committee(committee)
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
    assert_eq!(CommitteeGroup::get_members(), new_committee);
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

#[test]
fn disable_member() {
    ExtBuilder::default()
        .build()
        .execute_with(disable_member_we);
}

fn disable_member_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);

    let alice_acc = AccountKeyring::Alice.public();
    let (_, alice_id) = make_account(alice_acc).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (_, bob_id) = make_account(bob_acc).unwrap();
    let charlie_acc = AccountKeyring::Charlie.public();
    let (_, charlie_id) = make_account(charlie_acc).unwrap();

    // 0. Create group
    let mut committee = vec![alice_id, bob_id, charlie_id];
    committee.sort();
    assert_ok!(CommitteeGroup::reset_members(
        root.clone(),
        committee.clone()
    ));

    assert_eq!(CommitteeGroup::get_members(), committee);

    // Revoke.
    assert_ok!(CommitteeGroup::disable_member(
        root.clone(),
        bob_id,
        None,
        None
    ));
    assert_eq!(CommitteeGroup::get_members(), vec![charlie_id, alice_id]);
    assert_eq!(
        CommitteeGroup::get_valid_members(),
        vec![charlie_id, alice_id, bob_id]
    );

    // Revoke at
    assert_ok!(CommitteeGroup::disable_member(
        root.clone(),
        charlie_id,
        Some(10),
        None
    ));
    assert_eq!(CommitteeGroup::get_members(), vec![alice_id]);
    assert_eq!(
        CommitteeGroup::get_valid_members_at(10),
        vec![alice_id, bob_id]
    );
    assert_eq!(
        CommitteeGroup::get_valid_members_at(9),
        vec![alice_id, charlie_id, bob_id]
    );
}
