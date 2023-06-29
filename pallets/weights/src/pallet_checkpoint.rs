// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_checkpoint
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-25, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512
//! HOSTNAME: `dev-fsn001`, CPU: `AMD Ryzen 9 5950X 16-Core Processor`

// Executed Command:
// ./target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=pallet_checkpoint
// -e=*
// --heap-pages
// 4096
// --db-cache
// 512
// --execution
// wasm
// --wasm-execution
// compiled
// --output
// ./pallets/weights/src/
// --template
// ./.maintain/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

/// Weights for pallet_checkpoint using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_asset::checkpoint::WeightInfo for SubstrateWeight {
    // Storage: Checkpoint SchedulesMaxComplexity (r:0 w:1)
    fn set_schedules_max_complexity() -> Weight {
        // Minimum execution time: 17_362 nanoseconds.
        Weight::from_ref_time(17_463_000).saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:1 w:1)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Checkpoint TotalSupply (r:0 w:1)
    // Storage: Checkpoint Timestamps (r:0 w:1)
    fn create_checkpoint() -> Weight {
        // Minimum execution time: 52_406 nanoseconds.
        Weight::from_ref_time(54_421_000)
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Checkpoint Schedules (r:1 w:1)
    // Storage: Checkpoint SchedulesMaxComplexity (r:1 w:0)
    // Storage: Checkpoint ScheduleIdSequence (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:1 w:1)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Storage: Checkpoint SchedulePoints (r:1 w:1)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Checkpoint TotalSupply (r:0 w:1)
    // Storage: Checkpoint Timestamps (r:0 w:1)
    // Storage: Checkpoint ScheduleRefCount (r:0 w:1)
    /// The range of component `s` is `[1, 52]`.
    fn create_schedule(s: u32) -> Weight {
        // Minimum execution time: 79_877 nanoseconds.
        Weight::from_ref_time(82_377_747)
            // Standard Error: 18_602
            .saturating_add(Weight::from_ref_time(133_833).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(13))
            .saturating_add(DbWeight::get().writes(7))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Checkpoint Schedules (r:1 w:1)
    // Storage: Checkpoint ScheduleRefCount (r:1 w:1)
    /// The range of component `s` is `[1, 52]`.
    fn remove_schedule() -> Weight {
        // Minimum execution time: 47_819 nanoseconds.
        Weight::from_ref_time(50_762_879)
            // Standard Error: 14_306
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(2))
    }
}
