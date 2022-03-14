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

use crate::{impl_checked_inc, Balance, Moment, PortfolioId, Signature, Ticker, VenueId};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{traits::Verify, AccountId32};
use sp_std::prelude::*;

/// OrderBook id at a venue.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderBookId(u64);
impl_checked_inc!(OrderBookId);

/// Order id.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderId(u64);

/// On/off chain coin used to trade an asset.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Coin {
    /// Polymesh stablecoin.
    Asset(Ticker),
    /// Offchain: Fiat, BTC, ETH.
    Offchain(Ticker),
}

/// OrderBook details
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderBook {
    /// The Venue this orderbook belongs too.
    pub venue_id: VenueId,
    /// The Polymesh asset that is traded in this OrderBook.
    /// The `asset` must be an on-chain asset.
    pub asset: Ticker,
    /// The coin used in this order book for all trades.
    /// `coin` can be an on-chain asset/stablecoin or off-chain coin.
    pub coin: Coin,
}

/// Order's fee details.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeeDetails {
    /// Fiat, BTC, ETH, etc..
    /// The Venue handles charging this fee
    /// the investor will need to have an account
    /// with the Venue and deposit funds to cover fees.
    OffChain,
    /// Another asset or stablecoin on Polymesh
    OnChain(Ticker, Balance),
}

/// Buy/Sell order.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderSide {
    /// Buy order.
    Buy,
    /// Sell order.
    Sell,
}

/// Order details that needs to be signed by the investor
/// and submitted to the Venue to be included in the Orderbook.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order<AccountId> {
    /// Buy/Sell order.
    pub side: OrderSide,
    /// The OrderBook this order is valid for.
    /// The OrderBook details also contain the `asset` `coin` info.
    pub order_book_id: OrderBookId,
    /// Asset portfolio.
    pub asset_portfolio: PortfolioId,
    /// Number of asset tokens.
    pub amount: Balance,
    /// Coin portfolio.  (None => for off-chain coins).
    pub coin_portfolio: Option<PortfolioId>,
    /// None => Market price.  Market order immediately match the
    /// top orders in the order book.  Some investors just want to
    /// fill their order and not have it sit on the order book.
    /// If the asset's price is moving quickly, then setting a price
    /// will risk having the order get left behind and unfilled.
    /// Some(price) => 1 asset token = `price` coins.
    pub price: Option<Balance>,
    /// Fee details are included, because the investor needs to
    /// agree on the fee that might be charged on-chain.
    pub fee: FeeDetails,
    /// The investor's key used to sign the order.
    pub account_id: AccountId,
    /// Polymesh will also use this id to make sure the Venue
    /// doesn't try to replay an already settled/finalized Order.
    /// `OrderId` are unique for each `account_id`.
    pub order_id: OrderId,
    /// The venue should make sure that this timestamp is not older
    /// then a few seconds.  So the investor doesn't try to front-run
    /// other open order with an older timestamp.
    /// Venues can also use their own timestamp (time the order was received)
    /// during order matching.  But the two timestamp shouldn't be too
    /// far apart.
    pub timestamp: Moment,
}

/// Signed Order.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedOrder<AccountId> {
    /// Signature.
    pub signature: Signature,
    /// Order.
    pub order: Order<AccountId>,
}

impl<AccountId: Clone + Encode> SignedOrder<AccountId> {
    /// Verify that the order is signed by the `account_id`.
    pub fn verify(&self) -> bool {
        let data = self.order.encode();
        let signer: Result<AccountId32, _> =
            Decode::decode(&mut &self.order.account_id.encode()[..]);
        if let Ok(signer) = signer {
            self.signature.verify(data.as_slice(), &signer)
        } else {
            false
        }
    }
}

/// Matched order.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchedOrder<AccountId> {
    /// Signed order that matched.
    pub order: SignedOrder<AccountId>,
    /// `amount` might be a partial fill of one/both sides.
    pub amount: Balance,
}

/// A batch of signed orders to be settles on-chain.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettleOrders<AccountId> {
    /// This is the order that matched open-orders and
    /// triggered trades.
    /// if `main_order` is a buy-side, then all `matched_orders`
    /// must be sell-side orders.
    /// if `main_order` is a sell-side, then all `matched_orders`
    /// must be buy-side orders.
    pub main_order: SignedOrder<AccountId>,
    /// The orders that matched the `main_order`.
    pub matched_orders: Vec<MatchedOrder<AccountId>>,
}
