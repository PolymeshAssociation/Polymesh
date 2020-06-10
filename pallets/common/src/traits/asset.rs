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

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use polymesh_primitives::{IdentityId, Ticker};

/// This trait is used to call functions that accept transfer of a ticker or token ownership
pub trait AcceptTransfer {
    /// Accept and process a ticker transfer
    ///
    /// # Arguments
    /// * `to_did` did of the receiver
    /// * `auth_id` Authorization id of the authorization created by current ticker owner
    fn accept_ticker_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult;
    /// Accept and process a token ownership transfer
    ///
    /// # Arguments
    /// * `to_did` did of the receiver
    /// * `auth_id` Authorization id of the authorization created by current token owner
    fn accept_asset_ownership_transfer(to_did: IdentityId, auth_id: u64) -> DispatchResult;
}
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct IssueAssetItem<U> {
    pub investor_did: IdentityId,
    pub value: U,
}

pub trait Trait<V, U> {
    fn total_supply(ticker: &Ticker) -> V;
    fn balance(ticker: &Ticker, did: IdentityId) -> V;
    fn _mint_from_sto(
        ticker: &Ticker,
        caller: U,
        sender_did: IdentityId,
        assets_purchased: V,
    ) -> DispatchResult;
    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool;
    fn get_balance_at(ticker: &Ticker, did: IdentityId, at: u64) -> V;
}
