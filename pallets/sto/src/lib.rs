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
use pallet_portfolio::{PortfolioAssetBalances, Trait as PortfolioTrait};
use pallet_settlement::{
    self as settlement, Leg, ReceiptDetails, SettlementType, Trait as SettlementTrait, VenueInfo,
    VenueType,
};
use pallet_timestamp::{self as timestamp, Trait as TimestampTrait};
use polymesh_common_utilities::{
    traits::{asset::Trait as AssetTrait, identity::Trait as IdentityTrait},
    with_transaction, CommonTrait, Context,
};
use polymesh_primitives::{IdentityId, PortfolioId, Ticker};
use sp_runtime::traits::{CheckedAdd, CheckedMul};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

type Identity<T> = identity::Module<T>;
type Settlement<T> = settlement::Module<T>;
type Timestamp<T> = timestamp::Module<T>;

/// Details about the Fundraiser
#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fundraiser<Balance, Moment> {
    /// Portfolio containing the asset being offered
    pub offering_portfolio: PortfolioId,
    /// Asset being offered
    pub offering_asset: Ticker,
    /// Portfolio receiving funds raised
    pub raising_portfolio: PortfolioId,
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
#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PriceTier<Balance> {
    pub total: Balance,
    pub price: Balance,
}

/// Single price tier of a `Fundraiser`.
/// Similar to a `PriceTier` but with an extra field `remaining` for tracking the amount available for purchase in a tier.
#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FundraiserTier<Balance> {
    pub total: Balance,
    pub price: Balance,
    pub remaining: Balance,
}

