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
//! The Compliance Manager module provides functionality to set and evaluate a list of conditions.
//! Those conditions define transfer restrictions for the sender and receiver. For instance, you can limit your asset to investors
//! of specific jurisdictions.
//!
//!
//! ## Overview
//!
//! The Compliance Manager module provides functions for:
//!
//! - Adding conditions for allowing transfers.
//! - Removing conditions that allow transfers.
//! - Resetting all conditions.
//!
//! ### Use case
//!
//! This module is very versatile and offers infinite possibilities.
//! The conditions can dictate various requirements like:
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
//! - **AssetCompliance:** It is an array of compliance requirements that are currently enforced for a ticker.
//! - **ComplianceRequirement:** Every compliance requirement contains an array for sender conditions and an array for receiver conditions
//! - **sender conditions:** These are conditions that the sender of security tokens must follow
//! - **receiver conditions:** These are conditions that the receiver of security tokens must follow
//! - **Valid transfer:** For a transfer to be valid,
//!     All receiver and sender conditions of any of the asset compliance must be followed.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - [add_compliance_requirement](Module::add_compliance_requirement) - Adds a new compliance requirement to an asset's compliance.
//! - [remove_compliance_requirement](Module::remove_compliance_requirement) - Removes a compliance requirement from an asset's compliance.
//! - [reset_asset_compliance](Module::reset_asset_compliance) - Resets(remove) an asset's compliance.
//! - [pause_asset_compliance](Module::pause_asset_compliance) - Pauses the evaluation of asset compliance for a ticker before executing a
//! transaction.
//! - [add_default_trusted_claim_issuer](Module::add_default_trusted_claim_issuer) - Adds a default
//!  trusted claim issuer for a given asset.
//! - [remove_default_trusted_claim_issuer](Module::remove_default_trusted_claim_issuer) - Removes
//!  the default claim issuer.
//! - [change_compliance_requirement](Module::change_compliance_requirement) - Updates a compliance requirement, based on its id.
//! based on its id for a given asset.
//!
//! ### Public Functions
//!
//! - [verify_restriction](Module::verify_restriction) - Checks if a transfer is a valid transfer and returns the result

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

use alloc::borrow::Cow;
use codec::{Decode, Encode};
use core::result::Result;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    weights::Weight,
};
use frame_system::ensure_signed;
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
    proposition, Claim, ClaimType, Condition, ConditionType, IdentityId, Scope, Ticker,
    TrustedIssuer,
};
use polymesh_primitives_derive::Migrate;

#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{
    cmp::max,
    convert::{From, TryFrom},
    prelude::*,
};

type CallPermissions<T> = pallet_permissions::Module<T>;

/// The module's configuration trait.
pub trait Trait:
    pallet_timestamp::Trait + frame_system::Trait + BalancesTrait + IdentityTrait
{
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    /// Asset module
    type Asset: AssetTrait<Self::Balance, Self::AccountId>;

    /// The maximum claim reads that are allowed to happen in worst case of a condition resolution
    type MaxConditionComplexity: Get<u32>;
}

use polymesh_primitives::condition::ConditionOld;

/// A compliance requirement.
/// All sender and receiver conditions of the same compliance requirement must be true in order to execute the transfer.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, Migrate)]
#[migrate_context(Option<polymesh_primitives::CddId>)]
pub struct ComplianceRequirement {
    #[migrate(Condition)]
    pub sender_conditions: Vec<Condition>,
    #[migrate(Condition)]
    pub receiver_conditions: Vec<Condition>,
    /// Unique identifier of the compliance requirement
    pub id: u32,
}

impl ComplianceRequirement {
    /// Dedup `ClaimType`s in `TrustedFor::Specific`.
    fn dedup(&mut self) {
        let dedup_condition = |conds: &mut [Condition]| {
            conds
                .iter_mut()
                .flat_map(|c| &mut c.issuers)
                .for_each(|issuer| issuer.dedup())
        };
        dedup_condition(&mut self.sender_conditions);
        dedup_condition(&mut self.receiver_conditions);
    }
}

