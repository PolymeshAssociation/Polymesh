// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
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
use crate::Module as Staking;
use testing_utils::*;

pub use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_system::RawOrigin;
use polymesh_common_utilities::{
    benchs::{AccountIdOf, UserBuilder},
    TestUtilsFn,
};
use sp_runtime::traits::One;
use polymesh_primitives::{
    Permissions,
};
const SEED: u32 = 0;
const MAX_SPANS: u32 = 100;
const MAX_VALIDATORS: u32 = 1000;
const MAX_SLASHES: u32 = 1000;
const INIT_BALANCE: u32 = 10_000_000;
const MAX_STASHES: u32 = 100;

macro_rules! whitelist_account {
    ($acc:expr) => {
        let x = $acc.account();
        frame_benchmarking::whitelist_account!(x);
    };
}

// Add slashing spans to a user account. Not relevant for actual use, only to benchmark
// read and write operations.
fn add_slashing_spans<T: Config>(who: &T::AccountId, spans: u32) {
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

fn add_perm_validator<T: Config>(id: IdentityId, intended_count: Option<u32>) {
    Staking::<T>::set_validator_count(RawOrigin::Root.into(), 10)
        .expect("Failed to set the validator count");
    Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), id, intended_count)
        .expect("Failed to add permissioned validator");
}

use polymesh_common_utilities::benchs::User;

// This function clears all existing validators and nominators from the set, and generates one new
// validator being nominated by n nominators, and returns the validator stash account and the
// nominators' stash and controller. It also starts an era and creates pending payouts.
pub fn create_validator_with_nominators<
    T: Config + TestUtilsFn<<T as frame_system::Config>::AccountId>,
>(
    n: u32,
    upper_bound: u32,
    dead: bool,
) -> Result<(User<T>, Vec<(User<T>, User<T>)>), DispatchError> {
    create_validator_with_nominators_with_balance::<T>(n, upper_bound, INIT_BALANCE, dead)
}

// This function generates one validator being nominated by n nominators, and returns the validator
// stash account. It also starts an era and creates pending payouts.
// The balance is added to controller and stash accounts.
pub fn create_validator_with_nominators_with_balance<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    n: u32,
    upper_bound: u32,
    balance: u32,
    dead: bool,
) -> Result<(User<T>, Vec<(User<T>, User<T>)>), DispatchError> {
    clear_validators_and_nominators::<T>();
    let mut points_total = 0;
    let mut points_individual = Vec::new();

    let (v_stash, v_controller) = create_stash_controller_with_balance::<T>(0, balance).unwrap();
    let v_controller_origin = v_controller.origin();

    let validator_prefs = ValidatorPrefs {
        commission: Perbill::from_percent(50),
        ..Default::default()
    };
    emulate_validator_setup::<T>(1, 10, Perbill::from_percent(60));
    Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), v_stash.did(), Some(2))
        .expect("Failed to add permissioned validator");
    Staking::<T>::validate(v_controller_origin.into(), validator_prefs.clone())
        .expect("Failed to validate");
    assert_eq!(
        Staking::<T>::validators(v_stash.account()),
        validator_prefs,
        "Failed to set the validator"
    );
    let stash_lookup = v_stash.lookup();

    points_total += 10;
    points_individual.push((v_stash.account(), 10));

    let mut nominators = Vec::new();

    // Give the validator n nominators, but keep total users in the system the same.
    for i in 0..upper_bound {
        let (n_stash, n_controller) = if !dead {
            create_stash_controller_with_balance::<T>(u32::max_value() - i, INIT_BALANCE)?
        } else {
            create_stash_with_dead_controller::<T>(u32::max_value() - i, INIT_BALANCE)?
        };
        if i < n {
            Staking::<T>::nominate(
                RawOrigin::Signed(n_controller.account()).into(),
                vec![stash_lookup.clone()],
            )?;
            nominators.push((n_stash, n_controller));
        }
    }

    ValidatorCount::put(1);

    // Start a new Era
    let new_validators = Staking::<T>::new_era(SessionIndex::one()).unwrap();
    assert!(new_validators.len() == 1);
    assert!(
        new_validators[0] == v_stash.account(),
        "Our validator was not selected!",
    );

    // Give Era Points
    let reward = EraRewardPoints::<T::AccountId> {
        total: points_total,
        individual: points_individual.into_iter().collect(),
    };

    let current_era = CurrentEra::get().unwrap();
    ErasRewardPoints::<T>::insert(current_era, reward);

    // Create reward pool
    let total_payout: BalanceOf<T> = 10000u32.into();
    <ErasValidatorReward<T>>::insert(current_era, total_payout);

    Ok((v_stash, nominators))
}

