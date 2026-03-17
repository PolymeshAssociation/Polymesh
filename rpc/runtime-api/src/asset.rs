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

//! Runtime API definition for Asset module.

use frame_support::pallet_prelude::DispatchError;
use sp_std::vec::Vec;

use polymesh_primitives::asset::AssetId;
use polymesh_primitives::{AccountId, AssetHolder, Balance, PortfolioId};

/// The maximum number of DIDs allowed in a `balance_at` RPC query.
pub const MAX_BALANCE_AT_QUERY_SIZE: usize = 100;

pub type Error = Vec<u8>;

sp_api::decl_runtime_apis! {
    #[api_version(5)]
    pub trait AssetApi {
        /// Returns a vector containing all errors for the transfer. An empty vec means there's no error.
        #[changed_in(5)]
        fn transfer_report(
            sender_portfolio: PortfolioId,
            receiver_portfolio: PortfolioId,
            asset_id: AssetId,
            transfer_value: Balance,
            skip_locked_check: bool,
        ) -> Vec<DispatchError>;

        /// Returns a vector containing all errors for the transfer. An empty vec means there's no error.
        fn transfer_report(
            sender: AssetHolder,
            receiver: AssetHolder,
            asset_id: AssetId,
            transfer_value: Balance,
            skip_locked_check: bool,
        ) -> Vec<DispatchError>;

        /// Returns the remaining allowance that `spender` can transfer from `owner`
        /// for the given `asset_id`. Returns 0 if no allowance exists.
        /// `Balance::MAX` (u128::MAX) indicates an unlimited allowance.
        fn allowance(
            owner: AccountId,
            spender: AccountId,
            asset_id: AssetId,
        ) -> Balance;
    }
}
