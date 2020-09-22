//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0-rc6

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_asset::WeightInfo for WeightInfo {
	// WARNING! Some components were not used: ["t"]
	fn register_ticker() -> Weight {
		(50468000 as Weight)
			.saturating_add(DbWeight::get().reads(10 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
}
