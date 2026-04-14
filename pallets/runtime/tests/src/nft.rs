use chrono::prelude::Utc;
use frame_support::{assert_noop, assert_ok};

use pallet_nft::Event;
use pallet_nft::{
    Collection, CollectionKeys, CurrentCollectionId, CurrentNFTId, MetadataValue, NFTsInCollection,
    NumberOfNFTs, Owner,
};
use pallet_portfolio::PortfolioNFT;
use polymesh_primitives::asset::{AssetId, AssetName, AssetType, NonFungibleType};
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataSpec,
    AssetMetadataValue,
};
use polymesh_primitives::settlement::{InstructionId, Leg, SettlementType};
use polymesh_primitives::{
    with_transaction, AssetHolderKind, AuthorizationData, Claim, ClaimType, Condition,
    ConditionType, CountryCode, HoldingsUpdateReason, IdentityId, NFTCollectionId,
    NFTCollectionKeys, NFTId, NFTMetadataAttribute, NFTs, PortfolioId, PortfolioKind,
    PortfolioName, PortfolioNumber, Scope, Signatory, TrustedFor, TrustedIssuer, WeightMeter,
};
use sp_keyring::Sr25519Keyring;

use super::asset_test::{get_asset_details, set_timestamp};
use crate::asset_pallet::setup::{create_and_issue_sample_asset, create_and_issue_sample_nft};
use crate::ext_builder::ExtBuilder;
use crate::storage::{default_asset_holder_set, TestStorage, User};

type Asset = pallet_asset::Pallet<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Pallet<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;
type Identity = pallet_identity::Pallet<TestStorage>;
type NFT = pallet_nft::Pallet<TestStorage>;
type NFTError = pallet_nft::Error<TestStorage>;
type Portfolio = pallet_portfolio::Pallet<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

/// Successfully creates an NFT collection and an Asset.
#[test]
fn create_collection_unregistered_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: NFTCollectionKeys = vec![].into();

        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(NFT::create_nft_collection(
            alice.origin(),
            None,
            Some(nft_type),
            collection_keys
        ));
        assert_eq!(get_asset_details(&asset_id).divisible, false);
        assert_eq!(
            get_asset_details(&asset_id).asset_type,
            AssetType::NonFungible(nft_type)
        );
    });
}

/// An NFT collection can only be created for assets of type NFT.
#[test]
fn create_collection_invalid_asset_type() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);
        let collection_keys: NFTCollectionKeys = vec![].into();

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_noop!(
            NFT::create_nft_collection(alice.origin(), Some(asset_id), None, collection_keys),
            NFTError::InvalidAssetType
        );
    });
}

/// There can only be one NFT collection per asset_id.
#[test]
fn create_collection_already_registered() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: NFTCollectionKeys = vec![].into();

        let asset_id = create_and_issue_sample_nft(&alice);
        assert_noop!(
            NFT::create_nft_collection(
                alice.origin(),
                Some(asset_id),
                Some(nft_type),
                collection_keys
            ),
            NFTError::CollectionAlredyRegistered
        );
    });
}

/// An NFT collection can only be created if the number of metadata keys does not exceed 255.
#[test]
fn create_collection_max_keys_exceeded() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: Vec<AssetMetadataKey> = (0..256)
            .map(|key| AssetMetadataKey::Local(AssetMetadataLocalKey(key)))
            .collect();
        assert_noop!(
            NFT::create_nft_collection(
                alice.origin(),
                None,
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

        let alice: User = User::new(Sr25519Keyring::Alice);
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: NFTCollectionKeys = vec![
            AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
        ]
        .into();

        assert_noop!(
            NFT::create_nft_collection(
                alice.origin(),
                None,
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

        let alice: User = User::new(Sr25519Keyring::Alice);

        let nft_type = NonFungibleType::Derivative;
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(0))].into();

        assert_noop!(
            NFT::create_nft_collection(alice.origin(), None, Some(nft_type), collection_keys),
            NFTError::UnregisteredMetadataKey
        );
    });
}

