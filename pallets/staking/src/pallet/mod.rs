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

//! Staking FRAME Pallet.

use frame_support::{
    dispatch::Codec,
    pallet_prelude::*,
    traits::{
        Currency, CurrencyToVote, EnsureOrigin, EstimateNextNewSession, Get, LockableCurrency, 
        OnUnbalanced, UnixTime,
    },
    weights::Weight,
    BoundedVec,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use sp_runtime::{
    traits::{CheckedSub, SaturatedConversion, StaticLookup, Zero},
    ArithmeticError, Perbill, Percent,
};
use sp_staking::{EraIndex, SessionIndex};
use sp_std::prelude::*;

mod impls;

use crate::{
    slashing, weights::WeightInfo, AccountIdLookupOf, ActiveEraInfo, BalanceOf, EraRewardPoints, 
    Exposure, Forcing, NegativeImbalanceOf, Nominations, PositiveImbalanceOf,RewardDestination, 
    SessionInterface, StakingLedger, UnappliedSlash, ValidatorPrefs,
};

use frame_support::traits::IsSubType;
use frame_support::traits::schedule::Anon;
use frame_support::weights::constants::{WEIGHT_REF_TIME_PER_MICROS, WEIGHT_REF_TIME_PER_NANOS};
use frame_system::offchain::SendTransactionTypes;
use sp_npos_elections::{ElectionScore};
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::traits::{Dispatchable, Saturating};
use sp_runtime::Permill;

use polymesh_common_utilities::{GC_DID, Context};
use polymesh_common_utilities::identity::Config as IdentityConfig;
use polymesh_primitives::IdentityId;

use crate::types::{
    ElectionStatus, ElectionResult, ElectionCompute, PermissionedIdentityPrefs, SlashingSwitch, 
    ElectionSize
};
use crate::{ValidatorIndex, CompactAssignments, STAKING_ID};

type Identity<T> = pallet_identity::Module<T>;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(13);

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Possible operations on the configuration values of this pallet.
    #[derive(TypeInfo, Debug, Clone, Encode, Decode, PartialEq)]
    pub enum ConfigOp<T: Default + Codec> {
        /// Don't change.
        Noop,
        /// Set the given value.
        Set(T),
        /// Remove from storage.
        Remove,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> + pallet_babe::Config + IdentityConfig {
        /// The staking balance.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

        /// Time used for computing era duration.
        ///
        /// It is guaranteed to start being called from the first `on_finalize`. Thus value at
        /// genesis is not used.
        type UnixTime: UnixTime;

        /// Convert a balance into a number used for election calculation. This must fit into a
        /// `u64` but is allowed to be sensibly lossy. The `u64` is used to communicate with the
        /// [`frame_election_provider_support`] crate which accepts u64 numbers and does operations
        /// in 128.
        /// Consequently, the backward convert is used convert the u128s from sp-elections back to a
        /// [`BalanceOf`].
        type CurrencyToVote: CurrencyToVote<BalanceOf<Self>>;

        /// Maximum number of nominations per nominator.
        #[pallet::constant]
        type MaxNominations: Get<u32>;

        /// Tokens have been minted and are unused for validator-reward.
        /// See [Era payout](./index.html#era-payout).
        type RewardRemainder: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Handler for the unbalanced reduction when slashing a staker.
        type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// Handler for the unbalanced increment when rewarding a staker.
        /// NOTE: in most cases, the implementation of `OnUnbalanced` should modify the total
        /// issuance.
        type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;

        /// Number of sessions per era.
        type SessionsPerEra: Get<SessionIndex>;

        /// Number of eras that staked funds must remain bonded for.]
        type BondingDuration: Get<EraIndex>;

        /// Number of eras that slashes are deferred by, after computation.
        ///
        /// This should be less than the bonding duration. Set to 0 if slashes
        /// should be applied immediately, without opportunity for intervention.
        type SlashDeferDuration: Get<EraIndex>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The maximum number of `unlocking` chunks a [`StakingLedger`] can
        /// have. Effectively determines how many unique eras a staker may be
        /// unbonding in.
        ///
        /// Note: `MaxUnlockingChunks` is used as the upper bound for the
        /// `BoundedVec` item `StakingLedger.unlocking`. Setting this value
        /// lower than the existing value can lead to inconsistencies in the
        /// `StakingLedger` and will need to be handled properly in a runtime
        /// migration. The test `reducing_max_unlocking_chunks_abrupt` shows
        /// this effect.
        #[pallet::constant]
        type MaxUnlockingChunks: Get<u32>;

        // Polymesh Change: Have fixed rewards kicked in?
        // -----------------------------------------------------------------
        /// The origin which can cancel a deferred slash. Root can always do this.
        type SlashCancelOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Interface for interacting with a session module.
        type SessionInterface: self::SessionInterface<Self::AccountId>;

        /// The NPoS reward curve used to define yearly inflation.
        /// See [Era payout](./index.html#era-payout).
        type RewardCurve: Get<&'static PiecewiseLinear<'static>>;

        /// Something that can estimate the next session change, accurately or as a best effort guess.
        type NextNewSession: EstimateNextNewSession<Self::BlockNumber>;

        /// The number of blocks before the end of the era from which election submissions are allowed.
        ///
        /// Setting this to zero will disable the offchain compute and only on-chain seq-phragmen will
        /// be used.
        ///
        /// This is bounded by being within the last session. Hence, setting it to a value more than the
        /// length of a session will be pointless.
        type ElectionLookahead: Get<Self::BlockNumber>;

        /// The overarching call type.
        type Call: Dispatchable + From<Call<Self>> + IsSubType<Call<Self>> + Clone;

        /// Maximum number of balancing iterations to run in the offchain submission.
        ///
        /// If set to 0, balance_solution will not be executed at all.
        type MaxIterations: Get<u32>;

        /// The threshold of improvement that should be provided for a new solution to be accepted.
        type MinSolutionScoreBump: Get<Perbill>;

        /// The maximum number of nominators rewarded for each validator.
        ///
        /// For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can claim
        /// their reward. This used to limit the i/o cost for the nominator payout.
        type MaxNominatorRewardedPerValidator: Get<u32>;

        /// The fraction of the validator set that is safe to be offending.
        /// After the threshold is reached a new era will be forced.
        type OffendingValidatorsThreshold: Get<Perbill>;

        /// A configuration for base priority of unsigned transactions.
        ///
        /// This is exposed so that it can be tuned for particular runtime, when
        /// multiple pallets send unsigned transactions.
        type UnsignedPriority: Get<TransactionPriority>;

        /// Maximum weight that the unsigned transaction can have.
        ///
        /// Chose this value with care. On one hand, it should be as high as possible, so the solution
        /// can contain as many nominators/validators as possible. On the other hand, it should be small
        /// enough to fit in the block.
        type OffchainSolutionWeightLimit: Get<Weight>;

        /// Required origin for adding a potential validator (can always be Root).
        type RequiredAddOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Required origin for removing a validator (can always be Root).
        type RequiredRemoveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Required origin for changing validator commission.
        type RequiredCommissionOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// To schedule the rewards for the stakers after the end of era.
        type RewardScheduler: Anon<Self::BlockNumber, <Self as Config>::Call, Self::PalletsOrigin>;

        /// Overarching type of all pallets origins.
        type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

        /// Maximum amount of validators that can run by an identity.
        /// It will be MaxValidatorPerIdentity * Self::validator_count().
        type MaxValidatorPerIdentity: Get<Permill>;

        /// Maximum amount of total issuance after which fixed rewards kicks in.
        type MaxVariableInflationTotalIssuance: Get<BalanceOf<Self>>;

        /// Yearly total reward amount that gets distributed when fixed rewards kicks in.
        type FixedYearlyReward: Get<BalanceOf<Self>>;

        /// Minimum bond amount.
        type MinimumBond: Get<BalanceOf<Self>>;
        // -----------------------------------------------------------------
    }

    #[pallet::type_value]
    pub fn HistoryDepthDefault() -> u32 { 84 }

    /// Number of eras to keep in history.
    ///
    /// Information is kept for eras in `[current_era - history_depth; current_era]`.
    ///
    /// Must be more than the number of eras delayed by session otherwise. I.e. active era must
    /// always be in history. I.e. `active_era > current_era - history_depth` must be
    /// guaranteed.
    #[pallet::storage]
    #[pallet::getter(fn history_depth)]
    pub type HistoryDepth<T> = StorageValue<_, u32, ValueQuery, HistoryDepthDefault>;

    /// The ideal number of active validators.
    #[pallet::storage]
    #[pallet::getter(fn validator_count)]
    pub type ValidatorCount<T> = StorageValue<_, u32, ValueQuery>;

    /// Minimum number of staking participants before emergency conditions are imposed.
    #[pallet::storage]
    #[pallet::getter(fn minimum_validator_count)]
    pub type MinimumValidatorCount<T> = StorageValue<_, u32, ValueQuery>;

    /// Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
    /// easy to initialize and the performance hit is minimal (we expect no more than four
    /// invulnerables) and restricted to testnets.
    #[pallet::storage]
    #[pallet::getter(fn invulnerables)]
    #[pallet::unbounded]
    pub type Invulnerables<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    /// Map from all locked "stash" accounts to the controller account.
    ///
    /// TWOX-NOTE: SAFE since `AccountId` is a secure hash.
    #[pallet::storage]
    #[pallet::getter(fn bonded)]
    pub type Bonded<T: Config> = 
        StorageMap<_, Twox64Concat, T::AccountId, T::AccountId, OptionQuery>;

    /// Map from all (unlocked) "controller" accounts to the info regarding the staking.
    #[pallet::storage]
    #[pallet::getter(fn ledger)]
    #[pallet::unbounded]
    pub type Ledger<T: Config> = 
        StorageMap<_, Blake2_128Concat, T::AccountId, StakingLedger<T>, OptionQuery>;

    /// Where the reward payment should be made. Keyed by stash.
    ///
    /// TWOX-NOTE: SAFE since `AccountId` is a secure hash.
    #[pallet::storage]
    #[pallet::getter(fn payee)]
    pub type Payee<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, RewardDestination<T::AccountId>, ValueQuery>;

    /// The map from (wannabe) validator stash key to the preferences of that validator.
    ///
    /// TWOX-NOTE: SAFE since `AccountId` is a secure hash.
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> =
        CountedStorageMap<_, Twox64Concat, T::AccountId, ValidatorPrefs, ValueQuery>;

    /// The map from nominator stash key to their nomination preferences, namely the validators that
    /// they wish to support.
    ///
    /// Note that the keys of this storage map might become non-decodable in case the
    /// [`Config::MaxNominations`] configuration is decreased. In this rare case, these nominators
    /// are still existent in storage, their key is correct and retrievable (i.e. `contains_key`
    /// indicates that they exist), but their value cannot be decoded. Therefore, the non-decodable
    /// nominators will effectively not-exist, until they re-submit their preferences such that it
    /// is within the bounds of the newly set `Config::MaxNominations`.
    ///
    /// This implies that `::iter_keys().count()` and `::iter().count()` might return different
    /// values for this map. Moreover, the main `::count()` is aligned with the former, namely the
    /// number of keys that exist.
    ///
    /// Lastly, if any of the nominators become non-decodable, they can be chilled immediately via
    /// [`Call::chill_other`] dispatchable by anyone.
    ///
    /// TWOX-NOTE: SAFE since `AccountId` is a secure hash.
    #[pallet::storage]
    #[pallet::getter(fn nominators)]
    pub type Nominators<T: Config> =
        CountedStorageMap<_, Twox64Concat, T::AccountId, Nominations<T>, OptionQuery>;

    /// The current era index.
    ///
    /// This is the latest planned era, depending on how the Session pallet queues the validator
    /// set, it might be active or not.
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, EraIndex, OptionQuery>;

    /// The active era information, it holds index and start.
    ///
    /// The active era is the era being currently rewarded. Validator set of this era must be
    /// equal to [`SessionInterface::validators`].
    #[pallet::storage]
    #[pallet::getter(fn active_era)]
    pub type ActiveEra<T> = StorageValue<_, ActiveEraInfo, OptionQuery>;

    /// The session index at which the era start for the last `HISTORY_DEPTH` eras.
    ///
    /// Note: This tracks the starting session (i.e. session index when era start being active)
    /// for the eras in `[CurrentEra - HISTORY_DEPTH, CurrentEra]`.
    #[pallet::storage]
    #[pallet::getter(fn eras_start_session_index)]
    pub type ErasStartSessionIndex<T> = StorageMap<_, Twox64Concat, EraIndex, SessionIndex, OptionQuery>;

    /// Exposure of validator at era.
    ///
    /// This is keyed first by the era index to allow bulk deletion and then the stash account.
    ///
    /// Is it removed after `HISTORY_DEPTH` eras.
    /// If stakers hasn't been set or has been removed then empty exposure is returned.
    #[pallet::storage]
    #[pallet::getter(fn eras_stakers)]
    #[pallet::unbounded]
    pub type ErasStakers<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        EraIndex,
        Twox64Concat,
        T::AccountId,
        Exposure<T::AccountId, BalanceOf<T>>,
        ValueQuery,
    >;

    /// Clipped Exposure of validator at era.
    ///
    /// This is similar to [`ErasStakers`] but number of nominators exposed is reduced to the
    /// `T::MaxNominatorRewardedPerValidator` biggest stakers.
    /// (Note: the field `total` and `own` of the exposure remains unchanged).
    /// This is used to limit the i/o cost for the nominator payout.
    ///
    /// This is keyed fist by the era index to allow bulk deletion and then the stash account.
    ///
    /// Is it removed after `HISTORY_DEPTH` eras.
    /// If stakers hasn't been set or has been removed then empty exposure is returned.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn eras_stakers_clipped)]
    pub type ErasStakersClipped<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        EraIndex,
        Twox64Concat,
        T::AccountId,
        Exposure<T::AccountId, BalanceOf<T>>,
        ValueQuery,
    >;

    /// Similar to `ErasStakers`, this holds the preferences of validators.
    ///
    /// This is keyed first by the era index to allow bulk deletion and then the stash account.
    ///
    /// Is it removed after `HISTORY_DEPTH` eras.
    // If prefs hasn't been set or has been removed then 0 commission is returned.
    #[pallet::storage]
    #[pallet::getter(fn eras_validator_prefs)]
    pub type ErasValidatorPrefs<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        EraIndex,
        Twox64Concat,
        T::AccountId,
        ValidatorPrefs,
        ValueQuery,
    >;

    /// The total validator era payout for the last `HISTORY_DEPTH` eras.
    ///
    /// Eras that haven't finished yet or has been removed doesn't have reward.
    #[pallet::storage]
    #[pallet::getter(fn eras_validator_reward)]
    pub type ErasValidatorReward<T: Config> = StorageMap<_, Twox64Concat, EraIndex, BalanceOf<T>, OptionQuery>;

    /// Rewards for the last `HISTORY_DEPTH` eras.
    /// If reward hasn't been set or has been removed then 0 reward is returned.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn eras_reward_points)]
    pub type ErasRewardPoints<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, EraRewardPoints<T::AccountId>, ValueQuery>;

    /// The total amount staked for the last `HISTORY_DEPTH` eras.
    /// If total hasn't been set or has been removed then 0 stake is returned.
    #[pallet::storage]
    #[pallet::getter(fn eras_total_stake)]
    pub type ErasTotalStake<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, BalanceOf<T>, ValueQuery>;

    /// Mode of era forcing.
    #[pallet::storage]
    #[pallet::getter(fn force_era)]
    pub type ForceEra<T> = StorageValue<_, Forcing, ValueQuery>;

    /// The percentage of the slash that is distributed to reporters.
    ///
    /// The rest of the slashed value is handled by the `Slash`.
    #[pallet::storage]
    #[pallet::getter(fn slash_reward_fraction)]
    pub type SlashRewardFraction<T> = StorageValue<_, Perbill, ValueQuery>;

    /// The amount of currency given to reporters of a slash event which was
    /// canceled by extraordinary circumstances (e.g. governance).
    #[pallet::storage]
    #[pallet::getter(fn canceled_payout)]
    pub type CanceledSlashPayout<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// All unapplied slashes that are queued for later.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type UnappliedSlashes<T: Config> = StorageMap<
        _,
        Twox64Concat,
        EraIndex,
        Vec<UnappliedSlash<T::AccountId, BalanceOf<T>>>,
        ValueQuery,
    >;

    /// A mapping from still-bonded eras to the first session index of that era.
    ///
    /// Must contains information for eras for the range:
    /// `[active_era - bounding_duration; active_era]`
    #[pallet::storage]
    #[pallet::unbounded]
    pub type BondedEras<T: Config> =
        StorageValue<_, Vec<(EraIndex, SessionIndex)>, ValueQuery>;

    /// All slashing events on validators, mapped by era to the highest slash proportion
    /// and slash value of the era.
    #[pallet::storage]
    pub type ValidatorSlashInEra<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        EraIndex,
        Twox64Concat,
        T::AccountId,
        (Perbill, BalanceOf<T>),
        OptionQuery,
    >;

    /// All slashing events on nominators, mapped by era to the highest slash value of the era.
    #[pallet::storage]
    pub type NominatorSlashInEra<T: Config> = StorageDoubleMap<
        _, 
        Twox64Concat, 
        EraIndex, 
        Twox64Concat, 
        T::AccountId, 
        BalanceOf<T>, 
        OptionQuery,
    >;

    /// Slashing spans for stash accounts.
    #[pallet::storage]
    #[pallet::getter(fn slashing_spans)]
    #[pallet::unbounded]
    pub type SlashingSpans<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, slashing::SlashingSpans, OptionQuery>;

    /// Records information about the maximum slash of a stash within a slashing span,
    /// as well as how much reward has been paid out.
    #[pallet::storage]
    pub type SpanSlash<T: Config> = StorageMap<
        _,
        Twox64Concat,
        (T::AccountId, slashing::SpanIndex),
        slashing::SpanRecord<BalanceOf<T>>,
        ValueQuery,
    >;

    /// Indices of validators that have offended in the active era and whether they are currently
    /// disabled.
    ///
    /// This value should be a superset of disabled validators since not all offences lead to the
    /// validator being disabled (if there was no slash). This is needed to track the percentage of
    /// validators that have offended in the current era, ensuring a new era is forced if
    /// `OffendingValidatorsThreshold` is reached. The vec is always kept sorted so that we can find
    /// whether a given validator has previously offended using binary search. It gets cleared when
    /// the era ends.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn offending_validators)]
    pub type OffendingValidators<T: Config> = StorageValue<_, Vec<(u32, bool)>, ValueQuery>;

    // Polymesh Change: --------------------------------------------
    
    #[pallet::storage]
    /// The earliest era for which we have a pending, unapplied slash.
    pub(crate) type EarliestUnappliedSlash<T: Config> = StorageValue<_, EraIndex, OptionQuery>;

    /// Snapshot of validators at the beginning of the current election window. This should only
    /// have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn snapshot_validators)]
    pub(crate) type SnapshotValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, OptionQuery>;
    
    /// Snapshot of nominators at the beginning of the current election window. This should only
    /// have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn snapshot_nominators)]
    pub type SnapshotNominators<T: Config> = StorageValue<_, Vec<T::AccountId>, OptionQuery>;

    /// The next validator set. At the end of an era, if this is available (potentially from the
    /// result of an offchain worker), it is immediately used. Otherwise, the on-chain election
    /// is executed.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn queued_elected)]
    pub type QueuedElected<T: Config> = 
        StorageValue<_, ElectionResult<T::AccountId, BalanceOf<T>>, OptionQuery>;

    /// The score of the current [`QueuedElected`].
    #[pallet::storage]
    #[pallet::getter(fn queued_score)]
    pub type QueuedScore<T: Config> = StorageValue<_, ElectionScore, OptionQuery>;

    /// Flag to control the execution of the offchain election. When `Open(_)`, we accept solutions
    /// to be submitted.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn era_election_status)]
    pub type EraElectionStatus<T: Config> = 
        StorageValue<_, ElectionStatus<T::BlockNumber>, ValueQuery>;

    /// True if the current **planned** session is final. Note that this does not take era
    /// forcing into account.
    #[pallet::storage]
    #[pallet::getter(fn is_current_session_final)]
    pub type IsCurrentSessionFinal<T: Config> = StorageValue<_, bool, ValueQuery>; 

    /// Entities that are allowed to run operator/validator nodes.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn permissioned_identity)]
    pub type PermissionedIdentity<T: Config> =
        StorageMap<_, Twox64Concat, IdentityId, PermissionedIdentityPrefs, OptionQuery>;

    /// Allows flexibility in commission. Every validator has commission that should be in the 
    /// range [0, Cap].
    #[pallet::storage]
    #[pallet::getter(fn validator_commission_cap)]
    pub type ValidatorCommissionCap<T: Config> = StorageValue<_, Perbill, ValueQuery>;

    /// The minimum amount with which a validator can bond.
    #[pallet::storage]
    #[pallet::getter(fn min_bond_threshold)]
    pub type MinimumBondThreshold<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // Slashing switch for validators & Nominators.
    #[pallet::storage]
    #[pallet::getter(fn slashing_allowed_for)]
    pub type SlashingAllowedFor<T: Config> = StorageValue<_, SlashingSwitch, ValueQuery>;

    // -------------------------------------------------------------

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub validator_count: u32,
        pub minimum_validator_count: u32,
        pub invulnerables: Vec<T::AccountId>,
        pub force_era: Forcing,
        pub slash_reward_fraction: Perbill,
        pub canceled_payout: BalanceOf<T>,
        pub stakers: Vec<(
            IdentityId,
            T::AccountId,
            T::AccountId,
            BalanceOf<T>,
            crate::StakerStatus<T::AccountId>,
        )>,
        pub validator_commission_cap: Perbill,
        pub min_bond_threshold: BalanceOf<T>,
        pub slashing_allowed_for: SlashingSwitch,
        pub history_depth: u32,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                validator_count: Default::default(),
                minimum_validator_count: Default::default(),
                invulnerables: Default::default(),
                force_era: Default::default(),
                slash_reward_fraction: Default::default(),
                canceled_payout: Default::default(),
                stakers: Default::default(),
                validator_commission_cap: Default::default(),
                min_bond_threshold: Default::default(),
                slashing_allowed_for: Default::default(),
                history_depth: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            ValidatorCount::<T>::put(self.validator_count);
            MinimumValidatorCount::<T>::put(self.minimum_validator_count);
            Invulnerables::<T>::put(&self.invulnerables);
            ForceEra::<T>::put(self.force_era);
            SlashRewardFraction::<T>::put(self.slash_reward_fraction);
            CanceledSlashPayout::<T>::put(self.canceled_payout);
            ValidatorCommissionCap::<T>::put(self.validator_commission_cap);
            MinimumBondThreshold::<T>::put(self.min_bond_threshold);
            SlashingAllowedFor::<T>::put(self.slashing_allowed_for);
            //HistoryDepth::<T>::put(self.history_depth);

            for &(did, ref stash, ref controller, balance, ref status) in &self.stakers {
                crate::log!(
                    trace,
                    "inserting genesis staker: {:?} => {:?} => {:?}",
                    stash,
                    balance,
                    status
                );
                assert!(
                    T::Currency::free_balance(stash) >= balance,
                    "Stash does not have enough balance to bond."
                );
                frame_support::assert_ok!(<Pallet<T>>::bond(
                    T::RuntimeOrigin::from(Some(stash.clone()).into()),
                    T::Lookup::unlookup(controller.clone()),
                    balance,
                    RewardDestination::Staked,
                ));
                match status {
                    crate::StakerStatus::Validator => {
                        if <Pallet<T>>::permissioned_identity(&did).is_none() {
                            // Adding identity directly in the storage by assuming it is CDD'ed
                            PermissionedIdentity::<T>::insert(
                                &did, 
                                PermissionedIdentityPrefs::new(3)
                            );
                            <Pallet<T>>::deposit_event(Event::<T>::PermissionedIdentityAdded(
                                GC_DID, 
                                did
                            ));
                        }
                        let _ = <Pallet<T>>::validate(
                            T::RuntimeOrigin::from(Some(controller.clone()).into()),
                            ValidatorPrefs {
                                commission: self.validator_commission_cap,
                                blocked: Default::default(),
                            },
                        );
                    },
                    crate::StakerStatus::Nominator(votes) => {
                        let _ = <Pallet<T>>::nominate(
                            T::RuntimeOrigin::from(Some(controller.clone()).into()),
                            votes
                                .iter()
                                .map(|l| T::Lookup::unlookup(l.clone()))
                                .collect(),
                            );
                    },
                    _ => {},
                };
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The era payout has been set; the first balance is the validator-payout; the second is
        /// the remainder from the maximum amount of reward.
        EraPayout(EraIndex, BalanceOf<T>, BalanceOf<T>),
        /// The nominator has been rewarded by this amount.
        Reward(IdentityId, T::AccountId, BalanceOf<T>),
        /// A staker (validator or nominator) has been slashed by the given amount.
        Slash(T::AccountId, BalanceOf<T>),
        /// An old slashing report from a prior era was discarded because it could
        /// not be processed.
        OldSlashingReportDiscarded(SessionIndex),
        /// A new set of stakers was elected.
        StakingElection(ElectionCompute),
        /// A new solution for the upcoming election has been stored.
        SolutionStored(ElectionCompute),
        /// An account has bonded this amount. \[stash, amount\]
        ///
        /// NOTE: This event is only emitted when funds are bonded via a dispatchable. Notably,
        /// it will not be emitted for staking rewards when they are added to stake.
        Bonded(IdentityId, T::AccountId, BalanceOf<T>),
        /// An account has unbonded this amount.
        Unbonded(IdentityId, T::AccountId, BalanceOf<T>),
        /// User has updated their nominations
        Nominated(IdentityId, T::AccountId, Vec<T::AccountId>),
        /// An account has called `withdraw_unbonded` and removed unbonding chunks worth `Balance`
        /// from the unlocking queue.
        Withdrawn(T::AccountId, BalanceOf<T>),
        /// An DID has issued a candidacy. See the transaction for who.
        /// GC identity , Validator's identity.
        PermissionedIdentityAdded(IdentityId, IdentityId),
        /// The given member was removed. See the transaction for who.
        /// GC identity , Validator's identity.
        PermissionedIdentityRemoved(IdentityId, IdentityId),
        /// Remove the nominators from the valid nominators when there CDD expired.
        /// Caller, Stash accountId of nominators
        InvalidatedNominators(IdentityId, T::AccountId, Vec<T::AccountId>),
        /// When commission cap get updated.
        /// (old value, new value)
        CommissionCapUpdated(IdentityId, Perbill, Perbill),
        /// Min bond threshold was updated (new value).
        MinimumBondThresholdUpdated(Option<IdentityId>, BalanceOf<T>),
        /// When scheduling of reward payments get interrupted.
        RewardPaymentSchedulingInterrupted(T::AccountId, EraIndex, DispatchError),
        /// Update for whom balance get slashed.
        SlashingAllowedForChanged(SlashingSwitch),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Not a controller account.
        NotController,
        /// Not a stash account.
        NotStash,
        /// Stash is already bonded.
        AlreadyBonded,
        /// Controller is already paired.
        AlreadyPaired,
        /// Targets cannot be empty.
        EmptyTargets,
        /// Slash record index out of bounds.
        InvalidSlashIndex,
        /// Can not bond with value less than minimum balance.
        InsufficientValue,
        /// Can not schedule more unlock chunks.
        NoMoreChunks,
        /// Can not rebond without unlocking chunks.
        NoUnlockChunk,
        /// Attempting to target a stash that still has funds.
        FundedTarget,
        /// Invalid era to reward.
        InvalidEraToReward,
        /// Items are not sorted and unique.
        NotSortedAndUnique,
        /// Rewards for this era have already been claimed for this validator.
        AlreadyClaimed,
        /// The submitted result is received out of the open window.
        OffchainElectionEarlySubmission,
        /// The submitted result is not as good as the one stored on chain.
        OffchainElectionWeakSubmission,
        /// The snapshot data of the current window is missing.
        SnapshotUnavailable,
        /// Incorrect number of winners were presented.
        OffchainElectionBogusWinnerCount,
        /// One of the submitted winners is not an active candidate on chain (index is out of range
        /// in snapshot).
        OffchainElectionBogusWinner,
        /// Error while building the assignment type from the compact. This can happen if an index
        /// is invalid, or if the weights _overflow_.
        OffchainElectionBogusCompact,
        /// One of the submitted nominators is not an active nominator on chain.
        OffchainElectionBogusNominator,
        /// One of the submitted nominators has an edge to which they have not voted on chain.
        OffchainElectionBogusNomination,
        /// One of the submitted nominators has an edge which is submitted before the last non-zero
        /// slash of the target.
        OffchainElectionSlashedNomination,
        /// A self vote must only be originated from a validator to ONLY themselves.
        OffchainElectionBogusSelfVote,
        /// The submitted result has unknown edges that are not among the presented winners.
        OffchainElectionBogusEdge,
        /// The claimed score does not match with the one computed from the data.
        OffchainElectionBogusScore,
        /// The election size is invalid.
        OffchainElectionBogusElectionSize,
        /// The call is not allowed at the given time due to restrictions of election period.
        CallNotAllowed,
        /// Incorrect number of slashing spans provided.
        IncorrectSlashingSpans,
        /// Permissioned validator already exists.
        AlreadyExists,
        /// Permissioned validator not exists.
        NotExists,
        /// Updates with same value.
        NoChange,
        /// Given potential validator identity is invalid.
        InvalidValidatorIdentity,
        /// Validator prefs are not in valid range.
        InvalidValidatorCommission,
        /// Validator or nominator stash identity does not exist.
        StashIdentityDoesNotExist,
        /// Validator stash identity was not permissioned.
        StashIdentityNotPermissioned,
        /// Nominator stash was not CDDed.
        StashIdentityNotCDDed,
        /// Running validator count hit the intended count.
        HitIntendedValidatorCount,
        /// When the intended number of validators to run is >= 2/3 of `validator_count`.
        IntendedCountIsExceedingConsensusLimit,
        /// When the amount to be bonded is less than `MinimumBond`
        BondTooSmall,
        /// Internal state has become somehow corrupted and the operation cannot continue.
        BadState,
		/// Too many nomination targets supplied.
		TooManyTargets,
        /// A nomination target was supplied that was blocked or otherwise not a validator.
        BadTarget,
        /// Validator should have minimum 50k POLYX bonded.
        InvalidValidatorUnbondAmount,
        /// Some bound is not met.
        BoundNotMet,
        /// There are too many nominators in the system. Governance needs to adjust the staking
        /// settings to keep things safe for the runtime.
        TooManyNominators,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// sets `ElectionStatus` to `Open(now)` where `now` is the block number at which the
        /// election window has opened, if we are at the last session and less blocks than
        /// `T::ElectionLookahead` is remaining until the next new session schedule. The offchain
        /// worker, if applicable, will execute at the end of the current block, and solutions may
        /// be submitted.
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let mut consumed_weight = Weight::zero();
            
            let mut add_weight = |reads: u64, writes: u64, weight| {
                consumed_weight += T::DbWeight::get().reads_writes(reads, writes);
                consumed_weight += weight;
            };

            if
            // if we don't have any ongoing offchain compute.
            Self::era_election_status().is_closed() &&
                // either current session final based on the plan, or we're forcing.
                (Self::is_current_session_final() || Self::will_era_be_forced())
            {
                let (maybe_next_session_change, estimate_next_new_session_weight) =
                    T::NextNewSession::estimate_next_new_session(now);
                if let Some(next_session_change) = maybe_next_session_change {
                    if let Some(remaining) = next_session_change.checked_sub(&now) {
                        if remaining <= T::ElectionLookahead::get() && !remaining.is_zero() {
                            // create snapshot.
                            let (did_snapshot, snapshot_weight) = Self::create_stakers_snapshot();
                            add_weight(0, 0, snapshot_weight);
                            if did_snapshot {
                                // Set the flag to make sure we don't waste any compute here in the same era
                                // after we have triggered the offline compute.
                                <EraElectionStatus<T>>::put(
                                    ElectionStatus::<T::BlockNumber>::Open(now),
                                );
                                add_weight(0, 1, Weight::zero());
                                crate::log!(
                                    info,
                                    "💸 Election window is Open({:?}). Snapshot created",
                                    now
                                );
                            } else {
                                crate::log!(warn, "💸 Failed to create snapshot at {:?}.", now);
                            }
                        }
                    }
                } else {
                    crate::log!(warn, "💸 Estimating next session change failed.");
                }
                add_weight(0, 0, estimate_next_new_session_weight)
            }
            // For `era_election_status`, `is_current_session_final`, `will_era_be_forced`
            add_weight(3, 0, Weight::zero());
            // Additional read from `on_finalize`
            add_weight(1, 0, Weight::zero());
            consumed_weight
        }

        fn on_finalize(_n: BlockNumberFor<T>) {
            // Set the start of the first era.
            if let Some(mut active_era) = Self::active_era() {
                if active_era.start.is_none() {
                    let now_as_millis_u64 = T::UnixTime::now().as_millis().saturated_into::<u64>();
                    active_era.start = Some(now_as_millis_u64);
                    // This write only ever happens once, we don't include it in the weight in
                    // general
                    ActiveEra::<T>::put(active_era);
                }
            }
            // `on_finalize` weight is tracked in `on_initialize`
        }

        fn integrity_test() {
            sp_std::if_std! {
                sp_io::TestExternalities::new_empty().execute_with(||
                    assert!(
                        T::SlashDeferDuration::get() < T::BondingDuration::get() || T::BondingDuration::get() == 0,
                        "As per documentation, slash defer duration ({}) should be less than bonding duration ({}).",
                        T::SlashDeferDuration::get(),
                        T::BondingDuration::get(),
                    )
                );
            }
        }

        /// Check if the current block number is the one at which the election window has been set
        /// to open. If so, it runs the offchain worker code.
        fn offchain_worker(now: T::BlockNumber) {
            use crate::offchain_election::{
                compute_offchain_election, set_check_offchain_execution_status,
            };

            if Self::era_election_status().is_open_at(now) {
                let offchain_status = set_check_offchain_execution_status::<T>(now);
                if let Err(why) = offchain_status {
                    crate::log!(
                        warn,
                        "💸 skipping offchain worker in open election window due to [{:?}]",
                        why
                    );
                } else {
                    if let Err(e) = compute_offchain_election::<T>() {
                        crate::log!(error, "💸 Error in election offchain worker: {:?}", e);
                    } else {
                        crate::log!(debug, "💸 Executed offchain worker thread without errors.");
                    }
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Take the origin account as a stash and lock up `value` of its balance. `controller` will
        /// be the account that controls it.
        ///
        /// `value` must be more than the `minimum_balance` specified by `T::Currency`.
        ///
        /// The dispatch origin for this call must be _Signed_ by the stash account.
        ///
        /// Emits `Bonded`.
        /// ## Complexity
        /// - Independent of the arguments. Moderate complexity.
        /// - O(1).
        /// - Three extra DB entries.
        ///
        /// NOTE: Two of the storage writes (`Self::bonded`, `Self::payee`) are _never_ cleaned
        /// unless the `origin` falls below _existential deposit_ and gets removed as dust.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::zero())]
        pub fn bond(
            origin: OriginFor<T>,
            controller: AccountIdLookupOf<T>,
            #[pallet::compact] value: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            // Polymesh Change
            // -----------------------------------------------------------------
            ensure!(value >= T::MinimumBond::get(), Error::<T>::BondTooSmall);
            // -----------------------------------------------------------------

            let stash = ensure_signed(origin)?;

            if <Bonded<T>>::contains_key(&stash) {
                return Err(Error::<T>::AlreadyBonded.into());
            }

            let controller = T::Lookup::lookup(controller)?;

            if <Ledger<T>>::contains_key(&controller) {
                return Err(Error::<T>::AlreadyPaired.into());
            }

            // Reject a bond which is considered to be _dust_.
            if value < T::Currency::minimum_balance() {
                return Err(Error::<T>::InsufficientValue.into());
            }

            frame_system::Pallet::<T>::inc_consumers(&stash).map_err(|_| Error::<T>::BadState)?;

            // You're auto-bonded forever, here. We might improve this by only bonding when
            // you actually validate/nominate and remove once you unbond __everything__.
            <Bonded<T>>::insert(&stash, &controller);
            <Payee<T>>::insert(&stash, payee);

            let current_era = CurrentEra::<T>::get().unwrap_or(0);
            let history_depth = Self::history_depth();
            let last_reward_era = current_era.saturating_sub(history_depth);

            let stash_balance = T::Currency::free_balance(&stash);
            let value = value.min(stash_balance);

            // Polymesh Change: Add `stash`'s DID to event.
            // -----------------------------------------------------------------
            let did = Context::current_identity::<T::IdentityFn>().unwrap_or_default();
            Self::deposit_event(Event::<T>::Bonded(did, stash.clone(), value));
            // -----------------------------------------------------------------
                
            let item = StakingLedger {
                stash,
                total: value,
                active: value,
                unlocking: Default::default(),
                claimed_rewards: (last_reward_era..current_era).collect(),
            };
            Self::update_ledger(&controller, &item);
            Ok(())
        }

        /// Add some extra amount that have appeared in the stash `free_balance` into the balance up
        /// for staking.
        ///
        /// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
        ///
        /// Use this if there are additional funds in your stash account that you wish to bond.
        /// Unlike [`bond`](Self::bond) or [`unbond`](Self::unbond) this function does not impose
        /// any limitation on the amount that can be added.
        ///
        /// Emits `Bonded`.
        ///
        /// ## Complexity
        /// - Independent of the arguments. Insignificant complexity.
        /// - O(1).
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::zero())]
        pub fn bond_extra(
            origin: OriginFor<T>,
            #[pallet::compact] max_additional: BalanceOf<T>,
        ) -> DispatchResult {
            // Polymesh Change: 
            // -----------------------------------------------------------------
            ensure!(
                Self::era_election_status().is_closed(), 
                Error::<T>::CallNotAllowed
            );
            // -----------------------------------------------------------------

            let stash = ensure_signed(origin)?;

            let controller = Self::bonded(&stash).ok_or(Error::<T>::NotStash)?;
            let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;

            let stash_balance = T::Currency::free_balance(&stash);
            if let Some(extra) = stash_balance.checked_sub(&ledger.total) {
                let extra = extra.min(max_additional);
                ledger.total += extra;
                ledger.active += extra;
                // Last check: the new active amount of ledger must be more than ED.
                ensure!(
                    ledger.active >= T::Currency::minimum_balance(),
                    Error::<T>::InsufficientValue
                );

                // NOTE: ledger must be updated prior to calling `Self::weight_of`.
                Self::update_ledger(&controller, &ledger);

                // Polymesh Change: Add `stash`'s DID to event.
                // -----------------------------------------------------------------
                let did = Context::current_identity::<T::IdentityFn>().unwrap_or_default();
                Self::deposit_event(Event::<T>::Bonded(did, stash.clone(), extra));
                // -----------------------------------------------------------------
            }
            Ok(())
        }

        /// Schedule a portion of the stash to be unlocked ready for transfer out after the bond
        /// period ends. If this leaves an amount actively bonded less than
        /// T::Currency::minimum_balance(), then it is increased to the full amount.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// Once the unlock period is done, you can call `withdraw_unbonded` to actually move
        /// the funds out of management ready for transfer.
        ///
        /// No more than a limited number of unlocking chunks (see `MaxUnlockingChunks`)
        /// can co-exists at the same time. If there are no unlocking chunks slots available
        /// [`Call::withdraw_unbonded`] is called to remove some of the chunks (if possible).
        ///
        /// If a user encounters the `InsufficientBond` error when calling this extrinsic,
        /// they should call `chill` first in order to free up their bonded funds.
        ///
        /// Emits `Unbonded`.
        ///
        /// See also [`Call::withdraw_unbonded`].
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::zero())]
        pub fn unbond(
            origin: OriginFor<T>,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResult {
            // Polymesh Change: 
            // -----------------------------------------------------------------
            ensure!(
                Self::era_election_status().is_closed(), 
                Error::<T>::CallNotAllowed
            );
            // -----------------------------------------------------------------

            let controller = ensure_signed(origin)?;
            let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;

            ensure!(
                ledger.unlocking.len() < T::MaxUnlockingChunks::get() as usize,
                Error::<T>::NoMoreChunks,
            );

            if Validators::<T>::contains_key(&ledger.stash) {
                ensure!(
                    ledger.active.saturating_sub(value) >= <MinimumBondThreshold<T>>::get(), 
                    Error::<T>::InvalidValidatorUnbondAmount
                );
            }

            Self::unbond_balance(controller, &mut ledger, value)?;
            Ok(())
        }

        /// Remove any unlocked chunks from the `unlocking` queue from our management.
        ///
        /// This essentially frees up that balance to be used by the stash account to do
        /// whatever it wants.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller.
        ///
        /// Emits `Withdrawn`.
        ///
        /// See also [`Call::unbond`].
        ///
        /// ## Complexity
        /// O(S) where S is the number of slashing spans to remove
        /// NOTE: Weight annotation is the kill scenario, we refund otherwise.
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::zero())]
        pub fn withdraw_unbonded(
            origin: OriginFor<T>,
            num_slashing_spans: u32,
        ) -> DispatchResultWithPostInfo {
            // Polymesh Change: 
            // -----------------------------------------------------------------
            ensure!(
                Self::era_election_status().is_closed(), 
                Error::<T>::CallNotAllowed
            );
            // -----------------------------------------------------------------
            let controller = ensure_signed(origin)?;

            let actual_weight = Self::do_withdraw_unbonded(&controller, num_slashing_spans)?;
            Ok(actual_weight)
        }

        /// Declare the desire to validate for the origin controller.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::zero())]
        pub fn validate(origin: OriginFor<T>, prefs: ValidatorPrefs) -> DispatchResult {
            // Polymesh Change: 
            // -----------------------------------------------------------------
            ensure!(
                Self::era_election_status().is_closed(), 
                Error::<T>::CallNotAllowed
            );
            // -----------------------------------------------------------------
            let controller = ensure_signed(origin)?;

            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;

            ensure!(
                ledger.active >= Self::min_bond_threshold(),
                Error::<T>::InsufficientValue
            );
            let stash = &ledger.stash;

            // ensure their commission is correct.
            ensure!(
                prefs.commission <= Self::validator_commission_cap(),
                Error::<T>::InvalidValidatorCommission
            );

            // Polymesh Change: Make sure stash has valid permissioned identity.
            // -----------------------------------------------------------------
            let stash_identity = 
                <Identity<T>>::get_identity(stash).ok_or(Error::<T>::StashIdentityDoesNotExist)?;
            let mut stash_did_preferences = Self::permissioned_identity(stash_identity)
                .ok_or(Error::<T>::StashIdentityNotPermissioned)?;

            // Only check limits if they are not already a validator.
            if !Validators::<T>::contains_key(stash) {
                // Ensure identity doesn't run more validators than the intended count.
                ensure!(
                    stash_did_preferences.running_count < stash_did_preferences.intended_count, 
                    Error::<T>::HitIntendedValidatorCount
                );
                stash_did_preferences.running_count += 1;
                <Identity<T>>::add_account_key_ref_count(&stash);
            }
            PermissionedIdentity::<T>::insert(stash_identity, stash_did_preferences);
            // -----------------------------------------------------------------

            Self::do_remove_nominator(stash);
            Self::do_add_validator(stash, prefs.clone());

            Ok(())
        }

        /// Declare the desire to nominate `targets` for the origin controller.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// ## Complexity
        /// - The transaction's complexity is proportional to the size of `targets` (N)
        /// which is capped at CompactAssignments::LIMIT (T::MaxNominations).
        /// - Both the reads and writes follow a similar pattern.
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::zero())]
        pub fn nominate(
            origin: OriginFor<T>,
            targets: Vec<AccountIdLookupOf<T>>,
        ) -> DispatchResult {
            // Polymesh Change: 
            // -----------------------------------------------------------------
            ensure!(
                Self::era_election_status().is_closed(), 
                Error::<T>::CallNotAllowed
            );
            // -----------------------------------------------------------------
            let controller = ensure_signed(origin)?;

            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            let stash = &ledger.stash;

            ensure!(!targets.is_empty(), Error::<T>::EmptyTargets);
            ensure!(
                targets.len() <= T::MaxNominations::get() as usize,
                Error::<T>::TooManyTargets
            );

            let old = Nominators::<T>::get(stash).map_or_else(Vec::new, |x| x.targets.into_inner());

            let targets: BoundedVec<_, _> = targets
                .into_iter()
                .map(|t| T::Lookup::lookup(t).map_err(DispatchError::from))
                .map(|n| {
                    n.and_then(|n| {
                        if old.contains(&n) || !Validators::<T>::get(&n).blocked {
                            Ok(n)
                        } else {
                            Err(Error::<T>::BadTarget.into())
                        }
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| Error::<T>::TooManyNominators)?;

            // Polymesh Change: Gets Nominator DID and make sure it has a CDD claim
            // -----------------------------------------------------------------
            let nominator_identity = 
                <Identity<T>>::get_identity(stash).ok_or(Error::<T>::StashIdentityDoesNotExist)?;
            ensure!(
                <Identity<T>>::fetch_cdd(
                    nominator_identity,
                    (Self::get_bonding_duration_period() as u32).into()
                )
                .is_some(),
                Error::<T>::StashIdentityNotCDDed,
            );

            Self::release_running_validator(&stash);
            Self::deposit_event(Event::<T>::Nominated(
                nominator_identity,
                stash.clone(),
                targets.to_vec(),
            ));
            // -----------------------------------------------------------------

            let nominations = Nominations {
                targets,
                // Initial nominations are considered submitted at era 0. See `Nominations` doc.
                submitted_in: Self::current_era().unwrap_or(0),
                suppressed: false,
            };

            Self::do_remove_validator(stash);
            Self::do_add_nominator(stash, nominations);
            Ok(())
        }

        /// Declare no desire to either validate or nominate.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// ## Complexity
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains one read.
        /// - Writes are limited to the `origin` account key.
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::zero())]
        pub fn chill(origin: OriginFor<T>) -> DispatchResult {
            // Polymesh Change: 
            // -----------------------------------------------------------------
            ensure!(
                Self::era_election_status().is_closed(), 
                Error::<T>::CallNotAllowed
            );
            // -----------------------------------------------------------------
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            Self::chill_stash(&ledger.stash);
            Ok(())
        }

        /// (Re-)set the payment target for a controller.
        ///
        /// Effects will be felt instantly (as soon as this function is completed successfully).
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// ## Complexity
        /// - O(1)
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains a limited number of reads.
        /// - Writes are limited to the `origin` account key.
        /// ---------
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::zero())]
        pub fn set_payee(
            origin: OriginFor<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            let stash = &ledger.stash;
            <Payee<T>>::insert(stash, payee);
            Ok(())
        }

        /// (Re-)set the controller of a stash.
        ///
        /// Effects will be felt instantly (as soon as this function is completed successfully).
        ///
        /// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
        ///
        /// ## Complexity
        /// O(1)
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains a limited number of reads.
        /// - Writes are limited to the `origin` account key.
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::zero())]
        pub fn set_controller(
            origin: OriginFor<T>,
            controller: AccountIdLookupOf<T>,
        ) -> DispatchResult {
            let stash = ensure_signed(origin)?;
            let old_controller = Self::bonded(&stash).ok_or(Error::<T>::NotStash)?;
            let controller = T::Lookup::lookup(controller)?;
            if <Ledger<T>>::contains_key(&controller) {
                return Err(Error::<T>::AlreadyPaired.into());
            }
            if controller != old_controller {
                <Bonded<T>>::insert(&stash, &controller);
                if let Some(l) = <Ledger<T>>::take(&old_controller) {
                    <Ledger<T>>::insert(&controller, l);
                }
            }
            Ok(())
        }

        /// Sets the ideal number of validators.
        ///
        /// The dispatch origin must be Root.
        ///
        /// ## Complexity
        /// O(1)
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::zero())]
        pub fn set_validator_count(
            origin: OriginFor<T>,
            #[pallet::compact] new: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ValidatorCount::<T>::put(new);
            Ok(())
        }

        /// Increments the ideal number of validators upto maximum of
        /// `ElectionProviderBase::MaxWinners`.
        ///
        /// The dispatch origin must be Root.
        ///
        /// ## Complexity
        /// Same as [`Self::set_validator_count`].
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::zero())]
        pub fn increase_validator_count(
            origin: OriginFor<T>,
            #[pallet::compact] additional: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let old = ValidatorCount::<T>::get();
            let new = old
                .checked_add(additional)
                .ok_or(ArithmeticError::Overflow)?;
            ValidatorCount::<T>::put(new);
            Ok(())
        }

        /// Scale up the ideal number of validators by a factor upto maximum of
        /// `ElectionProviderBase::MaxWinners`.
        ///
        /// The dispatch origin must be Root.
        ///
        /// ## Complexity
        /// Same as [`Self::set_validator_count`].
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::zero())]
        pub fn scale_validator_count(origin: OriginFor<T>, factor: Percent) -> DispatchResult {
            ensure_root(origin)?;
            let old = ValidatorCount::<T>::get();
            let new = old
                .checked_add(factor.mul_floor(old))
                .ok_or(ArithmeticError::Overflow)?;
            ValidatorCount::<T>::put(new);
            Ok(())
        }

        /// Scale up the ideal number of validators by a factor upto maximum of 
        /// `ElectionProviderBase::MaxWinners`.
        ///
        /// The dispatch origin must be Root.
        ///
        /// ## Complexity
        /// Same as [`Self::set_validator_count`].
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::zero())]
        pub fn add_permissioned_validator(
            origin: OriginFor<T>, 
            identity: IdentityId, 
            intended_count: Option<u32>
        ) -> DispatchResult {
            T::RequiredAddOrigin::ensure_origin(origin)?;
            ensure!(
                Self::permissioned_identity(&identity).is_none(),
                Error::<T>::AlreadyExists
            );
            // Validate the cdd status of the identity.
            ensure!(
                <Identity<T>>::has_valid_cdd(identity),
                Error::<T>::InvalidValidatorIdentity
            );
            let preferences = match intended_count {
                Some(intended_count) => {
                    // Maximum allowed validator count is always less than the `MaxValidatorPerIdentity of validator_count()`.
                    ensure!(
                        intended_count < Self::get_allowed_validator_count(),
                        Error::<T>::IntendedCountIsExceedingConsensusLimit
                    );
                    PermissionedIdentityPrefs::new(intended_count)
                }
                None => PermissionedIdentityPrefs::default(),
            };

            // Change identity status to be Permissioned
            PermissionedIdentity::<T>::insert(&identity, preferences);
            Self::deposit_event(Event::<T>::PermissionedIdentityAdded(GC_DID, identity));
            Ok(())
        }

        /// Remove an identity from the pool of (wannabe) validator identities. Effects are known in the next session.
        /// Staking module checks `PermissionedIdentity` to ensure validators have
        /// completed KYB compliance
        ///
        /// # Arguments
        /// * origin Required origin for removing a potential validator.
        /// * identity Validator's IdentityId.
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::zero())]
        pub fn remove_permissioned_validator(
            origin: OriginFor<T>, 
            identity: IdentityId, 
        ) -> DispatchResult {
            T::RequiredRemoveOrigin::ensure_origin(origin)?;
            ensure!(
                Self::permissioned_identity(&identity).is_some(),
                Error::<T>::NotExists
            );
            // Change identity status to be Non-Permissioned
            PermissionedIdentity::<T>::remove(&identity);

            Self::deposit_event(Event::<T>::PermissionedIdentityRemoved(GC_DID, identity));
            Ok(())
        }

        /// Validate the nominators CDD expiry time.
        ///
        /// If an account from a given set of address is nominating then check the CDD expiry time 
        /// of it and if it is expired then the account should be unbonded and removed from the 
        /// nominating process.
        #[pallet::call_index(14)]
        #[pallet::weight(Weight::zero())]
        pub fn validate_cdd_expiry_nominators(
            origin: OriginFor<T>, 
            targets: Vec<T::AccountId> 
        ) -> DispatchResult {
            let (caller, caller_id) = Identity::<T>::ensure_did(origin)?;

            let mut expired_nominators = Vec::new();
            ensure!(!targets.is_empty(), "targets cannot be empty");
            // Iterate provided list of accountIds (These accountIds should be stash type account).
            for target in targets
                .iter()
                // Nominator must be vouching for someone.
                .filter(|target| Self::nominators(target).is_some())
                // Access the DIDs of the nominators whose CDDs have expired.
                .filter(|target| {
                    // Fetch all the claim values provided by the trusted service providers
                    // There is a possibility that nominator will have more than one claim for the same key,
                    // So we iterate all of them and if any one of the claim value doesn't expire then nominator posses
                    // valid CDD otherwise it will be removed from the pool of the nominators.
                    // If the target has no DID, it's also removed.
                    <Identity<T>>::get_identity(&target)
                        .filter(|did| <Identity<T>>::has_valid_cdd(*did))
                        .is_none()
                })
            {
                // Un-bonding the balance that bonded with the controller account of a Stash account
                // This unbonded amount only be accessible after completion of the BondingDuration
                // Controller account need to call the dispatchable function `withdraw_unbond` to withdraw fund.

                let controller = Self::bonded(target).ok_or(Error::<T>::NotStash)?;
                let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
                let active_balance = ledger.active;
                if ledger.unlocking.len() < T::MaxUnlockingChunks::get() as usize {
                    Self::unbond_balance(controller, &mut ledger, active_balance)?;

                    expired_nominators.push(target.clone());
                    // Free the nominator from the valid nominator list
                    <Nominators<T>>::remove(target);
                }
            }
            Self::deposit_event(Event::<T>::InvalidatedNominators(
                caller_id,
                caller,
                expired_nominators,
            ));
            Ok(())
        }

        /// Changes commission rate which applies to all validators. Only Governance
        /// committee is allowed to change this value.
        ///
        /// # Arguments
        /// * `new_cap` the new commission cap.
        #[pallet::call_index(15)]
        #[pallet::weight(Weight::zero())]
        pub fn set_commission_cap(
            origin: OriginFor<T>, 
            new_cap: Perbill 
        ) -> DispatchResult {
            T::RequiredCommissionOrigin::ensure_origin(origin.clone())?;

            // Update the cap, assuming it changed, or error.
            let old_cap = ValidatorCommissionCap::<T>::try_mutate(|cap| -> Result<_, DispatchError> {
                ensure!(*cap != new_cap, Error::<T>::NoChange);
                Ok(core::mem::replace(cap, new_cap))
            })?;

            // Update `commission` in each validator prefs to `min(comission, new_cap)`.
            <Validators<T>>::translate(|_, mut prefs: ValidatorPrefs| {
                prefs.commission = prefs.commission.min(new_cap);
                Some(prefs)
            });

            Self::deposit_event(Event::<T>::CommissionCapUpdated(GC_DID, old_cap, new_cap));
            Ok(())
        }

        /// Changes commission rate which applies to all validators. Only Governance
        /// committee is allowed to change this value.
        ///
        /// # Arguments
        /// * `new_cap` the new commission cap.
        #[pallet::call_index(16)]
        #[pallet::weight(Weight::zero())]
        pub fn set_min_bond_threshold(
            origin: OriginFor<T>, 
            new_value: BalanceOf<T> 
        ) -> DispatchResult {
            T::RequiredCommissionOrigin::ensure_origin(origin.clone())?;
            <MinimumBondThreshold<T>>::put(new_value);
            Self::deposit_event(Event::<T>::MinimumBondThresholdUpdated(
                Some(GC_DID),
                new_value,
            ));
            Ok(())
        }

        /// Force there to be no new eras indefinitely.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # Warning
        ///
        /// The election process starts multiple blocks before the end of the era.
        /// Thus the election process may be ongoing when this is called. In this case the
        /// election will continue until the next era is triggered.
        ///
        /// ## Complexity
        /// - No arguments.
        /// - Weight: O(1)
        #[pallet::call_index(17)]
        #[pallet::weight(Weight::zero())]
        pub fn force_no_eras(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            Self::set_force_era(Forcing::ForceNone);
            Ok(())
        }

        /// Force there to be a new era at the end of the next session. After this, it will be
        /// reset to normal (non-forced) behaviour.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # Warning
        ///
        /// The election process starts multiple blocks before the end of the era.
        /// If this is called just before a new era is triggered, the election process may not
        /// have enough blocks to get a result.
        ///
        /// ## Complexity
        /// - No arguments.
        /// - Weight: O(1)
        #[pallet::call_index(18)]
        #[pallet::weight(Weight::zero())]
        pub fn force_new_era(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            Self::set_force_era(Forcing::ForceNew);
            Ok(())
        }

        /// Set the validators who cannot be slashed (if any).
        ///
        /// The dispatch origin must be Root.
        #[pallet::call_index(19)]
        #[pallet::weight(Weight::zero())]
        pub fn set_invulnerables(
            origin: OriginFor<T>,
            invulnerables: Vec<T::AccountId>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <Invulnerables<T>>::put(invulnerables);
            Ok(())
        }

        /// Force a current staker to become completely unstaked, immediately.
        ///
        /// The dispatch origin must be Root.
        #[pallet::call_index(20)]
        #[pallet::weight(Weight::zero())]
        pub fn force_unstake(
            origin: OriginFor<T>,
            stash: T::AccountId,
            num_slashing_spans: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Remove all staking-related information.
            Self::kill_stash(&stash, num_slashing_spans)?;

            // Remove the lock.
            T::Currency::remove_lock(STAKING_ID, &stash);
            Ok(())
        }

        /// Force there to be a new era at the end of sessions indefinitely.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # Warning
        ///
        /// The election process starts multiple blocks before the end of the era.
        /// If this is called just before a new era is triggered, the election process may not
        /// have enough blocks to get a result.
        #[pallet::call_index(21)]
        #[pallet::weight(Weight::zero())]
        pub fn force_new_era_always(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            Self::set_force_era(Forcing::ForceAlways);
            Ok(())
        }

        /// Cancel enactment of a deferred slash.
        ///
        /// Can be called by the `T::AdminOrigin`.
        ///
        /// Parameters: era and indices of the slashes for that era to kill.
        #[pallet::call_index(22)]
        #[pallet::weight(Weight::zero())]
        pub fn cancel_deferred_slash(
            origin: OriginFor<T>,
            era: EraIndex,
            slash_indices: Vec<u32>,
        ) -> DispatchResult {
            T::SlashCancelOrigin::ensure_origin(origin)?;

            ensure!(!slash_indices.is_empty(), Error::<T>::EmptyTargets);
            ensure!(
                is_sorted_and_unique(&slash_indices),
                Error::<T>::NotSortedAndUnique
            );

            let mut unapplied = <Self as Store>::UnappliedSlashes::get(&era);
            let last_item = slash_indices[slash_indices.len() - 1];
            ensure!(
                (last_item as usize) < unapplied.len(),
                Error::<T>::InvalidSlashIndex
            );

            for (removed, index) in slash_indices.into_iter().enumerate() {
                let index = (index as usize) - removed;
                unapplied.remove(index);
            }

            <Self as Store>::UnappliedSlashes::insert(&era, &unapplied);
            Ok(())
        }

        /// Pay out all the stakers behind a single validator for a single era.
        ///
        /// - `validator_stash` is the stash account of the validator. Their nominators, up to
        ///   `T::MaxNominatorRewardedPerValidator`, will also receive their rewards.
        /// - `era` may be any era between `[current_era - history_depth; current_era]`.
        ///
        /// The origin of this call must be _Signed_. Any account can call this function, even if
        /// it is not one of the stakers.
        ///
        /// ## Complexity
        /// - At most O(MaxNominatorRewardedPerValidator).
        #[pallet::call_index(23)]
        #[pallet::weight(Weight::zero())]
        pub fn payout_stakers(
            origin: OriginFor<T>,
            validator_stash: T::AccountId,
            era: EraIndex,
        ) -> DispatchResult {
            // Polymesh Change: 
            // -----------------------------------------------------------------
            ensure!(
                Self::era_election_status().is_closed(), 
                Error::<T>::CallNotAllowed
            );
            // -----------------------------------------------------------------
            ensure_signed(origin)?;
            Self::do_payout_stakers(validator_stash, era)
        }

        /// Rebond a portion of the stash scheduled to be unlocked.
        ///
        /// The dispatch origin must be signed by the controller.
        ///
        /// ## Complexity
        /// - Time complexity: O(L), where L is unlocking chunks
        /// - Bounded by `MaxUnlockingChunks`.
        #[pallet::call_index(24)]
        #[pallet::weight(Weight::zero())]
        pub fn rebond(
            origin: OriginFor<T>,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            // Polymesh Change: 
            // -----------------------------------------------------------------
            ensure!(
                Self::era_election_status().is_closed(), 
                Error::<T>::CallNotAllowed
            );
            // -----------------------------------------------------------------
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            ensure!(!ledger.unlocking.is_empty(), Error::<T>::NoUnlockChunk);

            let (ledger, _rebonded_value) = ledger.rebond(value);
            // Last check: the new active amount of ledger must be more than ED.
            ensure!(
                ledger.active >= T::Currency::minimum_balance(),
                Error::<T>::InsufficientValue
            );

            // NOTE: ledger must be updated prior to calling `Self::weight_of`.
            Self::update_ledger(&controller, &ledger);

            Ok(Some(
                Weight::from_ref_time(
                    35u64 * WEIGHT_REF_TIME_PER_MICROS
                        + 50u64 * WEIGHT_REF_TIME_PER_NANOS * (ledger.unlocking.len() as u64),
                ) + T::DbWeight::get().reads_writes(3, 2),
            )
            .into())
        }

        /// Rebond a portion of the stash scheduled to be unlocked.
        ///
        /// The dispatch origin must be signed by the controller.
        ///
        /// ## Complexity
        /// - Time complexity: O(L), where L is unlocking chunks
        /// - Bounded by `MaxUnlockingChunks`.
        #[pallet::call_index(25)]
        #[pallet::weight(Weight::zero())]
        pub fn set_history_depth(
            origin: OriginFor<T>,
            #[pallet::compact] new_history_depth: EraIndex,
            #[pallet::compact] _era_items_deleted: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            if let Some(current_era) = Self::current_era() {
                HistoryDepth::<T>::mutate(|history_depth| {
					let last_kept = current_era.checked_sub(*history_depth).unwrap_or(0);
					let new_last_kept = current_era.checked_sub(new_history_depth).unwrap_or(0);
                    for era_index in last_kept..new_last_kept {
                        Self::clear_era_information(era_index);
                    }
                    *history_depth = new_history_depth
                })
            }
            Ok(())
        }

        /// Remove all data structures concerning a staker/stash once it is at a state where it can
        /// be considered `dust` in the staking system. The requirements are:
        ///
        /// 1. the `total_balance` of the stash is below existential deposit.
        /// 2. or, the `ledger.total` of the stash is below existential deposit.
        ///
        /// The former can happen in cases like a slash; the latter when a fully unbonded account
        /// is still receiving staking rewards in `RewardDestination::Staked`.
        ///
        /// It can be called by anyone, as long as `stash` meets the above requirements.
        ///
        /// Refunds the transaction fees upon successful execution.
        #[pallet::call_index(26)]
        #[pallet::weight(Weight::zero())]
        pub fn reap_stash(
            _origin: OriginFor<T>,
            stash: T::AccountId,
            num_slashing_spans: u32,
        ) -> DispatchResultWithPostInfo {
            //let _ = ensure_signed(origin)?;

            //let ed = T::Currency::minimum_balance();
            //let reapable = T::Currency::total_balance(&stash) < ed
            //    || Self::ledger(Self::bonded(stash.clone()).ok_or(Error::<T>::NotStash)?)
            //        .map(|l| l.total)
            //        .unwrap_or_default()
            //        < ed;
            //ensure!(reapable, Error::<T>::FundedTarget);

			ensure!(
                T::Currency::total_balance(&stash) == T::Currency::minimum_balance(), 
                Error::<T>::FundedTarget
            );
            Self::kill_stash(&stash, num_slashing_spans)?;
            T::Currency::remove_lock(STAKING_ID, &stash);

            Ok(Pays::No.into())
        }

        /// Submit an election result to the chain. If the solution:
        ///
        /// 1. is valid.
        /// 2. has a better score than a potentially existing solution on chain.
        ///
        /// then, it will be _put_ on chain.
        ///
        /// A solution consists of two pieces of data:
        ///
        /// 1. `winners`: a flat vector of all the winners of the round.
        /// 2. `assignments`: the compact version of an assignment vector that encodes the edge
        ///    weights.
        ///
        /// Both of which may be computed using _phragmen_, or any other algorithm.
        ///
        /// Additionally, the submitter must provide:
        ///
        /// - The `score` that they claim their solution has.
        ///
        /// Both validators and nominators will be represented by indices in the solution. The
        /// indices should respect the corresponding types ([`ValidatorIndex`] and
        /// [`NominatorIndex`]). Moreover, they should be valid when used to index into
        /// [`SnapshotValidators`] and [`SnapshotNominators`]. Any invalid index will cause the
        /// solution to be rejected. These two storage items are set during the election window and
        /// may be used to determine the indices.
        ///
        /// A solution is valid if:
        ///
        /// 0. It is submitted when [`EraElectionStatus`] is `Open`.
        /// 1. Its claimed score is equal to the score computed on-chain.
        /// 2. Presents the correct number of winners.
        /// 3. All indexes must be value according to the snapshot vectors. All edge values must
        ///    also be correct and should not overflow the granularity of the ratio type (i.e. 256
        ///    or billion).
        /// 4. For each edge, all targets are actually nominated by the voter.
        /// 5. Has correct self-votes.
        ///
        /// A solutions score is consisted of 3 parameters:
        ///
        /// 1. `min { support.total }` for each support of a winner. This value should be maximized.
        /// 2. `sum { support.total }` for each support of a winner. This value should be minimized.
        /// 3. `sum { support.total^2 }` for each support of a winner. This value should be
        ///    minimized (to ensure less variance)
        ///
        /// # <weight>
        /// The transaction is assumed to be the longest path, a better solution.
        ///   - Initial solution is almost the same.
        ///   - Worse solution is retraced in pre-dispatch-checks which sets its own weight.
        /// # </weight>
        #[pallet::call_index(27)]
        #[pallet::weight(Weight::zero())]
        pub fn submit_election_solution(
            origin: OriginFor<T>,
            winners: Vec<ValidatorIndex>,
            compact: CompactAssignments,
            score: ElectionScore,
            era: EraIndex,
            size: ElectionSize,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;
            Self::check_and_replace_solution(
                winners,
                compact,
                ElectionCompute::Signed,
                score,
                era,
                size,
            )
        }

        /// Unsigned version of `submit_election_solution`.
        ///
        /// Note that this must pass the [`ValidateUnsigned`] check which only allows transactions
        /// from the local node to be included. In other words, only the block author can include a
        /// transaction in the block.
        ///
        /// # <weight>
        /// See [`submit_election_solution`].
        /// # </weight>
        #[pallet::call_index(28)]
        #[pallet::weight(Weight::zero())]
        pub fn submit_election_solution_unsigned(
            origin: OriginFor<T>,
            winners: Vec<ValidatorIndex>,
            compact: CompactAssignments,
            score: ElectionScore,
            era: EraIndex,
            size: ElectionSize,
        ) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;
            let adjustments = Self::check_and_replace_solution(
                winners,
                compact,
                ElectionCompute::Unsigned,
                score,
                era,
                size,
            ).expect(
                "An unsigned solution can only be submitted by validators; A validator should \
                always produce correct solutions, else this block should not be imported, thus \
                effectively depriving the validators from their authoring reward. Hence, this panic
                is expected."
            );

            Ok(adjustments)
        }

        #[pallet::call_index(29)]
        #[pallet::weight(Weight::zero())]
        pub fn payout_stakers_by_system(
            origin: OriginFor<T>,
            validator_stash: T::AccountId, 
            era: EraIndex
        ) -> DispatchResult {
            ensure!(
                Self::era_election_status().is_closed(), 
                Error::<T>::CallNotAllowed
            );
            ensure_root(origin)?;
            Self::do_payout_stakers(validator_stash, era)
        }

        /// Switch slashing status on the basis of given `SlashingSwitch`. Can only be called by root.
        ///
        /// # Arguments
        /// * origin - AccountId of root.
        /// * slashing_switch - Switch used to set the targets for s
        #[pallet::call_index(30)]
        #[pallet::weight(Weight::zero())]
        pub fn change_slashing_allowed_for(
            origin: OriginFor<T>,
            slashing_switch: SlashingSwitch
        ) -> DispatchResult {
            // Ensure origin should be root.
            ensure_root(origin)?;
            SlashingAllowedFor::<T>::put(slashing_switch);
            Self::deposit_event(Event::<T>::SlashingAllowedForChanged(slashing_switch));
            Ok(())
        }

        /// Update the intended validator count for a given DID.
        ///
        /// # Arguments
        /// * origin which must be the required origin for adding a potential validator.
        /// * identity to add as a validator.
        /// * new_intended_count New value of intended co
        #[pallet::call_index(31)]
        #[pallet::weight(Weight::zero())]
        pub fn update_permissioned_validator_intended_count(
            origin: OriginFor<T>,
            identity: IdentityId, 
            new_intended_count: u32
        ) -> DispatchResult {
            T::RequiredAddOrigin::ensure_origin(origin)?;
            ensure!(
                Self::get_allowed_validator_count() > new_intended_count, 
                Error::<T>::IntendedCountIsExceedingConsensusLimit
            );
            PermissionedIdentity::<T>::try_mutate(&identity, |pref| {
                pref.as_mut()
                    .ok_or_else(|| Error::<T>::NotExists.into())
                    .map(|p| p.intended_count = new_intended_count)
            })
        }

        /// GC forcefully chills a validator.
        /// Effects will be felt at the beginning of the next era.
        /// And, it can be only called when [`EraElectionStatus`] is `Closed`.
        /// 
        /// # Arguments
        /// * origin which must be a GC.
        /// * identity must be permissioned to run operator/validator nodes.
        /// * stash_keys contains the secondary keys of the permissioned identity
        /// 
        /// # Errors
        /// * `BadOrigin` The origin was not a GC member.
        /// * `CallNotAllowed` The call is not allowed at the given time due to restrictions of election period.
        /// * `NotExists` Permissioned validator doesn't exist.
        /// * `NotStash` Not a stash account for the permissioned i
        #[pallet::call_index(32)]
        #[pallet::weight(Weight::zero())]
        pub fn chill_from_governance(
            origin: OriginFor<T>,
            identity: IdentityId, 
            stash_keys: Vec<T::AccountId>
        ) -> DispatchResult {
            Self::base_chill_from_governance(origin, identity, stash_keys)
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;
            
        fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            Self::validate_unsigned_call(source, call)
        }

        fn pre_dispatch(call: &Self::Call) -> Result<(), TransactionValidityError> {
            Self::pre_dispatch_call(call)
        }
    }
}

/// Check that list is sorted and has no duplicates.
fn is_sorted_and_unique(list: &[u32]) -> bool {
    list.windows(2).all(|w| w[0] < w[1])
}
