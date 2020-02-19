// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Tests for the module.

use super::*;
use chrono::prelude::Utc;
use frame_support::{
    assert_err, assert_noop, assert_ok,
    dispatch::DispatchError,
    traits::{Currency, ReservableCurrency},
};
use mock::*;
use polymesh_runtime_common::traits::identity::{ClaimValue, DataTypes};
use sp_runtime::{
    assert_eq_error_rate,
    traits::{BadOrigin, OnInitialize},
};
use sp_staking::offence::OffenceDetails;
use substrate_test_utils::assert_eq_uvec;
use test_client::AccountKeyring;

#[test]
fn force_unstake_works() {
    // Verifies initial conditions of mock
    ExtBuilder::default().build().execute_with(|| {
        // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(account_from(11)), Some(account_from(10)));
        // Cant transfer
        assert_noop!(
            Balances::transfer(Origin::signed(account_from(11)), account_from(1), 10),
            DispatchError::Module {
                index: 0,
                error: 1,
                message: Some("LiquidityRestrictions"),
            }
        );
        // Force unstake requires root.
        assert_noop!(
            Staking::force_unstake(Origin::signed(account_from(11)), account_from(11)),
            BadOrigin
        );
        // We now force them to unstake
        assert_ok!(Staking::force_unstake(Origin::ROOT, account_from(11)));
        // No longer bonded.
        assert_eq!(Staking::bonded(account_from(11)), None);
        // Transfer works.
        assert_ok!(Balances::transfer(
            Origin::signed(account_from(11)),
            account_from(1),
            10
        ));
    });
}

#[test]
fn basic_setup_works() {
    // Verifies initial conditions of mock
    ExtBuilder::default().build().execute_with(|| {
        // Set min bond value to be positive integer
        <MinimumBondThreshold<Test>>::put(10);
        // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(account_from(11)), Some(account_from(10)));
        // Account 21 is stashed and locked, and account 20 is the controller
        assert_eq!(Staking::bonded(account_from(21)), Some(account_from(20)));
        // Account 1 is not a stashed
        assert_eq!(Staking::bonded(account_from(1)), None);

        // Account 10 controls the stash from account 11, which is 100 * balance_factor units
        assert_eq!(
            Staking::ledger(account_from(10)),
            Some(StakingLedger {
                stash: account_from(11),
                total: 1000,
                active: 1000,
                unlocking: vec![]
            })
        );
        // Account 20 controls the stash from account 21, which is 200 * balance_factor units
        assert_eq!(
            Staking::ledger(account_from(20)),
            Some(StakingLedger {
                stash: account_from(21),
                total: 1000,
                active: 1000,
                unlocking: vec![]
            })
        );
        // Account 1 does not control any stash
        assert_eq!(Staking::ledger(account_from(1)), None);

        // ValidatorPrefs are default
        assert_eq!(
            <Validators<Test>>::enumerate().collect::<Vec<_>>(),
            vec![
                (account_from(31), ValidatorPrefs::default()),
                (account_from(21), ValidatorPrefs::default()),
                (account_from(11), ValidatorPrefs::default())
            ]
        );

        assert_eq!(
            Staking::ledger(account_from(100)),
            Some(StakingLedger {
                stash: account_from(101),
                total: 500,
                active: 500,
                unlocking: vec![]
            })
        );

        // To work with below assert statement
        fix_nominator_genesis_problem(2250);

        assert_eq!(
            Staking::nominators(account_from(101)).unwrap().targets,
            vec![account_from(11), account_from(21)]
        );

        start_era(1);

        if cfg!(feature = "equalize") {
            assert_eq!(
                Staking::stakers(account_from(11)),
                Exposure {
                    total: 1250,
                    own: 1000,
                    others: vec![IndividualExposure {
                        who: account_from(101),
                        value: 250
                    }]
                }
            );
            assert_eq!(
                Staking::stakers(account_from(21)),
                Exposure {
                    total: 1250,
                    own: 1000,
                    others: vec![IndividualExposure {
                        who: account_from(101),
                        value: 250
                    }]
                }
            );
            // initial slot_stake
            assert_eq!(Staking::slot_stake(), 1250);
        } else {
            assert_eq!(
                Staking::stakers(account_from(11)),
                Exposure {
                    total: 1125,
                    own: 1000,
                    others: vec![IndividualExposure {
                        who: account_from(101),
                        value: 125
                    }]
                }
            );
            assert_eq!(
                Staking::stakers(account_from(21)),
                Exposure {
                    total: 1375,
                    own: 1000,
                    others: vec![IndividualExposure {
                        who: account_from(101),
                        value: 375
                    }]
                }
            );
            // initial slot_stake
            assert_eq!(Staking::slot_stake(), 1125);
        }

        // The number of validators required.
        assert_eq!(Staking::validator_count(), 2);

        // Initial Era and session
        assert_eq!(Staking::current_era(), 1);

        // Account 10 has `balance_factor` free balance
        assert_eq!(Balances::free_balance(account_from(10)), 1);
        assert_eq!(Balances::free_balance(account_from(10)), 1);

        // New era is not being forced
        assert_eq!(Staking::force_era(), Forcing::NotForcing);

        // All exposures must be correct.
        check_exposure_all();
        check_nominator_all();
    });
}

#[test]
fn change_controller_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Staking::bonded(account_from(11)), Some(account_from(10)));

        assert!(<Validators<Test>>::enumerate()
            .map(|(c, _)| c)
            .collect::<Vec<AccountId>>()
            .contains(&account_from(11)));
        // 10 can control 11 who is initially a validator.
        assert_ok!(Staking::chill(Origin::signed(account_from(10))));
        assert!(!<Validators<Test>>::enumerate()
            .map(|(c, _)| c)
            .collect::<Vec<AccountId>>()
            .contains(&account_from(11)));

        assert_ok!(Staking::set_controller(
            Origin::signed(account_from(11)),
            account_from(5)
        ));

        start_era(1);

        assert_noop!(
            Staking::validate(Origin::signed(account_from(10)), ValidatorPrefs::default()),
            Error::<Test>::NotController,
        );
        assert_ok!(Staking::validate(
            Origin::signed(account_from(5)),
            ValidatorPrefs::default()
        ));
    })
}

#[test]
fn rewards_should_work() {
    // should check that:
    // * rewards get recorded per session
    // * rewards get paid per Era
    // * Check that nominators are also rewarded
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            // To work with below assert statement
            fix_nominator_genesis_problem(2250);
            // Start new era to reflect the change
            start_era(1);

            // Init some balances
            let _ = Balances::make_free_balance_be(&account_from(2), 500);

            let delay = 1000;
            let init_balance_2 = Balances::total_balance(&account_from(2));
            let init_balance_10 = Balances::total_balance(&account_from(10));
            let init_balance_11 = Balances::total_balance(&account_from(11));

            // Set payee to controller
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(10)),
                RewardDestination::Controller
            ));

            // Initial config should be correct
            assert_eq!(Staking::current_era(), 1);
            assert_eq!(Session::current_index(), 4);

            // Add a dummy nominator.
            //
            // Equal division indicates that the reward will be equally divided among validator and
            // nominator.
            <Stakers<Test>>::insert(
                &account_from(11),
                Exposure {
                    own: 500,
                    total: 1000,
                    others: vec![IndividualExposure {
                        who: account_from(2),
                        value: 500,
                    }],
                },
            );

            <Payee<Test>>::insert(&account_from(2), RewardDestination::Stash);
            assert_eq!(Staking::payee(account_from(2)), RewardDestination::Stash);
            assert_eq!(
                Staking::payee(account_from(11)),
                RewardDestination::Controller
            );

            let mut block = 3; // Block 3 => Session 5 => Era 1
            System::set_block_number(block);
            Timestamp::set_timestamp(block * 5000); // on time.
            Session::on_initialize(System::block_number());
            assert_eq!(Staking::current_era(), 1);
            assert_eq!(Session::current_index(), 5);
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 50)]);
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 50)]);
            // This is the second validator of the current elected set.
            <Module<Test>>::reward_by_ids(vec![(account_from(21), 50)]);
            // This must be no-op as it is not an elected validator.
            <Module<Test>>::reward_by_ids(vec![(account_from(1001), 10_000)]);

            // Compute total payout now for whole duration as other parameter won't change
            let total_payout = current_total_payout_for_duration(9 * 5 * 1000);
            assert!(total_payout > 10); // Test is meaningful if reward something

            // No reward yet
            assert_eq!(Balances::total_balance(&account_from(2)), init_balance_2);
            assert_eq!(Balances::total_balance(&account_from(10)), init_balance_10);
            assert_eq!(Balances::total_balance(&account_from(11)), init_balance_11);

            block = 6; // Block 6 => Session 6 => Era 1
            System::set_block_number(block);
            Timestamp::set_timestamp(block * 5000 + delay); // a little late.
            Session::on_initialize(System::block_number());
            assert_eq!(Staking::current_era(), 2);
            assert_eq!(Session::current_index(), 6);

            block = 9; // Block 9 => Session 7 => Era 2
            System::set_block_number(block);
            Timestamp::set_timestamp(block * 5000); // back to being on time. no delays
            Session::on_initialize(System::block_number());
            assert_eq!(Staking::current_era(), 2);
            assert_eq!(Session::current_index(), 7);

            assert_eq!(Balances::total_balance(&account_from(11)), init_balance_11);
        });
}

#[test]
fn multi_era_reward_should_work() {
    // Should check that:
    // The value of current_session_reward is set at the end of each era, based on
    // slot_stake and session_reward.
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            let init_balance_10 = Balances::total_balance(&account_from(10));

            // Set payee to controller
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(10)),
                RewardDestination::Controller
            ));

            // Compute now as other parameter won't change
            let total_payout_0 = current_total_payout_for_duration(3000);
            assert!(total_payout_0 > 10); // Test is meaningfull if reward something
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);

            start_session(0);
            start_session(1);
            start_session(2);
            start_session(3);

            assert_eq!(Staking::current_era(), 1);
            assert_eq!(
                Balances::total_balance(&account_from(10)),
                init_balance_10 + total_payout_0
            );

            start_session(4);

            let total_payout_1 = current_total_payout_for_duration(3000);
            assert!(total_payout_1 > 10); // Test is meaningfull if reward something
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 101)]);

            // new era is triggered here.
            start_session(5);

            // pay time
            assert_eq!(
                Balances::total_balance(&account_from(10)),
                init_balance_10 + total_payout_0 + total_payout_1
            );
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
            Timestamp::set_timestamp(1); // Initialize time.

            // remember + compare this along with the test.
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // put some money in account that we'll use.
            for i in 1..5 {
                let _ = Balances::make_free_balance_be(&account_from(i), 2000);
            }

            // --- Block 1:
            start_session(1);
            // Pass compliance for account 3
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(3)
            ));
            // add a new candidate for being a validator. account 3 controlled by 4.
            assert_ok!(Staking::bond(
                Origin::signed(account_from(3)),
                account_from(4),
                1500,
                RewardDestination::Controller
            ));
            assert_ok!(Staking::validate(
                Origin::signed(account_from(4)),
                ValidatorPrefs::default()
            ));

            // No effects will be seen so far.
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // --- Block 2:
            start_session(2);

            // No effects will be seen so far. Era has not been yet triggered.
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // --- Block 3: the validators will now be queued.
            start_session(3);
            assert_eq!(Staking::current_era(), 1);

            // --- Block 4: the validators will now be changed.
            start_session(4);

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(4)]
            );
            // --- Block 4: Unstake 4 as a validator, freeing up the balance stashed in 3
            // 4 will chill
            Staking::chill(Origin::signed(account_from(4))).unwrap();

            // --- Block 5: nothing. 4 is still there.
            start_session(5);
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(4)]
            );

            // --- Block 6: 4 will not be a validator.
            start_session(7);
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // Note: the stashed value of 4 is still lock
            assert_eq!(
                Staking::ledger(&account_from(4)),
                Some(StakingLedger {
                    stash: account_from(3),
                    total: 1500,
                    active: 1500,
                    unlocking: vec![]
                })
            );
            // e.g. it cannot spend more than 500 that it has free from the total 2000
            assert_noop!(
                Balances::reserve(&account_from(3), 501),
                DispatchError::Module {
                    index: 0,
                    error: 1,
                    message: Some("LiquidityRestrictions"),
                }
            );
            assert_ok!(Balances::reserve(&account_from(3), 409));
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
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(30), account_from(20), account_from(10)]
            );

            start_era(1);

            // Previous set is selected. NO election algorithm is even executed.
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(30), account_from(20), account_from(10)]
            );

            // But the exposure is updated in a simple way. No external votes exists.
            // This is purely self-vote.
            assert_eq!(Staking::stakers(account_from(10)).others.len(), 0);
            assert_eq!(Staking::stakers(account_from(20)).others.len(), 0);
            assert_eq!(Staking::stakers(account_from(30)).others.len(), 0);
            check_exposure_all();
            check_nominator_all();
        });
}

#[test]
fn no_candidate_emergency_condition() {
    ExtBuilder::default()
        .minimum_validator_count(10)
        .validator_count(15)
        .num_validators(4)
        .validator_pool(true)
        .nominate(false)
        .build()
        .execute_with(|| {
            // initial validators
            assert_eq_uvec!(
                validator_controllers(),
                vec![
                    account_from(10),
                    account_from(20),
                    account_from(30),
                    account_from(40)
                ]
            );

            // set the minimum validator count.
            <Staking as crate::Store>::MinimumValidatorCount::put(10);
            <Staking as crate::Store>::ValidatorCount::put(15);
            assert_eq!(Staking::validator_count(), 15);

            let _ = Staking::chill(Origin::signed(account_from(10)));

            // trigger era
            System::set_block_number(1);
            Session::on_initialize(System::block_number());

            // Previous ones are elected. chill is invalidates. TODO: #2494
            assert_eq_uvec!(
                validator_controllers(),
                vec![
                    account_from(10),
                    account_from(20),
                    account_from(30),
                    account_from(40)
                ]
            );
            assert_eq!(Staking::current_elected().len(), 0);
        });
}

