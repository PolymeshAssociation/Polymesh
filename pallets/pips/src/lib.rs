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

//! # Pips Module
//!
//! Polymesh Improvement Proposals (PIPs) are proposals (ballots) that can be proposed and voted on
//! by all POLYX token holders. If a ballot passes this community token holder vote it is then passed to the
//! governance council to ratify (or reject).
//! - minimum of 5,000 POLYX needs to be staked by the proposer of the ballot
//! in order to create a new ballot.
//! - minimum of 100,000 POLYX (quorum) needs to vote in favour of the ballot in order for the
//! ballot to be considered by the governing committee.
//! - ballots run for 1 week
//! - a simple majority is needed to pass the ballot so that it heads for the
//! next stage (governing committee)
//!
//! ## Overview
//!
//! The Pips module provides functions for:
//!
//! - Creating Mesh Improvement Proposals
//! - Voting on Mesh Improvement Proposals
//! - Governance committee to ratify or reject proposals
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `set_prune_historical_pips` change whether historical PIPs are pruned
//! - `set_min_proposal_deposit` change min deposit to create a proposal
//! - `set_quorum_threshold` change stake required to make a proposal into a referendum
//! - `set_proposal_duration` change duration in blocks for which proposal stays active
//! - `set_proposal_cool_off_period` change duration in blocks for which a proposal can be amended
//! - `set_default_enact_period` change the period after enactment after which the proposal is executed
//! - `propose` - token holders can propose a new ballot.
//! - `amend_proposal` - allows the creator of a proposal to amend the proposal details
//! - `cancel_proposal` - allows the creator of a proposal to cancel the proposal
//! - `bond_additional_deposit` - allows the creator of a proposal to bond additional POLYX to it
//! - `unbond_deposit` - allows the creator of a proposal to unbond POLYX from it
//! - `vote` - Token holders can vote on a ballot.
//! - `kill_proposal` - close a proposal and refund all deposits
//! - `fast_track_proposal` - move a proposal to a referendum stage
//! - `emergency_referendum` - create an emergency referndum, bypassing the token holder vote
//! - `reject_referendum` - reject a referendum which will be closed without executing
//! - `override_referendum_enactment_period` - release coordinator can reschedule a referendum
//! - `enact_referendum` committee calls to execute a referendum
//!
//! ### Public Functions
//!
//! - `end_block` - processes pending proposals and referendums
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::mem;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    storage::IterableStorageMap,
    traits::{Currency, EnsureOrigin, LockableCurrency, ReservableCurrency},
    weights::{DispatchClass, Pays, Weight},
    Parameter,
};
use frame_system::{self as system, ensure_signed};
use pallet_identity as identity;
use pallet_treasury::TreasuryTrait;
use polymesh_common_utilities::{
    constants::PIP_MAX_REPORTING_SIZE,
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    traits::{governance_group::GovernanceGroupTrait, group::GroupTrait, pip::PipId},
    CommonTrait, Context, SystematicIssuers,
};
use polymesh_primitives::{Beneficiary, IdentityId};
use polymesh_primitives_derive::VecU8StrongTyped;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::traits::{
    BlakeTwo256, CheckedAdd, CheckedSub, Dispatchable, Hash, Saturating, Zero,
};
use sp_std::{convert::From, prelude::*};

/// Balance
type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// A wrapper for a proposal url.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct Url(pub Vec<u8>);

/// A wrapper for a proposal description.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct PipDescription(pub Vec<u8>);

/// Represents a proposal
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Pip<Proposal, Balance> {
    /// The proposal's unique id.
    pub id: PipId,
    /// The proposal being voted on.
    pub proposal: Proposal,
    /// The latest state
    pub state: ProposalState,
    /// Beneficiaries of this Pips
    pub beneficiaries: Option<Vec<Beneficiary<Balance>>>,
}

/// A result of execution of get_votes.
#[derive(Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum VoteCount<Balance> {
    /// Proposal was found and has the following votes.
    ProposalFound {
        /// Stake for
        ayes: Balance,
        /// Stake against
        nays: Balance,
    },
    /// Proposal was not for given index.
    ProposalNotFound,
}

/// Either the entire proposal encoded as a byte vector or its hash. The latter represents large
/// proposals.
#[derive(Encode, Decode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProposalData {
    /// The hash of the proposal.
    Hash(H256),
    /// The entire proposal.
    Proposal(Vec<u8>),
}

/// Represents a proposal metadata
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct PipsMetadata<T: Trait> {
    /// The creator
    pub proposer: T::AccountId,
    /// The proposal's unique id.
    pub id: PipId,
    /// When voting will end.
    pub end: T::BlockNumber,
    /// The proposal url for proposal discussion.
    pub url: Option<Url>,
    /// The proposal description.
    pub description: Option<PipDescription>,
    /// This proposal allows any changes
    /// During Cool-off period, proposal owner can amend any PIP detail or cancel the entire
    pub cool_off_until: T::BlockNumber,
}

/// For keeping track of proposal being voted on.
#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VotingResult<Balance: Parameter> {
    /// The current set of voters that approved with their stake.
    pub ayes_count: u32,
    pub ayes_stake: Balance,
    /// The current set of voters that rejected with their stake.
    pub nays_count: u32,
    pub nays_stake: Balance,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum Vote<Balance> {
    None,
    Yes(Balance),
    No(Balance),
}

