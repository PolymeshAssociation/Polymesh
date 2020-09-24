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
use pallet_portfolio::PortfolioAssetBalances;
use pallet_settlement::{
    self as settlement, Leg, SettlementType, Trait as SettlementTrait, VenueInfo, VenueType,
};
use pallet_timestamp::{self as timestamp};
use polymesh_common_utilities::{
    constants::currency::*,
    traits::{asset::Trait as AssetTrait, identity::Trait as IdentityTrait},
    Context,
};
use polymesh_primitives::{IdentityId, PortfolioId, Ticker};
use sp_runtime::traits::{CheckedAdd, CheckedMul, Saturating};
use sp_std::{collections::btree_set::BTreeSet, iter, prelude::*};

type Identity<T> = identity::Module<T>;
type Settlement<T> = settlement::Module<T>;
type Timestamp<T> = timestamp::Module<T>;

/// Details about the Fundraiser
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fundraiser<Balance, Moment> {
    /// Portfolio containing the asset being offered
    pub offering_portfolio: PortfolioId,
    /// Asset being offered
    pub offering_asset: Ticker,
    /// Asset to receive payment in
    pub raising_asset: Ticker,
    /// Tiers of the fundraiser.
    /// Each tier has a set amount of tokens available at a fixed price.
    /// The sum of the tiers is the total amount available in this fundraiser.
    pub tiers: Vec<FundraiserTier<Balance>>,
    /// Id of the venue to use for this fundraise
    pub venue_id: u64,
    /// Start of the fundraiser
    pub start: Moment,
    /// End of the fundraiser
    pub end: Option<Moment>,
    /// Fundraiser is frozen
    pub frozen: bool,
}

/// Single tier of a tiered pricing model
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PriceTier<Balance> {
    amount: Balance,
    price: Balance,
}

/// Single price tier of a `Fundraiser`.
/// Similar to a `PriceTier` but with an extra field `remaining` for tracking the amount available in a tier.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FundraiserTier<Balance> {
    inner: PriceTier<Balance>,
    remaining: Balance,
}

impl<Balance: From<u8>> Into<FundraiserTier<Balance>> for PriceTier<Balance> {
    fn into(self) -> FundraiserTier<Balance> {
        FundraiserTier {
            inner: self,
            remaining: Balance::from(0),
        }
    }
}

