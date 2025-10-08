use core::ops::{Deref, DerefMut};

use anyhow::{Error, Result};

use polymesh_dart::{
    curve_tree::{
        AccountTreeConfig, AssetTreeConfig, AsyncCurveTreeBackend, AsyncCurveTreeWithBackend,
        CompressedCurveTreeRoot, CompressedInner, CompressedLeafValue, CurveTreeConfig as _,
        CurveTreeParameters, DefaultCurveTreeUpdater, FeeAccountTreeConfig, NodeLocation,
        NodePosition,
    },
    BlockNumber, LeafIndex, NodeLevel, ACCOUNT_TREE_HEIGHT, ACCOUNT_TREE_L, ACCOUNT_TREE_M,
    ASSET_TREE_HEIGHT, ASSET_TREE_L, ASSET_TREE_M, FEE_ACCOUNT_TREE_HEIGHT, FEE_ACCOUNT_TREE_L,
    FEE_ACCOUNT_TREE_M,
};

pub use polymesh_api::types::polymesh_dart::curve_tree::common::{
    NodeLocation as ChainNodeLocation, NodePosition as ChainNodePosition,
};
use polymesh_api::ChainApi as _;
pub use polymesh_api::{Api, TransactionResults};

use super::*;

pub type AssetLeaf = CompressedLeafValue<AssetTreeConfig>;
pub type AssetInnerNode = CompressedInner<ASSET_TREE_M, AssetTreeConfig>;
pub type AssetNodeLocation = NodeLocation<ASSET_TREE_L>;
pub type AssetTreeRoot = CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>;

pub type AccountLeaf = CompressedLeafValue<AccountTreeConfig>;
pub type AccountInnerNode = CompressedInner<ACCOUNT_TREE_M, AccountTreeConfig>;
pub type AccountNodeLocation = NodeLocation<ACCOUNT_TREE_L>;
pub type AccountTreeRoot =
    CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>;

pub type FeeAccountLeaf = CompressedLeafValue<FeeAccountTreeConfig>;
pub type FeeAccountInnerNode = CompressedInner<FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>;
pub type FeeAccountNodeLocation = NodeLocation<FEE_ACCOUNT_TREE_L>;
pub type FeeAccountTreeRoot =
    CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>;

/// Convert off-chain `NodeLocation` to on-chain `NodeLocation`.
pub fn node_location_to_chain<const L: usize>(location: NodeLocation<L>) -> ChainNodeLocation {
    match location {
        NodeLocation::Leaf(leaf_index) => ChainNodeLocation::Leaf(leaf_index),
        NodeLocation::Odd(NodePosition { level, index }) => {
            ChainNodeLocation::Odd(ChainNodePosition { level, index })
        }
        NodeLocation::Even(NodePosition { level, index }) => {
            ChainNodeLocation::Even(ChainNodePosition { level, index })
        }
    }
}

/// Asset Curve Tree Storage backend.
#[derive(Clone)]
pub struct AssetCurveTreeChainStorage {
    api: Api,
}

impl AssetCurveTreeChainStorage {
    fn new(api: &Api) -> Self {
        Self { api: api.clone() }
    }
}

