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

//! # Settlement Module
//!
//! Settlement module manages all kinds of transfers and settlements of assets
//!
//! ## Overview
//!
//! The settlement module provides functionality to settle onchain as well as offchain trades between multiple parties.
//! All trades are settled under venues. An appropriately permissioned external agent
//! can allow/block certain venues from settling trades that involve their tokens.
//! An atomic settlement is called an Instruction. An instruction can contain multiple legs. Legs are essentially simple one to one transfers.
//! When an instruction is settled, either all legs are executed successfully or none are. In other words, if one of the leg fails due to
//! compliance failure, all other legs will also fail.
//!
//! An instruction must be authorized by all the counter parties involved for it to be executed.
//! An instruction can be set to automatically execute in the next block when all authorizations are received or at a particular block number.
//!
//! Offchain settlements are represented via receipts. If a leg has a receipt attached to it, it will not be executed onchain.
//! All other legs will be executed onchain during settlement.
//!
//! ## Dispatchable Functions
//!
//! - `create_venue` - Registers a new venue.
//! - `add_instruction` - Adds a new instruction.
//! - `affirm_instruction` - Provides affirmation to an existing instruction.
//! - `withdraw_affirmation` - Withdraw an existing affirmation to given instruction.
//! - `reject_instruction` - Rejects an existing instruction.
//! - `set_venue_filtering` - Enables or disabled venue filtering for a token.
//! - `allow_venues` - Allows additional venues to create instructions involving an asset.
//! - `disallow_venues` - Revokes permission given to venues for creating instructions involving a particular asset.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    storage::{with_transaction as frame_storage_with_transaction, TransactionOutcome},
    traits::{
        schedule::{DispatchTime, Named as ScheduleNamed},
        Get,
    },
    weights::Weight,
    IterableStorageDoubleMap,
};
use frame_system::{ensure_root, RawOrigin};
use pallet_base::{ensure_string_limited, try_next_post};
use pallet_identity::{self as identity, PermissionedCallOriginData};
use polymesh_common_utilities::{
    constants::queue_priority::SETTLEMENT_INSTRUCTION_EXECUTION_PRIORITY,
    traits::{
        asset, identity::Config as IdentityConfig, portfolio::PortfolioSubTrait, CommonConfig,
    },
    with_transaction,
    SystematicIssuers::Settlement as SettlementDID,
};
use polymesh_primitives::{
    impl_checked_inc, storage_migration_ver, Balance, IdentityId, PortfolioId, SecondaryKey, Ticker,
};
use polymesh_primitives_derive::VecU8StrongTyped;
use scale_info::TypeInfo;
use sp_runtime::traits::{One, Verify};
use sp_std::{collections::btree_set::BTreeSet, convert::TryFrom, prelude::*};

type Identity<T> = identity::Module<T>;
type System<T> = frame_system::Pallet<T>;
type Asset<T> = pallet_asset::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;

pub trait Config:
    frame_system::Config
    + CommonConfig
    + IdentityConfig
    + pallet_timestamp::Config
    + asset::Config
    + pallet_compliance_manager::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    /// A call type used by the scheduler.
    type Proposal: From<Call<Self>> + Into<<Self as IdentityConfig>::Proposal>;

    /// Scheduler of settlement instructions.
    type Scheduler: ScheduleNamed<
        Self::BlockNumber,
        <Self as Config>::Proposal,
        Self::SchedulerOrigin,
    >;
    /// Maximum legs that can be in a single instruction.
    type MaxLegsInInstruction: Get<u32>;
    /// Weight information for extrinsic of the settlement pallet.
    type WeightInfo: WeightInfo;
}

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
pub enum InstructionStatus {
    /// Invalid instruction or details pruned
    Unknown,
    /// Instruction is pending execution
    Pending,
    /// Instruction has failed execution
    Failed,
}

