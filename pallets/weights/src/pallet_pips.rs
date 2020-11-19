//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_pips::WeightInfo for WeightInfo {
    fn set_prune_historical_pips() -> Weight {
        (67_858_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_min_proposal_deposit() -> Weight {
        (68_319_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_proposal_cool_off_period() -> Weight {
        (66_155_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_enactment_period() -> Weight {
        (66_765_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_pending_pip_expiry() -> Weight {
        (68_858_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_max_pip_skip_count() -> Weight {
        (66_734_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_active_pip_limit() -> Weight {
        (67_115_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn propose_from_community(c: u32) -> Weight {
        (481_917_000 as Weight)
            .saturating_add((1_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(9 as Weight))
    }
    fn propose_from_committee(c: u32) -> Weight {
        (122_255_000 as Weight)
            .saturating_add((4_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn amend_proposal(c: u32) -> Weight {
        (29_957_000 as Weight)
            .saturating_add((10_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn cancel_proposal(c: u32) -> Weight {
        (87_500_000 as Weight)
            .saturating_add((17_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    // WARNING! Some components were not used: ["a"]
    fn vote(c: u32, n: u32) -> Weight {
        (164_878_000 as Weight)
            .saturating_add((6_000 as Weight).saturating_mul(c as Weight))
            .saturating_add((10_471_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn approve_committee_proposal(c: u32) -> Weight {
        (104_857_000 as Weight)
            .saturating_add((10_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn reject_proposal(c: u32) -> Weight {
        (448_343_000 as Weight)
            .saturating_add((8_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn prune_proposal(c: u32) -> Weight {
        (112_592_000 as Weight)
            .saturating_add((7_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn reschedule_execution(c: u32) -> Weight {
        (54_960_000 as Weight)
            .saturating_add((6_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // WARNING! Some components were not used: ["c"]
    fn clear_snapshot() -> Weight {
        (103_047_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    // WARNING! Some components were not used: ["h"]
    fn snapshot(c: u32, p: u32, b: u32) -> Weight {
        (0 as Weight)
            .saturating_add((17_000 as Weight).saturating_mul(c as Weight))
            .saturating_add((400_140_000 as Weight).saturating_mul(p as Weight))
            .saturating_add((3_522_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(p as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn enact_snapshot_results(c: u32, p: u32, h: u32, b: u32) -> Weight {
        (0 as Weight)
            .saturating_add((29_000 as Weight).saturating_mul(c as Weight))
            .saturating_add((1_220_908_000 as Weight).saturating_mul(p as Weight))
            .saturating_add((161_260_000 as Weight).saturating_mul(h as Weight))
            .saturating_add((219_962_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(DbWeight::get().reads((17 as Weight).saturating_mul(p as Weight)))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(h as Weight)))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().writes((17 as Weight).saturating_mul(p as Weight)))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(h as Weight)))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(b as Weight)))
    }
}
