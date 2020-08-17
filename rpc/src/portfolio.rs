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

pub use node_rpc_runtime_api::portfolio::{
    GetPortfolioAssetsResult, GetPortfoliosResult, PortfolioApi as PortfolioRuntimeApi,
};

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use polymesh_primitives::{IdentityId, PortfolioId};
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait PortfolioApi<BlockHash, Balance> {
    /// Gets all user-defined portfolio names of an identity.
    #[rpc(name = "portfolio_getPortfolios")]
    fn get_portfolios(&self, did: IdentityId, at: Option<BlockHash>)
        -> Result<GetPortfoliosResult>;

    /// Gets the balances of all assets in a given portfolio.
    #[rpc(name = "portfolio_getPortfolioAssets")]
    fn get_portfolio_assets(
        &self,
        portfolio_id: PortfolioId,
        at: Option<BlockHash>,
    ) -> Result<GetPortfolioAssetsResult<Balance>>;
}

/// An implementation of portfolio-specific RPC methods.
pub struct Portfolio<T, U> {
    client: Arc<T>,
    _marker: std::marker::PhantomData<U>,
}

impl<T, U> Portfolio<T, U> {
    /// Create new `Asset` with the given reference to the client.
    pub fn new(client: Arc<T>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, Balance> PortfolioApi<<Block as BlockT>::Hash, Balance> for Portfolio<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: PortfolioRuntimeApi<Block, Balance>,
    Balance: Codec,
{
    fn get_portfolios(
        &self,
        did: IdentityId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<GetPortfoliosResult> {
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| api.get_portfolios(at, did),
            "Unable to get portfolios"
        )
    }

    fn get_portfolio_assets(
        &self,
        portfolio_id: PortfolioId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<GetPortfolioAssetsResult<Balance>> {
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| api
                .get_portfolio_assets(at, portfolio_id),
            "Unable to get portfolio assets"
        )
    }
}
