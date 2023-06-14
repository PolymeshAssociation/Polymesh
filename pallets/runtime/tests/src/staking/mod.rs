// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
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

//! Tests for the module.
//!
#[macro_export]
macro_rules! assert_session_era {
    ($session:expr, $era:expr) => {
        assert_eq!(
            Session::current_index(),
            $session,
            "wrong session {} != {}",
            Session::current_index(),
            $session,
        );
        assert_eq!(
            Staking::active_era().unwrap().index,
            $era,
            "wrong active era {} != {}",
            Staking::active_era().unwrap().index,
            $era,
        );
    };
}

macro_rules! assert_present_identity {
    ($acc_id:expr) => {
        assert!(Staking::permissioned_identity(Identity::get_identity($acc_id).unwrap()).is_some());
    };
}

macro_rules! assert_absent_identity {
    ($acc_id:expr) => {
        assert!(Staking::permissioned_identity(Identity::get_identity($acc_id).unwrap()).is_none());
    };
}

macro_rules! assert_add_permissioned_validator {
    ($acc_id:expr) => {
        assert_ok!(Staking::add_permissioned_validator(
            root(),
            Identity::get_identity($acc_id).unwrap(),
            None
        ));
    };
}

macro_rules! assert_permissioned_identity_prefs {
    ($id:expr, $i_count:expr, $r_count:expr) => {
        assert_eq!(
            Staking::permissioned_identity($id).unwrap(),
            PermissionedIdentityPrefs {
                intended_count: $i_count,
                running_count: $r_count
            }
        );
    };
}

// Polymath: Re-implement `assert_eq_uvec` from substrate to fix compile warnings.
#[macro_export]
#[doc(hidden)]
macro_rules! assert_eq_uvec {
    ( $x:expr, $y:expr $(,)? ) => {
        $crate::__assert_eq_uvec!($x, $y);
        $crate::__assert_eq_uvec!($y, $x);
    };
}
#[macro_export]
#[doc(hidden)]
macro_rules! __assert_eq_uvec {
    ( $x:expr, $y:expr ) => {
        $x.iter().for_each(|e| {
            // Polymath: removed the un-needed `format!`.
            if !$y.contains(e) {
                panic!("vectors not equal: {:?} != {:?}", $x, $y);
            }
        });
    };
}

mod mock;

use crate::asset_test::set_timestamp;
use chrono::prelude::Utc;
use codec::Decode;
use frame_support::{
    assert_noop, assert_ok,
    storage::{IterableStorageMap, StorageDoubleMap, StorageValue},
    traits::{Currency, Get, OnFinalize, OnInitialize, ReservableCurrency},
    StorageMap,
};
use mock::*;
use pallet_balances::Error as BalancesError;
use pallet_identity::Error as IdentityError;
use pallet_identity::{Claim1stKey, Claims};
use pallet_staking::*;
use polymesh_primitives::ClaimType;
use sp_npos_elections::ElectionScore;
use sp_runtime::{
    assert_eq_error_rate,
    offchain::storage::MutateStorageError::ValueFunctionFailed,
    traits::BadOrigin,
    transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
    Perbill,
};
use sp_staking::{
    offence::{DisableStrategy, OffenceDetails, OnOffenceHandler},
    SessionIndex,
};

#[test]
fn force_unstake_works() {
    ExtBuilder::default().build_and_execute(|| {
        // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(&11), Some(10));
        // Adds 2 slashing spans
        add_slash(&11);
        // Cant transfer
        assert_noop!(
            Balances::transfer(Origin::signed(11), 1, 10),
            BalancesError::<Test>::LiquidityRestrictions
        );
        // Force unstake requires root.
        assert_noop!(Staking::force_unstake(Origin::signed(11), 11, 2), BadOrigin);
        // Force unstake needs correct number of slashing spans (for weight calculation)
        assert_noop!(
            Staking::force_unstake(Origin::root(), 11, 0),
            Error::<Test>::IncorrectSlashingSpans
        );
        // We now force them to unstake
        assert_ok!(Staking::force_unstake(Origin::root(), 11, 2));
        // No longer bonded.
        assert_eq!(Staking::bonded(&11), None);
        // Transfer works.
        assert_ok!(Balances::transfer(Origin::signed(11), 1, 10));
    });
}

#[test]
fn kill_stash_works() {
    ExtBuilder::default().build_and_execute(|| {
        // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(&11), Some(10));
        // Adds 2 slashing spans
        add_slash(&11);
        // Only can kill a stash account
        assert_noop!(Staking::kill_stash(&12, 0), Error::<Test>::NotStash);
        // Respects slashing span count
        assert_noop!(
            Staking::kill_stash(&11, 0),
            Error::<Test>::IncorrectSlashingSpans
        );
        // Correct inputs, everything works
        assert_ok!(Staking::kill_stash(&11, 2));
        // No longer bonded.
        assert_eq!(Staking::bonded(&11), None);
    });
}

#[test]
fn basic_setup_works() {
    // Verifies initial conditions of mock
    ExtBuilder::default().build_and_execute(|| {
        // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(&11), Some(10));
        // Account 21 is stashed and locked, and account 20 is the controller
        assert_eq!(Staking::bonded(&21), Some(20));
        // Account 1 is not a stashed
        assert_eq!(Staking::bonded(&1), None);

        // Account 10 controls the stash from account 11, which is 100 * balance_factor units
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![]
            })
        );
        // Account 20 controls the stash from account 21, which is 200 * balance_factor units
        assert_eq!(
            Staking::ledger(&20),
            Some(StakingLedger {
                stash: 21,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![]
            })
        );
        // Account 1 does not control any stash
        assert_eq!(Staking::ledger(&1), None);

        // ValidatorPrefs are default
        assert_eq_uvec!(
            Validators::<Test>::iter().collect::<Vec<_>>(),
            vec![
                (31, ValidatorPrefs::default()),
                (21, ValidatorPrefs::default()),
                (11, ValidatorPrefs::default())
            ]
        );

        assert_eq!(
            Staking::ledger(100),
            Some(StakingLedger {
                stash: 101,
                total: 500,
                active: 500,
                unlocking: vec![],
                claimed_rewards: vec![]
            })
        );
        assert_eq!(Staking::nominators(101).unwrap().targets, vec![11, 21]);

        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
            Exposure {
                total: 1125,
                own: 1000,
                others: vec![IndividualExposure {
                    who: 101,
                    value: 125
                }]
            },
        );
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
            Exposure {
                total: 1375,
                own: 1000,
                others: vec![IndividualExposure {
                    who: 101,
                    value: 375
                }]
            },
        );

        // initial total stake = 1125 + 1375
        assert_eq!(
            Staking::eras_total_stake(Staking::active_era().unwrap().index),
            2500
        );

        // The number of validators required.
        assert_eq!(Staking::validator_count(), 2);

        // Initial Era and session
        assert_eq!(Staking::active_era().unwrap().index, 0);

        // Account 10 has `balance_factor` free balance
        assert_eq!(Balances::free_balance(10), 1);
        assert_eq!(Balances::free_balance(10), 1);

        // New era is not being forced
        assert_eq!(Staking::force_era(), Forcing::NotForcing);
    });
}

#[test]
fn change_controller_works() {
    ExtBuilder::default().build_and_execute(|| {
        // 10 and 11 are bonded as stash controller.
        assert_eq!(Staking::bonded(&11), Some(10));

        // 10 can control 11 who is initially a validator.
        assert_ok!(Staking::chill(Origin::signed(10)));

        // change controller
        assert_ok!(Staking::set_controller(Origin::signed(11), 5));
        assert_eq!(Staking::bonded(&11), Some(5));
        mock::start_active_era(1);

        // 10 is no longer in control.
        assert_noop!(
            Staking::validate(Origin::signed(10), ValidatorPrefs::default()),
            Error::<Test>::NotController,
        );
        assert_ok!(Staking::validate(
            Origin::signed(5),
            ValidatorPrefs::default()
        ));
    })
}

#[test]
fn rewards_should_work() {
    ExtBuilder::default()
        .nominate(true)
        .session_per_era(3)
        .build_and_execute(|| {
            let init_balance_10 = Balances::total_balance(&10);
            let init_balance_11 = Balances::total_balance(&11);
            let init_balance_20 = Balances::total_balance(&20);
            let init_balance_21 = Balances::total_balance(&21);
            let init_balance_100 = Balances::total_balance(&100);
            let init_balance_101 = Balances::total_balance(&101);

            // Set payees
            Payee::<Test>::insert(11, RewardDestination::Controller);
            Payee::<Test>::insert(21, RewardDestination::Controller);
            Payee::<Test>::insert(101, RewardDestination::Controller);

            <Module<Test>>::reward_by_ids(vec![(11, 50)]);
            <Module<Test>>::reward_by_ids(vec![(11, 50)]);
            // This is the second validator of the current elected set.
            <Module<Test>>::reward_by_ids(vec![(21, 50)]);

            // Compute total payout now for whole duration of the session.
            let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());
            let maximum_payout = maximum_payout_for_duration(reward_time_per_era());

            start_session(1);

            assert_eq!(Balances::total_balance(&10), init_balance_10);
            assert_eq!(Balances::total_balance(&11), init_balance_11);
            assert_eq!(Balances::total_balance(&20), init_balance_20);
            assert_eq!(Balances::total_balance(&21), init_balance_21);
            assert_eq!(Balances::total_balance(&100), init_balance_100);
            assert_eq!(Balances::total_balance(&101), init_balance_101);
            assert_eq_uvec!(Session::validators(), vec![11, 21]);
            assert_eq!(
                Staking::eras_reward_points(Staking::active_era().unwrap().index),
                EraRewardPoints {
                    total: 50 * 3,
                    individual: vec![(11, 100), (21, 50)].into_iter().collect(),
                }
            );
            let part_for_10 = Perbill::from_rational::<u32>(1000, 1125);
            let part_for_20 = Perbill::from_rational::<u32>(1000, 1375);
            let part_for_100_from_10 = Perbill::from_rational::<u32>(125, 1125);
            let part_for_100_from_20 = Perbill::from_rational::<u32>(375, 1375);

            start_session(2);
            start_session(3);

            assert_eq!(Staking::active_era().unwrap().index, 1);
            assert_eq!(
                mock::REWARD_REMAINDER_UNBALANCED.with(|v| *v.borrow()),
                maximum_payout - total_payout_0,
            );
            assert_eq!(
                *mock::staking_events().last().unwrap(),
                RawEvent::EraPayout(0, total_payout_0, maximum_payout - total_payout_0)
            );
            mock::make_all_reward_payment(0);

            assert_eq_error_rate!(
                Balances::total_balance(&10),
                init_balance_10 + part_for_10 * total_payout_0 * 2 / 3,
                2,
            );
            assert_eq_error_rate!(Balances::total_balance(&11), init_balance_11, 2,);
            assert_eq_error_rate!(
                Balances::total_balance(&20),
                init_balance_20 + part_for_20 * total_payout_0 * 1 / 3,
                2,
            );
            assert_eq_error_rate!(Balances::total_balance(&21), init_balance_21, 2,);
            assert_eq_error_rate!(
                Balances::total_balance(&100),
                init_balance_100
                    + part_for_100_from_10 * total_payout_0 * 2 / 3
                    + part_for_100_from_20 * total_payout_0 * 1 / 3,
                2
            );
            assert_eq_error_rate!(Balances::total_balance(&101), init_balance_101, 2);

            assert_eq_uvec!(Session::validators(), vec![11, 21]);
            <Module<Test>>::reward_by_ids(vec![(11, 1)]);

            // Compute total payout now for whole duration as other parameter won't change
            let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());

            mock::start_active_era(2);
            assert_eq!(
                mock::REWARD_REMAINDER_UNBALANCED.with(|v| *v.borrow()),
                maximum_payout * 2 - total_payout_0 - total_payout_1,
            );
            assert_eq!(
                *mock::staking_events().last().unwrap(),
                RawEvent::EraPayout(1, total_payout_1, maximum_payout - total_payout_1)
            );
            mock::make_all_reward_payment(1);

            assert_eq_error_rate!(
                Balances::total_balance(&10),
                init_balance_10 + part_for_10 * (total_payout_0 * 2 / 3 + total_payout_1),
                2,
            );
            assert_eq_error_rate!(Balances::total_balance(&11), init_balance_11, 2,);
            assert_eq_error_rate!(
                Balances::total_balance(&20),
                init_balance_20 + part_for_20 * total_payout_0 * 1 / 3,
                2,
            );
            assert_eq_error_rate!(Balances::total_balance(&21), init_balance_21, 2,);
            assert_eq_error_rate!(
                Balances::total_balance(&100),
                init_balance_100
                    + part_for_100_from_10 * (total_payout_0 * 2 / 3 + total_payout_1)
                    + part_for_100_from_20 * total_payout_0 * 1 / 3,
                2
            );
            assert_eq_error_rate!(Balances::total_balance(&101), init_balance_101, 2);
        });
}

#[test]
fn staking_should_work() {
    // should test:
    // * new validators can be added to the default set
    // * new ones will be chosen per era
    // * either one can unlock the stash and back-down from being a validator via `chill`ing.
    ExtBuilder::default()
        .nominate(false)
        .fair(false) // to give 20 more staked value
        .build()
        .execute_with(|| {
            // --- Block 1:
            start_session(1);

            // remember + compare this along with the test.
            assert_eq_uvec!(validator_controllers(), vec![20, 10]);

            // put some money in account that we'll use.
            for i in 1..5 {
                let _ = Balances::make_free_balance_be(&i, 2000);
            }

            // --- Block 2:
            start_session(2);
            provide_did_to_user(3);
            add_secondary_key(3, 4);
            // add a new candidate for being a validator. account 3 controlled by 4.
            assert_ok!(Staking::bond(
                Origin::signed(3),
                4,
                1500,
                RewardDestination::Controller
            ));
            assert_add_permissioned_validator!(&3);
            assert_ok!(Staking::validate(
                Origin::signed(4),
                ValidatorPrefs::default()
            ));
            assert_ok!(Session::set_keys(
                Origin::signed(4),
                SessionKeys { other: 4.into() },
                vec![]
            ));

            // No effects will be seen so far.
            assert_eq_uvec!(validator_controllers(), vec![20, 10]);

            // --- Block 3:
            start_session(3);

            // No effects will be seen so far. Era has not been yet triggered.
            assert_eq_uvec!(validator_controllers(), vec![20, 10]);

            // --- Block 4: the validators will now be queued.
            start_session(4);
            assert_eq!(Staking::active_era().unwrap().index, 1);

            // --- Block 5: the validators are still in queue.
            start_session(5);

            // --- Block 6: the validators will now be changed.
            start_session(6);

            assert_eq_uvec!(validator_controllers(), vec![20, 4]);
            // --- Block 6: Unstake 4 as a validator, freeing up the balance stashed in 3
            // 4 will chill
            Staking::chill(Origin::signed(4)).unwrap();

            // --- Block 7: nothing. 4 is still there.
            start_session(7);
            assert_eq_uvec!(validator_controllers(), vec![20, 4]);

            // --- Block 8:
            start_session(8);

            // --- Block 9: 4 will not be a validator.
            start_session(9);
            assert_eq_uvec!(validator_controllers(), vec![20, 10]);

            // Note: the stashed value of 4 is still lock
            assert_eq!(
                Staking::ledger(&4),
                Some(StakingLedger {
                    stash: 3,
                    total: 1500,
                    active: 1500,
                    unlocking: vec![],
                    claimed_rewards: vec![0],
                })
            );
            // e.g. it cannot reserve more than 500 that it has free from the total 2000
            assert_noop!(
                Balances::reserve(&3, 501),
                BalancesError::<Test>::LiquidityRestrictions
            );
            assert_ok!(Balances::reserve(&3, 409));
        });
}

#[test]
fn less_than_needed_candidates_works() {
    ExtBuilder::default()
        .minimum_validator_count(1)
        .validator_count(4)
        .nominate(false)
        .num_validators(3)
        .build()
        .execute_with(|| {
            assert_eq!(Staking::validator_count(), 4);
            assert_eq!(Staking::minimum_validator_count(), 1);
            assert_eq_uvec!(validator_controllers(), vec![30, 20, 10]);

            mock::start_active_era(1);

            // Previous set is selected. NO election algorithm is even executed.
            assert_eq_uvec!(validator_controllers(), vec![30, 20, 10]);

            // But the exposure is updated in a simple way. No external votes exists.
            // This is purely self-vote.
            assert!(
                ErasStakers::<Test>::iter_prefix_values(Staking::active_era().unwrap().index)
                    .all(|exposure| exposure.others.is_empty())
            );
        });
}

#[test]
fn no_candidate_emergency_condition() {
    ExtBuilder::default()
        .minimum_validator_count(1)
        .validator_count(8)
        .num_validators(4)
        .validator_pool(true)
        .nominate(false)
        .build()
        .execute_with(|| {
            // initial validators
            assert_eq_uvec!(validator_controllers(), vec![10, 20, 30, 40]);
            let prefs = ValidatorPrefs {
                commission: Perbill::one(),
                ..Default::default()
            };
            Validators::<Test>::insert(11, prefs.clone());

            // set the minimum validator count.
            MinimumValidatorCount::put(10);

            // try to chill
            let _ = Staking::chill(Origin::signed(10));

            // trigger era
            mock::start_active_era(1);

            // Previous ones are elected. chill is invalidates. TODO: #2494
            assert_eq_uvec!(validator_controllers(), vec![10, 20, 30, 40]);
            // Though the validator preferences has been removed.
            assert!(Staking::validators(11) != prefs);
        });
}

#[ignore]
#[test]
fn nominating_and_rewards_should_work() {
    ExtBuilder::default()
        .nominate(false)
        .validator_pool(true)
        .build()
        .execute_with(|| {
            // initial validators -- everyone is actually even.
            assert_eq_uvec!(validator_controllers(), vec![40, 30]);

            // Set payee to controller
            assert_ok!(Staking::set_payee(
                Origin::signed(10),
                RewardDestination::Controller
            ));
            assert_ok!(Staking::set_payee(
                Origin::signed(20),
                RewardDestination::Controller
            ));
            assert_ok!(Staking::set_payee(
                Origin::signed(30),
                RewardDestination::Controller
            ));
            assert_ok!(Staking::set_payee(
                Origin::signed(40),
                RewardDestination::Controller
            ));

            // give the man some money
            let initial_balance = 1000;
            for i in [1, 2, 3, 4, 5, 10, 11, 20, 21].iter() {
                let _ = Balances::make_free_balance_be(i, initial_balance);
            }

            // bond two account pairs and state interest in nomination.
            // 2 will nominate for 10, 20, 30
            assert_ok!(Staking::bond(
                Origin::signed(1),
                2,
                1000,
                RewardDestination::Controller
            ));
            assert_ok!(Staking::nominate(Origin::signed(2), vec![11, 21, 31]));
            // 4 will nominate for 10, 20, 40
            assert_ok!(Staking::bond(
                Origin::signed(3),
                4,
                1000,
                RewardDestination::Controller
            ));
            assert_ok!(Staking::nominate(Origin::signed(4), vec![11, 21, 41]));

            // the total reward for era 0
            let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());
            <Module<Test>>::reward_by_ids(vec![(41, 1)]);
            <Module<Test>>::reward_by_ids(vec![(31, 1)]);

            mock::start_active_era(1);

            // 10 and 20 have more votes, they will be chosen.
            assert_eq_uvec!(validator_controllers(), vec![20, 10]);

            // OLD validators must have already received some rewards.
            mock::make_all_reward_payment(0);
            assert_eq!(Balances::total_balance(&40), 1 + total_payout_0 / 2);
            assert_eq!(Balances::total_balance(&30), 1 + total_payout_0 / 2);

            // ------ check the staked value of all parties.

            // 30 and 40 are not chosen anymore
            assert_eq!(
                ErasStakers::<Test>::iter_prefix_values(Staking::active_era().unwrap().index)
                    .count(),
                2
            );
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                Exposure {
                    total: 1000 + 800,
                    own: 1000,
                    others: vec![
                        IndividualExposure { who: 3, value: 400 },
                        IndividualExposure { who: 1, value: 400 },
                    ]
                },
            );
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                Exposure {
                    total: 1000 + 1200,
                    own: 1000,
                    others: vec![
                        IndividualExposure { who: 3, value: 600 },
                        IndividualExposure { who: 1, value: 600 },
                    ]
                },
            );

            // the total reward for era 1
            let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
            <Module<Test>>::reward_by_ids(vec![(21, 2)]);
            <Module<Test>>::reward_by_ids(vec![(11, 1)]);

            mock::start_active_era(2);

            // nothing else will happen, era ends and rewards are paid again,
            // it is expected that nominators will also be paid. See below

            mock::make_all_reward_payment(1);
            let payout_for_10 = total_payout_1 / 3;
            let payout_for_20 = 2 * total_payout_1 / 3;
            // Nominator 2: has [400/1800 ~ 2/9 from 10] + [600/2200 ~ 3/11 from 20]'s reward. ==> 2/9 + 3/11
            assert_eq_error_rate!(
                Balances::total_balance(&2),
                initial_balance + (2 * payout_for_10 / 9 + 3 * payout_for_20 / 11),
                2,
            );
            // Nominator 4: has [400/1800 ~ 2/9 from 10] + [600/2200 ~ 3/11 from 20]'s reward. ==> 2/9 + 3/11
            assert_eq_error_rate!(
                Balances::total_balance(&4),
                initial_balance + (2 * payout_for_10 / 9 + 3 * payout_for_20 / 11),
                2,
            );

            // Validator 10: got 800 / 1800 external stake => 8/18 =? 4/9 => Validator's share = 5/9
            assert_eq_error_rate!(
                Balances::total_balance(&10),
                initial_balance + 5 * payout_for_10 / 9,
                2,
            );
            // Validator 20: got 1200 / 2200 external stake => 12/22 =? 6/11 => Validator's share = 5/11
            assert_eq_error_rate!(
                Balances::total_balance(&20),
                initial_balance + 5 * payout_for_20 / 11,
                2,
            );
        });
}

