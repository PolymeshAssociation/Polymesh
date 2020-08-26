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

use crate::{Claim, Condition, ConditionType, IdentityId, TargetIdentity, Ticker};
use codec::{Decode, Encode};

use sp_std::prelude::*;

/// Context using during an `Proposition` evaluation.
///
/// # TODO
///  - Use a lazy access to claims. It could be part of the optimization
///  process of CDD claims.
#[derive(Encode, Decode, Clone, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Context {
    /// Proposition evaluation will use those claims.
    pub claims: Vec<Claim>,
    /// Identity of this context.
    /// It could be the sender DID during the evaluation of sender's conditions or
    /// the receiver DID on a receiver's condition evaluation.
    pub id: IdentityId,
    /// Identity of the primary_issuance_agent of the token
    pub primary_issuance_agent: Option<IdentityId>,
}

// Proposition Trait
// ==================================

/// It allows composition and evaluation of claims based on a context.
pub trait Proposition {
    /// It evaluates this propositiond based on `context` context.
    fn evaluate(&self, context: &Context) -> bool;

    /// It generates a new proposition that represents the logical AND
    /// of two propositions: `Self` and `other`.
    #[inline]
    fn and<B>(self, other: B) -> AndProposition<Self, B>
    where
        Self: Sized,
        B: Proposition + Sized,
    {
        AndProposition::new(self, other)
    }

    /// It generates a new proposition that represents the logical OR
    /// of two propositions: `Self` and `other`.
    #[inline]
    fn or<B>(self, other: B) -> OrProposition<Self, B>
    where
        Self: Sized,
        B: Proposition + Sized,
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

/// Base and simple propositions
pub mod base;
pub use base::{
    AndProposition, AnyProposition, ExistentialProposition, NotProposition, OrProposition,
    TargetIdentityProposition,
};

/// Propositions for confidential stuff.
pub mod valid_proof_of_investor;
pub use valid_proof_of_investor::ValidProofOfInvestorProposition;

// Helper functions
// ======================================

/// It creates a proposition to evaluate the matching of `id` with primary_issuance_agent in the context.
#[inline]
pub fn equals<'a>(
    id: &'a TargetIdentity,
    primary_issuance_agent: &'a IdentityId,
) -> TargetIdentityProposition<'a> {
    match id {
        TargetIdentity::PrimaryIssuanceAgent => TargetIdentityProposition {
            identity: primary_issuance_agent,
        },
        TargetIdentity::Specific(identity) => TargetIdentityProposition { identity },
    }
}

/// It creates a proposition to evaluate the existential of `claim` in the context.
#[inline]
pub fn exists(claim: &'_ Claim) -> ExistentialProposition<'_> {
    ExistentialProposition { claim }
}

/// It creates a proposition to evaluate if any of `claims` are found in the context.
#[inline]
pub fn any(claims: &'_ [Claim]) -> AnyProposition<'_> {
    AnyProposition { claims }
}

/// It create a negate proposition of `proposition`.
#[inline]
pub fn not<P>(proposition: P) -> NotProposition<P>
where
    P: Proposition + Sized,
{
    NotProposition::new(proposition)
}

/// It verifies if the Identifier in the context has got a valid `InvestorZKProof` and its
/// associate `CustomDueDiligence`.
#[inline]
pub fn has_valid_proof_of_investor(ticker: Ticker) -> ValidProofOfInvestorProposition {
    ValidProofOfInvestorProposition { ticker }
}

/// Helper function to run propositions from a context.
pub fn run(condition: &Condition, context: &Context) -> bool {
    match condition.condition_type {
        ConditionType::IsPresent(ref claim) => exists(claim).evaluate(context),
        ConditionType::IsAbsent(ref claim) => not(exists(claim)).evaluate(context),
        ConditionType::IsAnyOf(ref claims) => any(claims).evaluate(context),
        ConditionType::IsNoneOf(ref claims) => not(any(claims)).evaluate(context),
        ConditionType::HasValidProofOfInvestor(ref ticker) => {
            has_valid_proof_of_investor(ticker.clone()).evaluate(context)
        }
        ConditionType::IsIdentity(ref id) => {
            equals(id, &context.primary_issuance_agent.unwrap_or_default()).evaluate(context)
        }
    }
}
