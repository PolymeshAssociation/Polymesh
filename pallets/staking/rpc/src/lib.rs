use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use node_rpc::Error;
pub use pallet_staking_rpc_runtime_api::StakingApi as StakingRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{traits::Block as BlockT, Perbill};
use std::sync::Arc;

#[rpc(client, server)]
pub trait StakingApi<BlockHash> {
    #[method(name = "staking_getCurve")]
    fn get_curve(&self, at: Option<BlockHash>) -> RpcResult<Vec<(Perbill, Perbill)>>;
}

/// A struct that implements the [`StakingApi`].
pub struct Staking<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Staking<C, M> {
    /// Create new `Staking` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> StakingApiServer<<Block as BlockT>::Hash> for Staking<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: StakingRuntimeApi<Block>,
{
    fn get_curve(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<(Perbill, Perbill)>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash);

        api.get_curve(at_hash).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get curve.",
                Some(e.to_string()),
            ))
            .into()
        })
    }
}
