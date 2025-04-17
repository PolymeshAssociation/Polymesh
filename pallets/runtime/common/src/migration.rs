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
