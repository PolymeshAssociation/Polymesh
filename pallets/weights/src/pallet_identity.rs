//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0-rc6

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_identity::WeightInfo for WeightInfo {
	// WARNING! Some components were not used: ["u"]
	fn add_claim() -> Weight {
		(44592000 as Weight)
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
