//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_permissions::WeightInfo for WeightInfo {
    fn set_call_metadata() -> Weight {
        (11_624_000 as Weight).saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn clear_call_metadata() -> Weight {
        (5_999_000 as Weight).saturating_add(DbWeight::get().writes(2 as Weight))
    }
}
