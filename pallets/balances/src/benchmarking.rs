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

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use sp_runtime::traits::Bounded;
use polymesh_primitives::{IdentityId, InvestorUid, CddId, Claim};

use crate::Module as Balances;

const SEED: u32 = 0;

benchmarks! {
	_ { }

	// Benchmark `transfer` extrinsic with the worst possible conditions:
	// * Transfer will create the recipient account.
	transfer {
		let caller = whitelisted_caller();
		// Give some balance to the caller
		let transfer_amount = T::Balance::from(500);
		let _ = <Balances<T> as Currency<_>>::make_free_balance_be(&caller, transfer_amount*2.into());
		// Give a valid CDD to the receiver
		let recipient: T::AccountId = account("recipient", 0, SEED);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		T::Identity::create_did_with_cdd(recipient.clone());
	}: transfer(RawOrigin::Signed(caller.clone()), recipient_lookup, transfer_amount)
	verify {
		assert_eq!(Balances::<T>::free_balance(&caller), transfer_amount);
		assert_eq!(Balances::<T>::free_balance(&recipient), transfer_amount);
	}
}
