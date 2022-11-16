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
//! DATE: 2022-11-16, STEPS: `10`, REPEAT: 3, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
pub struct WeightInfo;
impl polymesh_contracts::WeightInfo for WeightInfo {
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
        (567_398_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((12_000 as Weight).saturating_mul(k as Weight))
            // Standard Error: 2_000
            .saturating_add((2_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
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
        (364_617_000 as Weight)
            // Standard Error: 24_022_000
            .saturating_add((212_008_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
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
        (577_120_000 as Weight)
            // Standard Error: 2_444_000
            .saturating_add((660_215_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
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
        (836_971_000 as Weight)
            // Standard Error: 4_735_000
            .saturating_add((241_201_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
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
        (936_686_000 as Weight)
            // Standard Error: 352_000
            .saturating_add((73_557_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
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
        (708_772_000 as Weight)
            // Standard Error: 1_834_000
            .saturating_add((255_555_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
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
        (957_716_000 as Weight)
            // Standard Error: 393_000
            .saturating_add((82_985_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
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
        (672_244_000 as Weight)
            // Standard Error: 4_544_000
            .saturating_add((266_330_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
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
        (991_822_000 as Weight)
            // Standard Error: 1_046_000
            .saturating_add((101_495_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
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
    // Storage: Identity CurrentPayer (r:1 w:1)
    // Storage: Identity CurrentDid (r:1 w:1)
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    fn chain_extension_call_runtime(n: u32) -> Weight {
        (758_177_000 as Weight)
            // Standard Error: 0
            .saturating_add((5_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(16 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
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
        (467_775_000 as Weight)
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn basic_runtime_call(_n: u32) -> Weight {
        (1_432_000 as Weight)
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
        (833_434_000 as Weight)
            // Standard Error: 0
            .saturating_add((6_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(15 as Weight))
            .saturating_add(DbWeight::get().writes(9 as Weight))
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
        (1_995_183_000 as Weight)
            // Standard Error: 7_000
            .saturating_add((417_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 0
            .saturating_add((6_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(14 as Weight))
            .saturating_add(DbWeight::get().writes(10 as Weight))
    }
}
