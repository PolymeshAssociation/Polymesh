#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::dispatch::{DispatchResult, DispatchResultWithPostInfo, PostDispatchInfo};
use frame_support::pallet_prelude::DispatchError;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::{ensure, require_transactional};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::{vec, vec::Vec};

use pallet_asset::Frozen;
use pallet_base::try_next_pre;
use polymesh_primitives::asset::{
    AssetHolder, AssetHolderKind, AssetId, AssetName, AssetType, NonFungibleType,
};
use polymesh_primitives::asset_metadata::{AssetMetadataKey, AssetMetadataValue};
use polymesh_primitives::nft::{
    NFTCollection, NFTCollectionId, NFTCollectionKeys, NFTCount, NFTId, NFTMetadataAttribute,
    NFTOwnerStatus, NFTs,
};
use polymesh_primitives::portfolio::{Fund, FundDescription};
use polymesh_primitives::settlement::InstructionId;
use polymesh_primitives::traits::{ComplianceFnConfig, NFTTrait, SettlementFnTrait};
use polymesh_primitives::{
    AccountId as AccountId32, HoldingsUpdateReason, IdentityId, Memo, PortfolioId, WeightMeter,
};

type AssetPallet<T> = pallet_asset::Pallet<T>;
type ExternalAgents<T> = pallet_external_agents::Pallet<T>;
type IdentityPallet<T> = pallet_identity::Pallet<T>;
type PortfolioPallet<T> = pallet_portfolio::Pallet<T>;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub trait WeightInfo {
    fn create_nft_collection(n: u32) -> Weight;
    fn issue_nft(n: u32) -> Weight;
    fn redeem_nft(n: u32) -> Weight;
    fn base_nft_transfer(n: u32) -> Weight;
    fn controller_transfer(n: u32) -> Weight;
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::{OptionQuery, *};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_identity::Config
        + pallet_portfolio::Config
    {
        type WeightInfo: WeightInfo;
        type Compliance: ComplianceFnConfig;
        #[pallet::constant]
        type MaxNumberOfCollectionKeys: Get<u8>;
        #[pallet::constant]
        type MaxNumberOfNFTsCount: Get<u32>;
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when a new nft collection is created.
        NftCollectionCreated(IdentityId, AssetId, NFTCollectionId),
        /// Emitted when NFTs were issued, redeemed or transferred.
        /// Contains the [`IdentityId`] of the receiver/issuer/redeemer, the [`NFTs`], the [`AssetHolder`] of the source, the [`AssetHolder`]
        /// of the destination and the [`HoldingsUpdateReason`].
        NFTHoldingsUpdated(
            IdentityId,
            NFTs,
            Option<AssetHolder>,
            Option<AssetHolder>,
            HoldingsUpdateReason,
        ),
    }

    /// The total number of NFTs per identity.
    #[pallet::storage]
    pub type NumberOfNFTs<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, AssetId, Identity, IdentityId, NFTCount, ValueQuery>;

    /// The collection id corresponding to each asset.
    #[pallet::storage]
    pub type CollectionAsset<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, NFTCollectionId, ValueQuery>;

    /// All collection details for a given collection id.
    #[pallet::storage]
    pub type Collection<T: Config> =
        StorageMap<_, Blake2_128Concat, NFTCollectionId, NFTCollection, ValueQuery>;

    /// All mandatory metadata keys for a given collection.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type CollectionKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, NFTCollectionId, BTreeSet<AssetMetadataKey>, ValueQuery>;

    /// The metadata value of an nft given its collection id, token id and metadata key.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type MetadataValue<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (NFTCollectionId, NFTId),
        Blake2_128Concat,
        AssetMetadataKey,
        AssetMetadataValue,
        ValueQuery,
    >;

    /// The total number of NFTs in a collection.
    #[pallet::storage]
    pub type NFTsInCollection<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, NFTCount, ValueQuery>;

    /// The last `NFTId` used for an NFT.
    #[pallet::storage]
    pub type CurrentNFTId<T: Config> =
        StorageMap<_, Blake2_128Concat, NFTCollectionId, NFTId, OptionQuery>;

    /// The last `NFTCollectionId` used for a collection.
    #[pallet::storage]
    pub type CurrentCollectionId<T: Config> = StorageValue<_, NFTCollectionId, OptionQuery>;

    /// All NFTs associated to the account Key.
    #[pallet::storage]
    pub type NFTHolder<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Twox64Concat, AccountId32>,
            NMapKey<Blake2_128Concat, AssetId>,
            NMapKey<Blake2_128Concat, NFTId>,
        ),
        NFTOwnerStatus,
        ValueQuery,
    >;

    /// Reverse mapping for allowing to find the owner of a specific NFT.
    #[pallet::storage]
    pub type Owner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Blake2_128Concat,
        NFTId,
        AssetHolder,
        OptionQuery,
    >;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {}
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Cretes a new `NFTCollection`.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `asset_id` - optional [`AssetId`] associated to the new collection. `None` will create a new asset.
        /// * `nft_type` - in case the asset hasn't been created yet, one will be created with the given type.
        /// * `collection_keys` - all mandatory metadata keys that the tokens in the collection must have.
        ///
        /// ## Errors
        /// - `CollectionAlredyRegistered` - if the asset_id is already associated to an NFT collection.
        /// - `InvalidAssetType` - if the associated asset is not of type NFT.
        /// - `MaxNumberOfKeysExceeded` - if the number of metadata keys for the collection is greater than the maximum allowed.
        /// - `UnregisteredMetadataKey` - if any of the metadata keys needed for the collection has not been registered.
        /// - `DuplicateMetadataKey` - if a duplicate metadata keys has been passed as input.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::create_nft_collection(collection_keys.len() as u32))]
        #[pallet::call_index(0)]
        pub fn create_nft_collection(
            origin: OriginFor<T>,
            asset_id: Option<AssetId>,
            nft_type: Option<NonFungibleType>,
            collection_keys: NFTCollectionKeys,
        ) -> DispatchResult {
            Self::base_create_nft_collection(origin, asset_id, nft_type, collection_keys)
        }

        /// Issues an NFT to the caller.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id` - the [`AssetId`] of the NFT collection.
        /// * `nft_metadata_attributes` - all mandatory metadata keys and values for the NFT.
        /// - `holdings_kind` - the [`AssetHolderKind`] that will receive the minted nft.
        ///
        /// ## Errors
        /// - `CollectionNotFound` - if the collection associated to the given asset_id has not been created.
        /// - `InvalidMetadataAttribute` - if the number of attributes is not equal to the number set in the collection or attempting to set a value for a key not definied in the collection.
        /// - `DuplicateMetadataKey` - if a duplicate metadata keys has been passed as input.
        ///
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::issue_nft(nft_metadata_attributes.len() as u32))]
        #[pallet::call_index(1)]
        pub fn issue_nft(
            origin: OriginFor<T>,
            asset_id: AssetId,
            nft_metadata_attributes: Vec<NFTMetadataAttribute>,
            holdings_kind: AssetHolderKind,
        ) -> DispatchResult {
            Self::base_issue_nft(origin, asset_id, nft_metadata_attributes, holdings_kind)
        }

        /// Redeems the given NFT from the caller's portfolio.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `asset_id`.
        /// * `asset_id` - the [`AssetId`] of the NFT collection.
        /// * `nft_id` - the id of the NFT to be burned.
        /// * `holdings_kind` - the [`AssetHolderKind`] that contains the nft.
        ///
        /// ## Errors
        /// - `CollectionNotFound` - if the collection associated to the given asset_id has not been created.
        /// - `NFTNotFound` - if the given NFT does not exist in the portfolio.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::redeem_nft(
            number_of_keys.map_or(
                u32::from(T::MaxNumberOfCollectionKeys::get()),
                |v| u32::from(v)
            )
        ))]
        #[pallet::call_index(2)]
        pub fn redeem_nft(
            origin: OriginFor<T>,
            asset_id: AssetId,
            nft_id: NFTId,
            holdings_kind: AssetHolderKind,
            number_of_keys: Option<u8>,
        ) -> DispatchResultWithPostInfo {
            Self::base_redeem_nft(origin, asset_id, nft_id, holdings_kind, number_of_keys)
        }

        /// Forces the transfer of NFTs from a given portfolio to the caller's portfolio.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `asset_id`.
        /// * `nft_id` - the [`NFTId`] of the NFT to be transferred.
        /// * `source` - the [`AssetHolder`] that currently holds the NFT.
        /// * `destination_kind` - the [`AssetHolderKind`] of the caller's portfolio.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::controller_transfer(nfts.len() as u32))]
        #[pallet::call_index(3)]
        pub fn controller_transfer(
            origin: OriginFor<T>,
            nfts: NFTs,
            source: AssetHolder,
            destination_kind: AssetHolderKind,
        ) -> DispatchResult {
            Self::base_controller_transfer(origin, nfts, source, destination_kind)
        }

        /// Transfer NFTs from the caller's account to another account.
        ///
        /// Same-identity transfers move NFTs directly. Cross-identity transfers
        /// route through the settlement engine.
        ///
        /// For portfolio-based transfers, use `Settlement::transfer_funds`.
        ///
        /// # Arguments
        /// * `origin` — Signed origin. Caller must have a registered DID.
        /// * `nfts` — The NFTs to transfer.
        /// * `to` — Destination account.
        /// * `memo` — Optional memo attached to the transfer.
        #[pallet::call_index(4)]
        #[pallet::weight(
            <T as pallet_asset::Config>::SettlementFn::transfer_funds_weight_limit(
                None,
                &Fund { description: FundDescription::NonFungible(nfts.clone()), memo: memo.clone() },
            )
        )]
        pub fn transfer_nft(
            origin: OriginFor<T>,
            nfts: NFTs,
            to: T::AccountId,
            memo: Option<Memo>,
        ) -> DispatchResultWithPostInfo {
            Self::base_transfer_nft(origin, nfts, to, memo)
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// An overflow while calculating the balance.
        BalanceOverflow,
        /// An underflow while calculating the balance.
        BalanceUnderflow,
        /// The asset_id is already associated to an NFT collection.
        CollectionAlredyRegistered,
        /// The NFT collection does not exist.
        CollectionNotFound,
        /// A duplicate metadata key has been passed as parameter.
        DuplicateMetadataKey,
        /// Duplicate ids are not allowed.
        DuplicatedNFTId,
        /// The asset must be of type non-fungible.
        InvalidAssetType,
        /// Either the number of keys or the key identifier does not match the keys defined for the collection.
        InvalidMetadataAttribute,
        /// Failed to transfer an NFT - NFT collection not found.
        InvalidNFTTransferCollectionNotFound,
        /// Failed to transfer an NFT - attempt to move to the same portfolio.
        InvalidNFTTransferSamePortfolio,
        /// Failed to transfer an NFT - NFT not found in portfolio.
        InvalidNFTTransferNFTNotOwned,
        /// Failed to transfer an NFT - identity count would overflow.
        InvalidNFTTransferCountOverflow,
        /// Failed to transfer an NFT - compliance failed.
        InvalidNFTTransferComplianceFailure,
        /// Failed to transfer an NFT - asset is frozen.
        InvalidNFTTransferFrozenAsset,
        /// Failed to transfer an NFT - the number of nfts in the identity is insufficient.
        InvalidNFTTransferInsufficientCount,
        /// The maximum number of metadata keys was exceeded.
        MaxNumberOfKeysExceeded,
        /// The maximum number of nfts being transferred in one leg was exceeded.
        MaxNumberOfNFTsPerLegExceeded,
        /// The NFT does not exist.
        NFTNotFound,
        /// At least one of the metadata keys has not been registered.
        UnregisteredMetadataKey,
        /// It is not possible to transferr zero nft.
        ZeroCount,
        /// An overflow while calculating the updated supply.
        SupplyOverflow,
        /// An underflow while calculating the updated supply.
        SupplyUnderflow,
        /// Failed to transfer an NFT - nft is locked.
        InvalidNFTTransferNFTIsLocked,
        /// The sender identity can't be the same as the receiver identity.
        InvalidNFTTransferSenderDidMatchesReceiverDid,
        /// The receiver has an invalid DID.
        InvalidNFTTransferInvalidReceiverDID,
        /// There's no asset associated to the given asset_id.
        InvalidAssetId,
        /// The NFT is locked.
        NFTIsLocked,
        /// The number of keys in the collection is greater than the input.
        NumberOfKeysIsLessThanExpected,
        /// The NFT is not locked.
        NFTIsNotLocked,
    }
}

