use frame_support::traits::fungible::Inspect;
use frame_support::traits::schedule::v3::Anon as ScheduleAnon;
use frame_support::traits::schedule::{DispatchTime, HIGHEST_PRIORITY};
use frame_support::{pallet_prelude::*, traits::Get};
use frame_system::RawOrigin;
use frame_system::{ensure_root, pallet_prelude::*};
use sp_runtime::traits::SaturatedConversion;
use sp_std::prelude::*;

use polymesh_primitives::IdentityId;
use polymesh_primitives::GC_DID;

#[cfg(feature = "runtime-benchmarks")]
use polymesh_primitives::{traits::IdentityFnTrait, AuthorizationData, Permissions, Signatory};

use pallet_staking::permissioned_staking::{PermissionedStaking, WhoToSlash};

use crate::types::{PermissionedIdentityPrefs, SlashingSwitch};
use crate::*;

/// Adaptor to turn a `PiecewiseLinear` curve definition into an `EraPayout` impl, used for
/// backwards compatibility.
pub struct PolymeshConvertCurve<T, C>(sp_std::marker::PhantomData<(T, C)>);

impl<
        Balance: AtLeast32BitUnsigned + Clone,
        T: Config<CurrencyBalance = Balance>,
        C: Get<&'static PiecewiseLinear<'static>>,
    > EraPayout<Balance> for PolymeshConvertCurve<T, C>
{
    fn era_payout(
        total_staked: Balance,
        total_issuance: Balance,
        era_duration_millis: u64,
    ) -> (Balance, Balance) {
        let (validator_payout, max_payout) = inflation::compute_total_payout(
            C::get(),
            total_staked,
            total_issuance,
            // Duration of era; more than u64::MAX is rewarded as u64::MAX.
            era_duration_millis,
            T::MaxVariableInflationTotalIssuance::get(),
            T::FixedYearlyReward::get(),
        );
        let rest = max_payout.saturating_sub(validator_payout.clone());
        (validator_payout, rest)
    }
}

impl<T: Config> PermissionedStaking<T> for Pallet<T> {
    /// Onboard an account.
    #[cfg(feature = "runtime-benchmarks")]
    fn onboard_account(who: &T::AccountId) {
        let _ = T::IdentityFn::testing_register_did(who.clone());
    }

    /// Permission a validator.
    #[cfg(feature = "runtime-benchmarks")]
    fn permission_validator(stash: &T::AccountId) {
        let _ = Pallet::<T>::set_commission_cap(RawOrigin::Root.into(), Perbill::from_percent(99));
        let did =
            pallet_identity::Pallet::<T>::get_identity(stash).expect("Failed to get identity");
        Pallet::<T>::add_permissioned_validator(RawOrigin::Root.into(), did, Some(2))
            .expect("Failed to add permissioned validator");
    }

