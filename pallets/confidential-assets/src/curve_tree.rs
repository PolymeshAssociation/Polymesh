use core::ops::{Deref, DerefMut};

use polymesh_dart::{
    curve_tree::{
        CompressedCurveTreeRoot, CompressedInner, CompressedLeafValue, CurveTreeBackend,
        CurveTreeParameters, CurveTreeWithBackend, NodeLocation, ValidateCurveTreeRoot,
    },
    BlockNumber, LeafIndex, NodeLevel, ACCOUNT_TREE_HEIGHT, ACCOUNT_TREE_L, ACCOUNT_TREE_M,
    ASSET_TREE_HEIGHT, ASSET_TREE_L, ASSET_TREE_M, FEE_ACCOUNT_TREE_HEIGHT, FEE_ACCOUNT_TREE_L,
    FEE_ACCOUNT_TREE_M,
};
use polymesh_worker_protocol_dart_v0::{GetSessionId, HostCurveTreeUpdater};

#[cfg(feature = "std")]
use polymesh_dart::curve_tree::CurveTreeConfig;
pub use polymesh_dart::curve_tree::{AccountTreeConfig, AssetTreeConfig, FeeAccountTreeConfig};

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

use super::*;

#[cfg(feature = "std")]
lazy_static::lazy_static! {
    static ref ASSET_CURVE_TREE_PARAMETERS: CurveTreeParameters<AssetTreeConfig> = AssetTreeConfig::build_parameters();
    static ref ACCOUNT_CURVE_TREE_PARAMETERS: CurveTreeParameters<AccountTreeConfig> = AccountTreeConfig::build_parameters();
}

#[cfg(not(feature = "std"))]
static mut ASSET_CURVE_TREE_PARAMETERS: Option<CurveTreeParameters<AssetTreeConfig>> = None;
#[cfg(not(feature = "std"))]
static mut ACCOUNT_CURVE_TREE_PARAMETERS: Option<CurveTreeParameters<AccountTreeConfig>> = None;

impl<T: Config> GetSessionId for Pallet<T> {
    fn session_id(&self) -> Option<WorkerSessionId> {
        Pallet::<T>::session_id().ok()
    }
}

impl<T: Config> Pallet<T> {
    #[cfg(feature = "std")]
    pub fn get_asset_curve_tree_parameters() -> &'static CurveTreeParameters<AssetTreeConfig> {
        &ASSET_CURVE_TREE_PARAMETERS
    }

    #[allow(static_mut_refs)]
    #[cfg(not(feature = "std"))]
    pub fn get_asset_curve_tree_parameters() -> &'static CurveTreeParameters<AssetTreeConfig> {
        unsafe {
            if ASSET_CURVE_TREE_PARAMETERS.is_none() {
                let parameters = Self::get_cached_asset_curve_tree_parameters()
                    .expect("Failed to decode curve tree parameters");
                ASSET_CURVE_TREE_PARAMETERS = Some(parameters);
            }
            ASSET_CURVE_TREE_PARAMETERS.as_ref().unwrap()
        }
    }

    #[cfg(feature = "std")]
    pub fn get_account_curve_tree_parameters() -> &'static CurveTreeParameters<AccountTreeConfig> {
        &ACCOUNT_CURVE_TREE_PARAMETERS
    }

    #[allow(static_mut_refs)]
    #[cfg(not(feature = "std"))]
    pub fn get_account_curve_tree_parameters() -> &'static CurveTreeParameters<AccountTreeConfig> {
        unsafe {
            if ACCOUNT_CURVE_TREE_PARAMETERS.is_none() {
                let parameters = Self::get_cached_account_curve_tree_parameters()
                    .expect("Failed to decode curve tree parameters");
                ACCOUNT_CURVE_TREE_PARAMETERS = Some(parameters);
            }
            ACCOUNT_CURVE_TREE_PARAMETERS.as_ref().unwrap()
        }
    }

    pub fn get_cached_asset_curve_tree_parameters() -> Option<CurveTreeParameters<AssetTreeConfig>>
    {
        CachedAssetCurveTreeParameters::<T>::get()
            .or_else(|| {
                // If the parameters are not found in the storage, generate and cache them.
                let params = WrappedCurveTreeParameters::new::<AssetTreeConfig>().ok();
                if let Some(ref params) = params {
                    CachedAssetCurveTreeParameters::<T>::put(params);
                }
                params
            })
            .and_then(|p| p.decode::<AssetTreeConfig>().ok())
    }

    pub fn get_cached_account_curve_tree_parameters(
    ) -> Option<CurveTreeParameters<AccountTreeConfig>> {
        CachedAccountCurveTreeParameters::<T>::get()
            .or_else(|| {
                // If the parameters are not found in the storage, generate and cache them.
                let params = WrappedCurveTreeParameters::new::<AccountTreeConfig>().ok();
                if let Some(ref params) = params {
                    CachedAccountCurveTreeParameters::<T>::put(params);
                }
                params
            })
            .and_then(|p| p.decode::<AccountTreeConfig>().ok())
    }
}

