//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_pips::WeightInfo for WeightInfo {
    fn set_prune_historical_pips() -> Weight {
        (68_007_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_min_proposal_deposit() -> Weight {
        (68_218_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_proposal_cool_off_period() -> Weight {
        (66_154_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_enactment_period() -> Weight {
        (67_206_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_pending_pip_expiry() -> Weight {
        (68_217_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_max_pip_skip_count() -> Weight {
        (66_135_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_active_pip_limit() -> Weight {
        (66_895_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn propose_from_community() -> Weight {
        (498_484_000 as Weight)
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(9 as Weight))
    }
    fn propose_from_committee() -> Weight {
        (296_135_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn amend_proposal() -> Weight {
        (274_915_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn cancel_proposal() -> Weight {
        (572_062_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn vote() -> Weight {
        (239_439_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn approve_committee_proposal() -> Weight {
        (245_299_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn reject_proposal() -> Weight {
        (303_388_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn prune_proposal() -> Weight {
        (168_897_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn reschedule_execution() -> Weight {
        (297_075_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn clear_snapshot() -> Weight {
        (59_221_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn snapshot() -> Weight {
        (6_469_043_000 as Weight)
            .saturating_add(DbWeight::get().reads(204 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn enact_snapshot_results() -> Weight {
        (299_743_388_000 as Weight)
            .saturating_add(DbWeight::get().reads(5434 as Weight))
            .saturating_add(DbWeight::get().writes(5430 as Weight))
    }
    fn execute_scheduled_pip() -> Weight {
        (11_646_657_000 as Weight)
            .saturating_add(DbWeight::get().reads(305 as Weight))
            .saturating_add(DbWeight::get().writes(303 as Weight))
    }
    fn expire_scheduled_pip() -> Weight {
        (6_659_690_000 as Weight)
            .saturating_add(DbWeight::get().reads(304 as Weight))
            .saturating_add(DbWeight::get().writes(302 as Weight))
    }
}
