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

use polymesh_common_utilities::{
    benchs::{User, UserBuilder},
    constants::currency::POLY,
    traits::asset::AssetName,
};
use polymesh_contracts::ExtensionInfo;
use polymesh_primitives::{ticker::TICKER_LEN, ExtensionAttributes, SmartExtension, Ticker};

use frame_benchmarking::benchmarks;
use frame_support::StorageValue;
use frame_system::RawOrigin;
use sp_io::hashing::keccak_256;
use sp_std::{
    convert::{TryFrom, TryInto},
    iter,
    prelude::*,
};

const MAX_DOCS_PER_ASSET: u32 = 64;
const MAX_DOC_URI: usize = 4096;
const MAX_DOC_NAME: usize = 1024;
const MAX_DOC_TYPE: usize = 1024;
const MAX_IDENTIFIERS_PER_ASSET: u32 = 512;

/// Create a ticker and register it.
pub fn make_ticker<T: Trait>(owner: T::Origin, optional_ticker: Option<Ticker>) -> Ticker {
    let ticker = optional_ticker
        .unwrap_or_else(|| Ticker::try_from(vec![b'A'; TICKER_LEN as usize].as_slice()).unwrap());
    Module::<T>::register_ticker(owner, ticker).unwrap();
    ticker
}

pub fn make_asset<T: Trait>(owner: &User<T>) -> Ticker {
    make_base_asset::<T>(owner, true, None)
}

pub fn make_indivisible_asset<T: Trait>(owner: &User<T>) -> Ticker {
    make_base_asset::<T>(owner, false, None)
}

pub fn make_base_asset<T: Trait>(
    owner: &User<T>,
    divisible: bool,
    optional_ticker: Option<Ticker>,
) -> Ticker {
    let ticker = make_ticker::<T>(owner.origin().into(), optional_ticker);
    let name: AssetName = ticker.as_slice().into();
    let total_supply: T::Balance = (1_000_000 * POLY).into();

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

    ticker
}

pub fn make_document() -> Document {
    Document {
        uri: [b'u'; MAX_DOC_URI].into(),
        content_hash: b"572cdd8d8f1754dd0c4a75d99b569845"[..].try_into().unwrap(), // MD5 output is 128bits.
        name: [b'n'; MAX_DOC_NAME].into(),
        doc_type: Some([b't'; MAX_DOC_TYPE].into()),
        filing_date: None,
    }
}

fn make_default_reg_config<T: Trait>() -> TickerRegistrationConfig<T::Moment> {
    TickerRegistrationConfig {
        max_ticker_length: 8,
        registration_length: Some(10000.into()),
    }
}

fn make_classic_ticker<T: Trait>(eth_owner: ethereum::EthereumAddress, ticker: Ticker) {
    let classic_ticker = ClassicTickerImport {
        eth_owner,
        ticker,
        is_created: false,
        is_contract: false,
    };
    let reg_config = make_default_reg_config::<T>();
    let root = frame_system::RawOrigin::Root.into();

    <Module<T>>::reserve_classic_ticker(root, classic_ticker, 0.into(), reg_config)
        .expect("`reserve_classic_ticker` failed");
}

fn make_extension<T: Trait>(is_archive: bool) -> SmartExtension<T::AccountId> {
    // Simulate that extension was added.
    let extension_id = UserBuilder::<T>::default().build("extension").account;
    let extension_details = SmartExtension {
        extension_type: SmartExtensionType::TransferManager,
        extension_name: b"PTM".into(),
        extension_id: extension_id.clone(),
        is_archive,
    };

    // Add extension info into contracts wrapper.
    let version = 1u32;
    CompatibleSmartExtVersion::insert(&extension_details.extension_type, version);

    let attr = ExtensionAttributes {
        version,
        ..Default::default()
    };
    ExtensionInfo::<T>::insert(extension_id, attr);

    extension_details
}

