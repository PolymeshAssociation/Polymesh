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

// Modified by Polymath Inc - 18th November 2019
// Added ability to permission validators based on governance and identity credentials
// In Polymesh, validators must join the network through a governance process and have
// required credentials (claims on their identities)

//! # Staking Module
//!
//! The Staking module is used to manage funds at stake by network maintainers.
//!
//! - [`staking::Trait`](./trait.Trait.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! The Staking module is the means by which a set of network maintainers (known as _authorities_
//! in some contexts and _validators_ in others) are chosen based upon those who voluntarily place
//! funds under deposit. Under deposit, those funds are rewarded under normal operation but are
//! held at pain of _slash_ (expropriation) should the staked maintainer be found not to be
//! discharging its duties properly.
//!
//! ### Terminology
//! <!-- Original author of paragraph: @gavofyork -->
//!
//! - Staking: The process of locking up funds for some time, placing them at risk of slashing
//! (loss) in order to become a rewarded maintainer of the network.
//! - Validating: The process of running a node to actively maintain the network, either by
//! producing blocks or guaranteeing finality of the chain.
//! - Nominating: The process of placing staked funds behind one or more validators in order to
//! share in any reward, and punishment, they take.
//! - Stash account: The account holding an owner's funds used for staking.
//! - Controller account: The account that controls an owner's funds for staking.
//! - Era: A (whole) number of sessions, which is the period that the validator set (and each
//! validator's active nominator set) is recalculated and where rewards are paid out.
//! - Slash: The punishment of a staker by reducing its funds.
//!
//! ### Goals
//! <!-- Original author of paragraph: @gavofyork -->
//!
//! The staking system in Substrate NPoS is designed to make the following possible:
//!
//! - Stake funds that are controlled by a cold wallet.
//! - Withdraw some, or deposit more, funds without interrupting the role of an entity.
//! - Switch between roles (nominator, validator, idle) with minimal overhead.
//!
//! ### Scenarios
//!
//! #### Staking
//!
//! Almost any interaction with the Staking module requires a process of _**bonding**_ (also known
//! as being a _staker_). To become *bonded*, a fund-holding account known as the _stash account_,
//! which holds some or all of the funds that become frozen in place as part of the staking process,
//! is paired with an active **controller** account, which issues instructions on how they shall be
//! used.
//!
//! An account pair can become bonded using the [`bond`](./enum.Call.html#variant.bond) call.
//!
//! Stash accounts can change their associated controller using the
//! [`set_controller`](./enum.Call.html#variant.set_controller) call.
//!
//! There are three possible roles that any staked account pair can be in: `Validator`, `Nominator`
//! and `Idle` (defined in [`StakerStatus`](./enum.StakerStatus.html)). There are three
//! corresponding instructions to change between roles, namely:
//! [`validate`](./enum.Call.html#variant.validate), [`nominate`](./enum.Call.html#variant.nominate),
//! and [`chill`](./enum.Call.html#variant.chill).
//!
//! #### Validating
//!
//! A **validator** takes the role of either validating blocks or ensuring their finality,
//! maintaining the veracity of the network. A validator should avoid both any sort of malicious
//! misbehavior and going offline. Bonded accounts that state interest in being a validator do NOT
//! get immediately chosen as a validator. Instead, they are declared as a _candidate_ and they
//! _might_ get elected at the _next era_ as a validator. The result of the election is determined
//! by nominators and their votes.
//!
//! An account can become a validator candidate via the
//! [`validate`](./enum.Call.html#variant.validate) call.
//!
//! #### Nomination
//!
//! A **nominator** does not take any _direct_ role in maintaining the network, instead, it votes on
//! a set of validators  to be elected. Once interest in nomination is stated by an account, it
//! takes effect at the next election round. The funds in the nominator's stash account indicate the
//! _weight_ of its vote. Both the rewards and any punishment that a validator earns are shared
//! between the validator and its nominators. This rule incentivizes the nominators to NOT vote for
//! the misbehaving/offline validators as much as possible, simply because the nominators will also
//! lose funds if they vote poorly.
//!
//! An account can become a nominator via the [`nominate`](enum.Call.html#variant.nominate) call.
//!
//! #### Rewards and Slash
//!
//! The **reward and slashing** procedure is the core of the Staking module, attempting to _embrace
//! valid behavior_ while _punishing any misbehavior or lack of availability_.
//!
//! Slashing can occur at any point in time, once misbehavior is reported. Once slashing is
//! determined, a value is deducted from the balance of the validator and all the nominators who
//! voted for this validator (values are deducted from the _stash_ account of the slashed entity).
//!
//! Similar to slashing, rewards are also shared among a validator and its associated nominators.
//! Yet, the reward funds are not always transferred to the stash account and can be configured.
//! See [Reward Calculation](#reward-calculation) for more details.
//!
//! #### Chilling
//!
//! Finally, any of the roles above can choose to step back temporarily and just chill for a while.
//! This means that if they are a nominator, they will not be considered as voters anymore and if
//! they are validators, they will no longer be a candidate for the next election.
//!
//! An account can step back via the [`chill`](enum.Call.html#variant.chill) call.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! The dispatchable functions of the Staking module enable the steps needed for entities to accept
//! and change their role, alongside some helper functions to get/set the metadata of the module.
//!
//! ### Public Functions
//!
//! The Staking module contains many public storage items and (im)mutable functions.
//!
//! ## Implementation Details
//!
//! ### Slot Stake
//!
//! The term [`SlotStake`](./struct.Module.html#method.slot_stake) will be used throughout this
//! section. It refers to a value calculated at the end of each era, containing the _minimum value
//! at stake among all validators._ Note that a validator's value at stake might be a combination
//! of the validator's own stake and the votes it received. See [`Exposure`](./struct.Exposure.html)
//! for more details.
//!
//! ### Reward Calculation
//!
//! Validators and nominators are rewarded at the end of each era. The total reward of an era is
//! calculated using the era duration and the staking rate (the total amount of tokens staked by
//! nominators and validators, divided by the total token supply). It aims to incentivise toward a
//! defined staking rate. The full specification can be found
//! [here](https://research.web3.foundation/en/latest/polkadot/Token%20Economics/#inflation-model).
//!
//! Total reward is split among validators and their nominators depending on the number of points
//! they received during the era. Points are added to a validator using
//! [`reward_by_ids`](./enum.Call.html#variant.reward_by_ids) or
//! [`reward_by_indices`](./enum.Call.html#variant.reward_by_indices).
//!
//! [`Module`](./struct.Module.html) implements
//! [`authorship::EventHandler`](../srml_authorship/trait.EventHandler.html) to add reward points
//! to block producer and block producer of referenced uncles.
//!
//! The validator and its nominator split their reward as following:
//!
//! The validator can declare an amount, named
//! [`validator_payment`](./struct.ValidatorPrefs.html#structfield.validator_payment), that does not
//! get shared with the nominators at each reward payout through its
//! [`ValidatorPrefs`](./struct.ValidatorPrefs.html). This value gets deducted from the total reward
//! that is paid to the validator and its nominators. The remaining portion is split among the
//! validator and all of the nominators that nominated the validator, proportional to the value
//! staked behind this validator (_i.e._ dividing the
//! [`own`](./struct.Exposure.html#structfield.own) or
//! [`others`](./struct.Exposure.html#structfield.others) by
//! [`total`](./struct.Exposure.html#structfield.total) in [`Exposure`](./struct.Exposure.html)).
//!
//! All entities who receive a reward have the option to choose their reward destination
//! through the [`Payee`](./struct.Payee.html) storage item (see
//! [`set_payee`](enum.Call.html#variant.set_payee)), to be one of the following:
//!
//! - Controller account, (obviously) not increasing the staked value.
//! - Stash account, not increasing the staked value.
//! - Stash account, also increasing the staked value.
//!
//! ### Additional Fund Management Operations
//!
//! Any funds already placed into stash can be the target of the following operations:
//!
//! The controller account can free a portion (or all) of the funds using the
//! [`unbond`](enum.Call.html#variant.unbond) call. Note that the funds are not immediately
//! accessible. Instead, a duration denoted by [`BondingDuration`](./struct.BondingDuration.html)
//! (in number of eras) must pass until the funds can actually be removed. Once the
//! `BondingDuration` is over, the [`withdraw_unbonded`](./enum.Call.html#variant.withdraw_unbonded)
//! call can be used to actually withdraw the funds.
//!
//! Note that there is a limitation to the number of fund-chunks that can be scheduled to be
//! unlocked in the future via [`unbond`](enum.Call.html#variant.unbond). In case this maximum
//! (`MAX_UNLOCKING_CHUNKS`) is reached, the bonded account _must_ first wait until a successful
//! call to `withdraw_unbonded` to remove some of the chunks.
//!
//! ### Election Algorithm
//!
//! The current election algorithm is implemented based on Phragmén.
//! The reference implementation can be found
//! [here](https://github.com/w3f/consensus/tree/master/NPoS).
//!
//! The election algorithm, aside from electing the validators with the most stake value and votes,
//! tries to divide the nominator votes among candidates in an equal manner. To further assure this,
//! an optional post-processing can be applied that iteratively normalizes the nominator staked
//! values until the total difference among votes of a particular nominator are less than a
//! threshold.
//!
//! ## GenesisConfig
//!
//! The Staking module depends on the [`GenesisConfig`](./struct.GenesisConfig.html).
//!
//! ## Related Modules
//!
//! - [Balances](../srml_balances/index.html): Used to manage values at stake.
//! - [Session](../srml_session/index.html): Used to manage sessions. Also, a list of new validators
//! is stored in the Session module's `Validators` at the end of each era.

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact};
use phragmen::{elect, equalize, ExtendedBalance, Support, SupportMap, ACCURACY};
use rstd::{prelude::*, result};
use session::{historical::OnSessionEnding, SelectInitialValidators};
use sr_primitives::{
    curve::PiecewiseLinear,
    traits::{
        Bounded, CheckedSub, Convert, EnsureOrigin, One, SaturatedConversion, Saturating,
        SimpleArithmetic, StaticLookup, Zero,
    },
    weights::SimpleDispatchInfo,
    Perbill,
};
#[cfg(feature = "std")]
use sr_primitives::{Deserialize, Serialize};
use sr_staking::inflation;
use sr_staking_primitives::{
    offence::{Offence, OffenceDetails, OnOffenceHandler, ReportOffence},
    SessionIndex,
};
use srml_support::{
    decl_event, decl_module, decl_storage, ensure,
    traits::{
        Currency, Get, Imbalance, LockIdentifier, LockableCurrency, OnDilution, OnFreeBalanceZero,
        OnUnbalanced, Time, WithdrawReasons,
    },
};
use system::{ensure_root, ensure_signed};

