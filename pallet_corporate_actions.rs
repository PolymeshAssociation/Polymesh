//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
	fn set_max_details_length() -> Weight {
		(10_750_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn reset_caa() -> Weight {
		(40_339_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_targets(i: u32, ) -> Weight {
		(47_401_000 as Weight)
			.saturating_add((163_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_withholding_tax() -> Weight {
		(36_200_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_did_withholding_tax(i: u32, ) -> Weight {
		(48_298_000 as Weight)
			.saturating_add((245_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn initiate_corporate_action_use_defaults(i: u32, j: u32, k: u32, ) -> Weight {
		(90_328_000 as Weight)
			.saturating_add((16_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((197_000 as Weight).saturating_mul(j as Weight))
			.saturating_add((65_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(17 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	// WARNING! Some components were not used: ["i"]
	fn initiate_corporate_action_provided(j: u32, k: u32, ) -> Weight {
		(83_722_000 as Weight)
			.saturating_add((121_000 as Weight).saturating_mul(j as Weight))
			.saturating_add((73_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(14 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn link_ca_doc(i: u32, ) -> Weight {
		(38_800_000 as Weight)
			.saturating_add((4_361_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn remove_ca_with_ballot() -> Weight {
		(56_140_000 as Weight)
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().writes(7 as Weight))
	}
	fn remove_ca_with_dist() -> Weight {
		(56_263_000 as Weight)
			.saturating_add(DbWeight::get().reads(10 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
}
