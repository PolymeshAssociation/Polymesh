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

//! Implementations for the Staking FRAME Pallet.

use frame_election_provider_support::{
    data_provider, BoundedSupportsOf, ElectionDataProvider, ElectionProvider, ScoreProvider,
    SortedListProvider, VoteWeight, VoterOf,
};
use frame_support::{
    dispatch::WithPostDispatchInfo,
    pallet_prelude::*,
    traits::{
        Currency, CurrencyToVote, Defensive, DefensiveResult, EstimateNextNewSession, Get,
        Imbalance, LockableCurrency, OnUnbalanced, TryCollect, UnixTime, WithdrawReasons,
    },
    weights::Weight,
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use pallet_session::historical;
use sp_runtime::{
    traits::{Bounded, Convert, One, SaturatedConversion, Saturating, StaticLookup, Zero},
    Perbill,
};
use sp_staking::{
    offence::{DisableStrategy, OffenceDetails, OnOffenceHandler},
    EraIndex, SessionIndex, Stake, StakingInterface,
};
use sp_std::prelude::*;

use crate::{
    log, slashing, weights::WeightInfo, ActiveEraInfo, BalanceOf, EraPayout, Exposure, ExposureOf,
    Forcing, IndividualExposure, Nominations, PositiveImbalanceOf, RewardDestination,
    SessionInterface, StakingLedger, ValidatorPrefs,
};

use super::{pallet::*, STAKING_ID};

use frame_support::traits::DefensiveSaturating;
use sp_npos_elections::{
    Assignment, ElectionScore, Supports, to_support_map, EvaluateSupport, SupportMap, 
    ExtendedBalance
};

use polymesh_common_utilities::Context;
use polymesh_primitives::IdentityId;

use crate::{UnlockChunk, ValidatorIndex, CompactAssignments, NominatorIndex, OffchainAccuracy};
use crate::types::{ElectionSize, ElectionCompute, ElectionResult};
use crate::_feps::NposSolution;

/// The maximum number of iterations that we do whilst iterating over `T::VoterList` in
/// `get_npos_voters`.
///
/// In most cases, if we want n items, we iterate exactly n times. In rare cases, if a voter is
/// invalid (for any reason) the iteration continues. With this constant, we iterate at most 2 * n
/// times and then give up.
const NPOS_MAX_ITERATIONS_COEFFICIENT: u32 = 2;

type Identity<T> = pallet_identity::Module<T>;

impl<T: Config> Pallet<T> {
    /// The total balance that can be slashed from a stash account as of right now.
    pub fn slashable_balance_of(stash: &T::AccountId) -> BalanceOf<T> {
        // Weight note: consider making the stake accessible through stash.
        Self::bonded(stash)
            .and_then(Self::ledger)
            .map(|l| l.active)
            .unwrap_or_default()
    }

    /// Internal impl of [`Self::slashable_balance_of`] that returns [`VoteWeight`].
    pub fn slashable_balance_of_vote_weight(
        stash: &T::AccountId,
        issuance: BalanceOf<T>,
    ) -> VoteWeight {
        T::CurrencyToVote::to_vote(Self::slashable_balance_of(stash), issuance)
    }

    /// Returns a closure around `slashable_balance_of_vote_weight` that can be passed around.
    ///
    /// This prevents call sites from repeatedly requesting `total_issuance` from backend. But it is
    /// important to be only used while the total issuance is not changing.
    pub fn weight_of_fn() -> Box<dyn Fn(&T::AccountId) -> VoteWeight> {
        // NOTE: changing this to unboxed `impl Fn(..)` return type and the pallet will still
        // compile, while some types in mock fail to resolve.
        let issuance = T::Currency::total_issuance();
        Box::new(move |who: &T::AccountId| -> VoteWeight {
            Self::slashable_balance_of_vote_weight(who, issuance)
        })
    }

//    /// Same as `weight_of_fn`, but made for one time use.
//    pub fn weight_of(who: &T::AccountId) -> VoteWeight {
//        let issuance = T::Currency::total_issuance();
//        Self::slashable_balance_of_vote_weight(who, issuance)
//    }

    pub(super) fn do_withdraw_unbonded(
        controller: &T::AccountId,
        num_slashing_spans: u32,
    ) -> Result<Weight, DispatchError> {
        let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
        let (stash, old_total) = (ledger.stash.clone(), ledger.total);
        if let Some(current_era) = Self::current_era() {
            ledger = ledger.consolidate_unlocked(current_era)
        }

        let used_weight =
            if ledger.unlocking.is_empty() && ledger.active < T::Currency::minimum_balance() {
                // This account must have called `unbond()` with some value that caused the active
                // portion to fall below existential deposit + will have no more unlocking chunks
                // left. We can now safely remove all staking-related information.
                Self::kill_stash(&stash, num_slashing_spans)?;
                // Remove the lock.
                T::Currency::remove_lock(STAKING_ID, &stash);

                //T::WeightInfo::withdraw_unbonded_kill(num_slashing_spans)
                unimplemented!()
            } else {
                // This was the consequence of a partial unbond. just update the ledger and move on.
                Self::update_ledger(&controller, &ledger);

                // This is only an update, so we use less overall weight.
                //T::WeightInfo::withdraw_unbonded_update(num_slashing_spans)
                unimplemented!()
            };

        // `old_total` should never be less than the new total because
        // `consolidate_unlocked` strictly subtracts balance.
        if ledger.total < old_total {
            // Already checked that this won't overflow by entry condition.
            let value = old_total - ledger.total;
            Self::deposit_event(Event::<T>::Withdrawn(stash, value));
        }

        Ok(used_weight)
    }

    pub(super) fn do_payout_stakers(
        validator_stash: T::AccountId,
        era: EraIndex,
    ) -> DispatchResult {
        // Validate input data
        let current_era = CurrentEra::<T>::get().ok_or(Error::<T>::InvalidEraToReward)?;
        let history_depth = T::HistoryDepth::get();
        ensure!(
            era <= current_era && era >= current_era.saturating_sub(history_depth),
            Error::<T>::InvalidEraToReward
        );

        // Note: if era has no reward to be claimed, era may be future. better not to update
        // `ledger.claimed_rewards` in this case.
        let era_payout = <ErasValidatorReward<T>>::get(&era).ok_or(Error::<T>::InvalidEraToReward)?;

        let controller = Self::bonded(&validator_stash).ok_or(Error::<T>::NotStash)?;
        let mut ledger = <Ledger<T>>::get(&controller).ok_or(Error::<T>::NotController)?;

        ledger
            .claimed_rewards
            .retain(|&x| x >= current_era.saturating_sub(history_depth));

        match ledger.claimed_rewards.binary_search(&era) {
            Ok(_) => {
                return Err(Error::<T>::AlreadyClaimed.into())
            }
            Err(pos) => ledger
                .claimed_rewards
                .try_insert(pos, era)
                // Since we retain era entries in `claimed_rewards` only upto
                // `HistoryDepth`, following bound is always expected to be
                // satisfied.
                .defensive_map_err(|_| Error::<T>::BoundNotMet)?,
        }

        let exposure = <ErasStakersClipped<T>>::get(&era, &ledger.stash);

        // Input data seems good, no errors allowed after this point

        <Ledger<T>>::insert(&controller, &ledger);

        // Get Era reward points. It has TOTAL and INDIVIDUAL
        // Find the fraction of the era reward that belongs to the validator
        // Take that fraction of the eras rewards to split to nominator and validator
        //
        // Then look at the validator, figure out the proportion of their reward
        // which goes to them and each of their nominators.

        let era_reward_points = <ErasRewardPoints<T>>::get(&era);
        let total_reward_points = era_reward_points.total;
        let validator_reward_points = era_reward_points
            .individual
            .get(&ledger.stash)
            .copied()
            .unwrap_or_else(Zero::zero);

        // Nothing to do if they have no reward points.
        if validator_reward_points.is_zero() {
            return Ok(());
        }

        // This is the fraction of the total reward that the validator and the
        // nominators will get.
        let validator_total_reward_part =
            Perbill::from_rational(validator_reward_points, total_reward_points);

        // This is how much validator + nominators are entitled to.
        let validator_total_payout = validator_total_reward_part * era_payout;

        let validator_prefs = Self::eras_validator_prefs(&era, &validator_stash);
        // Validator first gets a cut off the top.
        let validator_commission = validator_prefs.commission;
        let validator_commission_payout = validator_commission * validator_total_payout;

        let validator_leftover_payout = validator_total_payout - validator_commission_payout;
        // Now let's calculate how this is split to the validator.
        let validator_exposure_part = Perbill::from_rational(exposure.own, exposure.total);
        let validator_staking_payout = validator_exposure_part * validator_leftover_payout;


        let mut total_imbalance = PositiveImbalanceOf::<T>::zero();
        // We can now make total validator payout:
        if let Some(imbalance) = Self::make_payout(
            &ledger.stash,
            validator_staking_payout + validator_commission_payout,
        ) {
            // Polymesh change: Provide DID of stash account.
            // -----------------------------------------------------------------
            let did = <Identity<T>>::get_identity(&ledger.stash).unwrap_or_default();
            Self::deposit_event(Event::<T>::Reward(did, ledger.stash, imbalance.peek()));
            // -----------------------------------------------------------------
            total_imbalance.subsume(imbalance);
        }

        // Track the number of payout ops to nominators. Note:
        // `WeightInfo::payout_stakers_alive_staked` always assumes at least a validator is paid
        // out, so we do not need to count their payout op.
        let mut nominator_payout_count: u32 = 0;

        // Lets now calculate how this is split to the nominators.
        // Reward only the clipped exposures. Note this is not necessarily sorted.
        for nominator in exposure.others.iter() {
            let nominator_exposure_part = Perbill::from_rational(nominator.value, exposure.total);

            let nominator_reward: BalanceOf<T> =
                nominator_exposure_part * validator_leftover_payout;
            // We can now make nominator payout:
            if let Some(imbalance) = Self::make_payout(&nominator.who, nominator_reward) {
                // Note: this logic does not count payouts for `RewardDestination::None`.
                nominator_payout_count += 1;
                // Polymesh change: Provide DID of nominator account.
                // -------------------------------------------------------------
                let did = <Identity<T>>::get_identity(&nominator.who).unwrap_or_default();
                Self::deposit_event(Event::<T>::Reward(did, nominator.who.clone(), imbalance.peek()));
                // -------------------------------------------------------------
                total_imbalance.subsume(imbalance);
            }
        }

        Ok(())
    }

    /// Update the ledger for a controller.
    ///
    /// This will also update the stash lock.
    pub(crate) fn update_ledger(controller: &T::AccountId, ledger: &StakingLedger<T>) {
        T::Currency::set_lock(
            STAKING_ID,
            &ledger.stash,
            ledger.total,
            WithdrawReasons::all(),
        );
        <Ledger<T>>::insert(controller, ledger);
    }

    /// Chill a stash account.
    pub(crate) fn chill_stash(stash: &T::AccountId) {
        // Polymesh Change: Decrement the running count by 1
        // -----------------------------------------------------------------
        Self::release_running_validator(stash);
        // -----------------------------------------------------------------
        Self::do_remove_validator(stash);
        Self::do_remove_nominator(stash);
    }

    /// Actually make a payment to a staker. This uses the currency's reward function
    /// to pay the right payee for the given staker account.
    fn make_payout(stash: &T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
        let dest = Self::payee(stash);
        match dest {
            RewardDestination::Controller => Self::bonded(stash)
                .map(|controller| T::Currency::deposit_creating(&controller, amount)),
            RewardDestination::Stash => T::Currency::deposit_into_existing(stash, amount).ok(),
            RewardDestination::Staked => Self::bonded(stash)
                .and_then(|c| Self::ledger(&c).map(|l| (c, l)))
                .and_then(|(controller, mut l)| {
                    l.active += amount;
                    l.total += amount;
                    let r = T::Currency::deposit_into_existing(stash, amount).ok();
                    Self::update_ledger(&controller, &l);
                    r
                }),
            RewardDestination::Account(dest_account) => {
                Some(T::Currency::deposit_creating(&dest_account, amount))
            }
            RewardDestination::None => None,
        }
    }

    /// Consume a set of [`BoundedSupports`] from [`sp_npos_elections`] and collect them into a
    /// [`Exposure`].
    pub(crate) fn collect_exposures(
        supports: SupportMap<T::AccountId>,
    ) -> Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)> {
        let total_issuance = T::Currency::total_issuance();
        let to_currency = |e: frame_election_provider_support::ExtendedBalance| {
            T::CurrencyToVote::to_currency(e, total_issuance)
        };

        supports
            .into_iter()
            .map(|(validator, support)| {
                // Build `struct exposure` from `support`.
                let mut others = Vec::with_capacity(support.voters.len());
                let mut own: BalanceOf<T> = Zero::zero();
                let mut total: BalanceOf<T> = Zero::zero();
                support
                    .voters
                    .into_iter()
                    .map(|(nominator, weight)| (nominator, to_currency(weight)))
                    .for_each(|(nominator, stake)| {
                        if nominator == validator {
                            own = own.saturating_add(stake);
                        } else {
                            others.push(IndividualExposure {
                                who: nominator,
                                value: stake,
                            });
                        }
                        total = total.saturating_add(stake);
                    });

                let exposure = Exposure { own, others, total };
                (validator, exposure)
            })
            .collect()
    }

    /// Remove all associated data of a stash account from the staking system.
    ///
    /// Assumes storage is upgraded before calling.
    ///
    /// This is called:
    /// - after a `withdraw_unbonded()` call that frees all of a stash's bonded balance.
    /// - through `reap_stash()` if the balance has fallen to zero (through slashing).
    pub(crate) fn kill_stash(stash: &T::AccountId, num_slashing_spans: u32) -> DispatchResult {
        let controller = <Bonded<T>>::get(stash).ok_or(Error::<T>::NotStash)?;

        slashing::clear_stash_metadata::<T>(stash, num_slashing_spans)?;

        <Bonded<T>>::remove(stash);
        <Ledger<T>>::remove(&controller);

        <Payee<T>>::remove(stash);
        Self::do_remove_validator(stash);
        Self::do_remove_nominator(stash);

        frame_system::Pallet::<T>::dec_consumers(stash);

        Ok(())
    }

//    /// Clear all era information for given era.
//    pub(crate) fn clear_era_information(era_index: EraIndex) {
//        #[allow(deprecated)]
//        <ErasStakers<T>>::remove_prefix(era_index, None);
//        #[allow(deprecated)]
//        <ErasStakersClipped<T>>::remove_prefix(era_index, None);
//        #[allow(deprecated)]
//        <ErasValidatorPrefs<T>>::remove_prefix(era_index, None);
//        <ErasValidatorReward<T>>::remove(era_index);
//        <ErasRewardPoints<T>>::remove(era_index);
//        <ErasTotalStake<T>>::remove(era_index);
//        ErasStartSessionIndex::<T>::remove(era_index);
//    }
//
//    /// Apply previously-unapplied slashes on the beginning of a new era, after a delay.
//    fn apply_unapplied_slashes(active_era: EraIndex) {
//        let era_slashes = <Self as Store>::UnappliedSlashes::take(&active_era);
//        log!(
//            debug,
//            "found {} slashes scheduled to be executed in era {:?}",
//            era_slashes.len(),
//            active_era,
//        );
//        for slash in era_slashes {
//            let slash_era = active_era.saturating_sub(T::SlashDeferDuration::get());
//            slashing::apply_slash::<T>(slash, slash_era);
//        }
//    }
//
//    /// Add reward points to validators using their stash account ID.
//    ///
//    /// Validators are keyed by stash account ID and must be in the current elected set.
//    ///
//    /// For each element in the iterator the given number of points in u32 is added to the
//    /// validator, thus duplicates are handled.
//    ///
//    /// At the end of the era each the total payout will be distributed among validator
//    /// relatively to their points.
//    ///
//    /// COMPLEXITY: Complexity is `number_of_validator_to_reward x current_elected_len`.
//    pub fn reward_by_ids(validators_points: impl IntoIterator<Item = (T::AccountId, u32)>) {
//        if let Some(active_era) = Self::active_era() {
//            <ErasRewardPoints<T>>::mutate(active_era.index, |era_rewards| {
//                for (validator, points) in validators_points.into_iter() {
//                    *era_rewards.individual.entry(validator).or_default() += points;
//                    era_rewards.total += points;
//                }
//            });
//        }
//    }

    /// Helper to set a new `ForceEra` mode.
    pub(crate) fn set_force_era(mode: Forcing) {
        log!(info, "Setting force era mode {:?}.", mode);
        ForceEra::<T>::put(mode);
    }

    /// Ensures that at the end of the current session there will be a new era.
    pub(crate) fn ensure_new_era() {
        match ForceEra::<T>::get() {
            Forcing::ForceAlways | Forcing::ForceNew => (),
            _ => Self::set_force_era(Forcing::ForceNew),
        }
    }

//    #[cfg(feature = "runtime-benchmarks")]
//    pub fn add_era_stakers(
//        current_era: EraIndex,
//        stash: T::AccountId,
//        exposure: Exposure<T::AccountId, BalanceOf<T>>,
//    ) {
//        <ErasStakers<T>>::insert(&current_era, &stash, &exposure);
//    }
//
//    #[cfg(feature = "runtime-benchmarks")]
//    pub fn set_slash_reward_fraction(fraction: Perbill) {
//        SlashRewardFraction::<T>::put(fraction);
//    }
//

    /// This function will add a nominator to the `Nominators` storage map,
    /// and `VoterList`.
    ///
    /// If the nominator already exists, their nominations will be updated.
    ///
    /// NOTE: you must ALWAYS use this function to add nominator or update their targets. Any access
    /// to `Nominators` or `VoterList` outside of this function is almost certainly
    /// wrong.
    pub fn do_add_nominator(who: &T::AccountId, nominations: Nominations<T>) {
        Nominators::<T>::insert(who, nominations);
    }

    /// This function will remove a nominator from the `Nominators` storage map,
    /// and `VoterList`.
    ///
    /// Returns true if `who` was removed from `Nominators`, otherwise false.
    ///
    /// NOTE: you must ALWAYS use this function to remove a nominator from the system. Any access to
    /// `Nominators` or `VoterList` outside of this function is almost certainly
    /// wrong.
    pub fn do_remove_nominator(who: &T::AccountId) -> bool {
        if Nominators::<T>::contains_key(who) {
            Nominators::<T>::remove(who);
            true
        } else {
            false
        }
    }

    /// This function will add a validator to the `Validators` storage map.
    ///
    /// If the validator already exists, their preferences will be updated.
    ///
    /// NOTE: you must ALWAYS use this function to add a validator to the system. Any access to
    /// `Validators` or `VoterList` outside of this function is almost certainly
    /// wrong.
    pub fn do_add_validator(who: &T::AccountId, prefs: ValidatorPrefs) {
        Validators::<T>::insert(who, prefs);
    }

    /// This function will remove a validator from the `Validators` storage map.
    ///
    /// Returns true if `who` was removed from `Validators`, otherwise false.
    ///
    /// NOTE: you must ALWAYS use this function to remove a validator from the system. Any access to
    /// `Validators` or `VoterList` outside of this function is almost certainly
    /// wrong.
    pub fn do_remove_validator(who: &T::AccountId) -> bool {
        if Validators::<T>::contains_key(who) {
            Validators::<T>::remove(who);
            true
        } else {
            false
        }
    }

//    /// Register some amount of weight directly with the system pallet.
//    ///
//    /// This is always mandatory weight.
//    fn register_weight(weight: Weight) {
//        <frame_system::Pallet<T>>::register_extra_weight_unchecked(
//            weight,
//            DispatchClass::Mandatory,
//        );
//    }

    // Polymesh Change: 
    // ----------------------------------------------------------------- 
    /// Returns the allowed validator count.
    pub(crate) fn get_allowed_validator_count() -> u32 {
        (T::MaxValidatorPerIdentity::get() * Self::validator_count()).max(1)
    }

    /// Decrease the running count of validators by 1 for the stash identity.
    pub(crate) fn release_running_validator(stash: &T::AccountId) {
        if !<Validators<T>>::contains_key(stash) {
            return;
        }

        if let Some(id) = <Identity<T>>::get_identity(stash) {
            PermissionedIdentity::<T>::mutate(&id, |pref| {
                if let Some(p) = pref {
                    if p.running_count > 0 {
                        p.running_count -= 1;
                        <Identity<T>>::remove_account_key_ref_count(&stash);
                    }
                }
            });
        }
    }

    /// Basic and cheap checks that we perform in validate unsigned, and in the execution.
    ///
    /// State reads: ElectionState, CurrentEr, QueuedScore.
    ///
    /// This function does weight refund in case of errors, which is based upon the fact that it is
    /// called at the very beginning of the call site's function.
    pub fn pre_dispatch_checks(score: ElectionScore, era: EraIndex) -> DispatchResultWithPostInfo {
        // discard solutions that are not in-time
        // check window open
        ensure!(
            Self::era_election_status().is_open(),
            Error::<T>::OffchainElectionEarlySubmission
                .with_weight(T::DbWeight::get().reads(1)),
        );

        // check current era.
        if let Some(current_era) = Self::current_era() {
            ensure!(
                current_era == era,
                Error::<T>::OffchainElectionEarlySubmission
                    .with_weight(T::DbWeight::get().reads(2)),
            )
        }

        // assume the given score is valid. Is it better than what we have on-chain, if we have any?
        if let Some(queued_score) = Self::queued_score() {
            ensure!(
                score.strict_threshold_better(queued_score, T::MinSolutionScoreBump::get()),
                Error::<T>::OffchainElectionWeakSubmission
                    .with_weight(T::DbWeight::get().reads(3)),
            )
        }

        Ok(None::<Weight>.into())
    }

    /// Checks a given solution and if correct and improved, writes it on chain as the queued result
    /// of the next round. This may be called by both a signed and an unsigned transaction.
    pub(crate) fn check_and_replace_solution(
        winners: Vec<ValidatorIndex>,
        compact_assignments: CompactAssignments,
        compute: ElectionCompute,
        claimed_score: ElectionScore,
        era: EraIndex,
        election_size: ElectionSize,
    ) -> DispatchResultWithPostInfo {
        // Do the basic checks. era, claimed score and window open.
        let _ = Self::pre_dispatch_checks(claimed_score, era)?;

        // before we read any further state, we check that the unique targets in compact is same as
        // compact. is a all in-memory check and easy to do. Moreover, it ensures that the solution
        // is not full of bogus edges that can cause lots of reads to SlashingSpans. Thus, we can
        // assume that the storage access of this function is always O(|winners|), not
        // O(|compact.edge_count()|).
        ensure!(
            compact_assignments.unique_targets().len() == winners.len(),
            Error::<T>::OffchainElectionBogusWinnerCount,
        );

        // Check that the number of presented winners is sane. Most often we have more candidates
        // than we need. Then it should be `Self::validator_count()`. Else it should be all the
        // candidates.
        let snapshot_validators_length = <SnapshotValidators<T>>::decode_len()
            .map(|l| l as u32)
            .ok_or_else(|| Error::<T>::SnapshotUnavailable)?;

        // size of the solution must be correct.
        ensure!(
            snapshot_validators_length == u32::from(election_size.validators),
            Error::<T>::OffchainElectionBogusElectionSize,
        );

        // check the winner length only here and when we know the length of the snapshot validators
        // length.
        let desired_winners = Self::validator_count().min(snapshot_validators_length);
        ensure!(
            winners.len() as u32 == desired_winners,
            Error::<T>::OffchainElectionBogusWinnerCount
        );

        let snapshot_nominators_len = <SnapshotNominators<T>>::decode_len()
            .map(|l| l as u32)
            .ok_or_else(|| Error::<T>::SnapshotUnavailable)?;

        // rest of the size of the solution must be correct.
        ensure!(
            snapshot_nominators_len == election_size.nominators,
            Error::<T>::OffchainElectionBogusElectionSize,
        );

        // decode snapshot validators.
        let snapshot_validators = Self::snapshot_validators();
        ensure!(
            !snapshot_validators.is_empty(),
            Error::<T>::SnapshotUnavailable
        );

        // check if all winners were legit; this is rather cheap. Replace with accountId.
        let winners = winners
            .into_iter()
            .map(|widx| {
                // NOTE: at the moment, since staking is explicitly blocking any offence until election
                // is closed, we don't check here if the account id at `snapshot_validators[widx]` is
                // actually a validator. If this ever changes, this loop needs to also check this.
                snapshot_validators
                    .get(widx as usize)
                    .cloned()
                    .ok_or(Error::<T>::OffchainElectionBogusWinner)
            })
            .collect::<Result<Vec<T::AccountId>, Error<T>>>()?;

        // decode the rest of the snapshot.
        let snapshot_nominators = Self::snapshot_nominators();
        ensure!(
            !snapshot_nominators.is_empty(),
            Error::<T>::SnapshotUnavailable
        );

        // helpers
        let nominator_at = |i: NominatorIndex| -> Option<T::AccountId> {
            snapshot_nominators.get(i as usize).cloned()
        };
        let validator_at = |i: ValidatorIndex| -> Option<T::AccountId> {
            snapshot_validators.get(i as usize).cloned()
        };

        // un-compact.
        let assignments = compact_assignments
            .into_assignment(nominator_at, validator_at)
            .map_err(|e| {
                // log the error since it is not propagated into the runtime error.
                log!(warn, "💸 un-compacting solution failed due to {:?}", e);
                Error::<T>::OffchainElectionBogusCompact
            })?;

        // check all nominators actually including the claimed vote. Also check correct self votes.
        // Note that we assume all validators and nominators in `assignments` are properly bonded,
        // because they are coming from the snapshot via a given index.
        for Assignment { who, distribution } in assignments.iter() {
            let is_validator = <Validators<T>>::contains_key(&who);
            let maybe_nomination = Self::nominators(&who);

            if !(maybe_nomination.is_some() ^ is_validator) {
                // all of the indices must map to either a validator or a nominator. If this is ever
                // not the case, then the locking system of staking is most likely faulty, or we
                // have bigger problems.
                log!(
                    error,
                    "💸 detected an error in the staking locking and snapshot."
                );
                // abort.
                return Err(Error::<T>::OffchainElectionBogusNominator.into());
            }

            if !is_validator {
                // a normal vote
                let nomination = maybe_nomination.expect(
                    "exactly one of `maybe_validator` and `maybe_nomination.is_some` is true. \
                    is_validator is false; maybe_nomination is some; qed",
                );

                // NOTE: we don't really have to check here if the sum of all edges are the
                // nominator correct. Un-compacting assures this by definition.

                for (t, _) in distribution {
                    // each target in the provided distribution must be actually nominated by the
                    // nominator after the last non-zero slash.
                    if nomination.targets.iter().find(|&tt| tt == t).is_none() {
                        return Err(Error::<T>::OffchainElectionBogusNomination.into());
                    }

                    if <Self as Store>::SlashingSpans::get(&t).map_or(false, |spans| {
                        nomination.submitted_in < spans.last_nonzero_slash()
                    }) {
                        return Err(Error::<T>::OffchainElectionSlashedNomination.into());
                    }
                }
            } else {
                // a self vote
                ensure!(
                    distribution.len() == 1,
                    Error::<T>::OffchainElectionBogusSelfVote
                );
                ensure!(
                    distribution[0].0 == *who,
                    Error::<T>::OffchainElectionBogusSelfVote
                );
                // defensive only. A compact assignment of length one does NOT encode the weight and
                // it is always created to be 100%.
                ensure!(
                    distribution[0].1 == OffchainAccuracy::one(),
                    Error::<T>::OffchainElectionBogusSelfVote,
                );
            }
        }

        // convert into staked assignments.
        let staked_assignments = sp_npos_elections::assignment_ratio_to_staked(
            assignments,
            Self::weight_of_fn(),
        );

        // build the support map thereof in order to evaluate.
        let supports_map = to_support_map::<T::AccountId>(&staked_assignments);
        let supports = supports_map
            .clone()
            .into_iter()
            .collect::<Supports<T::AccountId>>();

        // Check if the score is the same as the claimed one.
        let submitted_score = (&supports).evaluate();
        ensure!(
            submitted_score == claimed_score,
            Error::<T>::OffchainElectionBogusScore
        );

        // At last, alles Ok. Exposures and store the result.
        let exposures = Self::collect_exposures(supports_map);
        log!(
			info,
			"💸 A better solution (with compute {:?} and score {:?}) has been validated and stored on chain.",
			compute,
			submitted_score,
		);

        // write new results.
        <QueuedElected<T>>::put(Some(ElectionResult {
            elected_stashes: winners,
            compute,
            exposures,
        }));
        QueuedScore::<T>::put(Some(submitted_score));

        // emit event.
        Self::deposit_event(Event::<T>::SolutionStored(compute));

        Ok(None::<Weight>.into())
    }

    pub(crate) fn unbond_balance(
        controller: T::AccountId,
        ledger: &mut StakingLedger<T>,
        value: BalanceOf<T>,
    ) -> DispatchResult {
        let mut value = value.min(ledger.active);

        if !value.is_zero() {
            ledger.active -= value;

            // Avoid there being a dust balance left in the staking system.
            if ledger.active < T::Currency::minimum_balance() {
                value += ledger.active;
                ledger.active = Zero::zero();
            }

            // Note: in case there is no current era it is fine to bond one era more.
            let era = Self::current_era().unwrap_or(0) + T::BondingDuration::get();
            if let Some(mut chunk) =
                ledger.unlocking.last_mut().filter(|chunk| chunk.era == era)
            {
                // To keep the chunk count down, we only keep one chunk per era. Since
                // `unlocking` is a FiFo queue, if a chunk exists for `era` we know that it will
                // be the last one.
                chunk.value = chunk.value.defensive_saturating_add(value)
            } else {
                ledger
                    .unlocking
                    .try_push(UnlockChunk { value, era })
                    .map_err(|_| Error::<T>::NoMoreChunks)?;
            };
            // NOTE: ledger must be updated prior to calling `Self::weight_of`.
            Self::update_ledger(&controller, &ledger);

            let did = Context::current_identity::<T::IdentityFn>().unwrap_or_default();
            Self::deposit_event(Event::<T>::Unbonded(did, ledger.stash.clone(), value));
        }
        Ok(())
    }

    pub(crate) fn get_bonding_duration_period() -> u64 {
        (T::SessionsPerEra::get()  * T::BondingDuration::get()) as u64 // total session
            * T::EpochDuration::get() // session length
            * T::ExpectedBlockTime::get().saturated_into::<u64>()
    }

    pub(crate) fn base_chill_from_governance(
        origin: T::RuntimeOrigin, 
        identity: IdentityId, 
        stash_keys: Vec<T::AccountId>
    ) -> DispatchResult {
        // Checks that the era election status is closed.
        ensure!(
            Self::era_election_status().is_closed(), 
            Error::<T>::CallNotAllowed
        );
        // Required origin for removing a validator.
        T::RequiredRemoveOrigin::ensure_origin(origin)?;
        // Checks that the identity is allowed to run operator/validator nodes.
        ensure!(
            Self::permissioned_identity(&identity).is_some(), 
            Error::<T>::NotExists
        );

        for key in &stash_keys {
            let key_did = Identity::<T>::get_identity(&key);
            // Checks if the stash key identity is the same as the identity given.
            ensure!(key_did == Some(identity), Error::<T>::NotStash);   
            // Checks if the key is a validator if not returns an error.
            ensure!(<Validators<T>>::contains_key(&key), Error::<T>::NotExists); 
        }

        for key in stash_keys {
            Self::chill_stash(&key);
        }
       
        // Change identity status to be Non-Permissioned
        PermissionedIdentity::<T>::remove(&identity);
        Ok(())
    }

    // -------------------------------------------------------------------------
}
