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
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_balances as balances;
use polymesh_common_utilities::traits::identity::TargetIdAuthorization;
use polymesh_primitives::{
    AuthorizationData, Claim, CountryCode, IdentityId, InvestorUid, Permissions, Scope, Signatory,
};
use schnorrkel::Signature;
use sp_std::prelude::*;

const SEED: u32 = 0;
fn uid_from_name_and_idx(name: &'static str, u: u32) -> InvestorUid {
    InvestorUid::from((name, u).encode().as_slice())
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
    _ {}

    register_did {
        // Number of secondary items.
        let i in 0 .. 50;

        make_cdd_account::<T>(SEED);
        let (_, origin) = make_account_without_did::<T>("caller", SEED);
        let uid = uid_from_name_and_idx("caller", SEED);
        let secondary_keys = generate_secondary_keys::<T>(i as usize);
    }: _(origin, uid, secondary_keys)
}
