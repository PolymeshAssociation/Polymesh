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

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use polymesh_common_utilities::{asset::Trait as AssetTrait, identity::Trait as IdentityTrait};
use polymesh_primitives::{IdentityId, Ticker};
use sp_arithmetic::Permill;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::vec::Vec;

pub type Counter = u64;
pub type Percentage = Permill;

/// Transfer managers that can be attached to a Token for compliance.
#[derive(Eq, PartialEq, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TransferManager {
    /// CTM limits the number of active investors in a Token.
    CountTransferManager(Counter),
    /// PTM limits the percentage of token owned by a single Identity.
    PercentageTransferManager(Percentage),
}

// impl TransferManager {
//     fn get_type(&self) -> TransferManagerType {
//         match self {
//             Self::CountTransferManager(_) => TransferManagerType::CTM,
//             Self::PercentageTransferManager(_) => TransferManagerType::PTM,
//         }
//     }
// }

// /// Transfer managers types, used for maintaining uniqueness
// #[derive(Eq, PartialEq, Encode, Decode)]
// #[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
// enum TransferManagerType {
//     /// Represents a Count Transfer Manager
//     CountTransferManager,
//     /// Represents a Percentage Transfer Manager
//     PercentageTransferManager,
// }

/// The main trait for statistics module
pub trait Trait: frame_system::Trait + IdentityTrait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
    /// Asset module
    type Asset: AssetTrait<Self::Balance, Self::AccountId, Self::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as statistics {
        /// Transfer managers currently enabled for an Asset.
        pub ActiveTransferManagers get(fn transfer_managers): map hasher(blake2_128_concat) Ticker => Vec<TransferManager>;
        /// Number of current investors in an asset.
        pub InvestorCountPerAsset get(fn investor_count): map hasher(blake2_128_concat) Ticker => Counter;
        /// Identities exempt from transfer managers.
        pub ExemptIdentities get(fn identity_exempt):
            double_map
                hasher(blake2_128_concat) (Ticker, TransferManager),
                hasher(blake2_128_concat) IdentityId
            =>
                bool;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

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
        ///
        #[weight = 500_000_000]
        pub fn add_transfer_manager(origin, ticker: Ticker, new_transfer_manager: TransferManager) {
            let did = T::Asset::ensure_perms_owner_asset(origin, &ticker)?;
            ActiveTransferManagers::try_mutate(&ticker, |transfer_managers| {
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
        #[weight = 500_000_000]
        pub fn remove_transfer_manager(origin, ticker: Ticker, transfer_manager: TransferManager) {
            let did = T::Asset::ensure_perms_owner_asset(origin, &ticker)?;
            ActiveTransferManagers::try_mutate(&ticker, |transfer_managers| {
                let before = transfer_managers.len();
                transfer_managers.retain(|tm| *tm != transfer_manager);
                ensure!(before != transfer_managers.len(), Error::<T>::TransferManagerMissing);
                Ok(()) as DispatchResult
            })?;
            Self::deposit_event(Event::TransferManagerRemoved(did, ticker, transfer_manager));
        }

        /// Exempt identities from a transfer manager
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `ticker` ticker for which the exemption list is being modified.
        /// * `transfer_manager` Transfer manager for which the exemption list is being modified.
        /// * `identities` Transfer manager for which the exemption list is being modified.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the ticker.
        ///
        #[weight = 500_000_000]
        pub fn add_exempted_identities(origin, ticker: Ticker, transfer_manager: TransferManager, exempted_identities: Vec<IdentityId>) {
            let did = T::Asset::ensure_perms_owner_asset(origin, &ticker)?;
            let ticker_tm = (ticker.clone(), transfer_manager.clone());
            for identity in &exempted_identities {
                ExemptIdentities::insert(&ticker_tm, identity, true);
            }
            Self::deposit_event(Event::ExemptionsAdded(did, ticker, transfer_manager, exempted_identities));
        }

        /// remove identities from exemption list of a transfer manager
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e who signed the transaction to execute this function).
        /// * `ticker` ticker for which the exemption list is being modified.
        /// * `transfer_manager` Transfer manager for which the exemption list is being modified.
        /// * `identities` Transfer manager for which the exemption list is being modified.
        ///
        /// # Errors
        /// * `Unauthorized` if `origin` is not the owner of the ticker.
        ///
        #[weight = 500_000_000]
        pub fn remove_exempted_identities(origin, ticker: Ticker, transfer_manager: TransferManager, identities: Vec<IdentityId>) {
            let did = T::Asset::ensure_perms_owner_asset(origin, &ticker)?;
            let ticker_tm = (ticker.clone(), transfer_manager.clone());
            for identity in &identities {
                ExemptIdentities::remove(&ticker_tm, identity);
            }
            Self::deposit_event(Event::ExemptionsRemoved(did, ticker, transfer_manager, identities));
        }
    }
}

impl<T: Trait> Module<T> {
    /// It updates our statistics after transfer execution.
    /// The following counters could be updated:
    ///     - *Investor count per asset*.
    ///
    pub fn update_transfer_stats(
        ticker: &Ticker,
        updated_from_balance: Option<T::Balance>,
        updated_to_balance: Option<T::Balance>,
        amount: T::Balance,
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
                <InvestorCountPerAsset>::insert(ticker, new_counter)
            }
        }
    }
}

decl_event!(
    pub enum Event {
        /// A new transfer manager was added.
        TransferManagerAdded(IdentityId, Ticker, TransferManager),
        /// An existing transfer manager was removed.
        TransferManagerRemoved(IdentityId, Ticker, TransferManager),
        /// Identities were added to the exemption list.
        ExemptionsAdded(IdentityId, Ticker, TransferManager, Vec<IdentityId>),
        /// Identities were removed from the exemption list.
        ExemptionsRemoved(IdentityId, Ticker, TransferManager, Vec<IdentityId>),
    }
);

decl_error! {
    /// Statistics module errors.
    pub enum Error for Module<T: Trait> {
        /// The transfer manager already exists
        DuplicateTransferManager,
        /// Transfer manager is not enabled
        TransferManagerMissing,
    }
}
