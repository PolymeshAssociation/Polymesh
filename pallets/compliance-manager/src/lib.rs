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

//! # Compliance Manager Module
//!
//! The Compliance Manager module provides functionality to set and evaluate a list of rules.
//! Those rules define transfer restrictions for the sender and receiver. For instance, you can limit your asset to investors
//! of specific jurisdictions.
//!
//!
//! ## Overview
//!
//! The Compliance Manager module provides functions for:
//!
//! - Adding rules for allowing transfers.
//! - Removing rules that allow transfers.
//! - Resetting all rules.
//!
//! ### Use case
//!
//! This module is very versatile and offers infinite possibilities.
//! The rules can dictate various requirements like:
//!
//! - Only accredited investors should be able to trade.
//! - Only valid CDD holders should be able to trade.
//! - Only those with credit score of greater than 800 should be able to purchase this token.
//! - People from "Wakanda" should only be able to trade with people from "Wakanda".
//! - People from "Gryffindor" should not be able to trade with people from "Slytherin" (But allowed to trade with anyone else).
//! - Only "Marvel" supporters should be allowed to buy "Avengers" token.
//!
//! ### Terminology
//!
//! - **Active rules:** It is an array of Asset rules that are currently enforced for a ticker.
//! - **Asset rule:** Every asset rule contains an array for sender rules and an array for receiver rules
//! - **sender rules:** These are rules that the sender of security tokens must follow
//! - **receiver rules:** These are rules that the receiver of security tokens must follow
//! - **Valid transfer:** For a transfer to be valid,
//!     All receiver and sender rules of any of the active asset rule must be followed.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - [add_active_rule](Module::add_active_rule) - Adds a new asset rule to ticker's active rules.
//! - [remove_active_rule](Module::remove_active_rule) - Removes an asset rule from ticker's active rules.
//! - [reset_active_rules](Module::reset_active_rules) - Resets(remove) all active rules of a ticker.
//! - [pause_asset_rules](Module::pause_asset_rules) - Pauses the evaluation of the rules of a ticker before execute a
//! transaction.
//! - [resume_asset_rules](Module::resume_asset_rules) - Resumes a previous paused rules of a ticket.
//! - [add_default_trusted_claim_issuer](Module::add_default_trusted_claim_issuer) - Adds a default
//!  trusted claim issuer for a given asset.
//!  - [batch_add_default_trusted_claim_issuer](Module::batch_add_default_trusted_claim_issuer) -
//!  Adds a list of claim issuer to the default trusted claim issuers for a given asset.
//! - [remove_default_trusted_claim_issuer](Module::remove_default_trusted_claim_issuer) - Removes
//!  the default claim issuer.
//! - [change_asset_rule](Module::change_asset_rule) - Updates an asset rule, based on its id.
//! - [batch_change_asset_rule](Module::batch_change_asset_rule) - Updates a list of asset rules,
//! based on its id for a given asset.
//!
//! ### Public Functions
//!
//! - [verify_restriction](Module::verify_restriction) - Checks if a transfer is a valid transfer and returns the result
//!
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use pallet_identity as identity;
use polymesh_common_utilities::{
    asset::Trait as AssetTrait,
    balances::Trait as BalancesTrait,
    compliance_manager::Trait as ComplianceManagerTrait,
    constants::*,
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    Context,
};
use polymesh_primitives::{
    predicate, AccountKey, Claim, IdentityId, Rule, RuleType, Signatory, Ticker,
};

use codec::Encode;
use core::result::Result as StdResult;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    weights::{DispatchClass, FunctionOf, SimpleDispatchInfo},
};
use frame_system::{self as system, ensure_signed};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
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
    type Asset: AssetTrait<Self::Balance, Self::AccountId>;
}

/// An asset transfer rule.
/// All sender and receiver rule of the same asset rule must be true in order to execute the transfer.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetTransferRule {
    pub sender_rules: Vec<Rule>,
    pub receiver_rules: Vec<Rule>,
    /// Unique identifier of the asset rule
    pub rule_id: u32,
}