pub trait Trait: frame_system::Trait + IdentityTrait + SettlementTrait + PortfolioTrait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
        Moment = <T as TimestampTrait>::Moment,
    {
        /// A new fundraiser has been created
        /// (primary issuance agent, fundraiser)
        FundraiserCreated(IdentityId, Fundraiser<Balance, Moment>),
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
        InsufficientTokensRemaining,
        /// Fundraiser not found
        FundraiserNotFound,
        /// Fundraiser is frozen
        FundraiserFrozen,
        // Interacting with a fundraiser past the end `Moment`.
        FundraiserExpired,
        // Interacting with a fundraiser before the start `Moment`.
        FundraiserNotStated,
        // Using an invalid venue
        InvalidVenue,
        // Using an invalid portfolio
        InvalidPortfolio,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as StoCapped {
        /// All fundraisers that are currently running. (ticker, fundraiser_id) -> Fundraiser
        Fundraisers get(fn fundraisers): double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) u64 => Fundraiser<T::Balance, T::Moment>;
        /// Total fundraisers created for a token
        FundraiserCount get(fn fundraiser_count): map hasher(twox_64_concat) Ticker => u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Create a new offering. A fixed amount of pre-minted tokens are put up for sale at the specified tiered rate.
        #[weight = 800_000_000]
        pub fn create_fundraiser(
            origin,
            offering_portfolio: PortfolioId,
            offering_asset: Ticker,
            raising_asset: Ticker,
            tiers: Vec<PriceTier<T::Balance>>,
            venue_id: u64,
            start: Option<T::Moment>,
            end: Option<T::Moment>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::primary_issuance_agent(&offering_asset) == did, Error::<T>::Unauthorized);

            ensure!(VenueInfo::contains_key(venue_id), Error::<T>::InvalidVenue);
            let venue = VenueInfo::get(venue_id);

            ensure!(venue.creator == did, Error::<T>::InvalidVenue);
            ensure!(venue.venue_type == VenueType::Sto, Error::<T>::InvalidVenue);

            ensure!(offering_portfolio.did == did, Error::<T>::InvalidPortfolio);
            ensure!(<PortfolioAssetBalances<T>>::contains_key(offering_portfolio, offering_asset), Error::<T>::InvalidPortfolio);
            let asset_balance = <PortfolioAssetBalances<T>>::get(offering_portfolio, offering_asset);

            let offering_amount: T::Balance = tiers
                .iter()
                .map(|t| t.amount)
                .fold(0.into(), |x, total| total + x);

            ensure!(offering_amount >= asset_balance, Error::<T>::InsufficientTokensRemaining);

            let mut tiers = tiers;
            // (Stable) sort by price.
            tiers.sort_by_key(|a| a.price);

            // TODO: Take custodial ownership of $sell_amount of $offering_token from primary issuance agent?
            let fundraiser_id = Self::fundraiser_count(offering_asset) + 1;
            // TODO revise the defaults
            let fundraiser = Fundraiser {
                    offering_portfolio,
                    offering_asset,
                    raising_asset,
                    tiers: tiers.into_iter().map(Into::into).collect(),
                    venue_id,
                    start: start.unwrap_or(Timestamp::<T>::get()),
                    end,
                    frozen: false,
                };
            FundraiserCount::insert(offering_asset, fundraiser_id);
            <Fundraisers<T>>::insert(
                offering_asset,
                fundraiser_id,
                fundraiser
            );
            Self::deposit_event(
                RawEvent::FundraiserCreated(did, fundraiser)
            );
            Ok(())
        }

        /// Purchase tokens from an ongoing offering.
        #[weight = 2_000_000_000]
        pub fn invest(origin, portfolio: PortfolioId, offering_asset: Ticker, fundraiser_id: u64, offering_token_amount: T::Balance, max_price: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;

            ensure!(<Fundraisers<T>>::contains_key(offering_asset, fundraiser_id), Error::<T>::FundraiserNotFound);
            let fundraiser = <Fundraisers<T>>::get(offering_asset, fundraiser_id);

            let mut remaining = offering_token_amount;
            let mut cost = T::Balance::from(0);
            let mut order = Vec::new();
            for (id, tier) in fundraiser.tiers.iter().enumerate() {
                if remaining == 0.into() {
                    break
                }

                if tier.remaining == tier.inner.amount {
                    break
                }

                if tier.remaining > remaining {
                    order.push((id, remaining));
                    cost.checked_add(
                        &remaining
                        .checked_mul(&tier.inner.price)
                        .ok_or(Error::<T>::Overflow)?
                    )
                    .ok_or(Error::<T>::Overflow)?;
                    remaining = 0.into();
                    break;
                }
            }

            // // Ceil of offering_token_amount * price_per_million
            // let raise_token_amount = offering_token_amount
            //     .checked_mul(&fundraiser.price_per_token)
            //     .ok_or(Error::<T>::Overflow)?
            //     .saturating_add((ONE_UNIT - 1).into())
            //     / ONE_UNIT.into();
            //
            // let primary_issuance_agent = T::Asset::primary_issuance_agent(&offering_token);
            // let legs = vec![
            //     Leg {
            //         // TODO: Replace with did that actually hold the offering token
            //         from: PortfolioId::default_portfolio(primary_issuance_agent),
            //         to: PortfolioId::default_portfolio(did),
            //         asset: offering_token,
            //         amount: offering_token_amount
            //     },
            //     Leg {
            //         from: PortfolioId::default_portfolio(did),
            //         to: PortfolioId::default_portfolio(primary_issuance_agent),
            //         asset: fundraiser.raise_token,
            //         amount: raise_token_amount
            //     }
            // ];
            //
            // let instruction_id = Settlement::<T>::base_add_instruction(
            //     primary_issuance_agent,
            //     fundraiser.venue_id,
            //     SettlementType::SettleOnAuthorization,
            //     None,
            //     legs
            // )?;
            //
            // let pia_portfolios = iter::once(PortfolioId::default_portfolio(primary_issuance_agent)).collect::<BTreeSet<_>>();
            // Settlement::<T>::unsafe_authorize_instruction(primary_issuance_agent, instruction_id, pia_portfolios)?;
            //
            // let sender_portfolios = iter::once(PortfolioId::default_portfolio(did)).collect::<BTreeSet<_>>();
            // Settlement::<T>::authorize_instruction(origin, instruction_id, sender_portfolios).map_err(|err| err.error)?;
            //
            // Self::deposit_event(
            //     RawEvent::FundsRaised(did, offering_token, fundraiser.raise_token, offering_token_amount, raise_token_amount, fundraiser_id)
            // );
            //
            // fundraiser.remaining_amount -= offering_token_amount;
            // <Fundraisers<T>>::insert(offering_token, fundraiser_id, fundraiser);

            Ok(())
        }

        #[weight = 1_000]
        pub fn freeze_fundraiser(origin, offering_asset: Ticker, fundraiser_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::primary_issuance_agent(&offering_asset) == did, Error::<T>::Unauthorized);

            ensure!(<Fundraisers<T>>::contains_key(offering_asset, fundraiser_id), Error::<T>::FundraiserNotFound);

            <Fundraisers<T>>::mutate(offering_asset, fundraiser_id, |fundraiser| fundraiser.frozen = true);
            Ok(())
        }

        #[weight = 1_000]
        pub fn unfreeze_fundraiser(origin, offering_asset: Ticker, fundraiser_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::primary_issuance_agent(&offering_asset) == did, Error::<T>::Unauthorized);

            ensure!(<Fundraisers<T>>::contains_key(offering_asset, fundraiser_id), Error::<T>::FundraiserNotFound);

            <Fundraisers<T>>::mutate(offering_asset, fundraiser_id, |fundraiser| fundraiser.frozen = false);
            Ok(())
        }

        #[weight = 1_000]
        pub fn modify_fundraiser_window(origin, offering_asset: Ticker, fundraiser_id: u64, start: T::Moment, end: Option<T::Moment>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::primary_issuance_agent(&offering_asset) == did, Error::<T>::Unauthorized);

            ensure!(<Fundraisers<T>>::contains_key(offering_asset, fundraiser_id), Error::<T>::FundraiserNotFound);

            <Fundraisers<T>>::mutate(offering_asset, fundraiser_id, |fundraiser| {
                fundraiser.start = start;
                fundraiser.end = end;
            });
            Ok(())
        }
    }

}
