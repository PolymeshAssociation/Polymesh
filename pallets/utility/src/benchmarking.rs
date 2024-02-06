// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Benchmarks for Utility Pallet

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v1::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_core::sr25519::Signature;
use sp_runtime::MultiSignature;

use polymesh_common_utilities::benchs::{user, AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::traits::TestUtilsFn;

use super::*;

// POLYMESH:
const MAX_CALLS: u32 = 30;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

/// Generate `c` no-op system remark calls.
// POLYMESH:
fn make_calls<T: Config>(c: u32) -> Vec<<T as Config>::RuntimeCall> {
    let call: <T as Config>::RuntimeCall =
        frame_system::Call::<T>::remark { remark: vec![] }.into();
    vec![call; c as usize]
}

// POLYMESH:
fn make_relay_tx_users<T: Config + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, User<T>) {
    let alice = UserBuilder::<T>::default()
        .balance(1_000_000u32)
        .generate_did()
        .build("Caller");
    let bob = UserBuilder::<T>::default()
        .balance(1_000_000u32)
        .generate_did()
        .build("Target");

    (alice, bob)
}

// POLYMESH:
fn remark_call_builder<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    signer: &User<T>,
    _: T::AccountId,
) -> (UniqueCall<<T as Config>::RuntimeCall>, Vec<u8>) {
    let call = make_calls::<T>(1).pop().unwrap();
    let nonce: AuthorizationNonce = Pallet::<T>::nonce(signer.account());
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

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>>, <T::RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin: Clone }
    batch {
        let c in 0 .. 1000;
        let mut calls: Vec<<T as Config>::RuntimeCall> = Vec::new();
        for i in 0 .. c {
            let call = frame_system::Call::remark { remark: vec![] }.into();
            calls.push(call);
        }
        let caller = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), calls)
    verify {
        assert_last_event::<T>(Event::BatchCompleted.into())
    }

    batch_all {
        let c in 0 .. 1000;
        let mut calls: Vec<<T as Config>::RuntimeCall> = Vec::new();
        for i in 0 .. c {
            let call = frame_system::Call::remark { remark: vec![] }.into();
            calls.push(call);
        }
        let caller = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), calls)
    verify {
        assert_last_event::<T>(Event::BatchCompleted.into())
    }

    dispatch_as {
        let caller = account("caller", SEED, SEED);
        let call = Box::new(frame_system::Call::remark { remark: vec![] }.into());
        let origin: T::RuntimeOrigin = RawOrigin::Signed(caller).into();
        let pallets_origin: <T::RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin = origin.caller().clone();
        let pallets_origin = Into::<T::PalletsOrigin>::into(pallets_origin);
    }: _(RawOrigin::Root, Box::new(pallets_origin), call)

    force_batch {
        let c in 0 .. 1000;
        let mut calls: Vec<<T as Config>::RuntimeCall> = Vec::new();
        for i in 0 .. c {
            let call = frame_system::Call::remark { remark: vec![] }.into();
            calls.push(call);
        }
        let caller = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), calls)
    verify {
        assert_last_event::<T>(Event::BatchCompleted.into())
    }

    // POLYMESH:
    relay_tx {
        let (caller, target) = make_relay_tx_users::<T>();
        let (call, encoded) = remark_call_builder( &target, caller.account());

        // Rebuild signature from `encoded`.
        let signature = T::OffChainSignature::decode(&mut &encoded[..])
            .expect("OffChainSignature cannot be decoded from a MultiSignature");

    }: _(caller.origin.clone(), target.account(), signature, call)
    verify {
        // NB see comment at `batch` verify section.
    }

    // POLYMESH:
    ensure_root {
        let u = UserBuilder::<T>::default().generate_did().build("ALICE");
    }: {
        assert!(Pallet::<T>::ensure_root(u.origin.into()).is_err());
    }

    // POLYMESH:
    batch_old {
        let c in 0..MAX_CALLS;

        let u = UserBuilder::<T>::default().generate_did().build("ALICE");
        let calls = make_calls::<T>(c);

    }: _(u.origin, calls)
    verify {
        // NB In this case we are using `frame_system::Call::<T>::remark` which makes *no DB
        // operations*. This helps us to fetch the DB read/write ops only from `batch` instead of
        // its batched calls.
        // So there is no way to verify it.
        // The following cases use `balances::transfer` to be able to verify their outputs.
    }

    // POLYMESH:
    batch_atomic {
        let c in 0..MAX_CALLS;

        let alice = UserBuilder::<T>::default().generate_did().build("ALICE");
        let calls = make_calls::<T>(c);
    }: _(alice.origin, calls)
    verify {
        // NB see comment at `batch` verify section.
    }

    // POLYMESH:
    batch_optimistic {
        let c in 0..MAX_CALLS;

        let alice = UserBuilder::<T>::default().generate_did().build("ALICE");
        let calls = make_calls::<T>(c);

    }: _(alice.origin, calls)
    verify {
        // NB see comment at `batch` verify section.
    }

    // POLYMESH:
    as_derivative {
        let index = 1;
        let alice = user::<T>("Alice", 0);
        let call = Box::new(frame_system::Call::remark { remark: vec![] }.into());
    }: _(alice.origin, index, call)
}
