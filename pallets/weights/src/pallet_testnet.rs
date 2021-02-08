//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_testnet::WeightInfo for WeightInfo {
    fn register_did(i: u32) -> Weight {
        (1_314_848_000 as Weight)
            .saturating_add((52_299_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(i as Weight)))
            .saturating_add(DbWeight::get().writes(4 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
    fn mock_cdd_register_did() -> Weight {
        (1_208_967_000 as Weight)
            .saturating_add(DbWeight::get().reads(17 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
}
