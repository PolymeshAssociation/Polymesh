use frame_support::storage::migration::move_prefix;
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
            // This storage changed the Ticker key to AssetId.
            pub OldNumberOfNFTs get(fn balance_of):
                double_map hasher(blake2_128_concat) Ticker, hasher(identity) IdentityId => NFTCount;

            // This storage changed the Ticker key to AssetId.
            pub CollectionTicker get(fn collection_ticker):
                map hasher(blake2_128_concat) Ticker => NFTCollectionId;

            // This storage changed the Ticker key to AssetId.
            pub Collection get(fn nft_collection):
                map hasher(blake2_128_concat) NFTCollectionId => NFTCollection;

            // This storage changed the Ticker key to AssetId.
            pub OldNFTsInCollection get(fn nfts_in_collection):
                map hasher(blake2_128_concat) Ticker => NFTCount;

            // This storage changed the Ticker key to AssetId.
            pub OldNFTOwner get(fn nft_owner):
                double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) NFTId => Option<PortfolioId>;

            // This storage has been removed.
            pub NextCollectionId get(fn collection_id): NFTCollectionId;

            // This storage has been removed.
            pub NextNFTId get(fn nft_id): map hasher(blake2_128_concat) NFTCollectionId => NFTId;
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

    let mut count = 0;
    log::info!("Updating types for the NumberOfNFTs storage");
    move_prefix(
        &NumberOfNFTs::final_prefix(),
        &v3::OldNumberOfNFTs::final_prefix(),
    );
    v3::OldNumberOfNFTs::drain().for_each(|(ticker, did, n)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        NumberOfNFTs::insert(asset_id, did, n);
    });
    log::info!("Migrated {:?} NFT.NumberOfNFTs entries.", count);

    let mut count = 0;
    log::info!("Updating types for the CollectionTicker storage");
    v3::CollectionTicker::drain().for_each(|(ticker, id)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        CollectionAsset::insert(asset_id, id);
    });
    log::info!("Migrated {:?} NFT.CollectionTicker entries.", count);

    let mut count = 0;
    log::info!("Updating types for the Collection storage");
    v3::Collection::drain().for_each(|(id, collection)| {
        count += 1;
        Collection::insert(id, NFTCollection::from(collection));
    });
    log::info!("Migrated {:?} NFT.Collection entries.", count);

    let mut count = 0;
    log::info!("Updating types for the NFTsInCollection storage");
    move_prefix(
        &NFTsInCollection::final_prefix(),
        &v3::OldNFTsInCollection::final_prefix(),
    );
    v3::OldNFTsInCollection::drain().for_each(|(ticker, n)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        NFTsInCollection::insert(asset_id, n);
    });
    log::info!("Migrated {:?} NFT.NFTsInCollection entries.", count);

    let mut count = 0;
    log::info!("Updating types for the NFTOwner storage");
    move_prefix(&NFTOwner::final_prefix(), &v3::OldNFTOwner::final_prefix());
    v3::OldNFTOwner::drain().for_each(|(ticker, nft_id, portfolio)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        NFTOwner::insert(asset_id, nft_id, portfolio);
    });
    log::info!("Migrated {:?} NFT.NFTOwner entries.", count);

    log::info!("NextCollectionId has been cleared");
    v3::NextCollectionId::kill();

    log::info!("Removing old NextNFTId storage");
    let res = v3::NextNFTId::clear(u32::max_value(), None);
    log::info!("{:?} NFT.NextNFTId items have been cleared", res.unique);
}
