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

use polymesh_common_utilities::balances::Trait as BalancesTrait;
use polymesh_primitives::Ticker;

use frame_support::{decl_module, decl_storage};

pub type Counter = u64;

pub trait Trait: BalancesTrait {}

decl_storage! {
    trait Store for Module<T: Trait> as statistics {
        /// Number of investor per asset.
        pub InvestorCountPerAsset get(fn investor_count_per_asset): map hasher(blake2_128_concat) Ticker => Counter ;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
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
            let counter = Self::investor_count_per_asset(ticker);
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
