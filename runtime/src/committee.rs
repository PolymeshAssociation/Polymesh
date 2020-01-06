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
use crate::identity;
use primitives::{IdentityId, Key, Signer};
use rstd::{convert::TryFrom, prelude::*, result};
use sr_primitives::traits::{EnsureOrigin, Hash};
use sr_primitives::weights::SimpleDispatchInfo;
#[cfg(feature = "std")]
use sr_primitives::{Deserialize, Serialize};
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

pub trait Trait<I = DefaultInstance>: system::Trait + identity::Trait {
    /// The outer origin type.
    type Origin: From<RawOrigin<Self::AccountId, I>>;

    /// The outer call dispatch type.
    type Proposal: Parameter + Dispatchable<Origin = <Self as Trait<I>>::Origin>;

    /// The outer event type.
    type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ProportionMatch {
    AtLeast,
    MoreThan,
}

impl Default for ProportionMatch {
    fn default() -> Self {
        ProportionMatch::MoreThan
    }
}

/// Origin for the committee module.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RawOrigin<AccountId, I> {
    /// It has been condoned by M of N members of this committee.
    Members(MemberCount, MemberCount),
    /// Dummy to manage the fact we have instancing.
    _Phantom(rstd::marker::PhantomData<(AccountId, I)>),
}

/// Origin for the committee module.
pub type Origin<T, I = DefaultInstance> = RawOrigin<<T as system::Trait>::AccountId, I>;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
/// Info for keeping track of a motion being voted on.
pub struct Votes<IdentityId> {
    /// The proposal's unique index.
    index: ProposalIndex,
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
        /// Vote threshold for an approval.
        pub VoteThreshold get(vote_threshold) config(): (ProportionMatch, u32, u32);
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
        ///
        /// # Arguments
        /// * `origin` Root
        /// * `new_members` Members to be initialized as committee.
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        fn set_members(origin, new_members: Vec<IdentityId>) {
            ensure_root(origin)?;

            let mut new_members = new_members;
            new_members.sort();
            <Members<I>>::mutate(|m| {
                *m = new_members;
            });
        }

        /// Any committee member proposes a dispatchable.
        ///
        /// # Arguments
        /// * `did` Identity of the proposer
        /// * `proposal` A dispatchable call
        #[weight = SimpleDispatchInfo::FixedOperational(5_000_000)]
        fn propose(origin, did: IdentityId, proposal: Box<<T as Trait<I>>::Proposal>) {
            let who = ensure_signed(origin)?;
            let signer = Signer::Key(Key::try_from(who.encode())?);

            // Ensure sender can sign for the given identity
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            // Only committee members can propose
            ensure!(Self::is_member(&did), "proposer is not a member");

            // Reject duplicate proposals
            let proposal_hash = T::Hashing::hash_of(&proposal);
            ensure!(!<ProposalOf<T, I>>::exists(proposal_hash), "duplicate proposals not allowed");

            let index = Self::proposal_count();
            <ProposalCount<I>>::mutate(|i| *i += 1);
            <Proposals<T, I>>::mutate(|proposals| proposals.push(proposal_hash));
            <ProposalOf<T, I>>::insert(proposal_hash, *proposal);

            let votes = Votes { index, ayes: vec![did.clone()], nays: vec![] };
            <Voting<T, I>>::insert(proposal_hash, votes);

            Self::deposit_event(RawEvent::Proposed(did, index, proposal_hash));
        }

