//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_staking::WeightInfo for WeightInfo {
    fn bond() -> Weight {
        (161_990_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(5 as Weight))
    }
    fn bond_extra() -> Weight {
        (129_613_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn unbond() -> Weight {
        (118_398_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn withdraw_unbonded_update(s: u32) -> Weight {
        (108_951_000 as Weight)
            .saturating_add((97_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn withdraw_unbonded_kill(s: u32) -> Weight {
        (183_229_000 as Weight)
            .saturating_add((6_141_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(8 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn set_min_bond_threshold() -> Weight {
        (35_716_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn add_permissioned_validator() -> Weight {
        (109_711_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_permissioned_validator() -> Weight {
        (56_129_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_commission_cap(m: u32) -> Weight {
        (99_937_000 as Weight)
            .saturating_add((22_121_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(m as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(m as Weight)))
    }
    fn validate() -> Weight {
        (110_162_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn nominate(n: u32) -> Weight {
        (157_680_000 as Weight)
            .saturating_add((4_463_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn chill() -> Weight {
        (44_293_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn set_payee() -> Weight {
        (25_417_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_controller() -> Weight {
        (61_089_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    // WARNING! Some components were not used: ["c"]
    fn set_validator_count() -> Weight {
        (4_176_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn force_no_eras() -> Weight {
        (5_140_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn force_new_era() -> Weight {
        (5_336_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn force_new_era_always() -> Weight {
        (4_615_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_invulnerables(v: u32) -> Weight {
        (5_008_000 as Weight)
            .saturating_add((9_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn force_unstake(s: u32) -> Weight {
        (112_144_000 as Weight)
            .saturating_add((6_127_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(8 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn cancel_deferred_slash(s: u32) -> Weight {
        (8_079_708_000 as Weight)
            .saturating_add((50_218_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    // WARNING! Some components were not used: ["n"]
    fn payout_stakers(_: u32) -> Weight {
        (413_730_000 as Weight)
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    // WARNING! Some components were not used: ["n"]
    fn payout_stakers_alive_controller() -> Weight {
        (408_981_000 as Weight)
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn rebond(l: u32) -> Weight {
        (68_629_000 as Weight)
            .saturating_add((340_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn set_history_depth(e: u32) -> Weight {
        (0 as Weight)
            .saturating_add((53_169_000 as Weight).saturating_mul(e as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
            .saturating_add(DbWeight::get().writes((7 as Weight).saturating_mul(e as Weight)))
    }
    fn reap_stash(s: u32) -> Weight {
        (88_084_000 as Weight)
            .saturating_add((6_965_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(8 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn new_era(v: u32, n: u32) -> Weight {
        (0 as Weight)
            .saturating_add((2_845_286_000 as Weight).saturating_mul(v as Weight))
            .saturating_add((364_465_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(19 as Weight))
            .saturating_add(DbWeight::get().reads((9 as Weight).saturating_mul(v as Weight)))
            .saturating_add(DbWeight::get().reads((7 as Weight).saturating_mul(n as Weight)))
            .saturating_add(DbWeight::get().writes(8 as Weight))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(v as Weight)))
    }
    fn do_slash(l: u32) -> Weight {
        (116_098_000 as Weight)
            .saturating_add((468_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn payout_all(v: u32, n: u32) -> Weight {
        (0 as Weight)
            .saturating_add((8_314_398_000 as Weight).saturating_mul(v as Weight))
            .saturating_add((749_822_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads((24 as Weight).saturating_mul(v as Weight)))
            .saturating_add(DbWeight::get().reads((5 as Weight).saturating_mul(n as Weight)))
            .saturating_add(DbWeight::get().writes((13 as Weight).saturating_mul(v as Weight)))
            .saturating_add(DbWeight::get().writes((3 as Weight).saturating_mul(n as Weight)))
    }
    // WARNING! Some components were not used: ["v", "n"]
    fn submit_solution_initial(a: u32, w: u32, _: u32) -> Weight {
        (15_694_910_000 as Weight)
            .saturating_add((191_581_000 as Weight).saturating_mul(a as Weight))
            .saturating_add((18_612_000 as Weight).saturating_mul(w as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(a as Weight)))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(w as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    // WARNING! Some components were not used: ["n"]
    fn submit_solution_better(v: u32, a: u32, w: u32, _: u32) -> Weight {
        (0 as Weight)
            .saturating_add((328_000 as Weight).saturating_mul(v as Weight))
            .saturating_add((189_604_000 as Weight).saturating_mul(a as Weight))
            .saturating_add((13_153_000 as Weight).saturating_mul(w as Weight))
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().reads((4 as Weight).saturating_mul(a as Weight)))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(w as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    // WARNING! Some components were not used: ["n"]
    fn submit_solution_weaker(v: u32) -> Weight {
        (124_733_000 as Weight)
            .saturating_add((2_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
    }
    fn change_slashing_allowed_for() -> Weight {
        (32_227_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn update_permissioned_validator_intended_count() -> Weight {
        (32_536_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn increase_validator_count() -> Weight {
        (12_802_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn scale_validator_count() -> Weight {
        (13_185_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
}
