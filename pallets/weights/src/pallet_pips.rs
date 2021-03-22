//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_pips::WeightInfo for WeightInfo {
    fn set_prune_historical_pips() -> Weight {
        (37_273_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_min_proposal_deposit() -> Weight {
        (32_917_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_default_enactment_period() -> Weight {
        (32_579_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_pending_pip_expiry() -> Weight {
        (33_103_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_max_pip_skip_count() -> Weight {
        (40_613_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_active_pip_limit() -> Weight {
        (41_187_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn propose_from_community() -> Weight {
        (322_734_000 as Weight)
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(10 as Weight))
    }
    fn propose_from_committee() -> Weight {
        (203_751_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn vote() -> Weight {
        (287_509_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn approve_committee_proposal() -> Weight {
        (367_488_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn reject_proposal() -> Weight {
        (508_132_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(10 as Weight))
    }
    fn prune_proposal() -> Weight {
        (244_234_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn reschedule_execution() -> Weight {
        (420_456_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
    }
    fn clear_snapshot() -> Weight {
        (107_890_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn snapshot() -> Weight {
        (1_050_995_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn enact_snapshot_results(a: u32, r: u32, s: u32) -> Weight {
        (0 as Weight)
            .saturating_add((632_873_000 as Weight).saturating_mul(a as Weight))
            .saturating_add((21_772_263_000 as Weight).saturating_mul(r as Weight))
            .saturating_add((694_197_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(689 as Weight))
            .saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(a as Weight)))
            .saturating_add(DbWeight::get().reads((405 as Weight).saturating_mul(r as Weight)))
            .saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(s as Weight)))
            .saturating_add(DbWeight::get().writes(685 as Weight))
            .saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(a as Weight)))
            .saturating_add(DbWeight::get().writes((404 as Weight).saturating_mul(r as Weight)))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(s as Weight)))
    }
    fn execute_scheduled_pip() -> Weight {
        (26_700_112_000 as Weight)
            .saturating_add(DbWeight::get().reads(1205 as Weight))
            .saturating_add(DbWeight::get().writes(1606 as Weight))
    }
    fn expire_scheduled_pip() -> Weight {
        (26_496_473_000 as Weight)
            .saturating_add(DbWeight::get().reads(1207 as Weight))
            .saturating_add(DbWeight::get().writes(1607 as Weight))
    }
}
