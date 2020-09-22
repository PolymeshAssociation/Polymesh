// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
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

use crate as polymesh_primitives;
use crate::{identity_claim::ClaimOld, Claim, IdentityId, Ticker};
use codec::{Decode, Encode};
use polymesh_primitives_derive::Migrate;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
/// It defines a static/dynamic identity
pub enum TargetIdentity {
    /// Current primary issuance agent of an asset. Resolved dynamically.
    PrimaryIssuanceAgent,
    /// A static identity.
    Specific(IdentityId),
}

/// It defines the type of condition supported, and the filter information we will use to evaluate as a
/// predicate.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Migrate)]
#[migrate_context(Option<crate::CddId>)]
pub enum ConditionType {
    /// Condition to ensure that claim filter produces one claim.
    IsPresent(#[migrate] Claim),
    /// Condition to ensure that claim filter produces an empty list.
    IsAbsent(#[migrate] Claim),
    /// Condition to ensure that at least one claim is fetched when filter is applied.
    IsAnyOf(#[migrate(Claim)] Vec<Claim>),
    /// Condition to ensure that at none of claims is fetched when filter is applied.
    IsNoneOf(#[migrate(Claim)] Vec<Claim>),
    /// Condition to ensure that the sender/receiver is a particular identity or primary issuance agent
    IsIdentity(TargetIdentity),
    /// Condition to ensure that the target identity has a valid `InvestorZKProof` claim for the given
    /// ticker.
    HasValidProofOfInvestor(Ticker),
}

/// Type of claim requirements that a condition can have
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Migrate)]
#[migrate_context(Option<crate::CddId>)]
pub struct Condition {
    /// Type of condition.
    #[migrate]
    pub condition_type: ConditionType,
    /// Trusted issuers.
    pub issuers: Vec<IdentityId>,
}

impl From<ConditionType> for Condition {
    fn from(condition_type: ConditionType) -> Self {
        Condition {
            condition_type,
            issuers: Vec::<IdentityId>::new(),
        }
    }
}

impl Condition {
    /// Returns worst case complexity of a condition
    pub fn complexity(&self) -> (usize, usize) {
        let claims_count = match self.condition_type {
            ConditionType::IsIdentity(..)
            | ConditionType::IsPresent(..)
            | ConditionType::IsAbsent(..) => 1,
            ConditionType::IsNoneOf(ref claims) | ConditionType::IsAnyOf(ref claims) => {
                claims.len()
            }
            // NOTE: The complexity of this condition implies the use of cryptography libraries, which
            // are computational expensive.
            // So we've added a 10 factor here.
            ConditionType::HasValidProofOfInvestor(..) => 10,
        };
        (claims_count, self.issuers.len())
    }
}
