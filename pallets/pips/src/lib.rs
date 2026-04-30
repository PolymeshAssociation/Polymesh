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
//! The Pips module allows for the creation, voting, and management of Polymesh Improvement Proposals (PIPs).
//! PIPs can be proposed by either a committee or a community member. Community proposals can be voted on by all POLYX token holders.
//!
//! ## Voting and Signalling
//!
//! Voting in this system is used to direct the Governance Councils' (GCs) attention by moving proposals up and down a review queue.
//! The GC periodically takes a snapshot of this queue, reviews the PIPs, and decides to reject, approve, or skip them.
//! Approved PIPs are scheduled for execution on the blockchain. The Release Coordinator (RC), a GC member, can reschedule approved PIPs, except for those replacing the RC.
//! The snapshot can be cleared by the GC once it is no longer relevant.
//!
//! ## Skipping and Limits
//!
//! The GC can skip a PIP, which increments its "skipped count". If a configurable limit for the skipped count is exceeded, the PIP can no longer be skipped.
//! Committee proposals do not enter the snapshot or receive votes but can be approved by the GC at any time.
//! The GC can also reject active proposals or prune them to remove their storage.
//!
//! ## Overview
//!
//! The Pips module provides functions for:
//!
//! - Proposing and amending PIPs
//! - Voting on PIPs to adjust their priority in the review queue
//! - Taking and clearing snapshots of the queue
//! - Approving, rejecting, skipping, and rescheduling PIPs
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### Configuration Changes
//!
//! - `set_prune_historical_pips` - Change whether historical PIPs are pruned
//! - `set_min_proposal_deposit` - Change the minimum deposit to create a proposal
//! - `set_default_enactment_period` - Change the period after which the proposal is executed
//! - `set_pending_pip_expiry` - Change the amount of blocks after which a pending PIP is expired
//! - `set_max_pip_skip_count` - Change the maximum times a PIP can be skipped
//! - `set_active_pip_limit` - Change the maximum number of concurrently active PIPs
//!
//! #### Other Functions
//!
//! - `propose` - Token holders can propose a new PIP
//! - `vote` - Token holders can vote on a PIP
//! - `approve_committee_proposal` - Allows the GC to approve a committee proposal
//! - `reject_proposal` - Reject an active proposal and refund deposits
//! - `prune_proposal` - Prune all storage associated with a proposal and refund deposits
//! - `reschedule_execution` - Release Coordinator can reschedule a PIP's execution
//! - `clear_snapshot` - Clears the snapshot
//! - `snapshot` - Takes a new snapshot of the review queue
//! - `enact_snapshot_results` - Enters results (approve, reject, and skip) for PIPs in the snapshot

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

mod types;

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchClass::Operational;
use frame_support::dispatch::{DispatchResult, DispatchResultWithPostInfo};
use frame_support::ensure;
use frame_support::pallet_prelude::*;
use frame_support::storage::types::StorageValue;
use frame_support::traits::schedule::v3::Named as ScheduleNamed;
use frame_support::traits::schedule::DispatchTime;
use frame_support::traits::{Currency, EnsureOrigin, Get, WithdrawReasons};
use frame_support::traits::{QueryPreimage, StorePreimage};
use frame_support::weights::Weight;
use frame_system::pallet_prelude::*;
use frame_system::{ensure_root, ensure_signed, RawOrigin};
use sp_runtime::traits::{BlakeTwo256, Dispatchable, Hash, One, Saturating, Zero};
use sp_runtime::DispatchError;
use sp_std::boxed::Box;
use sp_std::convert::From;
use sp_std::vec::Vec;
use sp_version::RuntimeVersion;

use pallet_balances::impls::LockableCurrencyExt;
use pallet_base::{ensure_opt_string_limited, try_next_post};
use pallet_identity::{Config as IdentityConfig, PermissionedCallOriginData};
use polymesh_primitives::constants::PIP_MAX_REPORTING_SIZE;
use polymesh_primitives::protocol_fee::{ChargeProtocolFee, ProtocolOp};
use polymesh_primitives::storage_migration_ver;
use polymesh_primitives::traits::group::GroupTrait;
use polymesh_primitives::traits::GovernanceGroupTrait;
use polymesh_primitives::{Balance, IdentityId, MaybeBlock, Url};
use polymesh_primitives::{GC_DID, TECHNICAL_DID, UPGRADE_DID};
use polymesh_runtime_common::PipsEnactSnapshotMaximumWeight;

use crate::types::{compare_spip, ProposalData, MAX_NORMAL_PRIORITY, PIPS_LOCK_ID};
pub use crate::types::{Committee, DepositInfo, SnapshottedPip, VotingResult};
pub use crate::types::{Pip, PipDescription, PipsMetadata, SnapshotId, SnapshotMetadata};
pub use crate::types::{PipId, ProposalState, Proposer, SnapshotResult, Vote, VoteCount};
pub use pallet::*;

