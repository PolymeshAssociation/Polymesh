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

// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
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
// limitations under the License

//! Some configurable implementations as associated type for the substrate runtime.
//! Auxillary struct/enums

use frame_election_provider_support::BalancingConfig;
use frame_support::traits::{Currency, OnUnbalanced};
use frame_system as system;
use sp_runtime::traits::Convert;

use pallet_authorship as authorship;
use pallet_balances as balances;
use polymesh_primitives::Balance;

use crate::NegativeImbalance;

pub struct Author<R>(sp_std::marker::PhantomData<R>);

impl<R> OnUnbalanced<NegativeImbalance<R>> for Author<R>
where
    R: balances::Config + authorship::Config,
    <R as system::Config>::AccountId: From<polymesh_primitives::AccountId>,
    <R as system::Config>::AccountId: Into<polymesh_primitives::AccountId>,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
        if let Some(author) = authorship::Pallet::<R>::author() {
            <balances::Module<R>>::resolve_creating(&author, amount);
        }
    }
}

/// Struct that handles the conversion of Balance -> `u128`. This is used for staking's election
/// calculation.
pub struct CurrencyToVoteHandler<R>(sp_std::marker::PhantomData<R>);

impl<R> CurrencyToVoteHandler<R>
where
    R: balances::Config,
{
    fn factor() -> Balance {
        let issuance: Balance = <balances::Module<R>>::total_issuance();
        (issuance / u64::max_value() as Balance).max(1)
    }
}

impl<R> Convert<Balance, u64> for CurrencyToVoteHandler<R>
where
    R: balances::Config,
{
    fn convert(x: Balance) -> u64 {
        (x / Self::factor()) as u64
    }
}

impl<R> Convert<u128, Balance> for CurrencyToVoteHandler<R>
where
    R: balances::Config,
{
    fn convert(x: u128) -> Balance {
        x * Self::factor()
    }
}

pub struct EstimateCallFeeMax;

impl<Call, Balance: From<u32>> frame_support::traits::EstimateCallFee<Call, Balance>
    for EstimateCallFeeMax
{
    fn estimate_call_fee(_: &Call, _: frame_support::dispatch::PostDispatchInfo) -> Balance {
        u32::MAX.into()
    }
}

/// A source of random balance for NposSolver, which is meant to be run by the OCW election miner.
pub struct OffchainRandomBalancing;

impl frame_support::pallet_prelude::Get<Option<BalancingConfig>> for OffchainRandomBalancing {
    fn get() -> Option<BalancingConfig> {
        use codec::Decode;
        use sp_runtime::traits::TrailingZeroInput;

        let iterations = match crate::MINER_MAX_ITERATIONS {
            0 => 0,
            max => {
                let seed = sp_io::offchain::random_seed();
                let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
                    .expect("input is padded with zeroes; qed")
                    % max.saturating_add(1);
                random as usize
            }
        };

        let config = BalancingConfig {
            iterations,
            tolerance: 0,
        };
        Some(config)
    }
}

/// The numbers configured here could always be more than the the maximum limits of staking pallet
/// to ensure election snapshot will not run out of memory. For now, we set them to smaller values
/// since the staking is bounded and the weight pipeline takes hours for this single pallet.
pub struct ElectionProviderBenchmarkConfig;

impl pallet_election_provider_multi_phase::BenchmarkingConfig for ElectionProviderBenchmarkConfig {
    const VOTERS: [u32; 2] = [1000, 2000];
    const TARGETS: [u32; 2] = [500, 1000];
    const ACTIVE_VOTERS: [u32; 2] = [500, 800];
    const DESIRED_TARGETS: [u32; 2] = [200, 400];
    const SNAPSHOT_MAXIMUM_VOTERS: u32 = 1000;
    const MINER_MAXIMUM_VOTERS: u32 = 1000;
    const MAXIMUM_TARGETS: u32 = 300;
}
