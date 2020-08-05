// Copyright (c) 2020 Polymath

//! # STO Module

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use pallet_identity as identity;
use polymesh_common_utilities::{
    traits::{asset::Trait as AssetTrait, identity::Trait as IdentityTrait, CommonTrait},
    Context,
};
use polymesh_primitives::{IdentityId, Ticker};

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;
use sp_runtime::traits::{CheckedMul, Saturating};

type Identity<T> = identity::Module<T>;
type Settlement<T> = pallet_settlement::Module<T>;

pub trait Trait:
    frame_system::Trait
    + CommonTrait
    + IdentityTrait
    + pallet_timestamp::Trait
    + pallet_settlement::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

/// Details about the Fundraiser
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fundraiser<Balance> {
    /// Token to raise funds in
    raise_token: Ticker,
    /// Amount of offering token available for sale
    remaining_amount: Balance,
    /// Price of one million offering token units (one full token) in terms of raise token units
    price_per_token: u128,
    /// Id of the venue to use for this fundraise
    venue_id: u64,
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
    {
        /// A new fundraise has been created (offering token, raise token, amount to sell, price, venue id, fundraiser_id)
        FundraiseCreated(IdentityId, Ticker, Ticker, Balance, u128, u64, u64),
    }
);

decl_error! {
    /// Errors for the Settlement module.
    pub enum Error for Module<T: Trait> {
        /// Sender does not have required permissions
        Unauthorized,
        /// An arithmetic operation overflowed
        Overflow
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as StoCapped {
        /// All fundraisers that are currently running. (ticker, fundraiser_id) -> Fundraiser
        Fundraisers get(fn fundraisers): double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) u64 => Fundraiser<T::Balance>;
        /// Total fundraisers created for a token
        FundraiserCount get(fn fundraiser_count): map hasher(twox_64_concat) Ticker => u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Create a new offering.
        #[weight = 200_000_000]
        pub fn create_fundraiser(
            origin,
            offering_token: Ticker,
            raise_token: Ticker,
            sell_amount: T::Balance,
            price_per_token: u128,
            venue_id: u64
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::is_owner(&raise_token, did), Error::<T>::Unauthorized);
            // TODO: Take custodial ownership of $sell_amount of $offering_token from treasury?
            let fundraise_id = Self::fundraiser_count(offering_token) + 1;
            <Fundraisers<T>>::insert(
                offering_token,
                fundraise_id,
                Fundraiser {
                    raise_token,
                    price_per_token,
                    venue_id,
                    remaining_amount: sell_amount
                }
            );
            Self::deposit_event(
                RawEvent::FundraiseCreated(did, offering_token, raise_token, sell_amount, price_per_token, venue_id, fundraise_id)
            );
            Ok(())
        }

        /// Create a new offering.
        #[weight = 200_000_000]
        pub fn invest(origin, offering_token: Ticker, fundraise_id: u64, buy_amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let fundraiser = Self::fundraisers(offering_token, fundraise_id);
            // Ceil of buy_amount * price_per_million
            let sell_amount = buy_amount
                .checked_mul(&fundraiser.price_per_token.into())
                .ok_or(Error::<T>::Overflow)?
                .saturating_add(999_999.into())
                / 1_000_000.into();

            Ok(())
        }
    }

}

//impl<T: Trait> Module<T> {}
