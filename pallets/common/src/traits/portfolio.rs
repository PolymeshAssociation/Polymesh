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
use polymesh_primitives::IdentityId;

/// This trait is used to accept custody of a portfolio
pub trait PortfolioSubTrait {
    /// Accepts custody of a portfolio
    ///
    /// # Arguments
    /// * `new_custodian` - DID of the new custodian
    /// * `auth_id` - Authorization ID of the authorization created by the current custodian.
    fn accept_portfolio_custody(new_custodian: IdentityId, auth_id: u64) -> DispatchResult;
}
