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

/// Meters consumed weight and a hard limit for the maximal consumable weight.
pub struct WeightMeter(FrameWeightMeter);

impl WeightMeter {
    /// Creates [`Self`] from a limit for the maximal consumable weight.
    pub fn from_limit(limit: Weight) -> Self {
        WeightMeter(FrameWeightMeter::from_limit(limit))
    }

    /// Creates Self with the maximal possible limit for the consumable weight.
    pub fn max_limit() -> Self {
        WeightMeter(FrameWeightMeter::max_limit())
    }

    /// Returns the consumed weight.
    pub fn consumed(&self) -> Weight {
        self.0.consumed
    }

    /// Returns the maximum weight limit.
    pub fn limit(&self) -> Weight {
        self.0.limit
    }

    /// Consume the given weight after checking that it can be consumed. Returns an error otherwise.
    pub fn check_accrue(&mut self, weight: Weight) -> Result<(), String> {
        if !self.0.check_accrue(weight) {
            return Err(String::from("Maximum weight limit exceeded"));
        }
        Ok(())
    }

    /// Consumes the given `weight`.
    /// If the new consumed weight is greater than the limit, consumed will be set to limit and an error will be returned.
    pub fn consume_weght_until_limit(&mut self, weight: Weight) -> Result<(), String> {
        let updated_weight = self
            .0
            .consumed
            .checked_add(&weight)
            .ok_or(String::from("Weight value overflow"))?;
        if updated_weight.any_gt(self.0.limit) {
            self.0.consumed = self.0.limit;
            return Err(String::from("Maximum weight limit exceeded"));
        }
        self.0.consumed = updated_weight;
        Ok(())
    }
}
