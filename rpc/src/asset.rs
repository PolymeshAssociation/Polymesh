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

use std::sync::Arc;

use frame_support::pallet_prelude::DispatchError;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::{CallError, ErrorObject};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

pub use node_rpc_runtime_api::asset::AssetApi as AssetRuntimeApi;
use polymesh_primitives::asset::AssetID;
use polymesh_primitives::{Balance, PortfolioId};

use crate::Error;

#[rpc(client, server)]
pub trait AssetApi<BlockHash> {
    #[method(name = "asset_transferReport")]
    fn transfer_report(
        &self,
        sender_portfolio: PortfolioId,
        receiver_portfolio: PortfolioId,
        asset_id: AssetID,
        transfer_value: Balance,
        skip_locked_check: bool,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<DispatchError>>;
}

/// An implementation of asset specific RPC methods.
pub struct Asset<T, U> {
    client: Arc<T>,
    _marker: std::marker::PhantomData<U>,
}

impl<T, U> Asset<T, U> {
    /// Create new `Asset` with the given reference to the client.
    pub fn new(client: Arc<T>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<T, Block> AssetApiServer<<Block as BlockT>::Hash> for Asset<T, Block>
where
    Block: BlockT,
    T: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    T::Api: AssetRuntimeApi<Block>,
{
    fn transfer_report(
        &self,
        sender_portfolio: PortfolioId,
        receiver_portfolio: PortfolioId,
        asset_id: AssetID,
        transfer_value: Balance,
        skip_locked_check: bool,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<DispatchError>> {
        let api = self.client.runtime_api();
        // If the block hash is not supplied assume the best block.
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.transfer_report(
            at_hash,
            sender_portfolio,
            receiver_portfolio,
            asset_id,
            transfer_value,
            skip_locked_check,
        )
        .map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to call asset_transfer_report runtime",
                Some(e.to_string()),
            ))
            .into()
        })
    }
}
