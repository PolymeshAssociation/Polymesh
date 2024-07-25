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
use frame_support::StorageValue;
use frame_system::RawOrigin;
use scale_info::prelude::format;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::{convert::TryInto, iter, prelude::*};

use pallet_portfolio::{NextPortfolioNumber, PortfolioAssetBalances};
//use pallet_statistics::benchmarking::setup_transfer_restrictions;
use polymesh_common_utilities::benchs::{reg_unique_ticker, user, AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::constants::currency::{ONE_UNIT, POLY};
use polymesh_common_utilities::traits::compliance_manager::ComplianceFnConfig;
use polymesh_common_utilities::traits::nft::NFTTrait;
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::asset::{AssetName, NonFungibleType};
use polymesh_primitives::asset_metadata::{
    AssetMetadataDescription, AssetMetadataKey, AssetMetadataName, AssetMetadataSpec,
    AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::ticker::TICKER_LEN;
use polymesh_primitives::{
    AuthorizationData, Fund, FundDescription, IdentityId, NFTCollectionKeys, PortfolioKind,
    PortfolioName, PortfolioNumber, Signatory, Ticker, Url, WeightMeter,
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
fn register_metadata_global_name<T: Config>() -> AssetMetadataKey {
    let root = RawOrigin::Root.into();
    let name = make_metadata_name::<T>();
    let spec = make_metadata_spec::<T>();

    Module::<T>::register_asset_metadata_global_type(root, name, spec).unwrap();

    let key = Module::<T>::current_asset_metadata_global_key().unwrap();
    AssetMetadataKey::Global(key)
}

/// Inserts a [`TickerRegistrationConfig`] in storage.
fn set_ticker_registration_config<T: Config>() {
    TickerConfig::<T>::put(TickerRegistrationConfig {
        max_ticker_length: TICKER_LEN as u8,
        registration_length: Some((60u32 * 24 * 60 * 60).into()),
    });
}

/// Creates a new [`SecurityToken`] considering the worst case scenario.
fn create_sample_asset<T: Config>(asset_owner: &User<T>, divisible: bool) -> AssetID {
    let asset_name = AssetName::from(vec![b'N'; T::AssetNameMaxLength::get() as usize].as_slice());
    let funding_round_name =
        FundingRoundName::from(vec![b'F'; T::FundingRoundNameMaxLength::get() as usize].as_slice());
    let asset_identifiers = (0..MAX_IDENTIFIERS_PER_ASSET)
        .map(|_| AssetIdentifier::cusip(*b"17275R102").unwrap())
        .collect();
    let asset_id = Module::<T>::generate_asset_id(asset_owner.did(), false);
    Module::<T>::create_asset(
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

/// Creates an asset for `ticker`, creates a custom portfolio for the sender and receiver, sets up compliance and transfer restrictions.
/// Returns the sender and receiver portfolio.
pub fn setup_asset_transfer<T>(
    sender: &User<T>,
    receiver: &User<T>,
    sender_portfolio_name: Option<&str>,
    receiver_portolfio_name: Option<&str>,
    pause_compliance: bool,
    pause_restrictions: bool,
    n_mediators: u8,
) -> (PortfolioId, PortfolioId, Vec<User<T>>, AssetID)
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    let sender_portfolio =
        create_portfolio::<T>(sender, sender_portfolio_name.unwrap_or("SenderPortfolio"));
    let receiver_portfolio =
        create_portfolio::<T>(receiver, receiver_portolfio_name.unwrap_or("RcvPortfolio"));

    // Creates the asset
    let asset_id = create_sample_asset::<T>(sender, true);
    move_from_default_portfolio::<T>(sender, asset_id, ONE_UNIT * POLY, sender_portfolio);

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
        Module::<T>::add_mandatory_mediators(
            sender.origin().into(),
            asset_id,
            mediators_identity.try_into().unwrap(),
        )
        .unwrap();
    }

    // Adds the maximum number of compliance requirement
    // If pause_compliance is true, only the decoding cost will be considered.
    T::ComplianceManager::setup_asset_compliance(sender.did(), asset_id, 50, pause_compliance);

    // Adds transfer conditions only to consider the cost of decoding it
    // If pause_restrictions is true, only the decoding cost will be considered.
    //setup_transfer_restrictions::<T>(
    //    sender.origin().into(),
    //    sender.did(),
    //    asset_id,
    //    4,
    //    pause_restrictions,
    //);

    (
        sender_portfolio,
        receiver_portfolio,
        asset_mediators,
        asset_id,
    )
}

/// Creates a user portfolio for `user`.
pub fn create_portfolio<T: Config>(user: &User<T>, portofolio_name: &str) -> PortfolioId {
    let portfolio_number = Portfolio::<T>::next_portfolio_number(user.did()).0;

    Portfolio::<T>::create_portfolio(
        user.origin().clone().into(),
        PortfolioName(portofolio_name.as_bytes().to_vec()),
    )
    .unwrap();

    PortfolioId {
        did: user.did(),
        kind: PortfolioKind::User(PortfolioNumber(portfolio_number)),
    }
}

/// Moves `amount` from the user's default portfolio to `destination_portfolio`.
fn move_from_default_portfolio<T: Config>(
    user: &User<T>,
    asset_id: AssetID,
    amount: Balance,
    destination_portfolio: PortfolioId,
) {
    Portfolio::<T>::move_portfolio_funds(
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
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    register_unique_ticker {
        // For the worst case ticker must be of length `TICKER_LEN`
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        set_ticker_registration_config::<T>();
        let ticker = Ticker::repeating(b'A');
    }: _(alice.origin.clone(), ticker)
    verify {
        assert_eq!(TickersOwnedByUser::get(alice.did(), ticker), true);
        assert_eq!(
            UniqueTickerRegistration::<T>::get(ticker).unwrap(),
            TickerRegistration {
                owner: alice.did(),
                expiry: None
            }
        )
    }

    accept_ticker_transfer {
        // Transfers ticker from Alice to Bob
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");

        let ticker = reg_unique_ticker::<T>(alice.origin().into(), None);
        let new_owner_auth_id = pallet_identity::Module::<T>::add_auth(
            alice.did(),
            Signatory::from(bob.did()),
            AuthorizationData::TransferTicker(ticker),
            None
        )
        .unwrap();
    }: _(bob.origin.clone(), new_owner_auth_id)
    verify {
        assert_eq!(TickersOwnedByUser::get(alice.did(), ticker), false);
        assert_eq!(TickersOwnedByUser::get(bob.did(), ticker), true);
        assert_eq!(
            UniqueTickerRegistration::<T>::get(ticker).unwrap(),
            TickerRegistration {
                owner: bob.did(),
                expiry: None
            }
        )
    }

//    accept_asset_ownership_transfer {
//        let (owner, ticker) = owned_ticker::<T>();
//        let new_owner = UserBuilder::<T>::default().generate_did().build("new_owner");
//        let did = new_owner.did();
//
//        let new_owner_auth_id = pallet_identity::Module::<T>::add_auth(
//            owner.did(),
//            Signatory::from(did),
//            AuthorizationData::TransferAssetOwnership(ticker),
//            None,
//        )
//        .unwrap();
//    }: _(new_owner.origin, new_owner_auth_id)
//    verify {
//        assert_eq!(token_details::<T>(ticker).owner_did, did);
//        verify_ownership::<T>(ticker, owner.did(), did, AssetOwnershipRelation::AssetOwned);
//    }

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
        let asset_id = Module::<T>::generate_asset_id(alice.did(), false);
    }: _(alice.origin.clone(), asset_name.clone(), true, AssetType::default(), asset_identifiers.clone(), Some(funding_round_name.clone()))
    verify {
        assert_eq!(
            SecurityTokens::get(&asset_id),
            Some(SecurityToken::new(0, alice.did(), true, AssetType::default()))
        );
        assert_eq!(
            SecurityTokensOwnedByuser::get(alice.did(), &asset_id),
            true
        );
        assert_eq!(
            AssetNames::get(&asset_id),
            Some(asset_name)
        );
        assert_eq!(
            FundingRound::get(&asset_id),
            funding_round_name
        );
        assert_eq!(
            AssetIdentifiers::get(&asset_id),
            asset_identifiers
        );
    }

    freeze {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
    }: _(alice.origin, asset_id)
    verify {
        assert_eq!(Frozen::get(&asset_id), true);
    }

    unfreeze {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        Module::<T>::freeze(alice.origin().into(), asset_id).unwrap();
    }: _(alice.origin, asset_id)
    verify {
        assert_eq!(Frozen::get(&asset_id), false);
    }

    rename_asset {
        // New token name length.
        let n in 1 .. T::AssetNameMaxLength::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let new_asset_name = AssetName::from(vec![b'N'; n as usize].as_slice());
    }: _(alice.origin, asset_id, new_asset_name.clone())
    verify {
        assert_eq!(AssetNames::get(&asset_id), Some(new_asset_name));
    }

    issue {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let portfolio_id = create_portfolio::<T>(&alice, "MyPortfolio");
    }: _(alice.origin, asset_id, (1_000_000 * POLY).into(), portfolio_id.kind)
    verify {
        assert_eq!(
            SecurityTokens::get(&asset_id).unwrap().total_supply,
            (1_000_000 * POLY).into()
        );
    }

    redeem {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let portfolio_id = create_portfolio::<T>(&alice, "MyPortfolio");

        Module::<T>::issue(
            alice.origin.clone().into(),
            asset_id,
            (1_000_000 * POLY).into(),
            PortfolioKind::User(PortfolioNumber(1))
        )
        .unwrap();
    }: _(alice.origin, asset_id, (600_000 * POLY).into(), portfolio_id.kind)
    verify {
        assert_eq!(
            SecurityTokens::get(&asset_id).unwrap().total_supply,
            (400_000 * POLY).into()
        );
    }

    make_divisible {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, false);
    }: _(alice.origin, asset_id)
    verify {
        assert_eq!(
            SecurityTokens::get(&asset_id).unwrap().divisible,
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
                Module::<T>::asset_documents(asset_id, DocumentId(i)).unwrap(),
                docs[i as usize]
            );
        }
    }

    remove_documents {
        let d in 1 .. MAX_DOCS_PER_ASSET;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        let docs = iter::repeat(make_document()).take(d as usize).collect::<Vec<_>>();
        Module::<T>::add_documents(alice.origin().into(), docs.clone(), asset_id).unwrap();

        let remove_doc_ids = (1..d).map(|i| DocumentId(i - 1)).collect::<Vec<_>>();
    }: _(alice.origin, remove_doc_ids, asset_id)
    verify {
        for i in 1..d {
            assert_eq!(
                AssetDocuments::contains_key(&asset_id, DocumentId(i-1)),
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
            FundingRound::get(&asset_id),
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
            AssetIdentifiers::get(&asset_id),
            asset_identifiers
        );
    }

    controller_transfer {
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);

        Module::<T>::issue(
            alice.origin.clone().into(),
            asset_id,
            1_000_000,
            PortfolioKind::Default
        )
        .unwrap();

        let auth_id = pallet_identity::Module::<T>::add_auth(
            alice.did(),
            Signatory::from(bob.did()),
            AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
            None,
        )
        .unwrap();
        pallet_external_agents::Module::<T>::accept_become_agent(bob.origin().into(), auth_id)?;
    }: _(bob.origin.clone(), asset_id, 1_000,  PortfolioId::default_portfolio(alice.did()))
    verify {
        assert_eq!(
            Module::<T>::balance_of(asset_id, bob.did()),
            1_000
        );
    }

    register_custom_asset_type {
        let n in 1 .. T::MaxLen::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let ty = vec![b'X'; n as usize];
    }: _(alice.origin, ty)
    verify {
        assert_eq!(Module::<T>::custom_type_id_seq(), CustomAssetTypeId(2));
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
            SecurityTokens::get(&asset_id).unwrap().asset_type,
            AssetType::EquityPreferred
        );
    }

    remove_local_metadata_key {
        // Creates an asset of type NFT
        let user = user::<T>("target", 0);
        let asset_name = AssetName::from(b"MyAsset");
        let asset_id = Module::<T>::generate_asset_id(user.did(), false);
        Module::<T>::create_asset(
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
        Module::<T>::register_asset_metadata_local_type(
            user.origin().into(),
            asset_id,
            asset_metadata_name.clone(),
            asset_metadata_spec.clone()
        ).unwrap();
        Module::<T>::register_asset_metadata_local_type(
            user.origin().into(),
            asset_id,
            AssetMetadataName(b"mylocalkey2".to_vec()),
            asset_metadata_spec
        ).unwrap();
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(2));
        let collection_keys: NFTCollectionKeys = vec![asset_metada_key.clone()].into();
        T::NFTFn::create_nft_collection(user.origin().into(), asset_id, None, collection_keys).unwrap();
    }: _(user.origin, asset_id, AssetMetadataLocalKey(1))

    remove_metadata_value {
        // Creates an asset of type NFT
        let user = user::<T>("target", 0);
        let asset_name = AssetName::from(b"MyAsset");
        let asset_id = Module::<T>::generate_asset_id(user.did(), false);
        Module::<T>::create_asset(
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
        Module::<T>::register_asset_metadata_local_type(
            user.origin().into(),
            asset_id,
            asset_metadata_name.clone(),
            asset_metadata_spec.clone()
        ).unwrap();
        Module::<T>::set_asset_metadata(
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
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let (sender_portfolio, receiver_portfolio, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, true, true, 0);
    }: {
        Module::<T>::base_transfer(
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
        Module::<T>::exempt_asset_affirmation(RawOrigin::Root.into(), asset_id).unwrap();
    }: _(RawOrigin::Root, asset_id)

    pre_approve_asset {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
    }: _(alice.origin, asset_id)

    remove_asset_pre_approval {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
        Module::<T>::pre_approve_asset(alice.clone().origin().into(), asset_id).unwrap();
    }: _(alice.origin, asset_id)

    add_mandatory_mediators {
        let n in 1 .. T::MaxAssetMediators::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let mediators: BTreeSet<IdentityId> = (0..n).map(|i| IdentityId::from(i as u128)).collect();

        let asset_id = Module::<T>::generate_asset_id(alice.did(), false);
        Module::<T>::create_asset(
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

        let asset_id = Module::<T>::generate_asset_id(alice.did(), false);
        Module::<T>::create_asset(
            alice.clone().origin().into(),
            AssetName::from(b"MyAsset"),
            false,
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new(),
            None,
        )
        .unwrap();

        Module::<T>::add_mandatory_mediators(
            alice.clone().origin().into(),
            asset_id,
            mediators.clone().try_into().unwrap()
        )
        .unwrap();
    }: _(alice.origin, asset_id, mediators.try_into().unwrap())
}
