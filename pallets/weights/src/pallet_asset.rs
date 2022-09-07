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

//! Autogenerated weights for pallet_asset
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-09-06, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=pallet_asset
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

/// Weights for pallet_asset using the Substrate node and recommended hardware.
pub struct WeightInfo;
impl pallet_asset::WeightInfo for WeightInfo {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Asset TickerConfig (r:1 w:0)
    // Storage: Asset Tickers (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Storage: Identity CurrentPayer (r:1 w:0)
    // Storage: Asset AssetOwnershipRelations (r:0 w:1)
    // Storage: Asset ClassicTickers (r:0 w:1)
    fn register_ticker() -> Weight {
        (100_490_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity Authorizations (r:1 w:1)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Asset Tickers (r:1 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Storage: Asset AssetOwnershipRelations (r:0 w:2)
    // Storage: Asset ClassicTickers (r:0 w:1)
    fn accept_ticker_transfer() -> Weight {
        (95_460_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity Authorizations (r:1 w:1)
    // Storage: Asset Tokens (r:1 w:1)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset Tickers (r:1 w:1)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Storage: Asset AssetOwnershipRelations (r:0 w:2)
    fn accept_asset_ownership_transfer() -> Weight {
        (136_741_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:1)
    // Storage: Asset TickerConfig (r:1 w:0)
    // Storage: Asset Tickers (r:1 w:1)
    // Storage: Identity DidRecords (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:2 w:0)
    // Storage: Identity CurrentPayer (r:1 w:0)
    // Storage: ExternalAgents NumFullAgents (r:1 w:1)
    // Storage: Asset FundingRound (r:0 w:1)
    // Storage: Asset AssetOwnershipRelations (r:0 w:1)
    // Storage: Asset AssetNames (r:0 w:1)
    // Storage: Asset ClassicTickers (r:0 w:1)
    // Storage: Asset DisableInvestorUniqueness (r:0 w:1)
    // Storage: Asset Identifiers (r:0 w:1)
    // Storage: ExternalAgents AgentOf (r:0 w:1)
    // Storage: ExternalAgents GroupOfAgent (r:0 w:1)
    fn create_asset(_n: u32, i: u32, f: u32) -> Weight {
        (207_235_000 as Weight)
            // Standard Error: 10_000
            .saturating_add((66_000 as Weight).saturating_mul(i as Weight))
            // Standard Error: 41_000
            .saturating_add((100_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(12 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Asset Frozen (r:1 w:1)
    fn freeze() -> Weight {
        (72_089_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Asset Frozen (r:1 w:1)
    fn unfreeze() -> Weight {
        (122_650_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset AssetNames (r:0 w:1)
    fn rename_asset(n: u32) -> Weight {
        (73_085_000 as Weight)
            // Standard Error: 19_000
            .saturating_add((17_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:1)
    // Storage: Asset BalanceOf (r:1 w:1)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:1)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Storage: Checkpoint Schedules (r:1 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:1 w:0)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Asset ScopeIdOf (r:1 w:0)
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Storage: Asset FundingRound (r:1 w:0)
    // Storage: Asset IssuedInFundingRound (r:1 w:1)
    fn issue() -> Weight {
        (163_636_000 as Weight)
            .saturating_add(DbWeight::get().reads(18 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:1)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:1)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:0)
    // Storage: Asset BalanceOf (r:1 w:1)
    // Storage: Checkpoint Schedules (r:1 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:1 w:0)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Asset ScopeIdOf (r:1 w:0)
    // Storage: Asset AggregateBalance (r:1 w:1)
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Storage: Asset BalanceOfAtScope (r:0 w:1)
    fn redeem() -> Weight {
        (194_000_000 as Weight)
            .saturating_add(DbWeight::get().reads(16 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:1)
    fn make_divisible() -> Weight {
        (66_421_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset AssetDocumentsIdSequence (r:1 w:1)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:1 w:0)
    // Storage: Asset AssetDocuments (r:0 w:1)
    fn add_documents(d: u32) -> Weight {
        (118_263_000 as Weight)
            // Standard Error: 485_000
            .saturating_add((21_058_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset AssetDocuments (r:0 w:1)
    fn remove_documents(d: u32) -> Weight {
        (58_929_000 as Weight)
            // Standard Error: 295_000
            .saturating_add((9_941_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset FundingRound (r:0 w:1)
    fn set_funding_round(f: u32) -> Weight {
        (67_184_000 as Weight)
            // Standard Error: 16_000
            .saturating_add((3_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset Identifiers (r:0 w:1)
    fn update_identifiers(i: u32) -> Weight {
        (74_457_000 as Weight)
            // Standard Error: 7_000
            .saturating_add((94_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Asset ClassicTickers (r:1 w:0)
    // Storage: Asset Tickers (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Asset AssetOwnershipRelations (r:0 w:2)
    fn claim_classic_ticker() -> Weight {
        (164_988_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Asset Tickers (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Asset AssetOwnershipRelations (r:0 w:1)
    // Storage: Asset ClassicTickers (r:0 w:1)
    fn reserve_classic_ticker() -> Weight {
        (59_811_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Asset BalanceOf (r:2 w:2)
    // Storage: Checkpoint Schedules (r:1 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:2 w:2)
    // Storage: Portfolio PortfolioAssetCount (r:1 w:1)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Asset ScopeIdOf (r:2 w:0)
    // Storage: Asset AggregateBalance (r:2 w:2)
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Storage: Asset BalanceOfAtScope (r:0 w:2)
    fn controller_transfer() -> Weight {
        (199_491_000 as Weight)
            .saturating_add(DbWeight::get().reads(19 as Weight))
            .saturating_add(DbWeight::get().writes(9 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Asset CustomTypesInverse (r:1 w:1)
    // Storage: Asset CustomTypeIdSequence (r:1 w:1)
    // Storage: Asset CustomTypes (r:0 w:1)
    fn register_custom_asset_type(n: u32) -> Weight {
        (71_836_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((6_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset AssetMetadataGlobalKeyToName (r:1 w:0)
    // Storage: Asset AssetMetadataValueDetails (r:1 w:1)
    // Storage: Asset AssetMetadataValues (r:0 w:1)
    fn set_asset_metadata() -> Weight {
        (89_506_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset AssetMetadataGlobalKeyToName (r:1 w:0)
    // Storage: Asset AssetMetadataValueDetails (r:1 w:1)
    fn set_asset_metadata_details() -> Weight {
        (70_423_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset AssetMetadataLocalNameToKey (r:1 w:1)
    // Storage: Asset AssetMetadataNextLocalKey (r:1 w:1)
    // Storage: Asset AssetMetadataValueDetails (r:1 w:1)
    // Storage: Asset AssetMetadataValues (r:0 w:1)
    // Storage: Asset AssetMetadataLocalKeyToName (r:0 w:1)
    // Storage: Asset AssetMetadataLocalSpecs (r:0 w:1)
    fn register_and_set_local_asset_metadata() -> Weight {
        (163_104_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Asset AssetMetadataLocalNameToKey (r:1 w:1)
    // Storage: Asset AssetMetadataNextLocalKey (r:1 w:1)
    // Storage: Asset AssetMetadataLocalKeyToName (r:0 w:1)
    // Storage: Asset AssetMetadataLocalSpecs (r:0 w:1)
    fn register_asset_metadata_local_type() -> Weight {
        (116_836_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    // Storage: Asset AssetMetadataGlobalNameToKey (r:1 w:1)
    // Storage: Asset AssetMetadataNextGlobalKey (r:1 w:1)
    // Storage: Asset AssetMetadataGlobalKeyToName (r:0 w:1)
    // Storage: Asset AssetMetadataGlobalSpecs (r:0 w:1)
    fn register_asset_metadata_global_type() -> Weight {
        (70_226_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:1)
    // Storage: Portfolio NextPortfolioNumber (r:1 w:1)
    // Storage: Portfolio NameToNumber (r:1 w:1)
    // Storage: Portfolio PortfolioAssetBalances (r:1 w:2)
    // Storage: Portfolio PortfolioLockedAssets (r:2 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:2 w:2)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Asset BalanceOf (r:1 w:1)
    // Storage: Checkpoint Schedules (r:1 w:0)
    // Storage: Checkpoint CheckpointIdSequence (r:1 w:0)
    // Storage: Asset DisableInvestorUniqueness (r:1 w:0)
    // Storage: Asset ScopeIdOf (r:1 w:0)
    // Storage: Asset AggregateBalance (r:1 w:1)
    // Storage: Statistics ActiveAssetStats (r:1 w:0)
    // Storage: Portfolio Portfolios (r:0 w:1)
    // Storage: Asset BalanceOfAtScope (r:0 w:1)
    fn redeem_from_portfolio() -> Weight {
        (311_000_000 as Weight)
            .saturating_add(DbWeight::get().reads(21 as Weight))
            .saturating_add(DbWeight::get().writes(12 as Weight))
    }
}