#[test]
fn nominating_and_rewards_should_work() {
    // PHRAGMEN OUTPUT: running this test with the reference impl gives:
    //
    // Sequential Phragmén gives
    // 10  is elected with stake  2200.0 and score  0.0003333333333333333
    // 20  is elected with stake  1800.0 and score  0.0005555555555555556

    // 10  has load  0.0003333333333333333 and supported
    // 10  with stake  1000.0
    // 20  has load  0.0005555555555555556 and supported
    // 20  with stake  1000.0
    // 30  has load  0 and supported
    // 30  with stake  0
    // 40  has load  0 and supported
    // 40  with stake  0
    // 2  has load  0.0005555555555555556 and supported
    // 10  with stake  600.0 20  with stake  400.0 30  with stake  0.0
    // 4  has load  0.0005555555555555556 and supported
    // 10  with stake  600.0 20  with stake  400.0 40  with stake  0.0

    // Sequential Phragmén with post processing gives
    // 10  is elected with stake  2000.0 and score  0.0003333333333333333
    // 20  is elected with stake  2000.0 and score  0.0005555555555555556

    // 10  has load  0.0003333333333333333 and supported
    // 10  with stake  1000.0
    // 20  has load  0.0005555555555555556 and supported
    // 20  with stake  1000.0
    // 30  has load  0 and supported
    // 30  with stake  0
    // 40  has load  0 and supported
    // 40  with stake  0
    // 2  has load  0.0005555555555555556 and supported
    // 10  with stake  400.0 20  with stake  600.0 30  with stake  0
    // 4  has load  0.0005555555555555556 and supported
    // 10  with stake  600.0 20  with stake  400.0 40  with stake  0.0
    ExtBuilder::default()
        .nominate(false)
        .validator_pool(true)
        .build()
        .execute_with(|| {
            // initial validators -- everyone is actually even.
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(40), account_from(30)]
            );

            // Set payee to controller
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(10)),
                RewardDestination::Controller
            ));
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(20)),
                RewardDestination::Controller
            ));
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(30)),
                RewardDestination::Controller
            ));
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(40)),
                RewardDestination::Controller
            ));

            // give the man some money
            let initial_balance = 1000;
            for i in [1, 2, 3, 4, 5, 10, 11, 20, 21].iter() {
                let _ = Balances::make_free_balance_be(&account_from(*i), initial_balance);
            }

            // bond two account pairs and state interest in nomination.
            // 2 will nominate for 10, 20, 30
            assert_ok!(Staking::bond(
                Origin::signed(account_from(1)),
                account_from(2),
                1000,
                RewardDestination::Controller
            ));

            // Provide a valid kyc claim for
            // stash - 1
            // controller - 2
            let (nominator_signed, nominator_did) =
                make_account_with_balance(account_from(1), 1_000_000).unwrap();

            let service_provider_account = AccountId::from(AccountKeyring::Bob);
            let (service_provider_signed, service_provider_did) =
                make_account(service_provider_account.clone()).unwrap();
            add_trusted_kyc_provider(service_provider_did);

            let now = Utc::now();
            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: (now.timestamp() as u64 + 1000_u64).to_be_bytes().to_vec(),
            };
            add_nominator_claim(
                service_provider_did,
                nominator_did,
                service_provider_account,
                account_from(1),
                claim.clone(),
            );

            assert_ok!(Staking::nominate(
                Origin::signed(account_from(2)),
                vec![account_from(11), account_from(21), account_from(31)]
            ));

            assert_eq!(
                Staking::nominators(account_from(1)).unwrap().targets,
                vec![account_from(11), account_from(21), account_from(31)]
            );

            // 4 will nominate for 10, 20, 40
            assert_ok!(Staking::bond(
                Origin::signed(account_from(3)),
                account_from(4),
                1000,
                RewardDestination::Controller
            ));

            // Provide a valid kyc claim for
            // stash - 3
            // controller - 4

            let (nominator_signed_2, nominator_did_2) =
                make_account_with_balance(account_from(3), 1_000_000).unwrap();

            add_nominator_claim(
                service_provider_did,
                nominator_did_2,
                service_provider_account,
                account_from(3),
                claim.clone(),
            );

            assert_ok!(Staking::nominate(
                Origin::signed(account_from(4)),
                vec![account_from(11), account_from(21), account_from(41)]
            ));
            assert_eq!(
                Staking::nominators(account_from(3)).unwrap().targets,
                vec![account_from(11), account_from(21), account_from(41)]
            );

            // the total reward for era 0
            let total_payout_0 = current_total_payout_for_duration(3000);
            assert!(total_payout_0 > 100); // Test is meaningfull if reward something
            <Module<Test>>::reward_by_ids(vec![(account_from(41), 1)]);
            <Module<Test>>::reward_by_ids(vec![(account_from(31), 1)]);
            <Module<Test>>::reward_by_ids(vec![(account_from(21), 10)]); // must be no-op
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 10)]); // must be no-op

            start_era(1);

            // 10 and 20 have more votes, they will be chosen by phragmen.
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // OLD validators must have already received some rewards.
            assert_eq!(
                Balances::total_balance(&account_from(40)),
                1 + total_payout_0 / 2
            );
            assert_eq!(
                Balances::total_balance(&account_from(30)),
                1 + total_payout_0 / 2
            );

            // ------ check the staked value of all parties.

            if cfg!(feature = "equalize") {
                // total expo of 10, with 1200 coming from nominators (externals), according to phragmen.
                assert_eq!(Staking::stakers(account_from(11)).own, 1000);
                assert_eq_error_rate!(Staking::stakers(account_from(11)).total, 1000 + 1000, 2);
                // 2 and 4 supported 10, each with stake 600, according to phragmen.
                assert_eq!(
                    Staking::stakers(account_from(11))
                        .others
                        .iter()
                        .map(|e| e.value)
                        .collect::<Vec<BalanceOf<Test>>>(),
                    vec![600, 400]
                );
                assert_eq!(
                    Staking::stakers(account_from(11))
                        .others
                        .iter()
                        .map(|e| e.who)
                        .collect::<Vec<AccountId>>(),
                    vec![account_from(3), account_from(1)]
                );
                // total expo of 20, with 500 coming from nominators (externals), according to phragmen.
                assert_eq!(Staking::stakers(account_from(21)).own, 1000);
                assert_eq_error_rate!(Staking::stakers(account_from(21)).total, 1000 + 1000, 2);
                // 2 and 4 supported 20, each with stake 250, according to phragmen.
                assert_eq!(
                    Staking::stakers(account_from(21))
                        .others
                        .iter()
                        .map(|e| e.value)
                        .collect::<Vec<BalanceOf<Test>>>(),
                    vec![400, 600]
                );
                assert_eq!(
                    Staking::stakers(account_from(21))
                        .others
                        .iter()
                        .map(|e| e.who)
                        .collect::<Vec<AccountId>>(),
                    vec![account_from(3), account_from(1)]
                );
            } else {
                // total expo of 10, with 1200 coming from nominators (externals), according to phragmen.
                assert_eq!(Staking::stakers(account_from(11)).own, 1000);
                assert_eq!(Staking::stakers(account_from(11)).total, 1000 + 800);
                // 2 and 4 supported 10, each with stake 600, according to phragmen.
                assert_eq!(
                    Staking::stakers(account_from(11))
                        .others
                        .iter()
                        .map(|e| e.value)
                        .collect::<Vec<BalanceOf<Test>>>(),
                    vec![400, 400]
                );
                assert_eq!(
                    Staking::stakers(account_from(11))
                        .others
                        .iter()
                        .map(|e| e.who)
                        .collect::<Vec<AccountId>>(),
                    vec![account_from(3), account_from(1)]
                );
                // total expo of 20, with 500 coming from nominators (externals), according to phragmen.
                assert_eq!(Staking::stakers(account_from(21)).own, 1000);
                assert_eq_error_rate!(Staking::stakers(account_from(21)).total, 1000 + 1200, 2);
                // 2 and 4 supported 20, each with stake 250, according to phragmen.
                assert_eq!(
                    Staking::stakers(account_from(21))
                        .others
                        .iter()
                        .map(|e| e.value)
                        .collect::<Vec<BalanceOf<Test>>>(),
                    vec![600, 600]
                );
                assert_eq!(
                    Staking::stakers(account_from(21))
                        .others
                        .iter()
                        .map(|e| e.who)
                        .collect::<Vec<AccountId>>(),
                    vec![account_from(3), account_from(1)]
                );
            }

            // They are not chosen anymore
            assert_eq!(Staking::stakers(account_from(31)).total, 0);
            assert_eq!(Staking::stakers(account_from(41)).total, 0);

            // the total reward for era 1
            let total_payout_1 = current_total_payout_for_duration(3000);
            assert!(total_payout_1 > 100); // Test is meaningfull if reward something
            <Module<Test>>::reward_by_ids(vec![(account_from(41), 10)]); // must be no-op
            <Module<Test>>::reward_by_ids(vec![(account_from(31), 10)]); // must be no-op
            <Module<Test>>::reward_by_ids(vec![(account_from(21), 2)]);
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);

            start_era(2);

            // nothing else will happen, era ends and rewards are paid again,
            // it is expected that nominators will also be paid. See below

            let payout_for_10 = total_payout_1 / 3;
            let payout_for_20 = 2 * total_payout_1 / 3;
            if cfg!(feature = "equalize") {
                // Nominator 2: has [400 / 2000 ~ 1 / 5 from 10] + [600 / 2000 ~ 3 / 10 from 20]'s reward.
                assert_eq_error_rate!(
                    Balances::total_balance(&account_from(2)),
                    initial_balance + payout_for_10 / 5 + payout_for_20 * 3 / 10,
                    2,
                );
                // Nominator 4: has [400 / 2000 ~ 1 / 5 from 20] + [600 / 2000 ~ 3 / 10 from 10]'s reward.
                assert_eq_error_rate!(
                    Balances::total_balance(&account_from(4)),
                    initial_balance + payout_for_20 / 5 + payout_for_10 * 3 / 10,
                    2,
                );

                // Validator 10: got 1000 / 2000 external stake.
                assert_eq_error_rate!(
                    Balances::total_balance(&account_from(10)),
                    initial_balance + payout_for_10 / 2,
                    1,
                );
                // Validator 20: got 1000 / 2000 external stake.
                assert_eq_error_rate!(
                    Balances::total_balance(&account_from(20)),
                    initial_balance + payout_for_20 / 2,
                    1,
                );
            } else {
                // Nominator 2: has [400/1800 ~ 2/9 from 10] + [600/2200 ~ 3/11 from 20]'s reward. ==> 2/9 + 3/11
                assert_eq_error_rate!(
                    Balances::total_balance(&account_from(2)),
                    initial_balance + (2 * payout_for_10 / 9 + 3 * payout_for_20 / 11),
                    1,
                );
                // Nominator 4: has [400/1800 ~ 2/9 from 10] + [600/2200 ~ 3/11 from 20]'s reward. ==> 2/9 + 3/11
                assert_eq_error_rate!(
                    Balances::total_balance(&account_from(4)),
                    initial_balance + (2 * payout_for_10 / 9 + 3 * payout_for_20 / 11),
                    1,
                );

                // Validator 10: got 800 / 1800 external stake => 8/18 =? 4/9 => Validator's share = 5/9
                assert_eq_error_rate!(
                    Balances::total_balance(&account_from(10)),
                    initial_balance + 5 * payout_for_10 / 9,
                    1,
                );
                // Validator 20: got 1200 / 2200 external stake => 12/22 =? 6/11 => Validator's share = 5/11
                assert_eq_error_rate!(
                    Balances::total_balance(&account_from(20)),
                    initial_balance + 5 * payout_for_20 / 11,
                    1,
                );
            }

            check_exposure_all();
            check_nominator_all();
        });
}

#[test]
fn nominators_also_get_slashed() {
    // A nominator should be slashed if the validator they nominated is slashed
    // Here is the breakdown of roles:
    // 10 - is the controller of 11
    // 11 - is the stash.
    // 2 - is the nominator of 20, 10
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            fix_nominator_genesis_problem(2250);
            start_era(1);
            assert_eq!(Staking::validator_count(), 2);

            // Set payee to controller
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(10)),
                RewardDestination::Controller
            ));

            // give the man some money.
            let initial_balance = 1000;
            for i in [1, 2, 3, 10].iter() {
                let _ = Balances::make_free_balance_be(&account_from(*i), initial_balance);
            }

            // 2 will nominate for 10, 20
            let nominator_stake = 500;
            assert_ok!(Staking::bond(
                Origin::signed(account_from(1)),
                account_from(2),
                nominator_stake,
                RewardDestination::default()
            ));

            // Provide a valid kyc claim for
            // stash - 1
            // controller - 2
            let (nominator_signed, nominator_did) =
                make_account_with_balance(account_from(1), 1_000_000).unwrap();

            let service_provider_account = AccountId::from(AccountKeyring::Bob);
            let (service_provider_signed, service_provider_did) =
                make_account(service_provider_account.clone()).unwrap();
            add_trusted_kyc_provider(service_provider_did);

            let now = Utc::now();
            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: (now.timestamp() as u64 + 1000_u64).to_be_bytes().to_vec(),
            };
            add_nominator_claim(
                service_provider_did,
                nominator_did,
                service_provider_account,
                account_from(1),
                claim.clone(),
            );

            assert_ok!(Staking::nominate(
                Origin::signed(account_from(2)),
                vec![account_from(20), account_from(10)]
            ));

            let total_payout = current_total_payout_for_duration(3000);
            assert!(total_payout > 100); // Test is meaningfull if reward something
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);

            // new era, pay rewards,
            start_era(2);

            // Nominator stash didn't collect any.
            assert_eq!(Balances::total_balance(&account_from(2)), initial_balance);

            // 10 goes offline
            on_offence_now(
                &[OffenceDetails {
                    offender: (account_from(11), Staking::stakers(&account_from(11))),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(5)],
            );
            let expo = Staking::stakers(account_from(11));
            let slash_value = 50;
            let total_slash = expo.total.min(slash_value);
            let validator_slash = expo.own.min(total_slash);
            let nominator_slash = nominator_stake.min(total_slash - validator_slash);

            // initial + first era reward + slash
            assert_eq!(
                Balances::total_balance(&account_from(11)),
                initial_balance - validator_slash
            );
            assert_eq!(
                Balances::total_balance(&account_from(2)),
                initial_balance - nominator_slash
            );
            check_exposure_all();
            check_nominator_all();
            // Because slashing happened.
            assert!(is_disabled(account_from(10)));
        });
}

