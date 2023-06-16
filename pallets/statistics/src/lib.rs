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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::{decl_error, decl_module, decl_storage, ensure, BoundedBTreeSet};
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};

pub use polymesh_common_utilities::traits::statistics::{Config, Event, WeightInfo};
use polymesh_primitives::statistics::{
    AssetScope, Percentage, Stat1stKey, Stat2ndKey, StatOpType, StatType, StatUpdate,
};
use polymesh_primitives::transfer_compliance::{
    AssetTransferCompliance, TransferCondition, TransferConditionExemptKey, TransferConditionResult,
};
use polymesh_primitives::{
    storage_migration_ver, Balance, IdentityId, ScopeId, Ticker, WeightMeter,
};

type Identity<T> = pallet_identity::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;

storage_migration_ver!(1);

decl_storage! {
    trait Store for Module<T: Config> as Statistics {
        /// Active stats for a ticker/company.  There should be a max limit on the number of active stats for a ticker/company.
        pub ActiveAssetStats get(fn active_asset_stats): map hasher(blake2_128_concat) AssetScope => BoundedBTreeSet<StatType, T::MaxStatsPerAsset>;
        /// Asset stats.
        pub AssetStats get(fn asset_stats):
          double_map
            hasher(blake2_128_concat) Stat1stKey,
            hasher(blake2_128_concat) Stat2ndKey => u128;
        /// Asset transfer compliance for a ticker (AssetScope -> AssetTransferCompliance)
        pub AssetTransferCompliances get(fn asset_transfer_compliance): map hasher(blake2_128_concat) AssetScope => AssetTransferCompliance<T::MaxTransferConditionsPerAsset>;
        /// Entities exempt from a Transfer Compliance rule.
        pub TransferConditionExemptEntities get(fn transfer_condition_exempt_entities):
            double_map
                hasher(blake2_128_concat) TransferConditionExemptKey,
                hasher(blake2_128_concat) ScopeId
            =>
                bool;

        /// Storage migration version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1)): Version;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {
        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        const MaxStatsPerAsset: u32 = T::MaxStatsPerAsset::get();
        const MaxTransferConditionsPerAsset: u32 = T::MaxTransferConditionsPerAsset::get();

        /// Set the active asset stat_types.
        ///
        /// # Arguments
        /// - `origin` - a signer that has permissions to act as an agent of `asset`.
        /// - `asset` - the asset to change the active stats on.
        /// - `stat_types` - the new stat types to replace any existing types.
        ///
        /// # Errors
        /// - `StatTypeLimitReached` - too many stat types enabled for the `asset`.
        /// - `CannotRemoveStatTypeInUse` - can not remove a stat type that is in use by transfer conditions.
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset`.
        ///
        /// # Permissions
        /// - Agent
        /// - Asset
        #[weight = <T as Config>::WeightInfo::set_active_asset_stats(stat_types.len() as u32)]
        pub fn set_active_asset_stats(origin, asset: AssetScope, stat_types: BTreeSet<StatType>) {
            Self::base_set_active_asset_stats(origin, asset, stat_types)?;
        }

        /// Allow a trusted issuer to init/resync ticker/company stats.
        ///
        /// # Arguments
        /// - `origin` - a signer that has permissions to act as an agent of `asset`.
        /// - `asset` - the asset to change the active stats on.
        /// - `stat_type` - stat type to update.
        /// - `values` - Updated values for `stat_type`.
        ///
        /// # Errors
        /// - `StatTypeMissing` - `stat_type` is not enabled for the `asset`.
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset`.
        ///
        /// # Permissions
        /// - Agent
        /// - Asset
        #[weight = <T as Config>::WeightInfo::batch_update_asset_stats(values.len() as u32)]
        pub fn batch_update_asset_stats(origin, asset: AssetScope, stat_type: StatType, values: BTreeSet<StatUpdate>) {
            Self::base_batch_update_asset_stats(origin, asset, stat_type, values)?;
        }

        /// Set asset transfer compliance rules.
        ///
        /// # Arguments
        /// - `origin` - a signer that has permissions to act as an agent of `asset`.
        /// - `asset` - the asset to change the active stats on.
        /// - `transfer_conditions` - the new transfer condition to replace any existing conditions.
        ///
        /// # Errors
        /// - `TransferConditionLimitReached` - too many transfer condititon enabled for `asset`.
        /// - `StatTypeMissing` - a transfer condition requires a stat type that is not enabled for the `asset`.
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset`.
        ///
        /// # Permissions
        /// - Agent
        /// - Asset
        #[weight = <T as Config>::WeightInfo::set_asset_transfer_compliance(transfer_conditions.len() as u32)]
        pub fn set_asset_transfer_compliance(origin, asset: AssetScope, transfer_conditions: BTreeSet<TransferCondition>) {
            Self::base_set_asset_transfer_compliance(origin, asset, transfer_conditions)?;
        }

        /// Set/unset entities exempt from an asset's transfer compliance rules.
        ///
        /// # Arguments
        /// - `origin` - a signer that has permissions to act as an agent of `exempt_key.asset`.
        /// - `is_exempt` - enable/disable exemption for `entities`.
        /// - `exempt_key` - the asset and stat type to exempt the `entities` from.
        /// - `entities` - the entities to set/unset the exemption for.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset`.
        ///
        /// # Permissions
        /// - Agent
        /// - Asset
        #[weight = <T as Config>::WeightInfo::set_entities_exempt(entities.len() as u32)]
        pub fn set_entities_exempt(origin, is_exempt: bool, exempt_key: TransferConditionExemptKey, entities: BTreeSet<ScopeId>) {
            Self::base_set_entities_exempt(origin, is_exempt, exempt_key, entities)?;
        }
    }
}