impl Default for InstructionStatus {
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
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct InstructionId(pub u64);
impl_checked_inc!(InstructionId);

impl InstructionId {
    /// Converts an instruction id into a scheduler name.
    pub fn execution_name(&self) -> Vec<u8> {
        (polymesh_common_utilities::constants::schedule_name_prefix::SETTLEMENT_INSTRUCTION_EXECUTION, self.0).encode()
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
    /// Status of the instruction
    pub status: InstructionStatus,
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

pub trait WeightInfo {
    fn create_venue(d: u32, u: u32) -> Weight;
    fn update_venue_details(d: u32) -> Weight;
    fn update_venue_type() -> Weight;
    fn update_venue_signers(u: u32) -> Weight;
    fn add_instruction(u: u32) -> Weight;
    fn add_and_affirm_instruction(u: u32) -> Weight;
    fn affirm_instruction(l: u32) -> Weight;
    fn withdraw_affirmation(u: u32) -> Weight;
    fn affirm_with_receipts(r: u32) -> Weight;
    fn set_venue_filtering() -> Weight;
    fn allow_venues(u: u32) -> Weight;
    fn disallow_venues(u: u32) -> Weight;
    fn reject_instruction(u: u32) -> Weight;
    fn change_receipt_validity() -> Weight;
    fn execute_scheduled_instruction(l: u32) -> Weight;
    fn reschedule_instruction() -> Weight;
    fn execute_manual_instruction(l: u32) -> Weight;

    // Some multiple paths based extrinsic.
    // TODO: Will be removed once we get the worst case weight.
    fn add_instruction_with_settle_on_block_type(u: u32) -> Weight;
    fn add_and_affirm_instruction_with_settle_on_block_type(u: u32) -> Weight;
    fn add_instruction_with_memo_and_settle_on_block_type(u: u32) -> Weight;
    fn add_and_affirm_instruction_with_memo_and_settle_on_block_type(u: u32) -> Weight;
}

type EnsureValidInstructionResult<AccountId, Moment, BlockNumber> = Result<
    (
        IdentityId,
        Option<SecondaryKey<AccountId>>,
        Instruction<Moment, BlockNumber>,
    ),
    DispatchError,
>;

decl_event!(
    pub enum Event<T>
    where
        Moment = <T as pallet_timestamp::Config>::Moment,
        BlockNumber = <T as frame_system::Config>::BlockNumber,
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// A new venue has been created (did, venue_id, details, type)
        VenueCreated(IdentityId, VenueId, VenueDetails, VenueType),
        /// An existing venue's details has been updated (did, venue_id, details)
        VenueDetailsUpdated(IdentityId, VenueId, VenueDetails),
        /// An existing venue's type has been updated (did, venue_id, type)
        VenueTypeUpdated(IdentityId, VenueId, VenueType),
        /// A new instruction has been created
        /// (did, venue_id, instruction_id, settlement_type, trade_date, value_date, legs, memo)
        InstructionCreated(
            IdentityId,
            VenueId,
            InstructionId,
            SettlementType<BlockNumber>,
            Option<Moment>,
            Option<Moment>,
            Vec<Leg>,
            Option<InstructionMemo>,
        ),
        /// An instruction has been affirmed (did, portfolio, instruction_id)
        InstructionAffirmed(IdentityId, PortfolioId, InstructionId),
        /// An affirmation has been withdrawn (did, portfolio, instruction_id)
        AffirmationWithdrawn(IdentityId, PortfolioId, InstructionId),
        /// An instruction has been rejected (did, instruction_id)
        InstructionRejected(IdentityId, InstructionId),
        /// A receipt has been claimed (did, instruction_id, leg_id, receipt_uid, signer, receipt metadata)
        ReceiptClaimed(
            IdentityId,
            InstructionId,
            LegId,
            u64,
            AccountId,
            ReceiptMetadata,
        ),
        /// A receipt has been invalidated (did, signer, receipt_uid, validity)
        ReceiptValidityChanged(IdentityId, AccountId, u64, bool),
        /// A receipt has been unclaimed (did, instruction_id, leg_id, receipt_uid, signer)
        ReceiptUnclaimed(IdentityId, InstructionId, LegId, u64, AccountId),
        /// Venue filtering has been enabled or disabled for a ticker (did, ticker, filtering_enabled)
        VenueFiltering(IdentityId, Ticker, bool),
        /// Venues added to allow list (did, ticker, vec<venue_id>)
        VenuesAllowed(IdentityId, Ticker, Vec<VenueId>),
        /// Venues added to block list (did, ticker, vec<venue_id>)
        VenuesBlocked(IdentityId, Ticker, Vec<VenueId>),
        /// Execution of a leg failed (did, instruction_id, leg_id)
        LegFailedExecution(IdentityId, InstructionId, LegId),
        /// Instruction failed execution (did, instruction_id)
        InstructionFailed(IdentityId, InstructionId),
        /// Instruction executed successfully(did, instruction_id)
        InstructionExecuted(IdentityId, InstructionId),
        /// Venue not part of the token's allow list (did, Ticker, venue_id)
        VenueUnauthorized(IdentityId, Ticker, VenueId),
        /// Scheduling of instruction fails.
        SchedulingFailed(DispatchError),
        /// Instruction is rescheduled.
        /// (caller DID, instruction_id)
        InstructionRescheduled(IdentityId, InstructionId),
        /// An existing venue's signers has been updated (did, venue_id, signers, update_type)
        VenueSignersUpdated(IdentityId, VenueId, Vec<AccountId>, bool),
        /// Settlement manually executed (did, id)
        SettlementManuallyExecuted(IdentityId, InstructionId),
        /// Failed to execute instruction.
        FailedToExecuteInstruction(InstructionId, DispatchError),
    }
);

decl_error! {
    /// Errors for the Settlement module.
    pub enum Error for Module<T: Config> {
        /// Venue does not exist.
        InvalidVenue,
        /// Sender does not have required permissions.
        Unauthorized,
        /// No pending affirmation for the provided instruction.
        NoPendingAffirm,
        /// Instruction has not been affirmed.
        InstructionNotAffirmed,
        /// Provided instruction is not pending execution.
        InstructionNotPending,
        /// Provided instruction is not failing execution.
        InstructionNotFailed,
        /// Provided leg is not pending execution.
        LegNotPending,
        /// Signer is not authorized by the venue.
        UnauthorizedSigner,
        /// Receipt already used.
        ReceiptAlreadyClaimed,
        /// Receipt not used yet.
        ReceiptNotClaimed,
        /// Venue does not have required permissions.
        UnauthorizedVenue,
        /// While affirming the transfer, system failed to lock the assets involved.
        FailedToLockTokens,
        /// Instruction failed to execute.
        InstructionFailed,
        /// Instruction has invalid dates
        InstructionDatesInvalid,
        /// Instruction's target settle block reached.
        InstructionSettleBlockPassed,
        /// Offchain signature is invalid.
        InvalidSignature,
        /// Sender and receiver are the same.
        SameSenderReceiver,
        /// Portfolio in receipt does not match with portfolios provided by the user.
        PortfolioMismatch,
        /// The provided settlement block number is in the past and cannot be used by the scheduler.
        SettleOnPastBlock,
        /// Portfolio based actions require at least one portfolio to be provided as input.
        NoPortfolioProvided,
        /// The current instruction affirmation status does not support the requested action.
        UnexpectedAffirmationStatus,
        /// Scheduling of an instruction fails.
        FailedToSchedule,
        /// Legs count should matches with the total number of legs in which given portfolio act as `from_portfolio`.
        LegCountTooSmall,
        /// Instruction status is unknown
        UnknownInstruction,
        /// Maximum legs that can be in a single instruction.
        InstructionHasTooManyLegs,
        /// Signer is already added to venue.
        SignerAlreadyExists,
        /// Signer is not added to venue.
        SignerDoesNotExist,
        /// Instruction leg amount can't be zero
        ZeroAmount,
        /// Instruction settlement block has not yet been reached.
        InstructionSettleBlockNotReached,
        /// The caller is not a party of this instruction.
        CallerIsNotAParty
    }
}

storage_migration_ver!(0);

decl_storage! {
    trait Store for Module<T: Config> as Settlement {
        /// Info about a venue. venue_id -> venue
        pub VenueInfo get(fn venue_info): map hasher(twox_64_concat) VenueId => Option<Venue>;

        /// Free-form text about a venue. venue_id -> `VenueDetails`
        /// Only needed for the UI.
        pub Details get(fn details): map hasher(twox_64_concat) VenueId => VenueDetails;

        /// Instructions under a venue.
        /// Only needed for the UI.
        ///
        /// venue_id -> instruction_id -> ()
        pub VenueInstructions get(fn venue_instructions):
            double_map hasher(twox_64_concat) VenueId,
                       hasher(twox_64_concat) InstructionId
                    => ();

        /// Signers allowed by the venue. (venue_id, signer) -> bool
        VenueSigners get(fn venue_signers):
            double_map hasher(twox_64_concat) VenueId,
                       hasher(twox_64_concat) T::AccountId
                    => bool;
        /// Array of venues created by an identity. Only needed for the UI. IdentityId -> Vec<venue_id>
        UserVenues get(fn user_venues): map hasher(twox_64_concat) IdentityId => Vec<VenueId>;
        /// Details about an instruction. instruction_id -> instruction_details
        InstructionDetails get(fn instruction_details):
            map hasher(twox_64_concat) InstructionId => Instruction<T::Moment, T::BlockNumber>;
        /// Legs under an instruction. (instruction_id, leg_id) -> Leg
        pub InstructionLegs get(fn instruction_legs):
            double_map hasher(twox_64_concat) InstructionId, hasher(twox_64_concat) LegId => Leg;
        /// Status of a leg under an instruction. (instruction_id, leg_id) -> LegStatus
        InstructionLegStatus get(fn instruction_leg_status):
            double_map hasher(twox_64_concat) InstructionId, hasher(twox_64_concat) LegId => LegStatus<T::AccountId>;
        /// Number of affirmations pending before instruction is executed. instruction_id -> affirm_pending
        InstructionAffirmsPending get(fn instruction_affirms_pending): map hasher(twox_64_concat) InstructionId => u64;
        /// Tracks affirmations received for an instruction. (instruction_id, counter_party) -> AffirmationStatus
        AffirmsReceived get(fn affirms_received): double_map hasher(twox_64_concat) InstructionId, hasher(twox_64_concat) PortfolioId => AffirmationStatus;
        /// Helps a user track their pending instructions and affirmations (only needed for UI).
        /// (counter_party, instruction_id) -> AffirmationStatus
        UserAffirmations get(fn user_affirmations):
            double_map hasher(twox_64_concat) PortfolioId, hasher(twox_64_concat) InstructionId => AffirmationStatus;
        /// Tracks redemption of receipts. (signer, receipt_uid) -> receipt_used
        ReceiptsUsed get(fn receipts_used): double_map hasher(twox_64_concat) T::AccountId, hasher(blake2_128_concat) u64 => bool;
        /// Tracks if a token has enabled filtering venues that can create instructions involving their token. Ticker -> filtering_enabled
        VenueFiltering get(fn venue_filtering): map hasher(blake2_128_concat) Ticker => bool;
        /// Venues that are allowed to create instructions involving a particular ticker. Only used if filtering is enabled.
        /// (ticker, venue_id) -> allowed
        VenueAllowList get(fn venue_allow_list): double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) VenueId => bool;
        /// Number of venues in the system (It's one more than the actual number)
        VenueCounter get(fn venue_counter) build(|_| VenueId(1u64)): VenueId;
        /// Number of instructions in the system (It's one more than the actual number)
        InstructionCounter get(fn instruction_counter) build(|_| InstructionId(1u64)): InstructionId;
        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(0)): Version;
        /// Instruction memo
        InstructionMemos get(fn memo): map hasher(twox_64_concat) InstructionId => Option<InstructionMemo>;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Registers a new venue.
        ///
        /// * `details` - Extra details about a venue
        /// * `signers` - Array of signers that are allowed to sign receipts for this venue
        /// * `typ` - Type of venue being created
        #[weight = <T as Config>::WeightInfo::create_venue(details.len() as u32, signers.len() as u32)]
        pub fn create_venue(origin, details: VenueDetails, signers: Vec<T::AccountId>, typ: VenueType) {
            // Ensure permissions and details limit.
            let did = Identity::<T>::ensure_perms(origin)?;
            ensure_string_limited::<T>(&details)?;

            // Advance venue counter.
            // NB: Venue counter starts with 1.
            let id = VenueCounter::try_mutate(try_next_post::<T, _>)?;

            // Other commits to storage + emit event.
            let venue = Venue { creator: did, venue_type: typ };
            VenueInfo::insert(id, venue);
            Details::insert(id, details.clone());
            for signer in signers {
                <VenueSigners<T>>::insert(id, signer, true);
            }
            UserVenues::append(did, id);
            Self::deposit_event(RawEvent::VenueCreated(did, id, details, typ));
        }

