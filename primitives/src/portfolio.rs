// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
use scale_info::TypeInfo;

use crate::{Balance, Memo, NFTs, Ticker};

/// Describes what should be moved between portfolios. It can be either fungible or non-fungible tokens.
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct Fund {
    /// The type of token being moved.
    pub description: FundDescription,
    /// An optional memo for the transfer.
    pub memo: Option<Memo>,
}

/// Defines the types of tokens that can be moved.
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub enum FundDescription {
    /// Fungible token.
    Fungible {
        /// The Ticker of the token.
        ticker: Ticker,
        /// The Balance being transfered.
        amount: Balance,
    },
    /// Fungible token.
    NonFungible(NFTs),
}
