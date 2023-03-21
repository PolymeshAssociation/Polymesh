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

//! Autogenerated weights for pallet_scheduler
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
// -p=pallet_scheduler
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

/// Weights for pallet_scheduler using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_scheduler::WeightInfo for SubstrateWeight {
    // Storage: Scheduler IncompleteSince (r:1 w:1)
    fn service_agendas_base() -> Weight {
        // Minimum execution time: 5_690 nanoseconds.
        Weight::from_ref_time(5_921_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    /// The range of component `s` is `[0, 50]`.
    fn service_agenda_base(s: u32) -> Weight {
        // Minimum execution time: 4_849 nanoseconds.
        Weight::from_ref_time(8_917_460)
            // Standard Error: 3_633
            .saturating_add(Weight::from_ref_time(429_534).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    fn service_task_base() -> Weight {
        // Minimum execution time: 12_182 nanoseconds.
        Weight::from_ref_time(13_065_000)
    }
    // Storage: Preimage PreimageFor (r:1 w:1)
    // Storage: Preimage StatusFor (r:1 w:1)
    /// The range of component `s` is `[128, 4194304]`.
    fn service_task_fetched(s: u32) -> Weight {
        // Minimum execution time: 27_290 nanoseconds.
        Weight::from_ref_time(27_320_000)
            // Standard Error: 2
            .saturating_add(Weight::from_ref_time(819).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Scheduler Lookup (r:0 w:1)
    fn service_task_named() -> Weight {
        // Minimum execution time: 13_394 nanoseconds.
        Weight::from_ref_time(13_605_000).saturating_add(DbWeight::get().writes(1))
    }
    fn service_task_periodic() -> Weight {
        // Minimum execution time: 11_461 nanoseconds.
        Weight::from_ref_time(11_652_000)
    }
    fn execute_dispatch_signed() -> Weight {
        // Minimum execution time: 5_179 nanoseconds.
        Weight::from_ref_time(5_340_000)
    }
    fn execute_dispatch_unsigned() -> Weight {
        // Minimum execution time: 5_260 nanoseconds.
        Weight::from_ref_time(5_600_000)
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    /// The range of component `s` is `[0, 49]`.
    fn schedule(s: u32) -> Weight {
        // Minimum execution time: 21_470 nanoseconds.
        Weight::from_ref_time(24_835_964)
            // Standard Error: 5_328
            .saturating_add(Weight::from_ref_time(430_846).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Scheduler Lookup (r:0 w:1)
    /// The range of component `s` is `[1, 50]`.
    fn cancel(s: u32) -> Weight {
        // Minimum execution time: 21_911 nanoseconds.
        Weight::from_ref_time(23_175_482)
            // Standard Error: 3_158
            .saturating_add(Weight::from_ref_time(418_212).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    /// The range of component `s` is `[0, 49]`.
    fn schedule_named(s: u32) -> Weight {
        // Minimum execution time: 24_215 nanoseconds.
        Weight::from_ref_time(28_748_465)
            // Standard Error: 11_682
            .saturating_add(Weight::from_ref_time(490_645).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    /// The range of component `s` is `[1, 50]`.
    fn cancel_named(s: u32) -> Weight {
        // Minimum execution time: 23_794 nanoseconds.
        Weight::from_ref_time(26_378_940)
            // Standard Error: 18_377
            .saturating_add(Weight::from_ref_time(448_772).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(2))
    }
}
