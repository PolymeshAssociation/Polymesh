#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_support::traits::Get;
use frame_support::{decl_error, decl_module, decl_storage};
use pallet_base::try_next_pre;
pub use polymesh_common_utilities::traits::nft::{Config, Event, WeightInfo};
use polymesh_primitives::asset_metadata::{AssetMetadataKey, AssetMetadataValue};
use polymesh_primitives::nft::{
    NFTCollection, NFTCollectionId, NFTCollectionKeys, NFTId, NFTMetadataAttribute,
};
use polymesh_primitives::{storage_migration_ver, Ticker};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec::Vec;

type Asset<T> = pallet_asset::Module<T>;
type Identity<T> = pallet_identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

storage_migration_ver!(1);

decl_storage!(
    trait Store for Module<T: Config> as NFT {
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
        /// - `UnregisteredTicker` - if the ticker associated to the collection has not been registered.
        /// - `MaxNumberOfKeysExceeded` - if the number of metadata keys for the collection is greater than the maximum allowed.
        /// - `UnregisteredMetadataKey` - if any of the metadata keys needed for the collection has not been registered.
        #[weight = <T as Config>::WeightInfo::create_nft_collection()]
        pub fn create_nft_collection(origin, ticker: Ticker, collection_keys: NFTCollectionKeys) -> DispatchResult {
            Self::base_create_nft_collection(origin, ticker, collection_keys)
        }

        /// Mints an NFT to the caller.
        ///
        /// # Arguments
        /// * `origin` - is a signer that has permissions to act as an agent of `ticker`.
        /// * `nft_collection_id` - the id of the NFT collection .
        /// * `nft_metadata_attributes` - all mandatory metadata keys and values for the NFT.
        ///
        /// ## Errors
        /// - `CollectionNotFound` - if the collection associated to the given ticker has not been created.
        /// - `InvalidMetadataAttribute` - if the number of attributes is not equal to the number set in the collection or attempting to set a value for a key not definied in the collection.
        #[weight = <T as Config>::WeightInfo::mint_nft()]
        pub fn mint_nft(origin, nft_collection_id: NFTCollectionId, nft_metadata_attributes: Vec<NFTMetadataAttribute>) -> DispatchResult {
            Self::base_mint_nft(origin, nft_collection_id, nft_metadata_attributes)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The NFT collection does not exist.
        CollectionNotFound,
        /// A duplicate metadata key has been passed as parameter.
        DuplicateMetadataKey,
        /// Either the number of keys or the key identifier does not match the keys defined for the collection.
        InvalidMetadataAttribute,
        /// The maximum number of metadata keys was exceeded.
        MaxNumberOfKeysExceeded,
        /// The caller does not have the permissions for the requested operation.
        Unauthorized,
        /// At least one of the metadata keys has not been registered.
        UnregisteredMetadataKey,
        /// The ticker has not been registered.
        UnregisteredTicker,
    }
}

impl<T: Config> Module<T> {
    fn base_create_nft_collection(
        origin: T::Origin,
        ticker: Ticker,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult {
        // Verifies if the caller has the right permissions to create the collection
        let _did = Identity::<T>::ensure_perms(origin)?;

        // Verifies if the ticker has already been registered
        if !Asset::<T>::is_registered_ticker(&ticker) {
            return Err(Error::<T>::UnregisteredTicker.into());
        }

        // Verifies if the maximum number of keys is respected and that all keys have been registered
        if collection_keys.len() > T::MaxNumberOfCollectionKeys::get() as usize {
            return Err(Error::<T>::MaxNumberOfKeysExceeded.into());
        }

        // Returns an error in case a duplicated key is found
        let n_keys = collection_keys.len();
        let collection_keys: BTreeSet<AssetMetadataKey> = collection_keys.into_iter().collect();
        if n_keys != collection_keys.len() {
            return Err(Error::<T>::DuplicateMetadataKey.into());
        }

        for key in &collection_keys {
            if !Asset::<T>::check_asset_metadata_key_exists(&ticker, key) {
                return Err(Error::<T>::UnregisteredMetadataKey.into());
            }
        }

        // Creates the nft collection
        let collection_id = NextCollectionId::try_mutate(try_next_pre::<T, _>)?;
        let nft_collection = NFTCollection::new(collection_id.clone(), ticker.clone());
        Collection::insert(collection_id.clone(), nft_collection);
        CollectionKeys::insert(collection_id.clone(), collection_keys);

        Self::deposit_event(Event::NftCollectionCreated(ticker, collection_id));
        Ok(())
    }

    fn base_mint_nft(
        origin: T::Origin,
        collection_id: NFTCollectionId,
        metadata_attributes: Vec<NFTMetadataAttribute>,
    ) -> DispatchResult {
        // Verifies if the collection exists
        let collection =
            Collection::try_get(&collection_id).map_err(|_| Error::<T>::CollectionNotFound)?;

        // Verifies if the caller has the right permissions (regarding asset and portfolio)
        let caller_did =
            Asset::<T>::ensure_agent_with_custody_and_perms(origin, collection.ticker().clone())?;

        // Returns an error in case a duplicated key is found
        let mut nft_attributes = BTreeMap::new();
        for metadata_attribute in metadata_attributes {
            if nft_attributes
                .insert(metadata_attribute.key, metadata_attribute.value)
                .is_some()
            {
                return Err(Error::<T>::DuplicateMetadataKey.into());
            }
        }

        // Verifies if all mandatory metadata keys defined in the collection are being set
        let mandatory_keys: BTreeSet<AssetMetadataKey> = Self::collection_keys(&collection_id);
        if nft_attributes.len() != mandatory_keys.len() {
            return Err(Error::<T>::InvalidMetadataAttribute.into());
        }
        for metadata_key in nft_attributes.keys() {
            if !mandatory_keys.contains(metadata_key) {
                return Err(Error::<T>::InvalidMetadataAttribute.into());
            }
        }

        // Mints the NFT and adds it to the caller's portfolio
        let nft_id = NextNFTId::try_mutate(&collection_id, try_next_pre::<T, _>)?;
        for (metadata_key, metadata_value) in nft_attributes.into_iter() {
            MetadataValue::insert(
                (collection_id.clone(), nft_id.clone()),
                metadata_key,
                metadata_value,
            );
        }
        Portfolio::<T>::add_portfolio_nft(caller_did, collection_id.clone(), nft_id.clone());

        Self::deposit_event(Event::MintedNft(collection_id, nft_id));
        Ok(())
    }
}
