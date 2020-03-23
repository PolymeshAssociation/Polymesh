//! # Committee Module
//!
//! The Committee module is used to create a committee of members who vote and ratify proposals.
//! This was based on Substrate's `srml-collective` but this module differs in the following way:
//! - Winning proposal is determined by a vote threshold which is set at genesis
//! - Vote threshold can be modified per instance
//! - Membership consists of DIDs
//!
//! ## Overview
//! Allows control of membership of a set of `IdentityId`s, useful for managing membership of a
//! committee.
//! - Add members to committee
//! - Members can propose a dispatchable
//! - Members vote on a proposal.
//! - Proposal automatically dispatches if it meets a vote threshold
//!
//! ### Dispatchable Functions
//! - `propose` - Members can propose a new dispatchable
//! - `vote` - Members vote on proposals which are automatically dispatched if they meet vote threshold
//!
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult, Dispatchable, Parameter},
    ensure,
    traits::{ChangeMembers, InitializeMembers},
    weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_signed};
use polymesh_primitives::{AccountKey, IdentityId, Signatory};
use polymesh_runtime_common::{
    group::{GroupTrait, InactiveMember},
    identity::Trait as IdentityTrait,
    Context,
};
use polymesh_runtime_identity as identity;
use sp_core::u32_trait::Value as U32;
use sp_runtime::traits::{EnsureOrigin, Hash};
use sp_std::{convert::TryFrom, prelude::*, vec};

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

/// The number of committee members
pub type MemberCount = u32;

pub trait Trait<I>: frame_system::Trait + IdentityTrait {
    /// The outer origin type.
    type Origin: From<RawOrigin<Self::AccountId, I>>;

    /// The outer call dispatch type.
    type Proposal: Parameter + Dispatchable<Origin = <Self as Trait<I>>::Origin>;

    /// Required origin for changing behaviour of this module.
    type CommitteeOrigin: EnsureOrigin<<Self as frame_system::Trait>::Origin>;

    /// The outer event type.
    type Event: From<Event<Self, I>> + Into<<Self as frame_system::Trait>::Event>;
}

/// Origin for the committee module.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RawOrigin<AccountId, I> {
    /// It has been condoned by M of N members of this committee.
    Members(MemberCount, MemberCount),
    /// Dummy to manage the fact we have instancing.
    _Phantom(sp_std::marker::PhantomData<(AccountId, I)>),
}

/// Origin for the committee module.
pub type Origin<T, I = DefaultInstance> = RawOrigin<<T as system::Trait>::AccountId, I>;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
/// Info for keeping track of a motion being voted on.
pub struct PolymeshVotes<IdentityId> {
    /// The proposal's unique index.
    pub index: ProposalIndex,
    /// The current set of commmittee members that approved it.
    pub ayes: Vec<IdentityId>,
    /// The current set of commmittee members that rejected it.
    pub nays: Vec<IdentityId>,
}

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as Committee {
        /// The hashes of the active proposals.
        pub Proposals get(fn proposals): Vec<T::Hash>;
        /// Actual proposal for a given hash.
        pub ProposalOf get(fn proposal_of): map T::Hash => Option<<T as Trait<I>>::Proposal>;
        /// PolymeshVotes on a given proposal, if it is ongoing.
        pub Voting get(fn voting): map T::Hash => Option<PolymeshVotes<IdentityId>>;
        /// Proposals so far.
        pub ProposalCount get(fn proposal_count): u32;
        /// The current members of the committee.
        pub Members get(fn members) config(): Vec<IdentityId>;
        /// Vote threshold for an approval.
        pub VoteThreshold get(fn vote_threshold) config(): (u32, u32);
    }
    add_extra_genesis {
        config(phantom): sp_std::marker::PhantomData<(T, I)>;
    }
}

