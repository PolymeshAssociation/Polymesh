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

/// The various sorts of committees that can make a PIP.
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Committee {
    /// The technical committee.
    Technical,
    /// The upgrade committee tends to propose chain upgrades.
    Upgrade,
}

/// The proposer of a certain PIP.
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Proposer<AccountId> {
    /// The proposer is of the community.
    Community(AccountId),
    /// The proposer is a committee.
    Committee(Committee),
}

/// Represents a proposal metadata
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PipsMetadata<T: Trait> {
    /// They creator
    pub proposer: Proposer<T::AccountId>,
    /// The proposal's unique id.
    pub id: PipId,
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

/// A "vote" or "signal" on a PIP to move it up or down the review queue.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct Vote<Balance>(
    /// `true` if there's agreement.
    pub bool,
    /// How strongly do they feel about it?
    pub Balance,
);

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

    /// Voting majority origin for Technical Committee.
    type TechnicalCommitteeVMO: EnsureOrigin<Self::Origin>;

    /// Voting majority origin for Upgrade Committee.
    type UpgradeCommitteeVMO: EnsureOrigin<Self::Origin>;

    type Treasury: TreasuryTrait<<Self as CommonTrait>::Balance>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Pips {
        /// Determines whether historical PIP data is persisted or removed
        pub PruneHistoricalPips get(fn prune_historical_pips) config(): bool;

        /// The minimum amount to be used as a deposit for community PIP creation.
        pub MinimumProposalDeposit get(fn min_proposal_deposit) config(): BalanceOf<T>;

        /// During Cool-off period, proposal owner can amend any PIP detail or cancel the entire
        /// proposal.
        pub ProposalCoolOffPeriod get(fn proposal_cool_off_period) config(): T::BlockNumber;

        /// Default enactment period that will be use after a proposal is accepted by GC.
        pub DefaultEnactmentPeriod get(fn default_enactment_period) config(): T::BlockNumber;

        /// Maximum times a PIP can be skipped before triggering `CannotSkipPip` in `enact_snapshot_results`.
        pub MaxPipSkipCount get(fn max_pip_skip_count) config(): SkippedCount;

        /// The maximum allowed number for `ActivePipCount`.
        /// Once reached, new PIPs cannot be proposed by community members.
        pub ActivePipLimit get(fn active_pip_limit) config(): u32;

        /// Proposals so far. id can be used to keep track of PIPs off-chain.
        PipIdSequence: u32;

        /// Total count of current pending or scheduled PIPs.
        ActivePipCount: u32;

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
        pub ProposalVotes get(fn proposal_vote): double_map hasher(twox_64_concat) PipId, hasher(twox_64_concat) T::AccountId => Option<Vote<BalanceOf<T>>>;

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
        /// Caller DID, Proposer, PIP ID, deposit, URL, description, cool-off period end, proposal data.
        ProposalCreated(
            IdentityId,
            Proposer<AccountId>,
            PipId,
            Balance,
            Option<Url>,
            Option<PipDescription>,
            BlockNumber,
            ProposalData,
        ),
        /// A PIP's details (url & description) were amended.
        ProposalDetailsAmended(IdentityId, Proposer<AccountId>, PipId, Option<Url>, Option<PipDescription>),
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
        /// The maximum times a PIP can be skipped was changed.
        /// (caller DID, old value, new value)
        MaxPipSkipCountChanged(IdentityId, SkippedCount, SkippedCount),
        /// The maximum number of active PIPs was changed.
        /// (caller DID, old value, new value)
        ActivePipLimitChanged(IdentityId, u32, u32),
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
        /// Results (e.g., approved, rejected, and skipped), were enacted for some PIPs.
        /// (skipped_pips_with_new_count, rejected_pips, approved_pips)
        SnapshotResultsEnacted(Vec<(PipId, SkippedCount)>, Vec<PipId>, Vec<PipId>),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Incorrect origin
        BadOrigin,
        /// The given dispatchable call is not valid for this proposal.
        /// The proposal must be from the community, but isn't.
        NotFromCommunity,
        /// The given dispatchable call is not valid for this proposal.
        /// The proposal must be by community, but isn't.
        NotByCommittee,
        /// The current number of active (pending | scheduled) PIPs exceed the maximum
        /// and the proposal is not by a committee.
        TooManyActivePips,
        /// Proposer specifies an incorrect deposit
        IncorrectDeposit,
        /// Proposer can't afford to lock minimum deposit
        InsufficientDeposit,
        /// when voter vote gain
        DuplicateVote,
        /// The proposal does not exist.
        NoSuchProposal,
        /// Not part of governance committee.
        NotACommitteeMember,
        /// After Cool-off period, proposals are not cancelable.
        ProposalOnCoolOffPeriod,
        /// Proposal is immutable after cool-off period.
        ProposalIsImmutable,
        /// When a block number is less than current block number.
        InvalidFutureBlockNumber,
        /// When number of votes overflows.
        NumberOfVotesExceeded,
        /// When stake amount of a vote overflows.
        StakeAmountOfVotesExceeded,
        /// Missing current DID
        MissingCurrentIdentity,
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

        /// Change the proposal cool off period value. This is the number of blocks after which the proposer of a pip
        /// can modify or cancel their proposal, and other voting is prohibited
        ///
        /// # Arguments
        /// * `duration` proposal cool off period duration in blocks
        #[weight = (150_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_proposal_cool_off_period(origin, duration: T::BlockNumber) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::deposit_event(RawEvent::ProposalCoolOffPeriodChanged(SystematicIssuers::Committee.as_id(), Self::proposal_cool_off_period(), duration));
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

        /// Change the maximum number of active PIPs before community members cannot propose anything.
        #[weight = (100_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_active_pip_limit(origin, new_max: u32) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            let prev_max = ActivePipLimit::get();
            ActivePipLimit::put(prev_max);
            Self::deposit_event(RawEvent::ActivePipLimitChanged(SystematicIssuers::Committee.as_id(), prev_max, new_max));
        }

        /// A network member creates a PIP by submitting a dispatchable which
        /// changes the network in someway. A minimum deposit is required to open a new proposal.
        ///
        /// # Arguments
        /// * `proposer` is either a signing key or committee.
        ///    Used to understand whether this is a committee proposal and verified against `origin`.
        /// * `proposal` a dispatchable call
        /// * `deposit` minimum deposit value, which is ignored if `proposer` is a committee.
        /// * `url` a link to a website for proposal discussion
        #[weight = (1_850_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn propose(
            origin,
            proposer: Proposer<T::AccountId>,
            proposal: Box<T::Proposal>,
            deposit: BalanceOf<T>,
            url: Option<Url>,
            description: Option<PipDescription>,
            beneficiaries: Option<Vec<Beneficiary<T::Balance>>>
        ) -> DispatchResult {
            // 1. Ensure it's really the `proposer`.
            Self::ensure_signed_by(origin, &proposer)?;

            // 2. Add a deposit for community PIPs.
            if let Proposer::Community(ref proposer) = proposer {
                // ...but first make sure active PIP limit isn't crossed.
                // This doesn't apply to committee PIPs.
                // `0` is special and denotes no limit.
                let limit = ActivePipLimit::get();
                ensure!(limit == 0 || ActivePipCount::get() < limit, Error::<T>::TooManyActivePips);

                // Pre conditions: caller must have min balance.
                ensure!(deposit >= Self::min_proposal_deposit(), Error::<T>::IncorrectDeposit);

                // Reserve the minimum deposit.
                <T as Trait>::Currency::reserve(&proposer, deposit)
                    .map_err(|_| Error::<T>::InsufficientDeposit)?;
            }

            // 3. Charge protocol fees, even for committee PIPs.
            <T as IdentityTrait>::ProtocolFee::charge_fee(ProtocolOp::PipsPropose)?;

            // 4. Construct and add PIP to storage.
            let id = Self::next_pip_id();
            ActivePipCount::mutate(|count| *count += 1);
            let cool_off_until = <system::Module<T>>::block_number() + Self::proposal_cool_off_period();
            let proposal_metadata = PipsMetadata {
                proposer: proposer.clone(),
                id,
                url: url.clone(),
                description: description.clone(),
                cool_off_until: cool_off_until,
            };
            <ProposalMetadata<T>>::insert(id, proposal_metadata);

            let proposal_data = Self::reportable_proposal_data(&*proposal);
            let pip = Pip {
                id,
                proposal: *proposal,
                state: ProposalState::Pending,
                beneficiaries,
            };
            <Proposals<T>>::insert(id, pip);

            // 5. Record the deposit and as a signal if we have a community PIP.
            if let Proposer::Community(ref proposer) = proposer {
                let deposit_info = DepositInfo {
                    owner: proposer.clone(),
                    amount: deposit
                };
                <Deposits<T>>::insert(id, &proposer, deposit_info);

                // Add vote and update voting counter.
                // INTERNAL: It is impossible to overflow counters in the first vote.
                Self::unsafe_vote(id, proposer.clone(), Vote(true, deposit))
                    .map_err(|vote_error| {
                        debug::error!("The counters of voting (id={}) have an overflow during the 1st vote", id);
                        vote_error
                    })?;
            }

            // 6. Emit the event.
            Self::deposit_event(RawEvent::ProposalCreated(
                Self::current_did_or_missing()?,
                proposer,
                id,
                deposit,
                url,
                description,
                cool_off_until,
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
            description: Option<PipDescription>,
        ) -> DispatchResult {
            // 1. Fetch proposer and perform sanity checks.
            let proposer = Self::ensure_owned_by_alterable(origin, id)?;

            // 2. Update proposal metadata.
            <ProposalMetadata<T>>::mutate(id, |meta| {
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
            let new_state = Self::update_proposal_state(id, ProposalState::Cancelled);
            Self::prune_data(id, new_state, Self::prune_historical_pips());

            Ok(())
        }

        /// Id bonds an additional deposit to proposal with id `id`.
        /// That amount is added to the current deposit.
        ///
        /// # Errors
        /// * `BadOrigin` if other than the PIP's owner, or if owner is a committee.
        /// * `ProposalIsImmutable` if proposal is out of its cool-off period.
        #[weight = 900_000_000]
        pub fn bond_additional_deposit(origin,
            id: PipId,
            additional_deposit: BalanceOf<T>
        ) -> DispatchResult {
            // 1. Sanity checks.
            let proposer = Self::ensure_owned_by_community_alterable(origin, id)?;

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
            <ProposalVotes<T>>::insert(id, &proposer, Vote(true, curr_deposit + max_additional_deposit));

            Self::emit_proposal_bond_adjusted(proposer, id, true, max_additional_deposit)
        }

        /// It unbonds any amount from the deposit of the proposal with id `id`.
        ///
        /// # Errors
        /// * `BadOrigin` if other than the PIP's owner, or if owner is a committee.
        /// * `ProposalIsImmutable` if proposal is out of its cool-off period.
        /// * `InsufficientDeposit` if the final deposit < minimum deposit for a proposal.
        #[weight = 900_000_000]
        pub fn unbond_deposit(origin,
            id: PipId,
            amount: BalanceOf<T>
        ) -> DispatchResult {
            // 1. Sanity checks.
            let proposer = Self::ensure_owned_by_community_alterable(origin, id)?;

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
            <ProposalVotes<T>>::insert(id, &proposer, Vote(true, new_deposit));

            Self::emit_proposal_bond_adjusted(proposer, id, false, amount)
        }

        /// A network member can vote on any community PIP by selecting the id that
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
            ensure!(matches!(meta.proposer, Proposer::Community(_)), Error::<T>::NotFromCommunity);

            // No one should be able to vote during the proposal cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!(meta.cool_off_until <= curr_block_number, Error::<T>::ProposalOnCoolOffPeriod);

            // Check that the proposal is pending
            Self::is_proposal_state(id, ProposalState::Pending)?;

            // Valid PipId
            ensure!(<ProposalResult<T>>::contains_key(id), Error::<T>::NoSuchProposal);

            // Double-check vote duplication.
            ensure!(matches!(Self::proposal_vote(id, &proposer), None), Error::<T>::DuplicateVote);

            // Reserve the deposit
            <T as Trait>::Currency::reserve(&proposer, deposit).map_err(|_| Error::<T>::InsufficientDeposit)?;

            // Save your vote.
            Self::unsafe_vote(id, proposer.clone(), Vote(aye_or_nay, deposit))
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

        /// Approves the pending non-cooling committee PIP given by the `id`.
        ///
        /// # Errors
        /// * `BadOrigin` unless a GC voting majority executes this function.
        /// * `NoSuchProposal` if the PIP with `id` doesn't exist.
        /// * `IncorrectProposalState` if the proposal isn't pending.
        /// * `ProposalOnCoolOffPeriod` if the proposal is cooling off.
        /// * `NotByCommittee` if the proposal isn't by a committee.
        #[weight = (100_000, DispatchClass::Operational, Pays::Yes)]
        pub fn approve_committee_proposal(origin, id: PipId) {
            // 1. Only GC can do this.
            T::VotingMajorityOrigin::ensure_origin(origin)?;

            // 2. Proposal must be pending.
            Self::is_proposal_state(id, ProposalState::Pending)?;

            // 3. Proposal must not be cooling-off and must be by committee.
            let meta = Self::proposal_metadata(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!(meta.cool_off_until <= curr_block_number, Error::<T>::ProposalOnCoolOffPeriod);
            ensure!(matches!(meta.proposer, Proposer::Committee(_)), Error::<T>::NotByCommittee);

            // 4. All is good, schedule PIP for execution.
            Self::maybe_unsnapshot_pip(id, ProposalState::Pending);
            let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
            Self::schedule_pip_for_execution(current_did, id);
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
            Self::maybe_unsnapshot_pip(id, proposal.state);
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
            Self::maybe_unsnapshot_pip(id, proposal.state);
            Self::prune_data(id, proposal.state, true);
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
        pub fn reschedule_execution(origin, id: PipId, until: Option<T::BlockNumber>) -> DispatchResult {
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
                // Only keep community PIPs not cooling-off.
                .filter(|id| {
                    <ProposalMetadata<T>>::get(id)
                        .filter(|meta| meta.cool_off_until <= created_at)
                        .filter(|meta| matches!(meta.proposer, Proposer::Community(_)))
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
        pub fn enact_snapshot_results(origin, results: Vec<(u8, SnapshotResult)>) -> DispatchResult {
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
                        (None, _) => return Err(Error::<T>::SnapshotResultTooLarge.into()),
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
                for (pip_id, new_count) in to_bump_skipped.iter().copied() {
                    PipSkipCount::insert(pip_id, new_count);
                    Self::deposit_event(RawEvent::SkippedPip(pip_id, new_count));
                }

                // Reject proposals as instructed & refund.
                for pip_id in to_reject.iter().copied() {
                    Self::unsafe_reject_proposal(pip_id);
                }

                // Approve proposals as instructed.
                let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
                for pip_id in to_approve.iter().copied() {
                    Self::schedule_pip_for_execution(current_did, pip_id);
                }

                Self::deposit_event(RawEvent::SnapshotResultsEnacted(to_bump_skipped, to_reject, to_approve));

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
    /// Ensure that `origin` represents a signed extrinsic (i.e. transaction)
    /// and confirms that the account is the same as the given `proposer`.
    ///
    /// For example, if `proposer` denotes a committee,
    /// then `origin` is checked against the committee's origin.
    ///
    /// # Errors
    /// * `BadOrigin` unless the checks above pass.
    fn ensure_signed_by(origin: T::Origin, proposer: &Proposer<T::AccountId>) -> DispatchResult {
        match proposer {
            Proposer::Community(acc) => {
                ensure!(acc == &ensure_signed(origin)?, Error::<T>::BadOrigin)
            }
            Proposer::Committee(Committee::Technical) => {
                T::TechnicalCommitteeVMO::ensure_origin(origin)?;
            }
            Proposer::Committee(Committee::Upgrade) => {
                T::UpgradeCommitteeVMO::ensure_origin(origin)?;
            }
        }
        Ok(())
    }

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

    fn ensure_owned_by_community_alterable(
        origin: T::Origin,
        id: PipId,
    ) -> Result<T::AccountId, DispatchError> {
        match Self::ensure_owned_by_alterable(origin, id)? {
            Proposer::Community(p) => Ok(p),
            Proposer::Committee(_) => Err(Error::<T>::BadOrigin.into()),
        }
    }

    /// Ensure that proposer is owner of the proposal which mustn't be in the cool off period.
    ///
    /// # Errors
    /// * `ProposalIsImmutable`: A Proposal is mutable only during its cool off period.
    /// * `BadOrigin`: Only the owner of the proposal can mutate it.
    fn ensure_owned_by_alterable(
        origin: T::Origin,
        id: PipId,
    ) -> Result<Proposer<T::AccountId>, DispatchError> {
        // 1. Only owner can act on proposal.
        let meta = Self::proposal_metadata(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
        Self::ensure_signed_by(origin, &meta.proposer)?;

        // 2. Check that the proposal is pending.
        Self::is_proposal_state(id, ProposalState::Pending)?;

        // 3. Proposal is *ONLY* alterable during its cool-off period.
        let curr_block_number = <system::Module<T>>::block_number();
        ensure!(
            meta.cool_off_until > curr_block_number,
            Error::<T>::ProposalIsImmutable
        );

        Ok(meta.proposer)
    }

    /// Runs the following procedure:
    /// 1. Executes all PIPs scheduled for this block.
    pub fn end_block(block_number: T::BlockNumber) -> Result<Weight, DispatchError> {
        // Some arbitrary number right now, It is subject to change after proper benchmarking
        let mut weight: Weight = 50_000_000;
        // 1. Execute all PIPs scheduled for this block.
        <ExecutionSchedule<T>>::take(block_number)
            .into_iter()
            .for_each(|id| {
                <PipToSchedule<T>>::remove(id);
                weight += Self::execute_proposal(id);
            });
        <ExecutionSchedule<T>>::remove(block_number);
        Ok(weight)
    }

    /// Rejects the given `id`, refunding the deposit, and possibly pruning the proposal's data.
    fn unsafe_reject_proposal(id: PipId) {
        let new_state = Self::update_proposal_state(id, ProposalState::Rejected);
        Self::refund_proposal(id);
        Self::prune_data(id, new_state, Self::prune_historical_pips());
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

    /// Remove the PIP with `id` from the snapshot if it is there.
    fn maybe_unsnapshot_pip(id: PipId, state: ProposalState) {
        if let ProposalState::Pending = state {
            let cool_until = <ProposalMetadata<T>>::get(id).unwrap().cool_off_until;
            if cool_until <= <system::Module<T>>::block_number()
                && <SnapshotMeta<T>>::get()
                    .filter(|m| cool_until <= m.created_at)
                    .is_some()
            {
                // Proposal is pending, no longer in cool-down, and wasn't when snapshot was made.
                // Hence, it is in the snapshot and filtering it out will have an effect.
                // Note: These checks are not strictly necessary, but are done to avoid work.
                <SnapshotQueue<T>>::mutate(|queue| queue.retain(|i| i.id != id));
            }
        }
    }

    /// Prunes (nearly) all data associated with a proposal, removing it from storage.
    ///
    /// For efficiency, some data (e.g., re. execution schedules) is not removed in this function,
    /// but is removed in functions executing this one.
    fn prune_data(id: PipId, state: ProposalState, prune: bool) {
        let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
        Self::decrement_count_if_active(state);
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
                let new_state = match Self::check_beneficiaries(id) {
                    Ok(_) => match proposal.proposal.dispatch(system::RawOrigin::Root.into()) {
                        Ok(post_info) => {
                            actual_weight = post_info.actual_weight.unwrap_or(0);
                            Self::pay_to_beneficiaries(id);
                            Self::update_proposal_state(id, ProposalState::Executed)
                        }
                        Err(e) => {
                            debug::error!("Proposal {}, its execution fails: {:?}", id, e.error);
                            Self::update_proposal_state(id, ProposalState::Failed)
                        }
                    },
                    Err(e) => {
                        debug::error!("Proposal {}, its beneficiaries fails: {:?}", id, e);
                        Self::update_proposal_state(id, ProposalState::Failed)
                    }
                };
                Self::prune_data(id, new_state, Self::prune_historical_pips());
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

    fn update_proposal_state(id: PipId, new_state: ProposalState) -> ProposalState {
        <Proposals<T>>::mutate(id, |proposal| {
            if let Some(ref mut proposal) = proposal {
                Self::decrement_count_if_active(proposal.state);
                proposal.state = new_state;
            }
        });
        let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
        Self::deposit_event(RawEvent::ProposalStateUpdated(current_did, id, new_state));
        new_state
    }

    fn is_proposal_state(id: PipId, state: ProposalState) -> DispatchResult {
        let proposal = Self::proposals(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
        ensure!(proposal.state == state, Error::<T>::IncorrectProposalState);
        Ok(())
    }

    /// Decrement active proposal count if `state` signifies it is active.
    fn decrement_count_if_active(state: ProposalState) {
        if let ProposalState::Pending | ProposalState::Scheduled = state {
            ActivePipCount::mutate(|count| *count -= 1);
        }
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

    /// Retrieve proposals made by `proposer`.
    pub fn proposed_by(proposer: Proposer<T::AccountId>) -> Vec<PipId> {
        <ProposalMetadata<T>>::iter()
            .filter(|(_, meta)| meta.proposer == proposer)
            .map(|(_, meta)| meta.id)
            .collect()
    }

    /// Retrieve proposals `address` voted on
    pub fn voted_on(address: T::AccountId) -> Vec<PipId> {
        <ProposalMetadata<T>>::iter()
            .filter_map(|(_, meta)| Self::proposal_vote(meta.id, &address).map(|_| meta.id))
            .collect::<Vec<_>>()
    }

    /// Retrieve historical voting of `who` account.
    pub fn voting_history_by_address(
        who: T::AccountId,
    ) -> HistoricalVotingByAddress<Vote<BalanceOf<T>>> {
        <ProposalMetadata<T>>::iter()
            .filter_map(|(_, meta)| {
                Some(VoteByPip {
                    pip: meta.id,
                    vote: Self::proposal_vote(meta.id, &who)?,
                })
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

    /// Returns the id to use for the next PIP to be made.
    /// Invariant: `next_pip_id() == next_pip_id() + 1`.
    fn next_pip_id() -> u32 {
        <PipIdSequence>::mutate(|id| mem::replace(id, *id + 1))
    }

    /// It inserts the vote and updates the accountability of target proposal.
    fn unsafe_vote(id: PipId, proposer: T::AccountId, vote: Vote<BalanceOf<T>>) -> DispatchResult {
        let mut stats = Self::proposal_result(id);
        let Vote(direction, deposit) = vote;
        let (count, stake) = match direction {
            true => (&mut stats.ayes_count, &mut stats.ayes_stake),
            false => (&mut stats.nays_count, &mut stats.nays_stake),
        };
        *count = count
            .checked_add(1)
            .ok_or_else(|| Error::<T>::NumberOfVotesExceeded)?;
        *stake = stake
            .checked_add(&deposit)
            .ok_or_else(|| Error::<T>::StakeAmountOfVotesExceeded)?;

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
