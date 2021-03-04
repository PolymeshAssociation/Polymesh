//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_test_utils::WeightInfo for WeightInfo {
    fn register_did(i: u32) -> Weight {
        (1_413_637_000 as Weight)
            .saturating_add((22_583_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(i as Weight)))
            .saturating_add(DbWeight::get().writes(4 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
    fn mock_cdd_register_did() -> Weight {
        (1_509_792_000 as Weight)
            .saturating_add(DbWeight::get().reads(17 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn get_my_did() -> Weight {
        (96_034_000 as Weight).saturating_add(DbWeight::get().reads(6 as Weight))
    }
    fn get_cdd_of() -> Weight {
        (161_983_000 as Weight).saturating_add(DbWeight::get().reads(11 as Weight))
    }
}
