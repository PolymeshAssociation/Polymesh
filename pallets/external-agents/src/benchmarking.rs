// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
use frame_benchmarking::benchmarks;
use polymesh_common_utilities::benchs::{make_asset, user, AccountIdOf, User};
use polymesh_common_utilities::traits::asset::Config as Asset;
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::{AuthorizationData, ExtrinsicPermissions, PalletPermissions, Ticker};
use sp_std::prelude::*;

pub(crate) const SEED: u32 = 0;
const MAX_PALLETS: u32 = 19;

fn setup<T: Asset + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, Ticker) {
    let owner = user("owner", SEED);
    let ticker = make_asset::<T>(&owner, None);
    (owner, ticker)
}

fn perms(n: u32) -> ExtrinsicPermissions {
    ExtrinsicPermissions::elems(
        (0..=n as u64).map(|w| PalletPermissions::entire_pallet(Ticker::generate(w).into())),
    )
}

fn add_auth<T: Asset + TestUtilsFn<AccountIdOf<T>>>(
    owner: &User<T>,
    ticker: Ticker,
) -> (User<T>, u64) {
    let other = user("other", SEED);
    let auth_id = pallet_identity::Module::<T>::add_auth(
        owner.did(),
        other.did().into(),
        AuthorizationData::BecomeAgent(ticker, AgentGroup::Full),
        None,
    );
    (other, auth_id)
}

fn setup_removal<T: Asset + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, User<T>, Ticker) {
    let (owner, ticker) = setup::<T>();
    let (other, auth_id) = add_auth::<T>(&owner, ticker);
    Module::<T>::accept_become_agent(other.origin().into(), auth_id).unwrap();
    (owner, other, ticker)
}

fn custom_group<T: Config>(owner: User<T>, ticker: Ticker) {
    Module::<T>::create_group(owner.origin().into(), ticker, <_>::default()).unwrap();
}

benchmarks! {
    where_clause { where T: Asset, T: TestUtilsFn<AccountIdOf<T>> }

    create_group {
        let p in 0..MAX_PALLETS;

        let perms = perms(p);
        let (owner, ticker) = setup::<T>();
        assert_eq!(AGId(0), AGIdSequence::get(ticker));
    }: _(owner.origin, ticker, perms)
    verify {
        assert_eq!(AGId(1), AGIdSequence::get(ticker));
    }

    set_group_permissions {
        let p in 0..MAX_PALLETS;

        let (owner, ticker) = setup::<T>();
        custom_group(owner.clone(), ticker);

        let perms = perms(p);
        let perms2 = perms.clone();
    }: _(owner.origin(), ticker, AGId(1), perms)
    verify {
        assert_eq!(Some(perms2), GroupPermissions::get(ticker, AGId(1)));
    }

    remove_agent {
        let (owner, other, ticker) = setup_removal::<T>();
    }: _(owner.origin(), ticker, other.did())
    verify {
        assert_eq!(None, GroupOfAgent::get(ticker, other.did()));
    }

    abdicate {
        let (_, other, ticker) = setup_removal::<T>();
    }: _(other.origin(), ticker)
    verify {
        assert_eq!(None, GroupOfAgent::get(ticker, other.did()));
    }

    change_group_custom {
        let (owner, other, ticker) = setup_removal::<T>();
        custom_group(owner.clone(), ticker);
        let group = AgentGroup::Custom(AGId(1));
    }: change_group(owner.origin(), ticker, other.did(), group)
    verify {
        assert_eq!(Some(group), GroupOfAgent::get(ticker, other.did()));
    }

    change_group_builtin {
        let (owner, other, ticker) = setup_removal::<T>();
    }: change_group(owner.origin(), ticker, other.did(), AgentGroup::ExceptMeta)
    verify {
        assert_eq!(Some(AgentGroup::ExceptMeta), GroupOfAgent::get(ticker, other.did()));
    }

    accept_become_agent {
        let (owner, ticker) = setup::<T>();
        let (other, auth_id) = add_auth::<T>(&owner, ticker);
    }: _(other.origin(), auth_id)
    verify {
        assert!(GroupOfAgent::get(ticker, other.did()).is_some());
    }

    create_group_and_add_auth {
        let p in 0..MAX_PALLETS;

        let perms = perms(p);
        let (owner, ticker) = setup::<T>();
        assert_eq!(AGId(0), AGIdSequence::get(ticker));
    }: _(owner.origin, ticker, perms, owner.did(), None)
    verify {
        assert_eq!(AGId(1), AGIdSequence::get(ticker));
    }
}
