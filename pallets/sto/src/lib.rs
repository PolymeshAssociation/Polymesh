// Copyright (c) 2020 Polymesh Association

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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use frame_support::ensure;
use frame_support::weights::Weight;
use frame_system::pallet_prelude::OriginFor;
use polymesh_primitives::crypto::verify_signature;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

use pallet_base::try_next_post;
use pallet_identity::PermissionedCallOriginData;
use pallet_settlement::VenueInfo;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::settlement::{Leg, ReceiptDetails, SettlementType, VenueId, VenueType};
use polymesh_primitives::sto::{FundraiserId, FundraiserReceipt, FundraiserReceiptDetails};
use polymesh_primitives::{
    storage_migration_ver, traits::PortfolioSubTrait, Balance, EventDid, IdentityId, PortfolioId,
    Ticker,
};
use polymesh_primitives_derive::VecU8StrongTyped;

storage_migration_ver!(1);

pub const MAX_TIERS: usize = 10;

type ExternalAgents<T> = pallet_external_agents::Pallet<T>;
type Identity<T> = pallet_identity::Pallet<T>;
type Portfolio<T> = pallet_portfolio::Pallet<T>;
type Settlement<T> = pallet_settlement::Pallet<T>;
type Timestamp<T> = pallet_timestamp::Pallet<T>;

/// Status of a Fundraiser.
#[derive(
    Clone,
    PartialEq,
    Eq,
    Encode,
    Decode,
    TypeInfo,
    MaxEncodedLen,
    PartialOrd,
    Ord,
    Debug
)]
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

/// Funding method.  On-chain asset or off-chain receipt.
pub enum FundingMethod<AccountId, OffChainSignature> {
    /// On-chain asset.
    OnChain(PortfolioId),
    /// Off-chain receipt.
    OffChain(FundraiserReceiptDetails<AccountId, OffChainSignature>),
}

/// Which funding asset was used to invest in the fundraiser.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum FundingAsset {
    /// On-chain asset.
    OnChain(AssetId),
    /// Off-chain receipt.
    OffChain(Ticker),
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
    pub offering_asset: AssetId,
    /// Portfolio receiving funds raised.
    pub raising_portfolio: PortfolioId,
    /// Asset to receive payment in.
    pub raising_asset: AssetId,
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
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
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
    fn invest_with_receipt() -> Weight;
    fn freeze_fundraiser() -> Weight;
    fn unfreeze_fundraiser() -> Weight;
    fn modify_fundraiser_window() -> Weight;
    fn stop() -> Weight;
    fn enable_offchain_funding() -> Weight;
}

