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

//! # Trait Interface to the Portfolio Module
//!
//! The interface allows to accept portfolio custody

use frame_support::decl_event;
use frame_support::pallet_prelude::Get;
use frame_support::weights::Weight;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::traits::{AssetFnConfig, NFTTrait};
use polymesh_primitives::{
    Fund, FundDescription, IdentityId, Memo, PortfolioId, PortfolioName, PortfolioNumber,
};
use sp_std::vec::Vec;

pub trait WeightInfo {
    fn create_portfolio(l: u32) -> Weight;
    fn delete_portfolio() -> Weight;
    fn rename_portfolio(i: u32) -> Weight;
    fn quit_portfolio_custody() -> Weight;
    fn accept_portfolio_custody() -> Weight;
    fn pre_approve_portfolio() -> Weight;
    fn remove_portfolio_pre_approval() -> Weight;
    fn move_portfolio(funds: &[Fund]) -> Weight {
        let (f, n) = count_token_moves(funds);
        Self::move_portfolio_funds(f, n)
    }
    fn move_portfolio_funds(f: u32, u: u32) -> Weight;
    fn allow_identity_to_create_portfolios() -> Weight;
    fn revoke_create_portfolios_permission() -> Weight;
    fn create_custody_portfolio() -> Weight;
}

pub trait Config:
    frame_system::Config + pallet_permissions::Config + pallet_identity::Config + AssetFnConfig
{
    type RuntimeEvent: From<Event> + Into<<Self as frame_system::Config>::RuntimeEvent>;
    type WeightInfo: WeightInfo;
    /// Maximum number of fungible assets that can be moved in a single transfer call.
    type MaxNumberOfFungibleMoves: Get<u32>;
    /// Maximum number of NFTs that can be moved in a single transfer call.
    type MaxNumberOfNFTsMoves: Get<u32>;
    /// NFT module - required for updating the ownership of an NFT.
    type NFT: NFTTrait<Self::RuntimeOrigin>;
}

decl_event! {
    pub enum Event {
        /// The portfolio has been successfully created.
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio number
        /// * portfolio name
        PortfolioCreated(IdentityId, PortfolioNumber, PortfolioName),
        /// The portfolio has been successfully removed.
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio number
        PortfolioDeleted(IdentityId, PortfolioNumber),
        /// The portfolio identified with `num` has been renamed to `name`.
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio number
        /// * portfolio name
        PortfolioRenamed(IdentityId, PortfolioNumber, PortfolioName),
        /// All non-default portfolio numbers and names of a DID.
        ///
        /// # Parameters
        /// * origin DID
        /// * vector of number-name pairs
        UserPortfolios(IdentityId, Vec<(PortfolioNumber, PortfolioName)>),
        /// Custody of a portfolio has been given to a different identity
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio id
        /// * portfolio custodian did
        PortfolioCustodianChanged(IdentityId, PortfolioId, IdentityId),
        /// Funds have moved between portfolios
        ///
        /// # Parameters
        /// * Origin DID.
        /// * Source portfolio.
        /// * Destination portfolio.
        /// * The type of fund that was moved.
        /// * Optional memo for the move.
        FundsMovedBetweenPortfolios(
            IdentityId,
            PortfolioId,
            PortfolioId,
            FundDescription,
            Option<Memo>
        ),
        /// A portfolio has pre approved the receivement of an asset.
        ///
        /// # Parameters
        /// * [`IdentityId`] of the caller.
        /// * [`PortfolioId`] that will receive assets without explicit affirmation.
        /// * [`AssetId`] of the asset that has been exempt from explicit affirmation.
        PreApprovedPortfolio(
            IdentityId,
            PortfolioId,
            AssetId
        ),
        /// A portfolio has removed the approval of an asset.
        ///
        /// # Parameters
        /// * [`IdentityId`] of the caller.
        /// * [`PortfolioId`] that had its pre approval revoked.
        /// * [`AssetId`] of the asset that had its pre approval revoked.
        RevokePreApprovedPortfolio(
            IdentityId,
            PortfolioId,
            AssetId
        ),
        /// Allow another identity to create portfolios.
        ///
        /// # Parameters
        /// * [`IdentityId`] of the caller.
        /// * [`IdentityId`] allowed to create portfolios.
        AllowIdentityToCreatePortfolios(
            IdentityId,
            IdentityId,
        ),
        /// Revoke another identities permission to create portfolios.
        ///
        /// # Parameters
        /// * [`IdentityId`] of the caller.
        /// * [`IdentityId`] permissions to create portfolios is revoked.
        RevokeCreatePortfoliosPermission(
            IdentityId,
            IdentityId,
        ),
    }
}

fn count_token_moves(funds: &[Fund]) -> (u32, u32) {
    let mut fungible_moves = 0;
    let mut nfts_moves = 0;
    for fund in funds {
        match &fund.description {
            FundDescription::Fungible { .. } => {
                fungible_moves += 1;
            }
            FundDescription::NonFungible(nfts) => {
                nfts_moves += nfts.len();
            }
        }
    }
    (fungible_moves, nfts_moves as u32)
}