        /// Edit a venue's details.
        ///
        /// * `id` specifies the ID of the venue to edit.
        /// * `details` specifies the updated venue details.
        #[weight = <T as Config>::WeightInfo::update_venue_details(details.len() as u32)]
        pub fn update_venue_details(origin, id: VenueId, details: VenueDetails) -> DispatchResult {
            ensure_string_limited::<T>(&details)?;
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::venue_for_management(id, did)?;

            // Commit to storage.
            Details::insert(id, details.clone());
            Self::deposit_event(RawEvent::VenueDetailsUpdated(did, id, details));
            Ok(())
        }

        /// Edit a venue's type.
        ///
        /// * `id` specifies the ID of the venue to edit.
        /// * `type` specifies the new type of the venue.
        #[weight = <T as Config>::WeightInfo::update_venue_type()]
        pub fn update_venue_type(origin, id: VenueId, typ: VenueType) -> DispatchResult {
            let did = Identity::<T>::ensure_perms(origin)?;

            let mut venue = Self::venue_for_management(id, did)?;
            venue.venue_type = typ;
            VenueInfo::insert(id, venue);

            Self::deposit_event(RawEvent::VenueTypeUpdated(did, id, typ));
            Ok(())
        }

        /// Deprecated. Use `add_instruction_with_memo` instead.
        /// Adds a new instruction.
        ///
        /// # Arguments
        /// * `venue_id` - ID of the venue this instruction belongs to.
        /// * `settlement_type` - Defines if the instruction should be settled
        ///    in the next block after receiving all affirmations or waiting till a specific block.
        /// * `trade_date` - Optional date from which people can interact with this instruction.
        /// * `value_date` - Optional date after which the instruction should be settled (not enforced)
        /// * `legs` - Legs included in this instruction.
        ///
        /// # Weight
        /// `950_000_000 + 1_000_000 * legs.len()`
        #[weight = <T as Config>::WeightInfo::add_instruction_with_settle_on_block_type(legs.len() as u32)
        .saturating_add(
            <T as Config>::WeightInfo::execute_scheduled_instruction(legs.len() as u32)
        )]
        pub fn add_instruction(
            origin,
            venue_id: VenueId,
            settlement_type: SettlementType<T::BlockNumber>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, None)?;
        }

        /// Deprecated. Use `add_and_affirm_instruction_with_memo` instead.
        /// Adds and affirms a new instruction.
        ///
        /// # Arguments
        /// * `venue_id` - ID of the venue this instruction belongs to.
        /// * `settlement_type` - Defines if the instruction should be settled
        ///    in the next block after receiving all affirmations or waiting till a specific block.
        /// * `trade_date` - Optional date from which people can interact with this instruction.
        /// * `value_date` - Optional date after which the instruction should be settled (not enforced)
        /// * `legs` - Legs included in this instruction.
        /// * `portfolios` - Portfolios that the sender controls and wants to use in this affirmations.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::add_and_affirm_instruction_with_settle_on_block_type(legs.len() as u32)
        .saturating_add(
            <T as Config>::WeightInfo::execute_scheduled_instruction(legs.len() as u32)
        )]
        pub fn add_and_affirm_instruction(
            origin,
            venue_id: VenueId,
            settlement_type: SettlementType<T::BlockNumber>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
            portfolios: Vec<PortfolioId>,
        ) -> DispatchResult {
            let did = Identity::<T>::ensure_perms(origin.clone())?;
            with_transaction(|| {
                let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();
                let legs_count = legs.iter().filter(|l| portfolios_set.contains(&l.from)).count() as u32;
                let instruction_id = Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, None)?;
                Self::affirm_and_maybe_schedule_instruction(origin, instruction_id, portfolios_set.into_iter(), legs_count)
            })
        }

        /// Provide affirmation to an existing instruction.
        ///
        /// # Arguments
        /// * `id` - Instruction id to affirm.
        /// * `portfolios` - Portfolios that the sender controls and wants to affirm this instruction.
        /// * `max_legs_count` - Number of legs that need to be  affirmed.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::affirm_instruction(*max_legs_count as u32)]
        pub fn affirm_instruction(origin, id: InstructionId, portfolios: Vec<PortfolioId>, max_legs_count: u32) -> DispatchResult {
            Self::affirm_and_maybe_schedule_instruction(origin, id, portfolios.into_iter(), max_legs_count)
        }

        /// Withdraw an affirmation for a given instruction.
        ///
        /// # Arguments
        /// * `id` - Instruction id for that affirmation get withdrawn.
        /// * `portfolios` - Portfolios that the sender controls and wants to withdraw affirmation.
        /// * `max_legs_count` - Number of legs that need to be un-affirmed.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::withdraw_affirmation(*max_legs_count as u32)]
        pub fn withdraw_affirmation(origin, id: InstructionId, portfolios: Vec<PortfolioId>, max_legs_count: u32) {
            let (did, secondary_key, details) = Self::ensure_origin_perm_and_instruction_validity(origin, id, false)?;
            let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();

            // Withdraw an affirmation.
            Self::unsafe_withdraw_instruction_affirmation(did, id, portfolios_set, secondary_key.as_ref(), max_legs_count)?;
            if details.settlement_type == SettlementType::SettleOnAffirmation {
                // Cancel the scheduled task for the execution of a given instruction.
                let _ = T::Scheduler::cancel_named(id.execution_name());
            }
        }

        /// Rejects an existing instruction.
        ///
        /// # Arguments
        /// * `id` - Instruction id to reject.
        /// * `portfolio` - Portfolio to reject the instruction.
        /// * `num_of_legs` - Number of legs in the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::reject_instruction(*num_of_legs)]
        pub fn reject_instruction(origin, id: InstructionId, portfolio: PortfolioId, num_of_legs: u32) {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            ensure!(
                Self::instruction_details(id).status != InstructionStatus::Unknown,
                Error::<T>::UnknownInstruction
            );

            let legs = InstructionLegs::iter_prefix(id).collect::<Vec<_>>();

            // Ensure num_of_legs is correct.
            ensure!(
                legs.len() <= num_of_legs as usize,
                Error::<T>::LegCountTooSmall
            );

            // Ensure that the caller is a party of this instruction.
            T::Portfolio::ensure_portfolio_custody_and_permission(portfolio, primary_did, secondary_key.as_ref())?;
            ensure!(
                legs.iter().any(|(_, leg)| [leg.from, leg.to].contains(&portfolio)),
                Error::<T>::CallerIsNotAParty
            );

            Self::unsafe_unclaim_receipts(id, &legs);
            Self::unchecked_release_locks(id, &legs);
            let _ = T::Scheduler::cancel_named(id.execution_name());
            Self::prune_instruction(id);
            Self::deposit_event(RawEvent::InstructionRejected(primary_did, id));
        }

        /// Accepts an instruction and claims a signed receipt.
        ///
        /// # Arguments
        /// * `id` - Target instruction id.
        /// * `leg_id` - Target leg id for the receipt
        /// * `receipt_uid` - Receipt ID generated by the signer.
        /// * `signer` - Signer of the receipt.
        /// * `signed_data` - Signed receipt.
        /// * `portfolios` - Portfolios that the sender controls and wants to accept this instruction with
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::affirm_with_receipts(*max_legs_count as u32).max(<T as Config>::WeightInfo::affirm_instruction(*max_legs_count as u32))]
        pub fn affirm_with_receipts(origin, id: InstructionId, receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>, portfolios: Vec<PortfolioId>, max_legs_count: u32) -> DispatchResult {
            Self::affirm_with_receipts_and_maybe_schedule_instruction(origin, id, receipt_details, portfolios, max_legs_count)
        }

        /// Placeholder for removed `claim_receipt`
        #[weight = 1_000]
        pub fn placeholder_claim_receipt(_origin) {
        }

        /// Placeholder for removed `unclaim_receipt`
        #[weight = 1_000]
        pub fn placeholder_unclaim_receipt(_origin) {
        }

        /// Enables or disabled venue filtering for a token.
        ///
        /// # Arguments
        /// * `ticker` - Ticker of the token in question.
        /// * `enabled` - Boolean that decides if the filtering should be enabled.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_venue_filtering()]
        pub fn set_venue_filtering(origin, ticker: Ticker, enabled: bool) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            if enabled {
                VenueFiltering::insert(ticker, enabled);
            } else {
                VenueFiltering::remove(ticker);
            }
            Self::deposit_event(RawEvent::VenueFiltering(did, ticker, enabled));
        }

        /// Allows additional venues to create instructions involving an asset.
        ///
        /// * `ticker` - Ticker of the token in question.
        /// * `venues` - Array of venues that are allowed to create instructions for the token in question.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::allow_venues(venues.len() as u32)]
        pub fn allow_venues(origin, ticker: Ticker, venues: Vec<VenueId>) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            for venue in &venues {
                VenueAllowList::insert(&ticker, venue, true);
            }
            Self::deposit_event(RawEvent::VenuesAllowed(did, ticker, venues));
        }

        /// Revokes permission given to venues for creating instructions involving a particular asset.
        ///
        /// * `ticker` - Ticker of the token in question.
        /// * `venues` - Array of venues that are no longer allowed to create instructions for the token in question.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::disallow_venues(venues.len() as u32)]
        pub fn disallow_venues(origin, ticker: Ticker, venues: Vec<VenueId>) {
            let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            for venue in &venues {
                VenueAllowList::remove(&ticker, venue);
            }
            Self::deposit_event(RawEvent::VenuesBlocked(did, ticker, venues));
        }

        /// Marks a receipt issued by the caller as claimed or not claimed.
        /// This allows the receipt issuer to invalidate an already issued receipt or revalidate an already claimed receipt.
        ///
        /// * `receipt_uid` - Unique ID of the receipt.
        /// * `validity` - New validity of the receipt.
        #[weight = <T as Config>::WeightInfo::change_receipt_validity()]
        pub fn change_receipt_validity(origin, receipt_uid: u64, validity: bool) {
            let PermissionedCallOriginData {
                primary_did,
                sender: signer,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            <ReceiptsUsed<T>>::insert(&signer, receipt_uid, !validity);
            Self::deposit_event(RawEvent::ReceiptValidityChanged(primary_did, signer, receipt_uid, validity));
        }

        /// Root callable extrinsic, used as an internal call to execute a scheduled settlement instruction.
        #[weight = <T as Config>::WeightInfo::execute_scheduled_instruction(*_legs_count)]
        fn execute_scheduled_instruction(origin, id: InstructionId, _legs_count: u32) {
            ensure_root(origin)?;
            if let Err(e) = Self::execute_instruction_retryable(id) {
                Self::deposit_event(RawEvent::FailedToExecuteInstruction(id, e));
            }
        }

        /// Reschedules a failed instruction.
        ///
        /// # Arguments
        /// * `id` - Target instruction id to reschedule.
        ///
        /// # Permissions
        /// * Portfolio
        ///
        /// # Errors
        /// * `InstructionNotFailed` - Instruction not in a failed state or does not exist.
        #[weight = <T as Config>::WeightInfo::reschedule_instruction()]
        pub fn reschedule_instruction(origin, id: InstructionId) {
            let did = Identity::<T>::ensure_perms(origin)?;

            <InstructionDetails<T>>::try_mutate(id, |details| {
                ensure!(details.status == InstructionStatus::Failed, Error::<T>::InstructionNotFailed);
                details.status = InstructionStatus::Pending;
                Result::<_, Error<T>>::Ok(())
            })?;

            // Schedule instruction to be executed in the next block.
            let execution_at = System::<T>::block_number() + One::one();
            Self::schedule_instruction(id, execution_at, InstructionLegs::iter_prefix(id).count() as u32);

            Self::deposit_event(RawEvent::InstructionRescheduled(did, id));
        }

        /// Edit a venue's signers.
        /// * `id` specifies the ID of the venue to edit.
        /// * `signers` specifies the signers to add/remove.
        /// * `add_signers` specifies the update type add/remove of venue where add is true and remove is false.
        #[weight = <T as Config>::WeightInfo::update_venue_signers(signers.len() as u32)]
        pub fn update_venue_signers(origin, id: VenueId, signers: Vec<T::AccountId>, add_signers: bool) {
            let did = Identity::<T>::ensure_perms(origin)?;

            Self::base_update_venue_signers(did, id, signers, add_signers)?;
        }

        /// Adds a new instruction with memo.
        ///
        /// # Arguments
        /// * `venue_id` - ID of the venue this instruction belongs to.
        /// * `settlement_type` - Defines if the instruction should be settled
        ///    in the next block after receiving all affirmations or waiting till a specific block.
        /// * `trade_date` - Optional date from which people can interact with this instruction.
        /// * `value_date` - Optional date after which the instruction should be settled (not enforced)
        /// * `legs` - Legs included in this instruction.
        /// * `memo` - Memo field for this instruction.
        ///
        /// # Weight
        /// `950_000_000 + 1_000_000 * legs.len()`
        #[weight = <T as Config>::WeightInfo::add_instruction_with_memo_and_settle_on_block_type(legs.len() as u32)
        .saturating_add(
            <T as Config>::WeightInfo::execute_scheduled_instruction(legs.len() as u32)
        )]
        pub fn add_instruction_with_memo(
            origin,
            venue_id: VenueId,
            settlement_type: SettlementType<T::BlockNumber>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
            instruction_memo: Option<InstructionMemo>,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, instruction_memo)?;
        }

        /// Adds and affirms a new instruction.
        ///
        /// # Arguments
        /// * `venue_id` - ID of the venue this instruction belongs to.
        /// * `settlement_type` - Defines if the instruction should be settled
        ///    in the next block after receiving all affirmations or waiting till a specific block.
        /// * `trade_date` - Optional date from which people can interact with this instruction.
        /// * `value_date` - Optional date after which the instruction should be settled (not enforced)
        /// * `legs` - Legs included in this instruction.
        /// * `portfolios` - Portfolios that the sender controls and wants to use in this affirmations.
        /// * `memo` - Memo field for this instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::add_and_affirm_instruction_with_memo_and_settle_on_block_type(legs.len() as u32)
        .saturating_add(
            <T as Config>::WeightInfo::execute_scheduled_instruction(legs.len() as u32)
        )]
        pub fn add_and_affirm_instruction_with_memo(
            origin,
            venue_id: VenueId,
            settlement_type: SettlementType<T::BlockNumber>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
            portfolios: Vec<PortfolioId>,
            instruction_memo: Option<InstructionMemo>,
        ) -> DispatchResult {
            let did = Identity::<T>::ensure_perms(origin.clone())?;
            with_transaction(|| {
                let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();
                let legs_count = legs.iter().filter(|l| portfolios_set.contains(&l.from)).count() as u32;
                let instruction_id = Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, instruction_memo)?;
                Self::affirm_and_maybe_schedule_instruction(origin, instruction_id, portfolios_set.into_iter(), legs_count)
            })
        }

        /// Manually execute settlement
        ///
        /// # Arguments
        /// * `id` - Target instruction id to reschedule.
        /// * `_legs_count` - Legs included in this instruction.
        ///
        /// # Errors
        /// * `InstructionNotFailed` - Instruction not in a failed state or does not exist.
        #[weight = <T as Config>::WeightInfo::execute_manual_instruction(*legs_count)]
        pub fn execute_manual_instruction(origin, id: InstructionId, legs_count: u32, portfolio: Option<PortfolioId>) {
            // check origin has the permissions required and valid instruction
            let (did, sk, instruction_details) = Self::ensure_origin_perm_and_instruction_validity(origin, id, true)?;

            // Check for portfolio
            match portfolio {
                Some(portfolio) => {
                    // Ensure that the caller is a party of this instruction.
                    T::Portfolio::ensure_portfolio_custody_and_permission(portfolio, did, sk.as_ref())?;
                    let mut legs = InstructionLegs::iter_prefix(id);
                    ensure!(
                        legs.any(|(_, leg)| [leg.from, leg.to].contains(&portfolio)),
                        Error::<T>::CallerIsNotAParty
                    );
                }
                None => {
                    // Ensure venue exists & sender is its creator.
                    Self::venue_for_management(instruction_details.venue_id, did)?;
                }
            }

            // check that the instruction leg count matches
            ensure!(InstructionLegs::iter_prefix(id).count() as u32 <= legs_count, Error::<T>::LegCountTooSmall);

            // Executes the instruction
            Self::execute_instruction_retryable(id)?;

            Self::deposit_event(RawEvent::SettlementManuallyExecuted(did, id));
        }

    }
}

