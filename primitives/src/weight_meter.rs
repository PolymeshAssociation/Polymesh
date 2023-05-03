// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2021 Polymath

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

struct WeightMeterWithMinimum {
    minimum_charge: Weight,
    meter: FrameWeightMeter,
}

impl WeightMeterWithMinimum {
    fn new(minimum_charge: Weight, meter: FrameWeightMeter) -> Self {
        Self {
            minimum_charge,
            meter,
        }
    }
}

/// Meters consumed weight and a hard limit for the maximal consumable weight.
pub struct WeightMeter(WeightMeterWithMinimum);

impl WeightMeter {
    /// Creates [`Self`] from a limit for the maximal consumable weight and a minimum charge of `minimum_charge`.
    pub fn from_limit(minimum_charge: Weight, limit: Weight) -> Self {
        let weight_meter =
            WeightMeterWithMinimum::new(minimum_charge, FrameWeightMeter::from_limit(limit));
        WeightMeter(weight_meter)
    }

    /// Creates [`Self`] with the maximal possible limit for the consumable weight and a minimum charge of `minimum_charge`.
    pub fn max_limit(minimum_charge: Weight) -> Self {
        let weight_meter =
            WeightMeterWithMinimum::new(minimum_charge, FrameWeightMeter::max_limit());
        WeightMeter(weight_meter)
    }

    /// Creates [`Self`] with the maximal possible limit for the consumable weight and no minimum charge.
    pub fn max_limit_no_minimum() -> Self {
        let weight_meter =
            WeightMeterWithMinimum::new(Weight::zero(), FrameWeightMeter::max_limit());
        WeightMeter(weight_meter)
    }

    /// Returns the minimum charge if the consumed weight is less than the minimum, otherwise returns the consumed weight.
    pub fn consumed(&self) -> Weight {
        if self.0.meter.consumed.all_lt(self.0.minimum_charge) {
            return self.0.minimum_charge;
        }
        self.0.meter.consumed
    }

    /// Returns the maximum weight limit.
    pub fn limit(&self) -> Weight {
        self.0.meter.limit
    }

    /// Consumes the given weight after checking that it can be consumed. Returns an error otherwise.
    pub fn check_accrue(&mut self, weight: Weight) -> Result<(), String> {
        if !self.0.meter.check_accrue(weight) {
            return Err(String::from("Maximum weight limit exceeded"));
        }
        Ok(())
    }

    /// Consumes the given `weight`.
    /// If the new consumed weight is greater than the limit, consumed will be set to limit and an error will be returned.
    pub fn consume_weight_until_limit(&mut self, weight: Weight) -> Result<(), String> {
        if !self.0.meter.check_accrue(weight) {
            self.0.meter.consumed = self.0.meter.limit;
            return Err(String::from("Maximum weight limit exceeded"));
        }
        Ok(())
    }
}
