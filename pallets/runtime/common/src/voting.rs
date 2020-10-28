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

//! # Voting Module
//!
//! The Voting module provides functionality for corporate voting.
//!
//! ## Overview
//!
//! The Voting module provides functions for:
//!
//! - Creating ballots that can include multiple motions
//! - Voting on motions
//! - Cancelling ballots
//!
//! ### Terminology
//!
//! - **Ballot:** It is a collection of motions on which a token holder can vote.
//!     Additional parameters include voting start date, voting end date and checkpoint id.
//!     Checkpoint id is used to prevent double voting with same coins. When voting on a ballot,
//!     the total number of votes that a token holder can cast is equal to their balance at the checkpoint.
//!     Voters can distribute their votes across all the motions in the ballot.
//! - **motion:** It is a suggestion or a question that can have an infinite number of choices that can be voted on.
//!     Additional parameters include title of the motion and a link from where more info can be fetched.
//!     The most common motion is of accept/reject type where the motion has two choices, yes/no.
//!     Any voting power that is not used is considered as abstain.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `add_ballot` - Creates a ballot.
//! - `vote` - Casts a vote.
//! - `cancel_ballot` - Cancels an existing ballot.
use pallet_asset::checkpoint;
use pallet_identity as identity;
use polymesh_common_utilities::{
    asset::Trait as AssetTrait,
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    CommonTrait, Context,
};
use polymesh_primitives::{calendar::CheckpointId, IdentityId, Signatory, Ticker};
use polymesh_primitives_derive::VecU8StrongTyped;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    storage::StorageMap,
};
use frame_system::ensure_signed;
use sp_std::{convert::TryFrom, prelude::*, vec};

/// The module's configuration trait.
pub trait Trait: pallet_timestamp::Trait + frame_system::Trait + IdentityTrait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Asset: AssetTrait<Self::Balance, Self::AccountId>;
}

/// A wrapper for a motion title.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct MotionTitle(pub Vec<u8>);

/// A wrapper for a motion info link.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct MotionInfoLink(pub Vec<u8>);

/// Details about ballots
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Ballot<V> {
    /// The user's historic balance at this checkpoint is used as maximum vote weight
    pub checkpoint_id: CheckpointId,

    /// Timestamp at which voting should start
    pub voting_start: V,

    /// Timestamp at which voting should end
    pub voting_end: V,

    /// Array of motions that can be voted on
    pub motions: Vec<Motion>,
}

/// Details about motions
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Motion {
    /// Title of the motion
    pub title: MotionTitle,

    /// Link from where more information about the motion can be fetched
    pub info_link: MotionInfoLink,

    /// Choices for the motion excluding abstain
    /// Voting power not used is considered abstained
    pub choices: Vec<MotionTitle>,
}

type Identity<T> = identity::Module<T>;

