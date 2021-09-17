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

use crate::{Claim, ClaimType, IdentityId};
use codec::{Decode, Encode};
use core::iter;
use either::Either;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Defines a static / dynamic identity.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TargetIdentity {
    /// Matches any of the external agents of an asset. Resolved dynamically.
    ExternalAgent,
    /// A static identity.
    Specific(IdentityId),
}

/// It defines the type of condition supported, and the filter information we will use to evaluate as a
/// predicate.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ConditionType {
    /// Condition to ensure that claim filter produces one claim.
    IsPresent(Claim),
    /// Condition to ensure that claim filter produces an empty list.
    IsAbsent(Claim),
    /// Condition to ensure that at least one claim is fetched when filter is applied.
    IsAnyOf(Vec<Claim>),
    /// Condition to ensure that at none of claims is fetched when filter is applied.
    IsNoneOf(Vec<Claim>),
    /// Condition to ensure that the sender/receiver is a particular identity or primary issuance agent
    IsIdentity(TargetIdentity),
}

impl ConditionType {
    fn complexity(&self) -> usize {
        match self {
            ConditionType::IsIdentity(..)
            | ConditionType::IsPresent(..)
            | ConditionType::IsAbsent(..) => 1,
            ConditionType::IsNoneOf(claims) | ConditionType::IsAnyOf(claims) => claims.len(),
        }
    }
}

/// Denotes the set of `ClaimType`s for which an issuer is trusted.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TrustedFor {
    /// Issuer is trusted for any `ClaimType`.
    Any,
    /// Issuer is trusted only for the specific `ClaimType`s contained within.
    Specific(Vec<ClaimType>),
}

/// A trusted issuer for a certain compliance `Condition` and what `ClaimType`s is trusted for.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Hash)]
pub struct TrustedIssuer {
    /// The issuer trusted for the `Condition` or for the `Ticker`,
    /// depending on where `TrustedClaimIssuer` is included.
    pub issuer: IdentityId,
    /// The set of `ClaimType`s for which `issuer` is trusted.
    pub trusted_for: TrustedFor,
}

impl TrustedIssuer {
    /// Deduplicate any `ClaimType`s in `TrustedFor::Specific`.
    pub fn dedup(&mut self) {
        match &mut self.trusted_for {
            TrustedFor::Any => {}
            TrustedFor::Specific(types) => {
                types.sort();
                types.dedup();
            }
        }
    }

    /// Is the given issuer trusted for `ty`?
    pub fn is_trusted_for(&self, ty: ClaimType) -> bool {
        match &self.trusted_for {
            TrustedFor::Any => true,
            TrustedFor::Specific(ok_types) => ok_types.contains(&ty),
        }
    }
}

/// Create a `TrustedIssuer` trusted for any claim type.
impl From<IdentityId> for TrustedIssuer {
    fn from(issuer: IdentityId) -> Self {
        Self {
            issuer,
            trusted_for: TrustedFor::Any,
        }
    }
}

/// Type of claim requirements that a condition can have
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Condition {
    /// Type of condition.
    pub condition_type: ConditionType,
    /// Trusted issuers.
    pub issuers: Vec<TrustedIssuer>,
}

impl Condition {
    /// Generate condition on the basis of `condition_type` & `issuers`.
    pub fn new(condition_type: ConditionType, issuers: Vec<TrustedIssuer>) -> Self {
        Self {
            condition_type,
            issuers,
        }
    }

    /// Create a condition with the given type and issuers trusted for any claim type.
    pub fn from_dids(condition_type: ConditionType, issuers: &[IdentityId]) -> Self {
        Self::new(
            condition_type,
            issuers.iter().copied().map(TrustedIssuer::from).collect(),
        )
    }

    /// Returns worst case complexity of a condition.
    pub fn complexity(&self) -> (usize, usize) {
        (self.condition_type.complexity(), self.issuers.len())
    }

    /// Returns all the claims in the condition.
    pub fn claims(&self) -> impl Iterator<Item = &Claim> {
        match &self.condition_type {
            ConditionType::IsPresent(c) | ConditionType::IsAbsent(c) => Either::Left(iter::once(c)),
            ConditionType::IsAnyOf(cs) | ConditionType::IsNoneOf(cs) => Either::Right(cs.iter()),
            ConditionType::IsIdentity(_) => Either::Right([].iter()),
        }
    }
}

impl From<ConditionType> for Condition {
    fn from(condition_type: ConditionType) -> Self {
        Condition::new(condition_type, Vec::new())
    }
}
