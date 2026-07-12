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

//! # Corporate Ballots module.
//!
//! The corporate ballots module provides functionality for conducting corporate ballots,
//! e.g., for the annual general meeting.
//! Ballots consist of a set of motions, each with a set of choices like "Yay" or "Nay".
//!
//! The process works by first initiating the corporate action (CA) through `initiate_corporate_action`,
//! and then attaching a ballot to it via `attach_ballot`.
//! When attaching a ballot, the motions are provided, along with when the duration of the ballot.
//!
//! Once the start time is due, token holders in the CA's asset may cast their ballot.
//! To do so, they call the `vote` dispatchable,
//! dividing their available votes to each choice within a motion.
//!
//! The available votes are computed based on the record date provided when the CA was created.
//! The record date is then translated into a checkpoint,
//! and the holder's balance at that point is used as the available power.
//!
//! Eventually, the voting duration will be over.
//! The interpretation of the vote results can then be interpreted off-chain,
//! depending on the exact by-laws of the corporation.
//! For example, Ranked-Choice Voting (RCV), may be used, when fallbacks are provided in votes.
//!
//! ## Overview
//!
//! The Voting module provides functions for:
//!
//! - Creating ballots that can include multiple motions with multiple choices for each of those.
//! - Adjusting details of a ballot that hasn't yet started.
//! - Voting on motions.
//! - Removing/Cancelling ballots.
//!
//! ### Terminology
//!
//! - **Ballot:** A set of motions made, each with a set of choices on which a token holder can vote.
//!
//! - **Motion:** A motion can be e.g., "Elect Alice as CEO".
//!     That is, a motion is a suggested action or stance that the corporation should take.
//!     Each motion can then have a number of choices, e.g., "Yay", or "Nay".
//!     Token holders can then divide all of their power across the choices of one motion,
//!     and reuse the same amount of voting power on other motions.
//!     The motion is associated with some descriptive text, and a link for more information.
//!     Commonly, a motion will only have two choices, "Yay" or "Nay".
//!     Any voting power that is not used is considered as abstain.
//!
//! - **RCV:** Ranked-Choice Voting allows voters to select a fallback choice should their first
//!     preference fail to reach a certain threshold or e.g., be eliminated in the top-2 run-off.
//!     The chain supports this by admitting fallback choices, if the ballot is configured to support this.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `attach_ballot(origin, ca_id, range, meta)` attaches a ballot to CA with `ca_id`
//!   within the voting duration specified by `range`, and motions drawn from `meta`.
//! - `vote(origin, ca_id, votes)` casts `votes` in the ballot for CA with `ca_id`.
//! - `change_end(origin, ca_id, end)` changes the end date of the ballot for CA with `ca_id`.
//! - `change_meta(origin, ca_id, meta)` changes the motions of the ballot for CA with `ca_id`.
//! - `change_rcv(origin, ca_id, rcv)` changes the support for RCV to `rcv` in the ballot for CA with `ca_id`.
//! - `remove_ballot(origin, ca_id)` removes the ballot for CA with `ca_id`.

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use core::convert::TryInto;
use core::mem;
use frame_support::dispatch::DispatchResult;
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchError;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Zero;
use sp_std::prelude::*;

use pallet_asset::checkpoint;
use pallet_base::ensure_string_limited;
use polymesh_primitives::protocol_fee::{ChargeProtocolFee, ProtocolOp};
use polymesh_primitives::{storage_migration_ver, Balance, EventDid, IdentityId, Moment};
use polymesh_primitives_derive::VecU8StrongTyped;

use crate as ca;
use ca::{CAId, CAKind, CorporateAction};

type MaxMotions = frame_support::traits::ConstU32<8>;
type MaxChoicesPerMotion = frame_support::traits::ConstU32<128>;

type Checkpoint<T> = checkpoint::Pallet<T>;
type CA<T> = ca::Pallet<T>;
type ExternalAgents<T> = pallet_external_agents::Pallet<T>;

/// A wrapper for a motion title.
#[derive(Serialize, Deserialize, DecodeWithMemTracking)]
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct MotionTitle(pub Vec<u8>);

/// A wrapper for a motion info link.
#[derive(Serialize, Deserialize, DecodeWithMemTracking)]
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct MotionInfoLink(pub Vec<u8>);

