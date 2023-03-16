// Copyright (c) 2020 Polymath

//! # Sto Module
//!
//! Sto module creates and manages security token offerings
//!
//! ## Overview
//!
//! Sufficiently permissioned external agent's can create and manage fundraisers of assets.
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

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use pallet_base::try_next_post;
use pallet_identity::PermissionedCallOriginData;
use pallet_settlement::{
    LegAsset, LegV2, ReceiptDetails, SettlementType, VenueId, VenueInfo, VenueType,
};
use polymesh_common_utilities::{
    portfolio::PortfolioSubTrait,
    traits::{identity, portfolio},
    with_transaction,
};
use polymesh_primitives::impl_checked_inc;
use polymesh_primitives_derive::VecU8StrongTyped;
use scale_info::TypeInfo;

use frame_support::weights::Weight;
use polymesh_primitives::{Balance, EventDid, IdentityId, PortfolioId, Ticker};
use sp_runtime::DispatchError;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

pub const MAX_TIERS: usize = 10;

type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;
type Settlement<T> = pallet_settlement::Module<T>;
type Timestamp<T> = pallet_timestamp::Pallet<T>;

/// The per-ticker ID of a fundraiser.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct FundraiserId(pub u64);
impl_checked_inc!(FundraiserId);

/// Status of a Fundraiser.
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, PartialOrd, Ord, Debug)]
pub enum FundraiserStatus {
    /// Fundraiser is open for investments if start_time <= current_time < end_time.
    Live,
    /// Fundraiser has been frozen, New investments can not be made right now.
    Frozen,
    /// Fundraiser has been stopped.
    Closed,
    /// Fundraiser has been stopped before expiry.
    ClosedEarly,
}

impl Default for FundraiserStatus {
    fn default() -> Self {
        Self::Closed
    }
}

/// Details about the Fundraiser.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Fundraiser<Moment> {
    /// The permissioned agent that created the `Fundraiser`.
    pub creator: IdentityId,
    /// Portfolio containing the asset being offered.
    pub offering_portfolio: PortfolioId,
    /// Asset being offered.
    pub offering_asset: Ticker,
    /// Portfolio receiving funds raised.
    pub raising_portfolio: PortfolioId,
    /// Asset to receive payment in.
    pub raising_asset: Ticker,
    /// Tiers of the fundraiser.
    /// Each tier has a set amount of tokens available at a fixed price.
    /// The sum of the tiers is the total amount available in this fundraiser.
    pub tiers: Vec<FundraiserTier>,
    /// Id of the venue to use for this fundraise.
    pub venue_id: VenueId,
    /// Start time of the fundraiser.
    pub start: Moment,
    /// End time of the fundraiser.
    pub end: Option<Moment>,
    /// Fundraiser status.
    pub status: FundraiserStatus,
    /// Minimum raising amount per invest transaction.
    pub minimum_investment: Balance,
}

impl<Moment> Fundraiser<Moment> {
    pub fn is_closed(&self) -> bool {
        self.status == FundraiserStatus::Closed || self.status == FundraiserStatus::ClosedEarly
    }
}

/// Single tier of a tiered pricing model.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PriceTier {
    /// Total amount available.
    pub total: Balance,
    /// Price per unit.
    pub price: Balance,
}

/// Single price tier of a `Fundraiser`.
/// Similar to a `PriceTier` but with an extra field `remaining` for tracking the amount available for purchase in a tier.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct FundraiserTier {
    /// Total amount available.
    pub total: Balance,
    /// Price per unit.
    pub price: Balance,
    /// Total amount remaining for sale, set to `total` and decremented until `0`.
    pub remaining: Balance,
}

impl Into<FundraiserTier> for PriceTier {
    fn into(self) -> FundraiserTier {
        FundraiserTier {
            total: self.total,
            price: self.price,
            remaining: self.total,
        }
    }
}

/// Wrapper type for Fundraiser name.
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FundraiserName(Vec<u8>);

