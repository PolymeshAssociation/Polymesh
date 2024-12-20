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

//! # Pips Module
//!
//! Polymesh Improvement Proposals (PIPs) are dispatchables that can be `propose`d for execution.
//! These PIPs can either be proposed by a committee, or they can be proposed by a community member,
//! in which case they can `vote`d on by all POLYX token holders.
//!
//! Voting, or rather "signalling", which currently scales linearly with POLX,
//! in this system is used to direct the Governance Councils (GCs)
//! attention by moving proposals up and down a review queue, specific to community proposals.
//!
//! From time to time, the GC will take a `snapshot` of this queue,
//! meet and review PIPs, and reject, approve, or skip the proposal (via `enact_snapshot_results`).
//! Any approved PIPs from this snapshot will then be scheduled,
//! in order of signal value, to be executed automatically on the blockchain.
//! However, using `reschedule_execution`, a special Release Coordinator (RC), a member of the GC,
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

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

mod types;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode, FullCodec};
use core::cmp::Ordering;
use core::mem;
use frame_support::dispatch::DispatchClass::Operational;
use frame_support::dispatch::{DispatchResult, DispatchResultWithPostInfo, Weight};
use frame_support::pallet_prelude::*;
use frame_support::storage::types::StorageValue;
use frame_support::storage::{IterableStorageDoubleMap, IterableStorageMap};
use frame_support::traits::schedule::{DispatchTime, Named, Priority, HARD_DEADLINE};
use frame_support::traits::{Currency, EnsureOrigin, Get, LockIdentifier, WithdrawReasons};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use frame_system::pallet_prelude::OriginFor;
use frame_system::{ensure_root, ensure_signed, RawOrigin};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Dispatchable, Hash, One, Saturating, Zero};
use sp_runtime::DispatchError;
use sp_std::convert::From;
use sp_version::RuntimeVersion;

use pallet_base::{ensure_opt_string_limited, try_next_post};
use pallet_identity::PermissionedCallOriginData;
use polymesh_common_utilities::constants::PIP_MAX_REPORTING_SIZE;
use polymesh_common_utilities::protocol_fee::{ChargeProtocolFee, ProtocolOp};
use polymesh_common_utilities::traits::balances::LockableCurrencyExt;
use polymesh_common_utilities::traits::governance_group::GovernanceGroupTrait;
use polymesh_common_utilities::traits::group::GroupTrait;
use polymesh_common_utilities::{with_transaction, MaybeBlock, GC_DID, TECHNICAL_DID, UPGRADE_DID};
use polymesh_primitives::constants::{PIP_EXECUTION, PIP_EXPIRY};
use polymesh_primitives::{impl_checked_inc, storage_migration_ver, Balance, IdentityId, Url};
use polymesh_primitives_derive::VecU8StrongTyped;
use polymesh_runtime_common::PipsEnactSnapshotMaximumWeight;

use crate::types::SnapshotMetadata;
use crate::types::{DepositInfo, Pip, PipsMetadata, SnapshotId, SnapshottedPip, VotingResult};
use crate::types::{PipDescription, PipId, ProposalData, ProposalState, Proposer, Vote};

storage_migration_ver!(2);

