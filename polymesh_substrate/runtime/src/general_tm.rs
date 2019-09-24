use crate::asset::{self, AssetTrait};
use crate::identity::{self, InvestorList};
use crate::utils;

use codec::Encode;
use rstd::prelude::*;
use srml_support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue,
};
use system::{self, ensure_signed};

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
    bytes: Vec<RuleData>, // Array of {key value operator}
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
    operator: u16, // 0= 2! 3< 4> 5<= 6>=
}

decl_storage! {
    trait Store for Module<T: Trait> as GeneralTM {
        pub ActiveRules get(active_rules): map Vec<u8> => Vec<AssetRule>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn add_asset_rule(origin, _ticker: Vec<u8>, asset_rule: AssetRule) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_owner(ticker.clone(), sender.clone()), "Sender must be the token owner");

            <ActiveRules>::mutate(ticker.clone(), |old_asset_rules| {
                if !old_asset_rules.contains(&asset_rule) {
                    old_asset_rules.push(asset_rule.clone());
                }
            });

            //Self::deposit_event(RawEvent::NewAssetRule(ticker, asset_rule));

            Ok(())
        }

        fn remove_asset_rule(origin, _ticker: Vec<u8>, asset_rule: AssetRule) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_owner(ticker.clone(), sender.clone()), "Sender must be the token owner");

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
        T::Asset::is_owner(ticker.clone(), sender_did)
    }

    ///  Sender restriction verification
    pub fn verify_restriction(
        _ticker: Vec<u8>,
        from_did: Vec<u8>,
        to_did: Vec<u8>,
        _value: T::TokenBalance,
    ) -> Result {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        let active_rules = Self::active_rules(ticker.clone());
        for active_rule in active_rules {
            let rule_broken = false;
            for sender_rule in active_rule.sender_rules {
                for data in sender_rule.bytes {
                    // Verify all sender claims
                    // Set rule_broken = true if the rule is broken
                }
                if rule_broken {
                    break;
                }
            }
            for receiver_rule in active_rule.receiver_rules {
                if rule_broken {
                    break;
                }
                for data in receiver_rule.bytes {
                    // Verify all receiver claims
                    // Set rule_broken = true if the rule is broken
                }
            }
            if !rule_broken {
                return Ok(());
            }
        }

        sr_primitives::print("Identity TM restrictions not satisfied");
        Err("Cannot Transfer: Identity TM restrictions not satisfied")

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
mod tests {
}
