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

//! # Permissioned Validtors Module
//!

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(feature = "runtime-benchmarks")]
pub use pallet_staking::testing_utils;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

pub mod inflation;

pub mod permissioned;
pub use permissioned::PolymeshConvertCurve;

pub mod types;
pub use pallet_staking::permissioned_staking::PermissionedStaking;

use frame_support::pallet_prelude::*;
use frame_support::storage::bounded_vec::BoundedVec;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_system::pallet_prelude::*;
use sp_runtime::{curve::PiecewiseLinear, traits::AtLeast32BitUnsigned, Perbill, Permill};
use sp_staking::EraIndex;
use sp_std::prelude::*;

use polymesh_primitives::{IdentityId, GC_DID};

pub(crate) use pallet_staking::{
    BalanceOf, Bonded, Config as StakingConfig, EraPayout, Error as StakingError, Ledger,
    MinValidatorBond, Pallet as StakingPallet, SessionInterface as _, ValidatorCount,
    ValidatorPrefs, Validators, WeightInfo as _,
};

use types::{PermissionedIdentityPrefs, SlashingSwitch};

pub(crate) const LOG_TARGET: &str = "runtime::permissioned_validators";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

/// Weight functions needed for pallet_staking.
pub trait WeightInfo {
    fn add_permissioned_validator() -> Weight;
    fn remove_permissioned_validator() -> Weight;
    fn change_slashing_allowed_for() -> Weight;
    fn update_permissioned_validator_intended_count() -> Weight;
    fn chill_from_governance(n: u32) -> Weight;
    fn set_commission_cap(n: u32) -> Weight;
}

mod migrations {
    use super::*;
    use frame_support::traits::{Get, GetStorageVersion};

    pub fn migrate_v1<T: Config>() -> Weight {
        let in_code = Pallet::<T>::in_code_storage_version();
        let on_chain = Pallet::<T>::on_chain_storage_version();

        if on_chain == 1 && in_code == 1 {
            let old_pallet = b"Staking";
            let new_pallet = Pallet::<T>::name().as_bytes();
            // Move: PermissionedIdentity
            frame_support::storage::migration::move_storage_from_pallet(
                b"PermissionedIdentity",
                old_pallet,
                new_pallet,
            );
            // Move: SlashingAllowedFor
            frame_support::storage::migration::move_storage_from_pallet(
                b"SlashingAllowedFor",
                old_pallet,
                new_pallet,
            );
            // Move: ValidatorCommissionCap
            frame_support::storage::migration::move_storage_from_pallet(
                b"ValidatorCommissionCap",
                old_pallet,
                new_pallet,
            );
            in_code.put::<Pallet<T>>();

            log::info!(
                target: crate::LOG_TARGET,
                "Moved some storage from pallet Staking to pallet Validators",
            );
            return T::DbWeight::get().reads_writes(3, 3);
        }
        Weight::zero()
    }
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
    pub struct AwaitingPayout<AccountId, T: Get<u32>> {
        pub era_index: EraIndex,
        pub validators: BoundedVec<AccountId, T>,
    }

    impl<AccountId, T: Get<u32>> AwaitingPayout<AccountId, T> {
        pub fn new(era_index: EraIndex, validators: BoundedVec<AccountId, T>) -> Self {
            Self {
                era_index,
                validators,
            }
        }
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_identity::Config
        + pallet_staking::Config
        + pallet_babe::Config
    {
        /// pallet weights.
        type WeightInfo: WeightInfo;

        /// Maximum amount of validators that can run by an identity.
        /// It will be MaxValidatorPerIdentity * Self::validator_count().
        #[pallet::constant]
        type MaxValidatorPerIdentity: Get<Permill>;

        /// Maximum amount of total issuance after which fixed rewards kicks in.
        #[pallet::constant]
        type MaxVariableInflationTotalIssuance: Get<BalanceOf<Self>>;

        /// Yearly total reward amount that gets distributed when fixed rewards kicks in.
        #[pallet::constant]
        type FixedYearlyReward: Get<BalanceOf<Self>>;

        /// Overarching type of all pallets origins.
        type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

        /// Maximum weight for paying out stakers.
        #[pallet::constant]
        type MaxPayoutWeight: Get<Weight>;

        /// Maximum number of eras that can be awaiting payout.
        #[pallet::constant]
        type MaxEraAwaitingPayout: Get<u32>;

        /// Maximum number of validators that can be awaiting payout for a given era.
        #[pallet::constant]
        type MaxValidatorsPerEraAwaitingPayout: Get<u32> + TypeInfo;
    }

    /// Entities that are allowed to run operator/validator nodes.
    #[pallet::storage]
    #[pallet::getter(fn permissioned_identity)]
    pub type PermissionedIdentity<T: Config> =
        StorageMap<_, Twox64Concat, IdentityId, PermissionedIdentityPrefs, OptionQuery>;

    /// Slashing switch for validators & Nominators.
    #[pallet::storage]
    #[pallet::getter(fn slashing_allowed_for)]
    pub type SlashingAllowedFor<T: Config> = StorageValue<_, SlashingSwitch, ValueQuery>;

    /// Allows flexibility in commission. Every validator has commission that should be in the range [0, Cap].
    #[pallet::storage]
    #[pallet::getter(fn validator_commission_cap)]
    pub type ValidatorCommissionCap<T: Config> = StorageValue<_, Perbill, ValueQuery>;

