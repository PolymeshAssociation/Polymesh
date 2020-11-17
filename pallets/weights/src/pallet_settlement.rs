//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_settlement::WeightInfo for WeightInfo {
    fn create_venue(d: u32, s: u32) -> Weight {
        (58_829_000 as Weight)
            .saturating_add((7_000 as Weight).saturating_mul(d as Weight))
            .saturating_add((10_096_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn update_venue(d: u32) -> Weight {
        (96_817_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_instruction(l: u32) -> Weight {
        (831_000 as Weight)
            .saturating_add((83_403_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
    }
    fn add_and_affirm_instruction(l: u32) -> Weight {
        (193_715_000 as Weight)
            .saturating_add((243_771_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
    }
    fn set_venue_filtering() -> Weight {
        (164_638_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_venue_filtering_disallow() -> Weight {
        (163_998_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn allow_venues(v: u32) -> Weight {
        (139_489_000 as Weight)
            .saturating_add((5_859_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
    }
    fn disallow_venues(v: u32) -> Weight {
        (89_289_000 as Weight)
            .saturating_add((6_335_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
    }
    fn withdraw_affirmation(l: u32) -> Weight {
        (13_991_000 as Weight)
            .saturating_add((101_823_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn withdraw_affirmation_with_receipt(l: u32) -> Weight {
        (23_771_000 as Weight)
            .saturating_add((104_449_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn unclaim_receipt() -> Weight {
        (166_402_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn affirm_instruction() -> u64 {
        todo!()
    }
    fn reject_instruction() -> u64 {
        todo!()
    }
    fn affirm_with_receipts() -> u64 {
        todo!()
    }
    fn claim_receipt() -> u64 {
        todo!()
    }
}
