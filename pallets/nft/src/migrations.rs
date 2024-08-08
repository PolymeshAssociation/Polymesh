use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v3 {
    use scale_info::TypeInfo;

    use super::*;
    use polymesh_primitives::Ticker;

    #[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
    pub struct NFTCollection {
        pub id: NFTCollectionId,
        pub ticker: Ticker,
    }

    decl_storage! {
        trait Store for Module<T: Config> as NFT {
            // This storage changed the Ticker key to AssetID.
            pub NumberOfNFTs get(fn balance_of):
                double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => NFTCount;

            // This storage changed the Ticker key to AssetID.
            pub CollectionTicker get(fn collection_ticker):
                map hasher(blake2_128_concat) Ticker => NFTCollectionId;

            // This storage changed the Ticker key to AssetID.
            pub Collection get(fn nft_collection):
                map hasher(blake2_128_concat) NFTCollectionId => NFTCollection;

            // This storage changed the Ticker key to AssetID.
            pub NFTsInCollection get(fn nfts_in_collection):
                map hasher(blake2_128_concat) Ticker => NFTCount;

            // This storage changed the Ticker key to AssetID.
            pub NFTOwner get(fn nft_owner):
                double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) NFTId => Option<PortfolioId>;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

impl From<v3::NFTCollection> for NFTCollection {
    fn from(v3_nft_collection: v3::NFTCollection) -> NFTCollection {
        NFTCollection::new(v3_nft_collection.id, v3_nft_collection.ticker.into())
    }
}

pub(crate) fn migrate_to_v4<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    // Removes all elements in the old storage and inserts it in the new storage

    log::info!("Updating types for the NumberOfNFTs storage");
    v3::NumberOfNFTs::drain().for_each(|(ticker, did, n)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        NumberOfNFTs::insert(asset_id, did, n);
    });

    log::info!("Updating types for the CollectionTicker storage");
    v3::CollectionTicker::drain().for_each(|(ticker, id)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        CollectionAsset::insert(asset_id, id);
    });

    log::info!("Updating types for the Collection storage");
    v3::Collection::drain().for_each(|(id, collection)| {
        Collection::insert(id, NFTCollection::from(collection));
    });

    log::info!("Updating types for the NFTsInCollection storage");
    v3::NFTsInCollection::drain().for_each(|(ticker, n)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        NFTsInCollection::insert(asset_id, n);
    });

    log::info!("Updating types for the NFTOwner storage");
    v3::NFTOwner::drain().for_each(|(ticker, nft_id, portfolio)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        NFTOwner::insert(asset_id, nft_id, portfolio);
    });
}