/// An asset transfer rule along with its evaluation result
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub struct AssetTransferRuleResult {
    pub sender_rules: Vec<RuleResult>,
    pub receiver_rules: Vec<RuleResult>,
    /// Unique identifier of the asset rule
    pub rule_id: u32,
    /// Result of this transfer rule's evaluation
    pub transfer_rule_result: bool,
}

impl From<AssetTransferRule> for AssetTransferRuleResult {
    fn from(asset_rule: AssetTransferRule) -> Self {
        Self {
            sender_rules: asset_rule
                .sender_rules
                .iter()
                .map(|rule| RuleResult::from(rule.clone()))
                .collect(),
            receiver_rules: asset_rule
                .receiver_rules
                .iter()
                .map(|rule| RuleResult::from(rule.clone()))
                .collect(),
            rule_id: asset_rule.rule_id,
            transfer_rule_result: true,
        }
    }
}

/// An individual rule along with its evaluation result
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub struct RuleResult {
    // Rule being evaluated
    pub rule: Rule,
    // Result of evaluation
    pub result: bool,
}

impl From<Rule> for RuleResult {
    fn from(rule: Rule) -> Self {
        Self { rule, result: true }
    }
}

/// List of rules associated to an asset.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetTransferRules {
    /// This flag indicates if asset transfer rules are active or paused.
    pub is_paused: bool,
    /// List of rules.
    pub rules: Vec<AssetTransferRule>,
}

type Identity<T> = identity::Module<T>;

/// Rules evaluation result
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub struct AssetTransferRulesResult {
    /// This flag indicates if asset transfer rules are active or paused.
    pub is_paused: bool,
    /// List of rules.
    pub rules: Vec<AssetTransferRuleResult>,
    // Final evaluation result of all rules
    pub final_result: bool,
}

impl From<AssetTransferRules> for AssetTransferRulesResult {
    fn from(asset_rules: AssetTransferRules) -> Self {
        Self {
            is_paused: asset_rules.is_paused,
            rules: asset_rules
                .rules
                .iter()
                .map(|rule| AssetTransferRuleResult::from(rule.clone()))
                .collect(),
            final_result: false,
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as ComplianceManager {
        /// List of active rules for a ticker (Ticker -> Array of AssetTransferRules)
        pub AssetRulesMap get(fn asset_rules): map hasher(blake2_128_concat) Ticker => AssetTransferRules;
        /// List of trusted claim issuer Ticker -> Issuer Identity
        pub TrustedClaimIssuer get(fn trusted_claim_issuer): map hasher(blake2_128_concat) Ticker => Vec<IdentityId>;
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
        /// When parameter has length < 1
        InvalidLength,
        /// Rule id doesn't exist
        InvalidRuleId,
        /// Issuer exist but trying to add it again
        IncorrectOperationOnTrustedIssuer,
        /// Missing current DID
        MissingCurrentIdentity,
        /// There are dulpicate asset rules.
        DuplicateAssetRules,
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Adds an asset rule to active rules for a ticker.
        /// If rules are duplicated, it does nothing.
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
                ProtocolOp::ComplianceManagerAddActiveRule
            )?;
            let new_rule = AssetTransferRule {
                sender_rules: sender_rules,
                receiver_rules: receiver_rules,
                rule_id: Self::get_latest_rule_id(ticker) + 1u32
            };

            <AssetRulesMap>::mutate(ticker, |old_asset_rules| {
                if !old_asset_rules.rules.iter().position(|rule| rule.sender_rules == new_rule.sender_rules && rule.receiver_rules == new_rule.receiver_rules).is_some() {
                    old_asset_rules.rules.push(new_rule.clone());
                    Self::deposit_event(Event::NewAssetRuleCreated(did, ticker, new_rule));
                }
            });

            Ok(())
        }

        /// Removes a rule from asset rules.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        /// * asset_rule_id - Rule id which is need to be removed
        #[weight = SimpleDispatchInfo::FixedNormal(200_000)]
        pub fn remove_active_rule(origin, ticker: Ticker, asset_rule_id: u32) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            <AssetRulesMap>::mutate(ticker, |old_asset_rules| {
                old_asset_rules.rules.retain( |rule| { rule.rule_id != asset_rule_id });
            });

            Self::deposit_event(Event::AssetRuleRemoved(did, ticker, asset_rule_id));

            Ok(())
        }

