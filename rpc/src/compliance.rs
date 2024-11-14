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

use frame_support::dispatch::DispatchError;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::{CallError, ErrorObject};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

pub use node_rpc_runtime_api::compliance::ComplianceApi as ComplianceRuntimeApi;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::compliance_manager::ComplianceReport;
use polymesh_primitives::IdentityId;

use crate::Error;

#[rpc(client, server)]
pub trait ComplianceApi<BlockHash> {
    #[method(name = "compliance_complianceReport")]
    fn compliance_report(
        &self,
        asset_id: AssetId,
        sender_identity: IdentityId,
        receiver_identity: IdentityId,
        at: Option<BlockHash>,
    ) -> RpcResult<Result<ComplianceReport, DispatchError>>;
}

/// An implementation of Compliance specific RPC methods.
pub struct Compliance<T, U> {
    client: Arc<T>,
    _marker: std::marker::PhantomData<U>,
}

impl<T, U> Compliance<T, U> {
    /// Creates a new `Compliance` with the given reference to the client.
    pub fn new(client: Arc<T>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<T, Block> ComplianceApiServer<<Block as BlockT>::Hash> for Compliance<T, Block>
where
    Block: BlockT,
    T: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    T::Api: ComplianceRuntimeApi<Block>,
{
    fn compliance_report(
        &self,
        asset_id: AssetId,
        sender_identity: IdentityId,
        receiver_identity: IdentityId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Result<ComplianceReport, DispatchError>> {
        let api = self.client.runtime_api();
        // If the block hash is not supplied assume the best block.
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.compliance_report(at_hash, &asset_id, &sender_identity, &receiver_identity)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to call compliance_report runtime",
                    Some(e.to_string()),
                ))
                .into()
            })
    }
}
