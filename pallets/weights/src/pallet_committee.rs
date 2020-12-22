#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_committee::WeightInfo for WeightInfo {
    fn set_vote_threshold() -> Weight {
        (58_450_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_release_coordinator() -> Weight {
        (97_432_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_expires_after() -> Weight {
        (51_195_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn vote_or_propose_new_proposal(m: u32) -> Weight {
        (277_752_000 as Weight)
            .saturating_add((1_339_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn vote_or_propose_existing_proposal(m: u32) -> Weight {
        (317_718_000 as Weight)
            .saturating_add((424_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn vote(m: u32, a: u32) -> Weight {
        (252_375_000 as Weight)
            .saturating_add((582_000 as Weight).saturating_mul(m as Weight))
            .saturating_add((10_954_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn close(m: u32) -> Weight {
        (350_521_000 as Weight)
            .saturating_add((2_197_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
