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

use sp_std::vec::Vec;
use std::sync::Arc;

use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::{CallError, ErrorObject};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

pub use node_rpc_runtime_api::settlement::SettlementApi as SettlementRuntimeApi;
use polymesh_primitives::settlement::{AffirmationCount, ExecuteInstructionInfo, InstructionId};
use polymesh_primitives::PortfolioId;

use crate::Error;

#[rpc(client, server)]
pub trait SettlementApi<BlockHash> {
    #[method(name = "settlement_getExecuteInstructionInfo")]
    fn get_execute_instruction_info(
        &self,
        instruction_id: InstructionId,
        at: Option<BlockHash>,
    ) -> RpcResult<ExecuteInstructionInfo>;

    #[method(name = "settlement_getAffirmationCount")]
    fn get_affirmation_count(
        &self,
        instruction_id: InstructionId,
        portfolios: Vec<PortfolioId>,
        at: Option<BlockHash>,
    ) -> RpcResult<AffirmationCount>;
}

/// An implementation of Settlement specific RPC methods.
pub struct Settlement<T, U> {
    client: Arc<T>,
    _marker: std::marker::PhantomData<U>,
}

impl<T, U> Settlement<T, U> {
    /// Creates a new `Settlement` with the given reference to the client.
    pub fn new(client: Arc<T>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<T, Block> SettlementApiServer<<Block as BlockT>::Hash> for Settlement<T, Block>
where
    Block: BlockT,
    T: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    T::Api: SettlementRuntimeApi<Block>,
{
    fn get_execute_instruction_info(
        &self,
        instruction_id: InstructionId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<ExecuteInstructionInfo> {
        let api = self.client.runtime_api();
        // If the block hash is not supplied assume the best block.
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_execute_instruction_info(at_hash, &instruction_id)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to call get_execute_instruction_info runtime",
                    Some(e.to_string()),
                ))
                .into()
            })
    }

    fn get_affirmation_count(
        &self,
        instruction_id: InstructionId,
        portfolios: Vec<PortfolioId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<AffirmationCount> {
        let api = self.client.runtime_api();
        // If the block hash is not supplied assume the best block.
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_affirmation_count(at_hash, instruction_id, portfolios)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to call get_affirmation_count runtime",
                    Some(e.to_string()),
                ))
                .into()
            })
    }
}
