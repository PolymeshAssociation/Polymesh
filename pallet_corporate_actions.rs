//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
	fn set_max_details_length() -> Weight {
		(10_214_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
