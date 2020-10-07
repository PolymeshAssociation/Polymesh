use crate::{
    proposition::{Context, Proposition},
    Claim, IdentityId,
};
use codec::{Decode, Encode};

// TargetIdentityProposition
// ======================================================

/// It matches `id` with primary issuance agent in the context.
#[derive(Clone, Debug)]
pub struct TargetIdentityProposition<'a> {
    /// IdentityId we want to check.
    pub identity: &'a IdentityId,
}

impl<C> Proposition<C> for TargetIdentityProposition<'_> {
    #[inline]
    fn evaluate(&self, context: Context<C>) -> bool {
        context.id == *self.identity
    }
}

// ExistentialProposition
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
pub struct ExistentialProposition<'a> {
    /// Claims we want to check if it exists in context.
    pub claim: &'a Claim,
}

impl<C: Iterator<Item = Claim>> Proposition<C> for ExistentialProposition<'_> {
    fn evaluate(&self, mut context: Context<C>) -> bool {
        match &self.claim {
            // The wildcard search only double-checks if any CDD claim is in the context.
            Claim::CustomerDueDiligence(cdd_id) if cdd_id.is_wildcard() => context
                .claims
                .any(|ctx_claim| matches!(ctx_claim, Claim::CustomerDueDiligence(..))),
            // In regular claim evaluation, the data of the claim has to match too.
            _ => context.claims.any(|ctx_claim| &ctx_claim == self.claim),
        }
    }
}

// AndProposition
// ======================================================

/// A composition proposition of two others using logical AND operator.
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AndProposition<P1, P2> {
    lhs: P1,
    rhs: P2,
}

impl<P1, P2> AndProposition<P1, P2> {
    /// Create a new `AndProposition` over propositions `lhs` and `rhs`.
    #[inline]
    pub fn new(lhs: P1, rhs: P2) -> Self {
        Self { lhs, rhs }
    }
}

impl<P1, P2, C: Clone> Proposition<C> for AndProposition<P1, P2>
where
    P1: Proposition<C>,
    P2: Proposition<C>,
{
    /// Evaluate proposition against `context`.
    #[inline]
    fn evaluate(&self, context: Context<C>) -> bool {
        self.lhs.evaluate(context.clone()) && self.rhs.evaluate(context)
    }
}

// OrProposition
// ======================================================

/// A composition proposition of two others using logical OR operator.
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OrProposition<P1, P2> {
    lhs: P1,
    rhs: P2,
}

impl<P1, P2> OrProposition<P1, P2> {
    /// Create a new `OrProposition` over propositions `lhs` and `rhs`.
    #[inline]
    pub fn new(lhs: P1, rhs: P2) -> Self {
        Self { lhs, rhs }
    }
}

impl<P1, P2, C: Clone> Proposition<C> for OrProposition<P1, P2>
where
    P1: Proposition<C>,
    P2: Proposition<C>,
{
    /// Evaluate proposition against `context`.
    #[inline]
    fn evaluate(&self, context: Context<C>) -> bool {
        self.lhs.evaluate(context.clone()) || self.rhs.evaluate(context)
    }
}

// NotProposition
// ======================================================

/// proposition that returns a logical NOT of other proposition.
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct NotProposition<P> {
    proposition: P,
}

impl<P> NotProposition<P> {
    /// Create a new `OrProposition` over proposition `proposition`.
    #[inline]
    pub fn new(proposition: P) -> Self {
        Self { proposition }
    }
}

impl<P: Proposition<C>, C> Proposition<C> for NotProposition<P> {
    /// Evaluate proposition against `context`.
    #[inline]
    fn evaluate(&self, context: Context<C>) -> bool {
        !self.proposition.evaluate(context)
    }
}

// AnyProposition
// =========================================================

/// Proposition that checks if any of its internal claims exists in context.
#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AnyProposition<'a> {
    /// List of claims to find in context.
    pub claims: &'a [Claim],
}

