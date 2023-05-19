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
//! DATE: 2023-05-18, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    fn condition_costs(a: u32, _b: u32, c: u32, _d: u32) -> Weight {
        // Minimum execution time: 19_510 nanoseconds.
        Weight::from_ref_time(20_964_000)
            // Standard Error: 128_368
            .saturating_add(Weight::from_ref_time(6_124_168).saturating_mul(a.into()))
            // Standard Error: 128_368
            .saturating_add(Weight::from_ref_time(2_365_139).saturating_mul(c.into()))
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
        // Minimum execution time: 128_737 nanoseconds.
        Weight::from_ref_time(149_389_524)
            // Standard Error: 335_947
            .saturating_add(Weight::from_ref_time(1_437_636).saturating_mul(c.into()))
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    fn remove_compliance_requirement() -> Weight {
        // Minimum execution time: 109_256 nanoseconds.
        Weight::from_ref_time(113_328_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    fn pause_asset_compliance() -> Weight {
        // Minimum execution time: 118_766 nanoseconds.
        Weight::from_ref_time(122_861_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    fn resume_asset_compliance() -> Weight {
        // Minimum execution time: 109_644 nanoseconds.
        Weight::from_ref_time(145_490_000)
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
        // Minimum execution time: 128_138 nanoseconds.
        Weight::from_ref_time(131_570_000)
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:1)
    fn remove_default_trusted_claim_issuer() -> Weight {
        // Minimum execution time: 113_396 nanoseconds.
        Weight::from_ref_time(120_585_000)
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
    fn change_compliance_requirement(_c: u32) -> Weight {
        // Minimum execution time: 122_858 nanoseconds.
        Weight::from_ref_time(149_457_622)
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
        // Minimum execution time: 112_362 nanoseconds.
        Weight::from_ref_time(131_025_252)
            // Standard Error: 1_004_954
            .saturating_add(Weight::from_ref_time(17_305_506).saturating_mul(c.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:0 w:1)
    fn reset_asset_compliance() -> Weight {
        // Minimum execution time: 92_941 nanoseconds.
        Weight::from_ref_time(102_116_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(1))
    }
}