/// A compliance requirement along with its evaluation result
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct ComplianceRequirementResult {
    pub sender_conditions: Vec<ConditionResult>,
    pub receiver_conditions: Vec<ConditionResult>,
    /// Unique identifier of the compliance requirement.
    pub id: u32,
    /// Result of this transfer condition's evaluation.
    pub result: bool,
}

impl From<ComplianceRequirement> for ComplianceRequirementResult {
    fn from(requirement: ComplianceRequirement) -> Self {
        Self {
            sender_conditions: requirement
                .sender_conditions
                .iter()
                .map(|condition| ConditionResult::from(condition.clone()))
                .collect(),
            receiver_conditions: requirement
                .receiver_conditions
                .iter()
                .map(|condition| ConditionResult::from(condition.clone()))
                .collect(),
            id: requirement.id,
            result: true,
        }
    }
}

/// An individual condition along with its evaluation result
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct ConditionResult {
    // Condition being evaluated
    pub condition: Condition,
    // Result of evaluation
    pub result: bool,
}

impl From<Condition> for ConditionResult {
    fn from(condition: Condition) -> Self {
        Self {
            condition,
            result: true,
        }
    }
}

/// List of compliance requirements associated to an asset.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Migrate)]
#[migrate_context(Option<polymesh_primitives::CddId>)]
pub struct AssetCompliance {
    /// This flag indicates if asset compliance should be enforced
    pub paused: bool,
    /// List of compliance requirements.
    #[migrate(ComplianceRequirement)]
    pub requirements: Vec<ComplianceRequirement>,
}

/// Implicit requirement result.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct ImplicitRequirementResult {
    /// Result of implicit condition for the sender of the extrinsic.
    pub from_result: bool,
    /// Result of implicit condition for the receiver of the extrinsic.
    pub to_result: bool,
}

type Identity<T> = identity::Module<T>;

/// Asset compliance and it's evaluation result
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct AssetComplianceResult {
    /// This flag indicates if asset compliance should be enforced
    pub paused: bool,
    /// List of compliance requirements.
    pub requirements: Vec<ComplianceRequirementResult>,
    /// It is treated differently from other compliance requirements because
    /// it doesn't successfully execute the transaction even if it succeed. As txn
    /// also depends on the successful verification of one of the `requirement` set by
    /// the asset issuer. But it can fail the txn if it gets failed independently from
    /// the result of other requirements.
    ///
    /// Implicit requirements result.
    pub implicit_requirements_result: ImplicitRequirementResult,
    // Final evaluation result of the asset compliance
    pub result: bool,
}

impl From<AssetCompliance> for AssetComplianceResult {
    fn from(asset_compliance: AssetCompliance) -> Self {
        Self {
            paused: asset_compliance.paused,
            requirements: asset_compliance
                .requirements
                .into_iter()
                .map(ComplianceRequirementResult::from)
                .collect(),
            implicit_requirements_result: ImplicitRequirementResult::default(),
            result: false,
        }
    }
}

pub mod weight_for {
    use super::*;

    pub fn weight_for_verify_restriction<T: Trait>(no_of_compliance_requirements: u64) -> Weight {
        no_of_compliance_requirements * 100_000_000
    }