#[test]
fn nominators_also_get_slashed_pro_rata() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);
        let slash_percent = Perbill::from_percent(5);
        let initial_exposure = Staking::eras_stakers(active_era(), 11);
        // 101 is a nominator for 11
        assert_eq!(initial_exposure.others.first().unwrap().who, 101,);

        // staked values;
        let nominator_stake = Staking::ledger(100).unwrap().active;
        let nominator_balance = balances(&101).0;
        let validator_stake = Staking::ledger(10).unwrap().active;
        let validator_balance = balances(&11).0;
        let exposed_stake = initial_exposure.total;
        let exposed_validator = initial_exposure.own;
        let exposed_nominator = initial_exposure.others.first().unwrap().value;

        // 11 goes offline
        on_offence_now(
            &[OffenceDetails {
                offender: (11, initial_exposure.clone()),
                reporters: vec![],
            }],
            &[slash_percent],
        );

        // Only validator stakes have been decreased not the nominator one.
        // Change is for the sake of Polymesh
        assert!(Staking::ledger(100).unwrap().active == nominator_stake);
        assert!(Staking::ledger(10).unwrap().active < validator_stake);

        let slash_amount = slash_percent * exposed_stake;
        let validator_share =
            Perbill::from_rational(exposed_validator, exposed_stake) * slash_amount;
        let nominator_share =
            Perbill::from_rational(exposed_nominator, exposed_stake) * slash_amount;

        // both slash amounts need to be positive for the test to make sense.
        assert!(validator_share > 0);
        assert!(nominator_share > 0);

        // // both stakes must have been decreased pro-rata.
        // assert_eq!(
        //     Staking::ledger(100).unwrap().active,
        //     nominator_stake - nominator_share,
        // );
        // assert_eq!(
        //     balances(&101).0, // free balance
        //     nominator_balance - nominator_share,
        // );
        // Polymesh -note -> Only Validator stake get decreased
        assert_eq!(
            Staking::ledger(10).unwrap().active,
            validator_stake - validator_share,
        );
        assert_eq!(
            balances(&101).0, // free balance
            nominator_balance
        );
        assert_eq!(
            balances(&11).0, // free balance
            validator_balance - validator_share,
        );
        // Because slashing happened.
        assert!(is_disabled(10));
    });
}

#[test]
fn double_staking_should_fail() {
    // should test (in the same order):
    // * an account already bonded as stash cannot be be stashed again.
    // * an account already bonded as stash cannot nominate.
    // * an account already bonded as controller can nominate.
    ExtBuilder::default().build_and_execute(|| {
        let arbitrary_value = MinimumBond::get();
        // 2 = controller, 1 stashed => ok
        assert_ok!(Staking::bond(
            Origin::signed(1),
            2,
            arbitrary_value,
            RewardDestination::default()
        ));
        // 4 = not used so far, 1 stashed => not allowed.
        assert_noop!(
            Staking::bond(
                Origin::signed(1),
                4,
                arbitrary_value,
                RewardDestination::default()
            ),
            Error::<Test>::AlreadyBonded,
        );
        // 1 = stashed => attempting to nominate should fail.
        assert_noop!(
            Staking::nominate(Origin::signed(1), vec![1]),
            Error::<Test>::NotController
        );

        // 2 = controller  => nominating should work.
        assert_noop!(
            Staking::nominate(Origin::signed(2), vec![1]),
            Error::<Test>::StashIdentityDoesNotExist,
        );
        provide_did_to_user(1);
        assert_ok!(Staking::nominate(Origin::signed(2), vec![1]));
    });
}

#[test]
fn double_controlling_should_fail() {
    // should test (in the same order):
    // * an account already bonded as controller CANNOT be reused as the controller of another account.
    ExtBuilder::default().build_and_execute(|| {
        let arbitrary_value = MinimumBond::get();
        // 2 = controller, 1 stashed => ok
        assert_ok!(Staking::bond(
            Origin::signed(1),
            2,
            arbitrary_value,
            RewardDestination::default(),
        ));
        // 2 = controller, 3 stashed (Note that 2 is reused.) => no-op
        assert_noop!(
            Staking::bond(
                Origin::signed(3),
                2,
                arbitrary_value,
                RewardDestination::default()
            ),
            Error::<Test>::AlreadyPaired,
        );
    });
}

#[test]
fn session_and_eras_work() {
    ExtBuilder::default().build_and_execute(|| {
        assert_eq!(Staking::active_era().unwrap().index, 0);
        assert_eq!(Session::current_index(), 0);

        // Session 1: No change.
        start_session(1);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(Staking::active_era().unwrap().index, 0);

        // Session 2: No change.
        start_session(2);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(Staking::active_era().unwrap().index, 0);

        // Session 3: Era increment.
        start_session(3);
        assert_eq!(Session::current_index(), 3);
        assert_eq!(Staking::active_era().unwrap().index, 1);

        // Session 4: No change.
        start_session(4);
        assert_eq!(Session::current_index(), 4);
        assert_eq!(Staking::active_era().unwrap().index, 1);

        // Session 5: No change.
        start_session(5);
        assert_eq!(Session::current_index(), 5);
        assert_eq!(Staking::active_era().unwrap().index, 1);

        // Session 6: Era increment.
        start_session(6);
        assert_eq!(Session::current_index(), 6);
        assert_eq!(Staking::active_era().unwrap().index, 2);
    });
}

#[test]
fn forcing_new_era_works() {
    ExtBuilder::default().build_and_execute(|| {
        // normal flow of session.
        assert_eq!(Staking::active_era().unwrap().index, 0);
        start_session(0);
        assert_eq!(Staking::active_era().unwrap().index, 0);
        start_session(1);
        assert_eq!(Staking::active_era().unwrap().index, 0);
        start_session(2);
        assert_eq!(Staking::active_era().unwrap().index, 0);
        start_session(3);
        assert_eq!(Staking::active_era().unwrap().index, 1);

        // no era change.
        ForceEra::put(Forcing::ForceNone);
        start_session(4);
        assert_eq!(Staking::active_era().unwrap().index, 1);
        start_session(5);
        assert_eq!(Staking::active_era().unwrap().index, 1);
        start_session(6);
        assert_eq!(Staking::active_era().unwrap().index, 1);
        start_session(7);
        assert_eq!(Staking::active_era().unwrap().index, 1);

        // back to normal.
        // this immediately starts a new session.
        ForceEra::put(Forcing::NotForcing);
        start_session(8);
        assert_eq!(Staking::active_era().unwrap().index, 1); // There is one session delay
        start_session(9);
        assert_eq!(Staking::active_era().unwrap().index, 2);

        // forceful change
        ForceEra::put(Forcing::ForceAlways);
        start_session(10);
        assert_eq!(Staking::active_era().unwrap().index, 2); // There is one session delay
        start_session(11);
        assert_eq!(Staking::active_era().unwrap().index, 3);
        start_session(12);
        assert_eq!(Staking::active_era().unwrap().index, 4);

        // just one forceful change
        ForceEra::put(Forcing::ForceNew);
        start_session(13);
        assert_eq!(Staking::active_era().unwrap().index, 5);
        assert_eq!(ForceEra::get(), Forcing::NotForcing);
        start_session(14);
        assert_eq!(Staking::active_era().unwrap().index, 6);
        start_session(15);
        assert_eq!(Staking::active_era().unwrap().index, 6);
    });
}

#[test]
fn cannot_transfer_staked_balance() {
    // Tests that a stash account cannot transfer funds
    ExtBuilder::default().nominate(false).build_and_execute(|| {
        // Confirm account 11 is stashed
        assert_eq!(Staking::bonded(&11), Some(10));
        // Confirm account 11 has some free balance
        assert_eq!(Balances::free_balance(11), 1000);
        // Confirm account 11 (via controller 10) is totally staked
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11).total,
            1000
        );
        // Confirm account 11 cannot transfer as a result
        assert_noop!(
            Balances::transfer(Origin::signed(11), 20, 1),
            BalancesError::<Test>::LiquidityRestrictions
        );

        // Give account 11 extra free balance
        let _ = Balances::make_free_balance_be(&11, 10000);
        // Confirm that account 11 can now transfer some balance
        assert_ok!(Balances::transfer(Origin::signed(11), 20, 1));
    });
}

#[test]
fn cannot_transfer_staked_balance_2() {
    // Tests that a stash account cannot transfer funds
    // Same test as above but with 20, and more accurate.
    // 21 has 2000 free balance but 1000 at stake
    ExtBuilder::default()
        .nominate(false)
        .fair(true)
        .build_and_execute(|| {
            // Confirm account 21 is stashed
            assert_eq!(Staking::bonded(&21), Some(20));
            // Confirm account 21 has some free balance
            assert_eq!(Balances::free_balance(21), 2000);
            // Confirm account 21 (via controller 20) is totally staked
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 21).total,
                1000
            );
            // Confirm account 21 can transfer at most 1000
            assert_noop!(
                Balances::transfer(Origin::signed(21), 20, 1001),
                BalancesError::<Test>::LiquidityRestrictions
            );
            assert_ok!(Balances::transfer(Origin::signed(21), 20, 1000));
        });
}

#[test]
fn cannot_reserve_staked_balance() {
    // Checks that a bonded account cannot reserve balance from free balance
    ExtBuilder::default().build_and_execute(|| {
        // Confirm account 11 is stashed
        assert_eq!(Staking::bonded(&11), Some(10));
        // Confirm account 11 has some free balance
        assert_eq!(Balances::free_balance(11), 1000);
        // Confirm account 11 (via controller 10) is totally staked
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11).own,
            1000
        );
        // Confirm account 11 cannot reserve as a result
        assert_noop!(
            Balances::reserve(&11, 1),
            BalancesError::<Test>::LiquidityRestrictions,
        );

        // Give account 11 extra free balance
        let _ = Balances::make_free_balance_be(&11, 10000);
        // Confirm account 11 can now reserve balance
        assert_ok!(Balances::reserve(&11, 1));
    });
}

#[test]
fn reward_destination_works() {
    // Rewards go to the correct destination as determined in Payee
    ExtBuilder::default().nominate(false).build_and_execute(|| {
        // Check that account 11 is a validator
        assert!(Session::validators().contains(&11));
        // Check the balance of the validator account
        assert_eq!(Balances::free_balance(10), 1);
        // Check the balance of the stash account
        assert_eq!(Balances::free_balance(11), 1000);
        // Check how much is at stake
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );

        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());
        <Module<Test>>::reward_by_ids(vec![(11, 1)]);

        mock::start_active_era(1);
        mock::make_all_reward_payment(0);

        // Check that RewardDestination is Staked (default)
        assert_eq!(Staking::payee(&11), RewardDestination::Staked);
        // Check that reward went to the stash account of validator
        assert_eq!(Balances::free_balance(11), 1000 + total_payout_0);
        // Check that amount at stake increased accordingly
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + total_payout_0,
                active: 1000 + total_payout_0,
                unlocking: vec![],
                claimed_rewards: vec![0],
            })
        );

        //Change RewardDestination to Stash
        <Payee<Test>>::insert(&11, RewardDestination::Stash);

        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
        <Module<Test>>::reward_by_ids(vec![(11, 1)]);

        mock::start_active_era(2);
        mock::make_all_reward_payment(1);

        // Check that RewardDestination is Stash
        assert_eq!(Staking::payee(&11), RewardDestination::Stash);
        // Check that reward went to the stash account
        assert_eq!(
            Balances::free_balance(11),
            1000 + total_payout_0 + total_payout_1
        );
        // Record this value
        let recorded_stash_balance = 1000 + total_payout_0 + total_payout_1;
        // Check that amount at stake is NOT increased
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + total_payout_0,
                active: 1000 + total_payout_0,
                unlocking: vec![],
                claimed_rewards: vec![0, 1],
            })
        );

        // Change RewardDestination to Controller
        <Payee<Test>>::insert(&11, RewardDestination::Controller);

        // Check controller balance
        assert_eq!(Balances::free_balance(10), 1);

        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_2 = current_total_payout_for_duration(reward_time_per_era());
        <Module<Test>>::reward_by_ids(vec![(11, 1)]);

        mock::start_active_era(3);
        mock::make_all_reward_payment(2);

        // Check that RewardDestination is Controller
        assert_eq!(Staking::payee(&11), RewardDestination::Controller);
        // Check that reward went to the controller account
        assert_eq!(Balances::free_balance(10), 1 + total_payout_2);
        // Check that amount at stake is NOT increased
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + total_payout_0,
                active: 1000 + total_payout_0,
                unlocking: vec![],
                claimed_rewards: vec![0, 1, 2],
            })
        );
        // Check that amount in staked account is NOT increased.
        assert_eq!(Balances::free_balance(11), recorded_stash_balance);
    });
}

#[test]
fn validator_payment_prefs_work() {
    // Test that validator preferences are correctly honored
    // Note: unstake threshold is being directly tested in slashing tests.
    // This test will focus on validator payment.
    ExtBuilder::default().build_and_execute(|| {
        let commission = Perbill::from_percent(40);
        <Validators<Test>>::insert(
            &11,
            ValidatorPrefs {
                commission: commission.clone(),
                blocked: false,
            },
        );

        // Reward controller so staked ratio doesn't change.
        <Payee<Test>>::insert(&11, RewardDestination::Controller);
        <Payee<Test>>::insert(&101, RewardDestination::Controller);

        mock::start_active_era(1);
        mock::make_all_reward_payment(0);

        let balance_era_1_10 = Balances::total_balance(&10);
        let balance_era_1_100 = Balances::total_balance(&100);

        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
        let exposure_1 = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);
        <Module<Test>>::reward_by_ids(vec![(11, 1)]);

        mock::start_active_era(2);
        mock::make_all_reward_payment(1);

        let taken_cut = commission * total_payout_1;
        let shared_cut = total_payout_1 - taken_cut;
        let reward_of_10 = shared_cut * exposure_1.own / exposure_1.total + taken_cut;
        let reward_of_100 = shared_cut * exposure_1.others[0].value / exposure_1.total;
        assert_eq_error_rate!(
            Balances::total_balance(&10),
            balance_era_1_10 + reward_of_10,
            2
        );
        assert_eq_error_rate!(
            Balances::total_balance(&100),
            balance_era_1_100 + reward_of_100,
            2
        );
    });
}

#[test]
fn bond_extra_works() {
    // Tests that extra `free_balance` in the stash can be added to stake
    // NOTE: this tests only verifies `StakingLedger` for correct updates
    // See `bond_extra_and_withdraw_unbonded_works` for more details and updates on `Exposure`.
    ExtBuilder::default().build_and_execute(|| {
        // Check that account 10 is a validator
        assert!(<Validators<Test>>::contains_key(11));
        // Check that account 10 is bonded to account 11
        assert_eq!(Staking::bonded(&11), Some(10));
        // Check how much is at stake
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );

        // Give account 11 some large free balance greater than total
        let _ = Balances::make_free_balance_be(&11, 1000000);

        // Call the bond_extra function from controller, add only 100
        assert_ok!(Staking::bond_extra(Origin::signed(11), 100));
        // There should be 100 more `total` and `active` in the ledger
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 1000 + 100,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );

        // Call the bond_extra function with a large number, should handle it
        assert_ok!(Staking::bond_extra(
            Origin::signed(11),
            Balance::max_value()
        ));
        // The full amount of the funds should now be in the total and active
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000000,
                active: 1000000,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
    });
}

#[test]
fn bond_extra_and_withdraw_unbonded_works() {
    // * Should test
    // * Given an account being bonded [and chosen as a validator](not mandatory)
    // * It can add extra funds to the bonded account.
    // * it can unbond a portion of its funds from the stash account.
    // * Once the unbonding period is done, it can actually take the funds out of the stash.
    ExtBuilder::default().nominate(false).build_and_execute(|| {
        // Set payee to controller. avoids confusion
        assert_ok!(Staking::set_payee(
            Origin::signed(10),
            RewardDestination::Controller
        ));

        // Give account 11 some large free balance greater than total
        let _ = Balances::make_free_balance_be(&11, 1000000);

        // Initial config should be correct
        assert_eq!(Staking::active_era().unwrap().index, 0);

        // check the balance of a validator accounts.
        assert_eq!(Balances::total_balance(&10), 1);

        // confirm that 10 is a normal validator and gets paid at the end of the era.
        mock::start_active_era(1);

        // Initial state of 10
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
            Exposure {
                total: 1000,
                own: 1000,
                others: vec![]
            }
        );

        // deposit the extra 100 units
        Staking::bond_extra(Origin::signed(11), 100).unwrap();

        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 1000 + 100,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
        // Exposure is a snapshot! only updated after the next era update.
        assert_ne!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
            Exposure {
                total: 1000 + 100,
                own: 1000 + 100,
                others: vec![]
            }
        );

        // trigger next era.
        mock::start_active_era(2);
        assert_eq!(Staking::active_era().unwrap().index, 2);

        // ledger should be the same.
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 1000 + 100,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
        // Exposure is now updated.
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
            Exposure {
                total: 1000 + 100,
                own: 1000 + 100,
                others: vec![]
            }
        );

        // Unbond almost all of the funds in stash.
        Staking::unbond(Origin::signed(10), 1000).unwrap();
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 100,
                unlocking: vec![UnlockChunk {
                    value: 1000,
                    era: 2 + 3
                }],
                claimed_rewards: vec![]
            }),
        );

        // Attempting to free the balances now will fail. 2 eras need to pass.
        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10), 0));
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 100,
                unlocking: vec![UnlockChunk {
                    value: 1000,
                    era: 2 + 3
                }],
                claimed_rewards: vec![]
            }),
        );

        // trigger next era.
        mock::start_active_era(3);

        // nothing yet
        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10), 0));
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 100,
                unlocking: vec![UnlockChunk {
                    value: 1000,
                    era: 2 + 3
                }],
                claimed_rewards: vec![]
            }),
        );

        // trigger next era.
        mock::start_active_era(5);

        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10), 0));
        // Now the value is free and the staking ledger is updated.
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 100,
                active: 100,
                unlocking: vec![],
                claimed_rewards: vec![]
            }),
        );
    })
}

#[test]
fn too_many_unbond_calls_should_not_work() {
    ExtBuilder::default().build_and_execute(|| {
        // locked at era 0 until 3
        for _ in 0..MAX_UNLOCKING_CHUNKS - 1 {
            assert_ok!(Staking::unbond(Origin::signed(10), 1));
        }

        mock::start_active_era(1);

        // locked at era 1 until 4
        assert_ok!(Staking::unbond(Origin::signed(10), 1));
        // can't do more.
        assert_noop!(
            Staking::unbond(Origin::signed(10), 1),
            Error::<Test>::NoMoreChunks
        );

        mock::start_active_era(3);

        assert_noop!(
            Staking::unbond(Origin::signed(10), 1),
            Error::<Test>::NoMoreChunks
        );
        // free up.
        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10), 0));

        // Can add again.
        assert_ok!(Staking::unbond(Origin::signed(10), 1));
        assert_eq!(Staking::ledger(&10).unwrap().unlocking.len(), 2);
    })
}

