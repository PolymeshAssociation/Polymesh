#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_support::storage::StorageDoubleMap;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::{decl_error, decl_module, decl_storage, ensure, require_transactional};

use pallet_asset::Frozen;
use pallet_base::try_next_pre;
use pallet_portfolio::PortfolioNFT;
use polymesh_common_utilities::compliance_manager::ComplianceFnConfig;
pub use polymesh_common_utilities::traits::nft::{Config, Event, NFTTrait, WeightInfo};
use polymesh_primitives::asset::{AssetName, AssetType, NonFungibleType};
use polymesh_primitives::asset_metadata::{AssetMetadataKey, AssetMetadataValue};
use polymesh_primitives::nft::{
    NFTCollection, NFTCollectionId, NFTCollectionKeys, NFTCount, NFTId, NFTMetadataAttribute, NFTs,
};
use polymesh_primitives::settlement::InstructionId;
use polymesh_primitives::{
    storage_migrate_on, storage_migration_ver, IdentityId, Memo, PortfolioId, PortfolioKind,
    PortfolioUpdateReason, Ticker, WeightMeter,
};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::{vec, vec::Vec};

type Asset<T> = pallet_asset::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

storage_migration_ver!(1);

decl_storage!(
    trait Store for Module<T: Config> as NFT {
        /// The total number of NFTs per identity.
        pub NumberOfNFTs get(fn balance_of): double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => NFTCount;

        /// The collection id corresponding to each ticker.
        pub CollectionTicker get(fn collection_ticker): map hasher(blake2_128_concat) Ticker => NFTCollectionId;

        /// All collection details for a given collection id.
        pub Collection get(fn nft_collection): map hasher(blake2_128_concat) NFTCollectionId => NFTCollection;

        /// All mandatory metadata keys for a given collection.
        pub CollectionKeys get(fn collection_keys): map hasher(blake2_128_concat) NFTCollectionId => BTreeSet<AssetMetadataKey>;

        /// The metadata value of an nft given its collection id, token id and metadata key.
        pub MetadataValue get(fn metadata_value): double_map hasher(blake2_128_concat) (NFTCollectionId, NFTId), hasher(blake2_128_concat) AssetMetadataKey => AssetMetadataValue;

        /// The next available id for an NFT collection.
        pub NextCollectionId get(fn collection_id): NFTCollectionId;

        /// The next available id for an NFT within a collection.
        pub NextNFTId get(fn nft_id): map hasher(blake2_128_concat) NFTCollectionId => NFTId;

        /// The total number of NFTs in a collection
        pub NFTsInCollection get(fn nfts_in_collection): map hasher(blake2_128_concat) Ticker => NFTCount;

        /// Tracks the owner of an NFT
        pub NFTOwner get(fn nft_owner): double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) NFTId => Option<PortfolioId>;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1)): Version;
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {

        type Error = Error<T>;

        const MaxNumberOfCollectionKeys: u8 = T::MaxNumberOfCollectionKeys::get();
        const MaxNumberOfNFTsCount: u32 = T::MaxNumberOfNFTsCount::get();

        /// Initializes the default event for this module.
        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            storage_migrate_on!(StorageVersion, 1, {
                migration::migrate_to_v1::<T>();
            });
            Weight::zero()
        }

        /// Cretes a new `NFTCollection`.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `ticker` - the ticker associated to the new collection.
        /// * `nft_type` - in case the asset hasn't been created yet, one will be created with the given type.
        /// * `collection_keys` - all mandatory metadata keys that the tokens in the collection must have.
        ///
        /// ## Errors
        /// - `CollectionAlredyRegistered` - if the ticker is already associated to an NFT collection.
        /// - `InvalidAssetType` - if the associated asset is not of type NFT.
        /// - `MaxNumberOfKeysExceeded` - if the number of metadata keys for the collection is greater than the maximum allowed.
        /// - `UnregisteredMetadataKey` - if any of the metadata keys needed for the collection has not been registered.
        /// - `DuplicateMetadataKey` - if a duplicate metadata keys has been passed as input.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::create_nft_collection(collection_keys.len() as u32)]
        pub fn create_nft_collection(origin, ticker: Ticker, nft_type: Option<NonFungibleType>, collection_keys: NFTCollectionKeys) -> DispatchResult {
            Self::base_create_nft_collection(origin, ticker, nft_type, collection_keys)
        }

        /// Issues an NFT to the caller.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` - the ticker of the NFT collection.
        /// * `nft_metadata_attributes` - all mandatory metadata keys and values for the NFT.
        /// - `portfolio_kind` - the portfolio that will receive the minted nft.
        ///
        /// ## Errors
        /// - `CollectionNotFound` - if the collection associated to the given ticker has not been created.
        /// - `InvalidMetadataAttribute` - if the number of attributes is not equal to the number set in the collection or attempting to set a value for a key not definied in the collection.
        /// - `DuplicateMetadataKey` - if a duplicate metadata keys has been passed as input.
        ///
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::issue_nft(nft_metadata_attributes.len() as u32)]
        pub fn issue_nft(origin, ticker: Ticker, nft_metadata_attributes: Vec<NFTMetadataAttribute>, portfolio_kind: PortfolioKind) -> DispatchResult {
            Self::base_issue_nft(origin, ticker, nft_metadata_attributes, portfolio_kind)
        }

        /// Redeems the given NFT from the caller's portfolio.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` - the ticker of the NFT collection.
        /// * `nft_id` - the id of the NFT to be burned.
        /// * `portfolio_kind` - the portfolio that contains the nft.
        ///
        /// ## Errors
        /// - `CollectionNotFound` - if the collection associated to the given ticker has not been created.
        /// - `NFTNotFound` - if the given NFT does not exist in the portfolio.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::redeem_nft(T::MaxNumberOfCollectionKeys::get() as u32)]
        pub fn redeem_nft(origin, ticker: Ticker, nft_id: NFTId, portfolio_kind: PortfolioKind) -> DispatchResult {
            Self::base_redeem_nft(origin, ticker, nft_id, portfolio_kind)
        }

        /// Forces the transfer of NFTs from a given portfolio to the caller's portfolio.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` - the [`Ticker`] of the NFT collection.
        /// * `nft_id` - the [`NFTId`] of the NFT to be transferred.
        /// * `source_portfolio` - the [`PortfolioId`] that currently holds the NFT.
        /// * `callers_portfolio_kind` - the [`PortfolioKind`] of the caller's portfolio.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::controller_transfer(nfts.len() as u32)]
        pub fn controller_transfer(
            origin,
            ticker: Ticker,
            nfts: NFTs,
            source_portfolio: PortfolioId,
            callers_portfolio_kind: PortfolioKind
        ) -> DispatchResult {
            Self::base_controller_transfer(origin, ticker, nfts, source_portfolio, callers_portfolio_kind)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// An overflow while calculating the balance.
        BalanceOverflow,
        /// An underflow while calculating the balance.
        BalanceUnderflow,
        /// The ticker is already associated to an NFT collection.
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
        SupplyUnderflow
    }
}

