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
        (285_283_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_instruction(l: u32) -> Weight {
        (85_779_000 as Weight)
            .saturating_add((37_749_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
    }
    fn add_instruction_with_settle_on_block_type(l: u32) -> Weight {
        (162_626_000 as Weight)
            .saturating_add((35_711_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
    }
    fn add_and_affirm_instruction(l: u32) -> Weight {
        (209_467_000 as Weight)
            .saturating_add((147_055_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(4 as Weight))
            .saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
    }
    fn add_and_affirm_instruction_with_settle_on_block_type(l: u32) -> Weight {
        (295_530_000 as Weight)
            .saturating_add((140_680_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(6 as Weight))
            .saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
    }
    fn set_venue_filtering() -> Weight {
        (95_659_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_venue_filtering_disallow() -> Weight {
        (96_281_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn allow_venues(v: u32) -> Weight {
        (88_527_000 as Weight)
            .saturating_add((7_179_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
    }
    fn disallow_venues(v: u32) -> Weight {
        (89_788_000 as Weight)
            .saturating_add((6_918_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
    }
    fn withdraw_affirmation(l: u32) -> Weight {
        (105_547_000 as Weight)
            .saturating_add((115_164_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn withdraw_affirmation_with_receipt(l: u32) -> Weight {
        (124_563_000 as Weight)
            .saturating_add((117_448_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn unclaim_receipt() -> Weight {
        (171_059_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn reject_instruction(l: u32) -> Weight {
        (177_114_000 as Weight)
            .saturating_add((141_709_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn reject_instruction_with_no_pre_affirmations(l: u32) -> Weight {
        (204_320_000 as Weight)
            .saturating_add((51_929_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(l as Weight)))
    }
    fn affirm_instruction(l: u32) -> Weight {
        (2_338_000 as Weight)
            .saturating_add((68_569_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(l as Weight)))
    }
    fn claim_receipt() -> Weight {
        (432_078_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn affirm_with_receipts(r: u32) -> Weight {
        (695_891_000 as Weight)
            .saturating_add((151_078_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(r as Weight)))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(r as Weight)))
    }
    fn execute_scheduled_instruction(l: u32, s: u32, c: u32) -> Weight {
        (0 as Weight)
            .saturating_add((72_103_794_000 as Weight).saturating_mul(l as Weight))
            .saturating_add((100_534_682_000 as Weight).saturating_mul(s as Weight))
            .saturating_add((574_040_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads((19 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().reads((21 as Weight).saturating_mul(s as Weight)))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
            .saturating_add(DbWeight::get().writes((8 as Weight).saturating_mul(l as Weight)))
    }
}
