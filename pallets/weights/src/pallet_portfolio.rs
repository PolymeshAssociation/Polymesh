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
pub struct SubstrateWeight;
impl pallet_portfolio::WeightInfo for SubstrateWeight {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio NameToNumber (r:1 w:1)
    // Storage: Portfolio NextPortfolioNumber (r:1 w:1)
    // Storage: Portfolio Portfolios (r:0 w:1)
    fn create_portfolio() -> Weight {
        // Minimum execution time: 38_721 nanoseconds.
        Weight::from_ref_time(39_473_000)
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:1 w:1)
    // Storage: Portfolio Portfolios (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:1)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:1)
    // Storage: Portfolio NameToNumber (r:0 w:1)
    fn delete_portfolio() -> Weight {
        // Minimum execution time: 52_497 nanoseconds.
        Weight::from_ref_time(53_108_000)
            .saturating_add(DbWeight::get().reads(4))
            .saturating_add(DbWeight::get().writes(5))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Portfolio Portfolios (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:0)
    // Storage: Portfolio PortfolioAssetBalances (r:2 w:2)
    // Storage: Portfolio PortfolioLockedAssets (r:1 w:0)
    // Storage: Portfolio PortfolioAssetCount (r:2 w:2)
    /// The range of component `a` is `[1, 500]`.
    fn move_portfolio_funds(a: u32) -> Weight {
        // Minimum execution time: 58_878 nanoseconds.
        Weight::from_ref_time(59_269_000)
            // Standard Error: 28_100
            .saturating_add(Weight::from_ref_time(21_530_196).saturating_mul(a.into()))
            .saturating_add(DbWeight::get().reads(5))
            .saturating_add(DbWeight::get().reads((4_u64).saturating_mul(a.into())))
            .saturating_add(DbWeight::get().writes(2))
            .saturating_add(DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio Portfolios (r:1 w:1)
    // Storage: Portfolio NameToNumber (r:1 w:2)
    /// The range of component `i` is `[1, 500]`.
    fn rename_portfolio(i: u32) -> Weight {
        // Minimum execution time: 41_106 nanoseconds.
        Weight::from_ref_time(43_656_870)
            // Standard Error: 1_134
            .saturating_add(Weight::from_ref_time(11_802).saturating_mul(i.into()))
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(3))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:1)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:1)
    fn quit_portfolio_custody() -> Weight {
        // Minimum execution time: 37_319 nanoseconds.
        Weight::from_ref_time(38_331_000)
            .saturating_add(DbWeight::get().reads(2))
            .saturating_add(DbWeight::get().writes(2))
    }
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Identity Authorizations (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:1)
    // Storage: Portfolio PortfoliosInCustody (r:0 w:2)
    // Storage: Identity AuthorizationsGiven (r:0 w:1)
    fn accept_portfolio_custody() -> Weight {
        // Minimum execution time: 50_784 nanoseconds.
        Weight::from_ref_time(50_874_000)
            .saturating_add(DbWeight::get().reads(3))
            .saturating_add(DbWeight::get().writes(5))
    }
}
