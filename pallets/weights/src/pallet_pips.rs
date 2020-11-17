//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_pips::WeightInfo for WeightInfo {
    fn set_prune_historical_pips() -> Weight {
        (70_071_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_min_proposal_deposit() -> Weight {
        (70_161_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_proposal_cool_off_period() -> Weight {
        (68_077_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_enactment_period() -> Weight {
        (69_209_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_pending_pip_expiry() -> Weight {
        (69_871_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_max_pip_skip_count() -> Weight {
        (68_789_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_active_pip_limit() -> Weight {
        (69_150_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn propose_from_community(c: u32) -> Weight {
        (331_904_000 as Weight)
            .saturating_add((6_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(9 as Weight))
    }
    fn propose_from_committee(c: u32) -> Weight {
        (187_087_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn amend_proposal(c: u32) -> Weight {
        (332_651_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn cancel_proposal(c: u32) -> Weight {
        (210_413_000 as Weight)
            .saturating_add((11_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn vote(c: u32) -> Weight {
        (194_673_000 as Weight)
            .saturating_add((8_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn approve_committee_proposal(c: u32) -> Weight {
        (105_386_000 as Weight)
            .saturating_add((11_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn reject_proposal(c: u32) -> Weight {
        (341_472_000 as Weight)
            .saturating_add((14_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn prune_proposal(c: u32) -> Weight {
        (95_418_000 as Weight)
            .saturating_add((9_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn reschedule_execution(c: u32) -> Weight {
        (71_893_000 as Weight)
            .saturating_add((5_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // WARNING! Some components were not used: ["c"]
    fn clear_snapshot() -> Weight {
        (73_520_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn snapshot(c: u32) -> Weight {
        (214_692_000 as Weight)
            .saturating_add((2_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn enact_snapshot_results(c: u32) -> Weight {
        (181_242_000 as Weight)
            .saturating_add((7_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
}
