use frame_benchmarking::benchmarks;
use polymesh_common_utilities::benchs::{make_asset, user, AccountIdOf};
use polymesh_common_utilities::traits::asset::AssetFnTrait;
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataSpec, AssetMetadataValue,
};
use polymesh_primitives::nft::{NFTCollectionId, NFTCollectionKeys, NFTId};
use scale_info::prelude::format;
use sp_std::vec::Vec;

use crate::*;

/// Creates an NFT collection with 255 metadata keys
fn create_collection<T: Config>(origin: T::Origin, ticker: Ticker) -> NFTCollectionId {
    let collection_keys: NFTCollectionKeys = (1..256)
        .map(|key| AssetMetadataKey::Local(AssetMetadataLocalKey(key)))
        .collect::<Vec<AssetMetadataKey>>()
        .into();
    for i in 1..256 {
        let asset_metadata_name = format!("key{}", i).as_bytes().to_vec();
        T::AssetFn::register_asset_metadata_type(
            origin.clone(),
            Some(ticker.clone()),
            asset_metadata_name.into(),
            AssetMetadataSpec::default(),
        )
        .expect("failed to register asset metadata");
    }
    Module::<T>::create_nft_collection(origin, ticker, collection_keys)
        .expect("failed to create nft collection");
    Module::<T>::collection_id()
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    create_nft_collection {
        let user = user::<T>("target", 0);
        let ticker = make_asset::<T>(&user, None);
        let collection_keys: NFTCollectionKeys = (1..256)
            .map(|key| AssetMetadataKey::Local(AssetMetadataLocalKey(key)))
            .collect::<Vec<AssetMetadataKey>>()
            .into();
        for i in 1..256 {
            let asset_metadata_name = format!("key{}", i).as_bytes().to_vec();
            T::AssetFn::register_asset_metadata_type(
                user.origin.clone().into(),
                Some(ticker.clone()),
                asset_metadata_name.into(),
                AssetMetadataSpec::default()
            )
            .expect("failed to register asset metadata");
        }
    }: _(user.origin, ticker, collection_keys)
    verify {
        assert!(Collection::contains_key(NFTCollectionId(1)));
        assert_eq!(CollectionKeys::get(NFTCollectionId(1)).len(), 255);
    }

    mint_nft {
        let user = user::<T>("target", 0);
        let ticker = make_asset::<T>(&user, None);
        let collection_id = create_collection::<T>(user.origin().into(), ticker);
        let metadata_attributes: Vec<NFTMetadataAttribute> = (1..256)
            .map(|key| {
                NFTMetadataAttribute{
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(key)),
                    value: AssetMetadataValue(b"value".to_vec()),
                }
            })
            .collect();
    }: _(user.origin, collection_id, metadata_attributes)
    verify {
        for i in 1..256 {
            assert!(
                MetadataValue::contains_key(
                    (NFTCollectionId(1), NFTId(1)),
                    AssetMetadataKey::Local(AssetMetadataLocalKey(i))
                )
            );
        }
    }
}
