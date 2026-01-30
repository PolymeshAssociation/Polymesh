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
use sp_core::sr25519::Signature;
use sp_runtime::MultiSignature;

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

/// Generate `c` no-op system remark calls.
fn make_calls<T: Config>(c: u32) -> Vec<<T as pallet_utility::Config>::RuntimeCall> {
    let call: <T as pallet_utility::Config>::RuntimeCall =
        frame_system::Call::<T>::remark { remark: vec![] }.into();
    vec![call; c as usize]
}

fn remark_call_builder<T: Config>(
    signer: &User<T>,
    _: T::AccountId,
) -> (
    UniqueCall<<T as pallet_utility::Config>::RuntimeCall>,
    Vec<u8>,
) {
    let call = make_calls::<T>(1).pop().unwrap();
    let nonce: AuthorizationNonce = RelayTxNonces::<T>::get(signer.account());
    let call = UniqueCall::new(nonce, call);

    // Signer signs the relay call.
    // NB: Decode as T::OffChainSignature because there is not type constraints in
    // `T::OffChainSignature` to limit it.
    let raw_signature: [u8; 64] = signer
        .sign(&call.encode())
        .expect("Data cannot be signed")
        .0;
    let encoded = MultiSignature::from(Signature::from_raw(raw_signature)).encode();

    (call, encoded)
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

    relay_tx {
        let (caller, target) = setup_users::<T>();
        let (call, encoded) = remark_call_builder( &target, caller.account());

        // Rebuild signature from `encoded`.
        let signature = T::OffChainSignature::decode(&mut &encoded[..])
            .expect("OffChainSignature cannot be decoded from a MultiSignature");

    }: _(caller.origin.clone(), target.account(), signature, call)
    verify {
        // NB see comment at `batch` verify section.
    }
}