impl<T: Config> Module<T> {
    fn base_create_nft_collection(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        nft_type: Option<NonFungibleType>,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult {
        // Verifies if the asset has already been created and the caller's permission to create the collection
        let (create_asset, caller_did) = {
            match Asset::<T>::nft_asset(&ticker) {
                Some(is_nft_asset) => {
                    ensure!(is_nft_asset, Error::<T>::InvalidAssetType);
                    let caller_did =
                        <ExternalAgents<T>>::ensure_agent_asset_perms(origin.clone(), ticker)?
                            .primary_did;
                    (false, caller_did)
                }
                None => {
                    let caller_did = Identity::<T>::ensure_perms(origin.clone())?;
                    (true, caller_did)
                }
            }
        };

        // Verifies if the ticker is already associated to an NFT collection
        ensure!(
            !CollectionTicker::contains_key(&ticker),
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
                Asset::<T>::check_asset_metadata_key_exists(&ticker, key),
                Error::<T>::UnregisteredMetadataKey
            )
        }

        // Creates an nft asset if it hasn't been created yet
        if create_asset {
            let nft_type = nft_type.ok_or(Error::<T>::InvalidAssetType)?;
            Asset::<T>::create_asset(
                origin,
                AssetName(ticker.as_slice().to_vec()),
                ticker.clone(),
                false,
                AssetType::NonFungible(nft_type),
                Vec::new(),
                None,
            )?;
        }

        // Creates the nft collection
        let collection_id = NextCollectionId::try_mutate(try_next_pre::<T, _>)?;
        let nft_collection = NFTCollection::new(collection_id, ticker.clone());
        Collection::insert(&collection_id, nft_collection);
        CollectionKeys::insert(&collection_id, collection_keys);
        CollectionTicker::insert(&ticker, &collection_id);

        Self::deposit_event(Event::NftCollectionCreated(
            caller_did,
            ticker,
            collection_id,
        ));
        Ok(())
    }

