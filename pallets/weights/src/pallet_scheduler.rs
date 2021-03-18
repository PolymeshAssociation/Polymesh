//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_scheduler::WeightInfo for WeightInfo {
    fn schedule(s: u32) -> Weight {
        (51_519_000 as Weight)
            .saturating_add((176_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn cancel(s: u32) -> Weight {
        (28_548_000 as Weight)
            .saturating_add((7_730_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn schedule_named(s: u32) -> Weight {
        (65_697_000 as Weight)
            .saturating_add((132_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn cancel_named(s: u32) -> Weight {
        (40_735_000 as Weight)
            .saturating_add((7_084_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
