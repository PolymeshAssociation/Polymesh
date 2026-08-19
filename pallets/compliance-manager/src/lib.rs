// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

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
//! - **AssetCompliance:** It is an array of compliance requirements that are currently enforced for an asset.
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
//! - [add_compliance_requirement](Pallet::add_compliance_requirement) - Adds a new compliance requirement to an asset's compliance.
//! - [remove_compliance_requirement](Pallet::remove_compliance_requirement) - Removes a compliance requirement from an asset's compliance.
//! - [reset_asset_compliance](Pallet::reset_asset_compliance) - Resets(remove) an asset's compliance.
//! - [pause_asset_compliance](Pallet::pause_asset_compliance) - Pauses the evaluation of asset compliance for an asset  before executing a
//! transaction.
//! - [add_default_trusted_claim_issuer](Pallet::add_default_trusted_claim_issuer) - Adds a default
//!  trusted claim issuer for a given asset.
//! - [remove_default_trusted_claim_issuer](Pallet::remove_default_trusted_claim_issuer) - Removes
//!  the default claim issuer.
//! - [change_compliance_requirement](Pallet::change_compliance_requirement) - Updates a compliance requirement, based on its id.
//! based on its id for a given asset.
//!
//! ### Public Functions
//!
//! - [verify_restriction](Pallet::verify_restriction) - Checks if a transfer is a valid transfer and returns the result

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::dispatch::DispatchResult;
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchError;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_system::pallet_prelude::OriginFor;
use sp_std::{convert::From, prelude::*};

use pallet_base::ensure_length_ok;
use pallet_external_agents::{Config as EAConfig, GroupOfAgent};
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::compliance_manager::{
    AssetCompliance, ComplianceReport, ComplianceRequirement, ConditionReport, RequirementReport,
};
use polymesh_primitives::condition::{conditions_total_counts, Condition};
use polymesh_primitives::protocol_fee::{ChargeProtocolFee, ProtocolOp};
use polymesh_primitives::traits::{AssetFnConfig, ComplianceFnConfig};
use polymesh_primitives::{
    proposition, storage_migration_ver, Claim, ConditionType, Context, IdentityId, TargetIdentity,
    TrustedFor, TrustedIssuer, WeightMeter,
};

type ExternalAgents<T> = pallet_external_agents::Pallet<T>;
type Identity<T> = pallet_identity::Pallet<T>;

