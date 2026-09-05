// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use scale_info::prelude::format;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::{convert::TryInto, iter, prelude::*};

use pallet_identity::benchmarking::{user, User, UserBuilder};
use pallet_portfolio::NextPortfolioNumber;
use pallet_statistics::benchmarking::{setup_statistics, setup_transfer_restrictions};
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::asset::{AssetHolder, AssetHolderKind, AssetName, NonFungibleType};
use polymesh_primitives::asset_metadata::{
    AssetMetadataDescription, AssetMetadataKey, AssetMetadataName, AssetMetadataSpec,
    AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::bench::reg_unique_ticker;
use polymesh_primitives::constants::currency::{ONE_UNIT, POLY};
use polymesh_primitives::settlement::AffirmationRequirement;
use polymesh_primitives::ticker::TICKER_LEN;
use polymesh_primitives::traits::{ComplianceFnConfig, NFTTrait};
use polymesh_primitives::{
    AuthorizationData, Fund, FundDescription, IdentityId, NFTCollectionKeys, PortfolioId,
    PortfolioKind, PortfolioName, PortfolioNumber, Signatory, Ticker, Url, WeightMeter,
};

use crate::*;

const MAX_DOCS_PER_ASSET: u32 = 64;
const MAX_DOC_URI: usize = 1024;
const MAX_DOC_NAME: usize = 1024;
const MAX_DOC_TYPE: usize = 1024;
const MAX_IDENTIFIERS_PER_ASSET: u32 = 512;

pub fn make_document() -> Document {
    Document {
        uri: [b'u'; MAX_DOC_URI].into(),
        content_hash: b"572cdd8d8f1754dd0c4a75d99b569845"[..].try_into().unwrap(), // MD5 output is 128bits.
        name: [b'n'; MAX_DOC_NAME].into(),
        doc_type: Some([b't'; MAX_DOC_TYPE].into()),
        filing_date: None,
    }
}

/// Make metadata name for benchmarking.
fn make_metadata_name<T: Config>() -> AssetMetadataName {
    AssetMetadataName(vec![b'n'; T::AssetMetadataNameMaxLength::get() as usize])
}

/// Make metadata value for benchmarking.
fn make_metadata_value<T: Config>() -> AssetMetadataValue {
    AssetMetadataValue(vec![b'v'; T::AssetMetadataValueMaxLength::get() as usize])
}

/// Make metadata spec for benchmarking.
fn make_metadata_spec<T: Config>() -> AssetMetadataSpec {
    AssetMetadataSpec {
        url: Some(Url(vec![b'u'; T::MaxLen::get() as usize])),
        description: Some(AssetMetadataDescription(vec![
            b'd';
            T::MaxLen::get() as usize
        ])),
        type_def: Some(vec![b'x'; T::AssetMetadataTypeDefMaxLength::get() as usize]),
    }
}

/// Register a global metadata type for benchmarking.
fn register_metadata_global_name<T: AssetConfig>() -> AssetMetadataKey {
    let root = RawOrigin::Root.into();
    let name = make_metadata_name::<T>();
    let spec = make_metadata_spec::<T>();

    Pallet::<T>::register_asset_metadata_global_type(root, name, spec).unwrap();

    let key = CurrentAssetMetadataGlobalKey::<T>::get().unwrap();
    AssetMetadataKey::Global(key)
}

/// Inserts a [`TickerRegistrationConfig`] in storage.
fn set_ticker_registration_config<T: Config>() {
    TickerConfig::<T>::put(TickerRegistrationConfig {
        max_ticker_length: TICKER_LEN as u8,
        registration_length: Some((60u32 * 24 * 60 * 60).into()),
    });
}

/// Creates a new [`AssetDetails`] considering the worst case scenario.
pub(crate) fn create_sample_asset<T: AssetConfig>(
    asset_owner: &User<T>,
    divisible: bool,
) -> AssetId {
    let asset_name = AssetName::from(vec![b'N'; T::AssetNameMaxLength::get() as usize].as_slice());
    let funding_round_name =
        FundingRoundName::from(vec![b'F'; T::FundingRoundNameMaxLength::get() as usize].as_slice());
    let asset_identifiers = (0..MAX_IDENTIFIERS_PER_ASSET)
        .map(|_| AssetIdentifier::cusip(*b"17275R102").unwrap())
        .collect();
    let asset_id = Pallet::<T>::generate_asset_id(asset_owner.account(), false);
    Pallet::<T>::create_asset(
        asset_owner.origin.clone().into(),
        asset_name,
        divisible,
        AssetType::default(),
        asset_identifiers,
        Some(funding_round_name),
    )
    .unwrap();

    asset_id
}

pub(crate) fn create_and_issue_sample_asset<T: AssetConfig>(
    asset_owner: &User<T>,
    asset_holder_kind: Option<AssetHolderKind>,
) -> AssetId {
    let asset_id = create_sample_asset::<T>(asset_owner, true);

    Pallet::<T>::issue(
        asset_owner.origin().into(),
        asset_id,
        (ONE_UNIT * POLY).into(),
        asset_holder_kind.unwrap_or_default(),
    )
    .unwrap();

    asset_id
}

/// Creates an asset for `ticker`, creates a custom portfolio for the sender and receiver, sets up compliance and transfer restrictions.
/// Returns the sender and receiver portfolio.
pub fn setup_asset_transfer<T: AssetConfig>(
    sender: &User<T>,
    receiver: &User<T>,
    sender_portfolio_name: Option<&str>,
    receiver_portolfio_name: Option<&str>,
    pause_compliance: bool,
    pause_restrictions: bool,
    n_mediators: u8,
    move_to_sender_portfolio: bool,
    use_account_portfolio: bool,
) -> (AssetHolder, AssetHolder, Vec<User<T>>, AssetId) {
    let (sender_holdings, receiver_holdings) = {
        if use_account_portfolio {
            (
                AssetHolder::try_from(sender.account().encode()).unwrap(),
                AssetHolder::try_from(receiver.account().encode()).unwrap(),
            )
        } else {
            (
                create_portfolio::<T>(sender, sender_portfolio_name.unwrap_or("SenderPortfolio")),
                create_portfolio::<T>(receiver, receiver_portolfio_name.unwrap_or("RcvPortfolio")),
            )
        }
    };

    // Creates the asset
    let asset_id = create_and_issue_sample_asset::<T>(
        sender,
        use_account_portfolio.then_some(AssetHolderKind::Account),
    );
    if move_to_sender_portfolio {
        if let AssetHolder::Portfolio(sender_portfolio) = &sender_holdings {
            move_from_default_portfolio::<T>(
                sender,
                asset_id,
                ONE_UNIT * POLY,
                sender_portfolio.clone(),
            );
        }
    }

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
        Pallet::<T>::add_mandatory_mediators(
            sender.origin().into(),
            asset_id,
            mediators_identity.try_into().unwrap(),
        )
        .unwrap();
    }

    // Opt-in receiver to mandatory affirmation so benchmarks measure worst-case weights.
    T::AffirmationFn::set_mandatory_receiver_affirmation(
        receiver.did(),
        AffirmationRequirement::Required,
    );

    // Adds the maximum number of compliance requirement
    // If pause_compliance is true, only the decoding cost will be considered.
    T::ComplianceManager::setup_asset_compliance(sender.did(), asset_id, 50, pause_compliance);

    // Adds transfer conditions only to consider the cost of decoding it
    // If pause_restrictions is true, only the decoding cost will be considered.
    setup_transfer_restrictions::<T>(
        sender.origin().into(),
        sender.did(),
        asset_id,
        4,
        pause_restrictions,
    );

    (
        sender_holdings,
        receiver_holdings,
        asset_mediators,
        asset_id,
    )
}