/// Successfully creates an NFT collection.
pub(crate) fn create_nft_collection(
    owner: User,
    asset_type: AssetType,
    collection_keys: NFTCollectionKeys,
) -> AssetId {
    let asset_id = Asset::generate_asset_id(owner.acc(), false);
    Asset::create_asset(
        owner.origin(),
        AssetName(b"Myasset".to_vec()),
        false,
        asset_type,
        Vec::new(),
        None,
    )
    .expect("failed to create an asset");
    for (i, _) in collection_keys.keys().iter().enumerate() {
        Asset::register_asset_metadata_local_type(
            owner.origin(),
            asset_id,
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
        Some(asset_id),
        None,
        collection_keys
    ));
    assert!(Collection::<TestStorage>::contains_key(NFTCollectionId(1)));
    assert_eq!(
        CollectionKeys::<TestStorage>::get(NFTCollectionId(1)).len(),
        n_keys
    );
    assert_eq!(
        CurrentCollectionId::<TestStorage>::get(),
        Some(NFTCollectionId(1))
    );

    asset_id
}

/// An NFT can only be minted if its collection exists.
#[test]
fn mint_nft_collection_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                [0; 16].into(),
                vec![NFTMetadataAttribute {
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(0)),
                    value: AssetMetadataValue(b"test".to_vec())
                }],
                AssetHolderKind::DefaultPortfolio
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

        let alice: User = User::new(Sr25519Keyring::Alice);
        let collection_keys: NFTCollectionKeys = vec![
            AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(2)),
        ]
        .into();

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                asset_id,
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
                AssetHolderKind::DefaultPortfolio
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

        let alice: User = User::new(Sr25519Keyring::Alice);

        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                asset_id.clone(),
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
                AssetHolderKind::DefaultPortfolio
            ),
            NFTError::InvalidMetadataAttribute
        );
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                asset_id,
                vec![],
                AssetHolderKind::DefaultPortfolio
            ),
            NFTError::InvalidMetadataAttribute
        );
    });
}

/// An NFT can only be minted if it has the same keys that were defined in the collection.
#[test]
fn mint_nft_wrong_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);

        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                asset_id,
                vec![NFTMetadataAttribute {
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(35)),
                    value: AssetMetadataValue(b"test".to_vec())
                }],
                AssetHolderKind::DefaultPortfolio
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

        let alice: User = User::new(Sr25519Keyring::Alice);

        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                asset_id,
                vec![NFTMetadataAttribute {
                    key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                    value: AssetMetadataValue(b"test".to_vec())
                }],
                AssetHolderKind::UserPortfolio(PortfolioNumber(1))
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

        let alice: User = User::new(Sr25519Keyring::Alice);

        let alice_default_portfolio = PortfolioId::new(alice.did, PortfolioKind::Default);
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_ok!(NFT::issue_nft(
            alice.origin(),
            asset_id,
            vec![NFTMetadataAttribute {
                key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                value: AssetMetadataValue(b"test".to_vec())
            }],
            AssetHolderKind::DefaultPortfolio
        ));
        assert_eq!(
            MetadataValue::<TestStorage>::get(
                (NFTCollectionId(1), NFTId(1)),
                AssetMetadataKey::Local(AssetMetadataLocalKey(1))
            ),
            AssetMetadataValue(b"test".to_vec())
        );
        assert_eq!(NumberOfNFTs::<TestStorage>::get(&asset_id, alice.did), 1);
        assert_eq!(NFTsInCollection::<TestStorage>::get(&asset_id), 1);
        assert_eq!(
            PortfolioNFT::<TestStorage>::get((
                PortfolioId::default_portfolio(alice.did),
                &asset_id,
                NFTId(1)
            )),
            true
        );
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_default_portfolio.into())
        );
        assert_eq!(
            CurrentNFTId::<TestStorage>::get(NFTCollectionId(1)),
            Some(NFTId(1))
        );
    });
}

