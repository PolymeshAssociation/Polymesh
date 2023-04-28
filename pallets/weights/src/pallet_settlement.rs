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
    // Storage: Settlement InstructionStatuses (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:111 w:0)
    // Storage: Settlement UserAffirmations (r:111 w:111)
    // Storage: Settlement VenueSigners (r:1 w:0)
    // Storage: Settlement ReceiptsUsed (r:10 w:10)
    // Storage: Settlement InstructionLegsV2 (r:112 w:0)
    // Storage: Portfolio PortfolioNFT (r:100 w:0)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:111)
    // Storage: Settlement InstructionLegStatus (r:0 w:111)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    /// The range of component `o` is `[0, 10]`.
    fn affirm_with_receipts(f: u32, n: u32, o: u32) -> Weight {
        // Minimum execution time: 1_516_779 nanoseconds.
        Weight::from_ref_time(1_520_345_000)
            // Standard Error: 170_690
            .saturating_add(Weight::from_ref_time(35_042_812).saturating_mul(n.into()))
            // Standard Error: 1_700_464
            .saturating_add(Weight::from_ref_time(45_299_209).saturating_mul(o.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().reads((6_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((5_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().reads((4_u64).saturating_mul(o.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(o.into())))
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
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement InstructionStatuses (r:1 w:1)
    // Storage: Settlement InstructionLegsV2 (r:112 w:111)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement VenueFiltering (r:111 w:0)
    // Storage: Settlement InstructionLegStatus (r:111 w:111)
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
    // Storage: Settlement UserAffirmations (r:0 w:222)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:222)
    // Storage: Asset BalanceOfAtScope (r:0 w:2)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    /// The range of component `o` is `[0, 10]`.
    fn execute_manual_instruction(f: u32, n: u32, o: u32) -> Weight {
        // Minimum execution time: 4_376_820 nanoseconds.
        Weight::from_ref_time(140_931_575)
            // Standard Error: 4_213_917
            .saturating_add(Weight::from_ref_time(377_703_489).saturating_mul(f.into()))
            // Standard Error: 389_798
            .saturating_add(Weight::from_ref_time(121_317_401).saturating_mul(n.into()))
            // Standard Error: 3_817_158
            .saturating_add(Weight::from_ref_time(75_871_789).saturating_mul(o.into()))
            .saturating_add(DbWeight::get().reads(8))
            .saturating_add(DbWeight::get().reads((55_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((10_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(o.into())))
            .saturating_add(DbWeight::get().writes(6))
            .saturating_add(DbWeight::get().writes((24_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((11_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes((6_u64).saturating_mul(o.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Asset Tokens (r:111 w:0)
    // Storage: Settlement VenueFiltering (r:111 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:222)
    // Storage: Settlement InstructionLegsV2 (r:0 w:111)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionMemos (r:0 w:1)
    // Storage: Settlement InstructionStatuses (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    /// The range of component `o` is `[0, 10]`.
    fn add_instruction_with_memo_v2(f: u32, n: u32, o: u32) -> Weight {
        // Minimum execution time: 395_294 nanoseconds.
        Weight::from_ref_time(127_120_175)
            // Standard Error: 602_612
            .saturating_add(Weight::from_ref_time(20_709_044).saturating_mul(f.into()))
            // Standard Error: 55_743
            .saturating_add(Weight::from_ref_time(15_690_381).saturating_mul(n.into()))
            // Standard Error: 545_873
            .saturating_add(Weight::from_ref_time(7_397_795).saturating_mul(o.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(o.into())))
            .saturating_add(DbWeight::get().writes(8))
            .saturating_add(DbWeight::get().writes((3_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((3_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes((3_u64).saturating_mul(o.into())))
    }

    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Asset Tokens (r:111 w:0)
    // Storage: Settlement VenueFiltering (r:111 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:101 w:0)
    // Storage: Settlement InstructionLegsV2 (r:112 w:111)
    // Storage: Portfolio PortfolioNFT (r:100 w:0)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:222)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionMemos (r:0 w:1)
    // Storage: Settlement InstructionStatuses (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:101)
    // Storage: Settlement InstructionLegStatus (r:0 w:101)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    /// The range of component `o` is `[0, 10]`.
    fn add_and_affirm_instruction_with_memo_v2(f: u32, n: u32, o: u32) -> Weight {
        // Minimum execution time: 836_111 nanoseconds.
        Weight::from_ref_time(184_321_409)
            // Standard Error: 1_026_512
            .saturating_add(Weight::from_ref_time(54_026_829).saturating_mul(f.into()))
            // Standard Error: 94_955
            .saturating_add(Weight::from_ref_time(48_861_898).saturating_mul(n.into()))
            // Standard Error: 929_862
            .saturating_add(Weight::from_ref_time(13_973_301).saturating_mul(o.into()))
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().reads((6_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((6_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(o.into())))
            .saturating_add(DbWeight::get().writes(8))
            .saturating_add(DbWeight::get().writes((6_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((6_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes((3_u64).saturating_mul(o.into())))
    }

    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement InstructionStatuses (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:101 w:0)
    // Storage: Settlement UserAffirmations (r:101 w:101)
    // Storage: Settlement InstructionLegsV2 (r:112 w:0)
    // Storage: Portfolio PortfolioNFT (r:100 w:0)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:101)
    // Storage: Settlement InstructionLegStatus (r:0 w:101)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[1, 100]`.
    fn affirm_instruction_v2(f: u32, n: u32) -> Weight {
        // Minimum execution time: 687_736 nanoseconds.
        Weight::from_ref_time(323_357_782)
            // Standard Error: 646_659
            .saturating_add(Weight::from_ref_time(34_321_541).saturating_mul(f.into()))
            // Standard Error: 60_765
            .saturating_add(Weight::from_ref_time(40_161_935).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(15))
            .saturating_add(DbWeight::get().reads((6_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((5_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(n.into())))
    }

    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement InstructionStatuses (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:111 w:0)
    // Storage: Settlement UserAffirmations (r:111 w:111)
    // Storage: Settlement InstructionLegsV2 (r:112 w:0)
    // Storage: Settlement InstructionLegStatus (r:111 w:111)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement ReceiptsUsed (r:0 w:10)
    // Storage: Settlement AffirmsReceived (r:0 w:111)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    /// The range of component `o` is `[0, 10]`.
    fn withdraw_affirmation_v2(f: u32, n: u32, o: u32) -> Weight {
        // Minimum execution time: 890_863 nanoseconds.
        Weight::from_ref_time(890_983_000)
            // Standard Error: 1_245_212
            .saturating_add(Weight::from_ref_time(11_598_029).saturating_mul(f.into()))
            // Standard Error: 120_233
            .saturating_add(Weight::from_ref_time(38_302_882).saturating_mul(n.into()))
            // Standard Error: 1_197_804
            .saturating_add(Weight::from_ref_time(11_558_019).saturating_mul(o.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().reads((5_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((5_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().reads((4_u64).saturating_mul(o.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes((4_u64).saturating_mul(o.into())))
    }

    // Storage: Settlement InstructionStatuses (r:1 w:1)
    // Storage: Settlement InstructionLegsV2 (r:112 w:111)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegStatus (r:111 w:111)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:222)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement ReceiptsUsed (r:0 w:10)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:222)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    /// The range of component `o` is `[0, 10]`.
    fn reject_instruction_v2(f: u32, n: u32, o: u32) -> Weight {
        // Minimum execution time: 917_575 nanoseconds.
        Weight::from_ref_time(231_674_739)
            // Standard Error: 550_694
            .saturating_add(Weight::from_ref_time(32_072_683).saturating_mul(f.into()))
            // Standard Error: 50_940
            .saturating_add(Weight::from_ref_time(40_328_184).saturating_mul(n.into()))
            // Standard Error: 498_843
            .saturating_add(Weight::from_ref_time(40_665_859).saturating_mul(o.into()))
            .saturating_add(DbWeight::get().reads(7))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(o.into())))
            .saturating_add(DbWeight::get().writes(6))
            .saturating_add(DbWeight::get().writes((7_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((7_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes((7_u64).saturating_mul(o.into())))
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
    // Storage: Settlement InstructionLegStatus (r:111 w:0)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:100)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    fn unchecked_release_locks(f: u32, n: u32) -> Weight {
        // Minimum execution time: 185_781 nanoseconds.
        Weight::from_ref_time(12_298_725)
            // Standard Error: 297_006
            .saturating_add(Weight::from_ref_time(19_250_295).saturating_mul(f.into()))
            // Standard Error: 27_584
            .saturating_add(Weight::from_ref_time(13_501_476).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(10))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(f.into())))
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
    // Storage: Settlement InstructionLegsV2 (r:112 w:111)
    // Storage: Settlement VenueFiltering (r:111 w:0)
    // Storage: Settlement InstructionLegStatus (r:111 w:111)
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
    // Storage: Settlement UserAffirmations (r:0 w:222)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:222)
    // Storage: Asset BalanceOfAtScope (r:0 w:2)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    /// The range of component `o` is `[0, 10]`.
    fn execute_instruction_paused(f: u32, n: u32, o: u32) -> Weight {
        // Minimum execution time: 2_237_839 nanoseconds.
        Weight::from_ref_time(101_202_106)
            // Standard Error: 672_930
            .saturating_add(Weight::from_ref_time(183_370_875).saturating_mul(f.into()))
            // Standard Error: 62_247
            .saturating_add(Weight::from_ref_time(109_389_310).saturating_mul(n.into()))
            // Standard Error: 609_570
            .saturating_add(Weight::from_ref_time(27_340_784).saturating_mul(o.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().reads((21_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((10_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(o.into())))
            .saturating_add(DbWeight::get().writes(6))
            .saturating_add(DbWeight::get().writes((14_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((11_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes((6_u64).saturating_mul(o.into())))
    }
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement InstructionStatuses (r:1 w:1)
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement InstructionLegsV2 (r:112 w:111)
    // Storage: Settlement VenueFiltering (r:111 w:0)
    // Storage: Settlement InstructionLegStatus (r:111 w:111)
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
    // Storage: Settlement UserAffirmations (r:0 w:222)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:222)
    // Storage: Asset BalanceOfAtScope (r:0 w:2)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[0, 100]`.
    /// The range of component `o` is `[0, 10]`.
    fn execute_scheduled_instruction(f: u32, n: u32, o: u32) -> Weight {
        // Minimum execution time: 4_209_784 nanoseconds.
        Weight::from_ref_time(134_804_626)
            // Standard Error: 686_884
            .saturating_add(Weight::from_ref_time(383_026_015).saturating_mul(f.into()))
            // Standard Error: 63_538
            .saturating_add(Weight::from_ref_time(114_980_032).saturating_mul(n.into()))
            // Standard Error: 622_211
            .saturating_add(Weight::from_ref_time(27_316_546).saturating_mul(o.into()))
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().reads((55_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((10_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().reads((3_u64).saturating_mul(o.into())))
            .saturating_add(DbWeight::get().writes(6))
            .saturating_add(DbWeight::get().writes((24_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((11_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes((6_u64).saturating_mul(o.into())))
    }
}