/// A wrapper for a choice's title.
#[derive(Serialize, Deserialize, DecodeWithMemTracking)]
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct ChoiceTitle(pub Vec<u8>);

/// Details about motions
#[derive(Deserialize, DecodeWithMemTracking, Serialize)]
#[derive(Clone, PartialEq, Eq, Default, Debug, Encode, Decode, TypeInfo)]
pub struct Motion {
    /// Title of the motion
    pub title: MotionTitle,

    /// Link from where more information about the motion can be obtained.
    pub info_link: MotionInfoLink,

    /// Choices for the motion excluding abstain.
    /// Voting power not used is considered abstained.
    pub choices: BoundedVec<ChoiceTitle, MaxChoicesPerMotion>,
}

/// A wrapper for a ballot's title.]
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(PartialEq, Eq, Hash, Debug, DecodeWithMemTracking)]
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct BallotTitle(pub Vec<u8>);

/// Metadata about a ballot.
///
/// Beyond the number of motions and the number of choices within a motion,
/// the actual metadata strings have no on-chain effect.
/// When the metadata has been committed to chain,
/// the needed numbers aforementioned are cached away,
/// and the metadata is not read on-chain again.
#[derive(Debug, Decode, DecodeWithMemTracking, Eq, Encode, PartialEq, TypeInfo)]
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct BallotMeta {
    /// The ballot's title.
    pub title: BallotTitle,

    /// All motions with their associated titles, choices, etc.
    pub motions: BoundedVec<Motion, MaxMotions>,
}

/// Timestamp range details about vote start / end.
#[derive(Encode, Decode, DecodeWithMemTracking, MaxEncodedLen)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, TypeInfo)]
#[derive(Deserialize, Serialize)]
pub struct BallotTimeRange {
    /// Timestamp at which voting starts.
    pub start: Moment,

    /// Timestamp at which voting ends.
    pub end: Moment,
}

/// A vote cast on some choice in some motion in a ballot.
#[derive(Encode, Decode, DecodeWithMemTracking, MaxEncodedLen)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, TypeInfo)]
#[derive(Deserialize, Serialize)]
pub struct BallotVote {
    /// The weight / voting power assigned to this vote.
    pub power: Balance,
    /// The fallback choice, if any, to assign `power` to,
    /// should the vote total fail to reach some threshold.
    ///
    /// This is only used when Ranked-Choice Voting (RCV) is active.
    ///
    /// # Representation
    ///
    /// The fallback is encoded as an index into one of the other choices within the same motion.
    /// Representationally, this admits an arbitrary directed graph, but we do exclude self-cycles.
    ///
    /// # An example
    ///
    /// To understand the semantics of `fallback`,
    /// let's consider a ballot, with a simplified representation:
    ///
    /// ```text
    /// ballot: [
    ///     motion A: { title: "Everyone must love chocolate.", choices: ["Yay", "Nay"] },
    ///     motion B: { title: "Elect 🦄 for president", choices: ["OK", "Make them Veep", "Nope"] },
    /// ]
    /// ```
    ///
    /// Votes are provided as a flat list, assigned to each choice.
    /// For example, imagine that only Alice votes, using a total of 100 power.
    /// In this case, she voted like:
    /// ```text
    /// votes: [
    ///     BallotVote { power: 100, fallback: None },
    ///     BallotVote { power: 0,   fallback: None },
    ///
    ///     BallotVote { power: 41,  fallback: None },
    ///     BallotVote { power: 49,  fallback: None },
    ///     BallotVote { power: 10,  fallback: Some(0) },
    /// ]
    /// ```
    ///
    /// Here, the first two `BallotVote`s belong to the two choices in motion A.
    /// The three remaining belong to motion B.
    ///
    /// Now suppose that we have a top-2 run-off voting process.
    /// Zooming in on motion B, the third choice would be eliminated,
    /// but because of `fallback: Some(0)`, now choice "OK" receives an additional 10 votes,
    /// putting the choice at a total of 51 votes. As 51 > 49, this is the choice that wins.
    ///
    /// Note that `Some(0)` does *not* point into motion A's first choice.
    pub fallback: Option<u16>,
}

/// Weight abstraction for the corporate actions module.
pub trait WeightInfo {
    fn attach_ballot(motions: u32, choices: u32) -> Weight;
    fn vote(votes: u32, target_ids: u32) -> Weight;
    fn change_end() -> Weight;
    fn change_meta(motions: u32, choices: u32) -> Weight;
    fn change_rcv() -> Weight;
    fn remove_ballot() -> Weight;

