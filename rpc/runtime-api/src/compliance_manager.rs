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
use frame_support::traits::Currency;
use pallet_compliance_manager::AssetComplianceResult;
use polymesh_primitives::{IdentityId, Ticker};

pub trait Trait: frame_system::Trait {
    type Currency: Currency<Self::AccountId>;
}

sp_api::decl_runtime_apis! {

    /// The API to interact with Compliance manager.
    pub trait ComplianceManagerApi<AccountId, Balance>
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
         ///        "method": "compliance_canTransfer",
         ///        "params":[
         ///            "0x010000000000000000000000",
         ///            "0x2a00000000000000000000000000000000000000000000000000000000000000",
         ///            "0x3905000000000000000000000000000000000000000000000000000000000000"
         ///            ]
         ///       }
         ///    http://localhost:9933 | python3 -m json.tool
         /// ```
        fn can_transfer(
            ticker: Ticker,
            from_did: Option<IdentityId>,
            to_did: Option<IdentityId>,
        ) -> AssetComplianceResult;
    }
}