#[test]
fn rebond_works() {
    // * Should test
    // * Given an account being bonded [and chosen as a validator](not mandatory)
    // * it can unbond a portion of its funds from the stash account.
    // * it can re-bond a portion of the funds scheduled to unlock.
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            // Set payee to controller. avoids confusion
            assert_ok!(Staking::set_payee(
                Origin::signed(10),
                RewardDestination::Controller
            ));

            // Give account 11 some large free balance greater than total
            let _ = Balances::make_free_balance_be(&11, 1000000);

            // confirm that 10 is a normal validator and gets paid at the end of the era.
            mock::start_active_era(1);

            // Initial state of 10
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(2);
            assert_eq!(Staking::active_era().unwrap().index, 2);

            // Try to rebond some funds. We get an error since no fund is unbonded.
            assert_noop!(
                Staking::rebond(Origin::signed(10), 500),
                Error::<Test>::NoUnlockChunk,
            );

            // Unbond almost all of the funds in stash.
            Staking::unbond(Origin::signed(10), 900).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 100,
                    unlocking: vec![UnlockChunk {
                        value: 900,
                        era: 2 + 3,
                    }],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond all the funds unbonded.
            Staking::rebond(Origin::signed(10), 900).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );

            // Unbond almost all of the funds in stash.
            Staking::unbond(Origin::signed(10), 900).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 100,
                    unlocking: vec![UnlockChunk { value: 900, era: 5 }],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond part of the funds unbonded.
            Staking::rebond(Origin::signed(10), 500).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 600,
                    unlocking: vec![UnlockChunk { value: 400, era: 5 }],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond the remainder of the funds unbonded.
            Staking::rebond(Origin::signed(10), 500).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );

            // Unbond parts of the funds in stash.
            Staking::unbond(Origin::signed(10), 300).unwrap();
            Staking::unbond(Origin::signed(10), 300).unwrap();
            Staking::unbond(Origin::signed(10), 300).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 100,
                    unlocking: vec![
                        UnlockChunk { value: 300, era: 5 },
                        UnlockChunk { value: 300, era: 5 },
                        UnlockChunk { value: 300, era: 5 },
                    ],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond part of the funds unbonded.
            Staking::rebond(Origin::signed(10), 500).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 600,
                    unlocking: vec![
                        UnlockChunk { value: 300, era: 5 },
                        UnlockChunk { value: 100, era: 5 },
                    ],
                    claimed_rewards: vec![],
                })
            );
        })
}

#[test]
fn rebond_is_fifo() {
    // Rebond should proceed by reversing the most recent bond operations.
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            // Set payee to controller. avoids confusion
            assert_ok!(Staking::set_payee(
                Origin::signed(10),
                RewardDestination::Controller
            ));

            // Give account 11 some large free balance greater than total
            let _ = Balances::make_free_balance_be(&11, 1000000);

            // confirm that 10 is a normal validator and gets paid at the end of the era.
            mock::start_active_era(1);

            // Initial state of 10
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(2);

            // Unbond some of the funds in stash.
            Staking::unbond(Origin::signed(10), 400).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 600,
                    unlocking: vec![UnlockChunk {
                        value: 400,
                        era: 2 + 3
                    },],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(3);

            // Unbond more of the funds in stash.
            Staking::unbond(Origin::signed(10), 300).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 300,
                    unlocking: vec![
                        UnlockChunk {
                            value: 400,
                            era: 2 + 3
                        },
                        UnlockChunk {
                            value: 300,
                            era: 3 + 3
                        },
                    ],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(4);

            // Unbond yet more of the funds in stash.
            Staking::unbond(Origin::signed(10), 200).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 100,
                    unlocking: vec![
                        UnlockChunk {
                            value: 400,
                            era: 2 + 3
                        },
                        UnlockChunk {
                            value: 300,
                            era: 3 + 3
                        },
                        UnlockChunk {
                            value: 200,
                            era: 4 + 3
                        },
                    ],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond half of the unbonding funds.
            Staking::rebond(Origin::signed(10), 400).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 500,
                    unlocking: vec![
                        UnlockChunk {
                            value: 400,
                            era: 2 + 3
                        },
                        UnlockChunk {
                            value: 100,
                            era: 3 + 3
                        },
                    ],
                    claimed_rewards: vec![],
                })
            );
        })
}

#[test]
fn reward_to_stake_works() {
    ExtBuilder::default()
        .nominate(false)
        .fair(false)
        .build_and_execute(|| {
            // Confirm validator count is 2
            assert_eq!(Staking::validator_count(), 2);
            // Confirm account 10 and 20 are validators
            assert!(<Validators<Test>>::contains_key(&11) && <Validators<Test>>::contains_key(&21));

            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 11).total,
                1000
            );
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 21).total,
                2000
            );

            // Give the man some money.
            let _ = Balances::make_free_balance_be(&10, 1000);
            let _ = Balances::make_free_balance_be(&20, 1000);

            // Bypass logic and change current exposure
            ErasStakers::<Test>::insert(
                0,
                21,
                Exposure {
                    total: 69,
                    own: 69,
                    others: vec![],
                },
            );

            // Now lets lower account 20 stake
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 21).total,
                69
            );
            <Ledger<Test>>::insert(
                &20,
                StakingLedger {
                    stash: 21,
                    total: 69,
                    active: 69,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                },
            );

            // Compute total payout now for whole duration as other parameter won't change
            let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());
            <Module<Test>>::reward_by_ids(vec![(11, 1)]);
            <Module<Test>>::reward_by_ids(vec![(21, 1)]);

            // New era --> rewards are paid --> stakes are changed
            mock::start_active_era(1);
            mock::make_all_reward_payment(0);

            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 11).total,
                1000
            );
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 21).total,
                69
            );

            let _11_balance = Balances::free_balance(&11);
            assert_eq!(_11_balance, 1000 + total_payout_0 / 2);

            // Trigger another new era as the info are frozen before the era start.
            mock::start_active_era(2);

            // -- new infos
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 11).total,
                1000 + total_payout_0 / 2
            );
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 21).total,
                69 + total_payout_0 / 2
            );
        });
}

#[test]
fn on_free_balance_zero_stash_removes_validator() {
    // Tests that validator storage items are cleaned up when stash is empty
    // Tests that storage items are untouched when controller is empty
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            // Check the balance of the validator account
            assert_eq!(Balances::free_balance(10), 256);
            // Check the balance of the stash account
            assert_eq!(Balances::free_balance(11), 256000);
            // Check these two accounts are bonded
            assert_eq!(Staking::bonded(&11), Some(10));

            // Set some storage items which we expect to be cleaned up
            // Set payee information
            assert_ok!(Staking::set_payee(
                Origin::signed(10),
                RewardDestination::Stash
            ));

            // Check storage items that should be cleaned up
            assert!(<Ledger<Test>>::contains_key(&10));
            assert!(<Bonded<Test>>::contains_key(&11));
            assert!(<Validators<Test>>::contains_key(&11));
            assert!(<Payee<Test>>::contains_key(&11));

            // Reduce free_balance of controller to 0
            let _ = Balances::slash(&10, Balance::max_value());

            // Check the balance of the stash account has not been touched
            assert_eq!(Balances::free_balance(11), 256000);
            // Check these two accounts are still bonded
            assert_eq!(Staking::bonded(&11), Some(10));

            // Check storage items have not changed
            assert!(<Ledger<Test>>::contains_key(&10));
            assert!(<Bonded<Test>>::contains_key(&11));
            assert!(<Validators<Test>>::contains_key(&11));
            assert!(<Payee<Test>>::contains_key(&11));

            // Reduce free_balance of stash to 0
            let _ = Balances::slash(&11, Balance::max_value());
            // Check total balance of stash
            assert_eq!(Balances::total_balance(&11), 0);

            // Reap the stash
            assert_ok!(Staking::reap_stash(Origin::none(), 11, 0));

            // Check storage items do not exist
            assert!(!<Ledger<Test>>::contains_key(&10));
            assert!(!<Bonded<Test>>::contains_key(&11));
            assert!(!<Validators<Test>>::contains_key(&11));
            assert!(!<Nominators<Test>>::contains_key(&11));
            assert!(!<Payee<Test>>::contains_key(&11));
        });
}

#[test]
fn on_free_balance_zero_stash_removes_nominator() {
    // Tests that nominator storage items are cleaned up when stash is empty
    // Tests that storage items are untouched when controller is empty
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            // Make 10 a nominator
            assert_ok!(Staking::nominate(Origin::signed(10), vec![20]));
            // Check that account 10 is a nominator
            assert!(<Nominators<Test>>::contains_key(11));
            // Check the balance of the nominator account
            assert_eq!(Balances::free_balance(10), 256);
            // Check the balance of the stash account
            assert_eq!(Balances::free_balance(11), 256000);

            // Set payee information
            assert_ok!(Staking::set_payee(
                Origin::signed(10),
                RewardDestination::Stash
            ));

            // Check storage items that should be cleaned up
            assert!(<Ledger<Test>>::contains_key(&10));
            assert!(<Bonded<Test>>::contains_key(&11));
            assert!(<Nominators<Test>>::contains_key(&11));
            assert!(<Payee<Test>>::contains_key(&11));

            // Reduce free_balance of controller to 0
            let _ = Balances::slash(&10, Balance::max_value());
            // Check total balance of account 10
            assert_eq!(Balances::total_balance(&10), 0);

            // Check the balance of the stash account has not been touched
            assert_eq!(Balances::free_balance(11), 256000);
            // Check these two accounts are still bonded
            assert_eq!(Staking::bonded(&11), Some(10));

            // Check storage items have not changed
            assert!(<Ledger<Test>>::contains_key(&10));
            assert!(<Bonded<Test>>::contains_key(&11));
            assert!(<Nominators<Test>>::contains_key(&11));
            assert!(<Payee<Test>>::contains_key(&11));

            // Reduce free_balance of stash to 0
            let _ = Balances::slash(&11, Balance::max_value());
            // Check total balance of stash
            assert_eq!(Balances::total_balance(&11), 0);

            // Reap the stash
            assert_ok!(Staking::reap_stash(Origin::none(), 11, 0));

            // Check storage items do not exist
            assert!(!<Ledger<Test>>::contains_key(&10));
            assert!(!<Bonded<Test>>::contains_key(&11));
            assert!(!<Validators<Test>>::contains_key(&11));
            assert!(!<Nominators<Test>>::contains_key(&11));
            assert!(!<Payee<Test>>::contains_key(&11));
        });
}

#[test]
fn switching_roles() {
    // Test that it should be possible to switch between roles (nominator, validator, idle) with minimal overhead.
    ExtBuilder::default().nominate(false).build_and_execute(|| {
        // Reset reward destination
        for i in &[10, 20] {
            assert_ok!(Staking::set_payee(
                Origin::signed(*i),
                RewardDestination::Controller
            ));
        }

        assert_eq_uvec!(validator_controllers(), vec![20, 10]);

        // put some money in account that we'll use.
        for i in 1..7 {
            let _ = Balances::deposit_creating(&i, 5000);
        }

        provide_did_to_user(1);
        add_secondary_key(1, 2);
        // add 2 nominators
        assert_ok!(Staking::bond(
            Origin::signed(1),
            2,
            2000,
            RewardDestination::Controller
        ));
        assert_ok!(Staking::nominate(Origin::signed(2), vec![11, 5]));

        provide_did_to_user(3);
        add_secondary_key(3, 4);
        assert_ok!(Staking::bond(
            Origin::signed(3),
            4,
            500,
            RewardDestination::Controller
        ));
        assert_ok!(Staking::nominate(Origin::signed(4), vec![21, 1]));

        provide_did_to_user(5);
        add_secondary_key(5, 6);
        // add a new validator candidate
        assert_ok!(Staking::bond(
            Origin::signed(5),
            6,
            1000,
            RewardDestination::Controller
        ));
        assert_add_permissioned_validator!(&5);
        assert_ok!(Staking::validate(
            Origin::signed(6),
            ValidatorPrefs::default()
        ));
        assert_ok!(Session::set_keys(
            Origin::signed(6),
            SessionKeys { other: 6.into() },
            vec![]
        ));

        mock::start_active_era(1);

        // with current nominators 10 and 5 have the most stake
        assert_eq_uvec!(validator_controllers(), vec![6, 10]);

        assert_add_permissioned_validator!(&1);
        // 2 decides to be a validator. Consequences:
        assert_ok!(Staking::validate(
            Origin::signed(2),
            ValidatorPrefs::default()
        ));
        assert_ok!(Session::set_keys(
            Origin::signed(2),
            SessionKeys { other: 2.into() },
            vec![]
        ));
        // new stakes:
        // 10: 1000 self vote
        // 20: 1000 self vote + 250 vote
        // 6 : 1000 self vote
        // 2 : 2000 self vote + 250 vote.
        // Winners: 20 and 2

        mock::start_active_era(2);

        assert_eq_uvec!(validator_controllers(), vec![2, 20]);
    });
}

#[test]
fn wrong_vote_is_null() {
    ExtBuilder::default()
        .nominate(false)
        .validator_pool(true)
        .build_and_execute(|| {
            assert_eq_uvec!(validator_controllers(), vec![40, 30]);

            // put some money in account that we'll use.
            for i in 1..3 {
                let _ = Balances::deposit_creating(&i, 5000);
            }

            provide_did_to_user(1);
            add_secondary_key(1, 2);
            // add 1 nominators
            assert_ok!(Staking::bond(
                Origin::signed(1),
                2,
                2000,
                RewardDestination::default()
            ));
            assert_ok!(Staking::nominate(
                Origin::signed(2),
                vec![
                    11, 21, // good votes
                    1, 2, 15, 1000, 25 // crap votes. No effect.
                ]
            ));

            // new block
            mock::start_active_era(1);

            assert_eq_uvec!(validator_controllers(), vec![20, 10]);
        });
}

#[test]
fn bond_with_no_staked_value() {
    // Behavior when someone bonds with no staked value.
    // Particularly when she votes and the candidate is elected.
    ExtBuilder::default()
        .validator_count(3)
        .existential_deposit(5)
        .nominate(false)
        .minimum_validator_count(1)
        .build()
        .execute_with(|| {
            provide_did_to_user(1);
            add_secondary_key(1, 2);

            // Polymesh-note - Below check will fail as in code we have exestential deposit = 0 (Hard coded)
            // Can't bond with 1
            // assert_noop!(
            //     Staking::bond(Origin::signed(1), 2, 1, RewardDestination::Controller),
            //     Error::<Test>::InsufficientValue,
            // );
            // bonded with absolute minimum value possible.
            assert_ok!(Staking::bond(
                Origin::signed(1),
                2,
                MinimumBond::get(),
                RewardDestination::Controller
            ));
            assert_eq!(Balances::locks(&1)[0].amount, MinimumBond::get());

            // Polymesh -note -> Below check wouldn't be succeded as exesential deposit hard coded to 0
            // unbonding even 1 will cause all to be unbonded.
            assert_ok!(Staking::unbond(Origin::signed(2), 1));
            assert_eq!(
                Staking::ledger(2),
                Some(StakingLedger {
                    stash: 1,
                    active: MinimumBond::get() - 1,
                    total: MinimumBond::get(),
                    unlocking: vec![UnlockChunk { value: 1, era: 3 }],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(1);
            mock::start_active_era(2);

            // not yet removed.
            assert_ok!(Staking::withdraw_unbonded(Origin::signed(2), 0));
            assert!(Staking::ledger(2).is_some());
            assert_eq!(Balances::locks(&1)[0].amount, MinimumBond::get());

            mock::start_active_era(3);

            // poof. Account 1 is not fully removed from the staking system.
            assert_ok!(Staking::withdraw_unbonded(Origin::signed(2), 0));
            assert!(Staking::ledger(2).is_some());
            assert_eq!(Balances::locks(&1).len(), 1);
        });
}

#[test]
fn bond_with_little_staked_value_bounded() {
    // Behavior when someone bonds with little staked value.
    // Particularly when she votes and the candidate is elected.
    ExtBuilder::default()
        .validator_count(3)
        .nominate(false)
        .minimum_validator_count(1)
        .build()
        .execute_with(|| {
            // setup
            assert_ok!(Staking::chill(Origin::signed(30)));
            assert_ok!(Staking::set_payee(
                Origin::signed(10),
                RewardDestination::Controller
            ));
            let init_balance_2 = Balances::free_balance(&2);
            let init_balance_10 = Balances::free_balance(&10);

            provide_did_to_user(1);
            // Stingy validator.
            assert_ok!(Staking::bond(
                Origin::signed(1),
                2,
                MinimumBond::get(),
                RewardDestination::Controller
            ));
            add_secondary_key(1, 2);
            assert_add_permissioned_validator!(&1);
            assert_ok!(Staking::validate(
                Origin::signed(2),
                ValidatorPrefs::default()
            ));
            assert_ok!(Session::set_keys(
                Origin::signed(2),
                SessionKeys { other: 2.into() },
                vec![]
            ));

            // 1 era worth of reward. BUT, we set the timestamp after on_initialize, so outdated by
            // one block.
            let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());

            reward_all_elected();
            mock::start_active_era(1);
            mock::make_all_reward_payment(0);

            // 2 is elected.
            assert_eq_uvec!(validator_controllers(), vec![20, 10, 2]);
            // And has minimal stake
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 2).total,
                0
            );

            // Old ones are rewarded.
            assert_eq_error_rate!(
                Balances::free_balance(10),
                init_balance_10 + total_payout_0 / 3,
                1
            );
            // no rewards paid to 2. This was initial election.
            assert_eq!(Balances::free_balance(2), init_balance_2);

            // reward era 2
            let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
            reward_all_elected();
            mock::start_active_era(2);
            mock::make_all_reward_payment(1);

            assert_eq_uvec!(validator_controllers(), vec![20, 10, 2]);
            assert_eq!(
                Staking::eras_stakers(Staking::active_era().unwrap().index, 2).total,
                0
            );
            // 2 is now rewarded.
            assert_eq_error_rate!(
                Balances::free_balance(2),
                init_balance_2 + total_payout_1 / 3,
                1
            );
            assert_eq_error_rate!(
                Balances::free_balance(&10),
                init_balance_10 + total_payout_0 / 3 + total_payout_1 / 3,
                2,
            );
        });
}

#[test]
fn new_era_elects_correct_number_of_validators() {
    ExtBuilder::default()
        .nominate(true)
        .validator_pool(true)
        .fair(true)
        .validator_count(1)
        .build()
        .execute_with(|| {
            assert_eq!(Staking::validator_count(), 1);
            assert_eq!(validator_controllers().len(), 1);

            Session::on_initialize(System::block_number());

            assert_eq!(validator_controllers().len(), 1);
        })
}

#[test]
fn phragmen_should_not_overflow() {
    ExtBuilder::default().nominate(false).build_and_execute(|| {
        // This is the maximum value that we can have as the outcome of CurrencyToVote.
        type Votes = u64;

        let _ = Staking::chill(Origin::signed(10));
        let _ = Staking::chill(Origin::signed(20));

        bond_validator(3, 2, Votes::max_value() as Balance);
        bond_validator(5, 4, Votes::max_value() as Balance);

        provide_did_to_user(7);
        add_secondary_key(7, 6);

        provide_did_to_user(9);
        add_secondary_key(9, 8);

        bond_nominator(7, 6, Votes::max_value() as Balance, vec![3, 5]);
        bond_nominator(9, 8, Votes::max_value() as Balance, vec![3, 5]);

        mock::start_active_era(1);

        assert_eq_uvec!(validator_controllers(), vec![4, 2]);

        // We can safely convert back to values within [u64, u128].
        assert!(Staking::eras_stakers(active_era(), 3).total > Votes::max_value() as Balance);
        assert!(Staking::eras_stakers(active_era(), 5).total > Votes::max_value() as Balance);
    })
}

