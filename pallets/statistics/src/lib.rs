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

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
};
pub use polymesh_common_utilities::traits::statistics::{Config, Event, WeightInfo};
use polymesh_primitives::{
    statistics::{
        AssetScope, Counter, Percentage, Stat1stKey, Stat2ndKey, StatOpType, StatType, StatUpdate,
        TransferManager, TransferManagerResult,
    },
    transfer_compliance::*,
    Balance, IdentityId, ScopeId, Ticker,
};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

type Identity<T> = pallet_identity::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;

decl_storage! {
    trait Store for Module<T: Config> as Statistics {
        /// Transfer managers currently enabled for an Asset.
        pub ActiveTransferManagers get(fn transfer_managers): map hasher(blake2_128_concat) Ticker => Vec<TransferManager>;
        /// Number of current investors in an asset.
        pub InvestorCountPerAsset get(fn investor_count): map hasher(blake2_128_concat) Ticker => Counter;
        /// Entities exempt from transfer managers. Exemptions requirements are based on TMS.
        /// TMs may require just the sender, just the receiver, both or either to be exempted.
        /// CTM requires sender to be exempted while PTM requires receiver to be exempted.
        pub ExemptEntities get(fn entity_exempt):
            double_map
                hasher(blake2_128_concat) (Ticker, TransferManager),
                hasher(blake2_128_concat) ScopeId
            =>
                bool;
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
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        const MaxTransferManagersPerAsset: u32 = T::MaxTransferManagersPerAsset::get();
        const MaxStatsPerAsset: u32 = T::MaxStatsPerAsset::get();
        const MaxTransferConditionsPerAsset: u32 = T::MaxTransferConditionsPerAsset::get();

        /// Adds a new transfer manager.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `ticker` ticker for which the transfer managers are being updated.
        /// * `new_transfer_manager` Transfer manager being added.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the ticker.
        /// * `DuplicateTransferManager` if `new_transfer_manager` is already enabled for the ticker.
        /// * `TransferManagersLimitReached` if the `ticker` already has max TMs attached
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_transfer_manager()]
        pub fn add_transfer_manager(origin, ticker: Ticker, new_transfer_manager: TransferManager) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            ActiveTransferManagers::try_mutate(&ticker, |transfer_managers| {
                ensure!((transfer_managers.len() as u32) < T::MaxTransferManagersPerAsset::get(), Error::<T>::TransferManagersLimitReached);
                ensure!(!transfer_managers.contains(&new_transfer_manager), Error::<T>::DuplicateTransferManager);
                transfer_managers.push(new_transfer_manager.clone());
                Ok(()) as DispatchResult
            })?;
            Self::deposit_event(Event::TransferManagerAdded(did, ticker, new_transfer_manager));
        }

        /// Removes a transfer manager.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `ticker` ticker for which the transfer managers are being updated.
        /// * `transfer_manager` Transfer manager being removed.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the ticker.
        /// * `TransferManagerMissing` if `asset_compliance` contains multiple entries with the same `requirement_id`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_transfer_manager()]
        pub fn remove_transfer_manager(origin, ticker: Ticker, transfer_manager: TransferManager) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            ActiveTransferManagers::try_mutate(&ticker, |transfer_managers| {
                let before = transfer_managers.len();
                transfer_managers.retain(|tm| *tm != transfer_manager);
                ensure!(before != transfer_managers.len(), Error::<T>::TransferManagerMissing);
                Ok(()) as DispatchResult
            })?;
            Self::deposit_event(Event::TransferManagerRemoved(did, ticker, transfer_manager));
        }

        /// Exempt entities from a transfer manager
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `ticker` ticker for which the exemption list is being modified.
        /// * `transfer_manager` Transfer manager for which the exemption list is being modified.
        /// * `exempted_entities` ScopeIds for which the exemption list is being modified.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the ticker.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::add_exempted_entities(exempted_entities.len() as u32)]
        pub fn add_exempted_entities(origin, ticker: Ticker, transfer_manager: TransferManager, exempted_entities: Vec<ScopeId>) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            let ticker_tm = (ticker, transfer_manager.clone());
            for entity in &exempted_entities {
                ExemptEntities::insert(&ticker_tm, entity, true);
            }
            Self::deposit_event(Event::ExemptionsAdded(did, ticker, transfer_manager, exempted_entities));
        }

        /// remove entities from exemption list of a transfer manager
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `ticker` ticker for which the exemption list is being modified.
        /// * `transfer_manager` Transfer manager for which the exemption list is being modified.
        /// * `scope_ids` ScopeIds for which the exemption list is being modified.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the ticker.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_exempted_entities(entities.len() as u32)]
        pub fn remove_exempted_entities(origin, ticker: Ticker, transfer_manager: TransferManager, entities: Vec<ScopeId>) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            let ticker_tm = (ticker, transfer_manager.clone());
            for entity in &entities {
                ExemptEntities::remove(&ticker_tm, entity);
            }
            Self::deposit_event(Event::ExemptionsRemoved(did, ticker, transfer_manager, entities));
        }

        /// Set the active asset stat_types.
        ///
        /// # Permissions (EA)
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_active_asset_stats(stat_types.len() as u32)]
        pub fn set_active_asset_stats(origin, asset: AssetScope, stat_types: BTreeSet<StatType>) {
            Self::base_set_active_asset_stats(origin, asset, stat_types)?;
        }

        /// Allow a trusted issuer to init/resync ticker/company stats.
        ///
        /// # Permissions (EA)
        /// * Asset
        #[weight = <T as Config>::WeightInfo::batch_update_asset_stats(values.len() as u32)]
        pub fn batch_update_asset_stats(origin, asset: AssetScope, stat_type: StatType, values: BTreeSet<StatUpdate>) {
            Self::base_batch_update_asset_stats(origin, asset, stat_type, values)?;
        }

        /// Set asset transfer compliance rules.
        ///
        /// # Permissions (EA)
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_asset_transfer_compliance(transfer_conditions.len() as u32)]
        pub fn set_asset_transfer_compliance(origin, asset: AssetScope, transfer_conditions: BTreeSet<TransferCondition>) {
            Self::base_set_asset_transfer_compliance(origin, asset, transfer_conditions)?;
        }

        /// Set/unset entities exempt from an asset's transfer compliance rules.
        ///
        /// # Permissions (EA)
        /// * Asset
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
            .filter(|stat_type| !stat_types.contains(&stat_type))
            .map(|stat_type| {
                if required_types.contains(&stat_type) {
                    Err(Error::<T>::StatTypeLimitReached)
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
            // Add one investor.
            AssetStats::mutate(key1, to_key2, |balance| {
                *balance = balance.saturating_add(amount)
            });
        }
    }

    /// Update unique investor count per asset per claim.
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

        // Update old stats.
        Self::update_transfer_stats(ticker, from_balance, to_balance, amount);

        // Pre-Calculate the investor count changes.
        let count_changes = Self::investor_count_changes(from_balance, to_balance, amount);

        let asset = AssetScope::Ticker(*ticker);
        // Update active asset stats.
        for stat_type in Self::active_asset_stats(asset).into_iter() {
            let key1 = Stat1stKey { asset, stat_type };
            match stat_type.op {
                StatOpType::Count => {
                    if let Some(changes) = count_changes {
                        let from_key2 = Self::fetch_claim_as_key(from_did, &key1);
                        let to_key2 = Self::fetch_claim_as_key(to_did, &key1);
                        Self::update_asset_count_stats(key1, from_key2, to_key2, changes);
                    }
                }
                StatOpType::Balance => {
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

    /// It updates our statistics after transfer execution.
    /// The following counters could be updated:
    ///     - *Investor count per asset*.
    ///
    pub fn update_transfer_stats(
        ticker: &Ticker,
        updated_from_balance: Option<Balance>,
        updated_to_balance: Option<Balance>,
        amount: Balance,
    ) {
        if amount == 0u128.into() {
            return;
        }

        // 1. Unique investor count per asset.
        let counter = Self::investor_count(ticker);
        let mut new_counter = counter;

        if let Some(from_balance) = updated_from_balance {
            if from_balance == 0u128.into() {
                new_counter = new_counter.checked_sub(1).unwrap_or(new_counter);
            }
        }

        if let Some(to_balance) = updated_to_balance {
            if to_balance == amount {
                new_counter = new_counter.checked_add(1).unwrap_or(new_counter);
            }
        }

        // Only updates extrinsics if counter has been changed.
        if new_counter != counter {
            InvestorCountPerAsset::insert(ticker, new_counter)
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
        let from_maches = Self::has_matching_claim(from_did, &key1, &key2);
        let to_maches = Self::has_matching_claim(to_did, &key1, &key2);
        match changes {
            (true, true) if from_maches == to_maches => {
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
                if min > 0 && from_change && from_maches {
                    if count <= min {
                        return false;
                    }
                }
                if let Some(max) = max {
                    if to_change && to_maches {
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
        // Call old TMs logic.
        Self::verify_tm_restrictions(
            ticker,
            from,
            to,
            amount,
            from_balance,
            to_balance,
            total_supply,
        )?;

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
            if !passed {
                let exempt_key = condition.get_exempt_key(asset);
                let id = match exempt_key.op {
                    // Count transfer conditions require the sender to be exempt.
                    StatOpType::Count => from,
                    // Percent ownersip transfer conditions require the receiver to be exempt.
                    StatOpType::Balance => to,
                };
                let is_exempt = Self::transfer_condition_exempt_entities(exempt_key, id);
                ensure!(is_exempt, Error::<T>::InvalidTransfer);
            }
        }

        Ok(())
    }

    /// Verify transfer restrictions for a transfer
    /// TODO: Migrate old TMs.
    pub fn verify_tm_restrictions(
        ticker: &Ticker,
        sender: ScopeId,
        receiver: ScopeId,
        value: Balance,
        sender_balance: Balance,
        receiver_balance: Balance,
        total_supply: Balance,
    ) -> DispatchResult {
        Self::transfer_managers(ticker)
            .into_iter()
            .try_for_each(|tm| match tm {
                TransferManager::CountTransferManager(max_count) => Self::ensure_ctm(
                    ticker,
                    sender,
                    value,
                    sender_balance,
                    receiver_balance,
                    max_count,
                ),
                TransferManager::PercentageTransferManager(max_percentage) => Self::ensure_ptm(
                    ticker,
                    receiver,
                    value,
                    receiver_balance,
                    total_supply,
                    max_percentage,
                ),
            })
    }

    pub fn verify_tm_restrictions_granular(
        ticker: &Ticker,
        sender: ScopeId,
        receiver: ScopeId,
        value: Balance,
        sender_balance: Balance,
        receiver_balance: Balance,
        total_supply: Balance,
    ) -> Vec<TransferManagerResult> {
        Self::transfer_managers(ticker)
            .into_iter()
            .map(|tm| TransferManagerResult {
                result: match &tm {
                    TransferManager::CountTransferManager(max_count) => Self::ensure_ctm(
                        ticker,
                        sender,
                        value,
                        sender_balance,
                        receiver_balance,
                        *max_count,
                    )
                    .is_ok(),
                    TransferManager::PercentageTransferManager(max_percentage) => Self::ensure_ptm(
                        ticker,
                        receiver,
                        value,
                        receiver_balance,
                        total_supply,
                        max_percentage.clone(),
                    )
                    .is_ok(),
                },
                tm,
            })
            .collect()
    }

    fn ensure_ctm(
        ticker: &Ticker,
        sender: ScopeId,
        value: Balance,
        sender_balance: Balance,
        receiver_balance: Balance,
        max_count: Counter,
    ) -> DispatchResult {
        let current_count = Self::investor_count(ticker);
        ensure!(
            current_count < max_count
                || sender_balance == value
                || receiver_balance > 0u32.into()
                || Self::entity_exempt(
                    (*ticker, TransferManager::CountTransferManager(max_count)),
                    sender
                ),
            Error::<T>::InvalidTransfer
        );
        Ok(())
    }

    fn ensure_ptm(
        ticker: &Ticker,
        receiver: ScopeId,
        value: Balance,
        receiver_balance: Balance,
        total_supply: Balance,
        max_percentage: Percentage,
    ) -> DispatchResult {
        let new_percentage =
            sp_arithmetic::Permill::from_rational(receiver_balance + value, total_supply);
        ensure!(
            new_percentage <= *max_percentage
                || Self::entity_exempt(
                    (
                        *ticker,
                        TransferManager::PercentageTransferManager(max_percentage)
                    ),
                    receiver
                ),
            Error::<T>::InvalidTransfer
        );
        Ok(())
    }

    #[cfg(feature = "runtime-benchmarks")]
    pub fn set_investor_count(ticker: &Ticker, count: Counter) {
        InvestorCountPerAsset::insert(ticker, count);
    }
}

decl_error! {
    /// Statistics module errors.
    pub enum Error for Module<T: Config> {
        /// The transfer manager already exists.
        DuplicateTransferManager,
        /// Transfer manager is not enabled.
        TransferManagerMissing,
        /// Transfer not allowed.
        InvalidTransfer,
        /// The limit of transfer managers allowed for an asset has been reached.
        TransferManagersLimitReached,
        /// StatType is not enabled.
        StatTypeMissing,
        /// StatType is needed by TransferCondition.
        StatTypeNeededByTransferCondition,
        /// The limit of StatTypes allowed for an asset has been reached.
        StatTypeLimitReached,
        /// The limit of TransferConditions allowed for an asset has been reached.
        TransferConditionLimitReached,
    }
}
