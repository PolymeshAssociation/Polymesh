//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_multisig::WeightInfo for WeightInfo {
    fn create_multisig(i: u32, ) -> Weight {
        (380_536_000 as Weight)
            .saturating_add((6_271_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(6 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
    fn create_or_approve_proposal_as_identity() -> Weight {
        (140_441_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn create_or_approve_proposal_as_key() -> Weight {
        (152_003_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn create_proposal_as_identity() -> Weight {
        (138_650_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn create_proposal_as_key() -> Weight {
        (85_324_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn approve_as_identity() -> Weight {
        (83_801_000 as Weight)
            .saturating_add(DbWeight::get().reads(14 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn approve_as_key() -> Weight {
        (69_294_000 as Weight)
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn reject_as_identity() -> Weight {
        (50_067_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn reject_as_key() -> Weight {
        (42_928_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn accept_multisig_signer_as_identity() -> Weight {
        (48_075_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn accept_multisig_signer_as_key() -> Weight {
        (50_790_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn add_multisig_signer() -> Weight {
        (45_206_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn remove_multisig_signer() -> Weight {
        (42_960_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn add_multisig_signers_via_creator(i: u32, ) -> Weight {
        (46_415_000 as Weight)
            .saturating_add((10_725_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
    fn remove_multisig_signers_via_creator(i: u32, ) -> Weight {
        (135_618_000 as Weight)
            .saturating_add((10_111_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(i as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
    fn change_sigs_required() -> Weight {
        (35_858_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn change_all_signers_and_sigs_required() -> Weight {
        (4_614_926_000 as Weight)
            .saturating_add(DbWeight::get().reads(261 as Weight))
            .saturating_add(DbWeight::get().writes(514 as Weight))
    }
    fn make_multisig_signer() -> Weight {
        (63_004_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn make_multisig_primary() -> Weight {
        (54_698_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
}
