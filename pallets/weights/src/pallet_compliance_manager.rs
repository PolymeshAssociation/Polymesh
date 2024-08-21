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

/// Weights for pallet_compliance_manager using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_compliance_manager::WeightInfo for SubstrateWeight {
    /// The range of component `a` is `[1, 10]`.
    /// The range of component `b` is `[1, 10]`.
    /// The range of component `c` is `[1, 10]`.
    /// The range of component `d` is `[1, 10]`.
    fn condition_costs(a: u32, b: u32, c: u32, _d: u32) -> Weight {
        // Minimum execution time: 10_465 nanoseconds.
        Weight::from_ref_time(11_057_000)
            // Standard Error: 69_428
            .saturating_add(Weight::from_ref_time(4_673_715).saturating_mul(a.into()))
            // Standard Error: 69_428
            .saturating_add(Weight::from_ref_time(235_269).saturating_mul(b.into()))
            // Standard Error: 69_428
            .saturating_add(Weight::from_ref_time(1_231_611).saturating_mul(c.into()))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    // Proof Skipped: ComplianceManager AssetCompliances (max_values: None, max_size: None, mode: Measured)
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:0)
    // Proof Skipped: ComplianceManager TrustedClaimIssuer (max_values: None, max_size: None, mode: Measured)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Proof Skipped: ProtocolFee Coefficient (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Proof Skipped: ProtocolFee BaseFees (max_values: None, max_size: None, mode: Measured)
    /// The range of component `c` is `[1, 6]`.
    fn add_compliance_requirement(c: u32) -> Weight {
        // Minimum execution time: 56_006 nanoseconds.
        Weight::from_ref_time(61_051_689)
            // Standard Error: 137_468
            .saturating_add(Weight::from_ref_time(1_139_215).saturating_mul(c.into()))
            .saturating_add(DbWeight::get().reads(8))
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
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    // Proof Skipped: ComplianceManager AssetCompliances (max_values: None, max_size: None, mode: Measured)
    fn remove_compliance_requirement() -> Weight {
        // Minimum execution time: 43_766 nanoseconds.
        Weight::from_ref_time(46_069_000)
            .saturating_add(DbWeight::get().reads(5))
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
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    // Proof Skipped: ComplianceManager AssetCompliances (max_values: None, max_size: None, mode: Measured)
    fn pause_asset_compliance() -> Weight {
        // Minimum execution time: 44_347 nanoseconds.
        Weight::from_ref_time(46_119_000)
            .saturating_add(DbWeight::get().reads(5))
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
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    // Proof Skipped: ComplianceManager AssetCompliances (max_values: None, max_size: None, mode: Measured)
    fn resume_asset_compliance() -> Weight {
        // Minimum execution time: 41_192 nanoseconds.
        Weight::from_ref_time(41_453_000)
            .saturating_add(DbWeight::get().reads(5))
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
    // Storage: Identity DidRecords (r:1 w:0)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:1)
    // Proof Skipped: ComplianceManager TrustedClaimIssuer (max_values: None, max_size: None, mode: Measured)
    // Storage: ComplianceManager AssetCompliances (r:1 w:0)
    // Proof Skipped: ComplianceManager AssetCompliances (max_values: None, max_size: None, mode: Measured)
    fn add_default_trusted_claim_issuer() -> Weight {
        // Minimum execution time: 52_790 nanoseconds.
        Weight::from_ref_time(53_891_000)
            .saturating_add(DbWeight::get().reads(7))
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
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:1)
    // Proof Skipped: ComplianceManager TrustedClaimIssuer (max_values: None, max_size: None, mode: Measured)
    fn remove_default_trusted_claim_issuer() -> Weight {
        // Minimum execution time: 39_421 nanoseconds.
        Weight::from_ref_time(41_704_000)
            .saturating_add(DbWeight::get().reads(5))
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
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    // Proof Skipped: ComplianceManager AssetCompliances (max_values: None, max_size: None, mode: Measured)
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:0)
    // Proof Skipped: ComplianceManager TrustedClaimIssuer (max_values: None, max_size: None, mode: Measured)
    /// The range of component `c` is `[1, 6]`.
    fn change_compliance_requirement(c: u32) -> Weight {
        // Minimum execution time: 49_764 nanoseconds.
        Weight::from_ref_time(52_634_069)
            // Standard Error: 110_686
            .saturating_add(Weight::from_ref_time(1_691_278).saturating_mul(c.into()))
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
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:0)
    // Proof Skipped: ComplianceManager TrustedClaimIssuer (max_values: None, max_size: None, mode: Measured)
    // Storage: ComplianceManager AssetCompliances (r:1 w:1)
    // Proof Skipped: ComplianceManager AssetCompliances (max_values: None, max_size: None, mode: Measured)
    /// The range of component `c` is `[0, 2]`.
    fn replace_asset_compliance(c: u32) -> Weight {
        // Minimum execution time: 46_871 nanoseconds.
        Weight::from_ref_time(50_668_128)
            // Standard Error: 257_332
            .saturating_add(Weight::from_ref_time(10_897_810).saturating_mul(c.into()))
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
    // Storage: ComplianceManager AssetCompliances (r:0 w:1)
    // Proof Skipped: ComplianceManager AssetCompliances (max_values: None, max_size: None, mode: Measured)
    fn reset_asset_compliance() -> Weight {
        // Minimum execution time: 32_791 nanoseconds.
        Weight::from_ref_time(35_844_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: ComplianceManager TrustedClaimIssuer (r:1 w:0)
    // Proof Skipped: ComplianceManager TrustedClaimIssuer (max_values: None, max_size: None, mode: Measured)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity Claims (r:400 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    /// The range of component `c` is `[1, 400]`.
    /// The range of component `t` is `[0, 1]`.
    fn is_condition_satisfied(c: u32, t: u32) -> Weight {
        // Minimum execution time: 18_998 nanoseconds.
        Weight::from_ref_time(19_399_000)
            // Standard Error: 1_934
            .saturating_add(Weight::from_ref_time(4_582_996).saturating_mul(c.into()))
            // Standard Error: 834_540
            .saturating_add(Weight::from_ref_time(8_310_961).saturating_mul(t.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(c.into())))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(t.into())))
    }
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    /// The range of component `e` is `[0, 1]`.
    fn is_identity_condition(e: u32) -> Weight {
        // Minimum execution time: 661 nanoseconds.
        Weight::from_ref_time(1_004_911)
            // Standard Error: 67_418
            .saturating_add(Weight::from_ref_time(8_869_755).saturating_mul(e.into()))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(e.into())))
    }
    /// The range of component `i` is `[0, 10000]`.
    fn is_any_requirement_compliant(i: u32) -> Weight {
        // Minimum execution time: 541 nanoseconds.
        Weight::from_ref_time(5_217_705)
            // Standard Error: 188
            .saturating_add(Weight::from_ref_time(114_974).saturating_mul(i.into()))
    }
}
