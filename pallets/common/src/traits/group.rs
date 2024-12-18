// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use polymesh_primitives::{traits::group::MemberCount, IdentityId};

use frame_support::{
    decl_event,
    traits::{ChangeMembers, EnsureOrigin, InitializeMembers},
    weights::Weight,
};
use sp_std::vec::Vec;

pub trait WeightInfo {
    fn set_active_members_limit() -> Weight;
    fn add_member() -> Weight;
    fn remove_member() -> Weight;
    fn disable_member() -> Weight;
    fn swap_member() -> Weight;
    fn reset_members(new_members_len: u32) -> Weight;
    fn abdicate_membership() -> Weight;
}

pub trait Config<I>:
    frame_system::Config
    + pallet_permissions::Config
    + pallet_timestamp::Config
    + pallet_identity::Config
{
    /// The overarching event type.
    type RuntimeEvent: From<Event<Self, I>> + Into<<Self as frame_system::Config>::RuntimeEvent>;

    /// Required origin for changing the active limit.
    /// It's recommended that e.g., in case of a committee,
    /// this be an origin that cannot be formed through a committee majority.
    type LimitOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

    /// Required origin for adding a member (though can always be Root).
    type AddOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

    /// Required origin for removing a member (though can always be Root).
    type RemoveOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

    /// Required origin for adding and removing a member in a single action.
    type SwapOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

    /// Required origin for resetting membership.
    type ResetOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

    /// The receiver of the signal for when the membership has been initialized. This happens pre-
    /// genesis and will usually be the same as `MembershipChanged`. If you need to do something
    /// different on initialization, then you can change this accordingly.
    type MembershipInitialized: InitializeMembers<IdentityId>;

    /// The receiver of the signal for when the membership has changed.
    type MembershipChanged: ChangeMembers<IdentityId>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
}

decl_event!(
    pub enum Event<T, I> where
    <T as frame_system::Config>::AccountId,
    <T as Config<I>>::RuntimeEvent,
    {
        /// The given member was added; see the transaction for who.
        /// caller DID, New member DID.
        MemberAdded(IdentityId, IdentityId),
        /// The given member was removed; see the transaction for who.
        /// caller DID, member DID that get removed.
        MemberRemoved(IdentityId, IdentityId),
        /// The given member has been revoked at specific time-stamp.
        /// caller DID, member DID that get revoked.
        MemberRevoked(IdentityId, IdentityId),
        /// Two members were swapped; see the transaction for who.
        /// caller DID, Removed DID, New add DID.
        MembersSwapped(IdentityId, IdentityId, IdentityId),
        /// The membership was reset; see the transaction for who the new set is.
        /// caller DID, List of new members.
        MembersReset(IdentityId, Vec<IdentityId>),
        /// The limit of how many active members there can be concurrently was changed.
        ActiveLimitChanged(IdentityId, MemberCount, MemberCount),
        /// Phantom member, never used.
        Dummy(sp_std::marker::PhantomData<(AccountId, RuntimeEvent)>),
    }
);
