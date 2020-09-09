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

use frame_support::dispatch::DispatchResult;
use polymesh_primitives::{IdentityId, PortfolioId, Ticker};

/// This trait is used to accept custody of a portfolio
pub trait PortfolioSubTrait<Balance> {
    /// Accepts custody of a portfolio
    ///
    /// # Arguments
    /// * `new_custodian` - DID of the new custodian
    /// * `auth_id` - Authorization ID of the authorization created by the current custodian.
    fn accept_portfolio_custody(new_custodian: IdentityId, auth_id: u64) -> DispatchResult;

    /// Checks that the custodian is authorized for the portfolio
    ///
    /// # Arguments
    /// * `portfolio` - Portfolio to check
    /// * `custodian` - DID of the custodian
    fn check_portfolio_custody(portfolio: PortfolioId, custodian: IdentityId) -> DispatchResult;

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
}
