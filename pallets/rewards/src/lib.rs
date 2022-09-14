// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! # Rewards Module
//!
//! Implements reward claiming and distribution.
//!
//! ## Overview
//!
//! The only available rewards are those from the ITN(Incentivised Test Network).
//! These rewards can be claimed using the `claim_itn_reward` extrinsic.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `claim_itn_reward`: Claim an ITN reward with a valid signature.
//! - `set_itn_reward_status`: Set the status of an account ITN reward, can only be called by root.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use codec::{Decode, Encode};
use frame_support::dispatch::Weight;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement},
    unsigned::TransactionSource,
    unsigned::TransactionValidity,
};
use frame_system::{ensure_none, ensure_root, RawOrigin};
use pallet_staking::{self as staking, RewardDestination};
use polymesh_common_utilities::{
    constants::{currency::ONE_POLY, REWARDS_PALLET_ID},
    traits::identity::Config as IdentityConfig,
    with_transaction,
};
use polymesh_primitives::Balance;
use scale_info::TypeInfo;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionLongevity};
use sp_runtime::{
    traits::{AccountIdConversion, StaticLookup, Verify},
    transaction_validity::ValidTransaction,
    DispatchError,
};
use sp_std::{convert::TryInto, prelude::*, vec};

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

type Staking<T> = staking::Module<T>;
type BalanceOf<T> = <<T as pallet_staking::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

pub trait Config: frame_system::Config + IdentityConfig + staking::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    /// Weight information for extrinsic of the sto pallet.
    type WeightInfo: WeightInfo;
}

pub trait WeightInfo {
    fn claim_itn_reward() -> Weight;
    fn set_itn_reward_status() -> Weight;
}

/// Represents an Itn reward's status.
#[derive(Decode, Encode, TypeInfo, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItnRewardStatus {
    Unclaimed(Balance),
    Claimed,
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Address was not found in the list of Itn addresses.
        UnknownItnAddress,
        /// Itn reward was already claimed.
        ItnRewardAlreadyClaimed,
        /// Provided signature was invalid.
        InvalidSignature,
        /// Balance can not be converted to a primitive.
        UnableToCovertBalance,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Rewards {
        /// Map of (Itn Address `AccountId`) -> (Reward `ItnRewardStatus`).
        pub ItnRewards get(fn itn_rewards): map hasher(blake2_128_concat) T::AccountId
            => Option<ItnRewardStatus>;
    }
    add_extra_genesis {
        config(itn_rewards): Vec<(T::AccountId, Balance)>;
        build(|config: &GenesisConfig<T>| {
            for (account, balance) in &config.itn_rewards {
                let _ = <Module::<T>>::base_set_itn_reward_status(RawOrigin::Root.into(), account, ItnRewardStatus::Unclaimed(*balance));
            }
        });
    }
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// Itn reward was claimed.
        ItnRewardClaimed(AccountId, Balance),
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Claim an ITN reward.
        ///
        /// ## Arguments
        /// * `itn_address` specifying the awarded address on ITN.
        /// * `signature` authenticating the claim to the reward.
        ///    The signature should contain `reward_address` followed by the suffix `"claim_itn_reward"`,
        ///    and must have been signed by `itn_address`.
        ///
        /// # Errors
        /// * `InsufficientBalance` - Itn rewards has insufficient funds to issue the reward.
        /// * `InvalidSignature` - `signature` had an invalid signer or invalid message.
        /// * `ItnRewardAlreadyClaimed` - Reward issued to the `itn_address` has already been claimed.
        /// * `UnknownItnAddress` - `itn_address` is not in the rewards table and has no reward to be claimed.
        #[weight = <T as Config>::WeightInfo::claim_itn_reward()]
        pub fn claim_itn_reward(origin, reward_address: T::AccountId, itn_address: T::AccountId, signature: T::OffChainSignature) -> DispatchResult {
            ensure_none(origin)?;
            Self::base_claim_itn_reward(reward_address, itn_address, signature)
        }

        #[weight = <T as Config>::WeightInfo::set_itn_reward_status()]
        pub fn set_itn_reward_status(origin, itn_address: T::AccountId, status: ItnRewardStatus) -> DispatchResult {
            Self::base_set_itn_reward_status(origin, &itn_address, status)
        }
    }
}

impl<T: Config> Module<T> {
    /// The account ID of the rewards pot.
    pub fn account_id() -> T::AccountId {
        REWARDS_PALLET_ID.into_account()
    }

