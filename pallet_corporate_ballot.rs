//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_corporate_ballot::WeightInfo for WeightInfo {
	fn attach_ballot(i: u32, j: u32, ) -> Weight {
		(48_436_000 as Weight)
			.saturating_add((323_000 as Weight).saturating_mul(i as Weight))
			.saturating_add((241_000 as Weight).saturating_mul(j as Weight))
			.saturating_add(DbWeight::get().reads(11 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
}