    pub fn weight_for_reading_asset_compliance<T: Trait>() -> Weight {
        T::DbWeight::get().reads(1) + 1_000_000
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as ComplianceManager {
        /// Asset compliance for a ticker (Ticker -> AssetCompliance)
        pub AssetCompliances get(fn asset_compliance): map hasher(blake2_128_concat) Ticker => AssetCompliance;
        /// List of trusted claim issuer Ticker -> Issuer Identity
        pub TrustedClaimIssuer get(fn trusted_claim_issuer): map hasher(blake2_128_concat) Ticker => Vec<TrustedIssuer>;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The sender must be a secondary key for the DID.
        SenderMustBeSecondaryKeyForDid,
        /// User is not authorized.
        Unauthorized,
        /// Did not exist
        DidNotExist,
        /// When parameter has length < 1
        InvalidLength,
        /// Compliance requirement id doesn't exist
        InvalidComplianceRequirementId,
        /// Issuer exist but trying to add it again
        IncorrectOperationOnTrustedIssuer,
        /// Missing current DID
        MissingCurrentIdentity,
        /// There are duplicate compliance requirements.
        DuplicateComplianceRequirements,
        /// The worst case scenario of the compliance requirement is too complex
        ComplianceRequirementTooComplex,
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            use polymesh_primitives::migrate::{Empty, migrate_map, migrate_map_rename};
            use polymesh_primitives::condition::TrustedIssuerOld;

            migrate_map_rename::<AssetComplianceOld, _>(b"ComplianceManager", b"AssetRulesMap", b"AssetCompliance", |_| None);

            migrate_map::<Vec<TrustedIssuerOld>, _>(b"ComplianceManager", b"TrustedClaimIsuer", |_| Empty);

            1_000
        }

        /// Adds a compliance requirement to an asset's compliance by ticker.
        /// If the compliance requirement is a duplicate, it does nothing.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        /// * sender_conditions - Sender transfer conditions.
        /// * receiver_conditions - Receiver transfer conditions.
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 600_000_000 + 1_000_000 * u64::try_from(max(sender_conditions.len(), receiver_conditions.len())).unwrap_or_default()]
        pub fn add_compliance_requirement(origin, ticker: Ticker, sender_conditions: Vec<Condition>, receiver_conditions: Vec<Condition>) {
            let did = Self::ensure_can_modify_rules(origin, ticker)?;

            <<T as IdentityTrait>::ProtocolFee>::charge_fee(
                ProtocolOp::ComplianceManagerAddComplianceRequirement
            )?;
            let id = Self::get_latest_requirement_id(ticker) + 1u32;
            let mut new_requirement = ComplianceRequirement { sender_conditions, receiver_conditions, id };
            new_requirement.dedup();

            let mut asset_compliance = AssetCompliances::get(ticker);
            let reqs = &mut asset_compliance.requirements;

            if !reqs
                .iter()
                .any(|requirement| requirement.sender_conditions == new_requirement.sender_conditions && requirement.receiver_conditions == new_requirement.receiver_conditions)
            {
                reqs.push(new_requirement.clone());
                Self::verify_compliance_complexity(&reqs, ticker, 0)?;
                AssetCompliances::insert(&ticker, asset_compliance);
                Self::deposit_event(Event::ComplianceRequirementCreated(did, ticker, new_requirement));
            }
        }

