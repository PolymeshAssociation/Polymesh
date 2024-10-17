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
        // Minimum execution time: 440_006 nanoseconds.
        Weight::from_ref_time(459_066_226)
            // Standard Error: 251
            .saturating_add(Weight::from_ref_time(4_684).saturating_mul(k.into()))
            // Standard Error: 251
            .saturating_add(Weight::from_ref_time(869).saturating_mul(v.into()))
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
        // Minimum execution time: 421_918 nanoseconds.
        Weight::from_ref_time(445_548_305)
            // Standard Error: 106_573
            .saturating_add(Weight::from_ref_time(60_714_063).saturating_mul(r.into()))
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
    // Storage: Identity IsDidFrozen (r:2001 w:0)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `r` is `[1, 20]`.
    fn chain_extension_get_key_did(r: u32) -> Weight {
        // Minimum execution time: 1_223_281 nanoseconds.
        Weight::from_ref_time(209_767_426)
            // Standard Error: 1_452_883
            .saturating_add(Weight::from_ref_time(848_618_840).saturating_mul(r.into()))
            .saturating_add(DbWeight::get().reads(12))
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
        // Minimum execution time: 418_624 nanoseconds.
        Weight::from_ref_time(442_196_222)
            // Standard Error: 117_761
            .saturating_add(Weight::from_ref_time(76_220_639).saturating_mul(r.into()))
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
        // Minimum execution time: 498_565 nanoseconds.
        Weight::from_ref_time(558_442_400)
            // Standard Error: 99_195
            .saturating_add(Weight::from_ref_time(28_524_419).saturating_mul(n.into()))
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
        // Minimum execution time: 413_104 nanoseconds.
        Weight::from_ref_time(430_704_512)
            // Standard Error: 290_306
            .saturating_add(Weight::from_ref_time(79_866_487).saturating_mul(r.into()))
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
        // Minimum execution time: 499_916 nanoseconds.
        Weight::from_ref_time(560_288_569)
            // Standard Error: 104_136
            .saturating_add(Weight::from_ref_time(35_166_712).saturating_mul(n.into()))
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
        // Minimum execution time: 416_029 nanoseconds.
        Weight::from_ref_time(450_803_968)
            // Standard Error: 290_811
            .saturating_add(Weight::from_ref_time(82_861_097).saturating_mul(r.into()))
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
        // Minimum execution time: 527_438 nanoseconds.
        Weight::from_ref_time(570_747_114)
            // Standard Error: 94_735
            .saturating_add(Weight::from_ref_time(48_739_942).saturating_mul(n.into()))
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
    // Storage: PolymeshContracts CallRuntimeWhitelist (r:1 w:0)
    // Proof Skipped: PolymeshContracts CallRuntimeWhitelist (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CurrentPayer (r:1 w:1)
    // Proof Skipped: Identity CurrentPayer (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `n` is `[1, 8188]`.
    fn chain_extension_call_runtime(n: u32) -> Weight {
        // Minimum execution time: 454_277 nanoseconds.
        Weight::from_ref_time(482_341_570)
            // Standard Error: 339
            .saturating_add(Weight::from_ref_time(978).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(16))
            .saturating_add(DbWeight::get().writes(6))
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
        // Minimum execution time: 253_353 nanoseconds.
        Weight::from_ref_time(259_583_000)
            .saturating_add(DbWeight::get().reads(12))
            .saturating_add(DbWeight::get().writes(3))
    }
    /// The range of component `n` is `[1, 8188]`.
    fn basic_runtime_call(_n: u32) -> Weight {
        // Minimum execution time: 2_093 nanoseconds.
        Weight::from_ref_time(2_909_155)
    }
    /// The range of component `i` is `[0, 1048576]`.
    /// The range of component `s` is `[0, 1048576]`.
    fn base_weight_with_hash(i: u32, s: u32) -> Weight {
        // Minimum execution time: 960_746 nanoseconds.
        Weight::from_ref_time(40_570_806)
            // Standard Error: 2
            .saturating_add(Weight::from_ref_time(867).saturating_mul(i.into()))
            // Standard Error: 2
            .saturating_add(Weight::from_ref_time(955).saturating_mul(s.into()))
    }
    /// The range of component `c` is `[0, 61717]`.
    /// The range of component `i` is `[0, 1048576]`.
    /// The range of component `s` is `[0, 1048576]`.
    fn base_weight_with_code(c: u32, i: u32, s: u32) -> Weight {
        // Minimum execution time: 1_007_560 nanoseconds.
        Weight::from_ref_time(17_914_552)
            // Standard Error: 45
            .saturating_add(Weight::from_ref_time(1_330).saturating_mul(c.into()))
            // Standard Error: 2
            .saturating_add(Weight::from_ref_time(864).saturating_mul(i.into()))
            // Standard Error: 2
            .saturating_add(Weight::from_ref_time(951).saturating_mul(s.into()))
    }
    // Storage: Identity KeyRecords (r:2 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyPortfolioPermissions (r:0 w:1)
    // Proof Skipped: Identity KeyPortfolioPermissions (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyExtrinsicPermissions (r:0 w:1)
    // Proof Skipped: Identity KeyExtrinsicPermissions (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyAssetPermissions (r:0 w:1)
    // Proof Skipped: Identity KeyAssetPermissions (max_values: None, max_size: None, mode: Measured)
    fn link_contract_as_secondary_key() -> Weight {
        // Minimum execution time: 18_033 nanoseconds.
        Weight::from_ref_time(18_404_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(5))
    }
    // Storage: Identity KeyRecords (r:2 w:1)
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
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity ChildDid (r:0 w:1)
    // Proof Skipped: Identity ChildDid (max_values: None, max_size: None, mode: Measured)
    fn link_contract_as_primary_key() -> Weight {
        // Minimum execution time: 32_310 nanoseconds.
        Weight::from_ref_time(32_881_000)
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: PolymeshContracts CallRuntimeWhitelist (r:0 w:2000)
    // Proof Skipped: PolymeshContracts CallRuntimeWhitelist (max_values: None, max_size: None, mode: Measured)
    /// The range of component `u` is `[0, 2000]`.
    fn update_call_runtime_whitelist(u: u32) -> Weight {
        // Minimum execution time: 3_826 nanoseconds.
        Weight::from_ref_time(4_076_000)
            // Standard Error: 2_097
            .saturating_add(Weight::from_ref_time(1_640_807).saturating_mul(u.into()))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(u.into())))
    }
    // Storage: PolymeshContracts ApiNextUpgrade (r:0 w:1)
    // Proof Skipped: PolymeshContracts ApiNextUpgrade (max_values: None, max_size: None, mode: Measured)
    fn upgrade_api() -> Weight {
        // Minimum execution time: 11_718 nanoseconds.
        Weight::from_ref_time(12_259_000).saturating_add(DbWeight::get().writes(1))
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
    // Storage: PolymeshContracts CurrentApiHash (r:1 w:1)
    // Proof Skipped: PolymeshContracts CurrentApiHash (max_values: None, max_size: None, mode: Measured)
    // Storage: PolymeshContracts ApiNextUpgrade (r:1 w:1)
    // Proof Skipped: PolymeshContracts ApiNextUpgrade (max_values: None, max_size: None, mode: Measured)
    // Storage: System EventTopics (r:2 w:2)
    // Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
    /// The range of component `r` is `[0, 20]`.
    fn chain_extension_get_latest_api_upgrade(r: u32) -> Weight {
        // Minimum execution time: 412_925 nanoseconds.
        Weight::from_ref_time(444_988_600)
            // Standard Error: 190_111
            .saturating_add(Weight::from_ref_time(362_396_778).saturating_mul(r.into()))
            .saturating_add(DbWeight::get().reads(14))
            .saturating_add(DbWeight::get().writes(5))
    }
}
