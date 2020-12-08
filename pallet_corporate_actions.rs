//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
	fn set_max_details_length() -> Weight {
		(10_379_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn reset_caa() -> Weight {
		(36_227_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_targets(i: u32, ) -> Weight {
		(48_079_000 as Weight)
			.saturating_add((133_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_withholding_tax() -> Weight {
		(36_405_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_did_withholding_tax(i: u32, ) -> Weight {
		(44_714_000 as Weight)
			.saturating_add((358_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// WARNING! Some components were not used: ["i"]
	fn initiate_corporate_action_use_defaults(j: u32, k: u32, ) -> Weight {
		(86_994_000 as Weight)
			.saturating_add((159_000 as Weight).saturating_mul(j as Weight))
			.saturating_add((160_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(17 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn initiate_corporate_action_provided(i: u32, j: u32, k: u32, ) -> Weight {
		(75_135_000 as Weight)
			.saturating_add((2_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((112_000 as Weight).saturating_mul(j as Weight))
			.saturating_add((115_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(14 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
}
