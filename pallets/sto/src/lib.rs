// Copyright (c) 2020 Polymath

//! # Sto Module
//!
//! Sto module creates and manages security token offerings
//!
//! ## Overview
//!
//! Primary issuance agent's can create and manage fundraisers of assets.
//! Fundraisers are of fixed supply, with optional expiry and tiered pricing.
//! Fundraisers allow a single payment asset, known as the raising asset.
//! Investors can invest through on-chain balance or off-chain receipts.
//!
//! ## Dispatchable Functions
//!
//! - `create_fundraiser` - Create a new fundraiser.
//! - `invest` - Invest in a fundraiser.
//! - `freeze_fundraiser` - Freeze a fundraiser.
//! - `unfreeze_fundraiser` - Unfreeze a fundraiser.
//! - `modify_fundraiser_window` - Modify the time window a fundraiser is active.
//! - `stop` - stop a fundraiser.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use pallet_asset as asset;
use pallet_identity::{self as identity, PermissionedCallOriginData};
use pallet_portfolio::{self as portfolio, Trait as PortfolioTrait};
use pallet_settlement::{
    self as settlement, Leg, ReceiptDetails, SettlementType, Trait as SettlementTrait, VenueInfo,
    VenueType,
};
use pallet_timestamp::{self as timestamp, Trait as TimestampTrait};
use polymesh_common_utilities::{
    portfolio::PortfolioSubTrait,
    traits::{asset::Trait as AssetTrait, identity::Trait as IdentityTrait},
    with_transaction, CommonTrait,
};
use polymesh_primitives::{IdentityId, PortfolioId, SecondaryKey, Ticker};
use sp_runtime::traits::{CheckedAdd, CheckedMul};
use sp_runtime::DispatchError;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

const MAX_TIERS: usize = 10;

type Identity<T> = identity::Module<T>;
type Settlement<T> = settlement::Module<T>;
type Timestamp<T> = timestamp::Module<T>;
type Portfolio<T> = portfolio::Module<T>;
type Asset<T> = asset::Module<T>;

/// Details about the Fundraiser
#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fundraiser<Balance, Moment> {
    /// The primary issuance agent that created the `Fundraiser`
    pub creator: IdentityId,
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
    /// Total amount available
    pub total: Balance,
    /// Price per unit
    pub price: Balance,
}

