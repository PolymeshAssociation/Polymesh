// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

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
//! TODO
//!
//! ## Overview
//!
//! TODO
//!
//! ## Interface
//!
//! TODO
//!
//! ### Dispatchable Functions
//!
//! TODO
//!
//! ### Public Functions
//!
//! TODO
//!

#![cfg_attr(not(feature = "std"), no_std)]

use crate as ca;
use ca::{CACheckpoint, CAId, CAKind, CorporateAction, Trait};
use codec::{Decode, Encode};
use core::convert::TryInto;
use core::mem;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
};
use pallet_asset::{self as asset, checkpoint};
use pallet_identity as identity;
use polymesh_common_utilities::protocol_fee::{ChargeProtocolFee, ProtocolOp};
use polymesh_common_utilities::CommonTrait;
use polymesh_primitives::{IdentityId, Moment};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_runtime::traits::{CheckedAdd, Zero};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

type Identity<T> = identity::Module<T>;
type Asset<T> = asset::Module<T>;
type Checkpoint<T> = checkpoint::Module<T>;
type CA<T> = ca::Module<T>;

/// A wrapper for a motion title.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Decode, Encode, VecU8StrongTyped)]
pub struct MotionTitle(pub Vec<u8>);

/// A wrapper for a motion info link.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Decode, Encode, VecU8StrongTyped)]
pub struct MotionInfoLink(pub Vec<u8>);

/// Details about motions
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Motion {
    /// Title of the motion
    pub title: MotionTitle,

    /// Link from where more information about the motion can be obtained.
    pub info_link: MotionInfoLink,

    /// Choices for the motion excluding abstain.
    /// Voting power not used is considered abstained.
    pub choices: Vec<MotionTitle>,
}

/// Metadata about a ballot.
/// Has no functional on-chain effect.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct BallotMeta {
    /// The ballot's title.
    pub title: MotionTitle,

    /// All motions with their associated titles, choices, etc.
    pub motions: Vec<Motion>,
}

/// Timestamp range details about vote start / end.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct BallotRange {
    /// Timestamp at which voting starts.
    pub start: Moment,

    /// Timestamp at which voting ends.
    pub end: Moment,
}

