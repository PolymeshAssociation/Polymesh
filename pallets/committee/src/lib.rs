// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

// Modified by Polymesh Association - 23rd March 2020
// Polymesh changes - This module is inspired from the `pallet-collective`
// https://github.com/PolymeshAssociation/substrate/tree/a439a7aa5a9a3df2a42d9b25ea04288d3a0866e8/frame/collective
// It is modified as per the requirement of the Polymesh
// -`set_members()` dispatchable get removed and members are maintained by the group module
// - New instance of the group module is being added and assigned committee instance to
// `MembershipInitialized` & `MembershipChanged` trait

//! # Committee Module
//!
//! The Committee module is used to create a committee of members who vote and ratify proposals.
//! This was based on Substrate's `pallet-collective` but this module differs in the following way:
//! - The winning proposal is determined by a vote threshold which is set at genesis.
//! - The vote threshold can be modified per instance.
//! - The members are DIDs.
//!
//! ## Overview
//! The module allows to control of membership of a set of
//! [`IdentityId`](../../primitives/struct.IdentityId.html)s, which includes:
//! - changing the members of the committee,
//! - allowing the members to propose a dispatchable,
//! - allowing the members to vote on a proposal,
//! - automatically dispatching a proposal if it meets a vote threshold.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//! - `vote_or_propose` - Members can propose a new dispatchable.
//! - `vote` - Members vote on proposals which are automatically dispatched if they meet vote threshold.
//! - `close` - May be called by any signed account after the voting duration has ended in order to
//! finish voting and close the proposal.
//! - `set_vote_threshold` - Changes the threshold for a committee majority.
//! - `set_release_coordinator` - Changes the release coordinator.
//! - `set_expires_after` - Sets the time after which a proposal expires.
//!
//! ### Other Public Functions
//! - `is_member` - Returns true if a given DID is contained in the set of committee members, and
//! `false` otherwise.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use core::mem;
use frame_support::dispatch::{
    DispatchClass, DispatchResult, GetDispatchInfo, Parameter, PostDispatchInfo,
};
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchError;
use frame_support::traits::{ChangeMembers, EnsureOrigin, InitializeMembers};
use frame_support::weights::Weight;
use scale_info::TypeInfo;
use sp_runtime::traits::{Dispatchable, Hash};
use sp_std::{prelude::*, vec};

use polymesh_primitives::traits::group::{GroupTrait, InactiveMember, MemberCount};
use polymesh_primitives::traits::GovernanceGroupTrait;
use polymesh_primitives::{IdentityId, MaybeBlock, SystematicIssuers, GC_DID};

