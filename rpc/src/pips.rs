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

use codec::Codec;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
pub use node_rpc_runtime_api::pips::{
    self as runtime_api,
    capped::{Vote, VoteCount},
    PipsApi as PipsRuntimeApi,
};
use pallet_pips::PipId;
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_std::{prelude::*, vec::Vec};

/// Pips RPC methods.
#[rpc(client, server)]
pub trait PipsApi<BlockHash, AccountId> {
    /// Summary of votes of the proposal given by `id`.
    #[method(name = "pips_getVotes")]
    fn get_votes(&self, id: PipId, at: Option<BlockHash>) -> RpcResult<VoteCount>;

    /// Retrieves proposal indices started by `address`.
    #[method(name = "pips_proposedBy")]
    fn proposed_by(&self, address: AccountId, at: Option<BlockHash>) -> RpcResult<Vec<PipId>>;

    /// Retrieves proposal `address` indices voted on

    #[method(name = "pips_votedOn")]
    fn voted_on(&self, address: AccountId, at: Option<BlockHash>) -> RpcResult<Vec<PipId>>;
}

/// An implementation of pips specific RPC methods.
pub struct Pips<T, U> {
    client: Arc<T>,
    _marker: std::marker::PhantomData<U>,
}

impl<T, U> Pips<T, U> {
    /// Create new `Pips` with the given reference to the client.
    pub fn new(client: Arc<T>) -> Self {
        Pips {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> PipsApiServer<<Block as BlockT>::Hash, AccountId> for Pips<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: PipsRuntimeApi<Block, AccountId>,
    AccountId: Codec,
{
    fn get_votes(&self, id: PipId, at: Option<<Block as BlockT>::Hash>) -> RpcResult<VoteCount> {
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| api.get_votes(at, id),
            "Unable to query `get_votes`."
        )
        .map(VoteCount::from)
    }

    fn proposed_by(
        &self,
        address: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<PipId>> {
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| api.proposed_by(at, address),
            "Unable to query `proposed_by`."
        )
    }

    fn voted_on(
        &self,
        address: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<PipId>> {
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| api.voted_on(at, address),
            "Unable to query `voted_on`."
        )
    }
}
