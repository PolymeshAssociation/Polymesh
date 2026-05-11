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

fn relay_remark_call_builder<T: Config>(
    signer: &User<T>,
) -> (
    Box<<T as pallet_utility::Config>::RuntimeCall>,
    T::Moment,
    Vec<u8>,
) {
    let call: Box<<T as pallet_utility::Config>::RuntimeCall> =
        Box::new(frame_system::Call::<T>::remark { remark: vec![] }.into());
    let expires_at = pallet_timestamp::Pallet::<T>::get() + 1000u32.into();
    let nonce: AuthorizationNonce = RelayTxNonces::<T>::get(signer.account());
    let scoped_call =
        ChainScopedMessage::<T, _>::new_unchecked(nonce, RELAY_TX_LABEL, expires_at, &call);

    // Signer signs the relay call.
    // NB: Decode as T::OffChainSignature because there is not type constraints in
    // `T::OffChainSignature` to limit it.
    let raw_signature = scoped_call
        .native_sign(
            &signer
                .secret
                .as_ref()
                .expect("User without secret key")
                .to_bytes(),
        )
        .expect("Data cannot be signed")
        .0;
    let encoded = MultiSignature::from(Signature::from_raw(raw_signature)).encode();

    (call, expires_at, encoded)
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

    revoke_subsidy {
        let (payer, user) = setup_users::<T>();
        let limit = 100u128;
        // setup authorization
        Relayer::<T>::approve_subsidy(
            payer.origin().into(), user.account(), limit
        ).unwrap();
    }: _(payer.origin(), user.account())
    verify {
        let user_key = user.account();
        let payer_key = payer.account();
        assert!(PendingSubsidies::<T>::take(&user_key, &payer_key).is_none());
        assert_subsidy(user, None);
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
        let (call, expires_at, encoded) = relay_remark_call_builder(&target);

        // Rebuild signature from `encoded`.
        let signature = T::OffChainSignature::decode(&mut &encoded[..])
            .expect("OffChainSignature cannot be decoded from a MultiSignature");

    }: _(caller.origin.clone(), target.account(), signature, call, expires_at)
    verify {
        // NB see comment at `batch` verify section.
    }
}