pub(crate) fn mint_nft(
    user: User,
    asset_id: AssetId,
    metadata_atributes: Vec<NFTMetadataAttribute>,
    asset_holder_kind: AssetHolderKind,
) {
    assert_ok!(NFT::issue_nft(
        user.origin(),
        asset_id,
        metadata_atributes,
        asset_holder_kind
    ));
}

/// An NFT can only be burned if its collection exists.
#[test]
fn burn_nft_collection_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);
        assert_noop!(
            NFT::redeem_nft(
                alice.origin(),
                Asset::generate_asset_id(alice.acc(), false),
                NFTId(1),
                AssetHolderKind::DefaultPortfolio,
                None
            ),
            NFTError::CollectionNotFound
        );
    });
}

/// An NFT can only be burned if it exists in the portfolio.
#[test]
fn burn_nft_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);

        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );

        assert_noop!(
            NFT::redeem_nft(
                alice.origin(),
                asset_id,
                NFTId(1),
                AssetHolderKind::DefaultPortfolio,
                None
            ),
            NFTError::NFTNotFound
        );
    });
}

/// An NFT can only be burned if the caller has custody over the portfolio.
#[test]
fn burn_nft_no_custody() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let bob: User = User::new(Sr25519Keyring::Bob);
        let alice: User = User::new(Sr25519Keyring::Alice);
        let portfolio_kind = PortfolioKind::User(PortfolioNumber(1));
        let portfolio_id = PortfolioId::new(alice.did, portfolio_kind.clone());

        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

        assert_ok!(Portfolio::create_portfolio(
            alice.origin(),
            PortfolioName(b"AliceUserPortfolio".to_vec())
        ));

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );

        // Change custody of the default portfolio
        let authorization_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::PortfolioCustody(portfolio_id),
            None,
        )
        .unwrap();
        assert_ok!(Portfolio::accept_portfolio_custody(
            bob.origin(),
            authorization_id
        ));

        NFT::issue_nft(
            alice.origin(),
            asset_id,
            vec![NFTMetadataAttribute {
                key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                value: AssetMetadataValue(b"test".to_vec()),
            }],
            AssetHolderKind::UserPortfolio(PortfolioNumber(1)),
        )
        .unwrap();

        assert_noop!(
            NFT::redeem_nft(
                alice.origin(),
                asset_id,
                NFTId(1),
                AssetHolderKind::UserPortfolio(PortfolioNumber(1)),
                None
            ),
            PortfolioError::UnauthorizedCustodian
        );
    });
}

/// Successfully burns an NFT.
#[test]
fn burn_nft() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);

        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        NFT::issue_nft(
            alice.origin(),
            asset_id,
            vec![NFTMetadataAttribute {
                key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                value: AssetMetadataValue(b"test".to_vec()),
            }],
            AssetHolderKind::DefaultPortfolio,
        )
        .unwrap();

        assert_ok!(NFT::redeem_nft(
            alice.origin(),
            asset_id,
            NFTId(1),
            AssetHolderKind::DefaultPortfolio,
            None
        ));
        assert!(!MetadataValue::<TestStorage>::contains_key(
            (NFTCollectionId(1), NFTId(1)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(1))
        ),);
        assert_eq!(NumberOfNFTs::<TestStorage>::get(&asset_id, alice.did), 0);
        assert_eq!(NFTsInCollection::<TestStorage>::get(&asset_id), 0);
        assert!(!PortfolioNFT::<TestStorage>::contains_key((
            PortfolioId::default_portfolio(alice.did),
            &asset_id,
            NFTId(1)
        )),);
        assert_eq!(Owner::<TestStorage>::get(asset_id, NFTId(1)), None);
        assert_eq!(
            CurrentNFTId::<TestStorage>::get(NFTCollectionId(1)),
            Some(NFTId(1))
        );
        assert_eq!(
            CurrentCollectionId::<TestStorage>::get(),
            Some(NFTCollectionId(1))
        );
    });
}

