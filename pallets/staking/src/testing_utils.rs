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

//! Testing utils for staking. Provides some common functions to setup staking state, such as
//! bonding validators, nominators, and generating different types of solutions.

use crate::{Pallet as Staking, *};
use frame_benchmarking::account;
use frame_system::RawOrigin;
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use sp_io::hashing::blake2_256;

use frame_election_provider_support::SortedListProvider;
use frame_support::{pallet_prelude::*, traits::Currency};
use sp_runtime::Perbill;
use sp_std::prelude::*;

const SEED: u32 = 0;

// Polymesh change
// -----------------------------------------------------------------
use polymesh_common_utilities::benchs::{AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::{AuthorizationData, Permissions, Signatory};
// -----------------------------------------------------------------

/// This function removes all validators and nominators from storage.
pub fn clear_validators_and_nominators<T: Config>() {
    #[allow(deprecated)]
    Validators::<T>::remove_all();

    // whenever we touch nominators counter we should update `T::VoterList` as well.
    #[allow(deprecated)]
    Nominators::<T>::remove_all();

    // NOTE: safe to call outside block production
    T::VoterList::unsafe_clear();
}

/// Grab a funded user.
pub fn create_funded_user<T>(string: &'static str, n: u32, balance: u32) -> User<T>
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    // Polymesh change
    // -----------------------------------------------------------------
    let _ = T::Currency::issue(balance.into());
    UserBuilder::<T>::default()
        .balance(balance)
        .seed(n)
        .generate_did()
        .build(string)
    // -----------------------------------------------------------------
}

/// Create a stash and controller pair.
pub fn create_stash_controller<T>(
    n: u32,
    balance: u32,
    destination: RewardDestination<T::AccountId>,
) -> Result<(User<T>, User<T>), &'static str>
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    let stash = create_funded_user::<T>("stash", n, balance);
    let controller = UserBuilder::<T>::default()
        .balance(balance)
        .seed(n)
        .build("controller");

    // Polymesh change
    // -----------------------------------------------------------------
    // Attach the controller key as secondary key of the stash
    let auth_id = pallet_identity::Module::<T>::add_auth(
        stash.did(),
        Signatory::Account(controller.account()),
        AuthorizationData::JoinIdentity(Permissions::default()),
        None,
    )?;
    pallet_identity::Module::<T>::join_identity_as_key(controller.origin().into(), auth_id)?;
    // -----------------------------------------------------------------

    let controller_lookup = controller.lookup();
    Staking::<T>::bond(
        stash.origin().into(),
        controller_lookup,
        (balance / 10).into(),
        destination,
    )?;
    Ok((stash, controller))
}

/// Create a stash and controller pair, where the controller is dead, and payouts go to controller.
/// This is used to test worst case payout scenarios.
pub fn create_stash_and_dead_controller<T: Config>(
    n: u32,
    balance: u32,
    destination: RewardDestination<T::AccountId>,
) -> Result<(User<T>, User<T>), &'static str>
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    let stash = create_funded_user::<T>("stash", n, balance);
    let controller_account: T::AccountId = account("controller", n, 100);
    let controller = User {
        account: controller_account.clone(),
        origin: RawOrigin::Signed(controller_account),
        did: None,
        secret: None,
    };

    // Polymesh change
    // -----------------------------------------------------------------
    // Attach the controller key as secondary key of the stash
    let auth_id = pallet_identity::Module::<T>::add_auth(
        stash.did(),
        Signatory::Account(controller.account()),
        AuthorizationData::JoinIdentity(Permissions::default()),
        None,
    )?;
    pallet_identity::Module::<T>::join_identity_as_key(controller.origin().into(), auth_id)?;
    // -----------------------------------------------------------------

    let controller_lookup = controller.lookup();
    Staking::<T>::bond(
        stash.origin().into(),
        controller_lookup,
        (balance / 10).into(),
        destination,
    )?;
    Ok((stash, controller))
}

/// create `max` validators.
pub fn create_validators<T>(
    max: u32,
    balance: u32,
) -> Result<Vec<AccountIdLookupOf<T>>, &'static str>
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    create_validators_with_seed::<T>(max, balance, 0)
}