pub trait WeightInfo {
    fn create_fundraiser(i: u32) -> Weight;
    fn invest() -> Weight;
    fn freeze_fundraiser() -> Weight;
    fn unfreeze_fundraiser() -> Weight;
    fn modify_fundraiser_window() -> Weight;
    fn stop() -> Weight;
}

pub trait Config:
    frame_system::Config
    + identity::Config
    + pallet_settlement::Config
    + portfolio::Config
    + pallet_base::Config
{
    /// The overarching event type.
    type RuntimeEvent: From<Event<Self>> + Into<<Self as frame_system::Config>::RuntimeEvent>;
    /// Weight information for extrinsic of the sto pallet.
    type WeightInfo: WeightInfo;
}

decl_event!(
    pub enum Event<T>
    where
        Moment = <T as pallet_timestamp::Config>::Moment,
    {
        /// A new fundraiser has been created.
        /// (Agent DID, fundraiser id, fundraiser name, fundraiser details)
        FundraiserCreated(IdentityId, FundraiserId, FundraiserName, Fundraiser<Moment>),
        /// An investor invested in the fundraiser.
        /// (Investor, fundraiser_id, offering token, raise token, offering_token_amount, raise_token_amount)
        Invested(IdentityId, FundraiserId, Ticker, Ticker, Balance, Balance),
        /// A fundraiser has been frozen.
        /// (Agent DID, fundraiser id)
        FundraiserFrozen(IdentityId, FundraiserId),
        /// A fundraiser has been unfrozen.
        /// (Agent DID, fundraiser id)
        FundraiserUnfrozen(IdentityId, FundraiserId),
        /// A fundraiser window has been modified.
        /// (Agent DID, fundraiser id, old_start, old_end, new_start, new_end)
        FundraiserWindowModified(
            EventDid,
            FundraiserId,
            Moment,
            Option<Moment>,
            Moment,
            Option<Moment>,
        ),
        /// A fundraiser has been stopped.
        /// (Agent DID, fundraiser id)
        FundraiserClosed(IdentityId, FundraiserId),
    }
);