type IdentityPallet<T> = pallet_identity::Pallet<T>;
pub trait WeightInfo {
    fn set_vote_threshold() -> Weight;
    fn set_release_coordinator() -> Weight;
    fn set_expires_after() -> Weight;
    fn vote_or_propose_new_proposal() -> Weight;
    fn vote_or_propose_existing_proposal() -> Weight;
    fn vote_aye() -> Weight;
    fn vote_nay() -> Weight;
}

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// The committee trait.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config + pallet_identity::Config {
        /// The outer origin type.
        type RuntimeOrigin: From<RawOrigin<Self::AccountId, I>>
            + Into<<Self as frame_system::Config>::RuntimeOrigin>;

        /// The outer call type.
        type Proposal: Parameter
            + Dispatchable<
                RuntimeOrigin = <Self as Config<I>>::RuntimeOrigin,
                PostInfo = PostDispatchInfo,
            > + GetDispatchInfo
            + From<frame_system::Call<Self>>;

        /// Required origin for changing behaviour of this module.
        type CommitteeOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

        /// Required origin for changing the voting threshold.
        type VoteThresholdOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

        /// Weight computation.
        type WeightInfo: WeightInfo;
    }

    /// Origin for the committee module.
    #[derive(Debug, Decode, DecodeWithMemTracking, Encode)]
    #[derive(Clone, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
    #[scale_info(skip_type_params(I))]
    pub enum RawOrigin<AccountId, I> {
        /// It has been condoned by M of N members of this committee
        /// with `M` and `N` set dynamically in `set_vote_threshold`.
        Endorsed(PhantomData<(AccountId, I)>),
    }

    /// Origin for the committee module.
    #[pallet::origin]
    pub type Origin<T, I = ()> = RawOrigin<<T as frame_system::Config>::AccountId, I>;

    #[derive(PartialEq, Eq, Clone, Encode, Decode, TypeInfo, Debug)]
    /// Info for keeping track of a motion being voted on.
    pub struct PolymeshVotes<BlockNumber> {
        /// The proposal's unique index.
        pub index: ProposalIndex,
        /// The current set of committee members that approved it.
        pub ayes: Vec<IdentityId>,
        /// The current set of committee members that rejected it.
        pub nays: Vec<IdentityId>,
        /// The time **at** which the proposal is expired.
        pub expiry: MaybeBlock<BlockNumber>,
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    /// The hashes of the active proposals.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Proposals<T: Config<I>, I: 'static = ()> = StorageValue<_, Vec<T::Hash>, ValueQuery>;

    /// Actual proposal for a given hash.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type ProposalOf<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Identity, T::Hash, <T as Config<I>>::Proposal, OptionQuery>;

    /// PolymeshVotes on a given proposal, if it is ongoing.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Voting<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Identity, T::Hash, PolymeshVotes<BlockNumberFor<T>>, OptionQuery>;

    /// Proposals so far.
    #[pallet::storage]
    pub type ProposalCount<T: Config<I>, I: 'static = ()> = StorageValue<_, u32, ValueQuery>;

    /// The current members of the committee.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Members<T: Config<I>, I: 'static = ()> = StorageValue<_, Vec<IdentityId>, ValueQuery>;

    /// Vote threshold for an approval.
    #[pallet::storage]
    pub type VoteThreshold<T: Config<I>, I: 'static = ()> = StorageValue<_, (u32, u32), ValueQuery>;

    /// Release cooridinator.
    #[pallet::storage]
    pub type ReleaseCoordinator<T: Config<I>, I: 'static = ()> =
        StorageValue<_, IdentityId, OptionQuery>;

    /// Time after which a proposal will expire.
    #[pallet::storage]
    pub type ExpiresAfter<T: Config<I>, I: 'static = ()> =
        StorageValue<_, MaybeBlock<BlockNumberFor<T>>, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub members: Vec<IdentityId>,
        pub vote_threshold: (u32, u32),
        pub release_coordinator: IdentityId,
        pub expires_after: MaybeBlock<BlockNumberFor<T>>,
        #[serde(skip)]
        pub phantom: PhantomData<(T, I)>,
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
        fn build(&self) {
            Members::<T, I>::put(self.members.clone());
            VoteThreshold::<T, I>::put(self.vote_threshold);
            ReleaseCoordinator::<T, I>::put(self.release_coordinator);
            ExpiresAfter::<T, I>::put(self.expires_after);
        }
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            polymesh_primitives::migrate::frame_v2_instance_migrate::<Pallet<T, I>, I>(
                "Committee",
                STORAGE_VERSION,
            )
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// A motion (given hash) has been proposed (by given account) with a threshold (given `MemberCount`).
        /// Parameters: caller DID, proposal index, proposal hash.
        Proposed(IdentityId, ProposalIndex, <T as frame_system::Config>::Hash),
        /// A motion (given hash) has been voted on by given account, leaving
        /// a tally (yes votes, no votes and total seats given respectively as `MemberCount`).
        /// caller DID, Proposal index, Proposal hash, current vote, yay vote count, nay vote count, total seats.
        Voted(
            IdentityId,
            ProposalIndex,
            <T as frame_system::Config>::Hash,
            bool,
            MemberCount,
            MemberCount,
            MemberCount,
        ),
        /// A vote on a motion (given hash) has been retracted.
        /// caller DID, ProposalIndex, Proposal hash, vote that was retracted
        VoteRetracted(
            IdentityId,
            ProposalIndex,
            <T as frame_system::Config>::Hash,
            bool,
        ),
        /// Final votes on a motion (given hash)
        /// caller DID, ProposalIndex, Proposal hash, yes voters, no voter
        FinalVotes(
            Option<IdentityId>,
            ProposalIndex,
            <T as frame_system::Config>::Hash,
            Vec<IdentityId>,
            Vec<IdentityId>,
        ),
        /// A motion was approved by the required threshold with the following
        /// tally (yes votes, no votes and total seats given respectively as `MemberCount`).
        /// Parameters: caller DID, proposal hash, yay vote count, nay vote count, total seats.
        Approved(
            Option<IdentityId>,
            <T as frame_system::Config>::Hash,
            MemberCount,
            MemberCount,
            MemberCount,
        ),
        /// A motion was rejected by the required threshold with the following
        /// tally (yes votes, no votes and total seats given respectively as `MemberCount`).
        /// Parameters: caller DID, proposal hash, yay vote count, nay vote count, total seats.
        Rejected(
            Option<IdentityId>,
            <T as frame_system::Config>::Hash,
            MemberCount,
            MemberCount,
            MemberCount,
        ),
        /// A motion was executed; `DispatchResult` is `Ok(())` if returned without error.
        /// Parameters: caller DID, proposal hash, result of proposal dispatch.
        Executed(
            Option<IdentityId>,
            <T as frame_system::Config>::Hash,
            DispatchResult,
        ),
        /// Release coordinator has been updated.
        /// Parameters: DID of the release coordinator.
        ReleaseCoordinatorUpdated(Option<IdentityId>),
        /// Proposal expiry time has been updated.
        /// Parameters: caller DID, new expiry time (if any).
        ExpiresAfterUpdated(IdentityId, MaybeBlock<BlockNumberFor<T>>),
        /// Voting threshold has been updated
        /// Parameters: caller DID, numerator, denominator
        VoteThresholdUpdated(IdentityId, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Duplicate votes are not allowed.
        DuplicateVote,
        /// A DID isn't part of the committee.
        /// The DID may either be a caller or some other context.
        NotAMember,
        /// No such proposal.
        NoSuchProposal,
        /// Proposal exists, but it has expired.
        ProposalExpired,
        /// Duplicate proposal.
        DuplicateProposal,
        /// Mismatched voting index.
        MismatchedVotingIndex,
        /// Proportion must be a rational number.
        InvalidProportion,
        /// First vote on a proposal creates it, so it must be an approval.
        /// All proposals are motions to execute something as "GC majority".
        /// To reject e.g., a PIP, a motion to reject should be *approved*.
        FirstVoteReject,
        /// Maximum number of proposals has been reached.
        ProposalsLimitReached,
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Change the vote threshold the determines the winning proposal.
        /// For e.g., for a simple majority use (1, 2) which represents the in-equation ">= 1/2".
        ///
        /// # Arguments
        /// * `n` - Numerator of the fraction representing vote threshold.
        /// * `d` - Denominator of the fraction representing vote threshold.
        #[pallet::weight((<T as Config<I>>::WeightInfo::set_vote_threshold(), DispatchClass::Operational))]
        #[pallet::call_index(0)]
        pub fn set_vote_threshold(origin: OriginFor<T>, n: u32, d: u32) -> DispatchResult {
            T::VoteThresholdOrigin::ensure_origin(origin)?;
            // Proportion must be a rational number
            ensure!(d > 0 && n <= d, Error::<T, I>::InvalidProportion);
            <VoteThreshold<T, I>>::put((n, d));
            Self::deposit_event(Event::VoteThresholdUpdated(GC_DID, n, d));
            Ok(())
        }

        /// Changes the release coordinator.
        ///
        /// # Arguments
        /// * `id` - The DID of the new release coordinator.
        ///
        /// # Errors
        /// * `NotAMember`, If the new coordinator `id` is not part of the committee.
        #[pallet::weight((<T as Config<I>>::WeightInfo::set_release_coordinator(), DispatchClass::Operational))]
        #[pallet::call_index(1)]
        pub fn set_release_coordinator(origin: OriginFor<T>, id: IdentityId) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::ensure_did_is_member(&id)?;
            <ReleaseCoordinator<T, I>>::put(id);
            Self::deposit_event(Event::ReleaseCoordinatorUpdated(Some(id)));
            Ok(())
        }

        /// Changes the time after which a proposal expires.
        ///
        /// # Arguments
        /// * `expiry` - The new expiry time.
        #[pallet::weight((<T as Config<I>>::WeightInfo::set_expires_after(), DispatchClass::Operational))]
        #[pallet::call_index(2)]
        pub fn set_expires_after(
            origin: OriginFor<T>,
            expiry: MaybeBlock<BlockNumberFor<T>>,
        ) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;
            <ExpiresAfter<T, I>>::put(expiry);
            Self::deposit_event(Event::ExpiresAfterUpdated(GC_DID, expiry));
            Ok(())
        }

        /// Proposes to the committee that `call` should be executed in its name.
        /// Alternatively, if the hash of `call` has already been recorded, i.e., already proposed,
        /// then this call counts as a vote, i.e., as if `vote_by_hash` was called.
        ///
        /// # Weight
        ///
        /// The weight of this dispatchable is that of `call` as well as the complexity
        /// for recording the vote itself.
        ///
        /// # Arguments
        /// * `approve` - is this an approving vote?
        ///   If the proposal doesn't exist, passing `false` will result in error `FirstVoteReject`.
        /// * `call` - the call to propose for execution.
        ///
        /// # Errors
        /// * `FirstVoteReject`, if `call` hasn't been proposed and `approve == false`.
        /// * `NotAMember`, if the `origin` is not a member of this committee.
        #[pallet::weight((<T as Config<I>>::WeightInfo::vote_or_propose_new_proposal()
            .saturating_add(call.get_dispatch_info().call_weight),
            DispatchClass::Operational
        ))]
        #[pallet::call_index(3)]
        pub fn vote_or_propose(
            origin: OriginFor<T>,
            approve: bool,
            call: Box<<T as Config<I>>::Proposal>,
        ) -> DispatchResult {
            // Either create a new proposal or vote on an existing one.
            let hash = T::Hashing::hash_of(&call);
            match Voting::<T, I>::get(hash) {
                Some(voting) => Self::vote(origin, hash, voting.index, approve),
                // NOTE: boxing is necessary or the trait system will throw a fit.
                None if approve => Self::propose(origin, *call),
                None => Err(Error::<T, I>::FirstVoteReject.into()),
            }
        }

        /// Votes `approve`ingly (or not, if `false`)
        /// on an existing `proposal` given by its hash, `index`.
        ///
        /// # Arguments
        /// * `proposal` - A hash of the proposal to be voted on.
        /// * `index` - The proposal index.
        /// * `approve` - If `true` than this is a `for` vote, and `against` otherwise.
        ///
        /// # Errors
        /// * `NotAMember`, if the `origin` is not a member of this committee.
        #[pallet::weight(vote::<T,I>(*approve))]
        #[pallet::call_index(4)]
        pub fn vote(
            origin: OriginFor<T>,
            proposal: T::Hash,
            index: ProposalIndex,
            approve: bool,
        ) -> DispatchResult {
            // 1. Ensure `origin` is a committee member.
            let did = Self::ensure_is_member(origin)?;

            // 2a. Ensure a prior proposal exists and that their indices match.
            let mut voting = Self::ensure_proposal(&proposal, index)?;

            // 2b. Ensure proposal hasn't expired. If it has, prune the proposal and bail.
            Self::ensure_not_expired(&proposal, voting.expiry)?;

            // 3. Vote on aye / nay and remove from the other.
            let aye = (voting.ayes.iter().position(|a| a == &did), &mut voting.ayes);
            let nay = (voting.nays.iter().position(|a| a == &did), &mut voting.nays);
            let (main, other) = if approve { (aye, nay) } else { (nay, aye) };
            ensure!(main.0.is_none(), Error::<T, I>::DuplicateVote);
            main.1.push(did);
            if let Some(pos) = other.0 {
                other.1.swap_remove(pos);
            }
            let ayes = voting.ayes.len() as MemberCount;
            let nays = voting.nays.len() as MemberCount;
            <Voting<T, I>>::insert(&proposal, voting);

            // 4. Emit event.
            Self::deposit_event(Event::Voted(
                did,
                index,
                proposal,
                approve,
                ayes,
                nays,
                Self::seats(),
            ));

            // 5. Check whether majority has been reached and if so, execute proposal.
            Self::execute_if_passed(Some(did), proposal);
            Ok(())
        }
    }

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Ensure proposal with `hash` exists and has index `idx`.
        fn ensure_proposal(
            hash: &T::Hash,
            idx: ProposalIndex,
        ) -> Result<PolymeshVotes<BlockNumberFor<T>>, DispatchError> {
            let voting = Voting::<T, I>::get(&hash).ok_or(Error::<T, I>::NoSuchProposal)?;
            ensure!(voting.index == idx, Error::<T, I>::MismatchedVotingIndex);
            Ok(voting)
        }

        /// Ensures that `origin` is a committee member, returning its identity, or throws `NotAMember`.
        fn ensure_is_member(origin: OriginFor<T>) -> Result<IdentityId, DispatchError> {
            let did = IdentityPallet::<T>::ensure_perms(origin)?;
            Self::ensure_did_is_member(&did)?;
            Ok(did)
        }

        /// Returns true if `who` is contained in the set of committee members, and `false` otherwise.
        pub fn ensure_did_is_member(who: &IdentityId) -> DispatchResult {
            ensure!(
                Members::<T, I>::get().binary_search(who).is_ok(),
                Error::<T, I>::NotAMember
            );
            Ok(())
        }

        fn seats() -> MemberCount {
            Members::<T, I>::get().len() as _
        }

        /// Given `votes` number of votes out of `total` votes, this function compares`votes`/`total`
        /// in relation to the threshold proporion `n`/`d`.
        fn is_threshold_satisfied(votes: u32, total: u32, (n, d): (u32, u32)) -> bool {
            votes * d >= n * total
        }

        /// Removes the `id`'s vote from `proposal` if it exists.
        ///
        /// # Return
        /// It returns true if vote was removed.
        pub(crate) fn remove_vote_from(id: IdentityId, proposal: T::Hash) -> bool {
            if let Some(mut voting) = Voting::<T, I>::get(&proposal) {
                // If any element is removed, we have to update `voting`.
                let idx = voting.index;
                let remove = |from: &mut Vec<_>, sig| {
                    from.iter().position(|a| *a == id).map(|pos| {
                        Self::deposit_event(Event::VoteRetracted(id, idx, proposal, sig));
                        from.swap_remove(pos)
                    })
                };
                return remove(&mut voting.ayes, true)
                    .or_else(|| remove(&mut voting.nays, false))
                    .map(|_| <Voting<T, I>>::insert(&proposal, voting))
                    .is_some();
            }
            false
        }

        /// Accepts or rejects the proposal if its threshold is satisfied.
        pub(crate) fn execute_if_passed(did: Option<IdentityId>, proposal: T::Hash) {
            let voting = match Voting::<T, I>::get(&proposal) {
                // Make sure we don't have an expired proposal at this point.
                Some(v) if Self::ensure_not_expired(&proposal, v.expiry).is_ok() => v,
                _ => return,
            };
            let ayes = voting.ayes.len() as MemberCount;
            let nays = voting.nays.len() as MemberCount;

            let threshold = <VoteThreshold<T, I>>::get();
            let seats = Self::seats();
            let satisfied =
                |main, other| main >= other && Self::is_threshold_satisfied(main, seats, threshold);
            let approved = satisfied(ayes, nays);
            let rejected = satisfied(nays, ayes);
            if !approved && !rejected {
                return;
            }

            Self::finalize_proposal(approved, seats, ayes, nays, proposal, did);
            let event = Event::FinalVotes(did, voting.index, proposal, voting.ayes, voting.nays);
            Self::deposit_event(event);
        }

        /// Finalizes a proposal.
        ///
        /// # Complexity
        /// If `approved`:
        /// - the weight of `proposal` preimage.
        /// - two events deposited.
        /// - two removals, one mutation.
        /// - computation and i/o `O(P + L)` where:
        ///   - `P` is number of active proposals,
        ///   - `L` is the encoded length of `proposal` preimage.
        ///
        /// If not `approved`:
        /// - one event deposited.
        /// Two removals, one mutation.
        /// Computation and i/o `O(P)` where:
        /// - `P` is number of active proposals
        fn finalize_proposal(
            approved: bool,
            seats: MemberCount,
            yes_votes: MemberCount,
            no_votes: MemberCount,
            proposal: T::Hash,
            current_did: Option<IdentityId>,
        ) {
            let event = if approved {
                Event::Approved
            } else {
                Event::Rejected
            };
            Self::deposit_event(event(current_did, proposal, yes_votes, no_votes, seats));

            let proposal_of = <ProposalOf<T, I>>::take(&proposal);

            if approved {
                // execute motion, assuming it exists.
                if let Some(p) = proposal_of {
                    Self::execute(current_did, p, proposal);
                }
            }

            // Clear remaining proposal data.
            Self::clear_proposal(&proposal);
        }

        /// Clear data for `proposal`, except for `ProposalOf`,
        /// which needs to be cleared separately.
        fn clear_proposal(proposal: &T::Hash) {
            <Voting<T, I>>::remove(proposal);
            <Proposals<T, I>>::mutate(|proposals| proposals.retain(|h| h != proposal));
        }

        /// Ensure that the given `proposal` with associated `expiry` hasn't expired relative to `now`.
        /// As a side-effect, on error, any existing proposal data is pruned.
        fn ensure_not_expired(
            proposal: &T::Hash,
            expiry: MaybeBlock<BlockNumberFor<T>>,
        ) -> Result<(), Error<T, I>> {
            match expiry {
                MaybeBlock::Some(e) if e <= frame_system::Pallet::<T>::block_number() => {
                    Self::clear_proposal(proposal);
                    <ProposalOf<T, I>>::remove(proposal);
                    Err(Error::<T, I>::ProposalExpired)
                }
                _ => Ok(()),
            }
        }

        fn execute(did: Option<IdentityId>, proposal: <T as Config<I>>::Proposal, hash: T::Hash) {
            let origin = RawOrigin::Endorsed(PhantomData).into();
            let res = proposal.dispatch(origin).map_err(|e| e.error).map(drop);
            Self::deposit_event(Event::Executed(did, hash, res));
        }

        /// Any committee member proposes a dispatchable.
        ///
        /// # Arguments
        /// * `proposal` - A dispatchable call.
        fn propose(origin: OriginFor<T>, proposal: <T as Config<I>>::Proposal) -> DispatchResult {
            // 1. Ensure `origin` is a committee member.
            let did = Self::ensure_is_member(origin)?;

            // 2. Get hash & reject duplicate proposals.
            let proposal_hash = T::Hashing::hash_of(&proposal);
            ensure!(
                !<ProposalOf<T, I>>::contains_key(proposal_hash),
                Error::<T, I>::DuplicateProposal
            );

            // 3. Execute if committee is single member, and otherwise record the vote.
            if Self::seats() < 2 {
                Self::execute(Some(did), proposal, proposal_hash);
            } else {
                let index = <ProposalCount<T, I>>::mutate(|i| mem::replace(i, *i + 1));
                <Proposals<T, I>>::append(proposal_hash);
                <ProposalOf<T, I>>::insert(proposal_hash, proposal);
                let now = frame_system::Pallet::<T>::block_number();
                let votes = PolymeshVotes {
                    index,
                    ayes: vec![did],
                    nays: vec![],
                    expiry: ExpiresAfter::<T, I>::get() + now,
                };
                <Voting<T, I>>::insert(proposal_hash, votes);

                Self::deposit_event(Event::Proposed(did, index, proposal_hash));
            }

            Ok(())
        }
    }
}