impl<T: Config> Module<T> {
    fn lock_via_leg(leg: &Leg) -> DispatchResult {
        T::Portfolio::lock_tokens(&leg.from, &leg.asset, leg.amount)
    }

    fn unlock_via_leg(leg: &Leg) -> DispatchResult {
        T::Portfolio::unlock_tokens(&leg.from, &leg.asset, leg.amount)
    }

    /// Ensure origin call permission and the given instruction validity.
    fn ensure_origin_perm_and_instruction_validity(
        origin: <T as frame_system::Config>::Origin,
        id: InstructionId,
        is_execute: bool,
    ) -> EnsureValidInstructionResult<T::AccountId, T::Moment, T::BlockNumber> {
        let PermissionedCallOriginData {
            primary_did,
            secondary_key,
            ..
        } = Identity::<T>::ensure_origin_call_permissions(origin)?;
        Ok((
            primary_did,
            secondary_key,
            Self::ensure_instruction_validity(id, is_execute)?,
        ))
    }

    // Extract `Venue` with `id`, assuming it was created by `did`, or error.
    fn venue_for_management(id: VenueId, did: IdentityId) -> Result<Venue, DispatchError> {
        // Ensure venue exists & that DID created it.
        let venue = Self::venue_info(id).ok_or(Error::<T>::InvalidVenue)?;
        ensure!(venue.creator == did, Error::<T>::Unauthorized);
        Ok(venue)
    }

