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

//! Autogenerated weights for frame_system
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-06-22, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=frame_system
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

/// Weights for frame_system using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl frame_system::WeightInfo for SubstrateWeight {
    fn remark(b: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(1_000 as u64).saturating_mul(b as u64))
    }
    fn remark_with_event(b: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(3_000 as u64).saturating_mul(b as u64))
    }
    // Storage: System Digest (r:1 w:1)
    // Storage: unknown [0x3a686561707061676573] (r:0 w:1)
    fn set_heap_pages() -> Weight {
        Weight::from_ref_time(6_757_000 as u64)
            .saturating_add(DbWeight::get().reads(1 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Skipped Metadata (r:0 w:0)
    fn set_storage(i: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 22_000
            .saturating_add(Weight::from_ref_time(1_224_000 as u64).saturating_mul(i as u64))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
    // Storage: Skipped Metadata (r:0 w:0)
    fn kill_storage(i: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 16_000
            .saturating_add(Weight::from_ref_time(790_000 as u64).saturating_mul(i as u64))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(i as u64)))
    }
    // Storage: Skipped Metadata (r:0 w:0)
    fn kill_prefix(p: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 23_000
            .saturating_add(Weight::from_ref_time(1_696_000 as u64).saturating_mul(p as u64))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(p as u64)))
    }
}
