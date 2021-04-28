//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl polymesh_contracts::WeightInfo for WeightInfo {
    fn put_code(l: u32, u: u32, d: u32) -> Weight {
        (0 as Weight)
            .saturating_add((259_000 as Weight).saturating_mul(l as Weight))
            .saturating_add((14_000 as Weight).saturating_mul(u as Weight))
            .saturating_add((18_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn instantiate() -> Weight {
        (1_724_633_000 as Weight)
            .saturating_add(DbWeight::get().reads(19 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn freeze_instantiation() -> Weight {
        (123_290_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unfreeze_instantiation() -> Weight {
        (141_107_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn transfer_template_ownership() -> Weight {
        (201_119_000 as Weight)
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn change_template_fees() -> Weight {
        (2_494_202_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn change_template_meta_url(u: u32) -> Weight {
        (2_196_686_000 as Weight)
            .saturating_add((11_000 as Weight).saturating_mul(u as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn update_schedule() -> Weight {
        (55_649_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_put_code_flag() -> Weight {
        (25_048_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
}
