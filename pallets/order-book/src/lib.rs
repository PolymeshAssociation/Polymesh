// Copyright (c) 2020 Polymath

//! # OrderBook Module
//!
//! Provide support for off-chain OrderBooks.
//!

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
use pallet_settlement::{Leg, SettlementType, VenueInfo, VenueType};
use polymesh_common_utilities::{
    portfolio::PortfolioSubTrait,
    traits::{identity, portfolio},
    with_transaction,
};
use scale_info::TypeInfo;

use frame_support::weights::Weight;
use polymesh_primitives::{order_book::*, Balance, IdentityId, PortfolioId, Ticker, VenueId};
use sp_runtime::DispatchError;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Identity<T> = pallet_identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;
type Settlement<T> = pallet_settlement::Module<T>;
type Timestamp<T> = pallet_timestamp::Pallet<T>;
type System<T> = frame_system::Pallet<T>;

/// Lock status.
/// TODO: Add support for partial unlocking.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LockStatus<BlockNumber: Copy + PartialOrd> {
    /// Assets are locked.
    Locked,
    /// Assets will be unlocked at `BlockNumber`.
    Unlocking(BlockNumber),
    /// Assets are unlocked and available for withdrawl.
    Unlocked,
}

impl<BlockNumber: Copy + PartialOrd> Default for LockStatus<BlockNumber> {
    fn default() -> Self {
        Self::Locked
    }
}

/// Locked assets.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LockedAsset<BlockNumber: Copy + PartialOrd> {
    /// Amount locked.
    pub amount: Balance,
    /// Lock status.
    pub status: LockStatus<BlockNumber>,
}

impl<BlockNumber: Copy + PartialOrd> LockedAsset<BlockNumber> {
    pub fn is_locked(&self, block: BlockNumber) -> bool {
        match self.status {
            LockStatus::Unlocking(at) if at <= block => false,
            _ => true,
        }
    }
}

pub trait WeightInfo {
    fn create_orderbook() -> Weight;
}

pub trait Config:
    frame_system::Config
    + identity::Config
    + pallet_settlement::Config
    + portfolio::Config
    + pallet_base::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    /// Weight information for extrinsic of the sto pallet.
    type WeightInfo: WeightInfo;
}

decl_event!(
    pub enum Event<T>
    where
        Moment = <T as pallet_timestamp::Config>::Moment,
    {
        /// A new orderbook has been created.
        /// (Agent DID, orderbook id, orderbook)
        OrderBookCreated(IdentityId, OrderBookId, OrderBook),
        // TODO:
        Trade(
            IdentityId,
            OrderBookId,
            Ticker,
            Balance,
            Coin,
            Balance,
            Moment,
        ),
    }
);

