//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_identity::WeightInfo for WeightInfo {
    fn cdd_register_did(i: u32) -> Weight {
        (272_510_000 as Weight)
            .saturating_add((57_862_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(15 as Weight))
            .saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(i as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
    fn invalidate_cdd_claims() -> Weight {
        (181_671_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn remove_secondary_keys(i: u32) -> Weight {
        (175_384_000 as Weight)
            .saturating_add((30_590_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().reads((3 as Weight).saturating_mul(i as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn accept_primary_key() -> Weight {
        (257_991_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(7 as Weight))
    }
    fn change_cdd_requirement_for_mk_rotation() -> Weight {
        (32_914_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn join_identity_as_key() -> Weight {
        (236_197_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn join_identity_as_identity() -> Weight {
        (198_296_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn leave_identity_as_key() -> Weight {
        (163_285_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn leave_identity_as_identity() -> Weight {
        (112_937_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_claim() -> Weight {
        (171_882_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn revoke_claim() -> Weight {
        (159_965_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn revoke_claim_by_index() -> Weight {
        (140_936_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_permission_to_signer() -> Weight {
        (117_012_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn freeze_secondary_keys() -> Weight {
        (155_653_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn unfreeze_secondary_keys() -> Weight {
        (159_477_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_authorization() -> Weight {
        (116_041_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn remove_authorization() -> Weight {
        (149_911_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn revoke_offchain_authorization() -> Weight {
        (119_376_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_investor_uniqueness_claim() -> Weight {
        (2_301_277_000 as Weight)
            .saturating_add(DbWeight::get().reads(12 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_investor_uniqueness_claim_v2() -> Weight {
        (3_971_380_000 as Weight)
            .saturating_add(DbWeight::get().reads(15 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
}
