pub use pallet_identity::types::{
    AssetDidResult, CddStatus, DidStatus, KeyIdentityData, RpcDidRecords,
};
use polymesh_primitives::{Authorization, AuthorizationType, Signatory};

pub use node_rpc_runtime_api::identity::IdentityApi as IdentityRuntimeApi;

use std::{convert::TryInto, sync::Arc};

use super::Error;
use codec::Codec;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use sp_api::{ApiExt, ApiRef, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{Block as BlockT, Zero};

const MAX_IDENTITIES_ALLOWED_TO_QUERY: u32 = 500;

/// Identity RPC methods
#[rpc(client, server)]
pub trait IdentityApi<BlockHash, IdentityId, Ticker, AccountId, Moment> {
    /// Below function use to tell whether the given did has valid cdd claim or not
    #[method(name = "identity_isIdentityHasValidCdd")]
    fn is_identity_has_valid_cdd(
        &self,
        did: IdentityId,
        buffer_time: Option<u64>,
        at: Option<BlockHash>,
    ) -> RpcResult<CddStatus>;

    /// Below function is used to query the given ticker DID.
    #[method(name = "identity_getAssetDid")]
    fn get_asset_did(&self, ticker: Ticker, at: Option<BlockHash>) -> RpcResult<AssetDidResult>;

    /// DidRecords for a `did`
    #[method(name = "identity_getDidRecords")]
    fn get_did_records(
        &self,
        did: IdentityId,
        at: Option<BlockHash>,
    ) -> RpcResult<RpcDidRecords<AccountId>>;

    /// Retrieve the list of authorizations for a given signatory.
    #[method(name = "identity_getFilteredAuthorizations")]
    fn get_filtered_authorizations(
        &self,
        signatory: Signatory<AccountId>,
        allow_expired: bool,
        auth_type: Option<AuthorizationType>,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<Authorization<AccountId, Moment>>>;

    /// Provide the status of a given DID
    #[method(name = "identity_getDidStatus")]
    fn get_did_status(
        &self,
        dids: Vec<IdentityId>,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<DidStatus>>;

    /// Provide the `KeyIdentityData` from a given `AccountId`, including:
    /// - the corresponding DID,
    /// - whether the `AccountId` is a primary or secondary key,
    /// - any permissions related to the key.
    ///
    /// This is an aggregate call provided for UX convenience.
    #[method(name = "identity_getKeyIdentityData")]
    fn get_key_identity_data(
        &self,
        acc: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Option<KeyIdentityData<IdentityId>>>;
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

impl<C, Block, IdentityId, Ticker, AccountId, Moment>
    IdentityApiServer<<Block as BlockT>::Hash, IdentityId, Ticker, AccountId, Moment>
    for Identity<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: IdentityRuntimeApi<Block, IdentityId, Ticker, AccountId, Moment>,
    IdentityId: Codec,
    Ticker: Codec,
    AccountId: Codec,
    Moment: Codec,
{
    fn is_identity_has_valid_cdd(
        &self,
        did: IdentityId,
        buffer_time: Option<u64>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<CddStatus> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash);
        api.is_identity_has_valid_cdd(at_hash, did, buffer_time)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Either cdd claim not exist or Identity.",
                    Some(e.to_string()),
                ))
                .into()
            })
    }

    fn get_asset_did(
        &self,
        ticker: Ticker,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<AssetDidResult> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash);
        api.get_asset_did(at_hash, ticker).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to fetch did of the given ticker",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_did_records(
        &self,
        did: IdentityId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<RpcDidRecords<AccountId>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);
        let api_version = api
            .api_version::<dyn IdentityRuntimeApi<Block, IdentityId, Ticker, AccountId, Moment>>(
                at_hash,
            )
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to fetch DID records",
                    Some(e.to_string()),
                ))
            })?;

        match api_version {
            Some(version) if version >= 2 =>
            {
                #[allow(deprecated)]
                api.get_did_records(at_hash, did)
            }
            Some(1) =>
            {
                #[allow(deprecated)]
                api.get_did_records_before_version_2(at_hash, did)
                    .map(|rec| rec.into())
            }
            _ => {
                return Err(CallError::Custom(ErrorObject::owned(
                    ErrorCode::MethodNotFound.code(),
                    format!("Cannot find `IdentityApi` for block {:?}", at),
                    None::<()>,
                ))
                .into());
            }
        }
        .map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to fetch DID records",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_filtered_authorizations(
        &self,
        signatory: Signatory<AccountId>,
        allow_expired: bool,
        auth_type: Option<AuthorizationType>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<Authorization<AccountId, Moment>>> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_filtered_authorizations(at_hash, signatory, allow_expired, auth_type)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to fetch authorizations data",
                    Some(e.to_string()),
                ))
                .into()
            })
    }

    fn get_did_status(
        &self,
        dids: Vec<IdentityId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<DidStatus>> {
        if dids.len()
            > MAX_IDENTITIES_ALLOWED_TO_QUERY
                .try_into()
                .unwrap_or_else(|_| Zero::zero())
        {
            return Err(CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to fetch dids status",
                Some(format!(
                    "Provided vector length is more than the maximum allowed length i.e {:?}",
                    MAX_IDENTITIES_ALLOWED_TO_QUERY
                )),
            ))
            .into());
        }
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_did_status(at_hash, dids).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to fetch dids status",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_key_identity_data(
        &self,
        acc: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Option<KeyIdentityData<IdentityId>>> {
        rpc_forward_call!(
            self,
            at,
            |api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| {
                api.get_key_identity_data(at, acc)
            },
            "Unable to query `get_key_identity_data`."
        )
    }
}
