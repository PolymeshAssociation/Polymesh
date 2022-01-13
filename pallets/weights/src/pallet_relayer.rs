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

//! Autogenerated weights for pallet_relayer
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-01-13, STEPS: [100, ], REPEAT: 5, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 512

// Executed Command:
// ./target/release/polymesh
// benchmark
// -s
// 100
// -r
// 5
// -p=pallet_relayer
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

use frame_support::traits::Get;
use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};
use sp_std::marker::PhantomData;

/// Weights for pallet_relayer using the Substrate node and recommended hardware.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_relayer::WeightInfo for WeightInfo<T> {
	fn set_paying_key() -> Weight {
		(67_585_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn accept_paying_key() -> Weight {
		(166_613_000 as Weight)
			.saturating_add(DbWeight::get().reads(15 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn remove_paying_key() -> Weight {
		(64_417_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn update_polyx_limit() -> Weight {
		(51_208_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn increase_polyx_limit() -> Weight {
		(51_081_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn decrease_polyx_limit() -> Weight {
		(51_632_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