// re-export pallet types.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::{ValueQuery, *};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_identity::Config
        + pallet_settlement::Config
        + pallet_portfolio::Config
        + pallet_base::Config
    {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new fundraiser has been created.
        /// (Agent DID, fundraiser id, fundraiser name, fundraiser details)
        FundraiserCreated(
            IdentityId,
            FundraiserId,
            FundraiserName,
            Fundraiser<T::Moment>,
        ),
        /// An investor invested in the fundraiser.
        /// (Investor, fundraiser_id, offering token, raise token, offering_token_amount, raise_token_amount)
        Invested(IdentityId, FundraiserId, AssetId, AssetId, Balance, Balance),
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
            T::Moment,
            Option<T::Moment>,
            T::Moment,
            Option<T::Moment>,
        ),
        /// A fundraiser has been stopped.
        /// (Agent DID, fundraiser id)
        FundraiserClosed(IdentityId, FundraiserId),
        /// A fundraiser has enabled off-chain funding.
        /// (Agent DID, fundraiser id, ticker)
        FundraiserOffchainFundingEnabled(IdentityId, FundraiserId, Ticker),
        /// An investor invested in the fundraiser.
        /// (Investor, fundraiser_id, offering token, raise token, offering_token_amount, raise_token_amount)
        InvestedV2(
            IdentityId,
            FundraiserId,
            AssetId,
            FundingAsset,
            Balance,
            Balance,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
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
        InvestmentAmountTooLow,
        /// Invalid receipt signature.
        InvalidSignature,
        /// Off-chain funding is not allowed for this fundraiser.
        OffchainFundingNotAllowed,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// All fundraisers that are currently running.
    /// (AssetId, fundraiser_id) -> Fundraiser
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Fundraisers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Twox64Concat,
        FundraiserId,
        Fundraiser<T::Moment>,
        OptionQuery,
    >;

    /// Total fundraisers created for a token.
    #[pallet::storage]
    pub type FundraiserCount<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, FundraiserId, ValueQuery>;

    /// Name for the Fundraiser. Only used offchain.
    /// (AssetId, fundraiser_id) -> Fundraiser name
    #[pallet::storage]
    #[pallet::unbounded]
    pub type FundraiserNames<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Twox64Concat,
        FundraiserId,
        FundraiserName,
        OptionQuery,
    >;

    /// If the fundraiser supports off-chain funding payments using receipts.
    #[pallet::storage]
    pub type FundraiserOffchainAsset<T: Config> =
        StorageMap<_, Twox64Concat, FundraiserId, Ticker, OptionQuery>;

    /// Storage migration version.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig;

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            StorageVersion::<T>::put(Version::new(1));
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            Weight::zero()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
        #[pallet::weight(<T as Config>::WeightInfo::create_fundraiser(tiers.len() as u32))]
        #[pallet::call_index(0)]
        pub fn create_fundraiser(
            origin: OriginFor<T>,
            offering_portfolio: PortfolioId,
            offering_asset: AssetId,
            raising_portfolio: PortfolioId,
            raising_asset: AssetId,
            tiers: Vec<PriceTier>,
            venue_id: VenueId,
            start: Option<T::Moment>,
            end: Option<T::Moment>,
            minimum_investment: Balance,
            fundraiser_name: FundraiserName,
        ) -> DispatchResult {
            pallet_base::ensure_string_limited::<T>(&fundraiser_name)?;

            let PermissionedCallOriginData {
                primary_did: did,
                secondary_key,
                ..
            } = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, offering_asset)?;

            VenueInfo::<T>::get(venue_id)
                .filter(|v| v.creator == did && v.venue_type == VenueType::Sto)
                .ok_or(Error::<T>::InvalidVenue)?;

            <Portfolio<T>>::ensure_portfolio_custody_and_permission(
                raising_portfolio,
                did,
                secondary_key.as_ref(),
            )?;
            <Portfolio<T>>::ensure_portfolio_custody_and_permission(
                offering_portfolio,
                did,
                secondary_key.as_ref(),
            )?;

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
            let mut seq = FundraiserCount::<T>::get(&offering_asset);
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
                minimum_investment,
            };

            FundraiserCount::<T>::insert(offering_asset, seq);
            Fundraisers::<T>::insert(offering_asset, id, fundraiser.clone());
            FundraiserNames::<T>::insert(offering_asset, id, fundraiser_name.clone());

            Self::deposit_event(Event::FundraiserCreated(
                did,
                id,
                fundraiser_name,
                fundraiser,
            ));

            Ok(())
        }

        /// Invest in a fundraiser.
        ///
        /// * `investment_portfolio` - Portfolio that `offering_asset` will be deposited in.
        /// * `funding_portfolio` - Portfolio that will fund the investment.
        /// * `offering_asset` - Asset to invest in.
        /// * `id` - ID of the fundraiser to invest in.
        /// * `purchase_amount` - Amount of `offering_asset` to purchase.
        /// * `max_price` - Maximum price to pay per unit of `offering_asset`, If `None`there are no constraints on price.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::invest())]
        #[pallet::call_index(1)]
        pub fn invest(
            origin: OriginFor<T>,
            investment_portfolio: PortfolioId,
            funding_portfolio: PortfolioId,
            offering_asset: AssetId,
            id: FundraiserId,
            purchase_amount: Balance,
            max_price: Option<Balance>,
            receipt: Option<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        ) -> DispatchResult {
            // Old receipts are not supported anymore.
            ensure!(receipt.is_none(), Error::<T>::Unauthorized);

            Self::base_invest(
                origin,
                investment_portfolio,
                FundingMethod::OnChain(funding_portfolio),
                offering_asset,
                id,
                purchase_amount,
                max_price,
            )?;
            Ok(())
        }

        /// Freeze a fundraiser.
        ///
        /// * `offering_asset` - Asset to freeze.
        /// * `id` - ID of the fundraiser to freeze.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::freeze_fundraiser())]
        #[pallet::call_index(2)]
        pub fn freeze_fundraiser(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            id: FundraiserId,
        ) -> DispatchResult {
            Self::set_frozen(origin, offering_asset, id, true)?;
            Ok(())
        }

        /// Unfreeze a fundraiser.
        ///
        /// * `offering_asset` - Asset to unfreeze.
        /// * `id` - ID of the fundraiser to unfreeze.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::unfreeze_fundraiser())]
        #[pallet::call_index(3)]
        pub fn unfreeze_fundraiser(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            id: FundraiserId,
        ) -> DispatchResult {
            Self::set_frozen(origin, offering_asset, id, false)?;
            Ok(())
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
        #[pallet::weight(<T as Config>::WeightInfo::modify_fundraiser_window())]
        #[pallet::call_index(4)]
        pub fn modify_fundraiser_window(
            origin: OriginFor<T>,
            offering_asset: AssetId,
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
                Self::deposit_event(Event::FundraiserWindowModified(
                    did,
                    id,
                    fundraiser.start,
                    fundraiser.end,
                    start,
                    end,
                ));
                fundraiser.start = start;
                fundraiser.end = end;
                Ok::<_, DispatchError>(())
            })?;

            Ok(())
        }

        /// Stop a fundraiser.
        ///
        /// * `offering_asset` - Asset to stop.
        /// * `id` - ID of the fundraiser to stop.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::stop())]
        #[pallet::call_index(5)]
        pub fn stop(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            id: FundraiserId,
        ) -> DispatchResult {
            let mut fundraiser = Self::ensure_fundraiser(offering_asset, id)?;

            let did = <ExternalAgents<T>>::ensure_asset_perms(origin, offering_asset)?.primary_did;
            if fundraiser.creator != did {
                <ExternalAgents<T>>::ensure_agent_permissioned(&offering_asset, did)?;
            }

            ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);

            let remaining_amount: Balance = fundraiser
                .tiers
                .iter()
                .map(|t| t.remaining)
                .fold(0, |remaining, x| remaining + x);

            <Portfolio<T>>::unlock_tokens(
                &fundraiser.offering_portfolio,
                &fundraiser.offering_asset,
                remaining_amount,
            )?;
            fundraiser.status = match fundraiser.end {
                Some(end) if end > Timestamp::<T>::get() => FundraiserStatus::ClosedEarly,
                _ => FundraiserStatus::Closed,
            };
            <Fundraisers<T>>::insert(offering_asset, id, fundraiser);
            Self::deposit_event(Event::FundraiserClosed(did, id));

            Ok(())
        }

        /// Enable support for off-chain funding.
        ///
        /// * `offering_asset` - Asset to enable off-chain funding for.
        /// * `id` - ID of the fundraiser to enable off-chain funding for.
        /// * `ticker` - Ticker of the asset to use for off-chain funding.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::enable_offchain_funding())]
        #[pallet::call_index(6)]
        pub fn enable_offchain_funding(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            id: FundraiserId,
            ticker: Ticker,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_asset_perms(origin, offering_asset)?.primary_did;

            let fundraiser = Self::ensure_fundraiser(offering_asset, id)?;
            if fundraiser.creator != did {
                <ExternalAgents<T>>::ensure_agent_permissioned(&offering_asset, did)?;
            }
            ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);

            FundraiserOffchainAsset::<T>::insert(id, ticker);

            Self::deposit_event(Event::FundraiserOffchainFundingEnabled(did, id, ticker));

            Ok(())
        }

        /// Invest in a fundraiser using an off-chain receipt.
        ///
        /// * `investment_portfolio` - Portfolio that `offering_asset` will be deposited in.
        /// * `offering_asset` - Asset to invest in.
        /// * `id` - ID of the fundraiser to invest in.
        /// * `purchase_amount` - Amount of `offering_asset` to purchase.
        /// * `max_price` - Maximum price to pay per unit of `offering_asset`, If `None`there are no constraints on price.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::invest_with_receipt())]
        #[pallet::call_index(7)]
        pub fn invest_with_receipt(
            origin: OriginFor<T>,
            investment_portfolio: PortfolioId,
            offering_asset: AssetId,
            id: FundraiserId,
            purchase_amount: Balance,
            max_price: Option<Balance>,
            receipt: FundraiserReceiptDetails<T::AccountId, T::OffChainSignature>,
        ) -> DispatchResult {
            Self::base_invest(
                origin,
                investment_portfolio,
                FundingMethod::OffChain(receipt),
                offering_asset,
                id,
                purchase_amount,
                max_price,
            )?;

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn base_invest(
        origin: OriginFor<T>,
        investment_portfolio: PortfolioId,
        funding: FundingMethod<T::AccountId, T::OffChainSignature>,
        offering_asset: AssetId,
        fundraiser_id: FundraiserId,
        purchase_amount: Balance,
        max_price: Option<Balance>,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            primary_did: investor_did,
            secondary_key,
            ..
        } = Identity::<T>::ensure_origin_call_permissions(origin.clone())?;

        <Portfolio<T>>::ensure_portfolio_custody_and_permission(
            investment_portfolio,
            investor_did,
            secondary_key.as_ref(),
        )?;

        let mut fundraiser = Self::ensure_fundraiser(offering_asset, fundraiser_id)?;

        ensure!(
            fundraiser.status == FundraiserStatus::Live,
            Error::<T>::FundraiserNotLive
        );

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

        for (id, tier) in fundraiser
            .tiers
            .iter()
            .enumerate()
            .filter(|(_, tier)| tier.remaining > 0)
        {
            // fulfilled the investment amount
            if remaining == 0 {
                break;
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

        ensure!(remaining == 0, Error::<T>::InsufficientTokensRemaining);
        ensure!(
            cost >= fundraiser.minimum_investment,
            Error::<T>::InvestmentAmountTooLow
        );
        ensure!(
            max_price
                .map(|max_price| cost <= max_price.saturating_mul(purchase_amount) / price_divisor)
                .unwrap_or(true),
            Error::<T>::MaxPriceExceeded
        );

        let mut fundraiser_portfolios = [fundraiser.offering_portfolio]
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut investor_portfolios = [investment_portfolio]
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut legs = vec![Leg::Fungible {
            sender: fundraiser.offering_portfolio,
            receiver: investment_portfolio,
            asset_id: fundraiser.offering_asset,
            amount: purchase_amount,
        }];
        let funding_asset = match funding {
            FundingMethod::OnChain(funding_portfolio) => {
                <Portfolio<T>>::ensure_portfolio_custody_and_permission(
                    funding_portfolio,
                    investor_did,
                    secondary_key.as_ref(),
                )?;
                fundraiser_portfolios.insert(fundraiser.raising_portfolio);
                investor_portfolios.insert(funding_portfolio);
                legs.push(Leg::Fungible {
                    sender: funding_portfolio,
                    receiver: fundraiser.raising_portfolio,
                    asset_id: fundraiser.raising_asset,
                    amount: cost,
                });
                FundingAsset::OnChain(fundraiser.raising_asset)
            }
            FundingMethod::OffChain(receipt_details) => {
                let ticker = FundraiserOffchainAsset::<T>::get(fundraiser_id)
                    .ok_or(Error::<T>::OffchainFundingNotAllowed)?;
                Settlement::<T>::mark_receipt_as_used(
                    fundraiser.venue_id,
                    &receipt_details.signer,
                    receipt_details.uid,
                )?;
                let receipt = FundraiserReceipt::new(
                    receipt_details.uid,
                    fundraiser_id,
                    investor_did,
                    fundraiser.raising_portfolio.did,
                    ticker,
                    cost,
                );
                ensure!(
                    verify_signature::<T, T::OffChainSignature, _>(
                        &receipt_details.signer,
                        &receipt_details.signature,
                        &receipt,
                        true,
                    ),
                    Error::<T>::InvalidSignature
                );
                FundingAsset::OffChain(ticker)
            }
        };
        log::error!(
            "STO legs = {:?}, investor_portfolios={:?}",
            legs,
            investment_portfolio
        );

        <Portfolio<T>>::unlock_tokens(
            &fundraiser.offering_portfolio,
            &fundraiser.offering_asset,
            purchase_amount,
        )?;

        let instruction_id = Settlement::<T>::base_add_instruction(
            fundraiser.creator,
            Some(fundraiser.venue_id),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs,
            None,
            None,
        )?;

        Settlement::<T>::unsafe_affirm_instruction(
            fundraiser.creator,
            instruction_id,
            fundraiser_portfolios,
            None,
            None,
        )?;

        Settlement::<T>::affirm_and_execute_instruction(
            origin,
            instruction_id,
            None,
            investor_portfolios,
            investor_did,
        )?;

        for (id, amount) in purchases {
            fundraiser.tiers[id].remaining -= amount;
        }

        Self::deposit_event(Event::Invested(
            investor_did,
            fundraiser_id,
            offering_asset,
            fundraiser.raising_asset,
            purchase_amount,
            cost,
        ));

        Self::deposit_event(Event::InvestedV2(
            investor_did,
            fundraiser_id,
            offering_asset,
            funding_asset,
            purchase_amount,
            cost,
        ));

        <Fundraisers<T>>::insert(offering_asset, fundraiser_id, fundraiser);

        Ok(())
    }

    fn set_frozen(
        origin: T::RuntimeOrigin,
        offering_asset: AssetId,
        id: FundraiserId,
        frozen: bool,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, offering_asset)?;
        let mut fundraiser = Self::ensure_fundraiser(offering_asset, id)?;
        ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);
        if frozen {
            fundraiser.status = FundraiserStatus::Frozen;
            Self::deposit_event(Event::FundraiserFrozen(did, id));
        } else {
            fundraiser.status = FundraiserStatus::Live;
            Self::deposit_event(Event::FundraiserUnfrozen(did, id));
        }
        <Fundraisers<T>>::insert(offering_asset, id, fundraiser);
        Ok(())
    }

    fn ensure_fundraiser(
        asset_id: AssetId,
        id: FundraiserId,
    ) -> Result<Fundraiser<T::Moment>, DispatchError> {
        Ok(Fundraisers::<T>::get(asset_id, id).ok_or_else(|| Error::<T>::FundraiserNotFound)?)
    }
}
