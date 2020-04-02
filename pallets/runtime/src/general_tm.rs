//! # General Transfer Manager Module
//!
//! The GTM module provides functionality for setting whitelisting rules for transfers
//!
//! ## Overview
//!
//! The GTM module provides functions for:
//!
//! - Adding rules for allowing transfers
//! - Removing rules that allow transfers
//! - Resetting all rules
//!
//! ### Use case
//!
//! This module is very versatile and offers infinite possibilities.
//! The rules can dictate various requirements like:
//!
//! - Only accredited investors should be able to trade
//! - Only valid CDD holders should be able to trade
//! - Only those with credit score of greater than 800 should be able to purchase this token
//! - People from Wakanda should only be able to trade with people from Wakanda
//! - People from Gryffindor should not be able to trade with people from Slytherin (But allowed to trade with anyone else)
//! - Only marvel supporters should be allowed to buy avengers token
//!
//! ### Terminology
//!
//! - **Active rules:** It is an array of Asset rules that are currently enforced for a ticker
//! - **Asset rule:** Every asset rule contains an array for sender rules and an array for receiver rules
//! - **sender rules:** These are rules that the sender of security tokens must follow
//! - **receiver rules:** These are rules that the receiver of security tokens must follow
//! - **Valid transfer:** For a transfer to be valid,
//!     All reciever and sender rules of any of the active asset rule must be followed.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `add_active_rule` - Adds a new asset rule to ticker's active rules
//! - `remove_active_rule` - Removes an asset rule from ticker's active rules
//! - `reset_active_rules` - Reset(remove) all active rules of a tikcer
//!
//! ### Public Functions
//!
//! - `verify_restriction` - Checks if a transfer is a valid transfer and returns the result

use crate::asset::{self, AssetTrait};

use polymesh_primitives::{
    predicate, AccountKey, Claim, IdentityId, Rule, RuleType, Signatory, Ticker,
};
use polymesh_runtime_common::{
    balances::Trait as BalancesTrait,
    constants::*,
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    Context,
};
use polymesh_runtime_identity as identity;

use codec::Encode;
use core::result::Result as StdResult;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
use sp_std::{
    convert::{From, TryFrom},
    prelude::*,
};

/// The module's configuration trait.
pub trait Trait:
    pallet_timestamp::Trait + frame_system::Trait + BalancesTrait + IdentityTrait
{
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    /// Asset module
    type Asset: asset::AssetTrait<Self::Balance, Self::AccountId>;
}

