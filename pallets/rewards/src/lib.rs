// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
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
//! TODO.
//!
//! ## Overview
//!
//! TODO
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `TODO`: TODO.

use codec::{Decode, Encode};
use frame_support::sp_runtime::MultiAddress;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::Currency,
    weights::{DispatchClass, Pays},
};
use pallet_balances as balances;
use pallet_identity::{self as identity, PermissionedCallOriginData};
use pallet_staking as staking;
use pallet_staking::RewardDestination;
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_common_utilities::traits::{identity::Config as IdentityConfig, CommonConfig};
use polymesh_common_utilities::with_transaction;
use sp_runtime::traits::{StaticLookup, Verify};

type Balances<T> = balances::Module<T>;
type Identity<T> = identity::Module<T>;
type Staking<T> = staking::Module<T>;

pub trait Config:
    frame_system::Config + IdentityConfig + balances::Config + staking::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    /// Weight information for extrinsic of the sto pallet.
    type WeightInfo: WeightInfo;
}

pub trait WeightInfo {}

/// Represents an Itn reward's status.
#[derive(Decode, Encode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItnRewardStatus<Balance: Encode + Decode> {
    UnClaimed(Balance),
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
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Rewards {
        /// Map of (Itn Address `T::AccountId`) -> (Reward `ItnRewardStatus`).
        pub ItnRewards get(fn itn_rewards)
            config(): map hasher(blake2_128_concat) T::AccountId
            => Option<ItnRewardStatus<T::Balance>>;
    }
}

decl_event! {
    pub enum Event<T>
    where
        Balance = <T as CommonConfig>::Balance,
    {
        /// TODO.
        TODOEvent(Balance),
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Claim an ITN reward
        ///
        /// ## Arguments
        /// `itn_address` - Address of the awarded account on ITN.
        /// `signature` - Signature of the encoded reward data (originKey, ItnKey, amount)
        ///
        /// ## Errors
        /// - Todo
        #[weight = (0, DispatchClass::Operational, Pays::No)]
        pub fn claim_itn_reaward(origin, itn_address: T::AccountId, signiture: T::OffChainSignature) -> DispatchResult {
            let PermissionedCallOriginData{ sender, primary_did, .. } = <Identity<T>>::ensure_origin_call_permissions(origin)?;
            <ItnRewards<T>>::try_mutate(&itn_address, |reward| {
                match reward {
                    // Unclaimed. Attempt to claim.
                    Some(ItnRewardStatus::UnClaimed(amount)) => {
                        ensure!(
                            signiture.verify((sender.clone(), itn_address.clone(), *amount).encode().as_slice(), &itn_address),
                            Error::<T>::InvalidSignature
                        );
                        with_transaction(|| {
                            // Deposit `amount` + 1 because `amount` will be bounded, we want the user to have some unbonded balance.
                            let _ = <Balances<T>>::deposit_into_existing(&sender, *amount + (1 * POLY).into())?;
                            //TODO(Connor): Finalize bonding details.
                            <Staking<T>>::bond(origin, sender, *amount.into(), RewardDestination::Stash)?;
                            *reward = Some(ItnRewardStatus::Claimed);
                            Ok(())
                        })
                    },
                    // Already Claimed.
                    Some(ItnRewardStatus::Claimed) => Err(Error::<T>::ItnRewardAlreadyClaimed.into()),
                    // Unknown Address.
                    None => Err(Error::<T>::UnknownItnAddress.into()),
                }
            })
        }
    }
}
