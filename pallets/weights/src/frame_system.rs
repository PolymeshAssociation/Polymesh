//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0-rc6

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;
impl frame_system::WeightInfo for WeightInfo {
    // WARNING! Some components were not used: ["b"]
    fn remark() -> Weight {
        (516000 as Weight)
    }
    fn set_heap_pages() -> Weight {
        (1207000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_changes_trie_config() -> Weight {
        (6928000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn set_storage(i: u32) -> Weight {
        (0 as Weight)
            .saturating_add((495000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
    }
    fn kill_storage(i: u32) -> Weight {
        (0 as Weight)
            .saturating_add((456000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
    }
    fn kill_prefix(p: u32) -> Weight {
        (5109000 as Weight)
            .saturating_add((988000 as Weight).saturating_mul(p as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
    }
    fn suicide() -> Weight {
        (18711000 as Weight)
    }
}
