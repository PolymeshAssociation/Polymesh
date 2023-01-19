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
//! DATE: 2023-01-18, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// -p=pallet_identity
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
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Storage: System ParentHash (r:1 w:0)
    // Storage: Identity DidRecords (r:1 w:1)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Storage: Identity DidKeys (r:0 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:2)
    // Storage: Identity Authorizations (r:0 w:2)
    /// The range of component `i` is `[0, 200]`.
    fn cdd_register_did(i: u32) -> Weight {
        // Minimum execution time: 51_996 nanoseconds.
        Weight::from_ref_time(59_054_925 as u64)
            // Standard Error: 15_903
            .saturating_add(Weight::from_ref_time(10_439_179 as u64).saturating_mul(i as u64))
            .saturating_add(DbWeight::get().reads(8 as u64))
            .saturating_add(DbWeight::get().reads((1 as u64).saturating_mul(i as u64)))
            .saturating_add(DbWeight::get().writes(4 as u64))
            .saturating_add(DbWeight::get().writes((2 as u64).saturating_mul(i as u64)))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:1)
    // Storage: Instance2Group InactiveMembers (r:1 w:1)
    // Storage: Identity Claims (r:1 w:1)
    // Storage: Identity CurrentDid (r:1 w:0)
    fn invalidate_cdd_claims() -> Weight {
        // Minimum execution time: 58_899 nanoseconds.
        Weight::from_ref_time(59_851_000 as u64)
            .saturating_add(DbWeight::get().reads(5 as u64))
            .saturating_add(DbWeight::get().writes(3 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity AccountKeyRefCount (r:2 w:0)
    // Storage: MultiSig MultiSigToIdentity (r:2 w:0)
    // Storage: Identity Authorizations (r:2 w:0)
    // Storage: Identity DidKeys (r:0 w:2)
    /// The range of component `i` is `[0, 200]`.
    fn remove_secondary_keys(i: u32) -> Weight {
        // Minimum execution time: 26_659 nanoseconds.
        Weight::from_ref_time(37_221_231 as u64)
            // Standard Error: 12_322
            .saturating_add(Weight::from_ref_time(13_845_855 as u64).saturating_mul(i as u64))
            .saturating_add(DbWeight::get().reads(1 as u64))
            .saturating_add(DbWeight::get().reads((4 as u64).saturating_mul(i as u64)))
            .saturating_add(DbWeight::get().writes((2 as u64).saturating_mul(i as u64)))
    }
    // Storage: Identity Authorizations (r:2 w:2)
    // Storage: Identity DidRecords (r:1 w:1)
    // Storage: Identity KeyRecords (r:2 w:2)
    // Storage: Identity AccountKeyRefCount (r:1 w:0)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Identity AuthorizationsGiven (r:0 w:2)
    // Storage: Identity DidKeys (r:0 w:2)
    fn accept_primary_key() -> Weight {
        // Minimum execution time: 86_480 nanoseconds.
        Weight::from_ref_time(87_341_000 as u64)
            .saturating_add(DbWeight::get().reads(9 as u64))
            .saturating_add(DbWeight::get().writes(9 as u64))
    }
    // Storage: Identity Authorizations (r:2 w:2)
    // Storage: Identity DidRecords (r:1 w:1)
    // Storage: Identity KeyRecords (r:1 w:2)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Identity AuthorizationsGiven (r:0 w:2)
    // Storage: Identity DidKeys (r:0 w:1)
    fn rotate_primary_key_to_secondary() -> Weight {
        // Minimum execution time: 74_728 nanoseconds.
        Weight::from_ref_time(75_449_000 as u64)
            .saturating_add(DbWeight::get().reads(6 as u64))
            .saturating_add(DbWeight::get().writes(8 as u64))
    }
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:0 w:1)
    fn change_cdd_requirement_for_mk_rotation() -> Weight {
        // Minimum execution time: 17_552 nanoseconds.
        Weight::from_ref_time(18_124_000 as u64).saturating_add(DbWeight::get().writes(1 as u64))
    }
    // Storage: Identity Authorizations (r:1 w:1)
    // Storage: Identity DidRecords (r:1 w:0)
    // Storage: Identity KeyRecords (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Storage: Identity DidKeys (r:0 w:1)
    // Storage: Identity CurrentDid (r:0 w:1)
    fn join_identity_as_key() -> Weight {
        // Minimum execution time: 73_546 nanoseconds.
        Weight::from_ref_time(74_919_000 as u64)
            .saturating_add(DbWeight::get().reads(10 as u64))
            .saturating_add(DbWeight::get().writes(5 as u64))
    }
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: Identity KeyRecords (r:1 w:1)
    // Storage: Identity AccountKeyRefCount (r:1 w:0)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Identity DidKeys (r:0 w:1)
    fn leave_identity_as_key() -> Weight {
        // Minimum execution time: 44_883 nanoseconds.
        Weight::from_ref_time(45_124_000 as u64)
            .saturating_add(DbWeight::get().reads(4 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity DidRecords (r:1 w:0)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Identity Claims (r:1 w:1)
    fn add_claim() -> Weight {
        // Minimum execution time: 48_650 nanoseconds.
        Weight::from_ref_time(49_311_000 as u64)
            .saturating_add(DbWeight::get().reads(6 as u64))
            .saturating_add(DbWeight::get().writes(1 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity Claims (r:1 w:1)
    // Storage: Asset BalanceOfAtScope (r:1 w:0)
    fn revoke_claim() -> Weight {
        // Minimum execution time: 45_033 nanoseconds.
        Weight::from_ref_time(45_584_000 as u64)
            .saturating_add(DbWeight::get().reads(3 as u64))
            .saturating_add(DbWeight::get().writes(1 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity Claims (r:1 w:1)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Asset ScopeIdOf (r:1 w:0)
    // Storage: Asset BalanceOfAtScope (r:1 w:0)
    fn revoke_claim_by_index() -> Weight {
        // Minimum execution time: 50_533 nanoseconds.
        Weight::from_ref_time(51_044_000 as u64)
            .saturating_add(DbWeight::get().reads(5 as u64))
            .saturating_add(DbWeight::get().writes(1 as u64))
    }
    // Storage: Identity KeyRecords (r:2 w:1)
    fn set_secondary_key_permissions() -> Weight {
        // Minimum execution time: 35_666 nanoseconds.
        Weight::from_ref_time(36_547_000 as u64)
            .saturating_add(DbWeight::get().reads(2 as u64))
            .saturating_add(DbWeight::get().writes(1 as u64))
    }
    /// The range of component `a` is `[0, 1000]`.
    /// The range of component `p` is `[0, 1000]`.
    /// The range of component `l` is `[0, 100]`.
    /// The range of component `e` is `[0, 100]`.
    fn permissions_cost(a: u32, p: u32, l: u32, e: u32) -> Weight {
        // Minimum execution time: 185_843 nanoseconds.
        Weight::from_ref_time(186_966_000 as u64)
            // Manually set for `a`
            .saturating_add(Weight::from_ref_time(100_000 as u64).saturating_mul(a as u64))
            // Manually set for `p`
            .saturating_add(Weight::from_ref_time(100_000 as u64).saturating_mul(p as u64))
            // Standard Error: 157_629
            .saturating_add(Weight::from_ref_time(12_865_554 as u64).saturating_mul(l as u64))
            // Standard Error: 157_629
            .saturating_add(Weight::from_ref_time(12_133_724 as u64).saturating_mul(e as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity IsDidFrozen (r:0 w:1)
    fn freeze_secondary_keys() -> Weight {
        // Minimum execution time: 27_731 nanoseconds.
        Weight::from_ref_time(28_112_000 as u64)
            .saturating_add(DbWeight::get().reads(1 as u64))
            .saturating_add(DbWeight::get().writes(1 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity IsDidFrozen (r:0 w:1)
    fn unfreeze_secondary_keys() -> Weight {
        // Minimum execution time: 26_970 nanoseconds.
        Weight::from_ref_time(27_561_000 as u64)
            .saturating_add(DbWeight::get().reads(1 as u64))
            .saturating_add(DbWeight::get().writes(1 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Storage: Identity Authorizations (r:0 w:1)
    fn add_authorization() -> Weight {
        // Minimum execution time: 34_784 nanoseconds.
        Weight::from_ref_time(35_415_000 as u64)
            .saturating_add(DbWeight::get().reads(2 as u64))
            .saturating_add(DbWeight::get().writes(3 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity Authorizations (r:1 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    fn remove_authorization() -> Weight {
        // Minimum execution time: 36_177 nanoseconds.
        Weight::from_ref_time(36_648_000 as u64)
            .saturating_add(DbWeight::get().reads(2 as u64))
            .saturating_add(DbWeight::get().writes(2 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Identity OffChainAuthorizationNonce (r:1 w:1)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Storage: Identity DidKeys (r:0 w:2)
    /// The range of component `i` is `[0, 200]`.
    fn add_secondary_keys_with_authorization(i: u32) -> Weight {
        // Minimum execution time: 39_563 nanoseconds.
        Weight::from_ref_time(113_037_749 as u64)
            // Standard Error: 76_645
            .saturating_add(Weight::from_ref_time(52_138_660 as u64).saturating_mul(i as u64))
            .saturating_add(DbWeight::get().reads(5 as u64))
            .saturating_add(DbWeight::get().reads((1 as u64).saturating_mul(i as u64)))
            .saturating_add(DbWeight::get().writes(1 as u64))
            .saturating_add(DbWeight::get().writes((2 as u64).saturating_mul(i as u64)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity DidRecords (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:3 w:1)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Asset ScopeIdOf (r:1 w:1)
    // Storage: Asset BalanceOfAtScope (r:1 w:1)
    // Storage: Asset BalanceOf (r:1 w:0)
    // Storage: Asset AggregateBalance (r:1 w:1)
    fn add_investor_uniqueness_claim() -> Weight {
        // Minimum execution time: 1_000_638 nanoseconds.
        Weight::from_ref_time(1_009_965_000 as u64)
            .saturating_add(DbWeight::get().reads(13 as u64))
            .saturating_add(DbWeight::get().writes(4 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity DidRecords (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:3 w:1)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Asset ScopeIdOf (r:1 w:1)
    // Storage: Asset BalanceOfAtScope (r:1 w:1)
    // Storage: Asset BalanceOf (r:1 w:0)
    // Storage: Asset AggregateBalance (r:1 w:1)
    fn add_investor_uniqueness_claim_v2() -> Weight {
        // Minimum execution time: 1_899_157 nanoseconds.
        Weight::from_ref_time(1_904_146_000 as u64)
            .saturating_add(DbWeight::get().reads(13 as u64))
            .saturating_add(DbWeight::get().writes(4 as u64))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity CustomClaimsInverse (r:1 w:1)
    // Storage: Identity CustomClaimIdSequence (r:1 w:1)
    // Storage: Identity CustomClaims (r:0 w:1)
    /// The range of component `n` is `[1, 2048]`.
    fn register_custom_claim_type(n: u32) -> Weight {
        // Minimum execution time: 34_564 nanoseconds.
        Weight::from_ref_time(35_697_157 as u64)
            // Standard Error: 72
            .saturating_add(Weight::from_ref_time(5_617 as u64).saturating_mul(n as u64))
            .saturating_add(DbWeight::get().reads(3 as u64))
            .saturating_add(DbWeight::get().writes(3 as u64))
    }
}
