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
pub struct WeightInfo;
impl pallet_corporate_actions::ballot::WeightInfo for WeightInfo {
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
    fn attach_ballot(c: u32) -> Weight {
        Weight::from_ref_time(118_293_000 as u64)
            // Standard Error: 9_000
            .saturating_add(Weight::from_ref_time(109_000 as u64).saturating_mul(c as u64))
            .saturating_add(DbWeight::get().reads(9 as u64))
            .saturating_add(DbWeight::get().writes(4 as u64))
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
    fn vote(c: u32, t: u32) -> Weight {
        Weight::from_ref_time(132_433_000 as u64)
            // Standard Error: 9_000
            .saturating_add(Weight::from_ref_time(233_000 as u64).saturating_mul(c as u64))
            // Standard Error: 9_000
            .saturating_add(Weight::from_ref_time(254_000 as u64).saturating_mul(t as u64))
            .saturating_add(DbWeight::get().reads(10 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: CorporateBallot TimeRanges (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    fn change_end() -> Weight {
        Weight::from_ref_time(130_412_000 as u64)
            .saturating_add(DbWeight::get().reads(6 as u64))
            .saturating_add(DbWeight::get().writes(1 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: CorporateBallot TimeRanges (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: CorporateBallot MotionNumChoices (r:0 w:1)
    // Storage: CorporateBallot Metas (r:0 w:1)
    fn change_meta(c: u32) -> Weight {
        Weight::from_ref_time(76_724_000 as u64)
            // Standard Error: 8_000
            .saturating_add(Weight::from_ref_time(124_000 as u64).saturating_mul(c as u64))
            .saturating_add(DbWeight::get().reads(6 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: CorporateBallot TimeRanges (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: CorporateBallot RCV (r:0 w:1)
    fn change_rcv() -> Weight {
        Weight::from_ref_time(74_809_000 as u64)
            .saturating_add(DbWeight::get().reads(6 as u64))
            .saturating_add(DbWeight::get().writes(1 as u64))
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
        Weight::from_ref_time(81_801_000 as u64)
            .saturating_add(DbWeight::get().reads(6 as u64))
            .saturating_add(DbWeight::get().writes(4 as u64))
    }
}