#[test]
fn double_staking_should_fail() {
    // should test (in the same order):
    // * an account already bonded as stash cannot be be stashed again.
    // * an account already bonded as stash cannot nominate.
    // * an account already bonded as controller can nominate.
    ExtBuilder::default().build().execute_with(|| {
        let arbitrary_value = 5;
        // 2 = controller, 1 stashed => ok
        assert_ok!(Staking::bond(
            Origin::signed(account_from(1)),
            account_from(2),
            arbitrary_value,
            RewardDestination::default()
        ));
        // 4 = not used so far, 1 stashed => not allowed.
        assert_noop!(
            Staking::bond(
                Origin::signed(account_from(1)),
                account_from(4),
                arbitrary_value,
                RewardDestination::default()
            ),
            Error::<Test>::AlreadyBonded,
        );
        // 1 = stashed => attempting to nominate should fail.
        assert_noop!(
            Staking::nominate(Origin::signed(account_from(1)), vec![account_from(1)]),
            Error::<Test>::NotController
        );
        // 2 = controller  => nominating should work.
        assert_ok!(Staking::nominate(
            Origin::signed(account_from(2)),
            vec![account_from(1)]
        ));
    });
}

#[test]
fn double_controlling_should_fail() {
    // should test (in the same order):
    // * an account already bonded as controller CANNOT be reused as the controller of another account.
    ExtBuilder::default().build().execute_with(|| {
        let arbitrary_value = 5;
        // 2 = controller, 1 stashed => ok
        assert_ok!(Staking::bond(
            Origin::signed(account_from(1)),
            account_from(2),
            arbitrary_value,
            RewardDestination::default(),
        ));
        // 2 = controller, 3 stashed (Note that 2 is reused.) => no-op
        assert_noop!(
            Staking::bond(
                Origin::signed(account_from(3)),
                account_from(2),
                arbitrary_value,
                RewardDestination::default()
            ),
            Error::<Test>::AlreadyPaired,
        );
    });
}

#[test]
fn session_and_eras_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Staking::current_era(), 0);

        // Block 1: No change.
        start_session(0);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(Staking::current_era(), 0);

        // Block 2: Simple era change.
        start_session(2);
        assert_eq!(Session::current_index(), 3);
        assert_eq!(Staking::current_era(), 1);

        // Block 3: Schedule an era length change; no visible changes.
        start_session(3);
        assert_eq!(Session::current_index(), 4);
        assert_eq!(Staking::current_era(), 1);

        // Block 4: Era change kicks in.
        start_session(5);
        assert_eq!(Session::current_index(), 6);
        assert_eq!(Staking::current_era(), 2);

        // Block 5: No change.
        start_session(6);
        assert_eq!(Session::current_index(), 7);
        assert_eq!(Staking::current_era(), 2);

        // Block 6: No change.
        start_session(7);
        assert_eq!(Session::current_index(), 8);
        assert_eq!(Staking::current_era(), 2);

        // Block 7: Era increment.
        start_session(8);
        assert_eq!(Session::current_index(), 9);
        assert_eq!(Staking::current_era(), 3);
    });
}

#[test]
fn forcing_new_era_works() {
    ExtBuilder::default().build().execute_with(|| {
        // normal flow of session.
        assert_eq!(Staking::current_era(), 0);
        start_session(0);
        assert_eq!(Staking::current_era(), 0);
        start_session(1);
        assert_eq!(Staking::current_era(), 0);
        start_session(2);
        assert_eq!(Staking::current_era(), 1);

        // no era change.
        ForceEra::put(Forcing::ForceNone);
        start_session(3);
        assert_eq!(Staking::current_era(), 1);
        start_session(4);
        assert_eq!(Staking::current_era(), 1);
        start_session(5);
        assert_eq!(Staking::current_era(), 1);
        start_session(6);
        assert_eq!(Staking::current_era(), 1);

        // back to normal.
        // this immediately starts a new session.
        ForceEra::put(Forcing::NotForcing);
        start_session(7);
        assert_eq!(Staking::current_era(), 2);
        start_session(8);
        assert_eq!(Staking::current_era(), 2);

        // forceful change
        ForceEra::put(Forcing::ForceAlways);
        start_session(9);
        assert_eq!(Staking::current_era(), 3);
        start_session(10);
        assert_eq!(Staking::current_era(), 4);
        start_session(11);
        assert_eq!(Staking::current_era(), 5);

        // just one forceful change
        ForceEra::put(Forcing::ForceNew);
        start_session(12);
        assert_eq!(Staking::current_era(), 6);

        assert_eq!(ForceEra::get(), Forcing::NotForcing);
        start_session(13);
        assert_eq!(Staking::current_era(), 6);
    });
}

#[test]
fn cannot_transfer_staked_balance() {
    // Tests that a stash account cannot transfer funds
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            // Confirm account 11 is stashed
            assert_eq!(Staking::bonded(&account_from(11)), Some(account_from(10)));
            // Confirm account 11 has some free balance
            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            // Confirm account 11 (via controller 10) is totally staked
            assert_eq!(Staking::stakers(&account_from(11)).total, 1000);
            // Confirm account 11 cannot transfer as a result
            assert_noop!(
                Balances::transfer(Origin::signed(account_from(11)), account_from(20), 1),
                DispatchError::Module {
                    index: 0,
                    error: 1,
                    message: Some("LiquidityRestrictions"),
                }
            );

            // Give account 11 extra free balance
            let _ = Balances::make_free_balance_be(&account_from(11), 10000);
            // Confirm that account 11 can now transfer some balance
            assert_ok!(Balances::transfer(
                Origin::signed(account_from(11)),
                account_from(20),
                1
            ));
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
        .build()
        .execute_with(|| {
            // Confirm account 21 is stashed
            assert_eq!(Staking::bonded(&account_from(21)), Some(account_from(20)));
            // Confirm account 21 has some free balance
            assert_eq!(Balances::free_balance(&account_from(21)), 2000);
            // Confirm account 21 (via controller 20) is totally staked
            assert_eq!(Staking::stakers(&account_from(21)).total, 1000);
            // Confirm account 21 can transfer at most 1000
            assert_noop!(
                Balances::transfer(Origin::signed(account_from(21)), account_from(20), 1001),
                DispatchError::Module {
                    index: 0,
                    error: 1,
                    message: Some("LiquidityRestrictions"),
                }
            );
            assert_ok!(Balances::transfer(
                Origin::signed(account_from(21)),
                account_from(20),
                1000
            ));
        });
}

#[test]
fn cannot_reserve_staked_balance() {
    // Checks that a bonded account cannot reserve balance from free balance
    ExtBuilder::default().build().execute_with(|| {
        // Confirm account 11 is stashed
        assert_eq!(Staking::bonded(&account_from(11)), Some(account_from(10)));
        // Confirm account 11 has some free balance
        assert_eq!(Balances::free_balance(&account_from(11)), 1000);
        // Confirm account 11 (via controller 10) is totally staked
        assert_eq!(Staking::stakers(&account_from(11)).own, 1000);
        // Confirm account 11 cannot transfer as a result
        assert_noop!(
            Balances::reserve(&account_from(11), 1),
            DispatchError::Module {
                index: 0,
                error: 1,
                message: Some("LiquidityRestrictions"),
            }
        );

        // Give account 11 extra free balance
        let _ = Balances::make_free_balance_be(&account_from(11), 10000);
        // Confirm account 11 can now reserve balance
        assert_ok!(Balances::reserve(&account_from(11), 1));
    });
}

#[test]
fn reward_destination_works() {
    // Rewards go to the correct destination as determined in Payee
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            // Check that account 11 is a validator
            assert!(Staking::current_elected().contains(&account_from(11)));
            // Check the balance of the validator account
            assert_eq!(Balances::free_balance(&account_from(10)), 1);
            // Check the balance of the stash account
            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            // Check how much is at stake
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                })
            );

            // Compute total payout now for whole duration as other parameter won't change
            let total_payout_0 = current_total_payout_for_duration(3000);
            assert!(total_payout_0 > 100); // Test is meaningfull if reward something
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);

            start_era(1);

            // Check that RewardDestination is Staked (default)
            assert_eq!(Staking::payee(&account_from(11)), RewardDestination::Staked);
            // Check that reward went to the stash account of validator
            assert_eq!(
                Balances::free_balance(&account_from(11)),
                1000 + total_payout_0
            );
            // Check that amount at stake increased accordingly
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000 + total_payout_0,
                    active: 1000 + total_payout_0,
                    unlocking: vec![],
                })
            );

            //Change RewardDestination to Stash
            <Payee<Test>>::insert(&account_from(11), RewardDestination::Stash);

            // Compute total payout now for whole duration as other parameter won't change
            let total_payout_1 = current_total_payout_for_duration(3000);
            assert!(total_payout_1 > 100); // Test is meaningfull if reward something
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);

            start_era(2);

            // Check that RewardDestination is Stash
            assert_eq!(Staking::payee(&account_from(11)), RewardDestination::Stash);
            // Check that reward went to the stash account
            assert_eq!(
                Balances::free_balance(&account_from(11)),
                1000 + total_payout_0 + total_payout_1
            );
            // Record this value
            let recorded_stash_balance = 1000 + total_payout_0 + total_payout_1;
            // Check that amount at stake is NOT increased
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000 + total_payout_0,
                    active: 1000 + total_payout_0,
                    unlocking: vec![],
                })
            );

            // Change RewardDestination to Controller
            <Payee<Test>>::insert(&account_from(11), RewardDestination::Controller);

            // Check controller balance
            assert_eq!(Balances::free_balance(&account_from(10)), 1);

            // Compute total payout now for whole duration as other parameter won't change
            let total_payout_2 = current_total_payout_for_duration(3000);
            assert!(total_payout_2 > 100); // Test is meaningfull if reward something
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);

            start_era(3);

            // Check that RewardDestination is Controller
            assert_eq!(
                Staking::payee(&account_from(11)),
                RewardDestination::Controller
            );
            // Check that reward went to the controller account
            assert_eq!(
                Balances::free_balance(&account_from(10)),
                1 + total_payout_2
            );
            // Check that amount at stake is NOT increased
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000 + total_payout_0,
                    active: 1000 + total_payout_0,
                    unlocking: vec![],
                })
            );
            // Check that amount in staked account is NOT increased.
            assert_eq!(
                Balances::free_balance(&account_from(11)),
                recorded_stash_balance
            );
        });
}

#[test]
fn validator_payment_prefs_work() {
    // Test that validator preferences are correctly honored
    // Note: unstake threshold is being directly tested in slashing tests.
    // This test will focus on validator payment.
    ExtBuilder::default().build().execute_with(|| {
        // Initial config
        let stash_initial_balance = Balances::total_balance(&account_from(11));

        // check the balance of a validator accounts.
        assert_eq!(Balances::total_balance(&account_from(10)), 1);
        // check the balance of a validator's stash accounts.
        assert_eq!(
            Balances::total_balance(&account_from(11)),
            stash_initial_balance
        );
        // and the nominator (to-be)
        let _ = Balances::make_free_balance_be(&account_from(2), 500);

        <ValidatorCommission>::put(Commission::Individual);

        // add a dummy nominator.
        <Stakers<Test>>::insert(
            &account_from(11),
            Exposure {
                own: 500, // equal division indicates that the reward will be equally divided among validator and nominator.
                total: 1000,
                others: vec![IndividualExposure {
                    who: account_from(2),
                    value: 500,
                }],
            },
        );
        <Payee<Test>>::insert(&account_from(2), RewardDestination::Stash);
        <Validators<Test>>::insert(
            &account_from(11),
            ValidatorPrefs {
                commission: Perbill::from_percent(50),
            },
        );

        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_0 = current_total_payout_for_duration(3000);
        assert!(total_payout_0 > 100); // Test is meaningfull if reward something
        <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);

        start_era(1);

        // whats left to be shared is the sum of 3 rounds minus the validator's cut.
        let shared_cut = total_payout_0 / 2;
        // Validator's payee is Staked account, 11, reward will be paid here.
        assert_eq!(
            Balances::total_balance(&account_from(11)),
            stash_initial_balance + shared_cut / 2 + shared_cut
        );
        // Controller account will not get any reward.
        assert_eq!(Balances::total_balance(&account_from(10)), 1);
        // Rest of the reward will be shared and paid to the nominator in stake.
        assert_eq!(
            Balances::total_balance(&account_from(2)),
            500 + shared_cut / 2
        );

        check_exposure_all();
        check_nominator_all();
    });
}

