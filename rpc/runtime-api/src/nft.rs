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

use frame_support::pallet_prelude::DispatchError;
use sp_std::vec::Vec;

use polymesh_primitives::asset::AssetId;
use polymesh_primitives::{AccountId, AssetHolder, NFTId, NFTs, PortfolioId};

sp_api::decl_runtime_apis! {

    #[api_version(4)]
    pub trait NFTApi {
        #[changed_in(3)]
        fn transfer_report(
            sender_portfolio: PortfolioId,
            receiver_portfolio: PortfolioId,
            nfts: NFTs,
            skip_locked_check: bool,
        ) -> Vec<DispatchError>;

        /// Returns a vector containing all errors for the transfer. An empty vec means there's no error.
        fn transfer_report(
            sender: AssetHolder,
            receiver: AssetHolder,
            nfts: NFTs,
            skip_locked_check: bool,
        ) -> Vec<DispatchError>;

        /// Returns the account approved to transfer `nft_id`, if any.
        ///
        /// This is the ERC-721 `getApproved`.
        fn token_approval(asset_id: AssetId, nft_id: NFTId) -> Option<AccountId>;

        /// Returns `true` if `operator` may transfer any NFT of `asset_id` held by `owner`.
        ///
        /// This is the ERC-721 `isApprovedForAll`, scoped to a single collection.
        fn operator_approval(owner: AccountId, operator: AccountId, asset_id: AssetId) -> bool;
    }
}
