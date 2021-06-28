//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_external_agents::WeightInfo for WeightInfo {
    fn create_group(p: u32) -> Weight {
        (104_789_000 as Weight)
            .saturating_add((709_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn set_group_permissions(p: u32) -> Weight {
        (108_405_000 as Weight)
            .saturating_add((893_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_agent() -> Weight {
        (120_634_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn abdicate() -> Weight {
        (111_046_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn change_group_custom() -> Weight {
        (106_499_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn change_group_builtin() -> Weight {
        (100_127_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn accept_become_agent() -> Weight {
        (109_562_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
}
