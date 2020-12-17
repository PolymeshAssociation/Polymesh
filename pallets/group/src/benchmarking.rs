use crate::*;
use polymesh_common_utilities::{
    benchs::UserBuilder,
    group::{GroupTrait, Trait},
    identity::Trait as IdentityTrait,
    Context,
};

use frame_benchmarking::benchmarks_instance;
use frame_system::RawOrigin;

const MAX_MEMBERS: u32 = 1_000;

fn make_members<T: IdentityTrait + Trait<I>, I: Instance>(m: u32) -> Vec<IdentityId> {
    (0..m)
        .map(|s| {
            let did = UserBuilder::<T>::default()
                .generate_did()
                .seed(s)
                .build("member")
                .did();
            Module::<T, I>::add_member(RawOrigin::Root.into(), did)
                .expect("Member cannot be added");
            did
        })
        .collect::<Vec<_>>()
}

fn inactive_members_contains<T: Trait<I>, I: Instance>(did: &IdentityId) -> bool {
    Module::<T, I>::get_inactive_members()
        .into_iter()
        .map(|m| m.id)
        .find(|m_id| m_id == did)
        .is_some()
}

benchmarks_instance! {
    where_clause {  where T: IdentityTrait }

    _ {}

    set_active_members_limit {
    }: _(RawOrigin::Root, 5u32)
    verify {
        assert_eq!( ActiveMembersLimit::<I>::get(), 5u32);
    }

    add_member {
        let _members = make_members::<T,I>(MAX_MEMBERS-1);
        let new_member = UserBuilder::<T>::default().generate_did().build("new member").did();
    }: _(RawOrigin::Root, new_member)
    verify {
        assert_eq!( Module::<T,I>::get_members().contains(&new_member), true);
    }

    remove_member {
        let members = make_members::<T,I>(MAX_MEMBERS-1);
        let new_member = UserBuilder::<T>::default().generate_did().build("new member").did();
        Module::<T,I>::add_member(RawOrigin::Root.into(), new_member).expect("Member cannot be added");


        // Worst case is when you remove an inactive member, so we disable all members.
        members.iter().chain(&[new_member]).for_each( |did| {
            Module::<T,I>::disable_member(RawOrigin::Root.into(), *did, None, None)
                .expect("Member cannot be disabled");
        });
        Context::set_current_identity::<T::IdentityFn>(Some(new_member));

    }: _(RawOrigin::Root, new_member)
    verify {
        assert_eq!( Module::<T,I>::get_members().contains(&new_member), false);
        assert_eq!( inactive_members_contains::<T,I>(&new_member), false);
    }

    // Worst case where all members have expired.
    disable_member {
        let members = make_members::<T,I>(MAX_MEMBERS);
        let target = members.last().unwrap().clone();
    }: _(RawOrigin::Root, target, None, None)
    verify {
        assert_eq!( Module::<T,I>::get_members().contains(&target), false);
        assert_eq!( inactive_members_contains::<T,I>(&target), true);
    }

    swap_member {
        let members = make_members::<T,I>(MAX_MEMBERS);

        let old_member = members.last().unwrap().clone();
        let new_member = UserBuilder::<T>::default().generate_did().build("new member").did();

    }: _(RawOrigin::Root, old_member, new_member)
    verify {
        assert_eq!( Module::<T,I>::get_members().contains(&new_member), true);
        assert_eq!( Module::<T,I>::get_members().contains(&old_member), false);
        assert_eq!( inactive_members_contains::<T,I>(&old_member), false);
    }

    reset_members {
        let m in 1..MAX_MEMBERS;

        let new_members = (0..m)
            .map(|s| UserBuilder::<T>::default().generate_did().seed(s).build("member").did())
            .collect::<Vec<_>>();
        let mut new_members_exp = new_members.clone();
        new_members_exp.sort();

    }: _(RawOrigin::Root, new_members)
    verify {
        assert_eq!( Module::<T,I>::get_members(), new_members_exp);
    }

    abdicate_membership {
        let members = make_members::<T,I>(MAX_MEMBERS-1);
        let new_member = UserBuilder::<T>::default().generate_did().build("new member");

        Module::<T,I>::add_member( RawOrigin::Root.into(), new_member.did())
            .expect("Member cannot be added");

    }: _(new_member.origin())
    verify {
        assert_eq!( Module::<T,I>::get_members().contains(&new_member.did()), false);
    }
}