storage_migration_ver!(1);

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        pallet_timestamp::Config
        + frame_system::Config
        + pallet_permissions::Config
        + pallet_identity::Config
        + EAConfig
        + AssetFnConfig
    {
        /// Weight details of all extrinsic
        type WeightInfo: WeightInfo;

        /// The maximum claim reads that are allowed to happen in worst case of a condition resolution
        #[pallet::constant]
        type MaxConditionComplexity: Get<u32>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when new compliance requirement is created.
        /// (caller DID, AssetId, ComplianceRequirement).
        ComplianceRequirementCreated(IdentityId, AssetId, ComplianceRequirement),
        /// Emitted when a compliance requirement is removed.
        /// (caller DID, AssetId, requirement_id).
        ComplianceRequirementRemoved(IdentityId, AssetId, u32),
        /// Emitted when an asset compliance is replaced.
        /// Parameters: caller DID, AssetId, new asset compliance.
        AssetComplianceReplaced(IdentityId, AssetId, Vec<ComplianceRequirement>),
        /// Emitted when an asset compliance of a asset_id is reset.
        /// (caller DID, AssetId).
        AssetComplianceReset(IdentityId, AssetId),
        /// Emitted when an asset compliance for a given asset_id gets resume.
        /// (caller DID, AssetId).
        AssetComplianceResumed(IdentityId, AssetId),
        /// Emitted when an asset compliance for a given asset_id gets paused.
        /// (caller DID, AssetId).
        AssetCompliancePaused(IdentityId, AssetId),
        /// Emitted when compliance requirement get modified/change.
        /// (caller DID, AssetId, ComplianceRequirement).
        ComplianceRequirementChanged(IdentityId, AssetId, ComplianceRequirement),
        /// Emitted when default claim issuer list for a given asset_id gets added.
        /// (caller DID, AssetId, Added TrustedIssuer).
        TrustedDefaultClaimIssuerAdded(IdentityId, AssetId, TrustedIssuer),
        /// Emitted when default claim issuer list for a given asset_id get removed.
        /// (caller DID, AssetId, Removed TrustedIssuer).
        TrustedDefaultClaimIssuerRemoved(IdentityId, AssetId, IdentityId),
    }

    pub trait WeightInfo {
        fn add_compliance_requirement(c: u32) -> Weight;
        fn remove_compliance_requirement() -> Weight;
        fn pause_asset_compliance() -> Weight;
        fn resume_asset_compliance() -> Weight;
        fn add_default_trusted_claim_issuer() -> Weight;
        fn remove_default_trusted_claim_issuer() -> Weight;
        fn change_compliance_requirement(c: u32) -> Weight;
        fn replace_asset_compliance(c: u32) -> Weight;
        fn reset_asset_compliance() -> Weight;
        fn is_condition_satisfied(c: u32, t: u32) -> Weight;
        fn is_identity_condition(e: u32) -> Weight;
        fn is_any_requirement_compliant(i: u32) -> Weight;

        fn condition_costs(conditions: u32, claims: u32, issuers: u32, claim_types: u32) -> Weight;

        fn add_compliance_requirement_full(sender: &[Condition], receiver: &[Condition]) -> Weight {
            let (condtions, claims, issuers, claim_types) =
                conditions_total_counts(sender.iter().chain(receiver.iter()));
            Self::add_compliance_requirement(condtions).saturating_add(Self::condition_costs(
                0,
                claims,
                issuers,
                claim_types,
            ))
        }

        fn change_compliance_requirement_full(req: &ComplianceRequirement) -> Weight {
            let (conditions, claims, issuers, claim_types) = req.counts();
            Self::change_compliance_requirement(conditions).saturating_add(Self::condition_costs(
                0,
                claims,
                issuers,
                claim_types,
            ))
        }

        fn replace_asset_compliance_full(reqs: &[ComplianceRequirement]) -> Weight {
            let (conditions, claims, issuers, claim_types) =
                conditions_total_counts(reqs.iter().flat_map(|req| req.conditions()));
            Self::replace_asset_compliance(reqs.len() as u32).saturating_add(Self::condition_costs(
                conditions,
                claims,
                issuers,
                claim_types,
            ))
        }

        fn is_any_requirement_compliant_loop(i: u32) -> Weight {
            Self::is_any_requirement_compliant(i)
                .saturating_sub(Self::is_identity_condition(0).saturating_mul(i.into()))
        }
    }

    /// Compliance for an asset ([`AssetId`] -> [`AssetCompliance`])
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AssetCompliances<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, AssetCompliance, ValueQuery>;

    /// List of trusted claim issuer [`AssetId`] -> Issuer Identity
    #[pallet::storage]
    #[pallet::unbounded]
    pub type TrustedClaimIssuer<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, Vec<TrustedIssuer>, ValueQuery>;

    /// Storage version.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            StorageVersion::<T>::put(Version::new(1));
        }
    }

    #[pallet::error]
    pub enum Error<T> {
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
        WeightLimitExceeded,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Adds a compliance requirement to an asset given by `asset_id`.
        /// If there are duplicate ClaimTypes for a particular trusted issuer, duplicates are removed.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the asset
        /// * asset_id - Symbol of the asset
        /// * sender_conditions - Sender transfer conditions.
        /// * receiver_conditions - Receiver transfer conditions.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::add_compliance_requirement_full(&sender_conditions, &receiver_conditions))]
        #[pallet::call_index(0)]
        pub fn add_compliance_requirement(
            origin: OriginFor<T>,
            asset_id: AssetId,
            sender_conditions: Vec<Condition>,
            receiver_conditions: Vec<Condition>,
        ) -> DispatchResult {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
            Self::base_add_compliance_requirement(
                caller_did,
                asset_id,
                sender_conditions,
                receiver_conditions,
            )
        }

        /// Removes a compliance requirement from an asset's compliance.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the asset
        /// * asset_id - Symbol of the asset
        /// * id - Compliance requirement id which is need to be removed
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::remove_compliance_requirement())]
        #[pallet::call_index(1)]
        pub fn remove_compliance_requirement(
            origin: OriginFor<T>,
            asset_id: AssetId,
            id: u32,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

            AssetCompliances::<T>::try_mutate(
                asset_id,
                |AssetCompliance { requirements, .. }| {
                    let before = requirements.len();
                    requirements.retain(|requirement| requirement.id != id);
                    ensure!(
                        before != requirements.len(),
                        Error::<T>::InvalidComplianceRequirementId
                    );
                    Ok(()) as DispatchResult
                },
            )?;

            Self::deposit_event(Event::ComplianceRequirementRemoved(did, asset_id, id));
            Ok(())
        }

        /// Replaces an asset's compliance with a new compliance.
        ///
        /// Compliance requirements will be sorted (ascending by id) before
        /// replacing the current requirements.
        ///
        /// # Arguments
        /// * `asset_id` - the asset asset_id,
        /// * `asset_compliance - the new asset compliance.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the asset_id.
        /// * `DuplicateAssetCompliance` if `asset_compliance` contains multiple entries with the same `requirement_id`.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::replace_asset_compliance_full(&asset_compliance))]
        #[pallet::call_index(2)]
        pub fn replace_asset_compliance(
            origin: OriginFor<T>,
            asset_id: AssetId,
            asset_compliance: Vec<ComplianceRequirement>,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

            // Ensure `Scope::Custom(..)`s are limited.
            Self::ensure_custom_scopes_limited(
                asset_compliance.iter().flat_map(|c| c.conditions()),
            )?;

            // Ensure there are no duplicate requirement ids.
            let mut asset_compliance = asset_compliance;
            let start_len = asset_compliance.len();
            asset_compliance.sort_by_key(|r| r.id);
            asset_compliance.dedup_by_key(|r| r.id);
            ensure!(
                start_len == asset_compliance.len(),
                Error::<T>::DuplicateComplianceRequirements
            );

            // Dedup `ClaimType`s and ensure issuers are limited in length.
            asset_compliance
                .iter_mut()
                .try_for_each(Self::dedup_and_ensure_requirement_limited)?;

            // Ensure the complexity is limited.
            Self::verify_compliance_complexity(&asset_compliance, asset_id, 0)?;

            // Commit changes to storage + emit event.
            AssetCompliances::<T>::mutate(&asset_id, |old| {
                old.requirements = asset_compliance.clone()
            });
            Self::deposit_event(Event::AssetComplianceReplaced(
                did,
                asset_id,
                asset_compliance,
            ));
            Ok(())
        }

        /// Removes an asset's compliance
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the asset
        /// * asset_id - Symbol of the asset
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::reset_asset_compliance())]
        #[pallet::call_index(3)]
        pub fn reset_asset_compliance(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
            AssetCompliances::<T>::remove(asset_id);
            Self::deposit_event(Event::AssetComplianceReset(did, asset_id));
            Ok(())
        }

        /// Pauses the verification of conditions for `asset_id` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the asset
        /// * asset_id - Symbol of the asset
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::pause_asset_compliance())]
        #[pallet::call_index(4)]
        pub fn pause_asset_compliance(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
            let did = Self::pause_resume_asset_compliance(origin, &asset_id, true)?;
            Self::deposit_event(Event::AssetCompliancePaused(did, asset_id));
            Ok(())
        }

        /// Resumes the verification of conditions for `asset_id` during transfers.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the asset
        /// * asset_id - Symbol of the asset
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::resume_asset_compliance())]
        #[pallet::call_index(5)]
        pub fn resume_asset_compliance(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
            let did = Self::pause_resume_asset_compliance(origin, &asset_id, false)?;
            Self::deposit_event(Event::AssetComplianceResumed(did, asset_id));
            Ok(())
        }

        /// Adds another default trusted claim issuer at the asset level.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the asset.
        /// * asset_id - Symbol of the asset.
        /// * issuer - IdentityId of the trusted claim issuer.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::add_default_trusted_claim_issuer())]
        #[pallet::call_index(6)]
        pub fn add_default_trusted_claim_issuer(
            origin: OriginFor<T>,
            asset_id: AssetId,
            issuer: TrustedIssuer,
        ) -> DispatchResult {
            let caller_did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
            Self::base_add_default_trusted_claim_issuer(caller_did, asset_id, issuer)
        }

        /// Removes the given `issuer` from the set of default trusted claim issuers at the asset level.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the asset.
        /// * asset_id - Symbol of the asset.
        /// * issuer - IdentityId of the trusted claim issuer.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::remove_default_trusted_claim_issuer())]
        #[pallet::call_index(7)]
        pub fn remove_default_trusted_claim_issuer(
            origin: OriginFor<T>,
            asset_id: AssetId,
            issuer: IdentityId,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
            TrustedClaimIssuer::<T>::try_mutate(asset_id, |issuers| {
                let len = issuers.len();
                issuers.retain(|ti| ti.issuer != issuer);
                ensure!(
                    len != issuers.len(),
                    Error::<T>::IncorrectOperationOnTrustedIssuer
                );
                Ok(()) as DispatchResult
            })?;
            Self::deposit_event(Event::TrustedDefaultClaimIssuerRemoved(
                did, asset_id, issuer,
            ));
            Ok(())
        }

        /// Modify an existing compliance requirement of a given asset.
        ///
        /// # Arguments
        /// * origin - Signer of the dispatchable. It should be the owner of the asset.
        /// * asset_id - Symbol of the asset.
        /// * new_req - Compliance requirement.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::change_compliance_requirement_full(&new_req))]
        #[pallet::call_index(8)]
        pub fn change_compliance_requirement(
            origin: OriginFor<T>,
            asset_id: AssetId,
            new_req: ComplianceRequirement,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;

            // Ensure `Scope::Custom(..)`s are limited.
            Self::ensure_custom_scopes_limited(new_req.conditions())?;

            let mut asset_compliance = AssetCompliances::<T>::get(asset_id);
            let reqs = &mut asset_compliance.requirements;

            // If the compliance requirement is not found, throw an error.
            let pos = reqs
                .binary_search_by_key(&new_req.id, |req| req.id)
                .map_err(|_| Error::<T>::InvalidComplianceRequirementId)?;

            // Dedup `ClaimType`s and ensure issuers are limited in length.
            let mut new_req = new_req;
            Self::dedup_and_ensure_requirement_limited(&mut new_req)?;

            // Update asset compliance and verify complexity is limited.
            reqs[pos] = new_req.clone();
            Self::verify_compliance_complexity(&reqs, asset_id, 0)?;

            // Store updated asset compliance.
            AssetCompliances::<T>::insert(&asset_id, asset_compliance);
            Self::deposit_event(Event::ComplianceRequirementChanged(did, asset_id, new_req));
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Adds a compliance requirement to the given `asset_id`.
    fn base_add_compliance_requirement(
        caller_did: IdentityId,
        asset_id: AssetId,
        sender_conditions: Vec<Condition>,
        receiver_conditions: Vec<Condition>,
    ) -> DispatchResult {
        // Ensure `Scope::Custom(..)`s are limited.
        Self::ensure_custom_scopes_limited(sender_conditions.iter())?;
        Self::ensure_custom_scopes_limited(receiver_conditions.iter())?;

        // Bundle as a requirement.
        let id = Self::get_latest_requirement_id(&asset_id) + 1;
        let mut new_req = ComplianceRequirement {
            sender_conditions,
            receiver_conditions,
            id,
        };

        // Dedup `ClaimType`s and ensure issuers are limited in length.
        Self::dedup_and_ensure_requirement_limited(&mut new_req)?;

        // Add to existing requirements, and place a limit on the total complexity.
        let mut asset_compliance = AssetCompliances::<T>::get(&asset_id);
        asset_compliance.requirements.push(new_req.clone());
        Self::verify_compliance_complexity(&asset_compliance.requirements, asset_id, 0)?;

        // Last storage change, now we can charge the fee.
        T::ProtocolFee::charge_fee(ProtocolOp::ComplianceManagerAddComplianceRequirement)?;

        // Commit new compliance to storage & emit event.
        AssetCompliances::<T>::insert(&asset_id, asset_compliance);
        Self::deposit_event(Event::ComplianceRequirementCreated(
            caller_did, asset_id, new_req,
        ));
        Ok(())
    }

    /// Adds a `issuer` as a default trusted claim issuer for `asset_id`.
    fn base_add_default_trusted_claim_issuer(
        caller_did: IdentityId,
        asset_id: AssetId,
        issuer: TrustedIssuer,
    ) -> DispatchResult {
        ensure!(
            <Identity<T>>::is_did_active(issuer.issuer),
            Error::<T>::DidNotExist
        );

        // Ensure the new `issuer` is limited; the existing ones we have previously checked.
        Self::ensure_issuer_limited(&issuer)?;

        TrustedClaimIssuer::<T>::try_mutate(asset_id, |issuers| {
            // Ensure we don't have too many issuers now in total.
            let new_count = issuers.len().saturating_add(1);
            ensure_length_ok::<T>(new_count)?;

            // Ensure the new issuer is new.
            ensure!(
                !issuers.contains(&issuer),
                Error::<T>::IncorrectOperationOnTrustedIssuer
            );

            // Ensure the complexity is limited for the asset_id.
            Self::base_verify_compliance_complexity(
                &AssetCompliances::<T>::get(asset_id).requirements,
                new_count,
            )?;

            // Finally add the new issuer & commit...
            issuers.push(issuer.clone());
            Ok(()) as DispatchResult
        })?;

        Self::deposit_event(Event::TrustedDefaultClaimIssuerAdded(
            caller_did, asset_id, issuer,
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
    /// or otherwise returns the default trusted issuers for `asset_id`.
    /// Defaults are cached in `slot`.
    fn issuers_for<'a>(
        asset_id: &AssetId,
        condition: &'a Condition,
        slot: &'a mut Option<Vec<TrustedIssuer>>,
    ) -> &'a [TrustedIssuer] {
        if condition.issuers.is_empty() {
            slot.get_or_insert_with(|| TrustedClaimIssuer::<T>::get(asset_id))
        } else {
            &condition.issuers
        }
    }

    /// Fetches the proposition context for target `id` and specific `condition`.
    /// Default trusted issuers, if fetched, are cached in `slot`.
    fn fetch_context<'a>(
        id: IdentityId,
        asset_id: &AssetId,
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
                let trusted_issuers = Self::issuers_for(asset_id, condition, slot);
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
                let trusted_issuers = Self::issuers_for(asset_id, condition, slot);
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
        asset_id: &AssetId,
        did: IdentityId,
        conditions: &[Condition],
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let slot = &mut None;
        for condition in conditions {
            if !Self::is_condition_satisfied(asset_id, did, condition, slot, weight_meter)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Checks whether the given condition is satisfied or not.
    fn is_condition_satisfied(
        asset_id: &AssetId,
        did: IdentityId,
        condition: &Condition,
        slot: &mut Option<Vec<TrustedIssuer>>,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let context = Self::fetch_context(did, asset_id, slot, &condition, weight_meter)?;
        let any_ea = |ctx: Context<_>| GroupOfAgent::<T>::get(asset_id, ctx.id).is_some();
        Ok(proposition::run(&condition, context, any_ea))
    }

    /// Pauses or resumes the asset compliance.
    fn pause_resume_asset_compliance(
        origin: OriginFor<T>,
        asset_id: &AssetId,
        pause: bool,
    ) -> Result<IdentityId, DispatchError> {
        let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
        AssetCompliances::<T>::mutate(asset_id, |compliance| compliance.paused = pause);
        Ok(did)
    }

    /// Compute the id of the last requirement in an asset's compliance rules.
    fn get_latest_requirement_id(asset_id: &AssetId) -> u32 {
        AssetCompliances::<T>::get(asset_id)
            .requirements
            .last()
            .map(|r| r.id)
            .unwrap_or(0)
    }

    /// Verify that `asset_compliance`, with `base` complexity,
    /// is within the maximum condition complexity allowed.
    pub fn verify_compliance_complexity(
        asset_compliance: &[ComplianceRequirement],
        asset_id: AssetId,
        add: usize,
    ) -> DispatchResult {
        let count = TrustedClaimIssuer::<T>::decode_len(asset_id)
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
        asset_id: &AssetId,
        requirements: &[ComplianceRequirement],
        sender_did: IdentityId,
        receiver_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        for requirement in requirements {
            // Returns true if all conditions for the sender and receiver are satisfied
            if Self::are_all_conditions_satisfied(
                asset_id,
                sender_did,
                &requirement.sender_conditions,
                weight_meter,
            )? && Self::are_all_conditions_satisfied(
                asset_id,
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

impl<T: Config> ComplianceFnConfig for Pallet<T> {
    fn is_compliant(
        asset_id: &AssetId,
        sender_did: IdentityId,
        receiver_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let asset_compliance = AssetCompliances::<T>::get(asset_id);

        // If there are no requirements or compliance is paused, no rules are checked.
        if asset_compliance.paused || asset_compliance.requirements.is_empty() {
            return Ok(true);
        }

        Self::is_any_requirement_compliant(
            asset_id,
            &asset_compliance.requirements,
            sender_did,
            receiver_did,
            weight_meter,
        )
    }

    fn is_holder_compliant(
        holder_did: IdentityId,
        asset_id: &AssetId,
        holder_is_the_sender: bool,
        weight_meter: &mut WeightMeter,
    ) -> bool {
        let asset_compliance = AssetCompliances::<T>::get(asset_id);

        if asset_compliance.paused || asset_compliance.requirements.is_empty() {
            return true;
        }

        for requirement in &asset_compliance.requirements {
            let req_conditions = {
                if holder_is_the_sender {
                    &requirement.sender_conditions
                } else {
                    &requirement.receiver_conditions
                }
            };

            match Self::are_all_conditions_satisfied(
                asset_id,
                holder_did,
                req_conditions,
                weight_meter,
            ) {
                Ok(condition_satisfied) => {
                    if condition_satisfied {
                        return true;
                    }
                }
                Err(_) => return false,
            }
        }

        false
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn setup_asset_compliance(
        caller_did: IdentityId,
        asset_id: AssetId,
        n: u32,
        pause_compliance: bool,
    ) {
        benchmarking::setup_asset_compliance::<T>(caller_did, asset_id, n, pause_compliance);
    }
}

//==========================================================================
// All RPC functions!
//==========================================================================

impl<T: Config> Pallet<T> {
    /// Returns a [`ComplianceReport`] for the given `asset_id`.
    pub fn compliance_report(
        asset_id: &AssetId,
        sender_identity: &IdentityId,
        receiver_identity: &IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> Result<ComplianceReport, DispatchError> {
        let asset_compliance = AssetCompliances::<T>::get(asset_id);

        if asset_compliance.requirements.is_empty() {
            return Ok(ComplianceReport::new(
                Vec::new(),
                true,
                asset_compliance.paused,
            ));
        }

        let mut any_requirement_satisfied = false;
        // Get the [`RequirementReport`] for each requirement
        let mut requirements_report = Vec::new();
        for requirement in asset_compliance.requirements {
            // The requirement is satisfied only if all sender and receiver conditions hold.
            let mut requirement_satisfied = true;
            // Get the [`ConditionrReport`] for all sender conditions
            let sender_conditions_report = Self::get_conditions_report(
                asset_id,
                *sender_identity,
                requirement.sender_conditions,
                &mut requirement_satisfied,
                weight_meter,
            )?;
            // Get the [`ConditionrReport`] for all receiver conditions
            let receiver_conditions_report = Self::get_conditions_report(
                asset_id,
                *receiver_identity,
                requirement.receiver_conditions,
                &mut requirement_satisfied,
                weight_meter,
            )?;
            requirements_report.push(RequirementReport::new(
                sender_conditions_report,
                receiver_conditions_report,
                requirement.id,
                requirement_satisfied,
            ));
            any_requirement_satisfied = any_requirement_satisfied || requirement_satisfied;
        }

        Ok(ComplianceReport::new(
            requirements_report,
            any_requirement_satisfied,
            asset_compliance.paused,
        ))
    }

    /// Returns all [`ConditionReport`] for the given `conditions`.
    fn get_conditions_report(
        asset_id: &AssetId,
        identity: IdentityId,
        conditions: Vec<Condition>,
        requirement_satisfied: &mut bool,
        weight_meter: &mut WeightMeter,
    ) -> Result<Vec<ConditionReport>, DispatchError> {
        let mut conditions_report = Vec::new();
        for condition in conditions {
            let is_condition_satisfied = Self::is_condition_satisfied(
                asset_id,
                identity,
                &condition,
                &mut None,
                weight_meter,
            )?;
            conditions_report.push(ConditionReport::new(condition, is_condition_satisfied));
            *requirement_satisfied = *requirement_satisfied && is_condition_satisfied;
        }
        Ok(conditions_report)
    }
}
