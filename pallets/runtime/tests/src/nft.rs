use chrono::prelude::Utc;
use frame_support::{assert_noop, assert_ok};
use frame_support::{StorageDoubleMap, StorageMap};
use pallet_asset::BalanceOf;
use pallet_nft::{Collection, CollectionKeys, MetadataValue};
use pallet_portfolio::PortfolioNFT;
use polymesh_primitives::asset::AssetType;
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataSpec,
    AssetMetadataValue,
};
use polymesh_primitives::{
    NFTCollectionId, NFTCollectionKeys, NFTId, NFTMetadataAttribute, PortfolioId, PortfolioKind,
    Ticker,
};
use test_client::AccountKeyring;

use crate::ext_builder::ExtBuilder;
use crate::storage::{TestStorage, User};

type Asset = pallet_asset::Module<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type NFT = pallet_nft::Module<TestStorage>;
type NFTError = pallet_nft::Error<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

/// Successfully creates an NFT collection and an Asset.
#[test]
fn create_collection_unregistered_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys = vec![].into();

        assert_ok!(NFT::create_nft_collection(
            alice.origin(),
            ticker.clone(),
            collection_keys
        ));
        assert_eq!(Asset::token_details(&ticker).divisible, false);
        assert_eq!(Asset::token_details(&ticker).asset_type, AssetType::NFT);
    });
}

/// An NFT collection can only be created for assets of type NFT.
#[test]
fn create_collection_invalid_asset_type() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys = vec![].into();

        Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker.clone(),
            false,
            AssetType::default(),
            Vec::new(),
            None,
            false,
        )
        .expect("failed to create an asset");

        assert_noop!(
            NFT::create_nft_collection(alice.origin(), ticker, collection_keys),
            NFTError::InvalidAssetType
        );
    });
}

/// There can only be one NFT collection per ticker.
#[test]
fn create_collection_already_registered() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys = vec![].into();

        assert_ok!(NFT::create_nft_collection(
            alice.origin(),
            ticker,
            collection_keys.clone()
        ));
        assert_noop!(
            NFT::create_nft_collection(alice.origin(), ticker, collection_keys),
            NFTError::CollectionAlredyRegistered
        );
    });
}

/// An NFT collection can only be created if the number of metadata keys does not exceed 255.
#[test]
fn create_collection_max_keys_exceeded() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: Vec<AssetMetadataKey> = (0..256)
            .map(|key| AssetMetadataKey::Local(AssetMetadataLocalKey(key)))
            .collect();
        assert_noop!(
            NFT::create_nft_collection(alice.origin(), ticker, collection_keys.into()),
            NFTError::MaxNumberOfKeysExceeded
        );
    });
}

/// An NFT collection can only be created if there are no duplicated keys defined.
#[test]
fn create_collection_duplicate_key() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys = vec![
            AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
        ]
        .into();

        assert_noop!(
            NFT::create_nft_collection(alice.origin(), ticker, collection_keys.into()),
            NFTError::DuplicateMetadataKey
        );
    });
}

/// An NFT collection can only be created if all metadata keys are alredy registered.
#[test]
fn create_collection_unregistered_key() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(0))].into();

        assert_noop!(
            NFT::create_nft_collection(alice.origin(), ticker, collection_keys),
            NFTError::UnregisteredMetadataKey
        );
    });
}

/// Successfully creates an NFT collection.
fn create_nft_collection(owner: User, ticker: Ticker, collection_keys: NFTCollectionKeys) {
    Asset::create_asset(
        owner.origin(),
        ticker.as_ref().into(),
        ticker.clone(),
        false,
        AssetType::NFT,
        Vec::new(),
        None,
        false,
    )
    .expect("failed to create an asset");
    for (i, _) in collection_keys.keys().iter().enumerate() {
        Asset::register_asset_metadata_local_type(
            owner.origin(),
            ticker.clone(),
            AssetMetadataName(format!("key{}", i).as_bytes().to_vec()),
            AssetMetadataSpec {
                url: None,
                description: None,
                type_def: None,
            },
        )
        .unwrap();
    }
    let n_keys = collection_keys.len();
    assert_ok!(NFT::create_nft_collection(
        owner.origin(),
        ticker,
        collection_keys
    ));
    assert!(Collection::contains_key(NFTCollectionId(1)));
    assert_eq!(CollectionKeys::get(NFTCollectionId(1)).len(), n_keys);
}

/// An NFT can only be minted if its collection exists.
#[test]
fn mint_nft_collection_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        assert_noop!(
            NFT::mint_nft(
                alice.origin(),
                ticker,
                vec![NFTMetadataAttribute {
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
                    value: AssetMetadataValue(b"test".to_vec())
                }]
            ),
            NFTError::CollectionNotFound
        );
    });
}

