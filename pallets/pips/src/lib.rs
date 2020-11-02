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
//! Polymesh Improvement Proposals (PIPs) are dispatchables that can be `propose`d for execution.
//! These PIPs can either be proposed by a committee, or they can be proposed by a community member,
//! in which case they can `vote`d on by all POLYX token holders.
//! Once created, a proposal first enters a cool-off period, during which it can be amended
//! (via `amend_proposal` and `vote`) or cancelled (via `cancel_proposal`) but not approved.
//! During cool-off, only the PIPs proposer can use `vote`.
//!
//! Voting, or rather "signalling", which currently scales linearly with POLX,
//! in this system is used to direct the Governance Councils (GCs)
//! attention by moving proposals up and down a review queue, specific to community proposals.
//!
//! From time to time, the GC will take a `snapshot` of this queue,
//! meet and review PIPs, and reject, approve, or skip the proposal (via `enact_snapshot_results`).
//! Any approved PIPs from this snapshot will then be scheduled,
//! in order of signal value, to be executed automatically on the blockchain.
//! However, using `reschedule_proposal`, a special Release Coordinator (RC), a member of the GC,
//! can reschedule approved PIPs at will, except for a PIP to replace the RC.
//! Once no longer relevant, the snapshot can be cleared by the GC through `clear_snapshot`.
//!
//! As aforementioned, the GC can skip a PIP, which will increments its "skipped count".
//! Should a configurable limit for the skipped count be exceeded, a PIP can no longer be skipped.
//!
//! Committee proposals, as noted before, do not enter the snapshot or receive votes.
//! However, the GC can at any moment approve such a PIP via `approve_committee_proposal`.
//!
//! Should the GC want to reject an active (scheduled or pending) proposal,
//! they can do so at any time using `reject_proposal`.
//! For garbage collection purposes, it is also possible to use `prune_proposal`,
//! which will, without any restrictions on its state, remove the PIP's storage.
//!
//!
//! ## Overview
//!
//! The Pips module provides functions for:
//!
//! - Proposing and amending PIPs
//! - Signalling (voting) on them for adjusting priority in the review queue
//! - Taking and clearing snapshots of the queue
//! - Approving, rejecting, skipping, and rescheduling PIPs
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### Configuration changes
//!
//! - `set_prune_historical_pips` change whether historical PIPs are pruned
//! - `set_min_proposal_deposit` change min deposit to create a proposal
//! - `set_proposal_cool_off_period` change duration in blocks for which a proposal can be amended
//! - `set_default_enactment_period` change the period after enactment after which the proposal is executed
//! - `set_max_pip_skip_count` change the maximum times a PIP can be skipped
//! - `set_active_pip_limit` change the maximum number of concurrently active PIPs
//!
//! #### Other
//!
//! - `propose` - token holders can propose a new PIP.
//! - `amend_proposal` - allows the creator of a proposal to amend the proposal details
//! - `cancel_proposal` - allows the creator of a proposal to cancel the proposal
//! - `vote` - token holders, including the PIP's proposer, can vote on a PIP.
//! - `approve_committee_proposal` - allows the GC to approve a committee proposal
//! - `reject_proposal` - reject an active proposal and refund deposits
//! - `prune_proposal` - prune all storage associated with proposal and refund deposits
//! - `reschedule_execution` - release coordinator can reschedule a PIPs execution
//! - `clear_snapshot` - clears the snapshot
//! - `snapshot` - takes a new snapshot of the review queue
//! - `enact_snapshot_results` - enters results (approve, reject, and skip) for PIPs in snapshot
//!
//! ### Public Functions
//!
//! - `end_block` - executes scheduled proposals
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::mem;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    storage::IterableStorageMap,
    traits::{Currency, EnsureOrigin, Get, LockIdentifier, WithdrawReasons},
    weights::{DispatchClass, Pays, Weight},
};
use frame_system::{self as system, ensure_signed};
use pallet_identity as identity;
use pallet_treasury::TreasuryTrait;
use polymesh_common_utilities::{
    constants::PIP_MAX_REPORTING_SIZE,
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    traits::{
        balances::LockableCurrencyExt, governance_group::GovernanceGroupTrait, group::GroupTrait,
        pip::PipId,
    },
    with_transaction, CommonTrait, Context, MaybeBlock, GC_DID,
};
use polymesh_primitives::IdentityId;
use polymesh_primitives_derive::VecU8StrongTyped;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::traits::{
    BlakeTwo256, CheckedAdd, CheckedSub, Dispatchable, Hash, Saturating, Zero,
};
use sp_std::{convert::From, prelude::*};
use sp_version::RuntimeVersion;

