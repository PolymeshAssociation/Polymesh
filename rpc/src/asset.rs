// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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

pub use node_rpc_runtime_api::asset::{AssetApi as AssetRuntimeApi, CanTransferResult};

use std::{convert::TryInto, sync::Arc};

use crate::Error;
use codec::Codec;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use polymesh_primitives::asset::GranularCanTransferResult;
use polymesh_primitives::{IdentityId, PortfolioId, Ticker};
use sp_api::{ApiExt, ApiRef, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_rpc::number;
use sp_runtime::traits::Block as BlockT;

#[rpc(client, server)]
pub trait AssetApi<BlockHash, AccountId> {
    #[method(name = "asset_canTransfer")]
    fn can_transfer(
        &self,
        sender: AccountId,
        from_custodian: Option<IdentityId>,
        from_portfolio: PortfolioId,
        to_custodian: Option<IdentityId>,
        to_portfolio: PortfolioId,
        ticker: Ticker,
        value: number::NumberOrHex,
        at: Option<BlockHash>,
    ) -> RpcResult<CanTransferResult>;

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
    ) -> RpcResult<GranularCanTransferResult>;
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
    fn can_transfer(
        &self,
        sender: AccountId, // Keeping this here to avoid breaking API.
        from_custodian: Option<IdentityId>,
        from_portfolio: PortfolioId,
        to_custodian: Option<IdentityId>,
        to_portfolio: PortfolioId,
        ticker: Ticker,
        value: number::NumberOrHex,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<CanTransferResult> {
        // Make sure that value fits into 64 bits.
        let value: u64 = value.try_into().map_err(|_| {
            CallError::Custom(ErrorObject::owned(
                ErrorCode::InvalidParams.code(),
                format!("{:?} doesn't fit in 64 bit unsigned value", value),
                None::<()>,
            ))
        })?;
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| api.can_transfer(
                at,
                sender,
                from_custodian,
                from_portfolio,
                to_custodian,
                to_portfolio,
                &ticker,
                value.into()
            ),
            "Unable to check transfer"
        )
    }

    fn can_transfer_granular(
        &self,
        from_custodian: Option<IdentityId>,
        from_portfolio: PortfolioId,
        to_custodian: Option<IdentityId>,
        to_portfolio: PortfolioId,
        ticker: Ticker,
        value: number::NumberOrHex,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<GranularCanTransferResult> {
        // Make sure that value fits into 64 bits.
        let value: u64 = value.try_into().map_err(|_| {
            CallError::Custom(ErrorObject::owned(
                ErrorCode::InvalidParams.code(),
                format!("{:?} doesn't fit in 64 bit unsigned value", value),
                None::<()>,
            ))
        })?;
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);
        let api_version = api
            .api_version::<dyn AssetRuntimeApi<Block, AccountId>>(at_hash)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to check transfer",
                    Some(e.to_string()),
                ))
            })?;

        match api_version {
            Some(version) if version >= 2 => api.can_transfer_granular(
                at_hash,
                from_custodian,
                from_portfolio,
                to_custodian,
                to_portfolio,
                &ticker,
                value.into(),
            ),
            Some(1) =>
            {
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
                .map(|res| res.into())
            }
            _ => {
                return Err(CallError::Custom(ErrorObject::owned(
                    ErrorCode::MethodNotFound.code(),
                    format!("Cannot find `AssetApi` for block {:?}", at),
                    None::<()>,
                ))
                .into());
            }
        }
        .map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to check transfer",
                Some(e.to_string()),
            ))
            .into()
        })
    }
}
