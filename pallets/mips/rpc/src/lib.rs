use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_mips_rpc_runtime_api::MipsApi as MipsRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, ProvideRuntimeApi},
    Perbill,
};
use std::sync::Arc;

#[rpc]
pub trait MipsApi<BlockHash> {
    #[rpc(name = "mips_getVotes")]
    fn get_votes(&self, at: Option<BlockHash>) -> Result<Vec<u32>>;
}

/// A struct that implements the [`StakingApi`].
pub struct Mips<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Mips<C, M> {
    /// Create new `Mips` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
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

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

impl<C, Block> MipsApi<<Block as BlockT>::Hash> for Mips<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi,
    C: HeaderBackend<Block>,
    C::Api: MipsRuntimeApi<Block>,
{
    fn get_votes(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<u32>> {
        let api = self.client.runtime_api();

        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        api.get_votes(&at).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to query dispatch info.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
