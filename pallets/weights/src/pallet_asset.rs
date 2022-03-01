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

//! Autogenerated weights for pallet_asset
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
// -p=pallet_asset
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

/// Weights for pallet_asset using the Substrate node and recommended hardware.
pub struct WeightInfo;
impl pallet_asset::WeightInfo for WeightInfo {
    fn register_ticker() -> Weight {
        (80_765_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn accept_ticker_transfer() -> Weight {
        (90_227_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn accept_asset_ownership_transfer() -> Weight {
        (110_788_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn create_asset(n: u32, i: u32, f: u32) -> Weight {
        (157_890_000 as Weight)
            // Standard Error: 9_000
            .saturating_add((73_000 as Weight).saturating_mul(n as Weight))
            // Standard Error: 2_000
            .saturating_add((79_000 as Weight).saturating_mul(i as Weight))
            // Standard Error: 9_000
            .saturating_add((86_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(12 as Weight))
    }
    fn freeze() -> Weight {
        (70_409_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unfreeze() -> Weight {
        (71_421_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn rename_asset(n: u32) -> Weight {
        (62_463_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((47_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn issue() -> Weight {
        (162_071_000 as Weight)
            .saturating_add(DbWeight::get().reads(18 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn redeem() -> Weight {
        (167_266_000 as Weight)
            .saturating_add(DbWeight::get().reads(16 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn make_divisible() -> Weight {
        (62_479_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_documents(d: u32) -> Weight {
        (97_594_000 as Weight)
            // Standard Error: 187_000
            .saturating_add((28_420_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    fn remove_documents(d: u32) -> Weight {
        (39_507_000 as Weight)
            // Standard Error: 61_000
            .saturating_add((11_582_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    fn set_funding_round(_f: u32) -> Weight {
        (59_158_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn update_identifiers(i: u32) -> Weight {
        (63_879_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((63_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn claim_classic_ticker() -> Weight {
        (265_862_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn reserve_classic_ticker() -> Weight {
        (55_283_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn controller_transfer() -> Weight {
        (190_330_000 as Weight)
            .saturating_add(DbWeight::get().reads(20 as Weight))
            .saturating_add(DbWeight::get().writes(9 as Weight))
    }
    fn register_custom_asset_type(n: u32) -> Weight {
        (54_071_000 as Weight)
            // Standard Error: 0
            .saturating_add((13_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }

    fn set_asset_metadata() -> Weight {
        (1_000_042 as Weight)
    }
    fn set_asset_metadata_details() -> Weight {
        (1_000_042 as Weight)
    }
    fn register_asset_metadata_local_type() -> Weight {
        (1_000_042 as Weight)
    }
    fn register_asset_metadata_global_type() -> Weight {
        (1_000_042 as Weight)
    }
}
