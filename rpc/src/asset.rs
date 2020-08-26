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

pub use node_rpc_runtime_api::asset::{AssetApi as AssetRuntimeApi, CanTransferResult};
use polymesh_primitives::{IdentityId, Ticker};

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;

use codec::Codec;
use jsonrpc_core::Error;
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_rpc::number;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::convert::TryInto;
use std::sync::Arc;

#[rpc]
pub trait AssetApi<BlockHash, AccountId> {
    #[rpc(name = "asset_canTransfer")]
    fn can_transfer(
        &self,
        sender: AccountId,
        ticker: Ticker,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        value: number::NumberOrHex,
        at: Option<BlockHash>,
    ) -> Result<CanTransferResult>;
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

impl<C, Block, AccountId> AssetApi<<Block as BlockT>::Hash, AccountId> for Asset<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: AssetRuntimeApi<Block, AccountId>,
    AccountId: Codec,
{
    fn can_transfer(
        &self,
        sender: AccountId,
        ticker: Ticker,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        value: number::NumberOrHex,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<CanTransferResult> {
        // Make sure that value fits into 128 bits.
        let value: u64 = value.try_into().map_err(|_| Error {
            code: ErrorCode::InvalidParams,
            message: format!("{:?} doesn't fit in 64 bit unsigned value", value),
            data: None,
        })?;
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| api.can_transfer(
                at,
                sender,
                ticker,
                from_did,
                to_did,
                value.into()
            ),
            "Unable to check transfer"
        )
    }
}
