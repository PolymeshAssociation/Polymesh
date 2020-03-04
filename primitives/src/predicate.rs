use crate::{IdentityClaimData, IdentityId};

use codec::{Decode, Encode};
use sp_std::prelude::*;

#[derive(Encode, Decode, Clone, Debug, Default)]
pub struct FilterClaim {
    pub claim: IdentityClaimData,
    pub trusted_issuers: Vec<IdentityId>,
}

impl From<IdentityClaimData> for FilterClaim {
    fn from(claim: IdentityClaimData) -> FilterClaim {
        FilterClaim {
            claim,
            ..Default::default()
        }
    }
}

/// Context using during an `Predicate` evaluation.
///
#[derive(Encode, Decode, Clone, Debug, Default)]
pub struct PredicateContext {
    /// Target identity used to fetch its claims during evaluation.
    pub target_identity: IdentityId,

    #[cfg(test)]
    pub claims: Vec<IdentityClaimData>,
}

impl PredicateContext {
    #[cfg(test)]
    pub fn fetch_claims(&self, filter: &FilterClaim) -> Vec<IdentityClaimData> {
        self.claims.clone()
    }

    /// It fetchs all claims filtered by this context.
    #[cfg(not(test))]
    pub fn fetch_claims(&self, filter: &FilterClaim) -> Vec<IdentityClaimData> {
        unimplemented!()
    }
}

#[cfg(test)]
impl From<Vec<IdentityClaimData>> for PredicateContext {
    fn from(claims: Vec<IdentityClaimData>) -> Self {
        PredicateContext {
            claims,
            ..Default::default()
        }
    }
}

// Predicate Trait
// ==================================

/// It allows composition and evaluation of claims based on a context.
pub trait Predicate {
    /// It evaluates this predicated based on `context` context.
    fn evaluate(&self, context: &PredicateContext) -> bool;

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

#[inline]
pub fn exists(claim: IdentityClaimData) -> ExistentialPredicate {
    ExistentialPredicate { claim }
}

#[inline]
pub fn any(claims: Vec<IdentityClaimData>) -> AnyPredicate {
    AnyPredicate { claims }
}

#[inline]
pub fn not<P>(predicate: P) -> NotPredicate<P>
where
    P: Predicate + Sized,
{
    NotPredicate::new(predicate)
}

// ExistentialPredicate
// ======================================================

#[derive(Encode, Decode, Clone, Debug)]
pub struct ExistentialPredicate {
    pub claim: IdentityClaimData,
}

impl Predicate for ExistentialPredicate {
    #[inline]
    fn evaluate(&self, context: &PredicateContext) -> bool {
        let filtered_claims = context.fetch_claims();

        filtered_claims
            .into_iter()
            .any(|ctx_claim| ctx_claim == self.claim)
    }
}

// AndPredicate
// ======================================================

#[derive(Encode, Decode, Clone, Debug)]
pub struct AndPredicate<P1, P2>
where
    P1: Predicate,
    P2: Predicate,
{
    lhs: P1,
    rhs: P2,
}

impl<P1, P2> AndPredicate<P1, P2>
where
    P1: Predicate,
    P2: Predicate,
{
    #[inline]
    pub fn new(lhs: P1, rhs: P2) -> Self {
        AndPredicate { lhs, rhs }
    }
}

impl<P1, P2> Predicate for AndPredicate<P1, P2>
where
    P1: Predicate,
    P2: Predicate,
{
    #[inline]
    fn evaluate(&self, context: &PredicateContext) -> bool {
        self.lhs.evaluate(context) && self.rhs.evaluate(context)
    }
}

// OrPredicate
// ======================================================

#[derive(Encode, Decode, Clone, Debug)]
pub struct OrPredicate<P1, P2>
where
    P1: Predicate,
    P2: Predicate,
{
    lhs: P1,
    rhs: P2,
}

impl<P1, P2> OrPredicate<P1, P2>
where
    P1: Predicate,
    P2: Predicate,
{
    #[inline]
    pub fn new(lhs: P1, rhs: P2) -> Self {
        OrPredicate { lhs, rhs }
    }
}

impl<P1, P2> Predicate for OrPredicate<P1, P2>
where
    P1: Predicate,
    P2: Predicate,
{
    #[inline]
    fn evaluate(&self, context: &PredicateContext) -> bool {
        self.lhs.evaluate(context) || self.rhs.evaluate(context)
    }
}

// NotPredicate
// ======================================================

#[derive(Encode, Decode, Clone, Debug)]
pub struct NotPredicate<P: Predicate + Sized> {
    predicate: P,
}

impl<P> NotPredicate<P>
where
    P: Predicate + Sized,
{
    #[inline]
    pub fn new(predicate: P) -> Self {
        NotPredicate { predicate }
    }
}

impl<P: Predicate + Sized> Predicate for NotPredicate<P> {
    #[inline]
    fn evaluate(&self, context: &PredicateContext) -> bool {
        !self.predicate.evaluate(context)
    }
}

// AnyPredicate
// =========================================================
#[derive(Encode, Decode, Clone, Debug)]
pub struct AnyPredicate {
    pub claims: Vec<IdentityClaimData>,
}

impl Predicate for AnyPredicate {
    fn evaluate(&self, context: &PredicateContext) -> bool {
        let filtered_claims = context.fetch_claims();

        filtered_claims.iter().any(|ctx_claim| {
            self.claims
                .iter()
                .any(|valid_claim| ctx_claim == valid_claim)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        predicate::{self, Predicate, PredicateContext},
        IdentityClaimData, JurisdictionName,
    };

    use std::convert::From;

    #[test]
    fn existential_operators_test() {
        let id_claim_1 = IdentityClaimData::CustomerDueDiligence;
        let id_claim_2 = IdentityClaimData::Affiliate;

        let context = PredicateContext::from(vec![id_claim_1, id_claim_2]);
        // Affiliate && CustommerDueDiligenge
        let affiliate_and_cdd_pred = predicate::exists(IdentityClaimData::Affiliate)
            .and(predicate::exists(IdentityClaimData::CustomerDueDiligence));

        assert_eq!(affiliate_and_cdd_pred.evaluate(&context), true);
    }

    #[test]
    fn collection_operators_test() {
        // 1. Check jurisdiction "CAN" belongs to {ESP, CAN, IND}
        let can_jurisdiction_claim =
            IdentityClaimData::Jurisdiction(JurisdictionName::from(b"Canada"));

        let valid_jurisdictions = vec![
            IdentityClaimData::Jurisdiction(JurisdictionName::from(b"Spain")),
            IdentityClaimData::Jurisdiction(JurisdictionName::from(b"Canada")),
            IdentityClaimData::Jurisdiction(JurisdictionName::from(b"India")),
        ];

        let context = PredicateContext::from(vec![can_jurisdiction_claim]);
        let in_juridisction_pre = predicate::any(valid_jurisdictions);
        assert_eq!(in_juridisction_pre.evaluate(&context), true);

        // 2. Check USA does not belong to {ESP, CAN, IND}.
        let usa_jurisdiction_claim =
            IdentityClaimData::Jurisdiction(JurisdictionName::from(b"USA"));
        let context = PredicateContext::from(vec![usa_jurisdiction_claim]);
        assert_eq!(in_juridisction_pre.evaluate(&context), false);

        // 3. Check NOT in jurisdiction.
        let not_in_jurisdiction_pre = predicate::not(in_juridisction_pre);
        assert_eq!(not_in_jurisdiction_pre.evaluate(&context), true);
    }
}
