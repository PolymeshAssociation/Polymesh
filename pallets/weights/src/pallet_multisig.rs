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
//! DATE: 2024-07-23, STEPS: `10`, REPEAT: 2, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Native), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512
//! HOSTNAME: `trinity`, CPU: `AMD Ryzen 9 7950X 16-Core Processor`

// Executed Command:
// target/release/polymesh
// benchmark
// pallet
// -p=pallet_multisig
// -e=*
// -s
// 10
// -r
// 2
// --db-cache
// 512
// --heap-pages
// 4096
// --execution
// native
// --output
// ./
// --template
// ./.maintain/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

/// Weights for pallet_multisig using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_multisig::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigNonce (r:1 w:1)
    // Proof Skipped: MultiSig MultiSigNonce (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity NumberOfGivenAuths (r:1 w:1)
    // Proof Skipped: Identity NumberOfGivenAuths (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CurrentAuthId (r:1 w:1)
    // Proof Skipped: Identity CurrentAuthId (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:256)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:0 w:256)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:0 w:1)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:0 w:1)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[1, 256]`.
    fn create_multisig(i: u32) -> Weight {
        // Minimum execution time: 17_492 nanoseconds.
        Weight::from_ref_time(12_712_118)
            // Standard Error: 29_613
            .saturating_add(Weight::from_ref_time(3_259_421).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(5))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSigners (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigTxDone (r:1 w:1)
    // Proof Skipped: MultiSig MultiSigTxDone (max_values: None, max_size: None, mode: Measured)
    // Storage: Timestamp Now (r:1 w:0)
    // Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    // Storage: MultiSig Votes (r:1 w:1)
    // Proof Skipped: MultiSig Votes (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig ProposalVoteCounts (r:0 w:1)
    // Proof Skipped: MultiSig ProposalVoteCounts (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig Proposals (r:0 w:1)
    // Proof Skipped: MultiSig Proposals (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig ProposalStates (r:0 w:1)
    // Proof Skipped: MultiSig ProposalStates (max_values: None, max_size: None, mode: Measured)
    fn create_proposal() -> Weight {
        // Minimum execution time: 27_300 nanoseconds.
        Weight::from_ref_time(34_904_000)
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().writes(5))
    }
    // Storage: MultiSig ProposalStates (r:1 w:0)
    // Proof Skipped: MultiSig ProposalStates (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSigners (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig Votes (r:1 w:1)
    // Proof Skipped: MultiSig Votes (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig Proposals (r:1 w:0)
    // Proof Skipped: MultiSig Proposals (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig ProposalVoteCounts (r:1 w:1)
    // Proof Skipped: MultiSig ProposalVoteCounts (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    fn approve() -> Weight {
        // Minimum execution time: 24_365 nanoseconds.
        Weight::from_ref_time(25_297_000)
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: MultiSig Proposals (r:1 w:0)
    // Proof Skipped: MultiSig Proposals (max_values: None, max_size: None, mode: Measured)
    // Storage: Permissions CurrentPalletName (r:1 w:1)
    // Proof Skipped: Permissions CurrentPalletName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Permissions CurrentDispatchableName (r:1 w:1)
    // Proof Skipped: Permissions CurrentDispatchableName (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: MultiSig ExecutionReentry (r:1 w:1)
    // Proof Skipped: MultiSig ExecutionReentry (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: MultiSig ProposalStates (r:0 w:1)
    // Proof Skipped: MultiSig ProposalStates (max_values: None, max_size: None, mode: Measured)
    fn execute_proposal() -> Weight {
        // Minimum execution time: 12_804 nanoseconds.
        Weight::from_ref_time(13_745_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(4))
    }
    // Storage: MultiSig ProposalStates (r:1 w:1)
    // Proof Skipped: MultiSig ProposalStates (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSigners (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSigners (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig ProposalVoteCounts (r:1 w:1)
    // Proof Skipped: MultiSig ProposalVoteCounts (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig Votes (r:1 w:1)
    // Proof Skipped: MultiSig Votes (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig NumberOfSigners (r:1 w:0)
    // Proof Skipped: MultiSig NumberOfSigners (max_values: None, max_size: None, mode: Measured)
    fn reject() -> Weight {
        // Minimum execution time: 25_307 nanoseconds.
        Weight::from_ref_time(26_789_000)
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity Authorizations (r:1 w:1)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity OutdatedAuthorizations (r:1 w:0)
    // Proof Skipped: Identity OutdatedAuthorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Proof Skipped: Identity CddAuthForPrimaryKeyRotation (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSigners (r:1 w:1)
    // Proof Skipped: MultiSig MultiSigSigners (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyRecords (r:2 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig NumberOfSigners (r:1 w:1)
    // Proof Skipped: MultiSig NumberOfSigners (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity NumberOfGivenAuths (r:1 w:1)
    // Proof Skipped: Identity NumberOfGivenAuths (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    fn accept_multisig_signer() -> Weight {
        // Minimum execution time: 24_435 nanoseconds.
        Weight::from_ref_time(26_288_000)
            .saturating_add(DbWeight::get().reads(10))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity NumberOfGivenAuths (r:1 w:1)
    // Proof Skipped: Identity NumberOfGivenAuths (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CurrentAuthId (r:1 w:1)
    // Proof Skipped: Identity CurrentAuthId (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:256)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:0 w:256)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[1, 256]`.
    fn add_multisig_signers(i: u32) -> Weight {
        // Minimum execution time: 15_409 nanoseconds.
        Weight::from_ref_time(10_262_104)
            // Standard Error: 22_012
            .saturating_add(Weight::from_ref_time(3_263_983).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(2))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: Identity KeyRecords (r:256 w:255)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Proof Skipped: Identity CddAuthForPrimaryKeyRotation (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: MultiSig NumberOfSigners (r:1 w:1)
    // Proof Skipped: MultiSig NumberOfSigners (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSigners (r:255 w:255)
    // Proof Skipped: MultiSig MultiSigSigners (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[2, 256]`.
    fn remove_multisig_signers(i: u32) -> Weight {
        // Minimum execution time: 21_740 nanoseconds.
        Weight::from_ref_time(21_740_000)
            // Standard Error: 60_365
            .saturating_add(Weight::from_ref_time(4_560_279).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(i.into())))
            .saturating_add(DbWeight::get().writes(3))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig LostCreatorPrivileges (r:1 w:0)
    // Proof Skipped: MultiSig LostCreatorPrivileges (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity NumberOfGivenAuths (r:1 w:1)
    // Proof Skipped: Identity NumberOfGivenAuths (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CurrentAuthId (r:1 w:1)
    // Proof Skipped: Identity CurrentAuthId (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:256)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:0 w:256)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[1, 256]`.
    fn add_multisig_signers_via_creator(i: u32) -> Weight {
        // Minimum execution time: 18_224 nanoseconds.
        Weight::from_ref_time(12_326_920)
            // Standard Error: 25_818
            .saturating_add(Weight::from_ref_time(3_267_640).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(2))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: Identity KeyRecords (r:256 w:255)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig LostCreatorPrivileges (r:1 w:0)
    // Proof Skipped: MultiSig LostCreatorPrivileges (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Proof Skipped: Identity CddAuthForPrimaryKeyRotation (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: MultiSig NumberOfSigners (r:1 w:1)
    // Proof Skipped: MultiSig NumberOfSigners (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSigners (r:255 w:255)
    // Proof Skipped: MultiSig MultiSigSigners (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[2, 256]`.
    fn remove_multisig_signers_via_creator(i: u32) -> Weight {
        // Minimum execution time: 22_511 nanoseconds.
        Weight::from_ref_time(22_511_000)
            // Standard Error: 62_430
            .saturating_add(Weight::from_ref_time(4_555_729).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(i.into())))
            .saturating_add(DbWeight::get().writes(3))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(i.into())))
    }
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:1)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig NumberOfSigners (r:1 w:0)
    // Proof Skipped: MultiSig NumberOfSigners (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Proof Skipped: Identity CddAuthForPrimaryKeyRotation (max_values: Some(1), max_size: None, mode: Measured)
    fn change_sigs_required() -> Weight {
        // Minimum execution time: 15_058 nanoseconds.
        Weight::from_ref_time(17_132_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:2 w:1)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:1 w:0)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:1)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    fn make_multisig_secondary() -> Weight {
        // Minimum execution time: 15_468 nanoseconds.
        Weight::from_ref_time(16_540_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:2 w:2)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:2 w:0)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity DidRecords (r:1 w:1)
    // Proof Skipped: Identity DidRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AccountKeyRefCount (r:1 w:0)
    // Proof Skipped: Identity AccountKeyRefCount (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Proof Skipped: Identity CddAuthForPrimaryKeyRotation (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: Identity DidKeys (r:0 w:2)
    // Proof Skipped: Identity DidKeys (max_values: None, max_size: None, mode: Measured)
    fn make_multisig_primary() -> Weight {
        // Minimum execution time: 22_111 nanoseconds.
        Weight::from_ref_time(24_064_000)
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().writes(5))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig LostCreatorPrivileges (r:1 w:0)
    // Proof Skipped: MultiSig LostCreatorPrivileges (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig NumberOfSigners (r:1 w:0)
    // Proof Skipped: MultiSig NumberOfSigners (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity CddAuthForPrimaryKeyRotation (r:1 w:0)
    // Proof Skipped: Identity CddAuthForPrimaryKeyRotation (max_values: Some(1), max_size: None, mode: Measured)
    // Storage: MultiSig MultiSigSignsRequired (r:0 w:1)
    // Proof Skipped: MultiSig MultiSigSignsRequired (max_values: None, max_size: None, mode: Measured)
    fn change_sigs_required_via_creator() -> Weight {
        // Minimum execution time: 15_909 nanoseconds.
        Weight::from_ref_time(17_743_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig CreatorDid (r:1 w:0)
    // Proof Skipped: MultiSig CreatorDid (max_values: None, max_size: None, mode: Measured)
    // Storage: MultiSig LostCreatorPrivileges (r:0 w:1)
    // Proof Skipped: MultiSig LostCreatorPrivileges (max_values: None, max_size: None, mode: Measured)
    fn remove_creator_controls() -> Weight {
        // Minimum execution time: 10_399 nanoseconds.
        Weight::from_ref_time(10_860_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
}
