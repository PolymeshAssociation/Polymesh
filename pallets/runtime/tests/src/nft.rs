use chrono::prelude::Utc;
use frame_support::{assert_noop, assert_ok, StorageDoubleMap, StorageMap};

use pallet_nft::{Collection, CollectionKeys, MetadataValue, NumberOfNFTs};
use pallet_portfolio::PortfolioNFT;
use polymesh_common_utilities::traits::nft::Event;
use polymesh_common_utilities::with_transaction;
use polymesh_primitives::asset::{AssetType, NonFungibleType};
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataSpec,
    AssetMetadataValue,
};
use polymesh_primitives::settlement::InstructionId;
use polymesh_primitives::{
    IdentityId, NFTCollectionId, NFTCollectionKeys, NFTId, NFTMetadataAttribute, NFTs, PortfolioId,
    PortfolioKind, PortfolioNumber, PortfolioUpdateReason, Ticker, WeightMeter,
};
use test_client::AccountKeyring;

use crate::asset_test::set_timestamp;
use crate::ext_builder::ExtBuilder;
use crate::storage::{TestStorage, User};

type Asset = pallet_asset::Module<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type NFT = pallet_nft::Module<TestStorage>;
type NFTError = pallet_nft::Error<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

/// Successfully creates an NFT collection and an Asset.
#[test]
fn create_collection_unregistered_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: NFTCollectionKeys = vec![].into();

        assert_ok!(NFT::create_nft_collection(
            alice.origin(),
            ticker.clone(),
            Some(nft_type),
            collection_keys
        ));
        assert_eq!(Asset::token_details(&ticker).divisible, false);
        assert_eq!(
            Asset::token_details(&ticker).asset_type,
            AssetType::NonFungible(nft_type)
        );
    });
}

/// An NFT collection can only be created for assets of type NFT.
#[test]
fn create_collection_invalid_asset_type() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let collection_keys: NFTCollectionKeys = vec![].into();

        Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker.clone(),
            false,
            AssetType::default(),
            Vec::new(),
            None,
            true,
        )
        .expect("failed to create an asset");

        assert_noop!(
            NFT::create_nft_collection(alice.origin(), ticker, None, collection_keys),
            NFTError::InvalidAssetType
        );
    });
}

/// There can only be one NFT collection per ticker.
#[test]
fn create_collection_already_registered() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: NFTCollectionKeys = vec![].into();

        assert_ok!(NFT::create_nft_collection(
            alice.origin(),
            ticker,
            Some(nft_type),
            collection_keys.clone()
        ));
        assert_noop!(
            NFT::create_nft_collection(alice.origin(), ticker, Some(nft_type), collection_keys),
            NFTError::CollectionAlredyRegistered
        );
    });
}

/// An NFT collection can only be created if the number of metadata keys does not exceed 255.
#[test]
fn create_collection_max_keys_exceeded() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: Vec<AssetMetadataKey> = (0..256)
            .map(|key| AssetMetadataKey::Local(AssetMetadataLocalKey(key)))
            .collect();
        assert_noop!(
            NFT::create_nft_collection(
                alice.origin(),
                ticker,
                Some(nft_type),
                collection_keys.into()
            ),
            NFTError::MaxNumberOfKeysExceeded
        );
    });
}

/// An NFT collection can only be created if there are no duplicated keys defined.
#[test]
fn create_collection_duplicate_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: NFTCollectionKeys = vec![
            AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
        ]
        .into();

        assert_noop!(
            NFT::create_nft_collection(
                alice.origin(),
                ticker,
                Some(nft_type),
                collection_keys.into()
            ),
            NFTError::DuplicateMetadataKey
        );
    });
}

/// An NFT collection can only be created if all metadata keys are alredy registered.
#[test]
fn create_collection_unregistered_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(0))].into();

        assert_noop!(
            NFT::create_nft_collection(alice.origin(), ticker, Some(nft_type), collection_keys),
            NFTError::UnregisteredMetadataKey
        );
    });
}

/// Successfully creates an NFT collection.
pub(crate) fn create_nft_collection(
    owner: User,
    ticker: Ticker,
    asset_type: AssetType,
    collection_keys: NFTCollectionKeys,
) {
    Asset::create_asset(
        owner.origin(),
        ticker.as_ref().into(),
        ticker.clone(),
        false,
        asset_type,
        Vec::new(),
        None,
        true,
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
        None,
        collection_keys
    ));
    assert!(Collection::contains_key(NFTCollectionId(1)));
    assert_eq!(CollectionKeys::get(NFTCollectionId(1)).len(), n_keys);
}

