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

//! Staking pallet benchmarking.

use super::*;
use crate::{ConfigOp, Pallet as Staking};
use testing_utils::*;

use codec::Decode;
use frame_election_provider_support::SortedListProvider;
use frame_support::{
    dispatch::UnfilteredDispatchable,
    pallet_prelude::*,
    traits::{Currency, Get, Imbalance},
};
use sp_runtime::{
    traits::{Bounded, One, StaticLookup, TrailingZeroInput, Zero},
    Perbill, Percent,
};
use sp_staking::SessionIndex;
use sp_std::prelude::*;

pub use frame_benchmarking::v1::{
    account, benchmarks, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_system::RawOrigin;

const SEED: u32 = 0;
const MAX_SPANS: u32 = 100;
const MAX_SLASHES: u32 = 1000;

type MaxValidators<T> = <<T as Config>::BenchmarkingConfig as BenchmarkingConfig>::MaxValidators;
type MaxNominators<T> = <<T as Config>::BenchmarkingConfig as BenchmarkingConfig>::MaxNominators;

// Polymesh change
// -----------------------------------------------------------------
use polymesh_common_utilities::benchs::{AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::{IdentityId, Permissions};

use crate::types::SlashingSwitch;
// -----------------------------------------------------------------

// Polymesh change
// -----------------------------------------------------------------
macro_rules! whitelist_account {
    ($acc:expr) => {
        let x = $acc.account();
        frame_benchmarking::v1::whitelist_account!(x);
    };
}

fn add_permissioned_validator_<T: Config>(id: IdentityId, intended_count: Option<u32>) {
    Staking::<T>::set_validator_count(RawOrigin::Root.into(), 10)
        .expect("Failed to set the validator count");
    Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), id, intended_count)
        .expect("Failed to add permissioned validator");
}

// -----------------------------------------------------------------

// Add slashing spans to a user account. Not relevant for actual use, only to benchmark
// read and write operations.
pub fn add_slashing_spans<T: Config>(who: &T::AccountId, spans: u32) {
    if spans == 0 {
        return;
    }

    // For the first slashing span, we initialize
    let mut slashing_spans = crate::slashing::SlashingSpans::new(0);
    SpanSlash::<T>::insert((who, 0), crate::slashing::SpanRecord::default());

    for i in 1..spans {
        assert!(slashing_spans.end_span(i));
        SpanSlash::<T>::insert((who, i), crate::slashing::SpanRecord::default());
    }
    SlashingSpans::<T>::insert(who, slashing_spans);
}

// This function clears all existing validators and nominators from the set, and generates one new
// validator being nominated by n nominators, and returns the validator stash account and the
// nominators' stash and controller. It also starts an era and creates pending payouts.
pub fn create_validator_with_nominators<T>(
    n: u32,
    upper_bound: u32,
    dead: bool,
    destination: RewardDestination<T::AccountId>,
    balance: Option<u32>,
) -> Result<(User<T>, Vec<(User<T>, User<T>)>), &'static str>
where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    // Clean up any existing state.
    clear_validators_and_nominators::<T>();

    let mut points_total = 0;
    let mut points_individual = Vec::new();

    let (v_stash, v_controller) =
        create_stash_controller::<T>(0, balance.unwrap_or_default(), destination.clone())?;
    let validator_prefs = ValidatorPrefs {
        commission: Perbill::from_percent(50),
        ..Default::default()
    };
    Staking::<T>::set_commission_cap(RawOrigin::Root.into(), Perbill::from_percent(60)).unwrap();
    Staking::<T>::set_validator_count(RawOrigin::Root.into(), 10)?;
    Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), v_stash.did(), Some(2))?;
    Staking::<T>::validate(v_controller.origin().into(), validator_prefs)?;
    let stash_lookup = v_stash.lookup();

    points_total += 10;
    points_individual.push((v_stash.account().clone(), 10));

    let original_nominator_count = Nominators::<T>::count();
    let mut nominators = Vec::new();

    // Give the validator n nominators, but keep total users in the system the same.
    for i in 0..upper_bound {
        let (n_stash, n_controller) = if !dead {
            create_stash_controller::<T>(u32::MAX - i, 10_000_000, destination.clone())?
        } else {
            create_stash_and_dead_controller::<T>(u32::MAX - i, 10_000_000, destination.clone())?
        };
        if i < n {
            Staking::<T>::nominate(
                RawOrigin::Signed(n_controller.account()).into(),
                vec![stash_lookup.clone()],
            )?;
            nominators.push((n_stash, n_controller));
        }
    }

    ValidatorCount::<T>::put(1);

    // Start a new Era
    let new_validators = Staking::<T>::try_trigger_new_era(SessionIndex::one(), true).unwrap();

    assert_eq!(new_validators.len(), 1);
    assert_eq!(
        new_validators[0],
        v_stash.account(),
        "Our validator was not selected!"
    );
    assert_ne!(Validators::<T>::count(), 0);
    assert_eq!(
        Nominators::<T>::count(),
        original_nominator_count + nominators.len() as u32
    );

    // Give Era Points
    let reward = EraRewardPoints::<T::AccountId> {
        total: points_total,
        individual: points_individual.into_iter().collect(),
    };

    let current_era = CurrentEra::<T>::get().unwrap();
    ErasRewardPoints::<T>::insert(current_era, reward);

    // Create reward pool
    let total_payout: BalanceOf<T> = 10_000u32.into();
    <ErasValidatorReward<T>>::insert(current_era, total_payout);

    Ok((v_stash, nominators))
}