impl<T: Config> Pallet<T> {
    fn base_create_nft_collection(
        origin: T::RuntimeOrigin,
        asset_id: Option<AssetId>,
        nft_type: Option<NonFungibleType>,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult {
        // Verifies if the caller has asset permission and if the asset is an NFT.
        let (create_asset, caller_did, asset_id) = {
            match asset_id {
                Some(asset_id) => match AssetPallet::<T>::nft_asset(&asset_id) {
                    Some(is_nft_asset) => {
                        ensure!(is_nft_asset, Error::<T>::InvalidAssetType);
                        let caller_did = ExternalAgents::<T>::ensure_agent_asset_perms(
                            origin.clone(),
                            &asset_id,
                        )?
                        .primary_did;
                        (false, caller_did, asset_id)
                    }
                    None => return Err(Error::<T>::InvalidAssetId.into()),
                },
                None => {
                    let caller_data =
                        IdentityPallet::<T>::ensure_origin_call_permissions(origin.clone())?;
                    let asset_id = AssetPallet::<T>::generate_asset_id(caller_data.sender, false);
                    (true, caller_data.primary_did, asset_id)
                }
            }
        };

        // Verifies if the asset_id is already associated to an NFT collection
        ensure!(
            !CollectionAsset::<T>::contains_key(&asset_id),
            Error::<T>::CollectionAlredyRegistered
        );

        // Verifies if the maximum number of keys is respected
        ensure!(
            collection_keys.len() <= (T::MaxNumberOfCollectionKeys::get() as usize),
            Error::<T>::MaxNumberOfKeysExceeded
        );

        // Verifies that there are no duplicated keys
        let n_keys = collection_keys.len();
        let collection_keys: BTreeSet<AssetMetadataKey> = collection_keys.into_iter().collect();
        ensure!(
            n_keys == collection_keys.len(),
            Error::<T>::DuplicateMetadataKey
        );

        // Verifies that all keys have been registered
        for key in &collection_keys {
            ensure!(
                AssetPallet::<T>::check_asset_metadata_key_exists(&asset_id, key),
                Error::<T>::UnregisteredMetadataKey
            )
        }

        // Creates an nft asset if it hasn't been created yet
        if create_asset {
            let nft_type = nft_type.ok_or(Error::<T>::InvalidAssetType)?;
            AssetPallet::<T>::create_asset(
                origin,
                AssetName(Vec::new()),
                false,
                AssetType::NonFungible(nft_type),
                Vec::new(),
                None,
            )?;
        }

        // Creates the nft collection
        let collection_id = Self::update_current_collection_id()?;
        let nft_collection = NFTCollection::new(collection_id, asset_id.clone());
        Collection::<T>::insert(&collection_id, nft_collection);
        CollectionKeys::<T>::insert(&collection_id, collection_keys);
        CollectionAsset::<T>::insert(&asset_id, &collection_id);

        Self::deposit_event(Event::NftCollectionCreated(
            caller_did,
            asset_id,
            collection_id,
        ));
        Ok(())
    }

    fn base_issue_nft(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        metadata_attributes: Vec<NFTMetadataAttribute>,
        holdings_kind: AssetHolderKind,
    ) -> DispatchResult {
        // Verifies if the collection exists
        let collection_id =
            CollectionAsset::<T>::try_get(&asset_id).map_err(|_| Error::<T>::CollectionNotFound)?;

        // Verifies if the caller has the right permissions (regarding asset and portfolio/acc)
        let caller_holdings = AssetPallet::<T>::ensure_asset_and_holding_permissions(
            origin,
            asset_id.clone(),
            holdings_kind,
            false,
        )?;
        let holder_did = pallet_identity::Pallet::<T>::asset_holder_did(&caller_holdings)?;

        // Verifies that all mandatory keys are being set and that there are no duplicated keys
        let mandatory_keys: BTreeSet<AssetMetadataKey> = CollectionKeys::<T>::get(&collection_id);
        ensure!(
            mandatory_keys.len() == metadata_attributes.len(),
            Error::<T>::InvalidMetadataAttribute
        );

        let n_keys = metadata_attributes.len();
        let nft_attributes: BTreeMap<_, _> = metadata_attributes
            .into_iter()
            .map(|a| (a.key, a.value))
            .collect();
        ensure!(
            n_keys == nft_attributes.len(),
            Error::<T>::DuplicateMetadataKey
        );

        for metadata_key in nft_attributes.keys() {
            ensure!(
                mandatory_keys.contains(metadata_key),
                Error::<T>::InvalidMetadataAttribute
            );
        }

        // Mints the NFT and adds it to the caller's portfolio
        let new_supply = NFTsInCollection::<T>::get(&asset_id)
            .checked_add(1)
            .ok_or(Error::<T>::SupplyOverflow)?;
        let new_balance = NumberOfNFTs::<T>::get(&asset_id, &holder_did)
            .checked_add(1)
            .ok_or(Error::<T>::BalanceOverflow)?;
        let nft_id = Self::update_current_nft_id(&collection_id)?;

        NFTsInCollection::<T>::insert(&asset_id, new_supply);
        NumberOfNFTs::<T>::insert(&asset_id, &holder_did, new_balance);
        for (metadata_key, metadata_value) in nft_attributes.into_iter() {
            MetadataValue::<T>::insert((&collection_id, &nft_id), metadata_key, metadata_value);
        }
        Self::add_nft_holding(asset_id, nft_id, caller_holdings.clone())?;

        Self::deposit_event(Event::NFTHoldingsUpdated(
            holder_did,
            NFTs::new_unverified(asset_id, vec![nft_id]),
            None,
            Some(caller_holdings),
            HoldingsUpdateReason::Issued {
                funding_round_name: None,
            },
        ));
        Ok(())
    }

    fn base_redeem_nft(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        nft_id: NFTId,
        holdings_kind: AssetHolderKind,
        number_of_keys: Option<u8>,
    ) -> DispatchResultWithPostInfo {
        // Verifies if the collection exists
        let collection_id =
            CollectionAsset::<T>::try_get(&asset_id).map_err(|_| Error::<T>::CollectionNotFound)?;

        // Ensure origin is agent with custody and permissions for portfolio.
        let caller_holding = AssetPallet::<T>::ensure_asset_and_holding_permissions(
            origin,
            asset_id,
            holdings_kind,
            true,
        )?;
        let holder_did = pallet_identity::Pallet::<T>::asset_holder_did(&caller_holding)?;

        ensure!(
            Self::is_holder_of_nft(&asset_id, &nft_id, &caller_holding),
            Error::<T>::NFTNotFound
        );
        ensure!(
            !Self::is_nft_locked(&asset_id, &nft_id, &caller_holding),
            Error::<T>::NFTIsLocked
        );

        // Burns the NFT
        let new_supply = NFTsInCollection::<T>::get(&asset_id)
            .checked_sub(1)
            .ok_or(Error::<T>::SupplyUnderflow)?;
        let new_balance = NumberOfNFTs::<T>::get(&asset_id, &holder_did)
            .checked_sub(1)
            .ok_or(Error::<T>::BalanceUnderflow)?;

        NFTsInCollection::<T>::insert(&asset_id, new_supply);
        NumberOfNFTs::<T>::insert(&asset_id, &holder_did, new_balance);
        Self::remove_nft_from_asset_holder(&asset_id, &nft_id, &caller_holding)?;

        let removed_keys = MetadataValue::<T>::drain_prefix((&collection_id, &nft_id)).count();
        if let Some(number_of_keys) = number_of_keys {
            ensure!(
                usize::from(number_of_keys) >= removed_keys,
                Error::<T>::NumberOfKeysIsLessThanExpected,
            );
        }

        Self::deposit_event(Event::NFTHoldingsUpdated(
            holder_did,
            NFTs::new_unverified(asset_id, vec![nft_id]),
            Some(caller_holding),
            None,
            HoldingsUpdateReason::Redeemed,
        ));
        Ok(PostDispatchInfo::from(Some(
            <T as Config>::WeightInfo::redeem_nft(removed_keys as u32),
        )))
    }

    /// Tranfer ownership of all NFTs.
    #[require_transactional]
    pub fn base_nft_transfer(
        sender: AssetHolder,
        receiver: AssetHolder,
        nfts: NFTs,
        instruction_id: InstructionId,
        instruction_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Verifies if all rules for transfering the NFTs are being respected
        Self::validate_nft_transfer(&sender, &receiver, &nfts, false, Some(weight_meter))?;

        // Transfer ownership of the NFTs
        Self::unverified_nfts_transfer(&sender, receiver.clone(), &nfts)?;

        Self::deposit_event(Event::NFTHoldingsUpdated(
            caller_did,
            nfts,
            Some(sender),
            Some(receiver),
            HoldingsUpdateReason::Transferred {
                instruction_id: Some(instruction_id),
                instruction_memo,
            },
        ));
        Ok(())
    }

    /// Returns `Ok` if all rules for transferring the NFTs are satisfied.
    pub fn validate_nft_transfer(
        sender: &AssetHolder,
        receiver: &AssetHolder,
        nfts: &NFTs,
        is_controller_transfer: bool,
        weight_meter: Option<&mut WeightMeter>,
    ) -> DispatchResult {
        // Verifies if there is a collection associated to the NFTs
        if !CollectionAsset::<T>::contains_key(nfts.asset_id()) {
            return Err(Error::<T>::InvalidNFTTransferCollectionNotFound.into());
        }

        // Verifies that the sender and receiver are not the same
        let sender_did = pallet_identity::Pallet::<T>::asset_holder_did(sender)?;
        let receiver_did = pallet_identity::Pallet::<T>::asset_holder_did(receiver)?;
        ensure!(
            sender_did != receiver_did,
            Error::<T>::InvalidNFTTransferSenderDidMatchesReceiverDid
        );

        // Verifies that the sender has the required nft count
        let nfts_transferred = nfts.len() as u64;
        ensure!(
            NumberOfNFTs::<T>::get(nfts.asset_id(), sender_did) >= nfts_transferred,
            Error::<T>::InvalidNFTTransferInsufficientCount
        );

        // Verifies that the number of nfts being transferred are within the allowed limits
        Self::ensure_within_nfts_transfer_limits(nfts)?;
        // Verifies that all ids are unique
        Self::ensure_no_duplicate_nfts(nfts)?;
        // Verfies that the sender owns the nfts
        Self::ensure_nft_ownership(sender, nfts)?;

        // Verfies that the receiver will not overflow
        NumberOfNFTs::<T>::get(nfts.asset_id(), receiver_did)
            .checked_add(nfts_transferred)
            .ok_or(Error::<T>::InvalidNFTTransferCountOverflow)?;

        // Controllers are exempt from compliance and frozen rules.
        if is_controller_transfer {
            return Ok(());
        }

        pallet_asset::Pallet::<T>::ensure_asset_is_not_frozen(nfts.asset_id())?;

        // Verifies if the receiver has an active DID.
        ensure!(
            IdentityPallet::<T>::is_did_active(receiver_did),
            Error::<T>::InvalidNFTTransferInvalidReceiverDID
        );

        // Verifies that all compliance rules are being respected
        if !T::Compliance::is_compliant(
            nfts.asset_id(),
            sender_did,
            receiver_did,
            weight_meter.ok_or(Error::<T>::InvalidNFTTransferComplianceFailure)?,
        )? {
            return Err(Error::<T>::InvalidNFTTransferComplianceFailure.into());
        }

        Ok(())
    }

    /// Returns `Ok` if `sender` has all nfts and they are not locked. Otherwise, returns an `Err`.
    pub fn ensure_nft_ownership(sender: &AssetHolder, nfts: &NFTs) -> DispatchResult {
        for nft_id in nfts.ids() {
            ensure!(
                Self::is_holder_of_nft(nfts.asset_id(), nft_id, sender),
                Error::<T>::InvalidNFTTransferNFTNotOwned
            );
            ensure!(
                !Self::is_nft_locked(nfts.asset_id(), nft_id, sender),
                Error::<T>::InvalidNFTTransferNFTIsLocked
            );
        }

        Ok(())
    }

    /// Verifies that the number of NFTs being transferred is greater than zero and less or equal to `MaxNumberOfNFTsPerLeg`.
    pub fn ensure_within_nfts_transfer_limits(nfts: &NFTs) -> DispatchResult {
        ensure!(nfts.len() > 0, Error::<T>::ZeroCount);
        ensure!(
            nfts.len() <= (T::MaxNumberOfNFTsCount::get() as usize),
            Error::<T>::MaxNumberOfNFTsPerLegExceeded
        );
        Ok(())
    }

    /// Verifies that there are no duplicate ids in the `NFTs` struct.
    pub fn ensure_no_duplicate_nfts(nfts: &NFTs) -> DispatchResult {
        let unique_nfts: BTreeSet<&NFTId> = nfts.ids().iter().collect();
        ensure!(unique_nfts.len() == nfts.len(), Error::<T>::DuplicatedNFTId);
        Ok(())
    }

    /// Updates the storage for transferring all `nfts` from `sender` to `receiver`.
    fn unverified_nfts_transfer(
        sender: &AssetHolder,
        receiver: AssetHolder,
        nfts: &NFTs,
    ) -> DispatchResult {
        // Update the balance of the sender and the receiver
        let sender_did = pallet_identity::Pallet::<T>::asset_holder_did(sender)?;
        let receiver_did = pallet_identity::Pallet::<T>::asset_holder_did(&receiver)?;

        let transferred_amount = nfts.len() as u64;
        NumberOfNFTs::<T>::mutate(nfts.asset_id(), sender_did, |balance| {
            *balance = balance.saturating_sub(transferred_amount)
        });
        NumberOfNFTs::<T>::mutate(nfts.asset_id(), receiver_did, |balance| {
            *balance = balance.saturating_add(transferred_amount)
        });

        Self::transfer_holders_nfts(sender, receiver, nfts)?;

        Ok(())
    }

    pub fn transfer_holders_nfts(
        sender: &AssetHolder,
        receiver: AssetHolder,
        nfts: &NFTs,
    ) -> DispatchResult {
        for nft_id in nfts.ids() {
            Self::remove_nft_from_asset_holder(nfts.asset_id(), nft_id, sender)?;
            Self::add_nft_holding(nfts.asset_id().clone(), *nft_id, receiver.clone())?;
        }
        Ok(())
    }

    pub fn base_transfer_nft(
        origin: T::RuntimeOrigin,
        nfts: NFTs,
        to: T::AccountId,
        memo: Option<Memo>,
    ) -> DispatchResultWithPostInfo {
        let to_holder = AssetHolder::try_from(to.encode())
            .map_err(|_| pallet_base::Error::<T>::InvalidAccountId)?;
        let fund = Fund {
            description: FundDescription::NonFungible(nfts),
            memo,
        };

        let mut weight_meter = WeightMeter::from_limit_unchecked(
            Weight::zero(),
            <T as pallet_asset::Config>::SettlementFn::transfer_funds_weight_limit(None, &fund),
        );

        <T as pallet_asset::Config>::SettlementFn::transfer_funds(
            origin,
            None,
            to_holder,
            fund,
            &mut weight_meter,
            #[cfg(feature = "runtime-benchmarks")]
            false,
        )?;

        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }

    pub fn base_controller_transfer(
        origin: T::RuntimeOrigin,
        nfts: NFTs,
        source: AssetHolder,
        destination_kind: AssetHolderKind,
    ) -> DispatchResult {
        // Ensure origin is agent with custody and permissions for portfolio.
        let destination = AssetPallet::<T>::ensure_asset_and_holding_permissions(
            origin,
            *nfts.asset_id(),
            destination_kind,
            true,
        )?;
        let holder_did = pallet_identity::Pallet::<T>::asset_holder_did(&destination)?;

        // Verifies if all rules for transfering the NFTs are being respected
        Self::validate_nft_transfer(&source, &destination, &nfts, true, None)?;
        // Transfer ownership of the NFTs
        Self::unverified_nfts_transfer(&source, destination.clone(), &nfts)?;

        Self::deposit_event(Event::NFTHoldingsUpdated(
            holder_did,
            nfts,
            Some(source),
            Some(destination),
            HoldingsUpdateReason::ControllerTransfer,
        ));
        Ok(())
    }

    /// Returns a vector containing all errors for the transfer. An empty vec means there's no error.
    pub fn nft_transfer_report(
        sender: &AssetHolder,
        receiver: &AssetHolder,
        nfts: &NFTs,
        skip_locked_check: bool,
        weight_meter: &mut WeightMeter,
    ) -> Vec<DispatchError> {
        let mut nft_transfer_errors = Vec::new();

        // If the collection doesn't exist, there's no point in assessing anything else
        if !CollectionAsset::<T>::contains_key(nfts.asset_id()) {
            return vec![Error::<T>::InvalidNFTTransferCollectionNotFound.into()];
        }

        if Frozen::<T>::get(nfts.asset_id()) {
            nft_transfer_errors.push(Error::<T>::InvalidNFTTransferFrozenAsset.into());
        }

        let sender_did = {
            match pallet_identity::Pallet::<T>::asset_holder_did(sender) {
                Ok(did) => did,
                Err(e) => return vec![e.into()],
            }
        };

        let receiver_did = {
            match pallet_identity::Pallet::<T>::asset_holder_did(receiver) {
                Ok(did) => did,
                Err(e) => return vec![e.into()],
            }
        };

        if sender_did == receiver_did {
            nft_transfer_errors
                .push(Error::<T>::InvalidNFTTransferSenderDidMatchesReceiverDid.into());
        }

        let nfts_transferred = nfts.len() as u64;
        if NumberOfNFTs::<T>::get(nfts.asset_id(), &sender_did) < nfts_transferred {
            nft_transfer_errors.push(Error::<T>::InvalidNFTTransferInsufficientCount.into());
        }

        if let Err(e) = Self::ensure_within_nfts_transfer_limits(nfts) {
            nft_transfer_errors.push(e);
        }

        if let Err(e) = Self::ensure_no_duplicate_nfts(nfts) {
            nft_transfer_errors.push(e);
        }

        if skip_locked_check {
            for nft_id in nfts.ids() {
                if !Self::is_holder_of_nft(nfts.asset_id(), nft_id, sender) {
                    nft_transfer_errors.push(Error::<T>::InvalidNFTTransferNFTNotOwned.into());
                    break;
                }
            }
        } else {
            if let Err(e) = Self::ensure_nft_ownership(sender, nfts) {
                nft_transfer_errors.push(e);
            }
        }

        if !IdentityPallet::<T>::is_did_active(receiver_did) {
            nft_transfer_errors.push(Error::<T>::InvalidNFTTransferInvalidReceiverDID.into());
        }

        if NumberOfNFTs::<T>::get(nfts.asset_id(), &receiver_did)
            .checked_add(nfts_transferred)
            .is_none()
        {
            nft_transfer_errors.push(Error::<T>::InvalidNFTTransferCountOverflow.into());
        }

        match T::Compliance::is_compliant(nfts.asset_id(), sender_did, receiver_did, weight_meter) {
            Ok(is_compliant) => {
                if !is_compliant {
                    nft_transfer_errors
                        .push(Error::<T>::InvalidNFTTransferComplianceFailure.into());
                }
            }
            Err(e) => {
                nft_transfer_errors.push(e);
            }
        }

        nft_transfer_errors
    }

    /// Adds one to `CurrentCollectionId`.
    fn update_current_collection_id() -> Result<NFTCollectionId, DispatchError> {
        CurrentCollectionId::<T>::try_mutate(|current_collection_id| match current_collection_id {
            Some(current_id) => {
                let new_id = try_next_pre::<T, _>(current_id)?;
                *current_collection_id = Some(new_id);
                Ok::<NFTCollectionId, DispatchError>(new_id)
            }
            None => {
                let new_id = NFTCollectionId(1);
                *current_collection_id = Some(new_id);
                Ok::<NFTCollectionId, DispatchError>(new_id)
            }
        })
    }

    /// Adds one to the `NFTId` that belongs to `collection_id`.
    fn update_current_nft_id(collection_id: &NFTCollectionId) -> Result<NFTId, DispatchError> {
        CurrentNFTId::<T>::try_mutate(collection_id, |current_nft_id| match current_nft_id {
            Some(current_id) => {
                let new_nft_id = try_next_pre::<T, _>(current_id)?;
                *current_nft_id = Some(new_nft_id);
                Ok::<NFTId, DispatchError>(new_nft_id)
            }
            None => {
                let new_nft_id = NFTId(1);
                *current_nft_id = Some(new_nft_id);
                Ok::<NFTId, DispatchError>(new_nft_id)
            }
        })
    }

    /// Transfers all `nfts` from `sender_pid` to `receiver_pid`.
    /// Note: This functions skips all compliance checks and only checks for onwership.
    pub fn simplified_nft_transfer(
        sender: AssetHolder,
        receiver: AssetHolder,
        nfts: NFTs,
        inst_id: Option<InstructionId>,
        inst_memo: Option<Memo>,
        caller_did: IdentityId,
    ) -> DispatchResult {
        if let AssetHolder::Portfolio(receiver_pid) = &receiver {
            PortfolioPallet::<T>::ensure_portfolio_validity(receiver_pid)?;
        }

        Self::ensure_sender_owns_nfts(&sender, &nfts)?;
        Self::unverified_nfts_transfer(&sender, receiver.clone(), &nfts)?;

        Self::deposit_event(Event::NFTHoldingsUpdated(
            caller_did,
            nfts,
            Some(sender),
            Some(receiver),
            HoldingsUpdateReason::Transferred {
                instruction_id: inst_id,
                instruction_memo: inst_memo,
            },
        ));
        Ok(())
    }

    /// Returns `Ok` if `sender` holds all nfts.
    fn ensure_sender_owns_nfts(sender: &AssetHolder, nfts: &NFTs) -> DispatchResult {
        for nft_id in nfts.ids() {
            ensure!(
                Self::is_holder_of_nft(nfts.asset_id(), nft_id, sender),
                Error::<T>::InvalidNFTTransferNFTNotOwned
            );
        }
        Ok(())
    }

    /// If [`AssetHolder::Portfolio`], adds the NFT to the PortfolioNFT storage.
    /// If [`AssetHolder::Account`], adds the NFT to the NFTHolder storage.
    /// The [`Owner`] storage is updated in both cases.
    fn add_nft_holding(
        asset_id: AssetId,
        nft_id: NFTId,
        asset_owner: AssetHolder,
    ) -> DispatchResult {
        match asset_owner {
            AssetHolder::Account(ref acc_id) => {
                if !NFTHolder::<T>::contains_key((acc_id, &asset_id, &nft_id)) {
                    let acc_id = pallet_base::pallet_account_id::<T>(acc_id)?;
                    IdentityPallet::<T>::add_account_key_ref_count(&acc_id);
                }
                NFTHolder::<T>::insert((acc_id, asset_id, nft_id), NFTOwnerStatus::Owner);
            }
            AssetHolder::Portfolio(ref portfolio_id) => {
                PortfolioPallet::<T>::add_nft_to_portfolio(portfolio_id.clone(), asset_id, nft_id);
            }
        }
        Owner::<T>::insert(asset_id, nft_id, asset_owner);
        Ok(())
    }

    /// Removes the NFT from the current holder.
    fn remove_nft_from_asset_holder(
        asset_id: &AssetId,
        nft_id: &NFTId,
        asset_holder: &AssetHolder,
    ) -> DispatchResult {
        match asset_holder {
            AssetHolder::Account(acc_id) => {
                if NFTHolder::<T>::contains_key((acc_id, asset_id, nft_id)) {
                    let acc_id = pallet_base::pallet_account_id::<T>(&acc_id)?;
                    IdentityPallet::<T>::remove_account_key_ref_count(&acc_id);
                }
                NFTHolder::<T>::remove((acc_id, asset_id, nft_id));
            }
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::remove_nft_from_portfolio(portfolio_id, asset_id, nft_id);
            }
        }
        Owner::<T>::remove(asset_id, nft_id);
        Ok(())
    }

