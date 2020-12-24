//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_corporate_actions::distribution::WeightInfo for WeightInfo {
    fn distribute() -> Weight {
        (62_429_000 as Weight)
            .saturating_add(DbWeight::get().reads(15 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn claim(k: u32) -> Weight {
        (139_839_000 as Weight)
            .saturating_add((306_000 as Weight).saturating_mul(k as Weight))
            .saturating_add(DbWeight::get().reads(30 as Weight))
            .saturating_add(DbWeight::get().writes(11 as Weight))
    }
    fn push_benefit(k: u32) -> Weight {
        (142_940_000 as Weight)
            .saturating_add((299_000 as Weight).saturating_mul(k as Weight))
            .saturating_add(DbWeight::get().reads(30 as Weight))
            .saturating_add(DbWeight::get().writes(11 as Weight))
    }
    fn reclaim() -> Weight {
        (40_145_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn remove_distribution() -> Weight {
        (47_074_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
