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
//use cryptography::claim_proofs::{compute_cdd_id, compute_scope_id};
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_balances as balances;
use polymesh_primitives::{Claim, CountryCode, IdentityId, InvestorUid, Scope};
use sp_std::prelude::*;

const SEED: u32 = 0;
const MAX_USER_INDEX: u32 = 1_000;

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
    _ {
        // User account seed.
        let u in 1 .. MAX_USER_INDEX => ();
    }

    // register_did {
    //     let u in ...;
    //     // Number of secondary items.
    //     let i in 0 .. 50;
    //     let origin = make_account_without_did::<T>(NAME, u).1;
    //     let uid = uid_from_name_and_idx(NAME, u);
    //     let secondary_keys: Vec<SecondaryKey<T::AccountId>> = iter::repeat(Default::default())
    //         .take(i as usize)
    //         .collect();
    // }: _(origin, uid, secondary_keys)

    // add_claim {
    //     let u in ...;
    //     let (_, origin, origin_did) = make_account::<T>("caller", u);
    //     let uid = uid_from_name_and_idx("caller", u);
    //     let ticker = Ticker::try_from(vec![b'T'; 12].as_slice()).unwrap();
    //     let st_scope = Scope::Identity(IdentityId::try_from(ticker.as_slice()).unwrap());
    //     let scope_claim = InvestorZKProofData::make_scope_claim(&ticker, &uid);
    //     let scope_id = compute_scope_id(&scope_claim).compress().to_bytes().into();
    //     let inv_proof = InvestorZKProofData::new(&origin_did, &uid, &ticker);
    //     let cdd_claim = InvestorZKProofData::make_cdd_claim(&origin_did, &uid);
    //     let cdd_id = compute_cdd_id(&cdd_claim).compress().to_bytes().into();
    //     let conf_scope_claim = Claim::InvestorZKProof(st_scope, scope_id, cdd_id, inv_proof);
    // }: _(origin, origin_did, conf_scope_claim, Some(666.into()))

    add_claim {
        let u in ...;
        let (_, origin, origin_did) = make_account::<T>("caller", u);
        let (_, _, target_did) = make_account::<T>("target", u);
    }: _(origin, target_did, Claim::Jurisdiction(CountryCode::BB, Scope::Identity(origin_did)), Some(666.into()))
}