#[test]
fn reward_validator_slashing_validator_does_not_overflow() {
    ExtBuilder::default().build_and_execute(|| {
        let stake = u64::max_value() as Balance * 2;
        let reward_slash = u64::max_value() as Balance * 2;

        // Assert multiplication overflows in balance arithmetic.
        assert!(stake.checked_mul(reward_slash).is_none());

        // Set staker
        let _ = Balances::make_free_balance_be(&11, stake);

        let exposure = Exposure::<AccountId, Balance> {
            total: stake,
            own: stake,
            others: vec![],
        };
        let reward = EraRewardPoints::<AccountId> {
            total: 1,
            individual: vec![(11, 1)].into_iter().collect(),
        };

        // Check reward
        ErasRewardPoints::<Test>::insert(0, reward);
        ErasStakers::<Test>::insert(0, 11, &exposure);
        ErasStakersClipped::<Test>::insert(0, 11, exposure);
        ErasValidatorReward::<Test>::insert(0, stake);
        assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 0));
        assert_eq!(Balances::total_balance(&11), stake * 2);

        // Set staker
        let _ = Balances::make_free_balance_be(&11, stake);
        let _ = Balances::make_free_balance_be(&2, stake);

        provide_did_to_user(2);
        add_secondary_key(2, 20000);
        // only slashes out of bonded stake are applied. without this line,
        // it is 0.
        Staking::bond(
            Origin::signed(2),
            20000,
            stake - 1,
            RewardDestination::default(),
        )
        .unwrap();
        // Override exposure of 11
        ErasStakers::<Test>::insert(
            0,
            11,
            Exposure {
                total: stake,
                own: 1,
                others: vec![IndividualExposure {
                    who: 2,
                    value: stake - 1,
                }],
            },
        );

        // Check slashing
        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(100)],
        );

        assert_eq!(Balances::total_balance(&11), stake - 1);
        // Polymesh - note -> No slashing for the nominators
        //assert_eq!(Balances::total_balance(&2), 1);
    })
}

#[test]
fn reward_from_authorship_event_handler_works() {
    ExtBuilder::default().build_and_execute(|| {
        use pallet_authorship::EventHandler;

        assert_eq!(<pallet_authorship::Pallet<Test>>::author(), Some(11));

        <Module<Test>>::note_author(11);

        // Not mandatory but must be coherent with rewards
        assert_eq_uvec!(Session::validators(), vec![11, 21]);

        // 11 is rewarded as a block producer
        assert_eq!(
            ErasRewardPoints::<Test>::get(Staking::active_era().unwrap().index),
            EraRewardPoints {
                individual: vec![(11, 20)].into_iter().collect(),
                total: 20,
            },
        );
    })
}

#[test]
fn add_reward_points_fns_works() {
    ExtBuilder::default().build_and_execute(|| {
        // Not mandatory but must be coherent with rewards
        assert_eq!(Session::validators(), vec![21, 11]);

        <Module<Test>>::reward_by_ids(vec![(21, 1), (11, 1), (11, 1)]);

        <Module<Test>>::reward_by_ids(vec![(21, 1), (11, 1), (11, 1)]);

        assert_eq!(
            ErasRewardPoints::<Test>::get(Staking::active_era().unwrap().index),
            EraRewardPoints {
                individual: vec![(11, 4), (21, 2)].into_iter().collect(),
                total: 6,
            },
        );
    })
}

#[test]
fn unbonded_balance_is_not_slashable() {
    ExtBuilder::default().build_and_execute(|| {
        // total amount staked is slashable.
        assert_eq!(Staking::slashable_balance_of(&11), 1000);

        assert_ok!(Staking::unbond(Origin::signed(10), 800));

        // only the active portion.
        assert_eq!(Staking::slashable_balance_of(&11), 200);
    })
}

#[test]
fn era_is_always_same_length() {
    // This ensures that the sessions is always of the same length if there is no forcing no
    // session changes.
    ExtBuilder::default().build_and_execute(|| {
        let session_per_era = <SessionsPerEra as Get<SessionIndex>>::get();

        mock::start_active_era(1);
        assert_eq!(
            Staking::eras_start_session_index(current_era()).unwrap(),
            session_per_era
        );

        mock::start_active_era(2);
        assert_eq!(
            Staking::eras_start_session_index(current_era()).unwrap(),
            session_per_era * 2u32
        );

        let session = Session::current_index();
        ForceEra::put(Forcing::ForceNew);
        advance_session();
        advance_session();
        assert_eq!(current_era(), 3);
        assert_eq!(
            Staking::eras_start_session_index(current_era()).unwrap(),
            session + 2
        );

        mock::start_active_era(4);
        assert_eq!(
            Staking::eras_start_session_index(current_era()).unwrap(),
            session + 2u32 + session_per_era
        );
    });
}

#[test]
fn offence_forces_new_era() {
    ExtBuilder::default().build_and_execute(|| {
        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(5)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceNew);
    });
}

#[test]
fn offence_ensures_new_era_without_clobbering() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Staking::force_new_era_always(Origin::root()));
        assert_eq!(Staking::force_era(), Forcing::ForceAlways);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(5)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceAlways);
    });
}

#[test]
fn offence_deselects_validator_even_when_slash_is_zero() {
    ExtBuilder::default().build_and_execute(|| {
        assert!(Session::validators().contains(&11));
        assert!(<Validators<Test>>::contains_key(11));

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceNew);
        assert!(!<Validators<Test>>::contains_key(11));

        mock::start_active_era(1);

        assert!(!Session::validators().contains(&11));
        assert!(!<Validators<Test>>::contains_key(11));
    });
}

#[test]
fn slashing_performed_according_exposure() {
    // This test checks that slashing is performed according the exposure (or more precisely,
    // historical exposure), not the current balance.
    ExtBuilder::default().build_and_execute(|| {
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11).own,
            1000
        );

        // Handle an offence with a historical exposure.
        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Exposure {
                        total: 500,
                        own: 500,
                        others: vec![],
                    },
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(50)],
        );

        // The stash account should be slashed for 250 (50% of 500).
        assert_eq!(Balances::free_balance(11), 1000 - 250);
    });
}

#[test]
fn slash_in_old_span_does_not_deselect() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);

        assert!(<Validators<Test>>::contains_key(11));
        assert!(Session::validators().contains(&11));

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceNew);
        assert!(!<Validators<Test>>::contains_key(11));

        mock::start_active_era(2);

        Staking::validate(Origin::signed(10), Default::default()).unwrap();
        assert_eq!(Staking::force_era(), Forcing::NotForcing);
        assert!(<Validators<Test>>::contains_key(11));
        assert!(!Session::validators().contains(&11));

        mock::start_active_era(3);

        // this staker is in a new slashing span now, having re-registered after
        // their prior slash.

        on_offence_in_era(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
            1,
        );

        // the validator doesn't get chilled again
        assert!(<Validators<Test>>::iter().any(|(stash, _)| stash == 11));

        // but we are still forcing a new era
        assert_eq!(Staking::force_era(), Forcing::ForceNew);

        on_offence_in_era(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            // NOTE: A 100% slash here would clean up the account, causing de-registration.
            &[Perbill::from_percent(95)],
            1,
        );

        // the validator doesn't get chilled again
        assert!(<Validators<Test>>::iter().any(|(stash, _)| stash == 11));

        // but it's disabled
        assert!(is_disabled(10));
        // and we are still forcing a new era
        assert_eq!(Staking::force_era(), Forcing::ForceNew);
    });
}

#[test]
fn reporters_receive_their_slice() {
    // This test verifies that the reporters of the offence receive their slice from the slashed
    // amount.
    ExtBuilder::default().build_and_execute(|| {
        // The reporters' reward is calculated from the validator's exposure only.
        let initial_balance = 1000;

        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11).own,
            initial_balance
        );

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![1, 2],
            }],
            &[Perbill::from_percent(50)],
        );

        // F1 * (reward_proportion * slash - 0)
        // 50% * (10% * initial_balance / 2)
        let reward = (initial_balance / 20) / 2;
        let reward_each = reward / 2; // split into two pieces.
        assert_eq!(Balances::free_balance(1), 10 + reward_each);
        assert_eq!(Balances::free_balance(2), 20 + reward_each);
    });
}

#[test]
fn subsequent_reports_in_same_span_pay_out_less() {
    // This test verifies that the reporters of the offence receive their slice from the slashed
    // amount, but less and less if they submit multiple reports in one span.
    ExtBuilder::default().build_and_execute(|| {
        // The reporters' reward is calculated from the validator's exposure only not the total.
        let initial_balance = 1000;

        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11).own,
            initial_balance
        );

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![1],
            }],
            &[Perbill::from_percent(20)],
        );

        // F1 * (reward_proportion * slash - 0)
        // 50% * (10% * initial_balance * 20%)
        let reward = (initial_balance / 5) / 20;
        assert_eq!(Balances::free_balance(1), 10 + reward);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![1],
            }],
            &[Perbill::from_percent(50)],
        );

        let prior_payout = reward;

        // F1 * (reward_proportion * slash - prior_payout)
        // 50% * (10% * (initial_balance / 2) - prior_payout)
        let reward = ((initial_balance / 20) - prior_payout) / 2;
        assert_eq!(Balances::free_balance(1), 10 + prior_payout + reward);
    });
}

#[test]
fn invulnerables_are_not_slashed() {
    // For invulnerable validators no slashing is performed.
    ExtBuilder::default()
        .invulnerables(vec![11])
        .build_and_execute(|| {
            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(21), 2000);

            let exposure = Staking::eras_stakers(Staking::active_era().unwrap().index, 21);
            let initial_balance = Staking::slashable_balance_of(&21);

            let _nominator_balances: Vec<_> = exposure
                .others
                .iter()
                .map(|o| Balances::free_balance(&o.who))
                .collect();

            on_offence_now(
                &[
                    OffenceDetails {
                        offender: (
                            11,
                            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                        ),
                        reporters: vec![],
                    },
                    OffenceDetails {
                        offender: (
                            21,
                            Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                        ),
                        reporters: vec![],
                    },
                ],
                &[Perbill::from_percent(50), Perbill::from_percent(20)],
            );

            // The validator 11 hasn't been slashed, but 21 has been.
            assert_eq!(Balances::free_balance(11), 1000);
            // 2000 - (0.2 * initial_balance)
            assert_eq!(
                Balances::free_balance(21),
                2000 - (2 * initial_balance / 10)
            );

            // Polymesh - note -> Nominators are not gonna slashed
            // ensure that nominators were slashed as well.
            // for (initial_balance, other) in nominator_balances.into_iter().zip(exposure.others) {
            //     assert_eq!(
            //         Balances::free_balance(&other.who),
            //         initial_balance - (2 * other.value / 10),
            //     );
            // }
        });
}

#[test]
fn dont_slash_if_fraction_is_zero() {
    // Don't slash if the fraction is zero.
    ExtBuilder::default().build_and_execute(|| {
        assert_eq!(Balances::free_balance(11), 1000);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );

        // The validator hasn't been slashed. The new era is not forced.
        assert_eq!(Balances::free_balance(11), 1000);
        assert_eq!(Staking::force_era(), Forcing::ForceNew);
    });
}

#[test]
fn only_slash_for_max_in_era() {
    // multiple slashes within one era are only applied if it is more than any previous slash in the
    // same era.
    ExtBuilder::default().build_and_execute(|| {
        assert_eq!(Balances::free_balance(11), 1000);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(50)],
        );

        // The validator has been slashed and has been force-chilled.
        assert_eq!(Balances::free_balance(11), 500);
        assert_eq!(Staking::force_era(), Forcing::ForceNew);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(25)],
        );

        // The validator has not been slashed additionally.
        assert_eq!(Balances::free_balance(11), 500);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(60)],
        );

        // The validator got slashed 10% more.
        assert_eq!(Balances::free_balance(11), 400);
    })
}

#[test]
fn garbage_collection_after_slashing() {
    // ensures that `SlashingSpans` and `SpanSlash` of an account is removed after reaping.
    ExtBuilder::default()
        .existential_deposit(2)
        .build_and_execute(|| {
            assert_eq!(Balances::free_balance(11), 256_000);

            on_offence_now(
                &[OffenceDetails {
                    offender: (
                        11,
                        Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                    ),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            assert_eq!(Balances::free_balance(11), 256_000 - 25_600);
            assert!(SlashingSpans::<Test>::get(&11).is_some());
            assert_eq!(SpanSlash::<Test>::get(&(11, 0)).amount_slashed(), &25_600);

            on_offence_now(
                &[OffenceDetails {
                    offender: (
                        11,
                        Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                    ),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(100)],
            );

            // validator and nominator slash in era are garbage-collected by era change,
            // so we don't test those here.

            assert_eq!(Balances::free_balance(11), 0);
            assert_eq!(Balances::total_balance(&11), 0);

            let slashing_spans = SlashingSpans::<Test>::get(&11).unwrap();
            assert_eq!(slashing_spans.iter().count(), 2);

            // reap_stash respects num_slashing_spans so that weight is accurate
            assert_noop!(
                Staking::reap_stash(Origin::none(), 11, 0),
                Error::<Test>::IncorrectSlashingSpans
            );
            assert_ok!(Staking::reap_stash(Origin::none(), 11, 2));

            assert!(SlashingSpans::<Test>::get(&11).is_none());
            assert_eq!(SpanSlash::<Test>::get(&(11, 0)).amount_slashed(), &0);
        })
}

#[test]
fn garbage_collection_on_window_pruning() {
    // ensures that `ValidatorSlashInEra` and `NominatorSlashInEra` are cleared after
    // `BondingDuration`.
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);

        assert_eq!(Balances::free_balance(11), 1000);
        let now = Staking::active_era().unwrap().index;

        let exposure = Staking::eras_stakers(now, 11);
        assert_eq!(Balances::free_balance(101), 2000);
        let _nominated_value = exposure.others.iter().find(|o| o.who == 101).unwrap().value;

        on_offence_now(
            &[OffenceDetails {
                offender: (11, Staking::eras_stakers(now, 11)),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
        );

        assert_eq!(Balances::free_balance(11), 900);
        // Polymesh - note -> No slashing for the nominators
        // assert_eq!(Balances::free_balance(101), 2000 - (nominated_value / 10));
        assert_eq!(Balances::free_balance(101), 2000);

        assert!(ValidatorSlashInEra::<Test>::get(&now, &11).is_some());
        // Storage will not be updates as nominator didn't get slashed so it will be none.
        assert!(NominatorSlashInEra::<Test>::get(&now, &101).is_none());

        // + 1 because we have to exit the bonding window.
        for era in (0..(BondingDuration::get() + 1)).map(|offset| offset + now + 1) {
            assert!(ValidatorSlashInEra::<Test>::get(&now, &11).is_some());
            assert!(NominatorSlashInEra::<Test>::get(&now, &101).is_none());

            mock::start_active_era(era);
        }

        assert!(ValidatorSlashInEra::<Test>::get(&now, &11).is_none());
        assert!(NominatorSlashInEra::<Test>::get(&now, &101).is_none());
    })
}

#[test]
#[ignore] // Ignored because nominator will no more slashed and no storage will be updated.
fn slashing_nominators_by_span_max() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);
        mock::start_active_era(2);
        mock::start_active_era(3);

        assert_eq!(Balances::free_balance(11), 1000);
        assert_eq!(Balances::free_balance(21), 2000);
        assert_eq!(Balances::free_balance(101), 2000);
        assert_eq!(Staking::slashable_balance_of(&21), 1000);

        let exposure_11 = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);
        let exposure_21 = Staking::eras_stakers(Staking::active_era().unwrap().index, 21);
        let nominated_value_11 = exposure_11
            .others
            .iter()
            .find(|o| o.who == 101)
            .unwrap()
            .value;
        let nominated_value_21 = exposure_21
            .others
            .iter()
            .find(|o| o.who == 101)
            .unwrap()
            .value;

        on_offence_in_era(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
            2,
        );

        assert_eq!(Balances::free_balance(11), 900);

        let slash_1_amount = Perbill::from_percent(10) * nominated_value_11;
        // Polymesh-note -> No slashing of nominator
        //assert_eq!(Balances::free_balance(101), 2000 - slash_1_amount);
        assert_eq!(Balances::free_balance(101), 2000);

        let expected_spans = vec![
            slashing::SlashingSpan {
                index: 1,
                start: 4,
                length: None,
            },
            slashing::SlashingSpan {
                index: 0,
                start: 0,
                length: Some(4),
            },
        ];

        let get_span = |account| SlashingSpans::<Test>::get(&account).unwrap();

        assert_eq!(get_span(11).iter().collect::<Vec<_>>(), expected_spans,);

        assert_eq!(get_span(101).iter().collect::<Vec<_>>(), expected_spans,);

        // second slash: higher era, higher value, same span.
        on_offence_in_era(
            &[OffenceDetails {
                offender: (
                    21,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(30)],
            3,
        );

        // 11 was not further slashed, but 21 and 101 were.
        assert_eq!(Balances::free_balance(11), 900);
        assert_eq!(Balances::free_balance(21), 1700);

        let slash_2_amount = Perbill::from_percent(30) * nominated_value_21;
        assert!(slash_2_amount > slash_1_amount);

        // Polymesh-note -> No slashing for nominators
        // only the maximum slash in a single span is taken.
        //assert_eq!(Balances::free_balance(101), 2000 - slash_2_amount);
        assert_eq!(Balances::free_balance(101), 2000);

        // third slash: in same era and on same validator as first, higher
        // in-era value, but lower slash value than slash 2.
        on_offence_in_era(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(20)],
            2,
        );

        // 11 was further slashed, but 21 and 101 were not.
        assert_eq!(Balances::free_balance(11), 800);
        assert_eq!(Balances::free_balance(21), 1700);

        let slash_3_amount = Perbill::from_percent(20) * nominated_value_21;
        assert!(slash_3_amount < slash_2_amount);
        assert!(slash_3_amount > slash_1_amount);

        // Polymesh-note -> No slashing for nominators
        // only the maximum slash in a single span is taken.
        // assert_eq!(Balances::free_balance(101), 2000 - slash_2_amount);
        assert_eq!(Balances::free_balance(101), 2000);
    });
}

#[test]
fn slashes_are_summed_across_spans() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);
        mock::start_active_era(2);
        mock::start_active_era(3);

        assert_eq!(Balances::free_balance(21), 2000);
        assert_eq!(Staking::slashable_balance_of(&21), 1000);

        let get_span = |account| SlashingSpans::<Test>::get(&account).unwrap();

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    21,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
        );

        let expected_spans = vec![
            slashing::SlashingSpan {
                index: 1,
                start: 4,
                length: None,
            },
            slashing::SlashingSpan {
                index: 0,
                start: 0,
                length: Some(4),
            },
        ];

        assert_eq!(get_span(21).iter().collect::<Vec<_>>(), expected_spans);
        assert_eq!(Balances::free_balance(21), 1900);

        // 21 has been force-chilled. re-signal intent to validate.
        Staking::validate(Origin::signed(20), Default::default()).unwrap();

        mock::start_active_era(4);

        assert_eq!(Staking::slashable_balance_of(&21), 900);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    21,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
        );

        let expected_spans = vec![
            slashing::SlashingSpan {
                index: 2,
                start: 5,
                length: None,
            },
            slashing::SlashingSpan {
                index: 1,
                start: 4,
                length: Some(1),
            },
            slashing::SlashingSpan {
                index: 0,
                start: 0,
                length: Some(4),
            },
        ];

        assert_eq!(get_span(21).iter().collect::<Vec<_>>(), expected_spans);
        assert_eq!(Balances::free_balance(21), 1810);
    });
}

