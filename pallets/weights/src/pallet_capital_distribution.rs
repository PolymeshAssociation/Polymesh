//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_corporate_actions::distribution::WeightInfo for WeightInfo {
    fn distribute() -> Weight {
        (243_061_000 as Weight)
            .saturating_add(DbWeight::get().reads(16 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn claim(t: u32, w: u32) -> Weight {
        (675_140_000 as Weight)
            .saturating_add((436_000 as Weight).saturating_mul(t as Weight))
            .saturating_add((243_000 as Weight).saturating_mul(w as Weight))
            .saturating_add(DbWeight::get().reads(30 as Weight))
            .saturating_add(DbWeight::get().writes(11 as Weight))
    }
    fn push_benefit(t: u32, w: u32) -> Weight {
        (690_456_000 as Weight)
            .saturating_add((450_000 as Weight).saturating_mul(t as Weight))
            .saturating_add((250_000 as Weight).saturating_mul(w as Weight))
            .saturating_add(DbWeight::get().reads(32 as Weight))
            .saturating_add(DbWeight::get().writes(11 as Weight))
    }
    fn reclaim() -> Weight {
        (144_579_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn remove_distribution() -> Weight {
        (185_108_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