        /// Replaces asset rules of a ticker with new rules.
        ///
        /// # Arguments
        /// * `ticker` - the asset ticker,
        /// * `asset_rules - the new asset rules.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the ticker.
        /// * `DuplicateAssetRules` if `asset_rules` contains multiple entries with the same `rule_id`.
        ///
        /// # Weight
        /// `150_000 + 50_000 * asset_rules.len()`
        #[weight = FunctionOf(
            |(_, asset_rules): (
                &Ticker,
                &Vec<AssetTransferRule>,
            )| {
                150_000 + 50_000 * u32::try_from(asset_rules.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn replace_asset_rules(origin, ticker: Ticker, asset_rules: Vec<AssetTransferRule>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            let mut asset_rules_dedup = asset_rules.clone();
            asset_rules_dedup.dedup_by_key(|r| r.rule_id);
            ensure!(asset_rules.len() == asset_rules_dedup.len(), Error::<T>::DuplicateAssetRules);
            <AssetRulesMap>::mutate(&ticker, |old_asset_rules| {
                old_asset_rules.rules = asset_rules_dedup
            });
            Self::deposit_event(Event::AssetRulesReplaced(did, ticker, asset_rules));
            Ok(())
        }

        /// Removes all active rules of a given ticker
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        pub fn reset_active_rules(origin, ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            <AssetRulesMap>::remove(ticker);

            Self::deposit_event(Event::AssetRulesReset(did, ticker));

            Ok(())
        }

        /// It pauses the verification of rules for `ticker` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        pub fn pause_asset_rules(origin, ticker: Ticker) -> DispatchResult {
            Self::pause_resume_rules(origin, ticker, true)?;
            let current_did = Context::current_identity::<Identity<T>>().ok_or_else(|| Error::<T>::MissingCurrentIdentity)?;
            Self::deposit_event(Event::AssetRulesPaused(current_did, ticker));
            Ok(())
        }

        /// It resumes the verification of rules for `ticker` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        pub fn resume_asset_rules(origin, ticker: Ticker) -> DispatchResult {
            Self::pause_resume_rules(origin, ticker, false)?;
            let current_did = Context::current_identity::<Identity<T>>().ok_or_else(|| Error::<T>::MissingCurrentIdentity)?;
            Self::deposit_event(Event::AssetRulesResumed(current_did, ticker));
            Ok(())
        }

        /// To add the default trusted claim issuer for a given asset
        /// Addition - When the given element is not exist
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuer - IdentityId of the trusted claim issuer.
        #[weight = SimpleDispatchInfo::FixedNormal(300_000)]
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
        #[weight = SimpleDispatchInfo::FixedNormal(300_000)]
        pub fn remove_default_trusted_claim_issuer(origin, ticker: Ticker, trusted_issuer: IdentityId) -> DispatchResult {
            Self::modify_default_trusted_claim_issuer(origin, ticker, trusted_issuer, false)
        }