benchmarks! {
    _ { }

    register_ticker {
        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: TICKER_LEN as u8,
            registration_length: Some((60 * 24 * 60 * 60).into()),
        });

        let caller = UserBuilder::<T>::default().generate_did().build("caller");
        // Generate a ticker of length `t`.
        let ticker = Ticker::try_from(vec![b'A'; TICKER_LEN].as_slice()).unwrap(); }: _(caller.origin, ticker.clone()) verify {
        assert_eq!(Module::<T>::is_ticker_available(&ticker), false);
    }

    accept_ticker_transfer {
        let owner = UserBuilder::<T>::default().generate_did().build("owner");
        let new_owner = UserBuilder::<T>::default().generate_did().build("new_owner");
        let ticker = make_ticker::<T>(owner.origin().into(), None);

        Module::<T>::asset_ownership_relation(owner.did(), ticker.clone());
        let new_owner_auth_id = identity::Module::<T>::add_auth( owner.did(), Signatory::from(new_owner.did()), AuthorizationData::TransferTicker(ticker), None);
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
        let owner = UserBuilder::<T>::default().generate_did().build("owner");
        let new_owner = UserBuilder::<T>::default().generate_did().build("new_owner");

        let ticker = make_asset::<T>(&owner);

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
        let i in 1 .. MAX_IDENTIFIERS_PER_ASSET;
        // Funding round name length.
        let f in 1 .. T::FundingRoundNameMaxLength::get() as u32;

        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: TICKER_LEN as u8,
            registration_length: Some((60 * 24 * 60 * 60).into()),
        });
        let ticker = Ticker::try_from(vec![b'A'; TICKER_LEN].as_slice()).unwrap();
        let name = AssetName::from(vec![b'N'; n as usize].as_slice());

        let identifiers: Vec<AssetIdentifier> =
            iter::repeat(AssetIdentifier::cusip(*b"17275R102").unwrap()).take(i as usize).collect();
        let fundr = FundingRoundName::from(vec![b'F'; f as usize].as_slice());
        let owner = UserBuilder::<T>::default().generate_did().build("owner");
        let total_supply: T::Balance = 1_000_000.into();

        let token = SecurityToken {
            name,
            owner_did: owner.did(),
            total_supply: total_supply.clone(),
            divisible: true,
            asset_type: AssetType::default(),
            primary_issuance_agent: None,
        };

    }: _(owner.origin(), token.name.clone(), ticker, total_supply, token.divisible, token.asset_type.clone(), identifiers.clone(), Some(fundr))
    verify {
        assert_eq!(Module::<T>::token_details(ticker), token);

        assert_eq!(Module::<T>::identifiers(ticker), identifiers);
    }


    freeze {
        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_asset::<T>(&owner);
    }: _(owner.origin, ticker.clone())
    verify {
        assert_eq!( Module::<T>::frozen(&ticker), true);
    }

    unfreeze {
        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_asset::<T>(&owner);

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
        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_asset::<T>(&owner);
    }: _(owner.origin(), ticker.clone(), new_name.clone())
    verify {
        let token = Module::<T>::token_details(ticker);
        assert_eq!( token.name, new_name);
    }

    issue {
        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_asset::<T>(&owner);

    }: _(owner.origin, ticker.clone(), 1_000_000.into())
    verify {
        let token = Module::<T>::token_details(ticker);
        assert_eq!( token.total_supply, 2_000_000.into());
    }


    redeem {
        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_asset::<T>(&owner);
    }: _(owner.origin, ticker.clone(), 600_000.into())
    verify {
        let token = Module::<T>::token_details(ticker);
        assert_eq!( token.total_supply, 400_000.into());
    }

    make_divisible {
        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_indivisible_asset::<T>(&owner);
    }: _(owner.origin, ticker)
    verify {
        let token = Module::<T>::token_details(ticker);
        assert_eq!( token.divisible, true);
    }

    add_documents {
        // It starts at 1 in order to get something for `verify` section.
        let d in 1 .. MAX_DOCS_PER_ASSET;

        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_asset::<T>(&owner);
        let docs = iter::repeat( make_document()).take( d as usize).collect::<Vec<_>>();

    }: _(owner.origin, docs.clone(), ticker.clone())
    verify {
        for i in 1..d {
            assert_eq!(Module::<T>::asset_documents(ticker, DocumentId(i)), docs[i as usize]);
        }
    }

    remove_documents {
        let d in 1 .. MAX_DOCS_PER_ASSET;

        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_asset::<T>(&owner);
        let docs = iter::repeat( make_document()).take( MAX_DOCS_PER_ASSET as usize).collect::<Vec<_>>();
        Module::<T>::add_documents( owner.origin().into(), docs.clone(), ticker)
            .expect("Documents cannot be added");

        let remove_doc_ids = (1..d).map(|i| DocumentId(i-1)).collect::<Vec<_>>();

    }: _(owner.origin, remove_doc_ids.clone(), ticker.clone())
    verify {
        for i in 1..d {
            assert_eq!(<AssetDocuments>::contains_key( &ticker, DocumentId(i-1)), false);
        }
    }

     set_funding_round {
        let f in 1 .. T::FundingRoundNameMaxLength::get() as u32;

        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_asset::<T>(&owner);

        let fundr = FundingRoundName::from(vec![b'X'; f as usize].as_slice());
     }: _(owner.origin, ticker.clone(), fundr.clone())
     verify {
        assert_eq!( Module::<T>::funding_round(ticker), fundr);
     }


     update_identifiers {
         let i in 1 .. MAX_IDENTIFIERS_PER_ASSET;

         let owner = UserBuilder::default().generate_did().build("owner");
         let ticker = make_asset::<T>(&owner);

         let identifiers: Vec<AssetIdentifier> =
             iter::repeat(AssetIdentifier::cusip(*b"037833100").unwrap()).take(i as usize).collect();

     }: _(owner.origin(), ticker.clone(), identifiers.clone())
     verify {
        assert_eq!( Module::<T>::identifiers(ticker), identifiers);
     }

     add_extension {
         let owner = UserBuilder::default().generate_did().build("owner");
         let ticker = make_asset::<T>(&owner);
         let ext_details = make_extension::<T>(false);
         let ext_id = ext_details.extension_id.clone();
     }: _(owner.origin(), ticker.clone(), ext_details.clone())
     verify {
         assert_eq!( Module::<T>::extension_details((ticker, ext_id)), ext_details);
     }

     archive_extension {
         let owner = UserBuilder::default().generate_did().build("owner");
         let ticker = make_asset::<T>(&owner);
         let ext_details = make_extension::<T>(false);
         let ext_id = ext_details.extension_id.clone();
         Module::<T>::add_extension( owner.origin().into(), ticker.clone(), ext_details)
             .expect( "Extension cannot be added");

     }: _(owner.origin, ticker.clone(), ext_id.clone())
     verify {
         let ext_details = Module::<T>::extension_details((ticker,ext_id));
         assert_eq!( ext_details.is_archive, true);
     }

     unarchive_extension {
         let owner = UserBuilder::default().generate_did().build("owner");
         let ticker = make_asset::<T>(&owner);
         let ext_details = make_extension::<T>(true);
         let ext_id = ext_details.extension_id.clone();
         Module::<T>::add_extension( owner.origin().into(), ticker.clone(), ext_details)
             .expect( "Extension cannot be added");

     }: _(owner.origin(), ticker.clone(), ext_id.clone())
     verify {
         let ext_details = Module::<T>::extension_details((ticker,ext_id));
         assert_eq!( ext_details.is_archive, false);
     }

     remove_smart_extension {
         let owner = UserBuilder::default().generate_did().build("owner");
         let ticker = make_asset::<T>(&owner);
         let ext_details = make_extension::<T>(false);
         let ext_id = ext_details.extension_id.clone();
         Module::<T>::add_extension( owner.origin().into(), ticker.clone(), ext_details)
             .expect( "Extension cannot be added");
     }: _(owner.origin(), ticker.clone(), ext_id.clone())
     verify {
        assert_eq!(<ExtensionDetails<T>>::contains_key((ticker,ext_id)), false);
     }

    remove_primary_issuance_agent {
        let owner = UserBuilder::default().generate_did().build("owner");
        let ticker = make_asset::<T>(&owner);
    }: _(owner.origin(), ticker.clone())
    verify {
        let token = Module::<T>::token_details(ticker);
        assert_eq!( token.primary_issuance_agent, None);
    }


    claim_classic_ticker {
        let owner = UserBuilder::<T>::default().generate_did().build("owner");
        let owner_eth_sk = secp256k1::SecretKey::parse(&keccak_256(b"owner")).unwrap();
        let owner_eth_pk = ethereum::address(&owner_eth_sk);

        let ticker :Ticker = b"USDX1"[..].try_into().unwrap();
        make_classic_ticker::<T>( owner_eth_pk, ticker.clone());

        let eth_sig = ethereum::eth_msg(owner.did(), b"classic_claim", &owner_eth_sk);

    }: _(owner.origin(), ticker.clone(), eth_sig)
    verify {
        assert_eq!(owner.did(), Module::<T>::ticker_registration(ticker).owner);
    }

    reserve_classic_ticker {
        let owner = UserBuilder::<T>::default().generate_did().build("owner");

        let ticker :Ticker = b"ACME"[..].try_into().unwrap();
        let config = make_default_reg_config::<T>();
        let classic = ClassicTickerImport {
            eth_owner: ethereum::EthereumAddress(*b"0x012345678987654321"),
            ticker: ticker.clone(),
            is_created: true,
            is_contract: false,
        };
    }: _( RawOrigin::Root, classic.clone(), owner.did(), config)
    verify {
        assert_eq!(<Tickers<T>>::contains_key(&ticker), true);
    }

    accept_primary_issuance_agent_transfer {
        let owner = UserBuilder::<T>::default().generate_did().build("owner");
        let primary_issuance_agent = UserBuilder::<T>::default().generate_did().build("1stIssuance");
        let ticker = make_asset::<T>(&owner);

        let auth_id = identity::Module::<T>::add_auth(
            owner.did(),
            Signatory::from(primary_issuance_agent.did()),
            AuthorizationData::TransferPrimaryIssuanceAgent(ticker.clone()),
            None,
        );
    }: _(primary_issuance_agent.origin(), auth_id)
    verify {
        let token = Module::<T>::token_details(&ticker);
        assert_eq!(token.primary_issuance_agent, primary_issuance_agent.did);
    }
}
