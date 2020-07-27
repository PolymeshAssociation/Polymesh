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

use crate::{Claim, Rule, RuleType};
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
}

impl From<Vec<Claim>> for Context {
    fn from(claims: Vec<Claim>) -> Self {
        Context { claims }
    }
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

// Helper functions
// ======================================

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

/// Helper function to run predicates from a context.
pub fn run(rule: &Rule, context: &Context) -> bool {
    match rule.rule_type {
        RuleType::IsPresent(ref claim) => exists(claim).evaluate(context),
        RuleType::IsAbsent(ref claim) => not(exists(claim)).evaluate(context),
        RuleType::IsAnyOf(ref claims) => any(claims).evaluate(context),
        RuleType::IsNoneOf(ref claims) => not(any(claims)).evaluate(context),
    }
}

// ExistentialPredicate
// ======================================================

/// It checks the existential of a claim.
///
/// # `CustomerDueDiligence` wildcard search
/// The `CustomerDueDiligence` claim supports wildcard search if you use the default `CddId` (a zero filled data).
/// For instance:
///     - The `exists(Claim::CustomerDueDiligence(CddId::default()))` matches with any CDD claim.
///     - The `exists(Claim::CustomerDueDiligence(a_valid_cdd_id))` matches only for the given
///     `a_valid_cdd_id`.
///
#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExistentialPredicate<'a> {
    /// Claims we want to check if it exists in context.
    pub claim: &'a Claim,
}

impl<'a> Predicate for ExistentialPredicate<'a> {
    fn evaluate(&self, context: &Context) -> bool {
        match &self.claim {
            Claim::CustomerDueDiligence(ref cdd_id) if cdd_id.is_wildcard() => {
                self.evaluate_cdd_claim_wildcard(context)
            }
            _ => self.evaluate_regular_claim(context),
        }
    }
}

impl<'a> ExistentialPredicate<'a> {
    /// The wildcard search only double-checks if any CDD claim is in the context.
    fn evaluate_cdd_claim_wildcard(&self, context: &Context) -> bool {
        context.claims.iter().any(|ctx_claim| match ctx_claim {
            Claim::CustomerDueDiligence(..) => true,
            _ => false,
        })
    }

    /// In regular claim evaluation, the data of the claim has to match too.
    fn evaluate_regular_claim(&self, context: &Context) -> bool {
        context
            .claims
            .iter()
            .any(|ctx_claim| ctx_claim == self.claim)
    }
}

// AndPredicate
// ======================================================

/// A composition predicate of two others using logical AND operator.
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AndPredicate<P1, P2>
where
    P1: Predicate + Sized,
    P2: Predicate + Sized,
{
    lhs: P1,
    rhs: P2,
}

impl<P1, P2> AndPredicate<P1, P2>
where
    P1: Predicate + Sized,
    P2: Predicate + Sized,
{
    /// Create a new `AndPredicate` over predicates `lhs` and `rhs`.
    #[inline]
    pub fn new(lhs: P1, rhs: P2) -> Self {
        AndPredicate { lhs, rhs }
    }
}

impl<P1, P2> Predicate for AndPredicate<P1, P2>
where
    P1: Predicate + Sized,
    P2: Predicate + Sized,
{
    /// Evaluate predicate against `context`.
    #[inline]
    fn evaluate(&self, context: &Context) -> bool {
        self.lhs.evaluate(context) && self.rhs.evaluate(context)
    }
}

// OrPredicate
// ======================================================

/// A composition predicate of two others using logical OR operator.
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OrPredicate<P1, P2>
where
    P1: Predicate + Sized,
    P2: Predicate + Sized,
{
    lhs: P1,
    rhs: P2,
}

impl<P1, P2> OrPredicate<P1, P2>
where
    P1: Predicate + Sized,
    P2: Predicate + Sized,
{
    /// Create a new `OrPredicate` over predicates `lhs` and `rhs`.
    #[inline]
    pub fn new(lhs: P1, rhs: P2) -> Self {
        OrPredicate { lhs, rhs }
    }
}

impl<P1, P2> Predicate for OrPredicate<P1, P2>
where
    P1: Predicate + Sized,
    P2: Predicate + Sized,
{
    /// Evaluate predicate against `context`.
    #[inline]
    fn evaluate(&self, context: &Context) -> bool {
        self.lhs.evaluate(context) || self.rhs.evaluate(context)
    }
}

// NotPredicate
// ======================================================

/// Predicate that returns a logical NOT of other predicate.
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct NotPredicate<P: Predicate + Sized> {
    predicate: P,
}