type SkippedCount = u8;
type System<T> = frame_system::Pallet<T>;

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
    fn snapshot(q: u32) -> Weight;
    fn enact_snapshot_results(a: u32, r: u32, s: u32) -> Weight;
    fn execute_scheduled_pip() -> Weight;
    fn expire_scheduled_pip() -> Weight;
    fn remove_pending_storage(r: u32, v: u32) -> Weight;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// Only the GC release coordinator is allowed to reschedule proposal execution.
        RescheduleNotByReleaseCoordinator,
        /// The given dispatchable call is not valid for this proposal.
        /// The proposal must be from the community, but isn't.
        NotFromCommunity,
        /// The given dispatchable call is not valid for this proposal.
        /// The proposal must be from a committee, but isn't.
        NotByCommittee,
        /// The current number of active (pending or scheduled) PIPs exceeds the maximum
        /// and the proposal is not by a committee.
        TooManyActivePips,
        /// Proposer specifies an incorrect deposit amount.
        IncorrectDeposit,
        /// Proposer cannot afford to lock the minimum deposit.
        InsufficientDeposit,
        /// The proposal does not exist.
        NoSuchProposal,
        /// The caller is not a member of the governance committee.
        NotACommitteeMember,
        /// The specified block number is less than the current block number.
        InvalidFutureBlockNumber,
        /// The number of votes exceeds the allowed limit.
        NumberOfVotesExceeded,
        /// The stake amount of a vote exceeds the allowed limit.
        StakeAmountOfVotesExceeded,
        /// The current DID is missing.
        MissingCurrentIdentity,
        /// The proposal is not in the correct state for the requested operation.
        IncorrectProposalState,
        /// When enacting snapshot results, an unskippable PIP was skipped.
        CannotSkipPip,
        /// Tried to enact results for the snapshot queue overflowing its length.
        SnapshotResultTooLarge,
        /// Tried to enact result for PIP with an ID different from that at the position in the queue.
        SnapshotIdMismatch,
        /// Execution of a scheduled proposal failed because it is missing.
        ScheduledProposalDoesntExist,
        /// A proposal that is not in a scheduled state cannot be executed.
        ProposalNotInScheduledState,
        /// Invalid PIP ID. Pip id was not expected to be in the live queue.
        InvalidPipId,
        /// The provided `limit` is less than the actual queue length.
        SnapshotLimitTooSmall,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Historical PIPs Pruning has been set.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `bool`: The old value of the pruning setting.
        /// - `bool`: The new value of the pruning setting.
        HistoricalPipsPruned(IdentityId, bool, bool),
        /// A PIP was created with a specified `Balance` stake.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `Proposer<T::AccountId>`: The proposer of the PIP.
        /// - `PipId`: The ID of the PIP.
        /// - `Balance`: The deposit amount.
        /// - `Option<Url>`: The URL for proposal discussion.
        /// - `Option<PipDescription>`: The description of the proposal.
        /// - `MaybeBlock<T::BlockNumber>`: The expiry time of the proposal.
        /// - `ProposalData`: The data of the proposal.
        ProposalCreated(
            IdentityId,
            Proposer<T::AccountId>,
            PipId,
            Balance,
            Option<Url>,
            Option<PipDescription>,
            MaybeBlock<BlockNumberFor<T>>,
            ProposalData,
        ),
        /// The state of a proposal was updated.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `PipId`: The ID of the PIP.
        /// - `ProposalState`: The new state of the proposal.
        ProposalStateUpdated(IdentityId, PipId, ProposalState),
        /// An account voted on a proposal.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `T::AccountId`: The account that voted.
        /// - `PipId`: The ID of the PIP.
        /// - `bool`: The vote (true for aye, false for nay).
        /// - `Balance`: The deposit amount of the vote.
        Voted(IdentityId, T::AccountId, PipId, bool, Balance),
        /// A PIP was closed.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `PipId`: The ID of the PIP.
        /// - `bool`: Indicates whether the data was pruned.
        PipClosed(IdentityId, PipId, bool),
        /// The execution of a PIP was scheduled.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `PipId`: The ID of the PIP.
        /// - `BlockNumber`: The block number at which the PIP is scheduled for execution.
        ExecutionScheduled(IdentityId, PipId, BlockNumberFor<T>),
        /// The default enactment period was changed.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `BlockNumber`: The old enactment period.
        /// - `BlockNumber`: The new enactment period.
        DefaultEnactmentPeriodChanged(IdentityId, BlockNumberFor<T>, BlockNumberFor<T>),
        /// The minimum deposit amount for proposals was changed.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `Balance`: The old deposit amount.
        /// - `Balance`: The new deposit amount.
        MinimumProposalDepositChanged(IdentityId, Balance, Balance),
        /// The expiry time for pending PIPs was changed.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `MaybeBlock<T::BlockNumber>`: The old expiry time.
        /// - `MaybeBlock<T::BlockNumber>`: The new expiry time.
        PendingPipExpiryChanged(
            IdentityId,
            MaybeBlock<BlockNumberFor<T>>,
            MaybeBlock<BlockNumberFor<T>>,
        ),
        /// The maximum number of times a PIP can be skipped was changed.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `SkippedCount`: The old skip count.
        /// - `SkippedCount`: The new skip count.
        MaxPipSkipCountChanged(IdentityId, SkippedCount, SkippedCount),
        /// The maximum number of active PIPs was changed.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `u32`: The old active PIP limit.
        /// - `u32`: The new active PIP limit.
        ActivePipLimitChanged(IdentityId, u32, u32),
        /// A proposal was refunded.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `PipId`: The ID of the PIP.
        /// - `Balance`: The total amount refunded.
        ProposalRefund(IdentityId, PipId, Balance),
        /// The snapshot was cleared.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `SnapshotId`: The ID of the snapshot.
        SnapshotCleared(IdentityId, SnapshotId),
        /// A new snapshot was taken.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `SnapshotId`: The ID of the snapshot.
        /// - `Vec<SnapshottedPip>`: The list of PIPs in the snapshot.
        SnapshotTaken(IdentityId, SnapshotId, Vec<SnapshottedPip>),
        /// A PIP in the snapshot queue was skipped.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `PipId`: The ID of the PIP.
        /// - `SkippedCount`: The new skip count.
        PipSkipped(IdentityId, PipId, SkippedCount),
        /// Results were enacted for some PIPs in the snapshot queue.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `Option<SnapshotId>`: The ID of the snapshot, if any.
        /// - `Vec<(PipId, SkippedCount)>`: The list of skipped PIPs with their new skip counts.
        /// - `Vec<PipId>`: The list of rejected PIPs.
        /// - `Vec<PipId>`: The list of approved PIPs.
        SnapshotResultsEnacted(
            IdentityId,
            Option<SnapshotId>,
            Vec<(PipId, SkippedCount)>,
            Vec<PipId>,
            Vec<PipId>,
        ),
        /// Scheduling of the PIP for execution failed in the scheduler pallet.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `PipId`: The ID of the PIP.
        /// - `BlockNumber`: The block number at which the PIP was scheduled for execution.
        ExecutionSchedulingFailed(IdentityId, PipId, BlockNumberFor<T>),
        /// The PIP has been scheduled for expiry.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `PipId`: The ID of the PIP.
        /// - `BlockNumber`: The block number at which the PIP is scheduled for expiry.
        ExpiryScheduled(IdentityId, PipId, BlockNumberFor<T>),
        /// Scheduling of the PIP for expiry failed in the scheduler pallet.
        ///
        /// Parameters:
        /// - `IdentityId`: The DID of the caller.
        /// - `PipId`: The ID of the PIP.
        /// - `BlockNumber`: The block number at which the PIP was scheduled for expiry.
        ExpirySchedulingFailed(IdentityId, PipId, BlockNumberFor<T>),
        /// Cancelling the PIP execution failed in the scheduler pallet.
        ///
        /// Parameters:
        /// - `PipId`: The ID of the PIP.
        ExecutionCancellingFailed(PipId),
    }

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_timestamp::Config + IdentityConfig + pallet_base::Config
    {
        /// Currency type for this module.
        type Currency: LockableCurrencyExt<Self::AccountId, Balance, Moment = BlockNumberFor<Self>>;
        /// Origin type for enacting results for PIPs (e.g., reject, approve, skip).
        type VotingMajorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// Governance committee responsible for overseeing the PIPs.
        type GovernanceCommittee: GovernanceGroupTrait<<Self as pallet_timestamp::Config>::Moment>;
        /// Voting majority origin for Technical Committee.
        type TechnicalCommitteeVMO: EnsureOrigin<Self::RuntimeOrigin>;
        /// Voting majority origin for Upgrade Committee.
        type UpgradeCommitteeVMO: EnsureOrigin<Self::RuntimeOrigin>;
        /// Weight calculation for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
        /// Scheduler for executed or expired proposals. The scheduler module does not have instances,
        /// so the names of scheduled tasks must be unique within this pallet. Names cannot be just PIP
        /// IDs because names of executed and expired PIPs should be different.
        type Scheduler: ScheduleNamed<
            BlockNumberFor<Self>,
            Self::SchedulerCall,
            Self::SchedulerOrigin,
            Hasher = Self::Hashing,
        >;
        /// A call type used by the scheduler.
        type SchedulerCall: From<Call<Self>> + Into<<Self as IdentityConfig>::Proposal> + Encode;
        /// The maximum number of votes that can be pruned at once.
        #[pallet::constant]
        type MaxRefundsAndVotesPruned: Get<u32>;
        /// Preimage provider for the scheduler.
        type SchedulerPreimage: QueryPreimage<H = Self::Hashing> + StorePreimage;
    }

    /// Set to `true` if historical PIPs data must be removed.
    #[pallet::storage]
    pub type PruneHistoricalPips<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// The minimum amount to be used as a deposit for community PIP creation.
    #[pallet::storage]
    pub type MinimumProposalDeposit<T: Config> = StorageValue<_, Balance, ValueQuery>;

    /// Default enactment period that will be use after a proposal is accepted by GC.
    #[pallet::storage]
    pub type DefaultEnactmentPeriod<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Number of blocks it will take, after a `Pending` PIP expires, assuming it has not transitioned to another `ProposalState`.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type PendingPipExpiry<T: Config> =
        StorageValue<_, MaybeBlock<BlockNumberFor<T>>, ValueQuery>;

    /// Maximum times a PIP can be skipped before triggering `CannotSkipPip` in `enact_snapshot_results`.
    #[pallet::storage]
    pub type MaxPipSkipCount<T: Config> = StorageValue<_, SkippedCount, ValueQuery>;

    /// The maximum allowed number for active PIPs. Once reached, new PIPs cannot be proposed by community members.
    #[pallet::storage]
    pub type ActivePipLimit<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Proposal's identifier.
    #[pallet::storage]
    pub type PipIdSequence<T: Config> = StorageValue<_, PipId, ValueQuery>;

    /// Snaphot's identifier.
    #[pallet::storage]
    pub type SnapshotIdSequence<T: Config> = StorageValue<_, SnapshotId, ValueQuery>;

    /// Total count of pending or scheduled PIPs.
    #[pallet::storage]
    pub type ActivePipCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// The [`PipsMetadata`] for each proposal ([`PipId`]).
    #[pallet::storage]
    #[pallet::unbounded]
    pub type ProposalMetadata<T: Config> =
        StorageMap<_, Twox64Concat, PipId, PipsMetadata<BlockNumberFor<T>>, OptionQuery>;

    /// All locked [`DepositInfo`] per [`PipId`] for each account.
    #[pallet::storage]
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
    pub type Proposals<T: Config> =
        StorageMap<_, Twox64Concat, PipId, Pip<T::Proposal, T::AccountId>, OptionQuery>;

    /// The [`VotingResult`] for each proposal ([`PipId`]).
    #[pallet::storage]
    pub type ProposalResult<T: Config> =
        StorageMap<_, Twox64Concat, PipId, VotingResult, ValueQuery>;

    /// The Votes ([`Vote`]) for each proposal ([`PipId`]) per account.
    #[pallet::storage]
    pub type ProposalVotes<T: Config> =
        StorageDoubleMap<_, Twox64Concat, PipId, Twox64Concat, T::AccountId, Vote, OptionQuery>;

    /// Maps PIPs to the block at which they will be executed.
    #[pallet::storage]
    pub type PipToSchedule<T: Config> =
        StorageMap<_, Twox64Concat, PipId, BlockNumberFor<T>, OptionQuery>;

    /// A live priority queue (lowest priority at index 0)
    /// of pending PIPs up to the active limit.
    /// Priority is defined by the `weight` in the `SnapshottedPip`.
    ///
    /// Unlike `SnapshotQueue`, this queue is live, getting updated with each vote cast.
    /// The snapshot is therefore essentially a point-in-time clone of this queue.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type LiveQueue<T: Config> = StorageValue<_, Vec<SnapshottedPip>, ValueQuery>;

    /// The priority queue (lowest priority at index 0) of PIPs at the point of snapshotting.
    /// Priority is defined by the `weight` in the `SnapshottedPip`.
    ///
    /// A queued PIP can be skipped. Doing so bumps the `pip_skip_count`.
    /// Once a (configurable) threshhold is exceeded, a PIP cannot be skipped again.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type SnapshotQueue<T: Config> = StorageValue<_, Vec<SnapshottedPip>, ValueQuery>;

    /// The [`SnapshotMetadata`].
    #[pallet::storage]
    pub type SnapshotMeta<T: Config> =
        StorageValue<_, SnapshotMetadata<BlockNumberFor<T>, T::AccountId>, OptionQuery>;

    /// The number of times a certain PIP has been skipped.
    /// Once a (configurable) threshhold is exceeded, a PIP cannot be skipped again.
    #[pallet::storage]
    pub type PipSkipCount<T: Config> = StorageMap<_, Twox64Concat, PipId, SkippedCount, ValueQuery>;

    /// All existing PIPs where the proposer is a committee.
    /// This list is a cache of all ids in `Proposals` with `Proposer::Committee(_)`.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type CommitteePips<T: Config> = StorageValue<_, Vec<PipId>, ValueQuery>;

    /// The ([`ProposalState`]) of a given PIP ([`PipId`]).
    #[pallet::storage]
    pub type ProposalStates<T: Config> =
        StorageMap<_, Twox64Concat, PipId, ProposalState, OptionQuery>;

    /// All PIPs that still require refunds.
    #[pallet::storage]
    pub type PendingRefunds<T: Config> = StorageMap<_, Twox64Concat, PipId, bool, OptionQuery>;

    /// All PIPs that still require votes to be pruned.
    #[pallet::storage]
    pub type VotesToBePruned<T: Config> = StorageMap<_, Twox64Concat, PipId, bool, OptionQuery>;

    /// Storage version.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub prune_historical_pips: bool,
        pub min_proposal_deposit: Balance,
        pub default_enactment_period: BlockNumberFor<T>,
        pub pending_pip_expiry: MaybeBlock<BlockNumberFor<T>>,
        pub max_pip_skip_count: u8,
        pub active_pip_limit: u32,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            PruneHistoricalPips::<T>::put(self.prune_historical_pips);
            MinimumProposalDeposit::<T>::put(self.min_proposal_deposit);
            DefaultEnactmentPeriod::<T>::put(self.default_enactment_period);
            PendingPipExpiry::<T>::put(self.pending_pip_expiry);
            MaxPipSkipCount::<T>::put(self.max_pip_skip_count);
            ActivePipLimit::<T>::put(self.active_pip_limit);
            StorageVersion::<T>::put(Version::new(2));
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_idle(_now: BlockNumberFor<T>, _remaining_weight: Weight) -> Weight {
            Self::remove_pending_storage()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Sets the pruning setting for historical PIPs. This function can only be called by the root origin.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be the root.
        /// * `prune` - A boolean flag indicating whether completed PIPs should be pruned (`true`) or retained (`false`).
        ///
        /// # Events
        /// * `HistoricalPipsPruned` - Emitted when the pruning setting is changed, containing the old and new values.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by the root origin.
        #[pallet::call_index(0)]
        #[pallet::weight((<T as Config>::WeightInfo::set_prune_historical_pips(), Operational))]
        pub fn set_prune_historical_pips(origin: OriginFor<T>, prune: bool) -> DispatchResult {
            ensure_root(origin)?;
            let old_value = PruneHistoricalPips::<T>::get();
            PruneHistoricalPips::<T>::put(prune);
            Self::deposit_event(Event::HistoricalPipsPruned(GC_DID, old_value, prune));
            Ok(())
        }

        /// Changes the minimum proposal deposit amount required to start a proposal. This function can only be called by the root origin.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be the root.
        /// * `deposit` - The new minimum deposit required to start a proposal.
        ///
        /// # Events
        /// * `MinimumProposalDepositChanged` - Emitted when the minimum proposal deposit is changed, containing the old and new values.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by the root origin.
        #[pallet::call_index(1)]
        #[pallet::weight((<T as Config>::WeightInfo::set_min_proposal_deposit(), Operational))]
        pub fn set_min_proposal_deposit(origin: OriginFor<T>, deposit: Balance) -> DispatchResult {
            ensure_root(origin)?;
            let old_value = MinimumProposalDeposit::<T>::get();
            MinimumProposalDeposit::<T>::put(deposit);
            Self::deposit_event(Event::MinimumProposalDepositChanged(
                GC_DID, old_value, deposit,
            ));
            Ok(())
        }

        /// Changes the default enactment period. This function can only be called by the root origin.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be the root.
        /// * `period` - The new default enactment period.
        ///
        /// # Events
        /// * `DefaultEnactmentPeriodChanged` - Emitted when the default enactment period is changed, containing the old and new values.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by the root origin.
        #[pallet::call_index(2)]
        #[pallet::weight((<T as Config>::WeightInfo::set_default_enactment_period(), Operational))]
        pub fn set_default_enactment_period(
            origin: OriginFor<T>,
            duration: BlockNumberFor<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let old_value = DefaultEnactmentPeriod::<T>::get();
            DefaultEnactmentPeriod::<T>::put(duration);
            Self::deposit_event(Event::DefaultEnactmentPeriodChanged(
                GC_DID, old_value, duration,
            ));
            Ok(())
        }

        /// Sets the expiry duration (in blocks) for pending PIPs. This function can only be called by the root origin.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be the root.
        /// * `expiry` - The new expiry duration for pending PIPs. If `None`, PIPs never expire.
        ///
        /// # Events
        /// * `PendingPipExpiryChanged` - Emitted when the pending PIP expiry duration is changed, containing the old and new values.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by the root origin.
        #[pallet::call_index(3)]
        #[pallet::weight((<T as Config>::WeightInfo::set_pending_pip_expiry(), Operational))]
        pub fn set_pending_pip_expiry(
            origin: OriginFor<T>,
            expiry: MaybeBlock<BlockNumberFor<T>>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let old_value = PendingPipExpiry::<T>::get();
            PendingPipExpiry::<T>::put(expiry);
            Self::deposit_event(Event::PendingPipExpiryChanged(GC_DID, old_value, expiry));
            Ok(())
        }

        /// Sets the maximum number of times a PIP can be skipped. This function can only be called by the root origin.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be the root.
        /// * `max` - The new maximum skip count for PIPs.
        ///
        /// # Events
        /// * `MaxPipSkipCountChanged` - Emitted when the maximum PIP skip count is changed, containing the old and new values.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by the root origin.
        #[pallet::call_index(4)]
        #[pallet::weight((<T as Config>::WeightInfo::set_max_pip_skip_count(), Operational))]
        pub fn set_max_pip_skip_count(origin: OriginFor<T>, max: SkippedCount) -> DispatchResult {
            ensure_root(origin)?;
            let old_value = MaxPipSkipCount::<T>::get();
            MaxPipSkipCount::<T>::put(max);
            Self::deposit_event(Event::MaxPipSkipCountChanged(GC_DID, old_value, max));
            Ok(())
        }

        /// Sets the limit on the number of active PIPs. This function can only be called by the root origin.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be the root.
        /// * `limit` - The new limit on the number of active PIPs.
        ///
        /// # Events
        /// * `ActivePipLimitChanged` - Emitted when the active PIP limit is changed, containing the old and new values.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by the root origin.
        #[pallet::call_index(5)]
        #[pallet::weight((<T as Config>::WeightInfo::set_active_pip_limit(), Operational))]
        pub fn set_active_pip_limit(origin: OriginFor<T>, limit: u32) -> DispatchResult {
            ensure_root(origin)?;
            let old_value = ActivePipLimit::<T>::get();
            ActivePipLimit::<T>::put(limit);
            Self::deposit_event(Event::ActivePipLimitChanged(GC_DID, old_value, limit));
            Ok(())
        }

        /// Proposes a new PIP by submitting a dispatchable which changes the network.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `proposal` - The dispatchable call.
        /// * `deposit` - The deposit amount for the proposal.
        /// * `url` - A link to a website for proposal discussion.
        /// * `description` - A short description of the proposal.
        ///
        /// # Events
        /// * `ProposalCreated`.
        ///
        /// # Errors
        /// * `IncorrectDeposit` - If the deposit amount is less than the required minimum.
        /// * `TooManyActivePips` - If the number of active PIPs exceeds the maximum.
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_from_community())]
        pub fn propose(
            origin: OriginFor<T>,
            proposal: Box<T::Proposal>,
            deposit: Balance,
            url: Option<Url>,
            description: Option<PipDescription>,
        ) -> DispatchResult {
            // Infer the proposer from `origin`.
            let (proposer, did) = Self::ensure_infer_proposer(origin)?;

            // Ensure strings are limited in length.
            ensure_opt_string_limited::<T>(url.as_deref())?;
            ensure_opt_string_limited::<T>(description.as_deref())?;

            // Ensure we can advance the ID counter and get next one.
            let mut seq = PipIdSequence::<T>::get();
            let id = try_next_post::<T, _>(&mut seq)?;

            let charge = || T::ProtocolFee::charge_fee(ProtocolOp::PipsPropose);

            // Add a deposit for community PIPs.
            if let Proposer::Community(ref proposer) = proposer {
                // ...but first make sure active PIP limit isn't crossed.
                // This doesn't apply to committee PIPs.
                let limit = ActivePipLimit::<T>::get();
                ensure!(
                    ActivePipCount::<T>::get() < limit,
                    Error::<T>::TooManyActivePips
                );

                // Pre conditions: caller must have min balance.
                ensure!(
                    deposit >= MinimumProposalDeposit::<T>::get(),
                    Error::<T>::IncorrectDeposit
                );

                // Lock the deposit + charge protocol fees.
                Self::increase_lock(proposer, deposit)?;
                charge()?;
            } else {
                // Committee PIPs cannot have a deposit.
                ensure!(deposit.is_zero(), Error::<T>::NotFromCommunity);
                // Charge protocol fees even for committee PIPs.
                charge()?;
            }

            // Construct and add PIP to storage.
            let created_at = System::<T>::block_number();
            let expiry = PendingPipExpiry::<T>::get() + created_at;
            let transaction_version =
                <T::Version as Get<RuntimeVersion>>::get().transaction_version;
            let proposal_data = Self::reportable_proposal_data(&*proposal);
            ProposalMetadata::<T>::insert(
                id,
                PipsMetadata {
                    id,
                    created_at,
                    url: url.clone(),
                    description: description.clone(),
                    transaction_version,
                    expiry,
                },
            );
            Proposals::<T>::insert(
                id,
                Pip {
                    id,
                    proposal: *proposal,
                    proposer: proposer.clone(),
                },
            );
            ProposalStates::<T>::insert(id, ProposalState::Pending);
            PipIdSequence::<T>::put(seq);
            ActivePipCount::<T>::mutate(|count| *count += 1);

            // Schedule for expiry, as long as `Pending`, at block with number `expiring_at`.
            if let MaybeBlock::Some(expiring_at) = expiry {
                Self::schedule_pip_for_expiry(id, expiring_at)?;
            }

            // Record the deposit and as a signal if we have a community PIP.
            if let Proposer::Community(ref proposer) = proposer {
                <Deposits<T>>::insert(
                    id,
                    proposer,
                    DepositInfo {
                        owner: proposer.clone(),
                        amount: deposit,
                    },
                );

                // Add vote and update voting counter.
                // INTERNAL: It is impossible to overflow counters in the first vote.
                Self::unsafe_vote(id, proposer.clone(), Vote(true, deposit))?;

                // Adjust live queue.
                Self::insert_live_queue(id)?;
            } else {
                CommitteePips::<T>::append(id);
            }

            Self::deposit_event(Event::<T>::ProposalCreated(
                did,
                proposer,
                id,
                deposit,
                url,
                description,
                expiry,
                proposal_data,
            ));

            Ok(())
        }

        /// Casts a vote either in favor or against a PIP with `id`.
        /// The "conviction" or strength of the vote is given by `deposit`, which is reserved.
        ///
        /// Note that `vote` is *not* additive.
        /// That is, `vote(id, true, 50)` followed by `vote(id, true, 40)`
        /// will first reserve `50` and then refund `50 - 10`, ending up with `40` in deposit.
        /// To add atop of existing votes, you'll need `existing_deposit + addition`.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call.
        /// * `id` - The proposal ID to vote on.
        /// * `aye_or_nay` - A boolean representing a vote in favor (`true`) or against (`false`).
        /// * `deposit` - The "conviction" or strength of the vote, represented by the amount of deposit.
        ///
        /// # Events
        /// * `Voted` - Emitted when a vote is successfully cast.
        ///
        /// # Errors
        /// * `NoSuchProposal` - If the `id` does not reference a valid PIP.
        /// * `NotFromCommunity` - If the proposal was made by a committee.
        /// * `IncorrectProposalState` - If the PIP is not in a pending state.
        /// * `InsufficientDeposit` - If the `origin` cannot reserve the required deposit.
        /// * `IncorrectDeposit` - If the deposit amount is less than the required minimum.
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::vote())]
        pub fn vote(
            origin: OriginFor<T>,
            id: PipId,
            aye_or_nay: bool,
            deposit: Balance,
        ) -> DispatchResult {
            let PermissionedCallOriginData {
                sender: voter,
                primary_did,
                ..
            } = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;

            let pip = Proposals::<T>::get(id).ok_or(Error::<T>::NoSuchProposal)?;

            // Proposal must be from the community.
            let proposer = match pip.proposer {
                Proposer::Committee(_) => return Err(Error::<T>::NotFromCommunity.into()),
                Proposer::Community(p) => p,
            };

            if proposer == voter {
                // a) Deposit must be above minimum.
                // Note that proposer can still vote against their own PIP.
                ensure!(
                    deposit >= MinimumProposalDeposit::<T>::get(),
                    Error::<T>::IncorrectDeposit
                );
            }

            // Proposal must be pending.
            Self::is_proposal_state(id, ProposalState::Pending)?;

            let old_res = Self::aggregate_result(id);

            // Reserve the deposit, or refund if needed.
            let curr_deposit = Deposits::<T>::get(id, &voter)
                .map(|d| d.amount)
                .unwrap_or_default();
            if deposit < curr_deposit {
                Self::reduce_lock(&voter, curr_deposit - deposit)?;
            } else {
                Self::increase_lock(&voter, deposit - curr_deposit)?;
            }
            // Save the vote.
            Self::unsafe_vote(id, voter.clone(), Vote(aye_or_nay, deposit))?;

            // Adjust live queue.
            Self::adjust_live_queue(id, old_res);

            <Deposits<T>>::insert(
                id,
                &voter,
                DepositInfo {
                    owner: voter.clone(),
                    amount: deposit,
                },
            );

            Self::deposit_event(Event::Voted(primary_did, voter, id, aye_or_nay, deposit));

            Ok(())
        }

        /// Approves the pending committee PIP given by the `id`.
        ///
        /// This function can only be called by a Governance Committee (GC) voting majority.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be a GC voting majority.
        /// * `id` - The proposal ID of the PIP to be approved.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by a GC voting majority.
        /// * `NoSuchProposal` - If the PIP with the given `id` does not exist.
        /// * `IncorrectProposalState` - If the proposal is not in a pending state.
        /// * `NotByCommittee` - If the proposal was not made by a committee.
        ///
        /// # Notes
        /// This function schedules the PIP for execution if all checks pass.
        #[pallet::call_index(8)]
        #[pallet::weight((<T as Config>::WeightInfo::approve_committee_proposal(), Operational))]
        pub fn approve_committee_proposal(origin: OriginFor<T>, id: PipId) -> DispatchResult {
            // Ensure origin is GC.
            T::VotingMajorityOrigin::ensure_origin(origin)?;

            // Ensure proposal is pending.
            Self::is_proposal_state(id, ProposalState::Pending)?;

            // Ensure proposal is by committee.
            let pip = Proposals::<T>::get(id).ok_or(Error::<T>::NoSuchProposal)?;
            ensure!(
                matches!(pip.proposer, Proposer::Committee(_)),
                Error::<T>::NotByCommittee
            );

            // All is good, schedule PIP for execution.
            Self::schedule_pip_for_execution(id)?;
            Ok(())
        }

        /// Rejects the PIP given by the `id`. Bonded funds will be refunded, assuming it hasn't
        /// been cancelled or executed.
        ///
        /// This function can only be called by a Governance Committee (GC) voting majority.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be a GC voting majority.
        /// * `id` - The proposal ID of the PIP to be rejected.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by a GC voting majority.
        /// * `NoSuchProposal` - If the PIP with the given `id` does not exist.
        /// * `IncorrectProposalState` - If the proposal was cancelled or executed.
        ///
        /// # Notes
        /// This function will unschedule the PIP if it was scheduled for execution and will
        /// unsnapshot the PIP if it was part of a snapshot. It will also handle the rejection
        /// of the proposal and refund any bonded funds.
        #[pallet::call_index(9)]
        #[pallet::weight((<T as Config>::WeightInfo::reject_proposal(), Operational))]
        pub fn reject_proposal(origin: OriginFor<T>, id: PipId) -> DispatchResult {
            T::VotingMajorityOrigin::ensure_origin(origin)?;
            let proposal_state = ProposalStates::<T>::get(id).ok_or(Error::<T>::NoSuchProposal)?;
            ensure!(
                Self::is_active(proposal_state),
                Error::<T>::IncorrectProposalState
            );
            Self::maybe_unschedule_pip(id, proposal_state)?;
            Self::maybe_unsnapshot_pip(id, proposal_state);
            Self::unsafe_reject_proposal(GC_DID, id);
            Ok(())
        }

        /// Prunes the PIP given by the `id`. The PIP must not be active.
        ///
        /// This function is intended for storage garbage collection purposes and can only be called by a Governance Committee (GC) voting majority.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be a GC voting majority.
        /// * `id` - The proposal ID of the PIP to be pruned.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by a GC voting majority.
        /// * `NoSuchProposal` - If the PIP with the given `id` does not exist.
        /// * `IncorrectProposalState` - If the proposal is active.
        ///
        /// # Notes
        /// This function will remove the PIP from storage and refund any remaining bonded funds.
        #[pallet::call_index(10)]
        #[pallet::weight((<T as Config>::WeightInfo::prune_proposal(), Operational))]
        pub fn prune_proposal(origin: OriginFor<T>, id: PipId) -> DispatchResult {
            T::VotingMajorityOrigin::ensure_origin(origin)?;
            let proposal_state = ProposalStates::<T>::get(id).ok_or(Error::<T>::NoSuchProposal)?;
            ensure!(
                !Self::is_active(proposal_state),
                Error::<T>::IncorrectProposalState
            );
            Self::prune_data(GC_DID, id, proposal_state, true);
            Ok(())
        }

        /// Updates the execution schedule of the PIP given by `id`.
        /// This function can only be called by the release coordinator.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be the release coordinator.
        /// * `id` - The proposal ID of the PIP to be rescheduled.
        /// * `until` - An optional future block number where the enactment period will finish.
        ///    If `None`, the enactment period will finish in the next block.
        ///
        /// # Errors
        /// * `RescheduleNotByReleaseCoordinator` - If the call is not made by the release coordinator.
        /// * `IncorrectProposalState` - If the proposal is not in a scheduled state.
        /// * `InvalidFutureBlockNumber` - If the provided block number is not a valid future block number.
        #[pallet::call_index(11)]
        #[pallet::weight((<T as Config>::WeightInfo::reschedule_execution(), Operational))]
        pub fn reschedule_execution(
            origin: OriginFor<T>,
            id: PipId,
            until: Option<BlockNumberFor<T>>,
        ) -> DispatchResult {
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;

            // Ensure origin is release coordinator.
            ensure!(
                Some(did) == T::GovernanceCommittee::release_coordinator(),
                Error::<T>::RescheduleNotByReleaseCoordinator
            );

            // Ensure proposal is scheduled.
            Self::is_proposal_state(id, ProposalState::Scheduled)?;

            // Ensure new `until` is a valid block number.
            let next_block = System::<T>::block_number() + 1u32.into();
            let new_until = until.unwrap_or(next_block);
            ensure!(
                new_until >= next_block,
                Error::<T>::InvalidFutureBlockNumber
            );

            // Update enactment period & reschedule it.
            PipToSchedule::<T>::insert(id, new_until);
            let task_name = id.execution_name();
            let res = T::Scheduler::reschedule_named(task_name, DispatchTime::At(new_until));
            Self::handle_exec_scheduling_result(id, new_until, res);
            Ok(())
        }

        /// Clears the snapshot and emits the event `SnapshotCleared`.
        ///
        /// This function can only be called by a Governance Committee (GC) member.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be a GC member.
        ///
        /// # Events
        /// * `SnapshotCleared` - Emitted when the snapshot is successfully cleared, containing the ID of the cleared snapshot.
        ///
        /// # Errors
        /// * `NotACommitteeMember` - If the call is not made by a GC member.
        #[pallet::call_index(12)]
        #[pallet::weight((<T as Config>::WeightInfo::clear_snapshot(), Operational))]
        pub fn clear_snapshot(origin: OriginFor<T>) -> DispatchResult {
            // 1. Check that a GC member is executing this.
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
            ensure!(
                T::GovernanceCommittee::is_member(&did),
                Error::<T>::NotACommitteeMember
            );

            if let Some(meta) = <SnapshotMeta<T>>::get() {
                // 2. Clear the snapshot.
                SnapshotMeta::<T>::kill();
                SnapshotQueue::<T>::kill();

                // 3. Emit event.
                Self::deposit_event(Event::SnapshotCleared(did, meta.id));
            }

            Ok(())
        }

        /// Takes a new snapshot of the current list of active and pending PIPs.
        /// The PIPs are then sorted into a priority queue based on each PIP's weight.
        ///
        /// This function can only be called by a Governance Committee (GC) member.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be a GC member.
        /// * `limit` - Weight witness for the expected queue length (typically the active PIP limit). The actual weight is refunded based on the real queue size.
        ///
        /// # Events
        /// * `SnapshotTaken` - Emitted when a snapshot is successfully taken, containing the ID of the snapshot and the queue of PIPs.
        ///
        /// # Errors
        /// * `NotACommitteeMember` - If the call is not made by a GC member.
        #[pallet::call_index(13)]
        #[pallet::weight((<T as Config>::WeightInfo::snapshot(
            *limit
        ), Operational))]
        pub fn snapshot(origin: OriginFor<T>, limit: u32) -> DispatchResultWithPostInfo {
            // Ensure a GC member is executing this.
            let PermissionedCallOriginData {
                sender: made_by,
                primary_did: did,
                ..
            } = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
            ensure!(
                T::GovernanceCommittee::is_member(&did),
                Error::<T>::NotACommitteeMember
            );

            // Commit the new snapshot.
            let id = SnapshotIdSequence::<T>::try_mutate(try_next_post::<T, _>)?;
            let created_at = System::<T>::block_number();
            SnapshotMeta::<T>::set(Some(SnapshotMetadata {
                created_at,
                made_by,
                id,
            }));
            let queue = LiveQueue::<T>::get();
            let queue_len = queue.len() as u32;
            ensure!(limit >= queue_len, Error::<T>::SnapshotLimitTooSmall);
            SnapshotQueue::<T>::set(queue.clone());

            // Emit event.
            Self::deposit_event(Event::SnapshotTaken(did, id, queue));

            Ok(Some(<T as Config>::WeightInfo::snapshot(queue_len)).into())
        }

        /// Enacts the results for the PIPs in the snapshot queue.
        ///
        /// The snapshot will be available for further enactments until it is cleared.
        ///
        /// The `results` parameter is a list of `(id, result)` tuples where `result` is applied to the PIP with the given `id`.
        /// Note that the snapshot priority queue is encoded with the *lowest priority first*.
        /// For example, `results = [(id, Approve)]` will approve `SnapshotQueue[SnapshotQueue.len() - 1]`.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be a GC voting majority.
        /// * `results` - A vector of tuples where each tuple contains a PIP ID and a `SnapshotResult` (either `Approve`, `Reject`, or `Skip`).
        ///
        /// # Events
        /// * `SnapshotResultsEnacted` - Emitted when the snapshot results are successfully enacted, containing the ID of the snapshot and the actions taken.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by a GC voting majority.
        /// * `CannotSkipPip` - If a given PIP has already been skipped too many times.
        /// * `SnapshotResultTooLarge` - If the length of `results` is greater than the length of the snapshot queue.
        /// * `SnapshotIdMismatch` - If there is a mismatch between the PIP IDs in `results` and the snapshot queue.
        ///
        /// # Notes
        /// This function will:
        /// - Update the skip counts for PIPs that are skipped.
        /// - Reject PIPs that are marked for rejection and refund any bonded funds.
        /// - Approve PIPs that are marked for approval and schedule them for execution.
        #[pallet::call_index(14)]
        #[pallet::weight((enact_snapshot_results_weight::<T>(results), Operational))]
        pub fn enact_snapshot_results(
            origin: OriginFor<T>,
            results: Vec<(PipId, SnapshotResult)>,
        ) -> DispatchResult {
            T::VotingMajorityOrigin::ensure_origin(origin)?;

            let max_pip_skip_count = MaxPipSkipCount::<T>::get();

            SnapshotQueue::<T>::try_mutate(|queue| {
                let mut to_bump_skipped = Vec::new();
                // Default after-first-push capacity is 4, we bump this slightly.
                // Rationale: GC are humans sitting together and reaching conensus.
                // This is time consuming, so considering 20 PIPs in total might take few hours.
                let speculative_capacity = queue.len().min(results.len()).min(10);
                let mut to_reject = Vec::with_capacity(speculative_capacity);
                let mut to_approve = Vec::with_capacity(speculative_capacity);

                // Go over each result...
                for (id, action) in results.iter().copied() {
                    match queue.pop() {
                        // ...and "zip" with the queue in reverse.
                        // An action is missing a corresponding PIP in the queue, bail!
                        None => {
                            return Err(DispatchError::from(Error::<T>::SnapshotResultTooLarge))
                        }
                        // The id at queue position vs. results mismatches.
                        Some(p) if p.id != id => {
                            return Err(DispatchError::from(Error::<T>::SnapshotIdMismatch))
                        }
                        // All is right...
                        Some(_) => {}
                    }
                    match action {
                        // Make sure the PIP can be skipped and enqueue bumping of skip.
                        SnapshotResult::Skip => {
                            let count = PipSkipCount::<T>::get(id);
                            ensure!(count < max_pip_skip_count, Error::<T>::CannotSkipPip);
                            to_bump_skipped.push((id, count + 1));
                        }
                        // Mark PIP as rejected.
                        SnapshotResult::Reject => to_reject.push(id),
                        // Approve PIP.
                        SnapshotResult::Approve => to_approve.push(id),
                    }
                }

                // Update skip counts.
                for (pip_id, new_count) in to_bump_skipped.iter().copied() {
                    PipSkipCount::<T>::insert(pip_id, new_count);
                    Self::deposit_event(Event::PipSkipped(GC_DID, pip_id, new_count));
                }

                // Adjust the live queue, removing scheduled and rejected PIPs.
                LiveQueue::<T>::mutate(|live| {
                    live.retain(|e| !(to_reject.contains(&e.id) || to_approve.contains(&e.id)));
                });

                // Reject proposals as instructed & refund.
                for pip_id in to_reject.iter().copied() {
                    Self::unsafe_reject_proposal(GC_DID, pip_id);
                }

                // Approve proposals as instructed.
                for pip_id in to_approve.iter().copied() {
                    Self::schedule_pip_for_execution(pip_id)?;
                }

                let id = SnapshotMeta::<T>::get().map(|m| m.id);
                let event = Event::SnapshotResultsEnacted(
                    GC_DID,
                    id,
                    to_bump_skipped,
                    to_reject,
                    to_approve,
                );
                Self::deposit_event(event);

                Ok(())
            })?;

            Ok(())
        }

        /// Executes a scheduled PIP (Polymesh Improvement Proposal).
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be the root.
        /// * `id` - The unique identifier of the PIP to be executed.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by the root origin.
        ///
        /// # Notes
        /// This function will:
        /// - Remove the PIP from the scheduling queue.
        /// - Execute the proposal associated with the PIP.
        #[pallet::call_index(15)]
        #[pallet::weight((<T as Config>::WeightInfo::execute_scheduled_pip(), Operational))]
        pub fn execute_scheduled_pip(
            origin: OriginFor<T>,
            id: PipId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            PipToSchedule::<T>::remove(id);
            Self::execute_proposal(id)
        }

        /// Expires a scheduled PIP (Polymesh Improvement Proposal).
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, which must be the root.
        /// * `did` - The identity ID of the entity initiating the expiration.
        /// * `id` - The unique identifier of the PIP to be expired.
        ///
        /// # Errors
        /// * `BadOrigin` - If the call is not made by the root origin.
        ///
        /// # Notes
        /// This function will:
        /// - Check if the PIP is in a pending state.
        /// - Unsnapshot the PIP if it was part of a snapshot.
        /// - Prune the PIP data if it is in an expired state.
        #[pallet::call_index(16)]
        #[pallet::weight((<T as Config>::WeightInfo::expire_scheduled_pip(), Operational))]
        pub fn expire_scheduled_pip(
            origin: OriginFor<T>,
            did: IdentityId,
            id: PipId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            if Self::is_proposal_state(id, ProposalState::Pending).is_ok() {
                Self::maybe_unsnapshot_pip(id, ProposalState::Pending);
                Self::maybe_prune(did, id, ProposalState::Expired);
            }

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Ensure that `origin` represents one of:
    /// - a signed extrinsic (i.e. transaction), and infer the account id, as a community proposer.
    ///   In this case, permissions are also checked
    /// - a committee, where the committee is also inferred.
    ///
    /// Returns the inferred proposer and its DID.
    ///
    /// # Errors
    /// * `BadOrigin` if not a signed extrinsic.
    fn ensure_infer_proposer(
        origin: T::RuntimeOrigin,
    ) -> Result<(Proposer<T::AccountId>, IdentityId), DispatchError> {
        match ensure_signed(origin.clone()) {
            Ok(sender) => {
                let did =
                    pallet_permissions::Pallet::<T>::ensure_call_permissions(&sender)?.primary_did;
                Ok((Proposer::Community(sender), did))
            }
            Err(_) => {
                let (proposer, did) = T::TechnicalCommitteeVMO::ensure_origin(origin.clone())
                    .map(|_| (Committee::Technical, TECHNICAL_DID))
                    .or_else(|_| {
                        T::UpgradeCommitteeVMO::ensure_origin(origin)
                            .map(|_| (Committee::Upgrade, UPGRADE_DID))
                    })
                    .map(|(committee, did)| (Proposer::Committee(committee), did))?;
                Ok((proposer, did))
            }
        }
    }

    /// Increase `acc`'s locked deposit for all PIPs by `amount`,
    /// or fail if there's not enough free balance after adding `amount` to lock.
    fn increase_lock(acc: &T::AccountId, amount: Balance) -> DispatchResult {
        <T as Config>::Currency::increase_lock(
            PIPS_LOCK_ID,
            acc,
            amount,
            WithdrawReasons::all(),
            |sum| {
                <T as Config>::Currency::free_balance(acc)
                    .checked_sub(sum)
                    .ok_or_else(|| Error::<T>::InsufficientDeposit.into())
                    .map(drop)
            },
        )
    }

    /// Returns a reportable representation of a proposal,
    /// taking care that the reported data isn't too large.
    fn reportable_proposal_data(proposal: &T::Proposal) -> ProposalData {
        let encoded_proposal = proposal.encode();
        if encoded_proposal.len() > PIP_MAX_REPORTING_SIZE {
            ProposalData::Hash(BlakeTwo256::hash(encoded_proposal.as_slice()))
        } else {
            ProposalData::Proposal(encoded_proposal)
        }
    }

    /// Adds a PIP expiry call to the PIP expiry schedule.
    fn schedule_pip_for_expiry(id: PipId, at: BlockNumberFor<T>) -> DispatchResult {
        let did = GC_DID;

        let scheduler_call =
            <T as pallet::Config>::SchedulerCall::from(Call::<T>::expire_scheduled_pip { did, id });

        let expire_pip_call = <T as pallet::Config>::SchedulerPreimage::bound(scheduler_call)?;

        match T::Scheduler::schedule_named(
            id.expiry_name(),
            DispatchTime::At(at),
            None,
            MAX_NORMAL_PRIORITY,
            RawOrigin::Root.into(),
            expire_pip_call,
        ) {
            Err(_) => Self::deposit_event(Event::ExpirySchedulingFailed(did, id, at)),
            Ok(_) => Self::deposit_event(Event::ExpiryScheduled(did, id, at)),
        };

        Ok(())
    }

    /// Changes the vote of `voter` to `vote`, if any.
    fn unsafe_vote(id: PipId, voter: T::AccountId, vote: Vote) -> DispatchResult {
        let mut stats = ProposalResult::<T>::get(id);

        // Update the vote and get the old one, if any, in which case also remove it from stats.
        if let Some(Vote(direction, deposit)) = ProposalVotes::<T>::get(id, voter.clone()) {
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
            .ok_or(Error::<T>::NumberOfVotesExceeded)?;
        *stake = stake
            .checked_add(deposit)
            .ok_or(Error::<T>::StakeAmountOfVotesExceeded)?;

        // Commit all changes.
        ProposalResult::<T>::insert(id, stats);
        ProposalVotes::<T>::insert(id, voter, vote);

        Ok(())
    }

    /// Insert a new PIP into the live queue. Returns [`Error::<T>::InvalidPIPID`] if `pip_id` is not unique.
    fn insert_live_queue(pip_id: PipId) -> DispatchResult {
        let new_snapshotted_pip = Self::aggregate_result(pip_id);

        LiveQueue::<T>::try_mutate(|live_queue| -> DispatchResult {
            match live_queue.binary_search_by(|res| compare_spip(res, &new_snapshotted_pip)) {
                Ok(_) => {
                    // This is a bug, as the PIP ID should be unique
                    Err(Error::<T>::InvalidPipId.into())
                }
                Err(pos) => {
                    // PIP ID is new. Insert it into the queue
                    live_queue.insert(pos, new_snapshotted_pip);
                    Ok(())
                }
            }
        })?;

        Ok(())
    }

    /// Construct a `SnapshottedPip` from a `PipId`.
    /// `true` denotes a positive sign.
    fn aggregate_result(id: PipId) -> SnapshottedPip {
        let VotingResult {
            ayes_stake,
            nays_stake,
            ..
        } = ProposalResult::<T>::get(id);
        let weight = if ayes_stake >= nays_stake {
            (true, ayes_stake - nays_stake)
        } else {
            (false, nays_stake - ayes_stake)
        };
        SnapshottedPip { id, weight }
    }

    /// Returns `Ok` if the proposal's state identified `pip_id` equals `state`.
    fn is_proposal_state(pip_id: PipId, state: ProposalState) -> DispatchResult {
        let proposal_state = ProposalStates::<T>::get(pip_id).ok_or(Error::<T>::NoSuchProposal)?;
        ensure!(proposal_state == state, Error::<T>::IncorrectProposalState);
        Ok(())
    }

    /// Reduce `acc`'s locked deposit for all PIPs by `amount`,
    /// or fail if `amount` hasn't been locked for PIPs.
    fn reduce_lock(acc: &T::AccountId, amount: Balance) -> DispatchResult {
        <T as Config>::Currency::reduce_lock(PIPS_LOCK_ID, acc, amount)
    }

    /// Adjust the live queue under the assumption that `id` should be moved up or down the queue.
    fn adjust_live_queue(id: PipId, old: SnapshottedPip) {
        let new = Self::aggregate_result(id);
        LiveQueue::<T>::mutate(|queue| {
            // Remove the old element.
            //
            // Under normal conditions, we can assume its in the list and findable,
            // as the list is sorted, updated, and old is taken before modification.
            // However, we still prefer to be defensive here, and same below.
            if let Ok(old_pos) = queue.binary_search_by(|res| compare_spip(res, &old)) {
                queue.remove(old_pos);
            }

            // Insert the new element.
            if let Err(new_pos) = queue.binary_search_by(|res| compare_spip(res, &new)) {
                queue.insert(new_pos, new);
            }
        });
    }

    /// Add a PIP execution call to the PIP execution schedule.
    fn schedule_pip_for_execution(id: PipId) -> DispatchResult {
        // The enactment period is at least 1 block,
        // as you can only schedule calls for future blocks.
        let at = DefaultEnactmentPeriod::<T>::get()
            .max(One::one())
            .saturating_add(System::<T>::block_number());

        let scheduler_call =
            <T as pallet::Config>::SchedulerCall::from(Call::<T>::execute_scheduled_pip { id });

        let execute_pip_call = <T as pallet::Config>::SchedulerPreimage::bound(scheduler_call)?;

        let task_name = id.execution_name();

        let res = T::Scheduler::schedule_named(
            task_name,
            DispatchTime::At(at),
            None,
            MAX_NORMAL_PRIORITY,
            RawOrigin::Root.into(),
            execute_pip_call,
        );
        Self::handle_exec_scheduling_result(id, at, res);

        // Record that it has been scheduled.
        PipToSchedule::<T>::insert(id, at);

        // Set the proposal to scheduled.
        Self::update_proposal_state(GC_DID, id, ProposalState::Scheduled);

        Ok(())
    }

    /// Emit event based on a `result` from scheduling a PIP for execution.
    fn handle_exec_scheduling_result<A, B>(id: PipId, at: BlockNumberFor<T>, result: Result<A, B>) {
        Self::deposit_event(match result {
            Err(_) => Event::ExecutionSchedulingFailed(GC_DID, id, at),
            Ok(_) => Event::ExecutionScheduled(GC_DID, id, at),
        });
    }

    /// Update the proposal state of `did` setting it to `new_state`.
    fn update_proposal_state(
        did: IdentityId,
        id: PipId,
        new_state: ProposalState,
    ) -> ProposalState {
        ProposalStates::<T>::mutate(id, |proposal_state| {
            if let Some(ref mut proposal_state) = proposal_state {
                // Decrement active count, if the `new_state` is not active.
                if !Self::is_active(new_state) {
                    Self::decrement_count_if_active(*proposal_state);
                }
                *proposal_state = new_state;
            }
        });
        Self::deposit_event(Event::ProposalStateUpdated(did, id, new_state));
        new_state
    }

    /// Returns `true` if `state` is `Pending | Scheduled`.
    fn is_active(state: ProposalState) -> bool {
        matches!(state, ProposalState::Pending | ProposalState::Scheduled)
    }

    /// Decrement active proposal count if `state` signifies it is active.
    fn decrement_count_if_active(state: ProposalState) {
        if Self::is_active(state) {
            // The performance impact of a saturating sub is negligible and caution is good.
            ActivePipCount::<T>::mutate(|count| *count = count.saturating_sub(1));
        }
    }

    /// Unschedule PIP with given `id` if it's scheduled for execution.
    fn maybe_unschedule_pip(id: PipId, state: ProposalState) -> DispatchResult {
        if let ProposalState::Scheduled = state {
            Self::unschedule_pip(id)?;
        }
        Ok(())
    }

    /// Remove the PIP with `id` from the snapshot if it is there.
    fn maybe_unsnapshot_pip(id: PipId, state: ProposalState) {
        if let ProposalState::Pending = state {
            // Pending so therefore in live queue; evict `id`.
            LiveQueue::<T>::mutate(|queue| queue.retain(|i| i.id != id));

            if SnapshotMeta::<T>::get().is_some() {
                // Proposal is pending and wasn't when snapshot was made.
                // Hence, it is in the snapshot and filtering it out will have an effect.
                // Note: These checks are not strictly necessary, but are done to avoid work.
                SnapshotQueue::<T>::mutate(|queue| queue.retain(|i| i.id != id));
            }
        }
    }

    /// Sets the proposal state to [`ProposalState::Rejected`], adds the proposal to the pending
    /// refunds queue and prunes it if [`PruneHistoricalPips`] is set to true.
    fn unsafe_reject_proposal(did: IdentityId, pip_id: PipId) {
        Self::maybe_prune(did, pip_id, ProposalState::Rejected);
    }

    /// Remove the PIP with `id` from the `ExecutionSchedule` at `block_no`.
    fn unschedule_pip(id: PipId) -> DispatchResult {
        let task_name = id.execution_name();

        PipToSchedule::<T>::remove(id);
        if T::Scheduler::cancel_named(task_name).is_err() {
            Self::deposit_event(Event::ExecutionCancellingFailed(id));
        }
        Ok(())
    }

    /// Sets the proposal state to `new_state`, adds the proposal to the pending refunds queue and
    /// prunes it if [`PruneHistoricalPips`] is set to true.
    fn maybe_prune(did: IdentityId, pip_id: PipId, new_state: ProposalState) {
        Self::update_proposal_state(did, pip_id, new_state);
        Self::prune_data(did, pip_id, new_state, PruneHistoricalPips::<T>::get());
    }

    /// This function performs the following steps:
    /// 1. Adds the proposal to the refund pending queue.
    /// 2. Decrements the active proposal count if the proposal's state is active.
    /// 3. If `prune` is true, removes various data associated with the proposal from storage.
    ///
    /// The data removed includes:
    /// - Proposal results
    /// - Votes associated with the proposal
    /// - Proposal metadata
    /// - Committee PIPs if the proposal was made by a committee
    /// - The proposal itself
    /// - PIP skip count
    /// - Proposal states
    ///
    /// For efficiency, some data (e.g., related to execution schedules) is not removed in this function,
    /// but is removed in functions that execute this one.
    fn prune_data(did: IdentityId, pip_id: PipId, state: ProposalState, prune: bool) {
        Self::add_pip_to_pending_queue(pip_id, prune);
        Self::decrement_count_if_active(state);

        if prune {
            ProposalResult::<T>::remove(pip_id);
            ProposalMetadata::<T>::remove(pip_id);
            if let Some(Proposer::Committee(_)) = Proposals::<T>::get(pip_id).map(|p| p.proposer) {
                CommitteePips::<T>::mutate(|list| list.retain(|&i| i != pip_id));
            }
            Proposals::<T>::remove(pip_id);
            PipSkipCount::<T>::remove(pip_id);
            ProposalStates::<T>::remove(pip_id);
        }

        Self::deposit_event(Event::PipClosed(did, pip_id, prune));
    }

    /// Adds `pip_id` to the refund pending queue. If `prune` is true, also adds it to the votes to pruned queue.
    fn add_pip_to_pending_queue(pip_id: PipId, prune: bool) {
        PendingRefunds::<T>::insert(pip_id, true);
        if prune {
            VotesToBePruned::<T>::insert(pip_id, true);
        }
    }

    /// Execute the PIP given by `id`.
    /// Returns an error if the PIP doesn't exist or is not scheduled.
    fn execute_proposal(id: PipId) -> DispatchResultWithPostInfo {
        let proposal = Proposals::<T>::get(id).ok_or(Error::<T>::ScheduledProposalDoesntExist)?;
        let proposal_state =
            ProposalStates::<T>::get(id).ok_or(Error::<T>::ScheduledProposalDoesntExist)?;
        ensure!(
            proposal_state == ProposalState::Scheduled,
            Error::<T>::ProposalNotInScheduledState
        );
        let res = proposal
            .proposal
            .dispatch(frame_system::RawOrigin::Root.into());
        let weight = res.unwrap_or_else(|e| e.post_info).actual_weight;
        let new_state = res.map_or(ProposalState::Failed, |_| ProposalState::Executed);
        Self::maybe_prune(GC_DID, id, new_state);
        let weight = weight.map(|v| v.max(<T as Config>::WeightInfo::execute_scheduled_pip()));
        Ok(Some(weight.unwrap_or(<T as Config>::WeightInfo::execute_scheduled_pip())).into())
    }

    /// Retrieve votes for a proposal represented by PipId `id`.
    pub fn get_votes(id: PipId) -> VoteCount
    where
        T: Send + Sync,
    {
        if !ProposalResult::<T>::contains_key(id) {
            return VoteCount::ProposalNotFound;
        }

        let voting = ProposalResult::<T>::get(id);
        VoteCount::ProposalFound {
            ayes: voting.ayes_stake,
            nays: voting.nays_stake,
        }
    }

    /// Retrieve proposals `address` voted on
    pub fn voted_on(address: T::AccountId) -> Vec<PipId> {
        Proposals::<T>::iter()
            .filter_map(|(_, pip)| ProposalVotes::<T>::get(pip.id, &address).map(|_| pip.id))
            .collect::<Vec<_>>()
    }

    /// Retrieve proposals made by `proposer`.
    pub fn proposed_by(proposer: Proposer<T::AccountId>) -> Vec<PipId> {
        Proposals::<T>::iter()
            .filter(|(_, pip)| pip.proposer == proposer)
            .map(|(_, pip)| pip.id)
            .collect()
    }

    /// Clears from storage any pending refunds and votes.
    pub fn remove_pending_storage() -> Weight {
        let clear_max = T::MaxRefundsAndVotesPruned::get();

        let mut n_refunds: u32 = 0;
        // Checks if there are any pending refunds to be processed
        if let Some(pip_id) = PendingRefunds::<T>::iter_keys().next() {
            let mut refunded_amount = 0;
            for (_, deposit_info) in Deposits::<T>::drain_prefix(pip_id).take(clear_max as usize) {
                n_refunds += 1;
                match Self::reduce_lock(&deposit_info.owner, deposit_info.amount) {
                    Ok(_) => {
                        refunded_amount = refunded_amount.saturating_add(deposit_info.amount);
                    }
                    Err(_) => {
                        log::error!(
                            "Failed to refund deposit for PIP: {:?} Owner: {:?} Amount: {}",
                            pip_id,
                            deposit_info.owner,
                            deposit_info.amount
                        );
                    }
                }
            }

            // Checks if all deposits have been refunded. If so, removes the PIP from the queue.
            if Deposits::<T>::iter_prefix(pip_id).next().is_none() {
                PendingRefunds::<T>::remove(pip_id);
            }

            Self::deposit_event(Event::ProposalRefund(GC_DID, pip_id, refunded_amount));
        }

        // Checks if there are any votes to be pruned
        let mut n_pruned_votes: u32 = 0;
        if let Some(pip_id) = VotesToBePruned::<T>::iter_keys().next() {
            n_pruned_votes = ProposalVotes::<T>::drain_prefix(pip_id)
                .take(clear_max as usize)
                .count() as u32;

            // Checks if all votes have been pruned. If so, removes the PIP from the queue.
            if ProposalVotes::<T>::iter_prefix(pip_id).next().is_none() {
                VotesToBePruned::<T>::remove(pip_id);
            }
        }

        <T as Config>::WeightInfo::remove_pending_storage(n_refunds, n_pruned_votes)
    }
}

/// Returns the `Weight` based on the number of approves, rejects, and skips from `results`.
/// The `enact_snapshot_results` is always a `DispatchClass::Operational` transaction.
pub fn enact_snapshot_results_weight<T: Config>(results: &[(PipId, SnapshotResult)]) -> Weight {
    let mut approves = 0;
    let mut rejects = 0;
    let mut skips = 0;
    for r in results.iter().map(|result| result.1) {
        match r {
            SnapshotResult::Approve => approves += 1,
            SnapshotResult::Reject => rejects += 1,
            SnapshotResult::Skip => skips += 1,
        }
    }

    <T as Config>::WeightInfo::enact_snapshot_results(approves, rejects, skips)
        .min(PipsEnactSnapshotMaximumWeight::get())
}
