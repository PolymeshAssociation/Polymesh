use frame_benchmarking::benchmarks;
use scale_info::prelude::format;
use sp_std::prelude::*;
use sp_std::vec::Vec;

use codec::Encode;
use pallet_asset::benchmarking::create_portfolio;
use pallet_identity::benchmarking::{user, User, UserBuilder};
use polymesh_primitives::asset::{AssetHolder, AssetHolderKind, AssetType, NonFungibleType};
use polymesh_primitives::asset_metadata::{
    AssetMetadataGlobalKey, AssetMetadataKey, AssetMetadataSpec, AssetMetadataValue,
};
use polymesh_primitives::bench::create_and_issue_sample_asset;
use polymesh_primitives::nft::{NFTCollectionId, NFTCollectionKeys, NFTId};
use polymesh_primitives::traits::{AssetFnTrait, ComplianceFnConfig};
use polymesh_primitives::{with_transaction, IdentityId, WeightMeter};

use crate::*;

const MAX_COLLECTION_KEYS: u32 = 255;

/// Creates an NFT collection with `n` global metadata keys.
fn create_collection<T: Config>(collection_owner: &User<T>, n: u32) -> (AssetId, NFTCollectionId) {
    let asset_id = create_and_issue_sample_asset::<T>(
        collection_owner.account(),
        false,
        Some(AssetType::NonFungible(NonFungibleType::Invoice)),
        b"MyNFT",
        false,
    );
    let collection_keys: NFTCollectionKeys = creates_keys_register_metadata_types::<T>(n);
    Pallet::<T>::create_nft_collection(
        collection_owner.origin.clone().into(),
        Some(asset_id),
        None,
        collection_keys,
    )
    .expect("failed to create nft collection");
    (asset_id, CurrentCollectionId::<T>::get().unwrap())
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
    asset_holder_kind: AssetHolderKind,
) -> AssetId {
    let (asset_id, _) = create_collection::<T>(collection_owner, n_keys);

    let metadata_attributes: Vec<NFTMetadataAttribute> = (1..n_keys + 1)
        .map(|key| NFTMetadataAttribute {
            key: AssetMetadataKey::Global(AssetMetadataGlobalKey(key.into())),
            value: AssetMetadataValue(b"value".to_vec()),
        })
        .collect();
    for _ in 0..n_nfts {
        Pallet::<T>::issue_nft(
            collection_owner.origin.clone().into(),
            asset_id,
            metadata_attributes.clone(),
            asset_holder_kind.clone(),
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
    use_account_portfolio: bool,
) -> (AssetId, AssetHolder, AssetHolder, Vec<User<T>>)
where
    T: Config,
{
    let (sender_holdings, receiver_holdings) = if use_account_portfolio {
        (
            AssetHolder::try_from(sender.account().encode()).unwrap(),
            AssetHolder::try_from(receiver.account().encode()).unwrap(),
        )
    } else {
        (
            create_portfolio::<T>(sender, sender_portfolio_name.unwrap_or("SenderPortfolio")),
            create_portfolio::<T>(receiver, receiver_portolfio_name.unwrap_or("RcvPortfolio")),
        )
    };

    let asset_id =
        create_collection_issue_nfts::<T>(sender, 0, n_nfts, sender_holdings.clone().into());

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
        T::AssetFn::add_mandatory_mediators(sender.account(), asset_id, mediators_identity)
            .unwrap();
    }

    // Adds the maximum number of compliance requirement
    T::Compliance::setup_asset_compliance(sender.did(), asset_id, 50, pause_compliance);

    (
        asset_id,
        sender_holdings,
        receiver_holdings,
        asset_mediators,
    )
}

benchmarks! {
    create_nft_collection {
        let n in 1..MAX_COLLECTION_KEYS;

        let user = user::<T>("target", 0);
        let nft_type: Option<NonFungibleType> = Some(NonFungibleType::Derivative);
        let collection_keys: NFTCollectionKeys = creates_keys_register_metadata_types::<T>(n);
    }: _(user.origin, None, nft_type, collection_keys)
    verify {
        assert!(Collection::<T>::contains_key(NFTCollectionId(1)));
        assert_eq!(CollectionKeys::<T>::get(NFTCollectionId(1)).len(), n as usize);
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
    }: _(user.origin, asset_id, metadata_attributes, AssetHolderKind::DefaultPortfolio)
    verify {
        for i in 1..n + 1 {
            assert!(
                MetadataValue::<T>::contains_key(
                    (NFTCollectionId(1), NFTId(1)),
                    AssetMetadataKey::Global(AssetMetadataGlobalKey(i.into()))
                )
            );
        }
    }

    redeem_nft {
        let n in 1..MAX_COLLECTION_KEYS;

        let user = user::<T>("target", 0);
        let asset_id = create_collection_issue_nfts::<T>(&user, n, 1, AssetHolderKind::DefaultPortfolio);

    }: _(user.origin, asset_id, NFTId(1), AssetHolderKind::DefaultPortfolio, None)
    verify {
        for i in 1..n + 1 {
            assert!(
                !MetadataValue::<T>::contains_key(
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

        let (asset_id, sender, receiver, _) =
            setup_nft_transfer::<T>(&alice, &bob, n, None, None, true, 0, false);
        let nfts = NFTs::new_unverified(asset_id, (0..n).map(|i| NFTId((i + 1) as u64)).collect());
    }: {
        with_transaction(|| {
            Pallet::<T>::base_nft_transfer(
                sender,
                receiver,
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

        let (asset_id, alice_holdings, bob_holdings, _) =
            setup_nft_transfer::<T>(&alice, &bob, n, None, None, true, 0, false);
        let nfts = NFTs::new_unverified(asset_id, (0..n).map(|i| NFTId((i + 1) as u64)).collect());
        with_transaction(|| {
            Pallet::<T>::base_nft_transfer(
                alice_holdings.clone(),
                bob_holdings.clone(),
                nfts.clone(),
                InstructionId(1),
                None,
                IdentityId::default(),
                &mut weight_meter
            )
        })
        .unwrap();
        // Before the controller transfer all NFTs belong to bob
        assert_eq!(NumberOfNFTs::<T>::get(nfts.asset_id(), bob.did()), n as u64);
        assert_eq!(NumberOfNFTs::<T>::get(nfts.asset_id(), alice.did()), 0);
    }: _(alice.origin.clone(), nfts.clone(), bob_holdings.clone(), alice_holdings.clone().into())
    verify {
        assert_eq!(NumberOfNFTs::<T>::get(nfts.asset_id(), bob.did()), 0);
        assert_eq!(NumberOfNFTs::<T>::get(nfts.asset_id(), alice.did()), n as u64);
        for i in 1..n + 1 {
            assert!(Pallet::<T>::is_holder_of_nft(&asset_id, &NFTId(i.into()), &alice_holdings));
            assert!(!Pallet::<T>::is_holder_of_nft(&asset_id, &NFTId(i.into()), &bob_holdings));
        }
        assert_eq!(NFTsInCollection::<T>::get(nfts.asset_id()), n as u64);
    }

    controller_transfer_to {
        let n in 1..T::MaxNumberOfNFTsCount::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let (asset_id, alice_holdings, bob_holdings, _) =
            setup_nft_transfer::<T>(&alice, &bob, n, None, None, true, 0, false);
        let nfts = NFTs::new_unverified(asset_id, (0..n).map(|i| NFTId((i + 1) as u64)).collect());
        with_transaction(|| {
            Pallet::<T>::base_nft_transfer(
                alice_holdings.clone(),
                bob_holdings.clone(),
                nfts.clone(),
                InstructionId(1),
                None,
                IdentityId::default(),
                &mut weight_meter
            )
        })
        .unwrap();
        // Before the controller transfer all NFTs belong to bob
        assert_eq!(NumberOfNFTs::<T>::get(nfts.asset_id(), bob.did()), n as u64);
        assert_eq!(NumberOfNFTs::<T>::get(nfts.asset_id(), alice.did()), 0);
    }: _(alice.origin.clone(), nfts.clone(), bob_holdings.clone(), alice_holdings.clone())
    verify {
        assert_eq!(NumberOfNFTs::<T>::get(nfts.asset_id(), bob.did()), 0);
        assert_eq!(NumberOfNFTs::<T>::get(nfts.asset_id(), alice.did()), n as u64);
        for i in 1..n + 1 {
            assert!(Pallet::<T>::is_holder_of_nft(&asset_id, &NFTId(i.into()), &alice_holdings));
            assert!(!Pallet::<T>::is_holder_of_nft(&asset_id, &NFTId(i.into()), &bob_holdings));
        }
        assert_eq!(NFTsInCollection::<T>::get(nfts.asset_id()), n as u64);
    }

    approve {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice_holdings = AssetHolder::try_from(alice.account().encode()).unwrap();
        let asset_id = create_collection_issue_nfts::<T>(&alice, 0, 1, alice_holdings.into());
        let spender = Pallet::<T>::to_account_id32(&bob.account()).unwrap();
        // Worst case: overwrite an existing approval.
        TokenApproval::<T>::insert(asset_id, NFTId(1), &spender);
    }: _(alice.origin.clone(), asset_id, NFTId(1), Some(bob.account()))
    verify {
        assert_eq!(TokenApproval::<T>::get(asset_id, NFTId(1)), Some(spender));
    }

    set_approval_for_all {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice_holdings = AssetHolder::try_from(alice.account().encode()).unwrap();
        let asset_id = create_collection_issue_nfts::<T>(&alice, 0, 1, alice_holdings.into());
        let owner = Pallet::<T>::to_account_id32(&alice.account()).unwrap();
        let operator = Pallet::<T>::to_account_id32(&bob.account()).unwrap();
    }: _(alice.origin.clone(), asset_id, bob.account(), true)
    verify {
        assert!(OperatorApproval::<T>::get((&owner, &operator, &asset_id)));
    }

    spend_nft_approval {
        let n in 1..T::MaxNumberOfNFTsCount::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice_holdings = AssetHolder::try_from(alice.account().encode()).unwrap();
        let asset_id = create_collection_issue_nfts::<T>(&alice, 0, n, alice_holdings.into());

        let owner = Pallet::<T>::to_account_id32(&alice.account()).unwrap();
        let spender = Pallet::<T>::to_account_id32(&bob.account()).unwrap();
        // Worst case: no operator approval, so every per-token approval is read and consumed.
        for i in 1..n + 1 {
            TokenApproval::<T>::insert(asset_id, NFTId(i.into()), &spender);
        }
        let nfts = NFTs::new_unverified(asset_id, (1..n + 1).map(|i| NFTId(i.into())).collect());
    }: {
        Pallet::<T>::spend_nft_approval(&owner, &bob.account(), &nfts).unwrap();
    }
    verify {
        for i in 1..n + 1 {
            assert!(TokenApproval::<T>::get(asset_id, NFTId(i.into())).is_none());
        }
    }

}
