use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
pub use pallet_pips_rpc_runtime_api::{
    self as runtime_api, CappedVoteCount, PipsApi as PipsRuntimeApi, VoteCount,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, UniqueSaturatedInto},
};
use sp_std::{prelude::*, vec::Vec};
use std::sync::Arc;

/// Pips RPC methods.
#[rpc]
pub trait PipsApi<BlockHash, AccountId, Balance> {
    /// Summary of votes of a proposal given by `index`
    #[rpc(name = "pips_getVotes")]
    fn get_votes(&self, index: u32, at: Option<BlockHash>) -> Result<CappedVoteCount>;

    /// Retrieves proposal indices started by `address`
    #[rpc(name = "pips_proposedBy")]
    fn proposed_by(&self, address: AccountId, at: Option<BlockHash>) -> Result<Vec<u32>>;

    /// Retrieves proposal `address` indices voted on
    #[rpc(name = "pips_votedOn")]
    fn voted_on(&self, address: AccountId, at: Option<BlockHash>) -> Result<Vec<u32>>;
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

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl<C, Block, AccountId, Balance> PipsApi<<Block as BlockT>::Hash, AccountId, Balance>
    for Pips<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: PipsRuntimeApi<Block, AccountId, Balance>,
    AccountId: Codec,
    Balance: Codec + UniqueSaturatedInto<u64>,
{
    fn get_votes(
        &self,
        index: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<CappedVoteCount> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        api.get_votes(&at, index)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError as i64),
                message: "Unable to query get_votes.".into(),
                data: Some(format!("{:?}", e).into()),
            })
            .map(CappedVoteCount::new)
    }

    fn proposed_by(
        &self,
        address: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<u32>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let result = api.proposed_by(&at, address).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError as i64),
            message: "Unable to query proposed_by.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;

        Ok(result)
    }

    fn voted_on(
        &self,
        address: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<u32>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let result = api.voted_on(&at, address).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError as i64),
            message: "Unable to query voted_on.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;

        Ok(result)
    }
}
