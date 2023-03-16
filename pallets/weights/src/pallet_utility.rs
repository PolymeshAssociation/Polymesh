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

//! Autogenerated weights for pallet_utility
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
// -p=pallet_utility
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

use polymesh_runtime_common::GetDispatchInfo;
use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

fn sum_weights(calls: &[impl GetDispatchInfo]) -> Weight {
    let num_calls = calls.len() as u64;
    calls
        .iter()
        .map(|call| call.get_dispatch_info().weight)
        .fold(Weight::zero(), |a: Weight, n| a.saturating_add(n))
        .saturating_add(
            // Each call has 2 reads and 2 writes overhead.
            num_calls
                * DbWeight::get()
                    .reads(2 as u64)
                    .saturating_add(DbWeight::get().writes(2 as u64)),
        )
}

/// Weights for pallet_utility using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_utility::WeightInfo for SubstrateWeight {
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    fn batch(calls: &[impl GetDispatchInfo]) -> Weight {
        Weight::from_ref_time(38_672_000)
            // Standard Error: 438_000
            .saturating_add(sum_weights(calls))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    fn batch_atomic(calls: &[impl GetDispatchInfo]) -> Weight {
        Weight::from_ref_time(49_113_000)
            // Standard Error: 165_000
            .saturating_add(sum_weights(calls))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    fn batch_optimistic(calls: &[impl GetDispatchInfo]) -> Weight {
        Weight::from_ref_time(27_520_000)
            // Standard Error: 546_000
            .saturating_add(sum_weights(calls))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: Utility Nonces (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    fn relay_tx(call: &impl GetDispatchInfo) -> Weight {
        Weight::from_ref_time(167_964_000)
            .saturating_add(call.get_dispatch_info().weight)
            .saturating_add(DbWeight::get().reads(10))
            .saturating_add(DbWeight::get().writes(3))
    }
}
