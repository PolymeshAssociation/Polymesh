//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_asset::WeightInfo for WeightInfo {
    // WARNING! Some components were not used: ["t"]
    fn register_ticker() -> Weight {
        (3_776_977_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn accept_ticker_transfer() -> Weight {
        (4_724_986_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn accept_asset_ownership_transfer() -> Weight {
        (5_164_167_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    // WARNING! Some components were not used: ["n"]
    fn create_asset(i: u32, f: u32) -> Weight {
        (8_882_929_000 as Weight)
            .saturating_add((162_000 as Weight).saturating_mul(i as Weight))
            .saturating_add((106_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(DbWeight::get().reads(14 as Weight))
            .saturating_add(DbWeight::get().writes(11 as Weight))
    }
    fn freeze() -> Weight {
        (2_578_276_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unfreeze() -> Weight {
        (2_800_973_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn rename_asset(n: u32) -> Weight {
        (2_573_905_000 as Weight)
            .saturating_add((18_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn issue() -> Weight {
        (7_391_238_000 as Weight)
            .saturating_add(DbWeight::get().reads(17 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn redeem() -> Weight {
        (6_607_622_000 as Weight)
            .saturating_add(DbWeight::get().reads(14 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn make_divisible() -> Weight {
        (2_727_055_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_documents(d: u32) -> Weight {
        (2_249_285_000 as Weight)
            .saturating_add((849_501_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    fn remove_documents(d: u32) -> Weight {
        (1_094_729_000 as Weight)
            .saturating_add((634_328_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    // WARNING! Some components were not used: ["f"]
    fn set_funding_round() -> Weight {
        (2_536_595_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // WARNING! Some components were not used: ["i"]
    fn update_identifiers() -> Weight {
        (2_396_244_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_extension() -> Weight {
        (3_457_672_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn archive_extension() -> Weight {
        (2_898_236_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unarchive_extension() -> Weight {
        (2_741_949_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_smart_extension() -> Weight {
        (2_991_734_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn remove_primary_issuance_agent() -> Weight {
        (2_409_384_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn claim_classic_ticker() -> Weight {
        (6_974_275_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn reserve_classic_ticker() -> Weight {
        (2_114_787_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn accept_primary_issuance_agent_transfer() -> Weight {
        (4_233_043_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
}
