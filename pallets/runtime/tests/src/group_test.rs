use super::{
    storage::{get_identity_id, register_keyring_account, set_curr_did, TestStorage},
    ExtBuilder,
};
use pallet_group::{self as group};
use pallet_identity as identity;
use polymesh_common_utilities::traits::group::GroupTrait;
use polymesh_primitives::IdentityId;

use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use test_client::AccountKeyring;

type CommitteeGroup = group::Module<TestStorage, group::Instance1>;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type Identity = identity::Module<TestStorage>;

#[test]
fn query_membership_works() {
    // TODO(Centril): This `let` is duplicated across the file. Let's dedup.
    let committee = [
        AccountKeyring::Alice.to_account_id(),
        AccountKeyring::Bob.to_account_id(),
    ]
    .to_vec();

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
    let committee = [AccountKeyring::Alice.to_account_id()].to_vec();
    ExtBuilder::default()
        .governance_committee(committee)
        .build()
        .execute_with(add_member_works_we)
}

fn add_member_works_we() {
    let root = Origin::from(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Bob.to_account_id());
    let non_root_did = get_identity_id(AccountKeyring::Alice).unwrap();

    set_curr_did(Some(non_root_did));
    assert_noop!(
        CommitteeGroup::add_member(non_root, IdentityId::from(3)),
        DispatchError::BadOrigin
    );

    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();

    assert_noop!(
        CommitteeGroup::add_member(root.clone(), alice_id),
        group::Error::<TestStorage, group::Instance1>::DuplicateMember
    );
    assert_ok!(CommitteeGroup::add_member(
        root.clone(),
        IdentityId::from(4)
    ));
    assert_eq!(
        CommitteeGroup::get_members(),
        vec![alice_id, IdentityId::from(4)]
    );
}

#[test]
fn active_limit_works() {
    ExtBuilder::default()
        .governance_committee([AccountKeyring::Alice.to_account_id()].to_vec())
        .build()
        .execute_with(|| {
            let root = Origin::from(frame_system::RawOrigin::Root);
            let alice_signer = Origin::signed(AccountKeyring::Alice.to_account_id());
            let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();

            assert_ok!(CommitteeGroup::add_member(
                root.clone(),
                IdentityId::from(4)
            ));
            assert_eq!(
                CommitteeGroup::get_members(),
                vec![alice_id, IdentityId::from(4)]
            );

            let bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();
            let charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
            assert_ok!(CommitteeGroup::set_active_members_limit(root.clone(), 1));
            assert_noop!(
                CommitteeGroup::add_member(root.clone(), bob_id),
                group::Error::<TestStorage, group::Instance1>::ActiveMembersLimitExceeded,
            );
            assert_ok!(CommitteeGroup::set_active_members_limit(root.clone(), 2));
            assert_noop!(
                CommitteeGroup::add_member(root.clone(), bob_id),
                group::Error::<TestStorage, group::Instance1>::ActiveMembersLimitExceeded,
            );
            assert_ok!(CommitteeGroup::set_active_members_limit(root.clone(), 3));
            assert_ok!(CommitteeGroup::add_member(root.clone(), bob_id));
            assert_eq!(
                CommitteeGroup::get_members(),
                vec![alice_id, IdentityId::from(4), bob_id]
            );
            assert_noop!(
                CommitteeGroup::add_member(root.clone(), charlie_id),
                group::Error::<TestStorage, group::Instance1>::ActiveMembersLimitExceeded,
            );

            // Test swap, remove, and abdicate.
            assert_ok!(CommitteeGroup::swap_member(
                root.clone(),
                alice_id,
                charlie_id,
            ));
            assert_ok!(CommitteeGroup::remove_member(root.clone(), charlie_id));
            assert_ok!(CommitteeGroup::add_member(root.clone(), alice_id));
            assert_ok!(CommitteeGroup::abdicate_membership(alice_signer));
            assert_ok!(CommitteeGroup::add_member(root.clone(), alice_id));

            // Lower limit below current size; remove, but then we cannot add.
            assert_ok!(CommitteeGroup::set_active_members_limit(root.clone(), 0));
            assert_ok!(CommitteeGroup::remove_member(root.clone(), alice_id));
            assert_noop!(
                CommitteeGroup::add_member(root.clone(), charlie_id),
                group::Error::<TestStorage, group::Instance1>::ActiveMembersLimitExceeded,
            );

            // Limit is 0, try to reset to empty vec.
            assert_ok!(CommitteeGroup::reset_members(root.clone(), vec![]));
            // Raise to 2, and reset to 2 members, should also work.
            assert_ok!(CommitteeGroup::set_active_members_limit(root.clone(), 2));
            assert_ok!(CommitteeGroup::reset_members(
                root.clone(),
                vec![alice_id, bob_id]
            ));
            // Resetting to 3 members doesn't, however.
            assert_noop!(
                CommitteeGroup::reset_members(root, vec![alice_id, bob_id, charlie_id]),
                group::Error::<TestStorage, group::Instance1>::ActiveMembersLimitExceeded,
            );
        });
}

