//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_pips::WeightInfo for WeightInfo {
    fn set_prune_historical_pips() -> Weight {
        (71_002_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_min_proposal_deposit() -> Weight {
        (71_254_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_proposal_cool_off_period() -> Weight {
        (69_789_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_enactment_period() -> Weight {
        (70_612_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_pending_pip_expiry() -> Weight {
        (71_825_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_max_pip_skip_count() -> Weight {
        (69_040_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_active_pip_limit() -> Weight {
        (70_821_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn propose_from_community(c: u32) -> Weight {
        (316_106_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(9 as Weight))
    }
    fn propose_from_committee(c: u32) -> Weight {
        (225_716_000 as Weight)
            .saturating_add((3_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn amend_proposal(c: u32) -> Weight {
        (216_132_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn cancel_proposal(c: u32) -> Weight {
        (406_948_000 as Weight)
            .saturating_add((8_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn vote(c: u32) -> Weight {
        (179_310_000 as Weight)
            .saturating_add((6_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn approve_committee_proposal(c: u32) -> Weight {
        (0 as Weight)
            .saturating_add((16_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn reject_proposal(c: u32) -> Weight {
        (527_502_000 as Weight)
            .saturating_add((10_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn prune_proposal(c: u32) -> Weight {
        (213_400_000 as Weight)
            .saturating_add((5_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn reschedule_execution(c: u32) -> Weight {
        (148_278_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // WARNING! Some components were not used: ["c"]
    fn clear_snapshot() -> Weight {
        (61_795_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn snapshot(c: u32) -> Weight {
        (107_026_000 as Weight)
            .saturating_add((3_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn enact_snapshot_results(c: u32) -> Weight {
        (137_645_000 as Weight)
            .saturating_add((5_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
}