    pub fn base_add_instruction(
        did: IdentityId,
        venue_id: VenueId,
        settlement_type: SettlementType<T::BlockNumber>,
        trade_date: Option<T::Moment>,
        value_date: Option<T::Moment>,
        legs: Vec<Leg>,
        memo: Option<InstructionMemo>,
    ) -> Result<InstructionId, DispatchError> {
        // Ensure instruction does not have too many legs.
        ensure!(
            legs.len() <= T::MaxLegsInInstruction::get() as usize,
            Error::<T>::InstructionHasTooManyLegs
        );

        match &settlement_type {
            SettlementType::SettleOnBlock(block_number) => {
                // Ensure that the scheduled block number is in the future so that `T::Scheduler::schedule_named`
                // doesn't fail.
                ensure!(
                    *block_number > System::<T>::block_number(),
                    Error::<T>::SettleOnPastBlock
                );
            }
            SettlementType::SettleManual(_block_number) => {}
            _ => {}
        }

        // Ensure that instruction dates are valid.
        if let (Some(trade_date), Some(value_date)) = (trade_date, value_date) {
            ensure!(
                value_date >= trade_date,
                Error::<T>::InstructionDatesInvalid
            );
        }

        // Ensure venue exists & sender is its creator.
        Self::venue_for_management(venue_id, did)?;

        // Create a list of unique counter parties involved in the instruction.
        let mut counter_parties = BTreeSet::new();
        let mut tickers = BTreeSet::new();
        for leg in &legs {
            ensure!(leg.from != leg.to, Error::<T>::SameSenderReceiver);
            // Check if the venue is part of the token's list of allowed venues.
            // Only check each ticker once.
            if tickers.insert(leg.asset) && Self::venue_filtering(leg.asset) {
                ensure!(
                    Self::venue_allow_list(leg.asset, venue_id),
                    Error::<T>::UnauthorizedVenue
                );
            }

            ensure!(leg.amount != 0, Error::<T>::ZeroAmount);
            counter_parties.insert(leg.from);
            counter_parties.insert(leg.to);
        }

        // Advance and get next `instruction_id`.
        let instruction_id = InstructionCounter::try_mutate(try_next_post::<T, _>)?;

        let instruction = Instruction {
            instruction_id,
            venue_id,
            status: InstructionStatus::Pending,
            settlement_type,
            created_at: Some(<pallet_timestamp::Pallet<T>>::get()),
            trade_date,
            value_date,
        };

        // Write data to storage.
        for counter_party in &counter_parties {
            UserAffirmations::insert(counter_party, instruction_id, AffirmationStatus::Pending);
        }

        for (i, leg) in legs.iter().enumerate() {
            InstructionLegs::insert(
                instruction_id,
                u64::try_from(i).map(LegId).unwrap_or_default(),
                leg.clone(),
            );
        }

        if let SettlementType::SettleOnBlock(block_number) = settlement_type {
            Self::schedule_instruction(instruction_id, block_number, legs.len() as u32);
        }

        <InstructionDetails<T>>::insert(instruction_id, instruction);

        InstructionAffirmsPending::insert(
            instruction_id,
            u64::try_from(counter_parties.len()).unwrap_or_default(),
        );
        VenueInstructions::insert(venue_id, instruction_id, ());
        if let Some(ref memo) = memo {
            InstructionMemos::insert(instruction_id, &memo);
        }

        Self::deposit_event(RawEvent::InstructionCreated(
            did,
            venue_id,
            instruction_id,
            settlement_type,
            trade_date,
            value_date,
            legs,
            memo,
        ));

        Ok(instruction_id)
    }