#[test]
fn deferred_slashes_are_deferred() {
    ExtBuilder::default()
        .slash_defer_duration(2)
        .build_and_execute(|| {
            mock::start_active_era(1);

            assert_eq!(Balances::free_balance(11), 1000);

            let exposure = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);
            assert_eq!(Balances::free_balance(101), 2000);
            let _nominated_value = exposure.others.iter().find(|o| o.who == 101).unwrap().value;

            on_offence_now(
                &[OffenceDetails {
                    offender: (
                        11,
                        Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                    ),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(2);

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(3);

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            // at the start of era 4, slashes from era 1 are processed,
            // after being deferred for at least 2 full eras.
            mock::start_active_era(4);

            assert_eq!(Balances::free_balance(11), 900);
            // Polymesh-note - In polymesh nominators slashing is switched off
            //assert_eq!(Balances::free_balance(101), 2000 - (nominated_value / 10));
            // so instead of above it gives below result
            assert_eq!(Balances::free_balance(101), 2000);
        })
}

#[test]
fn remove_deferred() {
    ExtBuilder::default()
        .slash_defer_duration(2)
        .build_and_execute(|| {
            mock::start_active_era(1);

            assert_eq!(Balances::free_balance(11), 1000);

            let exposure = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);
            assert_eq!(Balances::free_balance(101), 2000);
            let nominated_value = exposure.others.iter().find(|o| o.who == 101).unwrap().value;

            on_offence_now(
                &[OffenceDetails {
                    offender: (11, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(2);

            on_offence_in_era(
                &[OffenceDetails {
                    offender: (11, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(15)],
                1,
            );

            // fails if empty
            assert_noop!(
                Staking::cancel_deferred_slash(Origin::root(), 1, vec![]),
                Error::<Test>::EmptyTargets
            );

            assert_ok!(Staking::cancel_deferred_slash(Origin::root(), 1, vec![0]));

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(3);

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            // at the start of era 4, slashes from era 1 are processed,
            // after being deferred for at least 2 full eras.
            mock::start_active_era(4);

            // the first slash for 10% was cancelled, so no effect.
            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(5);

            let slash_10 = Perbill::from_percent(10);
            let slash_15 = Perbill::from_percent(15);
            let initial_slash = slash_10 * nominated_value;

            let total_slash = slash_15 * nominated_value;
            let _actual_slash = total_slash - initial_slash;

            // Polymesh-note -> No slashing for the nominators
            // 5% slash (15 - 10) processed now.
            assert_eq!(Balances::free_balance(11), 950);
            assert_eq!(Balances::free_balance(101), 2000);
        })
}

#[test]
fn remove_multi_deferred() {
    ExtBuilder::default()
        .slash_defer_duration(2)
        .build_and_execute(|| {
            mock::start_active_era(1);

            assert_eq!(Balances::free_balance(11), 1000);

            let exposure = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);
            assert_eq!(Balances::free_balance(101), 2000);

            on_offence_now(
                &[OffenceDetails {
                    offender: (11, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (
                        21,
                        Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                    ),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (11, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(25)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (42, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(25)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (69, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(25)],
            );

            assert_eq!(UnappliedSlashes::<Test>::get(&1).len(), 5);

            // fails if list is not sorted
            assert_noop!(
                Staking::cancel_deferred_slash(Origin::root(), 1, vec![2, 0, 4]),
                Error::<Test>::NotSortedAndUnique
            );
            // fails if list is not unique
            assert_noop!(
                Staking::cancel_deferred_slash(Origin::root(), 1, vec![0, 2, 2]),
                Error::<Test>::NotSortedAndUnique
            );
            // fails if bad index
            assert_noop!(
                Staking::cancel_deferred_slash(Origin::root(), 1, vec![1, 2, 3, 4, 5]),
                Error::<Test>::InvalidSlashIndex
            );

            assert_ok!(Staking::cancel_deferred_slash(
                Origin::root(),
                1,
                vec![0, 2, 4]
            ));

            let slashes = UnappliedSlashes::<Test>::get(&1);
            assert_eq!(slashes.len(), 2);
            assert_eq!(slashes[0].validator, 21);
            assert_eq!(slashes[1].validator, 42);
        })
}

mod offchain_phragmen {
    use super::*;
    use codec::Encode;
    use frame_support::traits::OffchainWorker;
    use frame_support::{
        assert_err_with_weight, assert_noop, assert_ok, dispatch::DispatchResultWithPostInfo,
    };
    use pallet_staking::{offchain_election, OffchainAccuracy};
    use parking_lot::RwLock;
    use sp_core::offchain::{
        testing::{PoolState, TestOffchainExt, TestTransactionPoolExt},
        OffchainDbExt, OffchainWorkerExt, TransactionPoolExt,
    };
    use sp_io::TestExternalities;
    use sp_npos_elections::StakedAssignment;
    use sp_runtime::transaction_validity::TransactionSource;
    use std::sync::Arc;

    fn percent(x: u16) -> OffchainAccuracy {
        OffchainAccuracy::from_percent(x)
    }

    /// setup a new set of validators and nominator storage items independent of the parent mock
    /// file. This produces a edge graph that can be reduced.
    pub fn build_offchain_phragmen_test_ext() {
        for i in (10..=40).step_by(10) {
            // Note: we respect the convention of the mock (10, 11 pairs etc.) since these accounts
            // have corresponding keys in session which makes everything more ergonomic and
            // realistic.
            bond_validator(i + 1, i, 100);
        }

        let mut voter = 1;
        bond_nominator_cdd(voter, 1000 + voter, 100, vec![11]);
        voter = 2;
        bond_nominator_cdd(voter, 1000 + voter, 100, vec![11, 11]);
        voter = 3;
        bond_nominator_cdd(voter, 1000 + voter, 100, vec![21, 41]);
        voter = 4;
        bond_nominator_cdd(voter, 1000 + voter, 100, vec![21, 31, 41]);
        voter = 5;
        bond_nominator_cdd(voter, 1000 + voter, 100, vec![21, 31, 41]);
    }

    /// convert an externalities to one that can handle offchain worker tests.
    fn offchainify(ext: &mut TestExternalities, iterations: u32) -> Arc<RwLock<PoolState>> {
        let (offchain, offchain_state) = TestOffchainExt::new();
        let (pool, pool_state) = TestTransactionPoolExt::new();

        let mut seed = [0_u8; 32];
        seed[0..4].copy_from_slice(&iterations.to_le_bytes());
        offchain_state.write().seed = seed;

        ext.register_extension(OffchainDbExt::new(offchain.clone()));
        ext.register_extension(OffchainWorkerExt::new(offchain));
        ext.register_extension(TransactionPoolExt::new(pool));

        pool_state
    }

    fn election_size() -> ElectionSize {
        ElectionSize {
            validators: Staking::snapshot_validators().unwrap().len() as ValidatorIndex,
            nominators: Staking::snapshot_nominators().unwrap().len() as NominatorIndex,
        }
    }

    fn submit_solution(
        origin: Origin,
        winners: Vec<ValidatorIndex>,
        compact: CompactAssignments,
        score: ElectionScore,
    ) -> DispatchResultWithPostInfo {
        Staking::submit_election_solution(
            origin,
            winners,
            compact,
            score,
            current_era(),
            election_size(),
        )
    }

    #[test]
    fn is_current_session_final_works() {
        ExtBuilder::default()
            .session_per_era(3)
            .build()
            .execute_with(|| {
                mock::start_active_era(1);
                assert_eq!(Session::current_index(), 3);
                assert_eq!(Staking::current_era(), Some(1));
                assert_eq!(Staking::is_current_session_final(), false);

                start_session(4);
                assert_eq!(Session::current_index(), 4);
                assert_eq!(Staking::current_era(), Some(1));
                assert_eq!(Staking::is_current_session_final(), true);

                start_session(5);
                assert_eq!(Session::current_index(), 5);
                // era changed.
                assert_eq!(Staking::current_era(), Some(2));
                assert_eq!(Staking::is_current_session_final(), false);
            })
    }

    #[test]
    fn offchain_window_is_triggered() {
        ExtBuilder::default()
            .session_per_era(5)
            .period(10)
            .election_lookahead(3)
            .build()
            .execute_with(|| {
                run_to_block(7);
                assert_session_era!(0, 0);

                run_to_block(10);
                assert_session_era!(1, 0);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
                assert!(Staking::snapshot_nominators().is_none());
                assert!(Staking::snapshot_validators().is_none());

                run_to_block(36);
                assert_session_era!(3, 0);

                // fist era has session 0, which has 0 blocks length, so we have in total 40 blocks
                // in the era.
                run_to_block(37);
                assert_session_era!(3, 0);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(37));
                assert!(Staking::snapshot_nominators().is_some());
                assert!(Staking::snapshot_validators().is_some());

                run_to_block(38);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(37));

                run_to_block(39);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(37));

                run_to_block(40);
                assert_session_era!(4, 0);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
                assert!(Staking::snapshot_nominators().is_none());
                assert!(Staking::snapshot_validators().is_none());

                run_to_block(86);
                assert_session_era!(8, 1);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
                assert!(Staking::snapshot_nominators().is_none());
                assert!(Staking::snapshot_validators().is_none());

                // second era onwards has 50 blocks per era.
                run_to_block(87);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(87));
                assert!(Staking::snapshot_nominators().is_some());
                assert!(Staking::snapshot_validators().is_some());

                run_to_block(90);
                assert_session_era!(9, 1);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
                assert!(Staking::snapshot_nominators().is_none());
                assert!(Staking::snapshot_validators().is_none());
            })
    }

    #[test]
    fn offchain_window_is_triggered_when_forcing() {
        ExtBuilder::default()
            .session_per_era(5)
            .period(10)
            .election_lookahead(3)
            .build()
            .execute_with(|| {
                run_to_block(12);
                ForceEra::put(Forcing::ForceNew);
                run_to_block(13);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);

                run_to_block(17); // instead of 47
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(17));

                run_to_block(20);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
            })
    }

    #[test]
    fn offchain_window_is_triggered_when_force_always() {
        ExtBuilder::default()
            .session_per_era(5)
            .period(10)
            .election_lookahead(3)
            .build()
            .execute_with(|| {
                ForceEra::put(Forcing::ForceAlways);
                run_to_block(16);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);

                run_to_block(17); // instead of 37
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(17));

                run_to_block(20);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);

                run_to_block(26);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);

                run_to_block(27); // next one again
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(27));
            })
    }

    #[test]
    fn offchain_window_closes_when_forcenone() {
        ExtBuilder::default()
            .session_per_era(5)
            .period(10)
            .election_lookahead(3)
            .build()
            .execute_with(|| {
                ForceEra::put(Forcing::ForceNone);

                run_to_block(36);
                assert_session_era!(3, 0);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);

                // opens
                run_to_block(37);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(37));
                assert!(Staking::is_current_session_final());
                assert!(Staking::snapshot_validators().is_some());

                // closes normally
                run_to_block(40);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
                assert!(!Staking::is_current_session_final());
                assert!(Staking::snapshot_validators().is_none());
                assert_session_era!(4, 0);

                run_to_block(47);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
                assert_session_era!(4, 0);

                run_to_block(57);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
                assert_session_era!(5, 0);

                run_to_block(67);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);

                // Will not open again as scheduled
                run_to_block(87);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
                assert_session_era!(8, 0);

                run_to_block(90);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
                assert_session_era!(9, 0);
            })
    }

    #[test]
    fn offchain_window_on_chain_fallback_works() {
        ExtBuilder::default().build_and_execute(|| {
            start_session(1);
            start_session(2);
            assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
            // some election must have happened by now.
            /*assert_eq!(
                staking_events().into_iter().last().unwrap(),
                RawEvent::StakingElection(ElectionCompute::OnChain),
            );*/
        })
    }

    #[test]
    #[ignore] // This takes a few mins
    fn offchain_wont_work_if_snapshot_fails() {
        ExtBuilder::default()
            .offchain_election_ext()
            .build()
            .execute_with(|| {
                run_to_block(12);
                assert!(Staking::snapshot_validators().is_some());
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(12));

                // validate more than the limit
                let limit: NominatorIndex = ValidatorIndex::max_value() as NominatorIndex + 1;
                let ctrl = 1_000_000;
                for i in 0..limit {
                    bond_validator((1000 + i).into(), (1000 + i + ctrl).into(), 100);
                }

                // window stays closed since no snapshot was taken.
                run_to_block(27);
                assert!(Staking::snapshot_validators().is_none());
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);
            })
    }

    #[test]
    fn staking_is_locked_when_election_window_open() {
        ExtBuilder::default()
            .offchain_election_ext()
            .election_lookahead(3)
            .build()
            .execute_with(|| {
                run_to_block(12);
                assert!(Staking::snapshot_validators().is_some());
                // given
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(12));

                // chill et. al. are now not allowed.
                assert_noop!(
                    Staking::chill(Origin::signed(10)),
                    Error::<Test>::CallNotAllowed,
                );
            })
    }

    #[test]
    fn signed_result_can_be_submitted() {
        // should check that we have a new validator set normally, event says that it comes from
        // offchain.
        ExtBuilder::default()
            .offchain_election_ext()
            .build()
            .execute_with(|| {
                run_to_block(12);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(12));
                assert!(Staking::snapshot_validators().is_some());

                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});
                assert_ok!(submit_solution(Origin::signed(10), winners, compact, score,));

                let queued_result = Staking::queued_elected().unwrap();
                assert_eq!(queued_result.compute, ElectionCompute::Signed);
                /*assert_eq!(
                    staking_events().into_iter()
                        .last()
                        .unwrap(),
                    RawEvent::SolutionStored(ElectionCompute::Signed),
                );*/

                run_to_block(15);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);

                /*assert_eq!(
                    staking_events().into_iter()
                        .last()
                        .unwrap(),
                    RawEvent::StakingElection(ElectionCompute::Signed),
                );*/
            })
    }

    #[test]
    fn signed_result_can_be_submitted_later() {
        // same as `signed_result_can_be_submitted` but at a later block.
        ExtBuilder::default()
            .offchain_election_ext()
            .build()
            .execute_with(|| {
                run_to_block(14);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Open(12));

                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});
                assert_ok!(submit_solution(Origin::signed(10), winners, compact, score));

                let queued_result = Staking::queued_elected().unwrap();
                assert_eq!(queued_result.compute, ElectionCompute::Signed);

                run_to_block(15);
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);

                /*assert_eq!(
                    staking_events().into_iter()
                        .last()
                        .unwrap(),
                    RawEvent::StakingElection(ElectionCompute::Signed),
                );*/
            })
    }

    #[test]
    fn early_solution_submission_is_rejected() {
        // should check that we have a new validator set normally, event says that it comes from
        // offchain.
        ExtBuilder::default()
            .offchain_election_ext()
            .build()
            .execute_with(|| {
                run_to_block(11);
                // submission is not yet allowed
                assert_eq!(Staking::era_election_status(), ElectionStatus::Closed);

                // create all the indices just to build the solution.
                Staking::create_stakers_snapshot();
                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});
                Staking::kill_stakers_snapshot();

                assert_err_with_weight!(
                    Staking::submit_election_solution(
                        Origin::signed(10),
                        winners.clone(),
                        compact.clone(),
                        score,
                        current_era(),
                        ElectionSize::default(),
                    ),
                    Error::<Test>::OffchainElectionEarlySubmission,
                    Some(<Test as frame_system::Config>::DbWeight::get().reads(1)),
                );
            })
    }

    #[test]
    fn weak_solution_is_rejected() {
        // A solution which is weaker than what we currently have on-chain is rejected.
        ExtBuilder::default()
            .offchain_election_ext()
            .has_stakers(false)
            .validator_count(4)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                // a good solution
                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});
                assert_ok!(submit_solution(Origin::signed(10), winners, compact, score,));

                // a bad solution
                let (compact, winners, score) = horrible_phragmen_with_post_processing(false);
                assert_err_with_weight!(
                    submit_solution(Origin::signed(10), winners.clone(), compact.clone(), score,),
                    Error::<Test>::OffchainElectionWeakSubmission,
                    Some(<Test as frame_system::Config>::DbWeight::get().reads(3))
                );
            })
    }

    #[test]
    fn better_solution_is_accepted() {
        // A solution which is better than what we currently have on-chain is accepted.
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                // a meeeeh solution
                let (compact, winners, score) = horrible_phragmen_with_post_processing(false);
                assert_ok!(submit_solution(Origin::signed(10), winners, compact, score,));

                // a better solution
                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});
                assert_ok!(submit_solution(Origin::signed(10), winners, compact, score,));
            })
    }

    #[test]
    fn offchain_worker_runs_when_window_open() {
        // at the end of the first finalized block with ElectionStatus::open(_), it should execute.
        let mut ext = ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(2)
            .build();
        let state = offchainify(&mut ext, 0);
        ext.execute_with(|| {
            run_to_block(12);

            // local key 11 is in the elected set.
            assert_eq_uvec!(Session::validators(), vec![11, 21]);
            assert_eq!(state.read().transactions.len(), 0);
            Staking::offchain_worker(12);
            assert_eq!(state.read().transactions.len(), 1);

            let encoded = state.read().transactions[0].clone();
            let extrinsic: Extrinsic = Decode::decode(&mut &*encoded).unwrap();

            let call = extrinsic.call;
            let inner = match call {
                mock::RuntimeCall::Staking(inner) => inner,
                _ => panic!(),
            };

            assert_eq!(
                <Staking as sp_runtime::traits::ValidateUnsigned>::validate_unsigned(
                    TransactionSource::Local,
                    &inner,
                ),
                TransactionValidity::Ok(ValidTransaction {
                    priority: UnsignedPriority::get() + 1125, // the proposed slot stake.
                    requires: vec![],
                    provides: vec![("StakingOffchain", current_era()).encode()],
                    longevity: 3,
                    propagate: false,
                })
            )
        })
    }

    #[test]
    fn offchain_worker_runs_with_equalise() {
        // Offchain worker equalises based on the number provided by randomness. See the difference
        // in the priority, which comes from the computed score.
        let mut ext = ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(2)
            .max_offchain_iterations(2)
            .build();
        let state = offchainify(&mut ext, 2);
        ext.execute_with(|| {
            run_to_block(12);

            // local key 11 is in the elected set.
            assert_eq_uvec!(Session::validators(), vec![11, 21]);
            assert_eq!(state.read().transactions.len(), 0);
            Staking::offchain_worker(12);
            assert_eq!(state.read().transactions.len(), 1);

            let encoded = state.read().transactions[0].clone();
            let extrinsic: Extrinsic = Decode::decode(&mut &*encoded).unwrap();

            let call = extrinsic.call;
            let inner = match call {
                mock::RuntimeCall::Staking(inner) => inner,
                _ => panic!(),
            };

            assert_eq!(
                <Staking as sp_runtime::traits::ValidateUnsigned>::validate_unsigned(
                    TransactionSource::Local,
                    &inner,
                ),
                TransactionValidity::Ok(ValidTransaction {
                    // the proposed slot stake, with balance_solution.
                    priority: UnsignedPriority::get() + 1250,
                    requires: vec![],
                    provides: vec![("StakingOffchain", active_era()).encode()],
                    longevity: 3,
                    propagate: false,
                })
            )
        })
    }

    #[test]
    fn mediocre_submission_from_authority_is_early_rejected() {
        let mut ext = ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .build();
        let state = offchainify(&mut ext, 0);
        ext.execute_with(|| {
            run_to_block(12);
            // put a good solution on-chain
            let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});
            assert_ok!(submit_solution(Origin::signed(10), winners, compact, score,),);

            // now run the offchain worker in the same chain state.
            Staking::offchain_worker(12);
            assert_eq!(state.read().transactions.len(), 1);

            let encoded = state.read().transactions[0].clone();
            let extrinsic: Extrinsic = Decode::decode(&mut &*encoded).unwrap();

            let call = extrinsic.call;
            let inner = match call {
                mock::RuntimeCall::Staking(inner) => inner,
                _ => panic!(),
            };

            // pass this call to ValidateUnsigned
            assert_eq!(
                <Staking as sp_runtime::traits::ValidateUnsigned>::validate_unsigned(
                    TransactionSource::Local,
                    &inner,
                ),
                TransactionValidity::Err(InvalidTransaction::Custom(14).into()),
            )
        })
    }

    #[test]
    fn invalid_phragmen_result_correct_number_of_winners() {
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                ValidatorCount::put(3);
                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});
                ValidatorCount::put(4);

                assert_eq!(winners.len(), 3);

                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusWinnerCount,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_solution_size() {
        ExtBuilder::default()
            .offchain_election_ext()
            .build()
            .execute_with(|| {
                run_to_block(12);

                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});

                assert_noop!(
                    Staking::submit_election_solution(
                        Origin::signed(10),
                        winners,
                        compact,
                        score,
                        current_era(),
                        ElectionSize::default(),
                    ),
                    Error::<Test>::OffchainElectionBogusElectionSize,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_correct_number_of_winners_1() {
        // if we have too little validators, then the number of candidates is the bound.
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(8) // we simply cannot elect 8
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                ValidatorCount::put(3);
                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});
                ValidatorCount::put(4);

                assert_eq!(winners.len(), 3);

                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusWinnerCount,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_correct_number_of_winners_2() {
        // if we have too little validators, then the number of candidates is the bound.
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(8) // we simply cannot elect 8
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});

                assert_eq!(winners.len(), 4);

                // all good. We chose 4 and it works.
                assert_ok!(submit_solution(Origin::signed(10), winners, compact, score,),);
            })
    }

    #[test]
    fn invalid_phragmen_result_out_of_bound_nominator_index() {
        // A nominator index which is simply invalid
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                assert_eq!(Staking::snapshot_nominators().unwrap().len(), 5 + 4);
                assert_eq!(Staking::snapshot_validators().unwrap().len(), 4);
                let (mut compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});

                // index 9 doesn't exist.
                compact.push_votes1((9, 2));

                // The error type sadly cannot be more specific now.
                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusCompact,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_out_of_bound_validator_index() {
        // A validator index which is out of bound
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                assert_eq!(Staking::snapshot_nominators().unwrap().len(), 5 + 4);
                assert_eq!(Staking::snapshot_validators().unwrap().len(), 4);
                let (mut compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});

                // index 4 doesn't exist.
                compact.votes1.iter_mut().for_each(|(_, vidx)| {
                    if *vidx == 1 {
                        *vidx = 4
                    }
                });

                // The error type sadly cannot be more specific now.
                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusCompact,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_out_of_bound_winner_index() {
        // A winner index which is simply invalid
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                assert_eq!(Staking::snapshot_nominators().unwrap().len(), 5 + 4);
                assert_eq!(Staking::snapshot_validators().unwrap().len(), 4);
                let (compact, _, score) = prepare_submission_with(true, true, 2, |_| {});

                // index 4 doesn't exist.
                let winners = vec![0, 1, 2, 4];

                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusWinner,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_non_winner_validator_index() {
        // An edge that points to a correct validator index who is NOT a winner. This is very
        // similar to the test that raises `PhragmenBogusNomination`.
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(2) // we select only 2.
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                assert_eq!(Staking::snapshot_nominators().unwrap().len(), 5 + 4);
                assert_eq!(Staking::snapshot_validators().unwrap().len(), 4);
                let (compact, winners, score) = prepare_submission_with(false, true, 2, |a| {
                    // swap all 11 and 41s in the distribution with non-winners. Note that it is
                    // important that the count of winners and the count of unique targets remain
                    // valid.
                    a.iter_mut()
                        .for_each(|StakedAssignment { who, distribution }| {
                            distribution.iter_mut().for_each(|(t, _)| {
                                if *t == 41 {
                                    *t = 31
                                } else {
                                    *t = 21
                                }
                                // if it is self vote, correct that.
                                if *who == 41 {
                                    *who = 31
                                }
                                if *who == 11 {
                                    *who = 21
                                }
                            })
                        });
                });

                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusNomination,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_wrong_self_vote() {
        // A self vote for someone else.
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                let (compact, winners, score) = prepare_submission_with(true, true, 2, |a| {
                    // mutate a self vote to target someone else. That someone else is still among the
                    // winners
                    a.iter_mut().find(|x| x.who == 11).map(|x| {
                        x.distribution
                            .iter_mut()
                            .find(|y| y.0 == 11)
                            .map(|y| y.0 = 21)
                    });
                });

                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusSelfVote,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_wrong_self_vote_2() {
        // A self validator voting for someone else next to self vote.
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                let (compact, winners, score) = prepare_submission_with(true, true, 2, |a| {
                    // Remove the self vote.
                    a.retain(|x| x.who != 11);
                    // add is as a new double vote
                    a.push(StakedAssignment {
                        who: 11,
                        distribution: vec![(11, 50), (21, 50)],
                    });
                });

                // This raises score issue.
                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusSelfVote,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_over_stake() {
        // Someone's edge ratios sums to more than 100%.
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                // Note: we don't reduce here to be able to tweak votes3. votes3 will vanish if you
                // reduce.
                let (mut compact, winners, score) = prepare_submission_with(true, false, 0, |_| {});

                if let Some(c) = compact.get_votes3().iter_mut().find(|x| x.0 == 0) {
                    // by default it should have been (0, [(2, 33%), (1, 33%)], 0)
                    // now the sum is above 100%
                    c.1 = [(2, percent(66)), (1, percent(66))];
                }

                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusCompact,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_under_stake() {
        // at the time of this writing, we cannot under stake someone. The compact assignment works
        // in a way that some of the stakes are presented by the submitter, and the last one is read
        // from chain by subtracting the rest from total. Hence, the sum is always correct.
        // This test is only here as a demonstration.
    }

    #[test]
    fn invalid_phragmen_result_invalid_target_stealing() {
        // A valid voter who voted for someone who is a candidate, and is a correct winner, but is
        // actually NOT nominated by this nominator.
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                let (compact, winners, score) = prepare_submission_with(true, false, 0, |a| {
                    // 3 only voted for 20 and 40. We add a fake vote to 30. The stake sum is still
                    // correctly 100.
                    a.iter_mut()
                        .find(|x| x.who == 3)
                        .map(|x| x.distribution = vec![(21, 50), (41, 30), (31, 20)]);
                });

                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusNomination,
                );
            })
    }

    #[test]
    fn nomination_slash_filter_is_checked() {
        // If a nominator has voted for someone who has been recently slashed, that particular
        // nomination should be disabled for the upcoming election. A solution must respect this
        // rule.
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();

                // finalize the round with fallback. This is needed since all nominator submission
                // are in era zero and we want this one to pass with no problems.
                run_to_block(15);

                // go to the next session to trigger mock::start_active_era and bump the active era
                run_to_block(20);

                // slash 10. This must happen outside of the election window.
                let offender_expo = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);

                on_offence_now(
                    &[OffenceDetails {
                        offender: (11, offender_expo.clone()),
                        reporters: vec![],
                    }],
                    &[Perbill::from_percent(50)],
                );

                // validate 10 again for the next round. But this guy will not have the votes that
                // it should have had from 1 and 2.
                assert_ok!(Staking::validate(Origin::signed(10), Default::default()));

                // open the election window and create snapshots.
                // FIXME(Centril, Neopallium): We changed this to 32.
                // We're unsure as to why this was necessary and if it was entirely correct.
                run_to_block(23);

                // a solution that has been prepared after the slash.
                let (compact, winners, score) = prepare_submission_with(true, false, 0, |a| {
                    // no one is allowed to vote for 10, except for itself.
                    a.into_iter().filter(|s| s.who != 11).for_each(|s| {
                        assert!(s.distribution.iter().find(|(t, _)| *t == 11).is_none())
                    });
                });

                // can be submitted.
                assert_ok!(submit_solution(Origin::signed(10), winners, compact, score,));

                // a wrong solution.
                let (compact, winners, score) = prepare_submission_with(true, false, 0, |a| {
                    // add back the vote that has been filtered out.
                    a.push(StakedAssignment {
                        who: 1,
                        distribution: vec![(11, 100)],
                    });
                });

                // is rejected.
                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionSlashedNomination,
                );
            })
    }

    #[test]
    fn invalid_phragmen_result_wrong_score() {
        // A valid voter who's total distributed stake is more than what they bond
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);

                let (compact, winners, mut score) = prepare_submission_with(true, true, 2, |_| {});
                score.minimal_stake += 1;

                assert_noop!(
                    submit_solution(Origin::signed(10), winners, compact, score,),
                    Error::<Test>::OffchainElectionBogusScore,
                );
            })
    }

    #[test]
    fn off_chain_election_validator_non_compliance() {
        ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .has_stakers(false)
            .session_per_era(5)
            .period(10)
            .election_lookahead(3)
            .build()
            .execute_with(|| {
                build_offchain_phragmen_test_ext();
                run_to_block(12);
                ValidatorCount::put(10);

                // Add did to user
                provide_did_to_user(70);
                add_secondary_key(70, 71);
                // add a new validator candidate
                assert_ok!(Staking::bond(
                    Origin::signed(70),
                    71,
                    1000,
                    RewardDestination::Controller
                ));
                assert_add_permissioned_validator!(&70);

                assert_ok!(Staking::validate(
                    Origin::signed(71),
                    ValidatorPrefs::default()
                ));
                assert_ok!(Session::set_keys(
                    Origin::signed(71),
                    SessionKeys { other: 71.into() },
                    vec![]
                ));

                run_to_block(37);
                assert!(Staking::snapshot_validators().is_some());
                let entity_id = Identity::get_identity(&70).unwrap();

                let (compact, winners, score) = prepare_submission_with(true, true, 2, |_| {});

                // Ensure submit_solution runs successfully
                assert_ok!(submit_solution(
                    Origin::signed(70),
                    winners.clone(),
                    compact.clone(),
                    score,
                ));

                assert_eq!(winners.len(), 5);

                let claim_key = Claim1stKey {
                    target: entity_id,
                    claim_type: ClaimType::CustomerDueDiligence,
                };
                #[allow(deprecated)]
                Claims::remove_prefix(claim_key, None); // Remove all CDD claims for the validator.

                // Ensure did no longer has valid cdd
                assert_eq!(Identity::has_valid_cdd(entity_id), false);

                run_to_block(87);
                assert!(Staking::snapshot_validators().is_some());
                let (compact_2, winners_2, score_2) =
                    prepare_submission_with(true, true, 2, |_| {});

                assert_eq!(winners_2.len(), 4);

                // Ensure submit_solution runs successfully
                assert_ok!(submit_solution(
                    Origin::signed(10),
                    winners_2.clone(),
                    compact_2.clone(),
                    score_2,
                ));
            })
    }

    #[test]
    fn offchain_storage_is_set() {
        let mut ext = ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .build();
        let state = offchainify(&mut ext, 0);

        ext.execute_with(|| {
            use offchain_election::OFFCHAIN_HEAD_DB;
            use sp_runtime::offchain::storage::StorageValueRef;

            run_to_block(12);

            Staking::offchain_worker(12);
            // it works
            assert_eq!(state.read().transactions.len(), 1);

            // and it is set
            let storage = StorageValueRef::persistent(&OFFCHAIN_HEAD_DB);
            assert_eq!(storage.get::<BlockNumber>().unwrap().unwrap(), 12);
        })
    }

    #[test]
    fn offchain_storage_prevents_duplicate() {
        let mut ext = ExtBuilder::default()
            .offchain_election_ext()
            .validator_count(4)
            .build();
        let _ = offchainify(&mut ext, 0);

        ext.execute_with(|| {
            use offchain_election::OFFCHAIN_HEAD_DB;
            use sp_runtime::offchain::storage::StorageValueRef;
            let storage = StorageValueRef::persistent(&OFFCHAIN_HEAD_DB);

            run_to_block(12);

            // first run -- ok
            assert_eq!(
                offchain_election::set_check_offchain_execution_status::<Test>(12),
                Ok(()),
            );
            assert_eq!(storage.get::<BlockNumber>().unwrap().unwrap(), 12);

            // re-execute after the next. not allowed.
            assert_eq!(
                offchain_election::set_check_offchain_execution_status::<Test>(13),
                Err(ValueFunctionFailed("recently executed.")),
            );

            // a fork like situation -- re-execute 10, 11, 12. But it won't go through.
            assert_eq!(
                offchain_election::set_check_offchain_execution_status::<Test>(10),
                Err(ValueFunctionFailed("fork.")),
            );
            assert_eq!(
                offchain_election::set_check_offchain_execution_status::<Test>(11),
                Err(ValueFunctionFailed("fork.")),
            );
            assert_eq!(
                offchain_election::set_check_offchain_execution_status::<Test>(12),
                Err(ValueFunctionFailed("recently executed.")),
            );
        })
    }
}

