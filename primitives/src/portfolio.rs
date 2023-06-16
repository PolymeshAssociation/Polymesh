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

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use crate::asset::FundingRoundName;
use crate::settlement::InstructionId;
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

/// Reason for the portfolio update.
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub enum PortfolioUpdateReason {
    /// Tokens were issued.
    Issued {
        /// If the asset is fungible the [`FundingRoundName`] of the minted tokens.
        funding_round_name: Option<FundingRoundName>,
    },
    /// Tokens were redeemed.
    Redeemed,
    /// Tokens were transferred.
    Transferred {
        /// The [`InstructionId`] of the instruction which originated the transfer.
        instruction_id: Option<InstructionId>,
        /// The [`Memo`] of the instruction.
        instruction_memo: Option<Memo>,
    },
}
