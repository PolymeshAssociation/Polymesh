//! # Committee Module
//!
//! The Group module is used to manage a set of identities. A group of identities can be a
//! collection of KYC providers, council members for governance and so on. This is an instantiable
//! module.
//!
//! ## Overview
//! Allows control of membership of a set of `IdentityId`s, useful for managing membership of a
//! collective.
//!
//!
//! ### Dispatchable Functions
//!
//!
use primitives::IdentityId;
use rstd::{prelude::*, result};
use sr_primitives::traits::{EnsureOrigin, Hash};
use sr_primitives::weights::SimpleDispatchInfo;
use srml_support::{
    codec::{Decode, Encode},
    decl_event, decl_module, decl_storage,
    dispatch::{Dispatchable, Parameter},
    ensure,
    traits::{ChangeMembers, InitializeMembers},
};
use substrate_primitives::u32_trait::Value as U32;
use system::{self, ensure_root, ensure_signed};

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

/// The number of committee members
pub type MemberCount = u32;

pub trait Trait<I = DefaultInstance>: system::Trait {
    /// The outer origin type.
    type Origin: From<RawOrigin<I>>;

    /// The outer call dispatch type.
    type Proposal: Parameter + Dispatchable<Origin = <Self as Trait<I>>::Origin>;

    /// The outer event type.
    type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;
}

/// Origin for the committee module.
#[derive(PartialEq, Eq, Clone)]
pub enum RawOrigin<I> {
    /// It has been condoned by a M of N members of the committee.
    Members(MemberCount, MemberCount),
    /// Dummy to manage the fact we have instancing.
    _Phantom(rstd::marker::PhantomData<I>),
}

/// Origin for the committee module.
pub type Origin<I = DefaultInstance> = RawOrigin<I>;

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
/// Info for keeping track of a motion being voted on.
pub struct Votes<IdentityId> {
    /// The proposal's unique index.
    index: ProposalIndex,
    /// The number of approval votes that are needed to pass the motion.
    threshold: MemberCount,
    /// The current set of commmittee members that approved it.
    ayes: Vec<IdentityId>,
    /// The current set of commmittee members that rejected it.
    nays: Vec<IdentityId>,
}

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as Committee {
        /// The hashes of the active proposals.
        pub Proposals get(proposals): Vec<T::Hash>;
        /// Actual proposal for a given hash.
        pub ProposalOf get(proposal_of): map T::Hash => Option<<T as Trait<I>>::Proposal>;
        /// Votes on a given proposal, if it is ongoing.
        pub Voting get(voting): map T::Hash => Option<Votes<IdentityId>>;
        /// Proposals so far.
        pub ProposalCount get(proposal_count): u32;
        /// The current members of the committee.
        pub Members get(members) config(): Vec<IdentityId>;
    }
    add_extra_genesis {
        config(phantom): rstd::marker::PhantomData<(T, I)>;
    }
}

decl_event!(
	pub enum Event<T, I=DefaultInstance> where
		<T as system::Trait>::Hash,
	{
		/// A motion (given hash) has been proposed (by given account) with a threshold (given
		/// `MemberCount`).
		Proposed(IdentityId, ProposalIndex, Hash),
		/// A motion (given hash) has been voted on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		Voted(IdentityId, Hash, bool, MemberCount, MemberCount),
		/// A motion was approved by the required threshold.
		Approved(Hash),
		/// A motion was rejected by the required threshold.
		Rejected(Hash),
		/// A motion was executed; `bool` is true if returned without error.
		Executed(Hash, bool),
	}
);

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance=DefaultInstance> for enum Call where origin: <T as system::Trait>::Origin {
        fn deposit_event() = default;

        /// Set the committee's membership manually to `new_members`.
        /// Requires root origin.
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        fn set_members(origin, new_members: Vec<IdentityId>) {
            ensure_root(origin)?;
            let mut new_members = new_members;
            new_members.sort();
            <Members<I>>::mutate(|m| {
                *m = new_members;
            });
        }
    }
}
