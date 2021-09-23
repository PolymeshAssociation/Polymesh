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

//! Autogenerated weights for pallet_identity
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
// -p=pallet_identity
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


/// Weights for pallet_identity using the Substrate node and recommended hardware.
pub struct WeightInfo;
impl pallet_identity::WeightInfo for WeightInfo {
	fn cdd_register_did(i: u32, ) -> Weight {
		(110_240_000 as Weight)
			// Standard Error: 213_000
			.saturating_add((33_148_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(13 as Weight))
			.saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(3 as Weight))
			.saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
	}
	fn invalidate_cdd_claims() -> Weight {
		(103_330_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn remove_secondary_keys(i: u32, ) -> Weight {
		(84_752_000 as Weight)
			// Standard Error: 208_000
			.saturating_add((32_354_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
	}
	fn accept_primary_key() -> Weight {
		(141_907_000 as Weight)
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().writes(7 as Weight))
	}
	fn change_cdd_requirement_for_mk_rotation() -> Weight {
		(19_537_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn join_identity_as_key() -> Weight {
		(130_314_000 as Weight)
			.saturating_add(DbWeight::get().reads(11 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn leave_identity_as_key() -> Weight {
		(71_562_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn add_claim() -> Weight {
		(87_431_000 as Weight)
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn revoke_claim() -> Weight {
		(81_137_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn revoke_claim_by_index() -> Weight {
		(99_364_000 as Weight)
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_permission_to_signer() -> Weight {
		(63_747_000 as Weight)
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn freeze_secondary_keys() -> Weight {
		(56_201_000 as Weight)
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn unfreeze_secondary_keys() -> Weight {
		(55_681_000 as Weight)
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn add_authorization() -> Weight {
		(65_463_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn remove_authorization() -> Weight {
		(67_445_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn add_secondary_keys_with_authorization(i: u32, ) -> Weight {
		(151_046_000 as Weight)
			// Standard Error: 413_000
			.saturating_add((80_512_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(2 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
	}
	fn add_investor_uniqueness_claim() -> Weight {
		(1_843_379_000 as Weight)
			.saturating_add(DbWeight::get().reads(14 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
	fn add_investor_uniqueness_claim_v2() -> Weight {
		(3_400_805_000 as Weight)
			.saturating_add(DbWeight::get().reads(14 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
}