        /// Removes a compliance requirement from an asset's compliance.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        /// * id - Compliance requirement id which is need to be removed
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 200_000_000]
        pub fn remove_compliance_requirement(origin, ticker: Ticker, id: u32) {
            let did = Self::ensure_can_modify_rules(origin, ticker)?;

            AssetCompliances::try_mutate(ticker, |AssetCompliance { requirements, .. }| {
                let before = requirements.len();
                requirements.retain(|requirement| requirement.id != id);
                ensure!(before != requirements.len(), Error::<T>::InvalidComplianceRequirementId);
                Ok(()) as DispatchResult
            })?;

            Self::deposit_event(Event::ComplianceRequirementRemoved(did, ticker, id));
        }

        /// Replaces an asset's compliance by ticker with a new compliance.
        ///
        /// # Arguments
        /// * `ticker` - the asset ticker,
        /// * `asset_compliance - the new asset compliance.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the ticker.
        /// * `DuplicateAssetCompliance` if `asset_compliance` contains multiple entries with the same `requirement_id`.
        ///
        /// # Weight
        /// `read_and_write_weight + 100_000_000 + 500_000 * asset_compliance.len()`
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 400_000_000 + 500_000 * u64::try_from(asset_compliance.len()).unwrap_or_default()]
        pub fn replace_asset_compliance(origin, ticker: Ticker, asset_compliance: Vec<ComplianceRequirement>) {
            let did = Self::ensure_can_modify_rules(origin, ticker)?;

            // Ensure there are no duplicate requirement ids.
            let mut asset_compliance = asset_compliance;
            let start_len = asset_compliance.len();
            asset_compliance.dedup_by_key(|r| r.id);
            ensure!(start_len == asset_compliance.len(), Error::<T>::DuplicateComplianceRequirements);

            // Dedup `ClaimType`s in `TrustedFor::Specific`.
            asset_compliance.iter_mut().for_each(|r| r.dedup());

            Self::verify_compliance_complexity(&asset_compliance, ticker, 0)?;
            AssetCompliances::mutate(&ticker, |old| old.requirements = asset_compliance.clone());
            Self::deposit_event(Event::AssetComplianceReplaced(did, ticker, asset_compliance));
        }

        /// Removes an asset's compliance
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 100_000_000]
        pub fn reset_asset_compliance(origin, ticker: Ticker) {
            let did = Self::ensure_can_modify_rules(origin, ticker)?;
            AssetCompliances::remove(ticker);
            Self::deposit_event(Event::AssetComplianceReset(did, ticker));
        }

        /// Pauses the verification of conditions for `ticker` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 100_000_000]
        pub fn pause_asset_compliance(origin, ticker: Ticker) {
            let did = Self::pause_resume_asset_compliance(origin, ticker, true)?;
            Self::deposit_event(Event::AssetCompliancePaused(did, ticker));
        }

        /// Resumes the verification of conditions for `ticker` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 100_000_000]
        pub fn resume_asset_compliance(origin, ticker: Ticker) {
            let did = Self::pause_resume_asset_compliance(origin, ticker, false)?;
            Self::deposit_event(Event::AssetComplianceResumed(did, ticker));
        }

        /// Adds another default trusted claim issuer at the ticker level.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * issuer - IdentityId of the trusted claim issuer.
        #[weight = T::DbWeight::get().reads_writes(3, 1) + 300_000_000]
        pub fn add_default_trusted_claim_issuer(origin, ticker: Ticker, issuer: TrustedIssuer) {
            let did = Self::ensure_can_modify_rules(origin, ticker)?;
            ensure!(<Identity<T>>::is_identity_exists(&issuer.issuer), Error::<T>::DidNotExist);
            TrustedClaimIssuer::try_mutate(ticker, |issuers| {
                ensure!(!issuers.contains(&issuer), Error::<T>::IncorrectOperationOnTrustedIssuer);
                Self::base_verify_compliance_complexity(&AssetCompliances::get(ticker).requirements, issuers.len() + 1)?;
                issuers.push(issuer.clone());
                Ok(()) as DispatchResult
            })?;
            Self::deposit_event(Event::TrustedDefaultClaimIssuerAdded(did, ticker, issuer));
        }

        /// Removes the given `issuer` from the set of default trusted claim issuers at the ticker level.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * issuer - IdentityId of the trusted claim issuer.
        #[weight = T::DbWeight::get().reads_writes(3, 1) + 300_000_000]
        pub fn remove_default_trusted_claim_issuer(origin, ticker: Ticker, issuer: TrustedIssuer) {
            let did = Self::ensure_can_modify_rules(origin, ticker)?;
            TrustedClaimIssuer::try_mutate(ticker, |issuers| {
                let len = issuers.len();
                issuers.retain(|ti| ti != &issuer);
                ensure!(len != issuers.len(), Error::<T>::IncorrectOperationOnTrustedIssuer);
                Ok(()) as DispatchResult
            })?;
            Self::deposit_event(Event::TrustedDefaultClaimIssuerRemoved(did, ticker, issuer));
        }

        /// Modify an existing compliance requirement of a given ticker.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * new_req - Compliance requirement.
        #[weight = T::DbWeight::get().reads_writes(2, 1) + 720_000_000]
        pub fn change_compliance_requirement(origin, ticker: Ticker, new_req: ComplianceRequirement) {
            let did = Self::ensure_can_modify_rules(origin, ticker)?;
            ensure!(Self::get_latest_requirement_id(ticker) >= new_req.id, Error::<T>::InvalidComplianceRequirementId);

            let mut asset_compliance = AssetCompliances::get(ticker);
            let reqs = &mut asset_compliance.requirements;
            if let Some(req) = reqs.iter_mut().find(|req| req.id == new_req.id) {
                let mut new_req = new_req;
                new_req.dedup();

                *req = new_req.clone();
                Self::verify_compliance_complexity(&reqs, ticker, 0)?;
                AssetCompliances::insert(&ticker, asset_compliance);
                Self::deposit_event(Event::ComplianceRequirementChanged(did, ticker, new_req));
            }
        }
    }
}

