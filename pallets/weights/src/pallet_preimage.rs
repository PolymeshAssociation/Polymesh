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
//! DATE: 2023-08-24, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512
//! HOSTNAME: `ubuntu-8gb-hel1-5`, CPU: `AMD EPYC Processor`

// Executed Command:
// target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=*
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
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    // Storage: Preimage PreimageFor (r:0 w:1)
    // Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    /// The range of component `s` is `[0, 4194304]`.
    fn note_preimage(s: u32, ) -> Weight {
        // Minimum execution time: 49_463 nanoseconds.
        Weight::from_ref_time(54_365_427)
            // Standard Error: 19
            .saturating_add(Weight::from_ref_time(2_506).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    // Storage: Preimage PreimageFor (r:0 w:1)
    // Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    /// The range of component `s` is `[0, 4194304]`.
    fn note_requested_preimage(s: u32, ) -> Weight {
        // Minimum execution time: 30_557 nanoseconds.
        Weight::from_ref_time(86_987_255)
            // Standard Error: 15
            .saturating_add(Weight::from_ref_time(2_440).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    // Storage: Preimage PreimageFor (r:0 w:1)
    // Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    /// The range of component `s` is `[0, 4194304]`.
    fn note_no_deposit_preimage(s: u32, ) -> Weight {
        // Minimum execution time: 29_324 nanoseconds.
        Weight::from_ref_time(96_102_265)
            // Standard Error: 21
            .saturating_add(Weight::from_ref_time(2_419).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    // Storage: Preimage PreimageFor (r:0 w:1)
    // Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    fn unnote_preimage() -> Weight {
        // Minimum execution time: 71_673 nanoseconds.
        Weight::from_ref_time(79_127_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    // Storage: Preimage PreimageFor (r:0 w:1)
    // Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    fn unnote_no_deposit_preimage() -> Weight {
        // Minimum execution time: 54_833 nanoseconds.
        Weight::from_ref_time(62_658_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn request_preimage() -> Weight {
        // Minimum execution time: 49_632 nanoseconds.
        Weight::from_ref_time(52_268_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn request_no_deposit_preimage() -> Weight {
        // Minimum execution time: 23_323 nanoseconds.
        Weight::from_ref_time(30_597_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn request_unnoted_preimage() -> Weight {
        // Minimum execution time: 58_479 nanoseconds.
        Weight::from_ref_time(59_853_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn request_requested_preimage() -> Weight {
        // Minimum execution time: 23_725 nanoseconds.
        Weight::from_ref_time(25_997_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    // Storage: Preimage PreimageFor (r:0 w:1)
    // Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    fn unrequest_preimage() -> Weight {
        // Minimum execution time: 61_625 nanoseconds.
        Weight::from_ref_time(96_039_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn unrequest_unnoted_preimage() -> Weight {
        // Minimum execution time: 21_891 nanoseconds.
        Weight::from_ref_time(25_016_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn unrequest_multi_referenced_preimage() -> Weight {
        // Minimum execution time: 39_073 nanoseconds.
        Weight::from_ref_time(42_719_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
}
