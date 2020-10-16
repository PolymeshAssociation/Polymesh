// Copyright (c) 2020 Polymath

//! # STO Module
//!
//! This is a proof of concept module. It is not meant to be used in the real world in its' current state.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use pallet_identity as identity;
use pallet_settlement::{Leg, SettlementType};
use polymesh_common_utilities::{
    constants::currency::*,
    traits::{asset::Trait as AssetTrait, identity::Trait as IdentityTrait, CommonTrait},
    Context,
};
use polymesh_primitives::{IdentityId, PortfolioId, Ticker};
use sp_runtime::traits::{CheckedMul, Saturating};
use sp_std::{collections::btree_set::BTreeSet, iter, prelude::*};
type Identity<T> = identity::Module<T>;
type Settlement<T> = pallet_settlement::Module<T>;
type CallPermissions<T> = pallet_permissions::Module<T>;

pub trait Trait:
    frame_system::Trait + CommonTrait + IdentityTrait + pallet_settlement::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

/// Details about the Fundraiser
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fundraiser<Balance> {
    /// Token to raise funds in
    pub raise_token: Ticker,
    /// Amount of offering token available for sale
    pub remaining_amount: Balance,
    /// Price of one million offering token units (one full token) in terms of raise token units
    pub price_per_token: Balance,
    /// Id of the venue to use for this fundraise
    pub venue_id: u64,
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
    {
        /// A new fundraiser has been created
        /// (offering token, raise token, amount to sell, price, venue id, fundraiser_id)
        FundraiserCreated(IdentityId, Ticker, Ticker, Balance, Balance, u64, u64),
        /// An investor invested in the fundraiser
        /// (offering token, raise token, offering_token_amount, raise_token_amount, fundraiser_id)
        FundsRaised(IdentityId, Ticker, Ticker, Balance, Balance, u64),
    }
);

decl_error! {
    /// Errors for the Settlement module.
    pub enum Error for Module<T: Trait> {
        /// Sender does not have required permissions
        Unauthorized,
        /// An arithmetic operation overflowed
        Overflow,
        /// Not enough tokens left for sale
        InsufficientTokensRemaining
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

        /// Create a new offering. A fixed amount of pre-minted tokens are put up for sale at the specified flat rate.
        #[weight = 800_000_000]
        pub fn create_fundraiser(
            origin,
            offering_token: Ticker,
            raise_token: Ticker,
            sell_amount: T::Balance,
            price_per_token: T::Balance,
            venue_id: u64
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            CallPermissions::<T>::ensure_call_permissions(&sender)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::primary_issuance_agent_or_owner(&offering_token) == did, Error::<T>::Unauthorized);
            // TODO: Take custodial ownership of $sell_amount of $offering_token from primary issuance agent?
            let fundraiser_id = Self::fundraiser_count(offering_token) + 1;
            <Fundraisers<T>>::insert(
                offering_token,
                fundraiser_id,
                Fundraiser {
                    raise_token,
                    price_per_token,
                    venue_id,
                    remaining_amount: sell_amount
                }
            );
            Self::deposit_event(
                RawEvent::FundraiserCreated(did, offering_token, raise_token, sell_amount, price_per_token, venue_id, fundraiser_id)
            );
            Ok(())
        }

        /// Purchase tokens from an ongoing offering.
        #[weight = 2_000_000_000]
        pub fn invest(origin, offering_token: Ticker, fundraiser_id: u64, offering_token_amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            CallPermissions::<T>::ensure_call_permissions(&sender)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let mut fundraiser = Self::fundraisers(offering_token, fundraiser_id);
            ensure!(fundraiser.remaining_amount >= offering_token_amount, Error::<T>::InsufficientTokensRemaining);
            // Ceil of offering_token_amount * price_per_million
            let raise_token_amount = offering_token_amount
                .checked_mul(&fundraiser.price_per_token)
                .ok_or(Error::<T>::Overflow)?
                .saturating_add((ONE_UNIT - 1).into())
                / ONE_UNIT.into();

            let primary_issuance_agent = T::Asset::primary_issuance_agent_or_owner(&offering_token);
            let legs = vec![
                Leg {
                    // TODO: Replace with did that actually hold the offering token
                    from: PortfolioId::default_portfolio(primary_issuance_agent),
                    to: PortfolioId::default_portfolio(did),
                    asset: offering_token,
                    amount: offering_token_amount
                },
                Leg {
                    from: PortfolioId::default_portfolio(did),
                    to: PortfolioId::default_portfolio(primary_issuance_agent),
                    asset: fundraiser.raise_token,
                    amount: raise_token_amount
                }
            ];

            let instruction_id = Settlement::<T>::base_add_instruction(
                primary_issuance_agent,
                fundraiser.venue_id,
                SettlementType::SettleOnAffirmation,
                None,
                legs
            )?;

            let pia_portfolios = iter::once(PortfolioId::default_portfolio(primary_issuance_agent)).collect::<BTreeSet<_>>();
            Settlement::<T>::unsafe_affirm_instruction(primary_issuance_agent, instruction_id, pia_portfolios)?;

            let sender_portfolios = vec![PortfolioId::default_portfolio(did)];
            Settlement::<T>::affirm_instruction(origin, instruction_id, sender_portfolios).map_err(|err| err.error)?;

            Self::deposit_event(
                RawEvent::FundsRaised(did, offering_token, fundraiser.raise_token, offering_token_amount, raise_token_amount, fundraiser_id)
            );

            fundraiser.remaining_amount -= offering_token_amount;
            <Fundraisers<T>>::insert(offering_token, fundraiser_id, fundraiser);

            Ok(())
        }
    }

}