impl<T: Config<I>, I: 'static> GroupTrait<T::Moment> for Pallet<T, I> {
    /// Retrieve all members of this committee
    fn get_members() -> Vec<IdentityId> {
        Members::<T, I>::get()
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

    fn add_member(_who: IdentityId) -> DispatchResult {
        unimplemented!()
    }
}

impl<T: Config<I>, I: 'static> GovernanceGroupTrait<T::Moment> for Pallet<T, I> {
    fn release_coordinator() -> Option<IdentityId> {
        ReleaseCoordinator::<T, I>::get()
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn bench_set_release_coordinator(id: IdentityId) {
        if !Members::<T, I>::get().contains(&id) {
            Self::change_members_sorted(&[id], &[], &[id]);
        }
        <ReleaseCoordinator<T, I>>::put(id);
    }
}

impl<T: Config<I>, I: 'static> ChangeMembers<IdentityId> for Pallet<T, I> {
    /// This function is called when the group updates its members, and it executes the following
    /// actions:
    /// * It removes outgoing member's vote of each current proposal.
    /// * It adds the Systematic CDD claim (issued by `SystematicIssuers::Committee`) to new incoming members.
    /// * It removes the Systematic CDD claim (issued by `SystematicIssuers::Committee`) from
    /// outgoing members.
    fn change_members_sorted(incoming: &[IdentityId], outgoing: &[IdentityId], new: &[IdentityId]) {
        // Immediately set members so threshold is affected.
        <Members<T, I>>::put(new);

        // Remove accounts from all current voting in motions.
        Proposals::<T, I>::get()
            .into_iter()
            .filter(|proposal| {
                outgoing
                    .iter()
                    .any(|id| Self::remove_vote_from(*id, *proposal))
            })
            .for_each(|proposal| Self::execute_if_passed(None, proposal));

        // Double check if any `outgoing` is the Release coordinator.
        if let Some(curr_rc) = ReleaseCoordinator::<T, I>::get() {
            if outgoing.contains(&curr_rc) {
                <ReleaseCoordinator<T, I>>::kill();
                Self::deposit_event(Event::ReleaseCoordinatorUpdated(None));
            }
        }

        // Add/remove Systematic CDD claims for new/removed members.
        let issuer = SystematicIssuers::Committee;
        IdentityPallet::<T>::add_systematic_cdd_claims(incoming, issuer);
        IdentityPallet::<T>::revoke_systematic_cdd_claims(outgoing, issuer);
    }
}

