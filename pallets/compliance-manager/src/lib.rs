// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::{decl_error, decl_module, decl_storage, ensure};
use sp_std::{convert::From, prelude::*};

use pallet_base::ensure_length_ok;
use polymesh_common_utilities::protocol_fee::{ChargeProtocolFee, ProtocolOp};
pub use polymesh_common_utilities::traits::compliance_manager::{
    ComplianceFnConfig, Config, Event, WeightInfo,
};
use polymesh_primitives::compliance_manager::{
    AssetCompliance, AssetComplianceResult, ComplianceRequirement, ConditionResult,
};
use polymesh_primitives::{
    proposition, storage_migration_ver, Claim, Condition, ConditionType, Context, IdentityId,
    TargetIdentity, Ticker, TrustedFor, TrustedIssuer, WeightMeter,
};

type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;

pub mod weight_for {
    use super::*;

    pub fn weight_for_verify_restriction<T: Config>(no_of_compliance_requirements: u64) -> Weight {
        no_of_compliance_requirements * Weight::from_ref_time(100_000_000)
    }

    pub fn weight_for_reading_asset_compliance<T: Config>() -> Weight {
        T::DbWeight::get().reads(1) + Weight::from_ref_time(1_000_000)
    }
}

storage_migration_ver!(0);

decl_storage! {
    trait Store for Module<T: Config> as ComplianceManager {
        /// Asset compliance for a ticker (Ticker -> AssetCompliance)
        pub AssetCompliances get(fn asset_compliance): map hasher(blake2_128_concat) Ticker => AssetCompliance;
        /// List of trusted claim issuer Ticker -> Issuer Identity
        pub TrustedClaimIssuer get(fn trusted_claim_issuer): map hasher(blake2_128_concat) Ticker => Vec<TrustedIssuer>;
        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(0)): Version;
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// User is not authorized.
        Unauthorized,
        /// Did not exist.
        DidNotExist,
        /// Compliance requirement id doesn't exist.
        InvalidComplianceRequirementId,
        /// Issuer exist but trying to add it again.
        IncorrectOperationOnTrustedIssuer,
        /// There are duplicate compliance requirements.
        DuplicateComplianceRequirements,
        /// The worst case scenario of the compliance requirement is too complex.
        ComplianceRequirementTooComplex,
        /// The maximum weight limit for executing the function was exceeded.
        WeightLimitExceeded
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {
        type Error = Error<T>;

        fn deposit_event() = default;

        const MaxConditionComplexity: u32 = T::MaxConditionComplexity::get();

        /// Adds a compliance requirement to an asset's compliance by ticker.
        /// If there are duplicate ClaimTypes for a particular trusted issuer, duplicates are removed.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        /// * sender_conditions - Sender transfer conditions.
        /// * receiver_conditions - Receiver transfer conditions.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_compliance_requirement_full(&sender_conditions, &receiver_conditions)]
        pub fn add_compliance_requirement(
            origin,
            ticker: Ticker,
            sender_conditions: Vec<Condition>,
            receiver_conditions: Vec<Condition>
        ) -> DispatchResult {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            Self::base_add_compliance_requirement(caller_did, ticker, sender_conditions, receiver_conditions)
        }

        /// Removes a compliance requirement from an asset's compliance.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        /// * id - Compliance requirement id which is need to be removed
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_compliance_requirement()]
        pub fn remove_compliance_requirement(origin, ticker: Ticker, id: u32) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

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
        /// Compliance requirements will be sorted (ascending by id) before
        /// replacing the current requirements.
        ///
        /// # Arguments
        /// * `ticker` - the asset ticker,
        /// * `asset_compliance - the new asset compliance.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the ticker.
        /// * `DuplicateAssetCompliance` if `asset_compliance` contains multiple entries with the same `requirement_id`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::replace_asset_compliance_full(&asset_compliance)]
        pub fn replace_asset_compliance(origin, ticker: Ticker, asset_compliance: Vec<ComplianceRequirement>) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

            // Ensure `Scope::Custom(..)`s are limited.
            Self::ensure_custom_scopes_limited(asset_compliance.iter().flat_map(|c| c.conditions()))?;

            // Ensure there are no duplicate requirement ids.
            let mut asset_compliance = asset_compliance;
            let start_len = asset_compliance.len();
            asset_compliance.sort_by_key(|r| r.id);
            asset_compliance.dedup_by_key(|r| r.id);
            ensure!(start_len == asset_compliance.len(), Error::<T>::DuplicateComplianceRequirements);

            // Dedup `ClaimType`s and ensure issuers are limited in length.
            asset_compliance.iter_mut().try_for_each(Self::dedup_and_ensure_requirement_limited)?;

            // Ensure the complexity is limited.
            Self::verify_compliance_complexity(&asset_compliance, ticker, 0)?;

            // Commit changes to storage + emit event.
            AssetCompliances::mutate(&ticker, |old| old.requirements = asset_compliance.clone());
            Self::deposit_event(Event::AssetComplianceReplaced(did, ticker, asset_compliance));
        }

        /// Removes an asset's compliance
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::reset_asset_compliance()]
        pub fn reset_asset_compliance(origin, ticker: Ticker) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            AssetCompliances::remove(ticker);
            Self::deposit_event(Event::AssetComplianceReset(did, ticker));
        }

        /// Pauses the verification of conditions for `ticker` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::pause_asset_compliance()]
        pub fn pause_asset_compliance(origin, ticker: Ticker) {
            let did = Self::pause_resume_asset_compliance(origin, ticker, true)?;
            Self::deposit_event(Event::AssetCompliancePaused(did, ticker));
        }

        /// Resumes the verification of conditions for `ticker` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::resume_asset_compliance()]
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
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_default_trusted_claim_issuer()]
        pub fn add_default_trusted_claim_issuer(origin, ticker: Ticker, issuer: TrustedIssuer) -> DispatchResult {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            Self::base_add_default_trusted_claim_issuer(caller_did, ticker, issuer)
        }

        /// Removes the given `issuer` from the set of default trusted claim issuers at the ticker level.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker.
        /// * ticker - Symbol of the asset.
        /// * issuer - IdentityId of the trusted claim issuer.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_default_trusted_claim_issuer()]
        pub fn remove_default_trusted_claim_issuer(origin, ticker: Ticker, issuer: IdentityId) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            TrustedClaimIssuer::try_mutate(ticker, |issuers| {
                let len = issuers.len();
                issuers.retain(|ti| ti.issuer != issuer);
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
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::change_compliance_requirement_full(&new_req)]
        pub fn change_compliance_requirement(origin, ticker: Ticker, new_req: ComplianceRequirement) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

            // Ensure `Scope::Custom(..)`s are limited.
            Self::ensure_custom_scopes_limited(new_req.conditions())?;

            let mut asset_compliance = AssetCompliances::get(ticker);
            let reqs = &mut asset_compliance.requirements;

            // If the compliance requirement is not found, throw an error.
            let pos = reqs.binary_search_by_key(&new_req.id, |req| req.id)
                .map_err(|_| Error::<T>::InvalidComplianceRequirementId)?;

            // Dedup `ClaimType`s and ensure issuers are limited in length.
            let mut new_req = new_req;
            Self::dedup_and_ensure_requirement_limited(&mut new_req)?;

            // Update asset compliance and verify complexity is limited.
            reqs[pos] = new_req.clone();
            Self::verify_compliance_complexity(&reqs, ticker, 0)?;

            // Store updated asset compliance.
            AssetCompliances::insert(&ticker, asset_compliance);
            Self::deposit_event(Event::ComplianceRequirementChanged(did, ticker, new_req));
        }
    }
}

