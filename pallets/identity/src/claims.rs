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

use crate::{Claim1stKey, Claim2ndKey, Claims, DidRecords, Error, Module};
use core::convert::From;
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure, fail, StorageDoubleMap, StorageMap,
};
use frame_system::ensure_root;
pub use polymesh_common_utilities::traits::identity::WeightInfo;
use polymesh_common_utilities::{
    protocol_fee::ProtocolOp,
    traits::{
        asset::AssetSubTrait,
        group::{GroupTrait, InactiveMember},
        identity::{Config, RawEvent},
    },
    SystematicIssuers, SYSTEMATIC_ISSUERS,
};
use polymesh_primitives::{
    investor_zkproof_data::InvestorZKProofData as InvestorZKProof, valid_proof_of_investor, CddId,
    Claim, ClaimType, IdentityClaim, IdentityId, InvestorUid, Scope, ScopeId, SecondaryKey, Ticker,
};
use sp_runtime::traits::{CheckedAdd, SaturatedConversion, Zero};
use sp_std::prelude::*;

impl<T: Config> Module<T> {
    /// Ensure that any `Scope::Custom(data)` is limited to 32 characters.
    pub fn ensure_custom_scopes_limited(claim: &Claim) -> DispatchResult {
        if let Some(Scope::Custom(data)) = claim.as_scope() {
            ensure!(data.len() <= 32, Error::<T>::CustomScopeTooLong);
        }
        Ok(())
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

    /// See `Self::fetch_cdd`.
    #[inline]
    pub fn has_valid_cdd(claim_for: IdentityId) -> bool {
        // It will never happen in production but helpful during testing.
        #[cfg(feature = "no_cdd")]
        if T::CddServiceProviders::get_members().is_empty() {
            return true;
        }

        Self::base_fetch_cdd(claim_for, T::Moment::zero(), None).is_some()
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
        Self::base_fetch_cdd(claim_for, leeway, None)
    }

    fn base_fetch_cdd(
        claim_for: IdentityId,
        leeway: T::Moment,
        filter_cdd_id: Option<CddId>,
    ) -> Option<IdentityId> {
        Self::base_fetch_valid_cdd_claims(claim_for, leeway, filter_cdd_id)
            .map(|id_claim| id_claim.claim_issuer)
            .next()
    }

    pub fn base_fetch_valid_cdd_claims(
        claim_for: IdentityId,
        leeway: T::Moment,
        filter_cdd_id: Option<CddId>,
    ) -> impl Iterator<Item = IdentityClaim> {
        let exp_with_leeway = <pallet_timestamp::Pallet<T>>::get()
            .checked_add(&leeway)
            .unwrap_or_default();

        // Supressing `mut` warning since we need mut in `runtime-benchmarks` feature but not otherwise.
        #[allow(unused_mut)]
        let mut active_cdds_temp = T::CddServiceProviders::get_active_members();

        // For the benchmarks, self cdd claims are allowed and hence the claim target is added to the cdd providers list.
        #[cfg(feature = "runtime-benchmarks")]
        active_cdds_temp.push(claim_for);

        let active_cdds = active_cdds_temp;
        let inactive_not_expired_cdds = T::CddServiceProviders::get_inactive_members()
            .into_iter()
            .filter(|cdd| !T::CddServiceProviders::is_member_expired(cdd, exp_with_leeway))
            .collect::<Vec<_>>();

        Self::fetch_base_claims(claim_for, ClaimType::CustomerDueDiligence).filter(
            move |id_claim| {
                if let Some(cdd_id) = &filter_cdd_id {
                    if let Claim::CustomerDueDiligence(claim_cdd_id) = &id_claim.claim {
                        if claim_cdd_id != cdd_id {
                            return false;
                        }
                    }
                }

                Self::is_identity_cdd_claim_valid(
                    id_claim,
                    exp_with_leeway,
                    &active_cdds,
                    &inactive_not_expired_cdds,
                )
            },
        )
    }

    /// A CDD claims is considered valid if:
    /// * Claim is not expired at `exp_with_leeway` moment.
    /// * Its issuer is valid, that means:
    ///   * Issuer is an active CDD provider, or
    ///   * Issuer is an inactive CDD provider but claim was updated/created before that it was
    ///   deactivated.
    fn is_identity_cdd_claim_valid(
        id_claim: &IdentityClaim,
        exp_with_leeway: T::Moment,
        active_cdds: &[IdentityId],
        inactive_not_expired_cdds: &[InactiveMember<T::Moment>],
    ) -> bool {
        Self::is_identity_claim_not_expired_at(id_claim, exp_with_leeway)
            && (active_cdds.contains(&id_claim.claim_issuer)
                || SYSTEMATIC_ISSUERS
                    .iter()
                    .any(|si| si.as_id() == id_claim.claim_issuer)
                || inactive_not_expired_cdds
                    .iter()
                    .filter(|cdd| cdd.id == id_claim.claim_issuer)
                    .any(|cdd| {
                        id_claim.last_update_date < cdd.deactivated_at.saturated_into::<u64>()
                    }))
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
        Claims::contains_key(&pk, &sk).then(|| Claims::get(&pk, &sk))
    }

    /// It adds a new claim without any previous security check.
    pub fn base_add_claim(
        target: IdentityId,
        claim: Claim,
        issuer: IdentityId,
        expiry: Option<T::Moment>,
    ) {
        let inner_scope = claim.as_scope().cloned();
        Self::base_add_claim_with_scope(target, claim, inner_scope, issuer, expiry)
    }

    /// Adds claims with no inner scope.
    fn base_add_claim_with_scope(
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
    crate fn base_add_cdd_claim(
        target: IdentityId,
        claim: Claim,
        issuer: IdentityId,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        Self::ensure_authorized_cdd_provider(issuer)?;
        // Ensure cdd_id uniqueness for a given target DID.
        Self::ensure_cdd_id_validness(&claim, target)?;

        Self::base_add_claim(target, claim, issuer, expiry);
        Ok(())
    }

    /// Enforce CDD_ID uniqueness for a given target DID.
    ///
    /// # Errors
    /// - `CDDIdNotUniqueForIdentity` is returned when new cdd claim's cdd_id doesn't match the existing cdd claim's cdd_id.
    fn ensure_cdd_id_validness(claim: &Claim, target: IdentityId) -> DispatchResult {
        if let Claim::CustomerDueDiligence(cdd_id) = claim {
            ensure!(
                cdd_id.is_default_cdd()
                    || Self::base_fetch_valid_cdd_claims(target, 0u32.into(), None)
                        .filter_map(|c| match c.claim {
                            Claim::CustomerDueDiligence(c_id) => Some(c_id),
                            _ => None,
                        })
                        .all(|c_id| c_id.is_default_cdd() || c_id == *cdd_id),
                Error::<T>::CDDIdNotUniqueForIdentity
            );
        }
        Ok(())
    }

    /// Decodes needed fields from `claim` and `proof`, and ensures that both are in the same
    /// version.
    ///
    /// # Errors
    /// - `ClaimVariantNotAllowed` if `claim` is not a `Claim::InvestorUniqueness` neither
    /// `Claim::InvestorUniquenessV2`.
    /// - `ClaimAndProofVersionsDoNotMatch` if `claim` and `proof` are different versions.
    fn decode_investor_uniqueness_claim<'a>(
        claim: &'a Claim,
        proof: &'_ InvestorZKProof,
        scope: Option<&'a Scope>,
    ) -> Result<(&'a Scope, ScopeId, &'a CddId), DispatchError> {
        let decode = match &claim {
            Claim::InvestorUniqueness(scope, scope_id, cdd_id) => {
                ensure!(
                    matches!(proof, InvestorZKProof::V1(..)),
                    Error::<T>::ClaimAndProofVersionsDoNotMatch
                );
                (scope, scope_id.clone(), cdd_id)
            }
            Claim::InvestorUniquenessV2(cdd_id) => match proof {
                InvestorZKProof::V2(inner_proof) => (
                    scope.ok_or(Error::<T>::InvalidScopeClaim)?,
                    inner_proof.0.scope_id.compress().to_bytes().into(),
                    cdd_id,
                ),
                _ => fail!(Error::<T>::ClaimAndProofVersionsDoNotMatch),
            },
            _ => fail!(Error::<T>::ClaimVariantNotAllowed),
        };

        Ok(decode)
    }

    /// # Errors
    /// - 'ConfidentialScopeClaimNotAllowed` if :
    ///     - Sender is not the issuer. That claim can be only added by your-self.
    ///     - If claim is not valid.
    /// - 'InvalidCDDId' if you are not the owner of that CDD_ID.
    ///
    crate fn base_add_investor_uniqueness_claim(
        origin: T::Origin,
        target: IdentityId,
        claim: Claim,
        scope_opt: Option<Scope>,
        proof: InvestorZKProof,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        Self::ensure_custom_scopes_limited(&claim)?;

        // Decode needed fields and ensures `claim` is `InvestorUniqueness*`.
        let (scope, scope_id, cdd_id) =
            Self::decode_investor_uniqueness_claim(&claim, &proof, scope_opt.as_ref())?;

        // Only owner of the identity can add that confidential claim.
        let issuer = Self::ensure_signed_and_validate_claim_target(origin, target)?;
        ensure!(
            issuer == target,
            Error::<T>::ConfidentialScopeClaimNotAllowed
        );
        // Verify the owner of that CDD_ID.
        ensure!(
            Self::base_fetch_cdd(target, T::Moment::zero(), Some(*cdd_id)).is_some(),
            Error::<T>::InvalidCDDId
        );

        // Verify the confidential claim.
        ensure!(
            valid_proof_of_investor::evaluate_claim(scope, &claim, &target, &proof),
            Error::<T>::InvalidScopeClaim
        );

        if let Scope::Ticker(ticker) = scope {
            // Ensure uniqueness claims are allowed.
            T::AssetSubTraitTarget::ensure_investor_uniqueness_claims_allowed(ticker)?;

            // Update the balance of the IdentityId under the ScopeId provided in claim data.
            T::AssetSubTraitTarget::update_balance_of_scope_id(scope_id, target, *ticker);
        }

        let scope = Some(scope.clone());
        Self::base_add_claim_with_scope(target, claim, scope, issuer, expiry);
        Ok(())
    }

    /// It removes a claim from `target` which was issued by `issuer` without any security check.
    crate fn base_revoke_claim(
        target: IdentityId,
        claim_type: ClaimType,
        issuer: IdentityId,
        scope: Option<Scope>,
    ) -> DispatchResult {
        let (pk, sk) = Self::get_claim_keys(target, claim_type, issuer, scope);

        let investor_unique_scope_id = match Claims::get(&pk, &sk).claim {
            Claim::InvestorUniqueness(_, scope_id, _) => Some(scope_id),
            Claim::InvestorUniquenessV2(..) => match &sk.scope {
                Some(Scope::Ticker(ticker)) => {
                    Some(T::AssetSubTraitTarget::scope_id(ticker, &target))
                }
                _ => None,
            },
            _ => None,
        };

        // Only if claim is a `InvestorUniqueness*`.
        if let Some(scope_id) = investor_unique_scope_id {
            // Ensure the target is the issuer of the claim.
            ensure!(
                target == issuer,
                Error::<T>::ConfidentialScopeClaimNotAllowed
            );

            // Ensure that the target has balance at scope = 0.
            ensure!(
                T::AssetSubTraitTarget::balance_of_at_scope(&scope_id, &target) == Zero::zero(),
                Error::<T>::TargetHasNonZeroBalanceAtScopeId
            );
        }

        let claim = Claims::take(&pk, &sk);
        Self::deposit_event(RawEvent::ClaimRevoked(target, claim));
        Ok(())
    }

    /// Ensure that the origin is signed and that the given `target` is already in the system.
    crate fn ensure_signed_and_validate_claim_target(
        origin: T::Origin,
        target: IdentityId,
    ) -> Result<IdentityId, DispatchError> {
        let primary_did = Self::ensure_perms(origin)?;
        ensure!(
            DidRecords::<T>::contains_key(target),
            Error::<T>::DidMustAlreadyExist
        );
        Ok(primary_did)
    }

    /// Checks whether the sender and the receiver of a transfer have valid investor uniqueness claims for a given ticker
    pub fn verify_iu_claims_for_transfer(
        ticker: Ticker,
        from_did: IdentityId,
        to_did: IdentityId,
    ) -> bool {
        let asset_scope = Some(Scope::from(ticker));
        Self::base_verify_iu_claim(asset_scope.clone(), from_did)
            && Self::base_verify_iu_claim(asset_scope, to_did)
    }

    /// Checks whether the identity has a valid investor uniqueness claim for a given ticker
    pub fn verify_iu_claim(ticker: Ticker, did: IdentityId) -> bool {
        let asset_scope = Some(Scope::from(ticker));
        Self::base_verify_iu_claim(asset_scope, did)
    }

    crate fn base_verify_iu_claim(scope: Option<Scope>, did: IdentityId) -> bool {
        Self::fetch_claim(did, ClaimType::InvestorUniqueness, did, scope.clone()).is_some()
            || Self::fetch_claim(did, ClaimType::InvestorUniquenessV2, did, scope).is_some()
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
        origin: T::Origin,
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
    crate fn base_invalidate_cdd_claims(
        origin: T::Origin,
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
            let cdd_id = CddId::new_v1(new_member.clone(), InvestorUid::from(new_member.as_ref()));
            let cdd_claim = Claim::CustomerDueDiligence(cdd_id);
            Self::base_add_claim(*new_member, cdd_claim, issuer.as_id(), None);
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
}
