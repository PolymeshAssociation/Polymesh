use core::marker::PhantomData;

use codec::{Decode, Encode};

#[cfg(feature = "std")]
use polymesh_dart::curve_tree::DefaultCurveTreeUpdater;
#[cfg(not(feature = "std"))]
use polymesh_dart::{
    ACCOUNT_TREE_L, ACCOUNT_TREE_M, ASSET_TREE_L, ASSET_TREE_M, Error as DartError,
    FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M,
    curve_tree::{AccountTreeConfig, AssetTreeConfig, FeeAccountTreeConfig},
};
use polymesh_dart::{
    ChildIndex, CompressedAffine,
    curve_tree::{
        CompressedChildCommitments, CompressedInner, CompressedXCoords, CurveTreeConfig,
        CurveTreeUpdater,
    },
};

use crate::Error;

#[derive(Clone, Encode, Decode)]
pub struct UpdateTreeNodeRequest<const L: usize, const M: usize, C: CurveTreeConfig> {
    inner: CompressedInner<M, C>,
    child_index: ChildIndex,
    old_child: Option<CompressedChildCommitments<M>>,
    new_child: CompressedChildCommitments<M>,
    #[codec(skip)]
    _marker: PhantomData<C>,
}

#[cfg(feature = "std")]
impl<const L: usize, const M: usize, C: CurveTreeConfig> UpdateTreeNodeRequest<L, M, C> {
    pub fn update(mut self) -> Result<UpdateTreeNodeResult<M>, Error> {
        let x_coords = DefaultCurveTreeUpdater::<L, M, C>::update_node(
            &mut self.inner,
            self.child_index,
            self.old_child,
            self.new_child,
        )
        .map_err(|_| Error::CurveTreeUpdateError)?;

        Ok(UpdateTreeNodeResult {
            commitments: self.inner.commitments,
            x_coords,
        })
    }
}

#[cfg(not(feature = "std"))]
impl UpdateTreeNodeRequest<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig> {
    pub fn update(self) -> Result<UpdateTreeNodeResult<ASSET_TREE_M>, Error> {
        crate::native_dart_assets::asset_tree_update_inner_node(self)
    }
}

#[cfg(not(feature = "std"))]
impl UpdateTreeNodeRequest<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig> {
    pub fn update(self) -> Result<UpdateTreeNodeResult<ACCOUNT_TREE_M>, Error> {
        crate::native_dart_assets::account_tree_update_inner_node(self)
    }
}

#[cfg(not(feature = "std"))]
impl UpdateTreeNodeRequest<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig> {
    pub fn update(self) -> Result<UpdateTreeNodeResult<FEE_ACCOUNT_TREE_M>, Error> {
        crate::native_dart_assets::fee_account_tree_update_inner_node(self)
    }
}

#[derive(Clone, Encode, Decode)]
pub struct UpdateTreeNodeResult<const M: usize> {
    commitments: [CompressedAffine; M],
    x_coords: CompressedXCoords<M>,
}

impl<const M: usize> Default for UpdateTreeNodeResult<M> {
    fn default() -> Self {
        Self {
            commitments: [Default::default(); M],
            x_coords: Default::default(),
        }
    }
}

#[cfg(feature = "std")]
pub type HostCurveTreeUpdater<const L: usize, const M: usize, C> = DefaultCurveTreeUpdater<L, M, C>;

/// Curve tree updater that helps updating the tree root when a leaf is added or updated.
#[derive(Clone, Encode, Decode)]
#[cfg(not(feature = "std"))]
pub struct HostCurveTreeUpdater<const L: usize, const M: usize, C: CurveTreeConfig> {
    _marker: PhantomData<C>,
}

#[cfg(not(feature = "std"))]
impl CurveTreeUpdater<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>
    for HostCurveTreeUpdater<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>
{
    fn update_node(
        inner: &mut CompressedInner<ASSET_TREE_M, AssetTreeConfig>,
        child_index: ChildIndex,
        old_child: Option<CompressedChildCommitments<ASSET_TREE_M>>,
        new_child: CompressedChildCommitments<ASSET_TREE_M>,
    ) -> Result<CompressedXCoords<ASSET_TREE_M>, DartError> {
        let req = UpdateTreeNodeRequest::<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig> {
            inner: inner.clone(),
            child_index,
            old_child,
            new_child,
            _marker: PhantomData,
        };

        let res = req.update().map_err(|_| DartError::CurveTreeUpdateError)?;
        inner.commitments = res.commitments;
        Ok(res.x_coords)
    }
}

#[cfg(not(feature = "std"))]
impl CurveTreeUpdater<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>
    for HostCurveTreeUpdater<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>
{
    fn update_node(
        inner: &mut CompressedInner<ACCOUNT_TREE_M, AccountTreeConfig>,
        child_index: ChildIndex,
        old_child: Option<CompressedChildCommitments<ACCOUNT_TREE_M>>,
        new_child: CompressedChildCommitments<ACCOUNT_TREE_M>,
    ) -> Result<CompressedXCoords<ACCOUNT_TREE_M>, DartError> {
        let req = UpdateTreeNodeRequest::<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig> {
            inner: inner.clone(),
            child_index,
            old_child,
            new_child,
            _marker: PhantomData,
        };

        let res = req.update().map_err(|_| DartError::CurveTreeUpdateError)?;
        inner.commitments = res.commitments;
        Ok(res.x_coords)
    }
}

#[cfg(not(feature = "std"))]
impl CurveTreeUpdater<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>
    for HostCurveTreeUpdater<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>
{
    fn update_node(
        inner: &mut CompressedInner<FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>,
        child_index: ChildIndex,
        old_child: Option<CompressedChildCommitments<FEE_ACCOUNT_TREE_M>>,
        new_child: CompressedChildCommitments<FEE_ACCOUNT_TREE_M>,
    ) -> Result<CompressedXCoords<FEE_ACCOUNT_TREE_M>, DartError> {
        let req =
            UpdateTreeNodeRequest::<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig> {
                inner: inner.clone(),
                child_index,
                old_child,
                new_child,
                _marker: PhantomData,
            };

        let res = req.update().map_err(|_| DartError::CurveTreeUpdateError)?;
        inner.commitments = res.commitments;
        Ok(res.x_coords)
    }
}