    /// Returns `true` if the `asset_holder` holds the given `nft_id` of the `asset_id`
    fn is_holder_of_nft(asset_id: &AssetId, nft_id: &NFTId, asset_holder: &AssetHolder) -> bool {
        match asset_holder {
            AssetHolder::Account(acc_id) => match NFTHolder::<T>::get((acc_id, asset_id, nft_id)) {
                NFTOwnerStatus::Owner | NFTOwnerStatus::OwnerLocked => true,
                NFTOwnerStatus::NotOwned => false,
            },
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::is_nft_owner(portfolio_id, asset_id, nft_id)
            }
        }
    }

    /// Returns `true` if the `nft_id` of the `asset_id` held by the `asset_holder` is locked.
    fn is_nft_locked(asset_id: &AssetId, nft_id: &NFTId, asset_holder: &AssetHolder) -> bool {
        match asset_holder {
            AssetHolder::Account(acc_id) => match NFTHolder::<T>::get((acc_id, asset_id, nft_id)) {
                NFTOwnerStatus::OwnerLocked => true,
                NFTOwnerStatus::Owner | NFTOwnerStatus::NotOwned => false,
            },
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::is_nft_locked(portfolio_id, asset_id, nft_id)
            }
        }
    }

    /// Locks the given NFT
    pub fn lock_nft(asset_holder: AssetHolder, asset_id: AssetId, nft_id: NFTId) -> DispatchResult {
        ensure!(
            Self::is_holder_of_nft(&asset_id, &nft_id, &asset_holder),
            Error::<T>::NFTNotFound
        );
        ensure!(
            !Self::is_nft_locked(&asset_id, &nft_id, &asset_holder),
            Error::<T>::NFTIsLocked
        );

        match asset_holder {
            AssetHolder::Account(acc_id) => {
                NFTHolder::<T>::insert((acc_id, asset_id, nft_id), NFTOwnerStatus::OwnerLocked);
            }
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::lock_nft(portfolio_id, asset_id, nft_id);
            }
        }

        Ok(())
    }

    /// Unlocks the given NFT.
    pub fn unlock_nft(
        asset_holder: &AssetHolder,
        asset_id: &AssetId,
        nft_id: &NFTId,
    ) -> DispatchResult {
        ensure!(
            Self::is_nft_locked(asset_id, nft_id, asset_holder),
            Error::<T>::NFTIsNotLocked
        );

        match asset_holder {
            AssetHolder::Account(acc_id) => {
                NFTHolder::<T>::insert((acc_id, asset_id, nft_id), NFTOwnerStatus::Owner);
            }
            AssetHolder::Portfolio(portfolio_id) => {
                PortfolioPallet::<T>::unlock_nft(portfolio_id, asset_id, nft_id);
            }
        }

        Ok(())
    }
}

impl<T: Config> NFTTrait<T::RuntimeOrigin> for Pallet<T> {
    fn is_collection_key(asset_id: &AssetId, metadata_key: &AssetMetadataKey) -> bool {
        match CollectionAsset::<T>::try_get(asset_id) {
            Ok(collection_id) => {
                let key_set = CollectionKeys::<T>::get(&collection_id);
                key_set.contains(metadata_key)
            }
            Err(_) => false,
        }
    }

    fn update_nft_owner(asset_id: AssetId, nft_id: NFTId, new_owner_portfolio: PortfolioId) {
        let asset_holder = AssetHolder::Portfolio(new_owner_portfolio);
        Owner::<T>::insert(asset_id, nft_id, asset_holder);
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn create_nft_collection(
        origin: T::RuntimeOrigin,
        asset_id: Option<AssetId>,
        nft_type: Option<NonFungibleType>,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult {
        Pallet::<T>::create_nft_collection(origin, asset_id, nft_type, collection_keys)
    }
}
