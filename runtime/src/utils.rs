use crate::balances;
use crate::general_tm::Operators;
use crate::identity::DataTypes;
use codec::{Codec, Decode};
use rstd::prelude::*;
use session;
use sr_primitives::traits::{Member, SimpleArithmetic};
use srml_support::Parameter;
use system;

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait + session::Trait {
    type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy;
    fn as_u128(v: Self::TokenBalance) -> u128;
    fn as_tb(v: u128) -> Self::TokenBalance;
    fn token_balance_to_balance(v: Self::TokenBalance) -> <Self as balances::Trait>::Balance;
    fn balance_to_token_balance(v: <Self as balances::Trait>::Balance) -> Self::TokenBalance;
    fn validator_id_to_account_id(v: <Self as session::Trait>::ValidatorId) -> Self::AccountId;
}

// Other utility functions
#[inline]
/// Convert all letter characters of a slice to their upper case counterparts.
/// # TODO
/// This functions is always called on `ticket`, maybe we could create a type for `ticket` to
/// ensure that type is UPPER case, and **avoid vector clone** (using `collect`).
pub fn bytes_to_upper(v: &[u8]) -> Vec<u8> {
    v.iter()
        .map(|chr| match chr {
            97..=122 => chr - 32,
            other => *other,
        })
        .collect()
}

pub fn check_rule(
    rule_data: Vec<u8>,
    identity_data: Vec<u8>,
    data_type: DataTypes,
    operator: Operators,
) -> bool {
    let mut rule_broken = false;
    match data_type {
        DataTypes::U8 => {
            let rule_value = u8::decode(&mut &rule_data[..]).unwrap();
            let identity_value: u8 = u8::decode(&mut &identity_data[..]).unwrap();
            match operator {
                Operators::EqualTo => {
                    if rule_value != identity_value {
                        rule_broken = true;
                    }
                }
                Operators::NotEqualTo => {
                    if rule_value == identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessThan => {
                    if rule_value <= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterThan => {
                    if rule_value >= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessOrEqualTo => {
                    if rule_value < identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterOrEqualTo => {
                    if rule_value > identity_value {
                        rule_broken = true;
                    }
                }
            }
        }
        DataTypes::U16 => {
            let rule_value = u16::decode(&mut &rule_data[..]).unwrap();
            let identity_value = u16::decode(&mut &identity_data[..]).unwrap();
            match operator {
                Operators::EqualTo => {
                    if rule_value != identity_value {
                        rule_broken = true;
                    }
                }
                Operators::NotEqualTo => {
                    if rule_value == identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessThan => {
                    if rule_value <= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterThan => {
                    if rule_value >= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessOrEqualTo => {
                    if rule_value < identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterOrEqualTo => {
                    if rule_value > identity_value {
                        rule_broken = true;
                    }
                }
            }
        }
        DataTypes::U32 => {
            let rule_value = u32::decode(&mut &rule_data[..]).unwrap();
            let identity_value = u32::decode(&mut &identity_data[..]).unwrap();
            match operator {
                Operators::EqualTo => {
                    if rule_value != identity_value {
                        rule_broken = true;
                    }
                }
                Operators::NotEqualTo => {
                    if rule_value == identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessThan => {
                    if rule_value <= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterThan => {
                    if rule_value >= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessOrEqualTo => {
                    if rule_value < identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterOrEqualTo => {
                    if rule_value > identity_value {
                        rule_broken = true;
                    }
                }
            }
        }
        DataTypes::U64 => {
            let rule_value = u64::decode(&mut &rule_data[..]).unwrap();
            let identity_value = u64::decode(&mut &identity_data[..]).unwrap();
            match operator {
                Operators::EqualTo => {
                    if rule_value != identity_value {
                        rule_broken = true;
                    }
                }
                Operators::NotEqualTo => {
                    if rule_value == identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessThan => {
                    if rule_value <= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterThan => {
                    if rule_value >= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessOrEqualTo => {
                    if rule_value < identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterOrEqualTo => {
                    if rule_value > identity_value {
                        rule_broken = true;
                    }
                }
            }
        }
        DataTypes::U128 => {
            let rule_value = u128::decode(&mut &rule_data[..]).unwrap();
            let identity_value = u128::decode(&mut &identity_data[..]).unwrap();
            match operator {
                Operators::EqualTo => {
                    if rule_value != identity_value {
                        rule_broken = true;
                    }
                }
                Operators::NotEqualTo => {
                    if rule_value == identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessThan => {
                    if rule_value <= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterThan => {
                    if rule_value >= identity_value {
                        rule_broken = true;
                    }
                }
                Operators::LessOrEqualTo => {
                    if rule_value < identity_value {
                        rule_broken = true;
                    }
                }
                Operators::GreaterOrEqualTo => {
                    if rule_value > identity_value {
                        rule_broken = true;
                    }
                }
            }
        }
        DataTypes::Bool => {
            let rule_value = bool::decode(&mut &rule_data[..]).unwrap();
            let identity_value = bool::decode(&mut &identity_data[..]).unwrap();
            match operator {
                Operators::EqualTo => {
                    if rule_value != identity_value {
                        rule_broken = true;
                    }
                }
                Operators::NotEqualTo => {
                    if rule_value == identity_value {
                        rule_broken = true;
                    }
                }
                _ => {
                    rule_broken = true;
                }
            }
        }
        DataTypes::VecU8 => {
            let rule_value = rule_data;
            let identity_value = identity_data;
            match operator {
                Operators::EqualTo => {
                    if rule_value != identity_value {
                        rule_broken = true;
                    }
                }
                Operators::NotEqualTo => {
                    if rule_value == identity_value {
                        rule_broken = true;
                    }
                }
                _ => {
                    rule_broken = true;
                }
            }
        }
    }
    return rule_broken;
}
