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

//! Runtime API definition for Identity module.

use codec::Codec;
use polymesh_primitives::{IdentityId, Ticker};
use sp_std::vec::Vec;

pub type Error = Vec<u8>;
pub type CanTransferResult = Result<u8, Error>;

sp_api::decl_runtime_apis! {

    /// The API to interact with Asset.
    pub trait AssetApi<AccountId, Balance>
    where
        AccountId: Codec,
        Balance: Codec
    {
         /// Checks whether a transaction with given parameters can take place or not.
         ///
         /// # Example
         ///
         /// In this example we are checking if Alice can transfer 500 of ticket 0x01
         /// from herself (Id=0x2a) to Bob (Id=0x3905)
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
            ticker: Ticker,
            from_did: Option<IdentityId>,
            to_did: Option<IdentityId>,
            value: Balance
        ) -> CanTransferResult;
    }
}