    fn unsafe_withdraw_instruction_affirmation(
        did: IdentityId,
        id: InstructionId,
        portfolios: BTreeSet<PortfolioId>,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        max_legs_count: u32,
    ) -> Result<u32, DispatchError> {
        // checks custodianship of portfolios and affirmation status
        Self::ensure_portfolios_and_affirmation_status(
            id,
            &portfolios,
            did,
            secondary_key,
            &[AffirmationStatus::Affirmed],
        )?;
        // Unlock tokens that were previously locked during the affirmation
        let (total_leg_count, filtered_legs) =
            Self::filtered_legs(id, &portfolios, max_legs_count)?;
        for (leg_id, leg_details) in filtered_legs {
            match Self::instruction_leg_status(id, leg_id) {
                LegStatus::ExecutionToBeSkipped(signer, receipt_uid) => {
                    // Receipt was claimed for this instruction. Therefore, no token unlocking is required, we just unclaim the receipt.
                    <ReceiptsUsed<T>>::insert(&signer, receipt_uid, false);
                    Self::deposit_event(RawEvent::ReceiptUnclaimed(
                        did,
                        id,
                        leg_id,
                        receipt_uid,
                        signer,
                    ));
                }
                LegStatus::ExecutionPending => {
                    // Tokens are locked, need to be unlocked.
                    Self::unlock_via_leg(&leg_details)?;
                }
                LegStatus::PendingTokenLock => {
                    return Err(Error::<T>::InstructionNotAffirmed.into());
                }
            };
            <InstructionLegStatus<T>>::insert(id, leg_id, LegStatus::PendingTokenLock);
        }

        // Updates storage.
        for portfolio in &portfolios {
            UserAffirmations::insert(portfolio, id, AffirmationStatus::Pending);
            AffirmsReceived::remove(id, portfolio);
            Self::deposit_event(RawEvent::AffirmationWithdrawn(did, *portfolio, id));
        }

        InstructionAffirmsPending::mutate(id, |affirms_pending| {
            *affirms_pending += u64::try_from(portfolios.len()).unwrap_or_default()
        });

        Ok(total_leg_count)
    }

    fn ensure_instruction_validity(
        id: InstructionId,
        is_execute: bool,
    ) -> Result<Instruction<T::Moment, T::BlockNumber>, DispatchError> {
        let details = Self::instruction_details(id);
        ensure!(
            details.status != InstructionStatus::Unknown,
            Error::<T>::UnknownInstruction
        );

        match (details.settlement_type, is_execute) {
            // is_execute is true for execution
            (SettlementType::SettleOnBlock(block_number), true) => {
                // Ensures block number is less than or equal to current block number.
                ensure!(
                    block_number <= System::<T>::block_number(),
                    Error::<T>::InstructionSettleBlockNotReached
                );
            }
            // is_execute is false for affirmation
            (SettlementType::SettleOnBlock(block_number), false) => {
                // Ensures block number is greater than current block number.
                ensure!(
                    block_number > System::<T>::block_number(),
                    Error::<T>::InstructionSettleBlockPassed
                );
            }
            (SettlementType::SettleManual(block_number), true) => {
                // Ensures block number is less than  or equal to current block number.
                ensure!(
                    block_number <= System::<T>::block_number(),
                    Error::<T>::InstructionSettleBlockNotReached
                );
            }
            (_, _) => {}
        }

        Ok(details)
    }

    /// Execute the instruction with `instruction_id`, pruning it on success.
    /// On error, set the instruction status to failed.
    fn execute_instruction_retryable(id: InstructionId) -> Result<u32, DispatchError> {
        let result = Self::execute_instruction(id);
        if result.is_ok() {
            Self::prune_instruction(id);
        } else if <InstructionDetails<T>>::contains_key(id) {
            <InstructionDetails<T>>::mutate(id, |d| d.status = InstructionStatus::Failed);
        }
        result
    }

    fn execute_instruction(instruction_id: InstructionId) -> Result<u32, DispatchError> {
        // Verifies that there are no pending affirmations for the given instruction
        ensure!(
            Self::instruction_affirms_pending(instruction_id) == 0,
            Error::<T>::InstructionFailed
        );

        // Verifies that the instruction is not in a Failed or in an Unknown state
        let details = Self::instruction_details(instruction_id);
        ensure!(
            details.status == InstructionStatus::Pending,
            Error::<T>::InstructionNotPending
        );

        let mut instruction_legs: Vec<(LegId, Leg)> =
            InstructionLegs::iter_prefix(instruction_id).collect();
        // NB: The order of execution of the legs matter in some edge cases around compliance.
        // E.g: Consider a token with a total supply of 100 and maximum percentage ownership of 10%.
        // In a given moment, Alice owns 10 tokens, Bob owns 5 and Charlie owns 0.
        // Now, consider one instruction with two legs: 1. Alice transfers 5 tokens to Charlie; 2. Bob transfers 5 tokens to Alice;
        // If the second leg gets executed before the first leg, Alice will momentarily hold 15% of the asset and hence the settlement will fail compliance.
        instruction_legs.sort_by_key(|leg_id_leg| leg_id_leg.0);

        // Verifies that the venue still has the required permissions for the tokens involved.
        let mut tickers: BTreeSet<Ticker> = BTreeSet::new();
        for (_, leg) in &instruction_legs {
            // Each ticker is only checked once
            if tickers.insert(leg.asset)
                && Self::venue_filtering(leg.asset)
                && !Self::venue_allow_list(leg.asset, details.venue_id)
            {
                Self::deposit_event(RawEvent::VenueUnauthorized(
                    SettlementDID.as_id(),
                    leg.asset,
                    details.venue_id,
                ));
                return Err(Error::<T>::UnauthorizedVenue.into());
            }
        }

        match frame_storage_with_transaction(|| {
            Self::release_asset_locks_and_transfer_pending_legs(instruction_id, &instruction_legs)
        })? {
            Ok(_) => {
                Self::deposit_event(RawEvent::InstructionExecuted(
                    SettlementDID.as_id(),
                    instruction_id,
                ));
            }
            Err(leg_id) => {
                Self::deposit_event(RawEvent::LegFailedExecution(
                    SettlementDID.as_id(),
                    instruction_id,
                    leg_id,
                ));
                Self::deposit_event(RawEvent::InstructionFailed(
                    SettlementDID.as_id(),
                    instruction_id,
                ));
                // Unclaim receipts for the failed transaction so that they can be reused
                Self::unsafe_unclaim_receipts(instruction_id, &instruction_legs);
                return Err(Error::<T>::InstructionFailed.into());
            }
        }

        Ok(instruction_legs.len().try_into().unwrap_or_default())
    }

    fn release_asset_locks_and_transfer_pending_legs(
        instruction_id: InstructionId,
        instruction_legs: &[(LegId, Leg)],
    ) -> TransactionOutcome<Result<Result<(), LegId>, DispatchError>> {
        Self::unchecked_release_locks(instruction_id, instruction_legs);
        for (leg_id, leg) in instruction_legs {
            if Self::instruction_leg_status(instruction_id, leg_id) == LegStatus::ExecutionPending {
                if <Asset<T>>::base_transfer(leg.from, leg.to, &leg.asset, leg.amount).is_err() {
                    return TransactionOutcome::Rollback(Ok(Err(*leg_id)));
                }
            }
        }
        TransactionOutcome::Commit(Ok(Ok(())))
    }