impl<P> NotPredicate<P>
where
    P: Predicate + Sized,
{
    /// Create a new `OrPredicate` over predicate `predicate`.
    #[inline]
    pub fn new(predicate: P) -> Self {
        NotPredicate { predicate }
    }
}

impl<P: Predicate + Sized> Predicate for NotPredicate<P> {
    /// Evaluate predicate against `context`.
    #[inline]
    fn evaluate(&self, context: &Context) -> bool {
        !self.predicate.evaluate(context)
    }
}

// AnyPredicate
// =========================================================

/// Predicate that checks if any of its internal claims exists in context.
#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AnyPredicate<'a> {
    /// List of claims to find in context.
    pub claims: &'a [Claim],
}

impl<'a> Predicate for AnyPredicate<'a> {
    /// Evaluate predicate against `context`.
    fn evaluate(&self, context: &Context) -> bool {
        context.claims.iter().any(|ctx_claim| {
            self.claims
                .iter()
                .any(|valid_claim| ctx_claim == valid_claim)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        predicate::{self, Context, Predicate},
        CddId, Claim, IdentityId, InvestorUid, Rule, RuleType, Scope,
    };
    use std::convert::From;

    #[test]
    fn existential_operators_test() {
        let scope = Scope::from(0);
        let cdd_claim = Claim::CustomerDueDiligence(CddId::new(
            IdentityId::from(1),
            InvestorUid::from(b"UID1".as_ref()),
        ));
        let context = Context::from(vec![cdd_claim.clone(), Claim::Affiliate(scope)]);

        // Affiliate && CustommerDueDiligenge
        let affiliate_claim = Claim::Affiliate(scope);
        let affiliate_and_cdd_pred =
            predicate::exists(&affiliate_claim).and(predicate::exists(&cdd_claim));

        assert_eq!(affiliate_and_cdd_pred.evaluate(&context), true);
    }

    #[test]
    fn collection_operators_test() {
        let scope = Scope::from(0);

        // 1. Check jurisdiction "CAN" belongs to {ESP, CAN, IND}
        let valid_jurisdictions = vec![
            Claim::Jurisdiction(b"Spain".into(), scope),
            Claim::Jurisdiction(b"Canada".into(), scope),
            Claim::Jurisdiction(b"India".into(), scope),
        ];

        let context = Context::from(vec![Claim::Jurisdiction(b"Canada".into(), scope)]);
        let in_juridisction_pre = predicate::any(&valid_jurisdictions);
        assert_eq!(in_juridisction_pre.evaluate(&context), true);

        // 2. Check USA does not belong to {ESP, CAN, IND}.
        let context = Context::from(vec![Claim::Jurisdiction(b"USA".into(), scope)]);
        assert_eq!(in_juridisction_pre.evaluate(&context), false);

        // 3. Check NOT in jurisdiction.
        let not_in_jurisdiction_pre = predicate::not(in_juridisction_pre.clone());
        assert_eq!(not_in_jurisdiction_pre.evaluate(&context), true);
    }

    #[test]
    fn run_predicate() {
        let scope = Scope::from(0);

        let rules: Vec<Rule> = vec![
            RuleType::IsPresent(Claim::Accredited(scope)).into(),
            RuleType::IsAbsent(Claim::BuyLockup(scope)).into(),
            RuleType::IsAnyOf(vec![
                Claim::Jurisdiction(b"USA".into(), scope),
                Claim::Jurisdiction(b"Canada".into(), scope),
            ])
            .into(),
            RuleType::IsNoneOf(vec![Claim::Jurisdiction(b"Cuba".into(), scope)]).into(),
        ];

        // Valid case
        let context: Context = vec![
            Claim::Accredited(scope),
            Claim::Jurisdiction(b"Canada".into(), scope),
        ]
        .into();

        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, true);

        // Invalid case: `BuyLockup` is present.
        let context: Context = vec![
            Claim::Accredited(scope),
            Claim::BuyLockup(scope),
            Claim::Jurisdiction(b"Canada".into(), scope),
        ]
        .into();

        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, false);

        // Invalid case: Missing `Accredited`
        let context: Context = vec![
            Claim::BuyLockup(scope),
            Claim::Jurisdiction(b"Canada".into(), scope),
        ]
        .into();

        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, false);

        // Invalid case: Missing `Jurisdiction`
        let context: Context = vec![
            Claim::Accredited(scope),
            Claim::Jurisdiction(b"Spain".into(), scope),
        ]
        .into();

        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, false);

        // Check NoneOf
        let context: Context = vec![
            Claim::Accredited(scope),
            Claim::Jurisdiction(b"Cuba".into(), scope),
        ]
        .into();
        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, false);
    }
}
