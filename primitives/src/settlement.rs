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

use polymesh_primitives_derive::{SliceU8StrongTyped, VecU8StrongTyped};

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

/// Defines a [`Leg`] (i.e the action of a settlement).
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub enum Leg {
    /// Fungible token
    Fungible {
        /// The [`PortfolioId`] of the sender.
        sender: PortfolioId,
        /// The [`PortfolioId`] of the receiver.
        receiver: PortfolioId,
        /// The [`Ticker`] of the fungible token.
        ticker: Ticker,
        /// The amount being transferred.
        amount: Balance,
    },
    /// Non Fungible token.
    NonFungible {
        /// The [`PortfolioId`] of the sender.
        sender: PortfolioId,
        /// The [`PortfolioId`] of the receiver.
        receiver: PortfolioId,
        /// The [`NFTs`] being transferred.
        nfts: NFTs,
    },
    /// Assets that don't settle on-chain (e.g USD).
    OffChain {
        /// The [`IdentityId`] of the sender.
        sender_identity: IdentityId,
        /// The [`IdentityId`] of the receiver.
        receiver_identity: IdentityId,
        /// The [`Ticker`] for the off-chain asset.
        ticker: Ticker,
        /// The amount transferred.
        amount: Balance,
    },
}

impl Leg {
    /// Returns `true` if it's an [`Leg::OffChain`] leg, otherwise returns `false`.
    pub fn is_off_chain(&self) -> bool {
        if let Leg::OffChain { .. } = self {
            return true;
        }
        false
    }

