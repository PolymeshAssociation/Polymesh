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

//! Autogenerated weights for pallet_balances
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-09-22, STEPS: [100, ], REPEAT: 5, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
// -s
// 100
// -r
// 5
// -p=pallet_balances
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
// --raw


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight as DbWeight}};


/// Weights for pallet_balances using the Substrate node and recommended hardware.
pub struct WeightInfo;
impl pallet_balances::WeightInfo for WeightInfo {
	fn transfer() -> Weight {
		(145_419_000 as Weight)
			.saturating_add(DbWeight::get().reads(11 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn transfer_with_memo() -> Weight {
		(138_907_000 as Weight)
			.saturating_add(DbWeight::get().reads(11 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn deposit_block_reward_reserve_balance() -> Weight {
		(109_679_000 as Weight)
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn set_balance() -> Weight {
		(130_307_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn force_transfer() -> Weight {
		(77_561_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn burn_account_balance() -> Weight {
		(77_802_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
