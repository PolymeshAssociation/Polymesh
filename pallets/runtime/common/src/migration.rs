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

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::RuntimeDbWeight};
use sp_io::{hashing::twox_128, storage::clear_prefix, KillStorageResult};

pub struct RemovePallet<P: Get<&'static str>, DbWeight: Get<RuntimeDbWeight>>(
    PhantomData<(P, DbWeight)>,
);
impl<P: Get<&'static str>, DbWeight: Get<RuntimeDbWeight>> frame_support::traits::OnRuntimeUpgrade
    for RemovePallet<P, DbWeight>
{
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        let hashed_prefix = twox_128(P::get().as_bytes());
        let keys_removed = match clear_prefix(&hashed_prefix, None) {
            KillStorageResult::AllRemoved(value) => value,
            KillStorageResult::SomeRemaining(value) => {
                log::error!(
                    "`clear_prefix` failed to remove all keys for {}. THIS SHOULD NEVER HAPPEN! 🚨",
                    P::get()
                );
                value
            }
        } as u64;

        log::info!("Removed {} {} keys 🧹", keys_removed, P::get());

        DbWeight::get().reads_writes(keys_removed + 1, keys_removed)
    }
}

use frame_support::{
    migrations::VersionedMigration, storage, traits::UncheckedOnRuntimeUpgrade, weights::Weight,
};
use sp_consensus_grandpa::SetId;

use pallet_grandpa::migrations::v4::OLD_PREFIX;

const GRANDPA_AUTHORITIES_KEY: &[u8] = b":grandpa_authorities";
const CURRENT_SET_ID_STORAGE: &[u8] = b"CurrentSetId";

fn grandpa_finality_current_set_id() -> Option<SetId> {
    let mut key = [0u8; 32];
    key[..16].copy_from_slice(&sp_io::hashing::twox_128(OLD_PREFIX));
    key[16..].copy_from_slice(&sp_io::hashing::twox_128(CURRENT_SET_ID_STORAGE));
    storage::unhashed::get::<SetId>(&key)
}

/// Actual implementation of [`PolyMigrateToV5`].
pub struct UncheckedMigrateImpl<T>(PhantomData<T>);

impl<T: pallet_grandpa::Config> UncheckedOnRuntimeUpgrade for UncheckedMigrateImpl<T> {
    fn on_runtime_upgrade() -> Weight {
        // Note: resyncs CurrentSetId
        let current_set_id = pallet_grandpa::CurrentSetId::<T>::get();
        let legacy_current_set_id = {
            match grandpa_finality_current_set_id() {
                Some(set_id) => set_id,
                None => {
                    log::info!("Legacy current set id not found, assuming 0.");
                    0
                }
            }
        };

        let new_current_set_id = current_set_id.saturating_add(legacy_current_set_id);
        pallet_grandpa::CurrentSetId::<T>::put(new_current_set_id);

        storage::unhashed::kill(GRANDPA_AUTHORITIES_KEY);

        T::DbWeight::get().reads_writes(2, 2)
    }
}

/// Migrate the storage to V5.
///
/// Resyncs CurrentSetId and Switches from `GRANDPA_AUTHORITIES_KEY` to a normal FRAME storage item.
pub type MigrateGrandpaToV5<T> = VersionedMigration<
    0,
    5,
    UncheckedMigrateImpl<T>,
    pallet_grandpa::Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;
