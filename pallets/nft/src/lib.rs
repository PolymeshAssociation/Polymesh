#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
use frame_support::ensure;
use frame_support::traits::Get;
use frame_support::{decl_error, decl_module, decl_storage};
use pallet_asset::BalanceOf;
use pallet_base::try_next_pre;
use pallet_portfolio::PortfolioNFT;
use polymesh_common_utilities::constants::currency::ONE_UNIT;
pub use polymesh_common_utilities::traits::nft::{Config, Event, WeightInfo};
use polymesh_primitives::asset::{AssetName, AssetType};
use polymesh_primitives::asset_metadata::{AssetMetadataKey, AssetMetadataValue};
use polymesh_primitives::nft::{
    NFTCollection, NFTCollectionId, NFTCollectionKeys, NFTId, NFTMetadataAttribute,
};
use polymesh_primitives::{PortfolioKind, Ticker};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec::Vec;

type Asset<T> = pallet_asset::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

decl_storage!(
    trait Store for Module<T: Config> as NFT {
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
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        const MaxNumberOfCollectionKeys: u8 = T::MaxNumberOfCollectionKeys::get();

        /// Initializes the default event for this module.
        fn deposit_event() = default;

        /// Cretes a new `NFTCollection`.
        ///
        /// # Arguments
        /// * `origin` - contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `ticker` - the ticker associated to the new collection.
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
        pub fn create_nft_collection(origin, ticker: Ticker, collection_keys: NFTCollectionKeys) -> DispatchResult {
            Self::base_create_nft_collection(origin, ticker, collection_keys)
        }

        /// Mints an NFT to the caller.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `ticker`.
        /// * `ticker` - the ticker of the NFT collection.
        /// * `nft_metadata_attributes` - all mandatory metadata keys and values for the NFT.
        ///
        /// ## Errors
        /// - `CollectionNotFound` - if the collection associated to the given ticker has not been created.
        /// - `InvalidMetadataAttribute` - if the number of attributes is not equal to the number set in the collection or attempting to set a value for a key not definied in the collection.
        /// - `DuplicateMetadataKey` - if a duplicate metadata keys has been passed as input.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::mint_nft(nft_metadata_attributes.len() as u32)]
        pub fn mint_nft(origin, ticker: Ticker, nft_metadata_attributes: Vec<NFTMetadataAttribute>) -> DispatchResult {
            Self::base_mint_nft(origin, ticker, nft_metadata_attributes)
        }

        /// Burns the given NFT from the caller's portfolio.
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
        #[weight = <T as Config>::WeightInfo::burn_nft()]
        pub fn burn_nft(origin, ticker: Ticker, nft_id: NFTId, portfolio_kind: PortfolioKind) -> DispatchResult {
            Self::base_burn_nft(origin, ticker, nft_id, portfolio_kind)
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
        /// The associated asset is not of type NFT.
        InvalidAssetType,
        /// Either the number of keys or the key identifier does not match the keys defined for the collection.
        InvalidMetadataAttribute,
        /// The maximum number of metadata keys was exceeded.
        MaxNumberOfKeysExceeded,
        /// The NFT does not exist.
        NFTNotFound,
        /// At least one of the metadata keys has not been registered.
        UnregisteredMetadataKey,
    }
}

impl<T: Config> Module<T> {
    fn base_create_nft_collection(
        origin: T::Origin,
        ticker: Ticker,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult {
        // Verifies if the caller has the right permissions to create the collection
        let caller_did = Identity::<T>::ensure_perms(origin.clone())?;

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

        // Verifies if the asset is of type NFT or creates an nft asset if it does not exist
        match Asset::<T>::nft_asset(&ticker) {
            Some(is_nft_asset) => {
                ensure!(is_nft_asset, Error::<T>::InvalidAssetType);
                <ExternalAgents<T>>::ensure_agent_asset_perms(origin, ticker)?;
            }
            None => Asset::<T>::create_asset(
                origin,
                AssetName(ticker.as_slice().to_vec()),
                ticker.clone(),
                false,
                AssetType::NFT,
                Vec::new(),
                None,
                false,
            )?,
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

    fn base_mint_nft(
        origin: T::Origin,
        ticker: Ticker,
        metadata_attributes: Vec<NFTMetadataAttribute>,
    ) -> DispatchResult {
        // Verifies if the collection exists
        let collection_id =
            CollectionTicker::try_get(&ticker).map_err(|_| Error::<T>::CollectionNotFound)?;

        // Verifies if the caller has the right permissions (regarding asset and portfolio)
        let caller_portfolio = Asset::<T>::ensure_agent_with_custody_and_perms(
            origin,
            ticker.clone(),
            PortfolioKind::Default,
        )?;

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
        let new_balance = BalanceOf::get(&ticker, &caller_portfolio.did)
            .checked_add(ONE_UNIT)
            .ok_or(Error::<T>::BalanceOverflow)?;
        let nft_id = NextNFTId::try_mutate(&collection_id, try_next_pre::<T, _>)?;
        BalanceOf::insert(&ticker, &caller_portfolio.did, new_balance);
        for (metadata_key, metadata_value) in nft_attributes.into_iter() {
            MetadataValue::insert((&collection_id, &nft_id), metadata_key, metadata_value);
        }
        Portfolio::<T>::add_portfolio_nft(caller_portfolio.did, collection_id, nft_id);

        Self::deposit_event(Event::MintedNft(
            caller_portfolio.did,
            collection_id,
            nft_id,
        ));
        Ok(())
    }

    fn base_burn_nft(
        origin: T::Origin,
        ticker: Ticker,
        nft_id: NFTId,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        // Verifies if the collection exists
        let collection_id =
            CollectionTicker::try_get(&ticker).map_err(|_| Error::<T>::CollectionNotFound)?;

        // Ensure origin is agent with custody and permissions for default portfolio.
        let caller_portfolio =
            Asset::<T>::ensure_agent_with_custody_and_perms(origin, ticker, portfolio_kind)?;

        // Verifies if the NFT exists
        ensure!(
            PortfolioNFT::contains_key(&caller_portfolio, (&collection_id, &nft_id)),
            Error::<T>::NFTNotFound
        );

        // Burns the NFT
        let new_balance = BalanceOf::get(&ticker, &caller_portfolio.did)
            .checked_sub(ONE_UNIT)
            .ok_or(Error::<T>::BalanceUnderflow)?;
        BalanceOf::insert(&ticker, &caller_portfolio.did, new_balance);
        PortfolioNFT::remove(&caller_portfolio, (&collection_id, &nft_id));
        MetadataValue::remove_prefix((&collection_id, &nft_id), None);

        Self::deposit_event(Event::BurnedNFT(caller_portfolio.did, ticker, nft_id));
        Ok(())
    }
}