decl_event!(
    pub enum Event<T, I> where
        <T as frame_system::Trait>::Hash,
    {
        /// A motion (given hash) has been proposed (by given account) with a threshold (given
        /// `MemberCount`).
        Proposed(IdentityId, ProposalIndex, Hash),
        /// A motion (given hash) has been voted on by given account, leaving
        /// a tally (yes votes, no votes and total seats given respectively as `MemberCount`).
        Voted(IdentityId, Hash, bool, MemberCount, MemberCount, MemberCount),
        /// A motion was approved by the required threshold with the following
        /// tally (yes votes, no votes and total seats given respectively as `MemberCount`).
        Approved(Hash, MemberCount, MemberCount, MemberCount),
        /// A motion was rejected by the required threshold with the following
        /// tally (yes votes, no votes and total seats given respectively as `MemberCount`).
        Rejected(Hash, MemberCount, MemberCount, MemberCount),
        /// A motion was executed; `bool` is true if returned without error.
        Executed(Hash, bool),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait<I>, I: Instance> {
        /// Duplicate vote ignored
        DuplicateVote,
        /// Only master key of the identity is allowed.
        OnlyMasterKeyAllowed,
        /// Sender Identity is not part of the committee.
        MemberNotFound,
        /// Last member of the committee can not quit.
        LastMemberCannotQuit,
        /// The sender must be a signing key for the DID.
        SenderMustBeSigningKeyForDid,
        /// The proposer or voter is not a committee member.
        BadOrigin,
        /// No such proposal.
        NoSuchProposal,
        /// Duplicate proposal.
        DuplicateProposal,
        /// Mismatched voting index.
        MismatchedVotingIndex,
        /// Proportion must be a rational number.
        InvalidProportion,
    }
}

type Identity<T> = identity::Module<T>;

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance=DefaultInstance> for enum Call where origin: <T as frame_system::Trait>::Origin {

        type Error = Error<T, I>;

        fn deposit_event() = default;

        /// Change the vote threshold the determines the winning proposal. For e.g., for a simple
        /// majority use (1, 2) which represents the inequation ">= 1/2"
        ///
        /// # Arguments
        /// * `match_criteria` One of {AtLeast, MoreThan}
        /// * `n` Numerator of the fraction representing vote threshold
        /// * `d` Denominator of the fraction representing vote threshold
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn set_vote_threshold(origin, n: u32, d: u32) {
            T::CommitteeOrigin::ensure_origin(origin)?;

            // Proportion must be a nrational number
            ensure!(d > 0 && n <= d, Error::<T, I>::InvalidProportion);

            <VoteThreshold<I>>::put((n, d));
        }

        /// Any committee member proposes a dispatchable.
        ///
        /// # Arguments
        /// * `proposal` A dispatchable call
        #[weight = SimpleDispatchInfo::FixedOperational(5_000_000)]
        pub fn propose(origin, proposal: Box<<T as Trait<I>>::Proposal>) {
            let who_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&who_key)?;
            let signer = Signatory::AccountKey(who_key);

            // Ensure sender can sign for the given identity
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &signer),
                Error::<T, I>::SenderMustBeSigningKeyForDid
            );

            // Only committee members can propose
            ensure!(Self::is_member(&did), Error::<T, I>::BadOrigin);

            // Reject duplicate proposals
            let proposal_hash = T::Hashing::hash_of(&proposal);
            ensure!(!<ProposalOf<T, I>>::exists(proposal_hash), Error::<T, I>::DuplicateProposal);

            // If committee is composed of a single member, execite the proposal
            let seats = Self::members().len() as MemberCount;
            if seats < 2 {
                let ok = proposal.dispatch(RawOrigin::Members(1, seats).into()).is_ok();
                Self::deposit_event(RawEvent::Executed(proposal_hash, ok));
            } else {
                let index = Self::proposal_count();
                <ProposalCount<I>>::mutate(|i| *i += 1);
                <Proposals<T, I>>::mutate(|proposals| proposals.push(proposal_hash));
                <ProposalOf<T, I>>::insert(proposal_hash, *proposal);

                let votes = PolymeshVotes { index, ayes: vec![did], nays: vec![] };
                <Voting<T, I>>::insert(proposal_hash, votes);

                Self::deposit_event(RawEvent::Proposed(did, index, proposal_hash));
            }
        }

        /// Member casts a vote.
        ///
        /// # Arguments
        /// * `proposal` Hash of proposal to be voted on
        /// * `index` Proposal index
        /// * `approve` Represents a `for` or `against` vote
        #[weight = SimpleDispatchInfo::FixedOperational(200_000)]
        pub fn vote(origin, proposal: T::Hash, #[compact] index: ProposalIndex, approve: bool) -> DispatchResult {
            let who_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&who_key)?;
            let signer = Signatory::AccountKey(who_key);

            // Ensure sender can sign for the given identity
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &signer),
                Error::<T, I>::SenderMustBeSigningKeyForDid
            );

            // Only committee members can vote
            ensure!(Self::is_member(&did), Error::<T, I>::BadOrigin);

            let mut voting = Self::voting(&proposal).ok_or(Error::<T, I>::NoSuchProposal)?;
            ensure!(voting.index == index, Error::<T, I>::MismatchedVotingIndex);

            let position_yes = voting.ayes.iter().position(|a| a == &did);
            let position_no = voting.nays.iter().position(|a| a == &did);

            if approve {
                ensure!( position_yes.is_none(), Error::<T, I>::DuplicateVote);
                voting.ayes.push(did.clone());

                if let Some(pos) = position_no {
                    voting.nays.swap_remove(pos);
                }
            } else {
                ensure!(position_no.is_none(),  Error::<T, I>::DuplicateVote);
                voting.nays.push(did.clone());

                if let Some(pos) = position_yes {
                    voting.ayes.swap_remove(pos);
                }
            }

            <Voting<T, I>>::insert(&proposal, voting);
            Self::check_proposal_threshold(proposal);
            Ok(())
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
    fn is_threshold_satisfied(votes: u32, total: u32, (n, d): (u32, u32)) -> bool {
        (total < 2) || (votes * d >= n * total)
    }

    /// It removes the `id`'s vote from `proposal` if it exists.
    ///
    /// # Return
    /// It returns true if vote was removed.
    fn remove_vote_from(id: IdentityId, proposal: T::Hash) -> bool {
        let mut is_id_removed = None;
        if let Some(mut voting) = Self::voting(&proposal) {
            // If any element is removed, we have to update `voting`.
            is_id_removed = if let Some(idx) = voting.ayes.iter().position(|a| *a == id) {
                Some(voting.ayes.swap_remove(idx))
            } else if let Some(idx) = voting.nays.iter().position(|a| *a == id) {
                Some(voting.nays.swap_remove(idx))
            } else {
                None
            };

            if is_id_removed.is_some() {
                <Voting<T, I>>::insert(&proposal, voting);
            }
        }

        is_id_removed.is_some()
    }

    /// It accepts/rejects the proposal if its threshold is satisfied.
    fn check_proposal_threshold(proposal: T::Hash) {
        if let Some(voting) = Self::voting(&proposal) {
            let seats = Self::members().len() as MemberCount;
            let yes_votes = voting.ayes.len() as MemberCount;
            let no_votes = voting.nays.len() as MemberCount;

            let threshold = <VoteThreshold<I>>::get();

            let approved = Self::is_threshold_satisfied(yes_votes, seats, threshold);
            let rejected = Self::is_threshold_satisfied(no_votes, seats, threshold);

            if approved || rejected {
                if approved {
                    Self::deposit_event(RawEvent::Approved(proposal, yes_votes, no_votes, seats));

                    // execute motion, assuming it exists.
                    if let Some(p) = <ProposalOf<T, I>>::take(&proposal) {
                        let origin = RawOrigin::Members(yes_votes, seats).into();
                        let ok = p.dispatch(origin).is_ok();
                        Self::deposit_event(RawEvent::Executed(proposal, ok));
                    }
                } else {
                    // rejected
                    Self::deposit_event(RawEvent::Rejected(proposal, yes_votes, no_votes, seats));
                }

                // remove vote
                <Voting<T, I>>::remove(&proposal);
                <Proposals<T, I>>::mutate(|proposals| proposals.retain(|h| h != &proposal));
            }
        }
    }
}

