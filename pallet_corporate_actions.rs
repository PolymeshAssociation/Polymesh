//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
	fn set_max_details_length() -> Weight {
		(10_063_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn reset_caa() -> Weight {
		(36_036_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_targets(i: u32, ) -> Weight {
		(43_666_000 as Weight)
			.saturating_add((174_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_default_withholding_tax() -> Weight {
		(34_765_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_did_withholding_tax(i: u32, ) -> Weight {
		(47_632_000 as Weight)
			.saturating_add((197_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn initiate_corporate_action_use_defaults(i: u32, j: u32, k: u32, ) -> Weight {
		(74_200_000 as Weight)
			.saturating_add((35_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((215_000 as Weight).saturating_mul(j as Weight))
			.saturating_add((161_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(17 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	// WARNING! Some components were not used: ["k"]
	fn initiate_corporate_action_provided(i: u32, j: u32, ) -> Weight {
		(91_120_000 as Weight)
			.saturating_add((79_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((70_000 as Weight).saturating_mul(j as Weight))
			.saturating_add(DbWeight::get().reads(14 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn link_ca_doc(i: u32, ) -> Weight {
		(9_714_000 as Weight)
			.saturating_add((5_154_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn remove_ca_with_ballot() -> Weight {
		(54_758_000 as Weight)
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().writes(7 as Weight))
	}
	fn remove_ca_with_dist() -> Weight {
		(90_815_000 as Weight)
			.saturating_add(DbWeight::get().reads(10 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
	fn change_record_date_with_ballot() -> Weight {
		(82_178_000 as Weight)
			.saturating_add(DbWeight::get().reads(14 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
	fn change_record_date_with_dist() -> Weight {
		(69_263_000 as Weight)
			.saturating_add(DbWeight::get().reads(14 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
}
