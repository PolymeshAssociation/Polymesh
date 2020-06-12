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

use crate::*;
use pallet_balances as balances;
use pallet_identity as identity;
use polymesh_primitives::{AccountKey, AuthorizationData, IdentityId, Signatory, Ticker};

use codec::Encode;
use frame_benchmarking::{account, benchmarks};
use frame_support::{traits::Currency, StorageValue};
use frame_system::RawOrigin;
use sp_std::{convert::TryFrom, iter, prelude::*};

const SEED: u32 = 0;
const MAX_USER_INDEX: u32 = 1_000;
const MAX_TICKER_LENGTH: u8 = 12;
const MAX_NAME_LENGTH: u32 = 64;

fn make_account<T: Trait>(
    name: &'static str,
    u: u32,
) -> (T::AccountId, RawOrigin<T::AccountId>, IdentityId) {
    let account: T::AccountId = account(name, u, SEED);
    let origin = RawOrigin::Signed(account.clone());
    let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());
    let _ = identity::Module::<T>::register_did(origin.clone().into(), vec![]);
    let did = identity::Module::<T>::get_identity(&AccountKey::try_from(account.encode()).unwrap())
        .unwrap();
    (account, origin, did)
}

fn make_token<T: Trait>(
    origin: RawOrigin<T::AccountId>,
    ticker_len: u32,
    token_name_len: u32,
    identifiers_len: u32,
    funding_round_len: u32,
) -> Ticker {
    <TickerConfig<T>>::put(TickerRegistrationConfig {
        max_ticker_length: MAX_TICKER_LENGTH,
        registration_length: None,
    });
    let ticker = Ticker::try_from(vec![b'T'; ticker_len as usize].as_slice()).unwrap();
    let name = AssetName::from(vec![b'N'; token_name_len as usize].as_slice());
    let total_supply: T::Balance = 1_000_000_000.into();
    let asset_type = AssetType::default();
    let identifiers: Vec<(IdentifierType, AssetIdentifier)> = iter::repeat(Default::default())
        .take(identifiers_len as usize)
        .collect();
    let fundr = FundingRoundName::from(vec![b'F'; funding_round_len as usize].as_slice());
    Module::<T>::create_asset(
        origin.into(),
        name,
        ticker,
        total_supply,
        true,
        asset_type,
        identifiers,
        Some(fundr),
    )
    .unwrap();
    ticker
}