    fn get_motions_and_choices(ballot_meta: &BallotMeta) -> (u32, u32) {
        let motions = ballot_meta.motions.len() as u32;

        let mut choices = 0u32;
        for motion in ballot_meta.motions.iter() {
            choices = choices.saturating_add(motion.choices.len() as u32);
        }

        (motions, choices)
    }

    fn attach_ballot_weight(ballot_meta: &BallotMeta) -> Weight {
        let (motions, choices) = Self::get_motions_and_choices(ballot_meta);
        Self::attach_ballot(motions, choices)
    }

    fn change_meta_weight(ballot_meta: &BallotMeta) -> Weight {
        let (motions, choices) = Self::get_motions_and_choices(ballot_meta);
        Self::change_meta(motions, choices)
    }
}

storage_migration_ver!(1);

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::{ValueQuery, *};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + ca::Config {}

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Metadata of a corporate ballot.
    ///
    /// (CAId) => BallotMeta
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Metas<T: Config> = StorageMap<_, Blake2_128Concat, CAId, BallotMeta>;

    /// Time details of a corporate ballot associated with a CA.
    /// The timestamps denote when voting starts and stops.
    ///
    /// (CAId) => BallotTimeRange
    #[pallet::storage]
    pub type TimeRanges<T: Config> = StorageMap<_, Blake2_128Concat, CAId, BallotTimeRange>;

    /// Stores how many choices there are in each motion.
    ///
    /// At all times, the invariant holds that `motion_choices[idx]` is equal to
    /// `metas.unwrap().motions[idx].choices.len()`. That is, this is just a cache,
    /// used to avoid fetching all the motions with their associated texts.
    ///
    /// `u16` choices should be more than enough to fit real use cases.
    ///
    /// (CAId) => Number of choices in each motion.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type MotionNumChoices<T: Config> =
        StorageMap<_, Blake2_128Concat, CAId, Vec<u16>, ValueQuery>;

    /// Is ranked choice voting (RCV) enabled for this ballot?
    /// For an understanding of how RCV is handled, see note on `BallotVote`'s `fallback` field.
    ///
    /// (CAId) => bool
    #[pallet::storage]
    pub type RCV<T: Config> = StorageMap<_, Blake2_128Concat, CAId, bool, ValueQuery>;

    /// Stores the total vote tally on each choice.
    ///
    /// RCV is not accounted for,
    /// as there are too many wants to interpret the graph,
    /// and because it would not be efficient.
    ///
    /// (CAId) => [current vote weights]
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Results<T: Config> = StorageMap<_, Blake2_128Concat, CAId, Vec<Balance>, ValueQuery>;

    /// Stores each DID's votes in a given ballot.
    /// See the documentation of `BallotVote` for notes on semantics.
    ///
    /// (CAId) => (DID) => [vote weight]
    ///
    /// User must enter 0 vote weight if they don't want to vote for a choice.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Votes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CAId,
        Identity,
        IdentityId,
        Vec<BallotVote>,
        ValueQuery,
    >;

    /// Storage version.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            StorageVersion::<T>::put(Version::new(1));
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Attach a corporate ballot to the CA identified by `ca_id`.
        ///
        /// The ballot will admit votes within `range`.
        /// The ballot's metadata is provided by `meta`,
        /// which includes the ballot title, the motions, their choices, etc.
        /// See the `BallotMeta` for more.
        ///
        /// ## Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
        /// - `ca_id` identifies the CA to attach the ballot to.
        /// - `range` specifies when voting starts and ends.
        /// - `meta` specifies the ballot's metadata as aforementioned.
        /// - `rcv` specifies whether RCV is enabled for this ballot.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
        /// - `NoSuchCA` if `ca_id` does not identify an existing CA.
        /// - `CANotNotice` if the CA is not of the `IssuerNotice` kind.
        /// - `StartAfterEnd` if `range.start > range.end`.
        /// - `NowAfterEnd` if `now > range.end` where `now` is the current timestamp.
        /// - `NoRecordDate` if CA has no record date.
        /// - `RecordDateAfterStart` if `date > range.start` where `date` is the CA's record date.
        /// - `AlreadyExists` if there's a ballot already.
        /// - `NumberOfChoicesOverflow` if the total choice in `meta` overflows `usize`.
        /// - `TooLong` if any of the embedded strings in `meta` are too long.
        /// - `InsufficientBalance` if the protocol fee couldn't be charged.
        #[pallet::weight(<T as ca::Config>::BallotWeightInfo::attach_ballot_weight(&meta))]
        #[pallet::call_index(0)]
        pub fn attach_ballot(
            origin: OriginFor<T>,
            ca_id: CAId,
            range: BallotTimeRange,
            meta: BallotMeta,
            rcv: bool,
        ) -> DispatchResult {
            // Ensure that the caller is a permissioned agent
            let caller_did = ExternalAgents::<T>::ensure_perms(origin, &ca_id.asset_id)?;

            let motion_choices = Self::validate_ballot_creation_rules(ca_id, range, &meta)?;

            Self::unverified_create_ballot(caller_did, ca_id, motion_choices, range, meta, rcv)?;

            Ok(())
        }

        /// Cast `votes` in the ballot attached to the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` which must be a permissioned signer targeted by the CA.
        /// - `ca_id` identifies the attached ballot's CA.
        /// - `votes` specifies the balances to assign to each choice in the ballot.
        ///    The full voting power of `origin`'s DID may be used for each motion in the ballot.
        ///
        /// # Errors
        /// - `NoSuchBallot` if `ca_id` does not identify a ballot.
        /// - `VotingNotStarted` if the voting period hasn't commenced yet.
        /// - `VotingAlreadyEnded` if the voting period has ended.
        /// - `WrongVoteCount` if the number of choices in the ballot does not match `votes.len()`.
        /// - `NoSuchCA` if `ca_id` does not identify an existing CA.
        /// - `NotTargetedByCA` if the CA does not target `origin`'s DID.
        /// - `InsufficientVotes` if the voting power used for any motion in `votes`
        ///    exceeds `origin`'s DID's voting power.
        #[pallet::weight(<T as ca::Config>::BallotWeightInfo::vote(votes.len() as u32, T::MaxTargetIds::get()))]
        #[pallet::call_index(1)]
        pub fn vote(origin: OriginFor<T>, ca_id: CAId, votes: Vec<BallotVote>) -> DispatchResult {
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;

            // Ensure ballot has started but not ended, i.e. `start <= now <= end`.
            let range = Self::ensure_ballot_exists(ca_id)?;
            let now = <Checkpoint<T>>::now_unix();
            ensure!(range.start <= now, Error::<T>::VotingNotStarted);
            ensure!(now <= range.end, Error::<T>::VotingAlreadyEnded);

            // Ensure that `did` is targeted by this ballot.
            let ca = <CA<T>>::ensure_ca_exists(ca_id)?;
            <CA<T>>::ensure_ca_targets(&ca, &did)?;

            // Ensure we have balances provided for each choice.
            let choices_count = MotionNumChoices::<T>::get(ca_id);
            let total_choices = choices_count
                .iter()
                .copied()
                .map(|c| c as usize)
                .sum::<usize>();
            ensure!(votes.len() == total_choices, Error::<T>::WrongVoteCount);

            // Divide `votes` into motions.
            let motions = choices_count
                .iter()
                .map(|c| *c as usize)
                .scan(0, |start, count| {
                    let end = *start + count;
                    Some(&votes[mem::replace(start, end)..end])
                });

            if RCV::<T>::get(ca_id) {
                // RCV is enabled.
                // Ensure that all fallback choices point to some choice in the same motion.
                // For in-depth discussion on `fallback`, consult `BallotVote`'s definition.
                motions.clone().try_for_each(|votes| -> DispatchResult {
                    let count = votes.len();
                    votes
                        .iter()
                        .enumerate()
                        // Only check when a fallback is actually provided.
                        .filter_map(|(idx, vote)| Some((idx, vote.fallback? as usize)))
                        .try_for_each(|(idx, fallback)| {
                            // Exclude self-cycles.
                            ensure!(idx != fallback, Error::<T>::RCVSelfCycle);
                            // Ensure the index does not point outside, i.e. beyond, the motion.
                            ensure!(fallback < count, Error::<T>::NoSuchRCVFallback);
                            Ok(())
                        })
                })?;
            } else {
                // It's not. Make sure its also not used.
                votes
                    .iter()
                    .all(|vote| vote.fallback.is_none())
                    .then_some(())
                    .ok_or(Error::<T>::RCVNotAllowed)?;
            }

            // Extract `did`'s balance at the record date.
            // Record date has passed by definition.
            let cp_id = <CA<T>>::record_date_cp(&ca, ca_id)?;
            let available_power = <CA<T>>::balance_at_cp(did, ca_id, cp_id);

            // Ensure the total balance used in each motion doesn't exceed caller's voting power.
            motions
                .map(|vs| {
                    vs.iter()
                        .try_fold(Balance::zero(), |acc, vote| acc.checked_add(vote.power))
                })
                .all(|power| power.filter(|&p| p <= available_power).is_some())
                .then_some(())
                .ok_or(Error::<T>::InsufficientVotes)?;

            // Update vote and total results.
            Votes::<T>::mutate(ca_id, did, |vslot| {
                Results::<T>::mutate_exists(ca_id, |rslot| match rslot {
                    Some(rslot) => {
                        for (result, old) in rslot.iter_mut().zip(vslot.iter()) {
                            *result -= old.power;
                        }
                        for (result, new) in rslot.iter_mut().zip(votes.iter()) {
                            *result += new.power;
                        }
                    }
                    None => *rslot = Some(votes.iter().map(|v| v.power).collect()),
                });
                *vslot = votes.clone();
            });

            // Emit event.
            Self::deposit_event(Event::VoteCast(did, ca_id, votes));
            Ok(().into())
        }

        /// Amend the end date of the ballot of the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
        /// - `ca_id` identifies the attached ballot's CA.
        /// - `end` specifies the new end date of the ballot.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
        /// - `NoSuchBallot` if `ca_id` does not identify a ballot.
        /// - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
        /// - `StartAfterEnd` if `start > end`.
        #[pallet::weight(<T as ca::Config>::BallotWeightInfo::change_end())]
        #[pallet::call_index(2)]
        pub fn change_end(origin: OriginFor<T>, ca_id: CAId, end: Moment) -> DispatchResult {
            // Ensure origin is a permissioned agent, ballot exists, and start is in the future.
            let agent = <ExternalAgents<T>>::ensure_perms(origin, &ca_id.asset_id)?;
            let mut range = Self::ensure_ballot_exists(ca_id)?;
            Self::ensure_ballot_not_started(range)?;

            // Ensure we preserve `start <= end`.
            range.end = end;
            Self::ensure_range_consistent(range)?;

            // Commit new range to storage + emit event.
            TimeRanges::<T>::insert(ca_id, range);
            Self::deposit_event(Event::RangeChanged(agent, ca_id, range));
            Ok(().into())
        }

        /// Amend the metadata (title, motions, etc.) of the ballot of the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
        /// - `ca_id` identifies the attached ballot's CA.
        /// - `meta` specifies the new metadata.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
        /// - `NoSuchBallot` if `ca_id` does not identify a ballot.
        /// - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
        /// - `NumberOfChoicesOverflow` if the total choice in `meta` overflows `usize`.
        /// - `TooLong` if any of the embedded strings in `meta` are too long.
        #[pallet::weight(<T as ca::Config>::BallotWeightInfo::change_meta_weight(&meta))]
        #[pallet::call_index(3)]
        pub fn change_meta(origin: OriginFor<T>, ca_id: CAId, meta: BallotMeta) -> DispatchResult {
            // Ensure origin is a permissioned agent, a ballot exists, start is in the future.
            let agent = <ExternalAgents<T>>::ensure_perms(origin, &ca_id.asset_id)?;
            Self::ensure_ballot_not_started(Self::ensure_ballot_exists(ca_id)?)?;

            // Compute number-of-choices-in-motion cache.
            let choices = Self::derive_motion_num_choices(&meta.motions)?;
            Self::ensure_meta_lengths_limited(&meta)?;

            // Commit metadata to storage + emit event.
            MotionNumChoices::<T>::insert(ca_id, choices);
            Metas::<T>::insert(ca_id, meta.clone());
            Self::deposit_event(Event::MetaChanged(agent, ca_id, meta));
            Ok(().into())
        }

        /// Amend RCV support for the ballot of the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
        /// - `ca_id` identifies the attached ballot's CA.
        /// - `rcv` specifies if RCV is to be supported or not.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
        /// - `NoSuchBallot` if `ca_id` does not identify a ballot.
        /// - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
        #[pallet::weight(<T as ca::Config>::BallotWeightInfo::change_rcv())]
        #[pallet::call_index(4)]
        pub fn change_rcv(origin: OriginFor<T>, ca_id: CAId, rcv: bool) -> DispatchResult {
            // Ensure origin is a permissioned agent, a ballot exists, start is in the future.
            let agent = <ExternalAgents<T>>::ensure_perms(origin, &ca_id.asset_id)?;
            Self::ensure_ballot_not_started(Self::ensure_ballot_exists(ca_id)?)?;

            // Commit to storage + emit event.
            RCV::<T>::insert(ca_id, rcv);
            Self::deposit_event(Event::RCVChanged(agent, ca_id, rcv));
            Ok(().into())
        }

        /// Remove the ballot of the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
        /// - `ca_id` identifies the attached ballot's CA.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
        /// - `NoSuchBallot` if `ca_id` does not identify a ballot.
        /// - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
        #[pallet::weight(<T as ca::Config>::BallotWeightInfo::remove_ballot())]
        #[pallet::call_index(5)]
        pub fn remove_ballot(origin: OriginFor<T>, ca_id: CAId) -> DispatchResult {
            let agent = <ExternalAgents<T>>::ensure_perms(origin, &ca_id.asset_id)?.for_event();
            let range = Self::ensure_ballot_exists(ca_id)?;
            Self::remove_ballot_base(agent, ca_id, range)?;
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A corporate ballot was created.
        ///
        /// (Agent DID, CA's ID, Voting start/end, Ballot metadata, RCV enabled?)
        Created(IdentityId, CAId, BallotTimeRange, BallotMeta, bool),

        /// A vote was cast in a corporate ballot.
        ///
        /// (voter DID, CAId, Votes)
        VoteCast(IdentityId, CAId, Vec<BallotVote>),

        /// A corporate ballot changed its start/end date range.
        ///
        /// (Agent DID, CA's ID, Voting start/end)
        RangeChanged(IdentityId, CAId, BallotTimeRange),

        /// A corporate ballot changed its metadata.
        ///
        /// (Agent DID, CA's ID, New metadata)
        MetaChanged(IdentityId, CAId, BallotMeta),

        /// A corporate ballot changed its RCV support.
        ///
        /// (Agent DID, CA's ID, New support)
        RCVChanged(IdentityId, CAId, bool),

        /// A corporate ballot was removed.
        ///
        /// (Agent DID, CA's ID)
        Removed(EventDid, CAId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A corporate ballot was made for a non `IssuerNotice` CA.
        CANotNotice,
        /// A corporate ballot already exists for this CA.
        AlreadyExists,
        /// A corporate ballot doesn't exist for this CA.
        NoSuchBallot,
        /// A corporate ballot's start time was strictly after the ballot's end.
        StartAfterEnd,
        /// A corporate ballot's end time was strictly before the current time.
        NowAfterEnd,
        /// If some motion in a corporate ballot has more choices than would fit in `u16`.
        NumberOfChoicesOverflow,
        /// Voting started already. Amending a ballot is no longer possible.
        VotingAlreadyStarted,
        /// Voting hasn't started yet.
        VotingNotStarted,
        /// Voting ended already.
        VotingAlreadyEnded,
        /// Provided list of balances does not match the total number of choices.
        WrongVoteCount,
        /// Voting power used by a DID on a motion exceeds that which is available to them.
        InsufficientVotes,
        /// The RCV fallback of some choice does not exist.
        NoSuchRCVFallback,
        /// The RCV fallback points to the origin choice.
        RCVSelfCycle,
        /// RCV is not allowed for this ballot.
        RCVNotAllowed,
    }
}