impl AsyncCurveTreeBackend<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>
    for AssetCurveTreeChainStorage
{
    type Error = anyhow::Error;
    type Updater = DefaultCurveTreeUpdater<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>;

    async fn new(_height: NodeLevel) -> Result<Self, Self::Error> {
        Err(anyhow::anyhow!(
            "AssetCurveTreeChainStorage does not support new()"
        ))
    }

    fn parameters(&self) -> &CurveTreeParameters<AssetTreeConfig> {
        AssetTreeConfig::parameters()
    }

    async fn get_block_number(&self) -> Result<BlockNumber, Self::Error> {
        let last_block_number = self
            .api
            .query()
            .confidential_assets()
            .asset_curve_tree_last_update()
            .await?;
        Ok(last_block_number)
    }

    async fn fetch_root(&self, block_number: Option<BlockNumber>) -> Result<AssetTreeRoot> {
        let root = if let Some(block_number) = block_number {
            self.api
                .query()
                .confidential_assets()
                .asset_curve_tree_roots(block_number)
                .await?
        } else {
            self.api
                .query()
                .confidential_assets()
                .asset_curve_tree_current_root()
                .await?
        };
        if let Some(root) = root {
            Ok(to_scale(&root))
        } else {
            Err(anyhow::anyhow!(
                "Root not found for block number {:?}",
                block_number
            ))
        }
    }

    async fn height(&self) -> NodeLevel {
        ASSET_TREE_HEIGHT
    }

    async fn allocate_leaf_index(&mut self) -> LeafIndex {
        Default::default()
    }

    async fn get_leaf(
        &self,
        leaf_index: LeafIndex,
        block_number: Option<BlockNumber>,
    ) -> Result<Option<AssetLeaf>, Error> {
        let block_hash = match block_number {
            Some(num) => self.api.client().get_block_hash(num).await?,
            None => None,
        };
        if let Some(block_hash) = block_hash {
            if let Some(leaf) = self
                .api
                .query_at(block_hash)
                .confidential_assets()
                .asset_leaves(leaf_index)
                .await?
            {
                return Ok(Some(to_scale(&leaf)));
            }
        } else {
            if let Some(leaf) = self
                .api
                .query()
                .confidential_assets()
                .asset_leaves(leaf_index)
                .await?
            {
                return Ok(Some(to_scale(&leaf)));
            }
        }
        Ok(None)
    }

    async fn leaf_count(&self) -> LeafIndex {
        0
    }

    async fn get_inner_node(
        &self,
        location: NodeLocation<ASSET_TREE_L>,
        block_number: Option<BlockNumber>,
    ) -> Result<Option<AssetInnerNode>, Error> {
        let block_hash = match block_number {
            Some(num) => self.api.client().get_block_hash(num).await?,
            None => None,
        };
        let location = node_location_to_chain(location);
        if let Some(block_hash) = block_hash {
            if let Some(node) = self
                .api
                .query_at(block_hash)
                .confidential_assets()
                .asset_inner_nodes(location)
                .await?
            {
                return Ok(Some(to_scale(&node)));
            }
        } else {
            if let Some(node) = self
                .api
                .query()
                .confidential_assets()
                .asset_inner_nodes(location)
                .await?
            {
                return Ok(Some(to_scale(&node)));
            }
        }
        Ok(None)
    }
}

pub type AssetCurveTreeType = AsyncCurveTreeWithBackend<
    ASSET_TREE_L,
    ASSET_TREE_M,
    AssetTreeConfig,
    AssetCurveTreeChainStorage,
    anyhow::Error,
>;

#[derive(Clone)]
pub struct AssetCurveTree(AssetCurveTreeType);

