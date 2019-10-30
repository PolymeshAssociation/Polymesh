use crate::asset::{self, AssetTrait};
use crate::constants::*;
use crate::identity;
use crate::utils;
use codec::Encode;
use core::result::Result as StdResult;
use rstd::prelude::*;
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::{self, ensure_signed};
use identity::ClaimValue;

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
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
    type Asset: asset::AssetTrait<Self::TokenBalance>;
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetRule {
    sender_rules: Vec<RuleData>,
    receiver_rules: Vec<RuleData>,
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
        fn deposit_event() = default;

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

            Self::deposit_event(Event::NewAssetRule(ticker, asset_rule));

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

            Self::deposit_event(Event::RemoveAssetRule(ticker, asset_rule));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event {
        NewAssetRule(Vec<u8>, AssetRule),
        RemoveAssetRule(Vec<u8>, AssetRule),
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
                let identity_value =
                    Self::fetch_value(from_did.clone(), sender_rule.key, sender_rule.trusted_issuers);
                rule_broken = match identity_value {
                    None => true,
                    Some(x) => utils::check_rule(sender_rule.value, x.value, x.data_type, sender_rule.operator),
                };
                if rule_broken {
                    break;
                }
            }
            if rule_broken {
                continue;
            }
            for receiver_rule in active_rule.receiver_rules {
                let identity_value =
                    Self::fetch_value(from_did.clone(), receiver_rule.key, receiver_rule.trusted_issuers);
                rule_broken = match identity_value {
                    None => true,
                    Some(x) => utils::check_rule(receiver_rule.value, x.value, x.data_type, receiver_rule.operator),
                };
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