impl<T: Config> Pallet<T> {
    /// Returns the number-of-choices-in-motion if all rules for creating a ballot are satisfied. Otherwise, an error.
    pub(crate) fn validate_ballot_creation_rules(
        ca_id: CAId,
        ballot_time_range: BallotTimeRange,
        ballot_meta: &BallotMeta,
    ) -> Result<Vec<u16>, DispatchError> {
        let corporate_action = CA::<T>::ensure_ca_exists(ca_id)?;

        ensure!(
            corporate_action.kind == CAKind::IssuerNotice,
            Error::<T>::CANotNotice
        );

        Self::ensure_range_invariant(&corporate_action, ballot_time_range)?;

        // Ensure CA doesn't have a ballot yet
        ensure!(
            !TimeRanges::<T>::contains_key(ca_id),
            Error::<T>::AlreadyExists
        );

        let motion_choices = Self::derive_motion_num_choices(&ballot_meta.motions)?;
        Self::ensure_meta_lengths_limited(ballot_meta)?;

        Ok(motion_choices)
    }

    /// Charges the protocol fee and creates a ballot.
    pub(crate) fn unverified_create_ballot(
        caller_id: IdentityId,
        ca_id: CAId,
        motion_choices: Vec<u16>,
        ballot_time_range: BallotTimeRange,
        ballot_meta: BallotMeta,
        rcv: bool,
    ) -> DispatchResult {
        T::ProtocolFee::charge_fee(ProtocolOp::CorporateBallotAttachBallot)?;

        MotionNumChoices::<T>::insert(ca_id, motion_choices);
        TimeRanges::<T>::insert(ca_id, ballot_time_range);
        Metas::<T>::insert(ca_id, ballot_meta.clone());
        RCV::<T>::insert(ca_id, rcv);

        Self::deposit_event(Event::Created(
            caller_id,
            ca_id,
            ballot_time_range,
            ballot_meta,
            rcv,
        ));

        Ok(())
    }