impl Deref for AssetCurveTree {
    type Target = AssetCurveTreeType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AssetCurveTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AssetCurveTree {
    pub async fn new(api: &Api) -> Result<Self> {
        let backend = AssetCurveTreeChainStorage::new(api);
        Ok(Self(
            AsyncCurveTreeWithBackend::new_with_backend(backend).await?,
        ))
    }
}

/// Account Curve Tree Storage backend.
#[derive(Clone)]
pub struct AccountCurveTreeChainStorage {
    api: Api,
}

impl AccountCurveTreeChainStorage {
    fn new(api: &Api) -> Self {
        Self { api: api.clone() }
    }
}

impl AsyncCurveTreeBackend<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>
    for AccountCurveTreeChainStorage
{
    type Error = anyhow::Error;
    type Updater = DefaultCurveTreeUpdater<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>;

    async fn new(_height: NodeLevel) -> Result<Self, Self::Error> {
        Err(anyhow::anyhow!(
            "AccountCurveTreeChainStorage does not support new()"
        ))
    }

    fn parameters(&self) -> &CurveTreeParameters<AccountTreeConfig> {
        AccountTreeConfig::parameters()
    }

    async fn get_block_number(&self) -> Result<BlockNumber, Self::Error> {
        let last_block_number = self
            .api
            .query()
            .confidential_assets()
            .account_curve_tree_last_update()
            .await?;
        Ok(last_block_number)
    }

    async fn fetch_root(&self, block_number: Option<BlockNumber>) -> Result<AccountTreeRoot> {
        let root = if let Some(block_number) = block_number {
            self.api
                .query()
                .confidential_assets()
                .account_curve_tree_roots(block_number)
                .await?
        } else {
            self.api
                .query()
                .confidential_assets()
                .account_curve_tree_current_root()
                .await?
        };
        if let Some(root) = root {
            Ok(to_scale(&root))
        } else {
            Err(anyhow::anyhow!(
                "Root not found for block number {:?}",
                block_number
            ))
        }
    }

    async fn height(&self) -> NodeLevel {
        ACCOUNT_TREE_HEIGHT
    }

    async fn allocate_leaf_index(&mut self) -> LeafIndex {
        Default::default()
    }

    async fn get_leaf(
        &self,
        leaf_index: LeafIndex,
        block_number: Option<BlockNumber>,
    ) -> Result<Option<AccountLeaf>, Error> {
        let block_hash = match block_number {
            Some(num) => self.api.client().get_block_hash(num).await?,
            None => None,
        };
        if let Some(block_hash) = block_hash {
            if let Some(leaf) = self
                .api
                .query_at(block_hash)
                .confidential_assets()
                .account_leaves(leaf_index)
                .await?
            {
                return Ok(Some(to_scale(&leaf)));
            }
        } else {
            if let Some(leaf) = self
                .api
                .query()
                .confidential_assets()
                .account_leaves(leaf_index)
                .await?
            {
                return Ok(Some(to_scale(&leaf)));
            }
        }
        Ok(None)
    }

    async fn leaf_count(&self) -> LeafIndex {
        0
    }

    async fn get_inner_node(
        &self,
        location: NodeLocation<ACCOUNT_TREE_L>,
        block_number: Option<BlockNumber>,
    ) -> Result<Option<AccountInnerNode>, Error> {
        let block_hash = match block_number {
            Some(num) => self.api.client().get_block_hash(num).await?,
            None => None,
        };
        let location = node_location_to_chain(location);
        if let Some(block_hash) = block_hash {
            if let Some(node) = self
                .api
                .query_at(block_hash)
                .confidential_assets()
                .account_inner_nodes(location)
                .await?
            {
                return Ok(Some(to_scale(&node)));
            }
        } else {
            if let Some(node) = self
                .api
                .query()
                .confidential_assets()
                .account_inner_nodes(location)
                .await?
            {
                return Ok(Some(to_scale(&node)));
            }
        }
        Ok(None)
    }
}

pub type AccountCurveTreeType = AsyncCurveTreeWithBackend<
    ACCOUNT_TREE_L,
    ACCOUNT_TREE_M,
    AccountTreeConfig,
    AccountCurveTreeChainStorage,
    anyhow::Error,
>;

#[derive(Clone)]
pub struct AccountCurveTree(AccountCurveTreeType);

impl Deref for AccountCurveTree {
    type Target = AccountCurveTreeType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AccountCurveTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AccountCurveTree {
    pub async fn new(api: &Api) -> Result<Self> {
        let backend = AccountCurveTreeChainStorage::new(api);
        Ok(Self(
            AsyncCurveTreeWithBackend::new_with_backend(backend).await?,
        ))
    }
}

/// Fee Account Curve Tree Storage backend.
#[derive(Clone)]
pub struct FeeAccountCurveTreeChainStorage {
    api: Api,
}

impl FeeAccountCurveTreeChainStorage {
    fn new(api: &Api) -> Self {
        Self { api: api.clone() }
    }
}

impl AsyncCurveTreeBackend<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>
    for FeeAccountCurveTreeChainStorage
{
    type Error = anyhow::Error;
    type Updater =
        DefaultCurveTreeUpdater<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>;

    async fn new(_height: NodeLevel) -> Result<Self, Self::Error> {
        Err(anyhow::anyhow!(
            "FeeAccountCurveTreeChainStorage does not support new()"
        ))
    }

    fn parameters(&self) -> &CurveTreeParameters<FeeAccountTreeConfig> {
        FeeAccountTreeConfig::parameters()
    }

    async fn get_block_number(&self) -> Result<BlockNumber, Self::Error> {
        let last_block_number = self
            .api
            .query()
            .confidential_assets()
            .fee_account_curve_tree_last_update()
            .await?;
        Ok(last_block_number)
    }

    async fn fetch_root(&self, block_number: Option<BlockNumber>) -> Result<FeeAccountTreeRoot> {
        let root = if let Some(block_number) = block_number {
            self.api
                .query()
                .confidential_assets()
                .fee_account_curve_tree_roots(block_number)
                .await?
        } else {
            self.api
                .query()
                .confidential_assets()
                .fee_account_curve_tree_current_root()
                .await?
        };
        if let Some(root) = root {
            Ok(to_scale(&root))
        } else {
            Err(anyhow::anyhow!(
                "Root not found for block number {:?}",
                block_number
            ))
        }
    }

    async fn height(&self) -> NodeLevel {
        FEE_ACCOUNT_TREE_HEIGHT
    }

    async fn allocate_leaf_index(&mut self) -> LeafIndex {
        Default::default()
    }

    async fn get_leaf(
        &self,
        leaf_index: LeafIndex,
        block_number: Option<BlockNumber>,
    ) -> Result<Option<FeeAccountLeaf>, Error> {
        let block_hash = match block_number {
            Some(num) => self.api.client().get_block_hash(num).await?,
            None => None,
        };
        if let Some(block_hash) = block_hash {
            if let Some(leaf) = self
                .api
                .query_at(block_hash)
                .confidential_assets()
                .fee_account_leaves(leaf_index)
                .await?
            {
                return Ok(Some(to_scale(&leaf)));
            }
        } else {
            if let Some(leaf) = self
                .api
                .query()
                .confidential_assets()
                .fee_account_leaves(leaf_index)
                .await?
            {
                return Ok(Some(to_scale(&leaf)));
            }
        }
        Ok(None)
    }

    async fn leaf_count(&self) -> LeafIndex {
        0
    }

    async fn get_inner_node(
        &self,
        location: NodeLocation<FEE_ACCOUNT_TREE_L>,
        block_number: Option<BlockNumber>,
    ) -> Result<Option<FeeAccountInnerNode>, Error> {
        let block_hash = match block_number {
            Some(num) => self.api.client().get_block_hash(num).await?,
            None => None,
        };
        let location = node_location_to_chain(location);
        if let Some(block_hash) = block_hash {
            if let Some(node) = self
                .api
                .query_at(block_hash)
                .confidential_assets()
                .fee_account_inner_nodes(location)
                .await?
            {
                return Ok(Some(to_scale(&node)));
            }
        } else {
            if let Some(node) = self
                .api
                .query()
                .confidential_assets()
                .fee_account_inner_nodes(location)
                .await?
            {
                return Ok(Some(to_scale(&node)));
            }
        }
        Ok(None)
    }
}

pub type FeeAccountCurveTreeType = AsyncCurveTreeWithBackend<
    FEE_ACCOUNT_TREE_L,
    FEE_ACCOUNT_TREE_M,
    FeeAccountTreeConfig,
    FeeAccountCurveTreeChainStorage,
    anyhow::Error,
>;

#[derive(Clone)]
pub struct FeeAccountCurveTree(FeeAccountCurveTreeType);

impl Deref for FeeAccountCurveTree {
    type Target = FeeAccountCurveTreeType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FeeAccountCurveTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FeeAccountCurveTree {
    pub async fn new(api: &Api) -> Result<Self> {
        let backend = FeeAccountCurveTreeChainStorage::new(api);
        Ok(Self(
            AsyncCurveTreeWithBackend::new_with_backend(backend).await?,
        ))
    }
}
