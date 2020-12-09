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

//! Balances pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Module as Balances;
use polymesh_common_utilities::benchs::UserBuilder;

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

fn make_worst_memo() -> Option<Memo> {
    Some(Memo([7u8; 32]))
}

benchmarks! {
    _ { }

    // Benchmark `transfer` extrinsic with the worst possible conditions:
    // * Transfer will create the recipient account.
    transfer {
        let amount = T::Balance::from(500);
        let caller = UserBuilder::<T>::default().balance(1200).generate_did().build("caller");
        let recipient = UserBuilder::<T>::default().balance(0).generate_did().build( "recipient");
    }: _(caller.origin(), recipient.lookup(), amount)
    verify {
        assert_eq!(Balances::<T>::free_balance(&caller.account), (1200-500).into());
        assert_eq!(Balances::<T>::free_balance(&recipient.account), amount);
    }

    transfer_with_memo {
        let caller = UserBuilder::<T>::default().balance(1000).generate_did().build("caller");
        let recipient = UserBuilder::<T>::default().balance(0).generate_did().build("recipient");
        let amount = 42.into();
        let memo = make_worst_memo();

    }: _(caller.origin(), recipient.lookup(), amount, memo)
    verify {
        assert_eq!(Balances::<T>::free_balance(&caller.account), (1000-42).into());
        assert_eq!(Balances::<T>::free_balance(&recipient.account), amount);
    }

    deposit_block_reward_reserve_balance {
        let caller = UserBuilder::<T>::default().balance(1000).generate_did().build("caller");
        let amount = 500.into();
    }: _(caller.origin(), amount)
    verify {
        assert_eq!(Balances::<T>::free_balance(&caller.account), (1000-500).into());
        assert_eq!(Balances::<T>::block_rewards_reserve_balance(), amount);
    }

    set_balance {
        let caller = UserBuilder::<T>::default().balance(1000).generate_did().build("caller");
        let free_balance :T::Balance = 1_000_000.into();
        let reserved_balance :T::Balance = 100.into();
    }: _(RawOrigin::Root, caller.lookup(), free_balance.clone(), reserved_balance)
    verify {
        assert_eq!(Balances::<T>::free_balance(&caller.account), free_balance);
        assert_eq!(Balances::<T>::reserved_balance(&caller.account), reserved_balance);
    }

    force_transfer {
        let source = UserBuilder::<T>::default().balance(1000).generate_did().build("source");
        let dest = UserBuilder::<T>::default().balance(1).generate_did().build("dest");
        let amount = 500.into();
    }: _(RawOrigin::Root, source.lookup(), dest.lookup(), amount)
    verify {
        assert_eq!(Balances::<T>::free_balance(&source.account), (1000-500).into());
        assert_eq!(Balances::<T>::free_balance(&dest.account), (1+500).into());
    }

    burn_account_balance {
        let caller = UserBuilder::<T>::default().balance(1000).generate_did().build("caller");
        let amount = 500.into();
    }: _(caller.origin(), amount)
    verify {
        assert_eq!(Balances::<T>::free_balance(&caller.account), (1000-500).into());
    }
}