/// An NFT can only be minted if it has no duplicate metadata keys.
#[test]
fn mint_nft_duplicate_key() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        create_nft_collection(alice.clone(), ticker.clone(), collection_keys);
        assert_noop!(
            NFT::mint_nft(
                alice.origin(),
                ticker,
                vec![
                    NFTMetadataAttribute {
                        key: AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
                        value: AssetMetadataValue(b"test".to_vec())
                    },
                    NFTMetadataAttribute {
                        key: AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
                        value: AssetMetadataValue(b"test".to_vec())
                    }
                ]
            ),
            NFTError::DuplicateMetadataKey
        );
    });
}

/// An NFT can only be minted if it has the same number of keys that was defined in the collection.
#[test]
fn mint_nft_wrong_number_of_keys() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        create_nft_collection(alice.clone(), ticker.clone(), collection_keys);
        assert_noop!(
            NFT::mint_nft(
                alice.origin(),
                ticker.clone(),
                vec![
                    NFTMetadataAttribute {
                        key: AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
                        value: AssetMetadataValue(b"test".to_vec())
                    },
                    NFTMetadataAttribute {
                        key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                        value: AssetMetadataValue(b"test".to_vec())
                    }
                ]
            ),
            NFTError::InvalidMetadataAttribute
        );
        assert_noop!(
            NFT::mint_nft(alice.origin(), ticker, vec![]),
            NFTError::InvalidMetadataAttribute
        );
    });
}

/// An NFT can only be minted if it has the same keys that were defined in the collection.
#[test]
fn mint_nft_wrong_key() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        create_nft_collection(alice.clone(), ticker.clone(), collection_keys);
        assert_noop!(
            NFT::mint_nft(
                alice.origin(),
                ticker,
                vec![NFTMetadataAttribute {
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(35)),
                    value: AssetMetadataValue(b"test".to_vec())
                }]
            ),
            NFTError::InvalidMetadataAttribute
        );
    });
}

/// Successfully mints an NFT.
#[test]
fn mint_nft() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        create_nft_collection(alice.clone(), ticker.clone(), collection_keys);
        assert_ok!(NFT::mint_nft(
            alice.origin(),
            ticker,
            vec![NFTMetadataAttribute {
                key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                value: AssetMetadataValue(b"test".to_vec())
            }]
        ));
        assert_eq!(
            MetadataValue::get(
                (NFTCollectionId(1), NFTId(1)),
                AssetMetadataKey::Local(AssetMetadataLocalKey(1))
            ),
            AssetMetadataValue(b"test".to_vec())
        );
        assert_eq!(BalanceOf::get(&ticker, alice.did), 1);
        assert_eq!(
            PortfolioNFT::get(
                PortfolioId::default_portfolio(alice.did),
                (NFTCollectionId(1), NFTId(1))
            ),
            true
        );
    });
}

/// An NFT can only be burned if its collection exists.
#[test]
fn burn_nft_collection_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();

        assert_noop!(
            NFT::burn_nft(alice.origin(), ticker, NFTId(1), PortfolioKind::Default),
            NFTError::CollectionNotFound
        );
    });
}

/// An NFT can only be burned if it exists in the portfolio.
#[test]
fn burn_nft_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        create_nft_collection(alice.clone(), ticker.clone(), collection_keys);

        assert_noop!(
            NFT::burn_nft(alice.origin(), ticker, NFTId(1), PortfolioKind::Default),
            NFTError::NFTNotFound
        );
    });
}

/// Successfully burns an NFT.
#[test]
fn burn_nft() {
    ExtBuilder::default().build().execute_with(|| {
        Timestamp::set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = b"TICKER".as_ref().try_into().unwrap();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        create_nft_collection(alice.clone(), ticker.clone(), collection_keys);
        NFT::mint_nft(
            alice.origin(),
            ticker,
            vec![NFTMetadataAttribute {
                key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                value: AssetMetadataValue(b"test".to_vec()),
            }],
        )
        .unwrap();

        assert_ok!(NFT::burn_nft(
            alice.origin(),
            ticker,
            NFTId(1),
            PortfolioKind::Default
        ));
        assert!(!MetadataValue::contains_key(
            (NFTCollectionId(1), NFTId(1)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(1))
        ),);
        assert_eq!(BalanceOf::get(&ticker, alice.did), 0);
        assert!(!PortfolioNFT::contains_key(
            PortfolioId::default_portfolio(alice.did),
            (NFTCollectionId(1), NFTId(1))
        ),);
    });
}
