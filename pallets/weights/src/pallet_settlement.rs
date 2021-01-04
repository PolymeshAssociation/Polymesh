//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_settlement::WeightInfo for WeightInfo {
	fn create_venue(d: u32, s: u32, ) -> Weight {
		(35_964_000 as Weight)
			.saturating_add((8_000 as Weight).saturating_mul(d as Weight))
			.saturating_add((9_949_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
	}
	fn update_venue(d: u32, ) -> Weight {
		(97_675_000 as Weight)
			.saturating_add((9_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn add_instruction(l: u32, ) -> Weight {
		(152_295_000 as Weight)
			.saturating_add((43_097_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
			.saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
	}
	fn add_instruction_with_settle_on_block_type(l: u32, ) -> Weight {
		(294_553_000 as Weight)
			.saturating_add((64_472_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(3 as Weight))
			.saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
	}
	fn add_and_affirm_instruction(l: u32, ) -> Weight {
		(395_808_000 as Weight)
			.saturating_add((267_614_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(4 as Weight))
			.saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
	}
	fn add_and_affirm_instruction_with_settle_on_block_type(l: u32, ) -> Weight {
		(473_229_000 as Weight)
			.saturating_add((154_220_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(10 as Weight))
			.saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(6 as Weight))
			.saturating_add(DbWeight::get().writes((6 as Weight).saturating_mul(l as Weight)))
	}
	fn set_venue_filtering() -> Weight {
		(175_328_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_venue_filtering_disallow() -> Weight {
		(175_828_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn allow_venues(v: u32, ) -> Weight {
		(272_932_000 as Weight)
			.saturating_add((8_668_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
	}
	fn disallow_venues(v: u32, ) -> Weight {
		(246_220_000 as Weight)
			.saturating_add((6_935_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(v as Weight)))
	}
	fn withdraw_affirmation(l: u32, ) -> Weight {
		(23_476_000 as Weight)
			.saturating_add((209_523_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
			.saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
	}
	fn withdraw_affirmation_with_receipt(l: u32, ) -> Weight {
		(140_720_000 as Weight)
			.saturating_add((113_833_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
			.saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
	}
	fn unclaim_receipt() -> Weight {
		(170_668_000 as Weight)
			.saturating_add(DbWeight::get().reads(10 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn reject_instruction(l: u32, ) -> Weight {
		(173_449_000 as Weight)
			.saturating_add((139_795_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(3 as Weight))
			.saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(l as Weight)))
	}
	fn reject_instruction_with_no_pre_affirmations(l: u32, ) -> Weight {
		(177_026_000 as Weight)
			.saturating_add((54_983_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(3 as Weight))
			.saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(l as Weight)))
	}
	fn affirm_instruction(l: u32, ) -> Weight {
		(0 as Weight)
			.saturating_add((147_286_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(7 as Weight))
			.saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(2 as Weight))
			.saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(l as Weight)))
	}
	fn claim_receipt() -> Weight {
		(582_962_000 as Weight)
			.saturating_add(DbWeight::get().reads(11 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn affirm_with_receipts(r: u32, ) -> Weight {
		(909_386_000 as Weight)
			.saturating_add((209_440_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(r as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
			.saturating_add(DbWeight::get().writes((4 as Weight).saturating_mul(r as Weight)))
	}
	fn execute_scheduled_instruction(l: u32, s: u32, c: u32, ) -> Weight {
		(0 as Weight)
			.saturating_add((4_097_117_000 as Weight).saturating_mul(l as Weight))
			.saturating_add((3_695_656_000 as Weight).saturating_mul(s as Weight))
			.saturating_add((213_719_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads((13 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(s as Weight)))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(DbWeight::get().writes(7 as Weight))
			.saturating_add(DbWeight::get().writes((10 as Weight).saturating_mul(l as Weight)))
	}
}
