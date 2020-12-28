//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_actions::distribution::WeightInfo for WeightInfo {
	fn distribute() -> Weight {
		(63_053_000 as Weight)
			.saturating_add(DbWeight::get().reads(15 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn claim(i: u32, j: u32, ) -> Weight {
		(205_654_000 as Weight)
			.saturating_add((246_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((128_000 as Weight).saturating_mul(j as Weight))
			.saturating_add(DbWeight::get().reads(31 as Weight))
			.saturating_add(DbWeight::get().writes(11 as Weight))
	}
	fn push_benefit(i: u32, j: u32, ) -> Weight {
		(210_363_000 as Weight)
			.saturating_add((240_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((128_000 as Weight).saturating_mul(j as Weight))
			.saturating_add(DbWeight::get().reads(31 as Weight))
			.saturating_add(DbWeight::get().writes(11 as Weight))
	}
	fn reclaim() -> Weight {
		(40_246_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn remove_distribution() -> Weight {
		(47_413_000 as Weight)
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
}
