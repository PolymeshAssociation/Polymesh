use core::ops::{Deref, DerefMut};

use codec::DecodeWithMemTracking;
use polymesh_dart::{
    curve_tree::{
        CompressedCurveTreeRoot, CompressedInner, CompressedLeafValue, CurveTreeBackend,
        CurveTreeParameters, CurveTreeWithBackend, NodeLocation, ValidateCurveTreeRoot,
    },
    BlockNumber, LeafIndex, NodeLevel, ACCOUNT_TREE_L, ACCOUNT_TREE_M, ASSET_TREE_L, ASSET_TREE_M,
    FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M,
};
use polymesh_worker_protocol_dart_v1::{GetSessionId, HostCurveTreeUpdater};

#[cfg(feature = "std")]
use polymesh_dart::curve_tree::CurveTreeConfig;
pub use polymesh_dart::curve_tree::{AccountTreeConfig, AssetTreeConfig, FeeAccountTreeConfig};
use sp_runtime::Saturating;

pub type AssetLeaf = CompressedLeafValue<AssetTreeConfig>;
pub type AssetInnerNode = CompressedInner<ASSET_TREE_M, AssetTreeConfig>;
pub type AssetNodeLocation = NodeLocation<ASSET_TREE_L>;
pub type AssetTreeRoot = CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>;
pub type TimestampedAssetTreeRoot<T> = TimestampedTreeRoot<T, AssetTreeRoot>;

pub type AccountLeaf = CompressedLeafValue<AccountTreeConfig>;
pub type AccountInnerNode = CompressedInner<ACCOUNT_TREE_M, AccountTreeConfig>;
pub type AccountNodeLocation = NodeLocation<ACCOUNT_TREE_L>;
pub type AccountTreeRoot =
    CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>;
pub type TimestampedAccountTreeRoot<T> = TimestampedTreeRoot<T, AccountTreeRoot>;

pub type FeeAccountLeaf = CompressedLeafValue<FeeAccountTreeConfig>;
pub type FeeAccountInnerNode = CompressedInner<FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>;
pub type FeeAccountNodeLocation = NodeLocation<FEE_ACCOUNT_TREE_L>;
pub type FeeAccountTreeRoot =
    CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>;
pub type TimestampedFeeAccountTreeRoot<T> = TimestampedTreeRoot<T, FeeAccountTreeRoot>;

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

    /// Get the asset curve tree root for a given block number if it is still valid based on the current timestamp and the maximum age defined in the pallet configuration.
    ///
    /// Returns an error if the root is not found or has expired.
    pub fn get_asset_curve_tree_root(
        block_number: BlockNumberFor<T>,
    ) -> Result<AssetTreeRoot, Error<T>> {
        AssetCurveTreeRoots::<T>::get(block_number)
            .and_then(|timestamped_root| {
                timestamped_root
                    .get_root_if_valid(get_timestamp::<T>(), T::MaxAssetCurveTreeRootAge::get())
            })
            .ok_or(Error::<T>::CurveTreeRootNotFound)
    }

    /// Store the asset curve tree root.
    pub fn store_asset_curve_tree_root(root: TimestampedAssetTreeRoot<T>) -> BlockNumberFor<T> {
        let block_number = root.block_number();

        // Store the root in the storage.
        AssetCurveTreeRoots::<T>::insert(block_number, &root);

        // Store the block number for the asset curve tree update.
        AssetCurveTreeLastUpdate::<T>::put(root.timestamp());

        // Update the current root.
        AssetCurveTreeCurrentRoot::<T>::put(&root);

        // Emit an event for the asset curve tree root update.
        Pallet::<T>::deposit_event(Event::<T>::AssetCurveTreeRootUpdated { root });

        block_number
    }

    /// Get the account curve tree root for a given block number if it is still valid based on the current timestamp and the maximum age defined in the pallet configuration.
    ///
    /// Returns an error if the root is not found or has expired.
    pub fn get_account_curve_tree_root(
        block_number: BlockNumberFor<T>,
    ) -> Result<AccountTreeRoot, Error<T>> {
        AccountCurveTreeRoots::<T>::get(block_number)
            .and_then(|timestamped_root| {
                timestamped_root
                    .get_root_if_valid(get_timestamp::<T>(), T::MaxAccountCurveTreeRootAge::get())
            })
            .ok_or(Error::<T>::CurveTreeRootNotFound)
    }

    /// Store the account curve tree root.
    pub fn store_account_curve_tree_root(root: TimestampedAccountTreeRoot<T>) -> BlockNumberFor<T> {
        let block_number = root.block_number();

        // Store the root in the storage.
        AccountCurveTreeRoots::<T>::insert(block_number, &root);

        // Store the block number for the account curve tree update.
        AccountCurveTreeLastUpdate::<T>::put(root.timestamp());

        // Update the current root.
        AccountCurveTreeCurrentRoot::<T>::put(&root);

        // Emit an event for the account curve tree root update.
        Pallet::<T>::deposit_event(Event::<T>::AccountCurveTreeRootUpdated { root });

        block_number
    }

    /// Get the fee account curve tree root for a given block number if it is still valid based on the current timestamp and the maximum age defined in the pallet configuration.
    ///
    /// Returns an error if the root is not found or has expired.
    pub fn get_fee_account_curve_tree_root(
        block_number: BlockNumberFor<T>,
    ) -> Result<FeeAccountTreeRoot, Error<T>> {
        FeeAccountCurveTreeRoots::<T>::get(block_number)
            .and_then(|timestamped_root| {
                timestamped_root.get_root_if_valid(
                    get_timestamp::<T>(),
                    T::MaxFeeAccountCurveTreeRootAge::get(),
                )
            })
            .ok_or(Error::<T>::CurveTreeRootNotFound)
    }

    /// Store the fee account curve tree root.
    pub fn store_fee_account_curve_tree_root(
        root: TimestampedFeeAccountTreeRoot<T>,
    ) -> BlockNumberFor<T> {
        let block_number = root.block_number();

        // Store the root in the storage.
        FeeAccountCurveTreeRoots::<T>::insert(block_number, &root);

        // Store the block number for the fee account curve tree update.
        FeeAccountCurveTreeLastUpdate::<T>::put(root.timestamp());

        // Update the current root.
        FeeAccountCurveTreeCurrentRoot::<T>::put(&root);

        // Emit an event for the fee account curve tree root update.
        Pallet::<T>::deposit_event(Event::<T>::FeeAccountCurveTreeRootUpdated { root });

        block_number
    }
}