/// Single price tier of a `Fundraiser`.
/// Similar to a `PriceTier` but with an extra field `remaining` for tracking the amount available for purchase in a tier.
#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FundraiserTier<Balance> {
    /// Total amount available
    pub total: Balance,
    /// Price per unit
    pub price: Balance,
    /// Total amount remaining for sale, set to `total` and decremented until `0`.
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
        /// Interacting with a fundraiser past the end `Moment`.
        FundraiserExpired,
        /// Interacting with a fundraiser before the start `Moment`.
        FundraiserNotStarted,
        /// Using an invalid venue
        InvalidVenue,
        /// Using an invalid portfolio
        InvalidPortfolio,
        /// An individual price tier was invalid or a set of price tiers was invalid
        InvalidPriceTiers,
        /// Window (start time, end time) has invalid parameters, e.g start time is after end time.
        InvalidOfferingWindow,
        /// Price of an investment exceeded the max price
        MaxPriceExceeded,
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
    pub struct Module<T: Trait> for enum Call where origin: <T as frame_system::Trait>::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Create a new fundraiser.
        ///
        /// * `offering_portfolio` - Portfolio containing the `offering_asset`.
        /// * `offering_asset` - Asset being offered.
        /// * `raising_portfolio` - Portfolio containing the `raising_asset`.
        /// * `raising_asset` - Asset being exchanged for `offering_asset` on investment.
        /// * `tiers` - Price tiers to charge investors on investment.
        /// * `venue_id` - Venue to handle settlement.
        /// * `start` - Fundraiser start time, if `None` the fundraiser will start immediately.
        /// * `end` - Fundraiser end time, if `None` the fundraiser will never expire.
        ///
        /// # Weight
        /// `800_000_000` placeholder
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
            let (did, secondary_key) = Self::ensure_perms_pia(origin, &offering_asset)?;

            let venue = VenueInfo::get(venue_id).ok_or(Error::<T>::InvalidVenue)?;
            ensure!(
                venue.creator == did && venue.venue_type == VenueType::Sto,
                Error::<T>::InvalidVenue
            );

            <Portfolio<T>>::ensure_portfolio_custody_and_permission(raising_portfolio, did, secondary_key.as_ref())?;
            <Portfolio<T>>::ensure_portfolio_custody_and_permission(offering_portfolio, did, secondary_key.as_ref())?;

            ensure!(
                tiers.len() > 0 && tiers.len() <= MAX_TIERS && tiers.iter().all(|t| t.total > 0.into()),
                Error::<T>::InvalidPriceTiers
            );

            let offering_amount: T::Balance = tiers
                .iter()
                .map(|t| t.total)
                .fold(0.into(), |total, x| total + x);

            let start = start.unwrap_or_else(Timestamp::<T>::get);
            if let Some(end) = end {
                ensure!(start < end, Error::<T>::InvalidOfferingWindow);
            }

            <Portfolio<T>>::lock_tokens(&offering_portfolio, &offering_asset, &offering_amount)?;

            let fundraiser_id = Self::fundraiser_count(offering_asset);
            let fundraiser = Fundraiser {
                    creator: did,
                    offering_portfolio,
                    offering_asset,
                    raising_portfolio,
                    raising_asset,
                    tiers: tiers.into_iter().map(Into::into).collect(),
                    venue_id,
                    start,
                    end,
                    frozen: false,
            };

            FundraiserCount::insert(offering_asset, fundraiser_id + 1);
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

        /// Invest in a fundraiser.
        ///
        /// * `investment_portfolio` - Portfolio that `offering_asset` will be deposited in.
        /// * `funding_portfolio` - Portfolio that will fund the investment.
        /// * `offering_asset` - Asset to invest in.
        /// * `fundraiser_id` - ID of the fundraiser to invest in.
        /// * `investment_amount` - Amount of `offering_asset` to invest in.
        /// * `max_price` - Maximum price to pay per unit of `offering_asset`, If `None`there are no constraints on price.
        /// * `receipt` - Off-chain receipt to use instead of on-chain balance in `funding_portfolio`.
        ///
        /// # Weight
        /// `2_000_000_000` placeholder
        #[weight = 2_000_000_000]
        pub fn invest(
            origin,
            investment_portfolio: PortfolioId,
            funding_portfolio: PortfolioId,
            offering_asset: Ticker,
            fundraiser_id: u64,
            investment_amount: T::Balance,
            max_price: Option<T::Balance>,
            receipt: Option<ReceiptDetails<T::AccountId, T::OffChainSignature>>
        ) -> DispatchResult {
            let PermissionedCallOriginData {
                primary_did: did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin.clone())?;

            <Portfolio<T>>::ensure_portfolio_custody_and_permission(investment_portfolio, did, secondary_key.as_ref())?;
            <Portfolio<T>>::ensure_portfolio_custody_and_permission(funding_portfolio, did, secondary_key.as_ref())?;

            let now = Timestamp::<T>::get();
            let fundraiser = <Fundraisers<T>>::get(offering_asset, fundraiser_id).ok_or(Error::<T>::FundraiserNotFound)?;
            ensure!(!fundraiser.frozen, Error::<T>::FundraiserFrozen);
            ensure!(
                fundraiser.start <= now && fundraiser.end.filter(|e| now >= *e).is_none(),
                Error::<T>::FundraiserExpired
            );

            // Remaining tokens to fulfil the investment amount
            let mut remaining = investment_amount;
            // Total cost to to fulfil the investment amount.
            // Primary use is to calculate the blended price (offering_token_amount / cost).
            // Blended price must be <= to max_price or the investment will fail.
            let mut cost = T::Balance::from(0);
            // Individual purchases from each tier that accumulate to fulfil the investment amount.
            // Tuple of (tier_id, amount to purchase from that tier).
            let mut purchases = Vec::new();

            for (id, tier) in fundraiser.tiers.iter().enumerate().filter(|(_, tier)| tier.remaining > 0.into()) {
                // fulfilled the investment amount
                if remaining == 0.into() {
                    break
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
                cost = purchase_amount
                    .checked_mul(&tier.price)
                    .and_then(|pa| cost.checked_add(&pa))
                    .ok_or(Error::<T>::Overflow)?;
            }

            ensure!(remaining == 0.into(), Error::<T>::InsufficientTokensRemaining);
            ensure!(
                max_price.map(|max_price| cost <= max_price * investment_amount).unwrap_or(true),
                Error::<T>::MaxPriceExceeded
            );

            let legs = vec![
                Leg {
                    from: fundraiser.offering_portfolio,
                    to: investment_portfolio,
                    asset: fundraiser.offering_asset,
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
                <Portfolio<T>>::unlock_tokens(&fundraiser.offering_portfolio, &fundraiser.offering_asset, &investment_amount)?;

               let instruction_id = Settlement::<T>::base_add_instruction(
                    fundraiser.creator,
                    fundraiser.venue_id,
                    SettlementType::SettleOnAffirmation,
                    None,
                    legs
                )?;

                let portfolios= vec![fundraiser.offering_portfolio, fundraiser.raising_portfolio].into_iter().collect::<BTreeSet<_>>();
                Settlement::<T>::unsafe_affirm_instruction(fundraiser.creator, instruction_id, portfolios, None)?;

                let portfolios = vec![investment_portfolio, funding_portfolio];
                match receipt {
                    Some(receipt) => Settlement::<T>::base_affirm_with_receipts(
                        origin,
                        instruction_id,
                        vec![receipt],
                        portfolios,
                    ).map_err(|e| e.error)?,
                    None => Settlement::<T>::base_affirm_instruction(origin, instruction_id, portfolios).map_err(|e| e.error)?,
                };

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

        /// Freeze a fundraiser.
        ///
        /// * `offering_asset` - Asset to freeze.
        /// * `fundraiser_id` - ID of the fundraiser to freeze.
        ///
        /// # Weight
        /// `1_000` placeholder
        #[weight = 1_000]
        pub fn freeze_fundraiser(origin, offering_asset: Ticker, fundraiser_id: u64) -> DispatchResult {
            Self::set_frozen(origin, offering_asset, fundraiser_id, true)
        }

        /// Unfreeze a fundraiser.
        ///
        /// * `offering_asset` - Asset to unfreeze.
        /// * `fundraiser_id` - ID of the fundraiser to unfreeze.
        ///
        /// # Weight
        /// `1_000` placeholder
        #[weight = 1_000]
        pub fn unfreeze_fundraiser(origin, offering_asset: Ticker, fundraiser_id: u64) -> DispatchResult {
            Self::set_frozen(origin, offering_asset, fundraiser_id, false)
        }

        /// Modify the time window a fundraiser is active
        ///
        /// * `offering_asset` - Asset to modify.
        /// * `fundraiser_id` - ID of the fundraiser to modify.
        /// * `start` - New start of the fundraiser.
        /// * `end` - New end of the fundraiser to modify.
        ///
        /// # Weight
        /// `1_000` placeholder
        #[weight = 1_000]
        pub fn modify_fundraiser_window(origin, offering_asset: Ticker, fundraiser_id: u64, start: T::Moment, end: Option<T::Moment>) -> DispatchResult {
            Self::ensure_perms_pia(origin, &offering_asset)?;

            let now = Timestamp::<T>::get();
            let fundraiser = <Fundraisers<T>>::get(offering_asset, fundraiser_id)
                .ok_or(Error::<T>::FundraiserNotFound)?;
            if let Some(end) = fundraiser.end {
                ensure!(now < end, Error::<T>::FundraiserExpired);
            };

            if let Some(end) = end {
                ensure!(start < end, Error::<T>::InvalidOfferingWindow);
            }

            <Fundraisers<T>>::mutate(offering_asset, fundraiser_id, |fundraiser| {
                if let Some(fundraiser) = fundraiser {
                    fundraiser.start = start;
                    fundraiser.end = end;
                }
            });
            Ok(())
        }

        /// Stop a fundraiser.
        ///
        /// * `offering_asset` - Asset to stop.
        /// * `fundraiser_id` - ID of the fundraiser to stop.
        ///
        /// # Weight
        /// `1_000` placeholder
        #[weight = 1_000]
        pub fn stop(origin, offering_asset: Ticker, fundraiser_id: u64) -> DispatchResult {
            let did = Self::ensure_perms(origin, &offering_asset)?;

            let fundraiser = <Fundraisers<T>>::get(offering_asset, fundraiser_id)
                .ok_or(Error::<T>::FundraiserNotFound)?;

            ensure!(
                <Asset<T>>::primary_issuance_agent(&offering_asset).ok_or(Error::<T>::Unauthorized)? == did ||fundraiser.creator == did,
                Error::<T>::Unauthorized
            );

            let remaining_amount: T::Balance = fundraiser.tiers
                .iter()
                .map(|t| t.remaining)
                .fold(0.into(), |remaining, x| remaining + x);

            <Portfolio<T>>::unlock_tokens(&fundraiser.offering_portfolio, &fundraiser.offering_asset, &remaining_amount)?;
            <Fundraisers<T>>::remove(offering_asset, fundraiser_id);
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn set_frozen(
        origin: <T as frame_system::Trait>::Origin,
        offering_asset: Ticker,
        fundraiser_id: u64,
        frozen: bool,
    ) -> DispatchResult {
        Self::ensure_perms_pia(origin, &offering_asset)?;
        ensure!(
            <Fundraisers<T>>::contains_key(offering_asset, fundraiser_id),
            Error::<T>::FundraiserNotFound
        );
        <Fundraisers<T>>::mutate(offering_asset, fundraiser_id, |fundraiser| {
            if let Some(fundraiser) = fundraiser {
                fundraiser.frozen = frozen
            }
        });
        Ok(())
    }

    /// Ensure that `origin` is permissioned, returning its DID.
    fn ensure_perms(
        origin: <T as frame_system::Trait>::Origin,
        asset: &Ticker,
    ) -> Result<IdentityId, DispatchError> {
        let PermissionedCallOriginData {
            primary_did,
            secondary_key,
            ..
        } = Identity::<T>::ensure_origin_call_permissions(origin)?;
        <Asset<T>>::ensure_asset_perms(secondary_key.as_ref(), asset)?;
        Ok(primary_did)
    }

    /// Ensure that `origin` is permissioned and the PIA, returning its DID.
    fn ensure_perms_pia(
        origin: <T as frame_system::Trait>::Origin,
        asset: &Ticker,
    ) -> Result<(IdentityId, Option<SecondaryKey<T::AccountId>>), DispatchError> {
        let PermissionedCallOriginData {
            primary_did,
            secondary_key,
            ..
        } = Identity::<T>::ensure_origin_call_permissions(origin)?;
        ensure!(
            <Asset<T>>::primary_issuance_agent_or_owner(asset) == primary_did,
            Error::<T>::Unauthorized
        );
        <Asset<T>>::ensure_asset_perms(secondary_key.as_ref(), asset)?;
        Ok((primary_did, secondary_key))
    }
}