#[test]
fn slash_kicks_validators_not_nominators_and_disables_nominator_for_kicked_validator() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);
        assert_eq_uvec!(Session::validators(), vec![11, 21]);

        // pre-slash balance
        assert_eq!(Balances::free_balance(11), 1000);
        assert_eq!(Balances::free_balance(101), 2000);

        // 11 and 21 both have the support of 100
        let exposure_11 = Staking::eras_stakers(Staking::active_era().unwrap().index, &11);
        let exposure_21 = Staking::eras_stakers(Staking::active_era().unwrap().index, &21);

        assert_eq!(exposure_11.total, 1000 + 125);
        assert_eq!(exposure_21.total, 1000 + 375);

        on_offence_now(
            &[OffenceDetails {
                offender: (11, exposure_11.clone()),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
        );

        // post-slash balance
        let _nominator_slash_amount_11 = 125 / 10;
        assert_eq!(Balances::free_balance(11), 900);
        // Polymesh-Note -> No slashing for nominataors
        // assert_eq!(
        //     Balances::free_balance(101),
        //     2000 - nominator_slash_amount_11
        // );
        assert_eq!(Balances::free_balance(101), 2000);

        // This is the best way to check that the validator was chilled; `get` will
        // return default value.
        for (stash, _) in Validators::<Test>::iter() {
            assert!(stash != 11);
        }

        let nominations = mock::Staking::nominators(&101).unwrap();

        // and make sure that the vote will be ignored even if the validator
        // re-registers.
        let last_slash = SlashingSpans::<Test>::get(&11)
            .unwrap()
            .last_nonzero_slash();
        assert!(nominations.submitted_in < last_slash);

        // actually re-bond the slashed validator
        assert_ok!(Staking::validate(Origin::signed(10), Default::default()));

        mock::start_active_era(2);
        let exposure_11 = Staking::eras_stakers(Staking::active_era().unwrap().index, &11);
        let exposure_21 = Staking::eras_stakers(Staking::active_era().unwrap().index, &21);

        // 10 is re-elected, but without the support of 100
        assert_eq!(exposure_11.total, 900);

        // Polymesh-note -> No slashing for nominators
        // 20 is re-elected, with the (almost) entire support of 100
        //assert_eq!(exposure_21.total, 1000 + 500 - nominator_slash_amount_11);
        assert_eq!(exposure_21.total, 1000 + 500);
    });
}

#[test]
fn claim_reward_at_the_last_era_and_no_double_claim_and_invalid_claim() {
    // should check that:
    // * rewards get paid until history_depth for both validators and nominators
    // * an invalid era to claim doesn't update last_reward
    // * double claim of one era fails
    ExtBuilder::default().nominate(true).build_and_execute(|| {
        let init_balance_10 = Balances::total_balance(&10);
        let init_balance_100 = Balances::total_balance(&100);

        let part_for_10 = Perbill::from_rational::<u32>(1000, 1125);
        let part_for_100 = Perbill::from_rational::<u32>(125, 1125);

        // Check state
        Payee::<Test>::insert(11, RewardDestination::Controller);
        Payee::<Test>::insert(101, RewardDestination::Controller);

        <Module<Test>>::reward_by_ids(vec![(11, 1)]);
        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());

        mock::start_active_era(1);

        <Module<Test>>::reward_by_ids(vec![(11, 1)]);
        // Change total issuance in order to modify total payout
        let _ = Balances::deposit_creating(&999, 1_000_000_000);
        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
        assert!(total_payout_1 != total_payout_0);

        mock::start_active_era(2);

        <Module<Test>>::reward_by_ids(vec![(11, 1)]);
        // Change total issuance in order to modify total payout
        let _ = Balances::deposit_creating(&999, 1_000_000_000);
        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_2 = current_total_payout_for_duration(reward_time_per_era());
        assert!(total_payout_2 != total_payout_0);
        assert!(total_payout_2 != total_payout_1);

        mock::start_active_era(Staking::history_depth() + 1);

        let active_era = Staking::active_era().unwrap().index;

        // This is the latest planned era in staking, not the active era
        let current_era = Staking::current_era().unwrap();

        // Last kept is 1:
        assert!(current_era - Staking::history_depth() == 1);
        assert_noop!(
            Staking::payout_stakers(Origin::signed(1337), 11, 0),
            // Fail: Era out of history
            Error::<Test>::InvalidEraToReward
        );
        assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 1));
        assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 2));
        assert_noop!(
            Staking::payout_stakers(Origin::signed(1337), 11, 2),
            // Fail: Double claim
            Error::<Test>::AlreadyClaimed
        );
        assert_noop!(
            Staking::payout_stakers(Origin::signed(1337), 11, active_era),
            // Fail: Era not finished yet
            Error::<Test>::InvalidEraToReward
        );

        // Era 0 can't be rewarded anymore and current era can't be rewarded yet
        // only era 1 and 2 can be rewarded.

        assert_eq!(
            Balances::total_balance(&10),
            init_balance_10 + part_for_10 * (total_payout_1 + total_payout_2),
        );
        assert_eq!(
            Balances::total_balance(&100),
            init_balance_100 + part_for_100 * (total_payout_1 + total_payout_2),
        );
    });
}

#[test]
fn zero_slash_keeps_nominators() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);

        assert_eq!(Balances::free_balance(11), 1000);

        let exposure = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);
        assert_eq!(Balances::free_balance(101), 2000);

        on_offence_now(
            &[OffenceDetails {
                offender: (11, exposure.clone()),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );

        assert_eq!(Balances::free_balance(11), 1000);
        assert_eq!(Balances::free_balance(101), 2000);

        // This is the best way to check that the validator was chilled; `get` will
        // return default value.
        for (stash, _) in Validators::<Test>::iter() {
            assert!(stash != 11);
        }

        let nominations = mock::Staking::nominators(&101).unwrap();

        // and make sure that the vote will not be ignored, because the slash was
        // zero.
        let last_slash = SlashingSpans::<Test>::get(&11)
            .unwrap()
            .last_nonzero_slash();
        assert!(nominations.submitted_in >= last_slash);
    });
}

#[test]
fn six_session_delay() {
    ExtBuilder::default().build_and_execute(|| {
        use pallet_session::SessionManager;

        let val_set = Session::validators();
        let init_session = Session::current_index();
        let init_active_era = Staking::active_era().unwrap().index;
        // pallet-session is delaying session by one, thus the next session to plan is +2.
        assert_eq!(
            <Staking as SessionManager<_>>::new_session(init_session + 2),
            None
        );
        assert_eq!(
            <Staking as SessionManager<_>>::new_session(init_session + 3),
            Some(val_set.clone())
        );
        assert_eq!(
            <Staking as SessionManager<_>>::new_session(init_session + 4),
            None
        );
        assert_eq!(
            <Staking as SessionManager<_>>::new_session(init_session + 5),
            None
        );
        assert_eq!(
            <Staking as SessionManager<_>>::new_session(init_session + 6),
            Some(val_set.clone())
        );

        <Staking as SessionManager<_>>::end_session(init_session);
        <Staking as SessionManager<_>>::start_session(init_session + 1);
        assert_eq!(Staking::active_era().unwrap().index, init_active_era);
        <Staking as SessionManager<_>>::end_session(init_session + 1);
        <Staking as SessionManager<_>>::start_session(init_session + 2);
        assert_eq!(Staking::active_era().unwrap().index, init_active_era);

        // Reward current era
        Staking::reward_by_ids(vec![(11, 1)]);

        // New active era is triggered here.
        <Staking as SessionManager<_>>::end_session(init_session + 2);
        <Staking as SessionManager<_>>::start_session(init_session + 3);
        assert_eq!(Staking::active_era().unwrap().index, init_active_era + 1);
        <Staking as SessionManager<_>>::end_session(init_session + 3);
        <Staking as SessionManager<_>>::start_session(init_session + 4);
        assert_eq!(Staking::active_era().unwrap().index, init_active_era + 1);
        <Staking as SessionManager<_>>::end_session(init_session + 4);
        <Staking as SessionManager<_>>::start_session(init_session + 5);
        assert_eq!(Staking::active_era().unwrap().index, init_active_era + 1);

        // Reward current era
        Staking::reward_by_ids(vec![(21, 2)]);

        // New active era is triggered here.
        <Staking as SessionManager<_>>::end_session(init_session + 5);
        <Staking as SessionManager<_>>::start_session(init_session + 6);
        assert_eq!(Staking::active_era().unwrap().index, init_active_era + 2);

        // That reward are correct
        assert_eq!(Staking::eras_reward_points(init_active_era).total, 1);
        assert_eq!(Staking::eras_reward_points(init_active_era + 1).total, 2);
    });
}

