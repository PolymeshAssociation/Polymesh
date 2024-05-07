// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::condition::{conditions_total_counts, Condition};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

/// A compliance requirement.
/// All sender and receiver conditions of the same compliance requirement must be true in order to execute the transfer.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, Default, Clone, PartialEq, Eq, Debug)]
pub struct ComplianceRequirement {
    /// List of sender conditions
    pub sender_conditions: Vec<Condition>,
    /// List of receiver conditions
    pub receiver_conditions: Vec<Condition>,
    /// Unique identifier of the compliance requirement
    pub id: u32,
}

impl ComplianceRequirement {
    /// Dedup `ClaimType`s in `TrustedFor::Specific`.
    pub fn dedup(&mut self) {
        let dedup_condition = |conds: &mut [Condition]| {
            conds
                .iter_mut()
                .flat_map(|c| &mut c.issuers)
                .for_each(|issuer| issuer.dedup())
        };
        dedup_condition(&mut self.sender_conditions);
        dedup_condition(&mut self.receiver_conditions);
    }

    /// Returns an iterator for all conditions in this requirement.
    pub fn conditions(&self) -> impl Iterator<Item = &Condition> {
        self.sender_conditions
            .iter()
            .chain(self.receiver_conditions.iter())
    }

    /// Return the total number of conditions, claims, issuers, and claim_types.
    ///
    /// This is used for weight calculation.
    pub fn counts(&self) -> (u32, u32, u32, u32) {
        conditions_total_counts(self.conditions())
    }
}

/// A compliance requirement along with its evaluation result
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Hash)]
pub struct ComplianceRequirementResult {
    /// List of sender conditions
    pub sender_conditions: Vec<ConditionResult>,
    /// List of receiver conditions
    pub receiver_conditions: Vec<ConditionResult>,
    /// Unique identifier of the compliance requirement.
    pub id: u32,
    /// Result of this transfer condition's evaluation.
    pub result: bool,
}

impl From<ComplianceRequirement> for ComplianceRequirementResult {
    fn from(requirement: ComplianceRequirement) -> Self {
        let from_conds = |conds: Vec<_>| conds.into_iter().map(ConditionResult::from).collect();
        Self {
            sender_conditions: from_conds(requirement.sender_conditions),
            receiver_conditions: from_conds(requirement.receiver_conditions),
            id: requirement.id,
            result: true,
        }
    }
}

/// An individual condition along with its evaluation result
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Hash)]
pub struct ConditionResult {
    /// Condition being evaluated
    pub condition: Condition,
    /// Result of evaluation
    pub result: bool,
}

impl From<Condition> for ConditionResult {
    fn from(condition: Condition) -> Self {
        Self {
            condition,
            result: true,
        }
    }
}

/// List of compliance requirements associated to an asset.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, TypeInfo, Default, Clone, PartialEq, Eq)]
pub struct AssetCompliance {
    /// This flag indicates if asset compliance should be enforced
    pub paused: bool,
    /// List of compliance requirements.
    pub requirements: Vec<ComplianceRequirement>,
}

/// Asset compliance and it's evaluation result.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Hash)]
pub struct AssetComplianceResult {
    /// This flag indicates if asset compliance should be enforced.
    pub paused: bool,
    /// List of compliance requirements.
    pub requirements: Vec<ComplianceRequirementResult>,
    /// Final evaluation result of the asset compliance.
    pub result: bool,
}

impl From<AssetCompliance> for AssetComplianceResult {
    fn from(asset_compliance: AssetCompliance) -> Self {
        Self {
            paused: asset_compliance.paused,
            requirements: asset_compliance
                .requirements
                .into_iter()
                .map(ComplianceRequirementResult::from)
                .collect(),
            result: asset_compliance.paused,
        }
    }
}

/// Holds detailed information for all asset's requirements.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, TypeInfo)]
pub struct ComplianceReport {
    /// Set to `true` if any requirement is satisfied.
    any_requirement_satistifed: bool,
    /// Set to `true` if the asset compliance is paused.
    paused_compliance: bool,
    /// All [`RequirementReport`] containg the info for each of the asset's requirement.
    requirements: Vec<RequirementReport>,
}

impl ComplianceReport {
    /// Creates a new [`ComplianceReport`] instance.
    pub fn new(
        requirements: Vec<RequirementReport>,
        any_requirement_satistifed: bool,
        paused_compliance: bool,
    ) -> Self {
        Self {
            any_requirement_satistifed,
            paused_compliance,
            requirements,
        }
    }

    /// Returns [`Self::any_requirement_satistifed`].
    pub fn is_any_requirement_satisfied(&self) -> bool {
        self.any_requirement_satistifed
    }

    /// Returns the [`RequirementReport`] for the given `index`.
    pub fn get_requirement(&self, index: usize) -> Option<&RequirementReport> {
        self.requirements.get(index)
    }
}

/// Holds the information for an individual asset requirement.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, TypeInfo)]
pub struct RequirementReport {
    /// Set to `true` if all conditions are satisfied.
    requirement_satisfied: bool,
    /// Unique identifier of the compliance requirement.
    id: u32,
    /// All sender [`ConditionReport`].
    sender_conditions: Vec<ConditionReport>,
    /// All receiver [`ConditionReport`].
    receiver_conditions: Vec<ConditionReport>,
}

impl RequirementReport {
    /// Creates a new [`RequirementReport`] instance.
    pub fn new(
        sender_conditions: Vec<ConditionReport>,
        receiver_conditions: Vec<ConditionReport>,
        id: u32,
        requirement_satisfied: bool,
    ) -> Self {
        Self {
            requirement_satisfied,
            id,
            sender_conditions,
            receiver_conditions,
        }
    }

    /// Returns [`Self::requirement_satisfied`].
    pub fn is_requirement_satisfied(&self) -> bool {
        self.requirement_satisfied
    }

    /// Returns the sender [`ConditionReport`] for the given `index`.
    pub fn get_sender_condition(&self, index: usize) -> Option<&ConditionReport> {
        self.sender_conditions.get(index)
    }

    /// Returns the receiver [`ConditionReport`] for the given `index`.
    pub fn get_receiver_condition(&self, index: usize) -> Option<&ConditionReport> {
        self.receiver_conditions.get(index)
    }
}

/// Holds the information for an individual condition.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, TypeInfo)]
pub struct ConditionReport {
    /// Set to `true` if the condition is satisfied.
    pub satisfied: bool,
    /// The [`Condition`] assessed.
    pub condition: Condition,
}

impl ConditionReport {
    /// Creates a new [`ConditionReport`] instance.
    pub fn new(condition: Condition, satisfied: bool) -> Self {
        Self {
            satisfied,
            condition,
        }
    }

    /// Returns [`Self::satisfied`].
    pub fn is_condition_satisfied(&self) -> bool {
        self.satisfied
    }

    /// Returns [`Self.condition`].
    pub fn condition(&self) -> &Condition {
        &self.condition
    }
}