fn get_block_number<T: Config>() -> Result<BlockNumberFor<T>, Error<T>> {
    Ok(frame_system::Pallet::<T>::block_number())
}

/// Asset Curve Tree Storage backend.
#[derive(Clone)]
pub struct AssetCurveTreeStorage<T: Config> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: Config> CurveTreeBackend<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>
    for AssetCurveTreeStorage<T>
{
    type Error = Error<T>;
    type Updater = HostCurveTreeUpdater<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig, Pallet<T>>;

    fn new(_height: NodeLevel) -> Result<Self, Self::Error> {
        Ok(Self {
            _marker: core::marker::PhantomData,
        })
    }

    fn parameters(&self) -> &CurveTreeParameters<AssetTreeConfig> {
        Pallet::<T>::get_asset_curve_tree_parameters()
    }

    fn get_block_number(&self) -> Result<BlockNumber, Self::Error> {
        // Return the current block number from the storage.
        Ok(get_block_number::<T>()?.try_into().unwrap_or_default())
    }

    fn current_root(
        &self,
    ) -> Result<CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>, Self::Error>
    {
        AssetCurveTreeCurrentRoot::<T>::get().ok_or(Error::<T>::CurveTreeRootNotFound)
    }

    fn store_root(
        &mut self,
        root: CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>,
    ) -> Result<BlockNumber, Self::Error> {
        let block_number = get_block_number::<T>()?;

        // Store the root in the storage.
        AssetCurveTreeRoots::<T>::insert(block_number, &root);

        // Store the block number for the asset curve tree update.
        AssetCurveTreeLastUpdate::<T>::put(block_number);

        // Update the current root.
        AssetCurveTreeCurrentRoot::<T>::put(&root);

        // Emit an event for the asset curve tree root update.
        Pallet::<T>::deposit_event(Event::<T>::AssetCurveTreeRootUpdated { root });

        Ok(block_number.try_into().unwrap_or_default())
    }

    fn fetch_root(
        &self,
        block_number: Option<BlockNumber>,
    ) -> Result<CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>, Self::Error>
    {
        if let Some(block_number) = block_number {
            let block_number: BlockNumberFor<T> = block_number.into();
            // Fetch the root from the storage.
            let root = AssetCurveTreeRoots::<T>::get(block_number)
                .ok_or(Error::<T>::CurveTreeRootNotFound)?;
            Ok(root)
        } else {
            AssetCurveTreeCurrentRoot::<T>::get().ok_or(Error::<T>::CurveTreeRootNotFound)
        }
    }

    fn height(&self) -> NodeLevel {
        ASSET_TREE_HEIGHT
    }

    fn set_height(&mut self, _height: NodeLevel) -> Result<(), Self::Error> {
        Ok(())
    }

    fn allocate_leaf_index(&mut self) -> LeafIndex {
        let leaf_index = AssetLeaves::<T>::count() as LeafIndex;
        leaf_index
    }

    fn get_leaf(
        &self,
        leaf_index: LeafIndex,
        _block_number: Option<BlockNumber>,
    ) -> Result<Option<AssetLeaf>, Self::Error> {
        let leaf = AssetLeaves::<T>::get(leaf_index);
        Ok(leaf)
    }

    fn set_leaf(
        &mut self,
        leaf_index: LeafIndex,
        new_leaf_value: AssetLeaf,
    ) -> Result<Option<AssetLeaf>, Self::Error> {
        // Use `mutate` to update the leaf value in the storage and return the previous value.
        Ok(AssetLeaves::<T>::mutate(leaf_index, |leaf| {
            // Get the old value, if it exists.
            let previous_value = leaf.take();
            // Set the new value.
            *leaf = Some(new_leaf_value);
            previous_value
        }))
    }

    fn leaf_count(&self) -> LeafIndex {
        AssetLeaves::<T>::count() as LeafIndex
    }

    fn get_inner_node(
        &self,
        location: NodeLocation<ASSET_TREE_L>,
        _block_number: Option<BlockNumber>,
    ) -> Result<Option<AssetInnerNode>, Self::Error> {
        Ok(AssetInnerNodes::<T>::get(location))
    }

    fn set_inner_node(
        &mut self,
        location: NodeLocation<ASSET_TREE_L>,
        new_node: AssetInnerNode,
    ) -> Result<(), Self::Error> {
        // set the inner node in the storage.
        AssetInnerNodes::<T>::insert(location, new_node);
        Ok(())
    }
}

