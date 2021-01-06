//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_committee::WeightInfo for WeightInfo {
    fn set_vote_threshold() -> Weight {
        (53_084_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_release_coordinator() -> Weight {
        (229_520_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_expires_after() -> Weight {
        (82_943_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn vote_or_propose_new_proposal() -> Weight {
        (580_000_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn vote_or_propose_existing_proposal() -> Weight {
        (645_001_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn vote_aye() -> Weight {
        (1_741_715_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn vote_nay() -> Weight {
        (595_419_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn close() -> Weight {
        (1_201_183_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
