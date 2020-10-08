// Copyright (c) 2020 Polymath

//! # Settlement Module
//!
//! Settlement module manages all kinds of transfers and settlements of assets
//!
//! ## Overview
//!
//! The settlement module provides functionality to settle onchain as well as offchain trades between multiple parties.
//! All trades are settled under venues. A token issuer can allow/block certain venues from settling trades that involve their tokens.
//! An atomic settlement is called an Instruction. An instruction can contain multiple legs. Legs are essentially simple one to one transfers.
//! When an instruction is settled, either all legs are executed successfully or none are. In other words, if one of the leg fails due to
//! compliance failure, all other legs will also fail.
//!
//! An instruction must be authorized by all the counter parties involved for it to be executed.
//! An instruction can be set to automatically execute when all authorizations are received or at a particular block number.
//!
//! Offchain settlements are represented via receipts. If a leg has a receipt attached to it, it will not be executed onchain.
//! All other legs will be executed onchain during settlement.
//!
//! ## Dispatchable Functions
//!
//! - `create_venue` - Registers a new venue.
//! - `add_instruction` - Adds a new instruction.
//! - `authorize_instruction` - Authorizes an existing instruction.
//! - `unauthorize_instruction` - Unauthorizes an existing instruction.
//! - `reject_instruction` - Rejects an existing instruction.
//! - `claim_receipt` - Claims a signed receipt.
//! - `unclaim_receipt` - Unclaims a previously claimed receipt.
//! - `set_venue_filtering` - Enables or disabled venue filtering for a token.
//! - `allow_venues` - Allows additional venues to create instructions involving an asset.
//! - `disallow_venues` - Revokes permission given to venues for creating instructions involving a particular asset.
//!
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use codec::{Decode, Encode};
use cryptography::mercat::{
    transaction::TransactionValidator, EncryptedAmount, JustifiedTransferTx, PubAccount,
    TransferTransactionVerifier,
};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
    ensure, storage,
    traits::Get,
    weights::Weight,
    IterableStorageDoubleMap, StorageHasher, Twox128,
};
use frame_system::{self as system, ensure_signed};
use pallet_confidential_asset::{self as confidential_asset, MercatAccountId};
use pallet_identity as identity;
use polymesh_common_utilities::{
    traits::{
        asset::{Trait as AssetTrait, GAS_LIMIT},
        identity::Trait as IdentityTrait,
        portfolio::PortfolioSubTrait,
        CommonTrait,
    },
    with_transaction, Context,
    SystematicIssuers::Settlement as SettlementDID,
};
use polymesh_primitives::{IdentityId, PortfolioId, Ticker};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_runtime::traits::{Verify, Zero};
use sp_std::{collections::btree_set::BTreeSet, convert::TryFrom, prelude::*};

pub mod rng;

type Identity<T> = identity::Module<T>;
type ConfidentialAsset<T> = confidential_asset::Module<T>;

pub trait Trait:
    frame_system::Trait
    + CommonTrait
    + IdentityTrait
    + pallet_timestamp::Trait
    + confidential_asset::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Asset module.
    type Asset: AssetTrait<Self::Balance, Self::AccountId>;
    /// The maximum number of total legs in scheduled instructions that can be executed in a single block.
    /// Any excess instructions are scheduled in later blocks.
    type MaxScheduledInstructionLegsPerBlock: Get<u32>;
    /// The maximum number of total legs allowed for a instruction can have.
    type MaxLegsInAnInstruction: Get<u32>;
}

/// The wrapper for confidential transfer data which includes proofs and other related info from mercat library.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum MercatTxData {
    // The buyer (i.e., the sender), will submit this.
    InitializedTransfer(Vec<u8>),
    // The seller (i.e., the receiver) will submit this.
    FinalizedTransfer(Vec<u8>),
    // The exchange (i.e., the mediator) will submit this.
    JustifiedTransfer(Vec<u8>),
}

/// A wrapper for VenueDetails.
#[derive(
    Decode, Encode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct VenueDetails(Vec<u8>);

/// Status of an instruction.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstructionStatus {
    /// Invalid instruction or details pruned.
    Unknown,
    /// Instruction is pending execution.
    Pending,
}

impl Default for InstructionStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Type of the venue. Used for offchain filtering.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VenueType {
    /// Default type - used for mixed and unknown types.
    Other,
    /// Represents a primary distribution.
    Distribution,
    /// Represents an offering/fund raiser.
    Sto,
    /// Represents a match making service.
    Exchange,
}

impl Default for VenueType {
    fn default() -> Self {
        Self::Other
    }
}

/// Status of a leg.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LegStatus<AccountId> {
    /// It is waiting for authorization.
    PendingTokenLock,
    /// It is waiting execution (tokens currently locked).
    ExecutionPending,
    /// receipt used, (receipt signer, receipt uid).
    ExecutionToBeSkipped(AccountId, u64),
}

impl<AccountId> Default for LegStatus<AccountId> {
    fn default() -> Self {
        Self::PendingTokenLock
    }
}

/// Status of a authorization.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorizationStatus {
    /// Invalid authorization.
    Unknown,
    /// Pending user's consent.
    Pending,
    /// Authorized by the user.
    Authorized,
    /// Rejected by the user.
    Rejected,
}

impl Default for AuthorizationStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Type of settlement.
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettlementType<BlockNumber> {
    /// Instruction should be settled as soon as all authorizations are received.
    SettleOnAuthorization,
    /// Instruction should be settled on a particular block.
    SettleOnBlock(BlockNumber),
}

impl<BlockNumber> Default for SettlementType<BlockNumber> {
    fn default() -> Self {
        Self::SettleOnAuthorization
    }
}

/// Details about an instruction.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Instruction<Moment, BlockNumber> {
    /// Unique instruction id. It is an auto incrementing number.
    pub instruction_id: u64,
    /// Id of the venue this instruction belongs to.
    pub venue_id: u64,
    /// Status of the instruction.
    pub status: InstructionStatus,
    /// Type of settlement used for this instruction.
    pub settlement_type: SettlementType<BlockNumber>,
    /// Date at which this instruction was created.
    pub created_at: Option<Moment>,
    /// Date from which this instruction is valid.
    pub valid_from: Option<Moment>,
}

/// Details of a leg including the leg id in the instruction.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Leg<Balance> {
    NonConfidentialLeg(NonConfidentialLeg<Balance>),
    ConfidentialLeg(ConfidentialLeg),
    Undefined,
}

impl<Balance> Default for Leg<Balance> {
    fn default() -> Self {
        Leg::Undefined
    }
}

/// Details of a leg including the leg id in the instruction
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct NonConfidentialLeg<Balance> {
    /// Portfolio of the sender.
    pub from: PortfolioId,
    /// Portfolio of the receiver.
    pub to: PortfolioId,
    /// Ticker of the asset being transferred.
    pub asset: Ticker,
    /// Amount being transferred.
    pub amount: Balance,
}

/// Details of a confidential leg including the leg id in the instruction.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct ConfidentialLeg {
    /// Portfolio of the sender.
    pub from: PortfolioId,
    /// Portfolio of the receiver.
    pub to: PortfolioId,
    /// Portfolio of the mediator.
    pub mediator: PortfolioId,
    /// Mercat account id of the sender.
    pub from_account_id: MercatAccountId,
    /// Mercat account id of the receiver.
    pub to_account_id: MercatAccountId,
}

/// Details about a venue.
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Venue {
    /// Identity of the venue's creator.
    pub creator: IdentityId,
    /// instructions under this venue (Only needed for the UI).
    pub instructions: Vec<u64>,
    /// Additional details about this venue (Only needed for the UI).
    pub details: VenueDetails,
    /// Specifies type of the venue (Only needed for the UI).
    pub venue_type: VenueType,
}