const DEFAULT_MINIMUM_VALIDATOR_COUNT: u32 = 4;
const MAX_NOMINATIONS: usize = 16;
const MAX_UNLOCKING_CHUNKS: usize = 32;
const STAKING_ID: LockIdentifier = *b"staking ";

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// Counter for the number of "reward" points earned by a given validator.
pub type Points = u32;

/// Reward points of an era. Used to split era total payout between validators.
#[derive(Encode, Decode, Default)]
pub struct EraPoints {
    /// Total number of points. Equals the sum of reward points for each validator.
    total: Points,
    /// The reward points earned by a given validator. The index of this vec corresponds to the
    /// index into the current validator set.
    individual: Vec<Points>,
}

impl EraPoints {
    /// Add the reward to the validator at the given index. Index must be valid
    /// (i.e. `index < current_elected.len()`).
    fn add_points_to_index(&mut self, index: u32, points: u32) {
        if let Some(new_total) = self.total.checked_add(points) {
            self.total = new_total;
            self.individual
                .resize((index as usize + 1).max(self.individual.len()), 0);
            self.individual[index as usize] += points; // Addition is less than total
        }
    }
}

/// Indicates the initial status of the staker.
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum StakerStatus<AccountId> {
    /// Chilling.
    Idle,
    /// Declared desire in validating or already participating in it.
    Validator,
    /// Nominating for a group of other stakers.
    Nominator(Vec<AccountId>),
}

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum RewardDestination {
    /// Pay into the stash account, increasing the amount at stake accordingly.
    Staked,
    /// Pay into the stash account, not increasing the amount at stake.
    Stash,
    /// Pay into the controller account.
    Controller,
}

impl Default for RewardDestination {
    fn default() -> Self {
        RewardDestination::Staked
    }
}

/// Preference of what happens on a slash event.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValidatorPrefs<Balance: HasCompact> {
    /// Reward that validator takes up-front; only the rest is split between themselves and
    /// nominators.
    #[codec(compact)]
    pub validator_payment: Balance,
}

impl<B: Default + HasCompact + Copy> Default for ValidatorPrefs<B> {
    fn default() -> Self {
        ValidatorPrefs {
            validator_payment: Default::default(),
        }
    }
}

/// Just a Balance/BlockNumber tuple to encode when a chunk of funds will be unlocked.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct UnlockChunk<Balance: HasCompact> {
    /// Amount of funds to be unlocked.
    #[codec(compact)]
    value: Balance,
    /// Era number at which point it'll be unlocked.
    #[codec(compact)]
    era: EraIndex,
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct StakingLedger<AccountId, Balance: HasCompact> {
    /// The stash account whose balance is actually locked and at stake.
    pub stash: AccountId,
    /// The total amount of the stash's balance that we are currently accounting for.
    /// It's just `active` plus all the `unlocking` balances.
    #[codec(compact)]
    pub total: Balance,
    /// The total amount of the stash's balance that will be at stake in any forthcoming
    /// rounds.
    #[codec(compact)]
    pub active: Balance,
    /// Any balance that is becoming free, which may eventually be transferred out
    /// of the stash (assuming it doesn't get slashed first).
    pub unlocking: Vec<UnlockChunk<Balance>>,
}

impl<AccountId, Balance: HasCompact + Copy + Saturating> StakingLedger<AccountId, Balance> {
    /// Remove entries from `unlocking` that are sufficiently old and reduce the
    /// total by the sum of their balances.
    fn consolidate_unlocked(self, current_era: EraIndex) -> Self {
        let mut total = self.total;
        let unlocking = self
            .unlocking
            .into_iter()
            .filter(|chunk| {
                if chunk.era > current_era {
                    true
                } else {
                    total = total.saturating_sub(chunk.value);
                    false
                }
            })
            .collect();
        Self {
            total,
            active: self.active,
            stash: self.stash,
            unlocking,
        }
    }
}

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct IndividualExposure<AccountId, Balance: HasCompact> {
    /// The stash account of the nominator in question.
    who: AccountId,
    /// Amount of funds exposed.
    #[codec(compact)]
    value: Balance,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Exposure<AccountId, Balance: HasCompact> {
    /// The total balance backing this validator.
    #[codec(compact)]
    pub total: Balance,
    /// The validator's own stash that is exposed.
    #[codec(compact)]
    pub own: Balance,
    /// The portions of nominators stashes that are exposed.
    pub others: Vec<IndividualExposure<AccountId, Balance>>,
}

/// A slashing event occurred, slashing a validator for a given amount of balance.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SlashJournalEntry<AccountId, Balance: HasCompact> {
    who: AccountId,
    amount: Balance,
    own_slash: Balance, // the amount of `who`'s own exposure that was slashed
}

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type PositiveImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
type MomentOf<T> = <<T as Trait>::Time as Time>::Moment;

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug)]
pub enum Compliance {
    /// Compliance requirements not met.
    Pending,
    /// KYC compliant. Eligible to participate in validation.
    Active,
}

impl Default for Compliance {
    fn default() -> Self {
        Compliance::Pending
    }
}

/// Represents a requirement that must be met to be eligible to become a validator
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PermissionedValidator {
    /// Indicates the status of KYC compliance
    pub compliance: Compliance,
}

impl Default for PermissionedValidator {
    fn default() -> Self {
        Self {
            compliance: Compliance::default(),
        }
    }
}

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `session::Trait`
pub trait SessionInterface<AccountId>: system::Trait {
    /// Disable a given validator by stash ID.
    ///
    /// Returns `true` if new era should be forced at the end of this session.
    /// This allows preventing a situation where there is too many validators
    /// disabled and block production stalls.
    fn disable_validator(validator: &AccountId) -> Result<bool, ()>;
    /// Get the validators from session.
    fn validators() -> Vec<AccountId>;
    /// Prune historical session tries up to but not including the given index.
    fn prune_historical_up_to(up_to: SessionIndex);
}

impl<T: Trait> SessionInterface<<T as system::Trait>::AccountId> for T
where
    T: session::Trait<ValidatorId = <T as system::Trait>::AccountId>,
    T: session::historical::Trait<
        FullIdentification = Exposure<<T as system::Trait>::AccountId, BalanceOf<T>>,
        FullIdentificationOf = ExposureOf<T>,
    >,
    T::SessionHandler: session::SessionHandler<<T as system::Trait>::AccountId>,
    T::OnSessionEnding: session::OnSessionEnding<<T as system::Trait>::AccountId>,
    T::SelectInitialValidators: session::SelectInitialValidators<<T as system::Trait>::AccountId>,
    T::ValidatorIdOf:
        Convert<<T as system::Trait>::AccountId, Option<<T as system::Trait>::AccountId>>,
{
    fn disable_validator(validator: &<T as system::Trait>::AccountId) -> Result<bool, ()> {
        <session::Module<T>>::disable(validator)
    }

    fn validators() -> Vec<<T as system::Trait>::AccountId> {
        <session::Module<T>>::validators()
    }

    fn prune_historical_up_to(up_to: SessionIndex) {
        <session::historical::Module<T>>::prune_up_to(up_to);
    }
}

pub trait Trait: system::Trait {
    /// The staking balance.
    type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

    /// Time used for computing era duration.
    type Time: Time;

    /// Convert a balance into a number used for election calculation.
    /// This must fit into a `u64` but is allowed to be sensibly lossy.
    /// TODO: #1377
    /// The backward convert should be removed as the new Phragmen API returns ratio.
    /// The post-processing needs it but will be moved to off-chain. TODO: #2908
    type CurrencyToVote: Convert<BalanceOf<Self>, u64> + Convert<u128, BalanceOf<Self>>;

    /// Some tokens minted.
    type OnRewardMinted: OnDilution<BalanceOf<Self>>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Handler for the unbalanced reduction when slashing a staker.
    type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// Handler for the unbalanced increment when rewarding a staker.
    type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;

    /// Number of sessions per era.
    type SessionsPerEra: Get<SessionIndex>;