impl<T: Trait<I>, I: Instance> GroupTrait<T::Moment> for Module<T, I> {
    /// Retrieve all members of this committee
    fn get_members() -> Vec<IdentityId> {
        Self::members()
    }

    fn get_inactive_members() -> Vec<InactiveMember<T::Moment>> {
        vec![]
    }

    fn disable_member(
        _who: IdentityId,
        _expiry: Option<T::Moment>,
        _at: Option<T::Moment>,
    ) -> DispatchResult {
        unimplemented!()
    }
}

impl<T: Trait<I>, I: Instance> ChangeMembers<IdentityId> for Module<T, I> {
    fn change_members_sorted(
        _incoming: &[IdentityId],
        outgoing: &[IdentityId],
        new: &[IdentityId],
    ) {
        // remove accounts from all current voting in motions.
        Self::proposals()
            .into_iter()
            .filter(|proposal| {
                outgoing.iter().fold(false, |acc, id| {
                    acc || Self::remove_vote_from(*id, *proposal)
                })
            })
            .for_each(Self::check_proposal_threshold);

        <Members<I>>::put(new);
    }
}

impl<T: Trait<I>, I: Instance> InitializeMembers<IdentityId> for Module<T, I> {
    fn initialize_members(members: &[IdentityId]) {
        if !members.is_empty() {
            assert!(
                <Members<I>>::get().is_empty(),
                "Members are already initialized!"
            );
            <Members<I>>::put(members);
        }
    }
}

pub struct EnsureProportionMoreThan<N: U32, D: U32, AccountId, I = DefaultInstance>(
    sp_std::marker::PhantomData<(N, D, AccountId, I)>,
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
    sp_std::marker::PhantomData<(N, D, AccountId, I)>,
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
