use crate::{Claim, ClaimType, IdentityId};
use codec::{Decode, Encode};
use sp_std::prelude::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
/// It defines the type of rule supported, and the filter information we will use to evaluate as a
/// predicate.
pub enum RuleType {
    /// Rule to ensure that claim filter produces one claim.
    IsPresent(Claim),
    /// Rule to ensure that claim filter produces an empty list.
    IsAbsent(Claim),
    /// Rule to ensure that at least one claim is fetched when filter is applied.
    IsAnyOf(Vec<Claim>),
    /// Rule to ensure that at none of claims is fetched when filter is applied.
    IsNoneOf(Vec<Claim>),
}

impl RuleType {
    /// It returns the claim type which will be searched and fetched during the evaluation process.
    /// # NOTE
    /// The case `IsAnyOf` is special and all the claims should have the same type. The first one
    /// will be used as the reference type, and any other claim value which differs of that type,
    /// will be ignored.
    /// If user defines a empty list of claims in `IsAnyOf`, `Jurisdiction` type will be used by
    /// default.
    pub fn as_claim_type(&self) -> ClaimType {
        match self {
            RuleType::IsPresent(ref claim) => claim.claim_type(),
            RuleType::IsAbsent(ref claim) => claim.claim_type(),
            RuleType::IsNoneOf(ref claims) => Self::get_claim_type(claims.as_slice()),
            RuleType::IsAnyOf(ref claims) => Self::get_claim_type(claims.as_slice()),
        }
    }

    fn get_claim_type(claims: &[Claim]) -> ClaimType {
        claims
            .iter()
            .map(|claim| claim.claim_type())
            .nth(0)
            .unwrap_or(ClaimType::NoType)
    }
}

/// Type of claim requirements that a rule can have
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Rule {
    /// Type of rule.
    pub rule_type: RuleType,
    /// Trusted issuers.
    pub issuers: Vec<IdentityId>,
}

impl From<RuleType> for Rule {
    fn from(rule_type: RuleType) -> Self {
        Rule {
            rule_type,
            issuers: Vec::<IdentityId>::new(),
        }
    }
}
