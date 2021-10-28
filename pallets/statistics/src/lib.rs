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

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
};
pub use polymesh_common_utilities::traits::statistics::{Config, Event, WeightInfo};
use polymesh_primitives::{
    statistics::{
        AssetScope, Counter, Percentage, StatType, TransferManager, TransferManagerResult,
    },
    Balance, Claim, ClaimType, IdentityId, Scope, ScopeId, Ticker,
};
use sp_std::vec::Vec;

type Identity<T> = pallet_identity::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;

/// First stats key in double map.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug)]
pub struct Stat1stKey {
    pub asset: AssetScope,
    pub stat_type: StatType,
    /// The trusted issuer for `StatType`.  For `StatType::Count(None)` this is `None`.
    /// Only the trusted issuer or a permissioned external agent for the asset can init/resync the stats.
    pub issuer: Option<IdentityId>,
}

/// Second stats key in double map.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Stat2ndKey {
    /// For per-Claim stats (Jurisdiction, Accredited, etc...).
    /// Non-Accredited stats would be stored with a `None` here.
    pub claim: Option<Claim>,
}

/// Stats update.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct StatUpdate {
    pub claim: Option<Claim>,
    /// None - Remove stored value if any.
    pub value: Option<u128>,
}

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
        pub ActiveAssetStats get(fn active_asset_stats): map hasher(blake2_128_concat) AssetScope => Vec<(StatType, Option<IdentityId>)>;
        /// Asset stats.
        pub AssetStats get(fn asset_stats):
          double_map
            hasher(blake2_128_concat) Stat1stKey,
            hasher(blake2_128_concat) Stat2ndKey => u128;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        const MaxTransferManagersPerAsset: u32 = T::MaxTransferManagersPerAsset::get();

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
        /// # Permissions (EA)
        /// * Asset
        #[weight = 0]
        pub fn set_active_asset_stats(origin, asset: AssetScope, stat_types: Vec<(StatType, Option<IdentityId>)>) {
            // TODO: benchmark and weight.
            Self::base_set_active_asset_stats(origin, asset, stat_types)?;
        }

        /// Allow a trusted issuer to init/resync ticker/company stats.
        /// # Permissions (EA)
        /// * Asset
        #[weight = 0]
        pub fn batch_update_asset_stats(origin, asset: AssetScope, stat_type: StatType, issuer: Option<IdentityId>, values: Vec<StatUpdate>) {
            // TODO: benchmark and weight.
            Self::base_batch_update_asset_stats(origin, asset, stat_type, issuer, values)?;
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

    fn is_asset_stat_active(asset: AssetScope, stat_type: (StatType, Option<IdentityId>)) -> bool {
        Self::active_asset_stats(asset).contains(&stat_type)
    }

    fn base_set_active_asset_stats(
        origin: T::Origin,
        asset: AssetScope,
        stat_types: Vec<(StatType, Option<IdentityId>)>,
    ) -> DispatchResult {
        // Check EA permissions for asset.
        let _did = Self::ensure_asset_perms(origin, asset)?;
        // TODO: add `MaxStatsPerAsset` and error variant.
        ensure!(
            stat_types.len() < T::MaxTransferManagersPerAsset::get() as usize,
            Error::<T>::TransferManagersLimitReached
        );
        // Make sure stats with a `ClaimType` have an issuer.
        for (stat_type, issuer) in stat_types.iter() {
            // TODO: Need a better error.
            ensure!(
                stat_type.need_issuer() == issuer.is_some(),
                Error::<T>::TransferManagersLimitReached
            );
        }

        // Cleanup storage for old types to be removed.
        for (stat_type, issuer) in Self::active_asset_stats(asset).into_iter() {
            if !stat_types.contains(&(stat_type, issuer)) {
                // Cleanup storage for this stat type, since it is being removed.
                AssetStats::remove_prefix(Stat1stKey {
                    asset,
                    stat_type,
                    issuer,
                });
            }
        }
        // Save new stat types.
        ActiveAssetStats::insert(&asset, stat_types);
        // TODO: emit event.
        Ok(())
    }

    fn base_batch_update_asset_stats(
        origin: T::Origin,
        asset: AssetScope,
        stat_type: StatType,
        issuer: Option<IdentityId>,
        values: Vec<StatUpdate>,
    ) -> DispatchResult {
        // Check EA permissions for asset.
        // TODO: Also allow trusted issuers to update stats.
        let _did = Self::ensure_asset_perms(origin, asset)?;
        // Check that `stat_type` is active for `asset`.
        // TODO: add error variant.
        ensure!(
            Self::is_asset_stat_active(asset, (stat_type, issuer)),
            Error::<T>::TransferManagerMissing
        );
        let key1 = Stat1stKey {
            asset,
            stat_type,
            issuer,
        };
        // process `values` to update stats.
        values.into_iter().for_each(|StatUpdate { claim, value }| {
            let key2 = Stat2ndKey { claim };
            match value {
                Some(value) => {
                    AssetStats::insert(key1, key2, value);
                }
                None => {
                    AssetStats::remove(key1, key2);
                }
            }
        });
        // TODO: emit event.
        Ok(())
    }

    // Update asset stats.
    pub fn update_asset_balance_stats(
        key1: Stat1stKey,
        from_key2: Stat2ndKey,
        to_key2: Stat2ndKey,
        from_balance: Option<Balance>,
        to_balance: Option<Balance>,
        amount: Balance,
    ) {
        // If 2nd keys are the same, then no change needed.
        if from_key2 == to_key2 {
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

    // Update unique investor count per asset per claim.
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

    /// Fetch claims.
    fn fetch_claim(
        did: Option<IdentityId>,
        claim_type: Option<ClaimType>,
        issuer: Option<IdentityId>,
        scope: Scope,
    ) -> Stat2ndKey {
        let claim = match (claim_type, issuer, did) {
            (Some(claim_type), Some(issuer), Some(did)) => {
                Identity::<T>::fetch_claim(did, claim_type, issuer, Some(scope)).map(|c| c.claim)
            }
            _ => None,
        };

        Stat2ndKey { claim }
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
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
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

        // Calculate the investor count changes.
        let count_changes = Self::investor_count_changes(from_balance, to_balance, amount);

        let asset = AssetScope::Ticker(*ticker);
        let scope = Scope::Ticker(*ticker);
        // Update active asset stats.
        for (stat_type, issuer) in Self::active_asset_stats(asset).into_iter() {
            let key1 = Stat1stKey {
                asset,
                stat_type,
                issuer,
            };
            match stat_type {
                StatType::Count(claim_type) => {
                    if let Some(changes) = count_changes {
                        let from_key2 =
                            Self::fetch_claim(from_did, claim_type, issuer, scope.clone());
                        let to_key2 = Self::fetch_claim(to_did, claim_type, issuer, scope.clone());
                        Self::update_asset_count_stats(key1, from_key2, to_key2, changes);
                    }
                }
                StatType::Balance(None) => {
                    // No stats updated needed for per-investor balance.
                }
                StatType::Balance(claim_type) => {
                    let from_key2 = Self::fetch_claim(from_did, claim_type, issuer, scope.clone());
                    let to_key2 = Self::fetch_claim(to_did, claim_type, issuer, scope.clone());
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

    /// Verify transfer restrictions for a transfer
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
        /// The transfer manager already exists
        DuplicateTransferManager,
        /// Transfer manager is not enabled
        TransferManagerMissing,
        /// Transfer not allowed
        InvalidTransfer,
        /// The limit of transfer managers allowed for an asset has been reached
        TransferManagersLimitReached
    }
}
