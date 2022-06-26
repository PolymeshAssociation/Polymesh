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
//! DATE: 2022-06-25, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

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
pub struct WeightInfo;
impl pallet_settlement::WeightInfo for WeightInfo {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueCounter (r:1 w:1)
    // Storage: Settlement UserVenues (r:1 w:1)
    // Storage: Settlement VenueInfo (r:0 w:1)
    // Storage: Settlement Details (r:0 w:1)
    // Storage: Settlement VenueSigners (r:0 w:50)
    fn create_venue(d: u32, s: u32, ) -> Weight {
        (58_414_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((13_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 90_000
            .saturating_add((3_170_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement Details (r:0 w:1)
    fn update_venue_details(d: u32, ) -> Weight {
        (49_957_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((4_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:1)
    fn update_venue_type() -> Weight {
        (53_459_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
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
    fn add_instruction(l: u32, ) -> Weight {
        (76_131_000 as Weight)
            // Standard Error: 1_647_000
            .saturating_add((17_712_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
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
    fn add_instruction_with_settle_on_block_type(l: u32, ) -> Weight {
        (141_490_000 as Weight)
            // Standard Error: 2_662_000
            .saturating_add((14_169_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:2 w:1)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:1)
    fn add_and_affirm_instruction(l: u32, ) -> Weight {
        (113_407_000 as Weight)
            // Standard Error: 3_107_000
            .saturating_add((74_336_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(5 as Weight))
            .saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:2 w:1)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:1)
    fn add_and_affirm_instruction_with_settle_on_block_type(l: u32, ) -> Weight {
        (165_224_000 as Weight)
            // Standard Error: 1_879_000
            .saturating_add((69_521_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(7 as Weight))
            .saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:0 w:1)
    fn set_venue_filtering() -> Weight {
        (66_578_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Settlement VenueAllowList (r:0 w:1)
    fn allow_venues(v: u32, ) -> Weight {
        (61_220_000 as Weight)
            // Standard Error: 76_000
            .saturating_add((2_847_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Settlement VenueAllowList (r:0 w:1)
    fn disallow_venues(v: u32, ) -> Weight {
        (57_240_000 as Weight)
            // Standard Error: 61_000
            .saturating_add((2_769_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
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
    fn withdraw_affirmation(l: u32, ) -> Weight {
        (72_919_000 as Weight)
            // Standard Error: 1_952_000
            .saturating_add((63_198_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement InstructionLegStatus (r:1 w:1)
    // Storage: Settlement InstructionLegs (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Settlement ReceiptsUsed (r:0 w:1)
    fn unclaim_receipt() -> Weight {
        (94_830_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
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
    fn reject_instruction(l: u32, ) -> Weight {
        (118_608_000 as Weight)
            // Standard Error: 1_149_000
            .saturating_add((48_590_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement InstructionLegs (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Scheduler Lookup (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    fn affirm_instruction(l: u32, ) -> Weight {
        (102_322_000 as Weight)
            // Standard Error: 760_000
            .saturating_add((40_582_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(l as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Settlement InstructionLegStatus (r:1 w:1)
    // Storage: Settlement VenueSigners (r:1 w:0)
    // Storage: Settlement ReceiptsUsed (r:1 w:1)
    // Storage: Settlement InstructionLegs (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    fn claim_receipt() -> Weight {
        (212_979_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Settlement UserAffirmations (r:1 w:1)
    // Storage: Settlement VenueSigners (r:1 w:0)
    // Storage: Settlement ReceiptsUsed (r:1 w:1)
    // Storage: Settlement InstructionLegs (r:2 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:1)
    // Storage: Settlement InstructionLegStatus (r:0 w:1)
    fn affirm_with_receipts(r: u32, ) -> Weight {
        (158_888_000 as Weight)
            // Standard Error: 2_345_000
            .saturating_add((144_616_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(r as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(r as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement ReceiptsUsed (r:0 w:1)
    fn change_receipt_validity() -> Weight {
        (38_811_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement InstructionLegs (r:1 w:0)
    // Storage: Settlement InstructionAffirmsPending (r:1 w:1)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement VenueFiltering (r:1 w:0)
    // Storage: Settlement InstructionLegStatus (r:1 w:1)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Asset Frozen (r:1 w:0)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Identity Claims (r:52 w:0)
    // Storage: Portfolio Portfolios (r:2 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:2 w:2)
    // Storage: Asset ScopeIdOf (r:2 w:0)
    // Storage: Asset Tokens (r:1 w:0)
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
    // Storage: Settlement AffirmsReceived (r:0 w:2)
    // Storage: Asset BalanceOfAtScope (r:0 w:2)
    fn execute_scheduled_instruction(l: u32, ) -> Weight {
        (362_069_000 as Weight)
            // Standard Error: 6_743_000
            .saturating_add((839_685_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(39 as Weight))
            .saturating_add(DbWeight::get().reads((30 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(5 as Weight))
            .saturating_add(DbWeight::get().writes((16 as Weight).saturating_mul(l as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Settlement InstructionDetails (r:1 w:1)
    // Storage: Settlement InstructionLegs (r:11 w:0)
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn reschedule_instruction() -> Weight {
        (236_748_000 as Weight)
            .saturating_add(DbWeight::get().reads(15 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
}
