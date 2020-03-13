use crate::identity::Trait as IdentityTrait;

use polymesh_primitives::IdentityId;

use codec::{Decode, Encode};
use frame_support::{
    decl_event,
    traits::{ChangeMembers, InitializeMembers},
};
use sp_runtime::traits::EnsureOrigin;
use sp_std::{
    cmp::{Eq, Ordering, PartialEq},
    vec::Vec,
};

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct InactiveMember<Moment> {
    pub id: IdentityId,
    pub deactivated_at: Moment,
    pub expiry: Option<Moment>,
}

impl<M> PartialOrd for InactiveMember<M>
where
    M: Eq,
{
    fn partial_cmp(&self, other: &InactiveMember<M>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<M> PartialOrd<IdentityId> for InactiveMember<M>
where
    M: Eq,
{
    fn partial_cmp(&self, other: &IdentityId) -> Option<Ordering> {
        Some(self.id.cmp(other))
    }
}

impl<M> Ord for InactiveMember<M>
where
    M: Eq,
{
    fn cmp(&self, other: &InactiveMember<M>) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<M> PartialEq<IdentityId> for InactiveMember<M> {
    fn eq(&self, other: &IdentityId) -> bool {
        self.id.eq(other)
    }
}

impl<M: Default> From<IdentityId> for InactiveMember<M> {
    fn from(id: IdentityId) -> Self {
        InactiveMember {
            id,
            ..Default::default()
        }
    }
}

pub trait Trait<I>: frame_system::Trait + pallet_timestamp::Trait + IdentityTrait {
    /// The overarching event type.
    type Event: From<Event<Self, I>> + Into<<Self as frame_system::Trait>::Event>;

    /// Required origin for adding a member (though can always be Root).
    type AddOrigin: EnsureOrigin<Self::Origin>;

    /// Required origin for removing a member (though can always be Root).
    type RemoveOrigin: EnsureOrigin<Self::Origin>;

    /// Required origin for adding and removing a member in a single action.
    type SwapOrigin: EnsureOrigin<Self::Origin>;

    /// Required origin for resetting membership.
    type ResetOrigin: EnsureOrigin<Self::Origin>;

    /// The receiver of the signal for when the membership has been initialized. This happens pre-
    /// genesis and will usually be the same as `MembershipChanged`. If you need to do something
    /// different on initialization, then you can change this accordingly.
    type MembershipInitialized: InitializeMembers<IdentityId>;

    /// The receiver of the signal for when the membership has changed.
    type MembershipChanged: ChangeMembers<IdentityId>;
}

decl_event!(
    pub enum Event<T, I> where
    <T as frame_system::Trait>::AccountId,
    <T as Trait<I>>::Event,
    {
        /// The given member was added; see the transaction for who.
        MemberAdded(IdentityId),
        /// The given member was removed; see the transaction for who.
        MemberRemoved(IdentityId),
        /// The given member has been revoked at specific time-stamp.
        MemberRevoked(IdentityId),
        /// Two members were swapped; see the transaction for who.
        MembersSwapped(IdentityId, IdentityId),
        /// The membership was reset; see the transaction for who the new set is.
        MembersReset(Vec<IdentityId>),
        /// Phantom member, never used.
        Dummy(sp_std::marker::PhantomData<(AccountId, Event)>),
    }
);

pub trait GroupTrait<Moment> {
    /// Retrieve members
    fn get_members() -> Vec<IdentityId>;

    /// Retrieve valid members: active and revoked members.
    fn get_inactive_members() -> Vec<InactiveMember<Moment>>;

    #[inline]
    fn get_active_members() -> Vec<IdentityId> {
        Self::get_members()
    }

    /// Current set size
    #[inline]
    fn member_count() -> usize {
        Self::get_members().len()
    }

    #[inline]
    fn is_member(member_id: &IdentityId) -> bool {
        Self::get_members().contains(member_id)
    }
}