        /// To add a list of default trusted claim issuers for a given asset
        /// Addition - When the given element is not exist
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuers - Vector of IdentityId of the trusted claim issuers.
        ///
        /// # Weight
        /// `50_000 + 250_000 * trusted_issuers.len().max(values.len())`
        #[weight = FunctionOf(
            |(trusted_issuers, _): (
                &Vec<IdentityId>,
                &Ticker,
            )| {
                50_000 + 250_000 * u32::try_from(trusted_issuers.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn batch_add_default_trusted_claim_issuer(origin, trusted_issuers: Vec<IdentityId>, ticker: Ticker) -> DispatchResult {
            Self::batch_modify_default_trusted_claim_issuer(origin, ticker, trusted_issuers, true)
        }

        /// To remove the default trusted claim issuer for a given asset
        /// Removal - When the given element is already present
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuers - Vector of IdentityId of the trusted claim issuers.
        ///
        /// # Weight
        /// `50_000 + 250_000 * trusted_issuers.len().max(values.len())`
        #[weight = FunctionOf(
            |(trusted_issuers, _): (
                &Vec<IdentityId>,
                &Ticker,
            )| {
                50_000 + 250_000 * u32::try_from(trusted_issuers.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn batch_remove_default_trusted_claim_issuer(origin, trusted_issuers: Vec<IdentityId>, ticker: Ticker) -> DispatchResult {
            Self::batch_modify_default_trusted_claim_issuer(origin, ticker, trusted_issuers, false)
        }

        /// Change/Modify the existing asset rule of a given ticker
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * asset_rule - Asset rule.
        #[weight = SimpleDispatchInfo::FixedNormal(150_000)]
        pub fn change_asset_rule(origin, ticker: Ticker, asset_rule: AssetTransferRule) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            ensure!(Self::get_latest_rule_id(ticker) >= asset_rule.rule_id, Error::<T>::InvalidRuleId);
            Self::unsafe_change_asset_rule(did, ticker, asset_rule);
            Ok(())
        }

        /// Change/Modify the existing asset rule of a given ticker in batch
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * asset_rules - Vector of asset rule.
        /// * ticker - Symbol of the asset.
        ///
        /// # Weight
        /// `100_000 + 100_000 * asset_rules.len().max(values.len())`
        #[weight = FunctionOf(
            |(asset_rules, _): (
                &Vec<AssetTransferRule>,
                &Ticker,
            )| {
                100_000 + 100_000 * u32::try_from(asset_rules.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn batch_change_asset_rule(origin, asset_rules: Vec<AssetTransferRule> , ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            let latest_rule_id = Self::get_latest_rule_id(ticker);
            ensure!(asset_rules.iter().any(|rule| latest_rule_id >= rule.rule_id), Error::<T>::InvalidRuleId);

            asset_rules.into_iter().for_each(|asset_rule| {
                Self::unsafe_change_asset_rule(did, ticker, asset_rule);
            });
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event {
        /// Emitted when new asset rule is created.
        /// (caller DID, Ticker, AssetRule).
        NewAssetRuleCreated(IdentityId, Ticker, AssetTransferRule),
        /// Emitted when asset rule is removed.
        /// (caller DID, Ticker, Asset_rule_id).
        AssetRuleRemoved(IdentityId, Ticker, u32),
        /// Emitted when all asset rules of a ticker get replaced.
        /// Parameters: caller DID, ticker, new asset rules.
        AssetRulesReplaced(IdentityId, Ticker, Vec<AssetTransferRule>),
        /// Emitted when all asset rules of a ticker get reset.
        /// (caller DID, Ticker).
        AssetRulesReset(IdentityId, Ticker),
        /// Emitted when asset rules for a given ticker gets resume.
        /// (caller DID, Ticker).
        AssetRulesResumed(IdentityId, Ticker),
        /// Emitted when asset rules for a given ticker gets paused.
        /// (caller DID, Ticker).
        AssetRulesPaused(IdentityId, Ticker),
        /// Emitted when asset rule get modified/change.
        /// (caller DID, Ticker, AssetTransferRule).
        AssetRuleChanged(IdentityId, Ticker, AssetTransferRule),
        /// Emitted when default claim issuer list for a given ticker gets added.
        /// (caller DID, Ticker, New Claim issuer DID).
        TrustedDefaultClaimIssuerAdded(IdentityId, Ticker, IdentityId),
        /// Emitted when default claim issuer list for a given ticker get removed.
        /// (caller DID, Ticker, Removed Claim issuer DID).
        TrustedDefaultClaimIssuerRemoved(IdentityId, Ticker, IdentityId),
    }
);

impl<T: Trait> Module<T> {
    /// Returns true if `sender_did` is the owner of `ticker` asset.
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
    ///
    /// If `rule` does not define trusted issuers, it will use the default trusted issuer for
    /// `ticker` asset.
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

    /// It loads a context for each rule in `rules` and evaluates them.
    /// It updates the internal result variable of every rule.
    /// It returns the final result of all rules combined.
    fn evaluate_rules(ticker: &Ticker, did: IdentityId, rules: &mut Vec<RuleResult>) -> bool {
        let mut result = true;
        for rule in rules {
            let context = Self::fetch_context(ticker, did, &rule.rule);
            rule.result = predicate::run(rule.rule.clone(), &context);
            if !rule.result {
                result = false;
            }
        }
        result
    }

    /// Pauses or resumes the asset rules.
    fn pause_resume_rules(origin: T::Origin, ticker: Ticker, pause: bool) -> DispatchResult {
        let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
        let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

        ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

        <AssetRulesMap>::mutate(&ticker, |asset_rules| {
            asset_rules.is_paused = pause;
        });

        Ok(())
    }

    /// Updates the default trusted claim issuer for a given ticket.
    fn unsafe_modify_default_trusted_claim_issuer(
        caller_did: IdentityId,
        ticker: Ticker,
        trusted_issuer: IdentityId,
        is_add_call: bool,
    ) {
        TrustedClaimIssuer::mutate(ticker, |identity_list| {
            if !is_add_call {
                // remove the old one
                identity_list.retain(|&ti| ti != trusted_issuer);
                Self::deposit_event(Event::TrustedDefaultClaimIssuerRemoved(
                    caller_did,
                    ticker,
                    trusted_issuer,
                ));
            } else {
                // New trusted issuer addition case
                identity_list.push(trusted_issuer);
                Self::deposit_event(Event::TrustedDefaultClaimIssuerAdded(
                    caller_did,
                    ticker,
                    trusted_issuer,
                ));
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
        Self::unsafe_modify_default_trusted_claim_issuer(did, ticker, trusted_issuer, is_add_call);
        Ok(())
    }

    fn batch_modify_default_trusted_claim_issuer(
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
            Self::unsafe_modify_default_trusted_claim_issuer(
                did,
                ticker,
                default_issuer,
                is_add_call,
            );
        });
        Ok(())
    }

    fn unsafe_change_asset_rule(
        caller_did: IdentityId,
        ticker: Ticker,
        new_asset_rule: AssetTransferRule,
    ) {
        <AssetRulesMap>::mutate(&ticker, |asset_rules| {
            if let Some(index) = asset_rules
                .rules
                .iter()
                .position(|rule| &rule.rule_id == &new_asset_rule.rule_id)
            {
                asset_rules.rules[index] = new_asset_rule.clone();
            }
        });
        Self::deposit_event(Event::AssetRuleChanged(caller_did, ticker, new_asset_rule));
    }

    // TODO: Cache the latest_rule_id to avoid loading of all asset_rules in memory.
    fn get_latest_rule_id(ticker: Ticker) -> u32 {
        let length = Self::asset_rules(ticker).rules.len();
        match length > 0 {
            true => Self::asset_rules(ticker).rules[length - 1].rule_id,
            false => 0u32,
        }
    }

    /// verifies all rules and returns the result in an array of bools.
    /// this does not care if the rules are paused or not. It is meant to be
    /// called only in failure conditions (rules active)
    pub fn granular_verify_restriction(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
    ) -> AssetTransferRulesResult {
        let asset_rules = Self::asset_rules(ticker);
        let mut asset_rules_with_results = AssetTransferRulesResult::from(asset_rules);
        for active_rule in &mut asset_rules_with_results.rules {
            if let Some(from_did) = from_did_opt {
                // Evaluate all sender rules
                if !Self::evaluate_rules(ticker, from_did, &mut active_rule.sender_rules) {
                    // If the result of any of the sender rules was false, set this asset rule result to false.
                    active_rule.transfer_rule_result = false;
                }
            }
            if let Some(to_did) = to_did_opt {
                // Evaluate all receiver rules
                if !Self::evaluate_rules(ticker, to_did, &mut active_rule.receiver_rules) {
                    // If the result of any of the receiver rules was false, set this asset rule result to false.
                    active_rule.transfer_rule_result = false;
                }
            }
            // If the asset rule result is positive, update the final result to be positive
            if active_rule.transfer_rule_result {
                asset_rules_with_results.final_result = true;
            }
        }
        asset_rules_with_results
    }
}

impl<T: Trait> ComplianceManagerTrait<T::Balance> for Module<T> {
    ///  Sender restriction verification
    fn verify_restriction(
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
}