/// An NFT can only be minted if its collection exists.
#[test]
fn mint_nft_collection_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                ticker,
                vec![NFTMetadataAttribute {
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
                    value: AssetMetadataValue(b"test".to_vec())
                }],
                PortfolioKind::Default
            ),
            NFTError::CollectionNotFound
        );
    });
}

/// An NFT can only be minted if it has no duplicate metadata keys.
#[test]
fn mint_nft_duplicate_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let collection_keys: NFTCollectionKeys = vec![
            AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(2)),
        ]
        .into();

        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_noop!(
            NFT::issue_nft(
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
                ],
                PortfolioKind::Default
            ),
            NFTError::DuplicateMetadataKey
        );
    });
}

/// An NFT can only be minted if it has the same number of keys that was defined in the collection.
#[test]
fn mint_nft_wrong_number_of_keys() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_noop!(
            NFT::issue_nft(
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
                ],
                PortfolioKind::Default
            ),
            NFTError::InvalidMetadataAttribute
        );
        assert_noop!(
            NFT::issue_nft(alice.origin(), ticker, vec![], PortfolioKind::Default),
            NFTError::InvalidMetadataAttribute
        );
    });
}

/// An NFT can only be minted if it has the same keys that were defined in the collection.
#[test]
fn mint_nft_wrong_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                ticker,
                vec![NFTMetadataAttribute {
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(35)),
                    value: AssetMetadataValue(b"test".to_vec())
                }],
                PortfolioKind::Default
            ),
            NFTError::InvalidMetadataAttribute
        );
    });
}

/// An NFT can only be minted if the given portfolio exists.
#[test]
fn mint_nft_portfolio_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                ticker,
                vec![NFTMetadataAttribute {
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                    value: AssetMetadataValue(b"test".to_vec())
                }],
                PortfolioKind::User(PortfolioNumber(1))
            ),
            PortfolioError::PortfolioDoesNotExist
        );
    });
}

/// Successfully mints an NFT.
#[test]
fn mint_nft_successfully() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_ok!(NFT::issue_nft(
            alice.origin(),
            ticker,
            vec![NFTMetadataAttribute {
                key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                value: AssetMetadataValue(b"test".to_vec())
            }],
            PortfolioKind::Default
        ));
        assert_eq!(
            MetadataValue::get(
                (NFTCollectionId(1), NFTId(1)),
                AssetMetadataKey::Local(AssetMetadataLocalKey(1))
            ),
            AssetMetadataValue(b"test".to_vec())
        );
        assert_eq!(NumberOfNFTs::get(&ticker, alice.did), 1);
        assert_eq!(
            PortfolioNFT::get(
                PortfolioId::default_portfolio(alice.did),
                (&ticker, NFTId(1))
            ),
            true
        );
    });
}

pub(crate) fn mint_nft(
    user: User,
    ticker: Ticker,
    metadata_atributes: Vec<NFTMetadataAttribute>,
    portfolio_kind: PortfolioKind,
) {
    assert_ok!(NFT::issue_nft(
        user.origin(),
        ticker,
        metadata_atributes,
        portfolio_kind
    ));
}

/// An NFT can only be burned if its collection exists.
#[test]
fn burn_nft_collection_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());

        assert_noop!(
            NFT::redeem_nft(alice.origin(), ticker, NFTId(1), PortfolioKind::Default),
            NFTError::CollectionNotFound
        );
    });
}

/// An NFT can only be burned if it exists in the portfolio.
#[test]
fn burn_nft_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );

        assert_noop!(
            NFT::redeem_nft(alice.origin(), ticker, NFTId(1), PortfolioKind::Default),
            NFTError::NFTNotFound
        );
    });
}

/// Successfully burns an NFT.
#[test]
fn burn_nft() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        NFT::issue_nft(
            alice.origin(),
            ticker,
            vec![NFTMetadataAttribute {
                key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                value: AssetMetadataValue(b"test".to_vec()),
            }],
            PortfolioKind::Default,
        )
        .unwrap();

        assert_ok!(NFT::redeem_nft(
            alice.origin(),
            ticker,
            NFTId(1),
            PortfolioKind::Default
        ));
        assert!(!MetadataValue::contains_key(
            (NFTCollectionId(1), NFTId(1)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(1))
        ),);
        assert_eq!(NumberOfNFTs::get(&ticker, alice.did), 0);
        assert!(!PortfolioNFT::contains_key(
            PortfolioId::default_portfolio(alice.did),
            (&ticker, NFTId(1))
        ),);
    });
}

/// An NFT can only be transferred if its collection exists.
#[test]
fn transfer_nft_without_collection() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let bob: User = User::new(AccountKeyring::Bob);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let sender_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(ticker, vec![NFTId(1)]).unwrap();

        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio,
                    receiver_portfolio,
                    nfts,
                    InstructionId(0),
                    None,
                    IdentityId::default(),
                    &mut weight_meter,
                )
            }),
            NFTError::InvalidNFTTransferCollectionNotFound
        );
    });
}