/// Old venue details format. Used only for storage migration.
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct OldVenue {
    /// Identity of the venue's creator
    pub creator: IdentityId,
    /// Instructions under this venue (Only needed for the UI).
    pub instructions: Vec<u64>,
    /// Additional details about this venue (Only needed for the UI).
    pub details: VenueDetails,
}

impl Venue {
    pub fn new(creator: IdentityId, details: VenueDetails, venue_type: VenueType) -> Self {
        Self {
            creator,
            instructions: Vec::new(),
            details,
            venue_type,
        }
    }
}

/// Details about an offchain transaction receipt.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Receipt<Balance> {
    /// Unique receipt number set by the signer for their receipts.
    pub receipt_uid: u64,
    /// Identity of the sender.
    pub from: PortfolioId,
    /// Identity of the receiver.
    pub to: PortfolioId,
    /// Ticker of the asset being transferred.
    pub asset: Ticker,
    /// Amount being transferred.
    pub amount: Balance,
}

/// Details about an offchain transaction receipt that a user must input.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct ReceiptDetails<AccountId, OffChainSignature> {
    /// Unique receipt number set by the signer for their receipts.
    pub receipt_uid: u64,
    /// Target leg id.
    pub leg_id: u64,
    /// Signer for this receipt.
    pub signer: AccountId,
    /// Signature confirming the receipt details.
    pub signature: OffChainSignature,
}

pub mod weight_for {
    use super::*;

    pub fn weight_for_execute_instruction_if_auth_no_pending<T: Trait>(
        weight_for_custodian_transfer: Weight,
    ) -> Weight {
        T::DbWeight::get()
            .reads(4) // Weight for read.
            .saturating_add(150_000_000) // General weight.
            .saturating_add(weight_for_custodian_transfer) // Weight for custodian transfer.
    }

    pub fn weight_for_execute_instruction_if_auth_pending<T: Trait>() -> Weight {
        T::DbWeight::get()
            .reads_writes(2, 1) // For read and write.
            .saturating_add(900_000_000) // Mocking unchecked_release_locks() function weight.
    }

    pub fn weight_for_authorize_with_receipts<T: Trait>(no_of_receipts: u32) -> Weight {
        T::DbWeight::get()
            .reads_writes(6, 3) // Weight for read and write.
            .saturating_add((no_of_receipts * 80_000_000).into()) // Weight for receipts.
            .saturating_add(
                T::DbWeight::get()
                    .reads_writes(3, 1)
                    .saturating_mul(no_of_receipts.into()),
            ) // Weight for read and write related to receipts.
    }

    pub fn weight_for_authorize_instruction<T: Trait>() -> Weight {
        T::DbWeight::get()
            .reads_writes(5, 3) // Weight for read and writes.
            .saturating_add(600_000_000)
    }

    pub fn weight_for_reject_instruction<T: Trait>() -> Weight {
        T::DbWeight::get()
            .reads_writes(3, 2) // Weight for read and writes.
            .saturating_add(500_000_000) // Lump-sum weight for `unsafe_unauthorize_instruction()`.
    }

    pub fn weight_for_transfer<T: Trait>() -> Weight {
        GAS_LIMIT
            .saturating_mul(
                (T::Asset::max_number_of_tm_extension() * T::MaxLegsInAnInstruction::get()).into(),
            )
            .saturating_add(70_000_000) // Weight for compliance manager.
            .saturating_add(T::DbWeight::get().reads_writes(4, 5)) // Weight for read.
            .saturating_add(150_000_000)
    }

    pub fn weight_for_instruction_creation<T: Trait>(no_of_legs: usize) -> Weight {
        T::DbWeight::get()
            .reads_writes(2, 5)
            .saturating_add(u64::try_from(no_of_legs * 50_000_000).unwrap_or_default())
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
        Moment = <T as pallet_timestamp::Trait>::Moment,
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// A new venue has been created (did, venue_id, details, type).
        VenueCreated(IdentityId, u64, VenueDetails, VenueType),
        /// An existing venue has been updated (did, venue_id, details, type).
        VenueUpdated(IdentityId, u64, VenueDetails, VenueType),
        /// A new instruction has been created.
        /// (did, venue_id, instruction_id, settlement_type, valid_from, legs).
        InstructionCreated(
            IdentityId,
            u64,
            u64,
            SettlementType<BlockNumber>,
            Option<Moment>,
            Vec<Leg<Balance>>,
        ),
        /// An instruction has been authorized (did, portfolio, instruction_id).
        InstructionAuthorized(IdentityId, PortfolioId, u64),
        /// An instruction has been unauthorized (did, portfolio, instruction_id).
        InstructionUnauthorized(IdentityId, PortfolioId, u64),
        /// An instruction has been rejected (did, instruction_id).
        InstructionRejected(IdentityId, u64),
        /// A receipt has been claimed (did, instruction_id, leg_id, receipt_uid, signer).
        ReceiptClaimed(IdentityId, u64, u64, u64, AccountId),
        /// A receipt has been unclaimed (did, instruction_id, leg_id, receipt_uid, signer).
        ReceiptUnclaimed(IdentityId, u64, u64, u64, AccountId),
        /// Venue filtering has been enabled or disabled for a ticker (did, ticker, filtering_enabled).
        VenueFiltering(IdentityId, Ticker, bool),
        /// Venues added to allow list (did, ticker, vec<venue_id>).
        VenuesAllowed(IdentityId, Ticker, Vec<u64>),
        /// Venues added to block list (did, ticker, vec<venue_id>).
        VenuesBlocked(IdentityId, Ticker, Vec<u64>),
        /// Execution of a leg failed (did, instruction_id, leg_id).
        LegFailedExecution(IdentityId, u64, u64),
        /// Instruction failed execution (did, instruction_id).
        InstructionFailed(IdentityId, u64),
        /// Instruction executed successfully(did, instruction_id).
        InstructionExecuted(IdentityId, u64),
        /// Venue unauthorized by ticker owner (did, Ticker, venue_id).
        VenueUnauthorized(IdentityId, Ticker, u64),
    }
);