decl_error! {
    /// Errors for the Settlement module.
    pub enum Error for Module<T: Config> {
        /// Sender does not have required permissions.
        Unauthorized,
        /// An arithmetic operation overflowed.
        Overflow,
        /// Not enough tokens left for sale.
        InsufficientTokensRemaining,
        /// Fundraiser not found.
        FundraiserNotFound,
        /// Fundraiser is either frozen or stopped.
        FundraiserNotLive,
        /// Fundraiser has been closed/stopped already.
        FundraiserClosed,
        /// Interacting with a fundraiser past the end `Moment`.
        FundraiserExpired,
        /// An invalid venue provided.
        InvalidVenue,
        /// An individual price tier was invalid or a set of price tiers was invalid.
        InvalidPriceTiers,
        /// Window (start time, end time) has invalid parameters, e.g start time is after end time.
        InvalidOfferingWindow,
        /// Price of the investment exceeded the max price.
        MaxPriceExceeded,
        /// Investment amount is lower than minimum investment amount.
        InvestmentAmountTooLow
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Sto {
        /// All fundraisers that are currently running.
        /// (ticker, fundraiser_id) -> Fundraiser
        Fundraisers get(fn fundraisers):
            double_map
                hasher(blake2_128_concat) Ticker,
                hasher(twox_64_concat) FundraiserId
                => Option<Fundraiser<T::Moment>>;

        /// Total fundraisers created for a token.
        FundraiserCount get(fn fundraiser_count):
            map hasher(blake2_128_concat) Ticker
                => FundraiserId;

        /// Name for the Fundraiser. Only used offchain.
        /// (ticker, fundraiser_id) -> Fundraiser name
        FundraiserNames get(fn fundraiser_name):
            double_map
                hasher(blake2_128_concat) Ticker,
                hasher(twox_64_concat) FundraiserId
                => FundraiserName;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::RuntimeOrigin {
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
        /// * `minimum_investment` - Minimum amount of `raising_asset` that an investor needs to spend to invest in this raise.
        /// * `fundraiser_name` - Fundraiser name, only used in the UIs.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::create_fundraiser(tiers.len() as u32)]
        pub fn create_fundraiser(
            origin,
            offering_portfolio: PortfolioId,
            offering_asset: Ticker,
            raising_portfolio: PortfolioId,
            raising_asset: Ticker,
            tiers: Vec<PriceTier>,
            venue_id: VenueId,
            start: Option<T::Moment>,
            end: Option<T::Moment>,
            minimum_investment: Balance,
            fundraiser_name: FundraiserName
        ) {
            pallet_base::ensure_string_limited::<T>(&fundraiser_name)?;

            let PermissionedCallOriginData {
                primary_did: did,
                secondary_key,
                ..
            } = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, offering_asset)?;

            VenueInfo::get(venue_id)
                .filter(|v| v.creator == did && v.venue_type == VenueType::Sto)
                .ok_or(Error::<T>::InvalidVenue)?;

            <Portfolio<T>>::ensure_portfolio_custody_and_permission(raising_portfolio, did, secondary_key.as_ref())?;
            <Portfolio<T>>::ensure_portfolio_custody_and_permission(offering_portfolio, did, secondary_key.as_ref())?;

            // Ensure there are [1, MAX_TIERS] tiers and that all of their totals are non-zero.
            let mut totals = tiers.iter().map(|t| t.total);
            ensure!(
                (1..=MAX_TIERS).contains(&tiers.len()) && totals.clone().all(|t| t > 0),
                Error::<T>::InvalidPriceTiers
            );

            // Sum all totals, or bail on overflow.
            let offering_amount = totals
                .try_fold(0, |total: Balance, x| total.checked_add(x))
                .ok_or(Error::<T>::InvalidPriceTiers)?;

            // Use current time if start isn't provided.
            let start = start.unwrap_or_else(Timestamp::<T>::get);
            // The start must come strictly before the end.
            if let Some(end) = end {
                ensure!(start < end, Error::<T>::InvalidOfferingWindow);
            }

            // Get the next fundraiser ID.
            let mut seq = FundraiserCount::get(&offering_asset);
            let id = try_next_post::<T, _>(&mut seq)?;

            <Portfolio<T>>::lock_tokens(&offering_portfolio, &offering_asset, offering_amount)?;

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
                status: FundraiserStatus::Live,
                minimum_investment
            };

            FundraiserCount::insert(offering_asset, seq);
            Fundraisers::<T>::insert(offering_asset, id, fundraiser.clone());
            FundraiserNames::insert(offering_asset, id, fundraiser_name.clone());

            Self::deposit_event(RawEvent::FundraiserCreated(did, id, fundraiser_name, fundraiser));
        }

        /// Invest in a fundraiser.
        ///
        /// * `investment_portfolio` - Portfolio that `offering_asset` will be deposited in.
        /// * `funding_portfolio` - Portfolio that will fund the investment.
        /// * `offering_asset` - Asset to invest in.
        /// * `id` - ID of the fundraiser to invest in.
        /// * `purchase_amount` - Amount of `offering_asset` to purchase.
        /// * `max_price` - Maximum price to pay per unit of `offering_asset`, If `None`there are no constraints on price.
        /// * `receipt` - Off-chain receipt to use instead of on-chain balance in `funding_portfolio`.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::invest()]
        pub fn invest(
            origin,
            investment_portfolio: PortfolioId,
            funding_portfolio: PortfolioId,
            offering_asset: Ticker,
            id: FundraiserId,
            purchase_amount: Balance,
            max_price: Option<Balance>,
            receipt: Option<ReceiptDetails<T::AccountId, T::OffChainSignature>>
        ) {
            let PermissionedCallOriginData {
                primary_did: did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin.clone())?;

            <Portfolio<T>>::ensure_portfolio_custody_and_permission(investment_portfolio, did, secondary_key.as_ref())?;
            <Portfolio<T>>::ensure_portfolio_custody_and_permission(funding_portfolio, did, secondary_key.as_ref())?;

            let mut fundraiser = Self::ensure_fundraiser(offering_asset, id)?;

            ensure!(fundraiser.status == FundraiserStatus::Live, Error::<T>::FundraiserNotLive);

            let now = Timestamp::<T>::get();
            ensure!(
                fundraiser.start <= now && fundraiser.end.filter(|e| now >= *e).is_none(),
                Error::<T>::FundraiserExpired
            );

            // Remaining tokens to fulfil the investment amount
            let mut remaining = purchase_amount;
            // Total cost to to fulfil the investment amount.
            // Primary use is to calculate the blended price (offering_token_amount / cost).
            // Blended price must be <= to max_price or the investment will fail.
            let mut cost = Balance::from(0u32);

            // Price is entered as a multiple of 1_000_000
            // i.e. a price of 1 unit is 1_000_000
            // a price of 1.5 units is 1_500_00
            let price_divisor = Balance::from(1_000_000u32);
            // Individual purchases from each tier that accumulate to fulfil the investment amount.
            // Tuple of (tier_id, amount to purchase from that tier).
            let mut purchases = Vec::new();

            for (id, tier) in fundraiser.tiers.iter().enumerate().filter(|(_, tier)| tier.remaining > 0u32.into()) {
                // fulfilled the investment amount
                if remaining == 0u32.into() {
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
                    .checked_mul(tier.price)
                    .ok_or(Error::<T>::Overflow)?
                    .checked_div(price_divisor)
                    .and_then(|pa| cost.checked_add(pa))
                    .ok_or(Error::<T>::Overflow)?;
            }

            ensure!(remaining == 0u32.into(), Error::<T>::InsufficientTokensRemaining);
            ensure!(cost >= fundraiser.minimum_investment, Error::<T>::InvestmentAmountTooLow);
            ensure!(
                max_price.map(|max_price| cost <= max_price.saturating_mul(purchase_amount) / price_divisor).unwrap_or(true),
                Error::<T>::MaxPriceExceeded
            );

            let legs = vec![
                LegV2 {
                    from: fundraiser.offering_portfolio,
                    to: investment_portfolio,
                    asset: LegAsset::Fungible { ticker: fundraiser.offering_asset, amount: purchase_amount }
                },
                LegV2 {
                    from: funding_portfolio,
                    to: fundraiser.raising_portfolio,
                    asset: LegAsset::Fungible { ticker: fundraiser.raising_asset, amount: cost }
                }
            ];

            with_transaction(|| {
                <Portfolio<T>>::unlock_tokens(&fundraiser.offering_portfolio, &fundraiser.offering_asset, purchase_amount)?;

                let instruction_id = Settlement::<T>::base_add_instruction(
                    fundraiser.creator,
                    fundraiser.venue_id,
                    SettlementType::SettleOnAffirmation,
                    None,
                    None,
                    legs,
                    None,
                    true
                )?;

                let portfolios = [fundraiser.offering_portfolio, fundraiser.raising_portfolio].iter().copied().collect::<BTreeSet<_>>();
                Settlement::<T>::unsafe_affirm_instruction(fundraiser.creator, instruction_id, portfolios, 1, None, None)?;

                let portfolios = vec![investment_portfolio, funding_portfolio];
                Settlement::<T>::affirm_and_execute_instruction(
                    origin,
                    instruction_id,
                    receipt,
                    portfolios,
                    2,
                    None
                )
            })?;

            for (id, amount) in purchases {
                fundraiser.tiers[id].remaining -= amount;
            }

            Self::deposit_event(RawEvent::Invested(did, id, offering_asset, fundraiser.raising_asset, purchase_amount, cost));
            <Fundraisers<T>>::insert(offering_asset, id, fundraiser);
        }

        /// Freeze a fundraiser.
        ///
        /// * `offering_asset` - Asset to freeze.
        /// * `id` - ID of the fundraiser to freeze.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::freeze_fundraiser()]
        pub fn freeze_fundraiser(origin, offering_asset: Ticker, id: FundraiserId) -> DispatchResult {
            Self::set_frozen(origin, offering_asset, id, true)
        }

        /// Unfreeze a fundraiser.
        ///
        /// * `offering_asset` - Asset to unfreeze.
        /// * `id` - ID of the fundraiser to unfreeze.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::unfreeze_fundraiser()]
        pub fn unfreeze_fundraiser(origin, offering_asset: Ticker, id: FundraiserId) -> DispatchResult {
            Self::set_frozen(origin, offering_asset, id, false)
        }

        /// Modify the time window a fundraiser is active
        ///
        /// * `offering_asset` - Asset to modify.
        /// * `id` - ID of the fundraiser to modify.
        /// * `start` - New start of the fundraiser.
        /// * `end` - New end of the fundraiser to modify.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::modify_fundraiser_window()]
        pub fn modify_fundraiser_window(
            origin,
            offering_asset: Ticker,
            id: FundraiserId,
            start: T::Moment,
            end: Option<T::Moment>,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, offering_asset)?.for_event();

            <Fundraisers<T>>::try_mutate(offering_asset, id, |fundraiser| {
                let fundraiser = fundraiser.as_mut().ok_or(Error::<T>::FundraiserNotFound)?;
                ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);
                if let Some(end) = fundraiser.end {
                    ensure!(Timestamp::<T>::get() < end, Error::<T>::FundraiserExpired);
                }
                if let Some(end) = end {
                    ensure!(start < end, Error::<T>::InvalidOfferingWindow);
                }
                Self::deposit_event(RawEvent::FundraiserWindowModified(did, id, fundraiser.start, fundraiser.end, start, end));
                fundraiser.start = start;
                fundraiser.end = end;
                Ok(())
            })
        }

        /// Stop a fundraiser.
        ///
        /// * `offering_asset` - Asset to stop.
        /// * `id` - ID of the fundraiser to stop.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::stop()]
        pub fn stop(origin, offering_asset: Ticker, id: FundraiserId) {
            let mut fundraiser = Self::ensure_fundraiser(offering_asset, id)?;

            let did = <ExternalAgents<T>>::ensure_asset_perms(origin, &offering_asset)?.primary_did;
            if fundraiser.creator != did {
                 <ExternalAgents<T>>::ensure_agent_permissioned(offering_asset, did)?;
            }

            ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);

            let remaining_amount: Balance = fundraiser.tiers
                .iter()
                .map(|t| t.remaining)
                .fold(0u32.into(), |remaining, x| remaining + x);

            <Portfolio<T>>::unlock_tokens(&fundraiser.offering_portfolio, &fundraiser.offering_asset, remaining_amount)?;
            fundraiser.status = match fundraiser.end {
                Some(end) if end > Timestamp::<T>::get() => FundraiserStatus::ClosedEarly,
                _ => FundraiserStatus::Closed,
            };
            <Fundraisers<T>>::insert(offering_asset, id, fundraiser);
            Self::deposit_event(RawEvent::FundraiserClosed(did, id));
        }
    }
}

impl<T: Config> Module<T> {
    fn set_frozen(
        origin: T::RuntimeOrigin,
        offering_asset: Ticker,
        id: FundraiserId,
        frozen: bool,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, offering_asset)?;
        let mut fundraiser = Self::ensure_fundraiser(offering_asset, id)?;
        ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);
        if frozen {
            fundraiser.status = FundraiserStatus::Frozen;
            Self::deposit_event(RawEvent::FundraiserFrozen(did, id));
        } else {
            fundraiser.status = FundraiserStatus::Live;
            Self::deposit_event(RawEvent::FundraiserUnfrozen(did, id));
        }
        <Fundraisers<T>>::insert(offering_asset, id, fundraiser);
        Ok(())
    }

    fn ensure_fundraiser(
        ticker: Ticker,
        id: FundraiserId,
    ) -> Result<Fundraiser<T::Moment>, DispatchError> {
        Fundraisers::<T>::get(ticker, id).ok_or_else(|| Error::<T>::FundraiserNotFound.into())
    }
}
