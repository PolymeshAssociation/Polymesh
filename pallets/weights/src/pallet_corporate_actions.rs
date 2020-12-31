//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
    fn set_max_details_length() -> Weight {
        (9_740_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn reset_caa() -> Weight {
        (35_354_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_targets(i: u32) -> Weight {
        (43_850_000 as Weight)
            .saturating_add((144_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_withholding_tax() -> Weight {
        (35_105_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_did_withholding_tax(i: u32) -> Weight {
        (39_678_000 as Weight)
            .saturating_add((186_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn initiate_corporate_action_use_defaults(j: u32, k: u32) -> Weight {
        (103_340_000 as Weight)
            .saturating_add((171_000 as Weight).saturating_mul(j as Weight))
            .saturating_add((127_000 as Weight).saturating_mul(k as Weight))
            .saturating_add(DbWeight::get().reads(17 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn initiate_corporate_action_provided(j: u32, k: u32) -> Weight {
        (101_926_000 as Weight)
            .saturating_add((150_000 as Weight).saturating_mul(j as Weight))
            .saturating_add((97_000 as Weight).saturating_mul(k as Weight))
            .saturating_add(DbWeight::get().reads(14 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn link_ca_doc(i: u32) -> Weight {
        (0 as Weight)
            .saturating_add((4_891_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(i as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_ca_with_ballot() -> Weight {
        (50_362_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn remove_ca_with_dist() -> Weight {
        (53_469_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn change_record_date_with_ballot() -> Weight {
        (64_885_000 as Weight)
            .saturating_add(DbWeight::get().reads(14 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn change_record_date_with_dist() -> Weight {
        (66_906_000 as Weight)
            .saturating_add(DbWeight::get().reads(14 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
}
