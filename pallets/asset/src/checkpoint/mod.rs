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

//! # Checkpoint Module
//!
//! The Checkpoint module provides extrinsics and storage to take snapshots,
//! henceforth called *checkpoints*, of the supply of assets,
//! and how they were distributed at the time of checkpoint.
//!
//! Using the module, users can also schedule checkpoints in the future,
//! either at fixed points in time (e.g., "next friday"),
//! or at regular intervals (e.g., "every month").
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `create_checkpoint` creates a checkpoint.
//! - `set_schedules_max_complexity` sets the max total complexity of a ticker's schedule set.
//! - `create_schedule` creates a checkpoint schedule.
//! - `remove_schedule` removes a checkpoint schedule.
//!
//! ### Public Functions
//!
//! - `balance_at(ticker, did, cp)` returns the balance of `did` for `ticker` at least `>= cp`.
//! - `advance_update_balances(ticker, updates)` advances schedules for `ticker`
//!    and applies new balances in `updates` for the last checkpoint.
//! - Other misc storage items as defined in `decl_storage!`.

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult, Weight},
    ensure,
    traits::UnixTime,
};
use frame_system::ensure_root;
use sp_runtime::traits::SaturatedConversion;
use sp_std::prelude::*;
use sp_std::vec;

use pallet_base::try_next_pre;
pub use polymesh_common_utilities::traits::checkpoint::{Event, WeightInfo};
use polymesh_common_utilities::traits::checkpoint::{
    NextCheckpoints, ScheduleCheckpoints, ScheduleId,
};
use polymesh_common_utilities::{
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    GC_DID,
};
use polymesh_primitives::{
    asset::CheckpointId, storage_migrate_on, storage_migration_ver, IdentityId, Moment, Ticker,
};

use crate::Config;

type Asset<T> = crate::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;

storage_migration_ver!(1);

