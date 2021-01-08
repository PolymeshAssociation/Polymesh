//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_committee::WeightInfo for WeightInfo {
    fn set_vote_threshold() -> Weight {
        (40_125_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_release_coordinator() -> Weight {
        (242_855_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_expires_after() -> Weight {
        (58_530_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn vote_or_propose_new_proposal() -> Weight {
        (584_884_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn vote_or_propose_existing_proposal() -> Weight {
        (651_638_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn vote_aye() -> Weight {
        (1_442_498_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn vote_nay() -> Weight {
        (618_949_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn close() -> Weight {
        (998_207_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
