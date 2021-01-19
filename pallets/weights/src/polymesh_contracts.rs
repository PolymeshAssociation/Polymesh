//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl polymesh_contracts::WeightInfo for WeightInfo {
    // WARNING! Some components were not used: ["d"]
    fn put_code(l: u32, u: u32, _d: u32) -> Weight {
        (2_377_166_000 as Weight)
            .saturating_add((306_000 as Weight).saturating_mul(l as Weight))
            .saturating_add((6_000 as Weight).saturating_mul(u as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn instantiate() -> Weight {
        (840_235_000 as Weight)
            .saturating_add(DbWeight::get().reads(18 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn freeze_instantiation() -> Weight {
        (189_935_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unfreeze_instantiation() -> Weight {
        (216_966_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn transfer_template_ownership() -> Weight {
        (277_409_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn change_template_fees() -> Weight {
        (208_510_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn change_template_meta_url(u: u32) -> Weight {
        (1_131_628_000 as Weight)
            .saturating_add((8_000 as Weight).saturating_mul(u as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn update_schedule() -> Weight {
        (38_993_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
}