impl<T: Config> Module<T> {
    fn ensure_asset_perms(
        origin: T::RuntimeOrigin,
        asset: AssetScope,
    ) -> Result<IdentityId, DispatchError> {
        match asset {
            AssetScope::Ticker(ticker) => Ok(<ExternalAgents<T>>::ensure_perms(origin, ticker)?),
        }
    }

    fn is_asset_stat_active(asset: AssetScope, stat_type: StatType) -> bool {
        Self::active_asset_stats(asset).contains(&stat_type)
    }

    fn base_set_active_asset_stats(
        origin: T::RuntimeOrigin,
        asset: AssetScope,
        stat_types: BTreeSet<StatType>,
    ) -> DispatchResult {
        // Check EA permissions for asset.
        let did = Self::ensure_asset_perms(origin, asset)?;
        // converting from a btreeset to a bounded version
        let stat_types: BoundedBTreeSet<_, T::MaxStatsPerAsset> = stat_types
            .try_into()
            .map_err(|_| Error::<T>::StatTypeLimitReached)?;

        // Get list of StatTypes required by current TransferConditions.
        let required_types = AssetTransferCompliances::<T>::get(&asset)
            .requirements
            .into_iter()
            .map(|condition| condition.get_stat_type())
            .collect::<BTreeSet<_>>();

        // Check if removed StatTypes are needed by TransferConditions.
        let remove_types = Self::active_asset_stats(asset)
            .into_iter()
            // Only remove stats that are not in the new `stat_types` set.
            .filter(|stat_type| !stat_types.contains(&stat_type))
            .map(|stat_type| {
                if required_types.contains(&stat_type) {
                    // Throw an error if the user tries to remove a `StatType` required
                    // by the active `TransferConditions`.
                    Err(Error::<T>::CannotRemoveStatTypeInUse)
                } else {
                    Ok(stat_type)
                }
            })
            .collect::<Result<Vec<_>, Error<T>>>()?;

        // Cleanup storage for old types to be removed.
        for stat_type in &remove_types {
            // Cleanup storage for this stat type, since it is being removed.
            #[allow(deprecated)]
            AssetStats::remove_prefix(
                Stat1stKey {
                    asset,
                    stat_type: *stat_type,
                },
                None,
            );
        }

        // Save new stat types.
        let add_types = stat_types.iter().cloned().collect::<Vec<_>>();
        ActiveAssetStats::<T>::insert(&asset, stat_types);

        if remove_types.len() > 0 {
            Self::deposit_event(Event::StatTypesRemoved(did, asset, remove_types));
        }
        if add_types.len() > 0 {
            Self::deposit_event(Event::StatTypesAdded(did, asset, add_types));
        }
        Ok(())
    }

