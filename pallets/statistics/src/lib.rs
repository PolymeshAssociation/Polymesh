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

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_option)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    weights::Weight,
};
pub use polymesh_common_utilities::traits::statistics::{Config, Event, WeightInfo};
use polymesh_primitives::{
    statistics::{
        AssetScope, Percentage, Stat1stKey, Stat2ndKey, StatOpType, StatType, StatUpdate,
    },
    storage_migrate_on, storage_migration_ver,
    transfer_compliance::*,
    Balance, IdentityId, ScopeId, Ticker,
};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

type Identity<T> = pallet_identity::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;

storage_migration_ver!(1);

decl_storage! {
    trait Store for Module<T: Config> as Statistics {
        /// Active stats for a ticker/company.  There should be a max limit on the number of active stats for a ticker/company.
        pub ActiveAssetStats get(fn active_asset_stats): map hasher(blake2_128_concat) AssetScope => BTreeSet<StatType>;
        /// Asset stats.
        pub AssetStats get(fn asset_stats):
          double_map
            hasher(blake2_128_concat) Stat1stKey,
            hasher(blake2_128_concat) Stat2ndKey => u128;
        /// Asset transfer compliance for a ticker (AssetScope -> AssetTransferCompliance)
        pub AssetTransferCompliances get(fn asset_transfer_compliance): map hasher(blake2_128_concat) AssetScope => AssetTransferCompliance;
        /// Entities exempt from a Transfer Compliance rule.
        pub TransferConditionExemptEntities get(fn transfer_condition_exempt_entities):
            double_map
                hasher(blake2_128_concat) TransferConditionExemptKey,
                hasher(blake2_128_concat) ScopeId
            =>
                bool;

        /// Storage migration version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1).unwrap()): Version;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            storage_migrate_on!(StorageVersion::get(), 1, {
                migration::migrate_v1::<T>();
            });

            0
        }

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
        origin: T::Origin,
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
        origin: T::Origin,
        asset: AssetScope,
        stat_types: BTreeSet<StatType>,
    ) -> DispatchResult {
        // Check EA permissions for asset.
        let did = Self::ensure_asset_perms(origin, asset)?;

        // Check StatType per Asset limit.
        ensure!(
            stat_types.len() < T::MaxStatsPerAsset::get() as usize,
            Error::<T>::StatTypeLimitReached
        );

        // Get list of StatTypes required by current TransferConditions.
        let required_types = AssetTransferCompliances::get(&asset)
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
        ActiveAssetStats::insert(&asset, stat_types);

        if remove_types.len() > 0 {
            Self::deposit_event(Event::StatTypesRemoved(did, asset, remove_types));
        }
        if add_types.len() > 0 {
            Self::deposit_event(Event::StatTypesAdded(did, asset, add_types));
        }
        Ok(())
    }

    fn base_batch_update_asset_stats(
        origin: T::Origin,
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
        origin: T::Origin,
        asset: AssetScope,
        transfer_conditions: BTreeSet<TransferCondition>,
    ) -> DispatchResult {
        // Check EA permissions for asset.
        let did = Self::ensure_asset_perms(origin, asset)?;

        // TODO: Use complexity instead of count to limit TransferConditions per asset.
        // Check maximum TransferConditions per Asset limit.
        ensure!(
            transfer_conditions.len() < T::MaxTransferConditionsPerAsset::get() as usize,
            Error::<T>::TransferConditionLimitReached
        );

        // Commit changes to storage.
        if transfer_conditions.len() > 0 {
            // Check if required Stats are enabled.
            for condition in &transfer_conditions {
                let stat_type = condition.get_stat_type();
                ensure!(
                    Self::is_asset_stat_active(asset, stat_type),
                    Error::<T>::StatTypeMissing
                );
            }

            AssetTransferCompliances::mutate(&asset, |old| {
                old.requirements = transfer_conditions.clone()
            });
        } else {
            AssetTransferCompliances::remove(&asset);
        }

        Self::deposit_event(Event::SetAssetTransferCompliance(
            did,
            asset,
            transfer_conditions.into_iter().collect(),
        ));

        Ok(())
    }

    fn base_set_entities_exempt(
        origin: T::Origin,
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
    ) {
        // If 2nd keys are the same and it is not a mint/burn.
        if from_key2 == to_key2 && from_balance.is_some() && to_balance.is_some() {
            // Then the `amount` is transferred between investors
            // with the same claim/non-claim.
            // So no change is needed.
            return;
        }

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
    ) {
        match changes {
            (true, true) if from_key2 == to_key2 => {
                // Remove one investor and add another.
                // Both 2nd keys match, so don't need to update the counter.
            }
            (false, false) => {
                // No changes needed.
            }
            (from_change, to_change) => {
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
    }

    /// Fetch a claim for an identity as needed by the stat type.
    fn fetch_claim_as_key(did: Option<&IdentityId>, key1: &Stat1stKey) -> Stat2ndKey {
        key1.stat_type
            .claim_issuer
            .map(|(claim_type, issuer)| {
                // Get the claim.
                let did_claim = did.and_then(|did| {
                    let claim_scope = key1.claim_scope();
                    Identity::<T>::fetch_claim(*did, claim_type, issuer, Some(claim_scope.clone()))
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
    ) {
        // No updates needed if the transfer amount is zero.
        if amount == 0u128.into() {
            return;
        }

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
                        Self::update_asset_count_stats(key1, from_key2, to_key2, changes);
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
                    );
                }
            }
        }
    }

    /// Verify asset investor count restrictions.
    fn verify_asset_count_restriction(
        key1: Stat1stKey,
        changes: Option<(bool, bool)>,
        max_count: u128,
    ) -> bool {
        match changes {
            Some((true, true)) => {
                // Remove one investor and add another.
                // No count change.
                true
            }
            Some((false, false)) | None => {
                // No count change.
                true
            }
            Some((true, false)) => {
                // Remove one investor.
                // Count is decreasing, no need to check max limit.
                true
            }
            Some((false, true)) => {
                let current_count = AssetStats::get(key1, Stat2ndKey::NoClaimStat);
                current_count < max_count
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
    ) -> bool {
        let changes = match changes {
            Some(changes) => changes,
            None => {
                // No investor count changes, allow the transfer.
                return true;
            }
        };
        // Check if the investors have the claim.
        let from_matches = Self::has_matching_claim(from_did, &key1, &key2);
        let to_matches = Self::has_matching_claim(to_did, &key1, &key2);
        match changes {
            (true, true) if from_matches == to_matches => {
                // Remove one investor and add another.
                // No count change.
            }
            (false, false) => {
                // No count change.
            }
            (from_change, to_change) => {
                // Get current investor count.
                let count = AssetStats::get(key1, key2);
                // Check minimum count restriction.
                if min > 0 && from_change && from_matches {
                    // The `from` investor has the claim (`from_matches == true`) and
                    // is transfering the last of their tokens (`from_change == true`)
                    // so the investor count for the claim is decreasing.
                    if count <= min {
                        return false;
                    }
                }
                // Check the maximum count restriction.
                if let Some(max) = max {
                    if to_change && to_matches {
                        // The `to` investor has the claim (`to_matches == true`) and
                        // has a token balance of zero (`to_change == true`)
                        // so the investor count for the claim is increasing.
                        if count >= max {
                            return false;
                        }
                    }
                }
            }
        }
        return true;
    }

    /// Verify asset % ownership restrictions.
    fn verify_ownership_restriction(
        value: Balance,
        receiver_balance: Balance,
        total_supply: Balance,
        max_percentage: Percentage,
    ) -> bool {
        let new_percentage =
            sp_arithmetic::Permill::from_rational(receiver_balance + value, total_supply);
        new_percentage <= *max_percentage
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
    ) -> bool {
        let from_maches = Self::has_matching_claim(from_did, &key1, &key2);
        let to_maches = Self::has_matching_claim(to_did, &key1, &key2);
        match (from_maches, to_maches) {
            (true, true) => {
                // Both have the claim.  No % ownership change.
                true
            }
            (false, false) => {
                // Neither have the claim.  No % ownership change.
                true
            }
            (false, true) => {
                // Only the receiver has the claim.
                // Increasing the % ownership of the claim.

                // Calculate new claim % ownership.
                let claim_balance = AssetStats::get(key1, key2);
                let new_percentage = sp_arithmetic::Permill::from_rational(
                    claim_balance.saturating_add(value),
                    total_supply,
                );
                // Check new % ownership is less then maximum.
                new_percentage <= *max_percentage
            }
            (true, false) => {
                // Only the sender has the claim.
                // Decreasing the % ownership of the claim.

                // Calculate new claim % ownership.
                let claim_balance = AssetStats::get(key1, key2);
                let new_percentage = sp_arithmetic::Permill::from_rational(
                    claim_balance.saturating_sub(value),
                    total_supply,
                );
                // Check new % ownership is more then the minimum.
                new_percentage >= *min_percentage
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
    ) -> bool {
        use TransferCondition::*;

        let stat_type = condition.get_stat_type();
        let key1 = Stat1stKey { asset, stat_type };

        let passed = match &condition {
            MaxInvestorCount(max_count) => {
                Self::verify_asset_count_restriction(key1, count_changes, *max_count as u128)
            }
            MaxInvestorOwnership(max_percentage) => Self::verify_ownership_restriction(
                amount,
                to_balance,
                total_supply,
                *max_percentage,
            ),
            ClaimCount(claim, _, min, max) => Self::verify_claim_count_restriction(
                key1,
                claim.into(),
                from_did,
                to_did,
                count_changes,
                *min as u128,
                max.map(|m| m as u128),
            ),
            ClaimOwnership(claim, _, min, max) => Self::verify_claim_ownership_restriction(
                key1,
                claim.into(),
                from_did,
                to_did,
                amount,
                total_supply,
                *min,
                *max,
            ),
        };
        if passed {
            true
        } else {
            let exempt_key = condition.get_exempt_key(asset);
            let id = match exempt_key.op {
                // Count transfer conditions require the sender to be exempt.
                StatOpType::Count => from,
                // Percent ownersip transfer conditions require the receiver to be exempt.
                StatOpType::Balance => to,
            };
            let is_exempt = Self::transfer_condition_exempt_entities(exempt_key, id);
            is_exempt
        }
    }

    /// Verify transfer restrictions for a transfer.
    pub fn verify_transfer_restrictions(
        ticker: &Ticker,
        from: ScopeId,
        to: ScopeId,
        from_did: &IdentityId,
        to_did: &IdentityId,
        from_balance: Balance,
        to_balance: Balance,
        amount: Balance,
        total_supply: Balance,
    ) -> DispatchResult {
        let asset = AssetScope::Ticker(*ticker);
        let tm = AssetTransferCompliances::get(&asset);
        if tm.paused {
            // Transfer rules are paused.
            return Ok(());
        }

        // Pre-Calculate the investor count changes.
        let count_changes = Self::investor_count_changes(
            Some(from_balance.saturating_sub(amount)),
            Some(to_balance.saturating_add(amount)),
            amount,
        );

        for condition in tm.requirements {
            let result = Self::check_transfer_condition(
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
            );
            ensure!(result, Error::<T>::InvalidTransfer);
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
    ) -> Vec<TransferConditionResult> {
        let asset = AssetScope::Ticker(*ticker);
        let tm = AssetTransferCompliances::get(&asset);

        // Pre-Calculate the investor count changes.
        let count_changes = Self::investor_count_changes(
            Some(from_balance.saturating_sub(amount)),
            Some(to_balance.saturating_add(amount)),
            amount,
        );

        tm.requirements
            .into_iter()
            .map(|condition| {
                let result = Self::check_transfer_condition(
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
                );
                TransferConditionResult { condition, result }
            })
            .collect()
    }

    /// Helper function to get investor count for tests.
    pub fn investor_count(ticker: Ticker) -> u128 {
        AssetStats::get(Stat1stKey::investor_count(ticker), Stat2ndKey::NoClaimStat)
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
    }
}

mod migration {
    use super::*;

    mod v1 {
        use super::*;
        use scale_info::TypeInfo;

        pub type Counter = u64;

        #[derive(Decode, Encode, TypeInfo)]
        #[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub enum TransferManager {
            CountTransferManager(Counter),
            PercentageTransferManager(Percentage),
        }

        decl_storage! {
            trait Store for Module<T: Config> as Statistics {
                pub ActiveTransferManagers get(fn transfer_managers): map hasher(blake2_128_concat) Ticker => Vec<TransferManager>;
                pub InvestorCountPerAsset get(fn investor_count): map hasher(blake2_128_concat) Ticker => Counter;
                pub ExemptEntities get(fn entity_exempt):
                    double_map
                        hasher(blake2_128_concat) (Ticker, TransferManager),
                        hasher(blake2_128_concat) ScopeId
                    =>
                        bool;
            }
        }

        decl_module! {
            pub struct Module<T: Config> for enum Call where origin: T::Origin { }
        }
    }

    pub fn migrate_v1<T: Config>() {
        sp_runtime::runtime_logger::RuntimeLogger::init();

        log::info!(" >>> Updating Statistics storage. Migrating TransferManagers...");
        let total_managers =
            v1::ActiveTransferManagers::drain().fold(0usize, |total, (ticker, managers)| {
                let count = managers.len();
                let asset = AssetScope::from(ticker);

                let requirements = managers
                    .into_iter()
                    .map(|manager| {
                        // Convert TransferManager to TransferCondition.
                        let condition = match manager {
                            v1::TransferManager::CountTransferManager(max) => {
                                TransferCondition::MaxInvestorCount(max)
                            }
                            v1::TransferManager::PercentageTransferManager(max) => {
                                TransferCondition::MaxInvestorOwnership(max)
                            }
                        };

                        // Convert Exemptions for the TransferManager.
                        for (entity, exempt) in v1::ExemptEntities::drain_prefix((ticker, manager))
                        {
                            if exempt {
                                let exempt_key = condition.get_exempt_key(asset);
                                TransferConditionExemptEntities::insert(&exempt_key, entity, true);
                            }
                        }

                        condition
                    })
                    .collect();

                let compliance = AssetTransferCompliance {
                    paused: false,
                    requirements,
                };

                // Enable stats.
                let stats = compliance
                    .requirements
                    .iter()
                    .map(|c| c.get_stat_type())
                    .collect::<BTreeSet<_>>();
                ActiveAssetStats::insert(asset, stats);

                // Save new transfer compliance rules.
                AssetTransferCompliances::insert(asset, compliance);

                total + count
            });
        log::info!(" >>> Migrated {} TransferManagers.", total_managers);

        log::info!(" >>> Migrating Investor counts...");
        let total_counts =
            v1::InvestorCountPerAsset::drain().fold(0usize, |total, (ticker, count)| {
                // Make sure investor count stats are enabled.
                ActiveAssetStats::mutate(AssetScope::from(ticker), |stats| {
                    stats.insert(StatType::investor_count());
                });

                // Save investor count to new stats storage.
                AssetStats::insert(
                    Stat1stKey::investor_count(ticker),
                    Stat2ndKey::NoClaimStat,
                    count as u128,
                );

                total + 1
            });
        log::info!(" >>> Migrated {} Asset Investor counts.", total_counts);
    }
}