    fn base_issue_nft(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        metadata_attributes: Vec<NFTMetadataAttribute>,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        // Verifies if the collection exists
        let collection_id =
            CollectionTicker::try_get(&ticker).map_err(|_| Error::<T>::CollectionNotFound)?;

        // Verifies if the caller has the right permissions (regarding asset and portfolio)
        let caller_portfolio = Asset::<T>::ensure_origin_ticker_and_portfolio_permissions(
            origin,
            ticker.clone(),
            portfolio_kind,
            false,
        )?;

        Portfolio::<T>::ensure_portfolio_validity(&caller_portfolio)?;

        // Verifies that all mandatory keys are being set and that there are no duplicated keys
        let mandatory_keys: BTreeSet<AssetMetadataKey> = Self::collection_keys(&collection_id);
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
        let new_supply = NFTsInCollection::get(&ticker)
            .checked_add(1)
            .ok_or(Error::<T>::SupplyOverflow)?;
        let new_balance = NumberOfNFTs::get(&ticker, &caller_portfolio.did)
            .checked_add(1)
            .ok_or(Error::<T>::BalanceOverflow)?;
        let nft_id = NextNFTId::try_mutate(&collection_id, try_next_pre::<T, _>)?;
        NFTsInCollection::insert(&ticker, new_supply);
        NumberOfNFTs::insert(&ticker, &caller_portfolio.did, new_balance);
        for (metadata_key, metadata_value) in nft_attributes.into_iter() {
            MetadataValue::insert((&collection_id, &nft_id), metadata_key, metadata_value);
        }
        PortfolioNFT::insert(caller_portfolio, (ticker, nft_id), true);
        NFTOwner::insert(ticker, nft_id, caller_portfolio);

        Self::deposit_event(Event::NFTPortfolioUpdated(
            caller_portfolio.did,
            NFTs::new_unverified(ticker, vec![nft_id]),
            None,
            Some(caller_portfolio),
            PortfolioUpdateReason::Issued {
                funding_round_name: None,
            },
        ));
        Ok(())
    }

    fn base_redeem_nft(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        nft_id: NFTId,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        // Verifies if the collection exists
        let collection_id =
            CollectionTicker::try_get(&ticker).map_err(|_| Error::<T>::CollectionNotFound)?;

        // Ensure origin is agent with custody and permissions for portfolio.
        let caller_portfolio = Asset::<T>::ensure_origin_ticker_and_portfolio_permissions(
            origin,
            ticker,
            portfolio_kind,
            true,
        )?;

        // Verifies if the NFT exists
        ensure!(
            PortfolioNFT::contains_key(&caller_portfolio, (&ticker, &nft_id)),
            Error::<T>::NFTNotFound
        );

        // Burns the NFT
        let new_supply = NFTsInCollection::get(&ticker)
            .checked_sub(1)
            .ok_or(Error::<T>::SupplyUnderflow)?;
        let new_balance = NumberOfNFTs::get(&ticker, &caller_portfolio.did)
            .checked_sub(1)
            .ok_or(Error::<T>::BalanceUnderflow)?;
        NFTsInCollection::insert(&ticker, new_supply);
        NumberOfNFTs::insert(&ticker, &caller_portfolio.did, new_balance);
        PortfolioNFT::remove(&caller_portfolio, (&ticker, &nft_id));
        #[allow(deprecated)]
        MetadataValue::remove_prefix((&collection_id, &nft_id), None);
        NFTOwner::remove(ticker, nft_id);

        Self::deposit_event(Event::NFTPortfolioUpdated(
            caller_portfolio.did,
            NFTs::new_unverified(ticker, vec![nft_id]),
            Some(caller_portfolio),
            None,
            PortfolioUpdateReason::Redeemed,
        ));
        Ok(())
    }

