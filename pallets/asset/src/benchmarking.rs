// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.
#![cfg(feature = "runtime-benchmarks")]

use crate::*;

use pallet_identity as identity;
use pallet_identity::benchmarking::{User, UserBuilder};
use polymesh_common_utilities::traits::asset::AssetName;
use polymesh_primitives::Ticker;

use frame_benchmarking::benchmarks;
use frame_support::StorageValue;
use sp_std::{convert::TryFrom, iter, prelude::*};

const SEED: u32 = 0;
const MAX_TICKER_LENGTH: u8 = 12;
const MAX_DOCS_PER_ASSET: u32 = 1024;
const MAX_DOC_URI: usize = 4096;
const MAX_DOC_NAME: usize = 1024;
const MAX_DOC_TYPE: usize = 1024;

/// Create a ticker and register it.
fn make_ticker<T: Trait>(owner: T::Origin) -> Ticker {
    let ticker = Ticker::try_from(vec![b'A'; MAX_TICKER_LENGTH as usize].as_slice()).unwrap();
    Module::<T>::register_ticker(owner, ticker).unwrap();

    ticker
}

fn make_asset<T: Trait>(owner: &User<T>) -> (Ticker, SecurityToken<T::Balance>) {
    make_base_asset::<T>(owner, true)
}

fn make_indivisible_asset<T: Trait>(owner: &User<T>) -> (Ticker, SecurityToken<T::Balance>) {
    make_base_asset::<T>(owner, false)
}

fn make_base_asset<T: Trait>(
    owner: &User<T>,
    divisible: bool,
) -> (Ticker, SecurityToken<T::Balance>) {
    let ticker = make_ticker::<T>(owner.origin().into());
    let name: AssetName = ticker.as_slice().into();
    let total_supply: T::Balance = 1_000_000.into();

    let token = SecurityToken {
        name: name.clone(),
        owner_did: owner.did(),
        total_supply: total_supply.clone(),
        divisible,
        asset_type: AssetType::default(),
        primary_issuance_agent: Some(owner.did()),
    };

    Module::<T>::create_asset(
        owner.origin().into(),
        name,
        ticker,
        total_supply,
        divisible,
        AssetType::default(),
        vec![],
        None,
    )
    .expect("Asset cannot be created");

    (ticker, token)
}

fn make_document() -> Document {
    Document {
        uri: [b'u'; MAX_DOC_URI].into(),
        content_hash: [b'7'; 64].into(), // Hash output 512bits.
        name: [b'n'; MAX_DOC_NAME].into(),
        doc_type: Some([b't'; MAX_DOC_TYPE].into()),
        filing_date: None,
    }
}

