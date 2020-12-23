//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_actions::distribution::WeightInfo for WeightInfo {
	fn distribute() -> Weight {
		(60_889_000 as Weight)
			.saturating_add(DbWeight::get().reads(15 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn claim(i: u32, j: u32, ) -> Weight {
		(194_656_000 as Weight)
			.saturating_add((266_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((145_000 as Weight).saturating_mul(j as Weight))
			.saturating_add(DbWeight::get().reads(31 as Weight))
			.saturating_add(DbWeight::get().writes(11 as Weight))
	}
	fn push_benefit(i: u32, j: u32, ) -> Weight {
		(193_172_000 as Weight)
			.saturating_add((272_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((148_000 as Weight).saturating_mul(j as Weight))
			.saturating_add(DbWeight::get().reads(31 as Weight))
			.saturating_add(DbWeight::get().writes(11 as Weight))
	}
	fn reclaim() -> Weight {
		(40_798_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn remove_distribution() -> Weight {
		(47_975_000 as Weight)
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
}
