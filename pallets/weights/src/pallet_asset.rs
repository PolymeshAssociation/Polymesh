//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_asset::WeightInfo for WeightInfo {
    fn register_ticker(t: u32) -> Weight {
        (4_035_508_000 as Weight)
            .saturating_add((47_837_000 as Weight).saturating_mul(t as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn accept_ticker_transfer() -> Weight {
        (5_238_029_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn accept_asset_ownership_transfer() -> Weight {
        (6_351_980_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    // WARNING! Some components were not used: ["n"]
    fn create_asset(i: u32, f: u32) -> Weight {
        (9_952_510_000 as Weight)
            .saturating_add((536_000 as Weight).saturating_mul(i as Weight))
            .saturating_add((343_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(DbWeight::get().reads(14 as Weight))
            .saturating_add(DbWeight::get().writes(11 as Weight))
    }
    fn freeze() -> Weight {
        (2_967_658_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unfreeze() -> Weight {
        (2_957_461_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // WARNING! Some components were not used: ["n"]
    fn rename_asset() -> Weight {
        (2_899_281_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn issue() -> Weight {
        (8_133_233_000 as Weight)
            .saturating_add(DbWeight::get().reads(17 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn redeem() -> Weight {
        (7_489_990_000 as Weight)
            .saturating_add(DbWeight::get().reads(14 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn make_divisible() -> Weight {
        (2_686_572_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_documents(d: u32) -> Weight {
        (2_739_670_000 as Weight)
            .saturating_add((939_284_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    fn remove_documents(d: u32) -> Weight {
        (208_210_000 as Weight)
            .saturating_add((781_401_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
    }
    fn set_funding_round(f: u32) -> Weight {
        (2_970_181_000 as Weight)
            .saturating_add((62_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn update_identifiers(i: u32) -> Weight {
        (2_968_900_000 as Weight)
            .saturating_add((569_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_primary_issuance_agent() -> Weight {
        (3_099_364_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn claim_classic_ticker() -> Weight {
        (8_564_981_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn reserve_classic_ticker() -> Weight {
        (2_746_531_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn add_extension() -> Weight {
        0 as Weight
    }

    fn archive_extension() -> Weight {
        0 as Weight
    }

    fn unarchive_extension() -> Weight {
        0 as Weight
    }

    fn accept_primary_issuance_agent_transfer() -> Weight {
        0 as Weight
    }
}
