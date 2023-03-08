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

use std::sync::Arc;

use frame_support::dispatch::DispatchResult;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::generic::BlockId;
use sp_runtime::traits::Block as BlockT;

pub use node_rpc_runtime_api::nft::NFTApi as NFTRuntimeApi;
use polymesh_primitives::{NFTs, PortfolioId};

#[rpc]
pub trait NFTApi<BlockHash> {
    #[rpc(name = "nft_validateNFTTransfer")]
    fn validate_nft_transfer(
        &self,
        sender_portfolio: PortfolioId,
        receiver_portfolio: PortfolioId,
        nfts: NFTs,
        at: Option<BlockHash>,
    ) -> Result<DispatchResult>;
}

/// An implementation of NFT specific RPC methods.
pub struct NFT<T, U> {
    client: Arc<T>,
    _marker: std::marker::PhantomData<U>,
}

impl<T, U> NFT<T, U> {
    /// Creates a new `NFT` with the given reference to the client.
    pub fn new(client: Arc<T>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<T, Block> NFTApi<<Block as BlockT>::Hash> for NFT<T, Block>
where
    Block: BlockT,
    T: Send + Sync + 'static,
    T: ProvideRuntimeApi<Block>,
    T: HeaderBackend<Block>,
    T::Api: NFTRuntimeApi<Block>,
{
    fn validate_nft_transfer(
        &self,
        sender_portfolio: PortfolioId,
        receiver_portfolio: PortfolioId,
        nfts: NFTs,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<DispatchResult> {
        let api = self.client.runtime_api();
        // If the block hash is not supplied assume the best block.
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        api.validate_nft_transfer(&at, &sender_portfolio, &receiver_portfolio, &nfts)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(crate::Error::RuntimeError as i64),
                message: "Unable to call validate_nft_transfer runtime".into(),
                data: Some(format!("{:?}", e).into()),
            })
    }
}