decl_storage! {
    trait Store for Module<T: Trait> as Voting {
        /// Mapping of ticker and ballot name -> ballot details
        pub Ballots get(fn ballots): map hasher(blake2_128_concat) (Ticker, Vec<u8>) => Ballot<T::Moment>;

        /// Helper data to make voting cheaper.
        /// (ticker, BallotName) -> NoOfChoices
        pub TotalChoices get(fn total_choices): map hasher(blake2_128_concat) (Ticker, Vec<u8>) => u64;

        /// (Ticker, BallotName, DID) -> Vector of vote weights.
        /// weight at 0 index means weight for choice 1 of motion 1.
        /// weight at 1 index means weight for choice 2 of motion 1.
        /// User must enter 0 vote weight if they don't want to vote for a choice.
        pub Votes get(fn votes): map hasher(blake2_128_concat) (Ticker, Vec<u8>, IdentityId) => Vec<T::Balance>;

        /// (Ticker, BallotName) -> Vector of current vote weights.
        /// weight at 0 index means weight for choice 1 of motion 1.
        /// weight at 1 index means weight for choice 2 of motion 1.
        pub Results get(fn results): map hasher(blake2_128_concat) (Ticker, Vec<u8>) => Vec<T::Balance>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Define Error type
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        /// Adds a ballot
        ///
        /// # Arguments
        /// * `ticker` - Ticker of the token for which ballot is to be created
        /// * `ballot_name` - Name of the ballot
        /// * `ballot_details` - Other details of the ballot
        #[weight = 900_000_000]
        pub fn add_ballot(origin, ticker: Ticker, ballot_name: Vec<u8>, ballot_details: Ballot<T::Moment>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let signer = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), Error::<T>::InvalidSigner);
            ensure!(Self::is_owner(&ticker, did), Error::<T>::InvalidOwner);

            // This avoids cloning the variables to make the same tupple again and again.
            let ticker_ballot_name = (ticker, ballot_name.clone());

            // Ensure the uniqueness of the ballot
            ensure!(!<Ballots<T>>::contains_key(&ticker_ballot_name), Error::<T>::AlreadyExists);

            let now = <pallet_timestamp::Module<T>>::get();

            ensure!(now < ballot_details.voting_end, Error::<T>::InvalidDate);
            ensure!(ballot_details.voting_end > ballot_details.voting_start, Error::<T>::InvalidDate);
            ensure!(!ballot_details.motions.is_empty(), Error::<T>::NoMotions);

            // NB: Checkpoint ID is not verified here to allow creating ballots that will become active in future.
            // Voting will only be allowed on checkpoints that exist.

            let mut total_choices:usize = 0usize;

            for motion in &ballot_details.motions {
                ensure!(!motion.choices.is_empty(), Error::<T>::NoChoicesInMotions);
                total_choices += motion.choices.len();
            }
            <<T as IdentityTrait>::ProtocolFee>::charge_fee(ProtocolOp::VotingAddBallot)?;
            if let Ok(total_choices_u64) = u64::try_from(total_choices) {
                <TotalChoices>::insert(&ticker_ballot_name, total_choices_u64);
            } else {
                return Err(Error::<T>::InvalidChoicesType.into());
            }

            <Ballots<T>>::insert(&ticker_ballot_name, ballot_details.clone());

            let initial_results = vec![T::Balance::from(0); total_choices];
            <Results<T>>::insert(&ticker_ballot_name, initial_results);

            Self::deposit_event(RawEvent::BallotCreated(did, ticker, ballot_name, ballot_details));

            Ok(())
        }

        /// Casts a vote
        ///
        /// # Arguments
        /// * `ticker` - Ticker of the token for which vote is to be cast
        /// * `ballot_name` - Name of the ballot
        /// * `votes` - The actual vote to be cast
        #[weight = 950_000_000]
        pub fn vote(origin, ticker: Ticker, ballot_name: Vec<u8>, votes: Vec<T::Balance>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let signer = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), Error::<T>::InvalidSigner);

            // This avoids cloning the variables to make the same tupple again and again
            let ticker_ballot_name = (ticker, ballot_name.clone());

            // Ensure validity the ballot
            ensure!(<Ballots<T>>::contains_key(&ticker_ballot_name), Error::<T>::NotExists);
            let ballot = <Ballots<T>>::get(&ticker_ballot_name);
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(ballot.voting_start <= now, Error::<T>::NotStarted);
            ensure!(ballot.voting_end > now, Error::<T>::AlreadyEnded);

            // Ensure validity of checkpoint
            ensure!(checkpoint::CheckpointIdSequence::contains_key(&ticker), Error::<T>::NoCheckpoints);
            let count = checkpoint::CheckpointIdSequence::get(&ticker);
            ensure!(ballot.checkpoint_id <= count, Error::<T>::NoCheckpoints);

            // Ensure vote is valid
            if let Ok(votes_len) = u64::try_from(votes.len()) {
                ensure!(votes_len == <TotalChoices>::get(&ticker_ballot_name), Error::<T>::InvalidVote);
            } else {
                return Err(Error::<T>::InvalidVote.into())
            }

            let mut total_votes: T::Balance = 0.into();
            for vote in &votes {
                total_votes += *vote;
            }
            ensure!(total_votes <= T::Asset::get_balance_at(&ticker, did, ballot.checkpoint_id), Error::<T>::InsufficientBalance);

            // This avoids cloning the variables to make the same tupple again and again
            let ticker_ballot_name_did = (ticker, ballot_name.clone(), did);

            // Check if user has already voted for this ballot or if they are voting for the first time
            if <Votes<T>>::contains_key(&ticker_ballot_name_did) {
                //User wants to change their vote. We first need to subtract their existing vote
                let previous_votes = <Votes<T>>::get(&ticker_ballot_name_did);
                <Results<T>>::mutate(&ticker_ballot_name, |results| {
                    for i in 0..results.len() {
                        results[i] -= previous_votes[i];
                    }
                });
            }

            // Adding users' vote to the result
            <Results<T>>::mutate(&ticker_ballot_name, |results| {
                for i in 0..results.len() {
                    results[i] += votes[i];
                }
            });

            // Storing users' vote onchain. This is needed when user wants to their change vote
            <Votes<T>>::insert(&ticker_ballot_name_did, votes.clone());

            Self::deposit_event(RawEvent::VoteCast(did, ticker, ballot_name, votes));

            Ok(())
        }

        /// Cancels a vote by setting it as expired
        ///
        /// # Arguments
        /// * `ticker` - Ticker of the token for which ballot is to be cancelled
        /// * `ballot_name` - Name of the ballot
        #[weight = 500_000_000]
        pub fn cancel_ballot(origin, ticker: Ticker, ballot_name: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let signer = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), Error::<T>::InvalidSigner);
            ensure!(Self::is_owner(&ticker, did), Error::<T>::InvalidOwner);

            // This avoids cloning the variables to make the same tupple again and again
            let ticker_ballot_name = (ticker, ballot_name.clone());

            // Ensure the existance of valid ballot
            ensure!(<Ballots<T>>::contains_key(&ticker_ballot_name), Error::<T>::NotExists);
            let ballot = <Ballots<T>>::get(&ticker_ballot_name);
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(now < ballot.voting_end, Error::<T>::AlreadyEnded);

            // Clearing results
            <Results<T>>::mutate(&ticker_ballot_name, |results| {
                for result in results.iter_mut() {
                    *result = 0.into();
                }
            });

            // NB Not deleting the ballot to prevent someone from
            // deleting a ballot mid vote and creating a new one with same name to confuse voters

            // This will prevent further voting. Essentially, canceling the ballot
            <Ballots<T>>::mutate(&ticker_ballot_name, |ballot_details| {
                ballot_details.voting_end = now;
            });

            Self::deposit_event(RawEvent::BallotCancelled(did, ticker, ballot_name));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
        Moment = <T as pallet_timestamp::Trait>::Moment,
    {
        /// A new ballot is created (caller DID, Ticker, BallotName, BallotDetails)
        BallotCreated(IdentityId, Ticker, Vec<u8>, Ballot<Moment>),

        /// A vote is cast (caller DID, Ticker, BallotName, Vote)
        VoteCast(IdentityId, Ticker, Vec<u8>, Vec<Balance>),

        /// An existing ballot is cancelled (caller DID, Ticker, BallotName)
        BallotCancelled(IdentityId, Ticker, Vec<u8>),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// sender must be a secondary key for DID
        InvalidSigner,
        /// Sender must be the token owner
        InvalidOwner,
        /// A ballot with same name already exisits
        AlreadyExists,
        /// Voting end date in past / Voting end date before voting start date
        InvalidDate,
        /// No motion submitted
        NoMotions,
        /// No choice submitted
        NoChoicesInMotions,
        /// Could not decode choices
        InvalidChoicesType,
        /// Ballot does not exist
        NotExists,
        /// Voting hasn't started yet
        NotStarted,
        /// Voting ended already
        AlreadyEnded,
        /// No checkpoints created
        NoCheckpoints,
        /// Invalid vote
        InvalidVote,
        /// Not enough balance
        InsufficientBalance,
    }
}

impl<T: Trait> Module<T> {
    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, did)
    }
}
