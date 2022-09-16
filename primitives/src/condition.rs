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

use crate::{Claim, ClaimType, IdentityId};
use codec::{Decode, Encode};
use core::iter;
use either::Either;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::convert::TryInto;
use sp_std::prelude::*;

/// Defines a static / dynamic identity.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TargetIdentity {
    /// Matches any of the external agents of an asset. Resolved dynamically.
    ExternalAgent,
    /// A static identity.
    Specific(IdentityId),
}

/// It defines the type of condition supported, and the filter information we will use to evaluate as a
/// predicate.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ConditionType {
    /// Condition to ensure that claim filter produces one claim.
    IsPresent(Claim),
    /// Condition to ensure that claim filter produces an empty list.
    IsAbsent(Claim),
    /// Condition to ensure that at least one claim is fetched when filter is applied.
    IsAnyOf(Vec<Claim>),
    /// Condition to ensure that at none of claims is fetched when filter is applied.
    IsNoneOf(Vec<Claim>),
    /// Condition to ensure that the sender/receiver is a particular identity or an external agent.
    IsIdentity(TargetIdentity),
}

impl ConditionType {
    /// Return the number of `Claim` or `TargetIdentity`.
    fn count(&self) -> usize {
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
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TrustedFor {
    /// Issuer is trusted for any `ClaimType`.
    Any,
    /// Issuer is trusted only for the specific `ClaimType`s contained within.
    Specific(Vec<ClaimType>),
}

/// A trusted issuer for a certain compliance `Condition` and what `ClaimType`s is trusted for.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
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

    /// Count number of claim types this issuers is trusted for.
    ///
    /// Returns `1` for `TrustedFor::Any`.
    fn count(&self) -> usize {
        match &self.trusted_for {
            TrustedFor::Any => 1,
            TrustedFor::Specific(types) => types.len(),
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
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
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
    pub fn complexity(&self, default_issuer_count: usize) -> u32 {
        let issuers = match self.issuers.len() {
            0 => default_issuer_count,
            count => count,
        };
        self.condition_type
            .count()
            // NB: `max(1)` makes sure issuer count is not zero.
            .saturating_mul(issuers.max(1))
            .try_into()
            .unwrap_or(u32::MAX)
    }

    /// Return number of claims, issuers, and claim_types.
    ///
    /// This is used for weight calculation.
    ///
    /// Returns: `(claims_count, issuer_count, claim_type_count)`
    fn counts(&self) -> (u32, u32, u32) {
        // Count the number of claims.
        let claims = self.condition_type.count().try_into().unwrap_or(u32::MAX);
        // Count the number of issuers.
        let issuers = self.issuers.len().try_into().unwrap_or(u32::MAX);
        // Count the total number of claim types in all issuers.
        let claim_types = self
            .issuers
            .iter()
            .fold(0usize, |count, issuer| count.saturating_add(issuer.count()))
            .try_into()
            .unwrap_or(u32::MAX);

        (claims, issuers, claim_types)
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

/// Return the total number of condtions, claims, issuers, and claim_types.
///
/// This is used for weight calculation.
///
/// Returns: `(condition_count, claims_count, issuer_count, claim_type_count)`
pub fn conditions_total_counts<'a>(
    conditions: impl IntoIterator<Item = &'a Condition>,
) -> (u32, u32, u32, u32) {
    // Count the total number of claims, issuers, and claim_types in all conditions.
    conditions.into_iter().fold(
        (0u32, 0u32, 0u32, 0u32),
        |(count, total_claims, total_issuers, total_claim_types), condition| {
            let (claims, issuers, claim_types) = condition.counts();
            (
                count.saturating_add(1),
                total_claims.saturating_add(claims),
                total_issuers.saturating_add(issuers),
                total_claim_types.saturating_add(claim_types),
            )
        },
    )
}

impl From<ConditionType> for Condition {
    fn from(condition_type: ConditionType) -> Self {
        Condition::new(condition_type, Vec::new())
    }
}
