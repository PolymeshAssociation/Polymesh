use chrono::prelude::Utc;
use frame_support::storage::StorageValue;
use frame_support::{assert_noop, assert_ok, StorageDoubleMap, StorageMap};

use pallet_nft::{
    Collection, CollectionKeys, CurrentCollectionId, CurrentNFTId, MetadataValue, NFTOwner,
    NFTsInCollection, NextCollectionId, NextNFTId, NumberOfNFTs,
};
use pallet_portfolio::PortfolioNFT;
use polymesh_common_utilities::traits::nft::Event;
use polymesh_common_utilities::with_transaction;
use polymesh_primitives::asset::{AssetID, AssetName, AssetType, NonFungibleType};
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataSpec,
    AssetMetadataValue,
};
use polymesh_primitives::settlement::InstructionId;
use polymesh_primitives::{
    AuthorizationData, Claim, ClaimType, Condition, ConditionType, CountryCode, IdentityId,
    NFTCollectionId, NFTCollectionKeys, NFTId, NFTMetadataAttribute, NFTs, PortfolioId,
    PortfolioKind, PortfolioNumber, PortfolioUpdateReason, Scope, Signatory, TrustedFor,
    TrustedIssuer, WeightMeter,
};
use sp_keyring::AccountKeyring;

use super::asset_test::{get_security_token, set_timestamp};
use crate::asset_pallet::setup::{create_and_issue_sample_asset, create_and_issue_sample_nft};
use crate::ext_builder::ExtBuilder;
use crate::storage::{TestStorage, User};

type Asset = pallet_asset::Module<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;
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
        let nft_type = NonFungibleType::Derivative;
        let collection_keys: NFTCollectionKeys = vec![].into();

        let asset_id = Asset::generate_asset_id(alice.did, false);
        assert_ok!(NFT::create_nft_collection(
            alice.origin(),
            None,
            Some(nft_type),
            collection_keys
        ));
        assert_eq!(get_security_token(&asset_id).divisible, false);
        assert_eq!(
            get_security_token(&asset_id).asset_type,
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

        let alice: User = User::new(AccountKeyring::Alice);
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

        let alice: User = User::new(AccountKeyring::Alice);
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

        let alice: User = User::new(AccountKeyring::Alice);
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

        let alice: User = User::new(AccountKeyring::Alice);

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
) -> AssetID {
    let asset_id = Asset::generate_asset_id(owner.did, false);
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
    assert!(Collection::contains_key(NFTCollectionId(1)));
    assert_eq!(CollectionKeys::get(NFTCollectionId(1)).len(), n_keys);
    assert_eq!(NextCollectionId::get(), NFTCollectionId(1));
    assert_eq!(CurrentCollectionId::get(), Some(NFTCollectionId(1)));

    asset_id
}

/// An NFT can only be minted if its collection exists.
#[test]
fn mint_nft_collection_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        assert_noop!(
            NFT::issue_nft(
                alice.origin(),
                [0; 16].into(),
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
                PortfolioKind::Default
            ),
            NFTError::InvalidMetadataAttribute
        );
        assert_noop!(
            NFT::issue_nft(alice.origin(), asset_id, vec![], PortfolioKind::Default),
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
            PortfolioKind::Default
        ));
        assert_eq!(
            MetadataValue::get(
                (NFTCollectionId(1), NFTId(1)),
                AssetMetadataKey::Local(AssetMetadataLocalKey(1))
            ),
            AssetMetadataValue(b"test".to_vec())
        );
        assert_eq!(NumberOfNFTs::get(&asset_id, alice.did), 1);
        assert_eq!(NFTsInCollection::get(&asset_id), 1);
        assert_eq!(
            PortfolioNFT::get(
                PortfolioId::default_portfolio(alice.did),
                (&asset_id, NFTId(1))
            ),
            true
        );
        assert_eq!(
            NFTOwner::get(asset_id, NFTId(1)),
            Some(alice_default_portfolio)
        );
        assert_eq!(NextNFTId::get(NFTCollectionId(1)), NFTId(1));
        assert_eq!(CurrentNFTId::get(NFTCollectionId(1)), Some(NFTId(1)));
    });
}

