//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_scheduler::WeightInfo for WeightInfo {
    fn schedule(s: u32) -> Weight {
        (42_267_000 as Weight)
            .saturating_add((217_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn cancel(s: u32) -> Weight {
        (38_748_000 as Weight)
            .saturating_add((4_012_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn schedule_named(s: u32) -> Weight {
        (58_137_000 as Weight)
            .saturating_add((155_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn cancel_named(s: u32) -> Weight {
        (50_256_000 as Weight)
            .saturating_add((3_893_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
