//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
	fn set_max_details_length() -> Weight {
		(11_381_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn reset_caa() -> Weight {
		(35_821_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_targets(i: u32, ) -> Weight {
		(48_291_000 as Weight)
			.saturating_add((180_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_withholding_tax() -> Weight {
		(43_354_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
