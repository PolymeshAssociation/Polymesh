use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use polymesh_primitives::{IdentityId, Ticker};
use polymesh_runtime_identity_rpc_runtime_api::{
    self as runtime_api, AssetDidResult, CddStatus, IdentityApi as IdentityRuntimeApi,
};
use serde::{Deserialize, Serialize};
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, ProvideRuntimeApi},
};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum RpcCddStatus<IdentityId> {
    Success {
        /// Is cdd expired or not
        status: bool,
        /// Cdd claim provider
        cdd_claim_provider: IdentityId,
    },
    Error(Vec<u8>),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum RpcAssetDidResult<IdentityId> {
    Success {
        /// asset DID
        asset_did: IdentityId,
    },
    Error(Vec<u8>),
}

/// Conversion 
impl From<CddStatus<IdentityId>> for RpcCddStatus<IdentityId> {
    fn from(r: CddStatus<IdentityId>) -> Self {
        match r {
            CddStatus::Success {
                status,
                cdd_claim_provider,
            } => RpcCddStatus::Success {
                status,
                cdd_claim_provider,
            },
            CddStatus::Error => RpcCddStatus::Error(
                "Either cdd claim is expired or not yet provided to give identity".into(),
            ),
        }
    }
}

impl From<AssetDidResult<IdentityId>> for RpcAssetDidResult<IdentityId> {
    fn from(r: AssetDidResult<IdentityId>) -> Self {
        match r {
            AssetDidResult::Success { asset_did } => RpcAssetDidResult::Success { asset_did },
            AssetDidResult::Error => {
                RpcAssetDidResult::Error("Error in computing the given ticker error".into())
            }
        }
    }
}

/// Identity RPC methods
#[rpc]
pub trait IdentityApi<BlockHash, IdentityId, Ticker> {
    /// Below function use to tell whether the given did has valid cdd claim or not
    #[rpc(name = "identity_isIdentityHasValidCdd")]
    fn is_identity_has_valid_cdd(
        &self,
        did: IdentityId,
        buffer_time: Option<u64>,
        at: Option<BlockHash>,
    ) -> Result<RpcCddStatus<IdentityId>>;

    /// Below function is used to query the given ticker DID.
    #[rpc(name = "identity_getAssetDid")]
    fn get_asset_did(
        &self,
        ticker: Ticker,
        at: Option<BlockHash>,
    ) -> Result<RpcAssetDidResult<IdentityId>>;
}

/// A struct that implements the [`IdentityApi`].
pub struct Identity<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Identity<C, M> {
    /// Create new `Identity` instance with the given reference to the client.
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

impl<C, Block, IdentityId, Ticker> IdentityApi<<Block as BlockT>::Hash, IdentityId, Ticker>
    for Identity<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi,
    C: HeaderBackend<Block>,
    C::Api: IdentityRuntimeApi<Block, IdentityId, Ticker>,
    IdentityId: Codec,
    Ticker: Codec,
{
    fn is_identity_has_valid_cdd(
        &self,
        did: IdentityId,
        buffer_time: Option<u64>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RpcCddStatus<IdentityId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));
        let result = api
            .is_identity_has_valid_cdd(&at, did, buffer_time)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError.into()),
                message: "Either cdd claim not exist or Identity.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;
        Ok(result.into())
    }

    fn get_asset_did(
        &self,
        ticker: Ticker,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RpcAssetDidResult<IdentityId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));
        let result = api.get_asset_did(&at, ticker).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to fetch did of the given ticker".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        Ok(result.into())
    }
}
