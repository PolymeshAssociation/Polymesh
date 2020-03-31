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
//! ## Active and Inactive members
//! There are two kinds of members:
//!  - Active: Members who can *act* on behalf of this group. For instance, any active CDD providers can
//!  generate CDD claims.
//!  - Inactive: Members who were active previously but at some point they were disabled. Each
//!  inactive member has two timestamps:
//!     - `deactivated_at`: It indicates the moment when this member was disabled. Any claim generated *after*
//!     this moment is considered as invalid.
//!     - `expiry`: It is the moment when it should be removed completely from this group. From
//!     that moment any claim is considered invalid (as a group claim).
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
    group::{GroupTrait, InactiveMember, RawEvent, Trait},
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
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::EnsureOrigin;
use sp_std::{convert::TryFrom, prelude::*};

pub type Event<T, I> = polymesh_runtime_common::group::Event<T, I>;
type Identity<T> = identity::Module<T>;

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as Group {
        /// Identities that are part of this group, known as "Active members".
        pub ActiveMembers get(fn active_members) config(): Vec<IdentityId>;
        pub InactiveMembers get(fn inactive_members): Vec<InactiveMember<T::Moment>>;
    }
    add_extra_genesis {
        config(phantom): sp_std::marker::PhantomData<(T, I)>;
        build(|config: &Self| {
            let mut members = config.active_members.clone();
            members.sort();
            T::MembershipInitialized::initialize_members(&members);
            <ActiveMembers<I>>::put(members);
        })
    }
}

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance=DefaultInstance>
        for enum Call
        where origin: T::Origin
    {
        type Error = Error<T, I>;

        fn deposit_event() = default;

        /// It disables a member at specific moment.
        ///
        /// Please note that if member is already revoked (a "valid member"), its revocation
        /// time-stamp will be updated.
        ///
        /// Any disabled member should NOT allow to act like an active member of the group. For
        /// instance, a disabled CDD member should NOT be able to generate a CDD claim. However any
        /// generated claim issued before `at` would be considered as a valid one.
        ///
        /// If you want to invalidate any generated claim, you should use `Self::remove_member`.
        ///
        /// # Arguments
        /// * `at` Revocation time-stamp.
        /// * `who` Target member of the group.
        /// * `expiry` Time-stamp when `who` is removed from CDD. As soon as it is expired, the
        /// generated claims will be "invalid" as `who` is not considered a member of the group.
        pub fn disable_member( origin,
            who: IdentityId,
            expiry: Option<T::Moment>,
            at: Option<T::Moment>
        ) -> DispatchResult {
            T::RemoveOrigin::try_origin(origin).map_err(|_| Error::<T, I>::BadOrigin)?;

            <Self as GroupTrait<T::Moment>>::disable_member(who, expiry, at)
        }

        /// Add a member `who` to the set. May only be called from `AddOrigin` or root.
        ///
        /// # Arguments
        /// * `origin` Origin representing `AddOrigin` or root
        /// * `who` IdentityId to be added to the group.
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        pub fn add_member(origin, who: IdentityId) {
            T::AddOrigin::try_origin(origin).map_err(|_| Error::<T, I>::BadOrigin)?;

            let mut members = <ActiveMembers<I>>::get();
            let location = members.binary_search(&who).err().ok_or(Error::<T, I>::DuplicateMember)?;
            members.insert(location, who.clone());
            <ActiveMembers<I>>::put(&members);

            T::MembershipChanged::change_members_sorted(&[who], &[], &members[..]);

            Self::deposit_event(RawEvent::MemberAdded(who));
        }

        /// Remove a member `who` from the set. May only be called from `RemoveOrigin` or root.
        ///
        /// Any claim previously generated by this member is not valid as a group claim. For
        /// instance, if a CDD member group generated a claim for a target identity and then it is
        /// removed, that claim will be invalid.
        /// In case you want to keep the validity of generated claims, you have to use `Self::disable_member` function
        ///
        /// # Arguments
        /// * `origin` Origin representing `RemoveOrigin` or root
        /// * `who` IdentityId to be removed from the group.
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        pub fn remove_member(origin, who: IdentityId) -> DispatchResult {
            T::RemoveOrigin::try_origin(origin).map_err(|_| Error::<T, I>::BadOrigin)?;
            Self::unsafe_remove_member(who)
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
            T::SwapOrigin::try_origin(origin).map_err(|_| Error::<T, I>::BadOrigin)?;

            if remove == add { return Ok(()) }

            let mut members = <ActiveMembers<I>>::get();
            let remove_location = members.binary_search(&remove).ok().ok_or(Error::<T, I>::NoSuchMember)?;
            let _add_location = members.binary_search(&add).err().ok_or(Error::<T, I>::DuplicateMember)?;
            members[remove_location] = add;
            members.sort();
            <ActiveMembers<I>>::put(&members);

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
            T::ResetOrigin::try_origin(origin).map_err(|_| Error::<T, I>::BadOrigin)?;

            let mut new_members = members.clone();
            new_members.sort();
            <ActiveMembers<I>>::mutate(|m| {
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

            let mut members = Self::get_members();
            ensure!(members.contains(&remove_id),
                Error::<T,I>::NoSuchMember);
            ensure!( members.len() > 1,
                Error::<T,I>::LastMemberCannotQuit);

            members.retain( |id| *id != remove_id);
            <ActiveMembers<I>>::put(&members);

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
        /// Incorrect origin.
        BadOrigin,
        /// Group member was added alredy.
        DuplicateMember,
        /// Can't remove a member that doesnt exist.
        NoSuchMember,
        /// Last member of the committee can not quit.
        LastMemberCannotQuit,
    }
}

impl<T: Trait<I>, I: Instance> Module<T, I> {
    /// It returns the current "active members" and any "valid member" which its revocation
    /// time-stamp is in the future.
    pub fn get_valid_members() -> Vec<IdentityId> {
        let now = <pallet_timestamp::Module<T>>::get();
        Self::get_valid_members_at(now)
    }

    /// Remove a member `who` as "active" or "inactive" member.
    ///
    /// # Arguments
    /// * `who` IdentityId to be removed from the group.
    fn unsafe_remove_member(who: IdentityId) -> DispatchResult {
        Self::unsafe_remove_active_member(who).or(Self::unsafe_remove_inactive_member(who))
    }

    /// Remove `who` as "inactive member"
    ///
    /// # Errors
    /// * `NoSuchMember` if `who` is not part of *inactive members*.
    fn unsafe_remove_inactive_member(who: IdentityId) -> DispatchResult {
        let inactive_who = InactiveMember::<T::Moment>::from(who);
        let mut members = <InactiveMembers<T, I>>::get();
        let position = members
            .binary_search(&inactive_who)
            .ok()
            .ok_or(Error::<T, I>::NoSuchMember)?;

        members.swap_remove(position);

        <InactiveMembers<T, I>>::put(&members);
        Self::deposit_event(RawEvent::MemberRemoved(who));
        Ok(())
    }

    /// Remove `who` as "active member"
    ///
    /// # Errors
    /// * `NoSuchMember` if `who` is not part of *active members*.
    fn unsafe_remove_active_member(who: IdentityId) -> DispatchResult {
        let mut members = <ActiveMembers<I>>::get();
        let location = members
            .binary_search(&who)
            .ok()
            .ok_or(Error::<T, I>::NoSuchMember)?;

        members.remove(location);
        <ActiveMembers<I>>::put(&members);

        T::MembershipChanged::change_members_sorted(&[], &[who], &members[..]);
        Self::deposit_event(RawEvent::MemberRemoved(who));
        Ok(())
    }
}

/// Retrieve all members of this group
/// Is the given `IdentityId` a valid member?
impl<T: Trait<I>, I: Instance> GroupTrait<T::Moment> for Module<T, I> {
    /// It returns only the "active members".
    #[inline]
    fn get_members() -> Vec<IdentityId> {
        Self::active_members()
    }

    /// It returns inactive members who are not expired yet.
    #[inline]
    fn get_inactive_members() -> Vec<InactiveMember<T::Moment>> {
        let now = <pallet_timestamp::Module<T>>::get();
        Self::inactive_members()
            .into_iter()
            .filter(|member| !Self::is_member_expired(member, now))
            .collect::<Vec<_>>()
    }

    fn disable_member(
        who: IdentityId,
        expiry: Option<T::Moment>,
        at: Option<T::Moment>,
    ) -> DispatchResult {
        Self::unsafe_remove_active_member(who)?;

        let deactivated_at = at.unwrap_or_else(|| <pallet_timestamp::Module<T>>::get());
        let inactive_member = InactiveMember {
            id: who,
            expiry,
            deactivated_at,
        };

        <InactiveMembers<T, I>>::mutate(|members| {
            // Remove expired members.
            let now = <pallet_timestamp::Module<T>>::get();
            members.retain(|m| {
                if !Self::is_member_expired(m, now) {
                    true
                } else {
                    Self::deposit_event(RawEvent::MemberRemoved(who));
                    false
                }
            });

            // Update inactive member
            if let Some(idx) = members.binary_search(&inactive_member).ok() {
                members[idx] = inactive_member;
            } else {
                members.push(inactive_member);
                members.sort();
            }
        });

        Self::deposit_event(RawEvent::MemberRevoked(who));
        Ok(())
    }
}
