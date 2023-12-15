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

use crate::{
    Claim1stKey, Claim2ndKey, Claims, CustomClaimIdSequence, CustomClaims, CustomClaimsInverse,
    DidRecords, Error, Event, Module, ParentDid,
};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure, StorageDoubleMap, StorageMap, StorageValue,
};
use frame_system::ensure_root;
use pallet_base::{ensure_string_limited, try_next_pre};

use polymesh_common_utilities::{
    protocol_fee::ProtocolOp,
    traits::{
        group::{GroupTrait, InactiveMember},
        identity::{Config, RawEvent},
    },
    SystematicIssuers,
};
use polymesh_primitives::identity_claim::CustomClaimTypeId;
use polymesh_primitives::{
    CddId, Claim, ClaimType, IdentityClaim, IdentityId, Scope, SecondaryKey,
};
use sp_runtime::traits::{CheckedAdd, SaturatedConversion, Zero};
use sp_std::prelude::*;

struct CddClaimChecker<T: Config> {
    filter_cdd_id: Option<CddId>,
    exp_with_leeway: T::Moment,
    active_cdds: Vec<IdentityId>,
    inactive_cdds: Option<Vec<InactiveMember<T::Moment>>>,
}

impl<T: Config> CddClaimChecker<T> {
    pub fn new(_claim_for: IdentityId, leeway: T::Moment, filter_cdd_id: Option<CddId>) -> Self {
        let exp_with_leeway = <pallet_timestamp::Pallet<T>>::get()
            .checked_add(&leeway)
            .unwrap_or_default();

        // Supressing `mut` warning since we need mut in `runtime-benchmarks` feature but not otherwise.
        #[allow(unused_mut)]
        let mut active_cdds = T::CddServiceProviders::get_active_members();

        // For the benchmarks, self cdd claims are allowed and hence the claim target is added to the cdd providers list.
        #[cfg(feature = "runtime-benchmarks")]
        active_cdds.push(_claim_for);

        Self {
            filter_cdd_id,
            exp_with_leeway,
            active_cdds,
            inactive_cdds: None,
        }
    }

    /// It returns true if `id_claim` is not expired at `moment`.
    #[inline]
    fn is_identity_claim_not_expired_at(id_claim: &IdentityClaim, moment: T::Moment) -> bool {
        if let Some(expiry) = id_claim.expiry {
            expiry > moment.saturated_into::<u64>()
        } else {
            true
        }
    }

    /// Issuer is an active CDD provider.
    fn is_active(&self, id_claim: &IdentityClaim) -> bool {
        self.active_cdds.contains(&id_claim.claim_issuer)
    }

    /// Issuer is on of SystematicIssuers::CDDProvider or SystematicIssuers::Committee
    fn is_systematic_cdd_provider(&self, id_claim: &IdentityClaim) -> bool {
        SystematicIssuers::CDDProvider.as_id() == id_claim.claim_issuer
            || SystematicIssuers::Committee.as_id() == id_claim.claim_issuer
    }

    /// Issuer is an inactive CDD provider but claim was updated/created before that it was
    /// deactivated.
    fn is_inactive(&mut self, id_claim: &IdentityClaim) -> bool {
        // Lazy build list of inactive cdd providers.
        let inactive_cdds = self.inactive_cdds.get_or_insert_with(|| {
            T::CddServiceProviders::get_inactive_members()
                .into_iter()
                .filter(|cdd| !T::CddServiceProviders::is_member_expired(cdd, self.exp_with_leeway))
                .collect::<Vec<_>>()
        });

        inactive_cdds
            .iter()
            .filter(|cdd| cdd.id == id_claim.claim_issuer)
            .any(|cdd| id_claim.last_update_date < cdd.deactivated_at.saturated_into::<u64>())
    }

    /// A CDD claims is considered valid if:
    /// * Claim is not expired at `exp_with_leeway` moment.
    /// * Its issuer is valid, that means:
    ///   * Issuer is an active CDD provider, or
    ///   * Issuer is the SystematicIssuers::CDDProvider, or
    ///   * Issuer is an inactive CDD provider but claim was updated/created before that it was
    ///   deactivated.
    fn is_cdd_claim_valid(&mut self, id_claim: &IdentityClaim) -> bool {
        Self::is_identity_claim_not_expired_at(id_claim, self.exp_with_leeway)
            && (self.is_active(id_claim)
                || self.is_systematic_cdd_provider(id_claim)
                || self.is_inactive(id_claim))
    }