decl_error! {
    /// Errors for the OrderBook module.
    pub enum Error for Module<T: Config> {
        /// Sender does not have required permissions.
        Unauthorized,
        /// An arithmetic operation overflowed.
        Overflow,
        /// An invalid Venue provided.
        InvalidVenue,
        /// An invalid OrderBook provided.
        InvalidOrderBook,
        /// Assets are still locked.
        StillLocked,
        /// No assets locked.
        NoLockedAssets,
        /// The matched orders can't be on the same side (buy/sell) as the main order.
        InvalidMatchedOrderSide,
        /// The matched orders must have a `price`.  Only the main order can be a market order.
        MatchedOrderMissingPrice,
        /// The order's `amount` is lower then the matched `amount`.
        InsufficientOrderAmount,
        /// Not enough locked assets to cover the order.
        InsufficientLockedAssets,
        /// Invalid order signature.
        InvalidOrderSignature,
        /// No matched orders.
        NoMatchedOrders,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as OrderBook {
        /// Next orderbook id for a venue.
        VenueNextOrderBookId get(fn venue_next_order_book_id):
            map hasher(twox_64_concat) VenueId
                => OrderBookId;

        /// OrderBooks
        OrderBooks get(fn orderbooks):
            map hasher(twox_64_concat) OrderBookId
                => Option<OrderBook>;

        /// Assets locked to an orderbook.
        OrderBookLockedAssets get(fn orderbook_locked_assets):
            double_map
                hasher(twox_64_concat) OrderBookId,
                hasher(twox_64_concat) (T::AccountId, PortfolioId, Ticker)
                => LockedAsset<T::BlockNumber>;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// TODO
        #[weight = 0]
        pub fn create_orderbook(
            origin,
            venue_id: VenueId,
            asset: Ticker,
            coin: Coin,
        ) -> DispatchResult {
            Self::base_create_orderbook(origin, venue_id, asset, coin)
        }

        /// TODO
        #[weight = 0]
        pub fn lock_asset_to_orderbook(
            origin,
            orderbook_id: OrderBookId,
            portfolio: PortfolioId,
            asset: Ticker,
            amount: Balance,
        ) -> DispatchResult {
            Self::base_lock_asset_to_orderbook(origin, orderbook_id, portfolio, asset, amount)
        }

        /// TODO
        #[weight = 0]
        pub fn unlock_asset_to_orderbook(
            origin,
            orderbook_id: OrderBookId,
            portfolio: PortfolioId,
            asset: Ticker,
        ) -> DispatchResult {
            Self::base_unlock_asset_to_orderbook(origin, orderbook_id, portfolio, asset)
        }

        /// TODO
        #[weight = 0]
        pub fn withdraw_unlocked_assets(
            origin,
            orderbook_id: OrderBookId,
            portfolio: PortfolioId,
            asset: Ticker,
        ) -> DispatchResult {
            Self::base_withdraw_unlocked_assets(origin, orderbook_id, portfolio, asset)
        }

        /// TODO
        #[weight = 0]
        pub fn settle_off_chain_orders(
            origin,
            orderbook_id: OrderBookId,
            orders: SettleOrders<T::AccountId>,
        ) -> DispatchResult {
            Self::base_settle_off_chain_orders(origin, orderbook_id, orders)
        }
    }
}

impl<T: Config> Module<T> {
    fn base_create_orderbook(
        origin: T::Origin,
        venue_id: VenueId,
        asset: Ticker,
        coin: Coin,
    ) -> DispatchResult {
        let did = Identity::<T>::ensure_perms(origin)?;

        // Only the venue creator can add a new orderbook to that venue.
        VenueInfo::get(venue_id)
            .filter(|v| v.creator == did && v.venue_type == VenueType::OrderBook)
            .ok_or(Error::<T>::InvalidVenue)?;

        // Get the next orderbook ID.
        let mut seq = VenueNextOrderBookId::get(&venue_id);
        let id = try_next_post::<T, _>(&mut seq)?;
        VenueNextOrderBookId::insert(venue_id, seq);

        let orderbook = OrderBook {
            venue_id,
            asset,
            coin,
        };

        OrderBooks::insert(id, &orderbook);

        Self::deposit_event(RawEvent::OrderBookCreated(did, id, orderbook));
        Ok(())
    }

    fn base_lock_asset_to_orderbook(
        origin: T::Origin,
        orderbook_id: OrderBookId,
        portfolio: PortfolioId,
        asset: Ticker,
        amount: Balance,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            primary_did: did,
            secondary_key,
            sender,
        } = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, asset)?;

        <Portfolio<T>>::ensure_portfolio_custody_and_permission(
            portfolio,
            did,
            secondary_key.as_ref(),
        )?;

        // Check if orderbook exists.
        ensure!(
            OrderBooks::contains_key(orderbook_id),
            Error::<T>::InvalidOrderBook
        );

        with_transaction(|| -> DispatchResult {
            <Portfolio<T>>::lock_tokens(&portfolio, &asset, amount)?;
            OrderBookLockedAssets::<T>::try_mutate(
                orderbook_id,
                (sender, &portfolio, &asset),
                |assets| {
                    // Make sure to cancel any pending unlocking.
                    assets.status = LockStatus::Locked;
                    assets.amount = assets
                        .amount
                        .checked_add(amount)
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(())
                },
            )
        })?;
        // TODO: event
        //Self::deposit_event(RawEvent::AssetsLocked(did, ));
        Ok(())
    }

    fn base_unlock_asset_to_orderbook(
        origin: T::Origin,
        orderbook_id: OrderBookId,
        portfolio: PortfolioId,
        asset: Ticker,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            primary_did: did,
            secondary_key,
            sender,
        } = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, asset)?;

        <Portfolio<T>>::ensure_portfolio_custody_and_permission(
            portfolio,
            did,
            secondary_key.as_ref(),
        )?;

        // TODO: unlock period.
        let unlock_at = System::<T>::block_number() + 100u32.into();
        OrderBookLockedAssets::<T>::mutate(orderbook_id, (sender, &portfolio, &asset), |assets| {
            // Make sure to cancel any pending unlocking.
            assets.status = LockStatus::Unlocking(unlock_at);
        });
        // TODO: event
        //Self::deposit_event(RawEvent::AssetsLocked(did, ));
        Ok(())
    }

    fn base_withdraw_unlocked_assets(
        origin: T::Origin,
        orderbook_id: OrderBookId,
        portfolio: PortfolioId,
        asset: Ticker,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            primary_did: did,
            secondary_key,
            sender,
        } = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, asset)?;

        <Portfolio<T>>::ensure_portfolio_custody_and_permission(
            portfolio,
            did,
            secondary_key.as_ref(),
        )?;

        let block = System::<T>::block_number();
        OrderBookLockedAssets::<T>::try_mutate_exists(
            orderbook_id,
            (sender, &portfolio, &asset),
            |locked_assets| -> DispatchResult {
                match locked_assets {
                    Some(assets) if assets.is_locked(block) => Err(Error::<T>::StillLocked)?,
                    Some(assets) => {
                        <Portfolio<T>>::unlock_tokens(&portfolio, &asset, assets.amount)?;
                        *locked_assets = None;
                        Ok(())
                    }
                    None => Err(Error::<T>::NoLockedAssets)?,
                }
            },
        )?;
        // TODO: event
        //Self::deposit_event(RawEvent::AssetsLocked(did, ));
        Ok(())
    }

    fn base_settle_off_chain_orders(
        origin: T::Origin,
        orderbook_id: OrderBookId,
        orders: SettleOrders<T::AccountId>,
    ) -> DispatchResult {
        let venue_creator = Identity::<T>::ensure_perms(origin.clone())?;

        // Get orderbook details.
        let orderbook = OrderBooks::get(orderbook_id).ok_or(Error::<T>::InvalidOrderBook)?;

        // Only the venue creator can settle orders.
        VenueInfo::get(orderbook.venue_id)
            .filter(|v| v.creator == venue_creator && v.venue_type == VenueType::OrderBook)
            .ok_or(Error::<T>::InvalidVenue)?;

        let _now = Timestamp::<T>::get();

        // Ensure that there is at least one matched order.
        let matched_orders_len = orders.matched_orders.len();
        ensure!(matched_orders_len > 0, Error::<T>::NoMatchedOrders);

        with_transaction(|| {
            // Validate matched orders.
            let mut total_amount: Balance = 0;
            let mut total_price: Balance = 0;
            let mut legs = Vec::with_capacity(matched_orders_len * 2);
            let mut portfolios = BTreeSet::new();
            let main = &orders.main_order.order;

            portfolios.insert(main.asset_portfolio);
            if let Some(coin_portfolio) = main.coin_portfolio {
                portfolios.insert(coin_portfolio);
            }

            for order in &orders.matched_orders {
                let (order, amount, price) =
                    Self::ensure_valid_matched_order(&orderbook, orderbook_id, order, main)?;
                total_amount = total_amount
                    .checked_add(amount)
                    .ok_or(Error::<T>::Overflow)?;
                total_price = total_price.checked_add(price).ok_or(Error::<T>::Overflow)?;

                portfolios.insert(order.asset_portfolio);
                // Create lets for settlement.
                match (main.side, &orderbook.coin) {
                    (OrderSide::Buy, Coin::Asset(coin)) => {
                        let from = main
                            .coin_portfolio
                            .ok_or(Error::<T>::InsufficientLockedAssets)?;
                        let to = order
                            .coin_portfolio
                            .ok_or(Error::<T>::InsufficientLockedAssets)?;
                        portfolios.insert(to);
                        // Buyer sends `price` coins.
                        legs.push(Leg {
                            from,
                            to,
                            asset: coin.clone(),
                            amount: price,
                        });
                        // Seller sends `amount` assets.
                        legs.push(Leg {
                            from: order.asset_portfolio,
                            to: main.asset_portfolio,
                            asset: orderbook.asset,
                            amount,
                        });
                    }
                    (OrderSide::Sell, Coin::Asset(coin)) => {
                        let from = order
                            .coin_portfolio
                            .ok_or(Error::<T>::InsufficientLockedAssets)?;
                        let to = main
                            .coin_portfolio
                            .ok_or(Error::<T>::InsufficientLockedAssets)?;
                        portfolios.insert(from);
                        // Buyer sends `price` coins.
                        legs.push(Leg {
                            from,
                            to,
                            asset: coin.clone(),
                            amount: price,
                        });
                        // Seller sends `amount` assets.
                        legs.push(Leg {
                            from: main.asset_portfolio,
                            to: order.asset_portfolio,
                            asset: orderbook.asset,
                            amount,
                        });
                    }
                    (OrderSide::Buy, Coin::Offchain(_coin)) => {
                        // Seller sends `amount` assets.
                        legs.push(Leg {
                            from: order.asset_portfolio,
                            to: main.asset_portfolio,
                            asset: orderbook.asset,
                            amount,
                        });
                    }
                    (OrderSide::Sell, Coin::Offchain(_coin)) => {
                        // Seller sends `amount` assets.
                        legs.push(Leg {
                            from: main.asset_portfolio,
                            to: order.asset_portfolio,
                            asset: orderbook.asset,
                            amount,
                        });
                    }
                }
            }
            // Validate main order.
            Self::ensure_valid_main_order(
                &orderbook,
                orderbook_id,
                &orders.main_order,
                total_amount,
                total_price,
            )?;

            let num_legs = legs.len() as u32;
            let instruction_id = Settlement::<T>::base_add_instruction(
                venue_creator,
                orderbook.venue_id,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs,
            )?;

            // TODO: Improve affirm handling.
            for portfolio in &portfolios {
                let did = portfolio.did;
                let portfolios = [*portfolio].iter().copied().collect::<BTreeSet<_>>();
                Settlement::<T>::unsafe_affirm_instruction(
                    did,
                    instruction_id,
                    portfolios,
                    num_legs,
                    None,
                )?;
            }

            let portfolios = vec![];
            Settlement::<T>::affirm_and_execute_instruction(
                origin,
                instruction_id,
                None,
                portfolios,
                0,
            )
        })?;

        Ok(())
    }

    fn unlock_assets(
        orderbook_id: OrderBookId,
        account_id: &T::AccountId,
        portfolio: PortfolioId,
        asset: Ticker,
        amount: Balance,
    ) -> DispatchResult {
        // TODO: Only read `block_number` once.
        let block = System::<T>::block_number();
        OrderBookLockedAssets::<T>::try_mutate_exists(
            orderbook_id,
            (account_id, &portfolio, &asset),
            |locked_assets| -> DispatchResult {
                match locked_assets.as_mut() {
                    Some(assets) if !assets.is_locked(block) => {
                        // Assets are unlocked.
                        // TODO: Use different error.
                        Err(Error::<T>::InsufficientLockedAssets)?
                    }
                    Some(mut assets) => {
                        assets.amount = assets
                            .amount
                            .checked_sub(amount)
                            .ok_or(Error::<T>::InsufficientLockedAssets)?;
                        <Portfolio<T>>::unlock_tokens(&portfolio, &asset, amount)?;
                        Ok(())
                    }
                    None => Err(Error::<T>::NoLockedAssets)?,
                }
            },
        )?;
        Ok(())
    }

    fn ensure_signed_order(
        signed_order: &SignedOrder<T::AccountId>,
    ) -> Result<&Order<T::AccountId>, DispatchError> {
        // Verify that the order was correctly signed.
        ensure!(signed_order.verify(), Error::<T>::InvalidOrderSignature);
        // unwrap `SignedOrder` -> `Order`
        Ok(&signed_order.order)
    }

    fn ensure_valid_matched_order<'a>(
        orderbook: &OrderBook,
        orderbook_id: OrderBookId,
        matched_order: &'a MatchedOrder<T::AccountId>,
        main_order: &Order<T::AccountId>,
    ) -> Result<(&'a Order<T::AccountId>, Balance, Balance), DispatchError> {
        let order = Self::ensure_signed_order(&matched_order.order)?;

        // Main order side can't be the same as the matched orders.
        ensure!(
            main_order.side != order.side,
            Error::<T>::InvalidMatchedOrderSide
        );

        // The matched orders must have a price.  Only the main order can be a Market order.
        let price = order.price.ok_or(Error::<T>::MatchedOrderMissingPrice)?;
        let amount = matched_order.amount;

        // Ensure the main order price correctly matches the other orders.
        match (main_order.side, main_order.price) {
            (OrderSide::Buy, Some(main_price)) => {
                // The main order's price must be greator then or equal to the matched orders.
                ensure!(main_price >= price, Error::<T>::InsufficientOrderAmount);
            }
            (OrderSide::Sell, Some(main_price)) => {
                // The main order's price must be greator then or equal to the matched orders.
                ensure!(main_price <= price, Error::<T>::InsufficientOrderAmount);
            }
            (_, None) => {
                // The main order is a Market order.
            }
        }

        // Calculate the total price for the matched `amount` of tokens.
        let total_price = price.checked_mul(amount).ok_or(Error::<T>::Overflow)?;

        match order.side {
            OrderSide::Buy => {
                match orderbook.coin {
                    Coin::Asset(coin) => {
                        // Make sure there is enough locked coins to cover this order.
                        let portfolio = order
                            .coin_portfolio
                            .ok_or(Error::<T>::InsufficientLockedAssets)?;
                        Self::unlock_assets(
                            orderbook_id,
                            &order.account_id,
                            portfolio,
                            coin,
                            total_price,
                        )?;
                    }
                    Coin::Offchain(_) => {
                        // The venue handles off-chain assets.
                    }
                }
            }
            OrderSide::Sell => {
                // Make sure there is enough locked assets to cover this order.
                Self::unlock_assets(
                    orderbook_id,
                    &order.account_id,
                    order.asset_portfolio,
                    orderbook.asset,
                    amount,
                )?;
            }
        }

        Ok((order, amount, total_price))
    }

    fn ensure_valid_main_order(
        orderbook: &OrderBook,
        orderbook_id: OrderBookId,
        signed_order: &SignedOrder<T::AccountId>,
        total_amount: Balance,
        total_price: Balance,
    ) -> DispatchResult {
        let order = Self::ensure_signed_order(signed_order)?;

        // Main order total price.
        let main_total_price = match order.price {
            Some(price) => {
                // Limit order.
                price
                    .checked_mul(total_amount)
                    .ok_or(Error::<T>::Overflow)?
            }
            None => {
                // Market order.
                total_price
            }
        };

        // Make sure the main order amount is not smaller then the matched amount.
        ensure!(
            order.amount >= total_amount,
            Error::<T>::InsufficientOrderAmount
        );

        match order.side {
            OrderSide::Buy => {
                // The main order's price must be greator then or equal to the matched orders.
                ensure!(
                    main_total_price >= total_price,
                    Error::<T>::InsufficientOrderAmount
                );

                match orderbook.coin {
                    Coin::Asset(coin) => {
                        // Make sure there is enough locked coins to cover this order.
                        let portfolio = order
                            .coin_portfolio
                            .ok_or(Error::<T>::InsufficientLockedAssets)?;
                        Self::unlock_assets(
                            orderbook_id,
                            &order.account_id,
                            portfolio,
                            coin,
                            total_price,
                        )?;
                    }
                    Coin::Offchain(_) => {
                        // The venue handles off-chain assets.
                    }
                }
            }
            OrderSide::Sell => {
                // The main order's price must be greator then or equal to the matched orders.
                ensure!(
                    main_total_price <= total_price,
                    Error::<T>::InsufficientOrderAmount
                );

                // Make sure there is enough locked assets to cover this order.
                Self::unlock_assets(
                    orderbook_id,
                    &order.account_id,
                    order.asset_portfolio,
                    orderbook.asset,
                    total_amount,
                )?;
            }
        }

        Ok(())
    }
}