decl_event!(
    pub enum Event {
        /// Emitted when new compliance requirement is created.
        /// (caller DID, Ticker, ComplianceRequirement).
        ComplianceRequirementCreated(IdentityId, Ticker, ComplianceRequirement),
        /// Emitted when a compliance requirement is removed.
        /// (caller DID, Ticker, requirement_id).
        ComplianceRequirementRemoved(IdentityId, Ticker, u32),
        /// Emitted when an asset compliance is replaced.
        /// Parameters: caller DID, ticker, new asset compliance.
        AssetComplianceReplaced(IdentityId, Ticker, Vec<ComplianceRequirement>),
        /// Emitted when an asset compliance of a ticker is reset.
        /// (caller DID, Ticker).
        AssetComplianceReset(IdentityId, Ticker),
        /// Emitted when an asset compliance for a given ticker gets resume.
        /// (caller DID, Ticker).
        AssetComplianceResumed(IdentityId, Ticker),
        /// Emitted when an asset compliance for a given ticker gets paused.
        /// (caller DID, Ticker).
        AssetCompliancePaused(IdentityId, Ticker),
        /// Emitted when compliance requirement get modified/change.
        /// (caller DID, Ticker, ComplianceRequirement).
        ComplianceRequirementChanged(IdentityId, Ticker, ComplianceRequirement),
        /// Emitted when default claim issuer list for a given ticker gets added.
        /// (caller DID, Ticker, Added TrustedIssuer).
        TrustedDefaultClaimIssuerAdded(IdentityId, Ticker, TrustedIssuer),
        /// Emitted when default claim issuer list for a given ticker get removed.
        /// (caller DID, Ticker, Removed TrustedIssuer).
        TrustedDefaultClaimIssuerRemoved(IdentityId, Ticker, TrustedIssuer),
    }
);

impl<T: Trait> Module<T> {
    /// Ensure that `origin` can modify `ticker`'s compliance rules.
    fn ensure_can_modify_rules(
        origin: T::Origin,
        ticker: Ticker,
    ) -> Result<IdentityId, DispatchError> {
        let sender = ensure_signed(origin)?;
        CallPermissions::<T>::ensure_call_permissions(&sender)?;
        let did = Context::current_identity_or::<Identity<T>>(&sender)?;
        ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
        Ok(did)
    }

