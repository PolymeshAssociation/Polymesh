use crate::asset::{self, AssetTrait};
use crate::constants::*;
use crate::identity;
use crate::utils;
use codec::{Decode, Encode};
use core::result::Result as StdResult;
use rstd::prelude::*;
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::{self, ensure_signed};
use identity::{ClaimValue, DataTypes};

#[derive(codec::Encode, codec::Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Operators {
    EqualTo,
    NotEqualTo,
    LessThan,
    GreaterThan,
    LessOrEqualTo,
    GreaterOrEqualTo,
}

impl Default for Operators {
    fn default() -> Self {
        Operators::EqualTo
    }
}

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait + utils::Trait + identity::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Asset: asset::AssetTrait<Self::TokenBalance>;
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Rule {
    topic: u32,
    schema: u32,
    rules_data: Vec<RuleData>, // Array of {key value operator}
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetRule {
    sender_rules: Vec<Rule>,
    receiver_rules: Vec<Rule>,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct RuleData {
    key: Vec<u8>,
    value: Vec<u8>,
    trusted_issuers: Vec<Vec<u8>>,
    operator: Operators,
}

decl_storage! {
    trait Store for Module<T: Trait> as GeneralTM {
        // (Asset -> AssetRules)
        pub ActiveRules get(active_rules): map Vec<u8> => Vec<AssetRule>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn add_asset_rule(origin, did: Vec<u8>, _ticker: Vec<u8>, asset_rule: AssetRule) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;

            ensure!(<identity::Module<T>>::is_signing_key(&did, &sender.encode()), "sender must be a signing key for DID");

            ensure!(Self::is_owner(ticker.clone(), did.clone()), "user is not authorized");

            <ActiveRules>::mutate(ticker.clone(), |old_asset_rules| {
                if !old_asset_rules.contains(&asset_rule) {
                    old_asset_rules.push(asset_rule.clone());
                }
            });

            //Self::deposit_event(RawEvent::NewAssetRule(ticker, asset_rule));

            Ok(())
        }

        fn remove_asset_rule(origin, did: Vec<u8>, _ticker: Vec<u8>, asset_rule: AssetRule) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;

            ensure!(<identity::Module<T>>::is_signing_key(&did, &sender.encode()), "sender must be a signing key for DID");

            ensure!(Self::is_owner(ticker.clone(), did.clone()), "user is not authorized");

            <ActiveRules>::mutate(ticker.clone(), |old_asset_rules| {
                *old_asset_rules = old_asset_rules
                    .iter()
                    .cloned()
                    .filter(|an_asset_rule| *an_asset_rule != asset_rule)
                    .collect();
            });

            //Self::deposit_event(RawEvent::RemoveAssetRule(ticker, asset_rule));

            Ok(())
        }

        fn test(origin, ticker: Vec<u8>, from_did: Vec<u8>, to_did: Vec<u8>, value: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            let general_status_code =
                Self::verify_restriction(&ticker, &from_did, &to_did, value)?;
            if general_status_code != ERC1400_TRANSFER_SUCCESS {
                sr_primitives::print("satisfied");
            } else {
                sr_primitives::print("not satisfied2");
            }
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        NewAssetRule(Vec<u8>, AssetRule, AccountId),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(_ticker: Vec<u8>, sender_did: Vec<u8>) -> bool {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        T::Asset::is_owner(&ticker, &sender_did)
    }

    pub fn fetch_value(
        did: Vec<u8>,
        key: Vec<u8>,
        trusted_issuers: Vec<Vec<u8>>,
    ) -> Option<ClaimValue> {
        <identity::Module<T>>::fetch_claim_value_multiple_issuers(did, key, trusted_issuers)
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
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterThan => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::LessOrEqualTo => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterOrEqualTo => {
                        if rule_value < identity_value {
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
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterThan => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::LessOrEqualTo => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterOrEqualTo => {
                        if rule_value < identity_value {
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
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterThan => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::LessOrEqualTo => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterOrEqualTo => {
                        if rule_value < identity_value {
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
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterThan => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::LessOrEqualTo => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterOrEqualTo => {
                        if rule_value < identity_value {
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
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterThan => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::LessOrEqualTo => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    Operators::GreaterOrEqualTo => {
                        if rule_value < identity_value {
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
                let rule_value = Vec::<u8>::decode(&mut &rule_data[..]).unwrap();
                let identity_value = Vec::<u8>::decode(&mut &identity_data[..]).unwrap();
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

    ///  Sender restriction verification
    pub fn verify_restriction(
        ticker: &Vec<u8>,
        from_did: &Vec<u8>,
        to_did: &Vec<u8>,
        _value: T::TokenBalance,
    ) -> StdResult<u8, &'static str> {
        // Transfer is valid if All reciever and sender rules of any asset rule are valid.
        let ticker = utils::bytes_to_upper(ticker.as_slice());
        let active_rules = Self::active_rules(ticker.clone());
        for active_rule in active_rules {
            let mut rule_broken = false;
            for sender_rule in active_rule.sender_rules {
                for data in sender_rule.rules_data {
                    let identity_value =
                        Self::fetch_value(from_did.clone(), data.key, data.trusted_issuers);
                    rule_broken = match identity_value {
                        None => true,
                        Some(x) => Self::check_rule(data.value, x.value, x.data_type, data.operator),
                    };
                    if rule_broken {
                        break;
                    }
                }
                if rule_broken {
                    break;
                }
            }
            if rule_broken {
                continue;
            }
            for receiver_rule in active_rule.receiver_rules {
                for data in receiver_rule.rules_data {
                    let identity_value =
                        Self::fetch_value(from_did.clone(), data.key, data.trusted_issuers);
                    rule_broken = match identity_value {
                        None => true,
                        Some(x) => Self::check_rule(data.value, x.value, x.data_type, data.operator),
                    };
                    if rule_broken {
                        break;
                    }
                }
                if rule_broken {
                    break;
                }
            }
            if !rule_broken {
                return Ok(ERC1400_TRANSFER_SUCCESS);
            }
        }

        sr_primitives::print("Identity TM restrictions not satisfied");
        Ok(ERC1400_TRANSFER_FAILURE)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {}
