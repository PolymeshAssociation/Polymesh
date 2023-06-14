// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Default weights for the Babe Pallet
//! This file was not auto-generated.

use polymesh_runtime_common::{
    RocksDbWeight as DbWeight, Weight, WEIGHT_REF_TIME_PER_MICROS, WEIGHT_REF_TIME_PER_NANOS,
};

/// Weights for pallet_babe using the Substrate node and recommended hardware.
pub struct SubstrateWeight;
impl pallet_babe::WeightInfo for SubstrateWeight {
    fn plan_config_change() -> Weight {
        Weight::zero()
    }

    // WARNING! Some components were not used: ["x"]
    fn report_equivocation(validator_count: u32) -> Weight {
        // we take the validator set count from the membership proof to
        // calculate the weight but we set a floor of 100 validators.
        let validator_count = validator_count.max(100) as u64;

        // worst case we are considering is that the given offender
        // is backed by 200 nominators
        const MAX_NOMINATORS: u64 = 200;

        // checking membership proof
        Weight::from_ref_time(35u64 * WEIGHT_REF_TIME_PER_MICROS)
            .saturating_add(
                Weight::from_ref_time(175u64 * WEIGHT_REF_TIME_PER_NANOS)
                    .saturating_mul(validator_count),
            )
            .saturating_add(DbWeight::get().reads(5))
            // check equivocation proof
            .saturating_add(Weight::from_ref_time(110u64 * WEIGHT_REF_TIME_PER_MICROS))
            // report offence
            .saturating_add(Weight::from_ref_time(110u64 * WEIGHT_REF_TIME_PER_MICROS))
            .saturating_add(Weight::from_ref_time(
                25u64 * WEIGHT_REF_TIME_PER_MICROS * MAX_NOMINATORS,
            ))
            .saturating_add(DbWeight::get().reads(14 + 3 * MAX_NOMINATORS))
            .saturating_add(DbWeight::get().writes(10 + 3 * MAX_NOMINATORS))
    }
}
