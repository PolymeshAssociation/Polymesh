use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use scale_info::prelude::format;
use sp_std::prelude::*;
use sp_std::vec::Vec;

use pallet_asset::benchmarking::create_portfolio;
use polymesh_common_utilities::benchs::{
    create_and_issue_sample_asset, user, AccountIdOf, User, UserBuilder,
};
use polymesh_common_utilities::traits::asset::AssetFnTrait;
use polymesh_common_utilities::traits::compliance_manager::ComplianceFnConfig;
use polymesh_common_utilities::{with_transaction, TestUtilsFn};
use polymesh_primitives::asset::{AssetType, NonFungibleType};
use polymesh_primitives::asset_metadata::{
    AssetMetadataGlobalKey, AssetMetadataKey, AssetMetadataSpec, AssetMetadataValue,
};
use polymesh_primitives::nft::{NFTCollectionId, NFTCollectionKeys, NFTId};
use polymesh_primitives::{IdentityId, PortfolioKind, WeightMeter};

use crate::*;

const MAX_COLLECTION_KEYS: u32 = 255;

/// Creates an NFT collection with `n` global metadata keys.
fn create_collection<T: Config>(collection_owner: &User<T>, n: u32) -> (AssetID, NFTCollectionId) {
    let asset_id = create_and_issue_sample_asset::<T>(
        collection_owner,
        false,
        Some(AssetType::NonFungible(NonFungibleType::Invoice)),
        b"MyNFT",
        false,
    );
    let collection_keys: NFTCollectionKeys = creates_keys_register_metadata_types::<T>(n);
    Module::<T>::create_nft_collection(
        collection_owner.origin.clone().into(),
        asset_id,
        collection_keys,
    )
    .expect("failed to create nft collection");
    (asset_id, Module::<T>::current_collection_id().unwrap())
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

/// Creates an NFT collection with `n_keys` global metadata keys and issues `n_nfts`.
fn create_collection_issue_nfts<T: Config>(
    collection_owner: &User<T>,
    n_keys: u32,
    n_nfts: u32,
    portfolio_kind: PortfolioKind,
) -> AssetID {
    let (asset_id, nft_collection_id) = create_collection::<T>(collection_owner, n_keys);

    let metadata_attributes: Vec<NFTMetadataAttribute> = (1..n_keys + 1)
        .map(|key| NFTMetadataAttribute {
            key: AssetMetadataKey::Global(AssetMetadataGlobalKey(key.into())),
            value: AssetMetadataValue(b"value".to_vec()),
        })
        .collect();
    for _ in 0..n_nfts {
        Module::<T>::issue_nft(
            collection_owner.origin.clone().into(),
            asset_id,
            metadata_attributes.clone(),
            portfolio_kind,
        )
        .expect("failed to mint nft");
    }

    asset_id
}

/// Creates one NFT collection, mints `n_nfts` for that collection and
/// sets up compliance rules.
pub fn setup_nft_transfer<T>(
    sender: &User<T>,
    receiver: &User<T>,
    n_nfts: u32,
    sender_portfolio_name: Option<&str>,
    receiver_portolfio_name: Option<&str>,
    pause_compliance: bool,
    n_mediators: u8,
) -> (AssetID, PortfolioId, PortfolioId, Vec<User<T>>)
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    let sender_portfolio =
        create_portfolio::<T>(sender, sender_portfolio_name.unwrap_or("SenderPortfolio"));
    let receiver_portfolio =
        create_portfolio::<T>(receiver, receiver_portolfio_name.unwrap_or("RcvPortfolio"));

    let asset_id = create_collection_issue_nfts::<T>(sender, 0, n_nfts, sender_portfolio.kind);

    // Sets mandatory mediators
    let mut asset_mediators = Vec::new();
    if n_mediators > 0 {
        let mediators_identity: BTreeSet<IdentityId> = (0..n_mediators)
            .map(|i| {
                let mediator = UserBuilder::<T>::default()
                    .generate_did()
                    .build(&format!("Mediator{:?}{}", asset_id, i));
                asset_mediators.push(mediator.clone());
                mediator.did()
            })
            .collect();
        T::AssetFn::add_mandatory_mediators(sender.origin().into(), asset_id, mediators_identity)
            .unwrap();
    }

    // Adds the maximum number of compliance requirement
    T::Compliance::setup_asset_compliance(sender.did(), asset_id, 50, pause_compliance);

    (
        asset_id,
        sender_portfolio,
        receiver_portfolio,
        asset_mediators,
    )
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    create_nft_collection {
        let n in 1..MAX_COLLECTION_KEYS;

        let user = user::<T>("target", 0);
        let asset_id = create_and_issue_sample_asset::<T>(
            &user,
            false,
            Some(AssetType::NonFungible(NonFungibleType::Invoice)),
            b"MyNFT",
            false,
        );
        let collection_keys: NFTCollectionKeys = creates_keys_register_metadata_types::<T>(n);
    }: _(user.origin, asset_id, collection_keys)
    verify {
        assert!(Collection::contains_key(NFTCollectionId(1)));
        assert_eq!(CollectionKeys::get(NFTCollectionId(1)).len(), n as usize);
    }

    issue_nft {
        let n in 1..MAX_COLLECTION_KEYS;

        let user = user::<T>("target", 0);
        let (asset_id, collection_id) = create_collection::<T>(&user, n);
        let metadata_attributes: Vec<NFTMetadataAttribute> = (1..n + 1)
            .map(|key| {
                NFTMetadataAttribute{
                    key: AssetMetadataKey::Global(AssetMetadataGlobalKey(key.into())),
                    value: AssetMetadataValue(b"value".to_vec()),
                }
            })
            .collect();
    }: _(user.origin, asset_id, metadata_attributes, PortfolioKind::Default)
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

    redeem_nft {
        let n in 1..MAX_COLLECTION_KEYS;

        let user = user::<T>("target", 0);
        let asset_id = create_collection_issue_nfts::<T>(&user, n, 1, PortfolioKind::Default);

    }: _(user.origin, asset_id, NFTId(1), PortfolioKind::Default)
    verify {
        for i in 1..n + 1 {
            assert!(
                !MetadataValue::contains_key(
                    (NFTCollectionId(1), NFTId(1)),
                    AssetMetadataKey::Global(AssetMetadataGlobalKey(i.into()))
                )
            );
        }
    }

    base_nft_transfer {
        // The weight depends on the number of ids in the `NFTs` vec and the complexity of the compliance rules.
        // Since the compliance weight will be charged separately, the rules were paused and only the `Self::asset_compliance(ticker)`
        // read will be considered (this read was not charged in the is_condition_satisfied benchmark).

        let n in 1..10;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let (asset_id, sender_portfolio, receiver_portfolio, _) =
            setup_nft_transfer::<T>(&alice, &bob, n, None, None, true, 0);
        let nfts = NFTs::new_unverified(asset_id, (0..n).map(|i| NFTId((i + 1) as u64)).collect());
    }: {
        with_transaction(|| {
            Module::<T>::base_nft_transfer(
                sender_portfolio,
                receiver_portfolio,
                nfts,
                InstructionId(1),
                None,
                IdentityId::default(),
                &mut weight_meter
            )
        })
        .unwrap();
    }

    controller_transfer {
        let n in 1..T::MaxNumberOfNFTsCount::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let (asset_id, alice_user_portfolio, bob_user_portfolio, _) =
            setup_nft_transfer::<T>(&alice, &bob, n, None, None, true, 0);
        let nfts = NFTs::new_unverified(asset_id, (0..n).map(|i| NFTId((i + 1) as u64)).collect());
        with_transaction(|| {
            Module::<T>::base_nft_transfer(
                alice_user_portfolio,
                bob_user_portfolio,
                nfts.clone(),
                InstructionId(1),
                None,
                IdentityId::default(),
                &mut weight_meter
            )
        })
        .unwrap();
        // Before the controller transfer all NFTs belong to bob
        assert_eq!(NumberOfNFTs::get(nfts.asset_id(), bob.did()), n as u64);
        assert_eq!(NumberOfNFTs::get(nfts.asset_id(), alice.did()), 0);
    }: _(alice.origin.clone(), nfts.clone(), bob_user_portfolio, alice_user_portfolio.kind)
    verify {
        assert_eq!(NumberOfNFTs::get(nfts.asset_id(), bob.did()), 0);
        assert_eq!(NumberOfNFTs::get(nfts.asset_id(), alice.did()), n as u64);
        for i in 1..n + 1 {
            assert!(PortfolioNFT::contains_key(alice_user_portfolio, (asset_id, NFTId(i.into()))));
            assert!(!PortfolioNFT::contains_key(bob_user_portfolio, (asset_id, NFTId(i.into()))));
        }
        assert_eq!(NFTsInCollection::get(nfts.asset_id()), n as u64);
    }

}
