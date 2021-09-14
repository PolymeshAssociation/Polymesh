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
#![feature(const_option)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    weights::Weight,
};
use pallet_base::ensure_length_ok;
use pallet_external_agents::Config as EAConfig;
pub use polymesh_common_utilities::traits::compliance_manager::WeightInfo;
use polymesh_common_utilities::{
    asset::AssetFnTrait,
    balances::Config as BalancesConfig,
    compliance_manager::Config as ComplianceManagerConfig,
    constants::*,
    identity::Config as IdentityConfig,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
};
use polymesh_primitives::{
    compliance_manager::{
        AssetCompliance, AssetComplianceResult, ComplianceRequirement, ConditionResult,
    },
    proposition, storage_migration_ver, Balance, Claim, Condition, ConditionType, Context,
    IdentityId, Ticker, TrustedFor, TrustedIssuer,
};
use sp_std::{
    convert::{From, TryFrom},
    prelude::*,
};

type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;

/// The module's configuration trait.
pub trait Config:
    pallet_timestamp::Config + frame_system::Config + BalancesConfig + IdentityConfig + EAConfig
{
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    /// Asset module
    type Asset: AssetFnTrait<Self::AccountId, Self::Origin>;

    /// Weight details of all extrinsic
    type WeightInfo: WeightInfo;

    /// The maximum claim reads that are allowed to happen in worst case of a condition resolution
    type MaxConditionComplexity: Get<u32>;
}

pub mod weight_for {
    use super::*;

    pub fn weight_for_verify_restriction<T: Config>(no_of_compliance_requirements: u64) -> Weight {
        no_of_compliance_requirements * 100_000_000
    }

    pub fn weight_for_reading_asset_compliance<T: Config>() -> Weight {
        T::DbWeight::get().reads(1) + 1_000_000
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
        StorageVersion get(fn storage_version) build(|_| Version::new(0).unwrap()): Version;
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// User is not authorized.
        Unauthorized,
        /// Did not exist
        DidNotExist,
        /// Compliance requirement id doesn't exist
        InvalidComplianceRequirementId,
        /// Issuer exist but trying to add it again
        IncorrectOperationOnTrustedIssuer,
        /// There are duplicate compliance requirements.
        DuplicateComplianceRequirements,
        /// The worst case scenario of the compliance requirement is too complex
        ComplianceRequirementTooComplex,
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        const MaxConditionComplexity: u32 = T::MaxConditionComplexity::get();

        /// Adds a compliance requirement to an asset's compliance by ticker.
        /// If the compliance requirement is a duplicate, it does nothing.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the ticker
        /// * ticker - Symbol of the asset
        /// * sender_conditions - Sender transfer conditions.
        /// * receiver_conditions - Receiver transfer conditions.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_compliance_requirement(sender_conditions.len() as u32, receiver_conditions.len() as u32)]
        pub fn add_compliance_requirement(origin, ticker: Ticker, sender_conditions: Vec<Condition>, receiver_conditions: Vec<Condition>) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

            // Bundle as a requirement.
            let id = Self::get_latest_requirement_id(ticker) + 1u32;
            let mut new_req = ComplianceRequirement { sender_conditions, receiver_conditions, id };
            new_req.dedup();

            // Ensure issuers are limited in length.
            Self::ensure_issuers_in_req_limited(&new_req)?;

            // Add to existing requirements, and place a limit on the total complexity.
            let mut asset_compliance = AssetCompliances::get(ticker);
            let reqs = &mut asset_compliance.requirements;
            reqs.push(new_req.clone());
            Self::verify_compliance_complexity(&reqs, ticker, 0)?;

            // Last storage change, now we can charge the fee.
            T::ProtocolFee::charge_fee(ProtocolOp::ComplianceManagerAddComplianceRequirement)?;

            // Commit new compliance to storage & emit event.
            AssetCompliances::insert(&ticker, asset_compliance);
            Self::deposit_event(Event::ComplianceRequirementCreated(did, ticker, new_req));
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
        #[weight = <T as Config>::WeightInfo::replace_asset_compliance(asset_compliance.len() as u32)]
        pub fn replace_asset_compliance(origin, ticker: Ticker, asset_compliance: Vec<ComplianceRequirement>) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

            // Ensure there are no duplicate requirement ids.
            let mut asset_compliance = asset_compliance;
            let start_len = asset_compliance.len();
            asset_compliance.dedup_by_key(|r| r.id);
            ensure!(start_len == asset_compliance.len(), Error::<T>::DuplicateComplianceRequirements);

            // Dedup `ClaimType`s in `TrustedFor::Specific`.
            asset_compliance.iter_mut().for_each(|r| r.dedup());

            // Ensure issuers are limited in length + limit the complexity.
            asset_compliance.iter().try_for_each(Self::ensure_issuers_in_req_limited)?;
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
        pub fn add_default_trusted_claim_issuer(origin, ticker: Ticker, issuer: TrustedIssuer) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            ensure!(<Identity<T>>::is_identity_exists(&issuer.issuer), Error::<T>::DidNotExist);

            // Ensure the new `issuer` is limited; the existing ones we have previously checked.
            Self::ensure_issuer_limited(&issuer)?;

            TrustedClaimIssuer::try_mutate(ticker, |issuers| {
                // Ensure we don't have too many issuers now in total.
                let new_count = issuers.len().saturating_add(1);
                ensure_length_ok::<T>(new_count)?;

                // Ensure the new issuer is new.
                ensure!(!issuers.contains(&issuer), Error::<T>::IncorrectOperationOnTrustedIssuer);

                // Ensure the complexity is limited for the ticker.
                Self::base_verify_compliance_complexity(&AssetCompliances::get(ticker).requirements, new_count)?;

                // Finally add the new issuer & commit...
                issuers.push(issuer.clone());
                Ok(()) as DispatchResult
            })?;

            // ...and emit the event.
            Self::deposit_event(Event::TrustedDefaultClaimIssuerAdded(did, ticker, issuer));
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
        #[weight = <T as Config>::WeightInfo::change_compliance_requirement(
            new_req.sender_conditions.len() as u32,
            new_req.receiver_conditions.len() as u32,
        )]
        pub fn change_compliance_requirement(origin, ticker: Ticker, new_req: ComplianceRequirement) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
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
        TrustedDefaultClaimIssuerRemoved(IdentityId, Ticker, IdentityId),
    }
);