    /// Number of eras that staked funds must remain bonded for.
    type BondingDuration: Get<EraIndex>;

    /// Interface for interacting with a session module.
    type SessionInterface: self::SessionInterface<Self::AccountId>;

    /// The NPoS reward curve to use.
    type RewardCurve: Get<&'static PiecewiseLinear<'static>>;

    /// Required origin for adding a potential validator (can always be Root).
    type AddOrigin: EnsureOrigin<Self::Origin>;

    /// Required origin for removing a validator (can always be Root).
    type RemoveOrigin: EnsureOrigin<Self::Origin>;

    /// Required origin for changing compliance status (can always be Root).
    type ComplianceOrigin: EnsureOrigin<Self::Origin>;
}

/// Mode of era-forcing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum Forcing {
    /// Not forcing anything - just let whatever happen.
    NotForcing,
    /// Force a new era, then reset to `NotForcing` as soon as it is done.
    ForceNew,
    /// Avoid a new era indefinitely.
    ForceNone,
}

impl Default for Forcing {
    fn default() -> Self {
        Forcing::NotForcing
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Staking {

        /// The ideal number of staking participants.
        pub ValidatorCount get(validator_count) config(): u32;
        /// Minimum number of staking participants before emergency conditions are imposed.
        pub MinimumValidatorCount get(minimum_validator_count) config():
            u32 = DEFAULT_MINIMUM_VALIDATOR_COUNT;

        /// Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
        /// easy to initialize and the performance hit is minimal (we expect no more than four
        /// invulnerables) and restricted to testnets.
        pub Invulnerables get(invulnerables) config(): Vec<T::AccountId>;

        /// Map from all locked "stash" accounts to the controller account.
        pub Bonded get(bonded): map T::AccountId => Option<T::AccountId>;
        /// Map from all (unlocked) "controller" accounts to the info regarding the staking.
        pub Ledger get(ledger):
            map T::AccountId => Option<StakingLedger<T::AccountId, BalanceOf<T>>>;

        /// Where the reward payment should be made. Keyed by stash.
        pub Payee get(payee): map T::AccountId => RewardDestination;

        /// The map from (wannabe) validator stash key to the preferences of that validator.
        pub Validators get(validators): linked_map T::AccountId => ValidatorPrefs<BalanceOf<T>>;

        /// The map from nominator stash key to the set of stash keys of all validators to nominate.
        pub Nominators get(nominators): linked_map T::AccountId => Vec<T::AccountId>;

        /// Nominators for a particular account that is in action right now. You can't iterate
        /// through validators here, but you can find them in the Session module.
        ///
        /// This is keyed by the stash account.
        pub Stakers get(stakers): map T::AccountId => Exposure<T::AccountId, BalanceOf<T>>;

        /// The currently elected validator set keyed by stash account ID.
        pub CurrentElected get(current_elected): Vec<T::AccountId>;

        /// The current era index.
        pub CurrentEra get(current_era) config(): EraIndex;

        /// The start of the current era.
        pub CurrentEraStart get(current_era_start): MomentOf<T>;

        /// The session index at which the current era started.
        pub CurrentEraStartSessionIndex get(current_era_start_session_index): SessionIndex;

        /// Rewards for the current era. Using indices of current elected set.
        CurrentEraPointsEarned get(current_era_reward): EraPoints;

        /// The amount of balance actively at stake for each validator slot, currently.
        ///
        /// This is used to derive rewards and punishments.
        pub SlotStake get(slot_stake) build(|config: &GenesisConfig<T>| {
            config.stakers.iter().map(|&(_, _, value, _)| value).min().unwrap_or_default()
        }): BalanceOf<T>;

        /// True if the next session change will be a new era regardless of index.
        pub ForceEra get(force_era) config(): Forcing;

        /// The percentage of the slash that is distributed to reporters.
        ///
        /// The rest of the slashed value is handled by the `Slash`.
        pub SlashRewardFraction get(slash_reward_fraction) config(): Perbill;

        /// A mapping from still-bonded eras to the first session index of that era.
        BondedEras: Vec<(EraIndex, SessionIndex)>;

        /// All slashes that have occurred in a given era.
        EraSlashJournal get(era_slash_journal):
            map EraIndex => Vec<SlashJournalEntry<T::AccountId, BalanceOf<T>>>;

        /// The map from (wannabe) validators to the status of compliance
        pub PermissionedValidators get(permissioned_validators):
            linked_map T::AccountId => Option<PermissionedValidator>;
    }
    add_extra_genesis {
        config(stakers):
            Vec<(T::AccountId, T::AccountId, BalanceOf<T>, StakerStatus<T::AccountId>)>;
        build(|config: &GenesisConfig<T>| {
            for &(ref stash, ref controller, balance, ref status) in &config.stakers {
                assert!(
                    T::Currency::free_balance(&stash) >= balance,
                    "Stash does not have enough balance to bond."
                );
                let _ = <Module<T>>::bond(
                    T::Origin::from(Some(stash.clone()).into()),
                    T::Lookup::unlookup(controller.clone()),
                    balance,
                    RewardDestination::Staked,
                );
                let _ = match status {
                    StakerStatus::Validator => {
                        <Module<T>>::validate(
                            T::Origin::from(Some(controller.clone()).into()),
                            Default::default(),
                        )
                    },
                    StakerStatus::Nominator(votes) => {
                        <Module<T>>::nominate(
                            T::Origin::from(Some(controller.clone()).into()),
                            votes.iter().map(|l| T::Lookup::unlookup(l.clone())).collect(),
                        )
                    }, _ => Ok(())
                };
            }
        });
    }
}

decl_event!(
    pub enum Event<T> where Balance = BalanceOf<T>, <T as system::Trait>::AccountId {
        /// All validators have been rewarded by the given balance.
        Reward(Balance),
        /// One validator (and its nominators) has been slashed by the given amount.
        Slash(AccountId, Balance),
        /// An old slashing report from a prior era was discarded because it could
        /// not be processed.
        OldSlashingReportDiscarded(SessionIndex),
		/// An entity has issued a candidacy. See the transaction for who.
		PermissionedValidatorAdded(AccountId),
		/// The given member was removed. See the transaction for who.
		PermissionedValidatorRemoved(AccountId),
		/// The given member was removed. See the transaction for who.
		PermissionedValidatorStatusChanged(AccountId),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// Number of sessions per era.
        const SessionsPerEra: SessionIndex = T::SessionsPerEra::get();

        /// Number of eras that staked funds must remain bonded for.
        const BondingDuration: EraIndex = T::BondingDuration::get();

        fn deposit_event() = default;

        fn on_finalize() {
            // Set the start of the first era.
            if !<CurrentEraStart<T>>::exists() {
                <CurrentEraStart<T>>::put(T::Time::now());
            }
        }

        /// Take the origin account as a stash and lock up `value` of its balance. `controller` will
        /// be the account that controls it.
        ///
        /// `value` must be more than the `minimum_balance` specified by `T::Currency`.
        ///
        /// The dispatch origin for this call must be _Signed_ by the stash account.
        ///
        /// # <weight>
        /// - Independent of the arguments. Moderate complexity.
        /// - O(1).
        /// - Three extra DB entries.
        ///
        /// NOTE: Two of the storage writes (`Self::bonded`, `Self::payee`) are _never_ cleaned unless
        /// the `origin` falls below _existential deposit_ and gets removed as dust.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        fn bond(origin,
            controller: <T::Lookup as StaticLookup>::Source,
            #[compact] value: BalanceOf<T>,
            payee: RewardDestination
        ) {
            let stash = ensure_signed(origin)?;

            if <Bonded<T>>::exists(&stash) {
                return Err("stash already bonded")
            }

            let controller = T::Lookup::lookup(controller)?;

            if <Ledger<T>>::exists(&controller) {
                return Err("controller already paired")
            }

            // reject a bond which is considered to be _dust_.
            if value < T::Currency::minimum_balance() {
                return Err("can not bond with value less than minimum balance")
            }

            // You're auto-bonded forever, here. We might improve this by only bonding when
            // you actually validate/nominate and remove once you unbond __everything__.
            <Bonded<T>>::insert(&stash, controller.clone());
            <Payee<T>>::insert(&stash, payee);

            let stash_balance = T::Currency::free_balance(&stash);
            let value = value.min(stash_balance);
            let item = StakingLedger { stash, total: value, active: value, unlocking: vec![] };
            Self::update_ledger(&controller, &item);
        }

        /// Add some extra amount that have appeared in the stash `free_balance` into the balance up
        /// for staking.
        ///
        /// Use this if there are additional funds in your stash account that you wish to bond.
        /// Unlike [`bond`] or [`unbond`] this function does not impose any limitation on the amount
        /// that can be added.
        ///
        /// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - O(1).
        /// - One DB entry.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        fn bond_extra(origin, #[compact] max_additional: BalanceOf<T>) {
            let stash = ensure_signed(origin)?;

            let controller = Self::bonded(&stash).ok_or("not a stash")?;
            let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;

            let stash_balance = T::Currency::free_balance(&stash);

            if let Some(extra) = stash_balance.checked_sub(&ledger.total) {
                let extra = extra.min(max_additional);
                ledger.total += extra;
                ledger.active += extra;
                Self::update_ledger(&controller, &ledger);
            }
        }

