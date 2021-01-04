//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_committee::WeightInfo for WeightInfo {
    fn set_vote_threshold() -> Weight {
        (53_159_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_release_coordinator() -> Weight {
        (83_016_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_expires_after() -> Weight {
        (37_189_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn vote_or_propose_new_proposal() -> Weight {
        (1_178_600_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn vote_or_propose_existing_proposal() -> Weight {
        (1_484_542_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn vote_aye() -> Weight {
        // FIXME
        1_000_000_000
    }
    fn vote_nay() -> Weight {
        // FIXME
        1_000_000_000
    }
    fn close() -> Weight {
        (1_219_315_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
