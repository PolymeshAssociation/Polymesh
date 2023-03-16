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
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{ensure_root, ensure_signed};
use pallet_staking::{self as staking};
use polymesh_common_utilities::traits::identity::Config as IdentityConfig;
use polymesh_primitives::Balance;
use scale_info::TypeInfo;
use sp_std::{convert::TryInto, prelude::*};

pub trait Config: frame_system::Config + IdentityConfig + staking::Config {
    /// The overarching event type.
    type RuntimeEvent: From<Event<Self>> + Into<<Self as frame_system::Config>::RuntimeEvent>;
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
        build(|_config: &GenesisConfig<T>| {
            // Do nothing
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
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::RuntimeOrigin {
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
        pub fn claim_itn_reward(origin, _reward_address: T::AccountId, _itn_address: T::AccountId, _signature: T::OffChainSignature) -> DispatchResult {
            ensure_signed(origin)?;
            ensure!(false, Error::<T>::UnknownItnAddress);
            Ok(())
            //Self::base_claim_itn_reward(reward_address, itn_address, signature)
        }

        #[weight = <T as Config>::WeightInfo::set_itn_reward_status()]
        pub fn set_itn_reward_status(origin, _itn_address: T::AccountId, _status: ItnRewardStatus) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(false, Error::<T>::UnknownItnAddress);
            Ok(())
            //Self::base_set_itn_reward_status(origin, &itn_address, status)
        }
    }
}