        /// Schedule a portion of the stash to be unlocked ready for transfer out after the bond
        /// period ends. If this leaves an amount actively bonded less than
        /// T::Currency::minimum_balance(), then it is increased to the full amount.
        ///
        /// Once the unlock period is done, you can call `withdraw_unbonded` to actually move
        /// the funds out of management ready for transfer.
        ///
        /// No more than a limited number of unlocking chunks (see `MAX_UNLOCKING_CHUNKS`)
        /// can co-exists at the same time. In that case, [`Call::withdraw_unbonded`] need
        /// to be called first to remove some of the chunks (if possible).
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// See also [`Call::withdraw_unbonded`].
        ///
        /// # <weight>
        /// - Independent of the arguments. Limited but potentially exploitable complexity.
        /// - Contains a limited number of reads.
        /// - Each call (requires the remainder of the bonded balance to be above `minimum_balance`)
        ///   will cause a new entry to be inserted into a vector (`Ledger.unlocking`) kept in storage.
        ///   The only way to clean the aforementioned storage item is also user-controlled via `withdraw_unbonded`.
        /// - One DB entry.
        /// </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(400_000)]
        fn unbond(origin, #[compact] value: BalanceOf<T>) {
            let controller = ensure_signed(origin)?;
            let mut ledger = Self::ledger(&controller).ok_or("not a controller")?;
            ensure!(
                ledger.unlocking.len() < MAX_UNLOCKING_CHUNKS,
                "can not schedule more unlock chunks"
            );

            let mut value = value.min(ledger.active);

            if !value.is_zero() {
                ledger.active -= value;

                // Avoid there being a dust balance left in the staking system.
                if ledger.active < T::Currency::minimum_balance() {
                    value += ledger.active;
                    ledger.active = Zero::zero();
                }

                let era = Self::current_era() + T::BondingDuration::get();
                ledger.unlocking.push(UnlockChunk { value, era });
                Self::update_ledger(&controller, &ledger);
            }
        }

        /// Remove any unlocked chunks from the `unlocking` queue from our management.
        ///
        /// This essentially frees up that balance to be used by the stash account to do
        /// whatever it wants.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// See also [`Call::unbond`].
        ///
        /// # <weight>
        /// - Could be dependent on the `origin` argument and how much `unlocking` chunks exist.
        ///  It implies `consolidate_unlocked` which loops over `Ledger.unlocking`, which is
        ///  indirectly user-controlled. See [`unbond`] for more detail.
        /// - Contains a limited number of reads, yet the size of which could be large based on `ledger`.
        /// - Writes are limited to the `origin` account key.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(400_000)]
        fn withdraw_unbonded(origin) {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let ledger = ledger.consolidate_unlocked(Self::current_era());

            if ledger.unlocking.is_empty() && ledger.active.is_zero() {
                // This account must have called `unbond()` with some value that caused the active
                // portion to fall below existential deposit + will have no more unlocking chunks
                // left. We can now safely remove this.
                let stash = ledger.stash;
                // remove the lock.
                T::Currency::remove_lock(STAKING_ID, &stash);
                // remove all staking-related information.
                Self::kill_stash(&stash);
            } else {
                // This was the consequence of a partial unbond. just update the ledger and move on.
                Self::update_ledger(&controller, &ledger);
            }
        }

        /// Declare the desire to validate for the origin controller.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains a limited number of reads.
        /// - Writes are limited to the `origin` account key.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(750_000)]
        fn validate(origin, prefs: ValidatorPrefs<BalanceOf<T>>) {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let stash = &ledger.stash;
            ensure!(Self::is_controller_eligible(&controller), "controller has not passed compliance");
            <Nominators<T>>::remove(stash);
            <Validators<T>>::insert(stash, prefs);
        }

        /// Declare the desire to nominate `targets` for the origin controller.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// # <weight>
        /// - The transaction's complexity is proportional to the size of `targets`,
        /// which is capped at `MAX_NOMINATIONS`.
        /// - Both the reads and writes follow a similar pattern.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(750_000)]
        fn nominate(origin, targets: Vec<<T::Lookup as StaticLookup>::Source>) {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let stash = &ledger.stash;
            ensure!(!targets.is_empty(), "targets cannot be empty");
            let targets = targets.into_iter()
                .take(MAX_NOMINATIONS)
                .map(|t| T::Lookup::lookup(t))
                .collect::<result::Result<Vec<T::AccountId>, _>>()?;

            <Validators<T>>::remove(stash);
            <Nominators<T>>::insert(stash, targets);
        }

        /// Declare no desire to either validate or nominate.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains one read.
        /// - Writes are limited to the `origin` account key.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        fn chill(origin) {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let stash = &ledger.stash;
            <Validators<T>>::remove(stash);
            <Nominators<T>>::remove(stash);
        }

        /// (Re-)set the payment target for a controller.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains a limited number of reads.
        /// - Writes are limited to the `origin` account key.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        fn set_payee(origin, payee: RewardDestination) {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or("not a controller")?;
            let stash = &ledger.stash;
            <Payee<T>>::insert(stash, payee);
        }

        /// (Re-)set the controller of a stash.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains a limited number of reads.
        /// - Writes are limited to the `origin` account key.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(750_000)]
        fn set_controller(origin, controller: <T::Lookup as StaticLookup>::Source) {
            let stash = ensure_signed(origin)?;
            let old_controller = Self::bonded(&stash).ok_or("not a stash")?;
            let controller = T::Lookup::lookup(controller)?;
            if <Ledger<T>>::exists(&controller) {
                return Err("controller already paired")
            }
            if controller != old_controller {
                <Bonded<T>>::insert(&stash, &controller);
                if let Some(l) = <Ledger<T>>::take(&old_controller) {
                    <Ledger<T>>::insert(&controller, l);
                }
            }
        }

        /// The ideal number of validators.
        #[weight = SimpleDispatchInfo::FixedOperational(150_000)]
        fn set_validator_count(origin, #[compact] new: u32) {
            ensure_root(origin)?;
            ValidatorCount::put(new);
        }

        /// Governance committee on 2/3 rds majority can introduce a new potential validator
        /// to the pool of validators. Staking module uses `PermissionedValidators` to ensure
        /// validators have completed KYB compliance and considers them for validation.
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        fn add_potential_validator(origin, controller: T::AccountId) {
            T::AddOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;

            ensure!(!<PermissionedValidators<T>>::exists(&controller),
            "account already exists in permissioned_validators");

            <PermissionedValidators<T>>::insert(&controller, PermissionedValidator {
                compliance: Compliance::Active
            });

            Self::deposit_event(RawEvent::PermissionedValidatorAdded(controller));
        }

        /// Remove a validator from the pool of validators. Effects are known in the next session.
        /// Staking module checks `PermissionedValidators` to ensure validators have
        /// completed KYB compliance
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        fn remove_validator(origin, controller: T::AccountId) {
            T::RemoveOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;

            ensure!(<PermissionedValidators<T>>::exists(&controller),
            "account doesn't exist in permissioned_validators");

            <PermissionedValidators<T>>::remove(&controller);

            Self::deposit_event(RawEvent::PermissionedValidatorAdded(controller));
        }

        /// Governance committee on 2/3 rds majority can update the compliance status of a validator
        /// as `Pending`.
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        fn compliance_failed(origin, controller: T::AccountId) {
            T::ComplianceOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;

            ensure!(<PermissionedValidators<T>>::exists(&controller),
            "acount doesn't exist in permissioned_validators");

            <PermissionedValidators<T>>::mutate(&controller, |entry| {
                if let Some(validator) = entry {
                    validator.compliance = Compliance::Pending
                }
            });
            Self::deposit_event(RawEvent::PermissionedValidatorStatusChanged(controller));
        }

        /// Governance committee on 2/3 rds majority can update the compliance status of a validator
        /// as `Active`.
        #[weight = SimpleDispatchInfo::FixedNormal(50_000)]
        fn compliance_passed(origin, controller: T::AccountId) {
            T::ComplianceOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;

            ensure!(<PermissionedValidators<T>>::exists(&controller),
            "acount doesn't exist in permissioned_validators");

            <PermissionedValidators<T>>::mutate(&controller, |entry| {
                if let Some(validator) = entry {
                    validator.compliance = Compliance::Active
                }
            });
            Self::deposit_event(RawEvent::PermissionedValidatorStatusChanged(controller));
        }

        // ----- Root calls.

        /// Force there to be no new eras indefinitely.
        ///
        /// # <weight>
        /// - No arguments.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedOperational(10_000)]
        fn force_no_eras(origin) {
            ensure_root(origin)?;
            ForceEra::put(Forcing::ForceNone);
        }

        /// Force there to be a new era at the end of the next session. After this, it will be
        /// reset to normal (non-forced) behaviour.
        ///
        /// # <weight>
        /// - No arguments.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedOperational(10_000)]
        fn force_new_era(origin) {
            ensure_root(origin)?;
            ForceEra::put(Forcing::ForceNew);
        }

        /// Set the validators who cannot be slashed (if any).
        #[weight = SimpleDispatchInfo::FixedOperational(10_000)]
        fn set_invulnerables(origin, validators: Vec<T::AccountId>) {
            ensure_root(origin)?;
            <Invulnerables<T>>::put(validators);
        }
    }
}

impl<T: Trait> Module<T> {
    // PUBLIC IMMUTABLES

