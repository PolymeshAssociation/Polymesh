use codec::{Decode, Encode};

use polymesh_dart::{
    AssetState, CompressedAffine,
    curve_tree::{AssetTreeConfig, CompressedLeafValue},
};
use polymesh_worker_common::ProtocolError;

use crate::Error;

#[derive(Clone, Encode, Decode)]
pub struct UpdateAssetStateRequest {
    asset: AssetState,
}

impl UpdateAssetStateRequest {
    pub fn new(asset: AssetState) -> Self {
        Self { asset }
    }
}

impl UpdateAssetStateRequest {
    pub fn do_update(self) -> Result<UpdateAssetStateResult, ProtocolError> {
        let asset_leaf = self
            .asset
            .commitment()
            .map_err(|_| Error::AssetStateError)?;
        Ok(UpdateAssetStateResult {
            commitment: asset_leaf.into(),
        })
    }
}

#[cfg(feature = "impl_protocol")]
impl UpdateAssetStateRequest {
    pub fn update(self) -> Result<UpdateAssetStateResult, ProtocolError> {
        self.do_update()
    }
}

#[cfg(not(feature = "impl_protocol"))]
impl UpdateAssetStateRequest {
    pub fn update(self) -> Result<UpdateAssetStateResult, ProtocolError> {
        let req = crate::DartWorkRequest::UpdateAssetState(self);
        match req.execute()? {
            crate::DartWorkResponse::UpdateAssetState(res) => Ok(res),
            _ => Err(ProtocolError::UnexpectedResponse),
        }
    }
}

#[derive(Clone, Encode, Decode, Default)]
pub struct UpdateAssetStateResult {
    commitment: CompressedAffine,
}

impl UpdateAssetStateResult {
    pub fn asset_leaf(&self) -> CompressedLeafValue<AssetTreeConfig> {
        self.commitment.into()
    }
}
