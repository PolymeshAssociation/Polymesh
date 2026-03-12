// Copyright (c) 2020 Polymesh Association

//! # Security Token Offering (STO) Pallet
//!
//! The STO pallet enables the creation and management of security token offerings on Polymesh.
//! It provides a comprehensive framework for conducting compliant fundraising activities
//! through tokenized securities.
//!
//! ## Overview
//!
//! This pallet allows authorized external agents to create and manage fundraisers for assets.
//! Fundraisers support sophisticated pricing models with multiple tiers, optional time windows,
//! and flexible payment methods including both on-chain and off-chain funding mechanisms.
//!
//! ### Key Features
//!
//! - **Tiered Pricing**: Support for up to 10 price tiers per fundraiser with different token amounts and prices
//! - **Flexible Timing**: Optional start and end times for fundraising periods
//! - **Multiple Payment Methods**: Accept payments via on-chain portfolios or off-chain receipts
//! - **Venue Integration**: Leverage Polymesh's settlement infrastructure for secure token transfers
//! - **Fine-grained Control**: Freeze, unfreeze, modify, and stop fundraisers as needed
//! - **Minimum Investment**: Enforce minimum investment thresholds per transaction
//! - **Price Protection**: Optional maximum price limits to protect investors
//!
//! ### Use Cases
//!
//! - **Primary Offerings**: Initial public offerings (IPOs) and private placements
//! - **Secondary Fundraising**: Follow-on offerings and rights issues  
//! - **Tokenized Asset Sales**: Real estate, commodities, or other asset-backed tokens
//! - **Compliance-First Fundraising**: KYC/AML compliant token distributions
//! - **Institutional Investment**: Professional investor participation through off-chain receipts
//!
//! ## Dispatchable Functions
//!
//! ### Fundraiser Management
//! - [`create_fundraiser`] - Create a new fundraiser with tiered pricing and time windows
//! - [`stop`] - Permanently stop a fundraiser and unlock remaining tokens
//! - [`freeze_fundraiser`] - Temporarily freeze a fundraiser to prevent new investments
//! - [`unfreeze_fundraiser`] - Resume a frozen fundraiser
//! - [`modify_fundraiser_window`] - Update the active time window for a fundraiser
//! - [`enable_offchain_funding`] - Enable off-chain payment support for a fundraiser
//!
//! ### Investment
//! - [`invest`] - Invest in a fundraiser using on-chain or off-chain funding
//!
//! ## Permissions
//!
//! All fundraiser management functions require appropriate asset permissions through the
//! External Agents pallet. Investment functions require portfolio custody permissions.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use frame_support::ensure;
use frame_support::weights::Weight;
use frame_system::pallet_prelude::OriginFor;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

use pallet_base::try_next_post;
use pallet_identity::PermissionedCallOriginData;
use pallet_settlement::VenueInfo;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::crypto::{ChainScopedMessage, STO_FUNDRAISER_RECEIPT_LABEL};
use polymesh_primitives::settlement::{Leg, SettlementType, VenueId, VenueType};
use polymesh_primitives::sto::{FundraiserId, FundraiserReceipt, FundraiserReceiptDetails};
use polymesh_primitives::{
    storage_migration_ver, AssetHolder, Balance, EventDid, IdentityId, PortfolioId, Ticker,
};
use polymesh_primitives_derive::VecU8StrongTyped;

storage_migration_ver!(1);

pub const MAX_TIERS: usize = 10;

type Asset<T> = pallet_asset::Pallet<T>;
type ExternalAgents<T> = pallet_external_agents::Pallet<T>;
type Identity<T> = pallet_identity::Pallet<T>;
type Portfolio<T> = pallet_portfolio::Pallet<T>;
type Settlement<T> = pallet_settlement::Pallet<T>;
type Timestamp<T> = pallet_timestamp::Pallet<T>;