/// Account Curve Tree Storage backend.
#[derive(Clone)]
pub struct AccountCurveTreeStorage<T: Config> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: Config> CurveTreeBackend<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>
    for AccountCurveTreeStorage<T>
{
    type Error = Error<T>;
    type Updater =
        HostCurveTreeUpdater<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig, Pallet<T>>;

    fn new(_height: NodeLevel) -> Result<Self, Self::Error> {
        Ok(Self {
            _marker: core::marker::PhantomData,
        })
    }

    fn parameters(&self) -> &CurveTreeParameters<AccountTreeConfig> {
        Pallet::<T>::get_account_curve_tree_parameters()
    }

    fn get_block_number(&self) -> Result<BlockNumber, Self::Error> {
        // Return the current block number from the storage.
        Ok(get_block_number::<T>()?.try_into().unwrap_or_default())
    }

    fn current_root(
        &self,
    ) -> Result<
        CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>,
        Self::Error,
    > {
        AccountCurveTreeCurrentRoot::<T>::get().ok_or(Error::<T>::CurveTreeRootNotFound)
    }

    fn store_root(
        &mut self,
        root: CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>,
    ) -> Result<BlockNumber, Self::Error> {
        let block_number = get_block_number::<T>()?;
        // Store the root in the storage.
        AccountCurveTreeRoots::<T>::insert(block_number, &root);

        // Store the block number for the account curve tree update.
        AccountCurveTreeLastUpdate::<T>::put(block_number);

        // Update the current root.
        AccountCurveTreeCurrentRoot::<T>::put(&root);

        // Emit an event for the account curve tree root update.
        Pallet::<T>::deposit_event(Event::<T>::AccountCurveTreeRootUpdated { root });

        Ok(block_number.try_into().unwrap_or_default())
    }

    fn fetch_root(
        &self,
        block_number: Option<BlockNumber>,
    ) -> Result<
        CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>,
        Self::Error,
    > {
        if let Some(block_number) = block_number {
            let block_number: BlockNumberFor<T> = block_number.into();
            // Fetch the root from the storage.
            let root = AccountCurveTreeRoots::<T>::get(block_number)
                .ok_or(Error::<T>::CurveTreeRootNotFound)?;
            Ok(root)
        } else {
            AccountCurveTreeCurrentRoot::<T>::get().ok_or(Error::<T>::CurveTreeRootNotFound)
        }
    }

    fn height(&self) -> NodeLevel {
        ACCOUNT_TREE_HEIGHT
    }

    fn set_height(&mut self, _height: NodeLevel) -> Result<(), Self::Error> {
        Ok(())
    }

    fn allocate_leaf_index(&mut self) -> LeafIndex {
        Pallet::<T>::next_account_leaf_index()
    }

    fn batch_inserts_supported(&self) -> bool {
        true
    }

    fn last_committed_leaf_index(&self) -> Result<LeafIndex, Self::Error> {
        let leaf_index = LastCommittedAccountLeafIndex::<T>::get();
        Ok(leaf_index)
    }

    fn set_committed_leaf_index(&mut self, leaf_index: LeafIndex) -> Result<(), Self::Error> {
        LastCommittedAccountLeafIndex::<T>::put(leaf_index);
        Ok(())
    }

    fn get_leaf(
        &self,
        leaf_index: LeafIndex,
        _block_number: Option<BlockNumber>,
    ) -> Result<Option<AccountLeaf>, Self::Error> {
        match AccountLeaves::<T>::get(leaf_index) {
            Some(commitment) => Ok(Some(commitment.as_leaf_value()?)),
            None => Ok(None),
        }
    }

    fn set_leaf(
        &mut self,
        leaf_index: LeafIndex,
        new_leaf_value: AccountLeaf,
    ) -> Result<Option<AccountLeaf>, Self::Error> {
        let account_commitment = AccountStateCommitment::from(new_leaf_value);
        // Use `mutate` to update the leaf value in the storage and return the previous value.
        AccountLeaves::<T>::set(leaf_index, Some(account_commitment));

        // The account curve tree is append-only, so we don't support updating existing leaves.
        Ok(None)
    }

    fn leaf_count(&self) -> LeafIndex {
        NextAccountLeafIndex::<T>::get() as LeafIndex
    }

    fn get_inner_node(
        &self,
        location: NodeLocation<ACCOUNT_TREE_L>,
        _block_number: Option<BlockNumber>,
    ) -> Result<Option<AccountInnerNode>, Self::Error> {
        Ok(AccountInnerNodes::<T>::get(location))
    }

    fn set_inner_node(
        &mut self,
        location: NodeLocation<ACCOUNT_TREE_L>,
        new_node: AccountInnerNode,
    ) -> Result<(), Self::Error> {
        // set the inner node in the storage.
        AccountInnerNodes::<T>::insert(location, new_node);
        Ok(())
    }
}