impl<Balance> Default for Vote<Balance> {
    fn default() -> Self {
        Vote::None
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct VoteByPip<VoteType> {
    pub pip: PipId,
    pub vote: VoteType,
}

pub type HistoricalVotingByAddress<VoteType> = Vec<VoteByPip<VoteType>>;
pub type HistoricalVotingById<AccountId, VoteType> =
    Vec<(AccountId, HistoricalVotingByAddress<VoteType>)>;

/// The state a PIP is in.
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProposalState {
    /// Proposal is created and either in the cool-down period or open to voting.
    Pending,
    /// Proposal is cancelled by its owner.
    Cancelled,
    /// Proposal was rejected by the GC.
    Rejected,
    /// Proposal has been approved by the GC and scheduled for execution.
    Scheduled,
    /// Proposal execution was attempted by failed.
    Failed,
    /// Proposal was successfully executed.
    Executed,
}

impl Default for ProposalState {
    fn default() -> Self {
        ProposalState::Pending
    }
}

/// Information about deposit.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DepositInfo<AccountId, Balance> {
    /// Owner of the deposit.
    pub owner: AccountId,
    /// Amount. It can be updated during the cool off period.
    pub amount: Balance,
}

/// A snapshot's metadata, containing when it was created and who triggered it.
/// The priority queue is stored separately (see `SnapshottedPip`).
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SnapshotMetadata<T: Trait> {
    /// The block when the snapshot was made.
    pub created_at: T::BlockNumber,
    /// Who triggered this snapshot? Should refer to someone in the GC.
    pub made_by: T::AccountId,
}

/// A PIP in the snapshot's priority queue for consideration by the GC.
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SnapshottedPip<T: Trait> {
    /// Identifies the PIP this refers to.
    pub id: PipId,
    /// Weight of the proposal in the snapshot's priority queue.
    /// Higher weights come before lower weights.
    /// The `bool` denotes the sign, where `true` siginfies a positive number.
    pub weight: (bool, BalanceOf<T>),
}

/// A result to enact for one or many PIPs in the snapshot queue.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum SnapshotResult {
    /// Approve the PIP and move it to the execution queue.
    Approve,
    /// Reject the PIP, removing it from future consideration.
    Reject,
    /// Skip the PIP, bumping the `skipped_count`,
    /// or fail if the threshold for maximum skips is exceeded.
    Skip,
}

/// The number of times a PIP has been skipped.
pub type SkippedCount = u8;