impl<T: Config<I>, I: 'static> InitializeMembers<IdentityId> for Pallet<T, I> {
    /// Initializes the members and adds the Systemic CDD claim (issued by
    /// `SystematicIssuers::Committee`).
    fn initialize_members(members: &[IdentityId]) {
        if members.is_empty() {
            return;
        }
        assert!(
            <Members<T, I>>::get().is_empty(),
            "Members are already initialized!"
        );
        IdentityPallet::<T>::add_systematic_cdd_claims(members, SystematicIssuers::Committee);
        <Members<T, I>>::put(members);
    }
}

pub struct EnsureThresholdMet<AccountId, I: 'static>(PhantomData<(AccountId, I)>);

impl<O, AccountId, I> EnsureOrigin<O> for EnsureThresholdMet<AccountId, I>
where
    O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
{
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().map(|RawOrigin::Endorsed(PhantomData)| ())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(RawOrigin::Endorsed(PhantomData)))
    }
}

fn vote<T: Config<I>, I: 'static>(approve: bool) -> (Weight, DispatchClass) {
    let weight = if approve {
        <T as Config<I>>::WeightInfo::vote_aye()
    } else {
        <T as Config<I>>::WeightInfo::vote_nay()
    };

    (weight, DispatchClass::Operational)
}