    /// All validators that are awaiting payout for a given era.
    #[pallet::storage]
    #[pallet::getter(fn pending_payouts)]
    pub type PendingPayouts<T: Config> = StorageValue<
        _,
        BoundedVec<
            AwaitingPayout<T::AccountId, T::MaxValidatorsPerEraAwaitingPayout>,
            T::MaxEraAwaitingPayout,
        >,
        ValueQuery,
    >;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        pub validators: Vec<IdentityId>,
        pub slashing_allowed_for: SlashingSwitch,
        pub validator_commission_cap: Perbill,
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            SlashingAllowedFor::<T>::put(self.slashing_allowed_for);
            ValidatorCommissionCap::<T>::put(self.validator_commission_cap);

            for &did in &self.validators {
                crate::log!(trace, "inserting genesis permissioned validator: {:?}", did,);
                if <Pallet<T>>::permissioned_identity(&did).is_none() {
                    // Adding identity directly in the storage by assuming it is CDD'ed
                    PermissionedIdentity::<T>::insert(&did, PermissionedIdentityPrefs::new(3));
                    <Pallet<T>>::deposit_event(Event::<T>::PermissionedIdentityAdded {
                        governance_councill_did: GC_DID,
                        validators_identity: did,
                    });
                }
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// User has updated their nominations.
        Nominated {
            nominator_identity: IdentityId,
            stash: T::AccountId,
            targets: Vec<T::AccountId>,
        },
        /// An identity has issued a candidacy for becoming a validator.
        PermissionedIdentityAdded {
            governance_councill_did: IdentityId,
            validators_identity: IdentityId,
        },
        /// An identity has been removed from the permissioned identities pool.
        PermissionedIdentityRemoved {
            governance_councill_did: IdentityId,
            validators_identity: IdentityId,
        },
        /// Remove the nominators from the valid nominators when there CDD expired.
        InvalidatedNominators {
            governance_councill_did: IdentityId,
            governance_councill_account: IdentityId,
            expired_nominators: Vec<T::AccountId>,
        },
        /// Slashing allowed has been updated.
        SlashingAllowedForChanged { slashing_switch: SlashingSwitch },
        /// Reward scheduling interrupted.
        RewardPaymentSchedulingInterrupted {
            account_id: T::AccountId,
            era: EraIndex,
            error: DispatchError,
        },
        /// Commission cap has been updated.
        CommissionCapUpdated {
            governance_councill_did: IdentityId,
            old_commission_cap: Perbill,
            new_commission_cap: Perbill,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Validator or nominator stash identity does not exist.
        StashIdentityDoesNotExist,
        /// Validator's stash identity is not permissioned.
        StashIdentityNotPermissioned,
        /// Permissioned validator already exists.
        IdentityIsAlreadyPermissioned,
        /// Identity does not exist or is locked.
        IdentityIsInactive,
        /// When the intended number of validators to run is >= 2/3 of `validator_count`.
        IntendedCountIsExceedingConsensusLimit,
        /// Identity was not found in the permissioned identity pool.
        IdentityNotFound,
        /// No validator was found for the given key.
        ValidatorNotFound,
        /// Validator commiission is above maximum.
        CommissionTooHigh,
        /// New commission must be different from previous commission.
        CommissionUnchanged,
        /// Too many validators are awaiting payout for a given era.
        TooManyValidatorsForEra,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            migrations::migrate_v1::<T>()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Adds a permissioned identity and sets its preferences.
        ///
        /// The dispatch origin must be Root.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::add_permissioned_validator())]
        pub fn add_permissioned_validator(
            origin: OriginFor<T>,
            identity: IdentityId,
            intended_count: Option<u32>,
        ) -> DispatchResult {
            Self::base_add_permissioned_validator(origin, identity, intended_count)
        }

        /// Remove an identity from the pool of (wannabe) validator identities. Effects are known in the next session.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # Arguments
        /// * origin Required origin for removing a potential validator.
        /// * identity Validator's IdentityId.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_permissioned_validator())]
        pub fn remove_permissioned_validator(
            origin: OriginFor<T>,
            identity: IdentityId,
        ) -> DispatchResult {
            Self::base_remove_permissioned_validator(origin, identity)
        }

        /// Switch slashing status on the basis of given `slashing_switch`. Can only be called by root.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::change_slashing_allowed_for())]
        pub fn change_slashing_allowed_for(
            origin: OriginFor<T>,
            slashing_switch: SlashingSwitch,
        ) -> DispatchResult {
            Self::base_change_slashing_allowed_for(origin, slashing_switch)
        }

        /// Sets the intended count to `new_intended_count` for the given `identity`.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::update_permissioned_validator_intended_count())]
        pub fn update_permissioned_validator_intended_count(
            origin: OriginFor<T>,
            identity: IdentityId,
            new_intended_count: u32,
        ) -> DispatchResult {
            Self::base_update_permissioned_validator_intended_count(
                origin,
                identity,
                new_intended_count,
            )
        }

        /// Governance council forcefully chills a validator. Effects will be felt at the beginning of the next era.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::chill_from_governance(stash_keys.len() as u32))]
        pub fn chill_from_governance(
            origin: OriginFor<T>,
            identity: IdentityId,
            stash_keys: Vec<T::AccountId>,
        ) -> DispatchResult {
            Self::base_chill_from_governance(origin, identity, stash_keys)
        }

        /// Changes commission rate which applies to all validators. Only Governance
        /// committee is allowed to change this value.
        ///
        /// # Arguments
        /// * `new_cap` the new commission cap.
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::set_commission_cap(150))]
        pub fn set_commission_cap(origin: OriginFor<T>, new_cap: Perbill) -> DispatchResult {
            Self::base_set_commission_cap(origin, new_cap)
        }
    }
}