benchmarks! {
    _ {
        // User account seed.
        let u in 1 .. MAX_USER_INDEX => ();
    }

    register_ticker {
        let u in ...;
        let t in 1 .. MAX_TICKER_LENGTH as u32;
        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: None,
        });
        let origin = make_account::<T>("caller", u).1;
        // Generate a ticker of length `l`.
        let ticker = Ticker::try_from(vec![b'A'; t as usize].as_slice()).unwrap();
    }: _(origin, ticker)

    accept_ticker_transfer {
        let u in ...;
        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: None,
        });
        let (_, alice_origin, alice_did) = make_account::<T>("alice", u);
        let (_, bob_origin, bob_did) = make_account::<T>("bob", u);
        let ticker = Ticker::try_from(
            vec![b'A'; MAX_TICKER_LENGTH as usize].as_slice()
        ).unwrap();
        Module::<T>::register_ticker(alice_origin.clone().into(), ticker).unwrap();
        let bob_auth_id = identity::Module::<T>::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferTicker(ticker),
            None,
        );
    }: _(bob_origin, bob_auth_id)

    accept_asset_ownership_transfer {
        let u in ...;
        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: None,
        });
        let (_, alice_origin, alice_did) = make_account::<T>("alice", u);
        let (_, bob_origin, bob_did) = make_account::<T>("bob", u);
        let ticker = Ticker::try_from(
            vec![b'A'; MAX_TICKER_LENGTH as usize].as_slice()
        ).unwrap();
        Module::<T>::register_ticker(alice_origin.clone().into(), ticker).unwrap();
        let bob_auth_id = identity::Module::<T>::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );
    }: _(bob_origin, bob_auth_id)

    create_asset {
        let u in ...;
        // Token name length.
        let n in 1 .. MAX_NAME_LENGTH;
        // Ticker length.
        let t in 1 .. MAX_TICKER_LENGTH as u32;
        // Length of the vector of identifiers.
        let i in 1 .. 100;
        // Funding round name length.
        let f in 1 .. MAX_NAME_LENGTH;
        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: None,
        });
        let name = AssetName::from(vec![b'N'; n as usize].as_slice());
        let ticker = Ticker::try_from(vec![b'T'; t as usize].as_slice()).unwrap();
        let total_supply: T::Balance = 1_000_000.into();
        let asset_type = AssetType::default();
        let identifiers: Vec<(IdentifierType, AssetIdentifier)> =
            iter::repeat(Default::default()).take(i as usize).collect();
        let fundr = FundingRoundName::from(vec![b'F'; f as usize].as_slice());
        let origin = make_account::<T>("caller", u).1;
    }: _(origin, name, ticker, total_supply, true, asset_type, identifiers, Some(fundr))

    freeze {
        let u in ...;
        // Ticker length.
        let t in 1 .. MAX_TICKER_LENGTH as u32;
        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: None,
        });
        let ticker = Ticker::try_from(vec![b'T'; t as usize].as_slice()).unwrap();
        let origin = make_account::<T>("caller", u).1;
        Module::<T>::register_ticker(origin.clone().into(), ticker).unwrap();
    }: _(origin, ticker)

    unfreeze {
        let u in ...;
        // Ticker length.
        let t in 1 .. MAX_TICKER_LENGTH as u32;
        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: None,
        });
        let ticker = Ticker::try_from(vec![b'T'; t as usize].as_slice()).unwrap();
        let origin = make_account::<T>("caller", u).1;
        Module::<T>::register_ticker(origin.clone().into(), ticker).unwrap();
        Module::<T>::freeze(origin.clone().into(), ticker).unwrap();
    }: _(origin, ticker)

    rename_asset {
        let u in ...;
        // Old token name length.
        let n in 1 .. MAX_NAME_LENGTH;
        // New token name length.
        let m in 1 .. MAX_NAME_LENGTH;
        // Ticker length.
        let t in 1 .. MAX_TICKER_LENGTH as u32;
        // Length of the vector of identifiers.
        let i in 1 .. 100;
        // Funding round name length.
        let f in 1 .. MAX_NAME_LENGTH;
        let old_name = AssetName::from(vec![b'N'; n as usize].as_slice());
        let new_name = AssetName::from(vec![b'M'; m as usize].as_slice());
        let origin = make_account::<T>("caller", u).1;
        let ticker = make_token::<T>(origin.clone(), t, n, i, f);
    }: _(origin, ticker, new_name)

    transfer {
        let u in ...;
        // Token name length.
        let n in 1 .. MAX_NAME_LENGTH;
        // Ticker length.
        let t in 1 .. MAX_TICKER_LENGTH as u32;
        // Length of the vector of identifiers.
        let i in 1 .. 100;
        // Funding round name length.
        let f in 1 .. MAX_NAME_LENGTH;
        // Token amount.
        let a in 1 .. 100_000;
        let (_, alice_origin, _) = make_account::<T>("alice", u);
        let (_, _, bob_did) = make_account::<T>("bob", u);
        let ticker = make_token::<T>(alice_origin.clone(), t, n, i, f);
    }: _(alice_origin, ticker, bob_did, a.into())

    issue {
        let u in ...;
        // Token name length.
        let n in 1 .. MAX_NAME_LENGTH;
        // Ticker length.
        let t in 1 .. MAX_TICKER_LENGTH as u32;
        // Length of the vector of identifiers.
        let i in 1 .. 100;
        // Funding round name length.
        let f in 1 .. MAX_NAME_LENGTH;
        // Token amount.
        let a in 1 .. 1_000_000;
        let (_, alice_origin, _) = make_account::<T>("alice", u);
        let (_, _, bob_did) = make_account::<T>("bob", u);
        let ticker = make_token::<T>(alice_origin.clone(), t, n, i, f);
    }: _(alice_origin, ticker, bob_did, a.into(), vec![])

    batch_issue {
        let u in ...;
        // Token name length.
        let n in 1 .. MAX_NAME_LENGTH;
        // Ticker length.
        let t in 1 .. MAX_TICKER_LENGTH as u32;
        // Number of investors.
        let i in 1 .. 100;
        // Funding round name length.
        let f in 1 .. MAX_NAME_LENGTH;
        let alice_origin = make_account::<T>("alice", u).1;
        let ticker = make_token::<T>(alice_origin.clone(), t, n, i, f);
        let mut issue_asset_item = Vec::new();
        for j in 1 .. i {
            let did = make_account::<T>("investor", u + j).2;
            issue_asset_item.push(IssueAssetItem {
                investor_did: did,
                value: 1_000.into()
            });
        }
    }: _(alice_origin, issue_asset_item, ticker)
}
