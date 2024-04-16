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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_staking.
pub trait WeightInfo {
	fn bond() -> Weight;
    fn bond_extra() -> Weight;
    fn unbond() -> Weight;
    fn withdraw_unbonded_update(s: u32) -> Weight;
    fn withdraw_unbonded_kill(s: u32) -> Weight;
    fn validate() -> Weight;
    fn nominate(n: u32) -> Weight;
    fn chill() -> Weight;
    fn set_payee() -> Weight;
    fn set_controller() -> Weight;
    fn force_no_eras() -> Weight;
    fn force_new_era() -> Weight;
    fn force_new_era_always() -> Weight;
    fn set_invulnerables(v: u32) -> Weight;
    fn force_unstake(s: u32) -> Weight;
    fn cancel_deferred_slash(s: u32) -> Weight;
    fn payout_stakers(n: u32) -> Weight;
    fn payout_stakers_alive_controller(n: u32) -> Weight;
    fn rebond(l: u32) -> Weight;
    fn reap_stash(s: u32) -> Weight;
    fn new_era(v: u32, n: u32) -> Weight;
	// Polymesh Change
    // -----------------------------------------------------------------
	fn set_validator_count(c: u32) -> Weight;
	fn set_min_bond_threshold() -> Weight;
    fn add_permissioned_validator() -> Weight;
    fn remove_permissioned_validator() -> Weight;
    fn set_commission_cap(m: u32) -> Weight;
	fn set_history_depth(e: u32) -> Weight;
	fn do_slash(l: u32) -> Weight;
    fn payout_all(v: u32, n: u32) -> Weight;
    fn submit_solution_better(v: u32, n: u32, a: u32, w: u32) -> Weight;
    fn change_slashing_allowed_for() -> Weight;
    fn update_permissioned_validator_intended_count() -> Weight;
    fn increase_validator_count() -> Weight;
    fn scale_validator_count() -> Weight;
    fn chill_from_governance(s: u32) -> Weight;
	// -----------------------------------------------------------------
}