impl<T: Config> Module<T> {
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
    ) -> proposition::Context<impl 'a + Iterator<Item = Claim>> {
        // Because of `-> impl Iterator`, we need to return a **single type** in each of the branches below.
        // To do this, we use `Either<Either<MatchArm1, MatchArm2>, MatchArm3>`,
        // equivalent to a 3-variant enum with iterators in each variant corresponding to the branches below.
        // `Left(Left(arm1))`, `Left(Right(arm2))` and `Right(arm3)` correspond to arms 1, 2 and 3 respectively.
        use either::Either::{Left, Right};

        let claims = match &condition.condition_type {
            ConditionType::IsPresent(claim) | ConditionType::IsAbsent(claim) => Left(Left(
                Self::fetch_claims(id, claim, Self::issuers_for(ticker, condition, slot)),
            )),
            ConditionType::IsAnyOf(claims) | ConditionType::IsNoneOf(claims) => {
                let issuers = Self::issuers_for(ticker, condition, slot);
                Left(Right(claims.iter().flat_map(move |claim| {
                    Self::fetch_claims(id, claim, issuers)
                })))
            }
            ConditionType::IsIdentity(_) => Right(core::iter::empty()),
        };

        proposition::Context { claims, id }
    }

    /// Loads the context for each condition in `conditions` and verifies that all of them evaluate to `true`.
    fn are_all_conditions_satisfied(
        ticker: &Ticker,
        did: IdentityId,
        conditions: &[Condition],
    ) -> bool {
        let slot = &mut None;
        conditions
            .iter()
            .all(|condition| Self::is_condition_satisfied(ticker, did, condition, slot))
    }

    /// Checks whether the given condition is satisfied or not.
    fn is_condition_satisfied(
        ticker: &Ticker,
        did: IdentityId,
        condition: &Condition,
        slot: &mut Option<Vec<TrustedIssuer>>,
    ) -> bool {
        let context = Self::fetch_context(did, ticker, slot, &condition);
        let any_ea = |ctx: Context<_>| ExternalAgents::<T>::agents(ticker, ctx.id).is_some();
        proposition::run(&condition, context, any_ea)
    }

    /// Returns whether all conditions, in their proper context, hold when evaluated.
    /// As a side-effect, each condition will be updated with its result,
    /// implying strict (non-lazy) evaluation of the conditions.
    fn evaluate_conditions(
        ticker: &Ticker,
        did: IdentityId,
        conditions: &mut [ConditionResult],
    ) -> bool {
        conditions.iter_mut().fold(true, |overall, res| {
            res.result = Self::is_condition_satisfied(ticker, did, &res.condition, &mut None);
            overall & res.result
        })
    }

    /// Pauses or resumes the asset compliance.
    fn pause_resume_asset_compliance(
        origin: T::Origin,
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
}

impl<T: Config> ComplianceManagerConfig for Module<T> {
    ///  Sender restriction verification
    fn verify_restriction(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        _: Balance,
    ) -> Result<u8, DispatchError> {
        // Transfer is valid if ALL receiver AND sender conditions of ANY asset conditions are valid.
        let asset_compliance = Self::asset_compliance(ticker);
        if asset_compliance.paused {
            return Ok(ERC1400_TRANSFER_SUCCESS);
        }

        for req in asset_compliance.requirements {
            if let Some(from_did) = from_did_opt {
                if !Self::are_all_conditions_satisfied(ticker, from_did, &req.sender_conditions) {
                    // Skips checking receiver conditions because sender conditions are not satisfied.
                    continue;
                }
            }

            if let Some(to_did) = to_did_opt {
                if Self::are_all_conditions_satisfied(ticker, to_did, &req.receiver_conditions) {
                    // All conditions satisfied, return early
                    return Ok(ERC1400_TRANSFER_SUCCESS);
                }
            }
        }
        Ok(ERC1400_TRANSFER_FAILURE)
    }

    /// verifies all requirements and returns the result in an array of booleans.
    /// this does not care if the requirements are paused or not. It is meant to be
    /// called only in failure conditions
    fn verify_restriction_granular(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
    ) -> AssetComplianceResult {
        let mut compliance_with_results =
            AssetComplianceResult::from(Self::asset_compliance(ticker));

        // Evaluates all conditions.
        // False result in any of the conditions => False requirement result.
        let eval = |did: Option<_>, conds| {
            did.filter(|did| !Self::evaluate_conditions(ticker, *did, conds))
                .is_some()
        };
        for req in &mut compliance_with_results.requirements {
            if eval(from_did_opt, &mut req.sender_conditions) {
                req.result = false;
            }
            if eval(to_did_opt, &mut req.receiver_conditions) {
                req.result = false;
            }
            compliance_with_results.result |= req.result;
        }
        compliance_with_results
    }
}
