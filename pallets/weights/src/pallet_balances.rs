//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl pallet_balances::WeightInfo for WeightInfo {
    fn transfer() -> Weight {
        (194_714_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn transfer_with_memo() -> Weight {
        (193_980_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn deposit_block_reward_reserve_balance() -> Weight {
        (189_337_000 as Weight)
            .saturating_add(DbWeight::get().reads(10 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn set_balance() -> Weight {
        (122_723_000 as Weight)
            .saturating_add(DbWeight::get().reads(5 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn force_transfer() -> Weight {
        (126_297_000 as Weight)
            .saturating_add(DbWeight::get().reads(6 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn burn_account_balance() -> Weight {
        (156_705_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
}
