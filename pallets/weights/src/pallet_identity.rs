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

//! Autogenerated weights for pallet_identity
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

/// Weights for pallet_identity using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_identity::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:2 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity ParentDid (r:1 w:1)
    // Proof Skipped: Identity ParentDid (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AccountKeyRefCount (r:1 w:0)
    // Proof Skipped: Identity AccountKeyRefCount (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigToIdentity (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Proof Skipped: Identity MultiPurposeNonce (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: System ParentHash (r:1 w:0)
    // Proof: System ParentHash (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
    // Storage: Identity DidRecords (r:1 w:1)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:2)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    fn create_child_identity() -> Weight {
        // Minimum execution time: 103_173 nanoseconds.
        Weight::from_ref_time(119_804_000)
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: Identity KeyRecords (r:100 w:99)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity ParentDid (r:1 w:99)
    // Proof Skipped: Identity ParentDid (max_values: None, max_size: None, mode: Measured)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity OffChainAuthorizationNonce (r:1 w:1)
    // Proof Skipped: Identity OffChainAuthorizationNonce (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Proof Skipped: Identity MultiPurposeNonce (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: System ParentHash (r:1 w:0)
    // Proof: System ParentHash (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
    // Storage: Identity DidRecords (r:99 w:99)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:99)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[0, 100]`.
    fn create_child_identities(i: u32) -> Weight {
        // Minimum execution time: 33_253 nanoseconds.
        Weight::from_ref_time(271_214_196)
            // Standard Error: 1_301_922
            .saturating_add(Weight::from_ref_time(108_851_504).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(i.into())))
            .saturating_add(DbWeight::get().writes(2))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(i.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity ParentDid (r:1 w:1)
    // Proof Skipped: Identity ParentDid (max_values: None, max_size: None, mode: Measured)
    fn unlink_child_identity() -> Weight {
        // Minimum execution time: 37_189 nanoseconds.
        Weight::from_ref_time(38_672_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:201 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Proof Skipped: Identity MultiPurposeNonce (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: System ParentHash (r:1 w:0)
    // Proof: System ParentHash (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
    // Storage: Identity DidRecords (r:1 w:1)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Proof Skipped: ProtocolFee Coefficient (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Proof Skipped: ProtocolFee BaseFees (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:199)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:0 w:199)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[0, 200]`.
    fn cdd_register_did(i: u32) -> Weight {
        // Minimum execution time: 80_640 nanoseconds.
        Weight::from_ref_time(2_891_663)
            // Standard Error: 456_762
            .saturating_add(Weight::from_ref_time(23_230_992).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(i.into())))
            .saturating_add(DbWeight::get().writes(4))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Instance2Group ActiveMembers (r:1 w:1)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Instance2Group InactiveMembers (r:1 w:1)
    // Proof Skipped: Instance2Group InactiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:1 w:1)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CurrentDid (r:1 w:0)
    // Proof Skipped: Identity CurrentDid (max_values: Some(1), max_size: None, mode: Measured)
    fn invalidate_cdd_claims() -> Weight {
        // Minimum execution time: 78_086 nanoseconds.
        Weight::from_ref_time(79_458_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:200 w:199)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AccountKeyRefCount (r:199 w:0)
    // Proof Skipped: Identity AccountKeyRefCount (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigToIdentity (r:199 w:0)
    // Proof Skipped: MultiSig MultiSigToIdentity (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:199)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[0, 200]`.
    fn remove_secondary_keys(i: u32) -> Weight {
        // Minimum execution time: 29_735 nanoseconds.
        Weight::from_ref_time(74_725_975)
            // Standard Error: 324_580
            .saturating_add(Weight::from_ref_time(19_323_916).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(i.into())))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: Identity Authorizations (r:2 w:2)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidRecords (r:1 w:1)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyRecords (r:2 w:2)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AccountKeyRefCount (r:1 w:0)
    // Proof Skipped: Identity AccountKeyRefCount (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigToIdentity (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Proof Skipped: Identity CddAuthForPrimaryKeyRotation (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:2)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:2)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    fn accept_primary_key() -> Weight {
        // Minimum execution time: 147_655 nanoseconds.
        Weight::from_ref_time(165_940_000)
            .saturating_add(DbWeight::get().reads(9))
            .saturating_add(DbWeight::get().writes(9))
    }
    // Storage: Identity Authorizations (r:2 w:2)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidRecords (r:1 w:1)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyRecords (r:1 w:2)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Proof Skipped: Identity CddAuthForPrimaryKeyRotation (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:2)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    fn rotate_primary_key_to_secondary() -> Weight {
        // Minimum execution time: 112_621 nanoseconds.
        Weight::from_ref_time(114_755_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(8))
    }
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:0 w:1)
    // Proof Skipped: Identity CddAuthForPrimaryKeyRotation (max_values: Some(1), max_size: None, mode: Measured)
    fn change_cdd_requirement_for_mk_rotation() -> Weight {
        // Minimum execution time: 15_639 nanoseconds.
        Weight::from_ref_time(16_421_000).saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity Authorizations (r:1 w:1)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidRecords (r:1 w:0)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyRecords (r:1 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Proof Skipped: Instance2Group ActiveMembers (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity Claims (r:2 w:0)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Proof Skipped: ProtocolFee Coefficient (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Proof Skipped: ProtocolFee BaseFees (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CurrentDid (r:0 w:1)
    // Proof Skipped: Identity CurrentDid (max_values: Some(1), max_size: None, mode: Measured)
    fn join_identity_as_key() -> Weight {
        // Minimum execution time: 117_148 nanoseconds.
        Weight::from_ref_time(122_920_000)
            .saturating_add(DbWeight::get().reads(9))
            .saturating_add(DbWeight::get().writes(5))
    }
    // Storage: Identity CurrentDid (r:1 w:0)
    // Proof Skipped: Identity CurrentDid (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity KeyRecords (r:1 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AccountKeyRefCount (r:1 w:0)
    // Proof Skipped: Identity AccountKeyRefCount (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigToIdentity (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    fn leave_identity_as_key() -> Weight {
        // Minimum execution time: 60_383 nanoseconds.
        Weight::from_ref_time(61_816_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidRecords (r:1 w:0)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Proof Skipped: ProtocolFee Coefficient (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Proof Skipped: ProtocolFee BaseFees (max_values: None, max_size: None, mode: Measured)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity Claims (r:1 w:1)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    fn add_claim() -> Weight {
        // Minimum execution time: 66_304 nanoseconds.
        Weight::from_ref_time(69_650_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Claims (r:1 w:1)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    fn revoke_claim() -> Weight {
        // Minimum execution time: 42_668 nanoseconds.
        Weight::from_ref_time(43_160_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Claims (r:1 w:1)
    // Proof Skipped: Identity Claims (max_values: None, max_size: None, mode: Measured)
    fn revoke_claim_by_index() -> Weight {
        // Minimum execution time: 42_911 nanoseconds.
        Weight::from_ref_time(47_970_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:2 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    fn set_secondary_key_permissions() -> Weight {
        // Minimum execution time: 42_159 nanoseconds.
        Weight::from_ref_time(44_141_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
    /// The range of component `a` is `[0, 2000]`.
    /// The range of component `p` is `[0, 2000]`.
    /// The range of component `l` is `[0, 80]`.
    /// The range of component `e` is `[0, 80]`.
    fn permissions_cost(a: u32, p: u32, l: u32, e: u32) -> Weight {
        // Minimum execution time: 613_987 nanoseconds.
        Weight::from_ref_time(639_264_000)
            // Manually set for `a`
            .saturating_add(Weight::from_ref_time(100_000).saturating_mul(a.into()))
            // Manually set for `p`
            .saturating_add(Weight::from_ref_time(100_000).saturating_mul(p.into()))
            // Standard Error: 525_964
            .saturating_add(Weight::from_ref_time(24_922_614).saturating_mul(l.into()))
            // Standard Error: 525_964
            .saturating_add(Weight::from_ref_time(23_026_822).saturating_mul(e.into()))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity IsDidFrozen (r:0 w:1)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    fn freeze_secondary_keys() -> Weight {
        // Minimum execution time: 30_437 nanoseconds.
        Weight::from_ref_time(31_429_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity IsDidFrozen (r:0 w:1)
    // Proof Skipped: Identity IsDidFrozen (max_values: None, max_size: None, mode: Measured)
    fn unfreeze_secondary_keys() -> Weight {
        // Minimum execution time: 31_540 nanoseconds.
        Weight::from_ref_time(31_879_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Proof Skipped: Identity MultiPurposeNonce (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:0 w:1)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    fn add_authorization() -> Weight {
        // Minimum execution time: 42_370 nanoseconds.
        Weight::from_ref_time(43_071_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:1 w:1)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    fn remove_authorization() -> Weight {
        // Minimum execution time: 46_618 nanoseconds.
        Weight::from_ref_time(47_668_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:200 w:199)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: Identity OffChainAuthorizationNonce (r:1 w:1)
    // Proof Skipped: Identity OffChainAuthorizationNonce (max_values: None, max_size: None, mode: Measured)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Proof Skipped: ProtocolFee Coefficient (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Proof Skipped: ProtocolFee BaseFees (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:199)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[0, 200]`.
    fn add_secondary_keys_with_authorization(i: u32) -> Weight {
        // Minimum execution time: 48_401 nanoseconds.
        Weight::from_ref_time(156_302_865)
            // Standard Error: 564_523
            .saturating_add(Weight::from_ref_time(82_747_981).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(i.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CustomClaimsInverse (r:1 w:1)
    // Proof Skipped: Identity CustomClaimsInverse (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CustomClaimIdSequence (r:1 w:1)
    // Proof Skipped: Identity CustomClaimIdSequence (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity CustomClaims (r:0 w:1)
    // Proof Skipped: Identity CustomClaims (max_values: None, max_size: None, mode: Measured)
    /// The range of component `n` is `[1, 2048]`.
    fn register_custom_claim_type(n: u32) -> Weight {
        // Minimum execution time: 43_412 nanoseconds.
        Weight::from_ref_time(63_521_702)
            // Standard Error: 1_268
            .saturating_add(Weight::from_ref_time(1_068).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(3))
    }
}
