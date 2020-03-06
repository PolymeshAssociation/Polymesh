use crate::{Claim, ClaimType, IdentityId};
use codec::{Decode, Encode};
use sp_std::prelude::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
/// Type of claim requirements that a rule can have
pub enum RuleType {
    /// Rule to ensure that claim filter produces one claim.
    IsPresent(ClaimType),
    /// Rule to ensure that claim filter produces an empty list.
    IsAbsent(ClaimType),
    /// Rule to ensure that at least one claim is fetched when filter is applied.
    IsAnyOf(Vec<Claim>),
}

impl RuleType {
    /// It returns the
    pub fn as_claim_type(&self) -> ClaimType {
        match self {
            RuleType::IsPresent(claim_type) => *claim_type,
            RuleType::IsAbsent(claim_type) => *claim_type,
            RuleType::IsAnyOf(ref claims) => claims
                .iter()
                .map(|claim| claim.claim_type())
                .nth(0)
                .unwrap_or(ClaimType::CustomerDueDiligence),
        }
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
            issuers: vec![],
        }
    }
}
