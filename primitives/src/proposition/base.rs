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

impl<'a> Proposition for TargetIdentityProposition<'a> {
    #[inline]
    fn evaluate(&self, context: &Context) -> bool {
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

impl<'a> Proposition for ExistentialProposition<'a> {
    fn evaluate(&self, context: &Context) -> bool {
        match &self.claim {
            Claim::CustomerDueDiligence(ref cdd_id) if cdd_id.is_wildcard() => {
                self.evaluate_cdd_claim_wildcard(context)
            }
            _ => self.evaluate_regular_claim(context),
        }
    }
}

impl<'a> ExistentialProposition<'a> {
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

// AndProposition
// ======================================================

/// A composition proposition of two others using logical AND operator.
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AndProposition<P1, P2>
where
    P1: Proposition + Sized,
    P2: Proposition + Sized,
{
    lhs: P1,
    rhs: P2,
}

impl<P1, P2> AndProposition<P1, P2>
where
    P1: Proposition + Sized,
    P2: Proposition + Sized,
{
    /// Create a new `AndProposition` over propositions `lhs` and `rhs`.
    #[inline]
    pub fn new(lhs: P1, rhs: P2) -> Self {
        AndProposition { lhs, rhs }
    }
}

impl<P1, P2> Proposition for AndProposition<P1, P2>
where
    P1: Proposition + Sized,
    P2: Proposition + Sized,
{
    /// Evaluate proposition against `context`.
    #[inline]
    fn evaluate(&self, context: &Context) -> bool {
        self.lhs.evaluate(context) && self.rhs.evaluate(context)
    }
}

// OrProposition
// ======================================================

/// A composition proposition of two others using logical OR operator.
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OrProposition<P1, P2>
where
    P1: Proposition + Sized,
    P2: Proposition + Sized,
{
    lhs: P1,
    rhs: P2,
}

impl<P1, P2> OrProposition<P1, P2>
where
    P1: Proposition + Sized,
    P2: Proposition + Sized,
{
    /// Create a new `OrProposition` over propositions `lhs` and `rhs`.
    #[inline]
    pub fn new(lhs: P1, rhs: P2) -> Self {
        OrProposition { lhs, rhs }
    }
}

impl<P1, P2> Proposition for OrProposition<P1, P2>
where
    P1: Proposition + Sized,
    P2: Proposition + Sized,
{
    /// Evaluate proposition against `context`.
    #[inline]
    fn evaluate(&self, context: &Context) -> bool {
        self.lhs.evaluate(context) || self.rhs.evaluate(context)
    }
}

// NotProposition
// ======================================================

/// proposition that returns a logical NOT of other proposition.
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct NotProposition<P: Proposition + Sized> {
    proposition: P,
}

impl<P> NotProposition<P>
where
    P: Proposition + Sized,
{
    /// Create a new `OrProposition` over proposition `proposition`.
    #[inline]
    pub fn new(proposition: P) -> Self {
        NotProposition { proposition }
    }
}

impl<P: Proposition + Sized> Proposition for NotProposition<P> {
    /// Evaluate proposition against `context`.
    #[inline]
    fn evaluate(&self, context: &Context) -> bool {
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

impl<'a> Proposition for AnyProposition<'a> {
    /// Evaluate proposition against `context`.
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
        proposition::{self, Context, Proposition},
        CddId, Claim, Condition, ConditionType, CountryCode, IdentityId, InvestorUid, Scope, TargetIdentity,
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
            proposition::exists(&affiliate_claim).and(proposition::exists(&cdd_claim));

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
        let scope = Scope::from(0);

        let conditions: Vec<Condition> = vec![
            ConditionType::IsPresent(Claim::Accredited(scope)).into(),
            ConditionType::IsAbsent(Claim::BuyLockup(scope)).into(),
            ConditionType::IsAnyOf(vec![
                Claim::Jurisdiction(CountryCode::US, scope),
                Claim::Jurisdiction(CountryCode::CA, scope),
            ])
            .into(),
            ConditionType::IsNoneOf(vec![Claim::Jurisdiction(CountryCode::CU, scope)]).into(),
        ];

        // Valid case
        let context = Context {
            claims: vec![
                Claim::Accredited(scope),
                Claim::Jurisdiction(CountryCode::CA, scope),
            ],
            ..Default::default()
        };

        let out = !conditions.iter().any(|condition| !proposition::run(&condition, &context));
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

        let out = !conditions.iter().any(|condition| !proposition::run(&condition, &context));
        assert_eq!(out, false);

        // Invalid case: Missing `Accredited`
        let context = Context {
            claims: vec![
                Claim::BuyLockup(scope),
                Claim::Jurisdiction(CountryCode::CA, scope),
            ],
            ..Default::default()
        };

        let out = !conditions.iter().any(|condition| !proposition::run(&condition, &context));
        assert_eq!(out, false);

        // Invalid case: Missing `Jurisdiction`
        let context = Context {
            claims: vec![
                Claim::Accredited(scope),
                Claim::Jurisdiction(CountryCode::ES, scope),
            ],
            ..Default::default()
        };

        let out = !conditions.iter().any(|condition| !proposition::run(&condition, &context));
        assert_eq!(out, false);

        // Check NoneOf
        let context = Context {
            claims: vec![
                Claim::Accredited(scope),
                Claim::Jurisdiction(CountryCode::CU, scope),
            ],
            ..Default::default()
        };

        let out = !conditions.iter().any(|condition| !proposition::run(&condition, &context));
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
