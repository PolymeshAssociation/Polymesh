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
use polymesh_primitives::{Balance, IdentityId, PortfolioId, Ticker};
use sp_std::vec::Vec;

/// The maximum number of DIDs allowed in a `balance_at` RPC query.
pub const MAX_BALANCE_AT_QUERY_SIZE: usize = 100;

pub type Error = Vec<u8>;
pub type CanTransferResult = Result<u8, Error>;

sp_api::decl_runtime_apis! {

    /// The API to interact with Asset.
    #[api_version(2)]
    pub trait AssetApi<AccountId>
    where
        AccountId: Codec,
    {
        /// Checks whether a transaction with given parameters can take place or not.
        ///
        /// # Example
        ///
        /// In this example we are checking if Alice can transfer 500 of ticket 0x01
        /// from herself (Id=0x2a) to Bob (Id=0x3905)
        ///
        /// TODO: update example
        ///
        /// ```ignore
        ///  curl
        ///    -H "Content-Type: application/json"
        ///    -d {
        ///        "id":1, "jsonrpc":"2.0",
        ///        "method": "asset_canTransfer",
        ///        "params":[
        ///            "5CoRaw9Ex4DUjGcnPbPBnc2nez5ZeTmM5WL3ZDVLZzM6eEgE",
        ///            "0x010000000000000000000000",
        ///            "0x2a00000000000000000000000000000000000000000000000000000000000000",
        ///            "0x3905000000000000000000000000000000000000000000000000000000000000",
        ///            500]}
        ///    http://localhost:9933 | python3 -m json.tool
        /// ```
        fn can_transfer(
            sender: AccountId,
            from_custodian: Option<IdentityId>,
            from_portfolio: PortfolioId,
            to_custodian: Option<IdentityId>,
            to_portfolio: PortfolioId,
            ticker: &Ticker,
            value: Balance
        ) -> CanTransferResult;

        /// Checks whether a transaction with given parameters can take place or not.
        /// The result is "granular" meaning each check is run and returned regardless of outcome.
        fn can_transfer_granular(
            from_custodian: Option<IdentityId>,
            from_portfolio: PortfolioId,
            to_custodian: Option<IdentityId>,
            to_portfolio: PortfolioId,
            ticker: &Ticker,
            value: Balance
        ) -> polymesh_primitives::asset::GranularCanTransferResult;

        /// Checks whether a transaction with given parameters can take place or not.
        /// The result is "granular" meaning each check is run and returned regardless of outcome.
        ///
        /// v1 call with older TransferManagers (max investor count, max % ownership).
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
