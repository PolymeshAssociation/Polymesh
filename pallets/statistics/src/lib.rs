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
    decl_error, decl_module, decl_storage, dispatch::DispatchResult, ensure, traits::Get,
};
pub use polymesh_common_utilities::traits::statistics::{Config, Event, WeightInfo};
use polymesh_primitives::{
    statistics::{Counter, Percentage, TransferManager, TransferManagerResult},
    Balance, ScopeId, Ticker,
};
use sp_std::vec::Vec;

type ExternalAgents<T> = pallet_external_agents::Module<T>;

decl_storage! {
    trait Store for Module<T: Config> as statistics {
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
    }
}

impl<T: Config> Module<T> {
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
        // 1. Unique investor count per asset.
        if amount != 0u128.into() {
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
        let new_percentage = sp_arithmetic::Permill::from_rational_approximation(
            receiver_balance + value,
            total_supply,
        );
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
