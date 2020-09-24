//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0-rc6

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_asset::WeightInfo for WeightInfo {
	// WARNING! Some components were not used: ["t"]
	fn register_ticker() -> Weight {
		(50848000 as Weight)
			.saturating_add(DbWeight::get().reads(10 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	// WARNING! Some components were not used: ["f"]
	fn create_asset(n: u32, i: u32, ) -> Weight {
		(96128000 as Weight)
			.saturating_add((35000 as Weight).saturating_mul(n as Weight))
			.saturating_add((74000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(12 as Weight))
			.saturating_add(DbWeight::get().writes(11 as Weight))
	}
}
