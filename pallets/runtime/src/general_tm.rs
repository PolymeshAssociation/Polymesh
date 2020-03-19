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

use polymesh_primitives::{predicate, AccountKey, Claim, IdentityId, Rule, RuleType, Ticker};
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
        pub AssetRulesMap get(fn asset_rules): map Ticker => AssetTransferRules;
        /// List of trusted claim issuer Ticker -> Issuer Identity
        pub TrustedClaimIssuer get(fn trusted_claim_issuer): map Ticker => Vec<IdentityId>;
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
        fn add_default_trusted_claim_issuer(origin, ticker: Ticker, trusted_issuer: IdentityId) -> DispatchResult {
            Self::modify_default_trusted_claim_issuer(origin, ticker, trusted_issuer, true)
        }

        /// To remove the default trusted claim issuer for a given asset
        /// Removal - When the given element is already present
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuer - IdentityId of the trusted claim issuer.
        fn remove_default_trusted_claim_issuer(origin, ticker: Ticker, trusted_issuer: IdentityId) -> DispatchResult {
            Self::modify_default_trusted_claim_issuer(origin, ticker, trusted_issuer, false)
        }

        /// To add the default trusted claim issuer for a given asset
        /// Addition - When the given element is not exist
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuers - Vector of IdentityId of the trusted claim issuers.
        fn add_default_trusted_claim_issuers_batch(origin, ticker: Ticker, trusted_issuers: Vec<IdentityId>) -> DispatchResult {
            Self::modify_default_trusted_claim_issuers_batch(origin, ticker, trusted_issuers, true)
        }

        /// To remove the default trusted claim issuer for a given asset
        /// Removal - When the given element is already present
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * trusted_issuers - Vector of IdentityId of the trusted claim issuers.
        fn remove_default_trusted_claim_issuers_batch(origin, ticker: Ticker, trusted_issuers: Vec<IdentityId>) -> DispatchResult {
            Self::modify_default_trusted_claim_issuers_batch(origin, ticker, trusted_issuers, false)
        }

        /// Change/Modify the existing asset rule of a given ticker
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * asset_rule - Asset rule.
        fn change_asset_rule(origin, ticker: Ticker, asset_rule: AssetTransferRule) -> DispatchResult {
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
        fn change_asset_rule_batch(origin, ticker: Ticker, asset_rules: Vec<AssetTransferRule>) -> DispatchResult {
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

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use frame_support::traits::Currency;
    use frame_support::{
        assert_err, assert_ok, dispatch::DispatchResult, impl_outer_dispatch, impl_outer_origin,
        parameter_types, weights::DispatchInfo,
    };
    use frame_system::EnsureSignedBy;
    use sp_core::{crypto::key_types, H256};
    use sp_runtime::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys, Verify},
        transaction_validity::{TransactionValidity, ValidTransaction},
        AnySignature, KeyTypeId, Perbill,
    };
    use sp_std::result::Result;
    use test_client::{self, AccountKeyring};

    use polymesh_primitives::{Claim, IdentityId, Rule, RuleType, Scope, Signatory};
    use polymesh_runtime_balances as balances;
    use polymesh_runtime_common::traits::{
        asset::AcceptTransfer, group::GroupTrait, multisig::AddSignerMultiSig, CommonTrait,
    };

    use polymesh_runtime_group as group;
    use polymesh_runtime_identity as identity;

    use crate::{
        asset::{AssetType, Error as AssetError, SecurityToken, TickerRegistrationConfig},
        exemption, percentage_tm, statistics,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    impl_outer_dispatch! {
        pub enum Call for Test where origin: Origin {
            pallet_contracts::Contracts,
            identity::Identity,
        }
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;

    parameter_types! {
        pub const BlockHashCount: u32 = 250;
        pub const MaximumBlockWeight: u32 = 4096;
        pub const MaximumBlockLength: u32 = 4096;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }

    impl frame_system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = BlockNumber;
        type Call = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
    }

    parameter_types! {
        pub const ExistentialDeposit: u64 = 0;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
        pub const TransactionBaseFee: u64 = 0;
        pub const TransactionByteFee: u64 = 0;
    }

    impl CommonTrait for Test {
        type Balance = u128;
        type CreationFee = CreationFee;
        type AcceptTransferTarget = Test;
        type BlockRewardsReserve = balances::Module<Test>;
    }

    impl AcceptTransfer for Test {
        fn accept_ticker_transfer(_to_did: IdentityId, _auth_id: u64) -> DispatchResult {
            unimplemented!();
        }

        fn accept_token_ownership_transfer(_to_did: IdentityId, _auth_id: u64) -> DispatchResult {
            unimplemented!();
        }
    }

    impl GroupTrait for Test {
        fn get_members() -> Vec<IdentityId> {
            unimplemented!();
        }
    }

    impl balances::Trait for Test {
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type Identity = identity::Module<Test>;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    type SessionIndex = u32;
    type AuthorityId = <AnySignature as Verify>::Signer;
    type BlockNumber = u64;
    type AccountId = <AnySignature as Verify>::Signer;
    type OffChainSignature = AnySignature;

    impl pallet_timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    pub struct TestOnSessionEnding;
    impl pallet_session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
        fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
            None
        }
    }

    pub struct TestSessionHandler;
    impl pallet_session::SessionHandler<AuthorityId> for TestSessionHandler {
        const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];
        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            _validators: &[(AuthorityId, Ks)],
            _queued_validators: &[(AuthorityId, Ks)],
        ) {
        }

        fn on_disabled(_validator_index: usize) {}

        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}

        fn on_before_session_ending() {}
    }

    parameter_types! {
        pub const Period: BlockNumber = 1;
        pub const Offset: BlockNumber = 0;
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    }

    impl pallet_session::Trait for Test {
        type OnSessionEnding = TestOnSessionEnding;
        type Keys = UintAuthorityId;
        type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AuthorityId;
        type ValidatorIdOf = ConvertInto;
        type SelectInitialValidators = ();
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl pallet_session::historical::Trait for Test {
        type FullIdentification = ();
        type FullIdentificationOf = ();
    }

    parameter_types! {
        pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
    }

    impl group::Trait<group::Instance1> for Test {
        type Event = ();
        type AddOrigin = EnsureSignedBy<One, AccountId>;
        type RemoveOrigin = EnsureSignedBy<Two, AccountId>;
        type SwapOrigin = EnsureSignedBy<Three, AccountId>;
        type ResetOrigin = EnsureSignedBy<Four, AccountId>;
        type MembershipInitialized = ();
        type MembershipChanged = ();
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = Call;
        type AddSignerMultiSigTarget = Test;
        type CddServiceProviders = Test;
        type Balances = balances::Module<Test>;
        type ChargeTxFeeTarget = Test;
        type Public = AccountId;
        type OffChainSignature = OffChainSignature;
    }

    impl pallet_transaction_payment::ChargeTxFee for Test {
        fn charge_fee(_who: Signatory, _len: u32, _info: DispatchInfo) -> TransactionValidity {
            Ok(ValidTransaction::default())
        }
    }

    impl AddSignerMultiSig for Test {
        fn accept_multisig_signer(_: Signatory, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }

    impl asset::Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }

    impl statistics::Trait for Test {}

    impl percentage_tm::Trait for Test {
        type Event = ();
    }

    impl exemption::Trait for Test {
        type Event = ();
        type Asset = asset::Module<Test>;
    }

    impl Trait for Test {
        type Event = ();
        type Asset = asset::Module<Test>;
    }

    parameter_types! {
        pub const SignedClaimHandicap: u64 = 2;
        pub const TombstoneDeposit: u64 = 16;
        pub const StorageSizeOffset: u32 = 8;
        pub const RentByteFee: u64 = 4;
        pub const RentDepositOffset: u64 = 10_000;
        pub const SurchargeReward: u64 = 150;
        pub const ContractTransactionBaseFee: u64 = 2;
        pub const ContractTransactionByteFee: u64 = 6;
        pub const ContractFee: u64 = 21;
        pub const CallBaseFee: u64 = 135;
        pub const InstantiateBaseFee: u64 = 175;
        pub const MaxDepth: u32 = 100;
        pub const MaxValueSize: u32 = 16_384;
        pub const ContractTransferFee: u64 = 50000;
        pub const ContractCreationFee: u64 = 50;
        pub const BlockGasLimit: u64 = 10000000;
    }

    impl pallet_contracts::Trait for Test {
        type Currency = Balances;
        type Time = Timestamp;
        type Randomness = Randomness;
        type Call = Call;
        type Event = ();
        type DetermineContractAddress = pallet_contracts::SimpleAddressDeterminator<Test>;
        type ComputeDispatchFee = pallet_contracts::DefaultDispatchFeeComputor<Test>;
        type TrieIdGenerator = pallet_contracts::TrieIdFromParentCounter<Test>;
        type GasPayment = ();
        type RentPayment = ();
        type SignedClaimHandicap = SignedClaimHandicap;
        type TombstoneDeposit = TombstoneDeposit;
        type StorageSizeOffset = StorageSizeOffset;
        type RentByteFee = RentByteFee;
        type RentDepositOffset = RentDepositOffset;
        type SurchargeReward = SurchargeReward;
        type TransferFee = ContractTransferFee;
        type CreationFee = ContractCreationFee;
        type TransactionBaseFee = ContractTransactionBaseFee;
        type TransactionByteFee = ContractTransactionByteFee;
        type ContractFee = ContractFee;
        type CallBaseFee = CallBaseFee;
        type InstantiateBaseFee = InstantiateBaseFee;
        type MaxDepth = MaxDepth;
        type MaxValueSize = MaxValueSize;
        type BlockGasLimit = BlockGasLimit;
    }

    type Identity = identity::Module<Test>;
    type GeneralTM = Module<Test>;
    type Balances = balances::Module<Test>;
    type Asset = asset::Module<Test>;
    type Timestamp = pallet_timestamp::Module<Test>;
    type Randomness = pallet_randomness_collective_flip::Module<Test>;
    type Contracts = pallet_contracts::Module<Test>;

    /// Build a genesis identity instance owned by the specified account
    fn build() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        identity::GenesisConfig::<Test> {
            owner: AccountKeyring::Alice.public().into(),
            did_creation_fee: 250,
            ..Default::default()
        }
        .assimilate_storage(&mut t)
        .unwrap();
        asset::GenesisConfig::<Test> {
            asset_creation_fee: 0,
            ticker_registration_fee: 0,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(10000),
            },
            fee_collector: AccountKeyring::Dave.public().into(),
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }

    fn make_account(
        account_id: &AccountId,
    ) -> Result<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        Balances::make_free_balance_be(&account_id, 1_000_000);
        let _ = Identity::register_did(signed_id.clone(), vec![]);
        let did = Identity::get_identity(&AccountKey::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn should_add_and_verify_asset_rule() {
        build().execute_with(|| {
            let token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01].into(),
                owner_did: token_owner_did.clone(),
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
                ..Default::default()
            };
            let ticker = Ticker::from(token.name.0.as_slice());
            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_signed.clone(),
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
                None
            ));
            let claim_issuer_acc = AccountId::from(AccountKeyring::Bob);
            Balances::make_free_balance_be(&claim_issuer_acc, 1_000_000);
            let (claim_issuer_signed, claim_issuer_did) =
                make_account(&claim_issuer_acc.clone()).unwrap();

            assert_ok!(Identity::add_claim(
                claim_issuer_signed.clone(),
                token_owner_did,
                Claim::NoData,
                None,
            ));

            let now = Utc::now();
            <pallet_timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let sender_rule = Rule {
                issuers: vec![claim_issuer_did],
                rule_type: RuleType::IsPresent(Claim::NoData),
            };

            let receiver_rule1 = Rule {
                issuers: vec![claim_issuer_did],
                rule_type: RuleType::IsAbsent(Claim::CustomerDueDiligence),
            };

            let receiver_rule2 = Rule {
                issuers: vec![claim_issuer_did],
                rule_type: RuleType::IsPresent(Claim::Accredited(token_owner_did)),
            };

            let x = vec![sender_rule];
            let y = vec![receiver_rule1, receiver_rule2];

            assert_ok!(GeneralTM::add_active_rule(
                token_owner_signed.clone(),
                ticker,
                x,
                y
            ));

            assert_eq!(GeneralTM::asset_rules(ticker).rules.len(), 1);
            assert_eq!(GeneralTM::asset_rules(ticker).rules[0].rule_id, 1);

            assert_ok!(Identity::add_claim(
                claim_issuer_signed.clone(),
                token_owner_did,
                Claim::Accredited(claim_issuer_did),
                None,
            ));

            //Transfer tokens to investor
            assert_err!(
                Asset::transfer(
                    token_owner_signed.clone(),
                    ticker,
                    token_owner_did.clone(),
                    token.total_supply
                ),
                AssetError::<Test>::InvalidTransfer
            );

            assert_ok!(Identity::add_claim(
                claim_issuer_signed.clone(),
                token_owner_did,
                Claim::Accredited(token_owner_did),
                None,
            ));

            assert_ok!(Asset::transfer(
                token_owner_signed.clone(),
                ticker,
                token_owner_did.clone(),
                token.total_supply
            ));

            assert_ok!(Identity::add_claim(
                claim_issuer_signed.clone(),
                token_owner_did,
                Claim::CustomerDueDiligence,
                None,
            ));

            assert_err!(
                Asset::transfer(
                    token_owner_signed.clone(),
                    ticker,
                    token_owner_did.clone(),
                    token.total_supply
                ),
                AssetError::<Test>::InvalidTransfer
            );
        });
    }

    #[test]
    fn should_reset_asset_rules() {
        build().execute_with(|| {
            let token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01].into(),
                owner_did: token_owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
                ..Default::default()
            };
            let ticker = Ticker::from(token.name.0.as_slice());
            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_signed.clone(),
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
                None
            ));

            assert_ok!(GeneralTM::add_active_rule(
                token_owner_signed.clone(),
                ticker,
                vec![],
                vec![]
            ));

            let asset_rules = GeneralTM::asset_rules(ticker);
            assert_eq!(asset_rules.rules.len(), 1);

            assert_ok!(GeneralTM::reset_active_rules(
                token_owner_signed.clone(),
                ticker
            ));

            let asset_rules_new = GeneralTM::asset_rules(ticker);
            assert_eq!(asset_rules_new.rules.len(), 0);
        });
    }

    #[test]
    fn pause_resume_asset_rules() {
        build().execute_with(pause_resume_asset_rules_we);
    }

    fn pause_resume_asset_rules_we() {
        // 0. Create accounts
        let token_owner_acc = AccountId::from(AccountKeyring::Alice);
        let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();
        let receiver_acc = AccountId::from(AccountKeyring::Charlie);
        let (receiver_signed, receiver_did) = make_account(&receiver_acc.clone()).unwrap();

        Balances::make_free_balance_be(&receiver_acc, 1_000_000);

        // 1. A token representing 1M shares
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did: token_owner_did.clone(),
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::from(token.name.0.as_slice());
        Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

        // 2. Share issuance is successful
        assert_ok!(Asset::create_token(
            token_owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None
        ));

        assert_ok!(Identity::add_claim(
            receiver_signed.clone(),
            receiver_did.clone(),
            Claim::NoData,
            Some(99999999999999999u64),
        ));

        let now = Utc::now();
        <pallet_timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

        // 4. Define rules
        let receiver_rules = vec![Rule {
            issuers: vec![receiver_did],
            rule_type: RuleType::IsAbsent(Claim::NoData),
        }];

        assert_ok!(GeneralTM::add_active_rule(
            token_owner_signed.clone(),
            ticker,
            vec![],
            receiver_rules
        ));

        // 5. Verify pause/resume mechanism.
        // 5.1. Transfer should be cancelled.
        assert_err!(
            Asset::transfer(token_owner_signed.clone(), ticker, receiver_did, 10),
            AssetError::<Test>::InvalidTransfer
        );

        // 5.2. Pause asset rules, and run the transaction.
        assert_ok!(GeneralTM::pause_asset_rules(
            token_owner_signed.clone(),
            ticker
        ));
        assert_ok!(Asset::transfer(
            token_owner_signed.clone(),
            ticker,
            receiver_did,
            10
        ));

        // 5.3. Resume asset rules, and new transfer should fail again.
        assert_ok!(GeneralTM::resume_asset_rules(
            token_owner_signed.clone(),
            ticker
        ));
        assert_err!(
            Asset::transfer(token_owner_signed.clone(), ticker, receiver_did, 10),
            AssetError::<Test>::InvalidTransfer
        );
    }

    #[test]
    fn should_successfully_add_and_use_default_issuers() {
        build().execute_with(|| {
            // 0. Create accounts
            let token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();
            let trusted_issuer_acc = AccountId::from(AccountKeyring::Charlie);
            let (trusted_issuer_signed, trusted_issuer_did) =
                make_account(&trusted_issuer_acc.clone()).unwrap();
            let receiver_acc = AccountId::from(AccountKeyring::Dave);
            let (_, receiver_did) = make_account(&receiver_acc.clone()).unwrap();

            Balances::make_free_balance_be(&trusted_issuer_acc, 1_000_000);
            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);
            Balances::make_free_balance_be(&receiver_acc, 1_000_000);

            // 1. A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01].into(),
                owner_did: token_owner_did.clone(),
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
                ..Default::default()
            };
            let ticker = Ticker::from(token.name.0.as_slice());
            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

            // 2. Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_signed.clone(),
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
                None
            ));

            // Failed because trusted issuer identity not exist
            assert_err!(
                GeneralTM::add_default_trusted_claim_issuer(
                    token_owner_signed.clone(),
                    ticker,
                    IdentityId::from(1)
                ),
                Error::<Test>::DidNotExist
            );

            assert_ok!(GeneralTM::add_default_trusted_claim_issuer(
                token_owner_signed.clone(),
                ticker,
                trusted_issuer_did
            ));

            assert_eq!(GeneralTM::trusted_claim_issuer(ticker).len(), 1);
            assert_eq!(
                GeneralTM::trusted_claim_issuer(ticker),
                vec![trusted_issuer_did]
            );

            assert_ok!(Identity::add_claim(
                trusted_issuer_signed.clone(),
                receiver_did.clone(),
                Claim::CustomerDueDiligence,
                Some(99999999999999999u64),
            ));

            let now = Utc::now();
            <pallet_timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let sender_rule = Rule {
                issuers: vec![],
                rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
            };

            let receiver_rule = Rule {
                issuers: vec![],
                rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
            };

            let x = vec![sender_rule];
            let y = vec![receiver_rule];

            assert_ok!(GeneralTM::add_active_rule(
                token_owner_signed.clone(),
                ticker,
                x,
                y
            ));

            // fail when token owner doesn't has the valid claim
            assert_err!(
                Asset::transfer(
                    token_owner_signed.clone(),
                    ticker,
                    receiver_did.clone(),
                    100
                ),
                AssetError::<Test>::InvalidTransfer
            );

            assert_ok!(Identity::add_claim(
                trusted_issuer_signed.clone(),
                token_owner_did.clone(),
                Claim::CustomerDueDiligence,
                Some(99999999999999999u64),
            ));
            assert_ok!(Asset::transfer(
                token_owner_signed.clone(),
                ticker,
                receiver_did.clone(),
                100
            ));
        });
    }

    #[test]
    fn should_modify_vector_of_trusted_issuer() {
        build().execute_with(|| {
            // 0. Create accounts
            let token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();
            let trusted_issuer_acc_1 = AccountId::from(AccountKeyring::Charlie);
            let (trusted_issuer_signed_1, trusted_issuer_did_1) =
                make_account(&trusted_issuer_acc_1.clone()).unwrap();
            let trusted_issuer_acc_2 = AccountId::from(AccountKeyring::Ferdie);
            let (trusted_issuer_signed_2, trusted_issuer_did_2) =
                make_account(&trusted_issuer_acc_2.clone()).unwrap();
            let receiver_acc = AccountId::from(AccountKeyring::Dave);
            let (receiver_signed, receiver_did) = make_account(&receiver_acc.clone()).unwrap();

            Balances::make_free_balance_be(&trusted_issuer_acc_1, 1_000_000);
            Balances::make_free_balance_be(&trusted_issuer_acc_2, 1_000_000);
            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);
            Balances::make_free_balance_be(&receiver_acc, 1_000_000);

            // 1. A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01].into(),
                owner_did: token_owner_did.clone(),
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
                ..Default::default()
            };
            let ticker = Ticker::from(token.name.0.as_slice());
            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

            // 2. Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_signed.clone(),
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
                None
            ));

            // Failed because caller is not the owner of the ticker
            assert_err!(
                GeneralTM::add_default_trusted_claim_issuers_batch(
                    receiver_signed.clone(),
                    ticker,
                    vec![trusted_issuer_did_1, trusted_issuer_did_2]
                ),
                Error::<Test>::Unauthorized
            );

            // Failed because trusted issuer identity not exist
            assert_err!(
                GeneralTM::add_default_trusted_claim_issuers_batch(
                    token_owner_signed.clone(),
                    ticker,
                    vec![IdentityId::from(1), IdentityId::from(2)]
                ),
                Error::<Test>::DidNotExist
            );

            // Failed because trusted issuers length < 0
            assert_err!(
                GeneralTM::add_default_trusted_claim_issuers_batch(
                    token_owner_signed.clone(),
                    ticker,
                    vec![]
                ),
                Error::<Test>::InvalidLength
            );

            assert_ok!(GeneralTM::add_default_trusted_claim_issuers_batch(
                token_owner_signed.clone(),
                ticker,
                vec![trusted_issuer_did_1, trusted_issuer_did_2]
            ));

            assert_eq!(GeneralTM::trusted_claim_issuer(ticker).len(), 2);
            assert_eq!(
                GeneralTM::trusted_claim_issuer(ticker),
                vec![trusted_issuer_did_1, trusted_issuer_did_2]
            );

            // adding claim by trusted issuer 1
            assert_ok!(Identity::add_claim(
                trusted_issuer_signed_1.clone(),
                receiver_did.clone(),
                Claim::CustomerDueDiligence,
                None,
            ));

            // adding claim by trusted issuer 1
            assert_ok!(Identity::add_claim(
                trusted_issuer_signed_1.clone(),
                receiver_did.clone(),
                Claim::NoData,
                None,
            ));

            // adding claim by trusted issuer 2
            assert_ok!(Identity::add_claim(
                trusted_issuer_signed_2.clone(),
                token_owner_did.clone(),
                Claim::CustomerDueDiligence,
                None,
            ));

            let now = Utc::now();
            <pallet_timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let sender_rule = Rule {
                issuers: vec![],
                rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
            };

            let receiver_rule_1 = Rule {
                issuers: vec![],
                rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
            };

            let receiver_rule_2 = Rule {
                issuers: vec![],
                rule_type: RuleType::IsPresent(Claim::NoData),
            };

            let x = vec![sender_rule.clone()];
            let y = vec![receiver_rule_1, receiver_rule_2];

            assert_ok!(GeneralTM::add_active_rule(
                token_owner_signed.clone(),
                ticker,
                x,
                y
            ));

            assert_ok!(Asset::transfer(
                token_owner_signed.clone(),
                ticker,
                receiver_did.clone(),
                100
            ));

            // Remove the trusted issuer 1 from the list
            assert_ok!(GeneralTM::remove_default_trusted_claim_issuers_batch(
                token_owner_signed.clone(),
                ticker,
                vec![trusted_issuer_did_1]
            ));

            assert_eq!(GeneralTM::trusted_claim_issuer(ticker).len(), 1);
            assert_eq!(
                GeneralTM::trusted_claim_issuer(ticker),
                vec![trusted_issuer_did_2]
            );

            // Transfer should fail as issuer doesn't exist anymore but the rule data still exist
            assert_err!(
                Asset::transfer(
                    token_owner_signed.clone(),
                    ticker,
                    receiver_did.clone(),
                    500
                ),
                AssetError::<Test>::InvalidTransfer
            );

            // Change the asset rule to all the transfer happen again

            let receiver_rule_1 = Rule {
                issuers: vec![trusted_issuer_did_1],
                rule_type: RuleType::IsPresent(Claim::CustomerDueDiligence),
            };

            let receiver_rule_2 = Rule {
                issuers: vec![trusted_issuer_did_1],
                rule_type: RuleType::IsPresent(Claim::NoData),
            };

            let x = vec![sender_rule];
            let y = vec![receiver_rule_1, receiver_rule_2];

            let asset_rule = AssetTransferRule {
                sender_rules: x.clone(),
                receiver_rules: y.clone(),
                rule_id: 1,
            };

            // Failed because sender is not the owner of the ticker
            assert_err!(
                GeneralTM::change_asset_rule(receiver_signed.clone(), ticker, asset_rule.clone()),
                Error::<Test>::Unauthorized
            );

            let asset_rule_failure = AssetTransferRule {
                sender_rules: x,
                receiver_rules: y,
                rule_id: 5,
            };

            // Failed because passed rule id is not valid
            assert_err!(
                GeneralTM::change_asset_rule(
                    token_owner_signed.clone(),
                    ticker,
                    asset_rule_failure.clone()
                ),
                Error::<Test>::InvalidRuleId
            );

            // Should successfully change the asset rule
            assert_ok!(GeneralTM::change_asset_rule(
                token_owner_signed.clone(),
                ticker,
                asset_rule
            ));

            // Now the transfer should pass
            assert_ok!(Asset::transfer(
                token_owner_signed.clone(),
                ticker,
                receiver_did.clone(),
                500
            ));
        });
    }

    #[test]
    fn jurisdiction_asset_rules() {
        build().execute_with(jurisdiction_asset_rules_we);
    }
    fn jurisdiction_asset_rules_we() {
        // 0. Create accounts
        let token_owner_acc = AccountId::from(AccountKeyring::Alice);
        let (token_owner_signed, token_owner_id) = make_account(&token_owner_acc).unwrap();
        Balances::make_free_balance_be(&token_owner_acc, 1_000_000);
        let cdd_acc = AccountId::from(AccountKeyring::Bob);
        let (cdd_signed, cdd_id) = make_account(&cdd_acc).unwrap();
        let user_acc = AccountId::from(AccountKeyring::Charlie);
        let (_, user_id) = make_account(&user_acc).unwrap();
        // 1. Create a token.
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did: token_owner_id.clone(),
            total_supply: 1_000_000,
            divisible: true,
            ..Default::default()
        };
        let ticker = Ticker::from(token.name.0.as_slice());
        assert_ok!(Asset::create_token(
            token_owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None
        ));
        // 2. Set up rules for Asset transfer.
        let scope = Scope::from(0);
        let receiver_rules = vec![
            Rule {
                rule_type: RuleType::IsAnyOf(vec![
                    Claim::Jurisdiction(b"Canada".into(), scope),
                    Claim::Jurisdiction(b"Spain".into(), scope),
                ]),
                issuers: vec![cdd_id],
            },
            Rule {
                rule_type: RuleType::IsAbsent(Claim::BlackListed(scope)),
                issuers: vec![token_owner_id],
            },
        ];
        assert_ok!(GeneralTM::add_active_rule(
            token_owner_signed.clone(),
            ticker,
            vec![],
            receiver_rules
        ));
        // 3. Validate behaviour.
        // 3.1. Invalid transfer because missing jurisdiction.
        assert_err!(
            Asset::transfer(token_owner_signed.clone(), ticker, user_id, 100),
            AssetError::<Test>::InvalidTransfer
        );
        // 3.2. Add jurisdiction and transfer will be OK.
        assert_ok!(Identity::add_claim(
            cdd_signed.clone(),
            user_id,
            Claim::Jurisdiction(b"Canada".into(), scope),
            None
        ));
        assert_ok!(Asset::transfer(
            token_owner_signed.clone(),
            ticker,
            user_id,
            100
        ));
        // 3.3. Add user to blacklist
        assert_ok!(Identity::add_claim(
            token_owner_signed.clone(),
            user_id,
            Claim::BlackListed(scope),
            None,
        ));
        assert_err!(
            Asset::transfer(token_owner_signed.clone(), ticker, user_id, 100),
            AssetError::<Test>::InvalidTransfer
        );
    }
    #[test]
    fn scope_asset_rules() {
        build().execute_with(scope_asset_rules_we);
    }
    fn scope_asset_rules_we() {
        // 0. Create accounts
        let token_owner_acc = AccountId::from(AccountKeyring::Alice);
        let (token_owner_signed, token_owner_id) = make_account(&token_owner_acc).unwrap();
        Balances::make_free_balance_be(&token_owner_acc, 1_000_000);
        let cdd_acc = AccountId::from(AccountKeyring::Bob);
        let (cdd_signed, cdd_id) = make_account(&cdd_acc).unwrap();
        let user_acc = AccountId::from(AccountKeyring::Charlie);
        let (_, user_id) = make_account(&user_acc).unwrap();
        // 1. Create a token.
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did: token_owner_id.clone(),
            total_supply: 1_000_000,
            divisible: true,
            ..Default::default()
        };
        let ticker = Ticker::from(token.name.0.as_slice());
        assert_ok!(Asset::create_token(
            token_owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None
        ));
        // 2. Set up rules for Asset transfer.
        let scope = Identity::get_token_did(&ticker).unwrap();
        let receiver_rules = vec![Rule {
            rule_type: RuleType::IsPresent(Claim::Affiliate(scope)),
            issuers: vec![cdd_id],
        }];
        assert_ok!(GeneralTM::add_active_rule(
            token_owner_signed.clone(),
            ticker,
            vec![],
            receiver_rules
        ));
        // 3. Validate behaviour.
        // 3.1. Invalid transfer because missing jurisdiction.
        assert_err!(
            Asset::transfer(token_owner_signed.clone(), ticker, user_id, 100),
            AssetError::<Test>::InvalidTransfer
        );
        // 3.2. Add jurisdiction and transfer will be OK.
        assert_ok!(Identity::add_claim(
            cdd_signed.clone(),
            user_id,
            Claim::Affiliate(scope),
            None
        ));
        assert_ok!(Asset::transfer(
            token_owner_signed.clone(),
            ticker,
            user_id,
            100
        ));
    }
}
