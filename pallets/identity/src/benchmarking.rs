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
use polymesh_primitives::{Claim, CountryCode, IdentityId, InvestorUid, Scope, Ticker};
use polymesh_common_utilities::traits::identity::Trait as IdentityTrait;
use sp_std::prelude::*;
use schnorrkel::Signature;

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

    cdd_register_did {
        let u in ...;
        // Number of secondary items.
        let i in 0 .. 50;

        let (_, origin, origin_did) = make_account::<T>("caller", u);
        <T as IdentityTrait>::CddServiceProviders::add_member(origin_did);

        let account: T::AccountId = account("target", u, SEED);

        let secondary_keys: Vec<secondary_key::api::SecondaryKey<T::AccountId>> = iter::repeat(Default::default())
            .take(i as usize)
            .collect();
    }: _(origin, account, secondary_keys)

    add_investor_uniqueness_claim {
        let u in ...;
        let (account, origin) = make_account_without_did::<T>("caller", u);

        let did = IdentityId::from([152u8,25,31,70,229,131,2,22,68,84,54,151,136,3,105,122,94,58,182,27,30,137,81,212,254,154,230,123,171,97,74,95]);
        Module::<T>::link_did(account.clone(), did);

        let cdd_id = CddId::from([102u8,210,32,212,213,80,255,99,142,30,202,20,220,131,109,106,137,12,137,191,123,156,212,20,215,87,23,42,84,181,128,73]);
        let cdd_claim = Claim::CustomerDueDiligence(cdd_id);
        Module::<T>::base_add_claim(did, cdd_claim, did, Some(666.into()));

        let scope = Scope::Custom([228u8,152,116,104,5,8,30,188,143,185,10,208].to_vec());
        let scope_did = IdentityId::from([2u8,72,20,154,7,96,116,105,155,74,227,252,172,18,200,203,137,107,200,210,194,71,250,41,108,172,100,107,223,114,182,101]);
        let conf_scope_claim = Claim::InvestorUniqueness(scope, scope_did, cdd_id);

        let inv_proof = InvestorZKProofData(Signature::from_bytes(&[216u8,224,57,254,200,45,150,202,12,108,226,233,148,213,237,7,35,150,142,18,127,146,162,19,161,164,95,67,181,100,156,25,201,210,209,165,182,74,184,145,230,255,215,144,223,100,100,147,226,58,142,92,103,153,153,204,123,120,133,113,218,51,208,132]).unwrap());

    }: _(origin, did, conf_scope_claim, inv_proof, Some(666.into()))

    add_claim {
        let u in ...;
        let (_, origin, origin_did) = make_account::<T>("caller", u);
        let (_, _, target_did) = make_account::<T>("target", u);
    }: _(origin, target_did, Claim::Jurisdiction(CountryCode::BB, Scope::Identity(origin_did)), Some(666.into()))
}
