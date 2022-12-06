use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use polymesh_common_utilities::benchs::{user, AccountIdOf};
use polymesh_common_utilities::traits::asset::AssetFnTrait;
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::asset_metadata::{
    AssetMetadataGlobalKey, AssetMetadataKey, AssetMetadataSpec, AssetMetadataValue,
};
use polymesh_primitives::nft::{NFTCollectionId, NFTCollectionKeys, NFTId};
use polymesh_primitives::PortfolioKind;
use scale_info::prelude::format;
use sp_std::prelude::*;
use sp_std::vec::Vec;

use crate::*;

const MAX_COLLECTION_KEYS: u32 = 255;

/// Creates an NFT collection with `n` global metadata keys.
fn create_collection<T: Config>(origin: T::Origin, ticker: Ticker, n: u32) -> NFTCollectionId {
    let collection_keys: NFTCollectionKeys = creates_keys_register_metadata_types::<T>(n);
    Module::<T>::create_nft_collection(origin, ticker, collection_keys)
        .expect("failed to create nft collection");
    Module::<T>::collection_id()
}

/// Creates a set of `NFTCollectionKeys` made of `n` global keys and registers `n` global asset metadata types.
fn creates_keys_register_metadata_types<T: Config>(n: u32) -> NFTCollectionKeys {
    let collection_keys: NFTCollectionKeys = (1..n + 1)
        .map(|key| AssetMetadataKey::Global(AssetMetadataGlobalKey(key.into())))
        .collect::<Vec<AssetMetadataKey>>()
        .into();
    for i in 1..n + 1 {
        let asset_metadata_name = format!("key{}", i).as_bytes().to_vec();
        T::AssetFn::register_asset_metadata_type(
            RawOrigin::Root.into(),
            None,
            asset_metadata_name.into(),
            AssetMetadataSpec::default(),
        )
        .expect("failed to register asset metadata");
    }
    collection_keys
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    create_nft_collection {
        let n in 1..MAX_COLLECTION_KEYS;

        let user = user::<T>("target", 0);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys = creates_keys_register_metadata_types::<T>(n);
    }: _(user.origin, ticker, collection_keys)
    verify {
        assert!(Collection::contains_key(NFTCollectionId(1)));
        assert_eq!(CollectionKeys::get(NFTCollectionId(1)).len(), n as usize);
    }

    mint_nft {
        let n in 1..MAX_COLLECTION_KEYS;

        let user = user::<T>("target", 0);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_id = create_collection::<T>(user.origin().into(), ticker, n);
        let metadata_attributes: Vec<NFTMetadataAttribute> = (1..n + 1)
            .map(|key| {
                NFTMetadataAttribute{
                    key: AssetMetadataKey::Global(AssetMetadataGlobalKey(key.into())),
                    value: AssetMetadataValue(b"value".to_vec()),
                }
            })
            .collect();
    }: _(user.origin, ticker, metadata_attributes)
    verify {
        for i in 1..n + 1 {
            assert!(
                MetadataValue::contains_key(
                    (NFTCollectionId(1), NFTId(1)),
                    AssetMetadataKey::Global(AssetMetadataGlobalKey(i.into()))
                )
            );
        }
    }

    burn_nft {
        let user = user::<T>("target", 0);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_id = create_collection::<T>(user.origin().into(), ticker, 1);

        let metadata_attributes: Vec<NFTMetadataAttribute> = vec![
            NFTMetadataAttribute {
                key: AssetMetadataKey::Global(AssetMetadataGlobalKey(1)),
                value: AssetMetadataValue(b"value".to_vec())
            }
        ];
        Module::<T>::mint_nft(user.origin().into(), ticker, metadata_attributes).expect("failed to mint nft");
    }: _(user.origin, ticker, NFTId(1), PortfolioKind::Default)
    verify {
        assert!(!MetadataValue::contains_key(
            (NFTCollectionId(1), NFTId(1)),
            AssetMetadataKey::Global(AssetMetadataGlobalKey(1))
        ),);
    }
}