/// An NFT can only be transferred if its collection exists.
#[test]
fn transfer_nft_without_collection() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let sender_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(Asset::generate_asset_id(alice.acc(), false), vec![NFTId(1)]).unwrap();

        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio.into(),
                    receiver_portfolio.into(),
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
        let alice: User = User::new(Sr25519Keyring::Alice);

        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let asset_id = create_nft_collection(
            alice.clone(),
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
        let nfts = NFTs::new(asset_id, vec![NFTId(1)]).unwrap();
        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio.into(),
                    receiver_portfolio.into(),
                    nfts,
                    InstructionId(0),
                    None,
                    IdentityId::default(),
                    &mut weight_meter,
                )
            }),
            NFTError::InvalidNFTTransferSenderDidMatchesReceiverDid
        );
    });
}

/// An NFT can only be transferred if there is enough balance.
#[test]
fn transfer_nft_invalid_count() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        // First we need to create a collection and mint one NFT
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            asset_id.clone(),
            nfts_metadata,
            AssetHolderKind::DefaultPortfolio,
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
        let nfts = NFTs::new(asset_id, vec![NFTId(1), NFTId(2)]).unwrap();
        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio.into(),
                    receiver_portfolio.into(),
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
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            asset_id.clone(),
            nfts_metadata.clone(),
            AssetHolderKind::DefaultPortfolio,
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
        let nfts = NFTs::new(asset_id, vec![NFTId(1)]).unwrap();
        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio.into(),
                    receiver_portfolio.into(),
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
        let bob: User = User::new(Sr25519Keyring::Bob);
        let dave: User = User::new(Sr25519Keyring::Dave);
        let alice: User = User::new(Sr25519Keyring::Alice);

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            asset_id.clone(),
            nfts_metadata,
            AssetHolderKind::DefaultPortfolio,
        );

        assert_ok!(ComplianceManager::add_compliance_requirement(
            alice.origin(),
            asset_id,
            Vec::new(),
            vec![Condition {
                condition_type: ConditionType::IsPresent(Claim::Jurisdiction(
                    CountryCode::BR,
                    Scope::Identity(bob.did)
                )),
                issuers: vec![TrustedIssuer {
                    issuer: dave.did,
                    trusted_for: TrustedFor::Specific(vec![ClaimType::Jurisdiction])
                }]
            }],
        ));

        // transfer the NFT
        let sender_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(asset_id, vec![NFTId(1)]).unwrap();
        assert_noop!(
            with_transaction(|| {
                NFT::base_nft_transfer(
                    sender_portfolio.into(),
                    receiver_portfolio.into(),
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
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            asset_id.clone(),
            nfts_metadata,
            AssetHolderKind::DefaultPortfolio,
        );
        ComplianceManager::pause_asset_compliance(alice.origin(), asset_id.clone()).unwrap();

        // transfer the NFT
        let sender_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(asset_id, vec![NFTId(1)]).unwrap();
        assert_ok!(with_transaction(|| {
            NFT::base_nft_transfer(
                sender_portfolio.clone().into(),
                receiver_portfolio.clone().into(),
                nfts.clone(),
                InstructionId(0),
                None,
                IdentityId::default(),
                &mut weight_meter,
            )
        }));
        assert_eq!(NumberOfNFTs::<TestStorage>::get(&asset_id, alice.did), 0);
        assert_eq!(
            PortfolioNFT::<TestStorage>::get((
                PortfolioId::default_portfolio(alice.did),
                &asset_id,
                NFTId(1)
            )),
            false
        );
        assert_eq!(NumberOfNFTs::<TestStorage>::get(&asset_id, bob.did), 1);
        assert_eq!(NFTsInCollection::<TestStorage>::get(&asset_id), 1);
        assert_eq!(
            PortfolioNFT::<TestStorage>::get((
                PortfolioId::default_portfolio(bob.did),
                &asset_id,
                NFTId(1)
            )),
            true
        );
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(receiver_portfolio.clone().into())
        );
        assert_eq!(
            super::storage::EventTest::Nft(Event::NFTHoldingsUpdated(
                IdentityId::default(),
                nfts,
                Some(sender_portfolio.into()),
                Some(receiver_portfolio.into()),
                HoldingsUpdateReason::Transferred {
                    instruction_id: Some(InstructionId(0)),
                    instruction_memo: None
                }
            )),
            System::events().last().unwrap().event,
        );
    });
}

