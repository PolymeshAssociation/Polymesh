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

//! Autogenerated weights for pallet_preimage
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-17, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512
//! HOSTNAME: `ubuntu-8gb-hel1-1`, CPU: `Intel Xeon Processor (Skylake, IBRS)`

// Executed Command:
// ./target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=pallet_preimage
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

/// Weights for pallet_preimage using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_preimage::WeightInfo for SubstrateWeight {
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    /// The range of component `s` is `[0, 4194304]`.
    fn note_preimage(s: u32) -> Weight {
        // Minimum execution time: 85_590 nanoseconds.
        Weight::from_ref_time(95_350_000)
            // Standard Error: 15
            .saturating_add(Weight::from_ref_time(3_340).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    /// The range of component `s` is `[0, 4194304]`.
    fn note_requested_preimage(s: u32) -> Weight {
        // Minimum execution time: 53_237 nanoseconds.
        Weight::from_ref_time(53_592_000)
            // Standard Error: 21
            .saturating_add(Weight::from_ref_time(3_407).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    /// The range of component `s` is `[0, 4194304]`.
    fn note_no_deposit_preimage(s: u32) -> Weight {
        // Minimum execution time: 42_918 nanoseconds.
        Weight::from_ref_time(47_304_000)
            // Standard Error: 24
            .saturating_add(Weight::from_ref_time(3_415).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    fn unnote_preimage() -> Weight {
        // Minimum execution time: 148_458 nanoseconds.
        Weight::from_ref_time(156_886_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    fn unnote_no_deposit_preimage() -> Weight {
        // Minimum execution time: 117_303 nanoseconds.
        Weight::from_ref_time(143_439_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn request_preimage() -> Weight {
        // Minimum execution time: 112_483 nanoseconds.
        Weight::from_ref_time(116_050_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn request_no_deposit_preimage() -> Weight {
        // Minimum execution time: 73_890 nanoseconds.
        Weight::from_ref_time(75_115_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn request_unnoted_preimage() -> Weight {
        // Minimum execution time: 71_104 nanoseconds.
        Weight::from_ref_time(74_930_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn request_requested_preimage() -> Weight {
        // Minimum execution time: 29_314 nanoseconds.
        Weight::from_ref_time(38_353_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    fn unrequest_preimage() -> Weight {
        // Minimum execution time: 119_106 nanoseconds.
        Weight::from_ref_time(141_510_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn unrequest_unnoted_preimage() -> Weight {
        // Minimum execution time: 28_368 nanoseconds.
        Weight::from_ref_time(30_984_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn unrequest_multi_referenced_preimage() -> Weight {
        // Minimum execution time: 28_127 nanoseconds.
        Weight::from_ref_time(28_848_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
}
