//! # Group Module
//!
//! The Group module is used to manage a set of identities. A group of identities can be a
//! collection of CDD providers, council members for governance and so on. This is an instantiable
//! module.
//!
//! ## Overview
//! Allows control of membership of a set of `IdentityId`s, useful for managing membership of a
//! collective.
//!
//! - Add a new identity
//! - Remove identity from the group
//! - Swam members
//! - Reset group members
//!
//! ### Dispatchable Functions
//!
//! - `add_member` - Adds a new identity to the group.
//! - `remove_member` - Remove identity from the group if it exists.
//! - `swap_member` - Replace one identity with the other.
//! - `reset_members` - Re-initialize group members.

#![cfg_attr(not(feature = "std"), no_std)]

use polymesh_primitives::{AccountKey, IdentityId};
pub use polymesh_runtime_common::{
    group::{GroupTrait, RawEvent, Trait},
    Context,
};
use polymesh_runtime_identity as identity;

use frame_support::{
    codec::Encode,
    decl_error, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{ChangeMembers, InitializeMembers},
    weights::SimpleDispatchInfo,
    StorageValue,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::traits::EnsureOrigin;
use sp_std::{convert::TryFrom, prelude::*};

pub type Event<T, I> = polymesh_runtime_common::group::Event<T, I>;
type Identity<T> = identity::Module<T>;

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as Group {
        /// Identities that are part of this group
        pub Members get(fn members) config(): Vec<IdentityId>;
    }
    add_extra_genesis {
        config(phantom): sp_std::marker::PhantomData<(T, I)>;
        build(|config: &Self| {
            let mut members = config.members.clone();
            members.sort();
            T::MembershipInitialized::initialize_members(&members);
            <Members<I>>::put(members);
        })
    }
}

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance=DefaultInstance>
        for enum Call
        where origin: T::Origin
    {
        fn deposit_event() = default;

        /// Add a member `who` to the set. May only be called from `AddOrigin` or root.
        ///
        /// # Arguments
        /// * `origin` Origin representing `AddOrigin` or root
        /// * `who` IdentityId to be added to the group.
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        pub fn add_member(origin, who: IdentityId) {
            T::AddOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;

            let mut members = <Members<I>>::get();
            let location = members.binary_search(&who).err().ok_or("already a member")?;
            members.insert(location, who.clone());
            <Members<I>>::put(&members);

            T::MembershipChanged::change_members_sorted(&[who], &[], &members[..]);

            Self::deposit_event(RawEvent::MemberAdded(who));
        }

        /// Remove a member `who` from the set. May only be called from `RemoveOrigin` or root.
        ///
        /// # Arguments
        /// * `origin` Origin representing `RemoveOrigin` or root
        /// * `who` IdentityId to be removed from the group.
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        pub fn remove_member(origin, who: IdentityId) {
            T::RemoveOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;

            let mut members = <Members<I>>::get();
            let location = members.binary_search(&who).ok().ok_or("not a member")?;
            members.remove(location);
            <Members<I>>::put(&members);

            T::MembershipChanged::change_members_sorted(&[], &[who], &members[..]);

            Self::deposit_event(RawEvent::MemberRemoved(who));
        }

        /// Swap out one member `remove` for another `add`.
        /// May only be called from `SwapOrigin` or root.
        ///
        /// # Arguments
        /// * `origin` Origin representing `SwapOrigin` or root
        /// * `remove` IdentityId to be removed from the group.
        /// * `add` IdentityId to be added in place of `remove`.
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        pub fn swap_member(origin, remove: IdentityId, add: IdentityId) {
            T::SwapOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;

            if remove == add { return Ok(()) }

            let mut members = <Members<I>>::get();

            let location = members.binary_search(&remove).ok().ok_or("not a member")?;
            members[location] = add.clone();

            let _location = members.binary_search(&add).err().ok_or("already a member")?;
            members.sort();
            <Members<I>>::put(&members);

            T::MembershipChanged::change_members_sorted(
                &[add],
                &[remove],
                &members[..],
            );

            Self::deposit_event(RawEvent::MembersSwapped(remove, add));
        }

        /// Change the membership to a new set, disregarding the existing membership.
        /// May only be called from `ResetOrigin` or root.
        ///
        /// # Arguments
        /// * `origin` Origin representing `ResetOrigin` or root
        /// * `members` New set of identities
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        pub fn reset_members(origin, members: Vec<IdentityId>) {
            T::ResetOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;

            let mut new_members = members.clone();
            new_members.sort();
            <Members<I>>::mutate(|m| {
                T::MembershipChanged::set_members_sorted(&members[..], m);
                *m = new_members;
            });

            Self::deposit_event(RawEvent::MembersReset(members));
        }

        /// It allows a caller member to unilaterally quit without this
        /// being subject to a GC vote.
        ///
        /// # Arguments
        /// * `origin` Member of committee who wants to quit.
        /// # Error
        /// * Only master key can abdicate.
        /// * Last member of a group
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn abdicate_membership(origin) -> DispatchResult {
            let who = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let remove_id = Context::current_identity_or::<Identity<T>>(&who)?;

            ensure!(<Identity<T>>::is_master_key(remove_id, &who),
                Error::<T,I>::OnlyMasterKeyAllowed);

            let mut members = Self::members();
            ensure!(members.contains(&remove_id),
                Error::<T,I>::MemberNotFound);
            ensure!( members.len() > 1,
                Error::<T,I>::LastMemberCannotQuit);

            members.retain( |id| *id != remove_id);
            <Members<I>>::put(&members);

            T::MembershipChanged::change_members_sorted(
                &[],
                &[remove_id],
                &members[..],
            );

            Ok(())
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Trait<I>, I: Instance> {
        /// Only master key of the identity is allowed.
        OnlyMasterKeyAllowed,
        /// Sender Identity is not part of the committee.
        MemberNotFound,
        /// Last member of the committee can not quit.
        LastMemberCannotQuit,
    }
}

/// Retrieve all members of this group
/// Is the given `IdentityId` a valid member?
impl<T: Trait<I>, I: Instance> GroupTrait for Module<T, I> {
    fn get_members() -> Vec<IdentityId> {
        return Self::members();
    }

    fn is_member(did: &IdentityId) -> bool {
        Self::members().iter().any(|id| id == did)
    }
}
