pub use pallet_group_rpc_runtime_api::{GroupApi as GroupRuntimeApi, Member};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use std::{marker::PhantomData, sync::Arc};

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use node_rpc::Error;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_std::prelude::*;

/// Group RPC methods.
#[rpc(client, server)]
pub trait GroupApi<BlockHash> {
    /// Valid members: active member and inactive who are not yet expired.
    #[method(name = "group_getCDDValidMembers")]
    fn get_cdd_valid_members(&self, at: Option<BlockHash>) -> RpcResult<Vec<Member>>;

    #[method(name = "group_getGCValidMembers")]
    fn get_gc_valid_members(&self, at: Option<BlockHash>) -> RpcResult<Vec<Member>>;
}

pub struct Group<T, U> {
    client: Arc<T>,
    _marker: PhantomData<U>,
}

impl<T, U> From<Arc<T>> for Group<T, U> {
    fn from(client: Arc<T>) -> Self {
        Group {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> GroupApiServer<<Block as BlockT>::Hash> for Group<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: GroupRuntimeApi<Block>,
{
    fn get_cdd_valid_members(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<Member>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_cdd_valid_members(at_hash).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to fetch CDD providers.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_gc_valid_members(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<Member>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_gc_valid_members(at_hash).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to fetch Governance Committee members.",
                Some(e.to_string()),
            ))
            .into()
        })
    }
}
