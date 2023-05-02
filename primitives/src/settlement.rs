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

//! Shareable types.

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::alloc::string::ToString;
use codec::{Decode, Encode};
use frame_support::weights::Weight;
use scale_info::prelude::string::String;
use scale_info::TypeInfo;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec::Vec;

use polymesh_primitives_derive::VecU8StrongTyped;

use crate::constants::SETTLEMENT_INSTRUCTION_EXECUTION;
use crate::{impl_checked_inc, Balance, IdentityId, NFTs, PortfolioId, Ticker};

/// A global and unique venue ID.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct VenueId(pub u64);
impl_checked_inc!(VenueId);

/// A wrapper for VenueDetails
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct VenueDetails(Vec<u8>);

/// Status of an instruction
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstructionStatus<BlockNumber> {
    /// Invalid instruction or details pruned
    Unknown,
    /// Instruction is pending execution
    Pending,
    /// Instruction has failed execution
    Failed,
    /// Instruction has been executed successfully
    Success(BlockNumber),
    /// Instruction has been rejected.
    Rejected(BlockNumber),
}

impl<BlockNumber> Default for InstructionStatus<BlockNumber> {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Type of the venue. Used for offchain filtering.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VenueType {
    /// Default type - used for mixed and unknown types
    Other,
    /// Represents a primary distribution
    Distribution,
    /// Represents an offering/fund raiser
    Sto,
    /// Represents a match making service
    Exchange,
}

impl Default for VenueType {
    fn default() -> Self {
        Self::Other
    }
}

/// Status of a leg
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LegStatus<AccountId> {
    /// It is waiting for affirmation
    PendingTokenLock,
    /// It is waiting execution (tokens currently locked)
    ExecutionPending,
    /// receipt used, (receipt signer, receipt uid)
    ExecutionToBeSkipped(AccountId, u64),
}

impl<AccountId> Default for LegStatus<AccountId> {
    fn default() -> Self {
        Self::PendingTokenLock
    }
}

/// Status of an affirmation
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AffirmationStatus {
    /// Invalid affirmation
    Unknown,
    /// Pending user's consent
    Pending,
    /// Affirmed by the user
    Affirmed,
}

impl Default for AffirmationStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Type of settlement
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettlementType<BlockNumber> {
    /// Instruction should be settled in the next block as soon as all affirmations are received.
    SettleOnAffirmation,
    /// Instruction should be settled on a particular block.
    SettleOnBlock(BlockNumber),
    /// Instruction must be settled manually on or after BlockNumber.
    SettleManual(BlockNumber),
}

impl<BlockNumber> Default for SettlementType<BlockNumber> {
    fn default() -> Self {
        Self::SettleOnAffirmation
    }
}

/// A per-Instruction leg ID.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct LegId(pub u64);
impl_checked_inc!(LegId);

/// A global and unique instruction ID.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct InstructionId(pub u64);
impl_checked_inc!(InstructionId);

impl InstructionId {
    /// Converts an instruction id into a scheduler name.
    pub fn execution_name(&self) -> Vec<u8> {
        (SETTLEMENT_INSTRUCTION_EXECUTION, self.0).encode()
    }
}

/// A wrapper for InstructionMemo
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstructionMemo(pub [u8; 32]);

/// Details about an instruction.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Instruction<Moment, BlockNumber> {
    /// Unique instruction id. It is an auto incrementing number
    pub instruction_id: InstructionId,
    /// Id of the venue this instruction belongs to
    pub venue_id: VenueId,
    /// Type of settlement used for this instruction
    pub settlement_type: SettlementType<BlockNumber>,
    /// Date at which this instruction was created
    pub created_at: Option<Moment>,
    /// Date from which this instruction is valid
    pub trade_date: Option<Moment>,
    /// Date after which the instruction should be settled (not enforced)
    pub value_date: Option<Moment>,
}

/// Details of a leg including the leg id in the instruction.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Leg {
    /// Portfolio of the sender
    pub from: PortfolioId,
    /// Portfolio of the receiver
    pub to: PortfolioId,
    /// Ticker of the asset being transferred
    pub asset: Ticker,
    /// Amount being transferred
    pub amount: Balance,
}

impl TryFrom<LegV2> for Leg {
    type Error = &'static str;

    fn try_from(leg_v2: LegV2) -> Result<Self, Self::Error> {
        match leg_v2.asset {
            LegAsset::NonFungible(_nfts) => Err("InvalidLegAsset"),
            LegAsset::Fungible { ticker, amount } => Ok(Leg {
                from: leg_v2.from,
                to: leg_v2.to,
                asset: ticker,
                amount,
            }),
        }
    }
}

/// Type of assets that can be transferred in a `Leg`.
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub enum LegAsset {
    /// Fungible token
    Fungible {
        /// Ticker of the fungible token.
        ticker: Ticker,
        /// Amount being trasnferred.
        amount: Balance,
    },
    /// Non Fungible token.
    NonFungible(NFTs),
}

impl LegAsset {
    /// Returns the ticker and amount being transferred.
    pub fn ticker_and_amount(&self) -> (Ticker, Balance) {
        match self {
            LegAsset::Fungible { ticker, amount } => (*ticker, *amount),
            LegAsset::NonFungible(nfts) => (*nfts.ticker(), nfts.len() as Balance),
        }
    }
}

impl Default for LegAsset {
    fn default() -> Self {
        LegAsset::Fungible {
            ticker: Ticker::default(),
            amount: Balance::default(),
        }
    }
}

/// Defines a leg (i.e the action of a settlement).
#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
pub struct LegV2 {
    /// Portfolio of the sender.
    pub from: PortfolioId,
    /// Portfolio of the receiver.
    pub to: PortfolioId,
    /// Assets being transferred.
    pub asset: LegAsset,
}

