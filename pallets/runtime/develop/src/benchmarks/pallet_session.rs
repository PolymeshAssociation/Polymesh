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

use codec::Decode;
use core::convert::TryInto;
use frame_benchmarking::benchmarks;
use frame_support::traits::{Currency, OnInitialize};
use frame_system::RawOrigin;
use pallet_session::{Call, Pallet as Session};
use pallet_staking::{
    benchmarking::create_validator_with_nominators_with_balance, MAX_NOMINATIONS,
};
use polymesh_common_utilities::{benchs::AccountIdOf, TestUtilsFn};
use sp_runtime::traits::TrailingZeroInput;
use sp_std::prelude::*;
use sp_std::vec;

use polymesh_common_utilities::constants::currency::POLY;

pub struct Pallet<T: Config>(Session<T>);
pub trait Config:
    pallet_session::Config + pallet_session::historical::Config + pallet_staking::Config
{
}

impl<T: Config> OnInitialize<T::BlockNumber> for Pallet<T> {
    fn on_initialize(n: T::BlockNumber) -> frame_support::weights::Weight {
        Session::<T>::on_initialize(n)
    }
}

struct ValidatorInfo<T: Config> {
    controller: T::AccountId,
    keys: T::Keys,
    proof: Vec<u8>,
}

impl<T: Config + TestUtilsFn<AccountIdOf<T>>> ValidatorInfo<T> {
    pub fn build(nominators: u32) -> Result<ValidatorInfo<T>, &'static str>
    where
        <<T as pallet_staking::Config>::Currency as Currency<
            <T as frame_system::Config>::AccountId,
        >>::Balance: From<u128>,
    {
        let balance: u32 = (4_000 * POLY).try_into().unwrap();
        let stash = create_validator_with_nominators_with_balance::<T>(
            nominators,
            MAX_NOMINATIONS as u32,
            balance,
            false,
        )
        .unwrap()
        .0
        .account();
        let controller = pallet_staking::Module::<T>::bonded(&stash).expect("not stash");

        let keys = T::Keys::decode(&mut TrailingZeroInput::zeroes()).unwrap();
        let proof: Vec<u8> = vec![0, 1, 2, 3];

        // Whitelist controller account from further DB operations.
        let controller_key = frame_system::Account::<T>::hashed_key_for(&controller);
        frame_benchmarking::benchmarking::add_to_whitelist(controller_key.into());

        Ok(Self {
            controller,
            keys,
            proof,
        })
    }
}

benchmarks! {
    where_clause {
        where
            T: TestUtilsFn<AccountIdOf<T>>,
            <<T as pallet_staking::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: From<u128>,
    }

    set_keys {
        let n = MAX_NOMINATIONS as u32;
        let validator = ValidatorInfo::<T>::build(n).unwrap();
        let proof = validator.proof.clone();
        let keys = validator.keys.clone();

    }: _(RawOrigin::Signed(validator.controller), keys, proof)

    purge_keys {
        let n = MAX_NOMINATIONS as u32;
        let validator = ValidatorInfo::<T>::build(n).unwrap();
        let controller = RawOrigin::Signed(validator.controller.clone());

        Session::<T>::set_keys(controller.clone().into(), validator.keys, validator.proof).unwrap();

    }: _(controller)
}