decl_storage! {
    trait Store for Module<T: Config> as Checkpoint {
        // --------------------- Supply / Balance storage ----------------------

        /// Total supply of the token at the checkpoint.
        ///
        /// (ticker, checkpointId) -> total supply at given checkpoint
        pub TotalSupply get(fn total_supply_at):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) CheckpointId => polymesh_primitives::Balance;

        /// Balance of a DID at a checkpoint.
        ///
        /// (ticker, did, checkpoint ID) -> Balance of a DID at a checkpoint
        pub Balance get(fn balance_at_checkpoint):
            double_map hasher(blake2_128_concat) (Ticker, CheckpointId), hasher(twox_64_concat) IdentityId => polymesh_primitives::Balance;

        // ------------------------ Checkpoint storage -------------------------

        /// Checkpoints ID generator sequence.
        /// ID of first checkpoint is 1 instead of 0.
        ///
        /// (ticker) -> no. of checkpoints
        pub CheckpointIdSequence get(fn checkpoint_id_sequence):
            map hasher(blake2_128_concat) Ticker => CheckpointId;

        /// Checkpoints where a DID's balance was updated.
        /// (ticker, did) -> [checkpoint ID where user balance changed]
        pub BalanceUpdates get(fn balance_updates):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) IdentityId => Vec<CheckpointId>;

        /// Checkpoint timestamps.
        ///
        /// Every schedule-originated checkpoint maps its ID to its due time.
        /// Every checkpoint manually created maps its ID to the time of recording.
        ///
        /// (ticker) -> (checkpoint ID) -> checkpoint timestamp
        pub Timestamps get(fn timestamps):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) CheckpointId => Moment;

        // -------------------- Checkpoint Schedule storage --------------------

        /// The maximum complexity allowed for a ticker's schedules.
        pub SchedulesMaxComplexity get(fn schedules_max_complexity) config(): u64;

        /// Checkpoint schedule ID sequence for tickers.
        ///
        /// (ticker) -> schedule ID
        pub ScheduleIdSequence get(fn schedule_id_sequence):
            map hasher(blake2_128_concat) Ticker => ScheduleId;

        /// Cached next checkpoint for each schedule.
        ///
        /// This is used to quickly find the next checkpoint from a ticker's schedules.
        ///
        /// (ticker) -> next checkpoints
        pub CachedNextCheckpoints get(fn cached_next_checkpoints):
            map hasher(blake2_128_concat) Ticker => Option<NextCheckpoints>;

        /// Scheduled checkpoints.
        ///
        /// (ticker, schedule ID) -> schedule checkpoints
        pub ScheduledCheckpoints get(fn scheduled_checkpoints):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) ScheduleId => Option<ScheduleCheckpoints>;

        /// How many "strong" references are there to a given `ScheduleId`?
        ///
        /// The presence of a "strong" reference, in the sense of `Rc<T>`,
        /// entails that the referenced schedule cannot be removed.
        /// Thus, as long as `strong_ref_count(schedule_id) > 0`,
        /// `remove_schedule(schedule_id)` will error.
        ///
        /// (ticker, schedule ID) -> strong ref count
        pub ScheduleRefCount get(fn schedule_ref_count):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) ScheduleId => u32;

        /// All the checkpoints a given schedule originated.
        ///
        /// (ticker, schedule ID) -> [checkpoint ID]
        pub SchedulePoints get(fn schedule_points):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) ScheduleId => Vec<CheckpointId>;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1)): Version;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {
        type Error = Error<T>;

        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            let mut weight = Weight::zero();
            storage_migrate_on!(StorageVersion, 1, {
                migration::migrate_to_v1::<T>(&mut weight);
            });
            weight
        }

        /// Creates a single checkpoint at the current time.
        ///
        /// # Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `ticker`.
        /// - `ticker` to create the checkpoint for.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `ticker`.
        /// - `CounterOverflow` if the total checkpoint counter would overflow.
        #[weight = T::CPWeightInfo::create_checkpoint()]
        pub fn create_checkpoint(origin, ticker: Ticker) {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            Self::create_at_by(caller_did, ticker, Self::now_unix())?;
        }

        /// Sets the max complexity of a schedule set for an arbitrary ticker to `max_complexity`.
        /// The new maximum is not enforced retroactively,
        /// and only applies once new schedules are made.
        ///
        /// Must be called as a PIP (requires "root").
        ///
        /// # Arguments
        /// - `origin` is the root origin.
        /// - `max_complexity` allowed for an arbitrary ticker's schedule set.
        #[weight = T::CPWeightInfo::set_schedules_max_complexity()]
        pub fn set_schedules_max_complexity(origin, max_complexity: u64) {
            ensure_root(origin)?;
            SchedulesMaxComplexity::put(max_complexity);
            Self::deposit_event(Event::MaximumSchedulesComplexityChanged(GC_DID, max_complexity));
        }

        /// Creates a schedule generating checkpoints
        /// in the future at either a fixed time or at intervals.
        ///
        /// The schedule starts out with `strong_ref_count(schedule_id) <- 0`.
        ///
        /// # Arguments
        /// - `origin` is a signer that has permissions to act as owner of `ticker`.
        /// - `ticker` to create the schedule for.
        /// - `schedule` that will generate checkpoints.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `ticker`.
        /// - `InsufficientAccountBalance` if the protocol fee could not be charged.
        /// - `CounterOverflow` if the schedule ID or total checkpoint counters would overflow.
        ///
        /// # Permissions
        /// * Asset
        #[weight = T::CPWeightInfo::create_schedule()]
        pub fn create_schedule(
            origin,
            ticker: Ticker,
            schedule: ScheduleCheckpoints,
        ) -> DispatchResult {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            Self::base_create_schedule(caller_did, ticker, schedule, 0)?;
            Ok(())
        }

        /// Removes the checkpoint schedule of an asset identified by `id`.
        ///
        /// # Arguments
        /// - `origin` is a signer that has permissions to act as owner of `ticker`.
        /// - `ticker` to remove the schedule from.
        /// - `id` of the schedule, when it was created by `created_schedule`.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `ticker`.
        /// - `NoCheckpointSchedule` if `id` does not identify a schedule for this `ticker`.
        /// - `ScheduleNotRemovable` if `id` exists but is not removable.
        ///
        /// # Permissions
        /// * Asset
        #[weight = T::CPWeightInfo::remove_schedule()]
        pub fn remove_schedule(
            origin,
            ticker: Ticker,
            id: ScheduleId,
        ) -> DispatchResult {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            Self::base_remove_schedule(caller_did, ticker, id)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// A checkpoint schedule does not exist for the asset.
        NoSuchSchedule,
        /// A checkpoint schedule is not removable as `ref_count(schedule_id) > 0`.
        ScheduleNotRemovable,
        /// The new schedule would put the ticker over the maximum complexity allowed.
        SchedulesOverMaxComplexity,
        /// Can't create an empty schedule.
        ScheduleIsEmpty,
        /// The schedule has no more checkpoints.
        ScheduleFinished,
        /// The schedule has expired checkpoints.
        ScheduleHasExpiredCheckpoints,
    }
}