fn get_block_number<T: Config>() -> BlockNumberFor<T> {
    frame_system::Pallet::<T>::block_number()
}

fn get_timestamp<T: Config>() -> T::Moment {
    pallet_timestamp::Pallet::<T>::get()
}

/// Curve tree timestamp and block number.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct CurveTreeTimestamp<T: Config> {
    pub timestamp: T::Moment,
    pub block_number: BlockNumberFor<T>,
}

impl<T: Config> Default for CurveTreeTimestamp<T> {
    fn default() -> Self {
        Self {
            timestamp: Default::default(),
            block_number: Default::default(),
        }
    }
}

impl<T: Config> sp_std::fmt::Debug for CurveTreeTimestamp<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        f.debug_struct("CurveTreeTimestamp")
            .field("timestamp", &self.timestamp)
            .field("block_number", &self.block_number)
            .finish()
    }
}

impl<T: Config> CurveTreeTimestamp<T> {
    /// Create a new `CurveTreeTimestamp` with the current timestamp and block number. Returns an error if the timestamp or block number cannot be retrieved.
    pub fn new() -> Self {
        Self {
            timestamp: get_timestamp::<T>(),
            block_number: get_block_number::<T>(),
        }
    }

    /// Check if the curve tree needs to be updated based on the current time and the minimum update interval. Returns `true` if an update is needed.
    pub fn needs_update(&self, current_time: T::Moment, min_update_interval: T::Moment) -> bool {
        current_time.saturating_sub(self.timestamp) >= min_update_interval
    }

    /// Check if the curve tree root is still valid based on the current time and the maximum age. Returns `true` if the root is valid.
    pub fn is_valid(&self, current_time: T::Moment, max_age: T::Moment) -> bool {
        current_time.saturating_sub(self.timestamp) <= max_age
    }
}

/// A struct to represent a curve tree root along with its timestamp and block number.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct TimestampedTreeRoot<T: Config, Root> {
    pub root: Root,
    pub timestamp: CurveTreeTimestamp<T>,
}

impl<T: Config, Root: sp_std::fmt::Debug> sp_std::fmt::Debug for TimestampedTreeRoot<T, Root> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        f.debug_struct("TimestampedTreeRoot")
            .field("root", &self.root)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

impl<T: Config, Root> TimestampedTreeRoot<T, Root> {
    /// Create a new `TimestampedTreeRoot` with the given root and the current timestamp and block number. Returns an error if the timestamp or block number cannot be retrieved.
    pub fn new(root: Root) -> Self {
        Self {
            root,
            timestamp: CurveTreeTimestamp::new(),
        }
    }

    /// Check if the curve tree root is still valid based on the current time and the maximum age. Returns `true` if the root is valid.
    pub fn is_valid(&self, current_time: T::Moment, max_age: T::Moment) -> bool {
        self.timestamp.is_valid(current_time, max_age)
    }

