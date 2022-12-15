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

pub use self::gen_client::Client as TransactionPaymentClient;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
pub use node_rpc_runtime_api::transaction_payment::{
    Encoded, FeeDetails, InclusionFee, RuntimeDispatchInfo,
    TransactionPaymentApi as TransactionPaymentRuntimeApi,
};
use polymesh_primitives::Balance;
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait TransactionPaymentApi<BlockHash, ResponseType> {
    #[rpc(name = "payment_queryInfo")]
    fn query_info(&self, encoded_xt: Bytes, at: Option<BlockHash>) -> Result<ResponseType>;
    #[rpc(name = "payment_queryFeeDetails")]
    fn query_fee_details(
        &self,
        encoded_xt: Bytes,
        at: Option<BlockHash>,
    ) -> Result<FeeDetails<NumberOrHex>>;
}

/// A struct that implements the [`TransactionPaymentApi`].
pub struct TransactionPayment<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> TransactionPayment<C, P> {
    /// Create new `TransactionPayment` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        TransactionPayment {
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

impl<C, Block, Extrinsic>
    TransactionPaymentApi<<Block as BlockT>::Hash, RuntimeDispatchInfo<Balance>>
    for TransactionPayment<C, (Block, Extrinsic)>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: TransactionPaymentRuntimeApi<Block, Extrinsic>,
    Extrinsic: Codec + Send + Sync + 'static,
{
    fn query_info(
        &self,
        encoded_xt: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RuntimeDispatchInfo<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| {
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash
        }));
        let api_version = api
            .api_version::<dyn TransactionPaymentRuntimeApi<Block, Extrinsic>>(&at)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError as i64),
                message: "Unable to query dispatch info.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;

        match api_version {
            Some(version) if version >= 2 => api
                .query_info(&at, encoded_xt.0)
                .map_err(|e| RpcError {
                    code: ErrorCode::ServerError(Error::RuntimeError.into()),
                    message: "Unable to query dispatch info.".into(),
                    data: Some(format!("{:?}", e).into()),
                })?
                .ok_or_else(|| RpcError {
                    code: ErrorCode::ServerError(Error::DecodeError.into()),
                    message: "Unable to query dispatch info.".into(),
                    data: None,
                }),
            Some(1) => {
                let encoded_len = encoded_xt.len() as u32;

                // Pass the raw encoded bytes to the runtime, without decoding them here.
                let uxt = Encoded(encoded_xt.0);
                #[allow(deprecated)]
                api.query_info_before_version_2(&at, uxt, encoded_len)
                    .map_err(|e| RpcError {
                        code: ErrorCode::ServerError(Error::RuntimeError.into()),
                        message: "Unable to query dispatch info.".into(),
                        data: Some(format!("{:?}", e).into()),
                    })
            }
            _ => {
                return Err(RpcError {
                    code: ErrorCode::MethodNotFound,
                    message: format!("Cannot find `TransactionPaymentApi` for block {:?}", at),
                    data: None,
                });
            }
        }
    }

    fn query_fee_details(
        &self,
        encoded_xt: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<FeeDetails<NumberOrHex>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| {
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash
        }));
        // The `query_fee_details` method was only added in v2.
        let has_v2 = api
            .has_api::<dyn TransactionPaymentRuntimeApi<Block, Extrinsic>>(&at)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError as i64),
                message: "Unable to query fee details.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;
        if !has_v2 {
            return Err(RpcError {
                code: ErrorCode::MethodNotFound,
                message: format!(
                    "Cannot find `TransactionPaymentApi::query_fee_details` for block {:?}",
                    at
                ),
                data: None,
            });
        }

        let fee_details = api
            .query_fee_details(&at, encoded_xt.0)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError.into()),
                message: "Unable to query fee details.".into(),
                data: Some(e.to_string().into()),
            })?
            .ok_or_else(|| RpcError {
                code: ErrorCode::ServerError(Error::DecodeError.into()),
                message: "Unable to query dispatch info.".into(),
                data: None,
            })?;

        let try_into_rpc_balance = |value: Balance| {
            value.try_into().map_err(|_| RpcError {
                code: ErrorCode::InvalidParams,
                message: format!("{} doesn't fit in NumberOrHex representation", value),
                data: None,
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