const PIPS_LOCK_ID: LockIdentifier = *b"pips    ";

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
pub struct Pip<T: Trait> {
    /// The proposal's unique id.
    pub id: PipId,
    /// The proposal being voted on.
    pub proposal: T::Proposal,
    /// The latest state
    pub state: ProposalState,
    /// The issuer of `propose`.
    pub proposer: Proposer<T::AccountId>,
    /// The block until which the PIP is cooling off.
    /// During the period, the `proposer` can amend details.
    /// After the period, people can vote on community PIPs.
    pub cool_off_until: T::BlockNumber,
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
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum Committee {
    /// The technical committee.
    Technical,
    /// The upgrade committee tends to propose chain upgrades.
    Upgrade,
}

/// The proposer of a certain PIP.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
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
    /// The proposal's unique id.
    pub id: PipId,
    /// The proposal url for proposal discussion.
    pub url: Option<Url>,
    /// The proposal description.
    pub description: Option<PipDescription>,
    /// The block when the PIP was made.
    pub created_at: T::BlockNumber,
    /// Assuming the runtime has a given `rv: RuntimeVersion` at the point of `Pips::propose`,
    /// then this field contains `rv.transaction_version`.
    ///
    /// Currently, this is only used for off-chain purposes to highlight any differences
    /// in the proposal's transaction version from the current one.
    pub transaction_version: u32,
    /// The point, if any, at which this PIP, if still in a `Pending` state,
    /// is expired, and thus no longer valid.
    ///
    /// This field has no operational on-chain effect and is provided for UI purposes only.
    /// On-chain effects are instead handled via scheduling.
    pub expiry: MaybeBlock<T::BlockNumber>,
}

/// For keeping track of proposal being voted on.
#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VotingResult<Balance> {
    /// The current set of voters that approved with their stake.
    pub ayes_count: u32,
    pub ayes_stake: Balance,
    /// The current set of voters that rejected with their stake.
    pub nays_count: u32,
    pub nays_stake: Balance,
}

/// A "vote" or "signal" on a PIP to move it up or down the review queue.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode)]
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
    /// Proposal has expired. Only previously `Pending` PIPs may end up here.
    Expired,
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

/// ID of the taken snapshot in a sequence.
pub type SnapshotId = u32;