#[test]
fn validator_global_payment_prefs_work() {
    // Test that validator preferences are correctly honored
    // Note: unstake threshold is being directly tested in slashing tests.
    // This test will focus on validator payment.
    ExtBuilder::default().build().execute_with(|| {
        fix_nominator_genesis_problem(2250);
        start_era(1);
        // Initial config
        let stash_initial_balance = Balances::total_balance(&account_from(11));

        // check the balance of a validator accounts.
        assert_eq!(Balances::total_balance(&account_from(10)), 1);
        // check the balance of a validator's stash accounts.
        assert_eq!(
            Balances::total_balance(&account_from(11)),
            stash_initial_balance
        );
        // and the nominator (to-be)
        let _ = Balances::make_free_balance_be(&account_from(2), 500);

        // Use global commission
        <ValidatorCommission>::put(Commission::Global(Perbill::from_rational_approximation(
            1u64, 4u64,
        )));

        // add a dummy nominator.
        <Stakers<Test>>::insert(
            &account_from(11),
            Exposure {
                own: 500, // equal division indicates that the reward will be equally divided among validator and nominator.
                total: 1000,
                others: vec![IndividualExposure {
                    who: account_from(2),
                    value: 500,
                }],
            },
        );
        <Payee<Test>>::insert(&account_from(2), RewardDestination::Stash);
        assert_err!(
            Staking::validate(
                Origin::signed(account_from(10)),
                ValidatorPrefs {
                    commission: Perbill::from_rational_approximation(1u64, 2u64),
                }
            ),
            Error::<Test>::InvalidCommission
        );
        assert_ok!(Staking::validate(
            Origin::signed(account_from(10)),
            ValidatorPrefs {
                commission: Perbill::from_rational_approximation(1u64, 4u64),
            }
        ));

        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_1 = current_total_payout_for_duration(3000);
        assert!(total_payout_1 > 100); // Test is meaningfull if reward something
        <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);

        start_era(2);

        // Even though validator's individual commission is 50%, because global commission
        // is in effect, shared cut is 75%. Using (x - 1 + n) / n for rounding.
        let shared_cut = (total_payout_1 * 3 - 1 + 4) / 4;
        let validator_reward_era_1 = (total_payout_1 - shared_cut) + shared_cut / 2;
        let staker_reward_era_1 = shared_cut / 2;

        // Validator's payee is Staked account, 11, reward will be paid here.
        assert_eq!(
            Balances::total_balance(&account_from(11)),
            stash_initial_balance + validator_reward_era_1
        );
        // Controller account will not get any reward.
        assert_eq!(Balances::total_balance(&account_from(10)), 1);
        // Rest of the reward will be shared and paid to the nominator in stake.
        assert_eq!(
            Balances::total_balance(&account_from(2)),
            500 + staker_reward_era_1
        );

        <ValidatorCommission>::put(Commission::Individual);

        // add a dummy nominator.
        <Stakers<Test>>::insert(
            &account_from(11),
            Exposure {
                own: 500, // equal division indicates that the reward will be equally divided among validator and nominator.
                total: 1000,
                others: vec![IndividualExposure {
                    who: account_from(2),
                    value: 500,
                }],
            },
        );
        <Payee<Test>>::insert(&account_from(2), RewardDestination::Stash);
        <Validators<Test>>::insert(
            &account_from(11),
            ValidatorPrefs {
                commission: Perbill::from_percent(50),
            },
        );

        // check the balance of a validator's stash accounts.
        assert_eq!(
            Balances::total_balance(&account_from(11)),
            stash_initial_balance + validator_reward_era_1
        );

        // Compute total payout now for whole duration as other parameter won't change
        let total_payout_2 = current_total_payout_for_duration(3000);
        assert!(total_payout_2 > 100); // Test is meaningfull if reward something
        <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);

        start_era(3);

        // whats left to be shared is the sum of 3 rounds minus the validator's cut.
        let shared_cut = total_payout_2 / 2;
        // Validator's payee is Staked account, 11, reward will be paid here.
        assert_eq!(
            Balances::total_balance(&account_from(11)),
            stash_initial_balance + validator_reward_era_1 + shared_cut / 2 + shared_cut
        );
        // Controller account will not get any reward.
        assert_eq!(Balances::total_balance(&account_from(10)), 1);
        // Rest of the reward will be shared and paid to the nominator in stake.
        // assert_eq!(
        //     Balances::total_balance(&account_from(11)),
        //     staker_reward_era_1 + 500 + shared_cut / 2
        // );

        check_exposure_all();
        check_nominator_all();
    });
}

#[test]
fn bond_extra_works() {
    // Tests that extra `free_balance` in the stash can be added to stake
    // NOTE: this tests only verifies `StakingLedger` for correct updates
    // See `bond_extra_and_withdraw_unbonded_works` for more details and updates on `Exposure`.
    ExtBuilder::default().build().execute_with(|| {
        // Check that account 10 is a validator
        assert!(<Validators<Test>>::exists(account_from(11)));
        // Check that account 10 is bonded to account 11
        assert_eq!(Staking::bonded(&account_from(11)), Some(account_from(10)));
        // Check how much is at stake
        assert_eq!(
            Staking::ledger(&account_from(10)),
            Some(StakingLedger {
                stash: account_from(11),
                total: 1000,
                active: 1000,
                unlocking: vec![],
            })
        );

        // Give account 11 some large free balance greater than total
        let _ = Balances::make_free_balance_be(&account_from(11), 1000000);

        // Call the bond_extra function from controller, add only 100
        assert_ok!(Staking::bond_extra(Origin::signed(account_from(11)), 100));
        // There should be 100 more `total` and `active` in the ledger
        assert_eq!(
            Staking::ledger(&account_from(10)),
            Some(StakingLedger {
                stash: account_from(11),
                total: 1000 + 100,
                active: 1000 + 100,
                unlocking: vec![],
            })
        );

        // Call the bond_extra function with a large number, should handle it
        assert_ok!(Staking::bond_extra(
            Origin::signed(account_from(11)),
            u128::max_value()
        ));
        // The full amount of the funds should now be in the total and active
        assert_eq!(
            Staking::ledger(&account_from(10)),
            Some(StakingLedger {
                stash: account_from(11),
                total: 1000000,
                active: 1000000,
                unlocking: vec![],
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
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            // Set payee to controller. avoids confusion
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(10)),
                RewardDestination::Controller
            ));

            // Give account 11 some large free balance greater than total
            let _ = Balances::make_free_balance_be(&account_from(11), 1000000);

            // Initial config should be correct
            assert_eq!(Staking::current_era(), 0);
            assert_eq!(Session::current_index(), 0);

            // check the balance of a validator accounts.
            assert_eq!(Balances::total_balance(&account_from(10)), 1);

            // confirm that 10 is a normal validator and gets paid at the end of the era.
            start_era(1);

            // Initial state of 10
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                })
            );
            assert_eq!(
                Staking::stakers(&account_from(11)),
                Exposure {
                    total: 1000,
                    own: 1000,
                    others: vec![]
                }
            );

            // deposit the extra 100 units
            Staking::bond_extra(Origin::signed(account_from(11)), 100).unwrap();

            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000 + 100,
                    active: 1000 + 100,
                    unlocking: vec![],
                })
            );
            // Exposure is a snapshot! only updated after the next era update.
            assert_ne!(
                Staking::stakers(&account_from(11)),
                Exposure {
                    total: 1000 + 100,
                    own: 1000 + 100,
                    others: vec![]
                }
            );

            // trigger next era.
            Timestamp::set_timestamp(10);
            start_era(2);
            assert_eq!(Staking::current_era(), 2);

            // ledger should be the same.
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000 + 100,
                    active: 1000 + 100,
                    unlocking: vec![],
                })
            );
            // Exposure is now updated.
            assert_eq!(
                Staking::stakers(&account_from(11)),
                Exposure {
                    total: 1000 + 100,
                    own: 1000 + 100,
                    others: vec![]
                }
            );

            // Unbond almost all of the funds in stash.
            Staking::unbond(Origin::signed(account_from(10)), 1000).unwrap();
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000 + 100,
                    active: 100,
                    unlocking: vec![UnlockChunk {
                        value: 1000,
                        era: 2 + 3
                    }]
                })
            );

            // Attempting to free the balances now will fail. 2 eras need to pass.
            Staking::withdraw_unbonded(Origin::signed(account_from(10))).unwrap();
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000 + 100,
                    active: 100,
                    unlocking: vec![UnlockChunk {
                        value: 1000,
                        era: 2 + 3
                    }]
                })
            );

            // trigger next era.
            start_era(3);

            // nothing yet
            Staking::withdraw_unbonded(Origin::signed(account_from(10))).unwrap();
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 1000 + 100,
                    active: 100,
                    unlocking: vec![UnlockChunk {
                        value: 1000,
                        era: 2 + 3
                    }]
                })
            );

            // trigger next era.
            start_era(5);

            Staking::withdraw_unbonded(Origin::signed(account_from(10))).unwrap();
            // Now the value is free and the staking ledger is updated.
            assert_eq!(
                Staking::ledger(&account_from(10)),
                Some(StakingLedger {
                    stash: account_from(11),
                    total: 100,
                    active: 100,
                    unlocking: vec![]
                })
            );
        })
}

#[test]
fn too_many_unbond_calls_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        // locked at era 0 until 3
        for _ in 0..MAX_UNLOCKING_CHUNKS - 1 {
            assert_ok!(Staking::unbond(Origin::signed(account_from(10)), 1));
        }

        start_era(1);

        // locked at era 1 until 4
        assert_ok!(Staking::unbond(Origin::signed(account_from(10)), 1));
        // can't do more.
        assert_noop!(
            Staking::unbond(Origin::signed(account_from(10)), 1),
            Error::<Test>::NoMoreChunks
        );

        start_era(3);

        assert_noop!(
            Staking::unbond(Origin::signed(account_from(10)), 1),
            Error::<Test>::NoMoreChunks
        );
        // free up.
        assert_ok!(Staking::withdraw_unbonded(Origin::signed(account_from(10))));

        // Can add again.
        assert_ok!(Staking::unbond(Origin::signed(account_from(10)), 1));
        assert_eq!(
            Staking::ledger(&account_from(10)).unwrap().unlocking.len(),
            2
        );
    })
}

#[test]
fn slot_stake_is_least_staked_validator_and_exposure_defines_maximum_punishment() {
    // Test that slot_stake is determined by the least staked validator
    // Test that slot_stake is the maximum punishment that can happen to a validator
    ExtBuilder::default()
        .nominate(false)
        .fair(false)
        .build()
        .execute_with(|| {
            // Confirm validator count is 2
            assert_eq!(Staking::validator_count(), 2);
            // Confirm account 10 and 20 are validators
            assert!(
                <Validators<Test>>::exists(&account_from(11))
                    && <Validators<Test>>::exists(&account_from(21))
            );

            assert_eq!(Staking::stakers(&account_from(11)).total, 1000);
            assert_eq!(Staking::stakers(&account_from(21)).total, 2000);

            // Give the man some money.
            let _ = Balances::make_free_balance_be(&account_from(10), 1000);
            let _ = Balances::make_free_balance_be(&account_from(20), 1000);

            // We confirm initialized slot_stake is this value
            assert_eq!(
                Staking::slot_stake(),
                Staking::stakers(&account_from(11)).total
            );

            // Now lets lower account 20 stake
            <Stakers<Test>>::insert(
                &account_from(21),
                Exposure {
                    total: 69,
                    own: 69,
                    others: vec![],
                },
            );
            assert_eq!(Staking::stakers(&account_from(21)).total, 69);
            <Ledger<Test>>::insert(
                &account_from(20),
                StakingLedger {
                    stash: account_from(22),
                    total: 69,
                    active: 69,
                    unlocking: vec![],
                },
            );

            // Compute total payout now for whole duration as other parameter won't change
            let total_payout_0 = current_total_payout_for_duration(3000);
            assert!(total_payout_0 > 100); // Test is meaningfull if reward something
            <Module<Test>>::reward_by_ids(vec![(account_from(11), 1)]);
            <Module<Test>>::reward_by_ids(vec![(account_from(21), 1)]);

            // New era --> rewards are paid --> stakes are changed
            start_era(1);

            // -- new balances + reward
            assert_eq!(
                Staking::stakers(&account_from(11)).total,
                1000 + total_payout_0 / 2
            );
            assert_eq!(
                Staking::stakers(&account_from(21)).total,
                69 + total_payout_0 / 2
            );

            let _11_balance = Balances::free_balance(&account_from(11));
            assert_eq!(_11_balance, 1000 + total_payout_0 / 2);

            // -- slot stake should also be updated.
            assert_eq!(Staking::slot_stake(), 69 + total_payout_0 / 2);

            check_exposure_all();
            check_nominator_all();
        });
}

#[test]
fn on_free_balance_zero_stash_removes_validator() {
    // Tests that validator storage items are cleaned up when stash is empty
    // Tests that storage items are untouched when controller is empty
    ExtBuilder::default()
        .existential_deposit(10)
        .build()
        .execute_with(|| {
            // fix_nominator_genesis_problem(2250);
            // start_era(1);
            // Check the balance of the validator account
            assert_eq!(Balances::free_balance(&account_from(10)), 256);
            // Check the balance of the stash account
            assert_eq!(Balances::free_balance(&account_from(11)), 256000);
            // Check these two accounts are bonded
            assert_eq!(Staking::bonded(&account_from(11)), Some(account_from(10)));

            // Set some storage items which we expect to be cleaned up
            // Set payee information
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(10)),
                RewardDestination::Stash
            ));

            // Check storage items that should be cleaned up
            assert!(<Ledger<Test>>::exists(&account_from(10)));
            assert!(<Bonded<Test>>::exists(&account_from(11)));
            assert!(<Validators<Test>>::exists(&account_from(11)));
            assert!(<Payee<Test>>::exists(&account_from(11)));

            // Reduce free_balance of controller to 0
            let _ = Balances::slash(&account_from(10), u128::max_value());

            // Check the balance of the stash account has not been touched
            assert_eq!(Balances::free_balance(&account_from(11)), 256000);
            // Check these two accounts are still bonded
            assert_eq!(Staking::bonded(&account_from(11)), Some(account_from(10)));

            // Check storage items have not changed
            assert!(<Ledger<Test>>::exists(&account_from(10)));
            assert!(<Bonded<Test>>::exists(&account_from(11)));
            assert!(<Validators<Test>>::exists(&account_from(11)));
            assert!(<Payee<Test>>::exists(&account_from(11)));

            // Reduce free_balance of stash to 0
            let _ = Balances::slash(&account_from(11), u128::max_value());
            // Check total balance of stash
            assert_eq!(Balances::total_balance(&account_from(11)), 0);

            // Check storage items do not exist
            assert!(!<Ledger<Test>>::exists(&account_from(10)));
            assert!(!<Bonded<Test>>::exists(&account_from(11)));
            assert!(!<Validators<Test>>::exists(&account_from(11)));
            assert!(!<Nominators<Test>>::exists(&account_from(11)));
            assert!(!<Payee<Test>>::exists(&account_from(11)));
        });
}