impl<T: Config> Module<T> {
    /// Does checkpoint with ID `cp_id` exist for `ticker`?
    pub fn checkpoint_exists(ticker: &Ticker, cp: CheckpointId) -> bool {
        cp > CheckpointId(0) && cp <= CheckpointIdSequence::get(ticker)
    }

    /// Returns the balance of `did` for `ticker` at first checkpoint ID `>= cp`, if any.
    ///
    /// Reasons for returning `None` include:
    /// - `cp` is not a valid checkpoint ID.
    /// - `did` hasn't made transfers in all of `ticker`'s checkpoints.
    /// - `did`'s last transaction was strictly before `cp`, so their balance is the current one.
    ///
    /// N.B. in case of `None`, you likely want the current balance instead.
    /// To compute that, use `Asset::get_balance_at(ticker, did, cp)`, which calls into here.
    pub fn balance_at(
        ticker: Ticker,
        did: IdentityId,
        cp: CheckpointId,
    ) -> Option<polymesh_primitives::Balance> {
        if Self::checkpoint_exists(&ticker, cp) && BalanceUpdates::contains_key(ticker, did) {
            // Checkpoint exists and user has some part in that.
            let balance_updates = BalanceUpdates::get(ticker, did);
            if cp <= balance_updates.last().copied().unwrap_or(CheckpointId(0)) {
                // Use first checkpoint created after target checkpoint.
                // The user has data for that checkpoint.
                let id = *find_ceiling(&balance_updates, &cp);
                return Some(Self::balance_at_checkpoint((ticker, id), did));
            }
            // User has not transacted after checkpoint creation.
            // This means their current balance = their balance at that cp.
        }
        None
    }

    /// Advances checkpoints for `ticker`,
    /// and for each DID in `updates`, sets their balance to the one provided.
    pub fn advance_update_balances(
        ticker: &Ticker,
        updates: &[(IdentityId, polymesh_primitives::Balance)],
    ) -> DispatchResult {
        Self::advance_schedules(ticker)?;
        Self::update_balances(ticker, updates);
        Ok(())
    }

    /// Updates manual and scheduled checkpoints if those are defined.
    ///
    /// # Assumption
    ///
    /// * When minting, the total supply of `ticker` is updated **after** this function is called.
    fn update_balances(ticker: &Ticker, updates: &[(IdentityId, polymesh_primitives::Balance)]) {
        let last_cp = CheckpointIdSequence::get(ticker);
        if last_cp < CheckpointId(1) {
            return;
        }
        for (did, balance) in updates {
            let first_key = (ticker, last_cp);
            if !Balance::contains_key(first_key, did) {
                Balance::insert(first_key, did, balance);
                BalanceUpdates::append(ticker, did, last_cp);
            }
        }
    }

