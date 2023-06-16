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

use frame_support::dispatch::DispatchResult;

use polymesh_primitives::{NFTs, PortfolioId};

sp_api::decl_runtime_apis! {

    pub trait NFTApi {
        /// Verifies if the given NFTs can be transferred from `sender_portfolio` to `receiver_portfolio`.
        /// In order for the transfer to be successfull, the following conditions must hold:
        /// The sender and receiver are not the same, both portfolios have valid balances, the sender owns the nft,
        /// all compliance rules are being respected, and no duplicate nfts being transferred.
        ///
        /// ```ignore
        /// curl http://localhost:9933 -H "Content-Type: application/json" -d '{
        ///     "id":1,
        ///     "jsonrpc":"2.0",
        ///     "method": "nft_validateNFTTransfer",
        ///     "params":[
        ///       { "did": "0x0100000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///       { "did": "0x0200000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///       { "ticker": "0x5449434B4552303030303031", "ids": [1]}
        ///     ]
        ///   }'
        /// ```
        fn validate_nft_transfer(sender_portfolio: &PortfolioId, receiver_portfolio: &PortfolioId, nfts: &NFTs) -> DispatchResult;
    }
}
