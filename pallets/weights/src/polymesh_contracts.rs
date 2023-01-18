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
//! DATE: 2022-11-17, STEPS: `10`, REPEAT: 3, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// target/release/polymesh
// benchmark
// pallet
// -p=polymesh_contracts
// -e=*
// -s
// 10
// -r
// 3
// --db-cache
// 512
// --heap-pages
// 4096
// --execution
// wasm
// --wasm-execution
// compiled
// --output
// ./
// --template
// ./.maintain/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

/// Weights for polymesh_contracts using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl polymesh_contracts::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    // Storage: unknown [0x00] (r:1 w:0)
    // Storage: unknown [0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f] (r:1 w:0)
    fn chain_extension_read_storage(k: u32, v: u32) -> Weight {
        Weight::from_ref_time(616_785_000 as u64)
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(8_000 as u64).saturating_mul(k as u64))
            // Standard Error: 2_000
            .saturating_add(Weight::from_ref_time(2_000 as u64).saturating_mul(v as u64))
            .saturating_add(DbWeight::get().reads(13 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    fn chain_extension_get_version(r: u32) -> Weight {
        Weight::from_ref_time(363_157_000 as u64)
            // Standard Error: 20_038_000
            .saturating_add(Weight::from_ref_time(224_003_000 as u64).saturating_mul(r as u64))
            .saturating_add(DbWeight::get().reads(12 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:3 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    fn chain_extension_get_key_did(r: u32) -> Weight {
        Weight::from_ref_time(616_219_000 as u64)
            // Standard Error: 11_686_000
            .saturating_add(Weight::from_ref_time(554_712_000 as u64).saturating_mul(r as u64))
            .saturating_add(DbWeight::get().reads(13 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    fn chain_extension_hash_twox_64(r: u32) -> Weight {
        Weight::from_ref_time(302_444_000 as u64)
            // Standard Error: 6_140_000
            .saturating_add(Weight::from_ref_time(261_925_000 as u64).saturating_mul(r as u64))
            .saturating_add(DbWeight::get().reads(12 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    fn chain_extension_hash_twox_64_per_kb(n: u32) -> Weight {
        Weight::from_ref_time(960_492_000 as u64)
            // Standard Error: 204_000
            .saturating_add(Weight::from_ref_time(73_699_000 as u64).saturating_mul(n as u64))
            .saturating_add(DbWeight::get().reads(12 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    fn chain_extension_hash_twox_128(r: u32) -> Weight {
        Weight::from_ref_time(611_259_000 as u64)
            // Standard Error: 9_446_000
            .saturating_add(Weight::from_ref_time(269_304_000 as u64).saturating_mul(r as u64))
            .saturating_add(DbWeight::get().reads(12 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    fn chain_extension_hash_twox_128_per_kb(n: u32) -> Weight {
        Weight::from_ref_time(972_907_000 as u64)
            // Standard Error: 258_000
            .saturating_add(Weight::from_ref_time(83_521_000 as u64).saturating_mul(n as u64))
            .saturating_add(DbWeight::get().reads(12 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    fn chain_extension_hash_twox_256(r: u32) -> Weight {
        Weight::from_ref_time(705_382_000 as u64)
            // Standard Error: 723_000
            .saturating_add(Weight::from_ref_time(255_173_000 as u64).saturating_mul(r as u64))
            .saturating_add(DbWeight::get().reads(12 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    fn chain_extension_hash_twox_256_per_kb(n: u32) -> Weight {
        Weight::from_ref_time(975_225_000 as u64)
            // Standard Error: 327_000
            .saturating_add(Weight::from_ref_time(102_619_000 as u64).saturating_mul(n as u64))
            .saturating_add(DbWeight::get().reads(12 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    // Storage: Contracts CallRuntimeWhitelist (r:1 w:0)
    // Storage: Identity CurrentPayer (r:1 w:1)
    // Storage: Identity CurrentDid (r:1 w:1)
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    fn chain_extension_call_runtime(n: u32) -> Weight {
        Weight::from_ref_time(790_893_000 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(4_000 as u64).saturating_mul(n as u64))
            .saturating_add(DbWeight::get().reads(17 as u64))
            .saturating_add(DbWeight::get().writes(6 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:0)
    // Storage: System Account (r:1 w:0)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    fn dummy_contract() -> Weight {
        Weight::from_ref_time(468_460_000 as u64)
            .saturating_add(DbWeight::get().reads(12 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    fn basic_runtime_call(_n: u32) -> Weight {
        Weight::from_ref_time(1_440_000 as u64)
    }
    // Storage: Identity KeyRecords (r:2 w:1)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:1)
    // Storage: System Account (r:2 w:2)
    // Storage: Contracts Nonce (r:1 w:1)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    // Storage: Contracts OwnerInfoOf (r:1 w:1)
    // Storage: Identity DidKeys (r:0 w:1)
    fn instantiate_with_hash_perms(s: u32) -> Weight {
        Weight::from_ref_time(797_786_000 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(6_000 as u64).saturating_mul(s as u64))
            .saturating_add(DbWeight::get().reads(15 as u64))
            .saturating_add(DbWeight::get().writes(9 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:1)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Contracts CodeStorage (r:1 w:1)
    // Storage: System Account (r:2 w:2)
    // Storage: Contracts Nonce (r:1 w:1)
    // Storage: Contracts ContractInfoOf (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Identity IsDidFrozen (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    // Storage: Identity DidKeys (r:0 w:1)
    // Storage: Contracts PristineCode (r:0 w:1)
    // Storage: Contracts OwnerInfoOf (r:0 w:1)
    fn instantiate_with_code_perms(c: u32, s: u32) -> Weight {
        Weight::from_ref_time(693_683_000 as u64)
            // Standard Error: 9_000
            .saturating_add(Weight::from_ref_time(438_000 as u64).saturating_mul(c as u64))
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(7_000 as u64).saturating_mul(s as u64))
            .saturating_add(DbWeight::get().reads(14 as u64))
            .saturating_add(DbWeight::get().writes(10 as u64))
    }
    // Storage: Contracts CallRuntimeWhitelist (r:0 w:200)
    fn update_call_runtime_whitelist(u: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 403_000
            .saturating_add(Weight::from_ref_time(2_749_000 as u64).saturating_mul(u as u64))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(u as u64)))
    }
}
