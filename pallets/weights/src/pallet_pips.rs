//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_pips::WeightInfo for WeightInfo {
    fn set_prune_historical_pips() -> Weight {
        (67_546_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_min_proposal_deposit() -> Weight {
        (68_599_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_proposal_cool_off_period() -> Weight {
        (67_206_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_enactment_period() -> Weight {
        (67_416_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_pending_pip_expiry() -> Weight {
        (69_240_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_max_pip_skip_count() -> Weight {
        (66_935_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_active_pip_limit() -> Weight {
        (67_066_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn propose_from_community() -> Weight {
        (502_902_000 as Weight)
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(9 as Weight))
    }
    fn propose_from_committee() -> Weight {
        (305_221_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn amend_proposal() -> Weight {
        (283_621_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn cancel_proposal() -> Weight {
        (590_064_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn vote() -> Weight {
        (222_567_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn approve_committee_proposal() -> Weight {
        (252_865_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn reject_proposal() -> Weight {
        (309_019_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn prune_proposal() -> Weight {
        (172_694_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn reschedule_execution() -> Weight {
        (302_346_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn clear_snapshot() -> Weight {
        (60_313_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn snapshot() -> Weight {
        (6_764_184_000 as Weight)
            .saturating_add(DbWeight::get().reads(204 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn enact_snapshot_results() -> Weight {
        (1_223_380_619_000 as Weight)
            .saturating_add(DbWeight::get().reads(21007 as Weight))
            .saturating_add(DbWeight::get().writes(21003 as Weight))
    }
    fn execute_scheduled_pip() -> Weight {
        (26_442_104_000 as Weight)
            .saturating_add(DbWeight::get().reads(1205 as Weight))
            .saturating_add(DbWeight::get().writes(1203 as Weight))
    }
    fn expire_scheduled_pip() -> Weight {
        (26_742_547_000 as Weight)
            .saturating_add(DbWeight::get().reads(1204 as Weight))
            .saturating_add(DbWeight::get().writes(1202 as Weight))
    }
}
