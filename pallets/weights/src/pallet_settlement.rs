//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_settlement::WeightInfo for WeightInfo {
    fn create_venue(d: u32, s: u32) -> Weight {
        (103_293_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(d as Weight))
            .saturating_add((4_689_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn update_venue(d: u32) -> Weight {
        (89_998_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_instruction(l: u32) -> Weight {
        (92_811_000 as Weight)
            .saturating_add((24_087_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
    }
    fn add_instruction_with_settle_on_block_type(l: u32) -> Weight {
        (136_588_000 as Weight)
            .saturating_add((24_284_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
    }
    fn add_and_affirm_instruction(l: u32) -> Weight {
        (192_016_000 as Weight)
            .saturating_add((97_195_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(4 as Weight))
            .saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
    }
    fn add_and_affirm_instruction_with_settle_on_block_type(l: u32) -> Weight {
        (239_745_000 as Weight)
            .saturating_add((97_432_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(6 as Weight))
            .saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
    }
    fn set_venue_filtering() -> Weight {
        (90_157_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_venue_filtering_disallow() -> Weight {
        (90_307_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn allow_venues(v: u32) -> Weight {
        (87_717_000 as Weight)
            .saturating_add((4_685_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
    }
    fn disallow_venues(v: u32) -> Weight {
        (88_416_000 as Weight)
            .saturating_add((4_517_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
    }
    fn withdraw_affirmation(l: u32) -> Weight {
        (103_125_000 as Weight)
            .saturating_add((80_051_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn withdraw_affirmation_with_receipt(l: u32) -> Weight {
        (108_001_000 as Weight)
            .saturating_add((81_282_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn unclaim_receipt() -> Weight {
        (148_609_000 as Weight)
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn reject_instruction(l: u32) -> Weight {
        (158_638_000 as Weight)
            .saturating_add((95_303_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
    }
    fn reject_instruction_with_no_pre_affirmations(l: u32) -> Weight {
        (159_301_000 as Weight)
            .saturating_add((38_114_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(l as Weight)))
    }
    fn affirm_instruction(l: u32) -> Weight {
        (158_649_000 as Weight)
            .saturating_add((50_330_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(l as Weight)))
    }
    fn claim_receipt() -> Weight {
        (222_823_000 as Weight)
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn affirm_with_receipts(r: u32) -> Weight {
        (130_208_000 as Weight)
            .saturating_add((147_574_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(r as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(r as Weight)))
    }
    fn execute_scheduled_instruction(l: u32) -> Weight {
        (45_023_000 as Weight)
            .saturating_add((1_829_726_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().reads((76 as Weight).saturating_mul(l as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((14 as Weight).saturating_mul(l as Weight)))
    }
}