    /// The total balance that can be slashed from a validator controller account as of
    /// right now.
    pub fn slashable_balance_of(stash: &T::AccountId) -> BalanceOf<T> {
        Self::bonded(stash)
            .and_then(Self::ledger)
            .map(|l| l.active)
            .unwrap_or_default()
    }

    // MUTABLES (DANGEROUS)

    /// Update the ledger for a controller. This will also update the stash lock. The lock will
    /// will lock the entire funds except paying for further transactions.
    fn update_ledger(
        controller: &T::AccountId,
        ledger: &StakingLedger<T::AccountId, BalanceOf<T>>,
    ) {
        T::Currency::set_lock(
            STAKING_ID,
            &ledger.stash,
            ledger.total,
            T::BlockNumber::max_value(),
            WithdrawReasons::all(),
        );
        <Ledger<T>>::insert(controller, ledger);
    }

    /// Slash a given validator by a specific amount with given (historical) exposure.
    ///
    /// Removes the slash from the validator's balance by preference,
    /// and reduces the nominators' balance if needed.
    ///
    /// Returns the resulting `NegativeImbalance` to allow distributing the slashed amount and
    /// pushes an entry onto the slash journal.
    fn slash_validator(
        stash: &T::AccountId,
        slash: BalanceOf<T>,
        exposure: &Exposure<T::AccountId, BalanceOf<T>>,
        journal: &mut Vec<SlashJournalEntry<T::AccountId, BalanceOf<T>>>,
    ) -> NegativeImbalanceOf<T> {
        // The amount we are actually going to slash (can't be bigger than the validator's total
        // exposure)
        let slash = slash.min(exposure.total);

        // limit what we'll slash of the stash's own to only what's in
        // the exposure.
        //
        // note: this is fine only because we limit reports of the current era.
        // otherwise, these funds may have already been slashed due to something
        // reported from a prior era.
        let already_slashed_own = journal
            .iter()
            .filter(|entry| &entry.who == stash)
            .map(|entry| entry.own_slash)
            .fold(<BalanceOf<T>>::zero(), |a, c| a.saturating_add(c));

        let own_remaining = exposure.own.saturating_sub(already_slashed_own);

        // The amount we'll slash from the validator's stash directly.
        let own_slash = own_remaining.min(slash);
        let (mut imbalance, missing) = T::Currency::slash(stash, own_slash);
        let own_slash = own_slash - missing;
        // The amount remaining that we can't slash from the validator,
        // that must be taken from the nominators.
        let rest_slash = slash - own_slash;
        if !rest_slash.is_zero() {
            // The total to be slashed from the nominators.
            let total = exposure.total - exposure.own;
            if !total.is_zero() {
                for i in exposure.others.iter() {
                    let per_u64 = Perbill::from_rational_approximation(i.value, total);
                    // best effort - not much that can be done on fail.
                    imbalance.subsume(T::Currency::slash(&i.who, per_u64 * rest_slash).0)
                }
            }
        }

        journal.push(SlashJournalEntry {
            who: stash.clone(),
            own_slash: own_slash.clone(),
            amount: slash,
        });

        // trigger the event
        Self::deposit_event(RawEvent::Slash(stash.clone(), slash));

        imbalance
    }

    /// Actually make a payment to a staker. This uses the currency's reward function
    /// to pay the right payee for the given staker account.
    fn make_payout(stash: &T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
        let dest = Self::payee(stash);
        match dest {
            RewardDestination::Controller => Self::bonded(stash).and_then(|controller| {
                T::Currency::deposit_into_existing(&controller, amount).ok()
            }),
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
        }
    }

    /// Reward a given validator by a specific amount. Add the reward to the validator's, and its
    /// nominators' balance, pro-rata based on their exposure, after having removed the validator's
    /// pre-payout cut.
    fn reward_validator(stash: &T::AccountId, reward: BalanceOf<T>) -> PositiveImbalanceOf<T> {
        let off_the_table = reward.min(Self::validators(stash).validator_payment);
        let reward = reward - off_the_table;
        let mut imbalance = <PositiveImbalanceOf<T>>::zero();
        let validator_cut = if reward.is_zero() {
            Zero::zero()
        } else {
            let exposure = Self::stakers(stash);
            let total = exposure.total.max(One::one());

            for i in &exposure.others {
                let per_u64 = Perbill::from_rational_approximation(i.value, total);
                imbalance.maybe_subsume(Self::make_payout(&i.who, per_u64 * reward));
            }

            let per_u64 = Perbill::from_rational_approximation(exposure.own, total);
            per_u64 * reward
        };

        imbalance.maybe_subsume(Self::make_payout(stash, validator_cut + off_the_table));

        imbalance
    }