#[test]
fn remove_member_works() {
    let committee = [
        AccountKeyring::Alice.to_account_id(),
        AccountKeyring::Bob.to_account_id(),
    ]
    .to_vec();

    ExtBuilder::default()
        .governance_committee(committee)
        .build()
        .execute_with(remove_member_works_we)
}

fn remove_member_works_we() {
    let root = Origin::from(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Charlie.to_account_id());

    let non_root_did = get_identity_id(AccountKeyring::Alice).unwrap();

    set_curr_did(Some(non_root_did));

    assert_noop!(
        CommitteeGroup::remove_member(non_root, IdentityId::from(3)),
        DispatchError::BadOrigin
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
    let committee = [
        AccountKeyring::Alice.to_account_id(),
        AccountKeyring::Bob.to_account_id(),
    ]
    .to_vec();

    ExtBuilder::default()
        .governance_committee(committee)
        .build()
        .execute_with(swap_member_works_we);
}

fn swap_member_works_we() {
    let root = Origin::from(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Charlie.to_account_id());
    let alice_id = get_identity_id(AccountKeyring::Alice).unwrap();
    let bob_id = get_identity_id(AccountKeyring::Bob).unwrap();
    let charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let non_root_did = get_identity_id(AccountKeyring::Charlie).unwrap();

    set_curr_did(Some(non_root_did));
    assert_noop!(
        CommitteeGroup::swap_member(non_root, alice_id, IdentityId::from(5)),
        DispatchError::BadOrigin
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
    let committee = [
        AccountKeyring::Alice.to_account_id(),
        AccountKeyring::Bob.to_account_id(),
    ]
    .to_vec();
    ExtBuilder::default()
        .governance_committee(committee)
        .build()
        .execute_with(reset_members_works_we);
}

fn reset_members_works_we() {
    let root = Origin::from(frame_system::RawOrigin::Root);
    let non_root = Origin::signed(AccountKeyring::Bob.to_account_id());
    let new_committee = (4..=6).map(IdentityId::from).collect::<Vec<_>>();
    assert_noop!(
        CommitteeGroup::reset_members(non_root, new_committee.clone()),
        DispatchError::BadOrigin
    );
    assert_ok!(CommitteeGroup::reset_members(root, new_committee.clone()));
    assert_eq!(CommitteeGroup::get_members(), new_committee);
}

#[test]
fn rage_quit() {
    ExtBuilder::default().build().execute_with(rage_quit_we);
}

fn rage_quit_we() {
    let root = Origin::from(frame_system::RawOrigin::Root);

    // 1. Add members to committee
    let alice_signer = Origin::signed(AccountKeyring::Alice.to_account_id());
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let bob_signer = Origin::signed(AccountKeyring::Bob.to_account_id());
    let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
    let charlie_signer = Origin::signed(AccountKeyring::Charlie.to_account_id());
    let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let ferdie_signer = Origin::signed(AccountKeyring::Ferdie.to_account_id());
    let ferdie_did = register_keyring_account(AccountKeyring::Ferdie).unwrap();

    // 0. Threshold is 2/3
    let committee = vec![alice_did, bob_did, charlie_did];
    assert_ok!(CommitteeGroup::reset_members(root.clone(), committee));

    // Ferdie is NOT a member
    assert_eq!(CommitteeGroup::is_member(&ferdie_did), false);
    set_curr_did(Some(ferdie_did));
    assert_noop!(
        CommitteeGroup::abdicate_membership(ferdie_signer),
        group::Error::<TestStorage, group::Instance1>::NoSuchMember
    );

    // Bob quits, its vote should be removed.
    assert_eq!(CommitteeGroup::is_member(&bob_did), true);
    set_curr_did(Some(bob_did));
    assert_ok!(CommitteeGroup::abdicate_membership(bob_signer.clone()));
    assert_eq!(CommitteeGroup::is_member(&bob_did), false);

    // Charlie quits, its vote should be removed and
    // propose should be accepted.
    assert_eq!(CommitteeGroup::is_member(&charlie_did), true);
    set_curr_did(Some(charlie_did));
    assert_ok!(CommitteeGroup::abdicate_membership(charlie_signer.clone()));
    assert_eq!(CommitteeGroup::is_member(&charlie_did), false);

    // Alice should not quit because she is the last member.
    assert_eq!(CommitteeGroup::is_member(&alice_did), true);
    set_curr_did(Some(alice_did));
    assert_noop!(
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
    let root = Origin::from(frame_system::RawOrigin::Root);
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    let charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();

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
        vec![alice_id, bob_id, charlie_id]
    );
}
