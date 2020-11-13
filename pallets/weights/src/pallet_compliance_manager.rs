//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_compliance_manager::WeightInfo for WeightInfo {
    fn add_compliance_requirement(s: u32, r: u32) -> Weight {
        (130_697_000 as Weight)
            .saturating_add((2_857_000 as Weight).saturating_mul(s as Weight))
            .saturating_add((1_255_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_compliance_requirement() -> Weight {
        (116_759_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn pause_asset_compliance() -> Weight {
        (119_788_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn resume_asset_compliance() -> Weight {
        (97_387_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_default_trusted_claim_issuer() -> Weight {
        (111_144_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_default_trusted_claim_issuer() -> Weight {
        (93_236_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn change_compliance_requirement(s: u32, r: u32) -> Weight {
        (134_277_000 as Weight)
            .saturating_add((1_384_000 as Weight).saturating_mul(s as Weight))
            .saturating_add((2_267_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn replace_asset_compliance(c: u32) -> Weight {
        (123_159_000 as Weight)
            .saturating_add((29_228_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn reset_asset_compliance() -> Weight {
        (87_000_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
}