#[test]
fn on_free_balance_zero_stash_removes_nominator() {
    // Tests that nominator storage items are cleaned up when stash is empty
    // Tests that storage items are untouched when controller is empty
    ExtBuilder::default()
        .existential_deposit(10)
        .build()
        .execute_with(|| {
            // Provide a valid kyc claim for
            // stash - 11
            // controller - 10
            let (nominator_signed, nominator_did) =
                make_account_with_balance(account_from(11), 256250).unwrap();

            let service_provider_account = AccountId::from(AccountKeyring::Bob);
            let (service_provider_signed, service_provider_did) =
                make_account(service_provider_account.clone()).unwrap();
            add_trusted_kyc_provider(service_provider_did);

            let now = Utc::now();
            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: (now.timestamp() as u64 + 1000_u64).to_be_bytes().to_vec(),
            };
            add_nominator_claim(
                service_provider_did,
                nominator_did,
                service_provider_account,
                account_from(11),
                claim.clone(),
            );

            // Make 10 a nominator
            assert_ok!(Staking::nominate(
                Origin::signed(account_from(10)),
                vec![account_from(20)]
            ));
            // Check that account 10 is a nominator
            assert!(<Nominators<Test>>::exists(account_from(11)));
            // Check the balance of the nominator account
            assert_eq!(Balances::free_balance(&account_from(10)), 256);
            // Check the balance of the stash account
            assert_eq!(Balances::free_balance(&account_from(11)), 256000);

            // Set payee information
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(10)),
                RewardDestination::Stash
            ));

            // Check storage items that should be cleaned up
            assert!(<Ledger<Test>>::exists(&account_from(10)));
            assert!(<Bonded<Test>>::exists(&account_from(11)));
            assert!(<Nominators<Test>>::exists(&account_from(11)));
            assert!(<Payee<Test>>::exists(&account_from(11)));

            // Reduce free_balance of controller to 0
            let _ = Balances::slash(&account_from(10), u128::max_value());
            // Check total balance of account 10
            assert_eq!(Balances::total_balance(&account_from(10)), 0);

            // Check the balance of the stash account has not been touched
            assert_eq!(Balances::free_balance(&account_from(11)), 256000);
            // Check these two accounts are still bonded
            assert_eq!(Staking::bonded(&account_from(11)), Some(account_from(10)));

            // Check storage items have not changed
            assert!(<Ledger<Test>>::exists(&account_from(10)));
            assert!(<Bonded<Test>>::exists(&account_from(11)));
            assert!(<Nominators<Test>>::exists(&account_from(11)));
            assert!(<Payee<Test>>::exists(&account_from(11)));

            // Reduce free_balance of stash to 0
            let _ = Balances::slash(&account_from(11), u128::max_value());
            // Check total balance of stash
            assert_eq!(Balances::total_balance(&account_from(11)), 0);

            // Check storage items do not exist
            assert!(!<Ledger<Test>>::exists(&account_from(10)));
            assert!(!<Bonded<Test>>::exists(&account_from(11)));
            assert!(!<Validators<Test>>::exists(&account_from(11)));
            assert!(!<Nominators<Test>>::exists(&account_from(11)));
            assert!(!<Payee<Test>>::exists(&account_from(11)));
        });
}

#[test]
fn switching_roles() {
    // Test that it should be possible to switch between roles (nominator, validator, idle) with minimal overhead.
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            Timestamp::set_timestamp(1); // Initialize time.

            // Reset reward destination
            for i in &[10, 20] {
                assert_ok!(Staking::set_payee(
                    Origin::signed(account_from(*i)),
                    RewardDestination::Controller
                ));
            }

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // put some money in account that we'll use.
            for i in 1..7 {
                let _ = Balances::deposit_creating(&account_from(i), 5000);
            }

            // Pass compliance for account 1
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(1)
            ));

            // add 2 nominators
            assert_ok!(Staking::bond(
                Origin::signed(account_from(1)),
                account_from(2),
                2000,
                RewardDestination::Controller
            ));

            // Provide a valid kyc claim for
            // stash - 1
            // controller - 2
            let (nominator_signed, nominator_did) =
                make_account_with_balance(account_from(1), 2250).unwrap();

            let service_provider_account = AccountId::from(AccountKeyring::Bob);
            let (service_provider_signed, service_provider_did) =
                make_account(service_provider_account.clone()).unwrap();
            add_trusted_kyc_provider(service_provider_did);

            let now = Utc::now();
            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: (now.timestamp() as u64 + 1000_u64).to_be_bytes().to_vec(),
            };
            add_nominator_claim(
                service_provider_did,
                nominator_did,
                service_provider_account,
                account_from(1),
                claim.clone(),
            );

            assert_ok!(Staking::nominate(
                Origin::signed(account_from(2)),
                vec![account_from(11), account_from(5)]
            ));

            assert_ok!(Staking::bond(
                Origin::signed(account_from(3)),
                account_from(4),
                500,
                RewardDestination::Controller
            ));

            // Provide a valid kyc claim for
            // stash - 3
            // controller - 4
            let (nominator_signed, nominator_did) =
                make_account_with_balance(account_from(3), 2250).unwrap();

            add_nominator_claim(
                service_provider_did,
                nominator_did,
                service_provider_account,
                account_from(3),
                claim.clone(),
            );

            assert_ok!(Staking::nominate(
                Origin::signed(account_from(4)),
                vec![account_from(21), account_from(1)]
            ));

            // Pass compliance for account 5
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(5)
            ));

            // add a new validator candidate
            assert_ok!(Staking::bond(
                Origin::signed(account_from(5)),
                account_from(6),
                1000,
                RewardDestination::Controller
            ));
            assert_ok!(Staking::validate(
                Origin::signed(account_from(6)),
                ValidatorPrefs::default()
            ));

            // new block
            start_session(1);

            // no change
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // new block
            start_session(2);

            // no change
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // new block --> ne era --> new validators
            start_session(3);

            // with current nominators 10 and 5 have the most stake
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(6), account_from(10)]
            );

            // 2 decides to be a validator. Consequences:
            assert_ok!(Staking::validate(
                Origin::signed(account_from(2)),
                ValidatorPrefs::default()
            ));
            // new stakes:
            // 10: 1000 self vote
            // 20: 1000 self vote + 250 vote
            // 6 : 1000 self vote
            // 2 : 2000 self vote + 250 vote.
            // Winners: 20 and 2

            start_session(4);
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(6), account_from(10)]
            );

            start_session(5);
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(6), account_from(10)]
            );

            // ne era
            start_session(6);
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(2), account_from(20)]
            );

            check_exposure_all();
            check_nominator_all();
        });
}

#[test]
fn switching_roles_when_min_bond_changes() {
    // Test that it should be possible to switch between roles (nominator, validator, idle) with minimal overhead.
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            Timestamp::set_timestamp(1); // Initialize time.

            // Reset reward destination
            for i in &[10, 20] {
                assert_ok!(Staking::set_payee(
                    Origin::signed(account_from(*i)),
                    RewardDestination::Controller
                ));
            }

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // put some money in account that we'll use.
            for i in 1..7 {
                let _ = Balances::deposit_creating(&account_from(i), 5000);
            }

            // Pass compliance for account 1
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(1)
            ));

            // add 2 nominators
            assert_ok!(Staking::bond(
                Origin::signed(account_from(1)),
                account_from(2),
                2000,
                RewardDestination::Controller
            ));
            assert_ok!(Staking::nominate(
                Origin::signed(account_from(2)),
                vec![account_from(11), account_from(5)]
            ));

            assert_ok!(Staking::bond(
                Origin::signed(account_from(3)),
                account_from(4),
                500,
                RewardDestination::Controller
            ));
            assert_ok!(Staking::nominate(
                Origin::signed(account_from(4)),
                vec![account_from(21), account_from(1)]
            ));

            <MinimumBondThreshold<Test>>::put(1250);

            // Pass compliance for account 1
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(5)
            ));

            // add a new validator candidate
            assert_ok!(Staking::bond(
                Origin::signed(account_from(5)),
                account_from(6),
                10,
                RewardDestination::Controller
            ));
            assert_err!(
                Staking::validate(Origin::signed(account_from(6)), ValidatorPrefs::default()),
                Error::<Test>::InsufficientValue
            );
            assert_ok!(Staking::bond_extra(Origin::signed(account_from(5)), 2000));
            assert_ok!(Staking::validate(
                Origin::signed(account_from(6)),
                ValidatorPrefs::default()
            ));

            // new block
            start_session(1);

            // no change
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // new block
            start_session(2);

            // no change
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );

            // new block --> new era --> new validators
            start_session(3);

            // with current nominators 10 and 5 have the most stake
            assert_eq_uvec!(validator_controllers(), vec![account_from(6)]);

            // Reduce the min bond value
            <MinimumBondThreshold<Test>>::put(1000);

            // 2 decides to be a validator. Consequences:
            assert_ok!(Staking::validate(
                Origin::signed(account_from(2)),
                ValidatorPrefs::default()
            ));
            // new stakes:
            // 10: 1000 self vote
            // 20: 1000 self vote + 250 vote
            // 6 : 2000 self vote
            // 2 : 2000 self vote + 250 vote.
            // Winners: 6 and 2

            start_session(4);
            assert_eq_uvec!(validator_controllers(), vec![account_from(6)]);

            start_session(5);
            assert_eq_uvec!(validator_controllers(), vec![account_from(6)]);

            // new era
            start_session(6);
            // 2 should join now that bond value has been
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(6), account_from(2)]
            );

            check_exposure_all();
            check_nominator_all();
        });
}

#[test]
fn wrong_vote_is_null() {
    ExtBuilder::default()
        .nominate(false)
        .validator_pool(true)
        .build()
        .execute_with(|| {
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(40), account_from(30)]
            );

            // put some money in account that we'll use.
            for i in 1..3 {
                let _ = Balances::deposit_creating(&account_from(i), 5000);
            }

            // add 1 nominators
            assert_ok!(Staking::bond(
                Origin::signed(account_from(1)),
                account_from(2),
                2000,
                RewardDestination::default()
            ));

            // Provide a valid kyc claim for
            // stash - 1
            // controller - 2
            let (nominator_signed, nominator_did) =
                make_account_with_balance(account_from(1), 2250).unwrap();

            let service_provider_account = AccountId::from(AccountKeyring::Bob);
            let (service_provider_signed, service_provider_did) =
                make_account(service_provider_account.clone()).unwrap();
            add_trusted_kyc_provider(service_provider_did);

            let now = Utc::now();
            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: (now.timestamp() as u64 + 1000_u64).to_be_bytes().to_vec(),
            };
            add_nominator_claim(
                service_provider_did,
                nominator_did,
                service_provider_account,
                account_from(1),
                claim.clone(),
            );

            assert_ok!(Staking::nominate(
                Origin::signed(account_from(2)),
                vec![
                    account_from(11),
                    account_from(21), // good votes
                    account_from(1),
                    account_from(2),
                    account_from(15),
                    account_from(1000),
                    account_from(25) // crap votes. No effect.
                ]
            ));

            // new block
            start_era(1);

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10)]
            );
        });
}

#[test]
fn bond_with_no_staked_value() {
    // Behavior when someone bonds with no staked value.
    // Particularly when she votes and the candidate is elected.
    ExtBuilder::default()
        .validator_count(3)
        .existential_deposit(5) // It will not affect the actual minimum balance logic
        .nominate(false)
        .minimum_validator_count(1)
        .build()
        .execute_with(|| {
            // Minimum balance in custom balance module is 0 so this
            // assert doesn't makes sense
            // assert_noop!(
            //     Staking::bond(
            //         Origin::signed(account_from(1)),
            //         account_from(2),
            //         1,
            //         RewardDestination::Controller
            //     ),
            //     Error::<Test>::InsufficientValue,
            // );
            let _ = Balances::make_free_balance_be(&account_from(1), 5);
            // bonded with absolute minimum value possible.
            assert_ok!(Staking::bond(
                Origin::signed(account_from(1)),
                account_from(2),
                5,
                RewardDestination::Controller
            ));
            assert_eq!(Balances::locks(&account_from(1))[0].amount, 5);

            // unbonding even 1 will cause all to be unbonded.
            assert_ok!(Staking::unbond(Origin::signed(account_from(2)), 1));
            assert_eq!(
                Staking::ledger(account_from(2)),
                Some(StakingLedger {
                    stash: account_from(1),
                    active: 4,
                    total: 5,
                    unlocking: vec![UnlockChunk { value: 1, era: 3 }]
                })
            );

            start_era(1);
            start_era(2);

            // not yet removed.
            assert_ok!(Staking::withdraw_unbonded(Origin::signed(account_from(2))));
            assert!(Staking::ledger(account_from(2)).is_some());
            assert_eq!(Balances::locks(&account_from(1))[0].amount, 5);

            start_era(3);

            // poof. Account 1 is not removed from the staking system because it have 4 balance as an active
            assert_ok!(Staking::withdraw_unbonded(Origin::signed(account_from(2))));
            // assert!(Staking::ledger(account_from(2)).is_none());
            // assert_eq!(Balances::locks(&account_from(1)).len(), 0);
        });
}

