//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_ballot::WeightInfo for WeightInfo {
	fn attach_ballot(i: u32, j: u32, ) -> Weight {
		(47_406_000 as Weight)
			.saturating_add((382_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((339_000 as Weight).saturating_mul(j as Weight))
			.saturating_add(DbWeight::get().reads(11 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
	// WARNING! Some components were not used: ["i"]
	fn vote(j: u32, k: u32, ) -> Weight {
		(66_026_000 as Weight)
			.saturating_add((1_393_000 as Weight).saturating_mul(j as Weight))
			.saturating_add((386_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(13 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn change_end() -> Weight {
		(40_296_000 as Weight)
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn change_rcv() -> Weight {
		(40_276_000 as Weight)
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn remove_ballot() -> Weight {
		(41_960_000 as Weight)
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
}
