//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, GetDispatchInfo, Weight};

fn sum_weights(calls: &[impl GetDispatchInfo]) -> Weight {
    calls
        .iter()
        .map(|call| call.get_dispatch_info().weight)
        .fold(0 as Weight, |a: Weight, n| a.saturating_add(n))
}

pub struct WeightInfo;
impl pallet_utility::WeightInfo for WeightInfo {
    fn batch(calls: &[impl GetDispatchInfo]) -> Weight {
        (1_018_140_000 as Weight)
            .saturating_add(sum_weights(calls))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }

    fn batch_atomic(calls: &[impl GetDispatchInfo]) -> Weight {
        (1_022_436_000 as Weight)
            .saturating_add(sum_weights(calls))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }

    fn batch_optimistic(calls: &[impl GetDispatchInfo]) -> Weight {
        (771_080_000 as Weight)
            .saturating_add(sum_weights(calls))
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }

    fn relay_tx(call: &impl GetDispatchInfo) -> Weight {
        (1_000_000 as Weight).saturating_add(call.get_dispatch_info().weight)
    }
}
