//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_asset::WeightInfo for WeightInfo {
    fn register_ticker() -> Weight {
        (184_043_000 as Weight)
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn accept_ticker_transfer() -> Weight {
        (219_366_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn accept_asset_ownership_transfer() -> Weight {
        (232_615_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn create_asset(n: u32, i: u32, f: u32) -> Weight {
        (476_950_000 as Weight)
            .saturating_add((13_000 as Weight).saturating_mul(n as Weight))
            .saturating_add((140_000 as Weight).saturating_mul(i as Weight))
            .saturating_add((9_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(DbWeight::get().reads(22 as Weight))
            .saturating_add(DbWeight::get().writes(11 as Weight))
    }
    fn freeze() -> Weight {
        (142_131_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unfreeze() -> Weight {
        (124_938_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn rename_asset(n: u32) -> Weight {
        (130_878_000 as Weight)
            .saturating_add((8_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn issue() -> Weight {
        (278_103_000 as Weight)
            .saturating_add(DbWeight::get().reads(18 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn redeem() -> Weight {
        (264_217_000 as Weight)
            .saturating_add(DbWeight::get().reads(16 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn make_divisible() -> Weight {
        (117_651_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_documents(d: u32) -> Weight {
        (123_423_000 as Weight)
            .saturating_add((78_732_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    fn remove_documents(d: u32) -> Weight {
        (90_449_000 as Weight)
            .saturating_add((21_055_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    fn set_funding_round(f: u32) -> Weight {
        (126_372_000 as Weight)
            .saturating_add((1_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn update_identifiers(i: u32) -> Weight {
        (132_695_000 as Weight)
            .saturating_add((173_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_extension() -> Weight {
        (179_296_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn archive_extension() -> Weight {
        (155_844_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unarchive_extension() -> Weight {
        (165_872_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_smart_extension() -> Weight {
        (163_610_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn claim_classic_ticker() -> Weight {
        (421_669_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn reserve_classic_ticker() -> Weight {
        (103_041_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn controller_transfer() -> Weight {
        (339_297_000 as Weight)
            .saturating_add(DbWeight::get().reads(19 as Weight))
            .saturating_add(DbWeight::get().writes(8 as Weight))
    }
    fn register_custom_asset_type(n: u32) -> Weight {
        (73_110_000 as Weight)
            // Standard Error: 0
            .saturating_add((12_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
}
