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
use jsonrpsee::types::error::ErrorObject;
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

pub use node_rpc_runtime_api::nft::NFTApi as NFTRuntimeApi;
use polymesh_primitives::{NFTs, PortfolioId};

#[rpc(client, server)]
pub trait NFTApi<BlockHash> {
    #[method(name = "nft_transferReport")]
    fn transfer_report(
        &self,
        sender_portfolio: PortfolioId,
        receiver_portfolio: PortfolioId,
        nfts: NFTs,
        skip_locked_check: bool,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<DispatchError>>;
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

impl<T, Block> NFTApiServer<<Block as BlockT>::Hash> for NFT<T, Block>
where
    Block: BlockT,
    T: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    T::Api: NFTRuntimeApi<Block>,
{
    fn transfer_report(
        &self,
        sender_portfolio: PortfolioId,
        receiver_portfolio: PortfolioId,
        nfts: NFTs,
        skip_locked_check: bool,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<DispatchError>> {
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<T as ProvideRuntimeApi<Block>>::Api>, at| api.transfer_report(
                at,
                sender_portfolio,
                receiver_portfolio,
                nfts,
                skip_locked_check,
            ),
            "Unable to query `transfer_report`."
        )
    }
}
