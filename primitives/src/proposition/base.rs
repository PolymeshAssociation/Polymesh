use crate::{
    proposition::{IdentityClaims, Proposition},
    Claim, IdentityId,
};
use codec::{Decode, Encode};

// TargetIdentityProposition
// ======================================================

/// Matches the contained `identity` against the `id` in the context.
#[derive(Clone, Debug)]
pub struct IsIdentityProposition {
    /// Identity to check against the one in the context.
    pub identity: IdentityId,
}

impl Proposition for IsIdentityProposition {
    #[inline]
    fn evaluate(&self, identity_claims: &IdentityClaims) -> bool {
        identity_claims.id == self.identity
    }
}

// ExistentialProposition
// ======================================================

/// It checks the existential of a claim.
///
/// # `CustomerDueDiligence` default search
/// The `CustomerDueDiligence` claim supports default search if you use the default `CddId` (a zero filled data).
/// For instance:
///     - The `exists(Claim::CustomerDueDiligence(CddId::default()))` matches with any CDD claim.
///     - The `exists(Claim::CustomerDueDiligence(a_valid_cdd_id))` matches only for the given
///     `a_valid_cdd_id`.
///
#[derive(Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExistentialProposition<'a> {
    /// Claims that must exist.
    pub claim: &'a Claim,
}

impl Proposition for ExistentialProposition<'_> {
    fn evaluate(&self, identity_claims: &IdentityClaims) -> bool {
        match &self.claim {
            // The default search only double-checks if any CDD claim is in the context.
            Claim::CustomerDueDiligence(cdd_id) if cdd_id.is_default_cdd() => identity_claims
                .claims
                .iter()
                .any(|id_claim| matches!(id_claim, Claim::CustomerDueDiligence(..))),
            // In regular claim evaluation, the data of the claim has to match too.
            _ => identity_claims
                .claims
                .iter()
                .any(|id_claim| id_claim == self.claim),
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

impl<P1, P2> Proposition for AndProposition<P1, P2>
where
    P1: Proposition,
    P2: Proposition,
{
    /// Evaluate proposition against `context`.
    #[inline]
    fn evaluate(&self, identity_claims: &IdentityClaims) -> bool {
        self.lhs.evaluate(identity_claims) && self.rhs.evaluate(identity_claims)
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

impl<P1, P2> Proposition for OrProposition<P1, P2>
where
    P1: Proposition,
    P2: Proposition,
{
    /// Evaluate proposition against `context`.
    #[inline]
    fn evaluate(&self, identity_claims: &IdentityClaims) -> bool {
        self.lhs.evaluate(identity_claims) || self.rhs.evaluate(identity_claims)
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

impl<P: Proposition> Proposition for NotProposition<P> {
    /// Evaluate proposition against `context`.
    #[inline]
    fn evaluate(&self, identity_claims: &IdentityClaims) -> bool {
        !self.proposition.evaluate(identity_claims)
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

impl Proposition for AnyProposition<'_> {
    /// Evaluate proposition against `context`.
    fn evaluate(&self, identity_claims: &IdentityClaims) -> bool {
        identity_claims.claims.iter().any(|id_claim| {
            self.claims
                .iter()
                .any(|valid_claim| id_claim == valid_claim)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        proposition::{self, IdentityClaims, Proposition},
        CddId, Claim, Condition, ConditionType, CountryCode, IdentityId, InvestorUid, Scope,
        TargetIdentity,
    };
    use std::convert::From;

    struct Dummy;
    impl Proposition for Dummy {
        fn evaluate(&self, _: &IdentityClaims) -> bool {
            false
        }
    }

    #[test]
    fn existential_operators_test() {
        let scope = Scope::Identity(IdentityId::from(0));
        let did = IdentityId::from(1);
        let cdd_claim =
            Claim::CustomerDueDiligence(CddId::new_v1(did, InvestorUid::from(b"UID1".as_ref())));
        let id_claims = IdentityClaims::new(
            did,
            vec![cdd_claim.clone(), Claim::Affiliate(scope.clone())],
        );

        // Affiliate && CustommerDueDiligenge
        let affiliate_claim = Claim::Affiliate(scope);
        let affiliate_and_cdd_pred = Proposition::and(
            proposition::exists(&affiliate_claim),
            proposition::exists(&cdd_claim),
        );

        assert_eq!(affiliate_and_cdd_pred.evaluate(&id_claims), true);
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

        let id_claims = IdentityClaims::new(
            IdentityId::default(),
            vec![Claim::Jurisdiction(CountryCode::CA, scope.clone())],
        );
        let in_juridisction_pre = proposition::any(&valid_jurisdictions);
        assert_eq!(in_juridisction_pre.evaluate(&id_claims), true);

        // 2. Check USA does not belong to {ESP, CAN, IND}.
        let id_claims = IdentityClaims::new(
            IdentityId::default(),
            vec![Claim::Jurisdiction(CountryCode::US, scope)],
        );
        assert_eq!(in_juridisction_pre.evaluate(&id_claims), false);

        // 3. Check NOT in jurisdiction.
        let not_in_jurisdiction_pre = proposition::not::<_>(in_juridisction_pre.clone());
        assert_eq!(not_in_jurisdiction_pre.evaluate(&id_claims), true);
    }

    #[test]
    fn run_proposition() {
        let scope = Scope::Identity(IdentityId::from(0));
        let external_agent_proposition = |_: &IdentityClaims| false;

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

        let check = |expected, id_claims: &IdentityClaims| {
            let out = !conditions.iter().any(|condition| {
                !proposition::run(&condition, id_claims, external_agent_proposition)
            });
            assert_eq!(out, expected);
        };

        // Valid case
        let id_claims = IdentityClaims::new(
            IdentityId::default(),
            vec![
                Claim::Accredited(scope.clone()),
                Claim::Jurisdiction(CountryCode::CA, scope.clone()),
            ],
        );

        check(true, &&id_claims);

        // Invalid case: `BuyLockup` is present.
        let id_claims = IdentityClaims::new(
            IdentityId::default(),
            vec![
                Claim::Accredited(scope.clone()),
                Claim::BuyLockup(scope.clone()),
                Claim::Jurisdiction(CountryCode::CA, scope.clone()),
            ],
        );
        check(false, &id_claims);

        // Invalid case: Missing `Accredited`
        let id_claims = IdentityClaims::new(
            IdentityId::default(),
            vec![
                Claim::BuyLockup(scope.clone()),
                Claim::Jurisdiction(CountryCode::CA, scope.clone()),
            ],
        );
        check(false, &id_claims);

        // Invalid case: Missing `Jurisdiction`
        let id_claims = IdentityClaims::new(
            IdentityId::default(),
            vec![
                Claim::Accredited(scope.clone()),
                Claim::Jurisdiction(CountryCode::ES, scope.clone()),
            ],
        );
        check(false, &id_claims);

        // Check NoneOf
        let id_claims = IdentityClaims::new(
            IdentityId::default(),
            vec![
                Claim::Accredited(scope.clone()),
                Claim::Jurisdiction(CountryCode::CU, scope.clone()),
            ],
        );
        check(false, &id_claims);

        let identity_id = IdentityId::from(1);
        let id_claims = IdentityClaims::new(identity_id, vec![]);
        assert!(proposition::run(
            &ConditionType::IsIdentity(TargetIdentity::ExternalAgent).into(),
            &id_claims,
            |id_claims: &IdentityClaims| id_claims.id == identity_id,
        ));
        assert!(proposition::run(
            &ConditionType::IsIdentity(TargetIdentity::Specific(identity_id)).into(),
            &id_claims,
            external_agent_proposition,
        ));
    }
}
