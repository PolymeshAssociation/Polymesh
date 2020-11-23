// TODO

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_pips::WeightInfo for WeightInfo {
    fn set_prune_historical_pips() -> Weight {
        0 as Weight
    }
    fn set_min_proposal_deposit() -> Weight {
        0 as Weight
    }
    fn set_proposal_cool_off_period() -> Weight {
        0 as Weight
    }
    fn set_default_enactment_period() -> Weight {
        0 as Weight
    }
    fn set_pending_pip_expiry() -> Weight {
        0 as Weight
    }
    fn set_max_pip_skip_count() -> Weight {
        0 as Weight
    }
    fn set_active_pip_limit() -> Weight {
        0 as Weight
    }
    fn propose_from_community() -> Weight {
        0 as Weight
    }
    fn propose_from_committee() -> Weight {
        0 as Weight
    }
    fn amend_proposal() -> Weight {
        0 as Weight
    }
    fn cancel_proposal() -> Weight {
        0 as Weight
    }
    fn vote() -> Weight {
        0 as Weight
    }
    fn approve_committee_proposal() -> Weight {
        0 as Weight
    }
    fn reject_proposal() -> Weight {
        0 as Weight
    }
    fn prune_proposal() -> Weight {
        0 as Weight
    }
    fn reschedule_execution() -> Weight {
        0 as Weight
    }
    fn clear_snapshot() -> Weight {
        0 as Weight
    }
    fn snapshot() -> Weight {
        0 as Weight
    }
    fn enact_snapshot_results() -> Weight {
        0 as Weight
    }
    fn execute_scheduled_pip() -> Weight {
        0 as Weight
    }
    fn expire_scheduled_pip() -> Weight {
        0 as Weight
    }
}