/// An asset rule.
/// All sender and receiver rules of the same asset rule must be true for transfer to be valid
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetTransferRule {
    pub sender_rules: Vec<Rule>,
    pub receiver_rules: Vec<Rule>,
    /// Unique identifier of the asset rule
    pub rule_id: u32,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetTransferRules {
    pub is_paused: bool,
    pub rules: Vec<AssetTransferRule>,
}

type Identity<T> = identity::Module<T>;

decl_storage! {
    trait Store for Module<T: Trait> as GeneralTM {
        /// List of active rules for a ticker (Ticker -> Array of AssetTransferRules)
        pub AssetRulesMap get(fn asset_rules): map hasher(blake2_256) Ticker => AssetTransferRules;
        /// List of trusted claim issuer Ticker -> Issuer Identity
        pub TrustedClaimIssuer get(fn trusted_claim_issuer): map hasher(blake2_256) Ticker => Vec<IdentityId>;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The sender must be a signing key for the DID.
        SenderMustBeSigningKeyForDid,
        /// User is not authorized.
        Unauthorized,
        /// Did not exist
        DidNotExist,
        /// When param has length < 1
        InvalidLength,
        /// Rule id doesn't exist
        InvalidRuleId,
        /// Issuer exist but trying to add it again
        IncorrectOperationOnTrustedIssuer
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Adds an asset rule to active rules for a ticker
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        /// * sender_rules - Sender transfer rule.
        /// * receiver_rules - Receiver transfer rule.
        pub fn add_active_rule(origin, ticker: Ticker, sender_rules: Vec<Rule>, receiver_rules: Vec<Rule>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            <<T as IdentityTrait>::ProtocolFee>::charge_fee(
                &Signatory::AccountKey(sender_key),
                ProtocolOp::GeneralTmAddActiveRule
            )?;
            let new_rule = AssetTransferRule {
                sender_rules: sender_rules,
                receiver_rules: receiver_rules,
                rule_id: Self::get_latest_rule_id(ticker) + 1u32
            };

            <AssetRulesMap>::mutate(ticker, |old_asset_rules| {
                if !old_asset_rules.rules.iter().position(|rule| rule.sender_rules == new_rule.sender_rules && rule.receiver_rules == new_rule.receiver_rules).is_some() {
                    old_asset_rules.rules.push(new_rule.clone());
                    Self::deposit_event(Event::NewAssetRule(ticker, new_rule));
                }
            });

            Ok(())
        }

        /// Removes a rule from active asset rules
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        /// * asset_rule_id - Rule id which is need to be removed
        pub fn remove_active_rule(origin, ticker: Ticker, asset_rule_id: u32) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            <AssetRulesMap>::mutate(ticker, |old_asset_rules| {
                old_asset_rules.rules.retain( |rule| { rule.rule_id != asset_rule_id });
            });

            Self::deposit_event(Event::RemoveAssetRule(ticker, asset_rule_id));

            Ok(())
        }

        /// Removes all active rules of a given ticker
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        pub fn reset_active_rules(origin, ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            <AssetRulesMap>::remove(ticker);

            Self::deposit_event(Event::ResetAssetRules(ticker));

            Ok(())
        }

        /// It pauses the verification of rules for `ticker` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        pub fn pause_asset_rules(origin, ticker: Ticker) -> DispatchResult {
            Self::pause_resume_rules(origin, ticker, true)?;

            Self::deposit_event(Event::PauseAssetRules(ticker));
            Ok(())
        }

        /// It resumes the verification of rules for `ticker` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        pub fn resume_asset_rules(origin, ticker: Ticker) -> DispatchResult {
            Self::pause_resume_rules(origin, ticker, false)?;

            Self::deposit_event(Event::ResumeAssetRules(ticker));
            Ok(())
        }

        /// To add the default trusted claim issuer for a given asset
        /// Addition - When the given element is not exist
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuer - IdentityId of the trusted claim issuer.
        pub fn add_default_trusted_claim_issuer(origin, ticker: Ticker, trusted_issuer: IdentityId) -> DispatchResult {
            Self::modify_default_trusted_claim_issuer(origin, ticker, trusted_issuer, true)
        }

        /// To remove the default trusted claim issuer for a given asset
        /// Removal - When the given element is already present
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuer - IdentityId of the trusted claim issuer.
        pub fn remove_default_trusted_claim_issuer(origin, ticker: Ticker, trusted_issuer: IdentityId) -> DispatchResult {
            Self::modify_default_trusted_claim_issuer(origin, ticker, trusted_issuer, false)
        }

        /// To add the default trusted claim issuer for a given asset
        /// Addition - When the given element is not exist
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuers - Vector of IdentityId of the trusted claim issuers.
        pub fn add_default_trusted_claim_issuers_batch(origin, ticker: Ticker, trusted_issuers: Vec<IdentityId>) -> DispatchResult {
            Self::modify_default_trusted_claim_issuers_batch(origin, ticker, trusted_issuers, true)
        }

        /// To remove the default trusted claim issuer for a given asset
        /// Removal - When the given element is already present
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuers - Vector of IdentityId of the trusted claim issuers.
        pub fn remove_default_trusted_claim_issuers_batch(origin, ticker: Ticker, trusted_issuers: Vec<IdentityId>) -> DispatchResult {
            Self::modify_default_trusted_claim_issuers_batch(origin, ticker, trusted_issuers, false)
        }

        /// Change/Modify the existing asset rule of a given ticker
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * asset_rule - Asset rule.
        pub fn change_asset_rule(origin, ticker: Ticker, asset_rule: AssetTransferRule) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            ensure!(Self::get_latest_rule_id(ticker) >= asset_rule.rule_id, Error::<T>::InvalidRuleId);
            Self::unsafe_change_asset_rule(ticker, asset_rule);
            Ok(())
        }

        /// Change/Modify the existing asset rule of a given ticker in batch
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * asset_rules - Vector of asset rule.
        pub fn change_asset_rule_batch(origin, ticker: Ticker, asset_rules: Vec<AssetTransferRule>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            let latest_rule_id = Self::get_latest_rule_id(ticker);
            ensure!(asset_rules.iter().any(|rule| latest_rule_id >= rule.rule_id), Error::<T>::InvalidRuleId);

            asset_rules.into_iter().for_each(|asset_rule| {
                Self::unsafe_change_asset_rule(ticker, asset_rule);
            });
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event {
        /// Emitted when new asset rule is created
        /// (Ticker, AssetRule)
        NewAssetRule(Ticker, AssetTransferRule),
        /// Emitted when asset rule is removed
        /// (Ticker, Asset_rule_id)
        RemoveAssetRule(Ticker, u32),
        /// Emitted when all asset rules of a ticker get reset
        ResetAssetRules(Ticker),
        /// Emitted when asset rules for a given ticker gets resume.
        ResumeAssetRules(Ticker),
        /// Emitted when asset rules for a given ticker gets paused.
        PauseAssetRules(Ticker),
        /// Emitted when asset rule get modified/change
        ChangeAssetRule(Ticker, AssetTransferRule),
        /// Emitted when default claim issuer list for a given ticker gets added
        AddTrustedDefaultClaimIssuer(Ticker, IdentityId),
        /// Emitted when default claim issuer list for a given ticker get removed
        RemoveTrustedDefaultClaimIssuer(Ticker, IdentityId),
    }
);