    fn base_batch_update_asset_stats(
        origin: T::RuntimeOrigin,
        asset: AssetScope,
        stat_type: StatType,
        values: BTreeSet<StatUpdate>,
    ) -> DispatchResult {
        // Check EA permissions for asset.
        let did = Self::ensure_asset_perms(origin, asset)?;
        // Check that `stat_type` is active for `asset`.
        ensure!(
            Self::is_asset_stat_active(asset, stat_type),
            Error::<T>::StatTypeMissing
        );
        let key1 = Stat1stKey { asset, stat_type };
        // process `values` to update stats.
        let updates = values
            .into_iter()
            .map(|update| {
                let key2 = update.key2.clone();
                match update.value {
                    Some(value) => {
                        AssetStats::insert(key1, key2, value);
                    }
                    None => {
                        AssetStats::remove(key1, key2);
                    }
                }
                update
            })
            .collect();

        Self::deposit_event(Event::AssetStatsUpdated(did, asset, stat_type, updates));
        Ok(())
    }

    fn base_set_asset_transfer_compliance(
        origin: T::RuntimeOrigin,
        asset: AssetScope,
        transfer_conditions: BTreeSet<TransferCondition>,
    ) -> DispatchResult {
        // Check EA permissions for asset.
        let did = Self::ensure_asset_perms(origin, asset)?;

        // TODO: Use complexity instead of count to limit TransferConditions per asset.
        // converting from a btreeset to a bounded version
        let transfer_conditions: BoundedBTreeSet<_, T::MaxTransferConditionsPerAsset> =
            transfer_conditions
                .try_into()
                .map_err(|_| Error::<T>::TransferConditionLimitReached)?;

        // Commit changes to storage.
        if transfer_conditions.len() > 0 {
            // Check if required Stats are enabled.
            for condition in transfer_conditions.iter() {
                let stat_type = condition.get_stat_type();
                ensure!(
                    Self::is_asset_stat_active(asset, stat_type),
                    Error::<T>::StatTypeMissing
                );
            }

            AssetTransferCompliances::<T>::mutate(&asset, |old| {
                old.requirements = transfer_conditions.clone()
            });
        } else {
            AssetTransferCompliances::<T>::remove(&asset);
        }

        Self::deposit_event(Event::SetAssetTransferCompliance(
            did,
            asset,
            transfer_conditions.into_iter().collect(),
        ));

        Ok(())
    }

    fn base_set_entities_exempt(
        origin: T::RuntimeOrigin,
        is_exempt: bool,
        exempt_key: TransferConditionExemptKey,
        entities: BTreeSet<ScopeId>,
    ) -> DispatchResult {
        // Check EA permissions for asset.
        let did = Self::ensure_asset_perms(origin, exempt_key.asset)?;
        if is_exempt {
            for entity in &entities {
                TransferConditionExemptEntities::insert(&exempt_key, entity, true);
            }
            Self::deposit_event(Event::TransferConditionExemptionsAdded(
                did,
                exempt_key,
                entities.into_iter().collect(),
            ));
        } else {
            for entity in &entities {
                TransferConditionExemptEntities::remove(&exempt_key, entity);
            }
            Self::deposit_event(Event::TransferConditionExemptionsRemoved(
                did,
                exempt_key,
                entities.into_iter().collect(),
            ));
        }
        Ok(())
    }