pub(crate) fn mint_nft(
    user: User,
    asset_id: AssetID,
    metadata_atributes: Vec<NFTMetadataAttribute>,
    portfolio_kind: PortfolioKind,
) {
    assert_ok!(NFT::issue_nft(
        user.origin(),
        asset_id,
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
        assert_noop!(
            NFT::redeem_nft(
                alice.origin(),
                Asset::generate_asset_id(alice.did, false),
                NFTId(1),
                PortfolioKind::Default
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

        let alice: User = User::new(AccountKeyring::Alice);

        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );

        assert_noop!(
            NFT::redeem_nft(alice.origin(), asset_id, NFTId(1), PortfolioKind::Default),
            NFTError::NFTNotFound
        );
    });
}

/// An NFT can only be burned if the caller has custody over the portfolio.
#[test]
fn burn_nft_no_custody() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let bob: User = User::new(AccountKeyring::Bob);
        let alice: User = User::new(AccountKeyring::Alice);

        let portfolio_id = PortfolioId::new(alice.did, PortfolioKind::Default);
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();

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
            PortfolioKind::Default,
        )
        .unwrap();

        assert_noop!(
            NFT::redeem_nft(alice.origin(), asset_id, NFTId(1), PortfolioKind::Default),
            PortfolioError::UnauthorizedCustodian
        );
    });
}

/// Successfully burns an NFT.
#[test]
fn burn_nft() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);

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
            PortfolioKind::Default,
        )
        .unwrap();

        assert_ok!(NFT::redeem_nft(
            alice.origin(),
            asset_id,
            NFTId(1),
            PortfolioKind::Default
        ));
        assert!(!MetadataValue::contains_key(
            (NFTCollectionId(1), NFTId(1)),
            AssetMetadataKey::Local(AssetMetadataLocalKey(1))
        ),);
        assert_eq!(NumberOfNFTs::get(&asset_id, alice.did), 0);
        assert_eq!(NFTsInCollection::get(&asset_id), 0);
        assert!(!PortfolioNFT::contains_key(
            PortfolioId::default_portfolio(alice.did),
            (&asset_id, NFTId(1))
        ),);
        assert_eq!(NFTOwner::get(asset_id, NFTId(1)), None);
        assert_eq!(NextNFTId::get(NFTCollectionId(1)), NFTId(1));
        assert_eq!(CurrentNFTId::get(NFTCollectionId(1)), Some(NFTId(1)));
        assert_eq!(NextCollectionId::get(), NFTCollectionId(1));
        assert_eq!(CurrentCollectionId::get(), Some(NFTCollectionId(1)));
    });
}

/// An NFT can only be transferred if its collection exists.
#[test]
fn transfer_nft_without_collection() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(Utc::now().timestamp() as _);

        let alice: User = User::new(AccountKeyring::Alice);
        let bob: User = User::new(AccountKeyring::Bob);

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let sender_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let receiver_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let nfts = NFTs::new(Asset::generate_asset_id(alice.did, false), vec![NFTId(1)]).unwrap();

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
                    sender_portfolio,
                    receiver_portfolio,
                    nfts,
                    InstructionId(0),
                    None,
                    IdentityId::default(),
                    &mut weight_meter,
                )
            }),
            NFTError::InvalidNFTTransferSenderIdMatchesReceiverId
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
        let nfts = NFTs::new(asset_id, vec![NFTId(1), NFTId(2)]).unwrap();
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
        let nfts = NFTs::new(asset_id, vec![NFTId(1)]).unwrap();
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
        let bob: User = User::new(AccountKeyring::Bob);
        let dave: User = User::new(AccountKeyring::Dave);
        let alice: User = User::new(AccountKeyring::Alice);

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
            PortfolioKind::Default,
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
            PortfolioKind::Default,
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
                sender_portfolio,
                receiver_portfolio,
                nfts.clone(),
                InstructionId(0),
                None,
                IdentityId::default(),
                &mut weight_meter,
            )
        }));
        assert_eq!(NumberOfNFTs::get(&asset_id, alice.did), 0);
        assert_eq!(
            PortfolioNFT::get(
                PortfolioId::default_portfolio(alice.did),
                (&asset_id, NFTId(1))
            ),
            false
        );
        assert_eq!(NumberOfNFTs::get(&asset_id, bob.did), 1);
        assert_eq!(NFTsInCollection::get(&asset_id), 1);
        assert_eq!(
            PortfolioNFT::get(
                PortfolioId::default_portfolio(bob.did),
                (&asset_id, NFTId(1))
            ),
            true
        );
        assert_eq!(NFTOwner::get(asset_id, NFTId(1)), Some(receiver_portfolio));
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

