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
//! - `set_schedules_max_complexity` sets the max total complexity of an asset's schedule set.
//! - `create_schedule` creates a checkpoint schedule.
//! - `remove_schedule` removes a checkpoint schedule.
//!
//! ### Public Functions
//!
//! - `balance_at(asset_id, did, cp)` returns the balance of `did` for `asset_id` at least `>= cp`.
//! - `advance_update_balances(asset_id, updates)` advances schedules for `asset_id`
//!    and applies new balances in `updates` for the last checkpoint.
//! - Other misc storage items as defined in `decl_storage!`.

//#[cfg(feature = "runtime-benchmarks")]
//pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
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
use polymesh_primitives::asset::AssetID;
use polymesh_primitives::{asset::CheckpointId, storage_migration_ver, IdentityId, Moment};

use crate::Config;

type Asset<T> = crate::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;

storage_migration_ver!(1);

decl_storage! {
    trait Store for Module<T: Config> as Checkpoint {
        // --------------------- Supply / Balance storage ----------------------

        /// Total supply of the token at the checkpoint.
        ///
        /// ([`AssetID`], checkpointId) -> total supply at given checkpoint
        pub TotalSupply get(fn total_supply_at):
            double_map hasher(blake2_128_concat) AssetID, hasher(twox_64_concat) CheckpointId => polymesh_primitives::Balance;

        /// Balance of a DID at a checkpoint.
        ///
        /// ([`AssetID`], did, checkpoint ID) -> Balance of a DID at a checkpoint
        pub Balance get(fn balance_at_checkpoint):
            double_map hasher(blake2_128_concat) (AssetID, CheckpointId), hasher(twox_64_concat) IdentityId => polymesh_primitives::Balance;

        // ------------------------ Checkpoint storage -------------------------

        /// Checkpoints ID generator sequence.
        /// ID of first checkpoint is 1 instead of 0.
        ///
        /// ([`AssetID`]) -> no. of checkpoints
        pub CheckpointIdSequence get(fn checkpoint_id_sequence):
            map hasher(blake2_128_concat) AssetID => CheckpointId;

        /// Checkpoints where a DID's balance was updated.
        /// ([`AssetID`], did) -> [checkpoint ID where user balance changed]
        pub BalanceUpdates get(fn balance_updates):
            double_map hasher(blake2_128_concat) AssetID, hasher(twox_64_concat) IdentityId => Vec<CheckpointId>;

        /// Checkpoint timestamps.
        ///
        /// Every schedule-originated checkpoint maps its ID to its due time.
        /// Every checkpoint manually created maps its ID to the time of recording.
        ///
        /// ([`AssetID`]) -> (checkpoint ID) -> checkpoint timestamp
        pub Timestamps get(fn timestamps):
            double_map hasher(blake2_128_concat) AssetID, hasher(twox_64_concat) CheckpointId => Moment;

        // -------------------- Checkpoint Schedule storage --------------------

        /// The maximum complexity allowed for an asset's schedules.
        pub SchedulesMaxComplexity get(fn schedules_max_complexity) config(): u64;

        /// Checkpoint schedule ID sequence for assets.
        ///
        /// ([`AssetID`]) -> schedule ID
        pub ScheduleIdSequence get(fn schedule_id_sequence):
            map hasher(blake2_128_concat) AssetID => ScheduleId;

        /// Cached next checkpoint for each schedule.
        ///
        /// This is used to quickly find the next checkpoint from a asset's schedules.
        ///
        /// ([`AssetID`]) -> next checkpoints
        pub CachedNextCheckpoints get(fn cached_next_checkpoints):
            map hasher(blake2_128_concat) AssetID => Option<NextCheckpoints>;

        /// Scheduled checkpoints.
        ///
        /// ([`AssetID`], schedule ID) -> schedule checkpoints
        pub ScheduledCheckpoints get(fn scheduled_checkpoints):
            double_map hasher(blake2_128_concat) AssetID, hasher(twox_64_concat) ScheduleId => Option<ScheduleCheckpoints>;

        /// How many "strong" references are there to a given `ScheduleId`?
        ///
        /// The presence of a "strong" reference, in the sense of `Rc<T>`,
        /// entails that the referenced schedule cannot be removed.
        /// Thus, as long as `strong_ref_count(schedule_id) > 0`,
        /// `remove_schedule(schedule_id)` will error.
        ///
        /// ([`AssetID`], schedule ID) -> strong ref count
        pub ScheduleRefCount get(fn schedule_ref_count):
            double_map hasher(blake2_128_concat) AssetID, hasher(twox_64_concat) ScheduleId => u32;

        /// All the checkpoints a given schedule originated.
        ///
        /// ([`AssetID`], schedule ID) -> [checkpoint ID]
        pub SchedulePoints get(fn schedule_points):
            double_map hasher(blake2_128_concat) AssetID, hasher(twox_64_concat) ScheduleId => Vec<CheckpointId>;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1)): Version;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Creates a single checkpoint at the current time.
        ///
        /// # Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `asset_id`.
        /// - `asset_id` to create the checkpoint for.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `asset_id`.
        /// - `CounterOverflow` if the total checkpoint counter would overflow.
        #[weight = T::CPWeightInfo::create_checkpoint()]
        pub fn create_checkpoint(origin, asset_id: AssetID) {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
            Self::create_at_by(caller_did, asset_id, Self::now_unix())?;
        }

        /// Sets the max complexity of a schedule set for an arbitrary asset_id to `max_complexity`.
        /// The new maximum is not enforced retroactively,
        /// and only applies once new schedules are made.
        ///
        /// Must be called as a PIP (requires "root").
        ///
        /// # Arguments
        /// - `origin` is the root origin.
        /// - `max_complexity` allowed for an arbitrary asset's schedule set.
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
        /// - `origin` is a signer that has permissions to act as owner of `asset_id`.
        /// - `asset_id` to create the schedule for.
        /// - `schedule` that will generate checkpoints.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `asset_id`.
        /// - `InsufficientAccountBalance` if the protocol fee could not be charged.
        /// - `CounterOverflow` if the schedule ID or total checkpoint counters would overflow.
        ///
        /// # Permissions
        /// * Asset
        #[weight = T::CPWeightInfo::create_schedule()]
        pub fn create_schedule(
            origin,
            asset_id: AssetID,
            schedule: ScheduleCheckpoints,
        ) -> DispatchResult {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
            Self::base_create_schedule(caller_did, asset_id, schedule, 0)?;
            Ok(())
        }

        /// Removes the checkpoint schedule of an asset identified by `id`.
        ///
        /// # Arguments
        /// - `origin` is a signer that has permissions to act as owner of `asset_id`.
        /// - `asset_id` to remove the schedule from.
        /// - `id` of the schedule, when it was created by `created_schedule`.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `asset_id`.
        /// - `NoCheckpointSchedule` if `id` does not identify a schedule for this `asset_id`.
        /// - `ScheduleNotRemovable` if `id` exists but is not removable.
        ///
        /// # Permissions
        /// * Asset
        #[weight = T::CPWeightInfo::remove_schedule()]
        pub fn remove_schedule(
            origin,
            asset_id: AssetID,
            id: ScheduleId,
        ) -> DispatchResult {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
            Self::base_remove_schedule(caller_did, asset_id, id)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// A checkpoint schedule does not exist for the asset.
        NoSuchSchedule,
        /// A checkpoint schedule is not removable as `ref_count(schedule_id) > 0`.
        ScheduleNotRemovable,
        /// The new schedule would put the asset over the maximum complexity allowed.
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
    /// Does checkpoint with ID `cp_id` exist for `asset_id`?
    pub fn checkpoint_exists(asset_id: &AssetID, cp: CheckpointId) -> bool {
        cp > CheckpointId(0) && cp <= CheckpointIdSequence::get(asset_id)
    }

    /// Returns the balance of `did` for `asset_id` at first checkpoint ID `>= cp`, if any.
    ///
    /// Reasons for returning `None` include:
    /// - `cp` is not a valid checkpoint ID.
    /// - `did` hasn't made transfers in all of `asset_id` checkpoints.
    /// - `did`'s last transaction was strictly before `cp`, so their balance is the current one.
    ///
    /// N.B. in case of `None`, you likely want the current balance instead.
    /// To compute that, use `Asset::get_balance_at(asset_id, did, cp)`, which calls into here.
    pub fn balance_at(
        asset_id: AssetID,
        did: IdentityId,
        cp: CheckpointId,
    ) -> Option<polymesh_primitives::Balance> {
        if Self::checkpoint_exists(&asset_id, cp) && BalanceUpdates::contains_key(asset_id, did) {
            // Checkpoint exists and user has some part in that.
            let balance_updates = BalanceUpdates::get(asset_id, did);
            if cp <= balance_updates.last().copied().unwrap_or(CheckpointId(0)) {
                // Use first checkpoint created after target checkpoint.
                // The user has data for that checkpoint.
                let id = *find_ceiling(&balance_updates, &cp);
                return Some(Self::balance_at_checkpoint((asset_id, id), did));
            }
            // User has not transacted after checkpoint creation.
            // This means their current balance = their balance at that cp.
        }
        None
    }

    /// Advances checkpoints for `asset_id`,
    /// and for each DID in `updates`, sets their balance to the one provided.
    pub fn advance_update_balances(
        asset_id: &AssetID,
        updates: &[(IdentityId, polymesh_primitives::Balance)],
    ) -> DispatchResult {
        Self::advance_schedules(asset_id)?;
        Self::update_balances(asset_id, updates);
        Ok(())
    }

    /// Updates manual and scheduled checkpoints if those are defined.
    ///
    /// # Assumption
    ///
    /// * When minting, the total supply of `asset_id` is updated **after** this function is called.
    fn update_balances(asset_id: &AssetID, updates: &[(IdentityId, polymesh_primitives::Balance)]) {
        let last_cp = CheckpointIdSequence::get(asset_id);
        if last_cp < CheckpointId(1) {
            return;
        }
        for (did, balance) in updates {
            let first_key = (asset_id, last_cp);
            if !Balance::contains_key(first_key, did) {
                Balance::insert(first_key, did, balance);
                BalanceUpdates::append(asset_id, did, last_cp);
            }
        }
    }

    /// Advance all checkpoint schedules for `asset_id`.
    fn advance_schedules(asset_id: &AssetID) -> DispatchResult {
        // Check if there are any pending checkpoints.
        let mut cached = match CachedNextCheckpoints::try_get(asset_id) {
            Ok(cached) => cached,
            Err(_) => {
                // No pending checkpoints for this asset_id.
                return Ok(());
            }
        };

        let now = Self::now_unix();
        // Check if there are any expired checkpoints.
        if !cached.expired(now) {
            // Haven't reached the `next_at` yet, nothing to do.
            return Ok(());
        }

        let mut cp_id = CheckpointIdSequence::get(asset_id);

        // Get the set of schedules that have expired checkpoints.
        let schedule_ids = cached.expired_schedules(now);

        for schedule_id in schedule_ids {
            match ScheduledCheckpoints::try_get(asset_id, schedule_id) {
                Ok(mut schedule) => {
                    // Remove expired checkpoints from the schedule.
                    let checkpoints = schedule.remove_expired(now);

                    // Check if the schedule still has pending checkpoints.
                    match schedule.next() {
                        Some(next) => {
                            // Update cached `next` for this schedule.
                            cached.add_schedule_next(schedule_id, next);
                            // Update the pending checkpoints for this schedule.
                            ScheduledCheckpoints::insert(asset_id, schedule_id, schedule);
                        }
                        None => {
                            // Schedule is finished, no more checkpoints.
                            ScheduledCheckpoints::remove(asset_id, schedule_id);
                        }
                    }

                    // Update the total_pending count.
                    cached.dec_total_pending(checkpoints.len() as u64);
                    // Create the scheduled checkpoints.
                    for at in checkpoints {
                        let id = try_next_pre::<T, _>(&mut cp_id)?;
                        Self::create_at(None, *asset_id, id, at);
                        SchedulePoints::append(asset_id, schedule_id, id);
                    }
                }
                _ => (),
            }
        }

        // Save updated cache.
        if cached.is_empty() {
            // No more scheduled checkpoints, remove the cache.
            CachedNextCheckpoints::remove(asset_id);
        } else {
            // Update the cache.
            CachedNextCheckpoints::insert(asset_id, cached);
        }

        CheckpointIdSequence::insert(asset_id, cp_id);
        Ok(())
    }

    /// Creates a schedule generating checkpoints
    /// in the future at either a fixed time or at intervals.
    pub fn base_create_schedule(
        caller_did: IdentityId,
        asset_id: AssetID,
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

        let mut cached = CachedNextCheckpoints::get(asset_id).unwrap_or_default();
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
        let id = try_next_pre::<T, _>(&mut ScheduleIdSequence::get(asset_id))?;

        // Charge the fee for checkpoint schedule creation.
        // N.B. this operation bundles verification + a storage change.
        // Thus, it must be last, and only storage changes follow.
        T::ProtocolFee::charge_fee(ProtocolOp::CheckpointCreateSchedule)?;

        // Add the new schedule's `next_at` to the cached next checkpoints.
        cached.add_schedule_next(id, next_at);
        cached.inc_total_pending(len);

        CachedNextCheckpoints::insert(asset_id, cached);
        ScheduledCheckpoints::insert(asset_id, id, &schedule);
        ScheduleRefCount::insert(asset_id, id, ref_count);
        ScheduleIdSequence::insert(asset_id, id);

        Self::deposit_event(Event::ScheduleCreated(caller_did, asset_id, id, schedule));
        Ok((id, next_at))
    }

    pub fn base_remove_schedule(
        caller_did: IdentityId,
        asset_id: AssetID,
        id: ScheduleId,
    ) -> DispatchResult {
        // Ensure that the schedule exists.
        let schedule =
            ScheduledCheckpoints::try_get(asset_id, id).map_err(|_| Error::<T>::NoSuchSchedule)?;

        // Can only remove the schedule if it doesn't have any references to it.
        ensure!(
            ScheduleRefCount::get(asset_id, id) == 0,
            Error::<T>::ScheduleNotRemovable
        );

        // Remove the schedule and any pending checkpoints.
        // We don't remove historical points related to the schedule.
        ScheduleRefCount::remove(asset_id, id);

        // Remove the schedule from the cached next checkpoints.
        CachedNextCheckpoints::mutate(&asset_id, |cached| {
            if let Some(cached) = cached {
                cached.remove_schedule(id);
                cached.dec_total_pending(schedule.len() as u64);
            }
        });
        // Remove scheduled checkpoints.
        ScheduledCheckpoints::remove(asset_id, id);

        // Emit event.
        Self::deposit_event(Event::ScheduleRemoved(caller_did, asset_id, id, schedule));
        Ok(())
    }

    /// The `caller_did` creates a checkpoint at `at` for `asset_id`.
    /// The ID of the new checkpoint is returned.
    fn create_at_by(
        caller_did: IdentityId,
        asset_id: AssetID,
        at: Moment,
    ) -> Result<CheckpointId, DispatchError> {
        let id = try_next_pre::<T, _>(&mut CheckpointIdSequence::get(asset_id))?;
        CheckpointIdSequence::insert(asset_id, id);
        Self::create_at(Some(caller_did), asset_id, id, at);
        Ok(id)
    }

    /// Creates a checkpoint at `at` for `asset_id`, with the given, in advanced reserved, `id`.
    /// The `caller_did` is the DID creating the checkpoint,
    /// or `None` scheduling created the checkpoint.
    ///
    /// Creating a checkpoint entails:
    /// - recording the total supply,
    /// - mapping the the ID to the `time`.
    fn create_at(caller_did: Option<IdentityId>, asset_id: AssetID, id: CheckpointId, at: Moment) {
        // Record total supply at checkpoint ID.
        let supply = <Asset<T>>::try_get_security_token(&asset_id)
            .map(|t| t.total_supply)
            .unwrap_or_default();
        TotalSupply::insert(asset_id, id, supply);

        // Relate AssetID -> ID -> time.
        Timestamps::insert(asset_id, id, at);

        // Emit event & we're done.
        Self::deposit_event(Event::CheckpointCreated(
            caller_did, asset_id, id, supply, at,
        ));
    }

    /// Increment the schedule ref count.
    pub fn inc_schedule_ref(asset_id: &AssetID, id: ScheduleId) {
        ScheduleRefCount::mutate(asset_id, id, |c| *c = c.saturating_add(1));
    }

    /// Decrement the schedule ref count.
    pub fn dec_schedule_ref(asset_id: &AssetID, id: ScheduleId) {
        ScheduleRefCount::mutate(asset_id, id, |c| *c = c.saturating_sub(1));
    }

    /// Ensure the schedule exists and get the next checkpoint.
    pub fn ensure_schedule_next_checkpoint(
        asset_id: &AssetID,
        id: ScheduleId,
    ) -> Result<(Moment, u64), DispatchError> {
        let schedule =
            ScheduledCheckpoints::try_get(asset_id, id).map_err(|_| Error::<T>::NoSuchSchedule)?;
        // Get the next checkpoint in the schedule, if it isn't finished.
        let cp_at = schedule.next().ok_or(Error::<T>::ScheduleFinished)?;
        // Get the index for the checkpoint in this schedule.
        let cp_at_idx = SchedulePoints::decode_len(asset_id, id).unwrap_or(0) as u64;
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
