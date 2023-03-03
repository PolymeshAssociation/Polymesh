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

//! Autogenerated weights for pallet_sto
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-10, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=pallet_sto
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

/// Weights for pallet_sto using the Substrate node and recommended hardware.
pub struct WeightInfo;
impl pallet_sto::WeightInfo for WeightInfo {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Sto FundraiserCount (r:1 w:1)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Sto FundraiserNames (r:0 w:1)
    // Storage: Sto Fundraisers (r:0 w:1)
    fn create_fundraiser(i: u32) -> Weight {
        (105_964_000 as Weight)
            .saturating_add((4_429_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:2 w:0)
    // Storage: Sto Fundraisers (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Portfolio PortfolioLockedAssets (r:2 w:2)
    // Storage: Settlement VenueInfo (r:1 w:0)
    // Storage: Settlement VenueFiltering (r:2 w:0)
    // Storage: Settlement InstructionCounter (r:1 w:1)
    // Storage: Settlement InstructionLegs (r:3 w:2)
    // Storage: Asset Tokens (r:2 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:4 w:4)
    // Storage: Asset Frozen (r:2 w:0)
    // Storage: Asset DisableInvestorUniqueness (r:2 w:0)
    // Storage: Identity Claims (r:54 w:0)
    // Storage: Asset ScopeIdOf (r:4 w:0)
    // Storage: Asset AggregateBalance (r:4 w:4)
    // Storage: Statistics AssetTransferCompliances (r:2 w:0)
    // Storage: Statistics AssetStats (r:2 w:2)
    // Storage: Statistics TransferConditionExemptEntities (r:2 w:0)
    // Storage: ComplianceManager AssetCompliances (r:2 w:0)
    // Storage: Asset BalanceOf (r:4 w:4)
    // Storage: Checkpoint Schedules (r:2 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:2 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:2 w:2)
    // Storage: Statistics ActiveAssetStats (r:2 w:0)
    // Storage: Settlement UserAffirmations (r:0 w:2)
    // Storage: Settlement InstructionAffirmsPending (r:0 w:1)
    // Storage: Settlement InstructionDetails (r:0 w:1)
    // Storage: Settlement VenueInstructions (r:0 w:1)
    // Storage: Settlement AffirmsReceived (r:0 w:2)
    // Storage: Settlement InstructionLegStatus (r:0 w:2)
    // Storage: Asset BalanceOfAtScope (r:0 w:2)
    fn invest() -> Weight {
        (1_801_070_000 as Weight)
            .saturating_add(DbWeight::get().reads(107 as Weight))
            .saturating_add(DbWeight::get().writes(34 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Sto Fundraisers (r:1 w:1)
    fn freeze_fundraiser() -> Weight {
        (59_129_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Sto Fundraisers (r:1 w:1)
    fn unfreeze_fundraiser() -> Weight {
        (58_314_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Sto Fundraisers (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    fn modify_fundraiser_window() -> Weight {
        (60_509_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Sto Fundraisers (r:1 w:1)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    fn stop() -> Weight {
        (57_759_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