    fn prune_instruction(id: InstructionId) {
        let legs = InstructionLegs::drain_prefix(id).collect::<Vec<_>>();
        let details = <InstructionDetails<T>>::take(id);
        VenueInstructions::remove(details.venue_id, id);
        <InstructionLegStatus<T>>::remove_prefix(id, None);
        InstructionAffirmsPending::remove(id);
        AffirmsReceived::remove_prefix(id, None);

        // We remove duplicates in memory before triggering storage actions
        let mut counter_parties = BTreeSet::new();
        for (_, leg) in &legs {
            counter_parties.insert(leg.from);
            counter_parties.insert(leg.to);
        }
        for counter_party in counter_parties {
            UserAffirmations::remove(counter_party, id);
        }
    }

    pub fn unsafe_affirm_instruction(
        did: IdentityId,
        id: InstructionId,
        portfolios: BTreeSet<PortfolioId>,
        max_legs_count: u32,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
    ) -> Result<u32, DispatchError> {
        // Checks portfolio's custodian and if it is a counter party with a pending affirmation.
        Self::ensure_portfolios_and_affirmation_status(
            id,
            &portfolios,
            did,
            secondary_key,
            &[AffirmationStatus::Pending],
        )?;

        let (total_leg_count, filtered_legs) =
            Self::filtered_legs(id, &portfolios, max_legs_count)?;
        with_transaction(|| {
            for (leg_id, leg_details) in filtered_legs {
                if let Err(_) = Self::lock_via_leg(&leg_details) {
                    // rustc fails to infer return type of `with_transaction` if you use ?/map_err here
                    return Err(DispatchError::from(Error::<T>::FailedToLockTokens));
                }
                <InstructionLegStatus<T>>::insert(id, leg_id, LegStatus::ExecutionPending);
            }
            Ok(())
        })?;

        let affirms_pending = Self::instruction_affirms_pending(id);

        // Updates storage
        for portfolio in &portfolios {
            UserAffirmations::insert(portfolio, id, AffirmationStatus::Affirmed);
            AffirmsReceived::insert(id, portfolio, AffirmationStatus::Affirmed);
            Self::deposit_event(RawEvent::InstructionAffirmed(did, *portfolio, id));
        }
        InstructionAffirmsPending::insert(
            id,
            affirms_pending.saturating_sub(u64::try_from(portfolios.len()).unwrap_or_default()),
        );

        Ok(total_leg_count)
    }

    // Unclaims all receipts for an instruction
    // Should only be used if user is unclaiming, or instruction has failed
    fn unsafe_unclaim_receipts(id: InstructionId, legs: &[(LegId, Leg)]) {
        for (leg_id, _) in legs {
            match Self::instruction_leg_status(id, leg_id) {
                LegStatus::ExecutionToBeSkipped(signer, receipt_uid) => {
                    <ReceiptsUsed<T>>::insert(&signer, receipt_uid, false);
                    Self::deposit_event(RawEvent::ReceiptUnclaimed(
                        SettlementDID.as_id(),
                        id,
                        *leg_id,
                        receipt_uid,
                        signer,
                    ));
                }
                LegStatus::PendingTokenLock | LegStatus::ExecutionPending => {}
            }
        }
    }

    fn unchecked_release_locks(id: InstructionId, instruction_legs: &[(LegId, Leg)]) {
        for (leg_id, leg) in instruction_legs {
            match Self::instruction_leg_status(id, leg_id) {
                LegStatus::ExecutionPending => {
                    // This can never return an error since the settlement module
                    // must've locked these tokens when instruction was affirmed
                    let _ = Self::unlock_via_leg(&leg);
                }
                LegStatus::ExecutionToBeSkipped(_, _) | LegStatus::PendingTokenLock => {}
            }
        }
    }

    /// Schedule a given instruction to be executed on the next block only if the
    /// settlement type is `SettleOnAffirmation` and no. of affirms pending is 0.
    fn maybe_schedule_instruction(affirms_pending: u64, id: InstructionId, legs_count: u32) {
        if affirms_pending == 0
            && Self::instruction_details(id).settlement_type == SettlementType::SettleOnAffirmation
        {
            // Schedule instruction to be executed in the next block.
            let execution_at = System::<T>::block_number() + One::one();
            Self::schedule_instruction(id, execution_at, legs_count);
        }
    }

    /// Schedule execution of given instruction at given block number.
    ///
    /// NB - It is expected to execute the given instruction into the given block number but
    /// it is not a guaranteed behavior, Scheduler may have other high priority task scheduled
    /// for the given block so there are chances where the instruction execution block no. may drift.
    fn schedule_instruction(id: InstructionId, execution_at: T::BlockNumber, _legs_count: u32) {
        let call = Call::<T>::execute_scheduled_instruction { id, _legs_count }.into();
        if let Err(_) = T::Scheduler::schedule_named(
            id.execution_name(),
            DispatchTime::At(execution_at),
            None,
            SETTLEMENT_INSTRUCTION_EXECUTION_PRIORITY,
            RawOrigin::Root.into(),
            call,
        ) {
            Self::deposit_event(RawEvent::SchedulingFailed(
                Error::<T>::FailedToSchedule.into(),
            ));
        }
    }

    pub fn base_affirm_with_receipts(
        origin: <T as frame_system::Config>::Origin,
        id: InstructionId,
        receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: Vec<PortfolioId>,
        max_legs_count: u32,
    ) -> Result<u32, DispatchError> {
        let (did, secondary_key, instruction_details) =
            Self::ensure_origin_perm_and_instruction_validity(origin, id, false)?;
        let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();

        // Verify that the receipts provided are unique
        let receipt_ids = receipt_details
            .iter()
            .map(|receipt| (receipt.signer.clone(), receipt.receipt_uid))
            .collect::<BTreeSet<_>>();

        ensure!(
            receipt_ids.len() == receipt_details.len(),
            Error::<T>::ReceiptAlreadyClaimed
        );

        // Verify portfolio custodianship and check if it is a counter party with a pending affirmation.
        Self::ensure_portfolios_and_affirmation_status(
            id,
            &portfolios_set,
            did,
            secondary_key.as_ref(),
            &[AffirmationStatus::Pending],
        )?;

        // Verify that the receipts are valid
        for receipt in &receipt_details {
            ensure!(
                Self::venue_signers(&instruction_details.venue_id, &receipt.signer),
                Error::<T>::UnauthorizedSigner
            );
            ensure!(
                !Self::receipts_used(&receipt.signer, &receipt.receipt_uid),
                Error::<T>::ReceiptAlreadyClaimed
            );

            let leg = Self::instruction_legs(&id, &receipt.leg_id);
            ensure!(
                portfolios_set.contains(&leg.from),
                Error::<T>::PortfolioMismatch
            );

            ensure!(
                !pallet_asset::Tokens::contains_key(&leg.asset),
                Error::<T>::UnauthorizedVenue
            );

            let msg = Receipt {
                receipt_uid: receipt.receipt_uid,
                from: leg.from,
                to: leg.to,
                asset: leg.asset,
                amount: leg.amount,
            };
            ensure!(
                receipt.signature.verify(&msg.encode()[..], &receipt.signer),
                Error::<T>::InvalidSignature
            );
        }

        let (total_leg_count, filtered_legs) =
            Self::filtered_legs(id, &portfolios_set, max_legs_count)?;
        // Lock tokens that do not have a receipt attached to their leg.
        with_transaction(|| {
            for (leg_id, leg_details) in filtered_legs {
                // Receipt for the leg was provided
                if let Some(receipt) = receipt_details
                    .iter()
                    .find(|receipt| receipt.leg_id == leg_id)
                {
                    <InstructionLegStatus<T>>::insert(
                        id,
                        leg_id,
                        LegStatus::ExecutionToBeSkipped(
                            receipt.signer.clone(),
                            receipt.receipt_uid,
                        ),
                    );
                } else if let Err(_) = Self::lock_via_leg(&leg_details) {
                    // rustc fails to infer return type of `with_transaction` if you use ?/map_err here
                    return Err(DispatchError::from(Error::<T>::FailedToLockTokens));
                } else {
                    <InstructionLegStatus<T>>::insert(id, leg_id, LegStatus::ExecutionPending);
                }
            }
            Ok(())
        })?;

        // Update storage
        let affirms_pending = Self::instruction_affirms_pending(id)
            .saturating_sub(u64::try_from(portfolios_set.len()).unwrap_or_default());

        // Mark receipts used in affirmation as claimed
        for receipt in &receipt_details {
            <ReceiptsUsed<T>>::insert(&receipt.signer, receipt.receipt_uid, true);
            Self::deposit_event(RawEvent::ReceiptClaimed(
                did,
                id,
                receipt.leg_id,
                receipt.receipt_uid,
                receipt.signer.clone(),
                receipt.metadata.clone(),
            ));
        }

        for portfolio in portfolios_set {
            UserAffirmations::insert(portfolio, id, AffirmationStatus::Affirmed);
            AffirmsReceived::insert(id, portfolio, AffirmationStatus::Affirmed);
            Self::deposit_event(RawEvent::InstructionAffirmed(did, portfolio, id));
        }

        InstructionAffirmsPending::insert(id, affirms_pending);
        Ok(total_leg_count)
    }