    fn filter_cdd_claims(&mut self, id_claim: &IdentityClaim) -> bool {
        if let Some(cdd_id) = &self.filter_cdd_id {
            if let Claim::CustomerDueDiligence(claim_cdd_id) = &id_claim.claim {
                if claim_cdd_id != cdd_id {
                    return false;
                }
            }
        }

        self.is_cdd_claim_valid(id_claim)
    }
}

impl<T: Config> Module<T> {
    /// Ensure that any `Scope::Custom(data)` is limited to 32 characters.
    pub fn ensure_custom_scopes_limited(claim: &Claim) -> DispatchResult {
        if let Some(Scope::Custom(data)) = claim.as_scope() {
            ensure!(data.len() <= 32, Error::<T>::CustomScopeTooLong);
        }
        Ok(())
    }

    /// It fetches an specific `claim_type` claim type for target identity `id`, which was issued
    /// by `issuer`.
    /// It only returns non-expired claims.
    pub fn fetch_claim(
        id: IdentityId,
        claim_type: ClaimType,
        issuer: IdentityId,
        scope: Option<Scope>,
    ) -> Option<IdentityClaim> {
        let now = <pallet_timestamp::Pallet<T>>::get();

        Self::fetch_base_claim_with_issuer(id, claim_type, issuer, scope)
            .into_iter()
            .find(|c| CddClaimChecker::<T>::is_identity_claim_not_expired_at(c, now))
    }

    /// See `Self::fetch_cdd`.
    pub fn has_valid_cdd(claim_for: IdentityId) -> bool {
        // It will never happen in production but helpful during testing.
        #[cfg(feature = "no_cdd")]
        if T::CddServiceProviders::get_members().is_empty() {
            return true;
        }

        Self::base_fetch_cdd(claim_for, T::Moment::zero(), None, true).is_some()
    }

    /// It returns the CDD identity which issued the current valid CDD claim for `claim_for`
    /// identity.
    /// # Parameters
    /// * `leeway` : This leeway is added to now() before check if claim is expired.
    ///
    /// # Safety
    ///
    /// No state change is allowed in this function because this function is used within the RPC
    /// calls.
    pub fn fetch_cdd(claim_for: IdentityId, leeway: T::Moment) -> Option<IdentityId> {
        Self::base_fetch_cdd(claim_for, leeway, None, true)
    }

    fn base_fetch_cdd(
        claim_for: IdentityId,
        leeway: T::Moment,
        filter_cdd_id: Option<CddId>,
        include_parent: bool,
    ) -> Option<IdentityId> {
        Self::base_fetch_valid_cdd_claims(claim_for, leeway, filter_cdd_id, include_parent)
            .map(|id_claim| id_claim.claim_issuer)
            .next()
    }

    // Returns a lazy iterator that will return the CDD claims from the
    // parent of `did` if they are a child identity.
    //
    // If `include_parent` is `false` then the iterator will not return claims
    // from the parent.
    pub fn base_fetch_parent_cdd_claims(
        did: IdentityId,
        include_parent: bool,
    ) -> impl Iterator<Item = IdentityClaim> {
        let mut first_call = include_parent;
        let mut parent_claims = None;
        core::iter::from_fn(move || -> Option<IdentityClaim> {
            // The first time this iterator function is called
            // we will initialize the `parent_claims` iterator.
            if first_call {
                first_call = false;
                parent_claims = ParentDid::get(did).map(|parent_did| {
                    Self::fetch_base_claims(parent_did, ClaimType::CustomerDueDiligence)
                });
            }

            // If `parent_claims` is `None` then this returns early with `None`.
            let claim = parent_claims.as_mut()?.next();
            if claim.is_none() {
                parent_claims = None;
            }
            claim
        })
    }

    pub(crate) fn base_fetch_valid_cdd_claims(
        claim_for: IdentityId,
        leeway: T::Moment,
        filter_cdd_id: Option<CddId>,
        include_parent: bool,
    ) -> impl Iterator<Item = IdentityClaim> {
        let mut cdd_checker = CddClaimChecker::<T>::new(claim_for, leeway, filter_cdd_id);

        Self::fetch_base_claims(claim_for, ClaimType::CustomerDueDiligence)
            .chain(Self::base_fetch_parent_cdd_claims(
                claim_for,
                include_parent,
            ))
            .filter(move |id_claim| cdd_checker.filter_cdd_claims(id_claim))
    }