/// A snapshot's metadata, containing when it was created and who triggered it.
/// The priority queue is stored separately (see `SnapshottedPip`).
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SnapshotMetadata<T: Trait> {
    /// The block when the snapshot was made.
    pub created_at: T::BlockNumber,
    /// Who triggered this snapshot? Should refer to someone in the GC.
    pub made_by: T::AccountId,
    /// Unique ID of this snapshot.
    pub id: SnapshotId,
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
// This type is only here due to `enact_snapshot_results`.
#[derive(codec::Encode, codec::Decode, Copy, Clone, PartialEq, Eq, Debug)]
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
    // TODO(centril): Remove `ReservableCurrency` bound once `on_runtime_upgrade` is nixed.
    type Currency: LockableCurrencyExt<Self::AccountId, Moment = Self::BlockNumber>
        + frame_support::traits::ReservableCurrency<Self::AccountId>;

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

        /// How many blocks will it take, after a `Pending` PIP's cooling-off period is over,
        /// until the the PIP expires, assuming it has not transitioned to another `ProposalState`?
        pub PendingPipExpiry get(fn pending_pip_expiry) config(): MaybeBlock<T::BlockNumber>;

        /// Maximum times a PIP can be skipped before triggering `CannotSkipPip` in `enact_snapshot_results`.
        pub MaxPipSkipCount get(fn max_pip_skip_count) config(): SkippedCount;

        /// The maximum allowed number for `ActivePipCount`.
        /// Once reached, new PIPs cannot be proposed by community members.
        pub ActivePipLimit get(fn active_pip_limit) config(): u32;

        /// Proposals so far. id can be used to keep track of PIPs off-chain.
        PipIdSequence get(fn pip_id_sequence): u32;

        /// Snapshots so far. id can be used to keep track of snapshots off-chain.
        SnapshotIdSequence get(fn snapshot_id_sequence): u32;

        /// Total count of current pending or scheduled PIPs.
        ActivePipCount get(fn active_pip_count): u32;

        /// The metadata of the active proposals.
        pub ProposalMetadata get(fn proposal_metadata): map hasher(twox_64_concat) PipId => Option<PipsMetadata<T>>;

        /// Those who have locked a deposit.
        /// proposal (id, proposer) -> deposit
        pub Deposits get(fn deposits): double_map hasher(twox_64_concat) PipId, hasher(twox_64_concat) T::AccountId => DepositInfo<T::AccountId, BalanceOf<T>>;

        /// Actual proposal for a given id, if it's current.
        /// proposal id -> proposal
        pub Proposals get(fn proposals): map hasher(twox_64_concat) PipId => Option<Pip<T>>;

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

        /// Maps block numbers to list of PIPs which should be expired at the block number.
        /// block number -> Pip id
        pub ExpirySchedule get(fn expiry_schedule): map hasher(twox_64_concat) T::BlockNumber => Vec<PipId>;

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

        /// All existing PIPs where the proposer is a committee.
        /// This list is a cache of all ids in `Proposals` with `Proposer::Committee(_)`.
        pub CommitteePips get(fn committee_pips): Vec<PipId>;
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
        /// Caller DID, Proposer, PIP ID, deposit, URL, description, cool-off period end, expiry time, proposal data.
        ProposalCreated(
            IdentityId,
            Proposer<AccountId>,
            PipId,
            Balance,
            Option<Url>,
            Option<PipDescription>,
            BlockNumber,
            MaybeBlock<BlockNumber>,
            ProposalData,
        ),
        /// A PIP's details (url & description) were amended.
        ProposalDetailsAmended(IdentityId, Proposer<AccountId>, PipId, Option<Url>, Option<PipDescription>),
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
        /// Amount of blocks, after the cool-off period, after which a pending PIP expires.
        /// (caller DID, old expiry, new expiry)
        PendingPipExpiryChanged(IdentityId, MaybeBlock<BlockNumber>, MaybeBlock<BlockNumber>),
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
        SnapshotCleared(IdentityId, SnapshotId),
        /// A new snapshot was taken.
        SnapshotTaken(IdentityId, SnapshotId),
        /// A PIP in the snapshot queue was skipped.
        /// (gc_did, pip_id, new_skip_count)
        PipSkipped(IdentityId, PipId, SkippedCount),
        /// Results (e.g., approved, rejected, and skipped), were enacted for some PIPs.
        /// (gc_did, snapshot_id_opt, skipped_pips_with_new_count, rejected_pips, approved_pips)
        SnapshotResultsEnacted(IdentityId, Option<SnapshotId>, Vec<(PipId, SkippedCount)>, Vec<PipId>, Vec<PipId>),
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
        /// When enacting snapshot results, an unskippable PIP was skipped.
        CannotSkipPip,
        /// Tried to enact results for the snapshot queue overflowing its length.
        SnapshotResultTooLarge,
        /// Tried to enact result for PIP with id different from that at the position in the queue.
        SnapshotIdMismatch
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            // Larger goal here is to clear Governance V1.
            use frame_support::{
                storage::{IterableStorageDoubleMap, migration::StorageIterator},
                traits::ReservableCurrency,
            };
            use polymesh_primitives::migrate::kill_item;

            // 1. Start with refunding all deposits.
            // As we've `drain`ed  `Deposits`, we need not do so again below.
            for (_, _, depo) in <Deposits<T>>::drain() {
                <T as Trait>::Currency::unreserve(&depo.owner, depo.amount);
            }

            // 2. Then we clear various storage items that were present on V1.
            // For future reference, the storage items are defined in:
            // https://github.com/PolymathNetwork/Polymesh/blob/0047b2570e7ac57771b4153d25867166e8091b9a/pallets/pips/src/lib.rs#L308-L357

            // 2a) Clear all the `map`s and `double_map`s by fully consuming a draining iterator.
            for item in &[
                "ProposalMetadata",
                "ProposalsMaturingAt",
                "Proposals",
                "ProposalResult",
                "Referendums",
                "ScheduledReferendumsAt",
                "ProposalVotes",
            ] {
                StorageIterator::<()>::new(b"Pips", item.as_bytes()).drain().for_each(drop)
            }

            // 2b) Reset the PIP ID sequence to `0`.
            PipIdSequence::kill();

            // 2c) Remove items no longer used in V2.
            for item in &["ProposalDuration", "QuorumThreshold"] {
                kill_item(b"Pips", item.as_bytes());
            }

            // Done; we've cleared all V1 storage needed; V2 can now be filled in.
            // As for the weight, clearing costs much more than this, but let's pretend.
            0
        }

        /// Change whether completed PIPs are pruned. Can only be called by governance council
        ///
        /// # Arguments
        /// * `deposit` the new min deposit required to start a proposal
        #[weight = (550_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_prune_historical_pips(origin, new_value: bool) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::deposit_event(RawEvent::HistoricalPipsPruned(GC_DID, Self::prune_historical_pips(), new_value));
            <PruneHistoricalPips>::put(new_value);
        }

        /// Change the minimum proposal deposit amount required to start a proposal. Only Governance
        /// committee is allowed to change this value.
        ///
        /// # Arguments
        /// * `deposit` the new min deposit required to start a proposal
        #[weight = (550_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_min_proposal_deposit(origin, deposit: BalanceOf<T>) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::deposit_event(RawEvent::MinimumProposalDepositChanged(GC_DID, Self::min_proposal_deposit(), deposit));
            <MinimumProposalDeposit<T>>::put(deposit);
        }

        /// Change the proposal cool off period value. This is the number of blocks after which the proposer of a pip
        /// can modify or cancel their proposal, and other voting is prohibited
        ///
        /// # Arguments
        /// * `duration` proposal cool off period duration in blocks
        #[weight = (550_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_proposal_cool_off_period(origin, duration: T::BlockNumber) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            Self::deposit_event(RawEvent::ProposalCoolOffPeriodChanged(GC_DID, Self::proposal_cool_off_period(), duration));
            <ProposalCoolOffPeriod<T>>::put(duration);
        }

        /// Change the default enact period.
        #[weight = (550_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_default_enactment_period(origin, duration: T::BlockNumber) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            let prev = <DefaultEnactmentPeriod<T>>::get();
            <DefaultEnactmentPeriod<T>>::put(duration);
            Self::deposit_event(RawEvent::DefaultEnactmentPeriodChanged(GC_DID, prev, duration));
        }

        /// Change the amount of blocks, after the cool-off, for which a pending PIP is expired.
        /// If `expiry` is `None` then PIPs never expire.
        #[weight = (550_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_pending_pip_expiry(origin, expiry: MaybeBlock<T::BlockNumber>) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            let prev = <PendingPipExpiry<T>>::get();
            <PendingPipExpiry<T>>::put(expiry);
            Self::deposit_event(RawEvent::PendingPipExpiryChanged(GC_DID, prev, expiry));
        }

        /// Change the maximum skip count (`max_pip_skip_count`).
        /// New values only
        #[weight = (150_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_max_pip_skip_count(origin, new_max: SkippedCount) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            let prev_max = MaxPipSkipCount::get();
            MaxPipSkipCount::put(new_max);
            Self::deposit_event(RawEvent::MaxPipSkipCountChanged(GC_DID, prev_max, new_max));
        }

        /// Change the maximum number of active PIPs before community members cannot propose anything.
        #[weight = (150_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn set_active_pip_limit(origin, new_max: u32) {
            T::CommitteeOrigin::ensure_origin(origin)?;
            let prev_max = ActivePipLimit::get();
            ActivePipLimit::put(new_max);
            Self::deposit_event(RawEvent::ActivePipLimitChanged(GC_DID, prev_max, new_max));
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
            proposal: Box<T::Proposal>,
            deposit: BalanceOf<T>,
            url: Option<Url>,
            description: Option<PipDescription>,
        ) -> DispatchResult {
            // 1. Infer the proposer from `origin`.
            let proposer = Self::ensure_infer_proposer(origin)?;

            let did = Self::current_did_or_missing()?;

            // 2. Add a deposit for community PIPs.
            if let Proposer::Community(ref proposer) = proposer {
                // ...but first make sure active PIP limit isn't crossed.
                // This doesn't apply to committee PIPs.
                // `0` is special and denotes no limit.
                let limit = ActivePipLimit::get();
                ensure!(limit == 0 || ActivePipCount::get() < limit, Error::<T>::TooManyActivePips);

                // Pre conditions: caller must have min balance.
                ensure!(deposit >= Self::min_proposal_deposit(), Error::<T>::IncorrectDeposit);

                // Lock the deposit.
                Self::increase_lock(proposer, deposit)?;
            } else {
                // Committee PIPs cannot have a deposit.
                ensure!(deposit.is_zero(), Error::<T>::NotFromCommunity);
            }

            // 3. Charge protocol fees, even for committee PIPs.
            <T as IdentityTrait>::ProtocolFee::charge_fee(ProtocolOp::PipsPropose)?;

            // 4. Construct and add PIP to storage.
            let id = Self::next_pip_id();
            let created_at = <system::Module<T>>::block_number();
            let cool_off_until = created_at + Self::proposal_cool_off_period();
            let expiry = Self::pending_pip_expiry() + cool_off_until;
            let transaction_version = <T::Version as Get<RuntimeVersion>>::get().transaction_version;
            let proposal_data = Self::reportable_proposal_data(&*proposal);
            <ProposalMetadata<T>>::insert(id, PipsMetadata {
                id,
                created_at,
                url: url.clone(),
                description: description.clone(),
                transaction_version,
                expiry,
            });
            <Proposals<T>>::insert(id, Pip {
                id,
                proposal: *proposal,
                state: ProposalState::Pending,
                proposer: proposer.clone(),
                cool_off_until,
            });
            ActivePipCount::mutate(|count| *count += 1);

            // 5. Schedule for expiry as long as `Pending` at block with number `expiring_at`.
            if let MaybeBlock::Some(expiring_at) = expiry {
                <ExpirySchedule<T>>::append(expiring_at, id);
            }

            // 6. Record the deposit and as a signal if we have a community PIP.
            if let Proposer::Community(ref proposer) = proposer {
                <Deposits<T>>::insert(id, &proposer, DepositInfo {
                    owner: proposer.clone(),
                    amount: deposit
                });

                // Add vote and update voting counter.
                // INTERNAL: It is impossible to overflow counters in the first vote.
                Self::unsafe_vote(id, proposer.clone(), Vote(true, deposit))
                    .map_err(|vote_error| {
                        debug::error!("The counters of voting (id={}) have an overflow during the 1st vote", id);
                        vote_error
                    })?;
            } else {
                CommitteePips::append(id);
            }

            // 7. Emit the event.
            Self::deposit_event(RawEvent::ProposalCreated(
                did,
                proposer,
                id,
                deposit,
                url,
                description,
                cool_off_until,
                expiry,
                proposal_data,
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
            let current_did = Self::current_did_or_missing()?;

            // 2. Update proposal metadata.
            <ProposalMetadata<T>>::mutate(id, |meta| {
                if let Some(meta) = meta {
                    meta.url = url.clone();
                    meta.description = description.clone();
                }
            });

            // 3. Emit event.
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

            // 2. Close that proposal (including refunding).
            let did = Context::current_identity::<Identity<T>>().unwrap_or_default();
            Self::maybe_prune(did, id, ProposalState::Cancelled);
            Ok(())
        }

        /// Vote either in favor (`aye_or_nay` == true) or against a PIP with `id`.
        /// The "convinction" or strength of the vote is given by `deposit`, which is reserved.
        ///
        /// Note that `vote` is *not* additive.
        /// That is, `vote(id, true, 50)` followed by `vote(id, true, 40)`
        /// will first reserve `50` and then refund `50 - 10`, ending up with `40` in deposit.
        /// To add atop of existing votes, you'll need `existing_deposit + addition`.
        ///
        /// # Arguments
        /// * `id`, proposal id
        /// * `aye_or_nay`, a bool representing for or against vote
        /// * `deposit`, the "conviction" with which the vote is made.
        ///
        /// # Errors
        /// * `NoSuchProposal` if `id` doesn't reference a valid PIP.
        /// * `NotFromCommunity` if proposal was made by a committee.
        /// * `ProposalOnCoolOffPeriod` if non-owner is voting and PIP is cooling off.
        /// * `IncorrectProposalState` if PIP isn't pending.
        /// * `InsufficientDeposit` if `origin` cannot reserve `deposit - old_deposit`.
        #[weight = 1_000_000_000]
        pub fn vote(origin, id: PipId, aye_or_nay: bool, deposit: BalanceOf<T>) {
            let voter = ensure_signed(origin)?;
            let pip = Self::proposals(id)
                .ok_or_else(|| Error::<T>::NoSuchProposal)?;

            // 1. Proposal must be from the community.
            let proposer = match pip.proposer {
                Proposer::Committee(_) => return Err(Error::<T>::NotFromCommunity.into()),
                Proposer::Community(p) => p,
            };

            if proposer == voter {
                // 2a. Deposit must be above minimum.
                // Note that proposer can still vote against their own PIP.
                ensure!(deposit >= Self::min_proposal_deposit(), Error::<T>::IncorrectDeposit);
            } else {
                // 2b. Only proposer can vote during PIP's cool-off period.
                let curr_block_number = <system::Module<T>>::block_number();
                ensure!(pip.cool_off_until <= curr_block_number, Error::<T>::ProposalOnCoolOffPeriod);
            }

            // 3. Proposal must be pending.
            Self::is_proposal_state(id, ProposalState::Pending)?;

            let current_did = Self::current_did_or_missing()?;

            with_transaction(|| {
                // 4. Reserve the deposit, or refund if needed.
                let curr_deposit = Self::deposits(id, &voter).amount;
                if deposit < curr_deposit {
                    Self::reduce_lock(&voter, curr_deposit - deposit)?;
                } else {
                    Self::increase_lock(&voter, deposit - curr_deposit)?;
                }
                // 5. Save the vote.
                Self::unsafe_vote(id, voter.clone(), Vote(aye_or_nay, deposit))
            })?;

            <Deposits<T>>::insert(id, &voter, DepositInfo {
                owner: voter.clone(),
                amount: deposit,
            });

            // 6. Emit event.
            Self::deposit_event(RawEvent::Voted(current_did, voter, id, aye_or_nay, deposit));
        }

        /// Approves the pending non-cooling committee PIP given by the `id`.
        ///
        /// # Errors
        /// * `BadOrigin` unless a GC voting majority executes this function.
        /// * `NoSuchProposal` if the PIP with `id` doesn't exist.
        /// * `IncorrectProposalState` if the proposal isn't pending.
        /// * `ProposalOnCoolOffPeriod` if the proposal is cooling off.
        /// * `NotByCommittee` if the proposal isn't by a committee.
        #[weight = (1_000_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn approve_committee_proposal(origin, id: PipId) {
            // 1. Only GC can do this.
            T::VotingMajorityOrigin::ensure_origin(origin)?;

            // 2. Proposal must be pending.
            Self::is_proposal_state(id, ProposalState::Pending)?;

            // 3. Proposal must not be cooling-off and must be by committee.
            let pip = Self::proposals(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!(pip.cool_off_until <= curr_block_number, Error::<T>::ProposalOnCoolOffPeriod);
            ensure!(matches!(pip.proposer, Proposer::Committee(_)), Error::<T>::NotByCommittee);

            // 4. All is good, schedule PIP for execution.
            Self::schedule_pip_for_execution(GC_DID, id);
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
        pub fn reject_proposal(origin, id: PipId) {
            T::VotingMajorityOrigin::ensure_origin(origin)?;
            let proposal = Self::proposals(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
            ensure!(Self::is_active(proposal.state), Error::<T>::IncorrectProposalState);
            Self::maybe_unschedule_pip(id, proposal.state);
            Self::maybe_unsnapshot_pip(id, proposal.state);
            Self::unsafe_reject_proposal(GC_DID, id);
        }

        /// Prune the PIP given by the `id`, refunding any funds not already refunded.
        /// The PIP may not be active
        ///
        /// This function is intended for storage garbage collection purposes.
        ///
        /// # Errors
        /// * `BadOrigin` unless a GC voting majority executes this function.
        /// * `NoSuchProposal` if the PIP with `id` doesn't exist.
        /// * `IncorrectProposalState` if the proposal is active.
        #[weight = (550_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn prune_proposal(origin, id: PipId) {
            T::VotingMajorityOrigin::ensure_origin(origin)?;
            let proposal = Self::proposals(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
            ensure!(!Self::is_active(proposal.state), Error::<T>::IncorrectProposalState);
            Self::prune_data(GC_DID, id, proposal.state, true);
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
                DispatchError::BadOrigin
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
        #[weight = (1_000_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn clear_snapshot(origin) -> DispatchResult {
            // 1. Check that a GC member is executing this.
            let actor = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&actor)?;
            ensure!(T::GovernanceCommittee::is_member(&did), Error::<T>::NotACommitteeMember);

            if let Some(meta) = <SnapshotMeta<T>>::get() {
                // 2. Clear the snapshot.
                <SnapshotMeta<T>>::kill();
                <SnapshotQueue<T>>::kill();

                // 3. Emit event.
                Self::deposit_event(RawEvent::SnapshotCleared(did, meta.id));
            }

            Ok(())
        }

        /// Takes a new snapshot of the current list of active && pending PIPs.
        /// The PIPs are then sorted into a priority queue based on each PIP's weight.
        ///
        /// # Errors
        /// * `NotACommitteeMember` - triggered when a non-GC-member executes the function.
        #[weight = (1_000_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn snapshot(origin) -> DispatchResult {
            // 1. Check that a GC member is executing this.
            let made_by = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&made_by)?;
            ensure!(T::GovernanceCommittee::is_member(&did), Error::<T>::NotACommitteeMember);

            // 2. Fetch intersection of pending && non-cooling PIPs and aggregate their votes.
            let created_at = <system::Module<T>>::block_number();
            let mut queue = <Proposals<T>>::iter_values()
                // Only keep non-cooling pending community PIPs.
                .filter(|pip| matches!(pip.state, ProposalState::Pending))
                .filter(|pip| matches!(pip.proposer, Proposer::Community(_)))
                .filter(|pip| pip.cool_off_until <= created_at)
                .map(|pip| pip.id)
                // Aggregate the votes; `true` denotes a positive sign.
                .map(|id| {
                    let VotingResult { ayes_stake, nays_stake, .. } = <ProposalResult<T>>::get(id);
                    let weight = if ayes_stake >= nays_stake {
                        (true, ayes_stake - nays_stake)
                    } else {
                        (false, nays_stake - ayes_stake)
                    };
                    SnapshottedPip { id, weight }
                })
                .collect::<Vec<_>>();

            // 5. Sort pips into priority queue, with highest priority *last*.
            // Having higher prio last allows efficient tail popping, so we have a LIFO structure.
            queue.sort_unstable_by(|l, r| {
                let (l_dir, l_stake): (bool, BalanceOf<T>) = l.weight;
                let (r_dir, r_stake): (bool, BalanceOf<T>) = r.weight;
                l_dir.cmp(&r_dir) // Negative has lower prio.
                    .then_with(|| match l_dir {
                        true => l_stake.cmp(&r_stake), // Higher stake, higher prio...
                         // Unless negative stake, in which case lower abs stake, higher prio.
                        false => r_stake.cmp(&l_stake)
                    })
                    // Lower id was made first, so assigned higher prio.
                    // This also gives us sorting stability through a total order.
                    // Moreover, as `queue` should be in by-id order originally.
                    .then(r.id.cmp(&l.id))
            });

            // 4. Commit the new snapshot.
            let id = SnapshotIdSequence::mutate(|id| mem::replace(id, *id + 1));
            <SnapshotMeta<T>>::set(Some(SnapshotMetadata { created_at, made_by, id }));
            <SnapshotQueue<T>>::set(queue);

            // 5. Emit event.
            Self::deposit_event(RawEvent::SnapshotTaken(did, id));
            Ok(())
        }

        /// Enacts `results` for the PIPs in the snapshot queue.
        /// The snapshot will be available for further enactments until it is cleared.
        ///
        /// The `results` are encoded a list of `(id, result)` where `result` is applied to `id`.
        /// Note that the snapshot priority queue is encoded with the *lowest priority first*.
        /// so `results = [(id, Approve)]` will approve `SnapshotQueue[SnapshotQueue.len() - 1]`.
        ///
        /// # Errors
        /// * `BadOrigin` - unless a GC voting majority executes this function.
        /// * `CannotSkipPip` - a given PIP has already been skipped too many times.
        /// * `SnapshotResultTooLarge` - on len(results) > len(snapshot_queue).
        /// * `SnapshotIdMismatch` - if:
        ///   ```
        ///    ∃ (i ∈ 0..SnapshotQueue.len()).
        ///      results[i].0 ≠ SnapshotQueue[SnapshotQueue.len() - i].id
        ///   ```
        ///    This is protects against clearing queue while GC is voting.
        #[weight = (1_000_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn enact_snapshot_results(origin, results: Vec<(PipId, SnapshotResult)>) -> DispatchResult {
            T::VotingMajorityOrigin::ensure_origin(origin)?;

            let max_pip_skip_count = Self::max_pip_skip_count();

            <SnapshotQueue<T>>::try_mutate(|queue| {
                let mut to_bump_skipped = Vec::new();
                // Default after-first-push capacity is 4, we bump this slightly.
                // Rationale: GC are humans sitting together and reaching conensus.
                // This is time consuming, so considering 20 PIPs in total might take few hours.
                let speculative_capacity = queue.len().max(10);
                let mut to_reject = Vec::with_capacity(speculative_capacity);
                let mut to_approve = Vec::with_capacity(speculative_capacity);

                // Go over each result...
                for (id, action) in results.iter().copied() {
                    match queue.pop() { // ...and "zip" with the queue in reverse.
                        // An action is missing a corresponding PIP in the queue, bail!
                        None => return Err(Error::<T>::SnapshotResultTooLarge.into()),
                        // The id at queue position vs. results mismatches.
                        Some(p) if p.id != id => return Err(Error::<T>::SnapshotIdMismatch.into()),
                        // All is right...
                        Some(_) => {},
                    }
                    match action {
                        // Make sure the PIP can be skipped and enqueue bumping of skip.
                        SnapshotResult::Skip => {
                            let count = PipSkipCount::get(id);
                            ensure!(count < max_pip_skip_count, Error::<T>::CannotSkipPip);
                            to_bump_skipped.push((id, count + 1));
                        },
                        // Mark PIP as rejected.
                        SnapshotResult::Reject => to_reject.push(id),
                        // Approve PIP.
                        SnapshotResult::Approve => to_approve.push(id),
                    }
                }

                // Update skip counts.
                for (pip_id, new_count) in to_bump_skipped.iter().copied() {
                    PipSkipCount::insert(pip_id, new_count);
                    Self::deposit_event(RawEvent::PipSkipped(GC_DID, pip_id, new_count));
                }

                // Reject proposals as instructed & refund.
                for pip_id in to_reject.iter().copied() {
                    Self::unsafe_reject_proposal(GC_DID, pip_id);
                }

                // Approve proposals as instructed.
                for pip_id in to_approve.iter().copied() {
                    Self::schedule_pip_for_execution(GC_DID, pip_id);
                }

                let id = Self::snapshot_metadata().map(|m| m.id);
                let event = RawEvent::SnapshotResultsEnacted(GC_DID, id, to_bump_skipped, to_reject, to_approve);
                Self::deposit_event(event);

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
    /// Ensure that `origin` represents one of:
    /// - a signed extrinsic (i.e. transaction), and infer the account id, as a community proposer.
    /// - a committee, where the committee is also inferred.
    ///
    /// Returns the inferred proposer.
    ///
    /// # Errors
    /// * `BadOrigin` if not a signed extrinsic.
    fn ensure_infer_proposer(
        origin: T::Origin,
    ) -> Result<Proposer<T::AccountId>, sp_runtime::traits::BadOrigin> {
        ensure_signed(origin.clone())
            .map(Proposer::Community)
            .or_else(|_| {
                T::TechnicalCommitteeVMO::ensure_origin(origin.clone())
                    .map(|_| Committee::Technical)
                    .or_else(|_| {
                        T::UpgradeCommitteeVMO::ensure_origin(origin).map(|_| Committee::Upgrade)
                    })
                    .map(Proposer::Committee)
            })
    }

    /// Returns the current identity or emits `MissingCurrentIdentity`.
    fn current_did_or_missing() -> Result<IdentityId, Error<T>> {
        Context::current_identity::<Identity<T>>().ok_or_else(|| Error::<T>::MissingCurrentIdentity)
    }

    /// Ensure that the proposer inferred via `origin` is the owner of the proposal,
    /// which must be in the cool off period.
    ///
    /// # Errors
    /// * `ProposalIsImmutable`: A Proposal is mutable only during its cool off period.
    /// * `BadOrigin`: Only the owner of the proposal can mutate it.
    fn ensure_owned_by_alterable(
        origin: T::Origin,
        id: PipId,
    ) -> Result<Proposer<T::AccountId>, DispatchError> {
        // 1. Only owner can act on proposal.
        let pip = Self::proposals(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
        let proposer = Self::ensure_infer_proposer(origin)?;
        ensure!(proposer == pip.proposer, DispatchError::BadOrigin);

        // 2. Check that the proposal is pending.
        Self::is_proposal_state(id, ProposalState::Pending)?;

        // 3. Proposal is *ONLY* alterable during its cool-off period.
        let curr_block_number = <system::Module<T>>::block_number();
        ensure!(
            pip.cool_off_until > curr_block_number,
            Error::<T>::ProposalIsImmutable
        );

        Ok(pip.proposer)
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

        // 2. Expire all PIPs scheduled for this block.
        let did = Context::current_identity::<Identity<T>>().unwrap_or_default();
        <ExpirySchedule<T>>::take(block_number)
            .into_iter()
            .filter(|id| Self::is_proposal_state(*id, ProposalState::Pending).is_ok())
            .for_each(|id| Self::maybe_prune(did, id, ProposalState::Expired));

        Ok(weight)
    }

    /// Rejects the given `id`, refunding the deposit, and possibly pruning the proposal's data.
    fn unsafe_reject_proposal(did: IdentityId, id: PipId) {
        Self::maybe_prune(did, id, ProposalState::Rejected);
    }

    /// Refunds any tokens used to vote or bond a proposal.
    ///
    /// This operation is idempotent wrt. chain state,
    /// i.e., once run, refunding again will refund nothing.
    fn refund_proposal(did: IdentityId, id: PipId) {
        let total_refund =
            <Deposits<T>>::iter_prefix_values(id).fold(0.into(), |acc, depo_info| {
                Self::reduce_lock(&depo_info.owner, depo_info.amount).unwrap();
                depo_info.amount.saturating_add(acc)
            });
        <Deposits<T>>::remove_prefix(id);
        Self::deposit_event(RawEvent::ProposalRefund(did, id, total_refund));
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
            let cool_until = Self::proposals(id).unwrap().cool_off_until;
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
    fn prune_data(did: IdentityId, id: PipId, state: ProposalState, prune: bool) {
        Self::refund_proposal(did, id);
        Self::decrement_count_if_active(state);
        if prune {
            <ProposalResult<T>>::remove(id);
            <ProposalVotes<T>>::remove_prefix(id);
            <ProposalMetadata<T>>::remove(id);
            if let Some(Proposer::Committee(_)) = Self::proposals(id).map(|p| p.proposer) {
                CommitteePips::mutate(|list| list.retain(|&i| i != id));
            }
            <Proposals<T>>::remove(id);
            PipSkipCount::remove(id);
        }
        Self::deposit_event(RawEvent::PipClosed(did, id, prune));
    }

    /// First set the state to `new_state`
    /// and then possibly prune (nearly) all the PIP data, if configuration allows.
    fn maybe_prune(did: IdentityId, id: PipId, new_state: ProposalState) {
        Self::update_proposal_state(did, id, new_state);
        Self::prune_data(did, id, new_state, Self::prune_historical_pips());
    }

    fn schedule_pip_for_execution(did: IdentityId, id: PipId) {
        // Set the default enactment period and move it to `Scheduled`
        let curr_block_number = <system::Module<T>>::block_number();
        let executed_at = curr_block_number + Self::default_enactment_period();

        Self::update_proposal_state(did, id, ProposalState::Scheduled);
        <PipToSchedule<T>>::insert(id, executed_at);
        <ExecutionSchedule<T>>::append(executed_at, id);
        let event = RawEvent::ExecutionScheduled(did, id, Zero::zero(), executed_at);
        Self::deposit_event(event);
    }

    /// Execute the PIP given by `id`.
    /// Panics if the PIP doesn't exist or isn't scheduled.
    fn execute_proposal(id: PipId) -> Weight {
        let proposal = Self::proposals(id).expect("PIP was scheduled but doesn't exist");
        assert_eq!(proposal.state, ProposalState::Scheduled);
        let res = proposal.proposal.dispatch(system::RawOrigin::Root.into());
        let weight = res.unwrap_or_else(|e| e.post_info).actual_weight;
        let new_state = res.map_or(ProposalState::Failed, |_| ProposalState::Executed);
        let did = Context::current_identity::<Identity<T>>().unwrap_or_default();
        Self::maybe_prune(did, id, new_state);
        weight.unwrap_or(0)
    }

    /// Update the proposal state of `did` setting it to `new_state`.
    fn update_proposal_state(
        did: IdentityId,
        id: PipId,
        new_state: ProposalState,
    ) -> ProposalState {
        <Proposals<T>>::mutate(id, |proposal| {
            if let Some(ref mut proposal) = proposal {
                if (proposal.state, new_state) != (ProposalState::Pending, ProposalState::Scheduled)
                {
                    Self::decrement_count_if_active(proposal.state);
                }
                proposal.state = new_state;
            }
        });
        Self::deposit_event(RawEvent::ProposalStateUpdated(did, id, new_state));
        new_state
    }

    /// Returns `Ok(_)` iff `id` has `state`.
    fn is_proposal_state(id: PipId, state: ProposalState) -> DispatchResult {
        let proposal = Self::proposals(id).ok_or_else(|| Error::<T>::NoSuchProposal)?;
        ensure!(proposal.state == state, Error::<T>::IncorrectProposalState);
        Ok(())
    }

    /// Returns `true` if `state` is `Pending | Scheduled`.
    fn is_active(state: ProposalState) -> bool {
        matches!(state, ProposalState::Pending | ProposalState::Scheduled)
    }

    /// Decrement active proposal count if `state` signifies it is active.
    fn decrement_count_if_active(state: ProposalState) {
        if Self::is_active(state) {
            ActivePipCount::mutate(|count| *count -= 1);
        }
    }
}

impl<T: Trait> Module<T> {
    /// Increase `acc`'s locked deposit for all PIPs by `amount`,
    /// or fail if there's not enough free balance after adding `amount` to lock.
    fn increase_lock(acc: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        <T as Trait>::Currency::increase_lock(
            PIPS_LOCK_ID,
            acc,
            amount,
            WithdrawReasons::all(),
            |sum| {
                <T as Trait>::Currency::free_balance(acc)
                    .checked_sub(&sum)
                    .ok_or(Error::<T>::InsufficientDeposit.into())
                    .map(drop)
            },
        )
    }

    /// Reduce `acc`'s locked deposit for all PIPs by `amount`,
    /// or fail if `amount` hasn't been locked for PIPs.
    fn reduce_lock(acc: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        <T as Trait>::Currency::reduce_lock(PIPS_LOCK_ID, acc, amount)
    }

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
        <Proposals<T>>::iter()
            .filter(|(_, pip)| pip.proposer == proposer)
            .map(|(_, pip)| pip.id)
            .collect()
    }

    /// Retrieve proposals `address` voted on
    pub fn voted_on(address: T::AccountId) -> Vec<PipId> {
        <Proposals<T>>::iter()
            .filter_map(|(_, pip)| Self::proposal_vote(pip.id, &address).map(|_| pip.id))
            .collect::<Vec<_>>()
    }

    /// Retrieve historical voting of `who` account.
    pub fn voting_history_by_address(
        who: T::AccountId,
    ) -> HistoricalVotingByAddress<Vote<BalanceOf<T>>> {
        <Proposals<T>>::iter()
            .filter_map(|(_, pip)| {
                Some(VoteByPip {
                    pip: pip.id,
                    vote: Self::proposal_vote(pip.id, &who)?,
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

    /// Changes the vote of `voter` to `vote`, if any.
    fn unsafe_vote(id: PipId, voter: T::AccountId, vote: Vote<BalanceOf<T>>) -> DispatchResult {
        let mut stats = Self::proposal_result(id);

        // Update the vote and get the old one, if any, in which case also remove it from stats.
        if let Some(Vote(direction, deposit)) = <ProposalVotes<T>>::get(id, voter.clone()) {
            let (count, stake) = match direction {
                true => (&mut stats.ayes_count, &mut stats.ayes_stake),
                false => (&mut stats.nays_count, &mut stats.nays_stake),
            };
            *count -= 1;
            *stake -= deposit;
        }

        // Add new vote to stats.
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

        // Commit all changes.
        <ProposalResult<T>>::insert(id, stats);
        <ProposalVotes<T>>::insert(id, voter, vote);

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