impl<T: Config> Module<T> {
    /// Adds a compliance requirement to the given `ticker`.
    fn base_add_compliance_requirement(
        caller_did: IdentityId,
        ticker: Ticker,
        sender_conditions: Vec<Condition>,
        receiver_conditions: Vec<Condition>,
    ) -> DispatchResult {
        // Ensure `Scope::Custom(..)`s are limited.
        Self::ensure_custom_scopes_limited(sender_conditions.iter())?;
        Self::ensure_custom_scopes_limited(receiver_conditions.iter())?;

        // Bundle as a requirement.
        let id = Self::get_latest_requirement_id(ticker) + 1;
        let mut new_req = ComplianceRequirement {
            sender_conditions,
            receiver_conditions,
            id,
        };

        // Dedup `ClaimType`s and ensure issuers are limited in length.
        Self::dedup_and_ensure_requirement_limited(&mut new_req)?;

        // Add to existing requirements, and place a limit on the total complexity.
        let mut asset_compliance = AssetCompliances::get(ticker);
        asset_compliance.requirements.push(new_req.clone());
        Self::verify_compliance_complexity(&asset_compliance.requirements, ticker, 0)?;

        // Last storage change, now we can charge the fee.
        T::ProtocolFee::charge_fee(ProtocolOp::ComplianceManagerAddComplianceRequirement)?;

        // Commit new compliance to storage & emit event.
        AssetCompliances::insert(&ticker, asset_compliance);
        Self::deposit_event(Event::ComplianceRequirementCreated(
            caller_did, ticker, new_req,
        ));
        Ok(())
    }