#[test]
fn test_max_nominator_rewarded_per_validator_and_cant_steal_someone_else_reward() {
    // Test:
    // * If nominator nomination is below the $MaxNominatorRewardedPerValidator other nominator
    //   then the nominator can't claim its reward
    // * A nominator can't claim another nominator reward
    ExtBuilder::default().build_and_execute(|| {
        for i in 0..=<Test as Config>::MaxNominatorRewardedPerValidator::get() {
            let stash = 10_000 + i as AccountId;
            let controller = 20_000 + i as AccountId;
            let balance = 10_000 + i as Balance;
            Balances::make_free_balance_be(&stash, balance);
            provide_did_to_user(stash);
            add_secondary_key(stash, controller);
            assert_ok!(Staking::bond(
                Origin::signed(stash),
                controller,
                balance,
                RewardDestination::Stash
            ));
            assert_ok!(Staking::nominate(Origin::signed(controller), vec![11]));
        }
        mock::start_active_era(1);

        <Module<Test>>::reward_by_ids(vec![(11, 1)]);
        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_0 = current_total_payout_for_duration(3 * 1000);
        assert!(total_payout_0 > 100); // Test is meaningful if reward something

        mock::start_active_era(2);
        mock::make_all_reward_payment(1);

        // Assert only nominators from 1 to Max are rewarded
        for i in 0..=<Test as Config>::MaxNominatorRewardedPerValidator::get() {
            let stash = 10_000 + i as AccountId;
            let balance = 10_000 + i as Balance;
            if stash == 10_000 {
                assert!(Balances::free_balance(&stash) == balance);
            } else {
                assert!(Balances::free_balance(&stash) > balance);
            }
        }
    });
}

fn root() -> Origin {
    Origin::from(frame_system::RawOrigin::Root)
}

#[test]
fn set_history_depth_works() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(10);
        Staking::set_history_depth(root(), 20, 0).unwrap();
        assert_ne!(mock::Staking::eras_total_stake(10 - 4), 0u128);
        assert_ne!(mock::Staking::eras_total_stake(10 - 5), 0);
        Staking::set_history_depth(root(), 4, 0).unwrap();
        assert_ne!(mock::Staking::eras_total_stake(10 - 4), 0);
        assert_eq!(mock::Staking::eras_total_stake(10 - 5), 0);
        Staking::set_history_depth(root(), 3, 0).unwrap();
        assert_eq!(mock::Staking::eras_total_stake(10 - 4), 0);
        assert_eq!(mock::Staking::eras_total_stake(10 - 5), 0);
        Staking::set_history_depth(root(), 8, 0).unwrap();
        assert_eq!(mock::Staking::eras_total_stake(10 - 4), 0);
        assert_eq!(mock::Staking::eras_total_stake(10 - 5), 0);
    });
}

#[test]
fn test_payout_stakers() {
    // Here we will test validator can set `max_nominators_payout` and it works.
    // We also test that `payout_extra_nominators` works.
    ExtBuilder::default()
        .has_stakers(false)
        .build_and_execute(|| {
            let balance = 1000;
            // Create three validators:
            bond_validator(11, 10, balance); // Default(64)

            // Create nominators, targeting stash of validators
            for i in 0..100 {
                bond_nominator_cdd(1000 + i, 100 + i, balance + i as Balance, vec![11]);
            }

            mock::start_active_era(1);
            Staking::reward_by_ids(vec![(11, 1)]);
            // Compute total payout now for whole duration as other parameter won't change
            let total_payout_0 = current_total_payout_for_duration(3 * 1000);
            assert!(total_payout_0 > 100); // Test is meaningful if reward something
            mock::start_active_era(2);
            assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 1));

            // Top 64 nominators of validator 11 automatically paid out, including the validator
            // Validator payout goes to controller.
            assert!(Balances::free_balance(&10) > balance);
            for i in 36..100 {
                assert!(Balances::free_balance(&(100 + i)) > balance + i as Balance);
            }
            // The bottom 36 do not
            for i in 0..36 {
                assert_eq!(Balances::free_balance(&(100 + i)), balance + i as Balance);
            }

            // We track rewards in `claimed_rewards` vec
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![1]
                })
            );

            for i in 3..16 {
                Staking::reward_by_ids(vec![(11, 1)]);
                // Compute total payout now for whole duration as other parameter won't change
                let total_payout_0 = current_total_payout_for_duration(3 * 1000);
                assert!(total_payout_0 > 100); // Test is meaningful if reward something
                mock::start_active_era(i);
                assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, i - 1));
            }

            // We track rewards in `claimed_rewards` vec
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: (1..=14).collect()
                })
            );

            for i in 16..100 {
                Staking::reward_by_ids(vec![(11, 1)]);
                // Compute total payout now for whole duration as other parameter won't change
                let total_payout_0 = current_total_payout_for_duration(3 * 1000);
                assert!(total_payout_0 > 100); // Test is meaningful if reward something
                mock::start_active_era(i);
            }

            // We clean it up as history passes
            assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 15));
            assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 98));
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![15, 98]
                })
            );

            // Out of order claims works.
            assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 69));
            assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 23));
            assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 42));
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![15, 23, 42, 69, 98]
                })
            );
        });
}

#[test]
fn payout_stakers_handles_basic_errors() {
    // Here we will test payouts handle all errors.
    ExtBuilder::default()
        .has_stakers(false)
        .build_and_execute(|| {
            // Same setup as the test above
            let balance = 1000;
            bond_validator(11, 10, balance); // Default(64)

            // Create nominators, targeting stash
            for i in 0..100 {
                provide_did_to_user(1000 + i);
                bond_nominator(1000 + i, 100 + i, balance + i as Balance, vec![11]);
            }

            mock::start_active_era(1);
            Staking::reward_by_ids(vec![(11, 1)]);
            // Compute total payout now for whole duration as other parameter won't change
            let total_payout_0 = current_total_payout_for_duration(3 * 1000);
            assert!(total_payout_0 > 100); // Test is meaningful if reward something
            mock::start_active_era(2);

            // Wrong Era, too big
            assert_noop!(
                Staking::payout_stakers(Origin::signed(1337), 11, 2),
                Error::<Test>::InvalidEraToReward
            );
            // Wrong Staker
            assert_noop!(
                Staking::payout_stakers(Origin::signed(1337), 10, 1),
                Error::<Test>::NotStash
            );

            for i in 3..100 {
                Staking::reward_by_ids(vec![(11, 1)]);
                // Compute total payout now for whole duration as other parameter won't change
                let total_payout_0 = current_total_payout_for_duration(3 * 1000);
                assert!(total_payout_0 > 100); // Test is meaningful if reward something
                mock::start_active_era(i);
            }
            // We are at era 99, with history depth of 84
            // We should be able to payout era 15 through 98 (84 total eras), but not 14 or 99.
            assert_noop!(
                Staking::payout_stakers(Origin::signed(1337), 11, 14),
                Error::<Test>::InvalidEraToReward
            );
            assert_noop!(
                Staking::payout_stakers(Origin::signed(1337), 11, 99),
                Error::<Test>::InvalidEraToReward
            );
            assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 15));
            assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 98));

            // Can't claim again
            assert_noop!(
                Staking::payout_stakers(Origin::signed(1337), 11, 15),
                Error::<Test>::AlreadyClaimed
            );
            assert_noop!(
                Staking::payout_stakers(Origin::signed(1337), 11, 98),
                Error::<Test>::AlreadyClaimed
            );
        });
}

#[test]
fn bond_during_era_correctly_populates_claimed_rewards() {
    ExtBuilder::default()
        .has_stakers(false)
        .build_and_execute(|| {
            // Era = None
            bond_validator(9, 8, 1000);
            assert_eq!(
                Staking::ledger(&8),
                Some(StakingLedger {
                    stash: 9,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );
            mock::start_active_era(5);
            bond_validator(11, 10, 1000);
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: (0..5).collect(),
                })
            );
            mock::start_active_era(99);
            bond_validator(13, 12, 1000);
            assert_eq!(
                Staking::ledger(&12),
                Some(StakingLedger {
                    stash: 13,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: (15..99).collect(),
                })
            );
        });
}

#[test]
fn offences_weight_calculated_correctly() {
    ExtBuilder::default().nominate(true).build_and_execute(|| {
		// On offence with zero offenders: 4 Reads, 1 Write
		let zero_offence_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(4, 1);
		assert_eq!(Staking::on_offence(&[], &[Perbill::from_percent(50)], 0, DisableStrategy::WhenSlashed), zero_offence_weight);

		// On Offence with N offenders, Unapplied: 4 Reads, 1 Write + 4 Reads, 5 Writes
		let n_offence_unapplied_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(4, 1)
			+ <Test as frame_system::Config>::DbWeight::get().reads_writes(4, 5);

		let offenders: Vec<OffenceDetails<<Test as frame_system::Config>::AccountId, pallet_session::historical::IdentificationTuple<Test>>>
			= (1..10).map(|i|
				OffenceDetails {
					offender: (i, Staking::eras_stakers(Staking::active_era().unwrap().index, i)),
					reporters: vec![],
				}
			).collect();
		assert_eq!(Staking::on_offence(&offenders, &[Perbill::from_percent(50)], 0, DisableStrategy::WhenSlashed), n_offence_unapplied_weight);

		// On Offence with one offenders, Applied
		let one_offender = [
			OffenceDetails {
				offender: (11, Staking::eras_stakers(Staking::active_era().unwrap().index, 11)),
				reporters: vec![1],
			},
		];

		let rw = 3; // rw reads and writes
		let one_offence_unapplied_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(4, 1)
			+ <Test as frame_system::Config>::DbWeight::get().reads_writes(rw, rw)
			// One `slash_cost`
			+ <Test as frame_system::Config>::DbWeight::get().reads_writes(6, 5)
			// `reward_cost` * reporters (1)
			+ <Test as frame_system::Config>::DbWeight::get().reads_writes(2, 2);

		assert_eq!(Staking::on_offence(&one_offender, &[Perbill::from_percent(50)], 0, DisableStrategy::WhenSlashed), one_offence_unapplied_weight);
	});
}

#[test]
fn on_initialize_weight_is_correct() {
    ExtBuilder::default()
        .has_stakers(false)
        .build_and_execute(|| {
            assert_eq!(Validators::<Test>::iter().count(), 0);
            assert_eq!(Nominators::<Test>::iter().count(), 0);
            // When this pallet has nothing, we do 4 reads each block
            let base_weight = <Test as frame_system::Config>::DbWeight::get().reads(4);
            assert_eq!(base_weight, Staking::on_initialize(0));
        });

    ExtBuilder::default()
        .offchain_election_ext()
        .validator_count(4)
        .has_stakers(false)
        .build()
        .execute_with(|| {
            offchain_phragmen::build_offchain_phragmen_test_ext();
            run_to_block(11);
            Staking::on_finalize(System::block_number());
            System::set_block_number((System::block_number() + 1).into());
            set_timestamp(System::block_number() * 1000 + INIT_TIMESTAMP);
            Session::on_initialize(System::block_number());

            assert_eq!(Validators::<Test>::iter().count(), 4);
            assert_eq!(Nominators::<Test>::iter().count(), 5);
            // With 4 validators and 5 nominator, we should increase weight by:
            // - (4 + 5) reads
            // - 3 Writes
            let final_weight =
                <Test as frame_system::Config>::DbWeight::get().reads_writes(4 + 9, 3);
            assert_eq!(final_weight, Staking::on_initialize(System::block_number()));
        });
}

#[test]
fn add_nominator_with_invalid_expiry() {
    ExtBuilder::default()
        .nominate(true)
        .build()
        .execute_with(|| {
            let now = Utc::now();
            let account_alice = 500;
            let (_alice_signed, _alice_did) =
                make_account_with_balance(account_alice, 1_000_000, Some(now.timestamp() as u64))
                    .unwrap();
            let account_alice_controller = account_alice + 1;
            let controller_signed = Origin::signed(account_alice_controller);

            // For valid trusted CDD service providers
            let account_bob = 600;
            let (_bob_signed, bob_did) = make_account_with_uid(account_bob).unwrap();
            add_trusted_cdd_provider(bob_did);

            // bond
            assert_ok!(Staking::bond(
                Origin::signed(account_alice.clone()),
                account_alice_controller,
                1000,
                RewardDestination::Stash
            ));

            let now = Utc::now();
            set_timestamp(now.timestamp() as u64);
            let validators = vec![10, 20, 30];
            assert_noop!(
                Staking::nominate(controller_signed.clone(), validators),
                Error::<Test>::StashIdentityNotCDDed,
            );
            assert!(Staking::nominators(&account_alice).is_none());
        });
}

#[test]
fn add_valid_nominator_with_multiple_claims() {
    ExtBuilder::default()
        .nominate(true)
        .build()
        .execute_with(|| {
            let account_alice = 500;
            let (_alice_signed, _alice_did) =
                make_account_with_balance(account_alice, 1_000_000, None).unwrap();

            let account_alice_controller = account_alice + 1;
            let controller_signed = Origin::signed(account_alice_controller);

            let now = Utc::now();

            // bond
            assert_ok!(Staking::bond(
                Origin::signed(account_alice.clone()),
                account_alice_controller,
                1000,
                RewardDestination::Stash
            ));

            set_timestamp(now.timestamp() as u64);
            let validators = vec![10, 20, 30];

            assert_ok!(Staking::nominate(controller_signed.clone(), validators));
            assert!(!Staking::nominators(&account_alice).is_none());
        });
}

#[test]
fn validate_nominators_with_valid_cdd() {
    ExtBuilder::default()
        .nominate(true)
        .build()
        .execute_with(|| {
            let mut now = Utc::now();
            let account_alice = 500;
            let (_alice_signed, _alice_did) = make_account_with_balance(
                account_alice,
                1_000_000,
                Some(now.timestamp() as u64 + 500u64),
            )
            .unwrap();

            let account_alice_controller = 500 + account_alice;
            let controller_signed_alice = Origin::signed(account_alice_controller);

            let claim_issuer_1 = 600;
            let (_claim_issuer_1_signed, claim_issuer_1_did) =
                make_account_with_uid(claim_issuer_1).unwrap();
            add_trusted_cdd_provider(claim_issuer_1_did);

            let account_eve = 700;
            let (_eve_signed, _eve_did) = make_account_with_balance(
                account_eve,
                1_000_000,
                Some(now.timestamp() as u64 + 7000u64),
            )
            .unwrap();

            let account_eve_controller = account_eve + 1;
            let controller_signed_eve = Origin::signed(account_eve_controller);

            // bond
            assert_ok!(Staking::bond(
                Origin::signed(account_alice),
                account_alice_controller,
                1000,
                RewardDestination::Stash
            ));

            // bond
            assert_ok!(Staking::bond(
                Origin::signed(account_eve),
                account_eve_controller,
                1000,
                RewardDestination::Stash
            ));

            now = Utc::now();
            set_timestamp(now.timestamp() as u64);
            let validators_1 = vec![10, 20, 30];
            assert_ok!(Staking::nominate(
                controller_signed_alice.clone(),
                validators_1
            ));
            assert!(!Staking::nominators(&account_alice).is_none());

            let validators_2 = vec![11, 21, 31];
            assert_ok!(Staking::nominate(
                controller_signed_eve.clone(),
                validators_2
            ));
            assert!(!Staking::nominators(&account_eve).is_none());
            now = Utc::now();
            set_timestamp((now.timestamp() as u64) + 800_u64);
            let claimed_nominator = vec![account_alice.clone(), account_eve.clone()];

            println!("Current timestamp: {:?}", Timestamp::now());

            assert_ok!(Staking::validate_cdd_expiry_nominators(
                Origin::signed(claim_issuer_1),
                claimed_nominator
            ));
            assert!(Staking::nominators(&account_alice).is_none());
            assert!(!Staking::nominators(&account_eve).is_none());

            let ledger_data = Staking::ledger(&account_alice_controller).unwrap();
            assert_eq!(ledger_data.active, 0);
            assert_eq!(ledger_data.unlocking.len(), 1);
        });
}

#[test]
fn should_initialize_stakers_and_validators() {
    // Verifies initial conditions of mock
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Staking::bonded(&11), Some(10)); // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(&21), Some(20)); // Account 21 is stashed and locked, and account 20 is the controller
        assert_eq!(Staking::bonded(&500), None); // Account 1 is not a stashed

        // Account 10 controls the stash from account 11, which is 100 * balance_factor units
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
        // Account 20 controls the stash from account 21, which is 200 * balance_factor units
        assert_eq!(
            Staking::ledger(&20),
            Some(StakingLedger {
                stash: 21,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
        // Account 1 does not control any stash
        assert_eq!(Staking::ledger(&500), None);
    });
}

#[test]
fn should_add_permissioned_validators() {
    ExtBuilder::default().build().execute_with(|| {
        let acc_10 = 10;
        let acc_20 = 20;

        provide_did_to_user(10);
        provide_did_to_user(20);

        assert_add_permissioned_validator!(&acc_10);
        assert_add_permissioned_validator!(&acc_20);
        assert_present_identity!(&acc_10);
        assert_present_identity!(&acc_20);
    });
}

#[test]
fn should_remove_permissioned_validators() {
    ExtBuilder::default().build().execute_with(|| {
        let acc_10 = 10;
        let acc_20 = 20;
        let acc_30 = 30;

        provide_did_to_user(10);
        provide_did_to_user(20);
        provide_did_to_user(30);

        assert_add_permissioned_validator!(&acc_10);
        assert_add_permissioned_validator!(&acc_20);

        assert_ok!(Staking::remove_permissioned_validator(
            Origin::signed(2000),
            Identity::get_identity(&acc_20).unwrap()
        ));

        assert_present_identity!(&acc_10);
        assert_absent_identity!(&acc_20);
        assert_absent_identity!(&acc_30);
    });
}

#[ignore]
#[test]
fn check_whether_nominator_selected_or_not_when_its_cdd_claim_expired() {
    ExtBuilder::default()
        .validator_count(3)
        .nominate(true)
        .build()
        .execute_with(|| {
            let bonding_duration: u64 = 90;

            start_active_era(1);

            let now = Timestamp::now();

            // 1. Add multiple nominators with some expiry
            // controller - 186, val - 2000, expiry - now + bonding_duration + 1000_u64, target - vec![11, 31]
            bond_nominator_with_expiry(186, 2000, now + bonding_duration + 1000_u64, vec![11, 31]);

            // verify nominator is added or not
            assert!(Staking::nominators(187).is_some());

            // controller - 196, val - 2000, expiry - now + bonding_duration + 4000_u64, target - vec![11, 31, 21]
            bond_nominator_with_expiry(
                196,
                2000,
                now + bonding_duration + 4000_u64,
                vec![11, 31, 21],
            );

            // verify nominator is added or not
            assert!(Staking::nominators(197).is_some());

            // controller - 206, val - 2000, expiry - now + bonding_duration + 4000_u64, target - vec![11, 31, 21]
            bond_nominator_with_expiry(
                206,
                2000,
                now + bonding_duration + 4000_u64,
                vec![11, 31, 21],
            );

            // verify nominator is added or not
            assert!(Staking::nominators(207).is_some());

            // controller - 216, val - 2000, expiry - now + bonding_duration + 1500_u64, target - vec![11, 21]
            bond_nominator_with_expiry(216, 2000, now + bonding_duration + 1500_u64, vec![11, 21]);

            // verify nominator is added or not
            assert!(Staking::nominators(217).is_some());

            // 3. change the era
            start_active_era(2);

            // validators of the new era
            let validators = Session::validators();
            assert!(validators.contains(&11));
            assert!(validators.contains(&21));
            assert!(validators.contains(&31));

            // 4. validate whether the expired nominators are the part of the individual exposure or not.
            let init_active_era = Staking::active_era().unwrap().index;

            let others = Staking::eras_stakers(init_active_era, 11).others;
            assert_eq!(others.len(), 3);
            assert_eq!(others[0].who, 207);
            assert_eq!(others[1].who, 101);
            assert_eq!(others[2].who, 197);

            let others = Staking::eras_stakers(init_active_era, 21).others;
            assert_eq!(others.len(), 3);
            assert_eq!(others[0].who, 207);
            assert_eq!(others[1].who, 101);
            assert_eq!(others[2].who, 197);

            let others = Staking::eras_stakers(init_active_era, 31).others;
            assert_eq!(others.len(), 2);
            assert_eq!(others[0].who, 207);
            assert_eq!(others[1].who, 197);
        });
}

type Pips = pallet_pips::Module<Test>;
type PError = pallet_pips::Error<Test>;

