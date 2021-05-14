//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
    fn set_max_details_length() -> Weight {
        (33_748_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_targets(t: u32) -> Weight {
        (196_760_000 as Weight)
            .saturating_add((597_000 as Weight).saturating_mul(t as Weight))
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_withholding_tax() -> Weight {
        (141_601_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_did_withholding_tax(w: u32) -> Weight {
        (170_530_000 as Weight)
            .saturating_add((407_000 as Weight).saturating_mul(w as Weight))
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn initiate_corporate_action_use_defaults(w: u32, t: u32) -> Weight {
        (461_255_000 as Weight)
            .saturating_add((545_000 as Weight).saturating_mul(w as Weight))
            .saturating_add((297_000 as Weight).saturating_mul(t as Weight))
            .saturating_add(DbWeight::get().reads(19 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn initiate_corporate_action_provided(w: u32, t: u32) -> Weight {
        (387_452_000 as Weight)
            .saturating_add((528_000 as Weight).saturating_mul(w as Weight))
            .saturating_add((530_000 as Weight).saturating_mul(t as Weight))
            .saturating_add(DbWeight::get().reads(16 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn link_ca_doc(d: u32) -> Weight {
        (0 as Weight)
            .saturating_add((14_056_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(d as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_ca_with_ballot() -> Weight {
        (221_152_000 as Weight)
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(7 as Weight))
    }
    fn remove_ca_with_dist() -> Weight {
        (203_172_000 as Weight)
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn change_record_date_with_ballot() -> Weight {
        (233_387_000 as Weight)
            .saturating_add(DbWeight::get().reads(17 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn change_record_date_with_dist() -> Weight {
        (236_588_000 as Weight)
            .saturating_add(DbWeight::get().reads(17 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
}
