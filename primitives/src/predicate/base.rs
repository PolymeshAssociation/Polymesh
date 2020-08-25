use crate::{
    predicate::{Context, Predicate},
    Claim, IdentityId,
};
use codec::{Decode, Encode};

// TargetIdentityPredicate
// ======================================================

/// It matches `id` with primary issuance agent in the context.
#[derive(Clone, Debug)]
pub struct TargetIdentityPredicate<'a> {
    /// IdentityId we want to check.
    pub identity: &'a IdentityId,
}

impl<'a> Predicate for TargetIdentityPredicate<'a> {
    #[inline]
    fn evaluate(&self, context: &Context) -> bool {
        context.id == *self.identity
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
        CddId, Claim, CountryCode, IdentityId, InvestorUid, Rule, RuleType, Scope, TargetIdentity,
    };
    use std::convert::From;

    #[test]
    fn existential_operators_test() {
        let scope = Scope::from(0);
        let did = IdentityId::from(1);
        let cdd_claim =
            Claim::CustomerDueDiligence(CddId::new(did, InvestorUid::from(b"UID1".as_ref())));
        let context = Context {
            claims: vec![cdd_claim.clone(), Claim::Affiliate(scope)],
            id: did,
            ..Default::default()
        };

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
            Claim::Jurisdiction(CountryCode::ES, scope),
            Claim::Jurisdiction(CountryCode::CA, scope),
            Claim::Jurisdiction(CountryCode::IN, scope),
        ];

        let context = Context {
            claims: vec![Claim::Jurisdiction(CountryCode::CA, scope)],
            ..Default::default()
        };
        let in_juridisction_pre = predicate::any(&valid_jurisdictions);
        assert_eq!(in_juridisction_pre.evaluate(&context), true);

        // 2. Check USA does not belong to {ESP, CAN, IND}.
        let context = Context {
            claims: vec![Claim::Jurisdiction(CountryCode::US, scope)],
            ..Default::default()
        };
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
                Claim::Jurisdiction(CountryCode::US, scope),
                Claim::Jurisdiction(CountryCode::CA, scope),
            ])
            .into(),
            RuleType::IsNoneOf(vec![Claim::Jurisdiction(CountryCode::CU, scope)]).into(),
        ];

        // Valid case
        let context = Context {
            claims: vec![
                Claim::Accredited(scope),
                Claim::Jurisdiction(CountryCode::CA, scope),
            ],
            ..Default::default()
        };

        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, true);

        // Invalid case: `BuyLockup` is present.
        let context = Context {
            claims: vec![
                Claim::Accredited(scope),
                Claim::BuyLockup(scope),
                Claim::Jurisdiction(CountryCode::CA, scope),
            ],
            ..Default::default()
        };

        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, false);

        // Invalid case: Missing `Accredited`
        let context = Context {
            claims: vec![
                Claim::BuyLockup(scope),
                Claim::Jurisdiction(CountryCode::CA, scope),
            ],
            ..Default::default()
        };

        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, false);

        // Invalid case: Missing `Jurisdiction`
        let context = Context {
            claims: vec![
                Claim::Accredited(scope),
                Claim::Jurisdiction(CountryCode::ES, scope),
            ],
            ..Default::default()
        };

        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, false);

        // Check NoneOf
        let context = Context {
            claims: vec![
                Claim::Accredited(scope),
                Claim::Jurisdiction(CountryCode::CU, scope),
            ],
            ..Default::default()
        };

        let out = !rules.iter().any(|rule| !predicate::run(&rule, &context));
        assert_eq!(out, false);

        let identity1 = IdentityId::from(1);
        let identity2 = IdentityId::from(2);
        assert!(predicate::run(
            &RuleType::IsIdentity(TargetIdentity::PrimaryIssuanceAgent).into(),
            &Context {
                id: identity1,
                primary_issuance_agent: Some(identity1),
                ..Default::default()
            }
        ));
        assert!(predicate::run(
            &RuleType::IsIdentity(TargetIdentity::Specific(identity1)).into(),
            &Context {
                id: identity1,
                primary_issuance_agent: Some(identity2),
                ..Default::default()
            }
        ));
    }
}