impl<Balance: Clone> Into<FundraiserTier<Balance>> for PriceTier<Balance> {
    fn into(self) -> FundraiserTier<Balance> {
        FundraiserTier {
            total: self.total.clone(),
            price: self.price.clone(),
            remaining: self.total.clone(),
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
        // An individual price tier was invalid or a set of price tiers was invalid
        InvalidPriceTiers,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as StoCapped {
        /// All fundraisers that are currently running. (ticker, fundraiser_id) -> Fundraiser
        Fundraisers get(fn fundraisers): double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) u64 => Option<Fundraiser<T::Balance, T::Moment>>;
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
            raising_portfolio: PortfolioId,
            raising_asset: Ticker,
            tiers: Vec<PriceTier<T::Balance>>,
            venue_id: u64,
            start: Option<T::Moment>,
            end: Option<T::Moment>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::primary_issuance_agent(&offering_asset) == did, Error::<T>::Unauthorized);

            let venue = VenueInfo::get(venue_id).ok_or(Error::<T>::InvalidVenue)?;
            ensure!(
                venue.creator == did && venue.venue_type == VenueType::Sto,
                Error::<T>::InvalidVenue
            );

            ensure!(offering_portfolio.did == did, Error::<T>::InvalidPortfolio);
            ensure!(<PortfolioAssetBalances<T>>::contains_key(offering_portfolio, offering_asset), Error::<T>::InvalidPortfolio);
            let asset_balance = <PortfolioAssetBalances<T>>::get(offering_portfolio, offering_asset);

            ensure!(
                tiers.len() > 0 && tiers.iter().all(|t| t.total > 0.into()),
                Error::<T>::InvalidPriceTiers
            );

            let offering_amount: T::Balance = tiers
                .iter()
                .map(|t| t.total)
                .fold(0.into(), |total, x| total + x);

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
                    raising_portfolio,
                    raising_asset,
                    tiers: tiers.into_iter().map(Into::into).collect(),
                    venue_id,
                    start: start.unwrap_or_else(Timestamp::<T>::get),
                    end,
                    frozen: false,
                };
            FundraiserCount::insert(offering_asset, fundraiser_id);
            <Fundraisers<T>>::insert(
                offering_asset,
                fundraiser_id,
                fundraiser.clone()
            );
            Self::deposit_event(
                RawEvent::FundraiserCreated(did, fundraiser)
            );
            Ok(())
        }

        /// Purchase tokens from an ongoing offering.
        #[weight = 2_000_000_000]
        pub fn invest(
            origin,
            investment_portfolio: PortfolioId,
            funding_portfolio: PortfolioId,
            offering_asset: Ticker,
            fundraiser_id: u64,
            investment_amount: T::Balance,
            max_price: T::Balance,
            reciept: Option<ReceiptDetails<T::AccountId, T::OffChainSignature>>
        ) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;

            ensure!(investment_portfolio.did == did, Error::<T>::InvalidPortfolio);
            ensure!(funding_portfolio.did == did, Error::<T>::InvalidPortfolio);

            let now = Timestamp::<T>::get();
            let fundraiser = <Fundraisers<T>>::get(offering_asset, fundraiser_id).ok_or(Error::<T>::FundraiserNotFound)?;
            ensure!(fundraiser.start <= now, Error::<T>::FundraiserNotStated);
            if let Some(end) = fundraiser.end {
                ensure!(end > now, Error::<T>::FundraiserExpired);
            }

            // Remaining tokens to fulfil the investment amount
            let mut remaining = investment_amount;
            // Total cost to to fulfil the investment amount.
            // Primary use is to calculate the blended price (offering_token_amount / cost).
            // Blended price must be <= to max_price or the investment will fail.
            let mut cost = T::Balance::from(0);
            // Individual purchases from each tier that accumulate to fulfil the investment amount.
            // Tuple of (tier_id, amount to purchase from that tier).
            let mut purchases = Vec::new();

            for (id, tier) in fundraiser.tiers.iter().enumerate() {
                // fulfilled the investment amount
                if remaining == 0.into() {
                    break
                }

                // tier is exhausted, move on
                if tier.remaining == 0.into() {
                    continue
                }

                // Check if this tier can fulfil the remaining investment amount.
                // If it can, purchase the remaining amount.
                // If it can't, purchase what's remaining in the tier.
                let purchase_amount = if tier.remaining >= remaining {
                    remaining
                } else {
                    tier.remaining
                };

                remaining -= purchase_amount;
                purchases.push((id, purchase_amount));
                cost = cost.checked_add(
                    &remaining
                    .checked_mul(&tier.price)
                    .ok_or(Error::<T>::Overflow)?
                ).ok_or(Error::<T>::Overflow)?;
            }

            ensure!(remaining == 0.into(), Error::<T>::InsufficientTokensRemaining);

            let primary_issuance_agent = T::Asset::primary_issuance_agent(&offering_asset);
            let legs = vec![
                Leg {
                    from: fundraiser.offering_portfolio,
                    to: investment_portfolio,
                    asset: offering_asset,
                    amount: investment_amount
                },
                Leg {
                    from: funding_portfolio,
                    to: fundraiser.raising_portfolio,
                    asset: fundraiser.raising_asset,
                    amount: cost
                }
            ];

            with_transaction(|| {
               let instruction_id = Settlement::<T>::base_add_instruction(
                    primary_issuance_agent,
                    fundraiser.venue_id,
                    SettlementType::SettleOnAuthorization,
                    None,
                    legs
                )?;

                let portfolios = vec![investment_portfolio, funding_portfolio];

                match reciept {
                    Some(reciept) => Settlement::<T>::authorize_with_receipts(
                        origin,
                        instruction_id,
                        vec![reciept],
                        portfolios
                    ).map_err(|e| e.error)?,
                    None => Settlement::<T>::authorize_instruction(origin, instruction_id, portfolios).map_err(|e| e.error)?,
                };

                let portfolios= vec![investment_portfolio, funding_portfolio].into_iter().collect::<BTreeSet<_>>();
                Settlement::<T>::unsafe_authorize_instruction(primary_issuance_agent, instruction_id, portfolios)?;

                <Fundraisers<T>>::mutate(offering_asset, fundraiser_id, |fundraiser| {
                    if let Some(fundraiser) = fundraiser {
                        for (id, amount) in purchases {
                            fundraiser.tiers[id].remaining -= amount;
                        }
                    }
                });

                Ok(())
            })
        }

        #[weight = 1_000]
        pub fn freeze_fundraiser(origin, offering_asset: Ticker, fundraiser_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::primary_issuance_agent(&offering_asset) == did, Error::<T>::Unauthorized);

            let now = Timestamp::<T>::get();
            let fundraiser = <Fundraisers<T>>::get(offering_asset, fundraiser_id).ok_or(Error::<T>::FundraiserNotFound)?;
            ensure!(fundraiser.start <= now, Error::<T>::FundraiserNotStated);
            if let Some(end) = fundraiser.end {
                ensure!(end > now, Error::<T>::FundraiserExpired);
            }

            <Fundraisers<T>>::mutate(offering_asset, fundraiser_id, |fundraiser| if let Some(fundraiser) = fundraiser {
            fundraiser.frozen = true
            });
            Ok(())
        }

        #[weight = 1_000]
        pub fn unfreeze_fundraiser(origin, offering_asset: Ticker, fundraiser_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::primary_issuance_agent(&offering_asset) == did, Error::<T>::Unauthorized);

            let now = Timestamp::<T>::get();
            let fundraiser = <Fundraisers<T>>::get(offering_asset, fundraiser_id).ok_or(Error::<T>::FundraiserNotFound)?;
            ensure!(fundraiser.start <= now, Error::<T>::FundraiserNotStated);
            if let Some(end) = fundraiser.end {
                ensure!(end > now, Error::<T>::FundraiserExpired);
            }

            <Fundraisers<T>>::mutate(offering_asset, fundraiser_id, |fundraiser| if let Some(fundraiser) = fundraiser {
            fundraiser.frozen = false
            });
            Ok(())
        }

        #[weight = 1_000]
        pub fn modify_fundraiser_window(origin, offering_asset: Ticker, fundraiser_id: u64, start: T::Moment, end: Option<T::Moment>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(T::Asset::primary_issuance_agent(&offering_asset) == did, Error::<T>::Unauthorized);

            let now = Timestamp::<T>::get();
            let fundraiser = <Fundraisers<T>>::get(offering_asset, fundraiser_id).ok_or(Error::<T>::FundraiserNotFound)?;
            ensure!(fundraiser.start <= now, Error::<T>::FundraiserNotStated);
            if let Some(end) = fundraiser.end {
                ensure!(end > now, Error::<T>::FundraiserExpired);
            }

            <Fundraisers<T>>::mutate(offering_asset, fundraiser_id, |fundraiser| {
                if let Some(fundraiser) = fundraiser {
                    fundraiser.start = start;
                    fundraiser.end = end;
                }
            });
            Ok(())
        }
    }

}