const USER_SEED: u32 = 999666;

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    bond {
        let stash = create_funded_user::<T>("stash", USER_SEED, 100);
        let controller = create_funded_user::<T>("controller", USER_SEED, 100);
        let controller_lookup = controller.lookup();
        let reward_destination = RewardDestination::Staked;
        whitelist_account!(stash);
    }: _(stash.origin(), controller_lookup, 2_000_000u32.into(), reward_destination)
    verify {
        assert!(Bonded::<T>::contains_key(stash.account()));
        assert!(Ledger::<T>::contains_key(controller.account()));
    }

    bond_extra {
        // clean up any existing state.
        clear_validators_and_nominators::<T>();

        // Polymesh change
        // -----------------------------------------------------------------
        // UseNominatorsAndValidatorsMap does not provide nominators in sorted order
        // -----------------------------------------------------------------

        let (stash, controller) =
            create_stash_controller::<T>(USER_SEED, 100, RewardDestination::Staked).unwrap();
        let original_bonded =
            Ledger::<T>::get(controller.account()).map(|l| l.active).ok_or("ledger not created")?;

        whitelist_account!(stash);
    }: _(stash.origin(), 2_000_000u32.into())
    verify {
        let ledger = Ledger::<T>::get(controller.account()).ok_or("ledger not created after")?;
        let new_bonded: BalanceOf<T> = ledger.active;
        assert!(original_bonded < new_bonded);
    }

    unbond {
        // clean up any existing state.
        clear_validators_and_nominators::<T>();

        // Polymesh change
        // -----------------------------------------------------------------
        // UseNominatorsAndValidatorsMap does not provide nominators in sorted order
        // -----------------------------------------------------------------

        let (stash, controller) =
            create_stash_controller::<T>(USER_SEED, 100, RewardDestination::Staked).unwrap();
        let original_bonded =
            Ledger::<T>::get(controller.account()).map(|l| l.active).ok_or("ledger not created")?;

        whitelist_account!(controller);
    }: _(controller.origin(), 20u32.into())
    verify {
        let ledger = Ledger::<T>::get(controller.account()).ok_or("ledger not created after")?;
        let new_bonded: BalanceOf<T> = ledger.active;
        assert!(original_bonded > new_bonded);
    }

    // Withdraw only updates the ledger
    withdraw_unbonded_update {
        // Slashing Spans
        let s in 0 .. MAX_SPANS;

        // clean up any existing state.
        clear_validators_and_nominators::<T>();

        let (stash, controller) = create_stash_controller::<T>(0, 10_000_000, Default::default())?;
        add_slashing_spans::<T>(&stash.account(), s);
        Staking::<T>::unbond(controller.origin().into(), 50u32.into())?;
        CurrentEra::<T>::put(EraIndex::max_value());
        let original_total =
            Ledger::<T>::get(controller.account()).map(|l| l.total).ok_or("ledger not created before")?;

        whitelist_account!(controller);
    }: withdraw_unbonded(controller.origin(), s)
    verify {
        let ledger = Ledger::<T>::get(controller.account()).ok_or("ledger not created after")?;
        let new_total: BalanceOf<T> = ledger.total;
        assert!(original_total > new_total);
    }

    // Worst case scenario, everything is removed after the bonding duration
    withdraw_unbonded_kill {
        // Slashing Spans
        let s in 0 .. MAX_SPANS;
        // clean up any existing state.
        clear_validators_and_nominators::<T>();

        // Polymesh change
        // -----------------------------------------------------------------
        // UseNominatorsAndValidatorsMap does not provide nominators in sorted order
        // -----------------------------------------------------------------

        let (stash, controller) = create_stash_controller::<T>(0, 10_000_000, Default::default())?;
        add_slashing_spans::<T>(&stash.account(), s);
        Staking::<T>::unbond(controller.origin().into(), 10_000_000u32.into())?;
        CurrentEra::<T>::put(EraIndex::max_value());
        let _ = Ledger::<T>::get(&controller.account()).expect("ledger not created before");

        whitelist_account!(controller);
    }: withdraw_unbonded(controller.origin(), s)
    verify {
        assert!(!Ledger::<T>::contains_key(controller.account()));
        assert!(!T::VoterList::contains(&stash.account()));
    }

    validate {
        clear_validators_and_nominators::<T>();

        let (stash, controller) = create_stash_controller::<T>(
            T::MaxNominations::get() - 1,
            100,
            Default::default(),
        )?;
        // because it is chilled.
        assert!(!T::VoterList::contains(&stash.account()));

        // Polymesh change
        // -----------------------------------------------------------------
        add_permissioned_validator_::<T>(stash.did(), Some(2));
        // -----------------------------------------------------------------

        whitelist_account!(controller);
    }: _(controller.origin(), ValidatorPrefs::default())
    verify {
        assert!(Validators::<T>::contains_key(stash.account()));
        assert!(T::VoterList::contains(&stash.account()));
    }