#[test]
fn bond_with_little_staked_value_bounded_by_slot_stake() {
    // Behavior when someone bonds with little staked value.
    // Particularly when she votes and the candidate is elected.
    ExtBuilder::default()
        .validator_count(3)
        .nominate(false)
        .minimum_validator_count(1)
        .build()
        .execute_with(|| {
            // setup
            assert_ok!(Staking::chill(Origin::signed(account_from(30))));
            assert_ok!(Staking::set_payee(
                Origin::signed(account_from(10)),
                RewardDestination::Controller
            ));
            let init_balance_2 = Balances::free_balance(&account_from(2));
            let init_balance_10 = Balances::free_balance(&account_from(10));

            // Stingy validator.
            let _ = Balances::make_free_balance_be(&account_from(1), 5);
            println!(
                "Print the free balance of the account_1: {:?}",
                Balances::free_balance(&account_from(1))
            );
            assert_ok!(Staking::bond(
                Origin::signed(account_from(1)),
                account_from(2),
                1,
                RewardDestination::Controller
            ));
            // Pass compliance for account 1
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(1)
            ));
            assert_ok!(Staking::validate(
                Origin::signed(account_from(2)),
                ValidatorPrefs::default()
            ));

            let total_payout_0 = current_total_payout_for_duration(3000);
            assert!(total_payout_0 > 100); // Test is meaningfull if reward something
            reward_all_elected();
            start_era(1);

            // 2 is elected.
            // and fucks up the slot stake.
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10), account_from(2)]
            );
            assert_eq!(Staking::slot_stake(), 1);

            // Old ones are rewarded.
            assert_eq!(
                Balances::free_balance(&account_from(10)),
                init_balance_10 + total_payout_0 / 3
            );
            // no rewards paid to 2. This was initial election.
            assert_eq!(Balances::free_balance(&account_from(2)), init_balance_2);

            let total_payout_1 = current_total_payout_for_duration(3000);
            assert!(total_payout_1 > 100); // Test is meaningfull if reward something
            reward_all_elected();
            start_era(2);

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(20), account_from(10), account_from(2)]
            );
            assert_eq!(Staking::slot_stake(), 1);

            assert_eq!(
                Balances::free_balance(&account_from(10)),
                init_balance_10 + total_payout_0 / 3 + total_payout_1 / 3,
            );
            check_exposure_all();
            check_nominator_all();
        });
}

#[cfg(feature = "equalize")]
#[test]
#[ignore]
fn phragmen_linear_worse_case_equalize() {
    ExtBuilder::default()
        .nominate(false)
        .validator_pool(true)
        .fair(true)
        .build()
        .execute_with(|| {
            bond_validator(50, 1000);
            bond_validator(60, 1000);
            bond_validator(70, 1000);

            // Pass copliance for account 51, 61 & 71
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(51)
            ));
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(61)
            ));
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(71)
            ));

            add_claim_for_nominator(
                account_from(3),
                AccountId::from(AccountKeyring::Bob),
                100000_u128,
            );
            bond_nominator(2, 2000, vec![account_from(11)]);
            add_claim_for_nominator(
                account_from(5),
                AccountId::from(AccountKeyring::Charlie),
                100000_u128,
            );
            bond_nominator(4, 1000, vec![account_from(11), account_from(21)]);
            add_claim_for_nominator(
                account_from(7),
                AccountId::from(AccountKeyring::Alice),
                100000_u128,
            );
            bond_nominator(6, 1000, vec![account_from(21), account_from(31)]);
            add_claim_for_nominator(account_from(9), account_from(555), 100000_u128);
            bond_nominator(8, 1000, vec![account_from(31), account_from(41)]);
            add_claim_for_nominator(account_from(111), account_from(556), 100000_u128);
            bond_nominator(110, 1000, vec![account_from(41), account_from(51)]);
            add_claim_for_nominator(account_from(121), account_from(557), 100000_u128);
            bond_nominator(120, 1000, vec![account_from(51), account_from(61)]);
            add_claim_for_nominator(account_from(131), account_from(558), 100000_u128);
            bond_nominator(130, 1000, vec![account_from(61), account_from(71)]);

            for i in &[10, 20, 30, 40, 50, 60, 70] {
                assert_ok!(Staking::set_payee(
                    Origin::signed(account_from(*i)),
                    RewardDestination::Controller
                ));
            }

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(40), account_from(30)]
            );
            assert_ok!(Staking::set_validator_count(Origin::ROOT, 7));
            fix_nominator_genesis_problem(2250);
            start_era(1);

            assert_eq_uvec!(
                validator_controllers(),
                vec![
                    account_from(10),
                    account_from(60),
                    account_from(40),
                    account_from(20),
                    account_from(50),
                    account_from(30),
                    account_from(70)
                ]
            );

            assert_eq_error_rate!(Staking::stakers(account_from(11)).total, 3000, 2);
            assert_eq_error_rate!(Staking::stakers(account_from(21)).total, 2255, 2);
            assert_eq_error_rate!(Staking::stakers(account_from(31)).total, 2255, 2);
            assert_eq_error_rate!(Staking::stakers(account_from(41)).total, 1925, 2);
            assert_eq_error_rate!(Staking::stakers(account_from(51)).total, 1870, 2);
            assert_eq_error_rate!(Staking::stakers(account_from(61)).total, 1890, 2);
            assert_eq_error_rate!(Staking::stakers(account_from(71)).total, 1800, 2);

            check_exposure_all();
            check_nominator_all();
        })
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

            System::set_block_number(1);
            Session::on_initialize(System::block_number());

            assert_eq!(validator_controllers().len(), 1);
            check_exposure_all();
            check_nominator_all();
        })
}

#[test]
fn phragmen_should_not_overflow_validators() {
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            let _ = Staking::chill(Origin::signed(account_from(10)));
            let _ = Staking::chill(Origin::signed(account_from(20)));

            bond_validator(2, u64::max_value() as u128);
            bond_validator(4, u64::max_value() as u128);

            // Pass copliance for account 3 & 5
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(3)
            ));
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(5)
            ));

            add_claim_for_nominator(
                account_from(7),
                AccountId::from(AccountKeyring::Bob),
                100000_u128,
            );
            add_claim_for_nominator(
                account_from(9),
                AccountId::from(AccountKeyring::Charlie),
                100000_u128,
            );
            bond_nominator(
                6,
                u64::max_value() as u128 / 2,
                vec![account_from(3), account_from(5)],
            );
            bond_nominator(
                8,
                u64::max_value() as u128 / 2,
                vec![account_from(3), account_from(5)],
            );

            start_era(1);

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(4), account_from(2)]
            );

            // This test will fail this. Will saturate.
            // check_exposure_all();
            assert_eq!(
                Staking::stakers(account_from(3)).total,
                u64::max_value() as u128
            );
            assert_eq!(
                Staking::stakers(account_from(5)).total,
                u64::max_value() as u128
            );
        })
}

#[test]
fn phragmen_should_not_overflow_nominators() {
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            let _ = Staking::chill(Origin::signed(account_from(10)));
            let _ = Staking::chill(Origin::signed(account_from(20)));

            bond_validator(2, u64::max_value() as u128 / 2);
            bond_validator(4, u64::max_value() as u128 / 2);

            // Pass compliance for account 3 & 5
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(3)
            ));
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(5)
            ));

            add_claim_for_nominator(
                account_from(7),
                AccountId::from(AccountKeyring::Bob),
                100000_u128,
            );
            add_claim_for_nominator(
                account_from(9),
                AccountId::from(AccountKeyring::Charlie),
                100000_u128,
            );
            bond_nominator(
                6,
                u64::max_value() as u128,
                vec![account_from(3), account_from(5)],
            );
            bond_nominator(
                8,
                u64::max_value() as u128,
                vec![account_from(3), account_from(5)],
            );

            start_era(1);

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(4), account_from(2)]
            );

            // Saturate.
            assert_eq!(
                Staking::stakers(account_from(3)).total,
                u64::max_value() as u128 / 2
            );
            assert_eq!(
                Staking::stakers(account_from(5)).total,
                u64::max_value() as u128 / 2
            );
        })
}

#[test]
fn phragmen_should_not_overflow_ultimate() {
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            bond_validator(2, u64::max_value() as u128);
            bond_validator(4, u64::max_value() as u128);

            assert!(Staking::ledger(account_from(2)).unwrap().active == u64::max_value() as u128);

            // Pass compliance for account 3 & 5
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(3)
            ));
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                account_from(5)
            ));

            add_claim_for_nominator(
                account_from(7),
                AccountId::from(AccountKeyring::Bob),
                100000_u128,
            );
            add_claim_for_nominator(
                account_from(9),
                AccountId::from(AccountKeyring::Charlie),
                100000_u128,
            );
            bond_nominator(
                6,
                u64::max_value() as u128,
                vec![account_from(3), account_from(5)],
            );
            bond_nominator(
                8,
                u64::max_value() as u128,
                vec![account_from(3), account_from(5)],
            );

            start_era(1);

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(4), account_from(2)]
            );

            // Saturate.
            assert_eq!(
                Staking::stakers(account_from(3)).total,
                u64::max_value() as u128
            );
            assert_eq!(
                Staking::stakers(account_from(5)).total,
                u64::max_value() as u128
            );
        })
}

#[test]
fn reward_validator_slashing_validator_doesnt_overflow() {
    ExtBuilder::default().build().execute_with(|| {
        let stake = u64::max_value() as u128 * 2;
        let reward_slash = u64::max_value() as u128 * 2;

        // Assert multiplication overflows in balance arithmetic.
        assert!(stake.checked_mul(reward_slash).is_none());

        // Set staker
        let _ = Balances::make_free_balance_be(&account_from(11), stake);
        <Stakers<Test>>::insert(
            &account_from(11),
            Exposure {
                total: stake,
                own: stake,
                others: vec![],
            },
        );

        // Check reward
        let _ = Staking::reward_validator(&account_from(11), reward_slash);
        assert_eq!(Balances::total_balance(&account_from(11)), stake * 2);

        // Set staker
        let _ = Balances::make_free_balance_be(&account_from(11), stake);
        let _ = Balances::make_free_balance_be(&account_from(2), stake);

        // only slashes out of bonded stake are applied. without this line,
        // it is 0.
        Staking::bond(
            Origin::signed(account_from(2)),
            account_from(2000),
            stake - 1,
            RewardDestination::default(),
        )
        .unwrap();
        <Stakers<Test>>::insert(
            &account_from(11),
            Exposure {
                total: stake,
                own: 1,
                others: vec![IndividualExposure {
                    who: account_from(2),
                    value: stake - 1,
                }],
            },
        );

        // Check slashing
        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(100)],
        );

        assert_eq!(Balances::total_balance(&account_from(11)), stake - 1);
        assert_eq!(Balances::total_balance(&account_from(2)), 1);
    })
}

#[test]
fn reward_from_authorship_event_handler_works() {
    ExtBuilder::default().build().execute_with(|| {
        use pallet_authorship::EventHandler;

        assert_eq!(
            <pallet_authorship::Module<Test>>::author(),
            account_from(11)
        );

        <Module<Test>>::note_author(account_from(11));
        <Module<Test>>::note_uncle(account_from(21), 1);
        // An uncle author that is not currently elected doesn't get rewards,
        // but the block producer does get reward for referencing it.
        <Module<Test>>::note_uncle(account_from(31), 1);
        // Rewarding the same two times works.
        <Module<Test>>::note_uncle(account_from(11), 1);

        // Not mandatory but must be coherent with rewards
        assert_eq!(
            <CurrentElected<Test>>::get(),
            vec![account_from(21), account_from(11)]
        );

        // 21 is rewarded as an uncle producer
        // 11 is rewarded as a block producer and uncle referencer and uncle producer
        assert_eq!(
            CurrentEraPointsEarned::get().individual,
            vec![1, 20 + 2 * 3 + 1]
        );
        assert_eq!(CurrentEraPointsEarned::get().total, 28);
    })
}

#[test]
fn add_reward_points_fns_works() {
    ExtBuilder::default().build().execute_with(|| {
        let validators = <Module<Test>>::current_elected();
        // Not mandatory but must be coherent with rewards
        assert_eq!(validators, vec![account_from(21), account_from(11)]);

        <Module<Test>>::reward_by_indices(vec![(0, 1), (1, 1), (2, 1), (1, 1)]);

        <Module<Test>>::reward_by_ids(vec![
            (account_from(21), 1),
            (account_from(11), 1),
            (account_from(31), 1),
            (account_from(11), 1),
        ]);

        assert_eq!(CurrentEraPointsEarned::get().individual, vec![2, 4]);
        assert_eq!(CurrentEraPointsEarned::get().total, 6);
    })
}

#[test]
fn unbonded_balance_is_not_slashable() {
    ExtBuilder::default().build().execute_with(|| {
        // total amount staked is slashable.
        assert_eq!(Staking::slashable_balance_of(&account_from(11)), 1000);

        assert_ok!(Staking::unbond(Origin::signed(account_from(10)), 800));

        // only the active portion.
        assert_eq!(Staking::slashable_balance_of(&account_from(11)), 200);
    })
}

#[test]
fn era_is_always_same_length() {
    // This ensures that the sessions is always of the same length if there is no forcing no
    // session changes.
    ExtBuilder::default().build().execute_with(|| {
        start_era(1);
        assert_eq!(
            Staking::current_era_start_session_index(),
            SessionsPerEra::get()
        );

        start_era(2);
        assert_eq!(
            Staking::current_era_start_session_index(),
            SessionsPerEra::get() * 2
        );

        let session = Session::current_index();
        ForceEra::put(Forcing::ForceNew);
        advance_session();
        assert_eq!(Staking::current_era(), 3);
        assert_eq!(Staking::current_era_start_session_index(), session + 1);

        start_era(4);
        assert_eq!(
            Staking::current_era_start_session_index(),
            session + SessionsPerEra::get() + 1
        );
    });
}

#[test]
fn offence_forces_new_era() {
    ExtBuilder::default().build().execute_with(|| {
        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(5)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceNew);
    });
}

#[test]
fn offence_ensures_new_era_without_clobbering() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Staking::force_new_era_always(Origin::ROOT));

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(5)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceAlways);
    });
}

#[test]
fn offence_deselects_validator_when_slash_is_zero() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(<Validators<Test>>::exists(account_from(11)));
        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );
        assert_eq!(Staking::force_era(), Forcing::ForceNew);
        assert!(!<Validators<Test>>::exists(account_from(11)));
    });
}

