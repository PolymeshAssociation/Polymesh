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
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_balances as balances;
use polymesh_primitives::{Claim, IdentityId, InvestorUid, SecondaryKey};
use sp_std::{iter, prelude::*};

const SEED: u32 = 0;
const MAX_USER_INDEX: u32 = 1_000;
const NAME: &'static str = "caller";

fn uid_from_name_and_idx(name: &'static str, u: u32) -> InvestorUid {
    let name_u = format!("{}-{}", name, u);
    InvestorUid::from(name_u.as_str())
}

fn make_account_without_did<T: Trait>(
    name: &'static str,
    u: u32,
) -> (T::AccountId, RawOrigin<T::AccountId>) {
    let account: T::AccountId = account(name, u, SEED);
    let origin = RawOrigin::Signed(account.clone());
    let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());
    (account, origin)
}

fn make_account<T: Trait>(
    name: &'static str,
    u: u32,
) -> (T::AccountId, RawOrigin<T::AccountId>, IdentityId) {
    let (account, origin) = make_account_without_did::<T>(name, u);
    let uid = uid_from_name_and_idx(name, u);
    let _ = Module::<T>::register_did(origin.clone().into(), uid, vec![]);
    let did = Module::<T>::get_identity(&account).unwrap();
    (account, origin, did)
}

benchmarks! {
    _ {
        // User account seed.
        let u in 1 .. MAX_USER_INDEX => ();
    }

    register_did {
        let u in ...;
        // Number of secondary items.
        let i in 0 .. 50;
        let origin = make_account_without_did::<T>(NAME, u).1;
        let uid = uid_from_name_and_idx(NAME, u);
        let secondary_keys: Vec<SecondaryKey<T::AccountId>> = iter::repeat(Default::default())
            .take(i as usize)
            .collect();
    }: _(origin, uid, secondary_keys)

    add_claim {
        let u in ...;
        let origin = make_account::<T>("caller", u).1;
        let did = make_account::<T>("target", u).2;
    }: _(origin, did, Claim::NoData, Some(555.into()))
}