    /// Update asset stats.
    pub fn update_asset_balance_stats(
        key1: Stat1stKey,
        from_key2: Stat2ndKey,
        to_key2: Stat2ndKey,
        from_balance: Option<Balance>,
        to_balance: Option<Balance>,
        amount: Balance,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // If 2nd keys are the same and it is not a mint/burn.
        if from_key2 == to_key2 && from_balance.is_some() && to_balance.is_some() {
            // Then the `amount` is transferred between investors
            // with the same claim/non-claim.
            // So no change is needed.
            Self::consume_weight_meter(
                weight_meter,
                <T as Config>::WeightInfo::update_asset_balance_stats(0),
            )?;
            return Ok(());
        }

        Self::consume_weight_meter(
            weight_meter,
            <T as Config>::WeightInfo::update_asset_balance_stats(
                from_balance.is_some() as u32 + to_balance.is_some() as u32,
            ),
        )?;
        if from_balance.is_some() {
            // Remove `amount` from `from_key2`.
            AssetStats::mutate(key1, from_key2, |balance| {
                *balance = balance.saturating_sub(amount)
            });
        }
        if to_balance.is_some() {
            // Add `amount` to `to_key2`.
            AssetStats::mutate(key1, to_key2, |balance| {
                *balance = balance.saturating_add(amount)
            });
        }
        Ok(())
    }

