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

//! Autogenerated weights for pallet_confidential_asset
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-19, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512
//! HOSTNAME: `comp002`, CPU: `Intel(R) Xeon(R) CPU E5-2697 v2 @ 2.70GHz`

// Executed Command:
// target/release/polymesh
// benchmark
// pallet
// -p=pallet_confidential_asset
// -e=*
// -s
// 100
// -r
// 5
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

/// Weights for pallet_confidential_asset using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_confidential_asset::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset MercatAccountBalance (r:1 w:1)
    // Storage: ConfidentialAsset MercatAccountDid (r:1 w:1)
    fn validate_mercat_account() -> Weight {
        // Minimum execution time: 3_267_268 nanoseconds.
        Weight::from_ref_time(3_354_277_000)
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset MediatorMercatAccounts (r:0 w:1)
    fn add_mediator_mercat_account() -> Weight {
        // Minimum execution time: 118_909 nanoseconds.
        Weight::from_ref_time(119_825_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset Details (r:1 w:1)
    fn create_confidential_asset() -> Weight {
        // Minimum execution time: 89_228 nanoseconds.
        Weight::from_ref_time(89_904_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset Details (r:1 w:1)
    // Storage: ConfidentialAsset MercatAccountBalance (r:1 w:1)
    fn mint_confidential_asset() -> Weight {
        // Minimum execution time: 6_016_725 nanoseconds.
        Weight::from_ref_time(6_189_188_000)
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset MercatAccountDid (r:1 w:0)
    // Storage: ConfidentialAsset IncomingBalance (r:1 w:0)
    fn apply_incoming_balance() -> Weight {
        // Minimum execution time: 78_954 nanoseconds.
        Weight::from_ref_time(100_611_000).saturating_add(DbWeight::get().reads(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset TransactionCounter (r:1 w:1)
    // Storage: ConfidentialAsset MercatAccountDid (r:2 w:0)
    // Storage: ConfidentialAsset PendingAffirms (r:0 w:1)
    // Storage: ConfidentialAsset UserAffirmations (r:0 w:3)
    // Storage: ConfidentialAsset TransactionLegs (r:0 w:1)
    fn add_transaction() -> Weight {
        // Minimum execution time: 210_502 nanoseconds.
        Weight::from_ref_time(222_561_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset TransactionLegs (r:1 w:0)
    // Storage: ConfidentialAsset UserAffirmations (r:1 w:1)
    // Storage: ConfidentialAsset MercatAccountDid (r:1 w:0)
    // Storage: ConfidentialAsset MercatAccountBalance (r:1 w:1)
    // Storage: ConfidentialAsset RngNonce (r:1 w:1)
    // Storage: Babe NextRandomness (r:1 w:0)
    // Storage: Babe EpochStart (r:1 w:0)
    // Storage: ConfidentialAsset PendingAffirms (r:1 w:1)
    // Storage: ConfidentialAsset SenderProofs (r:0 w:1)
    // Storage: ConfidentialAsset TxPendingState (r:0 w:1)
    fn sender_affirm_transaction() -> Weight {
        // Minimum execution time: 61_778_668 nanoseconds.
        Weight::from_ref_time(63_367_081_000)
            .saturating_add(DbWeight::get().reads(9))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset TransactionLegs (r:1 w:0)
    // Storage: ConfidentialAsset UserAffirmations (r:1 w:1)
    // Storage: ConfidentialAsset MercatAccountDid (r:1 w:0)
    // Storage: ConfidentialAsset PendingAffirms (r:1 w:1)
    fn receiver_affirm_transaction() -> Weight {
        // Minimum execution time: 113_528 nanoseconds.
        Weight::from_ref_time(144_399_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset TransactionLegs (r:1 w:0)
    // Storage: ConfidentialAsset UserAffirmations (r:1 w:1)
    // Storage: ConfidentialAsset PendingAffirms (r:1 w:1)
    fn mediator_affirm_transaction() -> Weight {
        // Minimum execution time: 98_428 nanoseconds.
        Weight::from_ref_time(114_076_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset UserAffirmations (r:1 w:1)
    // Storage: ConfidentialAsset TransactionLegs (r:1 w:0)
    // Storage: ConfidentialAsset MercatAccountDid (r:1 w:0)
    // Storage: ConfidentialAsset TxPendingState (r:1 w:1)
    // Storage: ConfidentialAsset MercatAccountBalance (r:1 w:1)
    // Storage: ConfidentialAsset PendingAffirms (r:1 w:1)
    // Storage: ConfidentialAsset SenderProofs (r:0 w:1)
    fn sender_unaffirm_transaction() -> Weight {
        // Minimum execution time: 534_873 nanoseconds.
        Weight::from_ref_time(558_880_000)
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().writes(5))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset UserAffirmations (r:1 w:1)
    // Storage: ConfidentialAsset TransactionLegs (r:1 w:0)
    // Storage: ConfidentialAsset MercatAccountDid (r:1 w:0)
    // Storage: ConfidentialAsset PendingAffirms (r:1 w:1)
    fn receiver_unaffirm_transaction() -> Weight {
        // Minimum execution time: 106_315 nanoseconds.
        Weight::from_ref_time(121_548_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset UserAffirmations (r:1 w:1)
    // Storage: ConfidentialAsset TransactionLegs (r:1 w:0)
    // Storage: ConfidentialAsset PendingAffirms (r:1 w:1)
    fn mediator_unaffirm_transaction() -> Weight {
        // Minimum execution time: 97_743 nanoseconds.
        Weight::from_ref_time(110_603_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset TransactionLegs (r:2 w:1)
    // Storage: ConfidentialAsset PendingAffirms (r:1 w:1)
    // Storage: ConfidentialAsset MercatAccountDid (r:2 w:0)
    // Storage: ConfidentialAsset UserAffirmations (r:3 w:3)
    // Storage: ConfidentialAsset TxPendingState (r:1 w:1)
    // Storage: ConfidentialAsset IncomingBalance (r:1 w:1)
    // Storage: ConfidentialAsset SenderProofs (r:0 w:1)
    fn execute_transaction(_l: u32) -> Weight {
        // Minimum execution time: 502_724 nanoseconds.
        Weight::from_ref_time(551_013_000)
            .saturating_add(DbWeight::get().reads(11))
            .saturating_add(DbWeight::get().writes(8))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ConfidentialAsset TransactionLegs (r:2 w:1)
    // Storage: ConfidentialAsset MercatAccountDid (r:2 w:0)
    // Storage: ConfidentialAsset UserAffirmations (r:1 w:3)
    // Storage: ConfidentialAsset TxPendingState (r:1 w:1)
    // Storage: ConfidentialAsset IncomingBalance (r:1 w:1)
    // Storage: ConfidentialAsset PendingAffirms (r:0 w:1)
    // Storage: ConfidentialAsset SenderProofs (r:0 w:1)
    fn revert_transaction(_l: u32) -> Weight {
        // Minimum execution time: 490_066 nanoseconds.
        Weight::from_ref_time(494_972_000)
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(8))
    }
}