    /// Get the root if it hasn't expired based on the provided maximum age. Returns `None` if the root has expired.
    pub fn get_root_if_valid(self, current_time: T::Moment, max_age: T::Moment) -> Option<Root> {
        if self.timestamp.is_valid(current_time, max_age) {
            Some(self.root)
        } else {
            None
        }
    }

    /// Get the timestamp of the tree root.
    pub fn timestamp(&self) -> CurveTreeTimestamp<T> {
        self.timestamp.clone()
    }

    /// Get the block number when the tree root was stored.
    pub fn block_number(&self) -> BlockNumberFor<T> {
        self.timestamp.block_number
    }
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
        Ok(get_block_number::<T>().try_into().unwrap_or_default())
    }

    fn current_root(
        &self,
    ) -> Result<CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>, Self::Error>
    {
        AssetCurveTreeCurrentRoot::<T>::get()
            .map(|t| t.root)
            .ok_or(Error::<T>::CurveTreeRootNotFound)
    }

    fn store_root(
        &mut self,
        root: CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>,
    ) -> Result<BlockNumber, Self::Error> {
        let root = TimestampedTreeRoot::<T, _>::new(root);
        let block_number = Pallet::<T>::store_asset_curve_tree_root(root);

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
            let root = Pallet::<T>::get_asset_curve_tree_root(block_number)?;
            Ok(root)
        } else {
            AssetCurveTreeCurrentRoot::<T>::get()
                .map(|t| t.root)
                .ok_or(Error::<T>::CurveTreeRootNotFound)
        }
    }

    fn height(&self, _block_number: Option<BlockNumber>) -> Result<NodeLevel, Self::Error> {
        Ok(AssetCurveTreeHeight::<T>::get())
    }

    fn set_height(&mut self, height: NodeLevel) -> Result<(), Self::Error> {
        AssetCurveTreeHeight::<T>::put(height);
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
        Ok(get_block_number::<T>().try_into().unwrap_or_default())
    }

    fn current_root(
        &self,
    ) -> Result<
        CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>,
        Self::Error,
    > {
        AccountCurveTreeCurrentRoot::<T>::get()
            .map(|t| t.root)
            .ok_or(Error::<T>::CurveTreeRootNotFound)
    }

    fn store_root(
        &mut self,
        root: CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>,
    ) -> Result<BlockNumber, Self::Error> {
        let root = TimestampedTreeRoot::<T, _>::new(root);
        let block_number = Pallet::<T>::store_account_curve_tree_root(root);

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
            let root = Pallet::<T>::get_account_curve_tree_root(block_number)?;
            Ok(root)
        } else {
            AccountCurveTreeCurrentRoot::<T>::get()
                .map(|t| t.root)
                .ok_or(Error::<T>::CurveTreeRootNotFound)
        }
    }

    fn height(&self, _block_number: Option<BlockNumber>) -> Result<NodeLevel, Self::Error> {
        Ok(AccountCurveTreeHeight::<T>::get())
    }

    fn set_height(&mut self, height: NodeLevel) -> Result<(), Self::Error> {
        AccountCurveTreeHeight::<T>::put(height);
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
        Ok(get_block_number::<T>().try_into().unwrap_or_default())
    }

    fn current_root(
        &self,
    ) -> Result<
        CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>,
        Self::Error,
    > {
        FeeAccountCurveTreeCurrentRoot::<T>::get()
            .map(|t| t.root)
            .ok_or(Error::<T>::CurveTreeRootNotFound)
    }

    fn store_root(
        &mut self,
        root: CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>,
    ) -> Result<BlockNumber, Self::Error> {
        let root = TimestampedTreeRoot::<T, _>::new(root);
        let block_number = Pallet::<T>::store_fee_account_curve_tree_root(root);

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
            let root = Pallet::<T>::get_fee_account_curve_tree_root(block_number)?;
            Ok(root)
        } else {
            FeeAccountCurveTreeCurrentRoot::<T>::get()
                .map(|t| t.root)
                .ok_or(Error::<T>::CurveTreeRootNotFound)
        }
    }

    fn height(&self, _block_number: Option<BlockNumber>) -> Result<NodeLevel, Self::Error> {
        Ok(FeeAccountCurveTreeHeight::<T>::get())
    }

    fn set_height(&mut self, height: NodeLevel) -> Result<(), Self::Error> {
        FeeAccountCurveTreeHeight::<T>::put(height);
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
        Pallet::<T>::get_asset_curve_tree_root(block).ok()
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
        Pallet::<T>::get_account_curve_tree_root(block).ok()
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
        Pallet::<T>::get_fee_account_curve_tree_root(block).ok()
    }

    fn params(&self) -> &CurveTreeParameters<FeeAccountTreeConfig> {
        self.parameters()
    }
}
