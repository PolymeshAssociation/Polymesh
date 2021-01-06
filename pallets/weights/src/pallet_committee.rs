//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_committee::WeightInfo for WeightInfo {
	fn set_vote_threshold() -> Weight {
		(53_681_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_release_coordinator() -> Weight {
		(72_866_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_expires_after() -> Weight {
		(49_533_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn vote_or_propose_new_proposal() -> Weight {
		(315_782_000 as Weight)
			.saturating_add(DbWeight::get().reads(10 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
	fn vote_or_propose_existing_proposal() -> Weight {
		(192_802_000 as Weight)
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn vote_aye() -> Weight {
		(268_254_000 as Weight)
			.saturating_add(DbWeight::get().reads(10 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn vote_nay() -> Weight {
		(157_015_000 as Weight)
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn close() -> Weight {
		(184_416_000 as Weight)
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
}
