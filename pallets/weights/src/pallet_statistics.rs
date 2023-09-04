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

//! Autogenerated weights for pallet_statistics
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

/// Weights for pallet_statistics using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_statistics::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Statistics AssetTransferCompliances (r:1 w:0)
    // Proof Skipped: Statistics AssetTransferCompliances (max_values: None, max_size: None, mode: Measured)
    // Storage: Statistics ActiveAssetStats (r:1 w:1)
    // Proof Skipped: Statistics ActiveAssetStats (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[1, 9]`.
    fn set_active_asset_stats(i: u32) -> Weight {
        // Minimum execution time: 65_192 nanoseconds.
        Weight::from_ref_time(80_979_517)
            // Manually set weight for `i`
            .saturating_add(Weight::from_ref_time(80_581).saturating_mul(i.into()))	
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Proof Skipped: Statistics ActiveAssetStats (max_values: None, max_size: None, mode: Measured)
    // Storage: Statistics AssetStats (r:0 w:250)
    // Proof Skipped: Statistics AssetStats (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[1, 250]`.
    fn batch_update_asset_stats(i: u32) -> Weight {
        // Minimum execution time: 67_886 nanoseconds.
        Weight::from_ref_time(33_050_783)
            // Standard Error: 73_989
            .saturating_add(Weight::from_ref_time(5_389_283).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(i.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Proof Skipped: Statistics ActiveAssetStats (max_values: None, max_size: None, mode: Measured)
    // Storage: Statistics AssetTransferCompliances (r:1 w:1)
    // Proof Skipped: Statistics AssetTransferCompliances (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[1, 3]`.
    fn set_asset_transfer_compliance(i: u32) -> Weight {
        // Minimum execution time: 68_288 nanoseconds.
        Weight::from_ref_time(86_012_363)
            // Standard Error: 1_372_829
            .saturating_add(Weight::from_ref_time(8_349_182).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Statistics TransferConditionExemptEntities (r:0 w:1000)
    // Proof Skipped: Statistics TransferConditionExemptEntities (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[0, 1000]`.
    fn set_entities_exempt(i: u32) -> Weight {
        // Minimum execution time: 47_750 nanoseconds.
        Weight::from_ref_time(35_337_385)
            // Standard Error: 92_352
            .saturating_add(Weight::from_ref_time(4_901_158).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(i.into())))
    }
    // Storage: Statistics AssetStats (r:1 w:0)
    // Proof Skipped: Statistics AssetStats (max_values: None, max_size: None, mode: Measured)
    /// The range of component `a` is `[0, 1]`.
    fn max_investor_count_restriction(a: u32) -> Weight {
        // Minimum execution time: 912 nanoseconds.
        Weight::from_ref_time(1_995_200)
            // Standard Error: 225_003
            .saturating_add(Weight::from_ref_time(10_244_465).saturating_mul(a.into()))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(a.into())))
    }
    fn max_investor_ownership_restriction() -> Weight {
        // Minimum execution time: 4_218 nanoseconds.
        Weight::from_ref_time(5_550_000)
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    /// The range of component `c` is `[0, 1]`.
    fn claim_count_restriction_no_stats(c: u32) -> Weight {
        // Minimum execution time: 1_002 nanoseconds.
        Weight::from_ref_time(1_518_465)
            // Standard Error: 119_759
            .saturating_add(Weight::from_ref_time(25_922_867).saturating_mul(c.into()))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(c.into())))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: Statistics AssetStats (r:1 w:0)
    // Proof Skipped: Statistics AssetStats (max_values: None, max_size: None, mode: Measured)
    fn claim_count_restriction_with_stats() -> Weight {
        // Minimum execution time: 33_844 nanoseconds.
        Weight::from_ref_time(34_215_000).saturating_add(DbWeight::get().reads(4))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: Statistics AssetStats (r:1 w:0)
    // Proof Skipped: Statistics AssetStats (max_values: None, max_size: None, mode: Measured)
    /// The range of component `a` is `[0, 1]`.
    fn claim_ownership_restriction(a: u32) -> Weight {
        // Minimum execution time: 25_237 nanoseconds.
        Weight::from_ref_time(30_771_867)
            // Standard Error: 1_302_288
            .saturating_add(Weight::from_ref_time(14_866_465).saturating_mul(a.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(a.into())))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: Statistics AssetStats (r:2 w:2)
    // Proof Skipped: Statistics AssetStats (max_values: None, max_size: None, mode: Measured)
    /// The range of component `a` is `[0, 2]`.
    fn update_asset_count_stats(a: u32) -> Weight {
        // Minimum execution time: 25_138 nanoseconds.
        Weight::from_ref_time(30_302_252)
            // Standard Error: 444_465
            .saturating_add(Weight::from_ref_time(15_430_180).saturating_mul(a.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(a.into())))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(a.into())))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: Statistics AssetStats (r:2 w:2)
    // Proof Skipped: Statistics AssetStats (max_values: None, max_size: None, mode: Measured)
    /// The range of component `a` is `[0, 2]`.
    fn update_asset_balance_stats(a: u32) -> Weight {
        // Minimum execution time: 25_216 nanoseconds.
        Weight::from_ref_time(31_710_856)
            // Standard Error: 315_760
            .saturating_add(Weight::from_ref_time(13_909_552).saturating_mul(a.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(a.into())))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(a.into())))
    }
    /// The range of component `i` is `[0, 4]`.
    fn verify_requirements(i: u32) -> Weight {
        // Minimum execution time: 931 nanoseconds.
        Weight::from_ref_time(1_763_032)
            // Standard Error: 19_500
            .saturating_add(Weight::from_ref_time(104_808).saturating_mul(i.into()))
    }
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Proof Skipped: Statistics ActiveAssetStats (max_values: None, max_size: None, mode: Measured)
    /// The range of component `a` is `[1, 10]`.
    fn active_asset_statistics_load(a: u32) -> Weight {
        // Minimum execution time: 12_595 nanoseconds.
        Weight::from_ref_time(14_421_204)
            // Standard Error: 24_708
            .saturating_add(Weight::from_ref_time(23_673).saturating_mul(a.into()))
            .saturating_add(DbWeight::get().reads(1))
    }
    // Storage: Statistics TransferConditionExemptEntities (r:1 w:0)
    // Proof Skipped: Statistics TransferConditionExemptEntities (max_values: None, max_size: None, mode: Measured)
    fn is_exempt() -> Weight {
        // Minimum execution time: 14_657 nanoseconds.
        Weight::from_ref_time(15_569_000).saturating_add(DbWeight::get().reads(1))
    }
}