/// Successfully transfer an NFT using the controller transfer.
#[test]
fn controller_transfer() {
    ExtBuilder::default().build().execute_with(|| {
        // First we need to create a collection and mint one NFT
        set_timestamp(Utc::now().timestamp() as _);
        System::set_block_number(1);
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);

        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new().into(),
        );
        mint_nft(
            alice.clone(),
            asset_id.clone(),
            Vec::new(),
            AssetHolderKind::DefaultPortfolio,
        );
        ComplianceManager::pause_asset_compliance(alice.origin(), asset_id.clone()).unwrap();

        // transfer the NFT
        let alice_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let bob_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(asset_id, vec![NFTId(1)]).unwrap();
        assert_ok!(with_transaction(|| {
            NFT::base_nft_transfer(
                alice_portfolio.clone().into(),
                bob_portfolio.clone().into(),
                nfts.clone(),
                InstructionId(0),
                None,
                IdentityId::default(),
                &mut weight_meter,
            )
        }));
        // Before the controller transfer all NFTs belong to bob
        assert_eq!(
            NumberOfNFTs::<TestStorage>::get(nfts.asset_id(), bob.did),
            1
        );
        assert!(PortfolioNFT::<TestStorage>::contains_key((
            &bob_portfolio,
            asset_id,
            NFTId(1)
        )));
        assert_eq!(
            NumberOfNFTs::<TestStorage>::get(nfts.asset_id(), alice.did),
            0
        );
        assert!(!PortfolioNFT::<TestStorage>::contains_key((
            &alice_portfolio,
            asset_id,
            NFTId(1)
        )));
        // Calls controller transfer
        assert_ok!(NFT::controller_transfer(
            alice.origin(),
            nfts.clone(),
            bob_portfolio.clone().into(),
            AssetHolderKind::DefaultPortfolio
        ));
        assert_eq!(
            NumberOfNFTs::<TestStorage>::get(nfts.asset_id(), bob.did),
            0
        );
        assert!(!PortfolioNFT::<TestStorage>::contains_key((
            &bob_portfolio,
            asset_id,
            NFTId(1)
        )));
        assert_eq!(
            NumberOfNFTs::<TestStorage>::get(nfts.asset_id(), alice.did),
            1
        );
        assert!(PortfolioNFT::<TestStorage>::contains_key((
            &alice_portfolio,
            asset_id,
            NFTId(1)
        )));
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_portfolio.clone().into())
        );
        assert_eq!(
            super::storage::EventTest::Nft(Event::NFTHoldingsUpdated(
                alice.did,
                nfts,
                Some(bob_portfolio.into()),
                Some(alice_portfolio.into()),
                HoldingsUpdateReason::ControllerTransfer
            )),
            System::events().last().unwrap().event,
        );
    });
}

#[test]
fn controller_transfer_unauthorized_agent() {
    ExtBuilder::default().build().execute_with(|| {
        // First we need to create a collection and mint one NFT
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new().into(),
        );
        mint_nft(
            alice.clone(),
            asset_id.clone(),
            Vec::new(),
            AssetHolderKind::DefaultPortfolio,
        );
        ComplianceManager::pause_asset_compliance(alice.origin(), asset_id.clone()).unwrap();
        // Calls controller transfer
        let bob_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        assert_noop!(
            NFT::controller_transfer(
                bob.origin(),
                NFTs::new(asset_id, vec![NFTId(1)]).unwrap(),
                bob_portfolio.into(),
                AssetHolderKind::DefaultPortfolio
            ),
            EAError::UnauthorizedAgent
        );
    });
}

