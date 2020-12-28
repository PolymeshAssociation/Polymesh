//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_corporate_actions::ballot::WeightInfo for WeightInfo {
    fn attach_ballot(j: u32) -> Weight {
        (50_541_000 as Weight)
            .saturating_add((17_000 as Weight).saturating_mul(j as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn vote(j: u32, k: u32) -> Weight {
        (53_634_000 as Weight)
            .saturating_add((48_000 as Weight).saturating_mul(j as Weight))
            .saturating_add((291_000 as Weight).saturating_mul(k as Weight))
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn change_end() -> Weight {
        (39_831_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn change_meta(j: u32) -> Weight {
        (41_678_000 as Weight)
            .saturating_add((18_000 as Weight).saturating_mul(j as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn change_rcv() -> Weight {
        (39_658_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_ballot() -> Weight {
        (41_601_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
}