fn payout_stakers_<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    alive: bool,
    n: u32,
) -> Result<(RawOrigin<T::AccountId>, T::AccountId, u32, BalanceOf<T>), DispatchError> {
    let validator = create_validator_with_nominators::<T>(
        n,
        T::MaxNominatorRewardedPerValidator::get() as u32,
        !alive,
    )
    .unwrap()
    .0
    .account();
    let current_era = CurrentEra::get().unwrap();
    <ErasValidatorPrefs<T>>::insert(
        current_era,
        validator.clone(),
        <Staking<T>>::validators(&validator),
    );
    let caller = UserBuilder::<T>::default()
        .seed(n)
        .generate_did()
        .build("caller");
    let caller_key = frame_system::Account::<T>::hashed_key_for(&caller.account());
    frame_benchmarking::benchmarking::add_to_whitelist(caller_key.into());
    let balance_before = T::Currency::free_balance(&validator);
    Ok((caller.origin(), validator, current_era, balance_before))
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    bond {
        let stash = create_funded_user::<T>("stash", 2, INIT_BALANCE);
        let controller = create_funded_user::<T>("controller", 5, 100);
        let controller_lookup = controller.lookup();
        let reward_destination = RewardDestination::Staked;
        let amount = 2_000_000u32.into();
        whitelist_account!(stash);
    }: _(stash.origin(), controller_lookup, amount, reward_destination)
    verify {
        assert!(Bonded::<T>::contains_key(stash.account()));
        assert!(Ledger::<T>::contains_key(controller.account()));
    }

    bond_extra {
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(5, INIT_BALANCE).unwrap();
        let max_additional = 2_000_000u32;
        let ledger = Ledger::<T>::get(&controller.account()).expect("ledger not created before");
        let original_bonded: BalanceOf<T> = ledger.active;
        whitelist_account!(stash);
    }: _(stash.origin(), max_additional.into())
    verify {
        let ledger = Ledger::<T>::get(&controller.account()).expect("ledger not created after");
        let new_bonded: BalanceOf<T> = ledger.active;
        assert!(original_bonded < new_bonded);
    }

    unbond {
        clear_validators_and_nominators::<T>();
        let (_, controller) = create_stash_controller::<T>(500, INIT_BALANCE).unwrap();
        let amount = 20u32;
        let ledger = Ledger::<T>::get(&controller.account()).expect("ledger not created before");
        let original_bonded: BalanceOf<T> = ledger.active;
        whitelist_account!(controller);
    }: _(controller.origin(), amount.into())
    verify {
        let ledger = Ledger::<T>::get(&controller.account()).expect("ledger not created after");
        let new_bonded: BalanceOf<T> = ledger.active;
        assert!(original_bonded > new_bonded);
    }

    // Withdraw only updates the ledger
    withdraw_unbonded_update {
        // Slashing Spans
        let s in 0 .. MAX_SPANS;
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(0, INIT_BALANCE).unwrap();
        add_slashing_spans::<T>(&stash.account(), s);
        let amount = 50u32; // Half of total
        Staking::<T>::unbond(controller.origin().into(), amount.into()).unwrap();
        CurrentEra::put(EraIndex::max_value());
        let ledger = Ledger::<T>::get(&controller.account()).expect("ledger not created before");
        let original_total: BalanceOf<T> = ledger.total;
        whitelist_account!(controller);
    }: withdraw_unbonded(controller.origin(), s)
    verify {
        let ledger = Ledger::<T>::get(&controller.account()).expect("ledger not created after");
        let new_total: BalanceOf<T> = ledger.total;
        assert!(original_total > new_total);
    }

    // Worst case scenario, everything is removed after the bonding duration
    withdraw_unbonded_kill {
        // Slashing Spans
        let s in 0 .. MAX_SPANS;
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(0, INIT_BALANCE).unwrap();
        add_slashing_spans::<T>(&stash.account(), s);
        let amount = INIT_BALANCE;
        Staking::<T>::unbond(controller.origin().into(), amount.into()).unwrap();
        CurrentEra::put(EraIndex::max_value());
        let ledger = Ledger::<T>::get(&controller.account()).expect("ledger not created before");
        let original_total: BalanceOf<T> = ledger.total;
        whitelist_account!(controller);
    }: withdraw_unbonded(controller.origin(), s)
    verify {
        assert!(!Ledger::<T>::contains_key(controller.account()));
    }

    validate {
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(70, INIT_BALANCE)?;
        add_perm_validator::<T>(stash.did(), Some(2));
        let prefs = ValidatorPrefs::default();
        whitelist_account!(controller);
    }: _(controller.origin(), prefs)
    verify {
        assert!(Validators::<T>::contains_key(stash.account()));
    }
    /*
    kick {
        // scenario: we want to kick `k` nominators from nominating us (we are a validator).
        // we'll assume that `k` is under 128 for the purposes of determining the slope.
        // each nominator should have `MAX_NOMINATIONS` validators nominated, and our validator
        // should be somewhere in there.
        let k in 1 .. 128;

        clear_validators_and_nominators::<T>();

        // these are the other validators; there are `MAX_NOMINATIONS - 1` of them, so there are a
        // total of `MAX_NOMINATIONS` validators in the system.
        let rest_of_validators = create_validators::<T>(MAX_NOMINATIONS as u32 - 1, 100)?;

        // this is the validator that will be kicking.
        let (stash, controller) = create_stash_controller::<T>(MAX_NOMINATIONS as u32 - 1, 100, Default::default())?;
        let stash_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(stash.clone());

        // they start validating.
        Staking::<T>::validate(RawOrigin::Signed(controller.clone()).into(), Default::default())?;

        // we now create the nominators. there will be `k` of them; each will nominate all
        // validators. we will then kick each of the `k` nominators from the main validator.
        let mut nominator_stashes = Vec::with_capacity(k as usize);
        for i in 0 .. k {
            // create a nominator stash.
            let (n_stash, n_controller) = create_stash_controller::<T>(MAX_NOMINATIONS as u32 + i, 100, Default::default())?;

            // bake the nominations; we first clone them from the rest of the validators.
            let mut nominations = rest_of_validators.clone();
            // then insert "our" validator somewhere in there (we vary it) to avoid accidental
            // optimisations/pessimisations.
            nominations.insert(i as usize % (nominations.len() + 1), stash_lookup.clone());
            // then we nominate.
            Staking::<T>::nominate(RawOrigin::Signed(n_controller.clone()).into(), nominations)?;

            nominator_stashes.push(n_stash);
        }

        // all nominators now should be nominating our validator...
        for n in nominator_stashes.iter() {
            assert!(Nominators::<T>::get(n).unwrap().targets.contains(&stash));
        }

        // we need the unlookuped version of the nominator stash for the kick.
        let kicks = nominator_stashes.iter()
            .map(|n| T::Lookup::unlookup(n.clone()))
            .collect::<Vec<_>>();

        whitelist_account!(controller);
    }: _(RawOrigin::Signed(controller), kicks)
    verify {
        // all nominators now should *not* be nominating our validator...
        for n in nominator_stashes.iter() {
            assert!(!Nominators::<T>::get(n).unwrap().targets.contains(&stash));
        }
    }
    */



    set_min_bond_threshold {
        let origin = RawOrigin::Root;
    }: _(origin, 10000u32.into())
    verify {
        assert_eq!(Staking::<T>::min_bond_threshold(), 10000u32.into());
    }

    add_permissioned_validator {
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(5, INIT_BALANCE).unwrap();
        Staking::<T>::set_validator_count(RawOrigin::Root.into(), 10).unwrap();
    }: _(RawOrigin::Root, stash.did(), Some(1))
    verify {
        let pref = Staking::<T>::permissioned_identity(stash.did());
        assert!(pref.is_some(), "fail to add a permissioned identity");
        assert_eq!(pref.unwrap().intended_count, 1, "fail to set a incorrect intended count");
    }

    remove_permissioned_validator {
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(5, INIT_BALANCE).unwrap();
        add_perm_validator::<T>(stash.did(), Some(1));
    }: _(RawOrigin::Root, stash.did())
    verify {
        let pref = Staking::<T>::permissioned_identity(stash.did());
        assert!(pref.is_none(), "fail to remove a permissioned identity");
    }

    set_commission_cap {
        let m in 0 .. MAX_ALLOWED_VALIDATORS;
        let mut stashes = Vec::with_capacity(m as usize);
        // Add validators
        for i in 0 .. m {
            let stash = create_funded_user::<T>("stash", i, 1000);
            stashes.push(stash.account());
            Validators::<T>::insert(stash.account(), ValidatorPrefs { commission: Perbill::from_percent(70), ..Default::default() });
        }
    }: _(RawOrigin::Root, Perbill::from_percent(50))
    verify {
        stashes.iter().for_each(|s| {
            assert_eq!(Staking::<T>::validators(s), ValidatorPrefs { commission: Perbill::from_percent(50), ..Default::default() });
        });
    }

    // Worst case scenario, MAX_NOMINATIONS
    nominate {
        let n in 1 .. MAX_NOMINATIONS as u32;
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(n + 1, INIT_BALANCE)?;
        let validators = create_validators::<T>(n, INIT_BALANCE)?;
        whitelist_account!(controller);
    }: _(controller.origin(), validators)
    verify {
        assert!(Nominators::<T>::contains_key(stash.account()));
    }

    chill {
        clear_validators_and_nominators::<T>();
        let (_, controller) = create_stash_controller::<T>(10, INIT_BALANCE)?;
        whitelist_account!(controller);
    }: _(controller.origin())

    set_payee {
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(10, INIT_BALANCE).unwrap();
        assert_eq!(Payee::<T>::get(&stash.account()), RewardDestination::Staked);
        whitelist_account!(controller);
    }: _(controller.origin(), RewardDestination::Controller)
    verify {
        assert_eq!(Payee::<T>::get(&stash.account()), RewardDestination::Controller);
    }

    set_controller {
        clear_validators_and_nominators::<T>();
        let (stash, _) = create_stash_controller::<T>(10, INIT_BALANCE).unwrap();
        let new_controller = create_funded_user::<T>("new_controller", 10, INIT_BALANCE);
        let new_controller_lookup = new_controller.lookup();
        whitelist_account!(stash);
    }: _(stash.origin(), new_controller_lookup)
    verify {
        assert!(Ledger::<T>::contains_key(&new_controller.account()));
    }

    set_validator_count {
        let c in 0 .. MAX_VALIDATORS;
    }: _(RawOrigin::Root, c)
    verify {
        assert_eq!(ValidatorCount::get(), c);
    }

    force_no_eras {}: _(RawOrigin::Root)
    verify { assert_eq!(ForceEra::get(), Forcing::ForceNone); }

    force_new_era {}: _(RawOrigin::Root)
    verify { assert_eq!(ForceEra::get(), Forcing::ForceNew); }

    force_new_era_always {}: _(RawOrigin::Root)
    verify { assert_eq!(ForceEra::get(), Forcing::ForceAlways); }

    // Worst case scenario, the list of invulnerables is very long.
    set_invulnerables {
        let v in 0 .. MAX_VALIDATORS;
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
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(0, INIT_BALANCE).unwrap();
        add_slashing_spans::<T>(&stash.account(), s);
    }: _(RawOrigin::Root, stash.account(), s)
    verify {
        assert!(!Ledger::<T>::contains_key(&controller.account()));
    }

    cancel_deferred_slash {
        let s in 1 .. MAX_SLASHES;
        let mut unapplied_slashes = Vec::new();
        let era = EraIndex::one();
        let unapplied_slash = UnappliedSlash {
            validator: account("validator", 0, 10000),
            own: Default::default(),
            others: Vec::new(),
            reporters: Vec::new(),
            payout: Default::default(),
        };
        for _ in 0 .. MAX_SLASHES {
            unapplied_slashes.push(unapplied_slash.clone());
        }
        UnappliedSlashes::<T>::insert(era, &unapplied_slashes);

        let slash_indices: Vec<u32> = (0 .. s).collect();
    }: _(RawOrigin::Root, era, slash_indices)
    verify {
        assert_eq!(UnappliedSlashes::<T>::get(&era).len(), (MAX_SLASHES - s) as usize);
    }

    payout_stakers {
        let n in 1 .. T::MaxNominatorRewardedPerValidator::get() as u32;
        let (origin, validator, current_era, balance_before) = payout_stakers_::<T>(false, n).unwrap();
    }: _(origin, validator.clone(), current_era)
    verify {
        // Validator has been paid!
        let balance_after = T::Currency::free_balance(&validator);
        assert!(balance_before < balance_after);
    }

    payout_stakers_alive_controller {
        let n in 1 .. T::MaxNominatorRewardedPerValidator::get() as u32;
        let (origin, validator, current_era, balance_before) = payout_stakers_::<T>(true, n).unwrap();
    }: payout_stakers(origin, validator.clone(), current_era)
    verify {
        // Validator has been paid!
        let balance_after = T::Currency::free_balance(&validator);
        assert!(balance_before < balance_after);
    }

    rebond {
        // User account seed
        let u in 0 .. 1000;
        let l in 1 .. MAX_UNLOCKING_CHUNKS as u32;
        clear_validators_and_nominators::<T>();
        let (_, controller) = create_stash_controller::<T>(u, INIT_BALANCE).unwrap();
        let mut staking_ledger = Ledger::<T>::get(controller.account()).unwrap();
        let unlock_chunk = UnlockChunk::<BalanceOf<T>> {
            value: 1u32.into(),
            era: EraIndex::zero(),
        };
        for _ in 0 .. l {
            staking_ledger.unlocking.push(unlock_chunk.clone())
        }
        Ledger::<T>::insert(controller.account(), staking_ledger.clone());
        let original_bonded: BalanceOf<T> = staking_ledger.active;
        whitelist_account!(controller);
    }: _(controller.origin(), (l + 100).into())
    verify {
        let ledger = Ledger::<T>::get(&controller.account()).expect("ledger not created after");
        let new_bonded: BalanceOf<T> = ledger.active;
        assert!(original_bonded < new_bonded);
    }

    set_history_depth {
        let e in 1 .. 100;
        HistoryDepth::put(e);
        CurrentEra::put(e);
        for i in 0 .. e {
            let acc: T::AccountId = account("validator", i, 10000);
            <ErasStakers<T>>::insert(i, acc.clone(), Exposure::<T::AccountId, BalanceOf<T>>::default());
            <ErasStakersClipped<T>>::insert(i, acc.clone(), Exposure::<T::AccountId, BalanceOf<T>>::default());
            <ErasValidatorPrefs<T>>::insert(i, acc, ValidatorPrefs::default());
            <ErasValidatorReward<T>>::insert(i, BalanceOf::<T>::one());
            <ErasRewardPoints<T>>::insert(i, EraRewardPoints::<T::AccountId>::default());
            <ErasTotalStake<T>>::insert(i, BalanceOf::<T>::one());
            ErasStartSessionIndex::insert(i, i);
        }
    }: _(RawOrigin::Root, EraIndex::zero(), u32::max_value())
    verify {
        assert_eq!(HistoryDepth::get(), 0);
    }

    reap_stash {
        let s in 1 .. MAX_SPANS;
        clear_validators_and_nominators::<T>();
        let (stash, controller) = create_stash_controller::<T>(0, INIT_BALANCE).unwrap();
        add_slashing_spans::<T>(&stash.account(), s);
        T::Currency::make_free_balance_be(&stash.account(), 0u32.into());
        whitelist_account!(controller);
    }: _(controller.origin(), stash.account(), s)
    verify {
        assert!(!Bonded::<T>::contains_key(&stash.account()));
    }

    new_era {
        let v in 1 .. 10;
        let n in 1 .. 100;

        create_validators_with_nominators_for_era::<T>(v, n, MAX_NOMINATIONS as usize, false, None).unwrap();
        let session_index = SessionIndex::one();
    }: {
        let validators = Staking::<T>::new_era(session_index).ok_or("`new_era` failed")?;
        assert!(validators.len() == v as usize);
    }

    payout_all {
        let v in 1 .. 10;
        let n in 1 .. 100;
        create_validators_with_nominators_for_era::<T>(v, n, MAX_NOMINATIONS as usize, false, None).unwrap();
        // Start a new Era
        let new_validators = Staking::<T>::new_era(SessionIndex::one()).unwrap();
        assert_eq!(new_validators.len(), v as usize);

        let current_era = CurrentEra::get().unwrap();
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
        let total_payout: BalanceOf<T> = 1000u32.into();
        <ErasValidatorReward<T>>::insert(current_era, total_payout);

        let caller: T::AccountId = whitelisted_caller();
    }: {
        for arg in payout_calls_arg {
            <Staking<T>>::payout_stakers(RawOrigin::Signed(caller.clone()).into(), arg.0, arg.1).unwrap();
        }
    }

    do_slash {
        let l in 1 .. MAX_UNLOCKING_CHUNKS as u32;
        let (stash, controller) = create_stash_controller::<T>(0, INIT_BALANCE)?;
        let mut staking_ledger = Ledger::<T>::get(controller.account()).unwrap();
        let unlock_chunk = UnlockChunk::<BalanceOf<T>> {
            value: 1u32.into(),
            era: EraIndex::zero(),
        };
        for _ in 0 .. l {
            staking_ledger.unlocking.push(unlock_chunk.clone())
        }
        Ledger::<T>::insert(controller.account(), staking_ledger);
        let balance_before = T::Currency::free_balance(&stash.account());
    }: {
        crate::slashing::do_slash::<T>(
            &stash.account(),
            10u32.into(),
            &mut BalanceOf::<T>::zero(),
            &mut NegativeImbalanceOf::<T>::zero()
        );
    } verify {
        let balance_after = T::Currency::free_balance(&stash.account());
        assert!(balance_before > balance_after);
    }

    // same as submit_solution_initial but we place a very weak solution on chian first.
    submit_solution_better {
        // number of validator intention.
        let v in 1000 .. 2000;
        // number of nominator intention.
        let n in 1000 .. 2000;
        // number of assignments. Basically, number of active nominators.
        let a in 200 .. 500;
        // number of winners, also ValidatorCount.
        let w in 16 .. 100;

        assert!(w as usize >= MAX_NOMINATIONS as usize, "doesn't support lower value");

        let winners = create_validators_with_nominators_for_era::<T>(
            v,
            n,
            MAX_NOMINATIONS as usize,
            false,
            Some(w),
        ).unwrap();

        // needed for the solution to be generates.
        assert!(<Staking<T>>::create_stakers_snapshot().0);

        // set number of winners
        ValidatorCount::put(w);

        // create a assignments in total for the w winners.
        let (winners, assignments) = create_assignments_for_offchain::<T>(a, winners).unwrap();

        let single_winner = winners[0].0.clone();

        let (
            winners,
            compact,
            score,
            size
        ) = offchain_election::prepare_submission::<T>(
            assignments,
            winners,
            false,
            T::BlockWeights::get().max_block,
        ).unwrap();

        assert_eq!(
            winners.len(), compact.unique_targets().len(),
            "unique targets ({}) and winners ({}) count not same. This solution is not valid.",
            compact.unique_targets().len(),
            winners.len(),
        );

        // needed for the solution to be accepted
        <EraElectionStatus<T>>::put(ElectionStatus::Open(T::BlockNumber::from(1u32)));

        let era = <Staking<T>>::current_era().unwrap_or(0);
        let caller = create_funded_user::<T>("caller", n, 10000);
        whitelist_account!(caller);

        // submit a very bad solution on-chain
        {
            // this is needed to fool the chain to accept this solution.
            ValidatorCount::put(1);
            let (winners, compact, score, size) = get_single_winner_solution::<T>(single_winner).unwrap();
            assert!(
                <Staking<T>>::submit_election_solution(
                    caller.origin().into(),
                    winners,
                    compact,
                    score.clone(),
                    era,
                    size,
            ).is_ok());

            // new solution has been accepted.
            assert_eq!(<Staking<T>>::queued_score().unwrap(), score);
            ValidatorCount::put(w);
        }
    }: {
        let result = <Staking<T>>::submit_election_solution(
            caller.origin().into(),
            winners,
            compact,
            score.clone(),
            era,
            size,
        );
        assert!(result.is_ok());
    }
    verify {
        // new solution has been accepted.
        assert_eq!(<Staking<T>>::queued_score().unwrap(), score);
    }

    change_slashing_allowed_for {

    }: _(RawOrigin::Root, SlashingSwitch::ValidatorAndNominator)
    verify {
        assert_eq!(Staking::<T>::slashing_allowed_for(), SlashingSwitch::ValidatorAndNominator, "Incorrect value set");
    }

    update_permissioned_validator_intended_count {
        let (stash, controller) = create_stash_controller::<T>(5, INIT_BALANCE).unwrap();
        let stash_id = stash.did();
        add_perm_validator::<T>(stash_id, Some(1));
    }: _(RawOrigin::Root, stash_id, 2)
    verify {
        assert_eq!(Staking::<T>::permissioned_identity(stash_id).unwrap().intended_count, 2, "Unable to update intended validator count");
    }

    increase_validator_count {
        Staking::<T>::set_validator_count(RawOrigin::Root.into(), 10)
        .expect("Failed to set the validator count");
    }: _(RawOrigin::Root, 15)
    verify {
        assert_eq!(Staking::<T>::validator_count(), 25);
    }

    scale_validator_count {
        Staking::<T>::set_validator_count(RawOrigin::Root.into(), 10)
        .expect("Failed to set the validator count");
    }: _(RawOrigin::Root, Percent::from_percent(25))
    verify {
        assert_eq!(Staking::<T>::validator_count(), 12);
    }

    chill_from_governance {
        let s in 1 .. MAX_STASHES;
        let validator = create_funded_user::<T>("caller", 2, 10000);
        let validator_did = validator.did();
        let mut signatories = Vec::with_capacity(s as usize);

        // Increases the maximum number of validators
        emulate_validator_setup::<T>(1, 1000, Perbill::from_percent(60));
        assert_eq!(Staking::<T>::validator_count(), 1000);

        // Add validator
        Staking::<T>::add_permissioned_validator(RawOrigin::Root.into(), validator_did, Some(MAX_STASHES))
        .expect("Failed to add permissioned validator");
        whitelist_account!(validator);

        for x in 0 .. s {
            // Create stash key
            let key = create_funded_user_without_did::<T>("stash", x, 10000);
            // Add key to signatories vec
            signatories.push(key.account.clone());
            // Add key as secondary key to validator_did
            Identity::<T>::unsafe_join_identity(validator_did, Permissions::default(), key.account.clone());
            // Use key to bond
            Staking::<T>::bond(key.origin().into(), key.lookup(), 2_000_000u32.into(), RewardDestination::Staked).expect("Bond failed.");
            // Create ValidatorPrefs
            let validator_prefs = ValidatorPrefs {
                commission: Perbill::from_percent(50),
                ..Default::default()
            };

            whitelist_account!(key);
            // Use stash key to validate
            Staking::<T>::validate(key.origin().into(), validator_prefs).expect("Validate fails");
            // Checks that the stash key is in validators storage
            assert_eq!(<Validators<T>>::contains_key(&key.account), true);
        }    
    }: _(RawOrigin::Root, validator_did, signatories.clone())
    verify {
        for key in signatories {
            // Checks that the stash key has been removed from storage
            assert_eq!(<Validators<T>>::contains_key(&key), false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{Balances, ExtBuilder, Origin, Staking, Test};
    use frame_support::assert_ok;

    #[test]
    fn create_validators_with_nominators_for_era_works() {
        ExtBuilder::default()
            .has_stakers(true)
            .build()
            .execute_with(|| {
                let v = 10;
                let n = 100;

                create_validators_with_nominators_for_era::<Test>(
                    v,
                    n,
                    MAX_NOMINATIONS,
                    false,
                    None,
                )
                .unwrap();

                let count_validators = Validators::<Test>::iter().count();
                let count_nominators = Nominators::<Test>::iter().count();

                assert_eq!(count_validators, v as usize);
                assert_eq!(count_nominators, n as usize);
            });
    }

    #[test]
    fn create_validator_with_nominators_works() {
        ExtBuilder::default()
            .has_stakers(true)
            .build()
            .execute_with(|| {
                let n = 10;

                let (validator_stash, nominators) = create_validator_with_nominators::<Test>(
                    n,
                    <Test as Config>::MaxNominatorRewardedPerValidator::get() as u32,
                    false,
                )
                .unwrap();

                assert_eq!(nominators.len() as u32, n);

                let current_era = CurrentEra::get().unwrap();

                let original_free_balance = Balances::free_balance(&validator_stash);
                assert_ok!(Staking::payout_stakers(
                    Origin::signed(1337),
                    validator_stash,
                    current_era
                ));
                let new_free_balance = Balances::free_balance(&validator_stash);

                assert!(original_free_balance < new_free_balance);
            });
    }

    #[test]
    fn add_slashing_spans_works() {
        ExtBuilder::default()
            .has_stakers(true)
            .build()
            .execute_with(|| {
                let n = 10;

                let (validator_stash, _nominators) = create_validator_with_nominators::<Test>(
                    n,
                    <Test as Config>::MaxNominatorRewardedPerValidator::get() as u32,
                    false,
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
        ExtBuilder::default()
            .has_stakers(true)
            .build()
            .execute_with(|| {
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
                    )
                    .unwrap();

                assert_ok!(closure_to_benchmark());
            });
    }

    #[test]
    fn test_benchmarks() {
        ExtBuilder::default()
            .has_stakers(true)
            .build()
            .execute_with(|| {
                assert_ok!(test_benchmark_bond::<Test>());
                assert_ok!(test_benchmark_bond_extra::<Test>());
                assert_ok!(test_benchmark_unbond::<Test>());
                assert_ok!(test_benchmark_withdraw_unbonded_update::<Test>());
                assert_ok!(test_benchmark_withdraw_unbonded_kill::<Test>());
                assert_ok!(test_benchmark_validate::<Test>());
                assert_ok!(test_benchmark_nominate::<Test>());
                assert_ok!(test_benchmark_chill::<Test>());
                assert_ok!(test_benchmark_set_payee::<Test>());
                assert_ok!(test_benchmark_set_controller::<Test>());
                assert_ok!(test_benchmark_set_validator_count::<Test>());
                assert_ok!(test_benchmark_force_no_eras::<Test>());
                assert_ok!(test_benchmark_force_new_era::<Test>());
                assert_ok!(test_benchmark_force_new_era_always::<Test>());
                assert_ok!(test_benchmark_set_invulnerables::<Test>());
                assert_ok!(test_benchmark_force_unstake::<Test>());
                assert_ok!(test_benchmark_cancel_deferred_slash::<Test>());
                assert_ok!(test_benchmark_payout_stakers::<Test>());
                assert_ok!(test_benchmark_payout_stakers_alive_controller::<Test>());
                assert_ok!(test_benchmark_rebond::<Test>());
                assert_ok!(test_benchmark_set_history_depth::<Test>());
                assert_ok!(test_benchmark_reap_stash::<Test>());
                assert_ok!(test_benchmark_new_era::<Test>());
                assert_ok!(test_benchmark_do_slash::<Test>());
                assert_ok!(test_benchmark_payout_all::<Test>());
                // only run one of them to same time on the CI. ignore the other two.
                assert_ok!(test_benchmark_submit_solution_initial::<Test>());
            });
    }

    #[test]
    #[ignore]
    fn test_benchmarks_offchain() {
        ExtBuilder::default()
            .has_stakers(false)
            .build()
            .execute_with(|| {
                assert_ok!(test_benchmark_submit_solution_better::<Test>());
                assert_ok!(test_benchmark_submit_solution_weaker::<Test>());
            });
    }
}
