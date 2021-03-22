//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_asset::checkpoint::WeightInfo for WeightInfo {
    fn set_schedules_max_complexity() -> Weight {
        (31_163_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn create_checkpoint() -> Weight {
        (164_904_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // WARNING! Some components were not used: ["s"]
    fn create_schedule(_: u32) -> Weight {
        (270_003_000 as Weight)
            .saturating_add(DbWeight::get().reads(15 as Weight))
            .saturating_add(DbWeight::get().writes(7 as Weight))
    }
    fn remove_schedule(s: u32) -> Weight {
        (165_827_000 as Weight)
            .saturating_add((83_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