#[test]
fn slashing_performed_according_exposure() {
    // This test checks that slashing is performed according the exposure (or more precisely,
    // historical exposure), not the current balance.
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Staking::stakers(&account_from(11)).own, 1000);

        // Handle an offence with a historical exposure.
        on_offence_now(
            &[OffenceDetails {
                offender: (
                    account_from(11),
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
        assert_eq!(Balances::free_balance(&account_from(11)), 1000 - 250);
    });
}

#[test]
fn slash_in_old_span_does_not_deselect() {
    ExtBuilder::default().build().execute_with(|| {
        fix_nominator_genesis_problem(2250);
        start_era(1);

        assert!(<Validators<Test>>::exists(account_from(11)));
        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceNew);
        assert!(!<Validators<Test>>::exists(account_from(11)));
        start_era(2);

        Staking::validate(Origin::signed(account_from(10)), Default::default()).unwrap();
        assert_eq!(Staking::force_era(), Forcing::NotForcing);
        assert!(<Validators<Test>>::exists(account_from(11)));
        start_era(3);

        // this staker is in a new slashing span now, having re-registered after
        // their prior slash.

        on_offence_in_era(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
            1,
        );
        // not for zero-slash.
        assert_eq!(Staking::force_era(), Forcing::NotForcing);
        assert!(<Validators<Test>>::exists(account_from(11)));

        on_offence_in_era(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(100)],
            1,
        );
        // or non-zero.
        assert_eq!(Staking::force_era(), Forcing::NotForcing);
        // Check that account 11 is a validator
        assert!(Staking::current_elected().contains(&account_from(11)));
        assert!(!<Validators<Test>>::exists(account_from(11)));
        // Commented the below check as we change the condition of reaping
        // to <= 0 (i.e hard coded existential deposit) so that condition
        // will remove the account 11 from the staking storage.
        //assert_ledger_consistent(11);
    });
}

#[test]
fn reporters_receive_their_slice() {
    // This test verifies that the reporters of the offence receive their slice from the slashed
    // amount.
    ExtBuilder::default().build().execute_with(|| {
        // The reporters' reward is calculated from the total exposure.
        #[cfg(feature = "equalize")]
        let initial_balance = 1250;
        #[cfg(not(feature = "equalize"))]
        let initial_balance = 1125;
        fix_nominator_genesis_problem(2250);
        start_era(1);

        assert_eq!(Staking::stakers(&account_from(11)).total, initial_balance);

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![account_from(1), account_from(2)],
            }],
            &[Perbill::from_percent(50)],
        );

        // F1 * (reward_proportion * slash - 0)
        // 50% * (10% * initial_balance / 2)
        let reward = (initial_balance / 20) / 2;
        let reward_each = reward / 2; // split into two pieces.
                                      // Remove 10 from 10 + reward as reward calculation in being affected after the start of new era
        assert_eq!(Balances::free_balance(&account_from(1)), reward_each);
        assert_eq!(Balances::free_balance(&account_from(2)), reward_each);
        assert_ledger_consistent(11);
    });
}

#[test]
fn subsequent_reports_in_same_span_pay_out_less() {
    // This test verifies that the reporters of the offence receive their slice from the slashed
    // amount.
    ExtBuilder::default().build().execute_with(|| {
        // The reporters' reward is calculated from the total exposure.
        #[cfg(feature = "equalize")]
        let initial_balance = 1250;
        #[cfg(not(feature = "equalize"))]
        let initial_balance = 1125;
        fix_nominator_genesis_problem(2250);
        start_era(1);

        assert_eq!(Staking::stakers(&account_from(11)).total, initial_balance);

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![account_from(1)],
            }],
            &[Perbill::from_percent(20)],
        );

        // F1 * (reward_proportion * slash - 0)
        // 50% * (10% * initial_balance * 20%)
        let reward = (initial_balance / 5) / 20;
        // Remove 10 from 10 + reward as reward calculation in being affected after the start of new era
        assert_eq!(Balances::free_balance(&account_from(1)), reward);

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![account_from(1)],
            }],
            &[Perbill::from_percent(50)],
        );

        let prior_payout = reward;

        // F1 * (reward_proportion * slash - prior_payout)
        // 50% * (10% * (initial_balance / 2) - prior_payout)
        let reward = ((initial_balance / 20) - prior_payout) / 2;
        // Remove 10 from 10 + reward as reward calculation in being affected after the start of new era
        assert_eq!(
            Balances::free_balance(&account_from(1)),
            prior_payout + reward
        );
        assert_ledger_consistent(11);
    });
}

#[test]
fn invulnerables_are_not_slashed() {
    // For invulnerable validators no slashing is performed.
    ExtBuilder::default()
        .invulnerables(vec![account_from(11)])
        .build()
        .execute_with(|| {
            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            assert_eq!(Balances::free_balance(&account_from(21)), 2000);

            let exposure = Staking::stakers(&account_from(21));
            let initial_balance = Staking::slashable_balance_of(&account_from(21));

            let nominator_balances: Vec<_> = exposure
                .others
                .iter()
                .map(|o| Balances::free_balance(&o.who))
                .collect();

            on_offence_now(
                &[
                    OffenceDetails {
                        offender: (account_from(11), Staking::stakers(&account_from(11))),
                        reporters: vec![],
                    },
                    OffenceDetails {
                        offender: (account_from(21), Staking::stakers(&account_from(21))),
                        reporters: vec![],
                    },
                ],
                &[Perbill::from_percent(50), Perbill::from_percent(20)],
            );

            // The validator 11 hasn't been slashed, but 21 has been.
            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            // 2000 - (0.2 * initial_balance)
            assert_eq!(
                Balances::free_balance(&account_from(21)),
                2000 - (2 * initial_balance / 10)
            );

            // ensure that nominators were slashed as well.
            for (initial_balance, other) in nominator_balances.into_iter().zip(exposure.others) {
                assert_eq!(
                    Balances::free_balance(&other.who),
                    initial_balance - (2 * other.value / 10),
                );
            }
            assert_ledger_consistent(11);
            assert_ledger_consistent(21);
        });
}

#[test]
fn dont_slash_if_fraction_is_zero() {
    // Don't slash if the fraction is zero.
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Balances::free_balance(&account_from(11)), 1000);

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );

        // The validator hasn't been slashed. The new era is not forced.
        assert_eq!(Balances::free_balance(&account_from(11)), 1000);
        assert_ledger_consistent(11);
    });
}

#[test]
fn only_slash_for_max_in_era() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Balances::free_balance(&account_from(11)), 1000);

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(50)],
        );

        // The validator has been slashed and has been force-chilled.
        assert_eq!(Balances::free_balance(&account_from(11)), 500);
        assert_eq!(Staking::force_era(), Forcing::ForceNew);

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(25)],
        );

        // The validator has not been slashed additionally.
        assert_eq!(Balances::free_balance(&account_from(11)), 500);

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(60)],
        );

        // The validator got slashed 10% more.
        assert_eq!(Balances::free_balance(&account_from(11)), 400);
        assert_ledger_consistent(11);
    })
}

