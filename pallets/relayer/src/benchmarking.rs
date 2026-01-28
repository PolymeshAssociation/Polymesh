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

use crate::*;

use frame_benchmarking::benchmarks;
use pallet_identity::benchmarking::{user_without_did, User};
use polymesh_primitives::Balance;

type Relayer<T> = crate::Pallet<T>;

pub(crate) const SEED: u32 = 0;

fn setup_users<T: Config>() -> (User<T>, User<T>) {
    let payer = user_without_did::<T>("payer", SEED);
    let user = user_without_did::<T>("user", SEED);
    (payer, user)
}

fn setup_subsidy<T: Config>(limit: u128) -> (User<T>, User<T>) {
    let (payer, user) = setup_users::<T>();
    // Add subsidy
    Subsidies::<T>::insert(
        &user.account(),
        Subsidy {
            paying_key: payer.account(),
            remaining: limit,
        },
    );

    (payer, user)
}

#[track_caller]
fn assert_subsidy<T: Config>(user: User<T>, subsidy: Option<(User<T>, Balance)>) {
    let expect = subsidy.map(|(payer, limit)| Subsidy {
        paying_key: payer.account(),
        remaining: limit,
    });
    assert_eq!(Subsidies::<T>::get(user.account()), expect);
}

benchmarks! {
    where_clause { where T: Config }

    approve_subsidy {
        let (payer, user) = setup_users::<T>();
    }: _(payer.origin(), user.account(), 0u128)

    accept_subsidy {
        let (payer, user) = setup_users::<T>();
        let limit = 100u128;
        // setup authorization
        Relayer::<T>::approve_subsidy(
            payer.origin().into(), user.account(), limit
        ).unwrap();
    }: _(user.origin(), payer.account())
    verify {
        assert_subsidy(user, Some((payer, limit)));
    }

    remove_subsidy {
        let (payer, user) = setup_subsidy::<T>(0u128);
    }: _(payer.origin(), user.account(), payer.account())
    verify {
        assert_subsidy(user, None);
    }

    update_polyx_limit {
        let limit = 1_000u128;
        let (payer, user) = setup_subsidy::<T>(42u128);
    }: _(payer.origin(), user.account(), limit)
    verify {
        assert_subsidy(user, Some((payer, limit)));
    }

    increase_polyx_limit {
        let limit = 500u128;
        let (payer, user) = setup_subsidy::<T>(0u128);
    }: _(payer.origin(), user.account(), limit)
    verify {
        assert_subsidy(user, Some((payer, limit)));
    }

    decrease_polyx_limit {
        let limit = 500u128;
        let (payer, user) = setup_subsidy::<T>(1_000u128);
    }: _(payer.origin(), user.account(), limit)
    verify {
        assert_subsidy(user, Some((payer, limit)));
    }
}
