//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_capital_distribution::WeightInfo for WeightInfo {
	fn distribute() -> Weight {
		(62_746_000 as Weight)
			.saturating_add(DbWeight::get().reads(15 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn claim(k: u32, ) -> Weight {
		(144_552_000 as Weight)
			.saturating_add((397_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(30 as Weight))
			.saturating_add(DbWeight::get().writes(11 as Weight))
	}
	fn push_benefit(k: u32, ) -> Weight {
		(146_214_000 as Weight)
			.saturating_add((527_000 as Weight).saturating_mul(k as Weight))
			.saturating_add(DbWeight::get().reads(30 as Weight))
			.saturating_add(DbWeight::get().writes(11 as Weight))
	}
	fn reclaim() -> Weight {
		(41_477_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn remove_distribution() -> Weight {
		(46_970_000 as Weight)
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
}