#[test]
fn voting_for_pip_overlays_with_staking() {
    use crate::staking::mock::RuntimeCall;

    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        // Initialize with 100 POLYX.
        let alice_acc = 500;
        let (alice_signer, _) = make_account_with_balance(alice_acc, 100, None).unwrap();

        let alice_proposal = |deposit: u128| {
            let signer = Origin::signed(alice_acc);
            let proposal = Box::new(RuntimeCall::Pips(
                pallet_pips::Call::set_min_proposal_deposit { deposit: 0 },
            ));
            Pips::propose(signer, proposal, deposit, None, None)
        };

        // Bond all but 10.
        assert_ok!(Staking::bond(
            alice_signer,
            alice_acc,
            90,
            RewardDestination::Stash
        ));
        // OK, because we're overlaying with 90 tokens already locked.
        assert_ok!(alice_proposal(50));
        // OK, because we're still overlaying, but also increasing it by 10.
        assert_ok!(alice_proposal(50));
        // Error, because we don't have 101 tokens to bond.
        assert_noop!(alice_proposal(1), PError::InsufficientDeposit);
    });
}

#[test]
fn slashing_leaves_pips_untouched() {
    use crate::staking::mock::RuntimeCall;
    use pallet_pips::PipId;

    ExtBuilder::default().build_and_execute(|| {
        let acc = 11;
        let propose = |deposit| {
            let signer = Origin::signed(acc);
            let proposal = Box::new(RuntimeCall::Pips(pallet_pips::Call::set_active_pip_limit {
                limit: 0,
            }));
            Pips::propose(signer, proposal, deposit, None, None)
        };
        let slash = |amount| {
            let exposure = Exposure {
                total: amount,
                own: amount,
                others: vec![],
            };
            let details = OffenceDetails {
                offender: (acc, exposure),
                reporters: vec![],
            };
            on_offence_now(&[details], &[Perbill::from_percent(100)]);
        };
        let balance_is = |bal| {
            assert_eq!(Balances::free_balance(acc), bal);
        };
        let id = PipId(0);
        let vote_is = |bal| {
            Pips::proposals(id).unwrap();
            assert_eq!(Pips::proposal_vote(id, acc).unwrap().1, bal);
        };
        let vote = |bal| Pips::vote(Origin::signed(acc), id, true, bal);

        // Ensure we start with 1000 balance.
        balance_is(1000);

        // Raise min deposit to 1000.
        assert_ok!(Pips::set_prune_historical_pips(root(), false));
        assert_ok!(Pips::set_min_proposal_deposit(root(), 1000));

        assert_ok!(propose(1000));

        // Fall below minimum deposit; still have 1000 as vote, and proposal exists.
        slash(700);
        balance_is(300);
        vote_is(1000);
        assert_ok!(vote(1000));

        // Lower account balance to 1; ditto.
        start_active_era(12);
        slash(299);
        balance_is(1);
        vote_is(1000);

        // Cannot propose anything; still locked 1000 into PIPs.
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        assert_noop!(propose(2), PError::InsufficientDeposit);

        // Lower vote to 1; locked decreases to 1.
        assert_ok!(vote(1));
        vote_is(1);

        // Slash again, we'll have 0 in balance now, but 1 still locked.
        slash(1);
        vote_is(1);
        assert_ok!(vote(1));
    });
}

#[ignore]
#[test]
fn test_with_multiple_validators_from_entity() {
    ExtBuilder::default()
        .validator_count(8)
        .minimum_validator_count(5)
        .build()
        .execute_with(|| {
            start_active_era(1);

            // add new validator
            bond_validator_with_intended_count(50, 51, 500000, Some(3));

            // Add other stash and controller to the same did.
            // 60 stash and 61 controller.
            add_secondary_key(50, 60);
            add_secondary_key(50, 61);

            assert_ok!(Staking::bond(
                Origin::signed(60),
                61,
                700000,
                RewardDestination::Controller,
            ));
            let entity_id = Identity::get_identity(&60).unwrap();
            if Staking::permissioned_identity(entity_id).is_none() {
                assert_add_permissioned_validator!(&60);
            }
            assert_ok!(Staking::validate(
                Origin::signed(61),
                ValidatorPrefs::default()
            ));

            start_active_era(2);

            assert!(Session::validators().contains(&50));
            assert!(Session::validators().contains(&60));
        });
}

#[ignore]
#[test]
fn test_reward_scheduling() {
    ExtBuilder::default()
        .validator_count(2)
        .nominate(true)
        .build()
        .execute_with(|| {
            let init_balance_10 = Balances::total_balance(&10);
            let init_balance_11 = Balances::total_balance(&11);
            let init_balance_20 = Balances::total_balance(&20);
            let init_balance_21 = Balances::total_balance(&21);
            let init_balance_100 = Balances::total_balance(&100);
            let init_balance_101 = Balances::total_balance(&101);

            // Check state
            Payee::<Test>::insert(11, RewardDestination::Controller);
            Payee::<Test>::insert(21, RewardDestination::Controller);
            Payee::<Test>::insert(101, RewardDestination::Controller);

            <Module<Test>>::reward_by_ids(vec![(11, 50)]);
            <Module<Test>>::reward_by_ids(vec![(11, 50)]);
            // This is the second validator of the current elected set.
            <Module<Test>>::reward_by_ids(vec![(21, 50)]);

            // Compute total payout now for whole duration as other parameter won't change
            let total_payout_0 = current_total_payout_for_duration(3 * 1000);
            assert!(total_payout_0 > 10); // Test is meaningful if reward something

            println!("Print the total payout for the reward {:?}", total_payout_0);

            start_session(1);

            assert_eq!(Balances::total_balance(&10), init_balance_10);
            assert_eq!(Balances::total_balance(&11), init_balance_11);
            assert_eq!(Balances::total_balance(&20), init_balance_20);
            assert_eq!(Balances::total_balance(&21), init_balance_21);
            assert_eq!(Balances::total_balance(&100), init_balance_100);
            assert_eq!(Balances::total_balance(&101), init_balance_101);
            assert_eq_uvec!(Session::validators(), vec![21, 11]);
            assert_eq!(
                Staking::eras_reward_points(Staking::active_era().unwrap().index),
                EraRewardPoints {
                    total: 50 * 3,
                    individual: vec![(11, 100), (21, 50)].into_iter().collect(),
                }
            );
            let _part_for_10 = Perbill::from_rational::<u32>(1000, 1125);
            let _part_for_20 = Perbill::from_rational::<u32>(1000, 1375);
            let part_for_100_from_10 = Perbill::from_rational::<u32>(125, 1125);
            let part_for_100_from_20 = Perbill::from_rational::<u32>(375, 1375);

            start_session(2);
            start_session(3);

            assert_eq!(Staking::active_era().unwrap().index, 1);
            assert_eq!(
                mock::REWARD_REMAINDER_UNBALANCED.with(|v| *v.borrow()),
                33225
            );

            let current_block_no = System::block_number();
            // Check whether the rewards get scheduled or not.
            assert_eq!(
                pallet_scheduler::Agenda::<Test>::get(current_block_no + 1).len(),
                1
            );
            assert_eq!(
                pallet_scheduler::Agenda::<Test>::get(current_block_no + 2).len(),
                1
            );

            run_to_block_scheduler(current_block_no + 1);

            assert_eq_error_rate!(Balances::total_balance(&20), 2686, 2);
            assert_eq_error_rate!(Balances::total_balance(&21), init_balance_21, 2);

            // Balance of 10 & 11 remain same as reward scheduled only for 21
            assert_eq!(Balances::total_balance(&10), init_balance_10);
            assert_eq!(Balances::total_balance(&11), init_balance_11);

            // Execute scheduling for the validator 11
            run_to_block_scheduler(current_block_no + 2);

            assert_eq_error_rate!(Balances::total_balance(&10), 6564, 2);
            assert_eq_error_rate!(Balances::total_balance(&11), init_balance_11, 2);

            assert_eq_error_rate!(
                Balances::total_balance(&100),
                init_balance_100
                    + part_for_100_from_10 * total_payout_0 * 2 / 3
                    + part_for_100_from_20 * total_payout_0 * 1 / 3,
                2
            );
            assert_eq_error_rate!(Balances::total_balance(&101), init_balance_101, 2);
        });
}

#[test]
fn check_slashing_switch_for_validators_and_nominators() {
    ExtBuilder::default()
        .slashing_allowed_for(SlashingSwitch::None)
        .validator_count(4)
        .build()
        .execute_with(|| {
            // Check the initial state of the Slashing Switch.
            assert_eq!(Staking::slashing_allowed_for(), SlashingSwitch::None);

            let change_slashing_allowed_for = |switch: SlashingSwitch| {
                assert_ok!(Staking::change_slashing_allowed_for(root(), switch));
                assert_eq!(Staking::slashing_allowed_for(), switch);
            };

            change_slashing_allowed_for(SlashingSwitch::Validator);
            change_slashing_allowed_for(SlashingSwitch::ValidatorAndNominator);
        });
}

#[test]
fn offence_is_blocked_when_slashing_status_is_off() {
    ExtBuilder::default()
        .offchain_election_ext()
        .validator_count(4)
        .slashing_allowed_for(SlashingSwitch::None)
        .has_stakers(false)
        .build()
        .execute_with(|| {
            assert_eq!(Staking::slashing_allowed_for(), SlashingSwitch::None);
            let initial_balance = Balances::free_balance(10);
            create_on_offence_now(10);
            // No slashing happened.
            assert_eq!(Balances::free_balance(10), initial_balance);
        });
}

#[test]
fn check_slashing_for_different_switches() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);

        assert_eq!(Balances::free_balance(11), 1000);
        assert_eq!(Balances::free_balance(21), 2000);

        // Switch to ValidatorAndNominator.
        assert_ok!(Staking::change_slashing_allowed_for(
            root(),
            SlashingSwitch::ValidatorAndNominator
        ));
        assert_eq!(
            Staking::slashing_allowed_for(),
            SlashingSwitch::ValidatorAndNominator
        );

        // Add nominator.
        // add a new candidate for being a nominator. account 3 controlled by 4.
        bond_nominator_with_expiry(3, 2000, 99999999, vec![11, 21]);
        add_secondary_key(4, 3);

        mock::start_active_era(2);

        assert_eq!(Balances::free_balance(101), 2000);

        create_on_offence_now(11);

        create_on_offence_now(21);

        // Balance of account 11 [validator] get slashed by 10% i.e 10 % of 1000 (total staked).
        assert_eq!(Balances::free_balance(11), 900);
        // Balance of account 4 [nominator] get slashed by 10% i.e 10 % of 2000 (total staked).
        assert_eq!(Balances::free_balance(4), 1800);
        // Balance of account 21 [validator] get slashed by 10% i.e 10 % of 1000 (total staked).
        assert_eq!(Balances::free_balance(21), 1900);
    })
}

fn create_on_offence_now(offender: u64) {
    on_offence_now(
        &[OffenceDetails {
            offender: (
                offender,
                Staking::eras_stakers(Staking::active_era().unwrap().index, offender),
            ),
            reporters: vec![],
        }],
        &[Perbill::from_percent(10)],
    );
}

#[test]
fn payout_to_any_account_works() {
    ExtBuilder::default()
        .has_stakers(false)
        .build_and_execute(|| {
            let balance = 1000;
            // Create a validator:
            bond_validator(11, 10, balance); // Default(64)

            // Create a stash/controller pair
            bond_nominator_cdd(1234, 1337, 100, vec![11]);

            // Update payout location
            assert_ok!(Staking::set_payee(
                Origin::signed(1337),
                RewardDestination::Account(42)
            ));

            // Reward Destination account doesn't exist
            assert_eq!(Balances::free_balance(42), 0);

            mock::start_active_era(1);
            Staking::reward_by_ids(vec![(11, 1)]);
            // Compute total payout now for whole duration as other parameters won't change
            let total_payout_0 = current_total_payout_for_duration(3 * 1000);
            assert!(total_payout_0 > 100); // Test is meaningful if something is rewarded.
            mock::start_active_era(2);
            assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 1));

            // Payment is successful
            assert!(Balances::free_balance(42) > 0);
        })
}
#[ignore]
#[test]
fn test_multiple_validators_from_an_entity() {
    ExtBuilder::default()
        .validator_count(8)
        .minimum_validator_count(5)
        .build()
        .execute_with(|| {
            start_active_era(1);

            // Add new validator
            bond(50, 51, 500000);
            let entity_id = Identity::get_identity(&50).unwrap();
            assert_noop!(
                Staking::add_permissioned_validator(
                    frame_system::RawOrigin::Root.into(),
                    entity_id,
                    Some(6)
                ),
                Error::<Test>::IntendedCountIsExceedingConsensusLimit
            );

            // Add a new validator successfully.
            bond_validator_with_intended_count(50, 51, 500000, Some(2));

            assert_permissioned_identity_prefs!(entity_id, 2, 1);

            // Adding the same validator again should not increment the `running_count`
            bond_validator_with_intended_count(50, 51, 500000, Some(2));
            assert_permissioned_identity_prefs!(entity_id, 2, 1);

            // Add other stash and controller to the same did.
            // 60 stash and 61 controller.
            add_secondary_key(50, 60);
            add_secondary_key(50, 61);

            // Validate one more validator from the same entity.
            bond_validator(60, 61, 500000);
            assert_permissioned_identity_prefs!(entity_id, 2, 2);

            // Add other stash and controller to the same did.
            // 70 stash and 71 controller.
            add_secondary_key(50, 70);
            add_secondary_key(50, 71);

            bond(70, 71, 500000);
            assert_noop!(
                Staking::validate(Origin::signed(71), ValidatorPrefs::default()),
                Error::<Test>::HitIntendedValidatorCount
            );

            // Can change commission of existing nodes,
            // even after hitting the operator node limit.
            ValidatorCommissionCap::put(Perbill::one());
            assert_ok!(Staking::validate(
                Origin::signed(51),
                ValidatorPrefs {
                    commission: Perbill::one(),
                    ..Default::default()
                }
            ));

            // Increase the validator count
            assert_ok!(Staking::update_permissioned_validator_intended_count(
                root(),
                entity_id,
                3
            ));

            assert_ok!(Staking::validate(
                Origin::signed(71),
                ValidatorPrefs::default()
            ));

            assert_permissioned_identity_prefs!(entity_id, 3, 3);

            // Making a validator chill status.
            assert_ok!(Staking::chill(Origin::signed(61)));

            assert_permissioned_identity_prefs!(entity_id, 3, 2);

            // Chilling an already disabled validator should not reduce the `running_count`
            assert_ok!(Staking::chill(Origin::signed(61)));
            assert_permissioned_identity_prefs!(entity_id, 3, 2);
        });
}

#[test]
fn test_bond_too_small() {
    ExtBuilder::default().build().execute_with(|| {
        let pre_bond = |stash, ctrl, val| {
            let _ = Balances::make_free_balance_be(&stash, val);
            let _ = Balances::make_free_balance_be(&ctrl, val);
            provide_did_to_user(stash);
            add_secondary_key(stash, ctrl);
        };
        let bond = |stash, ctrl, val| {
            Staking::bond(
                Origin::signed(stash),
                ctrl,
                val,
                RewardDestination::Controller,
            )
        };
        let both = |stash, ctrl, val| {
            pre_bond(stash, ctrl, val);
            bond(stash, ctrl, val)
        };

        pre_bond(50, 51, MinimumBond::get() - 1);
        assert_noop!(
            bond(50, 51, MinimumBond::get() - 1),
            Error::<Test>::BondTooSmall
        );
        assert_ok!(both(70, 71, MinimumBond::get()));
        assert_ok!(both(90, 91, MinimumBond::get() + 1));
    });
}

#[test]
fn validator_unbonding() {
    ExtBuilder::default()
        .validator_count(8)
        .minimum_validator_count(1)
        .build()
        .execute_with(|| {
            // Add a new validator successfully.
            bond_validator_with_intended_count(50, 51, 500_000, Some(2));
            let entity_id = Identity::get_identity(&50).unwrap();
            assert_permissioned_identity_prefs!(entity_id, 2, 1);
            // Set minimum bond threshold to 50k POLYX
            assert_ok!(Staking::set_min_bond_threshold(Origin::root(), 50_000));
            // Check that an error is given when unbonding beyond the minimum bond threshold as a validator
            assert_noop!(
                Staking::unbond(Origin::signed(51), 480_000),
                Error::<Test>::InvalidValidatorUnbondAmount
            );
            // Check that validator can unbond once it doesn't go below the minimum bond threshold
            assert_ok!(Staking::unbond(Origin::signed(51), 80_000));
            assert_ok!(Staking::unbond(Origin::signed(51), 80_000));
            assert_ok!(Staking::unbond(Origin::signed(51), 80_000));
            assert_ok!(Staking::unbond(Origin::signed(51), 80_000));

            // Check the remaining bond amount can't all be unbond
            assert_noop!(
                Staking::unbond(Origin::signed(51), 180_000),
                Error::<Test>::InvalidValidatorUnbondAmount
            );
            // Chill validator
            assert_ok!(Staking::chill(Origin::signed(51)));
            // After chilling validator checks that entity can unbond successfully
            assert_ok!(Staking::unbond(Origin::signed(51), 180_000));
        });
}

#[test]
fn chill_from_governance() {
    ExtBuilder::default()
        .validator_count(8)
        .minimum_validator_count(1)
        .build()
        .execute_with(|| {
            // 50 stash and 51 controller (corrected)
            bond(50, 51, 500000);
            let entity_id = Identity::get_identity(&50).unwrap();

            // Add a new validator successfully.
            bond_validator_with_intended_count(50, 51, 500000, Some(2));
            assert_permissioned_identity_prefs!(entity_id, 2, 1);

            // Add other stash and controller to the same did.
            add_secondary_key(50, 60);
            add_secondary_key(50, 61);

            // Validate one more validator from the same entity.
            // 60 stash and 61 controller.
            bond_validator(60, 61, 500000);
            assert_permissioned_identity_prefs!(entity_id, 2, 2);

            // Removes 50 and 60 from being validators
            assert_ok!(Staking::chill_from_governance(
                Origin::signed(2000),
                entity_id,
                vec![50, 60]
            ));

            // No longer permissioned identity
            assert_noop!(
                Staking::chill_from_governance(Origin::signed(2000), entity_id, vec![50, 60]),
                Error::<Test>::NotExists
            );

            // 70 stash and 71 controller
            bond(70, 71, 500000);
            let entity_id_2 = Identity::get_identity(&70).unwrap();

            // Add a new validator successfully.
            bond_validator_with_intended_count(70, 71, 500000, Some(2));

            // Add other stash and controller to the same did.
            add_secondary_key(70, 80);
            add_secondary_key(70, 81);

            // Check keys that aren't joined with identity gives error
            assert_noop!(
                Staking::chill_from_governance(Origin::signed(2000), entity_id_2, vec![90, 95]),
                Error::<Test>::NotStash
            );

            // Check key that is not GC gives error
            assert_noop!(
                Staking::chill_from_governance(Origin::signed(20), entity_id_2, vec![90, 95]),
                BadOrigin
            );
        });
}

#[test]
fn test_running_count() {
    ExtBuilder::default()
        .validator_count(8)
        .minimum_validator_count(1)
        .build()
        .execute_with(|| {
            // 50 stash and 51 controller
            bond(50, 51, 500000);
            let entity_id = Identity::get_identity(&50).unwrap();

            // Add a new validator successfully.
            bond_validator_with_intended_count(50, 51, 500000, Some(2));
            assert_permissioned_identity_prefs!(entity_id, 2, 1);

            // Add other stash and controller to the same did.
            add_secondary_key(50, 60);
            add_secondary_key(50, 61);

            // Validate one more validator from the same entity.
            // 60 stash and 61 controller.
            bond_validator(60, 61, 500000);
            assert_permissioned_identity_prefs!(entity_id, 2, 2);

            // Ensure that the validator's stash key can't be removed from it's identity.
            assert_noop!(
                Identity::remove_secondary_keys(Origin::signed(50), vec![60]),
                IdentityError::<Test>::AccountKeyIsBeingUsed
            );

            // chill to remove the validator
            assert_ok!(Staking::chill(Origin::signed(61)));

            // remove validator's stash key from it's identity
            assert_ok!(Identity::remove_secondary_keys(
                Origin::signed(50),
                vec![60]
            ));
        });
}
