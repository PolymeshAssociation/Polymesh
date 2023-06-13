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

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use node_rpc::Error;
pub use pallet_protocol_fee_rpc_runtime_api::{CappedFee, ProtocolFeeApi as ProtocolFeeRuntimeApi};
use polymesh_common_utilities::protocol_fee::ProtocolOp;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[rpc(client, server)]
pub trait ProtocolFeeApi<BlockHash> {
    #[method(name = "protocolFee_computeFee")]
    fn compute_fee(&self, op: ProtocolOp, at: Option<BlockHash>) -> RpcResult<CappedFee>;
}

/// A struct that implements the [`ProtocolFeeApi`].
pub struct ProtocolFee<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> ProtocolFee<C, M> {
    /// Create new `ProtocolFee` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> ProtocolFeeApiServer<<Block as BlockT>::Hash> for ProtocolFee<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: ProtocolFeeRuntimeApi<Block>,
{
    fn compute_fee(
        &self,
        op: ProtocolOp,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<CappedFee> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash);

        api.compute_fee(at_hash, op).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query dispatch info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }
}
