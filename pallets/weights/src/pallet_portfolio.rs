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

//! Autogenerated weights for pallet_portfolio
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-08-21, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512
//! HOSTNAME: `ubuntu-8gb-nbg1-1-bench2`, CPU: `AMD EPYC-Milan Processor`

// Executed Command:
// ./polymesh
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
// ./Polymesh/pallets/weights/src/
// --template
// ./Polymesh/.maintain/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

/// Weights for pallet_portfolio using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_portfolio::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio NameToNumber (r:1 w:1)
    // Proof Skipped: Portfolio NameToNumber (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio NextPortfolioNumber (r:1 w:1)
    // Proof Skipped: Portfolio NextPortfolioNumber (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio Portfolios (r:0 w:1)
    // Proof Skipped: Portfolio Portfolios (max_values: None, max_size: None, mode: Measured)
    /// The range of component `l` is `[1, 500]`.
    fn create_portfolio(l: u32) -> Weight {
        // Minimum execution time: 39_290 nanoseconds.
        Weight::from_ref_time(47_172_000)
            // Standard Error: 356
            .saturating_add(Weight::from_ref_time(7_568).saturating_mul(l.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioAssetCount (r:1 w:1)
    // Proof Skipped: Portfolio PortfolioAssetCount (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioNFT (r:1 w:0)
    // Proof Skipped: Portfolio PortfolioNFT (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioLockedNFT (r:1 w:0)
    // Proof Skipped: Portfolio PortfolioLockedNFT (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio Portfolios (r:1 w:1)
    // Proof Skipped: Portfolio Portfolios (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioCustodian (r:1 w:1)
    // Proof Skipped: Portfolio PortfolioCustodian (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:1)
    // Proof Skipped: Portfolio PortfoliosInCustody (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio NameToNumber (r:0 w:1)
    // Proof Skipped: Portfolio NameToNumber (max_values: None, max_size: None, mode: Measured)
    fn delete_portfolio() -> Weight {
        // Minimum execution time: 62_003 nanoseconds.
        Weight::from_ref_time(66_942_000)
            .saturating_add(DbWeight::get().reads(6))
            .saturating_add(DbWeight::get().writes(5))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio Portfolios (r:1 w:1)
    // Proof Skipped: Portfolio Portfolios (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio NameToNumber (r:1 w:2)
    // Proof Skipped: Portfolio NameToNumber (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[1, 500]`.
    fn rename_portfolio(i: u32) -> Weight {
        // Minimum execution time: 35_985 nanoseconds.
        Weight::from_ref_time(42_844_700)
            // Standard Error: 1_203
            .saturating_add(Weight::from_ref_time(14_355).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioCustodian (r:1 w:1)
    // Proof Skipped: Portfolio PortfolioCustodian (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:1)
    // Proof Skipped: Portfolio PortfoliosInCustody (max_values: None, max_size: None, mode: Measured)
    fn quit_portfolio_custody() -> Weight {
        // Minimum execution time: 35_484 nanoseconds.
        Weight::from_ref_time(40_380_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity Authorizations (r:1 w:1)
    // Proof Skipped: Identity Authorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity OutdatedAuthorizations (r:1 w:0)
    // Proof Skipped: Identity OutdatedAuthorizations (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioCustodian (r:1 w:1)
    // Proof Skipped: Portfolio PortfolioCustodian (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity NumberOfGivenAuths (r:1 w:1)
    // Proof Skipped: Identity NumberOfGivenAuths (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:2)
    // Proof Skipped: Portfolio PortfoliosInCustody (max_values: None, max_size: None, mode: Measured)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    // Proof Skipped: Identity AuthorizationsGiven (max_values: None, max_size: None, mode: Measured)
    fn accept_portfolio_custody() -> Weight {
        // Minimum execution time: 58_519 nanoseconds.
        Weight::from_ref_time(63_246_000)
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().writes(6))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio Portfolios (r:1 w:0)
    // Proof Skipped: Portfolio Portfolios (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Proof Skipped: Portfolio PortfolioCustodian (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioNFT (r:100 w:200)
    // Proof Skipped: Portfolio PortfolioNFT (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:0)
    // Proof Skipped: Portfolio PortfolioLockedNFT (max_values: None, max_size: None, mode: Measured)
    // Storage: Asset Assets (r:10 w:0)
    // Proof Skipped: Asset Assets (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioAssetBalances (r:20 w:20)
    // Proof Skipped: Portfolio PortfolioAssetBalances (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioLockedAssets (r:10 w:0)
    // Proof Skipped: Portfolio PortfolioLockedAssets (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioAssetCount (r:1 w:1)
    // Proof Skipped: Portfolio PortfolioAssetCount (max_values: None, max_size: None, mode: Measured)
    // Storage: NFT NFTOwner (r:0 w:100)
    // Proof Skipped: NFT NFTOwner (max_values: None, max_size: None, mode: Measured)
    /// The range of component `f` is `[1, 10]`.
    /// The range of component `n` is `[1, 100]`.
    fn move_portfolio_funds(f: u32, n: u32) -> Weight {
        // Minimum execution time: 300_674 nanoseconds.
        Weight::from_ref_time(39_661_933)
            // Standard Error: 330_164
            .saturating_add(Weight::from_ref_time(26_359_130).saturating_mul(f.into()))
            // Standard Error: 31_024
            .saturating_add(Weight::from_ref_time(15_348_533).saturating_mul(n.into()))
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().reads((4_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(DbWeight::get().writes(1))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(f.into())))
            .saturating_add(DbWeight::get().writes((3_u64).saturating_mul(n.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio Portfolios (r:1 w:0)
    // Proof Skipped: Portfolio Portfolios (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Proof Skipped: Portfolio PortfolioCustodian (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PreApprovedPortfolios (r:0 w:1)
    // Proof Skipped: Portfolio PreApprovedPortfolios (max_values: None, max_size: None, mode: Measured)
    fn pre_approve_portfolio() -> Weight {
        // Minimum execution time: 29_075 nanoseconds.
        Weight::from_ref_time(32_850_000)
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio Portfolios (r:1 w:0)
    // Proof Skipped: Portfolio Portfolios (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Proof Skipped: Portfolio PortfolioCustodian (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PreApprovedPortfolios (r:0 w:1)
    // Proof Skipped: Portfolio PreApprovedPortfolios (max_values: None, max_size: None, mode: Measured)
    fn remove_portfolio_pre_approval() -> Weight {
        // Minimum execution time: 30_095 nanoseconds.
        Weight::from_ref_time(36_186_000)
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio AllowedCustodians (r:0 w:1)
    // Proof Skipped: Portfolio AllowedCustodians (max_values: None, max_size: None, mode: Measured)
    fn allow_identity_to_create_portfolios() -> Weight {
        // Minimum execution time: 14_933 nanoseconds.
        Weight::from_ref_time(15_653_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio AllowedCustodians (r:0 w:1)
    // Proof Skipped: Portfolio AllowedCustodians (max_values: None, max_size: None, mode: Measured)
    fn revoke_create_portfolios_permission() -> Weight {
        // Minimum execution time: 14_262 nanoseconds.
        Weight::from_ref_time(16_965_000)
            .saturating_add(DbWeight::get().reads(1))
            .saturating_add(DbWeight::get().writes(1))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Proof Skipped: Identity KeyRecords (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio AllowedCustodians (r:1 w:0)
    // Proof Skipped: Portfolio AllowedCustodians (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio NextPortfolioNumber (r:1 w:1)
    // Proof Skipped: Portfolio NextPortfolioNumber (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio NameToNumber (r:1 w:1)
    // Proof Skipped: Portfolio NameToNumber (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfolioCustodian (r:0 w:1)
    // Proof Skipped: Portfolio PortfolioCustodian (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:1)
    // Proof Skipped: Portfolio PortfoliosInCustody (max_values: None, max_size: None, mode: Measured)
    // Storage: Portfolio Portfolios (r:0 w:1)
    // Proof Skipped: Portfolio Portfolios (max_values: None, max_size: None, mode: Measured)
    fn create_custody_portfolio() -> Weight {
        // Minimum execution time: 46_220 nanoseconds.
        Weight::from_ref_time(49_625_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(5))
    }
}