/// Returns a [`AssetHolder::Portfolio`] of [`PortfolioKind::User`] with the given name.
pub fn create_portfolio<T: Config>(user: &User<T>, portofolio_name: &str) -> AssetHolder {
    let portfolio_number = NextPortfolioNumber::<T>::get(user.did()).0;

    pallet_portfolio::Pallet::<T>::create_portfolio(
        user.origin().clone().into(),
        PortfolioName(portofolio_name.as_bytes().to_vec()),
    )
    .unwrap();

    AssetHolder::from(PortfolioId::new(
        user.did(),
        PortfolioKind::User(PortfolioNumber(portfolio_number)),
    ))
}

/// Moves `amount` from the user's default portfolio to `destination_portfolio`.
fn move_from_default_portfolio<T: Config>(
    user: &User<T>,
    asset_id: AssetId,
    amount: Balance,
    destination_portfolio: PortfolioId,
) {
    pallet_portfolio::Pallet::<T>::move_portfolio_funds(
        user.origin().clone().into(),
        PortfolioId {
            did: user.did(),
            kind: PortfolioKind::Default,
        },
        destination_portfolio,
        vec![Fund {
            description: FundDescription::Fungible { asset_id, amount },
            memo: None,
        }],
    )
    .unwrap();
}

benchmarks! {
    where_clause {  where T: AssetConfig }

    register_unique_ticker {
        // For the worst case ticker must be of length `TICKER_LEN`
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        set_ticker_registration_config::<T>();
        let ticker = Ticker::repeating(b'A');
    }: _(alice.origin.clone(), ticker)
    verify {
        assert_eq!(
            TickersOwnedByUser::<T>::get(alice.did(), ticker),
            true
        );
        assert_eq!(
            UniqueTickerRegistration::<T>::get(ticker).unwrap().owner,
            alice.did(),
        )
    }

    accept_ticker_transfer {
        // Transfers ticker from Alice to Bob
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");

        let ticker = reg_unique_ticker::<T>(alice.account(), None);
        let new_owner_auth_id = pallet_identity::Pallet::<T>::add_auth(
            alice.did(),
            Signatory::from(bob.did()),
            AuthorizationData::TransferTicker(ticker),
            None
        )
        .unwrap();
    }: _(bob.origin.clone(), new_owner_auth_id)
    verify {
        assert_eq!(
            TickersOwnedByUser::<T>::get(alice.did(), ticker),
            false
        );
        assert_eq!(
            TickersOwnedByUser::<T>::get(bob.did(), ticker),
            true
        );
        assert_eq!(
            UniqueTickerRegistration::<T>::get(ticker).unwrap().owner,
            bob.did(),
        )
    }

    accept_asset_ownership_transfer {
        set_ticker_registration_config::<T>();
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let ticker = reg_unique_ticker::<T>(alice.account(), None);
        Pallet::<T>::link_ticker_to_asset_id(alice.origin().into(), ticker, asset_id).unwrap();

        let new_owner_auth_id = pallet_identity::Pallet::<T>::add_auth(
            alice.did(),
            Signatory::from(bob.did()),
            AuthorizationData::TransferAssetOwnership(asset_id),
            None,
        )
        .unwrap();
    }: _(bob.origin.clone(), new_owner_auth_id)
    verify {
        assert_eq!(
            Assets::<T>::get(&asset_id).unwrap().owner_did,
            bob.did()
        );
        assert_eq!(
            SecurityTokensOwnedByUser::<T>::get(bob.did(), asset_id),
            true
        );
        assert_eq!(
            TickersOwnedByUser::<T>::get(bob.did(), ticker),
            true
        );
    }

    create_asset {
        // Token name length.
        let n in 1 .. T::AssetNameMaxLength::get() as u32;
        // Length of the vector of identifiers.
        let i in 1 .. MAX_IDENTIFIERS_PER_ASSET;
        // Funding round name length.
        let f in 1 .. T::FundingRoundNameMaxLength::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_name = AssetName::from(vec![b'N'; n as usize].as_slice());
        let funding_round_name = FundingRoundName::from(vec![b'F'; f as usize].as_slice());
        let asset_identifiers: Vec<AssetIdentifier> = (0..i)
            .map(|_| AssetIdentifier::cusip(*b"17275R102").unwrap())
            .collect();
        let asset_id = Pallet::<T>::generate_asset_id(alice.account(), false);
    }: _(alice.origin.clone(), asset_name.clone(), true, AssetType::default(), asset_identifiers.clone(), Some(funding_round_name.clone()))
    verify {
        assert_eq!(
            Assets::<T>::get(&asset_id),
            Some(AssetDetails::new(0, alice.did(), true, AssetType::default()))
        );
        assert_eq!(
            SecurityTokensOwnedByUser::<T>::get(alice.did(), &asset_id),
            true
        );
        assert_eq!(
            AssetNames::<T>::get(&asset_id),
            Some(asset_name)
        );
        assert_eq!(
            FundingRound::<T>::get(&asset_id),
            funding_round_name
        );
        assert_eq!(
            AssetIdentifiers::<T>::get(&asset_id),
            asset_identifiers
        );
    }

    freeze {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
    }: _(alice.origin, asset_id)
    verify {
        assert_eq!(Frozen::<T>::get(&asset_id), true);
    }

    unfreeze {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        Pallet::<T>::freeze(alice.origin().into(), asset_id).unwrap();
    }: _(alice.origin, asset_id)
    verify {
        assert_eq!(Frozen::<T>::get(&asset_id), false);
    }

    rename_asset {
        // New token name length.
        let n in 1 .. T::AssetNameMaxLength::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let new_asset_name = AssetName::from(vec![b'N'; n as usize].as_slice());
    }: _(alice.origin, asset_id, new_asset_name.clone())
    verify {
        assert_eq!(AssetNames::<T>::get(&asset_id), Some(new_asset_name));
    }

    issue {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let alice_holdings = create_portfolio::<T>(&alice, "MyPortfolio");

        setup_statistics::<T>(alice.origin().into(), asset_id, T::MaxStatsPerAsset::get());

    }: _(alice.origin, asset_id, (1_000_000 * POLY).into(), alice_holdings.into())
    verify {
        assert_eq!(
            Assets::<T>::get(&asset_id).unwrap().total_supply,
            (1_000_000 * POLY).into()
        );
    }

    redeem {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let alice_holdings = create_portfolio::<T>(&alice, "MyPortfolio");

        setup_statistics::<T>(alice.origin().into(), asset_id, T::MaxStatsPerAsset::get());

        Pallet::<T>::issue(
            alice.origin.clone().into(),
            asset_id,
            (1_000_000 * POLY).into(),
            alice_holdings.clone().into()
        )
        .unwrap();

    }: _(alice.origin, asset_id, (600_000 * POLY).into(), alice_holdings.into())
    verify {
        assert_eq!(
            Assets::<T>::get(&asset_id).unwrap().total_supply,
            (400_000 * POLY).into()
        );
    }

    make_divisible {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, false);
    }: _(alice.origin, asset_id)
    verify {
        assert_eq!(
            Assets::<T>::get(&asset_id).unwrap().divisible,
            true
        );
    }

    add_documents {
        let d in 1 .. MAX_DOCS_PER_ASSET;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let docs = iter::repeat(make_document()).take(d as usize).collect::<Vec<_>>();
    }: _(alice.origin, docs.clone(), asset_id)
    verify {
        for i in 1..d {
            assert_eq!(
                AssetDocuments::<T>::get(asset_id, DocumentId(i)).unwrap(),
                docs[i as usize]
            );
        }
    }

    remove_documents {
        let d in 1 .. MAX_DOCS_PER_ASSET;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let docs = iter::repeat(make_document()).take(d as usize).collect::<Vec<_>>();
        Pallet::<T>::add_documents(alice.origin().into(), docs.clone(), asset_id).unwrap();

        let remove_doc_ids = (1..d).map(|i| DocumentId(i - 1)).collect::<Vec<_>>();
    }: _(alice.origin, remove_doc_ids, asset_id)
    verify {
        for i in 1..d {
            assert_eq!(
                AssetDocuments::<T>::contains_key(&asset_id, DocumentId(i-1)),
                false
            );
        }
    }

    set_funding_round {
        let f in 1 .. T::FundingRoundNameMaxLength::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let funding_round_name = FundingRoundName::from(vec![b'X'; f as usize].as_slice());
    }: _(alice.origin, asset_id, funding_round_name.clone())
    verify {
        assert_eq!(
            FundingRound::<T>::get(&asset_id),
            funding_round_name
        );
    }

    update_identifiers {
        let i in 1 .. MAX_IDENTIFIERS_PER_ASSET;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);

        let asset_identifiers: Vec<_> = iter::repeat(AssetIdentifier::cusip(*b"037833100").unwrap())
            .take(i as usize)
            .collect();
    }: _(alice.origin, asset_id, asset_identifiers.clone())
    verify {
        assert_eq!(
            AssetIdentifiers::<T>::get(&asset_id),
            asset_identifiers
        );
    }

    controller_transfer {
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);

        let alice_holdings = AssetHolder::from(PortfolioId::default_portfolio(alice.did()));

        Pallet::<T>::issue(
            alice.origin.clone().into(),
            asset_id,
            1_000_000,
            alice_holdings.clone().into()
        )
        .unwrap();

        let auth_id = pallet_identity::Pallet::<T>::add_auth(
            alice.did(),
            Signatory::from(bob.did()),
            AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
            None,
        )
        .unwrap();
        pallet_external_agents::Pallet::<T>::accept_become_agent(bob.origin().into(), auth_id)?;
    }: _(bob.origin.clone(), asset_id, 1_000,  alice_holdings, AssetHolderKind::Account)
    verify {
        assert_eq!(
            BalanceOf::<T>::get(asset_id, bob.did()),
            1_000
        );
    }

    controller_transfer_to {
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);

        let alice_holdings = AssetHolder::from(PortfolioId::default_portfolio(alice.did()));
        let bob_holdings = AssetHolder::from(PortfolioId::default_portfolio(bob.did()));

        Pallet::<T>::issue(
            alice.origin.clone().into(),
            asset_id,
            1_000_000,
            alice_holdings.clone().into()
        )
        .unwrap();

        let auth_id = pallet_identity::Pallet::<T>::add_auth(
            alice.did(),
            Signatory::from(bob.did()),
            AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
            None,
        )
        .unwrap();
        pallet_external_agents::Pallet::<T>::accept_become_agent(bob.origin().into(), auth_id)?;
    }: _(bob.origin.clone(), asset_id, 1_000, alice_holdings, bob_holdings)
    verify {
        assert_eq!(
            BalanceOf::<T>::get(asset_id, bob.did()),
            1_000
        );
    }

    register_custom_asset_type {
        let n in 1 .. T::MaxLen::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let ty = vec![b'X'; n as usize];
        assert_eq!(CustomTypeIdSequence::<T>::get(), CustomAssetTypeId(0));
    }: _(alice.origin, ty)
    verify {
        assert_eq!(CustomTypeIdSequence::<T>::get(), CustomAssetTypeId(1));
    }

    set_asset_metadata {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);

        let key = register_metadata_global_name::<T>();
        let value = make_metadata_value::<T>();
        let details = AssetMetadataValueDetail::default();
    }: _(alice.origin, asset_id, key, value, Some(details))

    set_asset_metadata_details {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let key = register_metadata_global_name::<T>();
        let details = AssetMetadataValueDetail::default();
    }: _(alice.origin, asset_id, key, details)

    register_and_set_local_asset_metadata {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let name = make_metadata_name::<T>();
        let spec = make_metadata_spec::<T>();
        let value = make_metadata_value::<T>();
        let details = Some(AssetMetadataValueDetail::default());
    }: _(alice.origin, asset_id, name, spec, value, details)

    register_asset_metadata_local_type {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let name = make_metadata_name::<T>();
        let spec = make_metadata_spec::<T>();
    }: _(alice.origin, asset_id, name, spec)

    register_asset_metadata_global_type {
        let name = make_metadata_name::<T>();
        let spec = make_metadata_spec::<T>();
    }: _(RawOrigin::Root, name, spec)

    update_asset_type {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
    }: _(alice.origin, asset_id, AssetType::EquityPreferred)
    verify {
        assert_eq!(
            Assets::<T>::get(&asset_id).unwrap().asset_type,
            AssetType::EquityPreferred
        );
    }

    remove_local_metadata_key {
        // Creates an asset of type NFT
        let user = user::<T>("target", 0);
        let asset_name = AssetName::from(b"MyAsset");
        let asset_id = Pallet::<T>::generate_asset_id(user.account(), false);
        Pallet::<T>::create_asset(
            user.origin().into(),
            asset_name,
            false,
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new(),
            None,
        )
        .unwrap();
        // Creates two metadata keys, one that belong to the NFT collection and one that doesn't
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        Pallet::<T>::register_asset_metadata_local_type(
            user.origin().into(),
            asset_id,
            asset_metadata_name.clone(),
            asset_metadata_spec.clone()
        ).unwrap();
        Pallet::<T>::register_asset_metadata_local_type(
            user.origin().into(),
            asset_id,
            AssetMetadataName(b"mylocalkey2".to_vec()),
            asset_metadata_spec
        ).unwrap();
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(2));
        let collection_keys: NFTCollectionKeys = vec![asset_metada_key.clone()].into();
        T::NFTFn::create_nft_collection(user.origin().into(), Some(asset_id), None, collection_keys).unwrap();
    }: _(user.origin, asset_id, AssetMetadataLocalKey(1))

    remove_metadata_value {
        // Creates an asset of type NFT
        let user = user::<T>("target", 0);
        let asset_name = AssetName::from(b"MyAsset");
        let asset_id = Pallet::<T>::generate_asset_id(user.account(), false);
        Pallet::<T>::create_asset(
            user.origin().into(),
            asset_name,
            false,
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new(),
            None,
        ).unwrap();
        // Creates one metadata key and set its value
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        Pallet::<T>::register_asset_metadata_local_type(
            user.origin().into(),
            asset_id,
            asset_metadata_name.clone(),
            asset_metadata_spec.clone()
        ).unwrap();
        Pallet::<T>::set_asset_metadata(
            user.origin().into(),
            asset_id,
            AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ).unwrap();
    }: _(user.origin, asset_id, AssetMetadataKey::Local(AssetMetadataLocalKey(1)))

    base_transfer {
        // For the worst case, the portfolios are not the the default ones, the complexity of the transfer depends on
        // the complexity of the compliance rules and the number of statistics to be updated.
        // Since the compliance weight will be charged separately, the rules were paused and only the `Self::asset_compliance(ticker)`
        // read will be considered (this read was not charged in the is_condition_satisfied benchmark).

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let (sender_portfolio, receiver_portfolio, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, true, true, 0, true, false);
    }: {
        Pallet::<T>::base_transfer(
            sender_portfolio,
            receiver_portfolio,
            asset_id,
            ONE_UNIT,
            None,
            None,
            IdentityId::default(),
            &mut weight_meter
        )
        .unwrap();
    }

    exempt_asset_affirmation {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
    }: _(RawOrigin::Root, asset_id)

    remove_asset_affirmation_exemption {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        Pallet::<T>::exempt_asset_affirmation(RawOrigin::Root.into(), asset_id).unwrap();
    }: _(RawOrigin::Root, asset_id)

    pre_approve_asset {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
    }: _(alice.origin, asset_id)

    remove_asset_pre_approval {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        Pallet::<T>::pre_approve_asset(alice.clone().origin().into(), asset_id).unwrap();
    }: _(alice.origin, asset_id)

    add_mandatory_mediators {
        let n in 1 .. T::MaxAssetMediators::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let mediators: BTreeSet<IdentityId> = (0..n).map(|i| IdentityId::from(i as u128)).collect();

        let asset_id = Pallet::<T>::generate_asset_id(alice.account(), false);
        Pallet::<T>::create_asset(
            alice.clone().origin().into(),
            AssetName::from(b"MyAsset"),
            false,
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new(),
            None,
        )
        .unwrap();

    }: _(alice.origin, asset_id, mediators.try_into().unwrap())

    remove_mandatory_mediators {
        let n in 1 .. T::MaxAssetMediators::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let mediators: BTreeSet<IdentityId> = (0..n).map(|i| IdentityId::from(i as u128)).collect();

        let asset_id = Pallet::<T>::generate_asset_id(alice.account(), false);
        Pallet::<T>::create_asset(
            alice.clone().origin().into(),
            AssetName::from(b"MyAsset"),
            false,
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new(),
            None,
        )
        .unwrap();

        Pallet::<T>::add_mandatory_mediators(
            alice.clone().origin().into(),
            asset_id,
            mediators.clone().try_into().unwrap()
        )
        .unwrap();
    }: _(alice.origin, asset_id, mediators.try_into().unwrap())

    link_ticker_to_asset_id {
        set_ticker_registration_config::<T>();
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let ticker = reg_unique_ticker::<T>(alice.account(), None);
    }: _(alice.origin, ticker, asset_id)

    unlink_ticker_from_asset_id {
        set_ticker_registration_config::<T>();
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let ticker = reg_unique_ticker::<T>(alice.account(), None);
        Pallet::<T>::link_ticker_to_asset_id(
            alice.clone().origin().into(),
            ticker,
            asset_id
        )
        .unwrap();
    }: _(alice.origin, ticker, asset_id)

    update_global_metadata_spec {
        let asset_metadata_name = make_metadata_name::<T>();
        let asset_metadata_spec = make_metadata_spec::<T>();

        Pallet::<T>::register_asset_metadata_global_type(
            RawOrigin::Root.into(),
            asset_metadata_name.clone(),
            asset_metadata_spec.clone()
        )
        .unwrap();
    }: _(RawOrigin::Root, asset_metadata_name, asset_metadata_spec)

    receiver_affirm_asset_transfer_base_weight {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");

        // Setup the transfer with worse case conditions.
        // Don't move the assets from the default portfolio.
        let (_sender_portfolio, _receiver_portfolio, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, true, true, 0, true, true);

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let to = AssetHolder::try_from(bob.account().encode()).unwrap();
        let fund = Fund {
            description: FundDescription::Fungible { asset_id, amount: ONE_UNIT },
            memo: None,
        };
        let instruction_id = T::SettlementFn::transfer_funds(
            alice.origin.into(),
            None,
            to,
            fund,
            &mut weight_meter,
            false,
        ).expect("Transfer setup must work");
        let instruction_id = instruction_id.expect("Pending transfer must have an ID");
    }: {
        Pallet::<T>::base_receiver_affirm_asset_transfer(
            bob.origin.into(),
            instruction_id,
            // Only benchmark the base cost.
            true,
        )
        .expect("Receiver affirm must work");
    }

    approve {
        let caller = UserBuilder::<T>::default().generate_did().build("Caller");
        let spender = UserBuilder::<T>::default().generate_did().build("Spender");
        let asset_id = create_sample_asset::<T>(&caller, true);
        // Pre-insert an allowance to benchmark the overwrite path (worst case).
        Allowances::<T>::insert(
            (&caller.account(), &spender.account(), asset_id),
            1000u128,
        );
    }: _(RawOrigin::Signed(caller.account()), asset_id, spender.account(), 500u128)
    verify {
        assert_eq!(
            Allowances::<T>::get((&caller.account(), &spender.account(), asset_id)),
            500u128
        );
    }

    spend_allowance {
        let caller = UserBuilder::<T>::default().generate_did().build("Caller");
        let spender = UserBuilder::<T>::default().generate_did().build("Spender");
        let asset_id = create_sample_asset::<T>(&caller, true);
        Allowances::<T>::insert(
            (&caller.account(), &spender.account(), asset_id),
            ONE_UNIT * 10,
        );
    }: {
        Pallet::<T>::spend_allowance(&caller.account(), &spender.account(), asset_id, ONE_UNIT).unwrap();
    }
    verify {
        assert_eq!(
            Allowances::<T>::get((&caller.account(), &spender.account(), asset_id)),
            ONE_UNIT * 9
        );
    }

    issue_without_statistics {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let alice_holdings = create_portfolio::<T>(&alice, "MyPortfolio");
    }: {
        Pallet::<T>::issue(
            alice.origin.into(),
            asset_id,
            (1_000_000 * POLY).into(),
            alice_holdings.into(),
        )
        .unwrap();
    }

    asset_transfer_report_best_case {
        // No statistics or compliance rules are set
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let (sender, receiver, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, true, true, 0, true, true);
    }: {
        assert!(
            Pallet::<T>::asset_transfer_report(
                &sender,
                &receiver,
                &asset_id,
                ONE_UNIT,
                false,
                &mut weight_meter
            )
            .is_empty()
        );
    }

    asset_transfer_report_worst_case {
        // Max Statistics and Compliance rules are set
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let (sender, receiver, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, false, false, 0, true, true);
    }: {
        assert!(
            Pallet::<T>::asset_transfer_report(
                &sender,
                &receiver,
                &asset_id,
                ONE_UNIT,
                false,
                &mut weight_meter
            )
            .is_empty()
        );
    }

    set_frozen_tokens {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let alice_portfolio = create_portfolio::<T>(&alice, "SenderPortfolio");
    }: _(alice.origin, asset_id, alice_portfolio, ONE_UNIT)

    get_holders_frozen_balance {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let alice_asset_holder = AssetHolder::try_from(alice.account().encode()).unwrap();

        Pallet::<T>::set_frozen_tokens(
            alice.origin.into(),
            asset_id,
            alice_asset_holder.clone(),
            ONE_UNIT
        )
        .unwrap();
    }: {
        assert_eq!(
            Pallet::<T>::get_holders_frozen_balance(
                &alice_asset_holder,
                &asset_id,
            ),
            ONE_UNIT
        );
    }

    transfer_is_allowed_for_holder_best_case {
        // No statistics or compliance rules are set
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let (sender, receiver, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, true, true, 0, true, true);
    }: {
        assert!(
            Pallet::<T>::transfer_is_allowed_for_holder(
                &sender,
                &asset_id,
                true,
                &mut weight_meter
            )
        );
    }

    transfer_is_allowed_for_holder_worst_case {
        // Max Statistics and Compliance rules are set
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let (sender, receiver, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, false, false, 0, true, true);
    }: {
        assert!(
            Pallet::<T>::transfer_is_allowed_for_holder(
                &sender,
                &asset_id,
                true,
                &mut weight_meter
            )
        );
    }

    set_holder_frozen {
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob_portfolio = create_portfolio::<T>(&bob, "SenderPortfolio");
        let asset_id = create_sample_asset::<T>(&alice, true);
    }: _(alice.origin, bob_portfolio.into(), asset_id, true)
}
