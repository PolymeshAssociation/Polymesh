//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_identity::WeightInfo for WeightInfo {
	fn register_did(i: u32, ) -> Weight {
		(1_022_823_000 as Weight)
			.saturating_add((44_949_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(11 as Weight))
			.saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(4 as Weight))
			.saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
	}
	fn cdd_register_did(i: u32, ) -> Weight {
		(193_927_000 as Weight)
			.saturating_add((42_471_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(13 as Weight))
			.saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(3 as Weight))
			.saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
	}
	fn mock_cdd_register_did() -> Weight {
		(1_049_263_000 as Weight)
			.saturating_add(DbWeight::get().reads(15 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
	fn invalidate_cdd_claims() -> Weight {
		(128_262_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn remove_secondary_keys(i: u32, ) -> Weight {
		(45_652_000 as Weight)
			.saturating_add((40_453_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
	}
	fn accept_primary_key() -> Weight {
		(201_914_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(7 as Weight))
	}
	fn change_cdd_requirement_for_mk_rotation() -> Weight {
		(23_214_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn join_identity_as_key() -> Weight {
		(143_735_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
	fn join_identity_as_identity() -> Weight {
		(142_565_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn leave_identity_as_key() -> Weight {
		(122_126_000 as Weight)
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn leave_identity_as_identity() -> Weight {
		(83_669_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn add_claim() -> Weight {
		(116_808_000 as Weight)
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn forwarded_call() -> Weight {
		(144_182_000 as Weight)
			.saturating_add(DbWeight::get().reads(14 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn add_investor_uniqueness_claim() -> Weight {
		(1_604_140_000 as Weight)
			.saturating_add(DbWeight::get().reads(10 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
