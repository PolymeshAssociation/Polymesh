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

//! Autogenerated weights for pallet_settlement
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-26, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// -p=pallet_settlement
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

/// Weights for pallet_settlement using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_settlement::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueCounter (r:1 w:1)
    // Storage: Settlement UserVenues (r:1 w:1)
    // Storage: Settlement VenueInfo (r:0 w:1)
    // Storage: Settlement Details (r:0 w:1)
    // Storage: Settlement VenueSigners (r:0 w:50)
    /// The range of component `d` is `[1, 2048]`.
    /// The range of component `s` is `[0, 50]`.
    fn create_venue(d: u32, s: u32) -> Weight {
        // Minimum execution time: 39_733 nanoseconds.
        Weight::from_ref_time(39_161_374)
            // Standard Error: 202
            .saturating_add(Weight::from_ref_time(2_353).saturating_mul(d.into()))
            // Standard Error: 8_239
            .saturating_add(Weight::from_ref_time(1_930_155).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(4))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(s.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement Details (r:0 w:1)
    /// The range of component `d` is `[1, 2048]`.
    fn update_venue_details(d: u32) -> Weight {
        // Minimum execution time: 32_680 nanoseconds.
        Weight::from_ref_time(34_950_775)
            // Standard Error: 288
            .saturating_add(Weight::from_ref_time(762).saturating_mul(d.into()))
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:1)
    fn update_venue_type() -> Weight {
        // Minimum execution time: 32_370 nanoseconds.
        Weight::from_ref_time(32_891_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement VenueSigners (r:1 w:1)
    /// The range of component `s` is `[0, 50]`.
    fn update_venue_signers(s: u32) -> Weight {
        // Minimum execution time: 30_626 nanoseconds.
        Weight::from_ref_time(37_642_859)
            // Standard Error: 10_259
            .saturating_add(Weight::from_ref_time(3_462_863).saturating_mul(s.into()))
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(s.into())))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(s.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:0 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    /// The range of component `l` is `[1, 10]`.
    fn add_instruction(l: u32) -> Weight {
        // Minimum execution time: 43_019 nanoseconds.
        Weight::from_ref_time(37_650_918)
            // Standard Error: 45_281
            .saturating_add(Weight::from_ref_time(9_341_374).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((3_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:0 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    /// The range of component `l` is `[1, 10]`.
    fn add_instruction_with_settle_on_block_type(l: u32) -> Weight {
        // Minimum execution time: 61_073 nanoseconds.
        Weight::from_ref_time(55_770_778)
            // Standard Error: 36_316
            .saturating_add(Weight::from_ref_time(9_236_082).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(3))
            .saturating_add(DbWeight::get().writes((3_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:2 w:1)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:1)
    /// The range of component `l` is `[1, 10]`.
    fn add_and_affirm_instruction(l: u32) -> Weight {
        // Minimum execution time: 110_675 nanoseconds.
        Weight::from_ref_time(79_732_657)
            // Standard Error: 97_715
            .saturating_add(Weight::from_ref_time(36_375_230).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().reads((6_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(4))
            .saturating_add(DbWeight::get().writes((6_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:2 w:1)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:1)
    /// The range of component `l` is `[1, 10]`.
    fn add_and_affirm_instruction_with_settle_on_block_type(l: u32) -> Weight {
        // Minimum execution time: 132_565 nanoseconds.
        Weight::from_ref_time(97_850_453)
            // Standard Error: 58_630
            .saturating_add(Weight::from_ref_time(37_324_304).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().reads((6_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(6))
            .saturating_add(DbWeight::get().writes((6_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:0 w:1)
    fn set_venue_filtering() -> Weight {
        // Minimum execution time: 40_985 nanoseconds.
        Weight::from_ref_time(41_226_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Settlement VenueAllowList (r:0 w:1)
    /// The range of component `v` is `[0, 100]`.
    fn allow_venues(v: u32) -> Weight {
        // Minimum execution time: 39_272 nanoseconds.
        Weight::from_ref_time(40_103_592)
            // Standard Error: 17_875
            .saturating_add(Weight::from_ref_time(1_774_899).saturating_mul(v.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(v.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Settlement VenueAllowList (r:0 w:1)
    /// The range of component `v` is `[0, 100]`.
    fn disallow_venues(v: u32) -> Weight {
        // Minimum execution time: 38_962 nanoseconds.
        Weight::from_ref_time(40_296_047)
            // Standard Error: 2_619
            .saturating_add(Weight::from_ref_time(1_687_616).saturating_mul(v.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(v.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:1 w:0)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Scheduler Lookup (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:1 w:1)
    // Storage: Settlement InstructionLegStatus (r:1 w:1)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    /// The range of component `l` is `[0, 10]`.
    fn withdraw_affirmation(l: u32) -> Weight {
        // Minimum execution time: 33_151 nanoseconds.
        Weight::from_ref_time(46_008_498)
            // Standard Error: 119_212
            .saturating_add(Weight::from_ref_time(31_545_130).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().reads((5_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement InstructionLegs (r:2 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegStatus (r:1 w:1)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Scheduler Lookup (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    /// The range of component `l` is `[1, 10]`.
    fn reject_instruction(l: u32) -> Weight {
        // Minimum execution time: 94_174 nanoseconds.
        Weight::from_ref_time(74_822_703)
            // Standard Error: 94_753
            .saturating_add(Weight::from_ref_time(26_427_027).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(3))
            .saturating_add(DbWeight::get().writes((6_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:1 w:0)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Scheduler Lookup (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    /// The range of component `l` is `[0, 10]`.
    fn affirm_instruction(l: u32) -> Weight {
        // Minimum execution time: 52_097 nanoseconds.
        Weight::from_ref_time(64_876_042)
            // Standard Error: 109_941
            .saturating_add(Weight::from_ref_time(20_111_105).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(2))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:1 w:1)
    // Storage: Settlement VenueSigners (r:1 w:0)
    // Storage: Settlement ReceiptsUsed (r:1 w:1)
    // Storage: Settlement InstructionLegs (r:2 w:0)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:1)
    /// The range of component `r` is `[1, 10]`.
    fn affirm_with_receipts(r: u32) -> Weight {
        // Minimum execution time: 135_991 nanoseconds.
        Weight::from_ref_time(66_099_220)
            // Standard Error: 104_920
            .saturating_add(Weight::from_ref_time(79_400_333).saturating_mul(r.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().reads((4_u64).saturating_mul(r.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(r.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement ReceiptsUsed (r:0 w:1)
    fn change_receipt_validity() -> Weight {
        // Minimum execution time: 28_082 nanoseconds.
        Weight::from_ref_time(28_433_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement InstructionLegs (r:11 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn reschedule_instruction() -> Weight {
        // Minimum execution time: 128_167 nanoseconds.
        Weight::from_ref_time(128_858_000)
            .saturating_add(DbWeight::get().reads(15))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionMemos (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:0 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    /// The range of component `l` is `[1, 10]`.
    fn add_instruction_with_memo_and_settle_on_block_type(l: u32) -> Weight {
        // Minimum execution time: 59_731 nanoseconds.
        Weight::from_ref_time(55_103_129)
            // Standard Error: 43_109
            .saturating_add(Weight::from_ref_time(9_209_349).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(4))
            .saturating_add(DbWeight::get().writes((3_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:2 w:1)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionMemos (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:1)
    /// The range of component `l` is `[1, 10]`.
    fn add_and_affirm_instruction_with_memo_and_settle_on_block_type(l: u32) -> Weight {
        // Minimum execution time: 129_519 nanoseconds.
        Weight::from_ref_time(99_489_498)
            // Standard Error: 70_208
            .saturating_add(Weight::from_ref_time(35_692_783).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().reads((6_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(7))
            .saturating_add(DbWeight::get().writes((6_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:2 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionLegStatus (r:1 w:1)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Asset Frozen (r:1 w:0)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Identity Claims (r:52 w:0)
    // Storage: Portfolio Portfolios (r:2 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:2 w:2)
    // Storage: Asset ScopeIdOf (r:2 w:0)
    // Storage: Asset AggregateBalance (r:2 w:2)
    // Storage: Statistics AssetTransferCompliances (r:1 w:0)
    // Storage: Statistics AssetStats (r:1 w:1)
    // Storage: Statistics TransferConditionExemptEntities (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:1 w:0)
    // Storage: Asset BalanceOf (r:2 w:2)
    // Storage: Checkpoint Schedules (r:1 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:1 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:2 w:2)
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:2)
    // Storage: Asset BalanceOfAtScope (r:0 w:2)
    /// The range of component `l` is `[1, 10]`.
    fn execute_manual_instruction(l: u32) -> Weight {
        // Minimum execution time: 579_329 nanoseconds.
        Weight::from_ref_time(131_712_065)
            // Standard Error: 447_198
            .saturating_add(Weight::from_ref_time(436_517_218).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(56))
            .saturating_add(DbWeight::get().reads((28_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(5))
            .saturating_add(DbWeight::get().writes((16_u64).saturating_mul(l.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:2 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionLegsV2 (r:0 w:11)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionMemos (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    fn add_instruction_with_memo_v2(f: u32) -> Weight {
        Weight::from_ref_time(119_496_000 as u64)
            // Standard Error: 191_000
            .saturating_add(Weight::from_ref_time(2_101_000 as u64).saturating_mul(f as u64))
            .saturating_add(DbWeight::get().reads(8 as u64))
            .saturating_add(DbWeight::get().writes(19 as u64))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(f as u64)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:2 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegsV2 (r:12 w:11)
    // Storage: Portfolio PortfolioNFT (r:100 w:0)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionMemos (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:11)
    fn add_and_affirm_instruction_with_memo_v2(f: u32, n: u32) -> Weight {
        Weight::from_ref_time(114_317_000 as u64)
            // Standard Error: 356_000
            .saturating_add(Weight::from_ref_time(18_329_000 as u64).saturating_mul(f as u64))
            // Standard Error: 17_000
            .saturating_add(Weight::from_ref_time(12_938_000 as u64).saturating_mul(n as u64))
            .saturating_add(DbWeight::get().reads(14 as u64))
            .saturating_add(DbWeight::get().reads((1 as u64).saturating_mul(f as u64)))
            .saturating_add(DbWeight::get().reads((2 as u64).saturating_mul(n as u64)))
            .saturating_add(DbWeight::get().writes(12 as u64))
            .saturating_add(DbWeight::get().writes((2 as u64).saturating_mul(f as u64)))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(n as u64)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:1 w:1)
    // Storage: Settlement InstructionLegsV2 (r:12 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Portfolio PortfolioNFT (r:100 w:0)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:11)
    fn affirm_instruction_v2(f: u32, n: u32) -> Weight {
        Weight::from_ref_time(84_908_000 as u64)
            // Standard Error: 360_000
            .saturating_add(Weight::from_ref_time(17_455_000 as u64).saturating_mul(f as u64))
            // Standard Error: 18_000
            .saturating_add(Weight::from_ref_time(12_961_000 as u64).saturating_mul(n as u64))
            .saturating_add(DbWeight::get().reads(10 as u64))
            .saturating_add(DbWeight::get().reads((1 as u64).saturating_mul(f as u64)))
            .saturating_add(DbWeight::get().reads((2 as u64).saturating_mul(n as u64)))
            .saturating_add(DbWeight::get().writes(5 as u64))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(f as u64)))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(n as u64)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:1 w:1)
    // Storage: Settlement InstructionLegsV2 (r:12 w:0)
    // Storage: Settlement InstructionLegStatus (r:11 w:11)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    fn withdraw_affirmation_v2(f: u32, n: u32) -> Weight {
        Weight::from_ref_time(68_187_000 as u64)
            // Standard Error: 290_000
            .saturating_add(Weight::from_ref_time(17_795_000 as u64).saturating_mul(f as u64))
            // Standard Error: 14_000
            .saturating_add(Weight::from_ref_time(9_530_000 as u64).saturating_mul(n as u64))
            .saturating_add(DbWeight::get().reads(8 as u64))
            .saturating_add(DbWeight::get().reads((2 as u64).saturating_mul(f as u64)))
            .saturating_add(DbWeight::get().reads((1 as u64).saturating_mul(n as u64)))
            .saturating_add(DbWeight::get().writes(5 as u64))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(f as u64)))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(n as u64)))
    }
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement InstructionLegsV2 (r:12 w:11)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegStatus (r:11 w:11)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    fn reject_instruction_v2(f: u32, n: u32) -> Weight {
        Weight::from_ref_time(92_422_000 as u64)
            // Standard Error: 365_000
            .saturating_add(Weight::from_ref_time(24_665_000 as u64).saturating_mul(f as u64))
            // Standard Error: 18_000
            .saturating_add(Weight::from_ref_time(10_175_000 as u64).saturating_mul(n as u64))
            .saturating_add(DbWeight::get().reads(8 as u64))
            .saturating_add(DbWeight::get().reads((2 as u64).saturating_mul(f as u64)))
            .saturating_add(DbWeight::get().reads((1 as u64).saturating_mul(n as u64)))
            .saturating_add(DbWeight::get().writes(10 as u64))
            .saturating_add(DbWeight::get().writes((2 as u64).saturating_mul(f as u64)))
            .saturating_add(DbWeight::get().writes((1 as u64).saturating_mul(n as u64)))
    }
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement VenueAllowList (r:1 w:0)
    /// The range of component `n` is `[1, 110]`.
    fn ensure_allowed_venue(n: u32) -> Weight {
        // Minimum execution time: 23_690 nanoseconds.
        Weight::from_ref_time(46_294_649)
            // Standard Error: 41_108
            .saturating_add(Weight::from_ref_time(7_723_084).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(n.into())))
    }
    // Storage: Settlement InstructionAffirmsPending (r:1 w:0)
    // Storage: Settlement InstructionStatuses (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement InstructionLegsV2 (r:2 w:0)
    /// The range of component `n` is `[1, 110]`.
    fn execute_instruction_initial_checks(n: u32) -> Weight {
        // Minimum execution time: 37_653 nanoseconds.
        Weight::from_ref_time(25_182_678)
            // Standard Error: 33_686
            .saturating_add(Weight::from_ref_time(7_473_922).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(n.into())))
    }
    // Storage: Settlement InstructionLegStatus (r:10 w:0)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    /// The range of component `f` is `[0, 10]`.
    /// The range of component `n` is `[0, 100]`.
    fn unchecked_release_locks(f: u32, n: u32) -> Weight {
        // Minimum execution time: 97_932 nanoseconds.
        Weight::from_ref_time(33_273_379)
            // Standard Error: 79_242
            .saturating_add(Weight::from_ref_time(7_627_259).saturating_mul(f.into()))
            // Standard Error: 8_130
            .saturating_add(Weight::from_ref_time(7_096_432).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(n.into())))
    }
    // Storage: Settlement InstructionLegsV2 (r:2 w:1)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionStatuses (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:1)
    /// The range of component `l` is `[1, 110]`.
    /// The range of component `p` is `[1, 110]`.
    fn prune_instruction(l: u32, p: u32) -> Weight {
        // Minimum execution time: 55_738 nanoseconds.
        Weight::from_ref_time(56_084_000)
            // Standard Error: 47_388
            .saturating_add(Weight::from_ref_time(10_345_536).saturating_mul(l.into()))
            // Standard Error: 47_388
            .saturating_add(Weight::from_ref_time(1_035_224).saturating_mul(p.into()))
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes(8))
            .saturating_add(DbWeight::get().writes((3_u64).saturating_mul(l.into())))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(p.into())))
    }
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement InstructionStatuses (r:0 w:1)
    fn post_failed_execution() -> Weight {
        // Minimum execution time: 11_810 nanoseconds.
        Weight::from_ref_time(12_129_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement InstructionStatuses (r:1 w:1)
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement InstructionLegsV2 (r:102 w:101)
    // Storage: Settlement VenueFiltering (r:101 w:0)
    // Storage: Settlement InstructionLegStatus (r:101 w:101)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Asset Frozen (r:101 w:0)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Portfolio Portfolios (r:2 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:2 w:2)
    // Storage: Asset AggregateBalance (r:2 w:2)
    // Storage: Statistics AssetTransferCompliances (r:1 w:0)
    // Storage: ComplianceManager AssetCompliances (r:101 w:0)
    // Storage: Asset BalanceOf (r:2 w:2)
    // Storage: Checkpoint Schedules (r:1 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:1 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:1 w:1)
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Storage: NFT CollectionTicker (r:100 w:0)
    // Storage: NFT NumberOfNFTs (r:200 w:200)
    // Storage: Portfolio PortfolioNFT (r:100 w:200)
    // Storage: Settlement UserAffirmations (r:0 w:202)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:202)
    // Storage: Asset BalanceOfAtScope (r:0 w:2)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    fn execute_instruction_paused(f: u32, n: u32) -> Weight {
        // Minimum execution time: 1_925_227 nanoseconds.
        Weight::from_ref_time(1_925_926_000)
            // Standard Error: 2_859_750
            .saturating_add(Weight::from_ref_time(61_670_137).saturating_mul(f.into()))
            // Standard Error: 294_963
            .saturating_add(Weight::from_ref_time(99_017_749).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().reads((21_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((10_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes(6))
            .saturating_add(DbWeight::get().writes((14_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((11_u64).saturating_mul(n.into())))
    }
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement InstructionStatuses (r:1 w:1)
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement InstructionLegsV2 (r:102 w:101)
    // Storage: Settlement VenueFiltering (r:101 w:0)
    // Storage: Settlement InstructionLegStatus (r:101 w:101)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Asset Frozen (r:101 w:0)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Portfolio Portfolios (r:2 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:2 w:2)
    // Storage: Asset AggregateBalance (r:2 w:2)
    // Storage: Statistics AssetTransferCompliances (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Identity Claims (r:21 w:0)
    // Storage: Statistics AssetStats (r:14 w:10)
    // Storage: ComplianceManager AssetCompliances (r:101 w:0)
    // Storage: Asset BalanceOf (r:2 w:2)
    // Storage: Checkpoint Schedules (r:1 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:1 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:1 w:1)
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Storage: NFT CollectionTicker (r:100 w:0)
    // Storage: NFT NumberOfNFTs (r:200 w:200)
    // Storage: Portfolio PortfolioNFT (r:100 w:200)
    // Storage: Settlement UserAffirmations (r:0 w:202)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:202)
    // Storage: Asset BalanceOfAtScope (r:0 w:2)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    fn execute_scheduled_instruction(f: u32, n: u32) -> Weight {
        // Minimum execution time: 3_952_306 nanoseconds.
        Weight::from_ref_time(3_955_838_000)
            // Standard Error: 5_581_857
            .saturating_add(Weight::from_ref_time(126_109_297).saturating_mul(f.into()))
            // Standard Error: 575_729
            .saturating_add(Weight::from_ref_time(92_177_335).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().reads((55_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((10_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes(6))
            .saturating_add(DbWeight::get().writes((24_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((11_u64).saturating_mul(n.into())))
    }
}
