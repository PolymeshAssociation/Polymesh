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

//! Autogenerated weights for pallet_corporate_ballot
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-09-22, STEPS: [100, ], REPEAT: 5, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
// -s
// 100
// -r
// 5
// -p=pallet_corporate_ballot
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

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight as DbWeight, Weight},
};

/// Weights for pallet_corporate_ballot using the Substrate node and recommended hardware.
pub struct WeightInfo;
impl pallet_corporate_actions::ballot::WeightInfo for WeightInfo {
    fn attach_ballot(c: u32) -> Weight {
        (113_617_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((34_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn vote(c: u32, t: u32) -> Weight {
        (148_608_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((183_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 4_000
            .saturating_add((308_000 as Weight).saturating_mul(t as Weight))
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn change_end() -> Weight {
        (71_409_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn change_meta(c: u32) -> Weight {
        (78_480_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((52_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn change_rcv() -> Weight {
        (75_100_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_ballot() -> Weight {
        (86_433_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
}