/// Fee Account Curve Tree Storage backend.
#[derive(Clone)]
pub struct FeeAccountCurveTreeStorage<T: Config> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: Config> CurveTreeBackend<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>
    for FeeAccountCurveTreeStorage<T>
{
    type Error = Error<T>;
    type Updater = HostCurveTreeUpdater<
        FEE_ACCOUNT_TREE_L,
        FEE_ACCOUNT_TREE_M,
        FeeAccountTreeConfig,
        Pallet<T>,
    >;

    fn new(_height: NodeLevel) -> Result<Self, Self::Error> {
        Ok(Self {
            _marker: core::marker::PhantomData,
        })
    }

    fn parameters(&self) -> &CurveTreeParameters<FeeAccountTreeConfig> {
        Pallet::<T>::get_account_curve_tree_parameters()
    }

    fn get_block_number(&self) -> Result<BlockNumber, Self::Error> {
        // Return the current block number from the storage.
        Ok(get_block_number::<T>()?.try_into().unwrap_or_default())
    }

    fn current_root(
        &self,
    ) -> Result<
        CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>,
        Self::Error,
    > {
        FeeAccountCurveTreeCurrentRoot::<T>::get().ok_or(Error::<T>::CurveTreeRootNotFound)
    }

    fn store_root(
        &mut self,
        root: CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>,
    ) -> Result<BlockNumber, Self::Error> {
        let block_number = get_block_number::<T>()?;
        // Store the root in the storage.
        FeeAccountCurveTreeRoots::<T>::insert(block_number, &root);

        // Store the block number for the account curve tree update.
        FeeAccountCurveTreeLastUpdate::<T>::put(block_number);

        // Update the current root.
        FeeAccountCurveTreeCurrentRoot::<T>::put(&root);

        // Emit an event for the account curve tree root update.
        Pallet::<T>::deposit_event(Event::<T>::FeeAccountCurveTreeRootUpdated { root });

        Ok(block_number.try_into().unwrap_or_default())
    }

    fn fetch_root(
        &self,
        block_number: Option<BlockNumber>,
    ) -> Result<
        CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>,
        Self::Error,
    > {
        if let Some(block_number) = block_number {
            let block_number: BlockNumberFor<T> = block_number.into();
            // Fetch the root from the storage.
            let root = FeeAccountCurveTreeRoots::<T>::get(block_number)
                .ok_or(Error::<T>::CurveTreeRootNotFound)?;
            Ok(root)
        } else {
            FeeAccountCurveTreeCurrentRoot::<T>::get().ok_or(Error::<T>::CurveTreeRootNotFound)
        }
    }

    fn height(&self) -> NodeLevel {
        FEE_ACCOUNT_TREE_HEIGHT
    }

    fn set_height(&mut self, _height: NodeLevel) -> Result<(), Self::Error> {
        Ok(())
    }

    fn allocate_leaf_index(&mut self) -> LeafIndex {
        Pallet::<T>::next_fee_account_leaf_index()
    }

    fn batch_inserts_supported(&self) -> bool {
        true
    }

    fn last_committed_leaf_index(&self) -> Result<LeafIndex, Self::Error> {
        let leaf_index = LastCommittedFeeAccountLeafIndex::<T>::get();
        Ok(leaf_index)
    }

    fn set_committed_leaf_index(&mut self, leaf_index: LeafIndex) -> Result<(), Self::Error> {
        LastCommittedFeeAccountLeafIndex::<T>::put(leaf_index);
        Ok(())
    }

    fn get_leaf(
        &self,
        leaf_index: LeafIndex,
        _block_number: Option<BlockNumber>,
    ) -> Result<Option<FeeAccountLeaf>, Self::Error> {
        match FeeAccountLeaves::<T>::get(leaf_index) {
            Some(commitment) => Ok(Some(commitment.as_leaf_value()?)),
            None => Ok(None),
        }
    }

    fn set_leaf(
        &mut self,
        leaf_index: LeafIndex,
        new_leaf_value: FeeAccountLeaf,
    ) -> Result<Option<FeeAccountLeaf>, Self::Error> {
        let account_commitment = FeeAccountStateCommitment::from(new_leaf_value);
        // Use `mutate` to update the leaf value in the storage and return the previous value.
        FeeAccountLeaves::<T>::set(leaf_index, Some(account_commitment));

        // The account curve tree is append-only, so we don't support updating existing leaves.
        Ok(None)
    }

    fn leaf_count(&self) -> LeafIndex {
        NextFeeAccountLeafIndex::<T>::get() as LeafIndex
    }

    fn get_inner_node(
        &self,
        location: NodeLocation<FEE_ACCOUNT_TREE_L>,
        _block_number: Option<BlockNumber>,
    ) -> Result<Option<FeeAccountInnerNode>, Self::Error> {
        Ok(FeeAccountInnerNodes::<T>::get(location))
    }

    fn set_inner_node(
        &mut self,
        location: NodeLocation<FEE_ACCOUNT_TREE_L>,
        new_node: FeeAccountInnerNode,
    ) -> Result<(), Self::Error> {
        // set the inner node in the storage.
        FeeAccountInnerNodes::<T>::insert(location, new_node);
        Ok(())
    }
}

