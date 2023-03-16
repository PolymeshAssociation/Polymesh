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

//! Autogenerated weights for pallet_multisig
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-25, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// -p=pallet_multisig
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

/// Weights for pallet_multisig using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_multisig::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: MultiSig MultiSigNonce (r:1 w:1)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Storage: Identity Authorizations (r:0 w:1)
    // Storage: MultiSig MultiSigToIdentity (r:0 w:1)
    // Storage: MultiSig MultiSigSignsRequired (r:0 w:1)
    /// The range of component `i` is `[1, 256]`.
    fn create_multisig(i: u32) -> Weight {
        // Minimum execution time: 50_182 nanoseconds.
        Weight::from_ref_time(32_760_927)
            // Standard Error: 30_880
            .saturating_add(Weight::from_ref_time(9_478_768).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(4))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: MultiSig ProposalIds (r:1 w:1)
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Storage: MultiSig MultiSigTxDone (r:1 w:1)
    // Storage: MultiSig Votes (r:1 w:1)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: MultiSig ProposalDetail (r:0 w:1)
    // Storage: MultiSig Proposals (r:0 w:1)
    fn create_or_approve_proposal_as_identity() -> Weight {
        // Minimum execution time: 90_477 nanoseconds.
        Weight::from_ref_time(91_438_000)
            .saturating_add(DbWeight::get().reads(10))
            .saturating_add(DbWeight::get().writes(7))
    }
    // Storage: MultiSig ProposalIds (r:1 w:1)
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: MultiSig MultiSigTxDone (r:1 w:1)
    // Storage: MultiSig Votes (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: MultiSig ProposalDetail (r:0 w:1)
    // Storage: MultiSig Proposals (r:0 w:1)
    fn create_or_approve_proposal_as_key() -> Weight {
        // Minimum execution time: 93_222 nanoseconds.
        Weight::from_ref_time(94_665_000)
            .saturating_add(DbWeight::get().reads(11))
            .saturating_add(DbWeight::get().writes(7))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Storage: MultiSig MultiSigTxDone (r:1 w:1)
    // Storage: MultiSig Votes (r:1 w:1)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: MultiSig ProposalDetail (r:0 w:1)
    // Storage: MultiSig Proposals (r:0 w:1)
    fn create_proposal_as_identity() -> Weight {
        // Minimum execution time: 85_327 nanoseconds.
        Weight::from_ref_time(86_219_000)
            .saturating_add(DbWeight::get().reads(9))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: MultiSig MultiSigTxDone (r:1 w:1)
    // Storage: MultiSig Votes (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: MultiSig ProposalDetail (r:0 w:1)
    // Storage: MultiSig Proposals (r:0 w:1)
    fn create_proposal_as_key() -> Weight {
        // Minimum execution time: 86_550 nanoseconds.
        Weight::from_ref_time(87_702_000)
            .saturating_add(DbWeight::get().reads(10))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Storage: MultiSig Votes (r:1 w:1)
    // Storage: MultiSig Proposals (r:1 w:0)
    // Storage: MultiSig ProposalDetail (r:1 w:1)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn approve_as_identity() -> Weight {
        // Minimum execution time: 74_578 nanoseconds.
        Weight::from_ref_time(74_888_000)
            .saturating_add(DbWeight::get().reads(9))
            .saturating_add(DbWeight::get().writes(4))
    }
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Storage: MultiSig Votes (r:1 w:1)
    // Storage: MultiSig Proposals (r:1 w:0)
    // Storage: MultiSig ProposalDetail (r:1 w:1)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn approve_as_key() -> Weight {
        // Minimum execution time: 67_996 nanoseconds.
        Weight::from_ref_time(68_627_000)
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(4))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Storage: MultiSig Votes (r:1 w:1)
    // Storage: MultiSig ProposalDetail (r:1 w:1)
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: MultiSig NumberOfSigners (r:1 w:0)
    fn reject_as_identity() -> Weight {
        // Minimum execution time: 57_476 nanoseconds.
        Weight::from_ref_time(57_876_000)
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Storage: MultiSig Votes (r:1 w:1)
    // Storage: MultiSig ProposalDetail (r:1 w:1)
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: MultiSig NumberOfSigners (r:1 w:0)
    fn reject_as_key() -> Weight {
        // Minimum execution time: 52_477 nanoseconds.
        Weight::from_ref_time(53_088_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity Authorizations (r:1 w:1)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Storage: MultiSig MultiSigSigners (r:1 w:1)
    // Storage: MultiSig NumberOfSigners (r:1 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    fn accept_multisig_signer_as_identity() -> Weight {
        // Minimum execution time: 57_466 nanoseconds.
        Weight::from_ref_time(58_177_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(4))
    }
    // Storage: Identity Authorizations (r:1 w:1)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Storage: MultiSig MultiSigSigners (r:1 w:1)
    // Storage: Identity KeyRecords (r:1 w:1)
    // Storage: MultiSig NumberOfSigners (r:1 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    fn accept_multisig_signer_as_key() -> Weight {
        // Minimum execution time: 59_841 nanoseconds.
        Weight::from_ref_time(60_602_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(5))
    }
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Storage: Identity Authorizations (r:0 w:1)
    fn add_multisig_signer() -> Weight {
        // Minimum execution time: 41_977 nanoseconds.
        Weight::from_ref_time(42_599_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: MultiSig MultiSigSigners (r:1 w:1)
    // Storage: MultiSig NumberOfSigners (r:1 w:1)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Storage: Identity CurrentDid (r:1 w:0)
    fn remove_multisig_signer() -> Weight {
        // Minimum execution time: 47_187 nanoseconds.
        Weight::from_ref_time(47_428_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Identity DidRecords (r:1 w:0)
    // Storage: Identity MultiPurposeNonce (r:1 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Storage: Identity Authorizations (r:0 w:1)
    /// The range of component `i` is `[1, 256]`.
    fn add_multisig_signers_via_creator(i: u32) -> Weight {
        // Minimum execution time: 52_116 nanoseconds.
        Weight::from_ref_time(18_655_199)
            // Standard Error: 16_698
            .saturating_add(Weight::from_ref_time(13_340_764).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: Identity KeyRecords (r:2 w:1)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Identity DidRecords (r:1 w:0)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Storage: MultiSig NumberOfSigners (r:1 w:1)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Storage: MultiSig MultiSigSigners (r:1 w:1)
    /// The range of component `i` is `[1, 256]`.
    fn remove_multisig_signers_via_creator(i: u32) -> Weight {
        // Minimum execution time: 62_164 nanoseconds.
        Weight::from_ref_time(61_078_844)
            // Standard Error: 32_662
            .saturating_add(Weight::from_ref_time(13_669_782).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(i.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: MultiSig NumberOfSigners (r:1 w:0)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: MultiSig MultiSigSignsRequired (r:0 w:1)
    fn change_sigs_required() -> Weight {
        // Minimum execution time: 37_730 nanoseconds.
        Weight::from_ref_time(38_811_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:2 w:1)
    // Storage: MultiSig MultiSigToIdentity (r:1 w:0)
    // Storage: Identity DidKeys (r:0 w:1)
    fn make_multisig_secondary() -> Weight {
        // Minimum execution time: 44_262 nanoseconds.
        Weight::from_ref_time(44_593_000)
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity CurrentDid (r:1 w:0)
    // Storage: Identity KeyRecords (r:2 w:2)
    // Storage: MultiSig MultiSigToIdentity (r:2 w:0)
    // Storage: Identity DidRecords (r:1 w:1)
    // Storage: Identity AccountKeyRefCount (r:1 w:0)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Storage: Identity DidKeys (r:0 w:2)
    fn make_multisig_primary() -> Weight {
        // Minimum execution time: 60_802 nanoseconds.
        Weight::from_ref_time(61_013_000)
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(5))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Instance2Group ActiveMembers (r:1 w:0)
    // Storage: Instance2Group InactiveMembers (r:1 w:0)
    // Storage: Identity Claims (r:2 w:0)
    // Storage: Identity DidRecords (r:1 w:0)
    // Storage: MultiSig Proposals (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    // Storage: MultiSig ProposalDetail (r:1 w:1)
    // Storage: Identity CurrentDid (r:0 w:1)
    // Storage: Identity CurrentPayer (r:0 w:1)
    fn execute_scheduled_proposal() -> Weight {
        // Minimum execution time: 71_683 nanoseconds.
        Weight::from_ref_time(72_895_000)
            .saturating_add(DbWeight::get().reads(10))
            .saturating_add(DbWeight::get().writes(5))
    }
}