    /// Tranfer ownership of all NFTs.
    #[require_transactional]
    pub fn base_nft_transfer(
        sender_portfolio: PortfolioId,
        receiver_portfolio: PortfolioId,
        nfts: NFTs,
        instruction_id: InstructionId,
        instruction_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Verifies if all rules for transfering the NFTs are being respected
        Self::validate_nft_transfer(&sender_portfolio, &receiver_portfolio, &nfts, weight_meter)?;

        // Transfer ownership of the NFTs
        Self::unverified_nfts_transfer(&sender_portfolio, &receiver_portfolio, &nfts);

        Self::deposit_event(Event::NFTPortfolioUpdated(
            caller_did,
            nfts,
            Some(sender_portfolio),
            Some(receiver_portfolio),
            PortfolioUpdateReason::Transferred {
                instruction_id: Some(instruction_id),
                instruction_memo,
            },
        ));
        Ok(())
    }

    /// Returns `Ok` if the asset is not frozen, if `sender_portfolio` owns all `nfts`, all arithmetic updates succeed,
    /// if `sender_portfolio` is different from `receiver_portfolio`, and if all compliance rules are being respected.
    pub fn validate_nft_transfer(
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        nfts: &NFTs,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Verifies that the asset is not frozen
        ensure!(
            !Frozen::get(nfts.ticker()),
            Error::<T>::InvalidNFTTransferFrozenAsset
        );
        // Verifies that the sender_portfolio owns all nfts being transferred
        Self::validate_nft_ownership(sender_portfolio, receiver_portfolio, nfts)?;
        // Verifies that all compliance rules are being respected
        if !T::Compliance::is_compliant(
            nfts.ticker(),
            sender_portfolio.did,
            receiver_portfolio.did,
            weight_meter,
        )? {
            return Err(Error::<T>::InvalidNFTTransferComplianceFailure.into());
        }

        Ok(())
    }

