use core::marker::PhantomData;

use codec::{Decode, Encode};

#[cfg(not(feature = "impl_protocol"))]
use polymesh_dart::Error as DartError;
use polymesh_dart::{
    ACCOUNT_TREE_L, ACCOUNT_TREE_M, ASSET_TREE_L, ASSET_TREE_M, ChildIndex, CompressedAffine,
    FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M,
    curve_tree::{AccountTreeConfig, AssetTreeConfig, FeeAccountTreeConfig},
    curve_tree::{
        CompressedChildCommitments, CompressedInner, CompressedXCoords, CurveTreeConfig,
        CurveTreeUpdater, DefaultCurveTreeUpdater,
    },
};
use polymesh_worker_common::ProtocolError;

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

impl<const L: usize, const M: usize, C: CurveTreeConfig> UpdateTreeNodeRequest<L, M, C> {
    pub fn do_update(mut self) -> Result<UpdateTreeNodeResult<M>, ProtocolError> {
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

#[cfg(feature = "impl_protocol")]
pub type HostCurveTreeUpdater<const L: usize, const M: usize, C> = DefaultCurveTreeUpdater<L, M, C>;

/// Curve tree updater that helps updating the tree root when a leaf is added or updated.
#[derive(Clone, Encode, Decode)]
#[cfg(not(feature = "impl_protocol"))]
pub struct HostCurveTreeUpdater<const L: usize, const M: usize, C: CurveTreeConfig> {
    _marker: PhantomData<C>,
}

#[cfg(not(feature = "impl_protocol"))]
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

        let req =
            crate::DartWorkRequest::UpdateCurveTree(CurveTreeUpdateRequest::AssetTreeNode(req));
        let res = match req.execute().map_err(|_| DartError::CurveTreeUpdateError)? {
            crate::DartWorkResponse::UpdateCurveTree(CurveTreeUpdateResponse::AssetTreeNode(
                res,
            )) => res,
            _ => return Err(DartError::CurveTreeUpdateError),
        };
        inner.commitments = res.commitments;
        Ok(res.x_coords)
    }
}

#[cfg(not(feature = "impl_protocol"))]
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

        let req =
            crate::DartWorkRequest::UpdateCurveTree(CurveTreeUpdateRequest::AccountTreeNode(req));
        let res = match req.execute().map_err(|_| DartError::CurveTreeUpdateError)? {
            crate::DartWorkResponse::UpdateCurveTree(CurveTreeUpdateResponse::AccountTreeNode(
                res,
            )) => res,
            _ => return Err(DartError::CurveTreeUpdateError),
        };
        inner.commitments = res.commitments;
        Ok(res.x_coords)
    }
}

#[cfg(not(feature = "impl_protocol"))]
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

        let req = crate::DartWorkRequest::UpdateCurveTree(
            CurveTreeUpdateRequest::FeeAccountTreeNode(req),
        );
        let res = match req.execute().map_err(|_| DartError::CurveTreeUpdateError)? {
            crate::DartWorkResponse::UpdateCurveTree(
                CurveTreeUpdateResponse::FeeAccountTreeNode(res),
            ) => res,
            _ => return Err(DartError::CurveTreeUpdateError),
        };
        inner.commitments = res.commitments;
        Ok(res.x_coords)
    }
}

#[derive(Encode, Decode, Clone)]
pub enum CurveTreeUpdateRequest {
    AssetTreeNode(UpdateTreeNodeRequest<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>),
    AccountTreeNode(UpdateTreeNodeRequest<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>),
    FeeAccountTreeNode(
        UpdateTreeNodeRequest<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>,
    ),
}

impl CurveTreeUpdateRequest {
    pub fn do_update(self) -> Result<CurveTreeUpdateResponse, ProtocolError> {
        match self {
            Self::AssetTreeNode(req) => req.do_update().map(CurveTreeUpdateResponse::AssetTreeNode),
            Self::AccountTreeNode(req) => req
                .do_update()
                .map(CurveTreeUpdateResponse::AccountTreeNode),
            Self::FeeAccountTreeNode(req) => req
                .do_update()
                .map(CurveTreeUpdateResponse::FeeAccountTreeNode),
        }
    }
}

#[derive(Encode, Decode, Clone)]
pub enum CurveTreeUpdateResponse {
    AssetTreeNode(UpdateTreeNodeResult<ASSET_TREE_M>),
    AccountTreeNode(UpdateTreeNodeResult<ACCOUNT_TREE_M>),
    FeeAccountTreeNode(UpdateTreeNodeResult<FEE_ACCOUNT_TREE_M>),
}
