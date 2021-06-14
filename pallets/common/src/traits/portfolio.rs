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

//! # Trait Interface to the Portfolio Module
//!
//! The interface allows to accept portfolio custody

use crate::{
    traits::{balances::Memo, base, identity},
    CommonConfig,
};
use codec::{Decode, Encode};
use frame_support::decl_event;
use frame_support::dispatch::DispatchResult;
use frame_support::weights::Weight;
use polymesh_primitives::{
    IdentityId, PortfolioId, PortfolioName, PortfolioNumber, SecondaryKey, Ticker,
};
use sp_std::vec::Vec;

/// This trait is used to accept custody of a portfolio
pub trait PortfolioSubTrait<Balance, AccountId: Encode + Decode> {
    /// Accepts custody of a portfolio
    ///
    /// # Arguments
    /// * `to` - DID of the new custodian
    /// * `from` - Sender of the authorization
    /// * `pid` - The old portfolio ID
    fn accept_portfolio_custody(
        to: IdentityId,
        from: IdentityId,
        pid: PortfolioId,
    ) -> DispatchResult;

    /// Checks that the custodian is authorized for the portfolio
    ///
    /// # Arguments
    /// * `portfolio` - Portfolio to check
    /// * `custodian` - DID of the custodian
    fn ensure_portfolio_custody(portfolio: PortfolioId, custodian: IdentityId) -> DispatchResult;

    /// Locks some tokens of a portfolio
    ///
    /// # Arguments
    /// * `portfolio` - Portfolio to lock tokens
    /// * `ticker` - Ticker of the token to lock
    /// * `amount` - Amount of tokens to lock

    fn lock_tokens(portfolio: &PortfolioId, ticker: &Ticker, amount: &Balance) -> DispatchResult;

    /// Unlocks some tokens of a portfolio
    ///
    /// # Arguments
    /// * `portfolio` - Portfolio to unlock tokens
    /// * `ticker` - Ticker of the token to unlock
    /// * `amount` - Amount of tokens to unlock
    fn unlock_tokens(portfolio: &PortfolioId, ticker: &Ticker, amount: &Balance) -> DispatchResult;

    /// Ensures that the portfolio's custody is with the provided identity
    /// And the secondary key has the relevant portfolio permission
    ///
    /// # Arguments
    /// * `portfolio` - PortfolioId of the portfolio to check
    /// * `custodian` - Identity of the custodian
    /// * `secondary_key` - Secondary key that is accessing the portfolio
    fn ensure_portfolio_custody_and_permission(
        portfolio: PortfolioId,
        custodian: IdentityId,
        secondary_key: Option<&SecondaryKey<AccountId>>,
    ) -> DispatchResult;
}

pub trait WeightInfo {
    fn create_portfolio() -> Weight;
    fn delete_portfolio() -> Weight;
    fn move_portfolio_funds(i: u32) -> Weight;
    fn rename_portfolio(i: u32) -> Weight;
    fn quit_portfolio_custody() -> Weight;
}

pub trait Config: CommonConfig + identity::Config + base::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event<T> where
        Balance = <T as CommonConfig>::Balance,
    {
        /// The portfolio has been successfully created.
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio number
        /// * portfolio name
        PortfolioCreated(IdentityId, PortfolioNumber, PortfolioName),
        /// The portfolio has been successfully removed.
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio number
        PortfolioDeleted(IdentityId, PortfolioNumber),
        /// A token amount has been moved from one portfolio to another.
        ///
        /// # Parameters
        /// * origin DID
        /// * source portfolio
        /// * destination portfolio
        /// * asset ticker
        /// * asset balance that was moved
        MovedBetweenPortfolios(
            IdentityId,
            PortfolioId,
            PortfolioId,
            Ticker,
            Balance,
            Option<Memo>,
        ),
        /// The portfolio identified with `num` has been renamed to `name`.
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio number
        /// * portfolio name
        PortfolioRenamed(IdentityId, PortfolioNumber, PortfolioName),
        /// All non-default portfolio numbers and names of a DID.
        ///
        /// # Parameters
        /// * origin DID
        /// * vector of number-name pairs
        UserPortfolios(IdentityId, Vec<(PortfolioNumber, PortfolioName)>),
        /// Custody of a portfolio has been given to a different identity
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio id
        /// * portfolio custodian did
        PortfolioCustodianChanged(IdentityId, PortfolioId, IdentityId),
    }
}