    /// Session has just ended. Provide the validator set for the next session if it's an era-end, along
    /// with the exposure of the prior validator set.
    fn new_session(
        session_index: SessionIndex,
    ) -> Option<(
        Vec<T::AccountId>,
        Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)>,
    )> {
        let era_length = session_index
            .checked_sub(Self::current_era_start_session_index())
            .unwrap_or(0);
        match ForceEra::get() {
            Forcing::ForceNew => ForceEra::kill(),
            Forcing::NotForcing if era_length >= T::SessionsPerEra::get() => (),
            _ => return None,
        }

        Self::refresh_compliance_statuses();

        let validators = T::SessionInterface::validators();
        let prior = validators
            .into_iter()
            .map(|v| {
                let e = Self::stakers(&v);
                (v, e)
            })
            .collect();

        Self::new_era(session_index).map(move |new| (new, prior))
    }

    /// The era has changed - enact new staking set.
    ///
    /// NOTE: This always happens immediately before a session change to ensure that new validators
    /// get a chance to set their session keys.
    fn new_era(start_session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        // Payout
        let points = CurrentEraPointsEarned::take();
        let now = T::Time::now();
        let previous_era_start = <CurrentEraStart<T>>::mutate(|v| rstd::mem::replace(v, now));
        let era_duration = now - previous_era_start;
        if !era_duration.is_zero() {
            let validators = Self::current_elected();

            let validator_len: BalanceOf<T> = (validators.len() as u32).into();
            let total_rewarded_stake = Self::slot_stake() * validator_len;

            let total_payout = inflation::compute_total_payout(
                &T::RewardCurve::get(),
                total_rewarded_stake.clone(),
                T::Currency::total_issuance(),
                // Duration of era; more than u64::MAX is rewarded as u64::MAX.
                era_duration.saturated_into::<u64>(),
            );

            let mut total_imbalance = <PositiveImbalanceOf<T>>::zero();

            for (v, p) in validators.iter().zip(points.individual.into_iter()) {
                if p != 0 {
                    let reward = multiply_by_rational(total_payout, p, points.total);
                    total_imbalance.subsume(Self::reward_validator(v, reward));
                }
            }

            let total_reward = total_imbalance.peek();
            // assert!(total_reward <= total_payout)

            Self::deposit_event(RawEvent::Reward(total_reward));
            T::Reward::on_unbalanced(total_imbalance);
            T::OnRewardMinted::on_dilution(total_reward, total_rewarded_stake);
        }

        // Increment current era.
        let current_era = CurrentEra::mutate(|s| {
            *s += 1;
            *s
        });

        // prune journal for last era.
        <EraSlashJournal<T>>::remove(current_era - 1);

        CurrentEraStartSessionIndex::mutate(|v| {
            *v = start_session_index;
        });
        let bonding_duration = T::BondingDuration::get();

        if current_era > bonding_duration {
            let first_kept = current_era - bonding_duration;
            BondedEras::mutate(|bonded| {
                bonded.push((current_era, start_session_index));

                // prune out everything that's from before the first-kept index.
                let n_to_prune = bonded
                    .iter()
                    .take_while(|&&(era_idx, _)| era_idx < first_kept)
                    .count();

                bonded.drain(..n_to_prune);

                if let Some(&(_, first_session)) = bonded.first() {
                    T::SessionInterface::prune_historical_up_to(first_session);
                }
            })
        }

        // Reassign all Stakers.
        let (_slot_stake, maybe_new_validators) = Self::select_validators();

        maybe_new_validators
    }

    /// Select a new validator set from the assembled stakers and their role preferences.
    ///
    /// Returns the new `SlotStake` value and a set of newly selected _stash_ IDs.
    fn select_validators() -> (BalanceOf<T>, Option<Vec<T::AccountId>>) {
        let maybe_phragmen_result = elect::<_, _, _, T::CurrencyToVote>(
            Self::validator_count() as usize,
            Self::minimum_validator_count().max(1) as usize,
            <Validators<T>>::enumerate()
                .map(|(who, _)| who)
                .filter(|stash| Self::is_stash_eligible(stash))
                .collect::<Vec<T::AccountId>>(),
            <Nominators<T>>::enumerate().collect(),
            Self::slashable_balance_of,
            true,
        );

        if let Some(phragmen_result) = maybe_phragmen_result {
            let elected_stashes = phragmen_result
                .winners
                .iter()
                .map(|(s, _)| s.clone())
                .collect::<Vec<T::AccountId>>();
            let mut assignments = phragmen_result.assignments;

            // helper closure.
            let to_balance = |b: ExtendedBalance| {
                <T::CurrencyToVote as Convert<ExtendedBalance, BalanceOf<T>>>::convert(b)
            };
            let to_votes = |b: BalanceOf<T>| {
                <T::CurrencyToVote as Convert<BalanceOf<T>, u64>>::convert(b) as ExtendedBalance
            };

            // The return value of this is safe to be converted to u64.
            // The original balance, `b` is within the scope of u64. It is just extended to u128
            // to be properly multiplied by a ratio, which will lead to another value
            // less than u64 for sure. The result can then be safely passed to `to_balance`.
            // For now the backward convert is used. A simple `TryFrom<u64>` is also safe.
            let ratio_of = |b, r: ExtendedBalance| r.saturating_mul(to_votes(b)) / ACCURACY;

            // Initialize the support of each candidate.
            let mut supports = <SupportMap<T::AccountId>>::new();
            elected_stashes
                .iter()
                .map(|e| (e, to_votes(Self::slashable_balance_of(e))))
                .for_each(|(e, s)| {
                    let item = Support {
                        own: s,
                        total: s,
                        ..Default::default()
                    };
                    supports.insert(e.clone(), item);
                });

            // convert the ratio in-place (and replace) to the balance but still in the extended
            // balance type.
            for (n, assignment) in assignments.iter_mut() {
                for (c, r) in assignment.iter_mut() {
                    let nominator_stake = Self::slashable_balance_of(n);
                    let other_stake = ratio_of(nominator_stake, *r);
                    if let Some(support) = supports.get_mut(c) {
                        // This for an astronomically rich validator with more astronomically rich
                        // set of nominators, this might saturate.
                        support.total = support.total.saturating_add(other_stake);
                        support.others.push((n.clone(), other_stake));
                    }
                    // convert the ratio to extended balance
                    *r = other_stake;
                }
            }

            #[cfg(feature = "equalize")]
            {
                let tolerance = 0_u128;
                let iterations = 2_usize;
                equalize::<_, _, _, T::CurrencyToVote>(
                    assignments,
                    &mut supports,
                    tolerance,
                    iterations,
                    Self::slashable_balance_of,
                );
            }

            // Clear Stakers.
            for v in Self::current_elected().iter() {
                <Stakers<T>>::remove(v);
            }

            // Populate Stakers and figure out the minimum stake behind a slot.
            let mut slot_stake = BalanceOf::<T>::max_value();
            for (c, s) in supports.into_iter() {
                // build `struct exposure` from `support`
                let exposure = Exposure {
                    own: to_balance(s.own),
                    // This might reasonably saturate and we cannot do much about it. The sum of
                    // someone's stake might exceed the balance type if they have the maximum amount
                    // of balance and receive some support. This is super unlikely to happen, yet
                    // we simulate it in some tests.
                    total: to_balance(s.total),
                    others: s
                        .others
                        .into_iter()
                        .map(|(who, value)| IndividualExposure {
                            who,
                            value: to_balance(value),
                        })
                        .collect::<Vec<IndividualExposure<_, _>>>(),
                };
                if exposure.total < slot_stake {
                    slot_stake = exposure.total;
                }
                <Stakers<T>>::insert(c.clone(), exposure.clone());
            }

            // Update slot stake.
            <SlotStake<T>>::put(&slot_stake);

            // Set the new validator set in sessions.
            <CurrentElected<T>>::put(&elected_stashes);

            // In order to keep the property required by `n_session_ending`
            // that we must return the new validator set even if it's the same as the old,
            // as long as any underlying economic conditions have changed, we don't attempt
            // to do any optimization where we compare against the prior set.
            (slot_stake, Some(elected_stashes))
        } else {
            // There were not enough candidates for even our minimal level of functionality.
            // This is bad.
            // We should probably disable all functionality except for block production
            // and let the chain keep producing blocks until we can decide on a sufficiently
            // substantial set.
            // TODO: #2494
            (Self::slot_stake(), None)
        }
    }

    /// Remove all associated data of a stash account from the staking system.
    ///
    /// This is called :
    /// - Immediately when an account's balance falls below existential deposit.
    /// - after a `withdraw_unbond()` call that frees all of a stash's bonded balance.
    fn kill_stash(stash: &T::AccountId) {
        if let Some(controller) = <Bonded<T>>::take(stash) {
            <Ledger<T>>::remove(&controller);
        }
        <Payee<T>>::remove(stash);
        <Validators<T>>::remove(stash);
        <Nominators<T>>::remove(stash);
    }

    /// Add reward points to validators using their stash account ID.
    ///
    /// Validators are keyed by stash account ID and must be in the current elected set.
    ///
    /// For each element in the iterator the given number of points in u32 is added to the
    /// validator, thus duplicates are handled.
    ///
    /// At the end of the era each the total payout will be distributed among validator
    /// relatively to their points.
    ///
    /// COMPLEXITY: Complexity is `number_of_validator_to_reward x current_elected_len`.
    /// If you need to reward lots of validator consider using `reward_by_indices`.
    pub fn reward_by_ids(validators_points: impl IntoIterator<Item = (T::AccountId, u32)>) {
        CurrentEraPointsEarned::mutate(|rewards| {
            let current_elected = <Module<T>>::current_elected();
            for (validator, points) in validators_points.into_iter() {
                if let Some(index) = current_elected
                    .iter()
                    .position(|elected| *elected == validator)
                {
                    rewards.add_points_to_index(index as u32, points);
                }
            }
        });
    }

    /// Add reward points to validators using their validator index.
    ///
    /// For each element in the iterator the given number of points in u32 is added to the
    /// validator, thus duplicates are handled.
    pub fn reward_by_indices(validators_points: impl IntoIterator<Item = (u32, u32)>) {
        // TODO: This can be optimised once #3302 is implemented.
        let current_elected_len = <Module<T>>::current_elected().len() as u32;

        CurrentEraPointsEarned::mutate(|rewards| {
            for (validator_index, points) in validators_points.into_iter() {
                if validator_index < current_elected_len {
                    rewards.add_points_to_index(validator_index, points);
                }
            }
        });
    }

    /// Does the given account id have compliance status `Active`
    fn is_controller_eligible(account_id: &T::AccountId) -> bool {
        if let Some(validator) = Self::permissioned_validators(account_id) {
            validator.compliance == Compliance::Active
        } else {
            false
        }
    }

    /// Checks KYC compliance status of a controller associated with the stash
    fn is_stash_eligible(stash: &T::AccountId) -> bool {
        if let Some(controller) = <Bonded<T>>::take(stash) {
            Self::is_controller_eligible(&controller)
        } else {
            false
        }
    }

    /// Non-deterministic method that checks KYC status of each validator and persists
    /// any changes to compliance status.
    fn refresh_compliance_statuses() {
        let accounts = <PermissionedValidators<T>>::enumerate()
            .map(|(who, _)| who)
            .collect::<Vec<T::AccountId>>();

        for account in accounts {
            <PermissionedValidators<T>>::mutate(account.clone(), |v| {
                if let Some(validator) = v {
                    validator.compliance = if Self::is_validator_compliant(&account) {
                        Compliance::Active
                    } else {
                        Compliance::Pending
                    };
                }
            });
        }
    }

    /// Is the stash account one of the permissioned validators?
    pub fn is_validator_compliant(_controller: &T::AccountId) -> bool {
        //TODO: Get DID associated with controller and check they have a KYB attestation etc.
        true
    }
}

impl<T: Trait> session::OnSessionEnding<T::AccountId> for Module<T> {
    fn on_session_ending(
        _ending: SessionIndex,
        start_session: SessionIndex,
    ) -> Option<Vec<T::AccountId>> {
        Self::new_session(start_session - 1).map(|(new, _old)| new)
    }
}

impl<T: Trait> OnSessionEnding<T::AccountId, Exposure<T::AccountId, BalanceOf<T>>> for Module<T> {
    fn on_session_ending(
        _ending: SessionIndex,
        start_session: SessionIndex,
    ) -> Option<(
        Vec<T::AccountId>,
        Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)>,
    )> {
        Self::new_session(start_session - 1)
    }
}

