use codec::{Decode, Encode};

use polymesh_dart::{
    AssetState, CompressedAffine, PolymeshPrivateLimits,
    curve_tree::{AssetTreeConfig, CompressedLeafValue},
};

use crate::Error;

#[derive(Clone, Encode, Decode)]
pub struct UpdateAssetStateRequest {
    asset: AssetState<PolymeshPrivateLimits>,
}

impl UpdateAssetStateRequest {
    pub fn new(asset: AssetState<PolymeshPrivateLimits>) -> Self {
        Self { asset }
    }
}

#[cfg(feature = "std")]
impl UpdateAssetStateRequest {
    pub fn update(self) -> Result<UpdateAssetStateResult, Error> {
        let asset_leaf = self
            .asset
            .commitment()
            .map_err(|_| Error::AssetStateError)?;
        Ok(UpdateAssetStateResult {
            commitment: asset_leaf.into(),
        })
    }
}

#[cfg(not(feature = "std"))]
impl UpdateAssetStateRequest {
    pub fn update(self) -> Result<UpdateAssetStateResult, Error> {
        crate::native_dart_assets::update_asset_state(self)
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