/// Status of a Fundraiser.
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo)]
#[derive(Clone, Debug, Default, Eq, MaxEncodedLen, PartialEq, PartialOrd, Ord)]
pub enum FundraiserStatus {
    /// Fundraiser is open for investments if start_time <= current_time < end_time.
    Live,
    /// Fundraiser has been frozen, New investments can not be made right now.
    Frozen,
    /// Fundraiser has been stopped.
    #[default]
    Closed,
    /// Fundraiser has been stopped before expiry.
    ClosedEarly,
}

/// Funding method.  On-chain asset or off-chain receipt.
#[derive(Decode, DecodeWithMemTracking, Debug, Encode, TypeInfo)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FundingMethod<AccountId, OffChainSignature, Moment> {
    /// On-chain asset.
    OnChain(PortfolioId),
    /// Off-chain receipt.
    OffChain(FundraiserReceiptDetails<AccountId, OffChainSignature, Moment>),
}

impl<AccountId, OffChainSignature, Moment> FundingMethod<AccountId, OffChainSignature, Moment> {
    pub fn is_onchain(&self) -> bool {
        matches!(self, FundingMethod::OnChain(_))
    }
}

/// Which funding asset was used to invest in the fundraiser.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo)]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FundingAsset {
    /// On-chain asset.
    OnChain(AssetId),
    /// Off-chain receipt.
    OffChain(Ticker),
}

/// Details about the Fundraiser.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo)]
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo)]
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PriceTier {
    /// Total amount available.
    pub total: Balance,
    /// Price per unit.
    pub price: Balance,
}

/// Single price tier of a `Fundraiser`.
/// Similar to a `PriceTier` but with an extra field `remaining` for tracking the amount available for purchase in a tier.
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FundraiserName(Vec<u8>);

pub trait WeightInfo {
    fn create_fundraiser(i: u32) -> Weight;
    fn invest_onchain() -> Weight;
    fn invest_offchain() -> Weight;
    fn freeze_fundraiser() -> Weight;
    fn unfreeze_fundraiser() -> Weight;
    fn modify_fundraiser_window() -> Weight;
    fn stop() -> Weight;
    fn enable_offchain_funding() -> Weight;

