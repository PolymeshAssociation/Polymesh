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
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-01-13, STEPS: [100, ], REPEAT: 5, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
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
// --raw

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

/// Weights for frame_system using the Substrate node and recommended hardware.
pub struct WeightInfo;
impl frame_system::WeightInfo for WeightInfo {
    fn remark(_b: u32) -> Weight {
        (1_594_000 as Weight)
    }
    fn remark_with_event(_b: u32) -> Weight {
        (1_787_000 as Weight)
    }
    fn set_heap_pages() -> Weight {
        (2_017_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_changes_trie_config() -> Weight {
        (12_502_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn set_storage(i: u32) -> Weight {
        (4_121_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((697_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
    }
    fn kill_storage(i: u32) -> Weight {
        (9_061_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((514_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
    }
    fn kill_prefix(p: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 16_000
            .saturating_add((968_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
    }
}