benchmarks! {
    _ { }

    register_ticker {
        let t in 1 .. MAX_TICKER_LENGTH as u32;

        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: Some((60 * 24 * 60 * 60).into()),
        });

        let caller = UserBuilder::<T>::default().build_with_did("caller", SEED);
        // Generate a ticker of length `t`.
        let ticker = Ticker::try_from(vec![b'A'; t as usize].as_slice()).unwrap();
    }: _(caller.origin, ticker.clone())
    verify {
        assert_eq!(Module::<T>::is_ticker_available(&ticker), false);
    }

    accept_ticker_transfer {
        let owner = UserBuilder::<T>::default().build_with_did("owner", SEED);
        let new_owner = UserBuilder::<T>::default().build_with_did("new_owner", SEED);
        let ticker = make_ticker::<T>(owner.origin().into());

        Module::<T>::asset_ownership_relation(owner.did(), ticker.clone());
        let new_owner_auth_id = identity::Module::<T>::add_auth(
            owner.did(),
            Signatory::from(new_owner.did()),
            AuthorizationData::TransferTicker(ticker),
            None,
            );
    }: _(new_owner.origin(), new_owner_auth_id)
    verify {
        assert_eq!(
            Module::<T>::asset_ownership_relation(owner.did(), ticker),
            AssetOwnershipRelation::NotOwned
        );
        assert_eq!(
            Module::<T>::asset_ownership_relation(new_owner.did(), ticker),
            AssetOwnershipRelation::TickerOwned
        );
    }

    accept_asset_ownership_transfer {
        let owner = UserBuilder::<T>::default().build_with_did("owner", SEED);
        let new_owner = UserBuilder::<T>::default().build_with_did("new_owner", SEED);

        let (ticker, _) = make_asset::<T>(&owner);

        let new_owner_auth_id = identity::Module::<T>::add_auth(
            owner.did(),
            Signatory::from(new_owner.did()),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );
    }: _(new_owner.origin(), new_owner_auth_id)
    verify {
        assert_eq!(Module::<T>::token_details(&ticker).owner_did, new_owner.did());
        assert_eq!(
            Module::<T>::asset_ownership_relation(owner.did(), ticker),
            AssetOwnershipRelation::NotOwned
        );
        assert_eq!(
            Module::<T>::asset_ownership_relation(new_owner.did(), ticker),
            AssetOwnershipRelation::AssetOwned
        );
    }

    create_asset {
        // Token name length.
        let n in 1 .. T::AssetNameMaxLength::get() as u32;
        // Length of the vector of identifiers.
        let i in 1 .. T::MaxIdentifiersPerAsset::get() as u32;
        // Funding round name length.
        let f in 1 .. T::FundingRoundNameMaxLength::get() as u32;

        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: Some((60 * 24 * 60 * 60).into()),
        });
        let ticker = Ticker::try_from(vec![b'A'; MAX_TICKER_LENGTH as usize].as_slice()).unwrap();
        let name = AssetName::from(vec![b'N'; n as usize].as_slice());

        let identifiers: Vec<AssetIdentifier> =
            iter::repeat(AssetIdentifier::cusip(*b"17275R102").unwrap()).take(i as usize).collect();
        let fundr = FundingRoundName::from(vec![b'F'; f as usize].as_slice());
        let owner = UserBuilder::<T>::default().build_with_did("owner", SEED);
        let total_supply: T::Balance = 1_000_000.into();

        let token = SecurityToken {
            name,
            owner_did: owner.did(),
            total_supply: total_supply.clone(),
            divisible: true,
            asset_type: AssetType::default(),
            primary_issuance_agent: Some(owner.did()),
        };

    }: _(owner.origin(), token.name.clone(), ticker, total_supply, token.divisible, token.asset_type.clone(), identifiers.clone(), Some(fundr))
    verify {
        assert_eq!(Module::<T>::token_details(ticker), token);

        assert_eq!(Module::<T>::identifiers(ticker), identifiers);
    }


    freeze {
        let owner = UserBuilder::default().build_with_did("owner", SEED);
        let (ticker, _) = make_asset::<T>(&owner);
    }: _(owner.origin, ticker.clone())
    verify {
        assert_eq!( Module::<T>::frozen(&ticker), true);
    }

    unfreeze {
        let owner = UserBuilder::default().build_with_did("owner", SEED);
        let (ticker, _) = make_asset::<T>(&owner);

        Module::<T>::freeze( owner.origin().into(), ticker.clone())
            .expect("Asset cannot be frozen");

        assert_eq!( Module::<T>::frozen(&ticker), true);
    }: _(owner.origin, ticker.clone())
    verify {
        assert_eq!( Module::<T>::frozen(&ticker), false);
    }

    rename_asset {
        // New token name length.
        let n in 1 .. T::AssetNameMaxLength::get() as u32;

        let new_name = AssetName::from(vec![b'N'; n as usize].as_slice());
        let owner = UserBuilder::default().build_with_did("owner", SEED);
        let (ticker, _) = make_asset::<T>(&owner);
    }: _(owner.origin(), ticker.clone(), new_name.clone())
    verify {
        let token = Module::<T>::token_details(ticker);
        assert_eq!( token.name, new_name);
    }

    issue {
        let owner = UserBuilder::default().build_with_did("owner", SEED);
        let (ticker, _) = make_asset::<T>(&owner);

    }: _(owner.origin, ticker.clone(), 1_000_000.into())
    verify {
        let token = Module::<T>::token_details(ticker);
        assert_eq!( token.total_supply, 2_000_000.into());
    }


    redeem {
        let owner = UserBuilder::default().build_with_did("owner", SEED);
        let (ticker, _) = make_asset::<T>(&owner);
    }: _(owner.origin, ticker.clone(), 600_000.into())
    verify {
        let token = Module::<T>::token_details(ticker);
        assert_eq!( token.total_supply, 400_000.into());
    }

    make_divisible {
        let owner = UserBuilder::default().build_with_did("owner", SEED);
        let (ticker, _) = make_indivisible_asset::<T>(&owner);
    }: _(owner.origin, ticker)
    verify {
        let token = Module::<T>::token_details(ticker);
        assert_eq!( token.divisible, true);
    }

    add_documents {
        let d in 0 .. MAX_DOCS_PER_ASSET;

        let owner = UserBuilder::default().build_with_did("owner", SEED);
        let (ticker, _) = make_asset::<T>(&owner);
        let docs = (0..d).map(|_| make_document()).collect::<Vec<_>>();

    }: _(owner.origin, docs.clone(), ticker)
    verify {
    }
}
