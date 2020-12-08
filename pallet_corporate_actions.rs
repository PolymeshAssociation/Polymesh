//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
	fn set_max_details_length() -> Weight {
		(10_439_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn reset_caa() -> Weight {
		(35_411_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_targets(i: u32, ) -> Weight {
		(45_389_000 as Weight)
			.saturating_add((195_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_withholding_tax() -> Weight {
		(35_348_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_did_withholding_tax(i: u32, ) -> Weight {
		(48_241_000 as Weight)
			.saturating_add((175_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn initiate_corporate_action_use_defaults(i: u32, j: u32, k: u32, ) -> Weight {
		(77_172_000 as Weight)
			.saturating_add((24_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((177_000 as Weight).saturating_mul(j as Weight))
			.saturating_add((171_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(17 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	// WARNING! Some components were not used: ["i"]
	fn initiate_corporate_action_provided(j: u32, k: u32, ) -> Weight {
		(91_296_000 as Weight)
			.saturating_add((7_000 as Weight).saturating_mul(j as Weight))
			.saturating_add((101_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(14 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn link_ca_doc(i: u32, ) -> Weight {
		(37_033_000 as Weight)
			.saturating_add((3_423_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