    fn invest(onchain: bool) -> Weight {
        if onchain {
            Self::invest_onchain()
        } else {
            Self::invest_offchain()
        }
    }
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
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new fundraiser has been created.
        ///
        /// [agent_did, offering_asset, raising_asset, fundraiser_id, fundraiser_name, fundraiser]
        FundraiserCreated {
            /// Identity of the external agent who created the fundraiser.
            agent_did: IdentityId,
            /// Asset being offered for sale in the fundraiser.
            offering_asset: AssetId,
            /// Asset being accepted as payment in the fundraiser.
            raising_asset: AssetId,
            /// Unique identifier for the fundraiser.
            fundraiser_id: FundraiserId,
            /// Human-readable name of the fundraiser.
            fundraiser_name: FundraiserName,
            /// Complete fundraiser configuration.
            fundraiser: Fundraiser<T::Moment>,
        },
        /// An investor successfully invested in the fundraiser.
        ///
        /// [investor_did, offering_asset, fundraiser_id, funding_asset, offering_amount, raise_amount]
        Invested {
            /// Identity of the investor.
            investor_did: IdentityId,
            /// Asset being purchased.
            offering_asset: AssetId,
            /// Fundraiser that was invested in.
            fundraiser_id: FundraiserId,
            /// Type of funding used (on-chain or off-chain).
            funding_asset: FundingAsset,
            /// Amount of offering asset purchased.
            offering_amount: Balance,
            /// Amount of raising asset spent.
            raise_amount: Balance,
        },
        /// A fundraiser has been frozen, preventing new investments.
        ///
        /// [agent_did, offering_asset, fundraiser_id]
        FundraiserFrozen {
            /// Identity of the external agent who froze the fundraiser.
            agent_did: IdentityId,
            /// Asset associated with the fundraiser.
            offering_asset: AssetId,
            /// Fundraiser that was frozen.
            fundraiser_id: FundraiserId,
        },
        /// A fundraiser has been unfrozen, allowing new investments.
        ///
        /// [agent_did, offering_asset, fundraiser_id]
        FundraiserUnfrozen {
            /// Identity of the external agent who unfroze the fundraiser.
            agent_did: IdentityId,
            /// Asset associated with the fundraiser.
            offering_asset: AssetId,
            /// Fundraiser that was unfrozen.
            fundraiser_id: FundraiserId,
        },
        /// A fundraiser's time window has been modified.
        ///
        /// [agent_did, offering_asset, fundraiser_id, old_start, old_end, new_start, new_end]
        FundraiserWindowModified {
            /// Identity of the external agent who modified the window.
            agent_did: EventDid,
            /// Asset associated with the fundraiser.
            offering_asset: AssetId,
            /// Fundraiser whose window was modified.
            fundraiser_id: FundraiserId,
            /// Previous start time.
            old_start: T::Moment,
            /// Previous end time (if any).
            old_end: Option<T::Moment>,
            /// New start time.
            new_start: T::Moment,
            /// New end time (if any).
            new_end: Option<T::Moment>,
        },
        /// A fundraiser has been permanently closed.
        ///
        /// [agent_did, offering_asset, fundraiser_id]
        FundraiserClosed {
            /// Identity of the external agent who closed the fundraiser.
            agent_did: IdentityId,
            /// Asset associated with the fundraiser.
            offering_asset: AssetId,
            /// Fundraiser that was closed.
            fundraiser_id: FundraiserId,
        },
        /// Off-chain funding has been enabled for a fundraiser.
        ///
        /// [agent_did, offering_asset, fundraiser_id, ticker]
        FundraiserOffchainFundingEnabled {
            /// Identity of the external agent who enabled off-chain funding.
            agent_did: IdentityId,
            /// Asset associated with the fundraiser.
            offering_asset: AssetId,
            /// Fundraiser for which off-chain funding was enabled.
            fundraiser_id: FundraiserId,
            /// Ticker symbol of the off-chain asset.
            ticker: Ticker,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Sender does not have required permissions for the requested operation.
        Unauthorized,
        /// An arithmetic operation resulted in overflow or underflow.
        Overflow,
        /// The fundraiser does not have enough tokens remaining to fulfil the investment.
        InsufficientTokensRemaining,
        /// The specified fundraiser does not exist for the given asset.
        FundraiserNotFound,
        /// The fundraiser is not in a live state (either frozen or stopped).
        FundraiserNotLive,
        /// The fundraiser has been permanently closed or stopped.
        FundraiserClosed,
        /// Attempting to interact with a fundraiser after its end time has passed.
        FundraiserExpired,
        /// The provided venue is invalid (does not exist, wrong type, or wrong creator).
        InvalidVenue,
        /// One or more price tiers have invalid parameters (zero total, too many tiers, etc.).
        InvalidPriceTiers,
        /// The fundraiser time window has invalid parameters (start time after end time).
        InvalidOfferingWindow,
        /// The calculated price per token exceeds the maximum price specified by the investor.
        MaxPriceExceeded,
        /// The investment amount is below the minimum investment threshold for this fundraiser.
        InvestmentAmountTooLow,
        /// The off-chain receipt signature is invalid or could not be verified.
        InvalidSignature,
        /// Off-chain funding has not been enabled for this fundraiser.
        OffchainFundingNotAllowed,
        /// The off-chain receipt has expired and can no longer be used for investment.
        ReceiptExpired,
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
    pub type FundraiserOffchainAsset<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AssetId,
        Twox64Concat,
        FundraiserId,
        Ticker,
        OptionQuery,
    >;

    /// Storage migration version.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
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
        /// Create a new fundraiser for a security token offering.
        ///
        /// This function creates a tiered pricing fundraiser where investors can purchase
        /// tokens at different price points. The fundraiser uses Polymesh's settlement
        /// infrastructure to ensure compliant and secure token transfers.
        ///
        /// # Parameters
        /// * `offering_portfolio` - Portfolio containing the tokens being offered for sale
        /// * `offering_asset` - Asset ID of the security token being sold
        /// * `raising_portfolio` - Portfolio that will receive the raised funds
        /// * `raising_asset` - Asset ID of the payment token (e.g., POLYX, stablecoin)
        /// * `tiers` - Vector of price tiers (1-10 tiers), each with total amount and price per unit
        /// * `venue_id` - STO venue ID for handling settlements (must be owned by caller)
        /// * `start` - Optional start time; if `None`, fundraiser begins immediately
        /// * `end` - Optional end time; if `None`, fundraiser runs indefinitely
        /// * `minimum_investment` - Minimum amount of `raising_asset` required per investment
        /// * `fundraiser_name` - Human-readable name for UI display (length limited)
        ///
        /// # Permissions Required
        /// * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
        /// * **Portfolio Custody**: Caller must have custody of both `offering_portfolio` and `raising_portfolio`
        /// * **Venue Ownership**: The specified `venue_id` must be an STO venue owned by the caller
        ///
        /// # Errors
        /// * `InvalidVenue` - Venue doesn't exist, wrong type, or not owned by caller
        /// * `InvalidPriceTiers` - Invalid tier configuration (0 tiers, >10 tiers, zero amounts)
        /// * `InvalidOfferingWindow` - Start time is after end time
        /// * `Overflow` - Total offering amount calculation overflowed
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
            } = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, &offering_asset)?;

            VenueInfo::<T>::get(venue_id)
                .filter(|v| v.creator == did && v.venue_type == VenueType::Sto)
                .ok_or(Error::<T>::InvalidVenue)?;

            <Portfolio<T>>::ensure_portfolio_custody_and_permission(
                &raising_portfolio,
                did,
                secondary_key.as_ref(),
            )?;
            <Portfolio<T>>::ensure_portfolio_custody_and_permission(
                &offering_portfolio,
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
            let fundraiser_id = try_next_post::<T, _>(&mut seq)?;

            Portfolio::<T>::lock_asset_balance(
                offering_portfolio.clone(),
                offering_asset,
                offering_amount,
            )?;

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
            Fundraisers::<T>::insert(offering_asset, fundraiser_id, fundraiser.clone());
            FundraiserNames::<T>::insert(offering_asset, fundraiser_id, fundraiser_name.clone());

            Self::deposit_event(Event::FundraiserCreated {
                agent_did: did,
                offering_asset,
                raising_asset,
                fundraiser_id,
                fundraiser_name,
                fundraiser,
            });

            Ok(())
        }

        /// Invest in a fundraiser using on-chain or off-chain funding.
        ///
        /// This function allows investors to purchase tokens from an active fundraiser.
        /// The investment is processed through multiple price tiers in order, starting
        /// with the lowest-priced tier. The purchase creates a settlement instruction
        /// that transfers tokens and payment between the appropriate portfolios.
        ///
        /// # Parameters
        /// * `offering_asset` - Asset ID of the security token being purchased
        /// * `fundraiser_id` - Unique identifier of the fundraiser to invest in
        /// * `investment_portfolio` - Portfolio where purchased tokens will be deposited
        /// * `funding` - Payment method: either `OnChain(portfolio_id)` for on-chain assets
        ///   or `OffChain(receipt_details)` for off-chain receipts with signature verification
        /// * `purchase_amount` - Number of `offering_asset` tokens to purchase
        /// * `max_price` - Optional maximum price per token; if specified, investment fails
        ///   if the blended price across tiers exceeds this limit
        ///
        /// # Permissions Required
        /// * **Portfolio Custody**: Caller must have custody of `investment_portfolio`
        /// * **Funding Portfolio**: If using on-chain funding, caller must have custody
        ///   of the funding portfolio specified in the `FundingMethod`
        ///
        /// # Errors
        /// * `FundraiserNotFound` - Specified fundraiser doesn't exist
        /// * `FundraiserNotLive` - Fundraiser is frozen or closed
        /// * `FundraiserExpired` - Current time is outside fundraiser's active window
        /// * `InsufficientTokensRemaining` - Not enough tokens available across all tiers
        /// * `InvestmentAmountTooLow` - Total cost is below minimum investment threshold
        /// * `MaxPriceExceeded` - Blended price exceeds investor's maximum price limit
        /// * `OffchainFundingNotAllowed` - Off-chain funding not enabled for this fundraiser
        /// * `InvalidSignature` - Off-chain receipt signature verification failed
        #[pallet::weight(<T as Config>::WeightInfo::invest(funding.is_onchain()))]
        #[pallet::call_index(1)]
        pub fn invest(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            fundraiser_id: FundraiserId,
            investment_portfolio: PortfolioId,
            funding: FundingMethod<T::AccountId, T::OffChainSignature, T::Moment>,
            purchase_amount: Balance,
            max_price: Option<Balance>,
        ) -> DispatchResult {
            Self::base_invest(
                origin,
                offering_asset,
                fundraiser_id,
                investment_portfolio,
                funding,
                purchase_amount,
                max_price,
            )
        }

        /// Temporarily freeze a fundraiser to prevent new investments.
        ///
        /// When a fundraiser is frozen, it cannot accept new investments but remains
        /// otherwise intact. This is useful for pausing activity while resolving
        /// issues or during maintenance periods. The fundraiser can be unfrozen
        /// later to resume normal operations.
        ///
        /// # Parameters
        /// * `offering_asset` - Asset ID associated with the fundraiser to freeze
        /// * `fundraiser_id` - Unique identifier of the fundraiser to freeze
        ///
        /// # Permissions Required
        /// * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
        ///
        /// # Errors
        /// * `FundraiserNotFound` - Specified fundraiser doesn't exist
        /// * `FundraiserClosed` - Fundraiser has already been permanently closed
        /// * `Unauthorized` - Caller lacks required asset agent permissions
        #[pallet::weight(<T as Config>::WeightInfo::freeze_fundraiser())]
        #[pallet::call_index(2)]
        pub fn freeze_fundraiser(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            fundraiser_id: FundraiserId,
        ) -> DispatchResult {
            Self::set_frozen(origin, offering_asset, fundraiser_id, true)?;
            Ok(())
        }

        /// Resume a frozen fundraiser to allow new investments.
        ///
        /// This function unfreezes a previously frozen fundraiser, returning it to
        /// the Live status where it can accept new investments. The fundraiser
        /// must not be permanently closed for this operation to succeed.
        ///
        /// # Parameters
        /// * `offering_asset` - Asset ID associated with the fundraiser to unfreeze
        /// * `fundraiser_id` - Unique identifier of the fundraiser to unfreeze
        ///
        /// # Permissions Required
        /// * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
        ///
        /// # Errors
        /// * `FundraiserNotFound` - Specified fundraiser doesn't exist
        /// * `FundraiserClosed` - Fundraiser has been permanently closed and cannot be unfrozen
        /// * `Unauthorized` - Caller lacks required asset agent permissions
        #[pallet::weight(<T as Config>::WeightInfo::unfreeze_fundraiser())]
        #[pallet::call_index(3)]
        pub fn unfreeze_fundraiser(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            fundraiser_id: FundraiserId,
        ) -> DispatchResult {
            Self::set_frozen(origin, offering_asset, fundraiser_id, false)?;
            Ok(())
        }

        /// Modify the time window when a fundraiser is active for investments.
        ///
        /// This function allows authorized agents to update the start and end times
        /// of an active fundraiser. This can be useful for extending fundraising
        /// periods, adjusting launch timing, or responding to market conditions.
        /// The fundraiser must not be permanently closed to modify its window.
        ///
        /// # Parameters
        /// * `offering_asset` - Asset ID associated with the fundraiser to modify
        /// * `fundraiser_id` - Unique identifier of the fundraiser to modify
        /// * `start` - New start time for the fundraiser (can be in the past or future)
        /// * `end` - New optional end time; if `None`, the fundraiser runs indefinitely
        ///
        /// # Permissions Required
        /// * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
        ///
        /// # Errors
        /// * `FundraiserNotFound` - Specified fundraiser doesn't exist
        /// * `FundraiserClosed` - Fundraiser has been permanently closed
        /// * `FundraiserExpired` - Fundraiser has already expired (past its original end time)
        /// * `InvalidOfferingWindow` - New start time is after new end time
        /// * `Unauthorized` - Caller lacks required asset agent permissions
        #[pallet::weight(<T as Config>::WeightInfo::modify_fundraiser_window())]
        #[pallet::call_index(4)]
        pub fn modify_fundraiser_window(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            fundraiser_id: FundraiserId,
            start: T::Moment,
            end: Option<T::Moment>,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, &offering_asset)?.for_event();

            <Fundraisers<T>>::try_mutate(offering_asset, fundraiser_id, |fundraiser| {
                let fundraiser = fundraiser.as_mut().ok_or(Error::<T>::FundraiserNotFound)?;
                ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);
                if let Some(end) = fundraiser.end {
                    ensure!(Timestamp::<T>::get() < end, Error::<T>::FundraiserExpired);
                }
                if let Some(end) = end {
                    ensure!(start < end, Error::<T>::InvalidOfferingWindow);
                }
                Self::deposit_event(Event::FundraiserWindowModified {
                    agent_did: did,
                    offering_asset,
                    fundraiser_id,
                    old_start: fundraiser.start,
                    old_end: fundraiser.end,
                    new_start: start,
                    new_end: end,
                });
                fundraiser.start = start;
                fundraiser.end = end;
                Ok::<_, DispatchError>(())
            })?;

            Ok(())
        }

        /// Permanently stop a fundraiser and unlock remaining tokens.
        ///
        /// This function permanently closes a fundraiser, preventing any further
        /// investments. Any remaining tokens that haven't been sold are unlocked
        /// and returned to the offering portfolio. Once stopped, a fundraiser
        /// cannot be restarted.
        ///
        /// # Parameters
        /// * `offering_asset` - Asset ID associated with the fundraiser to stop
        /// * `fundraiser_id` - Unique identifier of the fundraiser to stop
        ///
        /// # Permissions Required
        /// * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
        ///   OR be the original creator of the fundraiser
        ///
        /// # Errors
        /// * `FundraiserNotFound` - Specified fundraiser doesn't exist
        /// * `FundraiserClosed` - Fundraiser has already been permanently closed
        /// * `Unauthorized` - Caller lacks required permissions
        #[pallet::weight(<T as Config>::WeightInfo::stop())]
        #[pallet::call_index(5)]
        pub fn stop(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            fundraiser_id: FundraiserId,
        ) -> DispatchResult {
            let mut fundraiser = Self::ensure_fundraiser(offering_asset, fundraiser_id)?;

            let agent_did =
                <ExternalAgents<T>>::ensure_asset_perms(origin, &offering_asset)?.primary_did;
            if fundraiser.creator != agent_did {
                <ExternalAgents<T>>::ensure_agent_permissioned(&offering_asset, agent_did)?;
            }

            ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);

            let remaining_amount: Balance = fundraiser
                .tiers
                .iter()
                .map(|t| t.remaining)
                .fold(0, |remaining, x| remaining + x);

            Portfolio::<T>::unlock_asset_balance(
                fundraiser.offering_portfolio.clone(),
                fundraiser.offering_asset,
                remaining_amount,
            )?;
            fundraiser.status = match fundraiser.end {
                Some(end) if end > Timestamp::<T>::get() => FundraiserStatus::ClosedEarly,
                _ => FundraiserStatus::Closed,
            };
            <Fundraisers<T>>::insert(offering_asset, fundraiser_id, fundraiser);
            Self::deposit_event(Event::FundraiserClosed {
                agent_did,
                offering_asset,
                fundraiser_id,
            });

            Ok(())
        }

        /// Enable off-chain funding support for a fundraiser.
        ///
        /// This function allows a fundraiser to accept off-chain payments through
        /// cryptographically signed receipts. Once enabled, investors can use the
        /// `invest` function with `FundingMethod::OffChain` to provide payment
        /// receipts instead of on-chain portfolio transfers.
        ///
        /// # Parameters
        /// * `offering_asset` - Asset ID associated with the fundraiser
        /// * `fundraiser_id` - Unique identifier of the fundraiser to enable off-chain funding for
        /// * `ticker` - Ticker symbol of the off-chain asset that will be accepted as payment
        ///
        /// # Permissions Required
        /// * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
        ///   OR be the original creator of the fundraiser
        ///
        /// # Errors
        /// * `FundraiserNotFound` - Specified fundraiser doesn't exist
        /// * `FundraiserClosed` - Fundraiser has been permanently closed
        /// * `Unauthorized` - Caller lacks required permissions
        #[pallet::weight(<T as Config>::WeightInfo::enable_offchain_funding())]
        #[pallet::call_index(6)]
        pub fn enable_offchain_funding(
            origin: OriginFor<T>,
            offering_asset: AssetId,
            fundraiser_id: FundraiserId,
            ticker: Ticker,
        ) -> DispatchResult {
            let agent_did =
                <ExternalAgents<T>>::ensure_asset_perms(origin, &offering_asset)?.primary_did;

            let fundraiser = Self::ensure_fundraiser(offering_asset, fundraiser_id)?;
            if fundraiser.creator != agent_did {
                <ExternalAgents<T>>::ensure_agent_permissioned(&offering_asset, agent_did)?;
            }
            ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);

            FundraiserOffchainAsset::<T>::insert(offering_asset, fundraiser_id, ticker);

            Self::deposit_event(Event::FundraiserOffchainFundingEnabled {
                agent_did,
                offering_asset,
                fundraiser_id,
                ticker,
            });

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn base_invest(
        origin: OriginFor<T>,
        offering_asset: AssetId,
        fundraiser_id: FundraiserId,
        investment_portfolio: PortfolioId,
        funding: FundingMethod<T::AccountId, T::OffChainSignature, T::Moment>,
        purchase_amount: Balance,
        max_price: Option<Balance>,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            primary_did: investor_did,
            secondary_key,
            ..
        } = Identity::<T>::ensure_origin_call_permissions(origin.clone())?;

        Portfolio::<T>::ensure_portfolio_custody_and_permission(
            &investment_portfolio,
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
        // Primary use is to calculate the blended price (offering_amount / cost).
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

        let mut fundraiser_portfolios = [fundraiser.offering_portfolio.clone()]
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<AssetHolder>>();
        let mut investor_portfolios = BTreeSet::<AssetHolder>::new();
        // Check if investment_portfolio (receiver) requires affirmation.
        let investment_holder: AssetHolder = investment_portfolio.clone().into();
        if !Asset::<T>::skip_asset_holder_affirmation(
            &investment_holder,
            &fundraiser.offering_asset,
        )? {
            investor_portfolios.insert(investment_holder);
        }
        let mut legs = vec![Leg::Fungible {
            sender: fundraiser.offering_portfolio.clone().into(),
            receiver: investment_portfolio.into(),
            asset_id: fundraiser.offering_asset,
            amount: purchase_amount,
        }];
        let funding_asset = match funding {
            FundingMethod::OnChain(funding_portfolio) => {
                Portfolio::<T>::ensure_portfolio_custody_and_permission(
                    &funding_portfolio,
                    investor_did,
                    secondary_key.as_ref(),
                )?;
                investor_portfolios.insert(funding_portfolio.clone().into());
                // Check if raising_portfolio (receiver) requires affirmation.
                let raising_holder: AssetHolder = fundraiser.raising_portfolio.clone().into();
                if !Asset::<T>::skip_asset_holder_affirmation(
                    &raising_holder,
                    &fundraiser.raising_asset,
                )? {
                    fundraiser_portfolios.insert(raising_holder);
                }
                legs.push(Leg::Fungible {
                    sender: funding_portfolio.into(),
                    receiver: fundraiser.raising_portfolio.clone().into(),
                    asset_id: fundraiser.raising_asset,
                    amount: cost,
                });
                FundingAsset::OnChain(fundraiser.raising_asset)
            }
            FundingMethod::OffChain(receipt_details) => {
                let ticker = FundraiserOffchainAsset::<T>::get(&offering_asset, fundraiser_id)
                    .ok_or(Error::<T>::OffchainFundingNotAllowed)?;
                Settlement::<T>::mark_receipt_as_used(
                    fundraiser.venue_id,
                    &receipt_details.signer,
                    receipt_details.uid,
                )?;
                let receipt = ChainScopedMessage::<T, _>::new(
                    receipt_details.uid,
                    STO_FUNDRAISER_RECEIPT_LABEL,
                    receipt_details.expires_at,
                    FundraiserReceipt::new(
                        fundraiser_id,
                        investor_did,
                        fundraiser.raising_portfolio.did,
                        ticker,
                        cost,
                    ),
                )
                .ok_or(Error::<T>::ReceiptExpired)?;
                ensure!(
                    receipt.verify_signature(&receipt_details.signer, &receipt_details.signature),
                    Error::<T>::InvalidSignature
                );
                FundingAsset::OffChain(ticker)
            }
        };

        Portfolio::<T>::unlock_asset_balance(
            fundraiser.offering_portfolio.clone(),
            fundraiser.offering_asset,
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

        if !fundraiser_portfolios.is_empty() {
            Settlement::<T>::unsafe_affirm_instruction(
                fundraiser.creator,
                instruction_id,
                fundraiser_portfolios,
                None,
                None,
            )?;
        }

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

        Self::deposit_event(Event::Invested {
            investor_did,
            offering_asset,
            fundraiser_id,
            funding_asset,
            offering_amount: purchase_amount,
            raise_amount: cost,
        });

        <Fundraisers<T>>::insert(offering_asset, fundraiser_id, fundraiser);

        Ok(())
    }

    fn set_frozen(
        origin: T::RuntimeOrigin,
        offering_asset: AssetId,
        fundraiser_id: FundraiserId,
        frozen: bool,
    ) -> DispatchResult {
        let agent_did = <ExternalAgents<T>>::ensure_perms(origin, &offering_asset)?;
        let mut fundraiser = Self::ensure_fundraiser(offering_asset, fundraiser_id)?;
        ensure!(!fundraiser.is_closed(), Error::<T>::FundraiserClosed);
        if frozen {
            fundraiser.status = FundraiserStatus::Frozen;
            Self::deposit_event(Event::FundraiserFrozen {
                agent_did,
                offering_asset,
                fundraiser_id,
            });
        } else {
            fundraiser.status = FundraiserStatus::Live;
            Self::deposit_event(Event::FundraiserUnfrozen {
                agent_did,
                offering_asset,
                fundraiser_id,
            });
        }
        <Fundraisers<T>>::insert(offering_asset, fundraiser_id, fundraiser);
        Ok(())
    }

    fn ensure_fundraiser(
        asset_id: AssetId,
        id: FundraiserId,
    ) -> Result<Fundraiser<T::Moment>, DispatchError> {
        Ok(Fundraisers::<T>::get(asset_id, id).ok_or_else(|| Error::<T>::FundraiserNotFound)?)
    }
}
