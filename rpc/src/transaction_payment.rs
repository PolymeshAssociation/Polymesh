// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! RPC interface for the transaction payment module.

use std::{convert::TryInto, sync::Arc};

use super::Error;
use codec::Decode;
use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorCode, ErrorObject},
};
pub use node_rpc_runtime_api::transaction_payment::{
    FeeDetails, InclusionFee, RuntimeDispatchInfo,
    TransactionPaymentApi as TransactionPaymentRuntimeApi,
};
use polymesh_primitives::Balance;
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_rpc::number::NumberOrHex;
use sp_runtime::traits::Block as BlockT;

#[rpc(client, server)]
pub trait TransactionPaymentApi<BlockHash, ResponseType> {
    #[method(name = "payment_queryInfo")]
    fn query_info(&self, encoded_xt: Bytes, at: Option<BlockHash>) -> RpcResult<ResponseType>;

    #[method(name = "payment_queryFeeDetails")]
    fn query_fee_details(
        &self,
        encoded_xt: Bytes,
        at: Option<BlockHash>,
    ) -> RpcResult<FeeDetails<NumberOrHex>>;
}

/// Provides RPC methods to query a dispatchable's class, weight and fee.
pub struct TransactionPayment<C, P> {
    /// Shared reference to the client.
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> TransactionPayment<C, P> {
    /// Creates a new instance of the TransactionPayment Rpc helper.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block>
    TransactionPaymentApiServer<
        <Block as BlockT>::Hash,
        RuntimeDispatchInfo<Balance, sp_weights::OldWeight>,
    > for TransactionPayment<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: TransactionPaymentRuntimeApi<Block>,
{
    fn query_info(
        &self,
        encoded_xt: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<RuntimeDispatchInfo<Balance, sp_weights::OldWeight>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| {
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash
        });

        let encoded_len = encoded_xt.len() as u32;

        let uxt: Block::Extrinsic = Decode::decode(&mut &*encoded_xt).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::DecodeError.into(),
                "Unable to query dispatch info.",
                Some(format!("{:?}", e)),
            ))
        })?;

        fn map_err(error: impl ToString, desc: &'static str) -> CallError {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                desc,
                Some(error.to_string()),
            ))
        }

        let api_version = api
            .api_version::<dyn TransactionPaymentRuntimeApi<Block>>(at_hash)
            .map_err(|e| map_err(e, "Failed to get transaction payment runtime api version"))?
            .ok_or_else(|| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Transaction payment runtime api wasn't found in the runtime",
                    None::<String>,
                ))
            })?;

        if api_version < 2 {
            #[allow(deprecated)]
            api.query_info_before_version_2(at_hash, uxt, encoded_len)
                .map_err(|e| map_err(e, "Unable to query dispatch info.").into())
        } else {
            let res = api
                .query_info(at_hash, uxt, encoded_len)
                .map_err(|e| map_err(e, "Unable to query dispatch info."))?;

            Ok(RuntimeDispatchInfo {
                weight: sp_weights::OldWeight(res.weight.ref_time()),
                class: res.class,
                partial_fee: res.partial_fee,
            })
        }
    }

    fn query_fee_details(
        &self,
        encoded_xt: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<FeeDetails<NumberOrHex>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| {
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash
        });
        let encoded_len = encoded_xt.len() as u32;

        let uxt: Block::Extrinsic = Decode::decode(&mut &*encoded_xt).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::DecodeError.into(),
                "Unable to query fee details.",
                Some(format!("{:?}", e)),
            ))
        })?;
        let fee_details = api
            .query_fee_details(at_hash, uxt, encoded_len)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to query fee details.",
                    Some(e.to_string()),
                ))
            })?;

        let try_into_rpc_balance = |value: Balance| {
            value.try_into().map_err(|_| {
                JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::InvalidParams.code(),
                    format!("{} doesn't fit in NumberOrHex representation", value),
                    None::<()>,
                )))
            })
        };

        Ok(FeeDetails {
            inclusion_fee: if let Some(inclusion_fee) = fee_details.inclusion_fee {
                Some(InclusionFee {
                    base_fee: try_into_rpc_balance(inclusion_fee.base_fee)?,
                    len_fee: try_into_rpc_balance(inclusion_fee.len_fee)?,
                    adjusted_weight_fee: try_into_rpc_balance(inclusion_fee.adjusted_weight_fee)?,
                })
            } else {
                None
            },
            tip: Default::default(),
        })
    }
}