    /// Update unique investor count per asset per claim.
    ///
    /// * `changes: (from_change, to_change)`
    /// If the `from` is transfering the total balance (decreasing investor count), then `from_change == true`.
    /// If the `to` has no tokens before this transfer (increasing investor count), then `to_change == true`.
    fn update_asset_count_stats(
        key1: Stat1stKey,
        from_key2: Stat2ndKey,
        to_key2: Stat2ndKey,
        changes: (bool, bool),
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        match changes {
            (true, true) if from_key2 == to_key2 => {
                // Remove one investor and add another.
                // Both 2nd keys match, so don't need to update the counter.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::update_asset_count_stats(0),
                )?;
            }
            (false, false) => {
                // No changes needed.
                // Consumes the weight for this function
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::update_asset_count_stats(0),
                )?;
            }
            (from_change, to_change) => {
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::update_asset_count_stats(
                        from_change as u32 + to_change as u32,
                    ),
                )?;
                if from_change {
                    // Remove one investor.
                    AssetStats::mutate(key1, from_key2, |counter| {
                        *counter = counter.saturating_sub(1)
                    });
                }
                if to_change {
                    // Add one investor.
                    AssetStats::mutate(key1, to_key2, |counter| {
                        *counter = counter.saturating_add(1)
                    });
                }
            }
        }
        Ok(())
    }

    /// Fetch a claim for an identity as needed by the stat type.
    fn fetch_claim_as_key(did: Option<&IdentityId>, key1: &Stat1stKey) -> Stat2ndKey {
        key1.stat_type
            .claim_issuer
            .map(|(claim_type, issuer)| {
                // Get the claim.
                let did_claim = did.and_then(|did| {
                    let claim_scope = key1.claim_scope();
                    Identity::<T>::fetch_claim(*did, claim_type, issuer, Some(claim_scope))
                        .map(|c| c.claim)
                });
                Stat2ndKey::new_from(&claim_type, did_claim)
            })
            .unwrap_or(Stat2ndKey::NoClaimStat)
    }

    /// Check if an identity has a claim matching `key2`.
    fn has_matching_claim(did: &IdentityId, key1: &Stat1stKey, key2: &Stat2ndKey) -> bool {
        Self::fetch_claim_as_key(Some(did), key1) == *key2
    }

    fn investor_count_changes(
        from_balance: Option<Balance>,
        to_balance: Option<Balance>,
        amount: Balance,
    ) -> Option<(bool, bool)> {
        // Check for two change conditions:
        // 1. Sender transfer their total balance: `(from == Some(0))`
        // 2. Receiver wasn't an investor before: `(to == Some(transfer_amount))`
        //
        // `from == None` - minting new tokens.
        // `to == None` - burning tokens.
        match (from_balance == Some(0), to_balance == Some(amount)) {
            // No changes needed.
            (false, false) => None,
            // Some changes needed.
            changes => Some(changes),
        }
    }

    /// Update asset stats.
    pub fn update_asset_stats(
        ticker: &Ticker,
        from_did: Option<&IdentityId>,
        to_did: Option<&IdentityId>,
        from_balance: Option<Balance>,
        to_balance: Option<Balance>,
        amount: Balance,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // No updates needed if the transfer amount is zero.
        if amount == 0u128 {
            return Ok(());
        }

        Self::consume_weight_meter(
            weight_meter,
            <T as Config>::WeightInfo::active_asset_statistics_load(T::MaxStatsPerAsset::get()),
        )?;

        // Pre-Calculate the investor count changes.
        let count_changes = Self::investor_count_changes(from_balance, to_balance, amount);

        let asset = AssetScope::Ticker(*ticker);
        // Update active asset stats.
        for stat_type in Self::active_asset_stats(asset).into_iter() {
            let key1 = Stat1stKey { asset, stat_type };
            // TODO: Avoid `fetch_claim_as_key` calls for no-claim stats.
            match stat_type.op {
                StatOpType::Count => {
                    if let Some(changes) = count_changes {
                        let from_key2 = Self::fetch_claim_as_key(from_did, &key1);
                        let to_key2 = Self::fetch_claim_as_key(to_did, &key1);
                        Self::update_asset_count_stats(
                            key1,
                            from_key2,
                            to_key2,
                            changes,
                            weight_meter,
                        )?;
                    }
                }
                StatOpType::Balance => {
                    // TODO: no-claim case doesn't need to update balances here.
                    let from_key2 = Self::fetch_claim_as_key(from_did, &key1);
                    let to_key2 = Self::fetch_claim_as_key(to_did, &key1);
                    Self::update_asset_balance_stats(
                        key1,
                        from_key2,
                        to_key2,
                        from_balance,
                        to_balance,
                        amount,
                        weight_meter,
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Verify asset investor count restrictions.
    fn verify_asset_count_restriction(
        key1: Stat1stKey,
        changes: Option<(bool, bool)>,
        max_count: u128,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        match changes {
            Some((true, true)) => {
                // Remove one investor and add another.
                // No count change.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::max_investor_count_restriction(0),
                )?;
                Ok(true)
            }
            Some((false, false)) | None => {
                // No count change.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::max_investor_count_restriction(0),
                )?;
                Ok(true)
            }
            Some((true, false)) => {
                // Remove one investor.
                // Count is decreasing, no need to check max limit.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::max_investor_count_restriction(0),
                )?;
                Ok(true)
            }
            Some((false, true)) => {
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::max_investor_count_restriction(1),
                )?;
                let current_count = AssetStats::get(key1, Stat2ndKey::NoClaimStat);
                Ok(current_count < max_count)
            }
        }
    }

    /// Verify claim count restrictions.
    fn verify_claim_count_restriction(
        key1: Stat1stKey,
        key2: Stat2ndKey,
        from_did: &IdentityId,
        to_did: &IdentityId,
        changes: Option<(bool, bool)>,
        min: u128,
        max: Option<u128>,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let changes = match changes {
            Some(changes) => changes,
            None => {
                // No investor count changes, allow the transfer.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::claim_count_restriction_no_stats(0),
                )?;
                return Ok(true);
            }
        };
        // Check if the investors have the claim.
        let from_matches = Self::has_matching_claim(from_did, &key1, &key2);
        let to_matches = Self::has_matching_claim(to_did, &key1, &key2);
        match changes {
            (true, true) if from_matches == to_matches => {
                // Remove one investor and add another.
                // No count change.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::claim_count_restriction_no_stats(1),
                )?;
            }
            (false, false) => {
                // No count change.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::claim_count_restriction_no_stats(1),
                )?;
            }
            (from_change, to_change) => {
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::claim_count_restriction_with_stats(),
                )?;
                // Get current investor count.
                let count = AssetStats::get(key1, key2);
                // Check minimum count restriction.
                if min > 0 && from_change && from_matches {
                    // The `from` investor has the claim (`from_matches == true`) and
                    // is transfering the last of their tokens (`from_change == true`)
                    // so the investor count for the claim is decreasing.
                    if count <= min {
                        return Ok(false);
                    }
                }
                // Check the maximum count restriction.
                if let Some(max) = max {
                    if to_change && to_matches {
                        // The `to` investor has the claim (`to_matches == true`) and
                        // has a token balance of zero (`to_change == true`)
                        // so the investor count for the claim is increasing.
                        if count >= max {
                            return Ok(false);
                        }
                    }
                }
            }
        }
        Ok(true)
    }

    /// Verify asset % ownership restrictions.
    fn verify_ownership_restriction(
        value: Balance,
        receiver_balance: Balance,
        total_supply: Balance,
        max_percentage: Percentage,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        Self::consume_weight_meter(
            weight_meter,
            <T as Config>::WeightInfo::max_investor_ownership_restriction(),
        )?;
        let new_percentage =
            sp_arithmetic::Permill::from_rational(receiver_balance + value, total_supply);
        Ok(new_percentage <= max_percentage)
    }

    /// Verify claim % ownership restrictions.
    fn verify_claim_ownership_restriction(
        key1: Stat1stKey,
        key2: Stat2ndKey,
        from_did: &IdentityId,
        to_did: &IdentityId,
        value: Balance,
        total_supply: Balance,
        min_percentage: Percentage,
        max_percentage: Percentage,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let from_maches = Self::has_matching_claim(from_did, &key1, &key2);
        let to_maches = Self::has_matching_claim(to_did, &key1, &key2);
        match (from_maches, to_maches) {
            (true, true) => {
                // Both have the claim.  No % ownership change.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::claim_ownership_restriction(0),
                )?;
                Ok(true)
            }
            (false, false) => {
                // Neither have the claim.  No % ownership change.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::claim_ownership_restriction(0),
                )?;
                Ok(true)
            }
            (false, true) => {
                // Only the receiver has the claim.
                // Increasing the % ownership of the claim.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::claim_ownership_restriction(1),
                )?;
                // Calculate new claim % ownership.
                let claim_balance = AssetStats::get(key1, key2);
                let new_percentage = sp_arithmetic::Permill::from_rational(
                    claim_balance.saturating_add(value),
                    total_supply,
                );
                // Check new % ownership is less then maximum.
                Ok(new_percentage <= max_percentage)
            }
            (true, false) => {
                // Only the sender has the claim.
                // Decreasing the % ownership of the claim.
                Self::consume_weight_meter(
                    weight_meter,
                    <T as Config>::WeightInfo::claim_ownership_restriction(1),
                )?;
                // Calculate new claim % ownership.
                let claim_balance = AssetStats::get(key1, key2);
                let new_percentage = sp_arithmetic::Permill::from_rational(
                    claim_balance.saturating_sub(value),
                    total_supply,
                );
                // Check new % ownership is more then the minimum.
                Ok(new_percentage >= min_percentage)
            }
        }
    }

    /// Check transfer condition.
    fn check_transfer_condition(
        condition: &TransferCondition,
        asset: AssetScope,
        from: ScopeId,
        to: ScopeId,
        from_did: &IdentityId,
        to_did: &IdentityId,
        to_balance: Balance,
        amount: Balance,
        total_supply: Balance,
        count_changes: Option<(bool, bool)>,
        weight_meter: &mut WeightMeter,
    ) -> Result<bool, DispatchError> {
        let stat_type = condition.get_stat_type();
        let key1 = Stat1stKey { asset, stat_type };

        let passed = match &condition {
            TransferCondition::MaxInvestorCount(max_count) => Self::verify_asset_count_restriction(
                key1,
                count_changes,
                *max_count as u128,
                weight_meter,
            )?,
            TransferCondition::MaxInvestorOwnership(max_percentage) => {
                Self::verify_ownership_restriction(
                    amount,
                    to_balance,
                    total_supply,
                    *max_percentage,
                    weight_meter,
                )?
            }
            TransferCondition::ClaimCount(claim, _, min, max) => {
                Self::verify_claim_count_restriction(
                    key1,
                    claim.into(),
                    from_did,
                    to_did,
                    count_changes,
                    *min as u128,
                    max.map(|m| m as u128),
                    weight_meter,
                )?
            }
            TransferCondition::ClaimOwnership(claim, _, min, max) => {
                Self::verify_claim_ownership_restriction(
                    key1,
                    claim.into(),
                    from_did,
                    to_did,
                    amount,
                    total_supply,
                    *min,
                    *max,
                    weight_meter,
                )?
            }
        };
        if passed {
            Ok(true)
        } else {
            Self::consume_weight_meter(weight_meter, <T as Config>::WeightInfo::is_exempt())?;
            Ok(Self::is_exempt(asset, condition, &from, &to))
        }
    }

    /// Returns `true` if the [`TransferCondition`] operation is of type [`StatOpType::Count`] and `sender_scope_id`
    /// is in the exemption list or if [`TransferCondition`] operation is of type [`StatOpType::Balance`] and
    /// `receiver_scope_id` is in the exemption list, otherwise returns `false`.
    fn is_exempt(
        asset_scope: AssetScope,
        transfer_condition: &TransferCondition,
        sender_scope_id: &ScopeId,
        receiver_scope_id: &ScopeId,
    ) -> bool {
        let transfer_condition_exempt_key = transfer_condition.get_exempt_key(asset_scope);
        match transfer_condition_exempt_key.op {
            // Count transfer conditions require the sender to be exempt.
            StatOpType::Count => Self::transfer_condition_exempt_entities(
                transfer_condition_exempt_key,
                sender_scope_id,
            ),
            // Percent ownersip transfer conditions require the receiver to be exempt.
            StatOpType::Balance => Self::transfer_condition_exempt_entities(
                transfer_condition_exempt_key,
                receiver_scope_id,
            ),
        }
    }

    /// Verify transfer restrictions for a transfer.
    pub fn verify_transfer_restrictions(
        ticker: &Ticker,
        sender_scope: ScopeId,
        receiver_scope: ScopeId,
        sender_did: &IdentityId,
        receiver_did: &IdentityId,
        sender_balance: Balance,
        receiver_balance: Balance,
        transfer_amount: Balance,
        total_supply: Balance,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let asset_scope = AssetScope::Ticker(*ticker);
        let asset_transfer_requirements = AssetTransferCompliances::<T>::get(&asset_scope);

        // If the requirements are paused, the conditions are not checked
        if asset_transfer_requirements.paused {
            return Ok(());
        }

        Self::verify_requirements(
            &asset_transfer_requirements.requirements,
            asset_scope,
            sender_scope,
            receiver_scope,
            sender_did,
            receiver_did,
            sender_balance,
            receiver_balance,
            transfer_amount,
            total_supply,
            weight_meter,
        )
    }

    /// Returns `true` if all `requirements` are met, otherwise returns `false`.
    fn verify_requirements<S: Get<u32>>(
        transfer_conditions: &BoundedBTreeSet<TransferCondition, S>,
        asset_scope: AssetScope,
        sender_scope: ScopeId,
        receiver_scope: ScopeId,
        sender_did: &IdentityId,
        receiver_did: &IdentityId,
        sender_balance: Balance,
        receiver_balance: Balance,
        transfer_amount: Balance,
        total_supply: Balance,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Checks if the number of investors should be updated
        let change_investors_count = Self::investor_count_changes(
            Some(sender_balance.saturating_sub(transfer_amount)),
            Some(receiver_balance.saturating_add(transfer_amount)),
            transfer_amount,
        );

        for transfer_condition in transfer_conditions {
            if !Self::check_transfer_condition(
                &transfer_condition,
                asset_scope,
                sender_scope,
                receiver_scope,
                sender_did,
                receiver_did,
                receiver_balance,
                transfer_amount,
                total_supply,
                change_investors_count,
                weight_meter,
            )? {
                return Err(Error::<T>::InvalidTransfer.into());
            }
        }

        Ok(())
    }

    /// Get the results of all transfer restrictions for a transfer.
    pub fn get_transfer_restrictions_results(
        ticker: &Ticker,
        from: ScopeId,
        to: ScopeId,
        from_did: &IdentityId,
        to_did: &IdentityId,
        from_balance: Balance,
        to_balance: Balance,
        amount: Balance,
        total_supply: Balance,
        weight_meter: &mut WeightMeter,
    ) -> Result<Vec<TransferConditionResult>, DispatchError> {
        let asset = AssetScope::Ticker(*ticker);
        let tm = AssetTransferCompliances::<T>::get(&asset);

        // Pre-Calculate the investor count changes.
        let count_changes = Self::investor_count_changes(
            Some(from_balance.saturating_sub(amount)),
            Some(to_balance.saturating_add(amount)),
            amount,
        );

        let mut transfer_conditions = Vec::new();
        for condition in tm.requirements {
            let condition_holds = Self::check_transfer_condition(
                &condition,
                asset,
                from,
                to,
                from_did,
                to_did,
                to_balance,
                amount,
                total_supply,
                count_changes,
                weight_meter,
            )?;
            transfer_conditions.push(TransferConditionResult {
                condition,
                result: condition_holds,
            });
        }
        Ok(transfer_conditions)
    }

    /// Helper function to get investor count for tests.
    pub fn investor_count(ticker: Ticker) -> u128 {
        AssetStats::get(Stat1stKey::investor_count(ticker), Stat2ndKey::NoClaimStat)
    }

    /// Consumes from `weight_meter` the given `weight`.
    /// If the new consumed weight is greater than the limit, consumed will be set to limit and an error will be returned.
    fn consume_weight_meter(weight_meter: &mut WeightMeter, weight: Weight) -> DispatchResult {
        weight_meter
            .consume_weight_until_limit(weight)
            .map_err(|_| Error::<T>::WeightLimitExceeded.into())
    }

    /// Helper function to set investor count for benchmarks.
    #[cfg(feature = "runtime-benchmarks")]
    pub fn set_investor_count(ticker: Ticker, count: u128) {
        AssetStats::insert(
            Stat1stKey::investor_count(ticker),
            Stat2ndKey::NoClaimStat,
            count,
        )
    }
}

decl_error! {
    /// Statistics module errors.
    pub enum Error for Module<T: Config> {
        /// Transfer not allowed.
        InvalidTransfer,
        /// StatType is not enabled.
        StatTypeMissing,
        /// StatType is needed by TransferCondition.
        StatTypeNeededByTransferCondition,
        /// A Stattype is in use and can't be removed.
        CannotRemoveStatTypeInUse,
        /// The limit of StatTypes allowed for an asset has been reached.
        StatTypeLimitReached,
        /// The limit of TransferConditions allowed for an asset has been reached.
        TransferConditionLimitReached,
        /// The maximum weight limit for executing the function was exceeded.
        WeightLimitExceeded
    }
}