    pub fn base_affirm_instruction(
        origin: <T as frame_system::Config>::Origin,
        id: InstructionId,
        portfolios: impl Iterator<Item = PortfolioId>,
        max_legs_count: u32,
    ) -> Result<u32, DispatchError> {
        let (did, sk, _) = Self::ensure_origin_perm_and_instruction_validity(origin, id, false)?;
        let portfolios_set = portfolios.collect::<BTreeSet<_>>();

        // Provide affirmation to the instruction
        Self::unsafe_affirm_instruction(did, id, portfolios_set, max_legs_count, sk.as_ref())
    }

    // It affirms the instruction and may schedule the instruction
    // depends on the settlement type.
    pub fn affirm_with_receipts_and_maybe_schedule_instruction(
        origin: <T as frame_system::Config>::Origin,
        id: InstructionId,
        receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: Vec<PortfolioId>,
        max_legs_count: u32,
    ) -> DispatchResult {
        let legs_count = Self::base_affirm_with_receipts(
            origin,
            id,
            receipt_details,
            portfolios,
            max_legs_count,
        )?;
        // Schedule instruction to be execute in the next block (expected) if conditions are met.
        Self::maybe_schedule_instruction(Self::instruction_affirms_pending(id), id, legs_count);
        Ok(())
    }

    /// Schedule settlement instruction execution in the next block, unless already scheduled.
    /// Used for general purpose settlement.
    pub fn affirm_and_maybe_schedule_instruction(
        origin: <T as frame_system::Config>::Origin,
        id: InstructionId,
        portfolios: impl Iterator<Item = PortfolioId>,
        max_legs_count: u32,
    ) -> DispatchResult {
        let legs_count = Self::base_affirm_instruction(origin, id, portfolios, max_legs_count)?;
        // Schedule the instruction if conditions are met
        Self::maybe_schedule_instruction(Self::instruction_affirms_pending(id), id, legs_count);
        Ok(())
    }

    /// Affirm with or without receipts, executing the instruction when all affirmations have been received.
    ///
    /// NB - Use this function only in the STO pallet to support DVP settlements.
    pub fn affirm_and_execute_instruction(
        origin: <T as frame_system::Config>::Origin,
        id: InstructionId,
        receipt: Option<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: Vec<PortfolioId>,
        max_legs_count: u32,
    ) -> DispatchResult {
        match receipt {
            Some(receipt) => Self::base_affirm_with_receipts(
                origin,
                id,
                vec![receipt],
                portfolios,
                max_legs_count,
            )?,
            None => {
                Self::base_affirm_instruction(origin, id, portfolios.into_iter(), max_legs_count)?
            }
        };
        Self::execute_settle_on_affirmation_instruction(
            id,
            Self::instruction_affirms_pending(id),
            Self::instruction_details(id).settlement_type,
        )?;
        Self::prune_instruction(id);
        Ok(())
    }

    fn execute_settle_on_affirmation_instruction(
        id: InstructionId,
        affirms_pending: u64,
        settlement_type: SettlementType<T::BlockNumber>,
    ) -> DispatchResult {
        // We assume `settlement_type == SettleOnAffirmation`,
        // to be defensive, however, this is checked before instruction execution.
        if settlement_type == SettlementType::SettleOnAffirmation && affirms_pending == 0 {
            // We use execute_instruction here directly
            // and not the execute_instruction_retryable variant
            // because direct settlement is not retryable.
            Self::execute_instruction(id)?;
        }
        Ok(())
    }

    fn ensure_portfolios_and_affirmation_status(
        id: InstructionId,
        portfolios: &BTreeSet<PortfolioId>,
        custodian: IdentityId,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        expected_statuses: &[AffirmationStatus],
    ) -> DispatchResult {
        for portfolio in portfolios {
            T::Portfolio::ensure_portfolio_custody_and_permission(
                *portfolio,
                custodian,
                secondary_key,
            )?;
            let user_affirmation = Self::user_affirmations(portfolio, id);
            ensure!(
                expected_statuses.contains(&user_affirmation),
                Error::<T>::UnexpectedAffirmationStatus
            );
        }
        Ok(())
    }

    /// Returns total number of legs of an `instruction_id` and vector of legs where sender is in the `portfolios` set.
    /// Also, ensures that the number of filtered legs is under the limit.
    fn filtered_legs(
        id: InstructionId,
        portfolios: &BTreeSet<PortfolioId>,
        max_filtered_legs: u32,
    ) -> Result<(u32, Vec<(LegId, Leg)>), DispatchError> {
        let mut legs_count = 0;
        let filtered_legs = InstructionLegs::iter_prefix(id)
            .into_iter()
            .inspect(|_| legs_count += 1)
            .filter(|(_, leg_details)| portfolios.contains(&leg_details.from))
            .collect::<Vec<_>>();
        // Ensure leg count is under the limit
        ensure!(
            filtered_legs.len() <= max_filtered_legs as usize,
            Error::<T>::LegCountTooSmall
        );
        Ok((legs_count, filtered_legs))
    }

    fn base_update_venue_signers(
        did: IdentityId,
        id: VenueId,
        signers: Vec<T::AccountId>,
        add_signers: bool,
    ) -> DispatchResult {
        // Ensure venue exists & sender is its creator.
        Self::venue_for_management(id, did)?;

        if add_signers {
            for signer in &signers {
                ensure!(
                    !Self::venue_signers(&id, &signer),
                    Error::<T>::SignerAlreadyExists
                );
            }
            for signer in &signers {
                <VenueSigners<T>>::insert(&id, &signer, true);
            }
        } else {
            for signer in &signers {
                ensure!(
                    Self::venue_signers(&id, &signer),
                    Error::<T>::SignerDoesNotExist
                );
            }
            for signer in &signers {
                <VenueSigners<T>>::remove(&id, &signer);
            }
        }

        Self::deposit_event(RawEvent::VenueSignersUpdated(did, id, signers, add_signers));
        Ok(())
    }
}