/// An NFT can only be transferred to a differrent portfolio.
#[test]
fn transfer_nft_same_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        // Creates a collection
        let alice: User = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );

        // Attempts to transfer to the same portfolio
        let sender_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(ticker, vec![NFTId(1)]).unwrap();
        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio,
                    receiver_portfolio,
                    nfts,
                    InstructionId(0),
                    None,
                    IdentityId::default(),
                    &mut weight_meter,
                )
            }),
            NFTError::InvalidNFTTransferSamePortfolio
        );
    });
}

/// An NFT can only be transferred if there is enough balance.
#[test]
fn transfer_nft_invalid_count() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        // First we need to create a collection and mint one NFT
        let alice: User = User::new(AccountKeyring::Alice);
        let bob: User = User::new(AccountKeyring::Bob);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            ticker.clone(),
            nfts_metadata,
            PortfolioKind::Default,
        );

        // Attempts to transfer two NFTs
        let sender_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(ticker, vec![NFTId(1), NFTId(2)]).unwrap();
        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio,
                    receiver_portfolio,
                    nfts,
                    InstructionId(0),
                    None,
                    IdentityId::default(),
                    &mut weight_meter,
                )
            }),
            NFTError::InvalidNFTTransferInsufficientCount
        );
    });
}

/// An NFT can only be transferred if it is owned by the sender.
#[test]
fn transfer_nft_not_owned() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        // First we need to create a collection and mint one NFT
        let alice: User = User::new(AccountKeyring::Alice);
        let bob: User = User::new(AccountKeyring::Bob);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            ticker.clone(),
            nfts_metadata.clone(),
            PortfolioKind::Default,
        );

        // Attempts to transfer an NFT not owned by the sender
        let sender_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(ticker, vec![NFTId(1)]).unwrap();
        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio,
                    receiver_portfolio,
                    nfts,
                    InstructionId(0),
                    None,
                    IdentityId::default(),
                    &mut weight_meter,
                )
            }),
            NFTError::InvalidNFTTransferInsufficientCount
        );
    });
}

/// An NFT can only be transferred if the compliance rules are respected.
#[test]
fn transfer_nft_failing_compliance() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        // First we need to create a collection and mint one NFT
        let alice: User = User::new(AccountKeyring::Alice);
        let bob: User = User::new(AccountKeyring::Bob);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            ticker.clone(),
            nfts_metadata,
            PortfolioKind::Default,
        );

        // transfer the NFT
        let sender_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(ticker, vec![NFTId(1)]).unwrap();
        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio,
                    receiver_portfolio,
                    nfts,
                    InstructionId(0),
                    None,
                    IdentityId::default(),
                    &mut weight_meter,
                )
            }),
            NFTError::InvalidNFTTransferComplianceFailure
        );
    });
}

/// Successfully transfer an NFT
#[test]
fn transfer_nft() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);
        System::set_block_number(1);

        // First we need to create a collection and mint one NFT
        let alice: User = User::new(AccountKeyring::Alice);
        let bob: User = User::new(AccountKeyring::Bob);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        create_nft_collection(
            alice.clone(),
            ticker.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            ticker.clone(),
            nfts_metadata,
            PortfolioKind::Default,
        );
        ComplianceManager::pause_asset_compliance(alice.origin(), ticker.clone()).unwrap();

        // transfer the NFT
        let sender_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(ticker, vec![NFTId(1)]).unwrap();
        assert_ok!(with_transaction(|| {
            NFT::base_nft_transfer(
                sender_portfolio,
                receiver_portfolio,
                nfts.clone(),
                InstructionId(0),
                None,
                IdentityId::default(),
                &mut weight_meter,
            )
        }));
        assert_eq!(NumberOfNFTs::get(&ticker, alice.did), 0);
        assert_eq!(
            PortfolioNFT::get(
                PortfolioId::default_portfolio(alice.did),
                (&ticker, NFTId(1))
            ),
            false
        );
        assert_eq!(NumberOfNFTs::get(&ticker, bob.did), 1);
        assert_eq!(
            PortfolioNFT::get(PortfolioId::default_portfolio(bob.did), (&ticker, NFTId(1))),
            true
        );
        assert_eq!(
            super::storage::EventTest::Nft(Event::NFTPortfolioUpdated(
                IdentityId::default(),
                nfts,
                Some(sender_portfolio),
                Some(receiver_portfolio),
                PortfolioUpdateReason::Transferred {
                    instruction_id: Some(InstructionId(0)),
                    instruction_memo: None
                }
            )),
            System::events().last().unwrap().event,
        );
    });
}
