//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_staking::WeightInfo for WeightInfo {
	fn payout_stakers(n: u32, ) -> Weight {
		(24_247_661_000 as Weight)
			.saturating_add((66_958_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(DbWeight::get().reads(13 as Weight))
			.saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(n as Weight)))
			.saturating_add(DbWeight::get().writes(4 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(n as Weight)))
	}
	fn payout_staker(n: u32, ) -> Weight {
		(1_426_388_000 as Weight)
			.saturating_add((213_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(DbWeight::get().reads(12 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
}
