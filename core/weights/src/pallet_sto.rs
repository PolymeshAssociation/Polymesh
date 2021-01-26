//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_sto::WeightInfo for WeightInfo {
    fn create_fundraiser(i: u32) -> Weight {
        (64_497_000 as Weight)
            .saturating_add((3_796_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn invest(c: u32) -> Weight {
        (373_222_000 as Weight)
            .saturating_add((15_652_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(49 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
            .saturating_add(DbWeight::get().writes(30 as Weight))
    }
    fn freeze_fundraiser() -> Weight {
        (45_670_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unfreeze_fundraiser() -> Weight {
        (45_069_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn modify_fundraiser_window() -> Weight {
        (52_246_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn stop() -> Weight {
        (47_577_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