impl<T: Trait> Module<T> {
    fn is_owner(ticker: &Ticker, sender_did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, sender_did)
    }

    /// It fetches all claims of `target` identity with type and scope from `claim` and generated
    /// by any of `issuers`.
    fn fetch_claims(target: IdentityId, claim: &Claim, issuers: &[IdentityId]) -> Vec<Claim> {
        let claim_type = claim.claim_type();
        let scope = claim.as_scope().cloned();

        issuers
            .iter()
            .flat_map(|issuer| {
                <identity::Module<T>>::fetch_claim(target, claim_type, *issuer, scope)
                    .map(|id_claim| id_claim.claim)
            })
            .collect::<Vec<_>>()
    }

    /// It fetches the predicate context for target `id` and specific `rule`.
    fn fetch_context(ticker: &Ticker, id: IdentityId, rule: &Rule) -> predicate::Context {
        let issuers = if !rule.issuers.is_empty() {
            rule.issuers.clone()
        } else {
            Self::trusted_claim_issuer(ticker)
        };

        let claims = match rule.rule_type {
            RuleType::IsPresent(ref claim) => Self::fetch_claims(id, claim, &issuers),
            RuleType::IsAbsent(ref claim) => Self::fetch_claims(id, claim, &issuers),
            RuleType::IsAnyOf(ref claims) => claims
                .iter()
                .flat_map(|claim| Self::fetch_claims(id, claim, &issuers))
                .collect::<Vec<_>>(),
            RuleType::IsNoneOf(ref claims) => claims
                .iter()
                .flat_map(|claim| Self::fetch_claims(id, claim, &issuers))
                .collect::<Vec<_>>(),
        };

        predicate::Context::from(claims)
    }

    /// It loads a context for each rule in `rules` and verify if any of them is evaluated as a
    /// false predicate. In that case, rule is considered as a "broken rule".
    fn is_any_rule_broken(ticker: &Ticker, did: IdentityId, rules: Vec<Rule>) -> bool {
        rules.into_iter().any(|rule| {
            let context = Self::fetch_context(ticker, did, &rule);
            !predicate::run(rule, &context)
        })
    }

    ///  Sender restriction verification
    pub fn verify_restriction(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        _value: T::Balance,
    ) -> StdResult<u8, &'static str> {
        // Transfer is valid if ALL receiver AND sender rules of ANY asset rule are valid.
        let asset_rules = Self::asset_rules(ticker);
        if asset_rules.is_paused {
            return Ok(ERC1400_TRANSFER_SUCCESS);
        }

        for active_rule in asset_rules.rules {
            let mut rule_broken = false;

            if let Some(from_did) = from_did_opt {
                rule_broken = Self::is_any_rule_broken(ticker, from_did, active_rule.sender_rules);
                if rule_broken {
                    // Skips checking receiver rules because sender rules are not satisfied.
                    continue;
                }
            }

            if let Some(to_did) = to_did_opt {
                rule_broken = Self::is_any_rule_broken(ticker, to_did, active_rule.receiver_rules)
            }

            if !rule_broken {
                return Ok(ERC1400_TRANSFER_SUCCESS);
            }
        }

        sp_runtime::print("Identity TM restrictions not satisfied");
        Ok(ERC1400_TRANSFER_FAILURE)
    }

    pub fn pause_resume_rules(origin: T::Origin, ticker: Ticker, pause: bool) -> DispatchResult {
        let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
        let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

        ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

        <AssetRulesMap>::mutate(&ticker, |asset_rules| {
            asset_rules.is_paused = pause;
        });

        Ok(())
    }

    fn unsafe_modify_default_trusted_claim_issuer(
        ticker: Ticker,
        trusted_issuer: IdentityId,
        is_add_call: bool,
    ) {
        TrustedClaimIssuer::mutate(ticker, |identity_list| {
            if !is_add_call {
                // remove the old one
                identity_list.retain(|&ti| ti != trusted_issuer);
                Self::deposit_event(Event::RemoveTrustedDefaultClaimIssuer(
                    ticker,
                    trusted_issuer,
                ));
            } else {
                // New trusted issuer addition case
                identity_list.push(trusted_issuer);
                Self::deposit_event(Event::AddTrustedDefaultClaimIssuer(ticker, trusted_issuer));
            }
        });
    }

    fn modify_default_trusted_claim_issuer(
        origin: T::Origin,
        ticker: Ticker,
        trusted_issuer: IdentityId,
        is_add_call: bool,
    ) -> DispatchResult {
        let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
        let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

        ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
        // ensure whether the trusted issuer's did is register did or not
        ensure!(
            <Identity<T>>::is_identity_exists(&trusted_issuer),
            Error::<T>::DidNotExist
        );
        ensure!(
            Self::trusted_claim_issuer(&ticker).contains(&trusted_issuer) == !is_add_call,
            Error::<T>::IncorrectOperationOnTrustedIssuer
        );
        Self::unsafe_modify_default_trusted_claim_issuer(ticker, trusted_issuer, is_add_call);
        Ok(())
    }

    fn modify_default_trusted_claim_issuers_batch(
        origin: T::Origin,
        ticker: Ticker,
        trusted_issuers: Vec<IdentityId>,
        is_add_call: bool,
    ) -> DispatchResult {
        let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
        let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

        ensure!(trusted_issuers.len() >= 1, Error::<T>::InvalidLength);
        ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
        // Perform validity checks on the data set
        for trusted_issuer in trusted_issuers.iter() {
            // Ensure whether the right operation is performed on trusted issuer or not
            // if is_add_call == true then trusted_claim_issuer should not exists.
            // if is_add_call == false then trusted_claim_issuer should exists.
            ensure!(
                Self::trusted_claim_issuer(&ticker).contains(&trusted_issuer) == !is_add_call,
                Error::<T>::IncorrectOperationOnTrustedIssuer
            );
            // ensure whether the trusted issuer's did is register did or not
            ensure!(
                <Identity<T>>::is_identity_exists(trusted_issuer),
                Error::<T>::DidNotExist
            );
        }

        // iterate all the trusted issuer and modify the data of those.
        trusted_issuers.into_iter().for_each(|default_issuer| {
            Self::unsafe_modify_default_trusted_claim_issuer(ticker, default_issuer, is_add_call);
        });
        Ok(())
    }

    fn unsafe_change_asset_rule(ticker: Ticker, new_asset_rule: AssetTransferRule) {
        <AssetRulesMap>::mutate(&ticker, |asset_rules| {
            if let Some(index) = asset_rules
                .rules
                .iter()
                .position(|rule| &rule.rule_id == &new_asset_rule.rule_id)
            {
                asset_rules.rules[index] = new_asset_rule.clone();
            }
        });
        Self::deposit_event(Event::ChangeAssetRule(ticker, new_asset_rule));
    }

    // TODO: Cache the latest_rule_id to avoid loading of all asset_rules in memory.
    fn get_latest_rule_id(ticker: Ticker) -> u32 {
        let length = Self::asset_rules(ticker).rules.len();
        match length > 0 {
            true => Self::asset_rules(ticker).rules[length - 1].rule_id,
            false => 0u32,
        }
    }
}
