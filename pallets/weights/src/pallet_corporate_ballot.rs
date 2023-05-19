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

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

/// Weights for pallet_corporate_ballot using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_corporate_actions::ballot::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: CorporateAction CorporateActions (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: CorporateBallot TimeRanges (r:1 w:1)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Storage: CorporateBallot MotionNumChoices (r:0 w:1)
    // Storage: CorporateBallot Metas (r:0 w:1)
    // Storage: CorporateBallot RCV (r:0 w:1)
    /// The range of component `c` is `[0, 1000]`.
    fn attach_ballot(c: u32) -> Weight {
        // Minimum execution time: 137_807 nanoseconds.
        Weight::from_ref_time(138_004_530)
            // Standard Error: 9_482
            .saturating_add(Weight::from_ref_time(142_926).saturating_mul(c.into()))
            .saturating_add(DbWeight::get().reads(9))
            .saturating_add(DbWeight::get().writes(4))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: CorporateBallot TimeRanges (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: CorporateAction CorporateActions (r:1 w:0)
    // Storage: CorporateBallot MotionNumChoices (r:1 w:0)
    // Storage: CorporateBallot RCV (r:1 w:0)
    // Storage: Checkpoint SchedulePoints (r:1 w:0)
    // Storage: Asset BalanceOf (r:1 w:0)
    // Storage: CorporateBallot Votes (r:1 w:1)
    // Storage: CorporateBallot Results (r:1 w:1)
    /// The range of component `c` is `[0, 1000]`.
    /// The range of component `t` is `[0, 1000]`.
    fn vote(c: u32, t: u32) -> Weight {
        // Minimum execution time: 263_222 nanoseconds.
        Weight::from_ref_time(151_846_644)
            // Standard Error: 14_292
            .saturating_add(Weight::from_ref_time(231_413).saturating_mul(c.into()))
            // Standard Error: 14_292
            .saturating_add(Weight::from_ref_time(773_998).saturating_mul(t.into()))
            .saturating_add(DbWeight::get().reads(10))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: CorporateBallot TimeRanges (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    fn change_end() -> Weight {
        // Minimum execution time: 112_417 nanoseconds.
        Weight::from_ref_time(120_802_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: CorporateBallot TimeRanges (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: CorporateBallot MotionNumChoices (r:0 w:1)
    // Storage: CorporateBallot Metas (r:0 w:1)
    /// The range of component `c` is `[0, 1000]`.
    fn change_meta(c: u32) -> Weight {
        // Minimum execution time: 119_879 nanoseconds.
        Weight::from_ref_time(144_768_355)
            // Standard Error: 7_435
            .saturating_add(Weight::from_ref_time(79_585).saturating_mul(c.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: CorporateBallot TimeRanges (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: CorporateBallot RCV (r:0 w:1)
    fn change_rcv() -> Weight {
        // Minimum execution time: 119_445 nanoseconds.
        Weight::from_ref_time(184_022_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: CorporateBallot TimeRanges (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: CorporateBallot MotionNumChoices (r:0 w:1)
    // Storage: CorporateBallot Metas (r:0 w:1)
    // Storage: CorporateBallot RCV (r:0 w:1)
    fn remove_ballot() -> Weight {
        // Minimum execution time: 121_765 nanoseconds.
        Weight::from_ref_time(122_847_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(4))
    }
}