impl<T: Trait> OnFreeBalanceZero<T::AccountId> for Module<T> {
    fn on_free_balance_zero(stash: &T::AccountId) {
        Self::kill_stash(stash);
    }
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a (non-uncle) block in the relay chain,
/// * 2 points to the block producer for each reference to a previously unreferenced uncle, and
/// * 1 point to the producer of each referenced uncle block.
impl<T: Trait + authorship::Trait> authorship::EventHandler<T::AccountId, T::BlockNumber>
    for Module<T>
{
    fn note_author(author: T::AccountId) {
        Self::reward_by_ids(vec![(author, 20)]);
    }
    fn note_uncle(author: T::AccountId, _age: T::BlockNumber) {
        Self::reward_by_ids(vec![(<authorship::Module<T>>::author(), 2), (author, 1)])
    }
}

// This is guarantee not to overflow on whatever values.
// `num` must be inferior to `den` otherwise it will be reduce to `den`.
fn multiply_by_rational<N>(value: N, num: u32, den: u32) -> N
where
    N: SimpleArithmetic + Clone,
{
    let num = num.min(den);

    let result_divisor_part = value.clone() / den.into() * num.into();

    let result_remainder_part = {
        let rem = value % den.into();

        // Fits into u32 because den is u32 and remainder < den
        let rem_u32 = rem.saturated_into::<u32>();

        // Multiplication fits into u64 as both term are u32
        let rem_part = rem_u32 as u64 * num as u64 / den as u64;

        // Result fits into u32 as num < total_points
        (rem_part as u32).into()
    };

    result_divisor_part + result_remainder_part
}

/// A `Convert` implementation that finds the stash of the given controller account,
/// if any.
pub struct StashOf<T>(rstd::marker::PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<T::AccountId>> for StashOf<T> {
    fn convert(controller: T::AccountId) -> Option<T::AccountId> {
        <Module<T>>::ledger(&controller).map(|l| l.stash)
    }
}

/// A typed conversion from stash account ID to the current exposure of nominators
/// on that account.
pub struct ExposureOf<T>(rstd::marker::PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<Exposure<T::AccountId, BalanceOf<T>>>>
    for ExposureOf<T>
{
    fn convert(validator: T::AccountId) -> Option<Exposure<T::AccountId, BalanceOf<T>>> {
        Some(<Module<T>>::stakers(&validator))
    }
}

impl<T: Trait> SelectInitialValidators<T::AccountId> for Module<T> {
    fn select_initial_validators() -> Option<Vec<T::AccountId>> {
        <Module<T>>::select_validators().1
    }
}

/// This is intended to be used with `FilterHistoricalOffences`.
impl<T: Trait> OnOffenceHandler<T::AccountId, session::historical::IdentificationTuple<T>>
    for Module<T>
where
    T: session::Trait<ValidatorId = <T as system::Trait>::AccountId>,
    T: session::historical::Trait<
        FullIdentification = Exposure<<T as system::Trait>::AccountId, BalanceOf<T>>,
        FullIdentificationOf = ExposureOf<T>,
    >,
    T::SessionHandler: session::SessionHandler<<T as system::Trait>::AccountId>,
    T::OnSessionEnding: session::OnSessionEnding<<T as system::Trait>::AccountId>,
    T::SelectInitialValidators: session::SelectInitialValidators<<T as system::Trait>::AccountId>,
    T::ValidatorIdOf:
        Convert<<T as system::Trait>::AccountId, Option<<T as system::Trait>::AccountId>>,
{
    fn on_offence(
        offenders: &[OffenceDetails<T::AccountId, session::historical::IdentificationTuple<T>>],
        slash_fraction: &[Perbill],
    ) {
        let mut remaining_imbalance = <NegativeImbalanceOf<T>>::zero();
        let slash_reward_fraction = SlashRewardFraction::get();

        let era_now = Self::current_era();
        let mut journal = Self::era_slash_journal(era_now);
        for (details, slash_fraction) in offenders.iter().zip(slash_fraction) {
            let stash = &details.offender.0;
            let exposure = &details.offender.1;

            // Skip if the validator is invulnerable.
            if Self::invulnerables().contains(stash) {
                continue;
            }

            // calculate the amount to slash
            let slash_exposure = exposure.total;
            let amount = *slash_fraction * slash_exposure;
            // in some cases `slash_fraction` can be just `0`,
            // which means we are not slashing this time.
            if amount.is_zero() {
                continue;
            }

            // make sure to disable validator till the end of this session
            if T::SessionInterface::disable_validator(stash).unwrap_or(false) {
                // force a new era, to select a new validator set
                ForceEra::put(Forcing::ForceNew);
            }
            // actually slash the validator
            let slashed_amount = Self::slash_validator(stash, amount, exposure, &mut journal);

            // distribute the rewards according to the slash
            let slash_reward = slash_reward_fraction * slashed_amount.peek();
            if !slash_reward.is_zero() && !details.reporters.is_empty() {
                let (mut reward, rest) = slashed_amount.split(slash_reward);
                // split the reward between reporters equally. Division cannot fail because
                // we guarded against it in the enclosing if.
                let per_reporter = reward.peek() / (details.reporters.len() as u32).into();
                for reporter in &details.reporters {
                    let (reporter_reward, rest) = reward.split(per_reporter);
                    reward = rest;
                    T::Currency::resolve_creating(reporter, reporter_reward);
                }
                // The rest goes to the treasury.
                remaining_imbalance.subsume(reward);
                remaining_imbalance.subsume(rest);
            } else {
                remaining_imbalance.subsume(slashed_amount);
            }
        }
        <EraSlashJournal<T>>::insert(era_now, journal);

        // Handle the rest of imbalances
        T::Slash::on_unbalanced(remaining_imbalance);
    }
}

/// Filter historical offences out and only allow those from the current era.
pub struct FilterHistoricalOffences<T, R> {
    _inner: rstd::marker::PhantomData<(T, R)>,
}

impl<T, Reporter, Offender, R, O> ReportOffence<Reporter, Offender, O>
    for FilterHistoricalOffences<Module<T>, R>
where
    T: Trait,
    R: ReportOffence<Reporter, Offender, O>,
    O: Offence<Offender>,
{
    fn report_offence(reporters: Vec<Reporter>, offence: O) {
        // disallow any slashing from before the current era.
        let offence_session = offence.session_index();
        if offence_session >= <Module<T>>::current_era_start_session_index() {
            R::report_offence(reporters, offence)
        } else {
            <Module<T>>::deposit_event(RawEvent::OldSlashingReportDiscarded(offence_session))
        }
    }
}

/// Tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::balances;
    use crate::identity;
    use std::{cell::RefCell, collections::HashSet};

    use sr_io::{with_externalities, TestExternalities};
    use sr_primitives::{
        testing::{Header, UintAuthorityId},
        traits::{
            BlakeTwo256, Convert, ConvertInto, IdentityLookup, OnInitialize, OpaqueKeys,
            SaturatedConversion,
        },
        Perbill,
    };
    use srml_support::traits::FindAuthor;
    use srml_support::{
        assert_ok,
        dispatch::{DispatchError, DispatchResult},
        impl_outer_origin, parameter_types,
    };
    use substrate_primitives::{Blake2Hasher, H256};
    use system::EnsureSignedBy;

    /// Mock types for testing
    /// The AccountId alias in this test module.
    pub type AccountId = u64;
    pub type BlockNumber = u64;
    pub type Balance = u128;

    pub struct CurrencyToVoteHandler;
    impl Convert<u64, u64> for CurrencyToVoteHandler {
        fn convert(x: u64) -> u64 {
            x
        }
    }
    impl Convert<u128, u64> for CurrencyToVoteHandler {
        fn convert(x: u128) -> u64 {
            x.saturated_into()
        }
    }
    impl Convert<u128, u128> for CurrencyToVoteHandler {
        fn convert(x: u128) -> u128 {
            x
        }
    }

    thread_local! {
        static SESSION: RefCell<(Vec<AccountId>, HashSet<AccountId>)> = RefCell::new(Default::default());
        static EXISTENTIAL_DEPOSIT: RefCell<u64> = RefCell::new(0);
    }

    pub struct TestSessionHandler;
    impl session::SessionHandler<AccountId> for TestSessionHandler {
        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AccountId, Ks)]) {}

        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            validators: &[(AccountId, Ks)],
            _queued_validators: &[(AccountId, Ks)],
        ) {
            SESSION.with(|x| {
                *x.borrow_mut() = (
                    validators.iter().map(|x| x.0.clone()).collect(),
                    HashSet::new(),
                )
            });
        }

        fn on_disabled(validator_index: usize) {
            SESSION.with(|d| {
                let mut d = d.borrow_mut();
                let value = d.0[validator_index];
                d.1.insert(value);
            })
        }
    }

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    /// Author of block is always 11
    pub struct Author11;
    impl FindAuthor<u64> for Author11 {
        fn find_author<'a, I>(_digests: I) -> Option<u64>
        where
            I: 'a + IntoIterator<Item = (srml_support::ConsensusEngineId, &'a [u8])>,
        {
            Some(11)
        }
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: u32 = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
    }

    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }

    parameter_types! {
        pub const ExistentialDeposit: u64 = 0;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
        pub const TransactionBaseFee: u64 = 0;
        pub const TransactionByteFee: u64 = 0;
    }

    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type DustRemoval = ();
        type TransferPayment = ();

        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type TransactionBaseFee = TransactionBaseFee;
        type TransactionByteFee = TransactionByteFee;
        type WeightToFee = ConvertInto;
        type Identity = identity::Module<Test>;
    }

    #[derive(codec::Encode, codec::Decode, Debug, Clone, Eq, PartialEq)]
    pub struct IdentityProposal {
        pub dummy: u8,
    }

    impl sr_primitives::traits::Dispatchable for IdentityProposal {
        type Origin = Origin;
        type Trait = Test;
        type Error = DispatchError;

        fn dispatch(self, _origin: Self::Origin) -> DispatchResult<Self::Error> {
            Ok(())
        }
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = IdentityProposal;
    }

    parameter_types! {
        pub const Period: BlockNumber = 1;
        pub const Offset: BlockNumber = 0;
        pub const UncleGenerations: u64 = 0;
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    }

    impl session::Trait for Test {
        type OnSessionEnding = session::historical::NoteHistoricalRoot<Test, Staking>;
        type Keys = UintAuthorityId;
        type ShouldEndSession = session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AccountId;
        type ValidatorIdOf = self::StashOf<Test>;
        type SelectInitialValidators = Staking;
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl session::historical::Trait for Test {
        type FullIdentification = crate::staking::Exposure<AccountId, Balance>;
        type FullIdentificationOf = crate::staking::ExposureOf<Test>;
    }

    impl authorship::Trait for Test {
        type FindAuthor = Author11;
        type UncleGenerations = UncleGenerations;
        type FilterUncle = ();
        type EventHandler = Module<Test>;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    srml_staking_reward_curve::build! {
        const I_NPOS: PiecewiseLinear<'static> = curve!(
            min_inflation: 0_025_000,
            max_inflation: 0_100_000,
            ideal_stake: 0_500_000,
            falloff: 0_050_000,
            max_piece_count: 40,
            test_precision: 0_005_000,
        );
    }

    parameter_types! {
        pub const SessionsPerEra: SessionIndex = 3;
        pub const BondingDuration: EraIndex = 3;
        pub const RewardCurve: &'static PiecewiseLinear<'static> = &I_NPOS;
        pub const One: u64 = 1;
        pub const Two: u64 = 2;
        pub const Three: u64 = 3;
        pub const Four: u64 = 4;
        pub const Five: u64 = 5;
    }

    impl super::Trait for Test {
        type Currency = balances::Module<Self>;
        type Time = timestamp::Module<Self>;
        type CurrencyToVote = CurrencyToVoteHandler;
        type OnRewardMinted = ();
        type Event = ();
        type Slash = ();
        type Reward = ();
        type SessionsPerEra = SessionsPerEra;
        type BondingDuration = BondingDuration;
        type SessionInterface = Self;
        type RewardCurve = RewardCurve;
        type AddOrigin = EnsureSignedBy<One, u64>;
        type RemoveOrigin = EnsureSignedBy<Two, u64>;
        type ComplianceOrigin = EnsureSignedBy<Three, u64>;
    }

    type Staking = super::Module<Test>;

    pub struct ExtBuilder {
        existential_deposit: u64,
        validator_pool: bool,
        nominate: bool,
        validator_count: u32,
        minimum_validator_count: u32,
        fair: bool,
        num_validators: Option<u32>,
        invulnerables: Vec<u64>,
    }

    impl Default for ExtBuilder {
        fn default() -> Self {
            Self {
                existential_deposit: 0,
                validator_pool: false,
                nominate: true,
                validator_count: 2,
                minimum_validator_count: 0,
                fair: true,
                num_validators: None,
                invulnerables: vec![],
            }
        }
    }

    impl ExtBuilder {
        pub fn existential_deposit(mut self, existential_deposit: u64) -> Self {
            self.existential_deposit = existential_deposit;
            self
        }
        pub fn validator_pool(mut self, validator_pool: bool) -> Self {
            self.validator_pool = validator_pool;
            self
        }
        pub fn nominate(mut self, nominate: bool) -> Self {
            self.nominate = nominate;
            self
        }
        pub fn validator_count(mut self, count: u32) -> Self {
            self.validator_count = count;
            self
        }
        pub fn minimum_validator_count(mut self, count: u32) -> Self {
            self.minimum_validator_count = count;
            self
        }
        pub fn fair(mut self, is_fair: bool) -> Self {
            self.fair = is_fair;
            self
        }
        pub fn num_validators(mut self, num_validators: u32) -> Self {
            self.num_validators = Some(num_validators);
            self
        }
        pub fn invulnerables(mut self, invulnerables: Vec<u64>) -> Self {
            self.invulnerables = invulnerables;
            self
        }
        pub fn set_associated_consts(&self) {
            EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        }
        pub fn build(self) -> TestExternalities<Blake2Hasher> {
            self.set_associated_consts();
            let mut storage = system::GenesisConfig::default()
                .build_storage::<Test>()
                .unwrap();
            let balance_factor = if self.existential_deposit > 0 { 256 } else { 1 };

            let num_validators = self.num_validators.unwrap_or(self.validator_count);
            let validators = (0..num_validators)
                .map(|x| ((x + 1) * 10 + 1) as u64)
                .collect::<Vec<_>>();

            let _ = balances::GenesisConfig::<Test> {
                balances: vec![
                    (1, 10 * balance_factor),
                    (2, 20 * balance_factor),
                    (3, 300 * balance_factor),
                    (4, 400 * balance_factor),
                    (10, balance_factor),
                    (11, balance_factor * 1000),
                    (20, balance_factor),
                    (21, balance_factor * 2000),
                    (30, balance_factor),
                    (31, balance_factor * 2000),
                    (40, balance_factor),
                    (41, balance_factor * 2000),
                    (100, 2000 * balance_factor),
                    (101, 2000 * balance_factor),
                    // This allow us to have a total_payout different from 0.
                    (999, 1_000_000_000_000),
                ],
                vesting: vec![],
            }
            .assimilate_storage(&mut storage);

            let stake_21 = if self.fair { 1000 } else { 2000 };
            let stake_31 = if self.validator_pool {
                balance_factor * 1000
            } else {
                1
            };
            let status_41 = if self.validator_pool {
                StakerStatus::<AccountId>::Validator
            } else {
                StakerStatus::<AccountId>::Idle
            };
            let nominated = if self.nominate { vec![11, 21] } else { vec![] };
            let _ = GenesisConfig::<Test> {
                current_era: 0,
                stakers: vec![
                    // (stash, controller, staked_amount, status)
                    (
                        11,
                        10,
                        balance_factor * 1000,
                        StakerStatus::<AccountId>::Validator,
                    ),
                    (21, 20, stake_21, StakerStatus::<AccountId>::Validator),
                    (31, 30, stake_31, StakerStatus::<AccountId>::Validator),
                    (41, 40, balance_factor * 1000, status_41),
                    // nominator
                    (
                        101,
                        100,
                        balance_factor * 500,
                        StakerStatus::<AccountId>::Nominator(nominated),
                    ),
                ],
                validator_count: self.validator_count,
                minimum_validator_count: self.minimum_validator_count,
                invulnerables: self.invulnerables,
                slash_reward_fraction: Perbill::from_percent(10),
                ..Default::default()
            }
            .assimilate_storage(&mut storage);

            let _ = session::GenesisConfig::<Test> {
                keys: validators
                    .iter()
                    .map(|x| (*x, UintAuthorityId(*x)))
                    .collect(),
            }
            .assimilate_storage(&mut storage);

            let mut ext = storage.into();
            with_externalities(&mut ext, || {
                let validators = Session::validators();
                SESSION.with(|x| *x.borrow_mut() = (validators.clone(), HashSet::new()));
            });
            ext
        }
    }

    pub fn start_era(era_index: EraIndex) {
        start_session((era_index * 3).into());
        assert_eq!(Staking::current_era(), era_index);
    }

    pub fn start_session(session_index: SessionIndex) {
        // Compensate for session delay
        let session_index = session_index + 1;
        for i in Session::current_index()..session_index {
            System::set_block_number((i + 1).into());
            Timestamp::set_timestamp(System::block_number() * 1000);
            Session::on_initialize(System::block_number());
        }

        assert_eq!(Session::current_index(), session_index);
    }

    pub type System = system::Module<Test>;
    pub type Balances = balances::Module<Test>;
    pub type Session = session::Module<Test>;
    pub type Timestamp = timestamp::Module<Test>;

    #[test]
    fn should_initialize_stakers_and_validators() {
        // Verifies initial conditions of mock
        with_externalities(&mut ExtBuilder::default().build(), || {
            assert_eq!(Staking::bonded(&11), Some(10)); // Account 11 is stashed and locked, and account 10 is the controller
            assert_eq!(Staking::bonded(&21), Some(20)); // Account 21 is stashed and locked, and account 20 is the controller
            assert_eq!(Staking::bonded(&1), None); // Account 1 is not a stashed

            // Account 10 controls the stash from account 11, which is 100 * balance_factor units
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![]
                })
            );
            // Account 20 controls the stash from account 21, which is 200 * balance_factor units
            assert_eq!(
                Staking::ledger(&20),
                Some(StakingLedger {
                    stash: 21,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![]
                })
            );
            // Account 1 does not control any stash
            assert_eq!(Staking::ledger(&1), None);
        });
    }

    #[test]
    fn should_add_potential_validators() {
        with_externalities(
            &mut ExtBuilder::default()
                .minimum_validator_count(2)
                .validator_count(2)
                .num_validators(2)
                .validator_pool(true)
                .nominate(false)
                .build(),
            || {
                assert_ok!(Staking::add_potential_validator(Origin::signed(1), 10));
                assert_ok!(Staking::add_potential_validator(Origin::signed(1), 20));

                assert_ok!(Staking::compliance_failed(Origin::signed(3), 20));

                assert_eq!(Staking::is_controller_eligible(&10), true);
                assert_eq!(Staking::is_controller_eligible(&20), false);
            },
        );
    }

    #[test]
    fn should_remove_validators() {
        with_externalities(
            &mut ExtBuilder::default()
                .minimum_validator_count(2)
                .validator_count(2)
                .num_validators(2)
                .validator_pool(true)
                .nominate(false)
                .build(),
            || {
                assert_ok!(Staking::add_potential_validator(Origin::signed(1), 10));
                assert_ok!(Staking::add_potential_validator(Origin::signed(1), 20));

                assert_ok!(Staking::compliance_failed(Origin::signed(3), 20));

                assert_ok!(Staking::remove_validator(Origin::signed(2), 20));

                assert_eq!(
                    Staking::permissioned_validators(&10),
                    Some(PermissionedValidator {
                        compliance: Compliance::Active
                    })
                );
                assert_eq!(Staking::permissioned_validators(&20), None);

                assert_eq!(Staking::permissioned_validators(&30), None);
            },
        );
    }
}