#[test]
fn controller_transfer_nft_not_owned() {
    ExtBuilder::default().build().execute_with(|| {
        // First we need to create a collection and mint one NFT
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new().into(),
        );
        mint_nft(
            alice.clone(),
            asset_id.clone(),
            Vec::new(),
            AssetHolderKind::DefaultPortfolio,
        );
        ComplianceManager::pause_asset_compliance(alice.origin(), asset_id.clone()).unwrap();
        // Calls controller transfer
        let bob_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        assert_noop!(
            NFT::controller_transfer(
                alice.origin(),
                NFTs::new(asset_id, vec![NFTId(1)]).unwrap(),
                bob_portfolio.into(),
                AssetHolderKind::DefaultPortfolio
            ),
            NFTError::InvalidNFTTransferInsufficientCount
        );
    });
}

#[test]
fn redeem_wrong_number_of_keys() {
    ExtBuilder::default().build().execute_with(|| {
        let alice: User = User::new(Sr25519Keyring::Alice);

        let collection_keys: NFTCollectionKeys = vec![
            AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(2)),
        ]
        .into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![
            NFTMetadataAttribute {
                key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
                value: AssetMetadataValue(b"test".to_vec()),
            },
            NFTMetadataAttribute {
                key: AssetMetadataKey::Local(AssetMetadataLocalKey(2)),
                value: AssetMetadataValue(b"test".to_vec()),
            },
        ];
        mint_nft(
            alice.clone(),
            asset_id,
            nfts_metadata,
            AssetHolderKind::DefaultPortfolio,
        );

        assert_noop!(
            NFT::redeem_nft(
                alice.origin(),
                asset_id,
                NFTId(1),
                AssetHolderKind::DefaultPortfolio,
                Some(1)
            ),
            NFTError::NumberOfKeysIsLessThanExpected
        );
    });
}

#[test]
fn redeem_locked_nft() {
    ExtBuilder::default().build().execute_with(|| {
        let bob: User = User::new(Sr25519Keyring::Bob);
        let alice: User = User::new(Sr25519Keyring::Alice);

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new().into(),
        );
        mint_nft(
            alice.clone(),
            asset_id,
            Vec::new(),
            AssetHolderKind::DefaultPortfolio,
        );

        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts: NFTs::new_unverified(asset_id, vec![NFTId(1)]),
        }];
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            None,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs,
            default_asset_holder_set(alice.did),
            None,
        ));

        assert_noop!(
            NFT::redeem_nft(
                alice.origin(),
                asset_id,
                NFTId(1),
                AssetHolderKind::DefaultPortfolio,
                None
            ),
            NFTError::NFTIsLocked
        );
    });
}

#[test]
fn reject_instruction_with_locked_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let bob: User = User::new(Sr25519Keyring::Bob);
        let alice: User = User::new(Sr25519Keyring::Alice);

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new().into(),
        );
        mint_nft(
            alice.clone(),
            asset_id,
            Vec::new(),
            AssetHolderKind::DefaultPortfolio,
        );

        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts: NFTs::new_unverified(asset_id, vec![NFTId(1)]),
        }];
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            None,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs,
            default_asset_holder_set(alice.did),
            None,
        ));

        // Force token redemption
        pallet_portfolio::PortfolioLockedNFT::<TestStorage>::remove(
            PortfolioId::default_portfolio(alice.did),
            (asset_id, NFTId(1)),
        );

        assert_noop!(
            Settlement::reject_instruction(
                alice.origin(),
                InstructionId(0),
                PortfolioId::default_portfolio(alice.did).into(),
            ),
            NFTError::NFTIsNotLocked
        );
    });
}
