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

//! Runtime API definition for the Portfolio module.

use codec::Codec;
use polymesh_primitives::{IdentityId, PortfolioId, PortfolioName, PortfolioNumber, Ticker};
use sp_std::vec::Vec;

pub type Error = Vec<u8>;
pub type GetPortfoliosResult = Result<Vec<(PortfolioNumber, PortfolioName)>, Error>;
pub type GetPortfolioAssetsResult<Balance> = Result<Vec<(Ticker, Balance)>, Error>;

sp_api::decl_runtime_apis! {
    /// The API to interact with Asset.
    pub trait PortfolioApi<Balance> where
        Balance: Codec
    {
        /// Gets all user-defined portfolio names of an identity.
        fn get_portfolios(did: IdentityId) -> GetPortfoliosResult;

        /// Gets the balances of all assets in a given portfolio.
        fn get_portfolio_assets(portfolio_id: PortfolioId) -> GetPortfolioAssetsResult<Balance>;
    }
}