impl<C: Iterator<Item = Claim>> Proposition<C> for AnyProposition<'_> {
    /// Evaluate proposition against `context`.
    fn evaluate(&self, mut context: Context<C>) -> bool {
        context.claims.any(|ctx_claim| {
            self.claims
                .iter()
                .any(|valid_claim| &ctx_claim == valid_claim)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        proposition::{self, Context, Proposition},
        CddId, Claim, Condition, ConditionType, CountryCode, IdentityId, InvestorUid, Scope,
        TargetIdentity,
    };
    use std::convert::From;

    #[test]
    fn existential_operators_test() {
        let scope = Scope::Identity(IdentityId::from(0));
        let did = IdentityId::from(1);
        let cdd_claim =
            Claim::CustomerDueDiligence(CddId::new(did, InvestorUid::from(b"UID1".as_ref())));
        let context = Context {
            claims: vec![cdd_claim.clone(), Claim::Affiliate(scope.clone())],
            id: did,
            ..Default::default()
        };

        // Affiliate && CustommerDueDiligenge
        let affiliate_claim = Claim::Affiliate(scope);
        let affiliate_and_cdd_pred =
            proposition::exists(&affiliate_claim).and(proposition::exists(&cdd_claim));

        assert_eq!(affiliate_and_cdd_pred.evaluate(&context), true);
    }

    #[test]
    fn collection_operators_test() {
        let scope = Scope::Identity(IdentityId::from(0));

        // 1. Check jurisdiction "CAN" belongs to {ESP, CAN, IND}
        let valid_jurisdictions = vec![
            Claim::Jurisdiction(CountryCode::ES, scope.clone()),
            Claim::Jurisdiction(CountryCode::CA, scope.clone()),
            Claim::Jurisdiction(CountryCode::IN, scope.clone()),
        ];

        let context = Context {
            claims: vec![Claim::Jurisdiction(CountryCode::CA, scope.clone())],
            ..Default::default()
        };
        let in_juridisction_pre = proposition::any(&valid_jurisdictions);
        assert_eq!(in_juridisction_pre.evaluate(&context), true);

        // 2. Check USA does not belong to {ESP, CAN, IND}.
        let context = Context {
            claims: vec![Claim::Jurisdiction(CountryCode::US, scope)],
            ..Default::default()
        };
        assert_eq!(in_juridisction_pre.evaluate(&context), false);

        // 3. Check NOT in jurisdiction.
        let not_in_jurisdiction_pre = proposition::not(in_juridisction_pre.clone());
        assert_eq!(not_in_jurisdiction_pre.evaluate(&context), true);
    }

    #[test]
    fn run_proposition() {
        let scope = Scope::Identity(IdentityId::from(0));

        let conditions: Vec<Condition> = vec![
            ConditionType::IsPresent(Claim::Accredited(scope.clone())).into(),
            ConditionType::IsAbsent(Claim::BuyLockup(scope.clone())).into(),
            ConditionType::IsAnyOf(vec![
                Claim::Jurisdiction(CountryCode::US, scope.clone()),
                Claim::Jurisdiction(CountryCode::CA, scope.clone()),
            ])
            .into(),
            ConditionType::IsNoneOf(vec![Claim::Jurisdiction(CountryCode::CU, scope.clone())])
                .into(),
        ];

        // Valid case
        let context = Context {
            claims: vec![
                Claim::Accredited(scope.clone()),
                Claim::Jurisdiction(CountryCode::CA, scope.clone()),
            ],
            ..Default::default()
        };

        let out = !conditions
            .iter()
            .any(|condition| !proposition::run(&condition, &context));
        assert_eq!(out, true);

        // Invalid case: `BuyLockup` is present.
        let context = Context {
            claims: vec![
                Claim::Accredited(scope.clone()),
                Claim::BuyLockup(scope.clone()),
                Claim::Jurisdiction(CountryCode::CA, scope.clone()),
            ],
            ..Default::default()
        };

        let out = !conditions
            .iter()
            .any(|condition| !proposition::run(&condition, &context));
        assert_eq!(out, false);

        // Invalid case: Missing `Accredited`
        let context = Context {
            claims: vec![
                Claim::BuyLockup(scope.clone()),
                Claim::Jurisdiction(CountryCode::CA, scope.clone()),
            ],
            ..Default::default()
        };

        let out = !conditions
            .iter()
            .any(|condition| !proposition::run(&condition, &context));
        assert_eq!(out, false);

        // Invalid case: Missing `Jurisdiction`
        let context = Context {
            claims: vec![
                Claim::Accredited(scope.clone()),
                Claim::Jurisdiction(CountryCode::ES, scope.clone()),
            ],
            ..Default::default()
        };

        let out = !conditions
            .iter()
            .any(|condition| !proposition::run(&condition, &context));
        assert_eq!(out, false);

        // Check NoneOf
        let context = Context {
            claims: vec![
                Claim::Accredited(scope.clone()),
                Claim::Jurisdiction(CountryCode::CU, scope.clone()),
            ],
            ..Default::default()
        };

        let out = !conditions
            .iter()
            .any(|condition| !proposition::run(&condition, &context));
        assert_eq!(out, false);

        let identity1 = IdentityId::from(1);
        let identity2 = IdentityId::from(2);
        assert!(proposition::run(
            &ConditionType::IsIdentity(TargetIdentity::PrimaryIssuanceAgent).into(),
            &Context {
                id: identity1,
                primary_issuance_agent: Some(identity1),
                ..Default::default()
            }
        ));
        assert!(proposition::run(
            &ConditionType::IsIdentity(TargetIdentity::Specific(identity1)).into(),
            &Context {
                id: identity1,
                primary_issuance_agent: Some(identity2),
                ..Default::default()
            }
        ));
    }
}