pub type AssetCurveTreeType<T> = CurveTreeWithBackend<
    ASSET_TREE_L,
    ASSET_TREE_M,
    AssetTreeConfig,
    AssetCurveTreeStorage<T>,
    Error<T>,
>;

pub struct AssetCurveTree<T: Config>(AssetCurveTreeType<T>);

impl<T: Config> Deref for AssetCurveTree<T> {
    type Target = AssetCurveTreeType<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Config> DerefMut for AssetCurveTree<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Config> AssetCurveTree<T> {
    pub fn new(height: NodeLevel) -> Result<Self, Error<T>> {
        Ok(Self(CurveTreeWithBackend::new_no_init(height)?))
    }
}

impl<T: Config> ValidateCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>
    for &AssetCurveTree<T>
{
    fn get_block_root(&self, block: BlockNumber) -> Option<AssetTreeRoot> {
        let block: BlockNumberFor<T> = block.into();
        AssetCurveTreeRoots::<T>::get(block)
    }

    fn params(&self) -> &CurveTreeParameters<AssetTreeConfig> {
        self.0.parameters()
    }
}

pub type AccountCurveTreeType<T> = CurveTreeWithBackend<
    ACCOUNT_TREE_L,
    ACCOUNT_TREE_M,
    AccountTreeConfig,
    AccountCurveTreeStorage<T>,
    Error<T>,
>;

pub struct AccountCurveTree<T: Config>(AccountCurveTreeType<T>);

impl<T: Config> Deref for AccountCurveTree<T> {
    type Target = AccountCurveTreeType<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Config> DerefMut for AccountCurveTree<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Config> AccountCurveTree<T> {
    pub fn new(height: NodeLevel) -> Result<Self, Error<T>> {
        Ok(Self(CurveTreeWithBackend::new_no_init(height)?))
    }
}

impl<T: Config> ValidateCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>
    for &AccountCurveTree<T>
{
    fn get_block_root(&self, block: BlockNumber) -> Option<AccountTreeRoot> {
        let block: BlockNumberFor<T> = block.into();
        AccountCurveTreeRoots::<T>::get(block)
    }

    fn params(&self) -> &CurveTreeParameters<AccountTreeConfig> {
        self.parameters()
    }
}

pub type FeeAccountCurveTreeType<T> = CurveTreeWithBackend<
    FEE_ACCOUNT_TREE_L,
    FEE_ACCOUNT_TREE_M,
    FeeAccountTreeConfig,
    FeeAccountCurveTreeStorage<T>,
    Error<T>,
>;

pub struct FeeAccountCurveTree<T: Config>(FeeAccountCurveTreeType<T>);

impl<T: Config> Deref for FeeAccountCurveTree<T> {
    type Target = FeeAccountCurveTreeType<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Config> DerefMut for FeeAccountCurveTree<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Config> FeeAccountCurveTree<T> {
    pub fn new(height: NodeLevel) -> Result<Self, Error<T>> {
        Ok(Self(CurveTreeWithBackend::new_no_init(height)?))
    }
}

impl<T: Config> ValidateCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>
    for &FeeAccountCurveTree<T>
{
    fn get_block_root(&self, block: BlockNumber) -> Option<FeeAccountTreeRoot> {
        let block: BlockNumberFor<T> = block.into();
        FeeAccountCurveTreeRoots::<T>::get(block)
    }

    fn params(&self) -> &CurveTreeParameters<FeeAccountTreeConfig> {
        self.parameters()
    }
}
