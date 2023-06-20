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

use codec::Codec;
use frame_support::dispatch::result::Result;
use frame_support::pallet_prelude::DispatchError;
use sp_std::vec::Vec;

use polymesh_primitives::asset::GranularCanTransferResult;
use polymesh_primitives::{Balance, IdentityId, PortfolioId, Ticker};

/// The maximum number of DIDs allowed in a `balance_at` RPC query.
pub const MAX_BALANCE_AT_QUERY_SIZE: usize = 100;

pub type Error = Vec<u8>;
pub type CanTransferResult = Result<u8, Error>;

sp_api::decl_runtime_apis! {

    /// The API to interact with Asset.
    #[api_version(3)]
    pub trait AssetApi<AccountId>
    where
        AccountId: Codec,
    {
        /// Checks whether a transaction with given parameters can take place or not.
        /// The result is "granular" meaning each check is run and returned regardless of outcome.
        ///
        /// ```ignore
        /// curl http://localhost:9933 -H "Content-Type: application/json" -d '{
        ///     "id":1,
        ///     "jsonrpc":"2.0",
        ///     "method": "asset_canTransferGranular",
        ///     "params":[
        ///       "0x0100000000000000000000000000000000000000000000000000000000000000",
        ///       { "did": "0x0100000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///       "0x0200000000000000000000000000000000000000000000000000000000000000",
        ///       { "did": "0x0200000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///       "0x5449434B4552303030303031",
        ///        0
        ///     ]
        ///   }'
        /// ```
        fn can_transfer_granular(
            from_custodian: Option<IdentityId>,
            from_portfolio: PortfolioId,
            to_custodian: Option<IdentityId>,
            to_portfolio: PortfolioId,
            ticker: &Ticker,
            value: Balance
        ) -> Result<GranularCanTransferResult, DispatchError>;

        #[changed_in(3)]
        fn can_transfer_granular(
            from_custodian: Option<IdentityId>,
            from_portfolio: PortfolioId,
            to_custodian: Option<IdentityId>,
            to_portfolio: PortfolioId,
            ticker: &Ticker,
            value: Balance
        ) -> GranularCanTransferResult;

        #[changed_in(2)]
        fn can_transfer_granular(
            from_custodian: Option<IdentityId>,
            from_portfolio: PortfolioId,
            to_custodian: Option<IdentityId>,
            to_portfolio: PortfolioId,
            ticker: &Ticker,
            value: Balance
        ) -> polymesh_primitives::asset::v1::GranularCanTransferResult;
    }
}
