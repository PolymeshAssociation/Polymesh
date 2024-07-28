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

//! Runtime API definition for NFT module.

use frame_support::dispatch::DispatchError;
use sp_std::vec::Vec;

use polymesh_primitives::{NFTs, PortfolioId};

sp_api::decl_runtime_apis! {

    #[api_version(2)]
    pub trait NFTApi {
        /// Returns a vector containing all errors for the transfer. An empty vec means there's no error.
        ///
        /// ```ignore
        /// curl http://localhost:9933 -H "Content-Type: application/json" -d '{
        ///     "id":1,
        ///     "jsonrpc":"2.0",
        ///     "method": "nft_transferReport",
        ///     "params": [
        ///        { "did": "0x0100000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///        { "did": "0x0100000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///        { "asset_id": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "ids": [1]},
        ///        false
        ///     ]
        /// }'
        /// ```
        fn transfer_report(
            sender_portfolio: PortfolioId,
            receiver_portfolio: PortfolioId,
            nfts: NFTs,
            skip_locked_check: bool,
        ) -> Vec<DispatchError>;
    }
}