    /// Converts `polymesh_primitive::Balance` balances into (`bonded_amount`, `deposit_amount`).
    /// `bonded_amount` is equal to `raw_balance` but type converted to `BalanceOf<T>`.
    /// `deposit_amount` is 1 Poly greater than `raw_balance` and also converted to `BalanceOf<T>`.
    fn convert_balance(
        raw_balance: Balance,
    ) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
        Ok((
            raw_balance
                .try_into()
                .map_err(|_| Error::<T>::UnableToCovertBalance)?,
            (raw_balance.saturating_add(ONE_POLY))
                .try_into()
                .map_err(|_| Error::<T>::UnableToCovertBalance)?,
        ))
    }

    fn valid_claim(
        reward_address: &T::AccountId,
        itn_address: &T::AccountId,
        signature: &T::OffChainSignature,
    ) -> bool {
        match <ItnRewards<T>>::get(itn_address) {
            Some(ItnRewardStatus::Unclaimed(_)) => {
                Self::verify_itn_sig(reward_address, itn_address, signature).is_ok()
            }
            _ => false,
        }
    }

    fn verify_itn_sig(
        reward_address: &T::AccountId,
        itn_address: &T::AccountId,
        signature: &T::OffChainSignature,
    ) -> DispatchResult {
        let mut msg = [0u8; 48];
        msg[..32].copy_from_slice(&reward_address.encode());
        msg[32..].copy_from_slice(b"claim_itn_reward");
        ensure!(
            signature.verify(&msg[..], itn_address),
            Error::<T>::InvalidSignature
        );
        Ok(())
    }

    pub fn base_claim_itn_reward(
        reward_address: T::AccountId,
        itn_address: T::AccountId,
        signature: T::OffChainSignature,
    ) -> DispatchResult {
        <ItnRewards<T>>::try_mutate(&itn_address, |reward| {
            match reward {
                // Unclaimed. Attempt to claim.
                Some(ItnRewardStatus::Unclaimed(amount)) => {
                    // `amount` and `bonded_amount` are equal in value but different types.
                    // `deposit_amount` is 1 POLY more because we bond `bonded_amount`
                    // and we don't want all of the user's poly bonded.
                    let amount = *amount;
                    let (bonded_amount, deposit_amount) = Self::convert_balance(amount)?;

                    // Verify the signature.
                    Self::verify_itn_sig(&reward_address, &itn_address, &signature)?;

                    // `with_transaction` is required because we must undo the transfer if bonding fails.
                    with_transaction(|| {
                        // Transfer `deposit_amount` from the rewards treasury to the `reward_address`.
                        T::Currency::transfer(
                            &Self::account_id(),
                            &reward_address,
                            deposit_amount,
                            ExistenceRequirement::AllowDeath,
                        )?;

                        // Bond additional `bonded_amount` for `reward_address`.
                        let origin = RawOrigin::Signed(reward_address.clone()).into();
                        if <Staking<T>>::bonded(&reward_address).is_some() {
                            <Staking<T>>::bond_extra(origin, bonded_amount)
                        } else {
                            <Staking<T>>::bond(
                                origin,
                                T::Lookup::unlookup(reward_address.clone()),
                                bonded_amount,
                                RewardDestination::Staked,
                            )
                        }
                    })?;

                    // Set the reward to claimed.
                    *reward = Some(ItnRewardStatus::Claimed);
                    Self::deposit_event(Event::<T>::ItnRewardClaimed(reward_address, amount));
                    Ok(())
                }
                // Already Claimed.
                Some(ItnRewardStatus::Claimed) => Err(Error::<T>::ItnRewardAlreadyClaimed.into()),
                // Unknown Address.
                None => Err(Error::<T>::UnknownItnAddress.into()),
            }
        })
    }

    pub fn base_set_itn_reward_status(
        origin: T::Origin,
        itn_address: &T::AccountId,
        status: ItnRewardStatus,
    ) -> DispatchResult {
        ensure_root(origin)?;
        <ItnRewards<T>>::insert(itn_address, status);
        Ok(())
    }
}

impl<T: Config> sp_runtime::traits::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
        const PRIORITY: u64 = 100;
        if let Call::claim_itn_reward {
            reward_address,
            itn_address,
            signature,
        } = call
        {
            if Self::valid_claim(reward_address, itn_address, signature) {
                return Ok(ValidTransaction {
                    priority: PRIORITY,
                    requires: Vec::new(),
                    provides: vec![("rewards", reward_address).encode()],
                    longevity: TransactionLongevity::MAX,
                    propagate: true,
                });
            }
        }
        Err(InvalidTransaction::Call.into())
    }
}
