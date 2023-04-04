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

//! Autogenerated weights for pallet_compliance_manager
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-25, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512
//! HOSTNAME: `dev-fsn001`, CPU: `AMD Ryzen 9 5950X 16-Core Processor`

// Executed Command:
// ./target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=pallet_compliance_manager
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

/// Weights for pallet_compliance_manager using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_compliance_manager::WeightInfo for SubstrateWeight {
    /// The range of component `a` is `[1, 10]`.
    /// The range of component `b` is `[1, 10]`.
    /// The range of component `c` is `[1, 10]`.
    /// The range of component `d` is `[1, 10]`.
    fn condition_costs(a: u32, b: u32, c: u32, d: u32) -> Weight {
        // Minimum execution time: 7_864 nanoseconds.
        Weight::from_ref_time(8_114_000)
            // Standard Error: 39_365
            .saturating_add(Weight::from_ref_time(2_949_096).saturating_mul(a.into()))
            // Standard Error: 39_365
            .saturating_add(Weight::from_ref_time(411_324).saturating_mul(b.into()))
            // Standard Error: 39_365
            .saturating_add(Weight::from_ref_time(671_782).saturating_mul(c.into()))
            // Manually set for `d`
            .saturating_add(Weight::from_ref_time(721_394).saturating_mul(d.into()))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:0)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    /// The range of component `c` is `[1, 6]`.
    fn add_compliance_requirement(c: u32) -> Weight {
        // Minimum execution time: 59_480 nanoseconds.
        Weight::from_ref_time(60_491_667)
            // Standard Error: 35_288
            .saturating_add(Weight::from_ref_time(1_128_590).saturating_mul(c.into()))
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    fn remove_compliance_requirement() -> Weight {
        // Minimum execution time: 48_980 nanoseconds.
        Weight::from_ref_time(50_152_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    fn pause_asset_compliance() -> Weight {
        // Minimum execution time: 50_974 nanoseconds.
        Weight::from_ref_time(51_204_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    fn resume_asset_compliance() -> Weight {
        // Minimum execution time: 47_508 nanoseconds.
        Weight::from_ref_time(47_998_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Identity DidRecords (r:1 w:0)
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:1)
    // Storage: ComplianceManager AssetCompliances (r:1 w:0)
    fn add_default_trusted_claim_issuer() -> Weight {
        // Minimum execution time: 54_821 nanoseconds.
        Weight::from_ref_time(55_422_000)
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:1)
    fn remove_default_trusted_claim_issuer() -> Weight {
        // Minimum execution time: 47_818 nanoseconds.
        Weight::from_ref_time(48_189_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:0)
    /// The range of component `c` is `[1, 6]`.
    fn change_compliance_requirement(c: u32) -> Weight {
        // Minimum execution time: 55_072 nanoseconds.
        Weight::from_ref_time(57_468_205)
            // Standard Error: 32_981
            .saturating_add(Weight::from_ref_time(683_464).saturating_mul(c.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    /// The range of component `c` is `[0, 2]`.
    fn replace_asset_compliance(c: u32) -> Weight {
        // Minimum execution time: 50_874 nanoseconds.
        Weight::from_ref_time(53_556_546)
            // Standard Error: 88_581
            .saturating_add(Weight::from_ref_time(6_514_529).saturating_mul(c.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:0 w:1)
    fn reset_asset_compliance() -> Weight {
        // Minimum execution time: 42_659 nanoseconds.
        Weight::from_ref_time(42_899_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Identity Claims (r:1 w:0)
    /// The range of component `c` is `[1, 400]`.
    /// The range of component `t` is `[0, 1]`.
    fn is_condition_satisfied(c: u32, t: u32) -> Weight {
        // Minimum execution time: 35_544 nanoseconds.
        Weight::from_ref_time(17_753_392)
            // Standard Error: 4_340
            .saturating_add(Weight::from_ref_time(4_628_473).saturating_mul(c.into()))
            // Standard Error: 1_146_187
            .saturating_add(Weight::from_ref_time(21_491_328).saturating_mul(t.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(c.into())))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(t.into())))
    }
    /// The range of component `e` is `[0, 1]`.
    fn is_identity_condition(e: u32) -> Weight {
        // Minimum execution time: 841 nanoseconds.
        Weight::from_ref_time(973_690)
            // Standard Error: 66_766
            .saturating_add(Weight::from_ref_time(19_986_642).saturating_mul(e.into()))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(e.into())))
    }
}
