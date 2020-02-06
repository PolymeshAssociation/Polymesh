use polymesh_runtime_common::{balances::Trait as BalancesTrait, identity::DataTypes};

use crate::general_tm::Operators;
use codec::{Decode, Encode};
use frame_system;
use pallet_session;
use sp_runtime::traits::{IdentifyAccount, Member, Verify};
use sp_std::prelude::*;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + BalancesTrait + pallet_session::Trait {
    type Public: IdentifyAccount<AccountId = Self::AccountId>;
    type OffChainSignature: Verify<Signer = Self::Public> + Member + Decode + Encode;
    fn validator_id_to_account_id(
        v: <Self as pallet_session::Trait>::ValidatorId,
    ) -> Self::AccountId;
}

pub fn is_rule_broken(
    rule_data: Vec<u8>,
    identity_data: Vec<u8>,
    data_type: DataTypes,
    operator: Operators,
) -> bool {
    let mut rule_broken = false;
    match data_type {
        DataTypes::U8 => {
            let rule_value_result = u8::decode(&mut &rule_data[..]);
            let identity_value_result = u8::decode(&mut &identity_data[..]);
            if rule_value_result.is_err() || identity_value_result.is_err() {
                return true;
            }
            let rule_value = rule_value_result.unwrap_or_default();
            let identity_value = identity_value_result.unwrap_or_default();
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
            let rule_value_result = u16::decode(&mut &rule_data[..]);
            let identity_value_result = u16::decode(&mut &identity_data[..]);
            if rule_value_result.is_err() || identity_value_result.is_err() {
                return true;
            }
            let rule_value = rule_value_result.unwrap_or_default();
            let identity_value = identity_value_result.unwrap_or_default();
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
            let rule_value_result = u32::decode(&mut &rule_data[..]);
            let identity_value_result = u32::decode(&mut &identity_data[..]);
            if rule_value_result.is_err() || identity_value_result.is_err() {
                return true;
            }
            let rule_value = rule_value_result.unwrap_or_default();
            let identity_value = identity_value_result.unwrap_or_default();
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
            let rule_value_result = u64::decode(&mut &rule_data[..]);
            let identity_value_result = u64::decode(&mut &identity_data[..]);
            if rule_value_result.is_err() || identity_value_result.is_err() {
                return true;
            }
            let rule_value = rule_value_result.unwrap_or_default();
            let identity_value = identity_value_result.unwrap_or_default();
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
            let rule_value_result = u128::decode(&mut &rule_data[..]);
            let identity_value_result = u128::decode(&mut &identity_data[..]);
            if rule_value_result.is_err() || identity_value_result.is_err() {
                return true;
            }
            let rule_value = rule_value_result.unwrap_or_default();
            let identity_value = identity_value_result.unwrap_or_default();
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
            let rule_value_result = bool::decode(&mut &rule_data[..]);
            let identity_value_result = bool::decode(&mut &identity_data[..]);
            if rule_value_result.is_err() || identity_value_result.is_err() {
                return true;
            }
            let rule_value = rule_value_result.unwrap_or_default();
            let identity_value = identity_value_result.unwrap_or_default();
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
