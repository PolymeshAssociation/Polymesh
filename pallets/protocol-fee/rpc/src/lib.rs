use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi as ProtocolFeeRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait ProtocolFeeApi<BlockHash, Balance, ProtocolOp> {
    #[rpc(name = "protocolFee_getFee")]
    fn get_fee(&self, op: ProtocolOp, at: Option<BlockHash>) -> Result<Balance>;
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

/// Error type of this RPC API.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError = 1,
    /// The call to runtime failed.
    RuntimeError = 2,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        e as i64
    }
}

impl<C, Block, Balance, ProtocolOp> ProtocolFeeApi<<Block as BlockT>::Hash>
    for ProtocolFee<C, Block, Balance, ProtocolOp>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: ProtocolFeeRuntimeApi<Block, Balance>,
    Balance: Codec,
    ProtocolOp: Codec,
{
    fn get_fee(&self, op: ProtocolOp, at: Option<<Block as BlockT>::Hash>) -> Result<Balance> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        api.get_fee(&at, op).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to query dispatch info.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