    /// Returns true if `sender_did` is the owner of `ticker` asset.
    fn is_owner(ticker: &Ticker, sender_did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, sender_did)
    }

    /// Fetches all claims of `target` identity with type
    /// and scope from `claim` and generated by any of `issuers`.
    fn fetch_claims<'a>(
        target: IdentityId,
        claim: &'a Claim,
        issuers: &'a [TrustedIssuer],
    ) -> impl 'a + Iterator<Item = Claim> {
        let claim_type = claim.claim_type();
        let scope = claim.as_scope();

        issuers
            .iter()
            .filter(move |issuer| issuer.is_trusted_for(claim_type))
            .filter_map(move |issuer| {
                Identity::<T>::fetch_claim(target, claim_type, issuer.issuer, scope.cloned())
                    .map(|id_claim| id_claim.claim)
            })
    }

    /// It fetches the `ConfidentialScopeClaim` of users `id` for the given ticker.
    /// Note that this vector could be 0 or 1 items.
    fn fetch_confidential_claims(id: IdentityId, ticker: &Ticker) -> impl Iterator<Item = Claim> {
        let claim_type = ClaimType::InvestorUniqueness;
        // NOTE: Ticker length is less by design that IdentityId.
        let asset_scope = Scope::from(*ticker);

        <identity::Module<T>>::fetch_claim(id, claim_type, id, Some(asset_scope))
            .into_iter()
            .map(|id_claim| id_claim.claim)
    }

    /// Fetches the trusted issuers for a certain `condition` if it defines one,
    /// or falls back to `ticker`'s default ones otherwise.
    fn fetch_issuers<'a>(ticker: &Ticker, condition: &'a Condition) -> Cow<'a, [TrustedIssuer]> {
        if condition.issuers.is_empty() {
            Cow::Owned(Self::trusted_claim_issuer(ticker))
        } else {
            Cow::Borrowed(&condition.issuers)
        }
    }

    /// Fetches the proposition context for target `id` and specific `condition`.
    /// The set of trusted issuers are taken from `issuers`.
    fn fetch_context<'a>(
        id: IdentityId,
        issuers: &'a [TrustedIssuer],
        condition: &'a Condition,
        primary_issuance_agent: Option<IdentityId>,
    ) -> proposition::Context<impl 'a + Iterator<Item = Claim>> {
        enum Iter<I1, I2, I3> {
            I1(I1),
            I2(I2),
            I3(I3),
            I4,
        }
        impl<I1, I2, I3> Iterator for Iter<I1, I2, I3>
        where
            I1: Iterator<Item = Claim>,
            I2: Iterator<Item = Claim>,
            I3: Iterator<Item = Claim>,
        {
            type Item = Claim;
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::I1(i) => i.next(),
                    Self::I2(i) => i.next(),
                    Self::I3(i) => i.next(),
                    Self::I4 => None,
                }
            }
        }

        let claims = match &condition.condition_type {
            ConditionType::IsPresent(claim) | ConditionType::IsAbsent(claim) => {
                Iter::I1(Self::fetch_claims(id, claim, issuers))
            }
            ConditionType::IsAnyOf(claims) | ConditionType::IsNoneOf(claims) => Iter::I2(
                claims
                    .iter()
                    .flat_map(move |claim| Self::fetch_claims(id, claim, issuers)),
            ),
            ConditionType::HasValidProofOfInvestor(proof_ticker) => {
                Iter::I3(Self::fetch_confidential_claims(id, proof_ticker))
            }
            ConditionType::IsIdentity(_) => Iter::I4,
        };

        proposition::Context {
            claims,
            id,
            primary_issuance_agent,
        }
    }

    /// Loads the context for each condition in `conditions` and verifies that all of them evaluate to `true`.
    fn are_all_conditions_satisfied(
        ticker: &Ticker,
        did: IdentityId,
        conditions: &[Condition],
        primary_issuance_agent: Option<IdentityId>,
    ) -> bool {
        conditions.iter().all(|condition| {
            Self::is_condition_satisfied(ticker, did, condition, primary_issuance_agent)
        })
    }

    /// Checks whether the given condition is satisfied or not.
    fn is_condition_satisfied(
        ticker: &Ticker,
        did: IdentityId,
        condition: &Condition,
        primary_issuance_agent: Option<IdentityId>,
    ) -> bool {
        let issuers = Self::fetch_issuers(ticker, condition);
        let context =
            Self::fetch_context(did, issuers.as_ref(), &condition, primary_issuance_agent);
        proposition::run(&condition, context)
    }

    /// Returns whether all conditions, in their proper context, hold when evaluated.
    /// As a side-effect, each condition will be updated with its result,
    /// implying strict (non-lazy) evaluation of the conditions.
    fn evaluate_conditions(
        ticker: &Ticker,
        did: IdentityId,
        conditions: &mut [ConditionResult],
        primary_issuance_agent: Option<IdentityId>,
    ) -> bool {
        conditions.iter_mut().fold(true, |result, condition| {
            let cond = &condition.condition;
            let issuers = Self::fetch_issuers(ticker, cond);
            let context = Self::fetch_context(did, issuers.as_ref(), cond, primary_issuance_agent);
            condition.result = proposition::run(cond, context);
            result & condition.result
        })
    }

    /// Pauses or resumes the asset compliance.
    fn pause_resume_asset_compliance(
        origin: T::Origin,
        ticker: Ticker,
        pause: bool,
    ) -> Result<IdentityId, DispatchError> {
        let did = Self::ensure_can_modify_rules(origin, ticker)?;
        AssetCompliances::mutate(&ticker, |compliance| compliance.paused = pause);
        Ok(did)
    }

    /// Compute the id of the last requirement in a `ticker`'s compliance rules.
    fn get_latest_requirement_id(ticker: Ticker) -> u32 {
        Self::asset_compliance(ticker)
            .requirements
            .last()
            .map(|r| r.id)
            .unwrap_or(0)
    }

    /// verifies all requirements and returns the result in an array of booleans.
    /// this does not care if the requirements are paused or not. It is meant to be
    /// called only in failure conditions
    pub fn granular_verify_restriction(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        primary_issuance_agent: Option<IdentityId>,
    ) -> AssetComplianceResult {
        let asset_compliance = Self::asset_compliance(ticker);

        let mut asset_compliance_with_results = AssetComplianceResult::from(asset_compliance);
        // It is to know the result of the scope claim (i.e investor does posses the valid `InvestorZKProof` claim or not).
        let from_has_scope_claim = Self::has_scope_claim(ticker, from_did_opt);
        let to_has_scope_claim = Self::has_scope_claim(ticker, to_did_opt);
        // Assigning the implicit requirement result.
        asset_compliance_with_results.implicit_requirements_result = ImplicitRequirementResult {
            from_result: from_has_scope_claim,
            to_result: to_has_scope_claim,
        };

        let implicit_result = from_has_scope_claim && to_has_scope_claim;

        for requirements in &mut asset_compliance_with_results.requirements {
            if let Some(from_did) = from_did_opt {
                // Evaluate all sender conditions
                if !Self::evaluate_conditions(
                    ticker,
                    from_did,
                    &mut requirements.sender_conditions,
                    primary_issuance_agent,
                ) {
                    // If the result of any of the sender conditions was false, set this requirements result to false.
                    requirements.result = false;
                }
            }
            if let Some(to_did) = to_did_opt {
                // Evaluate all receiver conditions
                if !Self::evaluate_conditions(
                    ticker,
                    to_did,
                    &mut requirements.receiver_conditions,
                    primary_issuance_agent,
                ) {
                    // If the result of any of the receiver conditions was false, set this requirements result to false.
                    requirements.result = false;
                }
            }
            // If the requirements result is positive, update the final result.
            if requirements.result {
                asset_compliance_with_results.result = implicit_result;
            }
        }
        asset_compliance_with_results
    }

    /// Verify that `asset_compliance`, with `add` number of default issuers to add,
    /// is within the maximum condition complexity allowed.
    fn verify_compliance_complexity(
        asset_compliance: &[ComplianceRequirement],
        ticker: Ticker,
        add: usize,
    ) -> DispatchResult {
        let count = TrustedClaimIssuer::decode_len(ticker)
            .unwrap_or_default()
            .saturating_add(add);
        Self::base_verify_compliance_complexity(asset_compliance, count)
    }

    /// Verify that `asset_compliance`, with `default_issuer_count` number of default issuers,
    /// is within the maximum condition complexity allowed.
    fn base_verify_compliance_complexity(
        asset_compliance: &[ComplianceRequirement],
        default_issuer_count: usize,
    ) -> DispatchResult {
        let complexity = asset_compliance
            .iter()
            .flat_map(|requirement| {
                requirement
                    .sender_conditions
                    .iter()
                    .chain(requirement.receiver_conditions.iter())
            })
            .fold(0usize, |complexity, condition| {
                let (claims, issuers) = condition.complexity();
                complexity.saturating_add(claims.saturating_mul(match issuers {
                    0 => default_issuer_count,
                    _ => issuers,
                }))
            });
        if let Ok(complexity_u32) = u32::try_from(complexity) {
            if complexity_u32 <= T::MaxConditionComplexity::get() {
                return Ok(());
            }
        }
        Err(Error::<T>::ComplianceRequirementTooComplex.into())
    }

    /// Helper function to know whether the given did has the valid scope claim or not.
    fn has_scope_claim(ticker: &Ticker, did_opt: Option<IdentityId>) -> bool {
        did_opt.map_or(false, |did| {
            // Generate the condition for `HasValidProofOfInvestor` condition type.
            let condition =
                &Condition::from_dids(ConditionType::HasValidProofOfInvestor(*ticker), &[did]);
            Self::is_condition_satisfied(ticker, did, condition, None)
        })
    }

    /// Know whether sender and receiver has valid scope claim or not before checking the transfer conditions.
    fn is_sender_and_receiver_has_valid_scope_claim(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
    ) -> bool {
        // Return the final boolean result.
        Self::has_scope_claim(ticker, to_did_opt) && Self::has_scope_claim(ticker, from_did_opt)
    }
}