type Identity<T> = identity::Module<T>;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait + pallet_timestamp::Trait + IdentityTrait + CommonTrait
{
    /// Currency type for this module.
    type Currency: ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

    /// Origin for proposals.
    type CommitteeOrigin: EnsureOrigin<Self::Origin>;

    /// Origin for enacting a referundum.
    type VotingMajorityOrigin: EnsureOrigin<Self::Origin>;

    /// Committee
    type GovernanceCommittee: GovernanceGroupTrait<<Self as pallet_timestamp::Trait>::Moment>;

    type Treasury: TreasuryTrait<<Self as CommonTrait>::Balance>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Pips {
        /// Determines whether historical PIP data is persisted or removed
        pub PruneHistoricalPips get(fn prune_historical_pips) config(): bool;

        /// The minimum amount to be used as a deposit for a public referendum proposal.
        pub MinimumProposalDeposit get(fn min_proposal_deposit) config(): BalanceOf<T>;

        /// Minimum stake a proposal must gather in order to be considered by the committee.
        pub QuorumThreshold get(fn quorum_threshold) config(): BalanceOf<T>;

        /// During Cool-off period, proposal owner can amend any PIP detail or cancel the entire
        /// proposal.
        pub ProposalCoolOffPeriod get(fn proposal_cool_off_period) config(): T::BlockNumber;

        /// How long (in blocks) a ballot runs
        pub ProposalDuration get(fn proposal_duration) config(): T::BlockNumber;

        /// Default enactment period that will be use after a proposal is accepted by GC.
        pub DefaultEnactmentPeriod get(fn default_enactment_period) config(): T::BlockNumber;

        /// Maximum times a PIP can be skipped before triggering `CannotSkipPip` in `enact_snapshot_results`.
        pub MaxPipSkipCount get(fn max_pip_skip_count) config(): SkippedCount;

        /// Proposals so far. id can be used to keep track of PIPs off-chain.
        PipIdSequence: u32;

        /// The metadata of the active proposals.
        pub ProposalMetadata get(fn proposal_metadata): map hasher(twox_64_concat) PipId => Option<PipsMetadata<T>>;

        /// Those who have locked a deposit.
        /// proposal (id, proposer) -> deposit
        pub Deposits get(fn deposits): double_map hasher(twox_64_concat) PipId, hasher(twox_64_concat) T::AccountId => DepositInfo<T::AccountId, BalanceOf<T>>;

        /// Actual proposal for a given id, if it's current.
        /// proposal id -> proposal
        pub Proposals get(fn proposals): map hasher(twox_64_concat) PipId => Option<Pip<T::Proposal, T::Balance>>;

        /// PolymeshVotes on a given proposal, if it is ongoing.
        /// proposal id -> vote count
        pub ProposalResult get(fn proposal_result): map hasher(twox_64_concat) PipId => VotingResult<BalanceOf<T>>;

        /// Votes per Proposal and account. Used to avoid double vote issue.
        /// (proposal id, account) -> Vote
        pub ProposalVotes get(fn proposal_vote): double_map hasher(twox_64_concat) PipId, hasher(twox_64_concat) T::AccountId => Vote<BalanceOf<T>>;

        /// Maps PIPs to the block at which they will be executed, if any.
        pub PipToSchedule get(fn pip_to_schedule): map hasher(twox_64_concat) PipId => Option<T::BlockNumber>;

        /// Maps block numbers to list of PIPs which should be executed at the block number.
        /// block number -> Pip id
        pub ExecutionSchedule get(fn execution_schedule): map hasher(twox_64_concat) T::BlockNumber => Vec<PipId>;

        /// The priority queue (lowest priority at index 0) of PIPs at the point of snapshotting.
        /// Priority is defined by the `weight` in the `SnapshottedPIP`.
        ///
        /// A queued PIP can be skipped. Doing so bumps the `pip_skip_count`.
        /// Once a (configurable) threshhold is exceeded, a PIP cannot be skipped again.
        pub SnapshotQueue get(fn snapshot_queue): Vec<SnapshottedPip<T>>;

        /// The metadata of the snapshot, if there is one.
        pub SnapshotMeta get(fn snapshot_metadata): Option<SnapshotMetadata<T>>;

        /// The number of times a certain PIP has been skipped.
        /// Once a (configurable) threshhold is exceeded, a PIP cannot be skipped again.
        pub PipSkipCount get(fn pip_skip_count): map hasher(twox_64_concat) PipId => SkippedCount;
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
        <T as frame_system::Trait>::AccountId,
        <T as frame_system::Trait>::BlockNumber,
    {
        /// Pruning Historical PIPs is enabled or disabled (caller DID, old value, new value)
        HistoricalPipsPruned(IdentityId, bool, bool),
        /// A PIP was made with a `Balance` stake.
        ///
        /// # Parameters:
        ///
        /// Caller DID, Proposer, PIP ID, deposit, URL, description, cool-off period end, proposal end, proposal
        /// data.
        ProposalCreated(
            IdentityId,
            AccountId,
            PipId,
            Balance,
            Option<Url>,
            Option<PipDescription>,
            BlockNumber,
            BlockNumber,
            ProposalData,
        ),
        /// A PIP's details (url & description) were amended.
        ProposalDetailsAmended(IdentityId, AccountId, PipId, Option<Url>, Option<PipDescription>),
        /// The deposit of a vote on a PIP was adjusted, either by increasing or decreasing.
        /// `true` represents an increase and `false` a decrease.
        ProposalBondAdjusted(IdentityId, AccountId, PipId, bool, Balance),
        /// Triggered each time the state of a proposal is amended
        ProposalStateUpdated(IdentityId, PipId, ProposalState),
        /// `AccountId` voted `bool` on the proposal referenced by `PipId`
        Voted(IdentityId, AccountId, PipId, bool, Balance),
        /// Pip has been closed, bool indicates whether data is pruned
        PipClosed(IdentityId, PipId, bool),
        /// Execution of a PIP has been scheduled at specific block.
        ExecutionScheduled(IdentityId, PipId, BlockNumber, BlockNumber),
        /// Default enactment period (in blocks) has been changed.
        /// (caller DID, old period, new period)
        DefaultEnactmentPeriodChanged(IdentityId, BlockNumber, BlockNumber),
        /// Minimum deposit amount modified
        /// (caller DID, old amount, new amount)
        MinimumProposalDepositChanged(IdentityId, Balance, Balance),
        /// Cool off period for proposals modified
        /// (caller DID, old period, new period)
        ProposalCoolOffPeriodChanged(IdentityId, BlockNumber, BlockNumber),
        /// Proposal duration changed
        /// (old value, new value)
        ProposalDurationChanged(IdentityId, BlockNumber, BlockNumber),
        /// The maximum times a PIP can be skipped was changed.
        /// (caller DID, old value, new value)
        MaxPipSkipCountChanged(IdentityId, SkippedCount, SkippedCount),
        /// Refund proposal
        /// (id, total amount)
        ProposalRefund(IdentityId, PipId, Balance),
        /// The snapshot was cleared.
        SnapshotCleared(IdentityId),
        /// A new snapshot was taken.
        SnapshotTaken(IdentityId),
        /// A PIP in the snapshot queue was skipped.
        /// (pip_id, new_skip_count)
        SkippedPip(PipId, SkippedCount),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Incorrect origin
        BadOrigin,
        /// Proposer specifies an incorrect deposit
        IncorrectDeposit,
        /// Proposer can't afford to lock minimum deposit
        InsufficientDeposit,
        /// when voter vote gain
        DuplicateVote,
        /// Duplicate proposal.
        DuplicateProposal,
        /// The proposal does not exist.
        NoSuchProposal,
        /// Not part of governance committee.
        NotACommitteeMember,
        /// After Cool-off period, proposals are not cancelable.
        ProposalOnCoolOffPeriod,
        /// Proposal is immutable after cool-off period.
        ProposalIsImmutable,
        /// Referendum is still on its enactment period.
        ReferendumOnEnactmentPeriod,
        /// Referendum is immutable.
        ReferendumIsImmutable,
        /// When a block number is less than current block number.
        InvalidFutureBlockNumber,
        /// When number of votes overflows.
        NumberOfVotesExceeded,
        /// When stake amount of a vote overflows.
        StakeAmountOfVotesExceeded,
        /// Missing current DID
        MissingCurrentIdentity,
        /// Cool off period is too large relative to the proposal duration
        BadCoolOffPeriod,
        /// The proposal duration is too small relative to the cool off period
        BadProposalDuration,
        /// Referendum is not in the correct state
        IncorrectReferendumState,
        /// Proposal is not in the correct state
        IncorrectProposalState,
        /// Insufficient treasury funds to pay beneficiaries
        InsufficientTreasuryFunds,
        /// When enacting snapshot results, an unskippable PIP was skipped.
        CannotSkipPip,
        /// Tried to enact results for the snapshot queue overflowing its length.
        SnapshotResultTooLarge
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Change whether completed PIPs are pruned. Can only be called by governance council
        ///
        /// # Arguments
        /// * `deposit` the new min deposit required to start a proposal
        #[weight = (150_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_prune_historical_pips(origin, new_value: bool) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::deposit_event(RawEvent::HistoricalPipsPruned(SystematicIssuers::Committee.as_id(), Self::prune_historical_pips(), new_value));
            <PruneHistoricalPips>::put(new_value);
        }

        /// Change the minimum proposal deposit amount required to start a proposal. Only Governance
        /// committee is allowed to change this value.
        ///
        /// # Arguments
        /// * `deposit` the new min deposit required to start a proposal
        #[weight = (150_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_min_proposal_deposit(origin, deposit: BalanceOf<T>) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::deposit_event(RawEvent::MinimumProposalDepositChanged(SystematicIssuers::Committee.as_id(), Self::min_proposal_deposit(), deposit));
            <MinimumProposalDeposit<T>>::put(deposit);
        }

        /// Change the quorum threshold amount. This is the amount which a proposal must gather so
        /// as to be considered by a committee. Only Governance committee is allowed to change
        /// this value.
        ///
        /// # Arguments
        /// * `threshold` the new quorum threshold amount value
        #[weight = (150_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_quorum_threshold(origin, threshold: BalanceOf<T>) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::deposit_event(RawEvent::MinimumProposalDepositChanged(SystematicIssuers::Committee.as_id(), Self::quorum_threshold(), threshold));
            <QuorumThreshold<T>>::put(threshold);
        }

        /// Change the proposal duration value. This is the number of blocks for which votes are
        /// accepted on a proposal. Only Governance committee is allowed to change this value.
        ///
        /// # Arguments
        /// * `duration` proposal duration in blocks
        #[weight = (150_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_proposal_duration(origin, duration: T::BlockNumber) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::deposit_event(RawEvent::ProposalDurationChanged(SystematicIssuers::Committee.as_id(), Self::proposal_duration(), duration));
            <ProposalDuration<T>>::put(duration);
        }

        /// Change the proposal cool off period value. This is the number of blocks after which the proposer of a pip
        /// can modify or cancel their proposal, and other voting is prohibited
        ///
        /// # Arguments
        /// * `duration` proposal cool off period duration in blocks
        #[weight = (150_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_proposal_cool_off_period(origin, duration: T::BlockNumber) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::deposit_event(RawEvent::ProposalDurationChanged(SystematicIssuers::Committee.as_id(), Self::proposal_cool_off_period(), duration));
            <ProposalCoolOffPeriod<T>>::put(duration);
        }

        /// Change the default enact period.
        #[weight = (300_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_default_enactment_period(origin, duration: T::BlockNumber) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            let previous_duration = <DefaultEnactmentPeriod<T>>::get();
            <DefaultEnactmentPeriod<T>>::put(duration);
            Self::deposit_event(RawEvent::DefaultEnactmentPeriodChanged(SystematicIssuers::Committee.as_id(), duration, previous_duration));
        }

        /// Change the maximum skip count (`max_pip_skip_count`).
        /// New values only
        #[weight = (100_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_max_pip_skip_count(origin, new_max: SkippedCount) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            let prev_max = MaxPipSkipCount::get();
            MaxPipSkipCount::put(prev_max);
            Self::deposit_event(RawEvent::MaxPipSkipCountChanged(SystematicIssuers::Committee.as_id(), prev_max, new_max));
        }

        /// A network member creates a PIP by submitting a dispatchable which
        /// changes the network in someway. A minimum deposit is required to open a new proposal.
        ///
        /// # Arguments
        /// * `proposal` a dispatchable call
        /// * `deposit` minimum deposit value
        /// * `url` a link to a website for proposal discussion
        #[weight = (1_850_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn propose(
            origin,
            proposal: Box<T::Proposal>,
            deposit: BalanceOf<T>,
            url: Option<Url>,
            description: Option<PipDescription>,
            beneficiaries: Option<Vec<Beneficiary<T::Balance>>>
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;

            // Pre conditions: caller must have min balance
            ensure!(
                deposit >= Self::min_proposal_deposit(),
                Error::<T>::IncorrectDeposit
            );

            // Reserve the minimum deposit
            <T as Trait>::Currency::reserve(&proposer, deposit).map_err(|_| Error::<T>::InsufficientDeposit)?;
            <T as IdentityTrait>::ProtocolFee::charge_fee(ProtocolOp::PipsPropose)?;

            let id = Self::next_pip_id();
            let curr_block_number = <system::Module<T>>::block_number();
            let cool_off_until = curr_block_number + Self::proposal_cool_off_period();
            let end = cool_off_until + Self::proposal_duration();
            let proposal_metadata = PipsMetadata {
                proposer: proposer.clone(),
                id,
                end: end,
                url: url.clone(),
                description: description.clone(),
                cool_off_until: cool_off_until,
            };
            <ProposalMetadata<T>>::insert(id, proposal_metadata);

            let deposit_info = DepositInfo {
                owner: proposer.clone(),
                amount: deposit
            };
            <Deposits<T>>::insert(id, &proposer, deposit_info);
            let proposal_data = Self::reportable_proposal_data(&*proposal);
            let pip = Pip {
                id,
                proposal: *proposal,
                state: ProposalState::Pending,
                beneficiaries,
            };
            <Proposals<T>>::insert(id, pip);

            // Add vote and update voting counter.
            // INTERNAL: It is impossible to overflow counters in the first vote.
            Self::unsafe_vote( id, proposer.clone(), Vote::Yes(deposit))
                .map_err(|vote_error| {
                    debug::error!("The counters of voting (id={}) have an overflow during the 1st vote", id);
                    vote_error
                })?;
            let current_did = Self::current_did_or_missing()?;
            Self::deposit_event(RawEvent::ProposalCreated(
                current_did,
                proposer,
                id,
                deposit,
                url,
                description,
                cool_off_until,
                end,
                proposal_data,
                //beneficiaries,
            ));
            Ok(())
        }

        /// It amends the `url` and the `description` of the proposal with `id`.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can amend it.
        /// * `ProposalIsImmutable`: A proposals is mutable only during its cool off period.
        ///
        #[weight = (1_000_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn amend_proposal(
                origin,
                id: PipId,
                url: Option<Url>,
                description: Option<PipDescription>
        ) -> DispatchResult {
            // 1. Fetch proposer and perform sanity checks.
            let proposer = Self::ensure_owned_by_alterable(origin, id)?;

            // 2. Update proposal metadata.
            <ProposalMetadata<T>>::mutate( id, |meta| {
                if let Some(meta) = meta {
                    meta.url = url.clone();
                    meta.description = description.clone();
                }
            });

            // 3. Emit event.
            let current_did = Self::current_did_or_missing()?;
            Self::deposit_event(RawEvent::ProposalDetailsAmended(current_did, proposer, id, url, description));

            Ok(())
        }

        /// It cancels the proposal of the id `id`.
        ///
        /// Proposals can be cancelled only during its _cool-off period.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can amend it.
        /// * `ProposalIsImmutable`: A Proposal is mutable only during its cool off period.
        #[weight = (750_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn cancel_proposal(origin, id: PipId) -> DispatchResult {
            // 1. Fetch proposer and perform sanity checks.
            let _ = Self::ensure_owned_by_alterable(origin, id)?;

            // 2. Refund the bond for the proposal.
            Self::refund_proposal(id);

            // 3. Close that proposal.
            Self::update_proposal_state(id, ProposalState::Cancelled);
            Self::prune_data(id, Self::prune_historical_pips());

            Ok(())
        }

        /// Id bonds an additional deposit to proposal with id `id`.
        /// That amount is added to the current deposit.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can bond an additional deposit.
        /// * `ProposalIsImmutable`: A Proposal is mutable only during its cool off period.
        #[weight = 900_000_000]
        pub fn bond_additional_deposit(origin,
            id: PipId,
            additional_deposit: BalanceOf<T>
        ) -> DispatchResult {
            // 1. Sanity checks.
            let proposer = Self::ensure_owned_by_alterable(origin, id)?;

            // 2. Reserve extra deposit & update deposit info for this proposal
            let curr_deposit = Self::deposits(id, &proposer).amount;
            let max_additional_deposit = curr_deposit.saturating_add( additional_deposit) - curr_deposit;
            <T as Trait>::Currency::reserve(&proposer, max_additional_deposit)
                .map_err(|_| Error::<T>::InsufficientDeposit)?;

            <Deposits<T>>::mutate(
                id,
                &proposer,
                |depo_info| depo_info.amount += max_additional_deposit);

            // 3. Update vote details to record additional vote
            <ProposalResult<T>>::mutate(
                id,
                |stats| stats.ayes_stake += max_additional_deposit
            );
            <ProposalVotes<T>>::insert(id, &proposer, Vote::Yes(curr_deposit + max_additional_deposit));

            Self::emit_proposal_bond_adjusted(proposer, id, true, max_additional_deposit)
        }

        /// It unbonds any amount from the deposit of the proposal with id `id`.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can release part of the deposit.
        /// * `ProposalIsImmutable`: A Proposal is mutable only during its cool off period.
        /// * `InsufficientDeposit`: If the final deposit will be less that the minimum deposit for
        /// a proposal.
        #[weight = 900_000_000]
        pub fn unbond_deposit(origin,
            id: PipId,
            amount: BalanceOf<T>
        ) -> DispatchResult {
            // 1. Sanity checks.
            let proposer = Self::ensure_owned_by_alterable(origin, id)?;

            // 2. Double-check that `amount` is valid.
            let mut depo_info = Self::deposits(id, &proposer);
            let new_deposit = depo_info.amount.checked_sub(&amount)
                    .ok_or_else(|| Error::<T>::InsufficientDeposit)?;
            ensure!(
                new_deposit >= Self::min_proposal_deposit(),
                Error::<T>::IncorrectDeposit);
            let diff_amount = depo_info.amount - new_deposit;
            depo_info.amount = new_deposit;

            // 2.1. Unreserve and update deposit info.
            <T as Trait>::Currency::unreserve(&depo_info.owner, diff_amount);
            <Deposits<T>>::insert(id, &proposer, depo_info);

            // 3. Update vote details to record reduced vote
            <ProposalResult<T>>::mutate(
                id,
                |stats| stats.ayes_stake = new_deposit
            );
            <ProposalVotes<T>>::insert(id, &proposer, Vote::Yes(new_deposit));

            Self::emit_proposal_bond_adjusted(proposer, id, false, amount)
        }

        /// A network member can vote on any PIP by selecting the id that
        /// corresponds ot the dispatchable action and vote with some balance.
        ///
        /// # Arguments
        /// * `proposal` a dispatchable call
        /// * `id` proposal id
        /// * `aye_or_nay` a bool representing for or against vote
        /// * `deposit` minimum deposit value
        #[weight = 1_000_000_000]
        pub fn vote(origin, id: PipId, aye_or_nay: bool, deposit: BalanceOf<T>) {
            let proposer = ensure_signed(origin)?;
            let meta = Self::proposal_metadata(id)
                .ok_or_else(|| Error::<T>::NoSuchProposal)?;

            // No one should be able to vote during the proposal cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!( meta.cool_off_until <= curr_block_number, Error::<T>::ProposalOnCoolOffPeriod);

            // Check that the proposal is pending
            Self::is_proposal_state(id, ProposalState::Pending)?;

            // Valid PipId
            ensure!(<ProposalResult<T>>::contains_key(id), Error::<T>::NoSuchProposal);

            // Double-check vote duplication.
            ensure!( Self::proposal_vote(id, &proposer) == Vote::None, Error::<T>::DuplicateVote);

            // Reserve the deposit
            <T as Trait>::Currency::reserve(&proposer, deposit).map_err(|_| Error::<T>::InsufficientDeposit)?;

            // Save your vote.
            let vote = if aye_or_nay {
                Vote::Yes(deposit)
            } else {
                Vote::No(deposit)
            };
            Self::unsafe_vote( id, proposer.clone(), vote)
                .map_err( |vote_error| {
                    debug::warn!("The counters of voting (id={}) have an overflow, transaction is roll-back", id);
                    let _ = <T as Trait>::Currency::unreserve(&proposer, deposit);
                    vote_error
                })?;

            let depo_info = DepositInfo {
                owner: proposer.clone(),
                amount: deposit,
            };
            <Deposits<T>>::insert(id, &proposer, depo_info);
            let current_did = Self::current_did_or_missing()?;
            Self::deposit_event(RawEvent::Voted(current_did, proposer, id, aye_or_nay, deposit));
        }

        /// Rejects the PIP given by the `id`, refunding any bonded funds,
        /// assuming it hasn't been cancelled or executed.
        /// Note that cooling-off and proposals scheduled-for-execution can also be rejected.
        ///
        /// # Errors
        /// * `BadOrigin` unless a GC voting majority executes this function.
        /// * `NoSuchProposal` if the PIP with `id` doesn't exist.
        /// * `IncorrectProposalState` if the proposal was cancelled or executed.
        #[weight = (550_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn reject_proposal(origin, id: PipId, prune: bool) {
            T::VotingMajorityOrigin::ensure_origin(origin)?;
            let proposal = Self::proposals(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
            ensure!(
                !matches!(proposal.state, ProposalState::Cancelled | ProposalState::Failed | ProposalState::Executed),
                Error::<T>::IncorrectProposalState,
            );
            Self::maybe_unschedule_pip(id, proposal.state);
            Self::unsafe_reject_proposal(id);
        }

        /// Prune the PIP given by the `id`, refunding any funds not already refunded.
        /// *No restrictions* on the PIP applies.
        ///
        /// This function is intended for storage garbage collection purposes.
        ///
        /// # Errors
        /// * `BadOrigin` unless a GC voting majority executes this function.
        /// * `NoSuchProposal` if the PIP with `id` doesn't exist.
        #[weight = (550_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn prune_proposal(origin, id: PipId) {
            T::VotingMajorityOrigin::ensure_origin(origin)?;
            let proposal = Self::proposals(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
            Self::refund_proposal(id);
            Self::maybe_unschedule_pip(id, proposal.state);
            Self::prune_data(id, true);
        }

        /// Updates the execution schedule of the PIP given by `id`.
        ///
        /// # Arguments
        /// * `until` defines the future block where the enactment period will finished.
        ///    `None` value means that enactment period is going to finish in the next block.
        ///
        /// # Errors
        /// * `BadOrigin` unless triggered by release coordinator.
        /// * `IncorrectProposalState` unless the proposal was in a scheduled state.
        #[weight = (750_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn override_referendum_enactment_period(origin, id: PipId, until: Option<T::BlockNumber>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let current_did = Context::current_identity_or::<Identity<T>>(&sender)?;

            // 1. Only release coordinator
            ensure!(
                Some(current_did) == T::GovernanceCommittee::release_coordinator(),
                Error::<T>::BadOrigin
            );

            Self::is_proposal_state(id, ProposalState::Scheduled)?;

            // 2. New value should be valid block number.
            let next_block = <system::Module<T>>::block_number() + 1.into();
            let new_until = until.unwrap_or(next_block);
            ensure!(new_until >= next_block, Error::<T>::InvalidFutureBlockNumber);

            // 3. Update enactment period & reschule it.
            let old_until = <PipToSchedule<T>>::mutate(id, |old| mem::replace(old, Some(new_until))).unwrap();
            <ExecutionSchedule<T>>::append(new_until, id);
            Self::remove_pip_from_schedule(old_until, id);

            // 4. Emit event.
            Self::deposit_event(RawEvent::ExecutionScheduled(current_did, id, old_until, new_until));
            Ok(())
        }

        /// Clears the snapshot and emits the event `SnapshotCleared`.
        ///
        /// # Errors
        /// * `NotACommitteeMember` - triggered when a non-GC-member executes the function.
        #[weight = (100_000, DispatchClass::Operational, Pays::Yes)]
        pub fn clear_snapshot(origin) -> DispatchResult {
            // 1. Check that a GC member is executing this.
            let actor = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&actor)?;
            ensure!(T::GovernanceCommittee::is_member(&did), Error::<T>::NotACommitteeMember);

            // 2. Clear the snapshot.
            <SnapshotMeta<T>>::kill();
            <SnapshotQueue<T>>::kill();

            // 3. Emit event.
            Self::deposit_event(RawEvent::SnapshotCleared(did));
            Ok(())
        }

        /// Takes a new snapshot of the current list of active && pending PIPs.
        /// The PIPs are then sorted into a priority queue based on each PIP's weight.
        ///
        /// # Errors
        /// * `NotACommitteeMember` - triggered when a non-GC-member executes the function.
        #[weight = (100_000, DispatchClass::Operational, Pays::Yes)]
        pub fn snapshot(origin) -> DispatchResult {
            // 1. Check that a GC member is executing this.
            let made_by = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&made_by)?;
            ensure!(T::GovernanceCommittee::is_member(&did), Error::<T>::NotACommitteeMember);

            // 2. Fetch intersection of pending && non-cooling PIPs and aggregate their votes.
            let created_at = <system::Module<T>>::block_number();
            let mut queue = <Proposals<T>>::iter_values()
                // Only keep pending PIPs.
                .filter(|pip| matches!(pip.state, ProposalState::Pending))
                .map(|pip| pip.id)
                // Omit cooling-off PIPs.
                .filter(|id| {
                    <ProposalMetadata<T>>::get(id)
                        .filter(|meta| meta.cool_off_until > created_at)
                        .is_some()
                })
                // Aggregate the votes; `true` denotes a positive sign.
                .map(|id| {
                    let VotingResult { ayes_stake, nays_stake, .. } = <ProposalResult<T>>::get(id);
                    let weight = if ayes_stake > nays_stake {
                        (true, ayes_stake - nays_stake)
                    } else {
                        (false, nays_stake - ayes_stake)
                    };
                    SnapshottedPip { id, weight }
                })
                .collect::<Vec<_>>();
            queue.sort_by_key(|s| s.weight);

            // 3. Commit the new snapshot.
            <SnapshotMeta<T>>::set(Some(SnapshotMetadata { created_at, made_by }));
            <SnapshotQueue<T>>::set(queue);

            // 3. Emit event.
            Self::deposit_event(RawEvent::SnapshotTaken(did));
            Ok(())
        }

        /// Enacts results for the PIPs in the snapshot queue.
        /// The snapshot will be available for further enactments until it is cleared.
        ///
        /// The `results` are encoded a list of `(n, result)` where `result` is applied to `n` PIPs.
        /// Note that the snapshot priority queue is encoded with the *lowest priority first*.
        /// so `results = [(2, Approve)]` will approve `SnapshotQueue[snapshot.len() - 2..]`.
        ///
        /// # Errors
        /// * `BadOrigin` - unless a GC voting majority executes this function.
        /// * `CannotSkipPip` - a given PIP has already been skipped too many times.
        /// * `SnapshotResultTooLarge` - on len(results) > len(snapshot_queue).
        #[weight = (100_000, DispatchClass::Operational, Pays::Yes)]
        pub fn enact_snapshot_results(origin, results: Vec<(SkippedCount, SnapshotResult)>) -> DispatchResult {
            T::VotingMajorityOrigin::ensure_origin(origin)?;

            let max_pip_skip_count = Self::max_pip_skip_count();

            <SnapshotQueue<T>>::try_mutate(|queue| {
                let mut to_bump_skipped = Vec::new();
                let mut to_reject = Vec::new();
                let mut to_approve = Vec::new();

                // Go over each result, which may apply up to `num` queued elements...
                for result in results.iter().copied().flat_map(|(n, r)| (0..n).map(move |_| r)) {
                    match (queue.pop(), result) { // ...and zip with the queue in reverse.
                        // An action is missing a corresponding PIP in the queue, bail!
                        (None, _) => Err(Error::<T>::SnapshotResultTooLarge)?,
                        // Make sure the PIP can be skipped and enqueue bumping of skip.
                        (Some(pip), SnapshotResult::Skip) => {
                            let count = PipSkipCount::get(pip.id);
                            ensure!(count >= max_pip_skip_count, Error::<T>::CannotSkipPip);
                            to_bump_skipped.push((pip.id, count + 1));
                        },
                        // Mark PIP as rejected.
                        (Some(pip), SnapshotResult::Reject) => to_reject.push(pip.id),
                        // Approve PIP.
                        (Some(pip), SnapshotResult::Approve) => to_approve.push(pip.id),
                    }
                }

                // Update skip counts.
                for (pip_id, new_count) in to_bump_skipped {
                    PipSkipCount::insert(pip_id, new_count);
                    Self::deposit_event(RawEvent::SkippedPip(pip_id, new_count));
                }

                // Reject proposals as instructed & refund.
                // TODO(centril): is refunding working properly?
                for pip_id in to_reject {
                    Self::unsafe_reject_proposal(pip_id);
                }

                // Approve proposals as instructed.
                // TODO(centril): is refunding working properly?
                // TODO(centril): will need some more tweaks.
                let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
                for pip_id in to_approve {
                    Self::schedule_pip_for_execution(current_did, pip_id);
                }

                Ok(())
            })
        }

        /// When constructing a block check if it's time for a ballot to end. If ballot ends,
        /// proceed to ratification process.
        fn on_initialize(n: T::BlockNumber) -> Weight {
            Self::end_block(n).unwrap_or_else(|e| {
                sp_runtime::print(e);
                0
            })
        }

    }
}