    /// Setup stash and controller.
    #[cfg(feature = "runtime-benchmarks")]
    fn setup_stash_and_controller(stash: &T::AccountId, controller: &T::AccountId) {
        let did =
            pallet_identity::Pallet::<T>::get_identity(stash).expect("Failed to get identity");
        let auth_id = pallet_identity::Pallet::<T>::add_auth(
            did,
            Signatory::Account(controller.clone()),
            AuthorizationData::JoinIdentity(Permissions::default()),
            None,
        )
        .expect("Failed to add auth");
        pallet_identity::Pallet::<T>::join_identity_as_key(
            RawOrigin::Signed(controller.clone()).into(),
            auth_id,
        )
        .expect("Failed to join identity as key");
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn setup_who_to_slash(who_to_slash: Option<WhoToSlash>) {
        match who_to_slash {
            Some(WhoToSlash::ValidatorAndNominator) => {
                SlashingAllowedFor::<T>::put(SlashingSwitch::ValidatorAndNominator)
            }
            Some(WhoToSlash::Validator) => SlashingAllowedFor::<T>::put(SlashingSwitch::Validator),
            None => SlashingAllowedFor::<T>::put(SlashingSwitch::None),
        }
    }

    /// Check if amount is under the existential deposit.
    fn reapable(amount: BalanceOf<T>) -> bool {
        amount <= T::Currency::minimum_balance()
    }

    fn on_validate(stash: &T::AccountId, commission: Perbill) -> DispatchResult {
        // ensure their commission is correct.
        ensure!(
            commission <= ValidatorCommissionCap::<T>::get(),
            Error::<T>::CommissionTooHigh
        );

        // Only increment running_count and ref count for new validators.
        // Existing validators are already counted.
        if !pallet_staking::Validators::<T>::contains_key(stash) {
            let stash_did = pallet_identity::Pallet::<T>::get_identity(stash)
                .ok_or(Error::<T>::StashIdentityDoesNotExist)?;
            let mut stash_did_prefs = PermissionedIdentity::<T>::get(stash_did)
                .ok_or(Error::<T>::StashIdentityNotPermissioned)?;

            // Ensure the identity doesn't run more validators than the intended count
            ensure!(
                stash_did_prefs.running_count < stash_did_prefs.intended_count,
                StakingError::<T>::TooManyValidators
            );
            stash_did_prefs.running_count += 1;
            pallet_identity::Pallet::<T>::add_account_key_ref_count(&stash);
            PermissionedIdentity::<T>::insert(stash_did, stash_did_prefs);
        }

        Ok(())
    }

    /// On chill hook.
    fn on_chill(stash: &T::AccountId) {
        Self::release_running_validator(stash);
    }

    /// On nominate hook.
    fn on_nominate(stash: &T::AccountId) -> DispatchResult {
        Self::release_running_validator(stash);
        Ok(())
    }

    /// Returns `true` if `stash` has an active DID and is permissioned. Otherwise, returns `false`.
    fn is_validator_compliant(stash: &T::AccountId) -> bool {
        pallet_identity::Pallet::<T>::get_identity(stash).map_or(false, |id| {
            !pallet_identity::Pallet::<T>::is_did_locked(id)
                && PermissionedIdentity::<T>::get(id).is_some()
                && Self::is_validator_active_balance_valid(stash)
        })
    }

    /// Returns `true` if `who` has an active DID. Otherwise, returns `false`.
    fn is_nominator_compliant(_who: &T::AccountId) -> bool {
        true
    }

    /// Schedule reward payouts.
    fn schedule_payouts(active_era: &ActiveEraInfo) -> DispatchResult {
        let next_block_number = <frame_system::Pallet<T>>::block_number() + 1u32.into();
        for (index, validator_id) in <T as StakingConfig>::SessionInterface::validators()
            .into_iter()
            .enumerate()
        {
            let schedule_block_number =
                next_block_number + index.saturated_into::<BlockNumberFor<T>>();

            let scheduler_call =
                <T as pallet::Config>::SchedulerCall::from(Call::<T>::payout_stakers_by_system {
                    validator_stash: validator_id.clone(),
                    era: active_era.index,
                });

            let payout_call = <T as pallet::Config>::SchedulerPreimage::bound(scheduler_call)?;

            match T::RewardScheduler::schedule(
                DispatchTime::At(schedule_block_number),
                None,
                HIGHEST_PRIORITY,
                RawOrigin::Root.into(),
                payout_call
            ) {
                Ok(_) => log!(
                    info,
                    "💸 Rewards are successfully scheduled for validator id: {:?} at block number: {:?}",
                    &validator_id,
                    schedule_block_number,
                ),
                Err(error) => {
                    log!(
                        error,
                        "⛔ Detected error in scheduling the reward payment: {:?}",
                        error
                    );
                    Pallet::<T>::deposit_event(Event::<T>::RewardPaymentSchedulingInterrupted {
                        account_id: validator_id,
                        era: active_era.index,
                        error
                    });
                }
            }
        }

        Ok(())
    }

    /// Who should be slashed?
    /// Returns `None` if no one should be slashed.
    fn who_to_slash() -> Option<WhoToSlash> {
        SlashingAllowedFor::<T>::get().into()
    }
}

impl<T: Config> Pallet<T> {
    /// Returns `true` if active balance is above [`MinValidatorBond`]. Otherwise, returns `false`.
    pub(crate) fn is_validator_active_balance_valid(who: &T::AccountId) -> bool {
        if let Some(controller) = Bonded::<T>::get(&who) {
            if let Some(ledger) = Ledger::<T>::get(&controller) {
                return ledger.active >= MinValidatorBond::<T>::get();
            }
        }
        false
    }

    /// Decrease the running count of validators by 1 for the stash identity.
    pub(crate) fn release_running_validator(stash: &T::AccountId) {
        if !<Validators<T>>::contains_key(stash) {
            return;
        }

        if let Some(did) = pallet_identity::Pallet::<T>::get_identity(stash) {
            PermissionedIdentity::<T>::mutate(&did, |preferences| {
                if let Some(p) = preferences {
                    if p.running_count > 0 {
                        p.running_count -= 1;
                    }
                }
            });
            pallet_identity::Pallet::<T>::remove_account_key_ref_count(&stash);
        }
    }

    /// Returns the maximum number of validators per identiy
    pub fn maximum_number_of_validators_per_identity() -> u32 {
        (T::MaxValidatorPerIdentity::get() * ValidatorCount::<T>::get()).max(1)
    }