    /// Adds a `issuer` as a default trusted claim issuer for `ticker`.
    fn base_add_default_trusted_claim_issuer(
        caller_did: IdentityId,
        ticker: Ticker,
        issuer: TrustedIssuer,
    ) -> DispatchResult {
        ensure!(
            <Identity<T>>::is_identity_exists(&issuer.issuer),
            Error::<T>::DidNotExist
        );

        // Ensure the new `issuer` is limited; the existing ones we have previously checked.
        Self::ensure_issuer_limited(&issuer)?;

        TrustedClaimIssuer::try_mutate(ticker, |issuers| {
            // Ensure we don't have too many issuers now in total.
            let new_count = issuers.len().saturating_add(1);
            ensure_length_ok::<T>(new_count)?;

            // Ensure the new issuer is new.
            ensure!(
                !issuers.contains(&issuer),
                Error::<T>::IncorrectOperationOnTrustedIssuer
            );

            // Ensure the complexity is limited for the ticker.
            Self::base_verify_compliance_complexity(
                &AssetCompliances::get(ticker).requirements,
                new_count,
            )?;

            // Finally add the new issuer & commit...
            issuers.push(issuer.clone());
            Ok(()) as DispatchResult
        })?;

        Self::deposit_event(Event::TrustedDefaultClaimIssuerAdded(
            caller_did, ticker, issuer,
        ));
        Ok(())
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

    /// Returns trusted issuers specified in `condition` if any,
    /// or otherwise returns the default trusted issuers for `ticker`.
    /// Defaults are cached in `slot`.
    fn issuers_for<'a>(
        ticker: &Ticker,
        condition: &'a Condition,
        slot: &'a mut Option<Vec<TrustedIssuer>>,
    ) -> &'a [TrustedIssuer] {
        if condition.issuers.is_empty() {
            slot.get_or_insert_with(|| Self::trusted_claim_issuer(ticker))
        } else {
            &condition.issuers
        }
    }

    /// Fetches the proposition context for target `id` and specific `condition`.
    /// Default trusted issuers, if fetched, are cached in `slot`.
    fn fetch_context<'a>(
        id: IdentityId,
        ticker: &Ticker,
        slot: &'a mut Option<Vec<TrustedIssuer>>,
        condition: &'a Condition,
        weight_meter: &mut WeightMeter,
    ) -> Result<proposition::Context<impl 'a + Iterator<Item = Claim>>, DispatchError> {
        // Because of `-> impl Iterator`, we need to return a **single type** in each of the branches below.
        // To do this, we use `Either<Either<MatchArm1, MatchArm2>, MatchArm3>`,
        // equivalent to a 3-variant enum with iterators in each variant corresponding to the branches below.
        // `Left(Left(arm1))`, `Left(Right(arm2))` and `Right(arm3)` correspond to arms 1, 2 and 3 respectively.
        use either::Either::{Left, Right};

        let claims = match &condition.condition_type {
            ConditionType::IsPresent(claim) | ConditionType::IsAbsent(claim) => {
                let trusted_issuers = Self::issuers_for(ticker, condition, slot);
                // Consumes the weight for this condition
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::is_condition_satisfied(
                        trusted_issuers.len() as u32,
                        condition.issuers.is_empty() as u32,
                    ),
                )?;
                Left(Left(Self::fetch_claims(id, claim, trusted_issuers)))
            }
            ConditionType::IsAnyOf(claims) | ConditionType::IsNoneOf(claims) => {
                let trusted_issuers = Self::issuers_for(ticker, condition, slot);
                // Consumes the weight for this condition
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::is_condition_satisfied(
                        (trusted_issuers.len() * claims.len()) as u32,
                        condition.issuers.is_empty() as u32,
                    ),
                )?;
                Left(Right(claims.iter().flat_map(move |claim| {
                    Self::fetch_claims(id, claim, trusted_issuers)
                })))
            }
            ConditionType::IsIdentity(TargetIdentity::ExternalAgent) => {
                // Consumes the weight for this condition
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::is_identity_condition(1),
                )?;
                Right(core::iter::empty())
            }
            ConditionType::IsIdentity(TargetIdentity::Specific(_)) => {
                // Consumes the weight for this condition
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::is_identity_condition(0),
                )?;
                Right(core::iter::empty())
            }
        };

        Ok(proposition::Context { claims, id })
    }

    /// Loads the context for each condition in `conditions` and verifies that all of them evaluate to `true`.
    fn are_all_conditions_satisfied(
        ticker: &Ticker,
        did: IdentityId,
        conditions: &[Condition],
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let slot = &mut None;
        for condition in conditions {
            if !Self::is_condition_satisfied(ticker, did, condition, slot, weight_meter)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Checks whether the given condition is satisfied or not.
    fn is_condition_satisfied(
        ticker: &Ticker,
        did: IdentityId,
        condition: &Condition,
        slot: &mut Option<Vec<TrustedIssuer>>,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let context = Self::fetch_context(did, ticker, slot, &condition, weight_meter)?;
        let any_ea = |ctx: Context<_>| ExternalAgents::<T>::agents(ticker, ctx.id).is_some();
        Ok(proposition::run(&condition, context, any_ea))
    }

    /// Returns whether all conditions, in their proper context, hold when evaluated.
    /// As a side-effect, each condition will be updated with its result,
    /// implying strict (non-lazy) evaluation of the conditions.
    fn evaluate_conditions(
        ticker: &Ticker,
        did: IdentityId,
        conditions: &mut [ConditionResult],
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let mut all_conditions_hold = true;
        for condition in conditions {
            let condition_holds = Self::is_condition_satisfied(
                ticker,
                did,
                &condition.condition,
                &mut None,
                weight_meter,
            )?;
            condition.result = condition_holds;
            all_conditions_hold = all_conditions_hold & condition_holds;
        }
        Ok(all_conditions_hold)
    }

    /// Pauses or resumes the asset compliance.
    fn pause_resume_asset_compliance(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        pause: bool,
    ) -> Result<IdentityId, DispatchError> {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
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

    /// Verify that `asset_compliance`, with `base` complexity,
    /// is within the maximum condition complexity allowed.
    pub fn verify_compliance_complexity(
        asset_compliance: &[ComplianceRequirement],
        ticker: Ticker,
        add: usize,
    ) -> DispatchResult {
        let count = TrustedClaimIssuer::decode_len(ticker)
            .unwrap_or_default()
            .saturating_add(add);
        Self::base_verify_compliance_complexity(asset_compliance, count)
    }

    /// Verify that `asset_compliance`, with `default_issuer_count`,
    /// is within the maximum condition complexity allowed.
    pub fn base_verify_compliance_complexity(
        asset_compliance: &[ComplianceRequirement],
        default_issuer_count: usize,
    ) -> DispatchResult {
        let complexity = asset_compliance
            .iter()
            .flat_map(|req| req.conditions())
            .fold(0u32, |total, condition| {
                let complexity = condition.complexity(default_issuer_count);
                total.saturating_add(complexity)
            })
            // NB: If the compliance requirements are empty (0 complexity),
            // then use the count of requirements.
            .max(asset_compliance.len() as u32);
        if complexity <= T::MaxConditionComplexity::get() {
            return Ok(());
        }
        Err(Error::<T>::ComplianceRequirementTooComplex.into())
    }

    fn ensure_custom_scopes_limited<'a>(
        condition: impl Iterator<Item = &'a Condition>,
    ) -> DispatchResult {
        condition
            .flat_map(|c| c.claims())
            .try_for_each(Identity::<T>::ensure_custom_scopes_limited)
    }

    fn dedup_and_ensure_requirement_limited(req: &mut ComplianceRequirement) -> DispatchResult {
        // Dedup `ClaimType`s in `TrustedFor::Specific`.
        req.dedup();

        // Ensure issuers are limited in length.
        Self::ensure_issuers_in_req_limited(req)
    }

    fn ensure_issuers_in_req_limited(req: &ComplianceRequirement) -> DispatchResult {
        req.conditions().try_for_each(|cond| {
            ensure_length_ok::<T>(cond.issuers.len())?;
            cond.issuers
                .iter()
                .try_for_each(Self::ensure_issuer_limited)
        })
    }

    fn ensure_issuer_limited(issuer: &TrustedIssuer) -> DispatchResult {
        match &issuer.trusted_for {
            TrustedFor::Any => Ok(()),
            TrustedFor::Specific(cts) => ensure_length_ok::<T>(cts.len()),
        }
    }

    /// Consumes from `weight_meter` the given `weight`.
    /// If the new consumed weight is greater than the limit, consumed will be set to limit and an error will be returned.
    fn consume_weight_meter(weight_meter: &mut WeightMeter, weight: Weight) -> DispatchResult {
        weight_meter
            .consume_weight_until_limit(weight)
            .map_err(|_| Error::<T>::WeightLimitExceeded.into())
    }

    // Returns `true` if any requirement is satisfied, otherwise returns `false`.
    fn is_any_requirement_compliant(
        ticker: &Ticker,
        requirements: &[ComplianceRequirement],
        sender_did: IdentityId,
        receiver_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        for requirement in requirements {
            // Returns true if all conditions for the sender and receiver are satisfied
            if Self::are_all_conditions_satisfied(
                ticker,
                sender_did,
                &requirement.sender_conditions,
                weight_meter,
            )? && Self::are_all_conditions_satisfied(
                ticker,
                receiver_did,
                &requirement.receiver_conditions,
                weight_meter,
            )? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl<T: Config> ComplianceFnConfig for Module<T> {
    fn is_compliant(
        ticker: &Ticker,
        sender_did: IdentityId,
        receiver_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let asset_compliance = Self::asset_compliance(ticker);

        // If compliance is paused, the rules are not checked
        if asset_compliance.paused {
            return Ok(true);
        }

        Self::is_any_requirement_compliant(
            ticker,
            &asset_compliance.requirements,
            sender_did,
            receiver_did,
            weight_meter,
        )
    }

    /// verifies all requirements and returns the result in an array of booleans.
    /// this does not care if the requirements are paused or not. It is meant to be
    /// called only in failure conditions
    fn verify_restriction_granular(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        weight_meter: &mut WeightMeter,
    ) -> Result<AssetComplianceResult, DispatchError> {
        let mut compliance_with_results =
            AssetComplianceResult::from(Self::asset_compliance(ticker));

        // Evaluates all conditions.
        // False result in any of the conditions => False requirement result.
        let all_conditions_hold = |did, conditions, weight_meter: &mut WeightMeter| match did {
            Some(did) => Self::evaluate_conditions(ticker, did, conditions, weight_meter),
            None => Ok(false),
        };

        for req in &mut compliance_with_results.requirements {
            if !all_conditions_hold(from_did_opt, &mut req.sender_conditions, weight_meter)? {
                req.result = false;
            }
            if !all_conditions_hold(to_did_opt, &mut req.receiver_conditions, weight_meter)? {
                req.result = false;
            }
            compliance_with_results.result |= req.result;
        }
        Ok(compliance_with_results)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn setup_ticker_compliance(
        caller_did: IdentityId,
        ticker: Ticker,
        n: u32,
        pause_compliance: bool,
    ) {
        benchmarking::setup_ticker_compliance::<T>(caller_did, ticker, n, pause_compliance);
    }
}
