use crate::asset::{self, AssetTrait};
use crate::constants::*;
use crate::identity;
use crate::utils;
use codec::{Decode, Encode};
use core::result::Result as StdResult;
use rstd::prelude::*;
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::{self, ensure_signed};

// pub enum Types {
//     U8, U16, U32, U64, U128,
//     BYTES, STRING,
// }

// pub enum Operators {
//     EQUAL_TO, NOT_EQUAL_TO, LESS_THAN, GREATER_THAN, LESS_OR_EQUAL_TO, GREATER_OR_EQUAL_TO,
// }

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
    trusted_issuers: Vec<Vec<u8>>,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct RuleData {
    key: Vec<u8>,
    value: Vec<u8>,
    data_type: u8, // 0u8 1u16 2u32 3u64 4u128, 5bool, 6Vec<u8>
    operator: u8,  // 0= 1! 2< 3> 4<= 5>=
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

    pub fn fetch_value(did: &Vec<u8>, key: &Vec<u8>, trusted_issuers: &Vec<Vec<u8>>) -> Vec<u8> {
        // TODO Fetch value from Identity module
        return 1.encode();
    }

    pub fn check_rule(
        rule_data: Vec<u8>,
        identity_data: Vec<u8>,
        data_type: u8,
        operator: u8,
    ) -> bool {
        let mut rule_broken = false;
        match data_type {
            0 => {
                let rule_value = u8::decode(&mut &rule_data[..]).unwrap();
                let identity_value: u8 = u8::decode(&mut &identity_data[..]).unwrap();
                match operator {
                    0 => {
                        if rule_value != identity_value {
                            rule_broken = true;
                        }
                    }
                    1 => {
                        if rule_value == identity_value {
                            rule_broken = true;
                        }
                    }
                    2 => {
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    3 => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    4 => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    5 => {
                        if rule_value < identity_value {
                            rule_broken = true;
                        }
                    }
                    _ => {
                        rule_broken = true;
                    }
                }
            }
            1 => {
                let rule_value = u16::decode(&mut &rule_data[..]).unwrap();
                let identity_value = u16::decode(&mut &identity_data[..]).unwrap();
                match operator {
                    0 => {
                        if rule_value != identity_value {
                            rule_broken = true;
                        }
                    }
                    1 => {
                        if rule_value == identity_value {
                            rule_broken = true;
                        }
                    }
                    2 => {
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    3 => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    4 => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    5 => {
                        if rule_value < identity_value {
                            rule_broken = true;
                        }
                    }
                    _ => {
                        rule_broken = true;
                    }
                }
            }
            2 => {
                let rule_value = u32::decode(&mut &rule_data[..]).unwrap();
                let identity_value = u32::decode(&mut &identity_data[..]).unwrap();
                match operator {
                    0 => {
                        if rule_value != identity_value {
                            rule_broken = true;
                        }
                    }
                    1 => {
                        if rule_value == identity_value {
                            rule_broken = true;
                        }
                    }
                    2 => {
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    3 => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    4 => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    5 => {
                        if rule_value < identity_value {
                            rule_broken = true;
                        }
                    }
                    _ => {
                        rule_broken = true;
                    }
                }
            }
            3 => {
                let rule_value = u64::decode(&mut &rule_data[..]).unwrap();
                let identity_value = u64::decode(&mut &identity_data[..]).unwrap();
                match operator {
                    0 => {
                        if rule_value != identity_value {
                            rule_broken = true;
                        }
                    }
                    1 => {
                        if rule_value == identity_value {
                            rule_broken = true;
                        }
                    }
                    2 => {
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    3 => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    4 => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    5 => {
                        if rule_value < identity_value {
                            rule_broken = true;
                        }
                    }
                    _ => {
                        rule_broken = true;
                    }
                }
            }
            4 => {
                let rule_value = u128::decode(&mut &rule_data[..]).unwrap();
                let identity_value = u128::decode(&mut &identity_data[..]).unwrap();
                match operator {
                    0 => {
                        if rule_value != identity_value {
                            rule_broken = true;
                        }
                    }
                    1 => {
                        if rule_value == identity_value {
                            rule_broken = true;
                        }
                    }
                    2 => {
                        if rule_value >= identity_value {
                            rule_broken = true;
                        }
                    }
                    3 => {
                        if rule_value <= identity_value {
                            rule_broken = true;
                        }
                    }
                    4 => {
                        if rule_value > identity_value {
                            rule_broken = true;
                        }
                    }
                    5 => {
                        if rule_value < identity_value {
                            rule_broken = true;
                        }
                    }
                    _ => {
                        rule_broken = true;
                    }
                }
            }
            5 => {
                let rule_value = bool::decode(&mut &rule_data[..]).unwrap();
                let identity_value = bool::decode(&mut &identity_data[..]).unwrap();
                match operator {
                    0 => {
                        if rule_value != identity_value {
                            rule_broken = true;
                        }
                    }
                    1 => {
                        if rule_value == identity_value {
                            rule_broken = true;
                        }
                    }
                    _ => {
                        rule_broken = true;
                    }
                }
            }
            6 => {
                let rule_value = Vec::<u8>::decode(&mut &rule_data[..]).unwrap();
                let identity_value = Vec::<u8>::decode(&mut &identity_data[..]).unwrap();
                match operator {
                    0 => {
                        if rule_value != identity_value {
                            rule_broken = true;
                        }
                    }
                    1 => {
                        if rule_value == identity_value {
                            rule_broken = true;
                        }
                    }
                    _ => {
                        rule_broken = true;
                    }
                }
            }
            _ => {
                rule_broken = true;
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
        let ticker = utils::bytes_to_upper(ticker.as_slice());
        let active_rules = Self::active_rules(ticker.clone());
        for active_rule in active_rules {
            let mut rule_broken = false;
            for sender_rule in active_rule.sender_rules {
                for data in sender_rule.rules_data {
                    let identity_value = Self::fetch_value(&from_did, &data.key, &active_rule.trusted_issuers);
                    rule_broken =
                        Self::check_rule(data.value, identity_value, data.data_type, data.operator);
                }
                if rule_broken {
                    break;
                }
            }
            for receiver_rule in active_rule.receiver_rules {
                if rule_broken {
                    break;
                }
                for data in receiver_rule.rules_data {
                    let identity_value = Self::fetch_value(&from_did, &data.key, &active_rule.trusted_issuers);
                    rule_broken =
                        Self::check_rule(data.value, identity_value, data.data_type, data.operator);
                }
            }
            if !rule_broken {
                return Ok(ERC1400_TRANSFER_SUCCESS);
            }
        }

        sr_primitives::print("Identity TM restrictions not satisfied");
        Ok(ERC1400_TRANSFER_FAILURE)

        // let now = <timestamp::Module<T>>::get();
        // // issuance case
        // if from == T::AccountId::default() {
        //     ensure!(
        //         Self::_check_investor_status(to.clone()).is_ok(),
        //         "Account is not active"
        //     );
        //     ensure!(
        //         Self::is_whitelisted(_ticker.clone(), to).is_ok(),
        //         "to account is not whitelisted"
        //     );
        //     runtime_io::print("GTM: Passed from the issuance case");
        //     return Ok(());
        // } else if to == T::AccountId::default() {
        //     // burn case
        //     ensure!(
        //         Self::_check_investor_status(from.clone()).is_ok(),
        //         "Account is not active"
        //     );
        //     ensure!(
        //         Self::is_whitelisted(_ticker.clone(), from).is_ok(),
        //         "from account is not whitelisted"
        //     );
        //     runtime_io::print("GTM: Passed from the burn case");
        //     return Ok(());
        // } else {
        //     // loop through existing whitelists
        //     let whitelist_count = Self::whitelist_count();
        //     ensure!(
        //         Self::_check_investor_status(from.clone()).is_ok(),
        //         "Account is not active"
        //     );
        //     ensure!(
        //         Self::_check_investor_status(to.clone()).is_ok(),
        //         "Account is not active"
        //     );
        //     for x in 0..whitelist_count {
        //         let whitelist_for_from =
        //             Self::whitelist_for_restriction((ticker.clone(), x, from.clone()));
        //         let whitelist_for_to =
        //             Self::whitelist_for_restriction((ticker.clone(), x, to.clone()));

        //         if (whitelist_for_from.can_send_after > T::Moment::sa(0)
        //             && now >= whitelist_for_from.can_send_after)
        //             && (whitelist_for_to.can_receive_after > T::Moment::sa(0)
        //                 && now > whitelist_for_to.can_receive_after)
        //         {
        //             return Ok(());
        //         }
        //     }
        // }
        // runtime_io::print("GTM: Not going through the restriction");
        // Err("Cannot Transfer: General TM restrictions not satisfied")
    }
}

/// tests for this module
#[cfg(test)]
mod tests {}
