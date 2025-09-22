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

use super::*;
use testing_utils::*;

pub use frame_benchmarking::v1::{account, benchmarks, impl_benchmark_test_suite};
pub use frame_benchmarking::v1::{whitelist_account, whitelisted_caller};
use frame_support::traits::fungible::Balanced;
use frame_system::RawOrigin;
use sp_runtime::traits::{One, StaticLookup};
use sp_runtime::{Perbill, Saturating};
use sp_staking::SessionIndex;
use sp_std::prelude::*;

use pallet_identity::benchmarking::UserBuilder;
use pallet_staking::{CurrentEra, EraRewardPoints, ErasRewardPoints};
use pallet_staking::{ErasValidatorReward, Nominators, RewardDestination};
use polymesh_primitives::IdentityId;
use polymesh_primitives::Permissions;

use crate::types::SlashingSwitch;

pub fn get_did<T: Config>(who: &T::AccountId) -> IdentityId {
    pallet_identity::Pallet::<T>::get_identity(who).expect("Failed to get identity id")
}

// This function clears all existing validators and nominators from the set, and generates one new
// validator being nominated by n nominators, and returns the validator stash account and the
// nominators' stash and controller. It also starts an era and creates pending payouts.
pub fn create_validator_with_nominators<T: Config>(
    n: u32,
    upper_bound: u32,
    dead_controller: bool,
    unique_controller: bool,
    destination: RewardDestination<T::AccountId>,
) -> Result<(T::AccountId, Vec<(T::AccountId, T::AccountId)>), &'static str> {
    // Clean up any existing state.
    clear_validators_and_nominators::<T>();
    let mut points_total = 0;
    let mut points_individual = Vec::new();

    let (v_stash, v_controller) = if unique_controller {
        create_unique_stash_controller::<T>(0, 100, destination.clone(), false)?
    } else {
        create_stash_controller::<T>(0, 100, destination.clone())?
    };

    let validator_prefs = ValidatorPrefs {
        commission: Perbill::from_percent(50),
        ..Default::default()
    };
    add_permissioned_validator_::<T>(&v_stash);
    StakingPallet::<T>::validate(RawOrigin::Signed(v_controller).into(), validator_prefs)?;
    let stash_lookup = T::Lookup::unlookup(v_stash.clone());

    points_total += 10;
    points_individual.push((v_stash.clone(), 10));

    let original_nominator_count = Nominators::<T>::count();
    let mut nominators = Vec::new();

    // Give the validator n nominators, but keep total users in the system the same.
    for i in 0..upper_bound {
        let (n_stash, n_controller) = if !dead_controller {
            create_stash_controller::<T>(u32::MAX - i, 100, destination.clone())?
        } else {
            create_unique_stash_controller::<T>(u32::MAX - i, 100, destination.clone(), true)?
        };
        if i < n {
            StakingPallet::<T>::nominate(
                RawOrigin::Signed(n_controller.clone()).into(),
                vec![stash_lookup.clone()],
            )?;
            nominators.push((n_stash, n_controller));
        }
    }

    ValidatorCount::<T>::put(1);

    // Start a new Era
    let new_validators =
        StakingPallet::<T>::try_trigger_new_era(SessionIndex::one(), true).unwrap();

    assert_eq!(new_validators.len(), 1);
    assert_eq!(
        new_validators[0], v_stash,
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
    let total_payout = minimum_balance::<T>()
        .saturating_mul(upper_bound.into())
        .saturating_mul(1000u32.into());
    <ErasValidatorReward<T>>::insert(current_era, total_payout);

    Ok((v_stash, nominators))
}

const USER_SEED: u32 = 999666;

benchmarks! {
    add_permissioned_validator {
        clear_validators_and_nominators::<T>();
        let (stash, controller) =
            create_stash_controller::<T>(1, 1, RewardDestination::Staked)?;
        StakingPallet::<T>::set_validator_count(RawOrigin::Root.into(), 10).unwrap();
        let did = get_did::<T>(&stash);
    }: _(RawOrigin::Root, did, Some(1))
    verify {
        let identity_preferences = Pallet::<T>::permissioned_identity(did);
        assert!(identity_preferences.is_some());
        assert_eq!(identity_preferences.unwrap().intended_count, 1);
    }

    remove_permissioned_validator {
        clear_validators_and_nominators::<T>();
        let (stash, controller) =
            create_stash_controller::<T>(1, 1, RewardDestination::Staked)?;
        add_permissioned_validator_::<T>(&stash);
        let did = get_did::<T>(&stash);
    }: _(RawOrigin::Root, did)
    verify {
        let identity_preferences = Pallet::<T>::permissioned_identity(did);
        assert!(identity_preferences.is_none());
    }

    change_slashing_allowed_for {}: _(RawOrigin::Root, SlashingSwitch::ValidatorAndNominator)
    verify {
        assert_eq!(Pallet::<T>::slashing_allowed_for(), SlashingSwitch::ValidatorAndNominator);
    }

    update_permissioned_validator_intended_count {
        clear_validators_and_nominators::<T>();
        let (stash, controller) =
            create_stash_controller::<T>(1, 1, RewardDestination::Staked)?;
        add_permissioned_validator_::<T>(&stash);
        let did = get_did::<T>(&stash);
    }: _(RawOrigin::Root, did, 2)
    verify {
        assert_eq!(Pallet::<T>::permissioned_identity(did).unwrap().intended_count, 2);
    }

    chill_from_governance {
        let s in 1..100;

        let validator = create_funded_user::<T>("validator", USER_SEED, 10_000);

        StakingPallet::<T>::set_validator_count(RawOrigin::Root.into(), 1_000).unwrap();
        assert_eq!(StakingPallet::<T>::validator_count(), 1_000);

        let did = get_did::<T>(&validator);
        Pallet::<T>::add_permissioned_validator(
            RawOrigin::Root.into(),
            did,
            Some(100)
        ).unwrap();

        whitelist_account!(validator);

        let mut signatories = Vec::new();
        for x in 0 .. s {
            let key = UserBuilder::<T>::default().seed(x).balance(10_000u32).build("key").account();
            let _ = T::Currency::issue(10_000u32.into());

            let did = get_did::<T>(&validator);
            pallet_identity::Pallet::<T>::unsafe_join_identity(
                did,
                Permissions::default(),
                key.clone()
            );
            StakingPallet::<T>::bond(
                RawOrigin::Signed(key.clone()).into(),
                2_000_000u32.into(),
                RewardDestination::Staked
            )
            .unwrap();
            whitelist_account!(key);

            StakingPallet::<T>::validate(RawOrigin::Signed(key.clone()).into(), ValidatorPrefs::default())?;
            assert_eq!(<Validators<T>>::contains_key(&key), true);
            signatories.push(key.clone());
        }
        let did = get_did::<T>(&validator);
    }: _(RawOrigin::Root, did, signatories.clone())
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
            stashes.push(stash.clone());
            Validators::<T>::insert(
                stash,
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
                StakingPallet::<T>::validators(s),
                ValidatorPrefs { commission: Perbill::from_percent(50), ..Default::default() }
            );
        });
    }
}