    /// Returns `Ok` if `sender_portfolio` owns all `nfts`, all arithmetic updates succeed, and if `sender_portfolio`
    /// is different from `receiver_portfolio`.
    fn validate_nft_ownership(
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        nfts: &NFTs,
    ) -> DispatchResult {
        // Verifies if there is a collection associated to the NFTs
        CollectionTicker::try_get(nfts.ticker())
            .map_err(|_| Error::<T>::InvalidNFTTransferCollectionNotFound)?;
        // Verifies that the sender and receiver are not the same
        ensure!(
            sender_portfolio != receiver_portfolio,
            Error::<T>::InvalidNFTTransferSamePortfolio
        );
        // Verifies that the sender has the required nft count
        let nfts_transferred = nfts.len() as u64;
        ensure!(
            NumberOfNFTs::get(nfts.ticker(), sender_portfolio.did) >= nfts_transferred,
            Error::<T>::InvalidNFTTransferInsufficientCount
        );
        // Verifies that the number of nfts being transferred are within the allowed limits
        Self::ensure_within_nfts_transfer_limits(nfts)?;
        // Verifies that all ids are unique
        Self::ensure_no_duplicate_nfts(nfts)?;
        // Verfies that the sender owns the nfts
        for nft_id in nfts.ids() {
            ensure!(
                PortfolioNFT::contains_key(sender_portfolio, (nfts.ticker(), nft_id)),
                Error::<T>::InvalidNFTTransferNFTNotOwned
            );
        }
        // Verfies that the receiver will not overflow
        NumberOfNFTs::get(nfts.ticker(), receiver_portfolio.did)
            .checked_add(nfts_transferred)
            .ok_or(Error::<T>::InvalidNFTTransferCountOverflow)?;
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

    /// Updates the storage for transferring all `nfts` from `sender_portfolio` to `receiver_portfolio`.
    fn unverified_nfts_transfer(
        sender_portfolio: &PortfolioId,
        receiver_portfolio: &PortfolioId,
        nfts: &NFTs,
    ) {
        // Update the balance of the sender and the receiver
        let transferred_amount = nfts.len() as u64;
        NumberOfNFTs::mutate(nfts.ticker(), sender_portfolio.did, |balance| {
            *balance -= transferred_amount
        });
        NumberOfNFTs::mutate(nfts.ticker(), receiver_portfolio.did, |balance| {
            *balance += transferred_amount
        });
        // Update the portfolio of the sender and the receiver
        for nft_id in nfts.ids() {
            PortfolioNFT::remove(sender_portfolio, (nfts.ticker(), nft_id));
            PortfolioNFT::insert(receiver_portfolio, (nfts.ticker(), nft_id), true);
            NFTOwner::insert(nfts.ticker(), nft_id, receiver_portfolio);
        }
    }

    pub fn base_controller_transfer(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        nfts: NFTs,
        source_portfolio: PortfolioId,
        callers_portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        // Ensure origin is agent with custody and permissions for portfolio.
        let caller_portfolio = Asset::<T>::ensure_origin_ticker_and_portfolio_permissions(
            origin,
            ticker,
            callers_portfolio_kind,
            true,
        )?;
        // Verifies if all rules for transfering the NFTs are being respected
        Self::validate_nft_ownership(&source_portfolio, &caller_portfolio, &nfts)?;
        // Transfer ownership of the NFTs
        Self::unverified_nfts_transfer(&source_portfolio, &caller_portfolio, &nfts);

        Self::deposit_event(Event::NFTPortfolioUpdated(
            caller_portfolio.did,
            nfts,
            Some(source_portfolio),
            Some(caller_portfolio),
            PortfolioUpdateReason::ControllerTransfer,
        ));
        Ok(())
    }
}

impl<T: Config> NFTTrait<T::RuntimeOrigin> for Module<T> {
    fn is_collection_key(ticker: &Ticker, metadata_key: &AssetMetadataKey) -> bool {
        match CollectionTicker::try_get(ticker) {
            Ok(collection_id) => {
                let key_set = CollectionKeys::get(&collection_id);
                key_set.contains(metadata_key)
            }
            Err(_) => false,
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn create_nft_collection(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        nft_type: Option<NonFungibleType>,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult {
        Module::<T>::create_nft_collection(origin, ticker, nft_type, collection_keys)
    }
}

pub mod migration {
    use crate::sp_api_hidden_includes_decl_storage::hidden_include::IterableStorageDoubleMap;
    use crate::{Config, NFTOwner, NFTsInCollection, NumberOfNFTs};
    use frame_support::storage::{StorageDoubleMap, StorageMap};
    use pallet_portfolio::PortfolioNFT;
    use sp_runtime::runtime_logger::RuntimeLogger;

    pub fn migrate_to_v1<T: Config>() {
        RuntimeLogger::init();
        log::info!(">>> Initializing NFTsInCollection and NFTOwner Storage");
        initialize_nfts_in_collection::<T>();
        initialize_nft_owner::<T>();
        log::info!(">>> NFTsInCollection and NFTOwner were successfully updated");
    }

    fn initialize_nfts_in_collection<T: Config>() {
        for (ticker, _, id_count) in NumberOfNFTs::iter() {
            NFTsInCollection::mutate(ticker, |collection_count| *collection_count += id_count);
        }
    }

    fn initialize_nft_owner<T: Config>() {
        for (portfolio_id, (ticker, nft_id), _) in PortfolioNFT::iter() {
            NFTOwner::insert(ticker, nft_id, portfolio_id);
        }
    }
}
