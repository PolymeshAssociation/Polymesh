//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_pips::WeightInfo for WeightInfo {
	fn set_prune_historical_pips() -> Weight {
		(75_621_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// WARNING! Some components were not used: ["b"]
	fn set_min_proposal_deposit() -> Weight {
		(71_886_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// WARNING! Some components were not used: ["b"]
	fn set_proposal_cool_off_period() -> Weight {
		(57_483_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// WARNING! Some components were not used: ["b"]
	fn set_default_enactment_period() -> Weight {
		(44_504_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// WARNING! Some components were not used: ["b"]
	fn set_pending_pip_expiry() -> Weight {
		(37_787_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// WARNING! Some components were not used: ["n"]
	fn set_max_pip_skip_count() -> Weight {
		(36_391_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// WARNING! Some components were not used: ["n"]
	fn set_active_pip_limit() -> Weight {
		(36_580_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn propose_from_community(c: u32, ) -> Weight {
		(46_414_000 as Weight)
			.saturating_add((5_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(13 as Weight))
			.saturating_add(DbWeight::get().writes(9 as Weight))
	}
	fn propose_from_committee(c: u32, ) -> Weight {
		(139_879_000 as Weight)
			.saturating_add((6_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn amend_proposal(c: u32, ) -> Weight {
		(70_185_000 as Weight)
			.saturating_add((6_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn cancel_proposal(c: u32, ) -> Weight {
		(384_069_000 as Weight)
			.saturating_add((8_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn vote(c: u32, v: u32, ) -> Weight {
		(0 as Weight)
			.saturating_add((8_000 as Weight).saturating_mul(c as Weight))
			.saturating_add((335_294_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn approve_committee_proposal(c: u32, ) -> Weight {
		(107_038_000 as Weight)
			.saturating_add((10_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn reject_proposal(c: u32, ) -> Weight {
		(0 as Weight)
			.saturating_add((21_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn prune_proposal(c: u32, ) -> Weight {
		(189_632_000 as Weight)
			.saturating_add((5_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn reschedule_execution(c: u32, ) -> Weight {
		(97_755_000 as Weight)
			.saturating_add((4_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	// WARNING! Some components were not used: ["c"]
	fn clear_snapshot() -> Weight {
		(86_542_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn snapshot(c: u32, ) -> Weight {
		(148_821_000 as Weight)
			.saturating_add((3_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn enact_snapshot_results(c: u32, ) -> Weight {
		(132_910_000 as Weight)
			.saturating_add((6_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
}
