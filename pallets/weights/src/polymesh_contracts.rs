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

//! Autogenerated weights for polymesh_contracts
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

/// Weights for polymesh_contracts using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl polymesh_contracts::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: unknown `0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f` (r:1 w:0)
    // Proof Skipped: unknown `0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f` (r:1 w:0)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    // Storage: unknown `0x00` (r:1 w:0)
    // Proof Skipped: unknown `0x00` (r:1 w:0)
    /// The range of component `k` is `[1, 8192]`.
    /// The range of component `v` is `[1, 8192]`.
    fn chain_extension_read_storage(k: u32, v: u32) -> Weight {
        // Minimum execution time: 619_920 nanoseconds.
        Weight::from_ref_time(812_486_598)
            // Standard Error: 3_268
            .saturating_add(Weight::from_ref_time(20_120).saturating_mul(k.into()))
            // Standard Error: 3_268
            .saturating_add(Weight::from_ref_time(181).saturating_mul(v.into()))
            .saturating_add(DbWeight::get().reads(13))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `r` is `[0, 20]`.
    fn chain_extension_get_version(r: u32) -> Weight {
        // Minimum execution time: 615_893 nanoseconds.
        Weight::from_ref_time(754_719_943)
            // Standard Error: 1_456_232
            .saturating_add(Weight::from_ref_time(115_242_611).saturating_mul(r.into()))
            .saturating_add(DbWeight::get().reads(12))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:2002 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:2000 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `r` is `[1, 20]`.
    fn chain_extension_get_key_did(r: u32) -> Weight {
        // Minimum execution time: 230_114_055 nanoseconds.
        Weight::from_ref_time(234_042_835_000)
            // Standard Error: 645_494_782
            .saturating_add(Weight::from_ref_time(219_847_004_564).saturating_mul(r.into()))
            .saturating_add(DbWeight::get().reads(11))
            .saturating_add(DbWeight::get().reads((200_u64).saturating_mul(r.into())))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `r` is `[0, 20]`.
    fn chain_extension_hash_twox_64(r: u32) -> Weight {
        // Minimum execution time: 580_686 nanoseconds.
        Weight::from_ref_time(801_258_758)
            // Standard Error: 2_177_621
            .saturating_add(Weight::from_ref_time(137_654_299).saturating_mul(r.into()))
            .saturating_add(DbWeight::get().reads(12))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `n` is `[0, 64]`.
    fn chain_extension_hash_twox_64_per_kb(n: u32) -> Weight {
        // Minimum execution time: 917_396 nanoseconds.
        Weight::from_ref_time(1_010_359_519)
            // Standard Error: 1_144_270
            .saturating_add(Weight::from_ref_time(45_436_748).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(12))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `r` is `[0, 20]`.
    fn chain_extension_hash_twox_128(r: u32) -> Weight {
        // Minimum execution time: 575_035 nanoseconds.
        Weight::from_ref_time(538_876_467)
            // Standard Error: 2_063_310
            .saturating_add(Weight::from_ref_time(169_196_200).saturating_mul(r.into()))
            .saturating_add(DbWeight::get().reads(12))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `n` is `[0, 64]`.
    fn chain_extension_hash_twox_128_per_kb(n: u32) -> Weight {
        // Minimum execution time: 826_787 nanoseconds.
        Weight::from_ref_time(1_059_756_444)
            // Standard Error: 1_207_008
            .saturating_add(Weight::from_ref_time(54_174_736).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(12))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `r` is `[0, 20]`.
    fn chain_extension_hash_twox_256(r: u32) -> Weight {
        // Minimum execution time: 565_497 nanoseconds.
        Weight::from_ref_time(694_899_046)
            // Standard Error: 1_392_828
            .saturating_add(Weight::from_ref_time(147_484_378).saturating_mul(r.into()))
            .saturating_add(DbWeight::get().reads(12))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `n` is `[0, 64]`.
    fn chain_extension_hash_twox_256_per_kb(n: u32) -> Weight {
        // Minimum execution time: 738_701 nanoseconds.
        Weight::from_ref_time(1_045_254_531)
            // Standard Error: 1_037_437
            .saturating_add(Weight::from_ref_time(67_628_541).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(12))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: Contracts CallRuntimeWhitelist (r:1 w:0)
    // Proof Skipped: Contracts CallRuntimeWhitelist (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CurrentPayer (r:1 w:1)
    // Proof Skipped: Identity CurrentPayer (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity CurrentDid (r:1 w:1)
    // Proof Skipped: Identity CurrentDid (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `n` is `[1, 8188]`.
    fn chain_extension_call_runtime(n: u32) -> Weight {
        // Minimum execution time: 625_560 nanoseconds.
        Weight::from_ref_time(861_333_120)
            // Standard Error: 4_278
            .saturating_add(Weight::from_ref_time(10_976).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(17))
            .saturating_add(DbWeight::get().writes(7))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    fn dummy_contract() -> Weight {
        // Minimum execution time: 369_892 nanoseconds.
        Weight::from_ref_time(379_158_000)
            .saturating_add(DbWeight::get().reads(12))
            .saturating_add(DbWeight::get().writes(3))
    }
    /// The range of component `n` is `[1, 8188]`.
    fn basic_runtime_call(n: u32) -> Weight {
        // Minimum execution time: 4_077 nanoseconds.
        Weight::from_ref_time(5_718_532)
            // Manually set weight for `n`
            .saturating_add(Weight::from_ref_time(11).saturating_mul(n.into()))
    }
    // Storage: Identity KeyRecords (r:3 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: System Account (r:3 w:3)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts Nonce (r:1 w:1)
    // Proof: Contracts Nonce (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: Contracts OwnerInfoOf (r:1 w:1)
    // Proof: Contracts OwnerInfoOf (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    /// The range of component `s` is `[0, 1048576]`.
    fn instantiate_with_hash_perms(s: u32) -> Weight {
        // Minimum execution time: 583_581 nanoseconds.
        Weight::from_ref_time(786_363_174)
            // Standard Error: 78
            .saturating_add(Weight::from_ref_time(5_152).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(17))
            .saturating_add(DbWeight::get().writes(10))
    }
    // Storage: Identity KeyRecords (r:3 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Contracts OwnerInfoOf (r:1 w:1)
    // Proof: Contracts OwnerInfoOf (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
    // Storage: System Account (r:3 w:3)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts Nonce (r:1 w:1)
    // Proof: Contracts Nonce (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:3 w:3)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    // Storage: Contracts CodeStorage (r:0 w:1)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Contracts PristineCode (r:0 w:1)
    // Proof: Contracts PristineCode (max_values: None, max_size: Some(125988), added: 128463, mode: MaxEncodedLen)
    /// The range of component `c` is `[0, 61717]`.
    /// The range of component `s` is `[0, 1048576]`.
    fn instantiate_with_code_perms(c: u32, s: u32) -> Weight {
        // Minimum execution time: 5_494_994 nanoseconds.
        Weight::from_ref_time(1_064_003_274)
            // Standard Error: 2_708
            .saturating_add(Weight::from_ref_time(248_194).saturating_mul(c.into()))
            // Standard Error: 159
            .saturating_add(Weight::from_ref_time(4_992).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(17))
            .saturating_add(DbWeight::get().writes(13))
    }
    // Storage: Contracts CallRuntimeWhitelist (r:0 w:2000)
    // Proof Skipped: Contracts CallRuntimeWhitelist (max_values: None, max_size: None, mode: Measured)
    /// The range of component `u` is `[0, 2000]`.
    fn update_call_runtime_whitelist(u: u32) -> Weight {
        // Minimum execution time: 7_734 nanoseconds.
        Weight::from_ref_time(8_015_000)
            // Standard Error: 18_625
            .saturating_add(Weight::from_ref_time(2_571_524).saturating_mul(u.into()))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(u.into())))
    }
    // Storage: Identity KeyRecords (r:3 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity ParentDid (r:1 w:1)
    // Proof Skipped: Identity ParentDid (max_values: None, max_size: None, mode: Measured)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Proof Skipped: ProtocolFee Coefficient (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Proof Skipped: ProtocolFee BaseFees (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Proof Skipped: Identity MultiPurposeNonce (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: System ParentHash (r:1 w:0)
    // Proof: System ParentHash (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
    // Storage: Identity DidRecords (r:1 w:1)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Contracts OwnerInfoOf (r:1 w:1)
    // Proof: Contracts OwnerInfoOf (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
    // Storage: System Account (r:3 w:3)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts Nonce (r:1 w:1)
    // Proof: Contracts Nonce (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:3 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:3 w:3)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    // Storage: Contracts CodeStorage (r:0 w:1)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Contracts PristineCode (r:0 w:1)
    // Proof: Contracts PristineCode (max_values: None, max_size: Some(125988), added: 128463, mode: MaxEncodedLen)
    /// The range of component `c` is `[0, 61717]`.
    /// The range of component `s` is `[0, 1048576]`.
    fn instantiate_with_code_as_primary_key(c: u32, s: u32) -> Weight {
        // Minimum execution time: 4_335_262 nanoseconds.
        Weight::from_ref_time(622_167_188)
            // Standard Error: 340
            .saturating_add(Weight::from_ref_time(134_179).saturating_mul(c.into()))
            // Standard Error: 20
            .saturating_add(Weight::from_ref_time(3_699).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(23))
            .saturating_add(DbWeight::get().writes(16))
    }
    // Storage: Identity KeyRecords (r:3 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity ParentDid (r:1 w:1)
    // Proof Skipped: Identity ParentDid (max_values: None, max_size: None, mode: Measured)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Proof Skipped: ProtocolFee Coefficient (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Proof Skipped: ProtocolFee BaseFees (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Proof Skipped: Identity MultiPurposeNonce (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: System ParentHash (r:1 w:0)
    // Proof: System ParentHash (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
    // Storage: Identity DidRecords (r:1 w:1)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: System Account (r:3 w:3)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts Nonce (r:1 w:1)
    // Proof: Contracts Nonce (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:3 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: Contracts OwnerInfoOf (r:1 w:1)
    // Proof: Contracts OwnerInfoOf (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    /// The range of component `s` is `[0, 1048576]`.
    fn instantiate_with_hash_as_primary_key(s: u32) -> Weight {
        // Minimum execution time: 532_686 nanoseconds.
        Weight::from_ref_time(514_127_182)
            // Standard Error: 7
            .saturating_add(Weight::from_ref_time(3_623).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(23))
            .saturating_add(DbWeight::get().writes(13))
    }
    // Storage: PolymeshContracts SupportedApiUpgrades (r:0 w:1)
    // Proof Skipped: PolymeshContracts SupportedApiUpgrades (max_values: None, max_size: None, mode: Measured)
    fn upgrade_api() -> Weight {
        // Minimum execution time: 20_304 nanoseconds.
        Weight::from_ref_time(20_633_000).saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: System Account (r:1 w:0)
    // Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Proof: Contracts ContractInfoOf (max_values: None, max_size: Some(290), added: 2765, mode: MaxEncodedLen)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Proof: Contracts CodeStorage (max_values: None, max_size: Some(126001), added: 128476, mode: MaxEncodedLen)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: PolymeshContracts SupportedApiUpgrades (r:22 w:0)
    // Proof Skipped: PolymeshContracts SupportedApiUpgrades (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `r` is `[0, 20]`.
    fn chain_extension_get_latest_api_upgrade(r: u32) -> Weight {
        // Minimum execution time: 473_093 nanoseconds.
        Weight::from_ref_time(474_569_000)
            // Standard Error: 48_456_568
            .saturating_add(Weight::from_ref_time(3_902_882_799).saturating_mul(r.into()))
            .saturating_add(DbWeight::get().reads(14))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(DbWeight::get().writes(3))
    }
}