    /// Ensure the ballot hasn't started and remove it.
    pub(crate) fn remove_ballot_base(
        agent: EventDid,
        ca_id: CAId,
        range: BallotTimeRange,
    ) -> DispatchResult {
        Self::ensure_ballot_not_started(range)?;

        // Remove all ballot data.
        TimeRanges::<T>::remove(ca_id);
        Metas::<T>::remove(ca_id);
        MotionNumChoices::<T>::remove(ca_id);
        RCV::<T>::remove(ca_id);

        // Emit event.
        Self::deposit_event(Event::Removed(agent, ca_id));
        Ok(())
    }

    /// Ensure that no string embedded within `meta` is too long.
    fn ensure_meta_lengths_limited(meta: &BallotMeta) -> DispatchResult {
        ensure_string_limited::<T>(&meta.title)?;
        for motion in &meta.motions {
            ensure_string_limited::<T>(&motion.title)?;
            ensure_string_limited::<T>(&motion.info_link)?;
            for choice in &motion.choices {
                ensure_string_limited::<T>(choice)?;
            }
        }
        Ok(())
    }

    // Compute number-of-choices-in-motion cache for `motions`.
    fn derive_motion_num_choices(motions: &[Motion]) -> Result<Vec<u16>, DispatchError> {
        let mut total: usize = 0;
        motions
            .iter()
            .map(|motion| {
                let len = motion.choices.len();
                // Overflowing usize here will never happen in practice,
                // but can happen in theory.
                // We do this now to avoid the potential overflow in `vote`.
                total = total.checked_add(len)?;
                len.try_into().ok()
            })
            .collect::<Option<_>>()
            .ok_or_else(|| Error::<T>::NumberOfChoicesOverflow.into())
    }

