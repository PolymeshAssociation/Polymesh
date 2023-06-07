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

pub use node_rpc_runtime_api::compliance_manager::ComplianceManagerApi as ComplianceManagerRuntimeApi;

use std::sync::Arc;

use crate::Error;
use codec::Codec;
use frame_support::traits::Currency;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use polymesh_primitives::{compliance_manager::AssetComplianceResult, IdentityId, Ticker};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

pub trait Trait: frame_system::Config {
    type Currency: Currency<Self::AccountId>;
}

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[rpc(client, server)]
pub trait ComplianceManagerApi<BlockHash, AccountId> {
    #[method(name = "compliance_canTransfer")]
    fn can_transfer(
        &self,
        ticker: Ticker,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        at: Option<BlockHash>,
    ) -> RpcResult<AssetComplianceResult>;
}

/// An implementation of Compliance manager specific RPC methods.
pub struct ComplianceManager<T, U> {
    client: Arc<T>,
    _marker: std::marker::PhantomData<U>,
}

impl<T, U> ComplianceManager<T, U> {
    /// Create new `ComplianceManager` with the given reference to the client.
    pub fn new(client: Arc<T>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> ComplianceManagerApiServer<<Block as BlockT>::Hash, AccountId>
    for ComplianceManager<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: ComplianceManagerRuntimeApi<Block, AccountId>,
    AccountId: Codec,
{
    fn can_transfer(
        &self,
        ticker: Ticker,
        from_did: Option<IdentityId>,
        to_did: Option<IdentityId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<AssetComplianceResult> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(||
                // If the block hash is not supplied assume the best block.
                self.client.info().best_hash);

        api.can_transfer(at_hash, ticker, from_did, to_did)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to fetch transfer status from compliance manager.",
                    Some(e.to_string()),
                ))
                .into()
            })
    }
}
