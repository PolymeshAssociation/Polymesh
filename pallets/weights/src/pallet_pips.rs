//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_pips::WeightInfo for WeightInfo {
    fn set_prune_historical_pips() -> Weight {
        (70_882_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_min_proposal_deposit() -> Weight {
        (69_290_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_enactment_period() -> Weight {
        (106_069_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_pending_pip_expiry() -> Weight {
        (73_618_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_max_pip_skip_count() -> Weight {
        (70_082_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_active_pip_limit() -> Weight {
        (80_121_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn propose_from_community() -> Weight {
        (517_458_000 as Weight)
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(10 as Weight))
    }
    fn propose_from_committee() -> Weight {
        (317_575_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn vote() -> Weight {
        (276_347_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn approve_committee_proposal() -> Weight {
        (248_335_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn reject_proposal() -> Weight {
        (296_215_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn prune_proposal() -> Weight {
        (175_589_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn reschedule_execution() -> Weight {
        (299_401_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn clear_snapshot() -> Weight {
        (60_394_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn snapshot() -> Weight {
        (4_180_379_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn enact_snapshot_results() -> Weight {
        (106_461_377_629_000 as Weight)
            .saturating_add(DbWeight::get().reads(21008 as Weight))
            .saturating_add(DbWeight::get().writes(21004 as Weight))
    }
    fn execute_scheduled_pip() -> Weight {
        (31_465_664_000 as Weight)
            .saturating_add(DbWeight::get().reads(1205 as Weight))
            .saturating_add(DbWeight::get().writes(1203 as Weight))
    }
    fn expire_scheduled_pip() -> Weight {
        (29_646_956_000 as Weight)
            .saturating_add(DbWeight::get().reads(1206 as Weight))
            .saturating_add(DbWeight::get().writes(1203 as Weight))
    }
}
