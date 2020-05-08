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

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_asset_rpc_runtime_api::{AssetApi as AssetRuntimeApi, CanTransferResult};
use polymesh_primitives::{IdentityId, Ticker};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

use frame_support::traits::Currency;
pub trait Trait: frame_system::Trait {
    type Currency: Currency<Self::AccountId>;
}

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

#[rpc]
pub trait AssetApi<BlockHash, AccountId, T> {
    #[rpc(name = "asset_canTransfer")]
    fn can_transfer(
        &self,
        sender: AccountId,
        ticker: Ticker,
        from_did: IdentityId,
        to_did: IdentityId,
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

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
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
        from_did: IdentityId,
        to_did: IdentityId,
        value: T,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<CanTransferResult> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
                // If the block hash is not supplied assume the best block.
                self.client.info().best_hash));

        api.can_transfer(&at, sender, ticker, from_did, to_did, value)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError as i64),
                message: "Unable to check trnsfer".into(),
                data: Some(format!("{:?}", e).into()),
            })
    }
}