pub trait WeightInfo {
    fn set_prune_historical_pips() -> Weight;
    fn set_min_proposal_deposit() -> Weight;
    fn set_default_enactment_period() -> Weight;
    fn set_pending_pip_expiry() -> Weight;
    fn set_max_pip_skip_count() -> Weight;
    fn set_active_pip_limit() -> Weight;
    fn propose_from_community() -> Weight;
    fn propose_from_committee() -> Weight;
    fn vote() -> Weight;
    fn approve_committee_proposal() -> Weight;
    fn reject_proposal() -> Weight;
    fn prune_proposal() -> Weight;
    fn reschedule_execution() -> Weight;
    fn clear_snapshot() -> Weight;
    fn snapshot() -> Weight;
    fn enact_snapshot_results(a: u32, r: u32, s: u32) -> Weight;
    fn execute_scheduled_pip() -> Weight;
    fn expire_scheduled_pip() -> Weight;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// Only the GC release coordinator is allowed to reschedule proposal execution.
        RescheduleNotByReleaseCoordinator,
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
        SnapshotIdMismatch,
        /// Execution of a scheduled proposal failed because it is missing.
        ScheduledProposalDoesntExist,
        /// A proposal that is not in a scheduled state cannot be executed.
        ProposalNotInScheduledState,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Pruning Historical PIPs is enabled or disabled (caller DID, old value, new value)
        HistoricalPipsPruned(IdentityId, bool, bool),
        /// A PIP was made with a `Balance` stake.
        ///
        /// # Parameters:
        ///
        /// Caller DID, Proposer, PIP ID, deposit, URL, description, expiry time, proposal data.
        ProposalCreated(
            IdentityId,
            Proposer<T::AccountId>,
            PipId,
            Balance,
            Option<Url>,
            Option<PipDescription>,
            MaybeBlock<T::BlockNumber>,
            ProposalData,
        ),
        /// Triggered each time the state of a proposal is amended
        ProposalStateUpdated(IdentityId, PipId, ProposalState),
        /// `AccountId` voted `bool` on the proposal referenced by `PipId`
        Voted(IdentityId, T::AccountId, PipId, bool, Balance),
        /// Pip has been closed, bool indicates whether data is pruned
        PipClosed(IdentityId, PipId, bool),
        /// Execution of a PIP has been scheduled at specific block.
        ExecutionScheduled(IdentityId, PipId, T::BlockNumber),
        /// Default enactment period (in blocks) has been changed.
        /// (caller DID, old period, new period)
        DefaultEnactmentPeriodChanged(IdentityId, T::BlockNumber, T::BlockNumber),
        /// Minimum deposit amount modified
        /// (caller DID, old amount, new amount)
        MinimumProposalDepositChanged(IdentityId, Balance, Balance),
        /// Amount of blocks after which a pending PIP expires.
        /// (caller DID, old expiry, new expiry)
        PendingPipExpiryChanged(
            IdentityId,
            MaybeBlock<T::BlockNumber>,
            MaybeBlock<T::BlockNumber>,
        ),
        /// The maximum times a PIP can be skipped was changed.
        /// (caller DID, old value, new value)
        MaxPipSkipCountChanged(IdentityId, u8, u8),
        /// The maximum number of active PIPs was changed.
        /// (caller DID, old value, new value)
        ActivePipLimitChanged(IdentityId, u32, u32),
        /// Refund proposal
        /// (id, total amount)
        ProposalRefund(IdentityId, PipId, Balance),
        /// The snapshot was cleared.
        SnapshotCleared(IdentityId, SnapshotId),
        /// A new snapshot was taken.
        SnapshotTaken(IdentityId, SnapshotId, Vec<SnapshottedPip>),
        /// A PIP in the snapshot queue was skipped.
        /// (gc_did, pip_id, new_skip_count)
        PipSkipped(IdentityId, PipId, u8),
        /// Results (e.g., approved, rejected, and skipped), were enacted for some PIPs.
        /// (gc_did, snapshot_id_opt, skipped_pips_with_new_count, rejected_pips, approved_pips)
        SnapshotResultsEnacted(
            IdentityId,
            Option<SnapshotId>,
            Vec<(PipId, u8)>,
            Vec<PipId>,
            Vec<PipId>,
        ),
        /// Scheduling of the PIP for execution failed in the scheduler pallet.
        ExecutionSchedulingFailed(IdentityId, PipId, T::BlockNumber),
        /// The PIP has been scheduled for expiry.
        ExpiryScheduled(IdentityId, PipId, T::BlockNumber),
        /// Scheduling of the PIP for expiry failed in the scheduler pallet.
        ExpirySchedulingFailed(IdentityId, PipId, T::BlockNumber),
        /// Cancelling the PIP execution failed in the scheduler pallet.
        ExecutionCancellingFailed(PipId),
    }

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_timestamp::Config
        + polymesh_common_utilities::traits::identity::Config
        + polymesh_common_utilities::CommonConfig
        + pallet_base::Config
    {
        /// Currency type for this module.
        type Currency: LockableCurrencyExt<Self::AccountId, Moment = Self::BlockNumber>;

        /// Origin for enacting results for PIPs (reject, approve, skip, etc.).
        type VotingMajorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Committee
        type GovernanceCommittee: GovernanceGroupTrait<<Self as pallet_timestamp::Config>::Moment>;

        /// Voting majority origin for Technical Committee.
        type TechnicalCommitteeVMO: EnsureOrigin<Self::RuntimeOrigin>;

        /// Voting majority origin for Upgrade Committee.
        type UpgradeCommitteeVMO: EnsureOrigin<Self::RuntimeOrigin>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight calaculation.
        type WeightInfo: WeightInfo;

        /// Scheduler of executed or expired proposals. Since the scheduler module does not have
        /// instances, the names of scheduled tasks should be guaranteed to be unique in this
        /// pallet. Names cannot be just PIP IDs because names of executed and expired PIPs should be
        /// different.
        type Scheduler: Named<Self::BlockNumber, Self::SchedulerCall, Self::SchedulerOrigin>;

        /// A call type used by the scheduler.
        type SchedulerCall: From<Call<Self>>
            + Into<<Self as polymesh_common_utilities::traits::identity::Config>::Proposal>;
    }

    /// Set to `true` if historical PIPs data must be removed.
    #[pallet::storage]
    #[pallet::getter(fn prune_historical_pips)]
    pub type PruneHistoricalPips<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// The minimum amount to be used as a deposit for community PIP creation.
    #[pallet::storage]
    #[pallet::getter(fn min_proposal_deposit)]
    pub type MinimumProposalDeposit<T: Config> = StorageValue<_, Balance, ValueQuery>;

    /// Default enactment period that will be use after a proposal is accepted by GC.
    #[pallet::storage]
    #[pallet::getter(fn default_enactment_period)]
    pub type DefaultEnactmentPeriod<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// Number of blocks it will take, after a `Pending` PIP expires, assuming it has not transitioned to another `ProposalState`.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn pending_pip_expiry)]
    pub type PendingPipExpiry<T: Config> = StorageValue<_, MaybeBlock<T::BlockNumber>, ValueQuery>;

    /// Maximum times a PIP can be skipped before triggering `CannotSkipPip` in `enact_snapshot_results`.
    #[pallet::storage]
    #[pallet::getter(fn max_pip_skip_count)]
    pub type MaxPipSkipCount<T: Config> = StorageValue<_, u8, ValueQuery>;

    /// The maximum allowed number for active PIPs. Once reached, new PIPs cannot be proposed by community members.
    #[pallet::storage]
    #[pallet::getter(fn active_pip_limit)]
    pub type ActivePipLimit<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Proposal's identifier.
    #[pallet::storage]
    #[pallet::getter(fn pip_id_sequence)]
    pub type PipIdSequence<T: Config> = StorageValue<_, PipId, ValueQuery>;

    /// Snaphot's identifier.
    #[pallet::storage]
    #[pallet::getter(fn snapshot_id_sequence)]
    pub type SnapshotIdSequence<T: Config> = StorageValue<_, SnapshotId, ValueQuery>;

    /// Total count of pending or scheduled PIPs.
    #[pallet::storage]
    #[pallet::getter(fn active_pip_count)]
    pub type ActivePipCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// The [`PipsMetadata`] for each proposal ([`PipId`]).
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn proposal_metadata)]
    pub type ProposalMetadata<T: Config> =
        StorageMap<_, Twox64Concat, PipId, PipsMetadata<T::BlockNumber>, OptionQuery>;

    /// All locked [`DepositInfo`] per [`PipId`] for each account.
    #[pallet::storage]
    #[pallet::getter(fn deposits)]
    pub type Deposits<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PipId,
        Twox64Concat,
        T::AccountId,
        DepositInfo<T::AccountId>,
        OptionQuery,
    >;

    /// The [`Pip`] for each proposal ([`PipId`]).
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageMap<_, Twox64Concat, PipId, Pip<T::Proposal, T::AccountId>, OptionQuery>;

    /// The [`VotingResult`] for each proposal ([`PipId`]).
    #[pallet::storage]
    #[pallet::getter(fn proposal_result)]
    pub type ProposalResult<T: Config> =
        StorageMap<_, Twox64Concat, PipId, VotingResult, ValueQuery>;

    /// The Votes ([`Vote`]) for each proposal ([`PipId`]) per account.
    #[pallet::storage]
    #[pallet::getter(fn proposal_vote)]
    pub type ProposalVotes<T: Config> =
        StorageDoubleMap<_, Twox64Concat, PipId, Twox64Concat, T::AccountId, Vote, OptionQuery>;

    /// Maps PIPs to the block at which they will be executed.
    #[pallet::storage]
    #[pallet::getter(fn pip_to_schedule)]
    pub type PipToSchedule<T: Config> =
        StorageMap<_, Twox64Concat, PipId, T::BlockNumber, OptionQuery>;

    /// A live priority queue (lowest priority at index 0)
    /// of pending PIPs up to the active limit.
    /// Priority is defined by the `weight` in the `SnapshottedPip`.
    ///
    /// Unlike `SnapshotQueue`, this queue is live, getting updated with each vote cast.
    /// The snapshot is therefore essentially a point-in-time clone of this queue.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn live_queue)]
    pub type LiveQueue<T> = StorageValue<_, Vec<SnapshottedPip>, ValueQuery>;

    /// The priority queue (lowest priority at index 0) of PIPs at the point of snapshotting.
    /// Priority is defined by the `weight` in the `SnapshottedPip`.
    ///
    /// A queued PIP can be skipped. Doing so bumps the `pip_skip_count`.
    /// Once a (configurable) threshhold is exceeded, a PIP cannot be skipped again.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn snapshot_queue)]
    pub type SnapshotQueue<T> = StorageValue<_, Vec<SnapshottedPip>, ValueQuery>;

    /// The [`SnapshotMetadata`].
    #[pallet::storage]
    #[pallet::getter(fn snapshot_metadata)]
    pub type SnapshotMeta<T: Config> =
        StorageValue<_, SnapshotMetadata<T::BlockNumber, T::AccountId>, OptionQuery>;

    /// The number of times a certain PIP has been skipped.
    /// Once a (configurable) threshhold is exceeded, a PIP cannot be skipped again.
    #[pallet::storage]
    #[pallet::getter(fn pip_skip_count)]
    pub type PipSkipCount<T: Config> = StorageMap<_, Twox64Concat, PipId, u8, OptionQuery>;

    /// All existing PIPs where the proposer is a committee.
    /// This list is a cache of all ids in `Proposals` with `Proposer::Committee(_)`.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn committee_pips)]
    pub type CommitteePips<T> = StorageValue<_, Vec<PipId>, ValueQuery>;

    /// The ([`ProposalState`]) of a given PIP ([`PipId`]).
    #[pallet::storage]
    #[pallet::getter(fn proposal_state)]
    pub type ProposalStates<T: Config> =
        StorageMap<_, Twox64Concat, PipId, ProposalState, OptionQuery>;

    /// Storage version.
    #[pallet::storage]
    #[pallet::getter(fn storage_version)]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}
