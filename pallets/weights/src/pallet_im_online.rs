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

//! Autogenerated weights for pallet_im_online
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

/// Weights for pallet_im_online using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_im_online::WeightInfo for SubstrateWeight {
    // Storage: Session Validators (r:1 w:0)
    // Proof Skipped: Session Validators (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Session CurrentIndex (r:1 w:0)
    // Proof Skipped: Session CurrentIndex (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ImOnline Keys (r:1 w:0)
    // Proof: ImOnline Keys (max_values: Some(1), max_size: Some(320002), added: 320497, mode: MaxEncodedLen)
    // Storage: ImOnline ReceivedHeartbeats (r:1 w:1)
    // Proof: ImOnline ReceivedHeartbeats (max_values: None, max_size: Some(10021032), added: 10023507, mode: MaxEncodedLen)
    // Storage: ImOnline AuthoredBlocks (r:1 w:0)
    // Proof: ImOnline AuthoredBlocks (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
    /// The range of component `k` is `[1, 1000]`.
    /// The range of component `e` is `[1, 100]`.
    fn validate_unsigned_and_then_heartbeat(k: u32, e: u32, ) -> Weight {
        // Minimum execution time: 210_253 nanoseconds.
        Weight::from_ref_time(206_389_791)
            // Standard Error: 3_546
            .saturating_add(Weight::from_ref_time(37_653).saturating_mul(k.into()))
            // Standard Error: 35_805
            .saturating_add(Weight::from_ref_time(773_137).saturating_mul(e.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(1))
    }
}
