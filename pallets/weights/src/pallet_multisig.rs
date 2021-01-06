//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_multisig::WeightInfo for WeightInfo {
    fn create_multisig(i: u32) -> Weight {
        (153_090_000 as Weight)
            .saturating_add((29_605_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
    fn create_or_approve_proposal_as_identity() -> Weight {
        (220_924_000 as Weight)
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(7 as Weight))
    }
    fn create_or_approve_proposal_as_key() -> Weight {
        (456_470_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(7 as Weight))
    }
    fn create_proposal_as_identity() -> Weight {
        (343_001_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(7 as Weight))
    }
    fn create_proposal_as_key() -> Weight {
        (260_906_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(7 as Weight))
    }
    fn approve_as_identity() -> Weight {
        (184_369_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn approve_as_key() -> Weight {
        (160_576_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn reject_as_identity() -> Weight {
        (127_077_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn reject_as_key() -> Weight {
        (105_077_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn accept_multisig_signer_as_identity() -> Weight {
        (147_207_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn accept_multisig_signer_as_key() -> Weight {
        (141_689_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn add_multisig_signer() -> Weight {
        (84_198_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn remove_multisig_signer() -> Weight {
        (81_636_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn add_multisig_signers_via_creator(i: u32) -> Weight {
        (0 as Weight)
            .saturating_add((45_151_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
    fn remove_multisig_signers_via_creator(i: u32) -> Weight {
        (162_217_000 as Weight)
            .saturating_add((33_020_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(i as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
    fn change_sigs_required() -> Weight {
        (74_364_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn make_multisig_signer() -> Weight {
        (95_135_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn make_multisig_primary() -> Weight {
        (113_158_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn execute_scheduled_proposal() -> Weight {
        (124_830_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
}