#[test]
fn garbage_collection_after_slashing() {
    ExtBuilder::default()
        .existential_deposit(1)
        .build()
        .execute_with(|| {
            fix_nominator_genesis_problem(256250);
            start_era(1);
            assert_eq!(Balances::free_balance(&account_from(11)), 256_000);

            on_offence_now(
                &[OffenceDetails {
                    offender: (account_from(11), Staking::stakers(&account_from(11))),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            assert_eq!(Balances::free_balance(&account_from(11)), 256_000 - 25_600);
            assert!(<Staking as crate::Store>::SlashingSpans::get(&account_from(11)).is_some());
            assert_eq!(
                <Staking as crate::Store>::SpanSlash::get(&(account_from(11), 0)).amount_slashed(),
                &25_600
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (account_from(11), Staking::stakers(&account_from(11))),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(100)],
            );

            // validator and nominator slash in era are garbage-collected by era change,
            // so we don't test those here.

            assert_eq!(Balances::free_balance(&account_from(11)), 0);
            assert!(<Staking as crate::Store>::SlashingSpans::get(&account_from(11)).is_none());
            assert_eq!(
                <Staking as crate::Store>::SpanSlash::get(&(account_from(11), 0)).amount_slashed(),
                &0
            );
        })
}

#[test]
fn garbage_collection_on_window_pruning() {
    ExtBuilder::default().build().execute_with(|| {
        fix_nominator_genesis_problem(2250);
        start_era(1);

        assert_eq!(Balances::free_balance(&account_from(11)), 1000);

        let exposure = Staking::stakers(&account_from(11));
        assert_eq!(Balances::free_balance(&account_from(101)), 2000);
        let nominated_value = exposure
            .others
            .iter()
            .find(|o| o.who == account_from(101))
            .unwrap()
            .value;

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
        );

        let now = Staking::current_era();

        assert_eq!(Balances::free_balance(&account_from(11)), 900);
        assert_eq!(
            Balances::free_balance(&account_from(101)),
            2000 - (nominated_value / 10)
        );

        assert!(
            <Staking as crate::Store>::ValidatorSlashInEra::get(&now, &account_from(11)).is_some()
        );
        assert!(
            <Staking as crate::Store>::NominatorSlashInEra::get(&now, &account_from(101)).is_some()
        );

        // + 1 because we have to exit the bonding window.
        for era in (0..(BondingDuration::get() + 1)).map(|offset| offset + now + 1) {
            assert!(
                <Staking as crate::Store>::ValidatorSlashInEra::get(&now, &account_from(11))
                    .is_some()
            );
            assert!(
                <Staking as crate::Store>::NominatorSlashInEra::get(&now, &account_from(101))
                    .is_some()
            );

            start_era(era);
        }

        assert!(
            <Staking as crate::Store>::ValidatorSlashInEra::get(&now, &account_from(11)).is_none()
        );
        assert!(
            <Staking as crate::Store>::NominatorSlashInEra::get(&now, &account_from(101)).is_none()
        );
    })
}

#[test]
fn slashing_nominators_by_span_max() {
    ExtBuilder::default().build().execute_with(|| {
        fix_nominator_genesis_problem(2250);
        start_era(1);
        start_era(2);
        start_era(3);

        assert_eq!(Balances::free_balance(&account_from(11)), 1000);
        assert_eq!(Balances::free_balance(&account_from(21)), 2000);
        assert_eq!(Balances::free_balance(&account_from(101)), 2000);
        assert_eq!(Staking::slashable_balance_of(&account_from(21)), 1000);

        let exposure_11 = Staking::stakers(&account_from(11));
        let exposure_21 = Staking::stakers(&account_from(21));
        assert_eq!(Balances::free_balance(&account_from(101)), 2000);
        let nominated_value_11 = exposure_11
            .others
            .iter()
            .find(|o| o.who == account_from(101))
            .unwrap()
            .value;
        let nominated_value_21 = exposure_21
            .others
            .iter()
            .find(|o| o.who == account_from(101))
            .unwrap()
            .value;

        on_offence_in_era(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
            2,
        );

        assert_eq!(Balances::free_balance(&account_from(11)), 900);

        let slash_1_amount = Perbill::from_percent(10) * nominated_value_11;
        assert_eq!(
            Balances::free_balance(&account_from(101)),
            2000 - slash_1_amount
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

        let get_span = |account| <Staking as crate::Store>::SlashingSpans::get(&account).unwrap();

        assert_eq!(
            get_span(account_from(11)).iter().collect::<Vec<_>>(),
            expected_spans,
        );

        assert_eq!(
            get_span(account_from(101)).iter().collect::<Vec<_>>(),
            expected_spans,
        );

        // second slash: higher era, higher value, same span.
        on_offence_in_era(
            &[OffenceDetails {
                offender: (account_from(21), Staking::stakers(&account_from(21))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(30)],
            3,
        );

        // 11 was not further slashed, but 21 and 101 were.
        assert_eq!(Balances::free_balance(&account_from(11)), 900);
        assert_eq!(Balances::free_balance(&account_from(21)), 1700);

        let slash_2_amount = Perbill::from_percent(30) * nominated_value_21;
        assert!(slash_2_amount > slash_1_amount);

        // only the maximum slash in a single span is taken.
        assert_eq!(
            Balances::free_balance(&account_from(101)),
            2000 - slash_2_amount
        );

        // third slash: in same era and on same validator as first, higher
        // in-era value, but lower slash value than slash 2.
        on_offence_in_era(
            &[OffenceDetails {
                offender: (account_from(11), Staking::stakers(&account_from(11))),
                reporters: vec![],
            }],
            &[Perbill::from_percent(20)],
            2,
        );

        // 11 was further slashed, but 21 and 101 were not.
        assert_eq!(Balances::free_balance(&account_from(11)), 800);
        assert_eq!(Balances::free_balance(&account_from(21)), 1700);

        let slash_3_amount = Perbill::from_percent(20) * nominated_value_21;
        assert!(slash_3_amount < slash_2_amount);
        assert!(slash_3_amount > slash_1_amount);

        // only the maximum slash in a single span is taken.
        assert_eq!(
            Balances::free_balance(&account_from(101)),
            2000 - slash_2_amount
        );
    });
}

#[test]
fn slashes_are_summed_across_spans() {
    ExtBuilder::default().build().execute_with(|| {
        start_era(1);
        start_era(2);
        start_era(3);

        assert_eq!(Balances::free_balance(&account_from(21)), 2000);
        assert_eq!(Staking::slashable_balance_of(&account_from(21)), 1000);

        let get_span = |account| <Staking as crate::Store>::SlashingSpans::get(&account).unwrap();

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(21), Staking::stakers(&account_from(21))),
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

        assert_eq!(
            get_span(account_from(21)).iter().collect::<Vec<_>>(),
            expected_spans
        );
        assert_eq!(Balances::free_balance(&account_from(21)), 1900);

        // 21 has been force-chilled. re-signal intent to validate.
        Staking::validate(Origin::signed(account_from(20)), Default::default()).unwrap();

        start_era(4);

        assert_eq!(Staking::slashable_balance_of(&account_from(21)), 900);

        on_offence_now(
            &[OffenceDetails {
                offender: (account_from(21), Staking::stakers(&account_from(21))),
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

        assert_eq!(
            get_span(account_from(21)).iter().collect::<Vec<_>>(),
            expected_spans
        );
        assert_eq!(Balances::free_balance(&account_from(21)), 1810);
    });
}

#[test]
fn deferred_slashes_are_deferred() {
    ExtBuilder::default()
        .slash_defer_duration(2)
        .build()
        .execute_with(|| {
            println!(
                "free balance of the 101 nominator: {:?}",
                Balances::free_balance(&account_from(101))
            );
            fix_nominator_genesis_problem(2250);
            start_era(1);

            assert_eq!(Balances::free_balance(&account_from(11)), 1000);

            let exposure = Staking::stakers(&account_from(11));
            println!(
                "free balance of the 101 nominator: {:?}",
                Balances::free_balance(&account_from(101))
            );
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);
            let nominated_value = exposure
                .others
                .iter()
                .find(|o| o.who == account_from(101))
                .unwrap()
                .value;

            on_offence_now(
                &[OffenceDetails {
                    offender: (account_from(11), Staking::stakers(&account_from(11))),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);

            start_era(2);

            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);

            start_era(3);

            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);

            // at the start of era 4, slashes from era 1 are processed,
            // after being deferred for at least 2 full eras.
            start_era(4);

            assert_eq!(Balances::free_balance(&account_from(11)), 900);
            assert_eq!(
                Balances::free_balance(&account_from(101)),
                2000 - (nominated_value / 10)
            );
        })
}

#[test]
fn remove_deferred() {
    ExtBuilder::default()
        .slash_defer_duration(2)
        .build()
        .execute_with(|| {
            fix_nominator_genesis_problem(2250);
            start_era(1);

            assert_eq!(Balances::free_balance(&account_from(11)), 1000);

            let exposure = Staking::stakers(&account_from(11));
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);
            let nominated_value = exposure
                .others
                .iter()
                .find(|o| o.who == account_from(101))
                .unwrap()
                .value;

            on_offence_now(
                &[OffenceDetails {
                    offender: (account_from(11), exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);

            start_era(2);

            on_offence_in_era(
                &[OffenceDetails {
                    offender: (account_from(11), exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(15)],
                1,
            );

            Staking::cancel_deferred_slash(Origin::ROOT, 1, vec![0]).unwrap();

            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);

            start_era(3);

            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);

            // at the start of era 4, slashes from era 1 are processed,
            // after being deferred for at least 2 full eras.
            start_era(4);

            // the first slash for 10% was cancelled, so no effect.
            assert_eq!(Balances::free_balance(&account_from(11)), 1000);
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);

            start_era(5);

            let slash_10 = Perbill::from_percent(10);
            let slash_15 = Perbill::from_percent(15);
            let initial_slash = slash_10 * nominated_value;

            let total_slash = slash_15 * nominated_value;
            let actual_slash = total_slash - initial_slash;

            // 5% slash (15 - 10) processed now.
            assert_eq!(Balances::free_balance(&account_from(11)), 950);
            assert_eq!(
                Balances::free_balance(&account_from(101)),
                2000 - actual_slash
            );
        })
}

#[test]
fn remove_multi_deferred() {
    ExtBuilder::default()
        .slash_defer_duration(2)
        .build()
        .execute_with(|| {
            start_era(1);

            assert_eq!(Balances::free_balance(&account_from(11)), 1000);

            let exposure = Staking::stakers(&account_from(11));
            assert_eq!(Balances::free_balance(&account_from(101)), 2000);

            on_offence_now(
                &[OffenceDetails {
                    offender: (account_from(11), exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (account_from(21), Staking::stakers(&account_from(21))),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (account_from(11), exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(25)],
            );

            assert_eq!(<Staking as Store>::UnappliedSlashes::get(&1).len(), 3);
            Staking::cancel_deferred_slash(Origin::ROOT, 1, vec![0, 2]).unwrap();

            let slashes = <Staking as Store>::UnappliedSlashes::get(&1);
            assert_eq!(slashes.len(), 1);
            assert_eq!(slashes[0].validator, account_from(21));
        })
}

#[test]
fn version_initialized() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            <Staking as Store>::StorageVersion::get(),
            crate::migration::CURRENT_VERSION
        );
    });
}

#[test]
fn add_nominator_with_invalid_expiry() {
    ExtBuilder::default()
        .nominate(true)
        .build()
        .execute_with(|| {
            let account_alice = AccountId::from(AccountKeyring::Alice);
            let (alice_signed, alice_did) =
                make_account_with_balance(account_alice.clone(), 1_000_000).unwrap();
            let account_alice_controller = AccountId::from(AccountKeyring::Dave);
            let controller_signed = Origin::signed(account_alice_controller.clone());

            // For valid trusted KYC service providers
            let account_bob = AccountId::from(AccountKeyring::Bob);
            let (bob_signed, bob_did) = make_account(account_bob.clone()).unwrap();
            add_trusted_kyc_provider(bob_did);

            let now = Utc::now();
            // Add nominator claim
            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: (now.timestamp() as u64).to_be_bytes().to_vec(),
            };

            add_nominator_claim(
                bob_did,
                alice_did,
                account_bob.clone(),
                account_alice.clone(),
                claim,
            );

            // bond
            assert_ok!(Staking::bond(
                Origin::signed(account_alice.clone()),
                account_alice_controller,
                1000,
                RewardDestination::Stash
            ));

            let now = Utc::now();
            Timestamp::set_timestamp(now.timestamp() as u64);
            let validators = vec![account_from(10), account_from(20), account_from(30)];
            assert_ok!(Staking::nominate(controller_signed.clone(), validators));
            assert!(Staking::nominators(&account_alice).is_none());
        });
}

#[test]
fn add_valid_nominator_with_multiple_claims() {
    ExtBuilder::default()
        .nominate(true)
        .build()
        .execute_with(|| {
            let account_alice = AccountId::from(AccountKeyring::Alice);
            let (alice_signed, alice_did) =
                make_account_with_balance(account_alice.clone(), 1_000_000).unwrap();

            let account_alice_controller = AccountId::from(AccountKeyring::Dave);
            let controller_signed = Origin::signed(account_alice_controller.clone());

            let claim_issuer_1 = AccountId::from(AccountKeyring::Bob);
            let (claim_issuer_1_signed, claim_issuer_1_did) =
                make_account(claim_issuer_1.clone()).unwrap();
            add_trusted_kyc_provider(claim_issuer_1_did);

            let now = Utc::now();
            // Add nominator claim
            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: (now.timestamp() as u64).to_be_bytes().to_vec(),
            };

            add_nominator_claim(
                claim_issuer_1_did,
                alice_did,
                claim_issuer_1.clone(),
                account_alice.clone(),
                claim,
            );

            // add one more claim issuer
            let claim_issuer_2 = AccountId::from(AccountKeyring::Charlie);
            let (claim_issuer_2_signed, claim_issuer_2_did) =
                make_account(claim_issuer_2.clone()).unwrap();
            add_trusted_kyc_provider(claim_issuer_2_did);

            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: ((now.timestamp() as u64) + 7000_u64).to_be_bytes().to_vec(),
            };

            // add claim by claim issuer
            add_nominator_claim(
                claim_issuer_2_did,
                alice_did,
                claim_issuer_2.clone(),
                account_alice.clone(),
                claim,
            );

            // bond
            assert_ok!(Staking::bond(
                Origin::signed(account_alice.clone()),
                account_alice_controller,
                1000,
                RewardDestination::Stash
            ));

            Timestamp::set_timestamp(now.timestamp() as u64);
            let validators = vec![account_from(10), account_from(20), account_from(30)];

            assert_ok!(Staking::nominate(controller_signed.clone(), validators));
            assert!(!Staking::nominators(&account_alice).is_none());
        });
}

#[test]
fn validate_nominators_with_valid_kyc() {
    ExtBuilder::default()
        .nominate(true)
        .build()
        .execute_with(|| {
            let account_alice = AccountId::from(AccountKeyring::Alice);
            let (alice_signed, alice_did) =
                make_account_with_balance(account_alice.clone(), 1_000_000).unwrap();

            let account_alice_controller = AccountId::from(AccountKeyring::Dave);
            let controller_signed_alice = Origin::signed(account_alice_controller.clone());

            let claim_issuer_1 = AccountId::from(AccountKeyring::Bob);
            let (claim_issuer_1_signed, claim_issuer_1_did) =
                make_account(claim_issuer_1.clone()).unwrap();
            add_trusted_kyc_provider(claim_issuer_1_did);

            let account_eve = AccountId::from(AccountKeyring::Eve);
            let (eve_signed, eve_did) =
                make_account_with_balance(account_eve.clone(), 1_000_000).unwrap();

            let account_eve_controller = AccountId::from(AccountKeyring::Ferdie);
            let controller_signed_eve = Origin::signed(account_eve_controller.clone());

            let claim_issuer_2 = AccountId::from(AccountKeyring::Charlie);
            let (claim_issuer_2_signed, claim_issuer_2_did) =
                make_account(claim_issuer_2.clone()).unwrap();
            add_trusted_kyc_provider(claim_issuer_2_did);

            let now = Utc::now();
            // Add nominator claim
            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: ((now.timestamp() as u64) + 500_u64).to_be_bytes().to_vec(),
            };

            add_nominator_claim(
                claim_issuer_1_did,
                alice_did,
                claim_issuer_1.clone(),
                account_alice.clone(),
                claim,
            );

            let claim = ClaimValue {
                data_type: DataTypes::U64,
                value: ((now.timestamp() as u64) + 7000_u64).to_be_bytes().to_vec(),
            };
            // add claim by claim issuer
            add_nominator_claim(
                claim_issuer_2_did,
                eve_did,
                claim_issuer_2.clone(),
                account_eve.clone(),
                claim,
            );

            // bond
            assert_ok!(Staking::bond(
                Origin::signed(account_alice.clone()),
                account_alice_controller.clone(),
                1000,
                RewardDestination::Stash
            ));

            // bond
            assert_ok!(Staking::bond(
                Origin::signed(account_eve.clone()),
                account_eve_controller,
                1000,
                RewardDestination::Stash
            ));

            Timestamp::set_timestamp(now.timestamp() as u64);
            let validators_1 = vec![account_from(10), account_from(20), account_from(30)];
            assert_ok!(Staking::nominate(
                controller_signed_alice.clone(),
                validators_1
            ));
            assert!(!Staking::nominators(&account_alice).is_none());

            let validators_2 = vec![account_from(11), account_from(21), account_from(31)];
            assert_ok!(Staking::nominate(
                controller_signed_eve.clone(),
                validators_2
            ));
            assert!(!Staking::nominators(&account_eve).is_none());

            Timestamp::set_timestamp((now.timestamp() as u64) + 800_u64);
            let claimed_nominator = vec![account_alice.clone(), account_eve.clone()];

            assert_ok!(Staking::validate_kyc_expiry_nominators(
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
        assert_eq!(Staking::bonded(&account_from(11)), Some(account_from(10))); // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(&account_from(21)), Some(account_from(20))); // Account 21 is stashed and locked, and account 20 is the controller
        assert_eq!(Staking::bonded(&AccountKeyring::Alice.public()), None); // Account 1 is not a stashed

        // Account 10 controls the stash from account 11, which is 100 * balance_factor units
        assert_eq!(
            Staking::ledger(&account_from(10)),
            Some(StakingLedger {
                stash: account_from(11),
                total: 1000,
                active: 1000,
                unlocking: vec![]
            })
        );
        // Account 20 controls the stash from account 21, which is 200 * balance_factor units
        assert_eq!(
            Staking::ledger(&account_from(20)),
            Some(StakingLedger {
                stash: account_from(21),
                total: 1000,
                active: 1000,
                unlocking: vec![]
            })
        );
        // Account 1 does not control any stash
        assert_eq!(Staking::ledger(&AccountKeyring::Alice.public()), None);
    });
}

#[test]
fn should_add_potential_validators() {
    ExtBuilder::default()
        .minimum_validator_count(2)
        .validator_count(2)
        .num_validators(2)
        .validator_pool(true)
        .nominate(false)
        .build()
        .execute_with(|| {
            let acc_10 = account_from(10);
            let acc_20 = account_from(20);

            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                acc_10.clone()
            ));
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                acc_20.clone()
            ));

            assert_ok!(Staking::compliance_failed(
                Origin::signed(account_from(3000)),
                acc_20.clone()
            ));
            assert_eq!(
                Staking::permissioned_validators(acc_10).unwrap().compliance,
                Compliance::Active
            );
            assert_eq!(
                Staking::permissioned_validators(acc_20).unwrap().compliance,
                Compliance::Pending
            );
        });
}

#[test]
fn should_remove_validators() {
    ExtBuilder::default()
        .minimum_validator_count(2)
        .validator_count(2)
        .num_validators(2)
        .validator_pool(true)
        .nominate(false)
        .build()
        .execute_with(|| {
            let acc_10 = account_from(10);
            let acc_20 = account_from(20);
            let acc_30 = account_from(30);

            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                acc_10.clone()
            ));
            assert_ok!(Staking::add_potential_validator(
                Origin::signed(account_from(1000)),
                acc_20.clone()
            ));

            assert_ok!(Staking::compliance_failed(
                Origin::signed(account_from(3000)),
                acc_20.clone()
            ));

            assert_ok!(Staking::remove_validator(
                Origin::signed(account_from(2000)),
                acc_20.clone()
            ));

            assert_eq!(
                Staking::permissioned_validators(&acc_10),
                Some(PermissionedValidator {
                    compliance: Compliance::Active
                })
            );
            assert_eq!(Staking::permissioned_validators(&acc_20), None);

            assert_eq!(Staking::permissioned_validators(&acc_30), None);
        });
}

#[test]
fn new_era_pays_rewards() {
    ExtBuilder::default()
        .minimum_validator_count(2)
        .validator_count(2)
        .num_validators(2)
        .validator_pool(true)
        .nominate(false)
        .build()
        .execute_with(|| {
            // initial validators
            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(10), account_from(20)]
            );

            // trigger era
            System::set_block_number(1);
            Session::on_initialize(System::block_number());

            assert_eq_uvec!(
                validator_controllers(),
                vec![account_from(10), account_from(20)]
            );

            // TODO: check the block rewards.
        });
}
