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

//! Autogenerated weights for pallet_session
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
// -p=pallet_session
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

/// Weights for pallet_session using the Substrate node and recommended hardware.
pub struct WeightInfo;
impl pallet_session::WeightInfo for WeightInfo {
    // Storage: Staking Ledger (r:1 w:0)
    // Storage: Session NextKeys (r:1 w:1)
    // Storage: Session KeyOwner (r:4 w:4)
    fn set_keys() -> Weight {
        Weight::from_ref_time(115_335_000 as u64)
            .saturating_add(DbWeight::get().reads(6 as u64))
            .saturating_add(DbWeight::get().writes(5 as u64))
    }
    // Storage: Staking Ledger (r:1 w:0)
    // Storage: Session NextKeys (r:1 w:1)
    // Storage: Session KeyOwner (r:0 w:4)
    fn purge_keys() -> Weight {
        Weight::from_ref_time(83_793_000 as u64)
            .saturating_add(DbWeight::get().reads(2 as u64))
            .saturating_add(DbWeight::get().writes(5 as u64))
    }
}