impl<T: Trait> Module<T> {
    /// Returns the current identity or emits `MissingCurrentIdentity`.
    fn current_did_or_missing() -> Result<IdentityId, Error<T>> {
        Context::current_identity::<Identity<T>>().ok_or_else(|| Error::<T>::MissingCurrentIdentity)
    }

    fn emit_proposal_bond_adjusted(
        proposer: T::AccountId,
        id: PipId,
        increased: bool,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let current_did = Self::current_did_or_missing()?;
        let event = RawEvent::ProposalBondAdjusted(current_did, proposer, id, increased, amount);
        Self::deposit_event(event);
        Ok(())
    }

    /// Ensure that proposer is owner of the proposal which mustn't be in the cool off period.
    ///
    /// # Errors
    /// * `BadOrigin`: Only the owner of the proposal can mutate it.
    /// * `ProposalIsImmutable`: A Proposal is mutable only during its cool off period.
    fn ensure_owned_by_alterable(
        origin: T::Origin,
        id: PipId,
    ) -> Result<T::AccountId, DispatchError> {
        let proposer = ensure_signed(origin)?;
        let meta = Self::proposal_metadata(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;

        // 1. Only owner can act on proposal.
        ensure!(meta.proposer == proposer, Error::<T>::BadOrigin);
        // 2. Check that the proposal is pending.
        Self::is_proposal_state(id, ProposalState::Pending)?;

        // 3. Proposal is *ONLY* alterable during its cool-off period.
        let curr_block_number = <system::Module<T>>::block_number();
        ensure!(
            meta.cool_off_until > curr_block_number,
            Error::<T>::ProposalIsImmutable
        );

        Ok(proposer)
    }

    /// Runs the following procedure:
    /// 1. Find all proposals that need to end as of this block and close voting
    /// 2. Tally votes
    /// 3. Submit any proposals that meet the quorum threshold, to the governance committee
    /// 4. Automatically execute any referendum
    pub fn end_block(block_number: T::BlockNumber) -> Result<Weight, DispatchError> {
        // Some arbitrary number right now, It is subject to change after proper benchmarking
        let mut weight: Weight = 50_000_000;
        // Execute automatically referendums after its enactment period.
        let referendum_ids = <ExecutionSchedule<T>>::take(block_number);
        referendum_ids.into_iter().for_each(|id| {
            <PipToSchedule<T>>::remove(id);
            weight += Self::execute_proposal(id);
        });
        <ExecutionSchedule<T>>::remove(block_number);
        Ok(weight)
    }

    /// Rejects the given `id`, refunding the deposit, and possibly pruning the proposal's data.
    fn unsafe_reject_proposal(id: PipId) {
        Self::update_proposal_state(id, ProposalState::Rejected);
        Self::refund_proposal(id);
        Self::prune_data(id, Self::prune_historical_pips());
    }

    /// Refunds any tokens used to vote or bond a proposal
    fn refund_proposal(id: PipId) {
        let total_refund =
            <Deposits<T>>::iter_prefix_values(id).fold(0.into(), |acc, depo_info| {
                let amount = <T as Trait>::Currency::unreserve(&depo_info.owner, depo_info.amount);
                amount.saturating_add(acc)
            });
        <Deposits<T>>::remove_prefix(id);
        let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
        Self::deposit_event(RawEvent::ProposalRefund(current_did, id, total_refund));
    }

    /// Unschedule PIP with given `id` if it's scheduled for execution.
    fn maybe_unschedule_pip(id: PipId, state: ProposalState) {
        if let ProposalState::Scheduled = state {
            Self::remove_pip_from_schedule(<PipToSchedule<T>>::take(id).unwrap(), id);
        }
    }

    /// Remove the PIP with `id` from the `ExecutionSchedule` at `block_no`.
    fn remove_pip_from_schedule(block_no: T::BlockNumber, id: PipId) {
        <ExecutionSchedule<T>>::mutate(block_no, |ids| ids.retain(|i| *i != id));
    }

    /// Prunes all data associated with a proposal, removing it from storage.
    ///
    /// # Internal
    /// * `ProposalsMaturingat` does not need to be deleted here.
    ///
    /// # TODO
    /// * Should we remove the proposal when it is Cancelled?, killed?, rejected?
    fn prune_data(id: PipId, prune: bool) {
        let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
        if prune {
            <ProposalResult<T>>::remove(id);
            <ProposalVotes<T>>::remove_prefix(id);
            <ProposalMetadata<T>>::remove(id);
            <Proposals<T>>::remove(id);
            PipSkipCount::remove(id);
        }
        Self::deposit_event(RawEvent::PipClosed(current_did, id, prune));
    }

    fn schedule_pip_for_execution(current_did: IdentityId, id: PipId) {
        // Set the default enactment period and move it to `Scheduled`
        let curr_block_number = <system::Module<T>>::block_number();
        let enactment_period = curr_block_number + Self::default_enactment_period();

        Self::update_proposal_state(id, ProposalState::Scheduled);
        <PipToSchedule<T>>::insert(id, enactment_period);
        <ExecutionSchedule<T>>::append(enactment_period, id);
        Self::deposit_event(RawEvent::ExecutionScheduled(
            current_did,
            id,
            Zero::zero(),
            enactment_period,
        ));
    }

    fn execute_proposal(id: PipId) -> Weight {
        let mut actual_weight: Weight = 0;
        if let Some(proposal) = Self::proposals(id) {
            if proposal.state == ProposalState::Scheduled {
                match Self::check_beneficiaries(id) {
                    Ok(_) => {
                        match proposal.proposal.dispatch(system::RawOrigin::Root.into()) {
                            Ok(post_info) => {
                                actual_weight = post_info.actual_weight.unwrap_or(0);
                                Self::pay_to_beneficiaries(id);
                                Self::update_proposal_state(id, ProposalState::Executed);
                            }
                            Err(e) => {
                                Self::update_proposal_state(id, ProposalState::Failed);
                                debug::error!(
                                    "Proposal {}, its execution fails: {:?}",
                                    id,
                                    e.error
                                );
                            }
                        };
                    }
                    Err(e) => {
                        Self::update_proposal_state(id, ProposalState::Failed);
                        debug::error!("Proposal {}, its beneficiaries fails: {:?}", id, e);
                    }
                }
                Self::prune_data(id, Self::prune_historical_pips());
            }
        }
        actual_weight
    }

    fn check_beneficiaries(id: PipId) -> DispatchResult {
        if let Some(proposal) = Self::proposals(id) {
            if let Some(beneficiaries) = proposal.beneficiaries {
                let total_amount = beneficiaries
                    .iter()
                    .fold(0.into(), |acc, b| b.amount.saturating_add(acc));
                ensure!(
                    T::Treasury::balance() >= total_amount,
                    Error::<T>::InsufficientTreasuryFunds
                );
            }
        }
        Ok(())
    }

    fn pay_to_beneficiaries(id: PipId) {
        if let Some(proposal) = Self::proposals(id) {
            if let Some(beneficiaries) = proposal.beneficiaries {
                let _ = beneficiaries.into_iter().fold(0.into(), |acc, b| {
                    T::Treasury::disbursement(b.id, b.amount);
                    b.amount.saturating_add(acc)
                });
            }
        }
    }

    fn update_proposal_state(id: PipId, new_state: ProposalState) {
        <Proposals<T>>::mutate(id, |proposal| {
            if let Some(ref mut proposal) = proposal {
                proposal.state = new_state;
            }
        });
        let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
        Self::deposit_event(RawEvent::ProposalStateUpdated(current_did, id, new_state));
    }

    fn is_proposal_state(id: PipId, state: ProposalState) -> DispatchResult {
        let proposal = Self::proposals(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
        ensure!(proposal.state == state, Error::<T>::IncorrectProposalState);
        Ok(())
    }
}

impl<T: Trait> Module<T> {
    /// Retrieve votes for a proposal represented by PipId `id`.
    pub fn get_votes(id: PipId) -> VoteCount<BalanceOf<T>>
    where
        T: Send + Sync,
        BalanceOf<T>: Send + Sync,
    {
        if !<ProposalResult<T>>::contains_key(id) {
            return VoteCount::ProposalNotFound;
        }

        let voting = Self::proposal_result(id);
        VoteCount::ProposalFound {
            ayes: voting.ayes_stake,
            nays: voting.nays_stake,
        }
    }

    /// Retrieve proposals made by `address`.
    pub fn proposed_by(address: T::AccountId) -> Vec<PipId> {
        <ProposalMetadata<T>>::iter()
            .filter(|(_, meta)| meta.proposer == address)
            .map(|(_, meta)| meta.id)
            .collect()
    }

    /// Retrieve proposals `address` voted on
    pub fn voted_on(address: T::AccountId) -> Vec<PipId> {
        <ProposalMetadata<T>>::iter()
            .filter_map(|(_, meta)| match Self::proposal_vote(meta.id, &address) {
                Vote::None => None,
                _ => Some(meta.id),
            })
            .collect::<Vec<_>>()
    }

    /// Retrieve historical voting of `who` account.
    pub fn voting_history_by_address(
        who: T::AccountId,
    ) -> HistoricalVotingByAddress<Vote<BalanceOf<T>>> {
        <ProposalMetadata<T>>::iter()
            .map(|(_, meta)| VoteByPip {
                pip: meta.id,
                vote: Self::proposal_vote(meta.id, &who),
            })
            .collect::<Vec<_>>()
    }

    /// Retrieve historical voting of `who` identity.
    /// It fetches all its keys recursively and it returns the voting history for each of them.
    pub fn voting_history_by_id(
        who: IdentityId,
    ) -> HistoricalVotingById<T::AccountId, Vote<BalanceOf<T>>> {
        let flatten_keys = <Identity<T>>::flatten_keys(who, 1);
        flatten_keys
            .into_iter()
            .map(|key| (key.clone(), Self::voting_history_by_address(key)))
            .collect::<HistoricalVotingById<_, _>>()
    }

    /// It generates the next id for proposals and referendums.
    fn next_pip_id() -> u32 {
        let id = <PipIdSequence>::get();
        <PipIdSequence>::put(id + 1);

        id
    }

    /// It inserts the vote and updates the accountability of target proposal.
    fn unsafe_vote(id: PipId, proposer: T::AccountId, vote: Vote<BalanceOf<T>>) -> DispatchResult {
        let mut stats = Self::proposal_result(id);
        match vote {
            Vote::Yes(deposit) => {
                stats.ayes_count = stats
                    .ayes_count
                    .checked_add(1)
                    .ok_or_else(|| Error::<T>::NumberOfVotesExceeded)?;
                stats.ayes_stake = stats
                    .ayes_stake
                    .checked_add(&deposit)
                    .ok_or_else(|| Error::<T>::StakeAmountOfVotesExceeded)?;
            }
            Vote::No(deposit) => {
                stats.nays_count += stats
                    .nays_count
                    .checked_add(1)
                    .ok_or_else(|| Error::<T>::NumberOfVotesExceeded)?;
                stats.nays_stake += stats
                    .nays_stake
                    .checked_add(&deposit)
                    .ok_or_else(|| Error::<T>::StakeAmountOfVotesExceeded)?;
            }
            Vote::None => {
                // It should be unreachable because public API only allows binary options.
                debug::warn!("Unexpected none vote");
            }
        };

        <ProposalResult<T>>::insert(id, stats);
        <ProposalVotes<T>>::insert(id, proposer, vote);
        Ok(())
    }

    /// Returns a reportable representation of a proposal taking care that the reported data are not
    /// too large.
    fn reportable_proposal_data(proposal: &T::Proposal) -> ProposalData {
        let encoded_proposal = proposal.encode();
        let proposal_data = if encoded_proposal.len() > PIP_MAX_REPORTING_SIZE {
            ProposalData::Hash(BlakeTwo256::hash(encoded_proposal.as_slice()))
        } else {
            ProposalData::Proposal(encoded_proposal)
        };
        proposal_data
    }
}