    /// It iterates over all claims of type `claim_type` for target `id` identity.
    /// Please note that it could return expired claims.
    fn fetch_base_claims<'a>(
        target: IdentityId,
        claim_type: ClaimType,
    ) -> impl Iterator<Item = IdentityClaim> + 'a {
        Claims::iter_prefix_values(Claim1stKey { target, claim_type })
    }

    /// It fetches an specific `claim_type` claim type for target identity `id`, which was issued
    /// by `issuer`.
    fn fetch_base_claim_with_issuer(
        target: IdentityId,
        claim_type: ClaimType,
        issuer: IdentityId,
        scope: Option<Scope>,
    ) -> Option<IdentityClaim> {
        let pk = Claim1stKey { target, claim_type };
        let sk = Claim2ndKey { issuer, scope };
        Claims::get(&pk, &sk)
    }

    /// It adds a new claim without any previous security check.
    pub fn base_add_claim(
        target: IdentityId,
        claim: Claim,
        issuer: IdentityId,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        let inner_scope = claim.as_scope().cloned();
        if let ClaimType::Custom(id) = claim.claim_type() {
            ensure!(
                CustomClaims::contains_key(id),
                Error::<T>::CustomClaimTypeDoesNotExist
            );
        }
        Self::unverified_add_claim_with_scope(target, claim, inner_scope, issuer, expiry);
        Ok(())
    }

    /// Adds claims with no inner scope.
    pub fn unverified_add_claim_with_scope(
        target: IdentityId,
        claim: Claim,
        scope: Option<Scope>,
        issuer: IdentityId,
        expiry: Option<T::Moment>,
    ) {
        let claim_type = claim.claim_type();
        let last_update_date = <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>();
        let issuance_date = Self::fetch_claim(target, claim_type, issuer, scope.clone())
            .map_or(last_update_date, |id_claim| id_claim.issuance_date);

        let expiry = expiry.map(|m| m.saturated_into::<u64>());
        let (pk, sk) = Self::get_claim_keys(target, claim_type, issuer, scope);
        let id_claim = IdentityClaim {
            claim_issuer: issuer,
            issuance_date,
            last_update_date,
            expiry,
            claim,
        };

        Claims::insert(&pk, &sk, id_claim.clone());
        Self::deposit_event(RawEvent::ClaimAdded(target, id_claim));
    }

    /// Returns claim keys.
    pub fn get_claim_keys(
        target: IdentityId,
        claim_type: ClaimType,
        issuer: IdentityId,
        scope: Option<Scope>,
    ) -> (Claim1stKey, Claim2ndKey) {
        let pk = Claim1stKey { target, claim_type };
        let sk = Claim2ndKey { issuer, scope };
        (pk, sk)
    }

    /// It ensures that CDD claim issuer is a valid CDD provider before add the claim.
    ///
    /// # Errors
    /// - 'UnAuthorizedCddProvider' is returned if `issuer` is not a CDD provider.
    pub(crate) fn base_add_cdd_claim(
        target: IdentityId,
        claim: Claim,
        issuer: IdentityId,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        Self::ensure_authorized_cdd_provider(issuer)?;

        Self::base_add_claim(target, claim, issuer, expiry)
    }

    /// It removes a claim from `target` which was issued by `issuer` without any security check.
    pub(crate) fn base_revoke_claim(
        target: IdentityId,
        claim_type: ClaimType,
        issuer: IdentityId,
        scope: Option<Scope>,
    ) -> DispatchResult {
        let (pk, sk) = Self::get_claim_keys(target, claim_type, issuer, scope);
        // Remove the claim.
        let claim = Claims::take(&pk, &sk).ok_or(Error::<T>::ClaimDoesNotExist)?;
        // Emit claim revoked event.
        Self::deposit_event(RawEvent::ClaimRevoked(target, claim));
        Ok(())
    }

    /// Ensure that the origin is signed and that the given `target` is already in the system.
    pub(crate) fn ensure_signed_and_validate_claim_target(
        origin: T::RuntimeOrigin,
        target: IdentityId,
    ) -> Result<IdentityId, DispatchError> {
        let primary_did = Self::ensure_perms(origin)?;
        ensure!(
            DidRecords::<T>::contains_key(target),
            Error::<T>::DidMustAlreadyExist
        );
        Ok(primary_did)
    }

    /// RPC call to know whether the given did has valid cdd claim or not
    pub fn is_identity_has_valid_cdd(
        target: IdentityId,
        leeway: Option<T::Moment>,
    ) -> Option<IdentityId> {
        Self::fetch_cdd(target, leeway.unwrap_or_default())
    }

    /// Ensures that the did is an active CDD Provider.
    fn ensure_authorized_cdd_provider(did: IdentityId) -> DispatchResult {
        ensure!(
            T::CddServiceProviders::get_members().contains(&did),
            Error::<T>::UnAuthorizedCddProvider
        );
        Ok(())
    }

    /// Ensures that the caller is an active CDD provider and creates a new did for the target.
    /// This function returns the new did of the target.
    ///
    /// # Failure
    /// - `origin` has to be a active CDD provider. Inactive CDD providers cannot add new
    /// claims.
    /// - `target_account` (primary key of the new Identity) can be linked to just one and only
    /// one identity.
    /// - External secondary keys can be linked to just one identity.
    pub fn base_cdd_register_did(
        origin: T::RuntimeOrigin,
        target_account: T::AccountId,
        secondary_keys: Vec<SecondaryKey<T::AccountId>>,
    ) -> Result<(IdentityId, IdentityId), DispatchError> {
        let cdd_did = Self::ensure_perms(origin)?;

        // Sender has to be part of CDDProviders
        Self::ensure_authorized_cdd_provider(cdd_did)?;

        // Check limit for the SK's permissions.
        for sk in &secondary_keys {
            Self::ensure_perms_length_limited(&sk.permissions)?;
        }

        // Register Identity
        let target_did = Self::_register_did(
            target_account,
            secondary_keys,
            Some(ProtocolOp::IdentityCddRegisterDid),
        )?;

        Ok((cdd_did, target_did))
    }

    /// Invalidates any claim generated by `cdd` from `disable_from` timestamps.
    pub(crate) fn base_invalidate_cdd_claims(
        origin: T::RuntimeOrigin,
        cdd: IdentityId,
        disable_from: T::Moment,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        ensure_root(origin)?;

        let now = <pallet_timestamp::Pallet<T>>::get();
        ensure!(
            T::CddServiceProviders::get_valid_members_at(now).contains(&cdd),
            Error::<T>::UnAuthorizedCddProvider
        );

        T::CddServiceProviders::disable_member(cdd, expiry, Some(disable_from))?;
        Self::deposit_event(RawEvent::CddClaimsInvalidated(cdd, disable_from));
        Ok(())
    }

    /// Adds systematic CDD claims.
    pub fn add_systematic_cdd_claims(targets: &[IdentityId], issuer: SystematicIssuers) {
        for new_member in targets {
            let cdd_claim = Claim::CustomerDueDiligence(CddId::default());
            let _ = Self::base_add_claim(*new_member, cdd_claim, issuer.as_id(), None);
        }
    }

    /// Removes systematic CDD claims.
    pub fn revoke_systematic_cdd_claims(targets: &[IdentityId], issuer: SystematicIssuers) {
        targets.iter().for_each(|removed_member| {
            let _ = Self::base_revoke_claim(
                *removed_member,
                ClaimType::CustomerDueDiligence,
                issuer.as_id(),
                None,
            );
        });
    }

    pub fn base_register_custom_claim_type(
        origin: T::RuntimeOrigin,
        ty: Vec<u8>,
    ) -> DispatchResult {
        let did = Self::ensure_perms(origin)?;
        let id = Self::unsafe_register_custom_claim_type(ty.clone())?;
        Self::deposit_event(Event::<T>::CustomClaimTypeAdded(did, id, ty));
        Ok(())
    }

    fn unsafe_register_custom_claim_type(ty: Vec<u8>) -> Result<CustomClaimTypeId, DispatchError> {
        ensure_string_limited::<T>(&ty)?;
        ensure!(
            !CustomClaimsInverse::contains_key(&ty),
            Error::<T>::CustomClaimTypeAlreadyExists
        );

        let id = CustomClaimIdSequence::try_mutate(try_next_pre::<T, _>)?;
        CustomClaimsInverse::insert(&ty, id);
        CustomClaims::insert(id, ty);
        Ok(id)
    }

    /// Returns all valid [`IdentityClaim`] of type `CustomerDueDiligence` for the given `target_identity`.
    pub fn valid_cdd_claims(
        target_identity: IdentityId,
        cdd_checker_leeway: Option<T::Moment>,
    ) -> Vec<IdentityClaim> {
        Self::base_fetch_valid_cdd_claims(
            target_identity,
            cdd_checker_leeway.unwrap_or_default(),
            None,
            true,
        )
        .collect()
    }
}
