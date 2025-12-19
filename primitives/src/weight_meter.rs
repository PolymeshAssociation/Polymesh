// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2021 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use frame_support::weights::{Weight, WeightMeter as FrameWeightMeter};
use scale_info::prelude::string::String;

/// Meters consumed weight that contains a hard limit for the maximal consumable weight and a minimum weight to be charged.
#[derive(Debug)]
pub struct WeightMeter {
    minimum_charge: Weight,
    meter: FrameWeightMeter,
}

impl WeightMeter {
    /// Creates [`Self`] from a limit for the maximal consumable weight and a minimum charge of `minimum_charge` without checking if the limit is less than the minimum charge.
    pub fn from_limit_unchecked(minimum_charge: Weight, mut limit: Weight) -> Self {
        if limit.checked_sub(&minimum_charge).is_none() {
            // Log a warning to help with debugging issues with the minimum charge.
            log::warn!(
                "WeightMeter limit {:?} is less than minimum charge {:?}, setting limit to minimum charge",
                limit,
                minimum_charge
            );
            limit = minimum_charge;
        }

        Self {
            minimum_charge,
            meter: FrameWeightMeter::from_limit(limit),
        }
    }

    /// Creates [`Self`] from a limit for the maximal consumable weight and a minimum charge of `minimum_charge`.
    pub fn from_limit(minimum_charge: Weight, limit: Weight) -> Option<Self> {
        if limit.checked_sub(&minimum_charge).is_none() {
            return None;
        }

        Some(Self {
            minimum_charge,
            meter: FrameWeightMeter::from_limit(limit),
        })
    }

    /// Creates [`Self`] with the maximal possible limit for the consumable weight and a minimum charge of `minimum_charge`.
    pub fn max_limit(minimum_charge: Weight) -> Self {
        Self {
            minimum_charge,
            meter: FrameWeightMeter::max_limit(),
        }
    }

    /// Creates [`Self`] with the maximal possible limit for the consumable weight and no minimum charge.
    pub fn max_limit_no_minimum() -> Self {
        Self {
            minimum_charge: Weight::zero(),
            meter: FrameWeightMeter::max_limit(),
        }
    }

    /// Returns the minimum charge if the consumed weight is less than the minimum, otherwise returns the consumed weight.
    pub fn consumed(&self) -> Weight {
        if self.meter.consumed.ref_time() < self.minimum_charge.ref_time() {
            return self.minimum_charge;
        }
        self.meter.consumed
    }

    /// Returns the maximum weight limit.
    pub fn limit(&self) -> Weight {
        self.meter.limit
    }

    /// Consumes the given weight after checking that it can be consumed. Returns an error otherwise.
    pub fn check_accrue(&mut self, weight: Weight) -> Result<(), String> {
        if !self.meter.check_accrue(weight) {
            return Err(String::from("Maximum weight limit exceeded"));
        }
        Ok(())
    }

    /// Consumes the given `weight`.
    /// If the new consumed weight is greater than the limit, consumed will be set to limit and an error will be returned.
    pub fn consume_weight_until_limit(&mut self, weight: Weight) -> Result<(), String> {
        if !self.meter.check_accrue(weight) {
            self.meter.consumed = self.meter.limit;
            return Err(String::from("Maximum weight limit exceeded"));
        }
        Ok(())
    }
}
