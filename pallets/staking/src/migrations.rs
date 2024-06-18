// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use frame_support::pallet_prelude::ValueQuery;

use super::*;

pub mod polymesh_v2 {

    use super::*;
    use frame_support::storage_alias;

    // NOTE: value type doesn't matter, we just set it to () here.
    #[storage_alias]
    type SnapshotValidators<T: Config> = StorageValue<Pallet<T>, ()>;
    #[storage_alias]
    type SnapshotNominators<T: Config> = StorageValue<Pallet<T>, ()>;
    #[storage_alias]
    type QueuedElected<T: Config> = StorageValue<Pallet<T>, ()>;
    #[storage_alias]
    type QueuedScore<T: Config> = StorageValue<Pallet<T>, ()>;
    #[storage_alias]
    type EraElectionStatus<T: Config> = StorageValue<Pallet<T>, ()>;
    #[storage_alias]
    type IsCurrentSessionFinal<T: Config> = StorageValue<Pallet<T>, ()>;

    fn migrate_old_election_storage<T: Config>() {
        log!(info, "Staking Migration: removing old election storage");

        SnapshotValidators::<T>::kill();
        SnapshotNominators::<T>::kill();
        QueuedElected::<T>::kill();
        QueuedScore::<T>::kill();
        EraElectionStatus::<T>::kill();
        IsCurrentSessionFinal::<T>::kill();

        log!(info, "Old storage has been cleared");
    }

    #[storage_alias]
    type CounterForValidators<T: Config> = StorageValue<Pallet<T>, u32>;
    #[storage_alias]
    type CounterForNominators<T: Config> = StorageValue<Pallet<T>, u32>;

    fn migrate_to_counted_map<T: Config>() {
        log!(
            info,
            "Staking Migration: changing StorageMap to CountedStorageMap"
        );

        let validator_count = Validators::<T>::iter().count() as u32;
        let nominator_count = Nominators::<T>::iter().count() as u32;

        CounterForValidators::<T>::put(validator_count);
        CounterForNominators::<T>::put(nominator_count);

        log!(info, "Staking Migration: maps have been updated");
    }

    #[storage_alias]
    type EarliestUnappliedSlash<T: Config> = StorageValue<Pallet<T>, EraIndex>;

    fn migrate_earliest_unaplied_slash_removal<T: Config>() {
        log!(info, "Staking Migration: EarliestUnappliedSlash removal");

        let pending_slashes = <Pallet<T> as Store>::UnappliedSlashes::iter().take(512);
        for (era, slashes) in pending_slashes {
            for slash in slashes {
                // in the old slashing scheme, the slash era was the key at which we read
                // from `UnappliedSlashes`.
                log!(
                    warn,
                    "prematurely applying a slash ({:?}) for era {:?}",
                    slash,
                    era
                );
                slashing::apply_slash::<T>(slash, era);
            }
        }

        EarliestUnappliedSlash::<T>::kill();
    }

    #[storage_alias]
    type HistoryDepth<T: Config> = StorageValue<Pallet<T>, u32, ValueQuery>;
    #[storage_alias]
    type MinimumBondThreshold<T: Config> = StorageValue<Pallet<T>, BalanceOf<T>, ValueQuery>;

    fn migrate_remove_old_storage<T: Config>() {
        log!(
            info,
            "Staking Migration: removing HistoryDepth and MinimumBondThreshold storage"
        );
        HistoryDepth::<T>::kill();
        MinimumBondThreshold::<T>::kill();
        log!(info, "Staking Migration: storage has been removed");
    }

    pub fn migrate_to_v2<T: Config>() {
        migrate_old_election_storage::<T>();
        migrate_to_counted_map::<T>();
        migrate_earliest_unaplied_slash_removal::<T>();
        migrate_remove_old_storage::<T>();
    }
}
