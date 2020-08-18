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

use crate::{Claim, IdentityId, Rule, RuleType, TargetIdentity, Ticker};
use codec::{Decode, Encode};

use sp_std::prelude::*;

/// Context using during an `Predicate` evaluation.
///
/// # TODO
///  - Use a lazy access to claims. It could be part of the optimization
///  process of CDD claims.
#[derive(Encode, Decode, Clone, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Context {
    /// Predicate evaluation will use those claims.
    pub claims: Vec<Claim>,
    /// Identity of this context.
    /// It could be the sender DID during the evaluation of sender's rules or
    /// the receiver DID on a receiver's rule evaluation.
    pub id: IdentityId,
    /// Identity of the primary issuance agent of the token
    pub primary_issuance_agent: Option<IdentityId>,
}

// Predicate Trait
// ==================================

/// It allows composition and evaluation of claims based on a context.
pub trait Predicate {
    /// It evaluates this predicated based on `context` context.
    fn evaluate(&self, context: &Context) -> bool;

    /// It generates a new predicate that represents the logical AND
    /// of two predicates: `Self` and `other`.
    #[inline]
    fn and<B>(self, other: B) -> AndPredicate<Self, B>
    where
        Self: Sized,
        B: Predicate + Sized,
    {
        AndPredicate::new(self, other)
    }

    /// It generates a new predicate that represents the logical OR
    /// of two predicates: `Self` and `other`.
    #[inline]
    fn or<B>(self, other: B) -> OrPredicate<Self, B>
    where
        Self: Sized,
        B: Predicate + Sized,
    {
        OrPredicate::new(self, other)
    }

    /// It generates a new predicate that represents the logical NOT
    /// of this predicate.
    #[inline]
    fn not(self) -> NotPredicate<Self>
    where
        Self: Sized,
    {
        NotPredicate::new(self)
    }
}

/// Base and simple predicates
pub mod base;
pub use base::{
    AndPredicate, AnyPredicate, ExistentialPredicate, NotPredicate, OrPredicate,
    TargetIdentityPredicate,
};

/// Predicates for confidential stuff.
pub mod valid_proof_of_investor;
pub use valid_proof_of_investor::ValidProofOfInvestorPredicate;

// Helper functions
// ======================================

/// It creates a predicate to evaluate the matching of `id` with primary issuance agent in the context.
#[inline]
pub fn equals<'a>(
    id: &'a TargetIdentity,
    primary_issuance_agent: &'a IdentityId,
) -> TargetIdentityPredicate<'a> {
    match id {
        TargetIdentity::PrimaryIssuanceAgent => TargetIdentityPredicate {
            identity: primary_issuance_agent,
        },
        TargetIdentity::Specific(identity) => TargetIdentityPredicate { identity },
    }
}

/// It creates a predicate to evaluate the existential of `claim` in the context.
#[inline]
pub fn exists(claim: &'_ Claim) -> ExistentialPredicate<'_> {
    ExistentialPredicate { claim }
}

/// It creates a predicate to evaluate if any of `claims` are found in the context.
#[inline]
pub fn any(claims: &'_ [Claim]) -> AnyPredicate<'_> {
    AnyPredicate { claims }
}

/// It create a negate predicate of `predicate`.
#[inline]
pub fn not<P>(predicate: P) -> NotPredicate<P>
where
    P: Predicate + Sized,
{
    NotPredicate::new(predicate)
}

/// It verifies if the Identifier in the context has got a valid `InvestorZKProof` and its
/// associate `CustomDueDiligence`.
#[inline]
pub fn has_valid_proof_of_investor(ticker: Ticker) -> ValidProofOfInvestorPredicate {
    ValidProofOfInvestorPredicate { ticker }
}

/// Helper function to run predicates from a context.
pub fn run(rule: &Rule, context: &Context) -> bool {
    match rule.rule_type {
        RuleType::IsPresent(ref claim) => exists(claim).evaluate(context),
        RuleType::IsAbsent(ref claim) => not(exists(claim)).evaluate(context),
        RuleType::IsAnyOf(ref claims) => any(claims).evaluate(context),
        RuleType::IsNoneOf(ref claims) => not(any(claims)).evaluate(context),
        RuleType::HasValidProofOfInvestor(ref ticker) => {
            has_valid_proof_of_investor(ticker.clone()).evaluate(context)
        }
        RuleType::IsIdentity(ref id) => {
            equals(id, &context.primary_issuance_agent.unwrap_or_default()).evaluate(context)
        }
    }
}