    /// Advance all checkpoint schedules for `ticker`.
    ///
    fn advance_schedules(ticker: &Ticker) -> DispatchResult {
        // Check if there are any pending checkpoints.
        let mut cached = match CachedNextCheckpoints::try_get(ticker) {
            Ok(cached) => cached,
            Err(_) => {
                // No pending checkpoints for this ticker.
                return Ok(());
            }
        };

        let now = Self::now_unix();
        // Check if there are any expired checkpoints.
        if !cached.expired(now) {
            // Haven't reached the `next_at` yet, nothing to do.
            return Ok(());
        }

        let mut cp_id = CheckpointIdSequence::get(ticker);

        // Get the set of schedules that have expired checkpoints.
        let schedule_ids = cached.expired_schedules(now);

        for schedule_id in schedule_ids {
            match ScheduledCheckpoints::try_get(ticker, schedule_id) {
                Ok(mut schedule) => {
                    // Remove expired checkpoints from the schedule.
                    let checkpoints = schedule.remove_expired(now);

                    // Check if the schedule still has pending checkpoints.
                    match schedule.next() {
                        Some(next) => {
                            // Update cached `next` for this schedule.
                            cached.add_schedule_next(schedule_id, next);
                            // Update the pending checkpoints for this schedule.
                            ScheduledCheckpoints::insert(ticker, schedule_id, schedule);
                        }
                        None => {
                            // Schedule is finished, no more checkpoints.
                            ScheduledCheckpoints::remove(ticker, schedule_id);
                        }
                    }

                    // Update the total_pending count.
                    cached.dec_total_pending(checkpoints.len() as u64);
                    // Create the scheduled checkpoints.
                    for at in checkpoints {
                        let id = try_next_pre::<T, _>(&mut cp_id)?;
                        Self::create_at(None, *ticker, id, at);
                        SchedulePoints::append(ticker, schedule_id, id);
                    }
                }
                _ => (),
            }
        }

        // Save updated cache.
        if cached.is_empty() {
            // No more scheduled checkpoints, remove the cache.
            CachedNextCheckpoints::remove(ticker);
        } else {
            // Update the cache.
            CachedNextCheckpoints::insert(ticker, cached);
        }

        CheckpointIdSequence::insert(ticker, cp_id);
        Ok(())
    }

    /// Creates a schedule generating checkpoints
    /// in the future at either a fixed time or at intervals.
    pub fn base_create_schedule(
        caller_did: IdentityId,
        ticker: Ticker,
        schedule: ScheduleCheckpoints,
        ref_count: u32,
    ) -> Result<(ScheduleId, Moment), DispatchError> {
        // Ensure the schedule is not empty.
        let next_at = schedule.next().ok_or(Error::<T>::ScheduleIsEmpty)?;
        let len: u64 = schedule
            .len()
            .try_into()
            .map_err(|_| Error::<T>::SchedulesOverMaxComplexity)?;
        let max_comp = SchedulesMaxComplexity::get();
        ensure!(len <= max_comp, Error::<T>::SchedulesOverMaxComplexity);

        let mut cached = CachedNextCheckpoints::get(ticker).unwrap_or_default();
        // Ensure the total complexity for all schedules is not too great.
        let total_pending = cached.total_pending.saturating_add(len);
        ensure!(
            total_pending <= max_comp,
            Error::<T>::SchedulesOverMaxComplexity
        );

        // Ensure the checkpoints are in the future.
        let now = Self::now_unix();
        ensure!(
            schedule.ensure_no_expired(now),
            Error::<T>::ScheduleHasExpiredCheckpoints
        );

        // Compute the next checkpoint schedule ID. Will store it later.
        let id = try_next_pre::<T, _>(&mut ScheduleIdSequence::get(ticker))?;

        // Charge the fee for checkpoint schedule creation.
        // N.B. this operation bundles verification + a storage change.
        // Thus, it must be last, and only storage changes follow.
        T::ProtocolFee::charge_fee(ProtocolOp::CheckpointCreateSchedule)?;

        // Add the new schedule's `next_at` to the cached next checkpoints.
        cached.add_schedule_next(id, next_at);
        cached.inc_total_pending(len);

        CachedNextCheckpoints::insert(ticker, cached);
        ScheduledCheckpoints::insert(ticker, id, &schedule);
        ScheduleRefCount::insert(ticker, id, ref_count);
        ScheduleIdSequence::insert(ticker, id);

        Self::deposit_event(Event::ScheduleCreated(caller_did, ticker, id, schedule));
        Ok((id, next_at))
    }