//    kick {
//        // scenario: we want to kick `k` nominators from nominating us (we are a validator).
//        // we'll assume that `k` is under 128 for the purposes of determining the slope.
//        // each nominator should have `T::MaxNominations::get()` validators nominated, and our validator
//        // should be somewhere in there.
//        let k in 1 .. 128;
//
//        // these are the other validators; there are `T::MaxNominations::get() - 1` of them, so
//        // there are a total of `T::MaxNominations::get()` validators in the system.
//        let rest_of_validators = create_validators_with_seed::<T>(T::MaxNominations::get() - 1, 100, 415)?;
//
//        // this is the validator that will be kicking.
//        let (stash, controller) = create_stash_controller::<T>(
//            T::MaxNominations::get() - 1,
//            100,
//            Default::default(),
//        )?;
//        let stash_lookup = T::Lookup::unlookup(stash.clone());
//
//        // they start validating.
//        Staking::<T>::validate(RawOrigin::Signed(controller.clone()).into(), Default::default())?;
//
//        // we now create the nominators. there will be `k` of them; each will nominate all
//        // validators. we will then kick each of the `k` nominators from the main validator.
//        let mut nominator_stashes = Vec::with_capacity(k as usize);
//        for i in 0 .. k {
//            // create a nominator stash.
//            let (n_stash, n_controller) = create_stash_controller::<T>(
//                T::MaxNominations::get() + i,
//                100,
//                Default::default(),
//            )?;
//
//            // bake the nominations; we first clone them from the rest of the validators.
//            let mut nominations = rest_of_validators.clone();
//            // then insert "our" validator somewhere in there (we vary it) to avoid accidental
//            // optimisations/pessimisations.
//            nominations.insert(i as usize % (nominations.len() + 1), stash_lookup.clone());
//            // then we nominate.
//            Staking::<T>::nominate(RawOrigin::Signed(n_controller.clone()).into(), nominations)?;
//
//            nominator_stashes.push(n_stash);
//        }
//
//        // all nominators now should be nominating our validator...
//        for n in nominator_stashes.iter() {
//            assert!(Nominators::<T>::get(n).unwrap().targets.contains(&stash));
//        }
//
//        // we need the unlookuped version of the nominator stash for the kick.
//        let kicks = nominator_stashes.iter()
//            .map(|n| T::Lookup::unlookup(n.clone()))
//            .collect::<Vec<_>>();
//
//        whitelist_account!(controller);
//    }: _(RawOrigin::Signed(controller), kicks)
//    verify {
//        // all nominators now should *not* be nominating our validator...
//        for n in nominator_stashes.iter() {
//            assert!(!Nominators::<T>::get(n).unwrap().targets.contains(&stash));
//        }
//    }
//
    // Worst case scenario, T::MaxNominations::get()
    nominate {
        let n in 1 .. T::MaxNominations::get();

        // clean up any existing state.
        clear_validators_and_nominators::<T>();

        let (stash, controller) = create_stash_controller::<T>(
            n + 1,
            10_000_000,
            Default::default(),
        )?;
        assert!(!Nominators::<T>::contains_key(&stash.account()));
        assert!(!T::VoterList::contains(&stash.account()));

        let validators = create_validators::<T>(n, 100).unwrap();

        whitelist_account!(controller);
    }: _(controller.origin(), validators)
    verify {
        assert!(Nominators::<T>::contains_key(&stash.account()));
        assert!(T::VoterList::contains(&stash.account()))
    }

    chill {
        // clean up any existing state.
        clear_validators_and_nominators::<T>();

        // Polymesh change
        // -----------------------------------------------------------------
        let (stash, controller) = create_stash_controller::<T>(0, 10_000_000, Default::default())?;
        Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), stash.did(), Some(2))?;
        Staking::<T>::validate(controller.origin().into(), ValidatorPrefs::default())?;
        assert!(T::VoterList::contains(&stash.account()));
        assert!(Validators::<T>::contains_key(&stash.account()));
        // -----------------------------------------------------------------

        whitelist_account!(controller);
    }: _(controller.origin())
    verify {
        assert!(!T::VoterList::contains(&stash.account()));
        assert!(!Validators::<T>::contains_key(&stash.account()));
    }

    set_payee {
        let (stash, controller) = create_stash_controller::<T>(USER_SEED, 100, Default::default())?;
        assert_eq!(Payee::<T>::get(&stash.account()), RewardDestination::Staked);
        whitelist_account!(controller);
    }: _(controller.origin(), RewardDestination::Controller)
    verify {
        assert_eq!(Payee::<T>::get(&stash.account()), RewardDestination::Controller);
    }

    set_controller {
        let (stash, _) = create_stash_controller::<T>(USER_SEED, 100, Default::default())?;
        let new_controller = create_funded_user::<T>("new_controller", USER_SEED, 100);
        whitelist_account!(stash);
    }: _(stash.origin(), new_controller.lookup())
    verify {
        assert!(Ledger::<T>::contains_key(&new_controller.account()));
    }

    set_validator_count {
        let validator_count = MaxValidators::<T>::get();
    }: _(RawOrigin::Root, validator_count)
    verify {
        assert_eq!(ValidatorCount::<T>::get(), validator_count);
    }

    force_no_eras {}: _(RawOrigin::Root)
    verify { assert_eq!(ForceEra::<T>::get(), Forcing::ForceNone); }

    force_new_era {}: _(RawOrigin::Root)
    verify { assert_eq!(ForceEra::<T>::get(), Forcing::ForceNew); }

    force_new_era_always {}: _(RawOrigin::Root)
    verify { assert_eq!(ForceEra::<T>::get(), Forcing::ForceAlways); }

    // Worst case scenario, the list of invulnerables is very long.
    set_invulnerables {
        let v in 0 .. MaxValidators::<T>::get();
        let mut invulnerables = Vec::new();
        for i in 0 .. v {
            invulnerables.push(account("invulnerable", i, SEED));
        }
    }: _(RawOrigin::Root, invulnerables)
    verify {
        assert_eq!(Invulnerables::<T>::get().len(), v as usize);
    }

    force_unstake {
        // Slashing Spans
        let s in 0 .. MAX_SPANS;
        // Clean up any existing state.
        clear_validators_and_nominators::<T>();

        // Polymesh change
        // -----------------------------------------------------------------
        let (stash, controller) = create_stash_controller::<T>(USER_SEED, 10_000_000, Default::default())?;
        Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), stash.did(), Some(2))?;
        Staking::<T>::validate(controller.origin().into(), ValidatorPrefs::default())?;
        add_slashing_spans::<T>(&stash.account(), s);
        assert!(T::VoterList::contains(&stash.account()));
        assert!(Validators::<T>::contains_key(&stash.account()));
        // -----------------------------------------------------------------

    }: _(RawOrigin::Root, stash.account(), s)
    verify {
        assert!(!Ledger::<T>::contains_key(&controller.account()));
        assert!(!T::VoterList::contains(&stash.account()));
        assert!(!Validators::<T>::contains_key(&stash.account()));
    }

    cancel_deferred_slash {
        let s in 1 .. MAX_SLASHES;
        let mut unapplied_slashes = Vec::new();
        let era = EraIndex::one();
        let dummy = || T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
        for _ in 0 .. MAX_SLASHES {
            unapplied_slashes.push(UnappliedSlash::<T::AccountId, BalanceOf<T>>::default_from(dummy()));
        }
        UnappliedSlashes::<T>::insert(era, &unapplied_slashes);

        let slash_indices: Vec<u32> = (0 .. s).collect();
    }: _(RawOrigin::Root, era, slash_indices)
    verify {
        assert_eq!(UnappliedSlashes::<T>::get(&era).len(), (MAX_SLASHES - s) as usize);
    }

    payout_stakers_dead_controller {
        let n in 0 .. T::MaxNominatorRewardedPerValidator::get() as u32;
        let (validator, nominators) = create_validator_with_nominators::<T>(
            n,
            T::MaxNominatorRewardedPerValidator::get() as u32,
            true,
            RewardDestination::Controller,
            Some(10_000_000)
        )?;

        let current_era = CurrentEra::<T>::get().unwrap();
        // set the commission for this particular era as well.
        <ErasValidatorPrefs<T>>::insert(current_era, validator.account().clone(), <Staking<T>>::validators(&validator.account()));

        let validator_controller = <Bonded<T>>::get(&validator.account()).unwrap();
        let balance_before = T::Currency::free_balance(&validator_controller);
        for (_, controller) in &nominators {
            let balance = T::Currency::free_balance(&controller.account());
            ensure!(balance.is_zero(), "Controller has balance, but should be dead.");
        }

        let caller = UserBuilder::<T>::default().seed(SEED).generate_did().build("caller");
        let caller_key = frame_system::Account::<T>::hashed_key_for(&caller.account());
        frame_benchmarking::benchmarking::add_to_whitelist(caller_key.into());
    }: payout_stakers(caller.origin(), validator.account(), current_era)
    verify {
        let balance_after = T::Currency::free_balance(&validator_controller);
        ensure!(
            balance_before < balance_after,
            "Balance of validator controller should have increased after payout.",
        );
        for (_, controller) in &nominators {
            let balance = T::Currency::free_balance(&controller.account());
            ensure!(!balance.is_zero(), "Payout not given to controller.");
        }
    }

    payout_stakers_alive_staked {
        let n in 0 .. T::MaxNominatorRewardedPerValidator::get() as u32;

        let (validator, nominators) = create_validator_with_nominators::<T>(
            n,
            T::MaxNominatorRewardedPerValidator::get() as u32,
            false,
            RewardDestination::Staked,
            Some(10_000_000)
        )?;

        let current_era = CurrentEra::<T>::get().unwrap();
        // set the commission for this particular era as well.
        <ErasValidatorPrefs<T>>::insert(
            current_era,
            validator.account().clone(),
            <Staking<T>>::validators(&validator.account())
        );

        let caller = UserBuilder::<T>::default().seed(SEED).generate_did().build("caller");
        let caller_key = frame_system::Account::<T>::hashed_key_for(&caller.account());
        frame_benchmarking::benchmarking::add_to_whitelist(caller_key.into());

        let balance_before = T::Currency::free_balance(&validator.account());
        let mut nominator_balances_before = Vec::new();
        for (stash, _) in &nominators {
            let balance = T::Currency::free_balance(&stash.account());
            nominator_balances_before.push(balance);
        }
    }: payout_stakers(caller.origin(), validator.account().clone(), current_era)
    verify {
        let balance_after = T::Currency::free_balance(&validator.account());
        ensure!(
            balance_before < balance_after,
            "Balance of validator stash should have increased after payout.",
        );
        for ((stash, _), balance_before) in nominators.iter().zip(nominator_balances_before.iter()) {
            let balance_after = T::Currency::free_balance(&stash.account());
            ensure!(
                balance_before < &balance_after,
                "Balance of nominator stash should have increased after payout.",
            );
        }
    }

    rebond {
        let l in 1 .. T::MaxUnlockingChunks::get() as u32;

        // clean up any existing state.
        clear_validators_and_nominators::<T>();

        let (stash, controller) =
            create_stash_controller::<T>(1, 10_000_000, RewardDestination::Staked)?;

        let mut staking_ledger = Ledger::<T>::get(controller.account()).unwrap();
        let unlock_chunk = UnlockChunk::<BalanceOf<T>> {
            value: 1u32.into(),
            era: EraIndex::zero(),
        };
        for _ in 0 .. l {
            staking_ledger.unlocking.try_push(unlock_chunk.clone()).unwrap()
        }
        Ledger::<T>::insert(controller.account(), staking_ledger.clone());
        let original_bonded: BalanceOf<T> = staking_ledger.active;

        whitelist_account!(controller);
    }: _(controller.origin(), 101u32.into())
    verify {
        let ledger = Ledger::<T>::get(&controller.account()).ok_or("ledger not created after")?;
        let new_bonded: BalanceOf<T> = ledger.active;
        assert!(original_bonded < new_bonded);
    }

    reap_stash {
        let s in 1 .. MAX_SPANS;
        // clean up any existing state.
        clear_validators_and_nominators::<T>();

        let (stash, controller) =
            create_stash_controller::<T>(1, 1, RewardDestination::Staked)?;
        Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), stash.did(), Some(2))?;
        Staking::<T>::validate(controller.origin().into(), ValidatorPrefs::default())?;

        add_slashing_spans::<T>(&stash.account(), s);
        let l = StakingLedger {
            stash: stash.account().clone(),
            active: T::Currency::minimum_balance(),
            total: T::Currency::minimum_balance(),
            unlocking: Default::default(),
            claimed_rewards: Default::default(),
        };
        T::Currency::make_free_balance_be(&stash.account(), 0u32.into());
        Ledger::<T>::insert(&controller.account(), l);

        assert!(Bonded::<T>::contains_key(&stash.account()));
        assert!(T::VoterList::contains(&stash.account()));

        whitelist_account!(controller);
    }: _(controller.origin(), stash.account(), s)
    verify {
        assert!(!Bonded::<T>::contains_key(&stash.account()));
        assert!(!T::VoterList::contains(&stash.account()));
    }

    new_era {
        let v in 1 .. 10;
        let n in 0 .. 100;

        create_validators_with_nominators_for_era::<T>(
            v,
            n,
            <T as Config>::MaxNominations::get() as usize,
            false,
            None,
        )?;
        let session_index = SessionIndex::one();
    }: {
        let validators = Staking::<T>::try_trigger_new_era(session_index, true)
            .ok_or("`new_era` failed")?;
        assert!(validators.len() == v as usize);
    }

    #[extra]
    payout_all {
        let v in 1 .. 10;
        let n in 0 .. 100;
        create_validators_with_nominators_for_era::<T>(
            v,
            n,
            <T as Config>::MaxNominations::get() as usize,
            false,
            None,
        )?;
        // Start a new Era
        let new_validators = Staking::<T>::try_trigger_new_era(SessionIndex::one(), true).unwrap();
        assert!(new_validators.len() == v as usize);

        let current_era = CurrentEra::<T>::get().unwrap();
        let mut points_total = 0;
        let mut points_individual = Vec::new();
        let mut payout_calls_arg = Vec::new();

        for validator in new_validators.iter() {
            points_total += 10;
            points_individual.push((validator.clone(), 10));
            payout_calls_arg.push((validator.clone(), current_era));
        }

        // Give Era Points
        let reward = EraRewardPoints::<T::AccountId> {
            total: points_total,
            individual: points_individual.into_iter().collect(),
        };

        ErasRewardPoints::<T>::insert(current_era, reward);

        // Create reward pool
        let total_payout = T::Currency::minimum_balance() * 1000u32.into();
        <ErasValidatorReward<T>>::insert(current_era, total_payout);

        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller);
        let calls: Vec<_> = payout_calls_arg.iter().map(|arg|
            Call::<T>::payout_stakers { validator_stash: arg.0.clone(), era: arg.1 }.encode()
        ).collect();
    }: {
        for call in calls {
            <Call<T> as Decode>::decode(&mut &*call)
                .expect("call is encoded above, encoding must be correct")
                .dispatch_bypass_filter(origin.clone().into())?;
        }
    }

    #[extra]
    do_slash {
        let l in 1 .. T::MaxUnlockingChunks::get() as u32;
        let (stash, controller) = create_stash_controller::<T>(0, 100, Default::default())?;
        let mut staking_ledger = Ledger::<T>::get(controller.account()).unwrap();
        let unlock_chunk = UnlockChunk::<BalanceOf<T>> {
            value: 1u32.into(),
            era: EraIndex::zero(),
        };
        for _ in 0 .. l {
            staking_ledger.unlocking.try_push(unlock_chunk.clone()).unwrap();
        }
        Ledger::<T>::insert(controller.account(), staking_ledger);
        let balance_before = T::Currency::free_balance(&stash.account());
    }: {
        crate::slashing::do_slash::<T>(
            &stash.account(),
            10u32.into(),
            &mut BalanceOf::<T>::zero(),
            &mut NegativeImbalanceOf::<T>::zero(),
            EraIndex::zero()
        );
    } verify {
        let balance_after = T::Currency::free_balance(&stash.account());
        assert!(balance_before > balance_after);
    }

    get_npos_voters {
        // number of validator intention. we will iterate all of them.
        let v in (MaxValidators::<T>::get() / 2) .. MaxValidators::<T>::get();
        // number of nominator intention. we will iterate all of them.
        let n in (MaxNominators::<T>::get() / 2) .. MaxNominators::<T>::get();

        let validators = create_validators_with_nominators_for_era::<T>(
            v, n, T::MaxNominations::get() as usize, false, None
        )?
        .into_iter()
        .map(|v| T::Lookup::lookup(v).unwrap())
        .collect::<Vec<_>>();

        assert_eq!(Validators::<T>::count(), v);
        assert_eq!(Nominators::<T>::count(), n);

        let num_voters = (v + n) as usize;
    }: {
        let voters = <Staking<T>>::get_npos_voters(None);
        assert_eq!(voters.len(), num_voters);
    }

    get_npos_targets {
        // number of validator intention.
        let v in (MaxValidators::<T>::get() / 2) .. MaxValidators::<T>::get();
        // number of nominator intention.
        let n = MaxNominators::<T>::get();

        let _ = create_validators_with_nominators_for_era::<T>(
            v, n, T::MaxNominations::get() as usize, false, None
        )?;
    }: {
        let targets = <Staking<T>>::get_npos_targets(None);
        assert_eq!(targets.len() as u32, v);
    }

    set_staking_configs_all_set {
    }: set_staking_configs(
        RawOrigin::Root,
        ConfigOp::Set(BalanceOf::<T>::max_value()),
        ConfigOp::Set(BalanceOf::<T>::max_value()),
        ConfigOp::Set(u32::MAX),
        ConfigOp::Set(u32::MAX),
        ConfigOp::Set(Percent::max_value()),
        ConfigOp::Set(Perbill::max_value())
    ) verify {
        assert_eq!(MinNominatorBond::<T>::get(), BalanceOf::<T>::max_value());
        assert_eq!(MinValidatorBond::<T>::get(), BalanceOf::<T>::max_value());
        assert_eq!(MaxNominatorsCount::<T>::get(), Some(u32::MAX));
        assert_eq!(MaxValidatorsCount::<T>::get(), Some(u32::MAX));
        assert_eq!(ChillThreshold::<T>::get(), Some(Percent::from_percent(100)));
        assert_eq!(MinCommission::<T>::get(), Perbill::from_percent(100));
    }

    set_staking_configs_all_remove {
    }: set_staking_configs(
        RawOrigin::Root,
        ConfigOp::Remove,
        ConfigOp::Remove,
        ConfigOp::Remove,
        ConfigOp::Remove,
        ConfigOp::Remove,
        ConfigOp::Remove
    ) verify {
        assert!(!MinNominatorBond::<T>::exists());
        assert!(!MinValidatorBond::<T>::exists());
        assert!(!MaxNominatorsCount::<T>::exists());
        assert!(!MaxValidatorsCount::<T>::exists());
        assert!(!ChillThreshold::<T>::exists());
        assert!(!MinCommission::<T>::exists());
    }

    chill_other {
        // clean up any existing state.
        clear_validators_and_nominators::<T>();

        // Create a validator with a commission of 50%
        let (stash, controller) =
            create_stash_controller::<T>(1, 1, RewardDestination::Staked)?;
        let validator_prefs =
            ValidatorPrefs { commission: Perbill::from_percent(50), ..Default::default() };
        Staking::<T>::set_commission_cap(RawOrigin::Root.into(), Perbill::from_percent(60)).unwrap();
        Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), stash.did(), Some(2))?;
        Staking::<T>::validate(controller.origin().into(), validator_prefs)?;
        assert!(T::VoterList::contains(&stash.account()));

        Staking::<T>::set_staking_configs(
            RawOrigin::Root.into(),
            ConfigOp::Set(BalanceOf::<T>::max_value()),
            ConfigOp::Set(BalanceOf::<T>::max_value()),
            ConfigOp::Set(0),
            ConfigOp::Set(0),
            ConfigOp::Set(Percent::from_percent(0)),
            ConfigOp::Set(Zero::zero()),
        )?;

        let caller = UserBuilder::<T>::default().seed(SEED).generate_did().build("caller");
    }: _(caller.origin(), controller.account())
    verify {
        assert!(!T::VoterList::contains(&stash.account()));
    }

    force_apply_min_commission {
        // Clean up any existing state
        clear_validators_and_nominators::<T>();

        // Create a validator with a commission of 50%
        let (stash, controller) =
            create_stash_controller::<T>(1, 1, RewardDestination::Staked)?;
        let validator_prefs =
            ValidatorPrefs { commission: Perbill::from_percent(50), ..Default::default() };
        Staking::<T>::set_commission_cap(RawOrigin::Root.into(), Perbill::from_percent(60)).unwrap();
        Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), stash.did(), Some(2))?;
        Staking::<T>::validate(controller.origin().into(), validator_prefs)?;

        // Sanity check
        assert_eq!(
            Validators::<T>::get(&stash.account()),
            ValidatorPrefs { commission: Perbill::from_percent(50), ..Default::default() }
        );

        // Set the min commission to 75%
        MinCommission::<T>::set(Perbill::from_percent(75));
    }: _(stash.origin(), stash.account().clone())
    verify {
        // The validators commission has been bumped to 75%
        assert_eq!(
            Validators::<T>::get(&stash.account()),
            ValidatorPrefs { commission: Perbill::from_percent(75), ..Default::default() }
        );
    }

    set_min_commission {
        let min_commission = Perbill::max_value();
    }: _(RawOrigin::Root, min_commission)
    verify {
        assert_eq!(MinCommission::<T>::get(), Perbill::from_percent(100));
    }

    // Polymesh change
    // -----------------------------------------------------------------
    add_permissioned_validator {
        clear_validators_and_nominators::<T>();
        let (stash, controller) =
            create_stash_controller::<T>(1, 1, RewardDestination::Staked)?;
        Staking::<T>::set_validator_count(RawOrigin::Root.into(), 10).unwrap();
    }: _(RawOrigin::Root, stash.did(), Some(1))
    verify {
        let identity_preferences = Staking::<T>::permissioned_identity(stash.did());
        assert!(identity_preferences.is_some());
        assert_eq!(identity_preferences.unwrap().intended_count, 1);
    }

    remove_permissioned_validator {
        clear_validators_and_nominators::<T>();
        let (stash, controller) =
            create_stash_controller::<T>(1, 1, RewardDestination::Staked)?;
        add_permissioned_validator_::<T>(stash.did(), Some(1));
    }: _(RawOrigin::Root, stash.did())
    verify {
        let identity_preferences = Staking::<T>::permissioned_identity(stash.did());
        assert!(identity_preferences.is_none());
    }

    change_slashing_allowed_for {}: _(RawOrigin::Root, SlashingSwitch::ValidatorAndNominator)
    verify {
        assert_eq!(Staking::<T>::slashing_allowed_for(), SlashingSwitch::ValidatorAndNominator);
    }

    update_permissioned_validator_intended_count {
        clear_validators_and_nominators::<T>();
        let (stash, controller) =
            create_stash_controller::<T>(1, 1, RewardDestination::Staked)?;
        add_permissioned_validator_::<T>(stash.did(), Some(1));
    }: _(RawOrigin::Root, stash.did(), 2)
    verify {
        assert_eq!(Staking::<T>::permissioned_identity(stash.did()).unwrap().intended_count, 2);
    }

    chill_from_governance {
        let s in 1..100;

        let validator = create_funded_user::<T>("validator", USER_SEED, 10_000);

        Staking::<T>::set_validator_count(RawOrigin::Root.into(), 1_000).unwrap();
        assert_eq!(Staking::<T>::validator_count(), 1_000);

        Staking::<T>::add_permissioned_validator(
            RawOrigin::Root.into(),
            validator.did(),
            Some(100)
        ).unwrap();

        whitelist_account!(validator);

        let mut signatories = Vec::new();
        for x in 0 .. s {
            let key = UserBuilder::<T>::default().seed(x).balance(10_000u32).build("key");
            let _ = T::Currency::issue(10_000u32.into());

            pallet_identity::Module::<T>::unsafe_join_identity(
                validator.did(),
                Permissions::default(),
                key.account.clone()
            );
            Staking::<T>::bond(
                key.origin().into(),
                key.lookup(),
                2_000_000u32.into(),
                RewardDestination::Staked
            )
            .unwrap();
            whitelist_account!(key);

            Staking::<T>::validate(key.origin().into(), ValidatorPrefs::default()).unwrap();
            assert_eq!(<Validators<T>>::contains_key(&key.account), true);
            signatories.push(key.account.clone());
        }
    }: _(RawOrigin::Root, validator.did(), signatories.clone())
    verify {
        for key in signatories {
            assert!(!<Validators<T>>::contains_key(&key));
        }
    }

    set_commission_cap {
        let m in 0 .. 150;

        let mut stashes = Vec::with_capacity(m as usize);
        for i in 0 .. m {
            let stash = create_funded_user::<T>("stash", i, 1000);
            stashes.push(stash.account());
            Validators::<T>::insert(
                stash.account(),
                ValidatorPrefs {
                    commission: Perbill::from_percent(70),
                    ..Default::default()
                }
            );
        }
    }: _(RawOrigin::Root, Perbill::from_percent(50))
    verify {
        stashes.iter().for_each(|s| {
            assert_eq!(
                Staking::<T>::validators(s),
                ValidatorPrefs { commission: Perbill::from_percent(50), ..Default::default() }
            );
        });
    }
    // -----------------------------------------------------------------
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{Balances, ExtBuilder, RuntimeOrigin, Staking, Test};
    use frame_support::assert_ok;

    #[test]
    fn create_validators_with_nominators_for_era_works() {
        ExtBuilder::default().build_and_execute(|| {
            let v = 10;
            let n = 100;

            create_validators_with_nominators_for_era::<Test>(
                v,
                n,
                <Test as Config>::MaxNominations::get() as usize,
                false,
                None,
            )
            .unwrap();

            let count_validators = Validators::<Test>::iter().count();
            let count_nominators = Nominators::<Test>::iter().count();

            assert_eq!(count_validators, Validators::<Test>::count() as usize);
            assert_eq!(count_nominators, Nominators::<Test>::count() as usize);

            assert_eq!(count_validators, v as usize);
            assert_eq!(count_nominators, n as usize);
        });
    }

    #[test]
    fn create_validator_with_nominators_works() {
        ExtBuilder::default().build_and_execute(|| {
            let n = 10;

            let (validator_stash, nominators) = create_validator_with_nominators::<Test>(
                n,
                <<Test as Config>::MaxNominatorRewardedPerValidator as Get<_>>::get(),
                false,
                RewardDestination::Staked,
            )
            .unwrap();

            assert_eq!(nominators.len() as u32, n);

            let current_era = CurrentEra::<Test>::get().unwrap();

            let original_free_balance = Balances::free_balance(&validator_stash);
            assert_ok!(Staking::payout_stakers(
                RuntimeOrigin::signed(1337),
                validator_stash,
                current_era
            ));
            let new_free_balance = Balances::free_balance(&validator_stash);

            assert!(original_free_balance < new_free_balance);
        });
    }

    #[test]
    fn add_slashing_spans_works() {
        ExtBuilder::default().build_and_execute(|| {
            let n = 10;

            let (validator_stash, _nominators) = create_validator_with_nominators::<Test>(
                n,
                <<Test as Config>::MaxNominatorRewardedPerValidator as Get<_>>::get(),
                false,
                RewardDestination::Staked,
            )
            .unwrap();

            // Add 20 slashing spans
            let num_of_slashing_spans = 20;
            add_slashing_spans::<Test>(&validator_stash, num_of_slashing_spans);

            let slashing_spans = SlashingSpans::<Test>::get(&validator_stash).unwrap();
            assert_eq!(
                slashing_spans.iter().count(),
                num_of_slashing_spans as usize
            );
            for i in 0..num_of_slashing_spans {
                assert!(SpanSlash::<Test>::contains_key((&validator_stash, i)));
            }

            // Test everything is cleaned up
            assert_ok!(Staking::kill_stash(&validator_stash, num_of_slashing_spans));
            assert!(SlashingSpans::<Test>::get(&validator_stash).is_none());
            for i in 0..num_of_slashing_spans {
                assert!(!SpanSlash::<Test>::contains_key((&validator_stash, i)));
            }
        });
    }

    #[test]
    fn test_payout_all() {
        ExtBuilder::default().build_and_execute(|| {
            let v = 10;
            let n = 100;

            let selected_benchmark = SelectedBenchmark::payout_all;
            let c = vec![
                (frame_benchmarking::BenchmarkParameter::v, v),
                (frame_benchmarking::BenchmarkParameter::n, n),
            ];
            let closure_to_benchmark =
                <SelectedBenchmark as frame_benchmarking::BenchmarkingSetup<Test>>::instance(
                    &selected_benchmark,
                    &c,
                    true,
                )
                .unwrap();

            assert_ok!(closure_to_benchmark());
        });
    }
}
