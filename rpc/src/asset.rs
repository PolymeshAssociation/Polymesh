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

use std::{convert::TryInto, sync::Arc};

use codec::Codec;
use frame_support::dispatch::result::Result;
use frame_support::pallet_prelude::DispatchError;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::{CallError, ErrorCode, ErrorObject};
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_rpc::number;
use sp_runtime::traits::Block as BlockT;

pub use node_rpc_runtime_api::asset::{AssetApi as AssetRuntimeApi, CanTransferResult};
use polymesh_primitives::asset::GranularCanTransferResult;
use polymesh_primitives::{IdentityId, PortfolioId, Ticker};

use crate::Error;

#[rpc(client, server)]
pub trait AssetApi<BlockHash, AccountId> {
    #[method(name = "asset_canTransferGranular")]
    fn can_transfer_granular(
        &self,
        from_custodian: Option<IdentityId>,
        from_portfolio: PortfolioId,
        to_custodian: Option<IdentityId>,
        to_portfolio: PortfolioId,
        ticker: Ticker,
        value: number::NumberOrHex,
        at: Option<BlockHash>,
    ) -> RpcResult<Result<GranularCanTransferResult, DispatchError>>;
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

impl<C, Block, AccountId> AssetApiServer<<Block as BlockT>::Hash, AccountId> for Asset<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: AssetRuntimeApi<Block, AccountId>,
    AccountId: Codec,
{
    fn can_transfer_granular(
        &self,
        from_custodian: Option<IdentityId>,
        from_portfolio: PortfolioId,
        to_custodian: Option<IdentityId>,
        to_portfolio: PortfolioId,
        ticker: Ticker,
        value: number::NumberOrHex,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Result<GranularCanTransferResult, DispatchError>> {
        // Make sure that value fits into 64 bits.
        let value: u64 = value.try_into().map_err(|_| {
            CallError::Custom(ErrorObject::owned(
                ErrorCode::InvalidParams.code(),
                format!("{:?} doesn't fit in 64 bit unsigned value", value),
                None::<()>,
            ))
        })?;

        let api = self.client.runtime_api();
        // If the block hash is not supplied assume the best block.
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);
        // Gets the api version, returns an error if not found.
        let api_version = api
            .api_version::<dyn AssetRuntimeApi<Block, AccountId>>(at_hash)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to find the api version",
                    Some(e.to_string()),
                ))
            })?
            .ok_or(CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Api version cannot be None",
                Some("None version"),
            )))?;

        let api_call_result = {
            if api_version >= 3 {
                api.can_transfer_granular(
                    at_hash,
                    from_custodian,
                    from_portfolio,
                    to_custodian,
                    to_portfolio,
                    &ticker,
                    value.into(),
                )
            } else if api_version == 2 {
                #[allow(deprecated)]
                api.can_transfer_granular_before_version_3(
                    at_hash,
                    from_custodian,
                    from_portfolio,
                    to_custodian,
                    to_portfolio,
                    &ticker,
                    value.into(),
                )
                .map(|value| Ok(value))
            } else {
                #[allow(deprecated)]
                api.can_transfer_granular_before_version_2(
                    at_hash,
                    from_custodian,
                    from_portfolio,
                    to_custodian,
                    to_portfolio,
                    &ticker,
                    value.into(),
                )
                .map(|value| Ok(GranularCanTransferResult::from(value)))
            }
        };

        api_call_result.map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to call can_transfer_granular runtime",
                Some(e.to_string()),
            ))
            .into()
        })
    }
}