    pub fn base_remove_schedule(
        caller_did: IdentityId,
        ticker: Ticker,
        id: ScheduleId,
    ) -> DispatchResult {
        // Ensure that the schedule exists.
        let schedule =
            ScheduledCheckpoints::try_get(ticker, id).map_err(|_| Error::<T>::NoSuchSchedule)?;

        // Can only remove the schedule if it doesn't have any references to it.
        ensure!(
            ScheduleRefCount::get(ticker, id) == 0,
            Error::<T>::ScheduleNotRemovable
        );

        // Remove the schedule and any pending checkpoints.
        // We don't remove historical points related to the schedule.
        ScheduleRefCount::remove(ticker, id);

        // Remove the schedule from the cached next checkpoints.
        CachedNextCheckpoints::mutate(&ticker, |cached| {
            if let Some(cached) = cached {
                cached.remove_schedule(id);
                cached.dec_total_pending(schedule.len() as u64);
            }
        });
        // Remove scheduled checkpoints.
        ScheduledCheckpoints::remove(ticker, id);

        // Emit event.
        Self::deposit_event(Event::ScheduleRemoved(caller_did, ticker, id, schedule));
        Ok(())
    }

    /// The `caller_did` creates a checkpoint at `at` for `ticker`.
    /// The ID of the new checkpoint is returned.
    fn create_at_by(
        caller_did: IdentityId,
        ticker: Ticker,
        at: Moment,
    ) -> Result<CheckpointId, DispatchError> {
        let id = try_next_pre::<T, _>(&mut CheckpointIdSequence::get(ticker))?;
        CheckpointIdSequence::insert(ticker, id);
        Self::create_at(Some(caller_did), ticker, id, at);
        Ok(id)
    }

    /// Creates a checkpoint at `at` for `ticker`, with the given, in advanced reserved, `id`.
    /// The `caller_did` is the DID creating the checkpoint,
    /// or `None` scheduling created the checkpoint.
    ///
    /// Creating a checkpoint entails:
    /// - recording the total supply,
    /// - mapping the the ID to the `time`.
    fn create_at(caller_did: Option<IdentityId>, ticker: Ticker, id: CheckpointId, at: Moment) {
        // Record total supply at checkpoint ID.
        let supply = <Asset<T>>::token_details(&ticker)
            .map(|t| t.total_supply)
            .unwrap_or_default();
        TotalSupply::insert(ticker, id, supply);

        // Relate Ticker -> ID -> time.
        Timestamps::insert(ticker, id, at);

        // Emit event & we're done.
        Self::deposit_event(Event::CheckpointCreated(caller_did, ticker, id, supply, at));
    }

    /// Increment the schedule ref count.
    pub fn inc_schedule_ref(ticker: &Ticker, id: ScheduleId) {
        ScheduleRefCount::mutate(ticker, id, |c| *c = c.saturating_add(1));
    }

    /// Decrement the schedule ref count.
    pub fn dec_schedule_ref(ticker: &Ticker, id: ScheduleId) {
        ScheduleRefCount::mutate(ticker, id, |c| *c = c.saturating_sub(1));
    }