    /// Ensure that `now < range.start`.
    pub(crate) fn ensure_ballot_not_started(range: BallotTimeRange) -> DispatchResult {
        ensure!(
            <Checkpoint<T>>::now_unix() < range.start,
            Error::<T>::VotingAlreadyStarted
        );
        Ok(())
    }

    /// Ensure that `ca_id` has an active ballot and return its date-time range.
    fn ensure_ballot_exists(ca_id: CAId) -> Result<BallotTimeRange, DispatchError> {
        TimeRanges::<T>::get(ca_id).ok_or_else(|| Error::<T>::NoSuchBallot.into())
    }

    /// Ensure that `range.start <= range.end`.
    fn ensure_range_consistent(range: BallotTimeRange) -> DispatchResult {
        ensure!(range.start <= range.end, Error::<T>::StartAfterEnd);
        Ok(())
    }

    // Ensure that `start <= end`, `now <= end`, and `record_date <= voting start`.
    fn ensure_range_invariant(ca: &CorporateAction, range: BallotTimeRange) -> DispatchResult {
        Self::ensure_range_consistent(range)?;
        ensure!(
            <Checkpoint<T>>::now_unix() <= range.end,
            Error::<T>::NowAfterEnd
        );
        <CA<T>>::ensure_record_date_before_start(ca, range.start)
    }
}
