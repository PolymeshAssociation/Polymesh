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

//! Autogenerated weights for pallet_external_agents
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

/// Weights for pallet_external_agents using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_external_agents::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ExternalAgents AGIdSequence (r:1 w:1)
    // Proof Skipped: ExternalAgents AGIdSequence (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupPermissions (r:0 w:1)
    // Proof Skipped: ExternalAgents GroupPermissions (max_values: None, max_size: None, mode: Measured)
    /// The range of component `p` is `[0, 19]`.
    fn create_group(p: u32) -> Weight {
        // Minimum execution time: 35_733 nanoseconds.
        Weight::from_ref_time(38_958_890)
            // Standard Error: 22_026
            .saturating_add(Weight::from_ref_time(598_029).saturating_mul(p.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ExternalAgents AGIdSequence (r:1 w:0)
    // Proof Skipped: ExternalAgents AGIdSequence (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupPermissions (r:0 w:1)
    // Proof Skipped: ExternalAgents GroupPermissions (max_values: None, max_size: None, mode: Measured)
    /// The range of component `p` is `[0, 19]`.
    fn set_group_permissions(p: u32) -> Weight {
        // Minimum execution time: 36_985 nanoseconds.
        Weight::from_ref_time(39_526_977)
            // Standard Error: 19_774
            .saturating_add(Weight::from_ref_time(673_117).saturating_mul(p.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:2 w:1)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ExternalAgents NumFullAgents (r:1 w:1)
    // Proof Skipped: ExternalAgents NumFullAgents (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents AgentOf (r:0 w:1)
    // Proof Skipped: ExternalAgents AgentOf (max_values: None, max_size: None, mode: Measured)
    fn remove_agent() -> Weight {
        // Minimum execution time: 44_036 nanoseconds.
        Weight::from_ref_time(45_248_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:1)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents NumFullAgents (r:1 w:1)
    // Proof Skipped: ExternalAgents NumFullAgents (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents AgentOf (r:0 w:1)
    // Proof Skipped: ExternalAgents AgentOf (max_values: None, max_size: None, mode: Measured)
    fn abdicate() -> Weight {
        // Minimum execution time: 36_846 nanoseconds.
        Weight::from_ref_time(37_346_000)
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:2 w:1)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ExternalAgents AGIdSequence (r:1 w:0)
    // Proof Skipped: ExternalAgents AGIdSequence (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents NumFullAgents (r:1 w:1)
    // Proof Skipped: ExternalAgents NumFullAgents (max_values: None, max_size: None, mode: Measured)
    fn change_group_custom() -> Weight {
        // Minimum execution time: 46_140 nanoseconds.
        Weight::from_ref_time(47_652_000)
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:2 w:1)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ExternalAgents NumFullAgents (r:1 w:1)
    // Proof Skipped: ExternalAgents NumFullAgents (max_values: None, max_size: None, mode: Measured)
    fn change_group_builtin() -> Weight {
        // Minimum execution time: 42_534 nanoseconds.
        Weight::from_ref_time(43_547_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:1 w:1)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity OutdatedAuthorizations (r:1 w:0)
    // Proof Skipped: Identity OutdatedAuthorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:2 w:1)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ExternalAgents NumFullAgents (r:1 w:1)
    // Proof Skipped: ExternalAgents NumFullAgents (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity NumberOfGivenAuths (r:1 w:1)
    // Proof Skipped: Identity NumberOfGivenAuths (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents AgentOf (r:0 w:1)
    // Proof Skipped: ExternalAgents AgentOf (max_values: None, max_size: None, mode: Measured)
    fn accept_become_agent() -> Weight {
        // Minimum execution time: 70_977 nanoseconds.
        Weight::from_ref_time(71_728_000)
            .saturating_add(DbWeight::get().reads(9))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ExternalAgents AGIdSequence (r:1 w:1)
    // Proof Skipped: ExternalAgents AGIdSequence (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity NumberOfGivenAuths (r:1 w:1)
    // Proof Skipped: Identity NumberOfGivenAuths (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CurrentAuthId (r:1 w:1)
    // Proof Skipped: Identity CurrentAuthId (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:0 w:1)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupPermissions (r:0 w:1)
    // Proof Skipped: ExternalAgents GroupPermissions (max_values: None, max_size: None, mode: Measured)
    /// The range of component `p` is `[0, 19]`.
    fn create_group_and_add_auth(p: u32) -> Weight {
        // Minimum execution time: 53_150 nanoseconds.
        Weight::from_ref_time(57_624_245)
            // Standard Error: 27_177
            .saturating_add(Weight::from_ref_time(595_641).saturating_mul(p.into()))
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupOfAgent (r:2 w:1)
    // Proof Skipped: ExternalAgents GroupOfAgent (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ExternalAgents AGIdSequence (r:1 w:1)
    // Proof Skipped: ExternalAgents AGIdSequence (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents NumFullAgents (r:1 w:1)
    // Proof Skipped: ExternalAgents NumFullAgents (max_values: None, max_size: None, mode: Measured)
    // Storage: ExternalAgents GroupPermissions (r:0 w:1)
    // Proof Skipped: ExternalAgents GroupPermissions (max_values: None, max_size: None, mode: Measured)
    /// The range of component `p` is `[0, 19]`.
    fn create_and_change_custom_group(p: u32) -> Weight {
        // Minimum execution time: 59_159 nanoseconds.
        Weight::from_ref_time(62_920_058)
            // Standard Error: 21_131
            .saturating_add(Weight::from_ref_time(562_056).saturating_mul(p.into()))
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().writes(4))
    }
}