impl<T: Trait> ComplianceManagerTrait<T::Balance> for Module<T> {
    ///  Sender restriction verification
    fn verify_restriction(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        _value: T::Balance,
        primary_issuance_agent: Option<IdentityId>,
    ) -> Result<(u8, Weight), DispatchError> {
        // Transfer is valid if ALL receiver AND sender conditions of ANY asset conditions are valid.
        let asset_compliance = Self::asset_compliance(ticker);
        if asset_compliance.paused {
            return Ok((
                ERC1400_TRANSFER_SUCCESS,
                weight_for::weight_for_reading_asset_compliance::<T>(),
            ));
        }

        // Check for whether sender & receiver has the scope claim or not if not then fail the txn.
        // To optimize we are not checking it again and again with the respective conditions, it
        // gets checked only once and if it is valid then its result is tied up with the conditions result.
        //
        // Note - Due to this check `ConditionType::HasValidProofOfInvestor` is an implicit transfer condition and it only
        // lookup for the claims those are provided by the user itself.
        let mut requirement_count: usize = 0;
        let verify_weight = |count| {
            weight_for::weight_for_verify_restriction::<T>(u64::try_from(count).unwrap_or(0))
        };

        if !Self::is_sender_and_receiver_has_valid_scope_claim(ticker, from_did_opt, to_did_opt) {
            return Ok((
                ERC1400_TRANSFER_FAILURE,
                weight_for::weight_for_reading_asset_compliance::<T>(),
            ));
        }
        for requirement in asset_compliance.requirements {
            if let Some(from_did) = from_did_opt {
                requirement_count += requirement.sender_conditions.len();
                if !Self::are_all_conditions_satisfied(
                    ticker,
                    from_did,
                    &requirement.sender_conditions,
                    primary_issuance_agent,
                ) {
                    // Skips checking receiver conditions because sender conditions are not satisfied.
                    continue;
                }
            }

            if let Some(to_did) = to_did_opt {
                requirement_count += requirement.receiver_conditions.len();
                if Self::are_all_conditions_satisfied(
                    ticker,
                    to_did,
                    &requirement.receiver_conditions,
                    primary_issuance_agent,
                ) {
                    // All conditions satisfied, return early
                    return Ok((ERC1400_TRANSFER_SUCCESS, verify_weight(requirement_count)));
                }
            }
        }
        Ok((ERC1400_TRANSFER_FAILURE, verify_weight(requirement_count)))
    }
}
