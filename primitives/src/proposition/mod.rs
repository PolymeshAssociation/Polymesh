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

use crate::{Claim, Condition, ConditionType, IdentityId, TargetIdentity};
use codec::{Decode, Encode};

use sp_std::prelude::*;

/// Claims of a given identity that will be assessed.
#[derive(Encode, Decode, Clone, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct IdentityClaims {
    /// Identity id.
    pub id: IdentityId,
    /// Identity claims.
    pub claims: Vec<Claim>,
}

impl IdentityClaims {
    /// Creates a new `IdentityClaims` instance.
    pub fn new(id: IdentityId, claims: Vec<Claim>) -> Self {
        IdentityClaims { id, claims }
    }
}

// Proposition Trait
// ==================================

/// It allows composition and evaluation of claims based on a context.
pub trait Proposition {
    /// Evaluates this proposition based on `context`.
    fn evaluate(&self, identity_claims: &IdentityClaims) -> bool;

    /// It generates a new proposition that represents the logical AND
    /// of two propositions: `Self` and `other`.
    #[inline]
    fn and<B: Proposition>(self, other: B) -> AndProposition<Self, B>
    where
        Self: Sized,
    {
        AndProposition::new(self, other)
    }

    /// It generates a new proposition that represents the logical OR
    /// of two propositions: `Self` and `other`.
    #[inline]
    fn or<B: Proposition>(self, other: B) -> OrProposition<Self, B>
    where
        Self: Sized,
    {
        OrProposition::new(self, other)
    }

    /// It generates a new proposition that represents the logical NOT
    /// of this proposition.
    #[inline]
    fn not(self) -> NotProposition<Self>
    where
        Self: Sized,
    {
        NotProposition::new(self)
    }
}

impl<F: Fn(&IdentityClaims) -> bool> Proposition for F {
    fn evaluate(&self, identity_claims: &IdentityClaims) -> bool {
        self(identity_claims)
    }
}

/// Base and simple propositions
pub mod base;
pub use base::{
    AndProposition, AnyProposition, ExistentialProposition, IsIdentityProposition, NotProposition,
    OrProposition,
};

// Helper functions
// ======================================

/// It creates a proposition to evaluate the existential of `claim` in the context.
#[inline]
pub fn exists(claim: &Claim) -> ExistentialProposition<'_> {
    ExistentialProposition { claim }
}

/// It creates a proposition to evaluate if any of `claims` are found in the context.
#[inline]
pub fn any(claims: &[Claim]) -> AnyProposition<'_> {
    AnyProposition { claims }
}

/// It create a negate proposition of `proposition`.
#[inline]
pub fn not<P: Proposition>(proposition: P) -> NotProposition<P> {
    NotProposition::new(proposition)
}

/// Returns `true` if the condition is satisfied.
pub fn run<P: Proposition>(
    condition: &Condition,
    identity_claims: &IdentityClaims,
    external_agent_proposition: P,
) -> bool {
    match &condition.condition_type {
        ConditionType::IsPresent(claim) => exists(claim).evaluate(identity_claims),
        ConditionType::IsAbsent(claim) => not::<_>(exists(claim)).evaluate(identity_claims),
        ConditionType::IsAnyOf(claims) => any(claims).evaluate(identity_claims),
        ConditionType::IsNoneOf(claims) => not::<_>(any(claims)).evaluate(identity_claims),
        ConditionType::IsIdentity(TargetIdentity::Specific(id)) => {
            IsIdentityProposition { identity: *id }.evaluate(identity_claims)
        }
        ConditionType::IsIdentity(TargetIdentity::ExternalAgent) => {
            external_agent_proposition.evaluate(identity_claims)
        }
    }
}