    /// Ensure the schedule exists and get the next checkpoint.
    pub fn ensure_schedule_next_checkpoint(
        ticker: &Ticker,
        id: ScheduleId,
    ) -> Result<(Moment, u64), DispatchError> {
        let schedule =
            ScheduledCheckpoints::try_get(ticker, id).map_err(|_| Error::<T>::NoSuchSchedule)?;
        // Get the next checkpoint in the schedule, if it isn't finished.
        let cp_at = schedule.next().ok_or(Error::<T>::ScheduleFinished)?;
        // Get the index for the checkpoint in this schedule.
        let cp_at_idx = SchedulePoints::decode_len(ticker, id).unwrap_or(0) as u64;
        Ok((cp_at, cp_at_idx))
    }

    /// Returns the current UNIX time, i.e. milli-seconds since UNIX epoch, 1970-01-01 00:00:00 UTC.
    pub fn now_unix() -> Moment {
        T::UnixTime::now().as_millis().saturated_into::<Moment>()
    }
}

/// Find the least element `>= key` in `arr`.
///
/// Assumes that key <= last element of the array,
/// the array consists of unique sorted elements,
/// and that array len > 0.
fn find_ceiling<'a, E: Ord>(arr: &'a [E], key: &E) -> &'a E {
    &arr[arr.binary_search(key).map_or_else(|i| i, |i| i)]
}

pub mod migration {
    use super::*;
    use polymesh_runtime_common::RocksDbWeight as DbWeight;
    use sp_runtime::runtime_logger::RuntimeLogger;

    mod v0 {
        use super::*;
        use polymesh_primitives::calendar::CheckpointSchedule;
        use scale_info::TypeInfo;

        // Limit each schedule to a maximum pending checkpoints.
        const MAX_CP: u32 = 10;

        #[derive(Encode, Decode, TypeInfo, Copy, Clone, Debug)]
        pub struct StoredSchedule {
            pub schedule: CheckpointSchedule,
            pub id: ScheduleId,
            pub at: Moment,
            pub remaining: u32,
        }

        impl From<StoredSchedule> for ScheduleCheckpoints {
            fn from(old: StoredSchedule) -> Self {
                let remaining = if old.remaining == 0 {
                    // 0 means infinite.
                    MAX_CP
                } else {
                    // Limit remaining to MAX_CP.
                    old.remaining.min(MAX_CP)
                };
                Self::from_old(old.schedule, remaining)
            }
        }

        decl_storage! {
            trait Store for Module<T: Config> as Checkpoint {
                pub Schedules get(fn schedules):
                    map hasher(blake2_128_concat) Ticker => Vec<StoredSchedule>;
            }
        }

        decl_module! {
            pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
        }
    }

    pub fn migrate_to_v1<T: Config>(weight: &mut Weight) {
        RuntimeLogger::init();
        log::info!(" >>> Updating Checkpoint storage. Migrating old schedules.");
        let mut count = 0;
        let mut reads = 0;
        let mut writes = 0;
        v0::Schedules::drain().for_each(|(ticker, old_schedules)| {
            reads += 1;
            let mut cached = NextCheckpoints::default();
            for old_schedule in old_schedules {
                let id = old_schedule.id;
                let schedule = ScheduleCheckpoints::from(old_schedule);
                if let Some(next_at) = schedule.next() {
                    cached.inc_total_pending(schedule.len() as u64);
                    cached.add_schedule_next(id, next_at);
                    count += 1;
                    writes += 1;
                    ScheduledCheckpoints::insert(ticker, id, schedule);
                }
            }
            if !cached.is_empty() {
                writes += 1;
                CachedNextCheckpoints::insert(ticker, cached);
            }
        });
        weight.saturating_accrue(DbWeight::get().reads_writes(reads, writes));
        log::info!(" >>> {count} checkpoint schedules have been migrated.");
    }
}
