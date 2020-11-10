//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_settlement::WeightInfo for WeightInfo {
    // WARNING! Some components were not used: ["d"]
    fn create_venue(s: u32) -> Weight {
        (176_857_000 as Weight)
            .saturating_add((7_646_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    // WARNING! Some components were not used: ["d"]
    fn update_venue() -> Weight {
        (192_733_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_instruction(l: u32) -> Weight {
        (54_632_000 as Weight)
            .saturating_add((48_594_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
    }
    fn add_and_affirm_instruction(l: u32) -> Weight {
        (302_559_000 as Weight)
            .saturating_add((207_866_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
    }
    fn set_venue_filtering() -> Weight {
        (129_793_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_venue_filtering_disallow() -> Weight {
        (123_320_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn allow_venues(v: u32) -> Weight {
        (95_877_000 as Weight)
            .saturating_add((6_462_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
    }
    fn disallow_venues(v: u32) -> Weight {
        (93_907_000 as Weight)
            .saturating_add((6_054_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
    }
    fn withdraw_affirmation(l: u32) -> Weight {
        (5_414_000 as Weight)
            .saturating_add((103_665_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn withdraw_affirmation_with_receipt(l: u32) -> Weight {
        (10_246_000 as Weight)
            .saturating_add((106_451_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn withdraw_affirmation_with_both_receipt_and_onchain_affirmation(l: u32) -> Weight {
        (0 as Weight)
            .saturating_add((109_970_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn unclaim_receipt() -> Weight {
        (164_417_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
}
