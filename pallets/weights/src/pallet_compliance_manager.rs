//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_compliance_manager::WeightInfo for WeightInfo {
    fn add_compliance_requirement(s: u32, r: u32) -> Weight {
        (157_799_000 as Weight)
            .saturating_add((5_828_000 as Weight).saturating_mul(s as Weight))
            .saturating_add((2_780_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_compliance_requirement() -> Weight {
        (159_357_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn pause_asset_compliance() -> Weight {
        (166_183_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn resume_asset_compliance() -> Weight {
        (132_554_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_default_trusted_claim_issuer() -> Weight {
        (154_659_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_default_trusted_claim_issuer() -> Weight {
        (124_046_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn change_compliance_requirement(s: u32, r: u32) -> Weight {
        (153_577_000 as Weight)
            .saturating_add((8_514_000 as Weight).saturating_mul(s as Weight))
            .saturating_add((8_272_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn replace_asset_compliance(c: u32) -> Weight {
        (180_412_000 as Weight)
            .saturating_add((60_773_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn reset_asset_compliance() -> Weight {
        (152_418_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
}