    pub(crate) fn base_add_permissioned_validator(
        origin: OriginFor<T>,
        identity: IdentityId,
        intended_count: Option<u32>,
    ) -> DispatchResult {
        T::AdminOrigin::ensure_origin(origin)?;

        ensure!(
            Self::permissioned_identity(&identity).is_none(),
            Error::<T>::IdentityIsAlreadyPermissioned
        );

        ensure!(
            !pallet_identity::Pallet::<T>::is_did_locked(identity),
            Error::<T>::IdentityIsInactive
        );

        match intended_count {
            Some(intended_count) => {
                ensure!(
                    intended_count < Self::maximum_number_of_validators_per_identity(),
                    Error::<T>::IntendedCountIsExceedingConsensusLimit
                );
                PermissionedIdentity::<T>::insert(
                    &identity,
                    PermissionedIdentityPrefs::new(intended_count),
                );
            }
            None => {
                PermissionedIdentity::<T>::insert(&identity, PermissionedIdentityPrefs::default());
            }
        }

        Pallet::<T>::deposit_event(Event::<T>::PermissionedIdentityAdded {
            governance_councill_did: GC_DID,
            validators_identity: identity,
        });
        Ok(())
    }

    pub(crate) fn base_remove_permissioned_validator(
        origin: OriginFor<T>,
        identity: IdentityId,
    ) -> DispatchResult {
        T::AdminOrigin::ensure_origin(origin)?;

        ensure!(
            Self::permissioned_identity(&identity).is_some(),
            Error::<T>::IdentityNotFound
        );

        PermissionedIdentity::<T>::remove(&identity);

        Pallet::<T>::deposit_event(Event::<T>::PermissionedIdentityRemoved {
            governance_councill_did: GC_DID,
            validators_identity: identity,
        });
        Ok(())
    }

    pub(crate) fn base_payout_stakers_by_system(
        origin: OriginFor<T>,
        validator_stash: T::AccountId,
        era: EraIndex,
    ) -> DispatchResultWithPostInfo {
        ensure_root(origin)?;
        StakingPallet::<T>::do_payout_stakers(validator_stash, era)
    }

    pub(crate) fn base_change_slashing_allowed_for(
        origin: OriginFor<T>,
        slashing_switch: SlashingSwitch,
    ) -> DispatchResult {
        ensure_root(origin)?;
        SlashingAllowedFor::<T>::put(slashing_switch);
        Pallet::<T>::deposit_event(Event::<T>::SlashingAllowedForChanged { slashing_switch });
        Ok(())
    }

    pub(crate) fn base_update_permissioned_validator_intended_count(
        origin: OriginFor<T>,
        identity: IdentityId,
        new_intended_count: u32,
    ) -> DispatchResult {
        T::AdminOrigin::ensure_origin(origin)?;

        ensure!(
            Self::maximum_number_of_validators_per_identity() > new_intended_count,
            Error::<T>::IntendedCountIsExceedingConsensusLimit
        );

        PermissionedIdentity::<T>::try_mutate(&identity, |pref| {
            pref.as_mut()
                .ok_or_else(|| Error::<T>::IdentityNotFound.into())
                .map(|p| p.intended_count = new_intended_count)
        })
    }

    pub(crate) fn base_set_commission_cap(
        origin: OriginFor<T>,
        new_cap: Perbill,
    ) -> DispatchResult {
        T::AdminOrigin::ensure_origin(origin.clone())?;

        // Update the cap, assuming it changed, or error.
        let old_cap = ValidatorCommissionCap::<T>::try_mutate(|cap| -> Result<_, DispatchError> {
            ensure!(*cap != new_cap, Error::<T>::CommissionUnchanged);
            Ok(core::mem::replace(cap, new_cap))
        })?;

        // Update `commission` in each validator prefs to `min(comission, new_cap)`.
        <Validators<T>>::translate(|_, mut prefs: ValidatorPrefs| {
            prefs.commission = prefs.commission.min(new_cap);
            Some(prefs)
        });

        Pallet::<T>::deposit_event(Event::<T>::CommissionCapUpdated {
            governance_councill_did: GC_DID,
            old_commission_cap: old_cap,
            new_commission_cap: new_cap,
        });

        Ok(())
    }
    pub(crate) fn base_chill_from_governance(
        origin: T::RuntimeOrigin,
        identity: IdentityId,
        stash_keys: Vec<T::AccountId>,
    ) -> DispatchResult {
        T::AdminOrigin::ensure_origin(origin)?;

        for key in &stash_keys {
            let key_did = pallet_identity::Pallet::<T>::get_identity(&key);
            // Checks if the stash key identity is the same as the identity given.
            ensure!(key_did == Some(identity), StakingError::<T>::NotStash);
            // Checks if the key is a validator if not returns an error.
            ensure!(
                <Validators<T>>::contains_key(&key),
                Error::<T>::ValidatorNotFound
            );
        }

        for key in stash_keys {
            StakingPallet::<T>::chill_stash(&key);
        }

        // Change identity status to be Non-Permissioned
        PermissionedIdentity::<T>::remove(&identity);

        Pallet::<T>::deposit_event(Event::<T>::PermissionedIdentityRemoved {
            governance_councill_did: GC_DID,
            validators_identity: identity,
        });
        Ok(())
    }
}