/// create `max` validators, with a seed to help unintentional prevent account collisions.
pub fn create_validators_with_seed<T>(
    max: u32,
    balance: u32,
    seed: u32,
) -> Result<Vec<AccountIdLookupOf<T>>, &'static str>
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    Staking::<T>::set_commission_cap(RawOrigin::Root.into(), Perbill::from_percent(50)).unwrap();
    let mut validators: Vec<AccountIdLookupOf<T>> = Vec::with_capacity(max as usize);
    for i in 0..max {
        let (stash, controller) =
            create_stash_controller::<T>(i + seed, balance, RewardDestination::Staked)?;
        let validator_prefs = ValidatorPrefs {
            commission: Perbill::from_percent(50),
            ..Default::default()
        };
        // Polymesh change
        // -----------------------------------------------------------------
        Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), stash.did(), Some(2))?;
        // -----------------------------------------------------------------
        Staking::<T>::validate(controller.origin().into(), validator_prefs)?;
        validators.push(stash.lookup());
    }

    Ok(validators)
}

/// This function generates validators and nominators who are randomly nominating
/// `edge_per_nominator` random validators (until `to_nominate` if provided).
///
/// NOTE: This function will remove any existing validators or nominators to ensure
/// we are working with a clean state.
///
/// Parameters:
/// - `validators`: number of bonded validators
/// - `nominators`: number of bonded nominators.
/// - `edge_per_nominator`: number of edge (vote) per nominator.
/// - `randomize_stake`: whether to randomize the stakes.
/// - `to_nominate`: if `Some(n)`, only the first `n` bonded validator are voted upon. Else, all of
///   them are considered and `edge_per_nominator` random validators are voted for.
///
/// Return the validators chosen to be nominated.
pub fn create_validators_with_nominators_for_era<T: Config>(
    validators: u32,
    nominators: u32,
    edge_per_nominator: usize,
    randomize_stake: bool,
    to_nominate: Option<u32>,
) -> Result<Vec<AccountIdLookupOf<T>>, &'static str>
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    clear_validators_and_nominators::<T>();

    Staking::<T>::set_commission_cap(RawOrigin::Root.into(), Perbill::from_percent(50)).unwrap();
    let mut validators_stash: Vec<AccountIdLookupOf<T>> = Vec::with_capacity(validators as usize);
    let mut rng = ChaChaRng::from_seed(SEED.using_encoded(blake2_256));

    // Create validators
    for i in 0..validators {
        let balance_factor = if randomize_stake {
            rng.next_u32() % 255 + 10
        } else {
            100u32
        };
        let (v_stash, v_controller) =
            create_stash_controller::<T>(i, balance_factor, RewardDestination::Staked)?;
        Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), v_stash.did(), Some(2))?;
        let validator_prefs = ValidatorPrefs {
            commission: Perbill::from_percent(50),
            ..Default::default()
        };
        Staking::<T>::validate(v_controller.origin().into(), validator_prefs)?;
        let stash_lookup = v_stash.lookup();
        validators_stash.push(stash_lookup.clone());
    }

    let to_nominate = to_nominate.unwrap_or(validators_stash.len() as u32) as usize;
    let validator_chosen = validators_stash[0..to_nominate].to_vec();

    // Create nominators
    for j in 0..nominators {
        let balance_factor = if randomize_stake {
            rng.next_u32() % 100_000_000u32 + 10_000_000u32
        } else {
            10_000_000u32
        };
        let (_n_stash, n_controller) =
            create_stash_controller::<T>(u32::MAX - j, balance_factor, RewardDestination::Staked)?;

        // Have them randomly validate
        let mut available_validators = validator_chosen.clone();
        let mut selected_validators: Vec<AccountIdLookupOf<T>> =
            Vec::with_capacity(edge_per_nominator);

        for _ in 0..validators.min(edge_per_nominator as u32) {
            let selected = rng.next_u32() as usize % available_validators.len();
            let validator = available_validators.remove(selected);
            selected_validators.push(validator);
        }
        Staking::<T>::nominate(n_controller.origin().into(), selected_validators)?;
    }

    ValidatorCount::<T>::put(validators);

    Ok(validator_chosen)
}
/// get the current era.
pub fn current_era<T: Config>() -> EraIndex {
    <Pallet<T>>::current_era().unwrap_or(0)
}
