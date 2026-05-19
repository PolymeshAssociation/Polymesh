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

use frame_support::ensure;
use frame_support::pallet_prelude::{DispatchError, DispatchResult};
use sp_runtime::traits::SaturatedConversion;
use sp_std::prelude::*;

use pallet_base::{ensure_string_limited, try_next_pre};
use polymesh_primitives::identity_claim::CustomClaimTypeId;
use polymesh_primitives::protocol_fee::ProtocolOp;
use polymesh_primitives::traits::group::GroupTrait;
use polymesh_primitives::{
    CddId, Claim, ClaimType, IdentityClaim, IdentityId, Scope, SecondaryKey, SystematicIssuers,
};

use crate::{
    Claim1stKey, Claim2ndKey, Claims, Config, CustomClaimIdSequence, CustomClaims,
    CustomClaimsInverse, DidRecords, Error, Event, Pallet,
};

impl<T: Config> Pallet<T> {
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
            .find(|c| Self::is_identity_claim_not_expired_at(c, now))
    }

    /// Check that the DID exists.
    ///
    /// A DID is considered "active" if existing.
    /// DIDs can only be created by permissioned
    /// DID registrars (formerly CDD providers), so existence implies onboarding.
    pub fn is_did_active(did: IdentityId) -> bool {
        <DidRecords<T>>::contains_key(did)
    }

    /// Check if a DID is locked.
    /// TODO: Implement DID locking. For now, always returns false.
    pub fn is_did_locked(_did: IdentityId) -> bool {
        false
    }

    /// Returns true if `id_claim` is not expired at `moment`.
    #[inline]
    fn is_identity_claim_not_expired_at(id_claim: &IdentityClaim, moment: T::Moment) -> bool {
        if let Some(expiry) = id_claim.expiry {
            expiry > moment.saturated_into::<u64>()
        } else {
            true
        }
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
        Claims::<T>::get(&pk, &sk)
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
                CustomClaims::<T>::contains_key(id),
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

        Claims::<T>::insert(&pk, &sk, id_claim.clone());
        Self::deposit_event(Event::ClaimAdded(target, id_claim));
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

    /// It ensures that CDD claim issuer is a valid DID registrar before adding the claim.
    ///
    /// # Errors
    /// - 'UnAuthorizedDidRegistrar' is returned if `issuer` is not a DID registrar.
    pub(crate) fn base_add_cdd_claim(
        target: IdentityId,
        claim: Claim,
        issuer: IdentityId,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        Self::ensure_authorized_did_registrar(issuer)?;

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
        let claim = Claims::<T>::take(&pk, &sk).ok_or(Error::<T>::ClaimDoesNotExist)?;
        // Emit claim revoked event.
        Self::deposit_event(Event::ClaimRevoked(target, claim));
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

    /// Ensures that the did is an active DID registrar (formerly CDD Provider).
    fn ensure_authorized_did_registrar(did: IdentityId) -> DispatchResult {
        ensure!(
            T::DidRegistrars::get_members().contains(&did),
            Error::<T>::UnAuthorizedDidRegistrar
        );
        Ok(())
    }

    /// Ensures that the caller is an active DID registrar and creates a new did for the target.
    ///
    /// # Failure
    /// - `origin` has to be an active DID registrar. Inactive registrars cannot add new
    /// identities.
    /// - `target_account` (primary key of the new Identity) can be linked to just one and only
    /// one identity.
    /// - `secondary_keys` list of secondary keys with their permissions.
    pub fn base_register_did(
        origin: T::RuntimeOrigin,
        target_account: T::AccountId,
        secondary_keys: Vec<SecondaryKey<T::AccountId>>,
    ) -> Result<(), DispatchError> {
        let registrar_did = Self::ensure_perms(origin)?;

        // Sender has to be part of DID registrars (formerly CddServiceProviders)
        Self::ensure_authorized_did_registrar(registrar_did)?;

        // Check limit for the SK's permissions.
        for sk in &secondary_keys {
            Self::ensure_perms_length_limited(&sk.permissions)?;
        }

        // Register Identity
        Self::register_did_without_cdd(
            target_account,
            secondary_keys,
            Some(ProtocolOp::IdentityRegisterDid),
        )?;

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
            !CustomClaimsInverse::<T>::contains_key(&ty),
            Error::<T>::CustomClaimTypeAlreadyExists
        );

        let id = CustomClaimIdSequence::<T>::try_mutate(try_next_pre::<T, _>)?;
        CustomClaimsInverse::<T>::insert(&ty, id);
        CustomClaims::<T>::insert(id, ty);
        Ok(id)
    }
}
