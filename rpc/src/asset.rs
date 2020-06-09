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
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use std::sync::Arc;

#[rpc]
pub trait AssetApi<BlockHash, AccountId, T> {
    #[rpc(name = "asset_canTransfer")]
    fn can_transfer(
        &self,
        sender: AccountId,
        ticker: Ticker,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        value: T,
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

impl<C, Block, AccountId, T> AssetApi<<Block as BlockT>::Hash, AccountId, T> for Asset<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: AssetRuntimeApi<Block, AccountId, T>,
    AccountId: Codec,
    T: Codec,
{
    fn can_transfer(
        &self,
        sender: AccountId,
        ticker: Ticker,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        value: T,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<CanTransferResult> {
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| api
                .can_transfer(at, sender, ticker, from_did, to_did, value),
            "Unable to check transfer"
        )
    }
}
