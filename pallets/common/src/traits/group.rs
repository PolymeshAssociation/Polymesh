use frame_support::{
    decl_event,
    traits::{ChangeMembers, InitializeMembers},
};
use polymesh_primitives::IdentityId;
use sp_runtime::traits::EnsureOrigin;
use sp_std::vec::Vec;

pub trait Trait<I>: frame_system::Trait {
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
        /// Two members were swapped; see the transaction for who.
        MembersSwapped(IdentityId, IdentityId),
        /// The membership was reset; see the transaction for who the new set is.
        MembersReset(Vec<IdentityId>),
        /// Phantom member, never used.
        Dummy(sp_std::marker::PhantomData<(AccountId, Event)>),
    }
);

pub trait GroupTrait {
    fn get_members() -> Vec<IdentityId>;

    /// Is the given `MemberId` a valid member?
    fn is_member(member_id: &IdentityId) -> bool;
}
