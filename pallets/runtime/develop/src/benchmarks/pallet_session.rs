// This file is part of Substrate.

// Copyright (C) 2020 Parity Technologies (UK) Ltd.
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

//! Benchmarks for the Session Pallet.
// This is separated into its own crate due to cyclic dependency issues.

// Modified by Polymath Inc - 4th January 2021
// - It uses our Staking pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_std::vec;

use frame_benchmarking::benchmarks;
use frame_support::{storage::StorageMap, traits::OnInitialize};
use frame_system::RawOrigin;
use pallet_session::{Module as Session, *};
use pallet_staking::{benchmarking::create_validator_with_nominators, MAX_NOMINATIONS};

pub struct Module<T: Trait>(pallet_session::Module<T>);
pub trait Trait:
    pallet_session::Trait + pallet_session::historical::Trait + pallet_staking::Trait
{
}

impl<T: Trait> OnInitialize<T::BlockNumber> for Module<T> {
    fn on_initialize(n: T::BlockNumber) -> frame_support::weights::Weight {
        pallet_session::Module::<T>::on_initialize(n)
    }
}

benchmarks! {
    _ {	}

    set_keys {
        let n = MAX_NOMINATIONS as u32;
        let v_stash = create_validator_with_nominators::<T>(
            n,
            MAX_NOMINATIONS as u32,
            false)?;
        let v_controller = pallet_staking::Module::<T>::bonded(&v_stash).ok_or("not stash")?;
        let keys = T::Keys::default();
        let proof: Vec<u8> = vec![0,1,2,3];
        // Whitelist controller account from further DB operations.
        let v_controller_key = frame_system::Account::<T>::hashed_key_for(&v_controller);
        frame_benchmarking::benchmarking::add_to_whitelist(v_controller_key.into());
    }: _(RawOrigin::Signed(v_controller), keys, proof)

    purge_keys {
        let n = MAX_NOMINATIONS as u32;
        let v_stash = create_validator_with_nominators::<T>(n, MAX_NOMINATIONS as u32, false)?;
        let v_controller = pallet_staking::Module::<T>::bonded(&v_stash).ok_or("not stash")?;
        let keys = T::Keys::default();
        let proof: Vec<u8> = vec![0,1,2,3];
        Session::<T>::set_keys(RawOrigin::Signed(v_controller.clone()).into(), keys, proof)?;
        // Whitelist controller account from further DB operations.
        let v_controller_key = frame_system::Account::<T>::hashed_key_for(&v_controller);
        frame_benchmarking::benchmarking::add_to_whitelist(v_controller_key.into());
    }: _(RawOrigin::Signed(v_controller))
}
