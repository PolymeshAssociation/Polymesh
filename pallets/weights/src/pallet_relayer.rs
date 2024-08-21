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

//! Autogenerated weights for pallet_relayer
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-08-21, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512
//! HOSTNAME: `ubuntu-8gb-nbg1-1-bench2`, CPU: `AMD EPYC-Milan Processor`

// Executed Command:
// ./polymesh
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
// ./Polymesh/pallets/weights/src/
// --template
// ./Polymesh/.maintain/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

/// Weights for pallet_relayer using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_relayer::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity NumberOfGivenAuths (r:1 w:1)
    // Proof Skipped: Identity NumberOfGivenAuths (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CurrentAuthId (r:1 w:1)
    // Proof Skipped: Identity CurrentAuthId (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:0 w:1)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    fn set_paying_key() -> Weight {
        // Minimum execution time: 35_133 nanoseconds.
        Weight::from_ref_time(35_975_000)
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(4))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:1 w:1)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity OutdatedAuthorizations (r:1 w:0)
    // Proof Skipped: Identity OutdatedAuthorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:4 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: Relayer Subsidies (r:1 w:1)
    // Proof Skipped: Relayer Subsidies (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AccountKeyRefCount (r:2 w:2)
    // Proof Skipped: Identity AccountKeyRefCount (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity NumberOfGivenAuths (r:1 w:1)
    // Proof Skipped: Identity NumberOfGivenAuths (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    fn accept_paying_key() -> Weight {
        // Minimum execution time: 91_288 nanoseconds.
        Weight::from_ref_time(95_355_000)
            .saturating_add(DbWeight::get().reads(14))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Relayer Subsidies (r:1 w:1)
    // Proof Skipped: Relayer Subsidies (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AccountKeyRefCount (r:2 w:2)
    // Proof Skipped: Identity AccountKeyRefCount (max_values: None, max_size: None, mode: Measured)
    fn remove_paying_key() -> Weight {
        // Minimum execution time: 34_642 nanoseconds.
        Weight::from_ref_time(37_337_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Relayer Subsidies (r:1 w:1)
    // Proof Skipped: Relayer Subsidies (max_values: None, max_size: None, mode: Measured)
    fn update_polyx_limit() -> Weight {
        // Minimum execution time: 26_710 nanoseconds.
        Weight::from_ref_time(28_843_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Relayer Subsidies (r:1 w:1)
    // Proof Skipped: Relayer Subsidies (max_values: None, max_size: None, mode: Measured)
    fn increase_polyx_limit() -> Weight {
        // Minimum execution time: 27_793 nanoseconds.
        Weight::from_ref_time(28_082_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Relayer Subsidies (r:1 w:1)
    // Proof Skipped: Relayer Subsidies (max_values: None, max_size: None, mode: Measured)
    fn decrease_polyx_limit() -> Weight {
        // Minimum execution time: 27_131 nanoseconds.
        Weight::from_ref_time(28_433_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
}
