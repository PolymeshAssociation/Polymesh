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

use polymesh_primitives::{AccountKey, IdentityClaimData, IdentityId, Signatory, Ticker};
use polymesh_runtime_common::{
    balances::Trait as BalancesTrait, constants::*, identity::Trait as IdentityTrait, Context,
};
use polymesh_runtime_identity as identity;

use codec::Encode;
use core::result::Result as StdResult;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
use sp_std::{convert::TryFrom, prelude::*};

/// Type of claim requirements that a rule can have
#[derive(codec::Encode, codec::Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum RuleType {
    ClaimIsPresent,
    ClaimIsAbsent,
}

impl Default for RuleType {
    fn default() -> Self {
        RuleType::ClaimIsPresent
    }
}

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
/// All sender and receiver rules of the same asset rule must be true for tranfer to be valid
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetRule {
    pub sender_rules: Vec<RuleData>,
    pub receiver_rules: Vec<RuleData>,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetRules {
    pub is_paused: bool,
    pub rules: Vec<AssetRule>,
}

/// Details about individual rules
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct RuleData {
    /// Claim key
    pub claim: IdentityClaimData,

    /// Array of trusted claim issuers
    pub trusted_issuers: Vec<IdentityId>,

    /// Defines if it is a whitelist based rule or a blacklist based rule
    pub rule_type: RuleType,
}

type Identity<T> = identity::Module<T>;

decl_storage! {
    trait Store for Module<T: Trait> as GeneralTM {
        /// List of active rules for a ticker (Ticker -> Array of AssetRules)
        pub AssetRulesMap get(fn asset_rules): map Ticker => AssetRules;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The sender must be a signing key for the DID.
        SenderMustBeSigningKeyForDid,
        /// User is not authorized.
        Unauthorized,
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Adds an asset rule to active rules for a ticker
        pub fn add_active_rule(origin, ticker: Ticker, asset_rule: AssetRule) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            <AssetRulesMap>::mutate(ticker, |old_asset_rules| {
                if !old_asset_rules.rules.contains(&asset_rule) {
                    old_asset_rules.rules.push(asset_rule.clone());
                }
            });

            Self::deposit_event(Event::NewAssetRule(ticker, asset_rule));

            Ok(())
        }

        /// Removes a rule from active asset rules
        pub fn remove_active_rule(origin, ticker: Ticker, asset_rule: AssetRule) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            <AssetRulesMap>::mutate(ticker, |old_asset_rules| {
                old_asset_rules.rules.retain( |rule| { *rule != asset_rule });
            });

            Self::deposit_event(Event::RemoveAssetRule(ticker, asset_rule));

            Ok(())
        }

        /// Removes all active rules of a ticker
        pub fn reset_active_rules(origin, ticker: Ticker) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            <AssetRulesMap>::remove(ticker);

            Self::deposit_event(Event::ResetAssetRules(ticker));

            Ok(())
        }

        /// It pauses the verification of rules for `ticker` during transfers.
        pub fn pause_asset_rules(origin, ticker: Ticker) -> DispatchResult {
            Self::pause_resume_rules(origin, ticker, true)?;

            Self::deposit_event(Event::PauseAssetRules(ticker));
            Ok(())
        }

        /// It resumes the verification of rules for `ticker` during transfers.
        pub fn resume_asset_rules(origin, ticker: Ticker) -> DispatchResult {
            Self::pause_resume_rules(origin, ticker, false)?;

            Self::deposit_event(Event::ResumeAssetRules(ticker));
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event {
        NewAssetRule(Ticker, AssetRule),
        RemoveAssetRule(Ticker, AssetRule),
        ResetAssetRules(Ticker),
        ResumeAssetRules(Ticker),
        PauseAssetRules(Ticker),
    }
);

impl<T: Trait> Module<T> {
    fn is_owner(ticker: &Ticker, sender_did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, sender_did)
    }

    fn is_any_rule_broken(did: IdentityId, rules: Vec<RuleData>) -> bool {
        for rule in rules {
            let is_valid_claim_present =
                <identity::Module<T>>::is_any_claim_valid(did, rule.claim, rule.trusted_issuers);
            if rule.rule_type == RuleType::ClaimIsPresent && !is_valid_claim_present
                || rule.rule_type == RuleType::ClaimIsAbsent && is_valid_claim_present
            {
                return true;
            }
        }
        return false;
    }

    ///  Sender restriction verification
    pub fn verify_restriction(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        _value: T::Balance,
    ) -> StdResult<u8, &'static str> {
        // Transfer is valid if ALL reciever AND sender rules of ANY asset rule are valid.
        let asset_rules = Self::asset_rules(ticker);
        if asset_rules.is_paused {
            return Ok(ERC1400_TRANSFER_SUCCESS);
        }

        for active_rule in asset_rules.rules {
            let mut rule_broken = false;

            if let Some(from_did) = from_did_opt {
                rule_broken = Self::is_any_rule_broken(from_did, active_rule.sender_rules);
                if rule_broken {
                    // Skips checking receiver rules because sender rules are not satisfied.
                    continue;
                }
            }

            if let Some(to_did) = to_did_opt {
                rule_broken = Self::is_any_rule_broken(to_did, active_rule.receiver_rules)
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
        let sender = Signatory::AccountKey(sender_key);

        ensure!(
            <identity::Module<T>>::is_signer_authorized(did, &sender),
            Error::<T>::SenderMustBeSigningKeyForDid
        );
        ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

        <AssetRulesMap>::mutate(&ticker, |asset_rules| {
            asset_rules.is_paused = pause;
        });

        Ok(())
    }
}
