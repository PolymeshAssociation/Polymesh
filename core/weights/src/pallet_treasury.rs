//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_treasury::WeightInfo for WeightInfo {
    fn disbursement(b: u32) -> Weight {
        (15_798_000 as Weight)
            .saturating_add((66_933_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
    }
    fn reimbursement() -> Weight {
        (200_441_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