    /// Returns the [`Ticker`] of the asset in the given leg.
    pub fn ticker(&self) -> Ticker {
        match self {
            Leg::Fungible { ticker, .. } => *ticker,
            Leg::NonFungible { nfts, .. } => *nfts.ticker(),
            Leg::OffChain { ticker, .. } => *ticker,
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

/// An offchain transaction receipt.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Receipt<Balance> {
    /// Unique receipt number set by the signer for their receipts.
    uid: u64,
    /// The [`InstructionId`] of the instruction for which the receipt is for.
    instruction_id: InstructionId,
    /// The [`LegId`] of the leg for which the receipt is for.
    leg_id: LegId,
    /// The [`IdentityId`] of the sender.
    sender_identity: IdentityId,
    /// The [`IdentityId`] of the receiver.
    receiver_identity: IdentityId,
    /// [`Ticker`] of the asset being transferred.
    ticker: Ticker,
    /// The amount transferred.
    amount: Balance,
}

impl<Balance> Receipt<Balance> {
    /// Creates a new [`Receipt`].
    pub fn new(
        uid: u64,
        instruction_id: InstructionId,
        leg_id: LegId,
        sender_identity: IdentityId,
        receiver_identity: IdentityId,
        ticker: Ticker,
        amount: Balance,
    ) -> Self {
        Receipt {
            uid,
            instruction_id,
            leg_id,
            sender_identity,
            receiver_identity,
            ticker,
            amount,
        }
    }
}

/// A wrapper of [`[u8; 32]`] that can be used for generic messages.
#[derive(Encode, Decode, TypeInfo, SliceU8StrongTyped)]
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReceiptMetadata([u8; 32]);

/// Details about an offchain transaction receipt.
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct ReceiptDetails<AccountId, OffChainSignature> {
    /// Unique receipt number set by the signer for their receipts
    uid: u64,
    /// The [`InstructionId`] of the instruction which contains the offchain transfer.
    instruction_id: InstructionId,
    /// The [`LegId`] which which contains the offchain transfer.
    leg_id: LegId,
    /// The [`AccountId`] of the Signer for this receipt.
    signer: AccountId,
    /// Signature confirming the receipt details.
    signature: OffChainSignature,
    /// The [`ReceiptMetadata`] that can be used to attach messages to receipts.
    metadata: Option<ReceiptMetadata>,
}

impl<AccountId, OffChainSignature> ReceiptDetails<AccountId, OffChainSignature> {
    /// Creates a new [`ReceiptDetails`].
    pub fn new(
        uid: u64,
        instruction_id: InstructionId,
        leg_id: LegId,
        signer: AccountId,
        signature: OffChainSignature,
        metadata: Option<ReceiptMetadata>,
    ) -> Self {
        Self {
            uid,
            instruction_id,
            leg_id,
            signer,
            signature,
            metadata,
        }
    }

    /// Returns the uid of the receipt details.
    pub fn uid(&self) -> u64 {
        self.uid
    }

    /// Returns the [`InstructionId`] of the receipt details.
    pub fn instruction_id(&self) -> &InstructionId {
        &self.instruction_id
    }

    /// Returns the [`LegId`] of the receipt details.
    pub fn leg_id(&self) -> LegId {
        self.leg_id
    }

    /// Returns the [`AccountId`] of the signer of the receipt details.
    pub fn signer(&self) -> &AccountId {
        &self.signer
    }

    /// Returns the signature of the receipt details.
    pub fn signature(&self) -> &OffChainSignature {
        &self.signature
    }

    /// Returns the [`ReceiptMetadata`] of the receipt details.
    pub fn metadata(&self) -> &Option<ReceiptMetadata> {
        &self.metadata
    }
}

/// Stores the number of fungible, non fungible and offchain transfers in a set of legs.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
pub struct AssetCount {
    fungible: u32,
    non_fungible: u32,
    off_chain: u32,
}

impl AssetCount {
    /// Creates an instance of [`AssetCount`].
    pub fn new(fungible: u32, non_fungible: u32, off_chain: u32) -> Self {
        AssetCount {
            fungible,
            non_fungible,
            off_chain,
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

    /// Returns the number of off-chain assets.
    pub fn off_chain(&self) -> u32 {
        self.off_chain
    }

    /// Adds one to the number of fungible transfers.
    pub fn add_fungible(&mut self) {
        self.fungible += 1;
    }

    /// Adds `nfts.len()` to the number of non fungible transfers.
    pub fn add_non_fungible(&mut self, nfts: &NFTs) {
        self.non_fungible += nfts.len() as u32;
    }

    /// Adds one to the number of off-chain assets.
    pub fn add_off_chain(&mut self) {
        self.off_chain += 1;
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

    /// Adds one to the number of off-chain assets.
    /// Returns an error if the number of off-chain assets is greater than 1024.
    pub fn try_add_off_chain(&mut self) -> Result<(), String> {
        if self.off_chain + 1 > 1024 {
            return Err(String::from(
                "Number of off-chain assets is greater than allowed",
            ));
        }
        self.off_chain += 1;
        Ok(())
    }

    /// Gets the [`AssetCount`] from a slice of [`Leg`].
    pub fn try_from_legs(legs: &[Leg]) -> Result<AssetCount, String> {
        let mut asset_count = AssetCount::default();
        for leg in legs {
            match &leg {
                Leg::Fungible { .. } => asset_count.try_add_fungible()?,
                Leg::NonFungible { nfts, .. } => asset_count.try_add_non_fungible(&nfts)?,
                Leg::OffChain { .. } => asset_count.try_add_off_chain()?,
            }
        }
        Ok(asset_count)
    }

    /// Gets the [`AssetCount`] from a slice of [`(LegId, Leg)`].
    /// Note: Doesn't check for overflows.
    pub fn from_legs(legs: &[(LegId, Leg)]) -> AssetCount {
        let mut asset_count = AssetCount::default();
        for (_, leg) in legs {
            match &leg {
                Leg::Fungible { .. } => asset_count.add_fungible(),
                Leg::NonFungible { nfts, .. } => asset_count.add_non_fungible(&nfts),
                Leg::OffChain { .. } => asset_count.add_off_chain(),
            }
        }
        asset_count
    }
}

/// Stores [`AssetCount`] for the instruction, all portfolio that have pre-affirmed the transfer
/// and all portfolios that still have to approve the transfer.
pub struct InstructionInfo {
    /// The number of fungible, non fungible and off-chain transfers in the instruction.
    instruction_asset_count: AssetCount,
    /// All portfolios that still need to affirm the instruction.
    portfolios_pending_approval: BTreeSet<PortfolioId>,
    /// All portfolios that have pre-approved the transfer of a ticker.
    portfolios_pre_approved: BTreeSet<PortfolioId>,
}

impl InstructionInfo {
    /// Creates an instance of [`InstructionInfo`].
    pub fn new(
        instruction_asset_count: AssetCount,
        portfolios_pending_approval: BTreeSet<PortfolioId>,
        portfolios_pre_approved: BTreeSet<PortfolioId>,
    ) -> Self {
        Self {
            instruction_asset_count,
            portfolios_pending_approval,
            portfolios_pre_approved,
        }
    }

    /// Returns a slice of all portfolios that still have to affirm the instruction.
    pub fn portfolios_pending_approval(&self) -> &BTreeSet<PortfolioId> {
        &self.portfolios_pending_approval
    }

    /// Returns a [`BTreeSet<&PortfolioId>`] of all portfolios that are in `self.portfolios_pre_approved`, but not in `self.portfolios_pending_approval`.
    pub fn portfolios_pre_approved_difference(&self) -> BTreeSet<&PortfolioId> {
        self.portfolios_pre_approved
            .difference(&self.portfolios_pending_approval)
            .collect()
    }

    /// Returns the number of pending affirmations for the instruction.
    /// The value must be equal to all unique portfolio that have not pre-approved the transfer + the number of offchain legs.
    pub fn number_of_pending_affirmations(&self) -> u64 {
        self.portfolios_pending_approval.len() as u64
            + self.instruction_asset_count.off_chain() as u64
    }

    /// Returns the number of fungible transfers.
    pub fn fungible_transfers(&self) -> u32 {
        self.instruction_asset_count.fungible()
    }

    /// Returns the number of non fungible transfers.
    pub fn nfts_transferred(&self) -> u32 {
        self.instruction_asset_count.non_fungible()
    }

    /// Returns the number of off-chain transfers.
    pub fn off_chain(&self) -> u32 {
        self.instruction_asset_count.off_chain()
    }
}

/// Holds the [`SenderSideInfo`] and the [`AssetCount`] both for the receiver and unfiltered set.
pub struct FilteredLegs {
    /// Holds the [`SenderSideInfo`] for the legs on the sender side of the instruction.
    sender_side_info: SenderSideInfo,
    /// The [`AssetCount`] for the legs on the receiver side of the instruction.
    receiver_asset_count: AssetCount,
    /// The [`AssetCount`] for the unfiltered set.
    unfiltered_asset_count: AssetCount,
}

/// Stores a subset of legs on the sender side of an instruction and their [`AssetCount`].
pub struct SenderSideInfo {
    /// A [`Vec<(LegId, Leg)>`] containing the legs on the sender side of an instruction.
    sender_subset: Vec<(LegId, Leg)>,
    /// The [`AssetCount`] for the subset of legs.
    sender_asset_count: AssetCount,
}

impl SenderSideInfo {
    /// Constructs a new [`SenderSideInfo`].
    pub fn new(sender_subset: Vec<(LegId, Leg)>, sender_asset_count: AssetCount) -> Self {
        Self {
            sender_subset,
            sender_asset_count,
        }
    }
}

impl FilteredLegs {
    /// Returns [`FilteredLegs`] where [`SenderSideInfo::sender_subset`] contain legs from a sender that belong to the specified `portfolio_set`.
    pub fn filter_sender(
        original_set: Vec<(LegId, Leg)>,
        portfolio_set: &BTreeSet<PortfolioId>,
    ) -> Self {
        let unfiltered_asset_count = AssetCount::from_legs(&original_set);
        let mut sender_asset_count = AssetCount::default();
        let mut receiver_asset_count = AssetCount::default();

        let mut sender_subset = Vec::new();
        for (leg_id, leg) in original_set {
            match leg {
                Leg::Fungible {
                    sender, receiver, ..
                } => {
                    if portfolio_set.contains(&sender) {
                        sender_subset.push((leg_id, leg));
                        sender_asset_count.add_fungible();
                    } else if portfolio_set.contains(&receiver) {
                        receiver_asset_count.add_fungible();
                    }
                }
                Leg::NonFungible {
                    sender,
                    receiver,
                    ref nfts,
                } => {
                    if portfolio_set.contains(&sender) {
                        sender_asset_count.add_non_fungible(&nfts);
                        sender_subset.push((leg_id, leg));
                    } else if portfolio_set.contains(&receiver) {
                        receiver_asset_count.add_non_fungible(&nfts);
                    }
                }
                Leg::OffChain { .. } => continue,
            }
        }

        FilteredLegs {
            sender_side_info: SenderSideInfo::new(sender_subset, sender_asset_count),
            receiver_asset_count,
            unfiltered_asset_count,
        }
    }

    /// Returns a slice of `[(LegId, Leg)]` containing all legs in the sender subset.
    pub fn sender_subset(&self) -> &[(LegId, Leg)] {
        &self.sender_side_info.sender_subset
    }

    /// Returns the [`AssetCount`] for the unfiltered set.
    pub fn unfiltered_asset_count(&self) -> &AssetCount {
        &self.unfiltered_asset_count
    }

    /// Returns the [`AssetCount`] for the sender subset of legs.
    pub fn sender_asset_count(&self) -> &AssetCount {
        &self.sender_side_info.sender_asset_count
    }

    /// Returns the [`AssetCount`] for the receiver side of the instruction.
    pub fn receiver_asset_count(&self) -> &AssetCount {
        &self.receiver_asset_count
    }
}

/// Holds the [`AssetCount`] for both the sender and receiver side and the number of offchain assets.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
pub struct AffirmationCount {
    /// The [`AssetCount`] for sender side.
    sender_asset_count: AssetCount,
    /// The [`AssetCount`] for receiver side.
    receiver_asset_count: AssetCount,
    /// The number of off-chain assets in the instruction.
    offchain_count: u32,
}

impl AffirmationCount {
    /// Creates a new [`AffirmationCount`] instance.
    pub fn new(
        sender_asset_count: AssetCount,
        receiver_asset_count: AssetCount,
        offchain_count: u32,
    ) -> Self {
        AffirmationCount {
            sender_asset_count,
            receiver_asset_count,
            offchain_count,
        }
    }

    /// Returns the [`AssetCount`] for the sender side.
    pub fn sender_asset_count(&self) -> &AssetCount {
        &self.sender_asset_count
    }

    /// Returns the [`AssetCount`] for the receiver side.
    pub fn receiver_asset_count(&self) -> &AssetCount {
        &self.receiver_asset_count
    }

    /// The number of off-chain assets in the instruction.
    pub fn offchain_count(&self) -> u32 {
        self.offchain_count
    }
}

/// Stores the number of fungible, non fungible and offchain assets in an instruction, the consumed weight for executing the instruction,
/// and if executing the instruction would fail, the error thrown.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode)]
pub struct ExecuteInstructionInfo {
    /// Number of fungible tokens in the instruction.
    fungible_tokens: u32,
    /// Number of non fungible tokens in the instruction.
    non_fungible_tokens: u32,
    /// Number of off-chain assets in the instruction.
    off_chain_assets: u32,
    /// The weight needed for executing the instruction.
    consumed_weight: Weight,
    /// If the instruction would fail, contains the error.
    error: Option<String>,
}

impl ExecuteInstructionInfo {
    /// Creates an instance of [`ExecuteInstructionInfo`].
    pub fn new(
        fungible_tokens: u32,
        non_fungible_tokens: u32,
        off_chain_assets: u32,
        consumed_weight: Weight,
        error: Option<&str>,
    ) -> Self {
        Self {
            fungible_tokens,
            non_fungible_tokens,
            off_chain_assets,
            consumed_weight,
            error: error.map(|e| e.to_string()),
        }
    }
}