/// Successfully transfer an NFT using the controller transfer.
#[test]
fn controller_transfer() {
    ExtBuilder::default().build().execute_with(|| {
        // First we need to create a collection and mint one NFT
        set_timestamp(Utc::now().timestamp() as _);
        System::set_block_number(1);
        let alice: User = User::new(AccountKeyring::Alice);
        let bob: User = User::new(AccountKeyring::Bob);

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
            PortfolioKind::Default,
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
                alice_portfolio,
                bob_portfolio,
                nfts.clone(),
                InstructionId(0),
                None,
                IdentityId::default(),
                &mut weight_meter,
            )
        }));
        // Before the controller transfer all NFTs belong to bob
        assert_eq!(NumberOfNFTs::get(nfts.asset_id(), bob.did), 1);
        assert!(PortfolioNFT::contains_key(
            bob_portfolio,
            (asset_id, NFTId(1))
        ));
        assert_eq!(NumberOfNFTs::get(nfts.asset_id(), alice.did), 0);
        assert!(!PortfolioNFT::contains_key(
            alice_portfolio,
            (asset_id, NFTId(1))
        ));
        // Calls controller transfer
        assert_ok!(NFT::controller_transfer(
            alice.origin(),
            nfts.clone(),
            bob_portfolio,
            alice_portfolio.kind
        ));
        assert_eq!(NumberOfNFTs::get(nfts.asset_id(), bob.did), 0);
        assert!(!PortfolioNFT::contains_key(
            bob_portfolio,
            (asset_id, NFTId(1))
        ));
        assert_eq!(NumberOfNFTs::get(nfts.asset_id(), alice.did), 1);
        assert!(PortfolioNFT::contains_key(
            alice_portfolio,
            (asset_id, NFTId(1))
        ));
        assert_eq!(NFTOwner::get(asset_id, NFTId(1)), Some(alice_portfolio));
        assert_eq!(
            super::storage::EventTest::Nft(Event::NFTPortfolioUpdated(
                alice.did,
                nfts,
                Some(bob_portfolio),
                Some(alice_portfolio),
                PortfolioUpdateReason::ControllerTransfer
            )),
            System::events().last().unwrap().event,
        );
    });
}

#[test]
fn controller_transfer_unauthorized_agent() {
    ExtBuilder::default().build().execute_with(|| {
        // First we need to create a collection and mint one NFT
        let alice: User = User::new(AccountKeyring::Alice);
        let bob: User = User::new(AccountKeyring::Bob);

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new().into(),
        );
        mint_nft(
            alice.clone(),
            asset_id.clone(),
            Vec::new(),
            PortfolioKind::Default,
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
                bob_portfolio,
                PortfolioKind::Default
            ),
            EAError::UnauthorizedAgent
        );
    });
}

#[test]
fn controller_transfer_nft_not_owned() {
    ExtBuilder::default().build().execute_with(|| {
        // First we need to create a collection and mint one NFT
        let alice: User = User::new(AccountKeyring::Alice);
        let bob: User = User::new(AccountKeyring::Bob);

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new().into(),
        );
        mint_nft(
            alice.clone(),
            asset_id.clone(),
            Vec::new(),
            PortfolioKind::Default,
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
                bob_portfolio,
                PortfolioKind::Default
            ),
            NFTError::InvalidNFTTransferInsufficientCount
        );
    });
}
