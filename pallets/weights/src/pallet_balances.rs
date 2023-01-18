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

//! Autogenerated weights for pallet_balances
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-06-25, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=pallet_balances
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

/// Weights for pallet_balances using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_balances::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    // Storage: System Account (r:2 w:2)
    fn transfer() -> Weight {
        Weight::from_ref_time(209_604_000 as u64)
            .saturating_add(DbWeight::get().reads(9 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    // Storage: System Account (r:2 w:2)
    fn transfer_with_memo() -> Weight {
        Weight::from_ref_time(145_217_000 as u64)
            .saturating_add(DbWeight::get().reads(9 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:2 w:2)
    fn deposit_block_reward_reserve_balance() -> Weight {
        Weight::from_ref_time(154_228_000 as u64)
            .saturating_add(DbWeight::get().reads(4 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:2 w:2)
    fn set_balance() -> Weight {
        Weight::from_ref_time(126_335_000 as u64)
            .saturating_add(DbWeight::get().reads(5 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: System Account (r:2 w:2)
    // Storage: Identity KeyRecords (r:2 w:0)
    fn force_transfer() -> Weight {
        Weight::from_ref_time(70_921_000 as u64)
            .saturating_add(DbWeight::get().reads(4 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn burn_account_balance() -> Weight {
        Weight::from_ref_time(60_874_000 as u64)
            .saturating_add(DbWeight::get().reads(3 as u64))
            .saturating_add(DbWeight::get().writes(1 as u64))
    }
}
