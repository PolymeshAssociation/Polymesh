#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
use frame_support::traits::Get;
use frame_support::{decl_error, decl_module, decl_storage};
use pallet_base::try_next_pre;
use polymesh_common_utilities::traits::asset::AssetFnTrait;
pub use polymesh_common_utilities::traits::nft::{Config, Event, WeightInfo};
use polymesh_primitives::asset_metadata::{AssetMetadataKey, AssetMetadataValue};
use polymesh_primitives::nft::{
    NFTCollection, NFTCollectionId, NFTCollectionKeys, NFTId, NFTMetadataAttribute,
};
use polymesh_primitives::Ticker;
use sp_std::vec::Vec;

decl_storage!(
    trait Store for Module<T: Config> as NFT {
        /// All collection details for a given collection id.
        pub Collection get(fn nft_collection): map hasher(blake2_128_concat) NFTCollectionId => NFTCollection;

        /// All mandatory metadata keys for a given collection.
        pub CollectionKeys get(fn collection_keys): map hasher(blake2_128_concat) NFTCollectionId => NFTCollectionKeys;

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
        /// * `nft_collection_id` - the ticker associated to the new collection.
        /// * `nft_attributes` - all mandatory metadata keys and values for the nft.
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
        CollectionNotFound,
        InvalidMetadataAttribute,
        MaxNumberOfKeysExceeded,
        Unauthorized,
        UnregisteredMetadataKey,
        UnregisteredTicker,
    }
}

impl<T: Config> Module<T> {
    fn base_create_nft_collection(
        _origin: T::Origin,
        ticker: Ticker,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult {
        // Verifies if the caller has the right permissions to create the collection

        // Verifies if the ticker has already been registered
        if !T::Asset::is_registered_ticker(&ticker) {
            return Err(Error::<T>::UnregisteredTicker.into());
        }

        // Verifies if the maximum number of keys is respected and that all keys have been registered
        if collection_keys.len() > T::MaxNumberOfCollectionKeys::get() as usize {
            return Err(Error::<T>::MaxNumberOfKeysExceeded.into());
        }

        for key in collection_keys.metadata_keys() {
            if !T::Asset::is_registered_metadata_key(&ticker, key) {
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
        _origin: T::Origin,
        collection_id: NFTCollectionId,
        metadata_attributes: Vec<NFTMetadataAttribute>,
    ) -> DispatchResult {
        // Verifies if the caller has the right permissions

        // Verifies if the collection exists
        if !Collection::contains_key(&collection_id) {
            return Err(Error::<T>::CollectionNotFound.into());
        }

        // Verifies if the metadata atributes are respecting the set defined in the collection
        let mandatory_keys = Self::collection_keys(&collection_id);
        if metadata_attributes.len() != mandatory_keys.len() {
            return Err(Error::<T>::InvalidMetadataAttribute.into());
        }
        for metadata_attribute in &metadata_attributes {
            if !mandatory_keys.contains(&metadata_attribute.key) {
                return Err(Error::<T>::InvalidMetadataAttribute.into());
            }
        }

        // Mints the NFT
        let nft_id = NextNFTId::try_mutate(&collection_id, try_next_pre::<T, _>)?;
        for metadata_attribute in metadata_attributes {
            MetadataValue::insert(
                (collection_id.clone(), nft_id.clone()),
                metadata_attribute.key,
                metadata_attribute.value,
            );
        }

        Self::deposit_event(Event::MintedNft(collection_id, nft_id));
        Ok(())
    }
}
