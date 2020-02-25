use crate::{IdentityId, ClaimValue, DataTypes };

use codec::{ Encode, Decode };
use sp_std::{ ops::Deref, prelude::* };

trait Evaluable<L,R> {
    fn evaluate(&self, lhs: &L, rhs: &R) -> bool;
}

/// Type of operators that a rule can have
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum RationalOperator {
    EqualTo,
    NotEqualTo,
    LessThan,
    GreaterThan,
    LessOrEqualTo,
    GreaterOrEqualTo,
}

impl Default for RationalOperator {
    fn default() -> Self {
        RationalOperator::EqualTo
    }
}

impl<L,R> Evaluable<L,R> for RationalOperator
where
    L: PartialEq<R> + PartialOrd<R>
{
    fn evaluate(&self, lhs: &L, rhs: &R) -> bool {
        match self {
            RationalOperator::EqualTo => lhs == rhs,
            RationalOperator::NotEqualTo => lhs != rhs,
            RationalOperator::LessThan => lhs < rhs,
            RationalOperator::GreaterThan => lhs > rhs,
            RationalOperator::LessOrEqualTo => lhs <= rhs,
            RationalOperator::GreaterOrEqualTo => lhs >= rhs,
        }
    }
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum LogicOperator {
    And,
    Or
}

/*
impl<L,R> Evaluable<L,R> for LogicOperator
where
    L: Into<bool>,
    R: Into<bool>
{
    fn evaluate(&self, lhs: &L, rhs: &R) -> bool {
        match self {
            LogicOperator::And => lhs.into() && rhs.into(),
            LogicOperator::Or => lhs.into() || rhs.into(),
        }
    }
}*/


#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum CollectionOperator {
    MemberOf,
    SubsetOf,
    ProperSubsetOf
}

impl Default for CollectionOperator {
    fn default() -> Self {
        CollectionOperator::MemberOf
    }
}

/*
impl<L,CR> Evaluable<L,CR> for CollectionOperator
where
    CR: Deref
{
    fn evaluate(&self, lhs: &L, rhs: &CR) -> bool {
        match self {
            CollectionOperator::MemberOf => rhs.deref().iter().any(lhs),
            CollectionOperator::SubsetOf => unimplemented!(),
            CollectionOperator::ProperSubsetOf => unimplemented!(),
        }
    }
}*/

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Operator {
    Rational(RationalOperator),
    Collection(CollectionOperator),
}

impl<L,R> Evaluable<L,R> for Operator
where
    L: PartialEq<R> + PartialOrd<R>
{
    fn evaluate(&self, lhs: &L, rhs: &R) -> bool {
        match self {
            Operator::Rational(op) => op.evaluate(lhs,rhs),
            // Operator::Collection(op) => op.evaluate(lhs,rhs),
            Operator::Collection(op) => unimplemented!(),
        }
    }
}

impl Default for Operator {
    fn default() -> Self {
        Operator::Rational( RationalOperator::default())
    }
}

/// Details about individual rules
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct RuleData {
    /// Claim key
    pub key: Vec<u8>,

    /// Claim target value. (RHS of operatior)
    pub value: ClaimValue,

    /// Array of trusted claim issuers
    pub trusted_issuers: Vec<IdentityId>,

    /// Operator. The rule is "Actual claim value" Operator "Rule value defined in this struct"
    /// Example: If the actual claim value is 5, value defined here is 10 and operator is NotEqualTo
    /// Then the rule will be resolved as 5 != 10 which is true and hence the rule will pass
    pub operator: Operator,
}


#[derive(Encode, Decode, Clone, Debug)]
pub enum Predicate {
    Unary(RuleData),
    Binary(Box<Predicate>, LogicOperator, Box<Predicate>),
}

fn evaluate_with_optional<O: Evaluable<T,T>, T>( op: O, lhs: T, rhs: Option<T>) -> bool
{
    if let Some(rhs) = rhs {
        op.evaluate( lhs, rhs)
    } else {
        false
    }
}

impl Predicate {
    pub fn evaluate(&self, v: &ClaimValue) -> bool {
        match self {
            Predicate::Unary(ref rule) => Self::evaluate_unary( rule, v),
            Predicate::Binary(ref lhs, ref op, ref rhs) => Self::evaluate_binary( lhs, *op, rhs),
        }
    }

    fn evaluate_unary( rule: &RuleData, v: ClaimValue) -> bool {
        match rule.value {
            ClaimValue::U8(claim_value) =>
                evaluate_with_optional(
                    rule.operator, claim_value,
                    rule.value.clone().try_into::<u8>().ok()),
            ClaimValue::U16(claim_value) =>
                evaluate_with_optional(
                    rule.operator, claim_value,
                    rule.value.clone().try_into::<u16>().ok()),
            _ => unimplemented!(),
        }
    }

    fn evaluate_binary( lhs: &Predicate, op: LogicOperator, rhs: &Predicate) -> bool {
        unimplemented!()
    }
}

impl From<RuleData> for Predicate {
    fn from( rule: RuleData) -> Predicate {
        Predicate::Unary(rule)
    }
}

pub struct PredicateBuilder {
    pub predicate: Option<Predicate>,
}

impl PredicateBuilder {
    pub fn new( predicate: Predicate) -> Self {
        PredicateBuilder { predicate }
    }

    pub fn and(mut self, p: Predicate) -> Self {
        self.predicate = match self.predicate {
            Some(lhs) => Predicate::Binary( lhs, LogicOperator::And, p),
            None => unimplemented!(),
        };
        self
    }

    pub fn or(mut self, p: Predicate) -> Self {
        self.predicate = match self.predicate {
            Some(lhs) => Predicate::Binary( lhs, LogicOperator::Or, p),
            None => unimplemented!(),
        };
        self
    }

    pub fn build(self) -> Predicate {
        self.predicate.unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rational_operators_test() {
        let cv_10 = ClaimValue::from( 10u8);
        let cv_20 = ClaimValue::from( 20u8);

        // Equal
        let rule_eq_10 = RuleData{
            key: b"equal to 10".to_vec(),
            value: 10u8.encode(),
            trusted_issuers: vec![],
            operator: Operator::Rational(RationalOperator::EqualTo)
        };
        let pre_eq_10 = Predicate::from( rule_eq_10);
        assert_eq!( pre_eq_10.evaluate(cv_10), true);
        assert_eq!( pre_eq_10.evaluate(cv_20), false);

        // NotEqualTo,
        let rule_ne_10 = RuleData{
            key: b"no equal to 10".to_vec(),
            value: 10u8.encode(),
            trusted_issuers: vec![],
            operator: Operator::Rational(RationalOperator::NotEqualTo)
        };
        let pre_ne_10 = Predicate::from( rule_ne_10);
        assert_eq!( pre_ne_10.evaluate(cv_10), false);
        assert_eq!( pre_ne_10.evaluate(cv_20), true);

        // LessThan,
        // GreaterThan,
        // LessOrEqualTo,
        // GreaterOrEqualTo,

    }

    #[test]
    fn collection_operators_test() {


    }
}