        /// Member casts a vote.
        ///
        /// # Arguments
        /// * `did` Identity of the proposer
        /// * `proposal` Hash of proposal to be voted on
        /// * `index` Proposal index
        /// * `approve` Represents a `for` or `against` vote
        #[weight = SimpleDispatchInfo::FixedOperational(200_000)]
        fn vote(origin, did: IdentityId, proposal: T::Hash, #[compact] index: ProposalIndex, approve: bool) {
            let who = ensure_signed(origin)?;
            let signer = Signer::Key(Key::try_from(who.encode())?);

            // Ensure sender can sign for the given identity
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            // Only committee members can vote
            ensure!(Self::is_member(&did), "voter is not a member");

            let mut voting = Self::voting(&proposal).ok_or("proposal must exist")?;
            ensure!(voting.index == index, "mismatched index");

            let position_yes = voting.ayes.iter().position(|a| a == &did);
            let position_no = voting.nays.iter().position(|a| a == &did);

            if approve {
                if position_yes.is_none() {
                    voting.ayes.push(did.clone());
                } else {
                    return Err("duplicate vote ignored")
                }
                if let Some(pos) = position_no {
                    voting.nays.swap_remove(pos);
                }
            } else {
                if position_no.is_none() {
                    voting.nays.push(did.clone());
                } else {
                    return Err("duplicate vote ignored")
                }
                if let Some(pos) = position_yes {
                    voting.ayes.swap_remove(pos);
                }
            }

            let yes_votes = voting.ayes.len() as MemberCount;
            let no_votes = voting.nays.len() as MemberCount;
            Self::deposit_event(RawEvent::Voted(did, proposal, approve, yes_votes, no_votes));

            let threshold = <VoteThreshold<I>>::get();
            let seats = Self::members().len() as MemberCount;

            let approved = Self::is_threshold_satisfied(yes_votes, seats, threshold);
            let rejected = Self::is_threshold_satisfied(no_votes, seats, threshold);

            if approved || rejected {
                if approved {
                    Self::deposit_event(RawEvent::Approved(proposal));

                    // execute motion, assuming it exists.
                    if let Some(p) = <ProposalOf<T, I>>::take(&proposal) {
                        let origin = RawOrigin::Members(yes_votes, seats).into();
                        let ok = p.dispatch(origin).is_ok();
                        Self::deposit_event(RawEvent::Executed(proposal, ok));
                    }
                } else {
                    // rejected
                    Self::deposit_event(RawEvent::Rejected(proposal));
                }

                // remove vote
                <Voting<T, I>>::remove(&proposal);
                <Proposals<T, I>>::mutate(|proposals| proposals.retain(|h| h != &proposal));
            } else {
                // update voting
                <Voting<T, I>>::insert(&proposal, voting);
            }
        }
    }
}

impl<T: Trait<I>, I: Instance> Module<T, I> {
    /// Returns true if given did is contained in `Members` set. `false`, otherwise.
    pub fn is_member(who: &IdentityId) -> bool {
        Self::members().contains(who)
    }

    /// Given `votes` number of votes out of `total` votes, this function compares`votes`/`total`
    /// in relation to the threshold proporion `n`/`d`.   
    fn is_threshold_satisfied(
        votes: u32,
        total: u32,
        (threshold, n, d): (ProportionMatch, u32, u32),
    ) -> bool {
        match threshold {
            ProportionMatch::AtLeast => votes * d >= n * total,
            ProportionMatch::MoreThan => votes * d > n * total,
        }
    }
}

pub struct EnsureProportionMoreThan<N: U32, D: U32, AccountId, I = DefaultInstance>(
    rstd::marker::PhantomData<(N, D, AccountId, I)>,
);
impl<
        O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
        N: U32,
        D: U32,
        AccountId,
        I,
    > EnsureOrigin<O> for EnsureProportionMoreThan<N, D, AccountId, I>
{
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Members(n, m) if n * D::VALUE > N::VALUE * m => Ok(()),
            r => Err(O::from(r)),
        })
    }
}

pub struct EnsureProportionAtLeast<N: U32, D: U32, AccountId, I = DefaultInstance>(
    rstd::marker::PhantomData<(N, D, AccountId, I)>,
);
impl<
        O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
        N: U32,
        D: U32,
        AccountId,
        I,
    > EnsureOrigin<O> for EnsureProportionAtLeast<N, D, AccountId, I>
{
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Members(n, m) if n * D::VALUE >= N::VALUE * m => Ok(()),
            r => Err(O::from(r)),
        })
    }
}
