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
//! DATE: 2023-03-29, STEPS: `100`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
// pallet
// -s
// 100
// -r
// 5
// -p=pallet_portfolio
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

/// Weights for pallet_portfolio using the Substrate node and recommended hardware.
pub struct WeightInfo;
impl pallet_portfolio::WeightInfo for WeightInfo {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio NameToNumber (r:1 w:1)
    // Storage: Portfolio NextPortfolioNumber (r:1 w:1)
    // Storage: Portfolio Portfolios (r:0 w:1)
    fn create_portfolio() -> Weight {
        (121_886_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:1 w:1)
    // Storage: Portfolio PortfolioNFT (r:1 w:0)
    // Storage: Portfolio PortfolioLockedNFT (r:1 w:0)
    // Storage: Portfolio Portfolios (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:1)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:1)
    // Storage: Portfolio NameToNumber (r:0 w:1)
    fn delete_portfolio() -> Weight {
        (149_138_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio Portfolios (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:2 w:2)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:2 w:2)
    fn move_portfolio_funds(a: u32, ) -> Weight {
        (0 as Weight)
            // Standard Error: 308_000
            .saturating_add((51_407_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(a as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio Portfolios (r:1 w:1)
    // Storage: Portfolio NameToNumber (r:1 w:2)
    fn rename_portfolio(i: u32, ) -> Weight {
        (75_868_000 as Weight)
            // Standard Error: 5_000
            .saturating_add((27_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:1)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:1)
    fn quit_portfolio_custody() -> Weight {
        (64_351_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity Authorizations (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:1)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:2)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    fn accept_portfolio_custody() -> Weight {
        (95_261_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio Portfolios (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Portfolio PortfolioNFT (r:100 w:200)
    // Storage: Portfolio PortfolioLockedNFT (r:100 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:2 w:2)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:1 w:1)
    fn move_portfolio_funds_v2(f: u32, n: u32, ) -> Weight {
        (7_128_000 as Weight)
            // Standard Error: 11_049_000
            .saturating_add((60_935_000 as Weight).saturating_mul(f as Weight))
            // Standard Error: 555_000
            .saturating_add((21_991_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(f as Weight)))
            .saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(n as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(f as Weight)))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(n as Weight)))
    }
}
