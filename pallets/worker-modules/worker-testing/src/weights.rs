// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2026 Polymesh

//! Weights for `pallet_worker_testing`.
//!
//! These are NOT benchmarked. `pallet_worker_testing` is a test-only pallet that is only wired
//! into the `develop` runtime, so every call is charged zero weight.

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_primitives::{RocksDbWeight as DbWeight, Weight};

/// Zero weights for `pallet_worker_testing`.
pub struct SubstrateWeight;
impl crate::WeightInfo for SubstrateWeight {
    fn test_version() -> Weight {
        Weight::zero()
    }
    fn submit_work_request() -> Weight {
        Weight::zero()
    }
    fn set_protocol_version() -> Weight {
        Weight::zero()
    }
    fn set_enable_work_session() -> Weight {
        Weight::zero()
    }
    fn on_init() -> Weight {
        Weight::zero()
    }
}
