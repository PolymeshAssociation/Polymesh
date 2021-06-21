//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_portfolio::WeightInfo for WeightInfo {
    fn create_portfolio() -> Weight {
        (134_527_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn delete_portfolio() -> Weight {
        (117_945_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn move_portfolio_funds(a: u32) -> Weight {
        (117_433_000 as Weight)
            .saturating_add((62_083_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(a as Weight)))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    fn rename_portfolio(i: u32) -> Weight {
        (143_901_000 as Weight)
            .saturating_add((23_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn quit_portfolio_custody() -> Weight {
        (116_199_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn accept_portfolio_custody() -> Weight {
        (98_413_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
}