decl_storage! {
    trait Store for Module<T: Trait> as CorporateBallot {
        /// Metadata of a corporate ballot.
        ///
        /// (CAId) => BallotMeta
        pub Metas get(fn metas): map hasher(blake2_128_concat) CAId => Option<BallotMeta>;

        /// Time details of a corporate ballot associated with a CA.
        /// The timestamps denote when voting starts and stops.
        ///
        /// (CAId) => BallotRange
        pub Ranges get(fn ranges): map hasher(blake2_128_concat) CAId => Option<BallotRange>;

        /// Stores how many choices there are in each motion.
        /// This is used to avoid fetching all the motions with their associated texts.
        ///
        /// `u16` choices should be more than enough to fit real use cases.
        ///
        /// (CAId) => Number of choices in each motion.
        pub MotionNumChoices get(fn motion_choices): map hasher(blake2_128_concat) CAId => Vec<u16>;

        /// Is ranked choice voting (RCV) enabled for this ballot?
        ///
        /// (CAId) => bool
        pub RCV get(fn rcv): map hasher(blake2_128_concat) CAId => bool;

        /// Stores the total vote tally on each choice.
        /// RCV is not accounted for, as there are
        ///
        /// (CAId) => [current vote weights]
        pub Results get(fn results): map hasher(blake2_128_concat) CAId => Vec<T::Balance>;

        /// Stores each DID's votes in a given ballot.
        ///
        /// (CAId) => (DID) => [vote weight]
        ///
        /// User must enter 0 vote weight if they don't want to vote for a choice.
        pub Votes get(fn votes): double_map hasher(blake2_128_concat) CAId, hasher(blake2_128_concat) IdentityId => Vec<(T::Balance, u16)>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Attach a corporate ballot to the CA identified by `ca_id`.
        ///
        /// The ballot will admit votes within `range`.
        /// The ballot's metadata is provided by `meta`,
        /// which includes the ballot title, the motions, their choices, etc.
        /// See the `BallotMeta` for more.
        ///
        /// ## Arguments
        /// - `origin` which must be a signer for the CAA of `ca_id`.
        /// - `ca_id` identifies the CA to attach the ballot to.
        /// - `range` specifies when voting starts and ends.
        /// - `meta` specifies the ballot's metadata as aforementioned.
        /// - `rcv` specifies whether RCV is enabled for this ballot.
        ///
        /// # Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        /// - `NoSuchCA` if `ca_id` does not identify an existing CA.
        /// - `CANotNotice` if the CA is not of the `IssuerNotice` kind.
        /// - `StartAfterEnd` if `range.start > range.end`.
        /// - `NowAfterEnd` if `now > range.end` where `now` is the current timestamp.
        /// - `NoRecordDate` if CA has no record date.
        /// - `RecordDateAfterStart` if `date > range.start` where `date` is the CA's record date.
        /// - `AlreadyExists` if there's a ballot already.
        /// - `NumberOfChoicesOverflow` if the total choice in `meta` overflows `usize`.
        /// - `InsufficientBalance` if the protocol fee couldn't be charged.
        #[weight = 900_000_000]
        pub fn attach_ballot(origin, ca_id: CAId, range: BallotRange, meta: BallotMeta, rcv: bool) {
            // Ensure origin is CAA, that `ca_id` exists, that its a notice, and the date invariant.
            let caa = <CA<T>>::ensure_ca_agent(origin, ca_id.ticker)?;
            let ca = <CA<T>>::ensure_ca_exists(ca_id)?;
            ensure!(matches!(ca.kind, CAKind::IssuerNotice), Error::<T>::CANotNotice);
            Self::ensure_range_invariant(&ca, range)?;

            // Ensure CA doesn't have a ballot yet.
            ensure!(!Ranges::contains_key(ca_id), Error::<T>::AlreadyExists);

            // Compute number-of-choices-in-motion cache.
            let choices = Self::derive_motion_num_choices(&meta.motions)?;

            // Charge protocol fee.
            T::ProtocolFee::charge_fee(ProtocolOp::VotingAddBallot)?;

            // Commit to storage.
            MotionNumChoices::insert(ca_id, choices);
            Ranges::insert(ca_id, range);
            Metas::insert(ca_id, meta.clone());
            RCV::insert(ca_id, rcv);

            // Emit event.
            Self::deposit_event(Event::<T>::Created(caa, ca_id, range, meta, rcv));
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
        /// - `NotTargetedByCA` if the ballot does not target `origin`'s DID.
        /// - `InsufficientVotes` if the voting power used for any motion in `votes`
        ///    exceeds `origin`'s DID's voting power.
        #[weight = 950_000_000]
        pub fn vote(origin, ca_id: CAId, votes: Vec<(T::Balance, u16)>) {
            let did = <Identity<T>>::ensure_perms(origin)?;

            // Ensure ballot has started but not ended, i.e. `start <= now <= end`.
            let range = Self::ensure_ballot_exists(ca_id)?;
            let now = <Checkpoint<T>>::now_unix();
            ensure!(range.start <= now, Error::<T>::VotingNotStarted);
            ensure!(now <= range.end, Error::<T>::VotingAlreadyEnded);

            // Ensure we have balances provided for each choice.
            let choices_count = MotionNumChoices::get(ca_id);
            let total_choices = choices_count.iter().copied().map(|c| c as usize).sum::<usize>();
            ensure!(votes.len() == total_choices, Error::<T>::WrongVoteCount);

            // Divide `votes` into motions.
            let motions = choices_count
                .iter()
                .map(|c| *c as usize)
                .scan(0, |start, count| Some(&votes[mem::replace(start, *start + count)..count]));

            if RCV::get(ca_id) {
                // RCV is enabled.
                // Ensure that all fallback choices point to some choice in the same motion.
                motions
                    .clone()
                    .all(|votes| {
                        let count = votes.len();
                        votes
                            .iter()
                            .map(|vote| vote.1 as usize)
                            .enumerate()
                            .all(|(idx, fb)| idx != fb && fb < count)
                    })
                    .then_some(())
                    .ok_or(Error::<T>::NoSuchRCVFallback)?;
            } else {
                // It's not. Make sure its also not used.
                votes
                    .iter()
                    .all(|(_, fb)| *fb == 0)
                    .then_some(())
                    .ok_or(Error::<T>::RCVNotAllowed)?;
            }

            // Ensure that `did` is targeted by this ballot.
            let ca = <CA<T>>::ensure_ca_exists(ca_id)?;
            ensure!(ca.targets.targets(&did), Error::<T>::NotTargetedByCA);

            // Extract `did`'s balance at the record date.
            // Record date has passed by definition.
            let ticker = ca_id.ticker;
            let cp_id = match ca.record_date.unwrap().checkpoint {
                CACheckpoint::Existing(id) => Some(id),
                CACheckpoint::Scheduled(id) => <Checkpoint<T>>::schedule_points((ticker, id)).pop(),
            };
            let available_power = match cp_id {
                // CP exists, use it.
                Some(cp_id) => <Asset<T>>::get_balance_at(ticker, did, cp_id),
                // No transfer yet. Use current balance instead.
                None => <Asset<T>>::balance_of(ticker, did),
            };

            // Ensure the total balance used in each motion doesn't exceed caller's voting power.
            motions
                .map(|vs| vs.iter().try_fold(T::Balance::zero(), |acc, vote| acc.checked_add(&vote.0)))
                .all(|power| power.filter(|&p| p <= available_power).is_some())
                .then_some(())
                .ok_or(Error::<T>::InsufficientVotes)?;

            // Update vote and total results.
            <Votes<T>>::mutate(ca_id, did, |vslot| {
                <Results<T>>::mutate(ca_id, |rslot| {
                    for ((old, new), result) in vslot.iter().zip(votes.iter()).zip(rslot.iter_mut()) {
                        *result -= old.0;
                        *result += new.0;
                    }
                });
                *vslot = votes.clone();
            });

            // Emit event.
            Self::deposit_event(Event::<T>::VoteCast(did, ca_id, votes));
        }

        /// Amend the end date of the ballot of the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` which must be a signer for the CAA of `ca_id`.
        /// - `ca_id` identifies the attached ballot's CA.
        /// - `end` specifies the new end date of the ballot.
        ///
        /// # Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        /// - `NoSuchBallot` if `ca_id` does not identify a ballot.
        /// - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
        /// - `StartAfterEnd` if `start > end`.
        #[weight = 950_000_000]
        pub fn change_end(origin, ca_id: CAId, end: Moment) {
            // Ensure origin is CAA, a ballot exists, start is in the future.
            let caa = <CA<T>>::ensure_ca_agent(origin, ca_id.ticker)?;
            let mut range = Self::ensure_ballot_exists(ca_id)?;
            Self::ensure_ballot_unstarted(range)?;

            // Ensure we preserve `start <= end`.
            range.end = end;
            Self::ensure_range_consistent(range)?;

            // Commit new range to storage + emit event.
            Ranges::insert(ca_id, range);
            Self::deposit_event(Event::<T>::RangeChanged(caa, ca_id, range));
        }

        /// Amend the metadata (title, motions, etc.) of the ballot of the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` which must be a signer for the CAA of `ca_id`.
        /// - `ca_id` identifies the attached ballot's CA.
        /// - `meta` specifies the new metadata.
        ///
        /// # Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        /// - `NoSuchBallot` if `ca_id` does not identify a ballot.
        /// - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
        /// - `NumberOfChoicesOverflow` if the total choice in `meta` overflows `usize`.
        #[weight = 950_000_000]
        pub fn change_meta(origin, ca_id: CAId, meta: BallotMeta) {
            // Ensure origin is CAA, a ballot exists, start is in the future.
            let caa = <CA<T>>::ensure_ca_agent(origin, ca_id.ticker)?;
            Self::ensure_ballot_unstarted(Self::ensure_ballot_exists(ca_id)?)?;

            // Compute number-of-choices-in-motion cache.
            let choices = Self::derive_motion_num_choices(&meta.motions)?;

            // Commit metadata to storage + emit event.
            MotionNumChoices::insert(ca_id, choices);
            Metas::insert(ca_id, meta.clone());
            Self::deposit_event(Event::<T>::MetaChanged(caa, ca_id, meta));
        }

        /// Amend RCV support for the ballot of the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` which must be a signer for the CAA of `ca_id`.
        /// - `ca_id` identifies the attached ballot's CA.
        /// - `rcv` specifies if RCV is to be supported or not.
        ///
        /// # Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        /// - `NoSuchBallot` if `ca_id` does not identify a ballot.
        /// - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
        #[weight = 950_000_000]
        pub fn change_rcv(origin, ca_id: CAId, rcv: bool) {
            // Ensure origin is CAA, a ballot exists, start is in the future.
            let caa = <CA<T>>::ensure_ca_agent(origin, ca_id.ticker)?;
            Self::ensure_ballot_unstarted(Self::ensure_ballot_exists(ca_id)?)?;

            // Commit to storage + emit event.
            RCV::insert(ca_id, rcv);
            Self::deposit_event(Event::<T>::RCVChanged(caa, ca_id, rcv));
        }

        /// Remove the ballot of the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` which must be a signer for the CAA of `ca_id`.
        /// - `ca_id` identifies the attached ballot's CA.
        ///
        /// # Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        /// - `NoSuchBallot` if `ca_id` does not identify a ballot.
        #[weight = 950_000_000]
        pub fn remove_ballot(origin, ca_id: CAId) {
            // Ensure origin is CAA + ballot exists.
            let caa = <CA<T>>::ensure_ca_agent(origin, ca_id.ticker)?;
            Self::ensure_ballot_exists(ca_id)?;

            // Remove all ballot data.
            Ranges::remove(ca_id);
            Metas::remove(ca_id);
            MotionNumChoices::remove(ca_id);
            <Results<T>>::remove(ca_id);
            <Votes<T>>::remove_prefix(ca_id);

            // Emit event.
            Self::deposit_event(Event::<T>::Removed(caa, ca_id));
        }
    }
}

decl_event! {
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
    {
        /// A corporate ballot was created.
        ///
        /// (Ticker's CAA, CA's ID, Voting start/end, Ballot metadata, RCV enabled?)
        Created(IdentityId, CAId, BallotRange, BallotMeta, bool),

        /// A vote was cast in a corporate ballot.
        ///
        /// (voter DID, CAId, Votes)
        VoteCast(IdentityId, CAId, Vec<(Balance, u16)>),

        /// A corporate ballot changed its start/end date range.
        ///
        /// (Ticker's CAA, CA's ID, Voting start/end)
        RangeChanged(IdentityId, CAId, BallotRange),

        /// A corporate ballot changed its metadata.
        ///
        /// (Ticker's CAA, CA's ID, New metadata)
        MetaChanged(IdentityId, CAId, BallotMeta),

        /// A corporate ballot changed its RCV support.
        ///
        /// (Ticker's CAA, CA's ID, New support)
        RCVChanged(IdentityId, CAId, bool),

        /// A corporate ballot was removed.
        ///
        /// (Ticker's CAA, CA's ID)
        Removed(IdentityId, CAId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
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
        /// A CA's record date was strictly after a ballot's start time.
        NoRecordDate,
        /// A CA's record date was strictly after a ballot's start time.
        RecordDateAfterStart,
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
        /// CA does not target the voter DID.
        NotTargetedByCA,
        /// Voting power used by a DID on a motion exceeds that which is available to them.
        InsufficientVotes,
        /// The RCV fallback of some choice does not exist.
        NoSuchRCVFallback,
        /// RCV is not allowed for this ballot.
        RCVNotAllowed
    }
}

impl<T: Trait> Module<T> {
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
    fn ensure_ballot_unstarted(range: BallotRange) -> DispatchResult {
        ensure!(
            <Checkpoint<T>>::now_unix() < range.start,
            Error::<T>::VotingAlreadyStarted
        );
        Ok(())
    }

    /// Ensure that `ca_id` has an active ballot and return its date-time range.
    fn ensure_ballot_exists(ca_id: CAId) -> Result<BallotRange, DispatchError> {
        Ranges::get(ca_id).ok_or_else(|| Error::<T>::NoSuchBallot.into())
    }

    /// Ensure that `range.start <= range.end`.
    fn ensure_range_consistent(range: BallotRange) -> DispatchResult {
        ensure!(range.start <= range.end, Error::<T>::StartAfterEnd);
        Ok(())
    }

    // Ensure that `start <= end`, `now <= end`, and `record_date <= voting start`.
    fn ensure_range_invariant(ca: &CorporateAction, range: BallotRange) -> DispatchResult {
        Self::ensure_range_consistent(range)?;
        ensure!(
            <Checkpoint<T>>::now_unix() <= range.end,
            Error::<T>::NowAfterEnd
        );
        match ca.record_date {
            Some(rd) if rd.date <= range.start => Ok(()),
            Some(_) => Err(Error::<T>::RecordDateAfterStart.into()),
            None => Err(Error::<T>::NoRecordDate.into()),
        }
    }
}
