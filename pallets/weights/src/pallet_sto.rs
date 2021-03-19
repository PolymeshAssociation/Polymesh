//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_sto::WeightInfo for WeightInfo {
    fn create_fundraiser(i: u32) -> Weight {
        (224_949_000 as Weight)
            .saturating_add((476_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn invest() -> Weight {
        (6_049_179_000 as Weight)
            .saturating_add(DbWeight::get().reads(159 as Weight))
            .saturating_add(DbWeight::get().writes(30 as Weight))
    }
    fn freeze_fundraiser() -> Weight {
        (165_192_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unfreeze_fundraiser() -> Weight {
        (139_159_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn modify_fundraiser_window() -> Weight {
        (162_407_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn stop() -> Weight {
        (166_831_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