impl From<Leg> for LegV2 {
    fn from(leg: Leg) -> Self {
        LegV2 {
            from: leg.from,
            to: leg.to,
            asset: LegAsset::Fungible {
                ticker: leg.asset,
                amount: leg.amount,
            },
        }
    }
}

/// Details about a venue.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Default, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Venue {
    /// Identity of the venue's creator
    pub creator: IdentityId,
    /// Specifies type of the venue (Only needed for the UI)
    pub venue_type: VenueType,
}

/// Details about an offchain transaction receipt
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Receipt<Balance> {
    /// Unique receipt number set by the signer for their receipts
    pub receipt_uid: u64,
    /// Identity of the sender
    pub from: PortfolioId,
    /// Identity of the receiver
    pub to: PortfolioId,
    /// Ticker of the asset being transferred
    pub asset: Ticker,
    /// Amount being transferred
    pub amount: Balance,
}

/// A wrapper for VenueDetails
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReceiptMetadata(Vec<u8>);

/// Details about an offchain transaction receipt that a user must input
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct ReceiptDetails<AccountId, OffChainSignature> {
    /// Unique receipt number set by the signer for their receipts
    pub receipt_uid: u64,
    /// Target leg id
    pub leg_id: LegId,
    /// Signer for this receipt
    pub signer: AccountId,
    /// signature confirming the receipt details
    pub signature: OffChainSignature,
    /// Generic text that can be used to attach messages to receipts
    pub metadata: ReceiptMetadata,
}

/// Stores the number of fungible and non fungible transfers in a set of legs.
#[derive(Default)]
pub struct TransferData {
    fungible: u32,
    non_fungible: u32,
}

impl TransferData {
    /// Creates an instance of `TransfersData`.
    pub fn new(fungible: u32, non_fungible: u32) -> Self {
        TransferData {
            fungible,
            non_fungible,
        }
    }

    /// Returns the number of fungible transfers.
    pub fn fungible(&self) -> u32 {
        self.fungible
    }

    /// Returns the number of non fungible transfers.
    pub fn non_fungible(&self) -> u32 {
        self.non_fungible
    }

    /// Adds one to the number of fungible transfers.
    pub fn add_fungible(&mut self) {
        self.fungible += 1;
    }

    /// Adds `nfts.len()` to the number of non fungible transfers.
    pub fn add_non_fungible(&mut self, nfts: &NFTs) {
        self.non_fungible += nfts.len() as u32;
    }

    /// Adds one to the number of fungible assets.
    /// Returns an error if the number of fungible assets is greater than 1024.
    pub fn try_add_fungible(&mut self) -> Result<(), String> {
        if self.fungible + 1 > 1024 {
            return Err(String::from(
                "Number of fungible assets is greater than allowed",
            ));
        }
        self.fungible += 1;
        Ok(())
    }

    /// Adds `nfts.len()` to the number of non fungible transfers.
    /// Returns an error if the number of non fungible transfers is greater than 1024.
    pub fn try_add_non_fungible(&mut self, nfts: &NFTs) -> Result<(), String> {
        match nfts.len().checked_add(self.non_fungible as usize) {
            Some(new_value) => {
                if new_value > 1024 {
                    return Err(String::from("Number of NFTs is greater than allowed"));
                }
                self.non_fungible += nfts.len() as u32;
            }
            None => return Err(String::from("Number of NFTs is greater than allowed")),
        }
        Ok(())
    }

    /// Gets the `TransferData` from a slice of `LegV2`.
    pub fn from_legs(legs_v2: &[LegV2]) -> Result<TransferData, String> {
        let mut transfer_data = TransferData::default();
        for leg_v2 in legs_v2 {
            match &leg_v2.asset {
                LegAsset::Fungible { .. } => transfer_data.try_add_fungible()?,
                LegAsset::NonFungible(nfts) => transfer_data.try_add_non_fungible(&nfts)?,
            }
        }
        Ok(transfer_data)
    }
}

/// Stores information about an Instruction.
pub struct InstructionInfo {
    /// Unique counter parties involved in the instruction.
    parties: BTreeSet<PortfolioId>,
    /// The number of fungible and non fungible transfers in the instruction.
    transfer_data: TransferData,
}

impl InstructionInfo {
    /// Creates an instance of `InstructionInfo`.
    pub fn new(parties: BTreeSet<PortfolioId>, transfer_data: TransferData) -> Self {
        Self {
            parties,
            transfer_data,
        }
    }

    /// Returns a slice of all unique parties in the instruction.
    pub fn parties(&self) -> &BTreeSet<PortfolioId> {
        &self.parties
    }

    /// Returns the number of fungible transfers.
    pub fn fungible_transfers(&self) -> u32 {
        self.transfer_data.fungible()
    }

    /// Returns the number of non fungible transfers.
    pub fn nfts_transferred(&self) -> u32 {
        self.transfer_data.non_fungible()
    }
}

/// Stores relevant information for executing an instruction.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode)]
pub struct ExecuteInstructionInfo {
    /// Number of fungible tokens in the instruction.
    fungible_tokens: u32,
    /// Number of non fungible tokens in the instruction.
    non_fungible_tokens: u32,
    /// The weight needed for executing the instruction.
    consumed_weight: Weight,
    /// If the instruction would fail, contains the error.
    error: Option<String>,
}

impl ExecuteInstructionInfo {
    /// Creates an instance of `ExecuteInstructionInfo`.
    pub fn new(
        fungible_tokens: u32,
        non_fungible_tokens: u32,
        consumed_weight: Weight,
        error: Option<&str>,
    ) -> Self {
        Self {
            fungible_tokens,
            non_fungible_tokens,
            consumed_weight,
            error: error.map(|e| e.to_string()),
        }
    }
}