decl_error! {
    /// Errors for the Settlement module.
    pub enum Error for Module<T: Trait> {
        /// Certain transfer modes are not yet supported in confidential modes.
        ConfidentialModeNotSupportedYet,
        /// Transaction proof failed to verify.
        InvalidMercatTransferProof,
        /// Undefined leg type.
        UndefinedLeg,
        /// Confidential legs do not have asset and amounts.
        ConfidentialLegDoesNotHaveAssetAndAmount,
        /// Venue does not exist.
        InvalidVenue,
        /// Sender does not have required permissions.
        Unauthorized,
        /// No pending authorization for the provided instruction.
        NoPendingAuth,
        /// Instruction has not been authorized.
        InstructionNotAuthorized,
        /// Provided instruction is not pending execution.
        InstructionNotPending,
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
        /// While authorizing the transfer, system failed to lock the assets involved.
        FailedToLockTokens,
        /// Instruction validity has not started yet.
        InstructionWaitingValidity,
        /// Instruction's target settle block reached.
        InstructionSettleBlockPassed,
        /// Instruction waiting for settle block.
        InstructionWaitingSettleBlock,
        /// Offchain signature is invalid.
        InvalidSignature,
        /// Sender and receiver are the same.
        SameSenderReceiver,
        /// Maximum numbers of legs in a instruction > `MaxLegsInAnInstruction`.
        LegsCountExceededMaxLimit,
        /// Portfolio in receipt does not match with portfolios provided by the user.
        PortfolioMismatch
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as StoCapped {
        /// Info about a venue. venue_id -> venue_details.
        VenueInfo get(fn venue_info): map hasher(twox_64_concat) u64 => Venue;
        /// Signers authorized by the venue. (venue_id, signer) -> authorized_bool.
        VenueSigners get(fn venue_signers): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) T::AccountId => bool;
        /// Array of venues created by an identity. Only needed for the UI. IdentityId -> Vec<venue_id>.
        UserVenues get(fn user_venues): map hasher(twox_64_concat) IdentityId => Vec<u64>;
        /// Details about an instruction. instruction_id -> instruction_details.
        InstructionDetails get(fn instruction_details): map hasher(twox_64_concat) u64 => Instruction<T::Moment, T::BlockNumber>;
        /// Legs under an instruction. (instruction_id, leg_id) -> Leg.
        InstructionLegs get(fn instruction_legs): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) u64 => Leg<T::Balance>;
        /// Status of a leg under an instruction. (instruction_id, leg_id) -> LegStatus.
        InstructionLegStatus get(fn instruction_leg_status): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) u64 => LegStatus<T::AccountId>;
        /// Number of authorizations pending before instruction is executed. instruction_id -> auths_pending.
        InstructionAuthsPending get(fn instruction_auths_pending): map hasher(twox_64_concat) u64 => u64;
        /// Tracks authorizations received for an instruction. (instruction_id, counter_party) -> AuthorizationStatus.
        AuthsReceived get(fn auths_received): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) PortfolioId => AuthorizationStatus;
        /// Helps a user track their pending instructions and authorizations (only needed for UI).
        /// (counter_party, instruction_id) -> AuthorizationStatus.
        UserAuths get(fn user_auths): double_map hasher(twox_64_concat) PortfolioId, hasher(twox_64_concat) u64 => AuthorizationStatus;
        /// Tracks redemption of receipts. (signer, receipt_uid) -> receipt_used.
        ReceiptsUsed get(fn receipts_used): double_map hasher(twox_64_concat) T::AccountId, hasher(blake2_128_concat) u64 => bool;
        /// Tracks if a token has enabled filtering venues that can create instructions involving their token. Ticker -> filtering_enabled.
        VenueFiltering get(fn venue_filtering): map hasher(blake2_128_concat) Ticker => bool;
        /// Venues that are allowed to create instructions involving a particular ticker. Oly used if filtering is enabled.
        /// (ticker, venue_id) -> allowed.
        VenueAllowList get(fn venue_allow_list): double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) u64 => bool;
        /// Number of venues in the system (It's one more than the actual number).
        VenueCounter get(fn venue_counter) build(|_| 1u64): u64;
        /// Number of instructions in the system (It's one more than the actual number).
        InstructionCounter get(fn instruction_counter) build(|_| 1u64): u64;
        /// The list of scheduled instructions with the block numbers in which those instructions.
        /// Become eligible to be executed. BlockNumber -> Vec<instruction_id>.
        ScheduledInstructions get(fn scheduled_instructions): map hasher(twox_64_concat) T::BlockNumber => Vec<u64>;
        /// The storage for mercat transaction proofs.
        /// The key is the instruction_id.
        MercatTxDataStorage get(fn mercat_tx_data): map hasher(twox_64_concat) u64 => Vec<MercatTxData>;
        //MercatTxDataStorage get(fn mercat_tx_data): double_map hasher(twox_64_concat) u64 => Vec<MercatTxData>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        const MaxScheduledInstructionLegsPerBlock: u32 = T::MaxScheduledInstructionLegsPerBlock::get();

        const MaxLegsInAnInstruction: u32 = T::MaxLegsInAnInstruction::get();

        fn on_runtime_upgrade() -> Weight {
            // Delete all settlement data.
            let prefix = Twox128::hash(b"Settlement");
            storage::unhashed::kill_prefix(&prefix);

            // Set venue counter and instruction counter to 1 so that the id(s) start from 1 instead of 0.
            <VenueCounter>::put(1);
            <InstructionCounter>::put(1);

            1_000
        }

        /// Registers a new venue.
        ///
        /// * `details` - Extra details about a venue.
        /// * `signers` - The signers that are allowed to sign receipts for this venue.
        /// * `venue_type` - Type of venue being created.
        ///
        /// # Weight
        /// `200_000_000 + 5_000_000 * signers.len()`
        #[weight = 200_000_000 + 5_000_000 * u64::try_from(signers.len()).unwrap_or_default()]
        pub fn create_venue(origin, details: VenueDetails, signers: Vec<T::AccountId>, venue_type: VenueType) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let venue = Venue::new(did, details, venue_type);
            // NB: Venue counter starts with 1.
            let venue_counter = Self::venue_counter();
            VenueInfo::insert(venue_counter, venue.clone());
            for signer in signers {
                <VenueSigners<T>>::insert(venue_counter, signer, true);
            }
            VenueCounter::put(venue_counter + 1);
            UserVenues::append(did, venue_counter);
            Self::deposit_event(RawEvent::VenueCreated(did, venue_counter, venue.details, venue.venue_type));
            Ok(())
        }

        /// Edit venue details and types.
        /// Both parameters are optional, they will be updated only if Some(value) is provided.
        ///
        /// * `venue_id` - ID of the venue to edit.
        /// * `details` - Extra details about a venue.
        /// * `type` - Type of venue being created.
        ///
        /// # Weight
        /// `200_000_000
        #[weight = 200_000_000]
        pub fn update_venue(origin, venue_id: u64, details: Option<VenueDetails>, venue_type: Option<VenueType>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            // Check if a venue exists and the sender is the creator of the venue.
            let mut venue = Self::venue_info(venue_id);
            ensure!(venue.creator == did, Error::<T>::Unauthorized);
            if let Some(venue_details) = details {
                venue.details = venue_details;
            }
            if let Some(v_type) = venue_type {
                venue.venue_type = v_type;
            }
            <VenueInfo>::insert(&venue_id, venue.clone());
            Self::deposit_event(RawEvent::VenueUpdated(did, venue_id, venue.details, venue.venue_type));
            Ok(())
        }

        /// Adds a new instruction.
        ///
        /// # Arguments
        /// * `venue_id` - ID of the venue this instruction belongs to.
        /// * `settlement_type` - Defines if the instruction should be settled.
        ///    immediately after receiving all auths or waiting till a specific block.
        /// * `valid_from` - Optional date from which people can interact with this instruction.
        /// * `legs` - Legs included in this instruction.
        ///
        /// # Weight
        /// `950_000_000 + 1_000_000 * legs.len()`
        #[weight = weight_for::weight_for_instruction_creation::<T>(legs.len())]
        pub fn add_instruction(
            origin,
            venue_id: u64,
            settlement_type: SettlementType<T::BlockNumber>,
            valid_from: Option<T::Moment>,
            legs: Vec<Leg<T::Balance>>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            Self::base_add_instruction(did, venue_id, settlement_type, valid_from, legs)?;

            Ok(())
        }

        /// Adds and authorizes a new instruction.
        ///
        /// # Arguments
        /// * `venue_id` - ID of the venue this instruction belongs to.
        /// * `settlement_type` - Defines if the instruction should be settled.
        ///    immediately after receiving all auths or waiting till a specific block.
        /// * `valid_from` - Optional date from which people can interact with this instruction.
        /// * `legs` - Legs included in this instruction.
        /// * `portfolios` - Portfolios that the sender controls and wants to authorize for this instruction.
        #[weight = weight_for::weight_for_instruction_creation::<T>(legs.len())
            + weight_for::weight_for_authorize_instruction::<T>()
            + weight_for::weight_for_transfer::<T>()
        ]
        pub fn add_and_authorize_instruction(
            origin,
            venue_id: u64,
            settlement_type: SettlementType<T::BlockNumber>,
            valid_from: Option<T::Moment>,
            legs: Vec<Leg<T::Balance>>,
            portfolios: Vec<PortfolioId>
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin.clone())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();
            let legs_count = legs.len();
            let instruction_id = Self::base_add_instruction(did, venue_id, settlement_type, valid_from, legs)?;
            let authorization_weight = Self::authorize_instruction(origin, instruction_id, portfolios_set.into_iter().collect::<Vec<_>>())?;
            Ok(
                Some(
                    weight_for::weight_for_instruction_creation::<T>(legs_count)
                        .saturating_add(authorization_weight.actual_weight.unwrap_or_default())
                ).into()
            )
        }

        /// Authorizes an existing instruction.
        ///
        /// # Arguments
        /// * `instruction_id` - Instruction id to authorize.
        /// * `portfolios` - Portfolios that the sender controls and wants to authorize for this instruction.
        #[weight = weight_for::weight_for_authorize_instruction::<T>()
            + weight_for::weight_for_transfer::<T>() // Maximum weight for `execute_instruction()`.
        ]
        pub fn authorize_instruction(origin, instruction_id: u64, portfolios: Vec<PortfolioId>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();

            // Authorize the instruction.
            Self::unsafe_authorize_instruction(did, instruction_id, portfolios_set)?;

            // Execute the instruction if conditions are met.
            let auths_pending = Self::instruction_auths_pending(instruction_id);
            let weight_for_instruction_execution = Self::is_instruction_executed(auths_pending, Self::instruction_details(instruction_id).settlement_type, instruction_id);

            Ok(Some(weight_for::weight_for_authorize_instruction::<T>() + weight_for_instruction_execution).into())
        }

        /// Authorizes an existing confidential instruction.
        ///
        /// # Arguments
        /// * `instruction_id` - Instruction id to authorize.
        /// * `portfolios` - Portfolios that the sender controls and wants to authorize for this instruction.
        #[weight = weight_for::weight_for_authorize_instruction::<T>()
            + weight_for::weight_for_transfer::<T>() // Maximum weight for `execute_instruction()`
        ]
        pub fn authorize_confidential_instruction(origin, instruction_id: u64, data: MercatTxData, portfolios: Vec<PortfolioId>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();

            // Authorize the instruction.
            Self::unsafe_authorize_instruction(did, instruction_id, portfolios_set)?;

            MercatTxDataStorage::append(instruction_id, data);
            // Execute the instruction if conditions are met.
            let auths_pending = Self::instruction_auths_pending(instruction_id);
            let weight_for_instruction_execution = Self::is_instruction_executed(auths_pending, Self::instruction_details(instruction_id).settlement_type, instruction_id);

            Ok(Some(weight_for::weight_for_authorize_instruction::<T>() + weight_for_instruction_execution).into())
        }

        /// Unauthorizes an existing instruction.
        ///
        /// # Arguments
        /// * `instruction_id` - Instruction id to unauthorize.
        /// * `portfolios` - Portfolios that the sender controls and wants to unauthorize for this instruction.
        #[weight = 25_000_000_000]
        pub fn unauthorize_instruction(origin, instruction_id: u64, portfolios: Vec<PortfolioId>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();

            Self::ensure_instruction_validity(instruction_id)?;

            // Unauthorize the instruction.
            Self::unsafe_unauthorize_instruction(did, instruction_id, portfolios_set)
        }

        /// Rejects an existing instruction.
        ///
        /// # Arguments
        /// * `instruction_id` - Instruction id to reject.
        /// * `portfolios` - Portfolios that the sender controls and wants them to reject this instruction.
        #[weight = weight_for::weight_for_reject_instruction::<T>()
            + weight_for::weight_for_transfer::<T>() // Maximum weight for `execute_instruction()`.
        ]
        pub fn reject_instruction(origin, instruction_id: u64, portfolios: Vec<PortfolioId>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();

            Self::ensure_instruction_validity(instruction_id)?;

            with_transaction(|| {
                let mut portfolios_to_unauthorize = BTreeSet::new();
                for portfolio in portfolios_set {
                    // Unauthorize the instruction if it was authorized earlier.
                    let user_auth_status = Self::user_auths(portfolio, instruction_id);

                    match user_auth_status {
                        AuthorizationStatus::Authorized => { portfolios_to_unauthorize.insert(portfolio); },
                        AuthorizationStatus::Pending => T::Portfolio::ensure_portfolio_custody(portfolio, did)?,
                        _ => return Err(DispatchError::from(Error::<T>::NoPendingAuth))
                    };

                    // Updates storage.
                    <UserAuths>::insert(portfolio, instruction_id, AuthorizationStatus::Rejected);
                    <AuthsReceived>::insert(instruction_id, portfolio, AuthorizationStatus::Rejected);
                }
                Self::unsafe_unauthorize_instruction(did, instruction_id, portfolios_to_unauthorize)?;
                Ok(())
            })?;


            // Execute the instruction if it was meant to be executed on authorization.
            let weight_for_instruction_execution = Self::is_instruction_executed(Zero::zero(), Self::instruction_details(instruction_id).settlement_type, instruction_id);

            Self::deposit_event(RawEvent::InstructionRejected(did, instruction_id));
            Ok(Some(weight_for::weight_for_reject_instruction::<T>() + weight_for_instruction_execution).into())
        }

        /// Accepts an instruction and claims a signed receipt.
        ///
        /// # Arguments
        /// * `instruction_id` - Target instruction id.
        /// * `leg_id` - Target leg id for the receipt.
        /// * `receipt_uid` - Receipt ID generated by the signer.
        /// * `signer` - Signer of the receipt.
        /// * `signed_data` - Signed receipt.
        /// * `portfolios` - Portfolios that the sender controls and wants to accept this instruction with.
        #[weight = weight_for::weight_for_authorize_with_receipts::<T>(u32::try_from(receipt_details.len()).unwrap_or_default())
            + weight_for::weight_for_transfer::<T>() // Maximum weight for `execute_instruction()`.
            ]
        pub fn authorize_with_receipts(origin, instruction_id: u64, receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>, portfolios: Vec<PortfolioId>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();

            Self::ensure_instruction_validity(instruction_id)?;

            // Verify that the receipts provided are unique.
            let receipt_ids = receipt_details.iter().map(|receipt| (receipt.signer.clone(), receipt.receipt_uid)).collect::<BTreeSet<_>>();

            ensure!(
                receipt_ids.len() == receipt_details.len(),
                Error::<T>::ReceiptAlreadyClaimed
            );

            let instruction_details = Self::instruction_details(instruction_id);

            // verify portfolio custodianship and check if it is a counter party with a pending or rejected authorization.
            for portfolio in &portfolios_set {
                T::Portfolio::ensure_portfolio_custody(*portfolio, did)?;
                let user_auth = Self::user_auths(portfolio, instruction_id);
                ensure!(
                    user_auth == AuthorizationStatus::Pending || user_auth == AuthorizationStatus::Rejected,
                    Error::<T>::NoPendingAuth
                );
            }

            // Verify that the receipts are valid.
            for receipt in &receipt_details {
                ensure!(
                    Self::venue_signers(&instruction_details.venue_id, &receipt.signer), Error::<T>::UnauthorizedSigner
                );
                ensure!(
                    !Self::receipts_used(&receipt.signer, &receipt.receipt_uid), Error::<T>::ReceiptAlreadyClaimed
                );

                let leg = Self::instruction_legs(&instruction_id, &receipt.leg_id);
                let (from, to) = Self::leg_from_to(&leg)?;
                ensure!(portfolios_set.contains(&from), Error::<T>::PortfolioMismatch);
                let (asset, amount) = Self::leg_asset_and_amount(&leg)?;
                let msg = Receipt {
                    receipt_uid: receipt.receipt_uid,
                    from,
                    to,
                    asset,
                    amount,
                };

                ensure!(
                    receipt.signature.verify(&msg.encode()[..], &receipt.signer),
                    Error::<T>::InvalidSignature
                );
            }

            // Lock tokens that do not have a receipt attached to their leg.
            with_transaction(|| {
                let legs = <InstructionLegs<T>>::iter_prefix(instruction_id);
                for (leg_id, leg_details) in legs.filter(|(_leg_id, leg_details)| {
                    match Self::leg_from_to(&leg_details) {
                        Ok((from, _)) => portfolios_set.contains(&from),
                        Err(_) => false
                    }
                }) {
                    // Receipt for the leg was provided.
                    if let Some(receipt) = receipt_details.iter().find(|receipt| receipt.leg_id == leg_id) {
                        <InstructionLegStatus<T>>::insert(
                            instruction_id,
                            leg_id,
                            LegStatus::ExecutionToBeSkipped(receipt.signer.clone(), receipt.receipt_uid)
                        );
                    } else {
                        let (asset, amount) = Self::leg_asset_and_amount(&leg_details)?;
                        let (from, _) = Self::leg_from_to(&leg_details)?;
                        if T::Portfolio::lock_tokens(&from, &asset, &amount).is_err() {
                            // rustc fails to infer return type of `with_transaction` if you use ?/map_err here.
                            return Err(DispatchError::from(Error::<T>::FailedToLockTokens));
                        }
                        <InstructionLegStatus<T>>::insert(instruction_id, leg_id, LegStatus::ExecutionPending);

                    }
                }
                Ok(())
            })?;

            // Update storage.
            let auths_pending = Self::instruction_auths_pending(instruction_id).saturating_sub(u64::try_from(portfolios_set.len()).unwrap_or_default());
            for portfolio in portfolios_set {
                <UserAuths>::insert(portfolio, instruction_id, AuthorizationStatus::Authorized);
                <AuthsReceived>::insert(instruction_id, portfolio, AuthorizationStatus::Authorized);
                Self::deposit_event(RawEvent::InstructionAuthorized(did, portfolio, instruction_id));
            }

            <InstructionAuthsPending>::insert(instruction_id, auths_pending);
            for receipt in &receipt_details {
                <ReceiptsUsed<T>>::insert(&receipt.signer, receipt.receipt_uid, true);
                Self::deposit_event(RawEvent::ReceiptClaimed(did, instruction_id, receipt.leg_id, receipt.receipt_uid, receipt.signer.clone()));
            }

            // Execute instruction if conditions are met.
            let execute_instruction_weight = Self::is_instruction_executed(auths_pending, instruction_details.settlement_type, instruction_id);

            Ok(Some(weight_for::weight_for_authorize_with_receipts::<T>(u32::try_from(receipt_details.len()).unwrap_or_default()) + execute_instruction_weight).into())
        }

        /// Claims a signed receipt.
        ///
        /// # Arguments
        /// * `instruction_id` - Target instruction id for the receipt.
        /// * `leg_id` - Target leg id for the receipt.
        /// * `receipt_uid` - Receipt ID generated by the signer.
        /// * `signer` - Signer of the receipt.
        /// * `signed_data` - Signed receipt.
        #[weight = 10_000_000_000]
        pub fn claim_receipt(origin, instruction_id: u64, receipt_details: ReceiptDetails<T::AccountId, T::OffChainSignature>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let (from, _) = Self::leg_from_to(&Self::instruction_legs(&instruction_id, &receipt_details.leg_id))?;
            T::Portfolio::ensure_portfolio_custody(from, did)?;
            Self::unsafe_claim_receipt(
                did,
                instruction_id,
                receipt_details.leg_id,
                receipt_details.receipt_uid,
                receipt_details.signer,
                receipt_details.signature
            )
        }

        /// Unclaims a previously claimed receipt.
        ///
        /// # Arguments
        /// * `instruction_id` - Target instruction id for the receipt.
        /// * `leg_id` - Target leg id for the receipt.
        #[weight = 5_000_000_000]
        pub fn unclaim_receipt(origin, instruction_id: u64, leg_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;

            Self::ensure_instruction_validity(instruction_id)?;

            if let LegStatus::ExecutionToBeSkipped(signer, receipt_uid) = Self::instruction_leg_status(instruction_id, leg_id) {
                let leg = Self::instruction_legs(instruction_id, leg_id);
                let (from, _) = Self::leg_from_to(&leg)?;
                T::Portfolio::ensure_portfolio_custody(from, did)?;
                // Lock tokens that are part of the leg.
                let (asset, amount) = Self::leg_asset_and_amount(&leg)?;
                T::Portfolio::lock_tokens(&from, &asset, &amount)?;
                <ReceiptsUsed<T>>::insert(&signer, receipt_uid, false);
                <InstructionLegStatus<T>>::insert(instruction_id, leg_id, LegStatus::ExecutionPending);
                Self::deposit_event(RawEvent::ReceiptUnclaimed(did, instruction_id, leg_id, receipt_uid, signer));
                Ok(())
            } else {
                Err(Error::<T>::ReceiptNotClaimed.into())
            }
        }

        /// Enables or disabled venue filtering for a token.
        ///
        /// # Arguments
        /// * `ticker` - Ticker of the token in question.
        /// * `enabled` - Boolean that decides if the filtering should be enabled.
        #[weight = 200_000_000]
        pub fn set_venue_filtering(origin, ticker: Ticker, enabled: bool) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            if enabled {
                <VenueFiltering>::insert(ticker, enabled);
            } else {
                <VenueFiltering>::remove(ticker);
            }
            Self::deposit_event(RawEvent::VenueFiltering(did, ticker, enabled));
            Ok(())
        }

        /// Allows additional venues to create instructions involving an asset.
        ///
        /// * `ticker` - Ticker of the token in question.
        /// * `venues` - Array of venues that are allowed to create instructions for the token in question.
        ///
        /// # Weight
        /// `200_000_000 + 500_000 * venues.len()`
        #[weight = 200_000_000 + 500_000 * u64::try_from(venues.len()).unwrap_or_default()]
        pub fn allow_venues(origin, ticker: Ticker, venues: Vec<u64>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            for venue in &venues {
                <VenueAllowList>::insert(&ticker, venue, true);
            }
            Self::deposit_event(RawEvent::VenuesAllowed(did, ticker, venues));
            Ok(())
        }

        /// Revokes permission given to venues for creating instructions involving a particular asset.
        ///
        /// * `ticker` - Ticker of the token in question.
        /// * `venues` - Array of venues that are no longer allowed to create instructions for the token in question.
        ///
        /// # Weight
        /// `200_000_000 + 500_000 * venues.len()`
        #[weight = 200_000_000 + 500_000 * u64::try_from(venues.len()).unwrap_or_default()]
        pub fn disallow_venues(origin, ticker: Ticker, venues: Vec<u64>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            for venue in &venues {
                <VenueAllowList>::remove(&ticker, venue);
            }
            Self::deposit_event(RawEvent::VenuesBlocked(did, ticker, venues));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Returns true if `sender_did` is the owner of `ticker` asset.
    fn is_owner(ticker: &Ticker, sender_did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, sender_did)
    }

    fn leg_asset_and_amount(leg: &Leg<T::Balance>) -> Result<(Ticker, T::Balance), DispatchError> {
        match leg {
            Leg::NonConfidentialLeg(leg) => Ok((leg.asset, leg.amount)),
            Leg::ConfidentialLeg(_) => {
                Err(Error::<T>::ConfidentialLegDoesNotHaveAssetAndAmount.into())
            }
            Leg::Undefined => Err(Error::<T>::UndefinedLeg.into()),
        }
    }

    fn leg_from_to(leg: &Leg<T::Balance>) -> Result<(PortfolioId, PortfolioId), DispatchError> {
        match leg {
            Leg::ConfidentialLeg(leg) => Ok((leg.from, leg.to)),
            Leg::NonConfidentialLeg(leg) => Ok((leg.from, leg.to)),
            Leg::Undefined => Err(Error::<T>::UndefinedLeg.into()),
        }
    }

    pub fn base_add_instruction(
        did: IdentityId,
        venue_id: u64,
        settlement_type: SettlementType<T::BlockNumber>,
        valid_from: Option<T::Moment>,
        legs: Vec<Leg<T::Balance>>,
    ) -> Result<u64, DispatchError> {
        // Ensure number of legs are within the limit.
        ensure!(
            u32::try_from(legs.len()).unwrap_or_default() <= T::MaxLegsInAnInstruction::get(),
            Error::<T>::LegsCountExceededMaxLimit
        );
        // Check if a venue exists and the sender is the creator of the venue.
        let mut venue = Self::venue_info(venue_id);
        ensure!(venue.creator == did, Error::<T>::Unauthorized);

        // Prepare data to store in storage.
        // NB Instruction counter starts from 1.
        let instruction_counter = Self::instruction_counter();
        let mut counter_parties = BTreeSet::new();
        let mut tickers = BTreeSet::new();
        // This is done to create a list of unique CP and tickers involved in the instruction.
        for leg in &legs {
            let (from, to) = Self::leg_from_to(&leg)?;
            ensure!(from != to, Error::<T>::SameSenderReceiver);
            counter_parties.insert(from);
            counter_parties.insert(to);
            match leg {
                Leg::ConfidentialLeg(c_leg) => {
                    // Mediator is added as a counter_party, so that we can wait for its authorization.
                    counter_parties.insert(c_leg.mediator);
                }
                Leg::NonConfidentialLeg(nc_leg) => {
                    tickers.insert(nc_leg.asset);
                }
                Leg::Undefined => return Err(Error::<T>::UndefinedLeg.into()),
            }
        }

        // Check if the venue has required permissions from token owners.
        ensure!(
            tickers
                .iter()
                .filter(|t| Self::venue_filtering(*t))
                .all(|t| Self::venue_allow_list(t, venue_id)),
            Error::<T>::UnauthorizedVenue,
        );

        let instruction = Instruction {
            instruction_id: instruction_counter,
            venue_id,
            status: InstructionStatus::Pending,
            settlement_type,
            created_at: Some(<pallet_timestamp::Module<T>>::get()),
            valid_from,
        };

        // Write data to storage.
        for counter_party in &counter_parties {
            UserAuths::insert(
                counter_party,
                instruction_counter,
                AuthorizationStatus::Pending,
            );
        }

        for (leg, idx) in legs.iter().zip(0u64..) {
            <InstructionLegs<T>>::insert(instruction_counter, idx, leg.clone());
        }

        if let SettlementType::SettleOnBlock(block_number) = settlement_type {
            <ScheduledInstructions<T>>::append(block_number, instruction_counter);
        }

        <InstructionDetails<T>>::insert(instruction_counter, instruction);
        InstructionAuthsPending::insert(
            instruction_counter,
            u64::try_from(counter_parties.len()).unwrap_or_default(),
        );
        venue.instructions.push(instruction_counter);
        <VenueInfo>::insert(venue_id, venue);
        <InstructionCounter>::put(instruction_counter + 1);
        Self::deposit_event(RawEvent::InstructionCreated(
            did,
            venue_id,
            instruction_counter,
            settlement_type,
            valid_from,
            legs,
        ));
        Ok(instruction_counter)
    }

    /// Settles scheduled instructions. This function is called at the start of every block.
    pub fn on_initialize(block_number: T::BlockNumber) -> Weight {
        let scheduled_instructions = <ScheduledInstructions<T>>::take(block_number);
        let mut legs_executed: u32 = 0;
        let mut weight_for_initialize: Weight = 0;
        let max_legs = T::MaxScheduledInstructionLegsPerBlock::get();

        for (i, scheduled_tx) in scheduled_instructions.iter().enumerate() {
            // NB The actual legs executed can be a bit more than max legs allowed since an instruction must be settled atomically.
            if legs_executed >= max_legs {
                // If max legs is triggered, the pending instructions in this block are rescheduled at the end of the next block.
                let mut next_block_instructions =
                    Self::scheduled_instructions(block_number + 1.into());
                next_block_instructions.extend_from_slice(&scheduled_instructions[i..]);
                <ScheduledInstructions<T>>::insert(
                    block_number + 1.into(),
                    next_block_instructions,
                );
                break;
            }
            let (temp_legs_executed, temp_weight_for_initialize) =
                Self::execute_instruction(*scheduled_tx);
            legs_executed += temp_legs_executed;
            weight_for_initialize += temp_weight_for_initialize;
        }
        weight_for_initialize
    }

    fn unsafe_unauthorize_instruction(
        did: IdentityId,
        instruction_id: u64,
        portfolios: BTreeSet<PortfolioId>,
    ) -> DispatchResult {
        // checks custodianship of portfolios and authorization status.
        for portfolio in &portfolios {
            T::Portfolio::ensure_portfolio_custody(*portfolio, did)?;
            ensure!(
                Self::user_auths(portfolio, instruction_id) == AuthorizationStatus::Authorized,
                Error::<T>::InstructionNotAuthorized
            );
        }
        // Unlock tokens that were previously locked during the authorization.
        let legs = <InstructionLegs<T>>::iter_prefix(instruction_id);
        for (leg_id, leg_details) in
            legs.filter(
                |(_leg_id, leg_details)| match Self::leg_from_to(&leg_details) {
                    Ok((from, _)) => portfolios.contains(&from),
                    Err(_) => false,
                },
            )
        {
            match Self::instruction_leg_status(instruction_id, leg_id) {
                LegStatus::ExecutionToBeSkipped(signer, receipt_uid) => {
                    // Receipt was claimed for this instruction. Therefore, no token unlocking is required, we just unclaim the receipt.
                    <ReceiptsUsed<T>>::insert(&signer, receipt_uid, false);
                    Self::deposit_event(RawEvent::ReceiptUnclaimed(
                        did,
                        instruction_id,
                        leg_id,
                        receipt_uid,
                        signer,
                    ));
                }
                LegStatus::ExecutionPending => {
                    // Tokens are unlocked, need to be unlocked.
                    match leg_details {
                        Leg::NonConfidentialLeg(_) => {
                            let (asset, amount) = Self::leg_asset_and_amount(&leg_details)?;
                            let (from, _) = Self::leg_from_to(&leg_details)?;
                            T::Portfolio::unlock_tokens(&from, &asset, &amount)?;
                        }
                        Leg::ConfidentialLeg(_) => {
                            return Err(Error::<T>::ConfidentialModeNotSupportedYet.into())
                        }
                        Leg::Undefined => return Err(Error::<T>::UndefinedLeg.into()),
                    }
                }
                LegStatus::PendingTokenLock => {
                    return Err(Error::<T>::InstructionNotAuthorized.into())
                }
            };
            <InstructionLegStatus<T>>::insert(instruction_id, leg_id, LegStatus::PendingTokenLock);
        }

        // Updates storage.
        for portfolio in &portfolios {
            <UserAuths>::insert(portfolio, instruction_id, AuthorizationStatus::Pending);
            <AuthsReceived>::remove(instruction_id, portfolio);
            Self::deposit_event(RawEvent::InstructionUnauthorized(
                did,
                *portfolio,
                instruction_id,
            ));
        }

        <InstructionAuthsPending>::mutate(instruction_id, |auths_pending| {
            *auths_pending += u64::try_from(portfolios.len()).unwrap_or_default()
        });

        Ok(())
    }

    fn ensure_instruction_validity(instruction_id: u64) -> DispatchResult {
        let instruction_details = Self::instruction_details(instruction_id);
        ensure!(
            instruction_details.status == InstructionStatus::Pending,
            Error::<T>::InstructionNotPending
        );
        if let Some(valid_from) = instruction_details.valid_from {
            ensure!(
                <pallet_timestamp::Module<T>>::get() >= valid_from,
                Error::<T>::InstructionWaitingValidity
            );
        }
        if let SettlementType::SettleOnBlock(block_number) = instruction_details.settlement_type {
            ensure!(
                block_number > system::Module::<T>::block_number(),
                Error::<T>::InstructionSettleBlockPassed
            );
        }
        Ok(())
    }

    fn get_account_and_balance(
        did: &IdentityId,
        account_id: &MercatAccountId,
    ) -> Result<(PubAccount, EncryptedAmount), DispatchError> {
        Ok((
            <ConfidentialAsset<T>>::mercat_accounts(did, account_id).to_mercat::<T>()?,
            <ConfidentialAsset<T>>::mercat_account_balance(did, account_id).to_mercat::<T>()?,
        ))
    }

    fn execute_instruction(instruction_id: u64) -> (u32, Weight) {
        let legs = <InstructionLegs<T>>::iter_prefix(instruction_id).collect::<Vec<_>>();
        let instructions_processed: u32 = u32::try_from(legs.len()).unwrap_or_default();
        Self::unchecked_release_locks(instruction_id, &legs);
        let weight_for_execution = if Self::instruction_auths_pending(instruction_id) > 0 {
            // Instruction rejected. Unlock any locked tokens and mark receipts as unused.
            // NB: Leg status is not updated because Instruction related details are deleted after settlement in any case.
            Self::deposit_event(RawEvent::InstructionRejected(
                SettlementDID.as_id(),
                instruction_id,
            ));
            weight_for::weight_for_execute_instruction_if_auth_pending::<T>()
        } else {
            let mut transaction_weight = 0;
            // All authorizations received, instruction should be settled.

            let mut failed = false;
            // Verify that the venue still has the required permissions for the tokens involved.
            let tickers: BTreeSet<Ticker> = legs
                .iter()
                .filter_map(|(_, leg)| match leg {
                    Leg::NonConfidentialLeg(non_conf_leg) => Some(non_conf_leg.asset),
                    _ => None,
                })
                .collect();
            let venue_id = Self::instruction_details(instruction_id).venue_id;
            for ticker in &tickers {
                if Self::venue_filtering(ticker) && !Self::venue_allow_list(ticker, venue_id) {
                    failed = true;
                    Self::deposit_event(RawEvent::VenueUnauthorized(
                        SettlementDID.as_id(),
                        *ticker,
                        venue_id,
                    ));
                }
            }

            enum FailureReason {
                BaseTransfer,
                ConfidentialTransfer,
                UndefinedLeg,
                InsufficientMercatAuthorizations,
                GetMercatAccount,
                GetMercatMediatorData,
            }

            if !failed {
                match with_transaction(|| {
                    for (leg_id, leg_details) in legs.iter().filter(|(leg_id, _)| {
                        let status = Self::instruction_leg_status(instruction_id, leg_id);
                        matches!(status, LegStatus::ExecutionPending)
                    }) {
                        match leg_details {
                            Leg::NonConfidentialLeg(leg) => {
                                let result = T::Asset::base_transfer(
                                    leg.from, leg.to, &leg.asset, leg.amount,
                                );
                                if let Ok(post_info) = result {
                                    transaction_weight +=
                                        post_info.actual_weight.unwrap_or_default();
                                } else {
                                    return Err((leg_id, FailureReason::BaseTransfer));
                                }
                            }
                            Leg::ConfidentialLeg(leg) => {
                                let mut tx_data = Self::mercat_tx_data(instruction_id);
                                if tx_data.len() != 3 {
                                    return Err((
                                        leg_id,
                                        FailureReason::InsufficientMercatAuthorizations,
                                    ));
                                }

                                // TODO: change the `MercatTxDataStorage` to make this type of query cleaner.
                                let tx_data = tx_data.remove(2);
                                let decoded_justified_tx = match tx_data {
                                    MercatTxData::JustifiedTransfer(finalized) => {
                                        let mut data: &[u8] = &finalized;
                                        JustifiedTransferTx::decode(&mut data).unwrap()
                                    }
                                    _ => {
                                        return Err((leg_id, FailureReason::GetMercatMediatorData));
                                    }
                                };

                                // Read the mercat_accounts from the confidential-asset pallet.
                                if let Ok((from_mercat, from_mercat_balance)) =
                                    Self::get_account_and_balance(
                                        &leg.from.did,
                                        &leg.from_account_id,
                                    )
                                {
                                    if let Ok((to_mercat, to_mercat_balance)) =
                                        Self::get_account_and_balance(
                                            &leg.to.did,
                                            &leg.to_account_id,
                                        )
                                    {
                                        // Verify the proofs.
                                        let mut rng = rng::Rng::default();
                                        let result = TransactionValidator {}.verify_transaction(
                                            &decoded_justified_tx,
                                            &from_mercat,
                                            &from_mercat_balance,
                                            &to_mercat,
                                            &to_mercat_balance,
                                            &[],
                                            &mut rng,
                                        );
                                        if let Ok((new_sender_balance, new_receiver_balance)) =
                                            result
                                        {
                                            <ConfidentialAsset<T>>::update_mercat_account_balance(
                                                &leg.from.did,
                                                &leg.from_account_id,
                                                new_sender_balance,
                                            );
                                            <ConfidentialAsset<T>>::update_mercat_account_balance(
                                                &leg.to.did,
                                                &leg.to_account_id,
                                                new_receiver_balance,
                                            );
                                        } else {
                                            return Err((leg_id, FailureReason::GetMercatAccount));
                                        }
                                    } else {
                                        return Err((leg_id, FailureReason::GetMercatAccount));
                                    }
                                } else {
                                    return Err((leg_id, FailureReason::ConfidentialTransfer));
                                }
                            }
                            Leg::Undefined => {
                                return Err((leg_id, FailureReason::UndefinedLeg));
                            }
                        }
                    }
                    Ok(())
                }) {
                    Ok(_) => {
                        Self::deposit_event(RawEvent::InstructionExecuted(
                            SettlementDID.as_id(),
                            instruction_id,
                        ));
                    }
                    Err((leg_id, _reason)) => {
                        Self::deposit_event(RawEvent::LegFailedExecution(
                            SettlementDID.as_id(),
                            instruction_id,
                            leg_id.clone(),
                        ));
                        Self::deposit_event(RawEvent::InstructionFailed(
                            SettlementDID.as_id(),
                            instruction_id,
                        ));
                        // TODO: do we want to emit the error reason as well?
                        // match reason {
                        //     FailureReason::BaseTransfer => Err(DispatchError::from(Error::<T>::TODONewError)),
                        //     FailureReason::ConfidentialTransfer => Err(DispatchError::from(Error::<T>::InvalidMercatTransferProof)),
                        //     FailureReason::InsufficientMercatAuthorizations => Err(DispatchError::from(Error::<T>::TODONewError)),
                        //     FailureReason::UndefinedLeg => Err(DispatchError::from(Error::<T>::UndefinedLeg)),
                        // }
                    }
                }
            }
            weight_for::weight_for_execute_instruction_if_auth_no_pending::<T>(transaction_weight)
        };

        // Clean up instruction details to reduce chain bloat.
        <InstructionLegs<T>>::remove_prefix(instruction_id);
        <InstructionDetails<T>>::remove(instruction_id);
        <InstructionLegStatus<T>>::remove_prefix(instruction_id);
        <InstructionAuthsPending>::remove(instruction_id);
        <AuthsReceived>::remove_prefix(instruction_id);
        // NB UserAuths mapping is not cleared.
        (
            instructions_processed,
            weight_for_execution.saturating_add(T::DbWeight::get().writes(5)),
        )
    }

    pub fn unsafe_authorize_instruction(
        did: IdentityId,
        instruction_id: u64,
        portfolios: BTreeSet<PortfolioId>,
    ) -> DispatchResult {
        Self::ensure_instruction_validity(instruction_id)?;

        // Checks portfolio's custodian and if it is a counter party with a pending or rejected authorization.
        for portfolio in &portfolios {
            let user_auth = Self::user_auths(portfolio, instruction_id);
            ensure!(
                user_auth == AuthorizationStatus::Pending
                    || user_auth == AuthorizationStatus::Rejected,
                Error::<T>::NoPendingAuth
            );
            T::Portfolio::ensure_portfolio_custody(*portfolio, did)?;
        }

        with_transaction(|| {
            let legs = <InstructionLegs<T>>::iter_prefix(instruction_id);
            for (leg_id, leg_details) in
                legs.filter(
                    |(_leg_id, leg_details)| match Self::leg_from_to(&leg_details) {
                        Ok((from, _)) => portfolios.contains(&from),
                        Err(_) => false,
                    },
                )
            {
                match leg_details {
                    Leg::NonConfidentialLeg(_) => {
                        let (asset, amount) = Self::leg_asset_and_amount(&leg_details)?;
                        let (from, _) = Self::leg_from_to(&leg_details)?;
                        if T::Portfolio::lock_tokens(&from, &asset, &amount).is_err() {
                            // rustc fails to infer return type of `with_transaction` if you use ?/map_err here.
                            return Err(DispatchError::from(Error::<T>::FailedToLockTokens));
                        }
                        <InstructionLegStatus<T>>::insert(
                            instruction_id,
                            leg_id,
                            LegStatus::ExecutionPending,
                        );
                    }
                    Leg::ConfidentialLeg(_) => {
                        <InstructionLegStatus<T>>::insert(
                            instruction_id,
                            leg_id,
                            LegStatus::ExecutionPending,
                        );
                    }
                    Leg::Undefined => {
                        return Err(DispatchError::from(Error::<T>::UndefinedLeg));
                    }
                }
            }
            Ok(())
        })?;

        let auths_pending = Self::instruction_auths_pending(instruction_id);

        // Updates storage
        for portfolio in &portfolios {
            <UserAuths>::insert(portfolio, instruction_id, AuthorizationStatus::Authorized);
            <AuthsReceived>::insert(instruction_id, portfolio, AuthorizationStatus::Authorized);
            Self::deposit_event(RawEvent::InstructionAuthorized(
                did,
                *portfolio,
                instruction_id,
            ));
        }
        <InstructionAuthsPending>::insert(
            instruction_id,
            auths_pending.saturating_sub(u64::try_from(portfolios.len()).unwrap_or_default()),
        );

        Ok(())
    }

    fn unsafe_claim_receipt(
        did: IdentityId,
        instruction_id: u64,
        leg_id: u64,
        receipt_uid: u64,
        signer: T::AccountId,
        signature: T::OffChainSignature,
    ) -> DispatchResult {
        Self::ensure_instruction_validity(instruction_id)?;

        ensure!(
            Self::instruction_leg_status(instruction_id, leg_id) == LegStatus::ExecutionPending,
            Error::<T>::LegNotPending
        );
        let venue_id = Self::instruction_details(instruction_id).venue_id;
        ensure!(
            Self::venue_signers(venue_id, &signer),
            Error::<T>::UnauthorizedSigner
        );
        ensure!(
            !Self::receipts_used(&signer, receipt_uid),
            Error::<T>::ReceiptAlreadyClaimed
        );

        let leg = Self::instruction_legs(instruction_id, leg_id);

        let (asset, amount) = Self::leg_asset_and_amount(&leg)?;
        let (from, to) = Self::leg_from_to(&leg)?;
        let msg = Receipt {
            receipt_uid,
            from,
            to,
            asset,
            amount,
        };

        ensure!(
            signature.verify(&msg.encode()[..], &signer),
            Error::<T>::InvalidSignature
        );

        let (asset, amount) = Self::leg_asset_and_amount(&leg)?;
        T::Portfolio::unlock_tokens(&from, &asset, &amount)?;

        <ReceiptsUsed<T>>::insert(&signer, receipt_uid, true);

        <InstructionLegStatus<T>>::insert(
            instruction_id,
            leg_id,
            LegStatus::ExecutionToBeSkipped(signer.clone(), receipt_uid),
        );
        Self::deposit_event(RawEvent::ReceiptClaimed(
            did,
            instruction_id,
            leg_id,
            receipt_uid,
            signer,
        ));
        Ok(())
    }

    fn unchecked_release_locks(instruction_id: u64, legs: &Vec<(u64, Leg<T::Balance>)>) {
        for (leg_id, leg_details) in legs.iter() {
            match Self::instruction_leg_status(instruction_id, leg_id) {
                LegStatus::ExecutionToBeSkipped(signer, receipt_uid) => {
                    <ReceiptsUsed<T>>::insert(&signer, receipt_uid, false);
                    Self::deposit_event(RawEvent::ReceiptUnclaimed(
                        SettlementDID.as_id(),
                        instruction_id,
                        *leg_id,
                        receipt_uid,
                        signer,
                    ));
                }
                LegStatus::ExecutionPending => {
                    // This can never return an error since the settlement module
                    // must've locked these tokens when instruction was authorized.
                    match leg_details {
                        Leg::NonConfidentialLeg(non_confidential_leg_detail) => {
                            T::Portfolio::unlock_tokens(
                                &non_confidential_leg_detail.from,
                                &non_confidential_leg_detail.asset,
                                &non_confidential_leg_detail.amount,
                            )
                            .ok();
                        }
                        Leg::ConfidentialLeg(_) => {
                            // TODO: should I dispatch error at this point since the function does not have an error type?
                            // return Err(Error::<T>::ConfidentialModeNotSupportedYet.into())
                        }
                        Leg::Undefined => {
                            // TODO: should I dispatch error at this point since the function does not have an error type?
                            // return Err(Error::<T>::UndefinedLeg.into())
                        }
                    }
                }
                LegStatus::PendingTokenLock => {}
            }
        }
    }

    fn is_instruction_executed(
        auths_pending: u64,
        settlement_type: SettlementType<T::BlockNumber>,
        id: u64,
    ) -> Weight {
        if auths_pending == 0 && settlement_type == SettlementType::SettleOnAuthorization {
            Self::execute_instruction(id).1
        } else {
            Zero::zero()
        }
    }
}
