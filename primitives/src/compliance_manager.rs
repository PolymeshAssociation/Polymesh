// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymath

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

/// An object for tracking the complexity of the `is_compliant` function.
#[derive(Clone, Debug, Decode, Eq, Encode, PartialEq, TypeInfo)]
pub struct ComplianceComplexity {
    /// Tracks the number of reads to the `TrustedClaimIssuer` storage.
    trusted_claims_calls: u32,
    /// Tracks the number of times that the `Identity::<T>::fetch_claim` was called.
    id_fetch_claim_calls: u32,
    /// Tracks the number of reads to the `agents` storage.
    external_agents_calls: u32,
}

impl ComplianceComplexity {
    /// Creates a new instance of `ComplianceComplexity`.
    pub fn new_2(t: u32, i: u32, e: u32) -> Self {
        ComplianceComplexity {
            trusted_claims_calls: t,
            id_fetch_claim_calls: i,
            external_agents_calls: e,
        }
    }

    /// Creates a new instance of `ComplianceComplexity`.
    pub fn new() -> Self {
        ComplianceComplexity {
            trusted_claims_calls: 0,
            id_fetch_claim_calls: 0,
            external_agents_calls: 0,
        }
    }

    /// Adds one to `trusted_claims_calls`.
    pub fn add_trusted_claims_call(&mut self) {
        self.trusted_claims_calls += 1;
    }

    /// Adds one to `id_fetch_claim_call`.
    pub fn add_id_fetch_claim_call(&mut self) {
        self.id_fetch_claim_calls += 1;
    }

    /// Adds one to `external_agents_calls`.
    pub fn add_external_agents_call(&mut self) {
        self.external_agents_calls += 1;
    }
}
