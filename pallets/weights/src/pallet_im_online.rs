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
//! DATE: 2023-02-10, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=pallet_im_online
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
pub struct WeightInfo;
impl pallet_im_online::WeightInfo for WeightInfo {
    // Storage: Session Validators (r:1 w:0)
    // Storage: Session CurrentIndex (r:1 w:0)
    // Storage: ImOnline ReceivedHeartbeats (r:1 w:1)
    // Storage: ImOnline AuthoredBlocks (r:1 w:0)
    // Storage: ImOnline Keys (r:1 w:0)
    fn validate_unsigned_and_then_heartbeat(k: u32, e: u32, ) -> Weight {
        (149_245_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((75_000 as Weight).saturating_mul(k as Weight))
            // Standard Error: 28_000
            .saturating_add((603_000 as Weight).saturating_mul(e as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
}
