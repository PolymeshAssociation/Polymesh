#![allow(missing_docs)]
// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::Balance;
use sp_runtime::transaction_validity::InvalidTransaction;

pub mod group;

// Polymesh note: This was specifically added for Polymesh
pub trait CddAndFeeDetails<AccountId, Call> {
    fn get_valid_payer(
        call: &Call,
        caller: &AccountId,
    ) -> Result<Option<AccountId>, InvalidTransaction>;
    fn clear_context();
    fn set_payer_context(payer: Option<AccountId>);
    fn get_payer_from_context() -> Option<AccountId>;
}

/// A currency that has a block rewards reserve.
pub trait BlockRewardsReserveCurrency<NegativeImbalance> {
    /// An instance of `Drop` for positive imbalance.
    fn drop_positive_imbalance(amount: Balance);
    /// An instance of `Drop` for negative imbalance.
    fn drop_negative_imbalance(amount: Balance);
    /// Issues a given amount of currency from the block rewards reserve if possible.
    fn issue_using_block_rewards_reserve(amount: Balance) -> NegativeImbalance;
    /// Returns the balance of the block rewards reserve.
    fn block_rewards_reserve_balance() -> Balance;
}
