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
//! - `affirm_instruction` - Affirms an existing instruction.
//! - `reject_instruction` - Rejects an existing instruction.
//! - `set_venue_filtering` - Enables or disabled venue filtering for a token.
//! - `allow_venues` - Allows additional venues to create instructions involving an asset.
//! - `disallow_venues` - Revokes permission given to venues for creating instructions involving a particular asset.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::Encode;
use frame_support::dispatch::{
    DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo, PostDispatchInfo,
};
use frame_support::pallet_prelude::*;
use frame_support::storage::with_transaction as frame_support_with_transaction;
use frame_support::storage::TransactionOutcome;
use frame_support::traits::schedule::v3::Named as ScheduleNamed;
use frame_support::traits::schedule::DispatchTime;
use frame_support::traits::{Get, QueryPreimage, StorePreimage};
use frame_support::weights::Weight;
use frame_support::{ensure, BoundedBTreeSet};
use frame_system::pallet_prelude::*;
use frame_system::{ensure_root, RawOrigin};
use sp_runtime::traits::One;
use sp_runtime::Saturating;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::convert::TryFrom;
use sp_std::prelude::*;
use sp_std::vec;

use pallet_asset::MandatoryMediators;
use pallet_base::{ensure_string_limited, try_next_post};
use pallet_identity::DidRecords;
use pallet_nft::WeightInfo as NFTWeightInfo;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::constants::queue_priority::SETTLEMENT_INSTRUCTION_EXECUTION_PRIORITY;
use polymesh_primitives::crypto::{ChainScopedMessage, SETTLEMENT_RECEIPT_LABEL};
use polymesh_primitives::settlement::{
    AffirmationCount, AffirmationRequirement, AffirmationStatus, AssetCount,
    ExecuteInstructionInfo, FilteredLegs, Instruction, InstructionId, InstructionInfo,
    InstructionStatus, Leg, LegId, LegStatus, MediatorAffirmationStatus, Receipt, ReceiptDetails,
    ReceiptMetadata, SettlementType, Venue, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::traits::{
    AffirmationFnTrait, AssetFnTrait, PortfolioFnConfig, PortfolioFnTrait, SettlementFnTrait,
};
use polymesh_primitives::with_transaction;
use polymesh_primitives::SystematicIssuers::Settlement as SettlementDID;
use polymesh_primitives::{
    AssetHolder, Balance, Fund, FundDescription, IdentityId, Memo, NFTs, SecondaryKey, WeightMeter,
};

type System<T> = frame_system::Pallet<T>;
type Asset<T> = pallet_asset::Pallet<T>;
type ExternalAgents<T> = pallet_external_agents::Pallet<T>;
type Nft<T> = pallet_nft::Pallet<T>;
type EnsureValidInstructionResult<AccountId, Moment, BlockNumber> = Result<
    (
        IdentityId,
        Option<SecondaryKey<AccountId>>,
        Instruction<Moment, BlockNumber>,
    ),
    DispatchError,
>;

// Move imports to pallet module
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new venue has been created (did, venue_id, details, type)
        VenueCreated(IdentityId, VenueId, VenueDetails, VenueType),
        /// An existing venue's details has been updated (did, venue_id, details)
        VenueDetailsUpdated(IdentityId, VenueId, VenueDetails),
        /// An existing venue's type has been updated (did, venue_id, type)
        VenueTypeUpdated(IdentityId, VenueId, VenueType),
        /// An instruction has been affirmed (did, asset_holder, instruction_id)
        InstructionAffirmed(IdentityId, AssetHolder, InstructionId),
        /// An affirmation has been withdrawn (did, asset_holder, instruction_id)
        AffirmationWithdrawn(IdentityId, AssetHolder, InstructionId),
        /// An instruction has been rejected (did, instruction_id)
        InstructionRejected(IdentityId, InstructionId),
        /// A receipt has been claimed (did, instruction_id, leg_id, receipt_uid, signer, receipt metadata)
        ReceiptClaimed(
            IdentityId,
            InstructionId,
            LegId,
            u64,
            T::AccountId,
            Option<ReceiptMetadata>,
        ),
        /// Venue filtering has been enabled or disabled for an asset (did, AssetId, filtering_enabled)
        VenueFiltering(IdentityId, AssetId, bool),
        /// Venues added to allow list (did, AssetId, vec<venue_id>)
        VenuesAllowed(IdentityId, AssetId, Vec<VenueId>),
        /// Venues added to block list (did, AssetId, vec<venue_id>)
        VenuesBlocked(IdentityId, AssetId, Vec<VenueId>),
        /// Execution of a leg failed (did, instruction_id, leg_id)
        LegFailedExecution(IdentityId, InstructionId, LegId),
        /// Instruction executed successfully(did, instruction_id)
        InstructionExecuted(IdentityId, InstructionId),
        /// Venue not part of the token's allow list (did, AssetId, venue_id)
        VenueUnauthorized(IdentityId, AssetId, VenueId),
        /// An existing venue's signers has been updated (did, venue_id, signers, update_type)
        VenueSignersUpdated(IdentityId, VenueId, BTreeSet<T::AccountId>, bool),
        /// Settlement manually executed (did, id)
        SettlementManuallyExecuted(IdentityId, InstructionId),
        /// A new instruction has been created
        /// (did, venue_id, instruction_id, settlement_type, trade_date, value_date, legs, memo)
        InstructionCreated(
            IdentityId,
            Option<VenueId>,
            InstructionId,
            SettlementType<BlockNumberFor<T>>,
            Option<T::Moment>,
            Option<T::Moment>,
            Vec<Leg>,
            Option<Memo>,
        ),
        /// Failed to execute instruction.
        FailedToExecuteInstruction(InstructionId, DispatchError),
        /// An instruction has been automatically affirmed.
        /// Parameters: [`IdentityId`] of the caller, [`AssetHolder`] of the receiver, and [`InstructionId`] of the instruction.
        InstructionAutomaticallyAffirmed(IdentityId, AssetHolder, InstructionId),
        /// An instruction has affirmed by a mediator.
        /// Parameters: [`IdentityId`] of the mediator and [`InstructionId`] of the instruction.
        MediatorAffirmationReceived(IdentityId, InstructionId, Option<T::Moment>),
        /// An instruction affirmation has been withdrawn by a mediator.
        /// Parameters: [`IdentityId`] of the mediator and [`InstructionId`] of the instruction.
        MediatorAffirmationWithdrawn(IdentityId, InstructionId),
        /// An instruction with mediators has been created.
        /// Parameters: [`InstructionId`] of the instruction and the [`IdentityId`] of all mediators.
        InstructionMediators(InstructionId, BTreeSet<IdentityId>),
        /// An instruction has been sucessfully locked for execution
        ///
        /// Parameters:
        /// - `IdentityId`: The [`IdentityId`] of the caller.
        /// - `InstructionId`: The [`InstructionId`] of the instruction.
        InstructionLocked(IdentityId, InstructionId),
        /// An identity's mandatory receiver affirmation policy has been updated.
        MandatoryReceiverAffirmationSet(IdentityId, AffirmationRequirement),
        /// An instruction has been unlocked by a mediator.
        ///
        /// Parameters:
        /// - `IdentityId`: The [`IdentityId`] of the mediator.
        /// - `InstructionId`: The [`InstructionId`] of the instruction.
        InstructionUnlocked(IdentityId, InstructionId),
        /// Funds have been transferred
        ///
        /// Parameters:
        /// - `IdentityId`: The [`IdentityId`] of the caller.
        /// - `AssetHolder`: The source [`AssetHolder`] of the transfer.
        /// - `AssetHolder`: The destination [`AssetHolder`] of the transfer.
        /// - `Fund`: The [`Fund`] being transferred.
        FundsTransferred(IdentityId, AssetHolder, AssetHolder, Fund),
    }

    pub trait WeightInfo {
        fn create_venue(d: u32, u: u32) -> Weight;
        fn update_venue_details(d: u32) -> Weight;
        fn update_venue_type() -> Weight;
        fn update_venue_signers(u: u32) -> Weight;
        fn affirm_with_receipts(f: u32, n: u32, o: u32) -> Weight;
        fn set_venue_filtering() -> Weight;
        fn allow_venues(u: u32) -> Weight;
        fn disallow_venues(u: u32) -> Weight;
        fn execute_manual_instruction(f: u32, n: u32, o: u32) -> Weight;
        fn add_instruction(f: u32, n: u32, o: u32) -> Weight;
        fn add_and_affirm_instruction(f: u32, n: u32, o: u32) -> Weight;
        fn affirm_instruction(f: u32, n: u32) -> Weight;
        fn execute_instruction_paused(f: u32, n: u32, o: u32) -> Weight;
        fn execute_scheduled_instruction(f: u32, n: u32, o: u32) -> Weight;
        fn ensure_root_origin() -> Weight;
        fn affirm_with_receipts_rcv(f: u32, n: u32, o: u32) -> Weight;
        fn affirm_instruction_rcv(f: u32, n: u32) -> Weight;
        /// Weight for writing the optional `InstructionMemos` entry during
        /// `base_add_instruction`. Charged dynamically only when the caller
        /// supplies a memo.
        fn set_instruction_memo() -> Weight;
        /// Weight for `unsafe_affirm_instruction` with a single-holder receiver set
        /// of type `AssetHolder::Account` (no sender-side locks). Used by
        /// `transfer_funds` for the spender-is-receiver inline affirmation path.
        fn unsafe_affirm_instruction_receiver_account() -> Weight;
        /// Weight for `unsafe_affirm_instruction` with a single-holder receiver set
        /// of type `AssetHolder::Portfolio` (no sender-side locks). Used by
        /// `transfer_funds` for the spender-is-receiver inline affirmation path.
        fn unsafe_affirm_instruction_receiver_portfolio() -> Weight;
        fn add_instruction_with_mediators(f: u32, n: u32, o: u32, m: u32) -> Weight;
        fn add_and_affirm_with_mediators(f: u32, n: u32, o: u32, m: u32) -> Weight;
        fn affirm_instruction_as_mediator() -> Weight;
        fn base_reject_instruction(f: u32, n: u32, o: u32) -> Weight;
        fn lock_instruction_extrinsic(f: u32, n: u32, o: u32) -> Weight;
        fn execute_locked_instruction(f: u32, n: u32, o: u32) -> Weight;
        fn execute_manual_instruction_paused(f: u32, n: u32, o: u32) -> Weight;
        fn set_mandatory_receiver_affirmation() -> Weight;

        /// Same-DID direct transfer between portfolios.
        fn transfer_funds_portfolio_same_did() -> Weight;
        /// Cross-DID transfer with portfolio holders (instruction stays pending).
        fn transfer_funds_portfolio_diff_did() -> Weight;
        /// Same-DID direct transfer between accounts.
        fn transfer_funds_account_same_did() -> Weight;
        /// Cross-DID transfer with account holders (instruction stays pending).
        fn transfer_funds_account_diff_did() -> Weight;

        /// Same-DID NFT transfer between portfolios (custody check).
        fn transfer_funds_nft_portfolio_same_did(n: u32) -> Weight;
        /// Cross-DID NFT transfer with portfolio holders (instruction stays pending).
        fn transfer_funds_nft_portfolio_diff_did(n: u32) -> Weight;
        /// Same-DID NFT transfer from account.
        fn transfer_funds_nft_account_same_did(n: u32) -> Weight;
        /// Cross-DID NFT transfer from account (instruction stays pending).
        fn transfer_funds_nft_account_diff_did(n: u32) -> Weight;

        fn unlock_instruction() -> Weight;

        fn add_and_affirm_with_mediators_legs(
            legs: &[Leg],
            portfolios: u32,
            n_mediators: u32,
        ) -> Weight {
            let (f, n, o) = Self::get_transfer_by_asset(legs, portfolios);
            Self::add_and_affirm_with_mediators(f, n, o, n_mediators)
        }
        fn add_instruction_with_mediators_legs(legs: &[Leg], n_mediators: u32) -> Weight {
            let (f, n, o) = Self::get_transfer_by_asset(legs, 0);
            Self::add_instruction_with_mediators(f, n, o, n_mediators)
        }
        fn add_instruction_legs(legs: &[Leg]) -> Weight {
            let (f, n, o) = Self::get_transfer_by_asset(legs, 0);
            Self::add_instruction(f, n, o)
        }
        fn add_and_affirm_instruction_legs(legs: &[Leg], portfolios: u32) -> Weight {
            let (f, n, o) = Self::get_transfer_by_asset(legs, portfolios);
            Self::add_and_affirm_instruction(f, n, o)
        }
        fn execute_manual_weight_limit(
            weight: &Option<Weight>,
            f: &u32,
            n: &u32,
            o: &u32,
        ) -> Weight {
            let min_weight = Self::execute_locked_instruction(0, 0, 1);

            if let Some(weight) = weight {
                return weight.max(min_weight);
            }

            Self::execute_manual_instruction(*f, *n, *o).max(min_weight)
        }
        fn get_transfer_by_asset(legs: &[Leg], portfolios: u32) -> (u32, u32, u32) {
            let asset_count =
                AssetCount::try_from_legs(legs).unwrap_or(AssetCount::new(1024, 1024, 1024));
            let f = asset_count.fungible();
            let n = asset_count.non_fungible();
            let max_portfolios = (f.saturating_add(n)).saturating_mul(2); // 2 portfolios per leg.  (f+n = max legs).
            if portfolios > max_portfolios {
                // Too many portfolios, return worse-case count based on portfolio count.
                return (portfolios, portfolios, 1024);
            }
            (f, n, asset_count.off_chain())
        }
        fn affirm_with_receipts_input(
            affirmation_count: Option<AffirmationCount>,
            portfolios: u32,
        ) -> Weight {
            match affirmation_count {
                Some(affirmation_count) => {
                    let max_portfolios = affirmation_count.max_portfolios();
                    if portfolios > max_portfolios {
                        // Too many portfolios, return worse-case weight based on portfolio count.
                        return Self::affirm_with_receipts(portfolios, portfolios, 10);
                    }
                    // The weight for the assets being sent
                    let sender_asset_count = affirmation_count.sender_asset_count();
                    let sender_side_weight = Self::affirm_with_receipts(
                        sender_asset_count.fungible(),
                        sender_asset_count.non_fungible(),
                        affirmation_count.offchain_count(),
                    );
                    // The weight for the assets being received
                    let receiver_asset_count = affirmation_count.receiver_asset_count();
                    let receiver_side_weight = Self::affirm_with_receipts_rcv(
                        receiver_asset_count.fungible(),
                        receiver_asset_count.non_fungible(),
                        0,
                    );
                    // Common reads/writes are being added twice
                    let duplicated_weight = Self::affirm_with_receipts_rcv(0, 0, 0);
                    // The actual weight is the sum of the sender and receiver weights subtracted by the duplicated weight
                    sender_side_weight
                        .saturating_add(receiver_side_weight)
                        .saturating_sub(duplicated_weight)
                }
                None => {
                    if portfolios > (10 + 100) * 2 {
                        // Too many portfolios, return worse-case weight based on portfolio count.
                        Self::affirm_with_receipts(portfolios, portfolios, 10)
                    } else {
                        Self::affirm_with_receipts(10, 100, 10)
                    }
                }
            }
        }
        fn affirm_instruction_input(
            affirmation_count: Option<AffirmationCount>,
            portfolios: u32,
        ) -> Weight {
            match affirmation_count {
                Some(affirmation_count) => {
                    let max_portfolios = affirmation_count.max_portfolios();
                    if portfolios > max_portfolios {
                        // Too many portfolios, return worse-case weight based on portfolio count.
                        return Self::affirm_instruction(portfolios, portfolios);
                    }
                    // The weight for the assets being sent
                    let sender_asset_count = affirmation_count.sender_asset_count();
                    let sender_side_weight = Self::affirm_instruction(
                        sender_asset_count.fungible(),
                        sender_asset_count.non_fungible(),
                    );
                    // The weight for the assets being received
                    let receiver_asset_count = affirmation_count.receiver_asset_count();
                    let receiver_side_weight = Self::affirm_instruction_rcv(
                        receiver_asset_count.fungible(),
                        receiver_asset_count.non_fungible(),
                    );
                    // Common reads/writes are being added twice
                    let duplicated_weight = Self::affirm_instruction_rcv(0, 0);
                    // The actual weight is the sum of the sender and receiver weights subtracted by the duplicated weight
                    sender_side_weight
                        .saturating_add(receiver_side_weight)
                        .saturating_sub(duplicated_weight)
                }
                None => {
                    if portfolios > (10 + 100) * 2 {
                        // Too many portfolios, return worse-case weight based on portfolio count.
                        Self::affirm_instruction(portfolios, portfolios)
                    } else {
                        Self::affirm_instruction(10, 100)
                    }
                }
            }
        }

        fn reject_instruction(inst_asset_count: Option<AssetCount>) -> Weight {
            let inst_asset_count = inst_asset_count.unwrap_or(AssetCount::new(10, 100, 10));

            let input_weight = Self::base_reject_instruction(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
            );

            let min_weight = Self::base_reject_instruction(0, 0, 1);

            input_weight.max(min_weight)
        }

        fn lock_instruction(weight_limit: Weight) -> Weight {
            let min_weight = Self::lock_instruction_extrinsic(0, 0, 1);
            weight_limit.max(min_weight)
        }
    }

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_asset::Config
        + pallet_compliance_manager::Config
        + pallet_identity::Config
        + pallet_permissions::Config
        + pallet_nft::Config
        + pallet_timestamp::Config
        + PortfolioFnConfig
    {
        /// A call type used by the scheduler.
        type SchedulerCall: From<Call<Self>>
            + Into<<Self as pallet_identity::Config>::Proposal>
            + Encode;

        /// Scheduler of settlement instructions.
        type Scheduler: ScheduleNamed<
            BlockNumberFor<Self>,
            Self::SchedulerCall,
            Self::SchedulerOrigin,
            Hasher = Self::Hashing,
        >;

        /// Maximum number of fungible assets that can be in a single instruction.
        #[pallet::constant]
        type MaxNumberOfFungibleAssets: Get<u32>;

        /// Weight information for extrinsic of the settlement pallet.  
        type WeightInfo: WeightInfo;

        /// Maximum number of NFTs that can be transferred in a leg.
        #[pallet::constant]
        type MaxNumberOfNFTsPerLeg: Get<u32>;

        /// Maximum number of NFTs that can be transferred in a instruction.
        #[pallet::constant]
        type MaxNumberOfNFTs: Get<u32>;

        /// Maximum number of off-chain assets that can be transferred in a instruction.
        #[pallet::constant]
        type MaxNumberOfOffChainAssets: Get<u32>;

        /// Maximum number of portfolios.
        #[pallet::constant]
        type MaxNumberOfAssetHolders: Get<u32>;

        /// Maximum number of venue signers.
        #[pallet::constant]
        type MaxNumberOfVenueSigners: Get<u32>;

        /// Maximum number mediators in the instruction level (this does not include asset mediators).
        #[pallet::constant]
        type MaxInstructionMediators: Get<u32>;

        /// The maximum time period that an instruction can be held in the `LockedForExecution` status.
        #[pallet::constant]
        type MaximumLockPeriod: Get<Self::Moment>;

        /// The minimum cooldown period a mediator must wait after unlocking before relocking an instruction.
        #[pallet::constant]
        type RelockCooldown: Get<Self::Moment>;

        /// The maximum number of times an instruction can be relocked.
        #[pallet::constant]
        type MaxRelockCount: Get<u32>;

        /// Preimage provider for the scheduler.
        type SchedulerPreimage: QueryPreimage<H = Self::Hashing> + StorePreimage;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Venue does not exist.
        InvalidVenue,
        /// Sender does not have required permissions.
        Unauthorized,
        /// Instruction has not been affirmed.
        InstructionNotAffirmed,
        /// Signer is not authorized by the venue.
        UnauthorizedSigner,
        /// Receipt already used.
        ReceiptAlreadyClaimed,
        /// Venue does not have required permissions.
        UnauthorizedVenue,
        /// Instruction has invalid dates
        InstructionDatesInvalid,
        /// Instruction's target settle block reached.
        InstructionSettleBlockPassed,
        /// Offchain signature is invalid.
        InvalidSignature,
        /// Sender and receiver are the same.
        SameSenderReceiver,
        /// The provided settlement block number is in the past and cannot be used by the scheduler.
        SettleOnPastBlock,
        /// The current instruction affirmation status does not support the requested action.
        UnexpectedAffirmationStatus,
        /// Scheduling of an instruction fails.
        FailedToSchedule,
        /// Instruction status is unknown
        UnknownInstruction,
        /// Signer is already added to venue.
        SignerAlreadyExists,
        /// Signer is not added to venue.
        SignerDoesNotExist,
        /// Instruction leg amount can't be zero.
        ZeroAmount,
        /// Instruction settlement block has not yet been reached.
        InstructionSettleBlockNotReached,
        /// The caller is not a party of this instruction.
        CallerIsNotAParty,
        /// The number of nfts being transferred in the instruction was exceeded.
        MaxNumberOfNFTsExceeded,
        /// The given number of nfts being transferred was underestimated.
        NumberOfTransferredNFTsUnderestimated,
        /// Off-chain receipts can only be used for off-chain leg type.
        ReceiptForInvalidLegType,
        /// The maximum weight limit for executing the function was exceeded.
        WeightLimitExceeded,
        /// The maximum number of fungible assets was exceeded.
        MaxNumberOfFungibleAssetsExceeded,
        /// The maximum number of off-chain assets was exceeded.
        MaxNumberOfOffChainAssetsExceeded,
        /// The given number of fungible transfers was underestimated.
        NumberOfFungibleTransfersUnderestimated,
        /// AssetId could not be found on chain.
        UnexpectedOFFChainAsset,
        /// Off-Chain assets cannot be locked.
        OffChainAssetCantBeLocked,
        /// The given number of off-chain transfers was underestimated.
        NumberOfOffChainTransfersUnderestimated,
        /// No leg with the given id was found
        LegNotFound,
        /// The input weight is less than the minimum required.
        InputWeightIsLessThanMinimum,
        /// The maximum number of receipts was exceeded.
        MaxNumberOfReceiptsExceeded,
        /// There are parties who have not affirmed the instruction.
        NotAllAffirmationsHaveBeenReceived,
        /// Only [`InstructionStatus::Pending`] or [`InstructionStatus::Failed`] instructions can be executed.
        InvalidInstructionStatusForExecution,
        /// The instruction failed to release asset locks or transfer the assets.
        FailedToReleaseLockOrTransferAssets,
        /// No duplicate uid are allowed for different receipts.
        DuplicateReceiptUid,
        /// The instruction id in all receipts must match the extrinsic parameter.
        ReceiptInstructionIdMissmatch,
        /// Multiple receipts for the same leg are not allowed.
        MultipleReceiptsForOneLeg,
        /// An invalid has been reached.
        UnexpectedLegStatus,
        /// The maximum number of venue signers was exceeded.
        NumberOfVenueSignersExceeded,
        /// The caller is not a mediator in the instruction.
        CallerIsNotAMediator,
        /// The mediator's expiry date must be in the future.
        InvalidExpiryDate,
        /// The expiry date for the mediator's affirmation has passed.
        MediatorAffirmationExpired,
        /// Offchain assets must have a venue.
        OffChainAssetsMustHaveAVenue,
        /// Unexpected settlement type.
        UnexpectedSettlementType,
        /// [`InstructionStatus::Unknow`] can't be rejected.
        InvalidInstructionStatusForRejection,
        /// All locked instructions must register a lock timestamp.
        LockTimestampNotFound,
        /// The instruction has been locked for too much time.
        ExceededMaximumLockingPeriod,
        /// Not all conditions for transferring the asset have been met.
        FailedAssetTransferringConditions,
        /// Locked instructions can't have affirmations withdrawn.
        InvalidInstructionStatusForWithdrawal,
        /// Receiver identity not found.
        ReceiverIdentityNotFound,
        /// Invalid account id.
        InvalidAccountId,
        /// The receipt has expired and can no longer be claimed.
        ReceiptExpired,
        /// Source and destination are the exact same AssetHolder.
        SenderSameAsReceiver,
        /// Deprecated placeholder kept to preserve error indices after removing
        /// `AllowancesNotSupportedForNFTs` (NFT spender transfers now use approvals).
        DeprecatedAllowancesNotSupportedForNFTs,
        /// The instruction is already locked. It must be unlocked before relocking.
        InstructionAlreadyLocked,
        /// The instruction is not in `LockedForExecution` status and cannot be unlocked.
        InstructionNotLocked,
        /// The relock cooldown period has not expired since the last unlock.
        RelockCooldownNotExpired,
        /// The maximum number of relocks for this instruction has been exceeded.
        MaxRelockCountExceeded,
        /// At least one mediator is required for this instruction.
        MissingInstructionMediators,
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    /// Info about a venue. venue_id -> venue
    pub type VenueInfo<T: Config> = StorageMap<_, Twox64Concat, VenueId, Venue, OptionQuery>;

    /// Free-form text about a venue. venue_id -> `VenueDetails`
    /// Only needed for the UI.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Details<T: Config> = StorageMap<_, Twox64Concat, VenueId, VenueDetails, ValueQuery>;

    /// Instructions under a venue.
    /// Only needed for the UI.
    ///
    /// venue_id -> instruction_id -> ()
    #[pallet::storage]
    pub type VenueInstructions<T: Config> =
        StorageDoubleMap<_, Twox64Concat, VenueId, Twox64Concat, InstructionId, (), ValueQuery>;

    /// Signers allowed by the venue. (venue_id, signer) -> bool
    #[pallet::storage]
    pub type VenueSigners<T: Config> =
        StorageDoubleMap<_, Twox64Concat, VenueId, Twox64Concat, T::AccountId, bool, ValueQuery>;

    /// Venues create by an identity.
    /// Only needed for the UI.
    ///
    /// identity -> venue_id -> ()
    #[pallet::storage]
    pub type UserVenues<T: Config> =
        StorageDoubleMap<_, Twox64Concat, IdentityId, Twox64Concat, VenueId, (), ValueQuery>;

    /// Details about an instruction. instruction_id -> instruction_details
    #[pallet::storage]
    pub type InstructionDetails<T: Config> = StorageMap<
        _,
        Twox64Concat,
        InstructionId,
        Instruction<T::Moment, BlockNumberFor<T>>,
        ValueQuery,
    >;

    /// Status of a leg under an instruction. (instruction_id, leg_id) -> LegStatus
    #[pallet::storage]
    pub type InstructionLegStatus<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        InstructionId,
        Twox64Concat,
        LegId,
        LegStatus<T::AccountId>,
        ValueQuery,
    >;

    /// Number of affirmations pending before instruction is executed. instruction_id -> affirm_pending
    #[pallet::storage]
    pub type InstructionAffirmsPending<T: Config> =
        StorageMap<_, Twox64Concat, InstructionId, u64, ValueQuery>;

    /// Tracks affirmations received for an instruction. (instruction_id, counter_party) -> AffirmationStatus
    #[pallet::storage]
    pub type AffirmsReceived<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        InstructionId,
        Twox64Concat,
        AssetHolder,
        AffirmationStatus,
        ValueQuery,
    >;

    /// When `true`, the identity has opted in to mandatory receiver affirmation.
    /// Default is `false` (no affirmation required).
    #[pallet::storage]
    pub type MandatoryReceiverAffirmation<T: Config> =
        StorageMap<_, Identity, IdentityId, bool, ValueQuery>;

    /// Helps a user track their pending instructions and affirmations (only needed for UI).
    /// (counter_party, instruction_id) -> AffirmationStatus
    #[pallet::storage]
    pub type UserAffirmations<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        AssetHolder,
        Twox64Concat,
        InstructionId,
        AffirmationStatus,
        ValueQuery,
    >;

    /// Tracks redemption of receipts. (signer, receipt_uid) -> receipt_used
    #[pallet::storage]
    pub(super) type ReceiptsUsed<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Blake2_128Concat, u64, bool, ValueQuery>;

    /// Tracks if a token has enabled filtering venues that can create instructions involving their token. AssetId -> filtering_enabled
    #[pallet::storage]
    pub(super) type VenueFiltering<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, bool, ValueQuery>;

    /// Venues that are allowed to create instructions involving a particular asset. Only used if filtering is enabled.
    /// ([`AssetId`], venue_id) -> allowed
    #[pallet::storage]
    pub(super) type VenueAllowList<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, AssetId, Twox64Concat, VenueId, bool, ValueQuery>;

    /// Number of venues in the system (It's one more than the actual number)
    #[pallet::storage]
    pub type VenueCounter<T: Config> = StorageValue<_, VenueId, ValueQuery>;

    /// Number of instructions in the system (It's one more than the actual number)
    #[pallet::storage]
    pub type InstructionCounter<T: Config> = StorageValue<_, InstructionId, ValueQuery>;

    /// Instruction memo
    #[pallet::storage]
    pub type InstructionMemos<T: Config> =
        StorageMap<_, Twox64Concat, InstructionId, Memo, OptionQuery>;

    /// Instruction statuses. instruction_id -> InstructionStatus
    #[pallet::storage]
    pub type InstructionStatuses<T: Config> = StorageMap<
        _,
        Twox64Concat,
        InstructionId,
        InstructionStatus<BlockNumberFor<T>>,
        ValueQuery,
    >;

    /// Legs under an instruction. (instruction_id, leg_id) -> Leg
    #[pallet::storage]
    #[pallet::unbounded]
    pub type InstructionLegs<T: Config> =
        StorageDoubleMap<_, Twox64Concat, InstructionId, Twox64Concat, LegId, Leg, OptionQuery>;

    /// Tracks the affirmation status for offchain legs in a instruction. [`(InstructionId, LegId)`] -> [`AffirmationStatus`]
    #[pallet::storage]
    pub type OffChainAffirmations<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        InstructionId,
        Twox64Concat,
        LegId,
        AffirmationStatus,
        ValueQuery,
    >;

    /// Tracks the number of signers each venue has.
    #[pallet::storage]
    pub type NumberOfVenueSigners<T: Config> =
        StorageMap<_, Twox64Concat, VenueId, u32, ValueQuery>;

    /// The status for the mediators affirmation.
    #[pallet::storage]
    pub type InstructionMediatorsAffirmations<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        InstructionId,
        Identity,
        IdentityId,
        MediatorAffirmationStatus<T::Moment>,
        ValueQuery,
    >;

    /// The moment the instruction was moved to the `LockedForExecution` status.
    #[pallet::storage]
    pub type LockedTimestamp<T: Config> =
        StorageMap<_, Twox64Concat, InstructionId, T::Moment, OptionQuery>;

    /// The moment the instruction was unlocked by a mediator. Used to enforce the relock cooldown.
    #[pallet::storage]
    pub type UnlockedTimestamp<T: Config> =
        StorageMap<_, Twox64Concat, InstructionId, T::Moment, OptionQuery>;

    /// The number of times an instruction has been relocked.
    #[pallet::storage]
    pub type InstructionRelockCount<T: Config> =
        StorageMap<_, Twox64Concat, InstructionId, u32, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            VenueCounter::<T>::put(VenueId(1));
            InstructionCounter::<T>::put(InstructionId(1));
            STORAGE_VERSION.put::<Pallet<T>>();
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Registers a new venue.
        ///
        /// * `details` - Extra details about a venue
        /// * `signers` - Array of signers that are allowed to sign receipts for this venue
        /// * `typ` - Type of venue being created
        #[pallet::weight(<T as Config>::WeightInfo::create_venue(details.len() as u32, signers.len() as u32))]
        #[pallet::call_index(0)]
        pub fn create_venue(
            origin: OriginFor<T>,
            details: VenueDetails,
            signers: BTreeSet<T::AccountId>,
            typ: VenueType,
        ) -> DispatchResult {
            // Ensure permissions and details limit.
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
            ensure_string_limited::<T>(&details)?;

            ensure!(
                signers.len() <= T::MaxNumberOfVenueSigners::get() as usize,
                Error::<T>::NumberOfVenueSignersExceeded
            );

            // Advance venue counter.
            let id = VenueCounter::<T>::try_mutate(try_next_post::<T, _>)?;

            // Other commits to storage + emit event.
            let venue = Venue {
                creator: did,
                venue_type: typ,
            };
            VenueInfo::<T>::insert(id, venue);
            Details::<T>::insert(id, details.clone());
            NumberOfVenueSigners::<T>::insert(id, signers.len() as u32);
            for signer in signers {
                <VenueSigners<T>>::insert(id, signer, true);
            }
            UserVenues::<T>::insert(did, id, ());
            Self::deposit_event(Event::VenueCreated(did, id, details, typ));
            Ok(())
        }

        /// Edit a venue's details.
        ///
        /// * `id` specifies the ID of the venue to edit.
        /// * `details` specifies the updated venue details.
        #[pallet::weight(<T as Config>::WeightInfo::update_venue_details(details.len() as u32))]
        #[pallet::call_index(1)]
        pub fn update_venue_details(
            origin: OriginFor<T>,
            id: VenueId,
            details: VenueDetails,
        ) -> DispatchResult {
            ensure_string_limited::<T>(&details)?;
            let caller_did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
            Self::ensure_venue_creator(&id, &caller_did)?;

            // Commit to storage.
            Details::<T>::insert(id, details.clone());
            Self::deposit_event(Event::VenueDetailsUpdated(caller_did, id, details));
            Ok(())
        }

        /// Edit a venue's type.
        ///
        /// * `id` specifies the ID of the venue to edit.
        /// * `type` specifies the new type of the venue.
        #[pallet::weight(<T as Config>::WeightInfo::update_venue_type())]
        #[pallet::call_index(2)]
        pub fn update_venue_type(
            origin: OriginFor<T>,
            id: VenueId,
            typ: VenueType,
        ) -> DispatchResult {
            let caller_did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;

            let mut venue = Self::ensure_venue_creator(&id, &caller_did)?;
            venue.venue_type = typ;
            VenueInfo::<T>::insert(id, venue);

            Self::deposit_event(Event::VenueTypeUpdated(caller_did, id, typ));
            Ok(())
        }

        /// Affirms an instruction using receipts for offchain transfers.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being affirmed.
        /// * `receipt_details` - a vector of [`ReceiptDetails`], which contain the details about the offchain transfer.
        /// * `holder_set` - a vector of [`AssetHolder`] under the caller's control and intended for affirmation.
        ///
        /// # Permissions
        /// * Portfolio/Account
        #[pallet::weight(<T as Config>::WeightInfo::affirm_with_receipts_input(None, holder_set.len() as u32))]
        #[pallet::call_index(3)]
        pub fn affirm_with_receipts(
            origin: OriginFor<T>,
            id: InstructionId,
            receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature, T::Moment>>,
            holder_set: BoundedBTreeSet<AssetHolder, T::MaxNumberOfAssetHolders>,
        ) -> DispatchResultWithPostInfo {
            Self::affirm_with_receipts_and_maybe_schedule_instruction(
                origin,
                id,
                receipt_details,
                holder_set.into_inner(),
                None,
            )
        }

        /// Enables or disabled venue filtering for a token.
        ///
        /// # Arguments
        /// * `asset_id` - AssetId of the token in question.
        /// * `enabled` - Boolean that decides if the filtering should be enabled.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::set_venue_filtering())]
        #[pallet::call_index(4)]
        pub fn set_venue_filtering(
            origin: OriginFor<T>,
            asset_id: AssetId,
            enabled: bool,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
            if enabled {
                VenueFiltering::<T>::insert(asset_id, enabled);
            } else {
                VenueFiltering::<T>::remove(asset_id);
            }
            Self::deposit_event(Event::VenueFiltering(did, asset_id, enabled));
            Ok(())
        }

        /// Allows additional venues to create instructions involving an asset.
        ///
        /// * `asset_id` - AssetId of the token in question.
        /// * `venues` - Array of venues that are allowed to create instructions for the token in question.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::allow_venues(venues.len() as u32))]
        #[pallet::call_index(5)]
        pub fn allow_venues(
            origin: OriginFor<T>,
            asset_id: AssetId,
            venues: Vec<VenueId>,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
            let next_venue_id = VenueCounter::<T>::get();
            for venue in &venues {
                ensure!(venue < &next_venue_id, Error::<T>::InvalidVenue);
                VenueAllowList::<T>::insert(&asset_id, venue, true);
            }
            Self::deposit_event(Event::VenuesAllowed(did, asset_id, venues));
            Ok(())
        }

        /// Revokes permission given to venues for creating instructions involving a particular asset.
        ///
        /// * `asset_id` - AssetId of the token in question.
        /// * `venues` - Array of venues that are no longer allowed to create instructions for the token in question.
        ///
        /// # Permissions
        /// * Asset
        #[pallet::weight(<T as Config>::WeightInfo::disallow_venues(venues.len() as u32))]
        #[pallet::call_index(6)]
        pub fn disallow_venues(
            origin: OriginFor<T>,
            asset_id: AssetId,
            venues: Vec<VenueId>,
        ) -> DispatchResult {
            let did = <ExternalAgents<T>>::ensure_perms(origin, &asset_id)?;
            let next_venue_id = VenueCounter::<T>::get();
            for venue in &venues {
                ensure!(venue < &next_venue_id, Error::<T>::InvalidVenue);
                VenueAllowList::<T>::remove(&asset_id, venue);
            }
            Self::deposit_event(Event::VenuesBlocked(did, asset_id, venues));
            Ok(())
        }

        /// Edit a venue's signers.
        /// * `id` specifies the ID of the venue to edit.
        /// * `signers` specifies the signers to add/remove.
        /// * `add_signers` specifies the update type add/remove of venue where add is true and remove is false.
        #[pallet::weight(<T as Config>::WeightInfo::update_venue_signers(signers.len() as u32))]
        #[pallet::call_index(7)]
        pub fn update_venue_signers(
            origin: OriginFor<T>,
            id: VenueId,
            signers: BTreeSet<T::AccountId>,
            add_signers: bool,
        ) -> DispatchResult {
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;

            Self::base_update_venue_signers(did, id, signers, add_signers)?;
            Ok(())
        }

        /// Manually executes an instruction.
        ///
        /// # Arguments
        /// * `id`: The [`InstructionId`] of the instruction to be executed.
        /// * `asset_holder`:  One of the caller's [`AssetHolder`] which is also a counter patry in the instruction.
        /// If None, the caller must be the venue creator or a counter party in a [`Leg::OffChain`].
        /// * `fungible_transfers`: The number of fungible legs in the instruction.
        /// * `nfts_transfers`: The number of nfts being transferred in the instruction.
        /// * `offchain_transfers`: The number of offchain legs in the instruction.
        /// * `weight_limit`: An optional maximum [`Weight`] value to be charged for executing the instruction.
        /// If the `weight_limit` is less than the required amount, the instruction will fail execution.
        ///
        /// Note: calling the rpc method `get_execute_instruction_info` returns an instance of [`ExecuteInstructionInfo`], which contains the count parameters.
        #[pallet::weight(<T as Config>::WeightInfo::execute_manual_weight_limit(weight_limit, fungible_transfers, nfts_transfers, offchain_transfers))]
        #[pallet::call_index(8)]
        pub fn execute_manual_instruction(
            origin: OriginFor<T>,
            id: InstructionId,
            asset_holder: Option<AssetHolder>,
            fungible_transfers: u32,
            nfts_transfers: u32,
            offchain_transfers: u32,
            weight_limit: Option<Weight>,
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_manual_instruction_minimum_weight(),
                weight_limit.unwrap_or_else(|| {
                    <T as Config>::WeightInfo::execute_manual_instruction(
                        fungible_transfers,
                        nfts_transfers,
                        offchain_transfers,
                    )
                }),
            )?;

            let input_cost =
                AssetCount::new(fungible_transfers, nfts_transfers, offchain_transfers);

            Self::base_manual_execution(
                origin,
                id,
                asset_holder.as_ref(),
                &input_cost,
                false,
                &mut weight_meter,
            )
            .map_err(|e| DispatchErrorWithPostInfo {
                post_info: Some(weight_meter.consumed()).into(),
                error: e.error,
            })
        }

        /// Adds a new instruction.
        ///
        /// # Arguments
        /// * `venue_id`: The optional [`VenueId`] of the venue this instruction belongs to.
        /// * `settlement_type`: The [`SettlementType`] specifying when the instruction should be settled.
        /// * `trade_date`: Optional date from which people can interact with this instruction.
        /// * `value_date`: Optional date after which the instruction should be settled (not enforced).
        /// * `legs`: A vector of all [`Leg`] included in this instruction.
        /// * `memo`: An optional [`Memo`] field for this instruction.
        #[pallet::weight(<T as Config>::WeightInfo::add_instruction_legs(legs))]
        #[pallet::call_index(9)]
        pub fn add_instruction(
            origin: OriginFor<T>,
            venue_id: Option<VenueId>,
            settlement_type: SettlementType<BlockNumberFor<T>>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
            instruction_memo: Option<Memo>,
        ) -> DispatchResult {
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
            Self::base_add_instruction(
                did,
                venue_id,
                settlement_type,
                trade_date,
                value_date,
                legs,
                instruction_memo,
                None,
            )?;
            Ok(())
        }

        /// Adds and affirms a new instruction.
        ///
        /// # Arguments
        /// * `venue_id`: The [`VenueId`] of the venue this instruction belongs to.
        /// * `settlement_type`: The [`SettlementType`] specifying when the instruction should be settled.
        /// * `trade_date`: Optional date from which people can interact with this instruction.
        /// * `value_date`: Optional date after which the instruction should be settled (not enforced).
        /// * `legs`: A vector of all [`Leg`] included in this instruction.
        /// * `holder_set`: A set of [`AssetHolder`] under the caller's control and intended for affirmation.
        /// * `memo`: An optional [`Memo`] field for this instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::add_and_affirm_instruction_legs(legs, holder_set.len() as u32))]
        #[pallet::call_index(10)]
        pub fn add_and_affirm_instruction(
            origin: OriginFor<T>,
            venue_id: Option<VenueId>,
            settlement_type: SettlementType<BlockNumberFor<T>>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
            holder_set: BoundedBTreeSet<AssetHolder, T::MaxNumberOfAssetHolders>,
            instruction_memo: Option<Memo>,
        ) -> DispatchResult {
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin.clone())?;
            let instruction_id = Self::base_add_instruction(
                did,
                venue_id,
                settlement_type,
                trade_date,
                value_date,
                legs,
                instruction_memo,
                None,
            )?;
            Self::affirm_and_maybe_schedule_instruction(
                origin,
                instruction_id,
                holder_set.into_inner(),
                None,
            )
            .map_err(|e| e.error)?;
            Ok(())
        }

        /// Provide affirmation to an existing instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being affirmed.
        /// * `holder_set` - a set of [`AssetHolder`] under the caller's control and intended for affirmation.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::affirm_instruction_input(None, holder_set.len() as u32))]
        #[pallet::call_index(11)]
        pub fn affirm_instruction(
            origin: OriginFor<T>,
            id: InstructionId,
            holder_set: BoundedBTreeSet<AssetHolder, T::MaxNumberOfAssetHolders>,
        ) -> DispatchResultWithPostInfo {
            Self::affirm_and_maybe_schedule_instruction(origin, id, holder_set.into_inner(), None)
        }

        /// Rejects an existing instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being rejected.
        /// * `asset_holder` - the [`AssetHolder`] that belongs to the instruction and is rejecting it.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::reject_instruction(None))]
        #[pallet::call_index(13)]
        pub fn reject_instruction(
            origin: OriginFor<T>,
            id: InstructionId,
            asset_holder: AssetHolder,
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::reject_instruction_minimum_weight(),
                <T as Config>::WeightInfo::reject_instruction(Some(AssetCount::new(10, 100, 10))),
            )?;
            Self::base_reject_instruction(
                origin,
                id,
                Some(asset_holder),
                None,
                false,
                &mut weight_meter,
            )
        }

        /// Root callable extrinsic, used as an internal call to execute a scheduled settlement instruction.
        #[pallet::weight((*weight_limit).max(<T as Config>::WeightInfo::execute_scheduled_instruction(0, 0, 0)))]
        #[pallet::call_index(14)]
        pub fn execute_scheduled_instruction(
            origin: OriginFor<T>,
            id: InstructionId,
            weight_limit: Weight,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_root_origin(origin)?;
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_scheduled_instruction_minimum_weight(),
                weight_limit,
            )?;
            Ok(Self::base_execute_scheduled_instruction(
                id,
                &mut weight_meter,
            ))
        }

        /// Affirms an instruction using receipts for offchain transfers.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being affirmed.
        /// * `receipt_details` - a vector of [`ReceiptDetails`], which contain the details about the offchain transfer.
        /// * `holder_set` - a vector of [`AssetHolder`] under the caller's control and intended for affirmation.
        /// * `number_of_assets` - an optional [`AffirmationCount`] that will be used for a precise fee estimation before executing the extrinsic.
        ///
        /// Note: calling the rpc method `get_affirmation_count` returns an instance of [`AffirmationCount`].
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::affirm_with_receipts_input(*number_of_assets, holder_set.len() as u32))]
        #[pallet::call_index(15)]
        pub fn affirm_with_receipts_with_count(
            origin: OriginFor<T>,
            id: InstructionId,
            receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature, T::Moment>>,
            holder_set: BoundedBTreeSet<AssetHolder, T::MaxNumberOfAssetHolders>,
            number_of_assets: Option<AffirmationCount>,
        ) -> DispatchResult {
            Self::affirm_with_receipts_and_maybe_schedule_instruction(
                origin,
                id,
                receipt_details,
                holder_set.into_inner(),
                number_of_assets,
            )
            .map_err(|e| e.error)?;
            Ok(())
        }

        /// Provide affirmation to an existing instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being affirmed.
        /// * `holder_set` - a vector of [`AssetHolder`] under the caller's control and intended for affirmation.
        /// * `number_of_assets` - an optional [`AffirmationCount`] that will be used for a precise fee estimation before executing the extrinsic.
        ///
        /// Note: calling the rpc method `get_affirmation_count` returns an instance of [`AffirmationCount`].
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::affirm_instruction_input(*number_of_assets, holder_set.len() as u32))]
        #[pallet::call_index(16)]
        pub fn affirm_instruction_with_count(
            origin: OriginFor<T>,
            id: InstructionId,
            holder_set: BoundedBTreeSet<AssetHolder, T::MaxNumberOfAssetHolders>,
            number_of_assets: Option<AffirmationCount>,
        ) -> DispatchResult {
            Self::affirm_and_maybe_schedule_instruction(
                origin,
                id,
                holder_set.into_inner(),
                number_of_assets,
            )
            .map_err(|e| e.error)?;
            Ok(())
        }

        /// Rejects an existing instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being rejected.
        /// * `asset_holder` - the [`AssetHolder`] that belongs to the instruction and is rejecting it.
        /// * `number_of_assets` - an optional [`AssetCount`] that will be used for a precise fee estimation before executing the extrinsic.
        ///
        /// Note: calling the rpc method `get_execute_instruction_info` returns an instance of [`ExecuteInstructionInfo`], which contain the asset count.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::reject_instruction(*number_of_assets))]
        #[pallet::call_index(17)]
        pub fn reject_instruction_with_count(
            origin: OriginFor<T>,
            id: InstructionId,
            asset_holder: AssetHolder,
            number_of_assets: Option<AssetCount>,
        ) -> DispatchResult {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::reject_instruction_minimum_weight(),
                <T as Config>::WeightInfo::reject_instruction(number_of_assets),
            )
            .map_err(|e| e.error)?;

            Self::base_reject_instruction(
                origin,
                id,
                Some(asset_holder),
                number_of_assets,
                false,
                &mut weight_meter,
            )
            .map_err(|e| e.error)?;

            Ok(())
        }

        /// Adds a new instruction with mediators.
        ///
        /// # Arguments
        /// * `venue_id`: The [`VenueId`] of the venue this instruction belongs to.
        /// * `settlement_type`: The [`SettlementType`] specifying when the instruction should be settled.
        /// * `trade_date`: Optional date from which people can interact with this instruction.
        /// * `value_date`: Optional date after which the instruction should be settled (not enforced).
        /// * `legs`: A vector of all [`Leg`] included in this instruction.
        /// * `instruction_memo`: An optional [`Memo`] field for this instruction.
        /// * `mediators`: A set of [`IdentityId`] of all the mandatory mediators for the instruction.
        #[pallet::weight(<T as Config>::WeightInfo::add_instruction_with_mediators_legs(legs, mediators.len() as u32))]
        #[pallet::call_index(19)]
        pub fn add_instruction_with_mediators(
            origin: OriginFor<T>,
            venue_id: Option<VenueId>,
            settlement_type: SettlementType<BlockNumberFor<T>>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
            instruction_memo: Option<Memo>,
            mediators: BoundedBTreeSet<IdentityId, T::MaxInstructionMediators>,
        ) -> DispatchResult {
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
            Self::base_add_instruction(
                did,
                venue_id,
                settlement_type,
                trade_date,
                value_date,
                legs,
                instruction_memo,
                Some(mediators),
            )?;
            Ok(())
        }

        /// Adds and affirms a new instruction with mediators.
        ///
        /// # Arguments
        /// * `venue_id`: The [`VenueId`] of the venue this instruction belongs to.
        /// * `settlement_type`: The [`SettlementType`] specifying when the instruction should be settled.
        /// * `trade_date`: Optional date from which people can interact with this instruction.
        /// * `value_date`: Optional date after which the instruction should be settled (not enforced).
        /// * `legs`: A vector of all [`Leg`] included in this instruction.
        /// * `holder_set`: A set of [`AssetHolder`] under the caller's control and intended for affirmation.
        /// * `instruction_memo`: An optional [`Memo`] field for this instruction.
        /// * `mediators`: A set of [`IdentityId`] of all the mandatory mediators for the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::add_and_affirm_with_mediators_legs(legs, holder_set.len() as u32, mediators.len() as u32))]
        #[pallet::call_index(20)]
        pub fn add_and_affirm_with_mediators(
            origin: OriginFor<T>,
            venue_id: Option<VenueId>,
            settlement_type: SettlementType<BlockNumberFor<T>>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
            holder_set: BoundedBTreeSet<AssetHolder, T::MaxNumberOfAssetHolders>,
            instruction_memo: Option<Memo>,
            mediators: BoundedBTreeSet<IdentityId, T::MaxInstructionMediators>,
        ) -> DispatchResult {
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin.clone())?;
            let instruction_id = Self::base_add_instruction(
                did,
                venue_id,
                settlement_type,
                trade_date,
                value_date,
                legs,
                instruction_memo,
                Some(mediators),
            )?;
            Self::affirm_and_maybe_schedule_instruction(
                origin,
                instruction_id,
                holder_set.into_inner(),
                None,
            )
            .map_err(|e| e.error)?;
            Ok(())
        }

        /// Affirms the instruction as a mediator - should only be called by mediators, otherwise it will fail.
        ///
        /// # Arguments
        /// * `origin`: The secondary key of the sender.
        /// * `instruction_id`: The [`InstructionId`] that will be affirmed by the mediator.
        /// * `expiry`: An Optional value for defining when the affirmation will expire (None means it will always be valid).
        #[pallet::weight(<T as Config>::WeightInfo::affirm_instruction_as_mediator())]
        #[pallet::call_index(21)]
        pub fn affirm_instruction_as_mediator(
            origin: OriginFor<T>,
            instruction_id: InstructionId,
            expiry: Option<T::Moment>,
        ) -> DispatchResult {
            Self::base_affirm_instruction_as_mediator(origin, instruction_id, expiry)?;
            Ok(())
        }

        /// Rejects an existing instruction - should only be called by mediators, otherwise it will fail.
        ///
        /// # Arguments
        /// * `instruction_id` - the [`InstructionId`] of the instruction being rejected.
        /// * `number_of_assets` - an optional [`AssetCount`] that will be used for a precise fee estimation before executing the extrinsic.
        ///
        /// Note: calling the rpc method `get_execute_instruction_info` returns an instance of [`ExecuteInstructionInfo`], which contain the asset count.
        #[pallet::weight(<T as Config>::WeightInfo::reject_instruction(*number_of_assets))]
        #[pallet::call_index(23)]
        pub fn reject_instruction_as_mediator(
            origin: OriginFor<T>,
            instruction_id: InstructionId,
            number_of_assets: Option<AssetCount>,
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::reject_instruction_minimum_weight(),
                <T as Config>::WeightInfo::reject_instruction(number_of_assets),
            )?;

            Self::base_reject_instruction(
                origin,
                instruction_id,
                None,
                number_of_assets,
                false,
                &mut weight_meter,
            )
        }

        /// Moves the instruction status to `LockedForExecution`. This function must be called by a
        /// mediator of the instruction and will only suceed if the following conditions are met:
        /// - All affirmations have been received.
        /// - Instruction is pending or has failed at least one time.
        /// - All mediator's affirmations are still valid.
        /// - All assets are in the allowed venue list.
        /// - All senders have the right amount of assets being transferred.
        /// - All senders and receivers are compliant and have valid CDD claims.
        /// - All assets' statistics are still valid.
        /// - There are no frozen assets.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, specifying the caller.
        /// * `inst_id` - The [`InstructionId`] of the instruction to be locked.
        /// * `weight_limit` - A maximum [`Weight`] value to be charged for locking the instruction.
        #[pallet::weight(<T as Config>::WeightInfo::lock_instruction(*weight_limit))]
        #[pallet::call_index(24)]
        pub fn lock_instruction(
            origin: OriginFor<T>,
            inst_id: InstructionId,
            weight_limit: Weight,
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::lock_instruction_minimum_weight(),
                weight_limit,
            )?;

            Self::base_lock_instruction(origin, inst_id, false, &mut weight_meter)?;

            Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
        }

        /// Sets whether the caller's identity requires mandatory receiver affirmation for incoming transfers.
        ///
        /// When `require` is `true`, the caller's identity must explicitly affirm any incoming asset transfer.
        /// When `require` is `false` (default), incoming transfers are auto-affirmed.
        ///
        /// # Events
        /// * `MandatoryReceiverAffirmationSet` - When the mandatory receiver affirmation flag is updated.
        #[pallet::call_index(25)]
        #[pallet::weight(<T as Config>::WeightInfo::set_mandatory_receiver_affirmation())]
        pub fn set_mandatory_receiver_affirmation(
            origin: OriginFor<T>,
            requirement: AffirmationRequirement,
        ) -> DispatchResult {
            let caller_did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
            if requirement == AffirmationRequirement::Required {
                MandatoryReceiverAffirmation::<T>::insert(&caller_did, true);
            } else {
                MandatoryReceiverAffirmation::<T>::remove(&caller_did);
            }
            Self::deposit_event(Event::MandatoryReceiverAffirmationSet(
                caller_did,
                requirement,
            ));
            Ok(())
        }

        /// Transfer assets between accounts and portfolios.
        ///
        /// Currently supports two modes:
        /// - Direct (owner, same-identity): `from` and `to` resolve to the same DID.
        ///   Transfers immediately via `base_transfer` — no settlement instruction, no affirmation.
        /// - Direct (spender): Caller differs from source owner. Spender-approval mode.
        ///   Allowance is checked and decremented. Spender mode is only available for
        ///   `AssetHolder::Account` sources with fungible funds.
        ///
        /// When `from` is `None`, defaults to `AssetHolder::Account(caller)`.
        ///
        /// # Spender-mode allowance behavior
        /// - Finite allowance: decremented by transfer amount. Removed when depleted to zero.
        /// - Unlimited allowance (`Balance::MAX`): never decremented, no storage write.
        /// - No `Approval` event emitted on spend. Use the `allowance` Runtime API to query
        ///   remaining allowance.
        ///
        /// # Arguments
        /// * `origin` — Signed origin. Caller must have a registered DID.
        /// * `from` — Source. `None` defaults to caller's account. When set to a different
        ///   account, spender-approval mode activates.
        /// * `to` — Destination account or portfolio.
        /// * `fund` — Asset and amount (fungible) or NFT IDs (non-fungible), plus optional memo.
        #[pallet::call_index(26)]
        #[pallet::weight(Pallet::<T>::transfer_funds_weight_limit(from.as_ref(), &fund))]
        pub fn transfer_funds(
            origin: OriginFor<T>,
            from: Option<AssetHolder>,
            to: AssetHolder,
            fund: Fund,
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = WeightMeter::from_limit_unchecked(
                Weight::zero(),
                Self::transfer_funds_weight_limit(from.as_ref(), &fund),
            );

            Self::base_transfer_funds(
                origin,
                from,
                to,
                fund,
                &mut weight_meter,
                #[cfg(feature = "runtime-benchmarks")]
                false,
            )?;
            Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
        }

        /// Unlocks an instruction that is currently in `LockedForExecution` status,
        /// moving it back to `Pending`. Only a mediator of the instruction can call this.
        ///
        /// After unlocking, the mediator must wait at least [`Config::RelockCooldown`] before
        /// locking the instruction again. This gives other parties time to reject the
        /// instruction if they wish to back out.
        ///
        /// # Arguments
        /// * `origin` - The origin of the call, must be a mediator of the instruction.
        /// * `inst_id` - The [`InstructionId`] of the instruction to unlock.
        #[pallet::call_index(27)]
        #[pallet::weight(<T as Config>::WeightInfo::unlock_instruction())]
        pub fn unlock_instruction(origin: OriginFor<T>, inst_id: InstructionId) -> DispatchResult {
            Self::base_unlock_instruction(origin, inst_id)
        }
    }
}

impl<T: Config> Pallet<T> {
    fn base_transfer_funds(
        origin: T::RuntimeOrigin,
        from: Option<AssetHolder>,
        to: AssetHolder,
        fund: Fund,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> Result<Option<InstructionId>, DispatchError> {
        let origin_data =
            pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin.clone())?;
        let origin_did = origin_data.primary_did;

        // Resolve source: None defaults to caller's account.
        let resolved_from = match from {
            Some(holder) => holder,
            None => AssetHolder::try_from(origin_data.sender.encode())
                .map_err(|_| Error::<T>::InvalidAccountId)?,
        };

        // Self-transfer guard.
        ensure!(resolved_from != to, Error::<T>::SenderSameAsReceiver);

        // Resolve DIDs and determine transfer path.
        let from_did = pallet_identity::Pallet::<T>::asset_holder_did(&resolved_from)?;
        let to_did = pallet_identity::Pallet::<T>::asset_holder_did(&to)?;

        let same_did = from_did == to_did;

        // Charge the benchmark-measured base weight for this path.
        // Compliance/statistics cost is charged dynamically through the meter when it runs.
        Self::check_accrue(
            weight_meter,
            Self::transfer_funds_actual_weight(&resolved_from, same_did, &fund),
        )?;

        let instruction_id = if same_did {
            // Authorize: spender allowance (account) or custody (portfolio).
            Self::ensure_transfer_source_authorized(
                &resolved_from,
                &origin_data,
                &fund,
                weight_meter,
            )?;
            match fund.description {
                FundDescription::Fungible { asset_id, amount } => {
                    ensure!(amount > 0, Error::<T>::ZeroAmount);
                    Asset::<T>::ensure_asset_is_not_frozen(&asset_id)?;
                    Asset::<T>::ensure_sufficient_balance(
                        &resolved_from,
                        &asset_id,
                        amount,
                        false,
                    )?;
                    Asset::<T>::transfer_holders_balance(
                        resolved_from.clone(),
                        to.clone(),
                        asset_id,
                        amount,
                    )?;
                }
                FundDescription::NonFungible(ref nfts) => {
                    ensure!(nfts.len() > 0, Error::<T>::ZeroAmount);
                    pallet_asset::Pallet::<T>::ensure_holder_is_not_frozen(
                        &resolved_from,
                        nfts.asset_id(),
                    )?;
                    Asset::<T>::ensure_asset_is_not_frozen(nfts.asset_id())?;
                    Nft::<T>::ensure_nft_ownership(&resolved_from, nfts)?;
                    Nft::<T>::transfer_holders_nfts(&resolved_from, to.clone(), nfts)?;
                }
            }
            Self::deposit_event(Event::FundsTransferred(
                origin_did,
                resolved_from,
                to,
                fund.clone(),
            ));
            None
        } else {
            // Cross-identity: authorize and create settlement instruction.
            Self::base_transfer_and_try_execute(
                origin,
                &origin_data,
                resolved_from,
                to,
                fund,
                weight_meter,
                #[cfg(feature = "runtime-benchmarks")]
                bench_base_weight,
            )?
        };

        Ok(instruction_id)
    }

    /// Authorize the transfer source.
    ///
    /// - Account source where caller != owner: checks and consumes the spender's approval.
    ///   Fungible funds use the `pallet_asset` allowance; NFTs use the `pallet_nft` per-token or
    ///   collection-wide operator approval.
    /// - Portfolio source: checks custody.
    fn ensure_transfer_source_authorized(
        resolved_from: &AssetHolder,
        caller_data: &pallet_identity::PermissionedCallOriginData<T::AccountId>,
        fund: &Fund,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        match resolved_from {
            AssetHolder::Account(ref owner) => {
                let owner_acc = pallet_base::pallet_account_id::<T>(owner)?;
                if owner_acc != caller_data.sender {
                    match &fund.description {
                        FundDescription::Fungible { asset_id, amount } => {
                            Self::check_accrue(weight_meter, T::AssetFn::spend_allowance_weight())?;
                            Asset::<T>::spend_allowance(
                                &owner_acc,
                                &caller_data.sender,
                                *asset_id,
                                *amount,
                            )?;
                        }
                        FundDescription::NonFungible(nfts) => {
                            Self::check_accrue(
                                weight_meter,
                                <T as pallet_nft::Config>::WeightInfo::spend_nft_approval(
                                    nfts.len() as u32,
                                ),
                            )?;
                            Nft::<T>::spend_nft_approval(owner, &caller_data.sender, nfts)?;
                        }
                    }
                }
            }
            AssetHolder::Portfolio(ref portfolio_id) => {
                T::PortfolioFn::ensure_portfolio_custody_and_permission(
                    portfolio_id,
                    caller_data.primary_did,
                    caller_data.secondary_key.as_ref(),
                )?;
            }
        }
        Ok(())
    }

    fn lock_asset(leg: &Leg) -> DispatchResult {
        match leg {
            Leg::Fungible {
                sender,
                asset_id,
                amount,
                ..
            } => Asset::<T>::add_locked_balance(sender.clone(), *asset_id, *amount),
            Leg::NonFungible { sender, nfts, .. } => {
                for nft_id in nfts.ids() {
                    Nft::<T>::lock_nft(sender.clone(), *nfts.asset_id(), *nft_id)?;
                }
                Ok(())
            }
            Leg::OffChain { .. } => Err(Error::<T>::OffChainAssetCantBeLocked.into()),
        }
    }

    fn unlock_asset(leg: &Leg) -> DispatchResult {
        match leg {
            Leg::Fungible {
                sender,
                asset_id,
                amount,
                ..
            } => Asset::<T>::remove_locked_balance(sender.clone(), *asset_id, *amount),
            Leg::NonFungible { sender, nfts, .. } => {
                for nft_id in nfts.ids() {
                    Nft::<T>::unlock_nft(&sender, nfts.asset_id(), nft_id)?;
                }
                Ok(())
            }
            Leg::OffChain { .. } => Err(Error::<T>::OffChainAssetCantBeLocked.into()),
        }
    }

    /// Ensure origin call permission and the given instruction validity.
    fn ensure_origin_perm_and_instruction_validity(
        origin: OriginFor<T>,
        id: InstructionId,
        is_execute: bool,
    ) -> EnsureValidInstructionResult<T::AccountId, T::Moment, BlockNumberFor<T>> {
        let origin_data = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
        Ok((
            origin_data.primary_did,
            origin_data.secondary_key,
            Self::ensure_instruction_validity(id, is_execute)?,
        ))
    }

    /// Returns `Ok(Venue)` if `venue_id` was created by `did`.
    fn ensure_venue_creator(venue_id: &VenueId, did: &IdentityId) -> Result<Venue, DispatchError> {
        let venue = VenueInfo::<T>::get(venue_id).ok_or(Error::<T>::InvalidVenue)?;
        ensure!(&venue.creator == did, Error::<T>::Unauthorized);
        Ok(venue)
    }

    pub fn base_add_instruction(
        did: IdentityId,
        venue_id: Option<VenueId>,
        settlement_type: SettlementType<BlockNumberFor<T>>,
        trade_date: Option<T::Moment>,
        value_date: Option<T::Moment>,
        legs: Vec<Leg>,
        memo: Option<Memo>,
        mediators: Option<BoundedBTreeSet<IdentityId, T::MaxInstructionMediators>>,
    ) -> Result<InstructionId, DispatchError> {
        match settlement_type {
            SettlementType::SettleOnBlock(block_number) => {
                ensure!(
                    block_number > System::<T>::block_number(),
                    Error::<T>::SettleOnPastBlock
                );
            }
            SettlementType::SettleAfterLock => {
                ensure!(
                    !mediators
                        .as_ref()
                        .ok_or(Error::<T>::MissingInstructionMediators)?
                        .is_empty(),
                    Error::<T>::MissingInstructionMediators
                );
            }
            SettlementType::SettleOnAffirmation | SettlementType::SettleManual(_) => {}
        }

        // Ensure that instruction dates are valid.
        if let (Some(trade_date), Some(value_date)) = (trade_date, value_date) {
            ensure!(
                value_date >= trade_date,
                Error::<T>::InstructionDatesInvalid
            );
        }

        // Ensure venue exists & sender is its creator.
        if let Some(venue_id) = venue_id {
            Self::ensure_venue_creator(&venue_id, &did)?;
        }

        // Verifies if all legs are valid.
        let mut instruction_info = Self::ensure_valid_legs(&legs, &venue_id)?;

        // Adds the instruction mediators
        if let Some(mediators) = mediators {
            instruction_info.extend_mediators(mediators.into())
        }

        // Advance and get next `instruction_id`.
        let instruction_id = InstructionCounter::<T>::try_mutate(try_next_post::<T, _>)?;

        // All checks have been made - Write data to storage.
        InstructionStatuses::<T>::insert(instruction_id, InstructionStatus::Pending);

        for asset_holder in instruction_info.holders_pending_approval() {
            UserAffirmations::<T>::insert(asset_holder, instruction_id, AffirmationStatus::Pending);
        }

        for mediator_id in instruction_info.mediators() {
            InstructionMediatorsAffirmations::<T>::insert(
                instruction_id,
                mediator_id,
                MediatorAffirmationStatus::Pending,
            );
        }
        InstructionAffirmsPending::<T>::insert(
            instruction_id,
            instruction_info.number_of_pending_affirmations(),
        );

        legs.iter().enumerate().for_each(|(index, leg)| {
            let leg_id = LegId(index as u64);
            InstructionLegs::<T>::insert(instruction_id, leg_id, leg.clone());
            if leg.is_off_chain() {
                OffChainAffirmations::<T>::insert(
                    instruction_id,
                    leg_id,
                    AffirmationStatus::Pending,
                );
            }
        });

        InstructionDetails::<T>::insert(
            instruction_id,
            Instruction {
                instruction_id,
                venue_id,
                settlement_type,
                created_at: Some(<pallet_timestamp::Pallet<T>>::get()),
                trade_date,
                value_date,
            },
        );
        if let Some(ref memo) = memo {
            InstructionMemos::<T>::insert(instruction_id, &memo);
        }
        if let Some(venue_id) = venue_id {
            VenueInstructions::<T>::insert(venue_id, instruction_id, ());
        }

        Self::deposit_event(Event::InstructionCreated(
            did,
            venue_id,
            instruction_id,
            settlement_type,
            trade_date,
            value_date,
            legs,
            memo,
        ));

        for asset_holder in instruction_info.holders_pre_approved_difference() {
            UserAffirmations::<T>::insert(
                asset_holder,
                instruction_id,
                AffirmationStatus::Affirmed,
            );
            AffirmsReceived::<T>::insert(instruction_id, asset_holder, AffirmationStatus::Affirmed);
            Self::deposit_event(Event::InstructionAutomaticallyAffirmed(
                did,
                asset_holder.clone(),
                instruction_id,
            ));
        }

        if !instruction_info.mediators().is_empty() {
            Self::deposit_event(Event::InstructionMediators(
                instruction_id,
                instruction_info.mediators().clone(),
            ));
        }

        if let SettlementType::SettleOnBlock(block_number) = settlement_type {
            let weight_limit = Self::execute_scheduled_instruction_weight_limit(
                instruction_info.fungible_transfers(),
                instruction_info.nfts_transferred(),
                instruction_info.off_chain(),
            );
            Self::schedule_instruction(instruction_id, block_number, weight_limit)?;
        }

        Ok(instruction_id)
    }

    /// Returns [`InstructionInfo`] if all legs are valid, otherwise returns an error.
    /// See also: [`Pallet::ensure_valid_fungible_leg`], [`Pallet::ensure_valid_nft_leg`] and [`Pallet::ensure_valid_off_chain_leg`].
    fn ensure_valid_legs(
        legs: &[Leg],
        venue_id: &Option<VenueId>,
    ) -> Result<InstructionInfo, DispatchError> {
        // Tracks the number of fungible, non-fungible and offchain assets across the legs
        let mut instruction_asset_count = AssetCount::default();
        // Tracks all asset holders that have not been pre-affirmed
        let mut holders_pending_approval = BTreeSet::new();
        // Tracks all asset holders that have pre-approved the transfer.
        let mut holders_pre_approved = BTreeSet::new();
        // Tracks all mediators that have to affirm the instruction.
        let mut mediators = BTreeSet::new();
        // Tracks all tickers that have been checked for filtering
        let mut tickers = BTreeSet::new();

        // Validates all legs and checks if they have been pre-affirmed
        for leg in legs {
            Self::ensure_valid_leg(leg, venue_id, &mut tickers, &mut instruction_asset_count)?;

            let (asset_id, sender, receiver) = {
                match leg {
                    Leg::Fungible {
                        sender,
                        receiver,
                        asset_id,
                        ..
                    } => (asset_id, sender, receiver),
                    Leg::NonFungible {
                        sender,
                        receiver,
                        nfts,
                    } => (nfts.asset_id(), sender, receiver),
                    Leg::OffChain { .. } => continue,
                }
            };

            Asset::<T>::ensure_valid_holder(&sender)?;
            Asset::<T>::ensure_valid_holder(&receiver)?;

            holders_pending_approval.insert(sender.clone());
            if Asset::<T>::skip_asset_holder_affirmation(receiver, asset_id)? {
                holders_pre_approved.insert(receiver.clone());
            } else {
                holders_pending_approval.insert(receiver.clone());
            }

            let asset_mediators = MandatoryMediators::<T>::get(asset_id);
            mediators.extend(asset_mediators.iter());
        }
        // The maximum number of each asset type in one instruction is checked here
        Self::ensure_within_instruction_max(&instruction_asset_count)?;

        Ok(InstructionInfo::new(
            instruction_asset_count,
            holders_pending_approval,
            holders_pre_approved,
            mediators,
        ))
    }

    fn ensure_instruction_validity(
        id: InstructionId,
        is_execute: bool,
    ) -> Result<Instruction<T::Moment, BlockNumberFor<T>>, DispatchError> {
        let details = InstructionDetails::<T>::get(id);
        ensure!(
            InstructionStatuses::<T>::get(id) != InstructionStatus::Unknown,
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

    /// Executes the instruction of the given `id`. If the execution succeeds, the instruction gets pruned,
    /// otherwise the instruction status is set to failed.
    fn execute_instruction_retryable(
        id: InstructionId,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
        skip_base_charge: bool,
    ) -> DispatchResult {
        if let Err(e) = Self::execute_instruction(id, caller_did, weight_meter, skip_base_charge) {
            InstructionStatuses::<T>::insert(id, InstructionStatus::Failed);
            return Err(e);
        }
        Ok(())
    }

    fn execute_instruction(
        inst_id: InstructionId,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
        skip_base_charge: bool,
    ) -> DispatchResult {
        // The order of execution of the legs matter in some edge cases around compliance
        let mut inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(&inst_id).collect();
        inst_legs.sort_by_key(|leg| leg.0);
        let inst_asset_count = AssetCount::from_legs(&inst_legs);

        // Manual executions charge the weight in advance
        if !skip_base_charge {
            weight_meter
                .check_accrue(<T as Config>::WeightInfo::execute_instruction_paused(
                    inst_asset_count.fungible(),
                    inst_asset_count.non_fungible(),
                    inst_asset_count.off_chain(),
                ))
                .map_err(|_| Error::<T>::WeightLimitExceeded)?;
        }

        Self::validate_execute_instruction_pre_conditions(&inst_id, &inst_legs, false)?;
        let inst_memo = InstructionMemos::<T>::get(&inst_id);

        let mut failed_leg_id = None;
        let tx_result = with_transaction(|| {
            Self::release_locks(&inst_id, &inst_legs)?;
            if let Err(leg_id) =
                Self::transfer_assets(inst_id, &inst_legs, inst_memo, caller_did, weight_meter)
            {
                failed_leg_id = Some(leg_id);
                return Err(Error::<T>::FailedAssetTransferringConditions.into());
            }
            Self::prune_instruction(&inst_id, &inst_legs)?;
            Self::deposit_event(Event::InstructionExecuted(caller_did, inst_id));
            InstructionStatuses::<T>::insert(
                inst_id,
                InstructionStatus::Success(System::<T>::block_number()),
            );
            Ok(())
        });

        // Since with_transaction reverts events as well, the event has to be emitted here
        if let Some(failed_leg_id) = failed_leg_id {
            Self::deposit_event(Event::LegFailedExecution(
                caller_did,
                inst_id,
                failed_leg_id,
            ));
        }

        tx_result
    }

    /// Returns `Ok` if the following conditions for executing the instruction are met:
    /// - Instruction is pending or has failed at least one time
    /// - All affirmations have been received
    /// - All mediator's affirmations are still valid
    /// - All assets are in the allowed venue list
    fn validate_execute_instruction_pre_conditions(
        inst_id: &InstructionId,
        inst_legs: &[(LegId, Leg)],
        allow_locked_inst: bool,
    ) -> DispatchResult {
        Self::ensure_instruction_is_pending_or_failed(inst_id, allow_locked_inst)?;

        ensure!(
            InstructionAffirmsPending::<T>::get(inst_id) == 0,
            Error::<T>::NotAllAffirmationsHaveBeenReceived
        );

        Self::validate_mediators_affirmations(inst_id)?;

        Self::validate_parties_affirmations(inst_id, inst_legs)?;

        let inst_details = InstructionDetails::<T>::get(inst_id);
        Self::ensure_allowed_venue(inst_legs, inst_details.venue_id)?;

        Ok(())
    }

    /// Returns `Ok` if the instruction status is `Pending` or `Failed`. If `allow_locked_inst`
    /// `LockedForExecution` is also allowed.
    fn ensure_instruction_is_pending_or_failed(
        inst_id: &InstructionId,
        allow_locked_inst: bool,
    ) -> DispatchResult {
        let inst_status = InstructionStatuses::<T>::get(inst_id);

        if allow_locked_inst {
            ensure!(
                inst_status == InstructionStatus::Pending
                    || inst_status == InstructionStatus::Failed
                    || inst_status == InstructionStatus::LockedForExecution,
                Error::<T>::InvalidInstructionStatusForExecution
            );
        } else {
            ensure!(
                inst_status == InstructionStatus::Pending
                    || inst_status == InstructionStatus::Failed,
                Error::<T>::InvalidInstructionStatusForExecution
            );
        }

        Ok(())
    }

    /// Returns `Ok` if all mediator's affirmation are still valid.
    fn validate_mediators_affirmations(inst_id: &InstructionId) -> DispatchResult {
        let current_timestamp = <pallet_timestamp::Pallet<T>>::get();

        for mediator_affirmation in
            InstructionMediatorsAffirmations::<T>::iter_prefix_values(inst_id)
        {
            match mediator_affirmation {
                MediatorAffirmationStatus::Affirmed { expiry, .. } => {
                    if let Some(expiry) = expiry {
                        ensure!(
                            expiry > current_timestamp,
                            Error::<T>::MediatorAffirmationExpired
                        );
                    }
                }
                MediatorAffirmationStatus::Unknown | MediatorAffirmationStatus::Pending => {
                    return Err(Error::<T>::NotAllAffirmationsHaveBeenReceived.into())
                }
            }
        }

        Ok(())
    }

    /// Returns `Ok` if all affirmations have been received.
    #[rustfmt::skip]
    fn validate_parties_affirmations(
        inst_id: &InstructionId,
        inst_legs: &[(LegId, Leg)],
    ) -> DispatchResult {
        let mut unique_portfolios = BTreeSet::new();

        for (leg_id, leg) in inst_legs {
            match leg {
                Leg::Fungible { sender, receiver, .. }
                | Leg::NonFungible { sender, receiver, .. } => {
                    ensure!(
                        InstructionLegStatus::<T>::get(inst_id, leg_id)
                            == LegStatus::ExecutionPending,
                        Error::<T>::UnexpectedLegStatus
                    );

                    if unique_portfolios.insert(sender) {
                        let sdr_affirmation_status = UserAffirmations::<T>::get(sender, inst_id);
                        ensure!(
                            sdr_affirmation_status == AffirmationStatus::Affirmed,
                            Error::<T>::NotAllAffirmationsHaveBeenReceived
                        );
                        let sdr_affirmation_status = AffirmsReceived::<T>::get(inst_id, sender);
                        ensure!(
                            sdr_affirmation_status == AffirmationStatus::Affirmed,
                            Error::<T>::NotAllAffirmationsHaveBeenReceived
                        );
                    }

                    if unique_portfolios.insert(receiver) {
                        let rcv_affirmation_status = UserAffirmations::<T>::get(receiver, inst_id);
                        ensure!(
                            rcv_affirmation_status == AffirmationStatus::Affirmed,
                            Error::<T>::NotAllAffirmationsHaveBeenReceived
                        );
                        let rcv_affirmation_status = AffirmsReceived::<T>::get(inst_id, receiver);
                        ensure!(
                            rcv_affirmation_status == AffirmationStatus::Affirmed,
                            Error::<T>::NotAllAffirmationsHaveBeenReceived
                        );
                    }
                }
                Leg::OffChain { .. } => {
                    match InstructionLegStatus::<T>::get(inst_id, leg_id) {
                        LegStatus::ExecutionToBeSkipped(_, _) => {
                            ensure!(
                                OffChainAffirmations::<T>::get(inst_id, leg_id)
                                    == AffirmationStatus::Affirmed,
                                Error::<T>::NotAllAffirmationsHaveBeenReceived,
                            );
                        }
                        LegStatus::PendingTokenLock | LegStatus::ExecutionPending => {
                            return Err(Error::<T>::UnexpectedLegStatus.into());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[rustfmt::skip]
    fn transfer_assets(
        inst_id: InstructionId,
        inst_legs: &[(LegId, Leg)],
        inst_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> Result<(), LegId> {
        for (leg_id, leg) in inst_legs {
            match leg {
                Leg::Fungible { sender, receiver, asset_id, amount } => {
                    if Asset::<T>::base_transfer(
                        sender.clone(),
                        receiver.clone(),
                        *asset_id,
                        *amount,
                        Some(inst_id),
                        inst_memo.clone(),
                        caller_did,
                        weight_meter,
                    )
                    .is_err()
                    {
                        return Err(*leg_id);
                    }
                }
                Leg::NonFungible { sender, receiver, nfts } => {
                    if Nft::<T>::base_nft_transfer(
                        sender.clone(),
                        receiver.clone(),
                        nfts.clone(),
                        inst_id,
                        inst_memo.clone(),
                        caller_did,
                        weight_meter,
                    )
                    .is_err()
                    {
                        return Err(*leg_id);
                    }
                }
                Leg::OffChain { .. } => {}
            }
        }

        Ok(())
    }

    /// Removes the following storage related to the instruction:
    /// - `InstructionDetails`
    /// - `VenueInstructions`
    /// - `InstructionAffirmsPending`
    /// - `InstructionMediatorsAffirmations`
    /// - `InstructionLegStatus`
    /// - `UserAffirmations`
    /// - `AffirmsReceived`
    /// - `OffChainAffirmations`
    /// - `LockedTimestamp`
    #[rustfmt::skip]
    fn prune_instruction(
        inst_id: &InstructionId,
        inst_legs: &[(LegId, Leg)],
    ) -> DispatchResult {
        let instruction_details = InstructionDetails::<T>::take(&inst_id);

        if let Some(venue_id) = instruction_details.venue_id {
            VenueInstructions::<T>::remove(venue_id, inst_id);
        }

        InstructionAffirmsPending::<T>::remove(inst_id);
        LockedTimestamp::<T>::remove(inst_id);
        UnlockedTimestamp::<T>::remove(inst_id);
        InstructionRelockCount::<T>::remove(inst_id);

        let _ = InstructionMediatorsAffirmations::<T>::clear_prefix(inst_id, u32::MAX, None);

        for (leg_id, leg) in inst_legs {
            match leg {
                Leg::Fungible { sender, receiver, .. }
                | Leg::NonFungible { sender, receiver, .. } => {
                    UserAffirmations::<T>::remove(sender, inst_id);
                    UserAffirmations::<T>::remove(receiver, inst_id);
                    AffirmsReceived::<T>::remove(inst_id, sender);
                    AffirmsReceived::<T>::remove(inst_id, receiver);
                    InstructionLegStatus::<T>::remove(inst_id, leg_id);
                    InstructionLegs::<T>::remove(inst_id, leg_id);
                }
                Leg::OffChain { .. } => {
                    OffChainAffirmations::<T>::remove(inst_id, leg_id);
                    InstructionLegStatus::<T>::remove(inst_id, leg_id);
                    InstructionLegs::<T>::remove(inst_id, leg_id);
                }
            }
        }

        Ok(())
    }

    fn release_locks(id: &InstructionId, instruction_legs: &[(LegId, Leg)]) -> DispatchResult {
        for (leg_id, leg) in instruction_legs {
            if let LegStatus::ExecutionPending = InstructionLegStatus::<T>::get(id, leg_id) {
                Self::unlock_asset(&leg)?;
            }
        }
        Ok(())
    }

    /// Schedule a given instruction to be executed on the next block only if the
    /// settlement type is `SettleOnAffirmation` and no. of affirms pending is 0.
    fn maybe_schedule_instruction(
        affirms_pending: u64,
        id: InstructionId,
        weight_limit: Weight,
    ) -> DispatchResult {
        if affirms_pending == 0
            && InstructionDetails::<T>::get(id).settlement_type
                == SettlementType::SettleOnAffirmation
        {
            // Schedule instruction to be executed in the next block.
            let execution_at = System::<T>::block_number() + One::one();
            Self::schedule_instruction(id, execution_at, weight_limit)?;
        }
        Ok(())
    }

    /// Schedule execution of given instruction at given block number.
    ///
    /// NB - It is expected to execute the given instruction into the given block number but
    /// it is not a guaranteed behavior, Scheduler may have other high priority task scheduled
    /// for the given block so there are chances where the instruction execution block no. may drift.
    pub(crate) fn schedule_instruction(
        id: InstructionId,
        execution_at: BlockNumberFor<T>,
        weight_limit: Weight,
    ) -> DispatchResult {
        let scheduler_call =
            <T as pallet::Config>::SchedulerCall::from(Call::<T>::execute_scheduled_instruction {
                id,
                weight_limit,
            });

        let execute_inst_call = <T as pallet::Config>::SchedulerPreimage::bound(scheduler_call)?;

        let task_name = id.execution_name();

        T::Scheduler::schedule_named(
            task_name,
            DispatchTime::At(execution_at),
            None,
            SETTLEMENT_INSTRUCTION_EXECUTION_PRIORITY,
            RawOrigin::Root.into(),
            execute_inst_call,
        )
        .map_err(|_| Error::<T>::FailedToSchedule)?;

        Ok(())
    }

    /// Affirms all legs from the instruction of the given `instruction_id`, where `portfolios` are a counter party.
    /// If the portfolio is the sender, the asset is also locked.
    pub fn base_affirm_with_receipts(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        receipts_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature, T::Moment>>,
        holder_set: BTreeSet<AssetHolder>,
        affirmation_count: Option<AffirmationCount>,
    ) -> Result<FilteredLegs, DispatchError> {
        ensure!(
            receipts_details.len() <= T::MaxNumberOfOffChainAssets::get() as usize,
            Error::<T>::MaxNumberOfReceiptsExceeded
        );

        let (caller_did, secondary_key, instruction_details) =
            Self::ensure_origin_perm_and_instruction_validity(origin, instruction_id, false)?;

        // The settlement must have a venue to use off-chain receipts.
        let venue_id = instruction_details
            .venue_id
            .ok_or(Error::<T>::OffChainAssetsMustHaveAVenue)?;

        Self::caller_is_permissioned_and_affirmation_is_pending(
            caller_did,
            secondary_key.as_ref(),
            &holder_set,
            &instruction_id,
        )?;

        Self::ensure_valid_receipts_details(venue_id, instruction_id, &receipts_details)?;

        // Lock tokens for all legs that are not of type [`Leg::OffChain`]
        let filtered_legs = Self::filtered_legs(instruction_id, &holder_set);
        // If the fee was estimated in advance, the input values must be at least equal to the actual values
        if let Some(affirmation_count) = affirmation_count {
            Self::ensure_valid_affirmation_count(&filtered_legs, &affirmation_count)?
        }
        for (leg_id, leg) in filtered_legs.sender_subset() {
            Self::lock_asset(&leg)?;
            InstructionLegStatus::<T>::insert(instruction_id, leg_id, LegStatus::ExecutionPending);
        }

        // Casting is safe since `Self::ensure_portfolios_and_affirmation_status` is called
        let affirms_pending = InstructionAffirmsPending::<T>::get(instruction_id)
            .saturating_sub(holder_set.len() as u64)
            .saturating_sub(receipts_details.len() as u64);
        InstructionAffirmsPending::<T>::insert(instruction_id, affirms_pending);

        // Update storage
        for receipt_detail in receipts_details {
            InstructionLegStatus::<T>::insert(
                instruction_id,
                receipt_detail.leg_id(),
                LegStatus::ExecutionToBeSkipped(
                    receipt_detail.signer().clone(),
                    receipt_detail.uid(),
                ),
            );
            OffChainAffirmations::<T>::insert(
                instruction_id,
                receipt_detail.leg_id(),
                AffirmationStatus::Affirmed,
            );
            ReceiptsUsed::<T>::insert(receipt_detail.signer(), receipt_detail.uid(), true);
            Self::deposit_event(Event::ReceiptClaimed(
                caller_did,
                instruction_id,
                receipt_detail.leg_id(),
                receipt_detail.uid(),
                receipt_detail.signer().clone(),
                receipt_detail.metadata().clone(),
            ));
        }

        for asset_holder in holder_set {
            UserAffirmations::<T>::insert(
                &asset_holder,
                instruction_id,
                AffirmationStatus::Affirmed,
            );
            AffirmsReceived::<T>::insert(
                instruction_id,
                &asset_holder,
                AffirmationStatus::Affirmed,
            );
            Self::deposit_event(Event::InstructionAffirmed(
                caller_did,
                asset_holder,
                instruction_id,
            ));
        }

        Ok(filtered_legs)
    }

    pub fn base_affirm_instruction(
        origin: OriginFor<T>,
        inst_id: InstructionId,
        holder_set: BTreeSet<AssetHolder>,
        affirmation_count: Option<AffirmationCount>,
    ) -> Result<FilteredLegs, DispatchError> {
        let (caller_did, sk, _) =
            Self::ensure_origin_perm_and_instruction_validity(origin, inst_id, false)?;

        Self::caller_is_permissioned_and_affirmation_is_pending(
            caller_did,
            sk.as_ref(),
            &holder_set,
            &inst_id,
        )?;

        Self::unverified_affirm_instruction(caller_did, inst_id, holder_set, affirmation_count)
    }

    // Checks that the caller has permission to affirm the instruction and that the affirmation status is pending.
    pub fn caller_is_permissioned_and_affirmation_is_pending(
        caller_did: IdentityId,
        sk: Option<&SecondaryKey<T::AccountId>>,
        holder_set: &BTreeSet<AssetHolder>,
        inst_id: &InstructionId,
    ) -> DispatchResult {
        // The caller must have permission to affirm the instruction and the affirmation status must be pending
        for asset_holder in holder_set {
            Asset::<T>::ensure_holder_permissions(asset_holder, caller_did, sk)?;
            ensure!(
                UserAffirmations::<T>::get(asset_holder, inst_id) == AffirmationStatus::Pending,
                Error::<T>::UnexpectedAffirmationStatus
            );
        }

        Ok(())
    }

    pub fn unverified_affirm_instruction(
        caller_did: IdentityId,
        inst_id: InstructionId,
        holder_set: BTreeSet<AssetHolder>,
        affirmation_count: Option<AffirmationCount>,
    ) -> Result<FilteredLegs, DispatchError> {
        let filtered_legs = Self::filtered_legs(inst_id, &holder_set);

        // If the fee was estimated in advance, the input values must be at least equal to the actual values
        if let Some(affirmation_count) = affirmation_count {
            Self::ensure_valid_affirmation_count(&filtered_legs, &affirmation_count)?
        }

        for (leg_id, leg) in filtered_legs.sender_subset() {
            Self::lock_asset(&leg)?;
            InstructionLegStatus::<T>::insert(inst_id, leg_id, LegStatus::ExecutionPending);
        }

        let affirms_pending = InstructionAffirmsPending::<T>::get(inst_id);
        let n_holders = holder_set.len();

        for asset_holder in holder_set {
            UserAffirmations::<T>::insert(&asset_holder, inst_id, AffirmationStatus::Affirmed);
            AffirmsReceived::<T>::insert(inst_id, &asset_holder, AffirmationStatus::Affirmed);
            Self::deposit_event(Event::InstructionAffirmed(
                caller_did,
                asset_holder,
                inst_id,
            ));
        }

        InstructionAffirmsPending::<T>::insert(
            inst_id,
            affirms_pending.saturating_sub(u64::try_from(n_holders).unwrap_or_default()),
        );

        Ok(filtered_legs)
    }

    /// Affirms all legs from the instruction of the given `id`, where `portfolios` are a counter party.
    /// If the portfolio is the sender, the asset is also locked. If all affirmation have been received and
    /// the settlement type is [`SettlementType::SettleOnAffirmation`] the instruction will be scheduled for
    /// the next block.
    pub fn affirm_with_receipts_and_maybe_schedule_instruction(
        origin: OriginFor<T>,
        id: InstructionId,
        receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature, T::Moment>>,
        holder_set: BTreeSet<AssetHolder>,
        affirmation_count: Option<AffirmationCount>,
    ) -> DispatchResultWithPostInfo {
        let filtered_legs = Self::base_affirm_with_receipts(
            origin,
            id,
            receipt_details,
            holder_set,
            affirmation_count,
        )?;
        let instruction_asset_count = filtered_legs.unfiltered_asset_count();
        let weight_limit = Self::execute_scheduled_instruction_weight_limit(
            instruction_asset_count.fungible(),
            instruction_asset_count.non_fungible(),
            instruction_asset_count.off_chain(),
        );
        // Schedule instruction to be executed in the next block (expected) if conditions are met.
        Self::maybe_schedule_instruction(
            InstructionAffirmsPending::<T>::get(id),
            id,
            weight_limit,
        )?;
        Ok(PostDispatchInfo::from(Some(
            Self::affirm_with_receipts_actual_weight(
                filtered_legs.sender_asset_count().clone(),
                filtered_legs.receiver_asset_count().clone(),
                filtered_legs.unfiltered_asset_count().off_chain(),
            ),
        )))
    }

    /// Affirms all legs from the instruction of the given `id`, where `portfolios` are a counter party.
    /// If the portfolio is the sender, the asset is also locked. If all affirmation have been received and
    /// the settlement type is [`SettlementType::SettleOnAffirmation`] the instruction will be scheduled for
    /// the next block.
    pub fn affirm_and_maybe_schedule_instruction(
        origin: OriginFor<T>,
        id: InstructionId,
        holder_set: BTreeSet<AssetHolder>,
        affirmation_count: Option<AffirmationCount>,
    ) -> DispatchResultWithPostInfo {
        let filtered_legs =
            Self::base_affirm_instruction(origin, id, holder_set, affirmation_count)?;
        let instruction_asset_count = filtered_legs.unfiltered_asset_count();
        let weight_limit = Self::execute_scheduled_instruction_weight_limit(
            instruction_asset_count.fungible(),
            instruction_asset_count.non_fungible(),
            instruction_asset_count.off_chain(),
        );
        // Schedule the instruction if conditions are met
        Self::maybe_schedule_instruction(
            InstructionAffirmsPending::<T>::get(id),
            id,
            weight_limit,
        )?;
        Ok(PostDispatchInfo::from(Some(
            Self::affirm_instruction_actual_weight(
                filtered_legs.sender_asset_count().clone(),
                filtered_legs.receiver_asset_count().clone(),
            ),
        )))
    }

    /// Affirm with or without receipts, executing the instruction when all affirmations have been received.
    ///
    /// NB - Use this function only in the STO pallet to support DVP settlements.
    pub fn affirm_and_execute_instruction(
        origin: OriginFor<T>,
        id: InstructionId,
        receipt: Option<ReceiptDetails<T::AccountId, T::OffChainSignature, T::Moment>>,
        holder_set: BTreeSet<AssetHolder>,
        caller_did: IdentityId,
    ) -> DispatchResult {
        match receipt {
            Some(receipt) => {
                Self::base_affirm_with_receipts(origin, id, vec![receipt], holder_set, None)?
            }
            None => Self::base_affirm_instruction(origin, id, holder_set, None)?,
        };
        Self::execute_settle_on_affirmation_instruction(
            id,
            InstructionAffirmsPending::<T>::get(id),
            InstructionDetails::<T>::get(id).settlement_type,
            caller_did,
        )?;
        Ok(())
    }

    fn execute_settle_on_affirmation_instruction(
        id: InstructionId,
        affirms_pending: u64,
        settlement_type: SettlementType<BlockNumberFor<T>>,
        caller_did: IdentityId,
    ) -> DispatchResult {
        // We assume `settlement_type == SettleOnAffirmation`,
        // to be defensive, however, this is checked before instruction execution.
        if settlement_type == SettlementType::SettleOnAffirmation && affirms_pending == 0 {
            // We use execute_instruction here directly
            // and not the execute_instruction_retryable variant
            // because direct settlement is not retryable.
            Self::execute_instruction(
                id,
                caller_did,
                &mut WeightMeter::max_limit_no_minimum(),
                true,
            )?;
        }
        Ok(())
    }

    /// Returns [`FilteredLegs`] where the orginal set is all legs in the instruction of the given
    /// `id` and the subset of legs are all legs where the sender is in the given `AssetHolder` set.
    fn filtered_legs(id: InstructionId, holder_set: &BTreeSet<AssetHolder>) -> FilteredLegs {
        let instruction_legs: Vec<(LegId, Leg)> = InstructionLegs::<T>::iter_prefix(&id).collect();
        FilteredLegs::filter_sender(instruction_legs, holder_set)
    }

    /// Returns the [`AssetCount`] for the given `inst_id`.
    pub fn instruction_asset_count(inst_id: &InstructionId) -> AssetCount {
        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(inst_id).collect();
        AssetCount::from_legs(&inst_legs)
    }

    fn base_update_venue_signers(
        did: IdentityId,
        venue_id: VenueId,
        signers: BTreeSet<T::AccountId>,
        add_signers: bool,
    ) -> DispatchResult {
        // Ensure venue exists & sender is its creator.
        Self::ensure_venue_creator(&venue_id, &did)?;

        if add_signers {
            let current_number_of_signers = NumberOfVenueSigners::<T>::get(venue_id);
            ensure!(
                (current_number_of_signers as usize).saturating_add(signers.len())
                    <= T::MaxNumberOfVenueSigners::get() as usize,
                Error::<T>::NumberOfVenueSignersExceeded
            );
            for signer in &signers {
                ensure!(
                    !VenueSigners::<T>::get(&venue_id, &signer),
                    Error::<T>::SignerAlreadyExists
                );
            }
            NumberOfVenueSigners::<T>::insert(
                venue_id,
                current_number_of_signers + signers.len() as u32,
            );
            for signer in &signers {
                <VenueSigners<T>>::insert(&venue_id, &signer, true);
            }
        } else {
            for signer in &signers {
                ensure!(
                    VenueSigners::<T>::get(&venue_id, &signer),
                    Error::<T>::SignerDoesNotExist
                );
            }
            let current_number_of_signers = NumberOfVenueSigners::<T>::get(venue_id);
            NumberOfVenueSigners::<T>::insert(
                venue_id,
                current_number_of_signers.saturating_sub(signers.len() as u32),
            );
            for signer in &signers {
                <VenueSigners<T>>::remove(&venue_id, &signer);
            }
        }

        Self::deposit_event(Event::VenueSignersUpdated(
            did,
            venue_id,
            signers,
            add_signers,
        ));
        Ok(())
    }

    fn base_reject_instruction(
        origin: OriginFor<T>,
        inst_id: InstructionId,
        asset_holder: Option<AssetHolder>,
        input_asset_count: Option<AssetCount>,
        use_account_holding: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo {
        let origin_data = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
        let caller_did = origin_data.primary_did;

        let caller = {
            if use_account_holding {
                Some(AssetHolder::try_from(origin_data.sender.encode())?)
            } else {
                asset_holder
            }
        };

        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(&inst_id).collect();
        let inst_asset_count = AssetCount::from_legs(&inst_legs);

        if let Some(input_asset_count) = input_asset_count {
            Self::ensure_valid_cost(&inst_asset_count, &input_asset_count)?;
        }

        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::base_reject_instruction(
                inst_asset_count.fungible() as u32,
                inst_asset_count.non_fungible() as u32,
                inst_asset_count.off_chain() as u32,
            ),
        )?;

        let inst_details = InstructionDetails::<T>::get(&inst_id);
        match InstructionStatuses::<T>::get(inst_id) {
            InstructionStatus::Pending | InstructionStatus::Failed => {
                Self::ensure_valid_caller(
                    caller_did,
                    origin_data.secondary_key.as_ref(),
                    caller.as_ref(),
                    inst_details.venue_id,
                    &inst_id,
                    &inst_legs,
                )?;
            }
            InstructionStatus::LockedForExecution => {
                // If the locking perid is exceeded, any party can reject the instruction
                if Self::ensure_maximum_locking_period_not_exceeded(&inst_id).is_err() {
                    Self::ensure_valid_caller(
                        caller_did,
                        origin_data.secondary_key.as_ref(),
                        caller.as_ref(),
                        inst_details.venue_id,
                        &inst_id,
                        &inst_legs,
                    )?;
                } else {
                    ensure!(
                        Self::ensure_mediator(&inst_id, &caller_did).is_ok(),
                        Error::<T>::CallerIsNotAMediator
                    );
                }
            }
            InstructionStatus::Unknown
            | InstructionStatus::Rejected(_)
            | InstructionStatus::Success(_) => {
                return Err(Error::<T>::InvalidInstructionStatusForRejection.into());
            }
        }

        Self::release_locks(&inst_id, &inst_legs)?;

        // Note: ignoring the error here is fine, since the instruction might not be scheduled yet
        let task_name = inst_id.execution_name();
        let _ = T::Scheduler::cancel_named(task_name);

        Self::prune_instruction(&inst_id, &inst_legs)?;
        InstructionStatuses::<T>::insert(
            inst_id,
            InstructionStatus::Rejected(System::<T>::block_number()),
        );

        Self::deposit_event(Event::InstructionRejected(caller_did, inst_id));

        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }

    /// Returns `Ok` if the number of fungible, nonfungible and offchain assets is under the input given by the user.
    fn ensure_valid_cost(real_cost: &AssetCount, input_cost: &AssetCount) -> DispatchResult {
        // Verifies if the number of nfts being transferred is under the limit
        ensure!(
            real_cost.non_fungible() <= input_cost.non_fungible(),
            Error::<T>::NumberOfTransferredNFTsUnderestimated
        );
        // Verifies if the number of fungible transfers is under the limit
        ensure!(
            real_cost.fungible() <= input_cost.fungible(),
            Error::<T>::NumberOfFungibleTransfersUnderestimated
        );
        // Verifies if the number of off-chain assets is under the limit
        ensure!(
            real_cost.off_chain() <= input_cost.off_chain(),
            Error::<T>::NumberOfOffChainTransfersUnderestimated
        );
        Ok(())
    }

    /// Ensures that all tickers in the instruction that have venue filtering enabled are also
    /// in the venue allowed list.
    fn ensure_allowed_venue(
        instruction_legs: &[(LegId, Leg)],
        venue_id: Option<VenueId>,
    ) -> DispatchResult {
        // Avoids reading the storage multiple times for the same asset_id
        let mut tickers: BTreeSet<AssetId> = BTreeSet::new();

        for (_, leg) in instruction_legs {
            if let Some(asset_id) = leg.asset_id() {
                Self::ensure_venue_filtering(&mut tickers, *asset_id, &venue_id)?;
            }
        }
        Ok(())
    }

    /// If `tickers` doesn't contain the given `asset_id` and venue_filtering is enabled, ensures that venue_id is in the allowed list
    fn ensure_venue_filtering(
        tickers: &mut BTreeSet<AssetId>,
        asset_id: AssetId,
        venue_id: &Option<VenueId>,
    ) -> DispatchResult {
        if tickers.insert(asset_id) && VenueFiltering::<T>::get(asset_id) {
            let venue_id = venue_id.ok_or(Error::<T>::UnauthorizedVenue)?;
            ensure!(
                VenueAllowList::<T>::get(asset_id, venue_id),
                Error::<T>::UnauthorizedVenue
            );
        }
        Ok(())
    }

    /// Executes the instruction of the given `id` returning the consumed weight for executing the instruction.
    fn base_execute_scheduled_instruction(
        id: InstructionId,
        weight_meter: &mut WeightMeter,
    ) -> PostDispatchInfo {
        let caller_did = SettlementDID.as_id();
        if let Err(e) = Self::execute_instruction_retryable(id, caller_did, weight_meter, false) {
            Self::deposit_event(Event::FailedToExecuteInstruction(id, e));
        }
        PostDispatchInfo::from(Some(weight_meter.consumed()))
    }

    /// Returns `Ok` if the leg is valid, otherwise returns an error.
    /// See also: [`Pallet::ensure_valid_fungible_leg`], [`Pallet::ensure_valid_nft_leg`] and [`Pallet::ensure_valid_off_chain_leg`].
    fn ensure_valid_leg(
        leg: &Leg,
        venue_id: &Option<VenueId>,
        tickers: &mut BTreeSet<AssetId>,
        instruction_asset_count: &mut AssetCount,
    ) -> DispatchResult {
        match leg {
            Leg::Fungible {
                sender,
                receiver,
                asset_id,
                amount,
            } => {
                let sender_did = pallet_identity::Pallet::<T>::asset_holder_did(sender)?;
                let receiver_did = pallet_identity::Pallet::<T>::asset_holder_did(receiver)?;
                ensure!(sender_did != receiver_did, Error::<T>::SameSenderReceiver);
                Self::ensure_valid_fungible_leg(tickers, *asset_id, *amount, venue_id)?;
                instruction_asset_count
                    .try_add_fungible()
                    .map_err(|_| Error::<T>::MaxNumberOfFungibleAssetsExceeded)?;
                Ok(())
            }
            Leg::NonFungible {
                sender,
                receiver,
                nfts,
            } => {
                let sender_did = pallet_identity::Pallet::<T>::asset_holder_did(sender)?;
                let receiver_did = pallet_identity::Pallet::<T>::asset_holder_did(receiver)?;
                ensure!(sender_did != receiver_did, Error::<T>::SameSenderReceiver);
                Self::ensure_valid_nft_leg(tickers, &nfts, venue_id)?;
                instruction_asset_count
                    .try_add_non_fungible(&nfts)
                    .map_err(|_| Error::<T>::MaxNumberOfNFTsExceeded)?;
                Ok(())
            }
            Leg::OffChain {
                sender_identity,
                receiver_identity,
                amount,
                ..
            } => {
                ensure!(venue_id.is_some(), Error::<T>::OffChainAssetsMustHaveAVenue);
                Self::ensure_valid_off_chain_leg(sender_identity, receiver_identity, *amount)?;
                instruction_asset_count
                    .try_add_off_chain()
                    .map_err(|_| Error::<T>::MaxNumberOfOffChainAssetsExceeded)?;
                Ok(())
            }
        }
    }

    /// Ensures all checks needed for a fungible leg hold. This includes making sure that the `amount` being
    /// transferred is not zero, that `asset_id` exists on chain and that `venue_id` is allowed.
    fn ensure_valid_fungible_leg(
        tickers: &mut BTreeSet<AssetId>,
        asset_id: AssetId,
        amount: Balance,
        venue_id: &Option<VenueId>,
    ) -> DispatchResult {
        ensure!(amount > 0, Error::<T>::ZeroAmount);
        ensure!(
            Self::is_on_chain_asset(&asset_id),
            Error::<T>::UnexpectedOFFChainAsset
        );
        Self::ensure_venue_filtering(tickers, asset_id, venue_id)?;
        Ok(())
    }

    /// Ensures all checks needed for a non fungible leg hold. This includes making sure that the number of NFTs being
    /// transferred is within the defined limits, that there are no duplicate NFTs in the same leg, that `asset_id` exists on chain,
    /// and that `venue_id` is allowed.
    fn ensure_valid_nft_leg(
        tickers: &mut BTreeSet<AssetId>,
        nfts: &NFTs,
        venue_id: &Option<VenueId>,
    ) -> DispatchResult {
        ensure!(
            Self::is_on_chain_asset(nfts.asset_id()),
            Error::<T>::UnexpectedOFFChainAsset
        );
        <Nft<T>>::ensure_within_nfts_transfer_limits(&nfts)?;
        <Nft<T>>::ensure_no_duplicate_nfts(&nfts)?;
        Self::ensure_venue_filtering(tickers, nfts.asset_id().clone(), venue_id)?;
        Ok(())
    }

    /// Ensures all checks needed for an off-chain asset leg hold. This includes making sure that the `amount` being
    /// transferred is not zero, and that `sender_identity` and `receiver_identity` are not the same.
    fn ensure_valid_off_chain_leg(
        sender_identity: &IdentityId,
        receiver_identity: &IdentityId,
        amount: Balance,
    ) -> DispatchResult {
        ensure!(amount > 0, Error::<T>::ZeroAmount);
        ensure!(
            sender_identity != receiver_identity,
            Error::<T>::SameSenderReceiver
        );
        Ok(())
    }

    /// Ensures that the number of fungible, non-fungible and offchain transfers is less or equal
    /// to the maximum allowed in an instruction.
    fn ensure_within_instruction_max(instruction_asset_count: &AssetCount) -> DispatchResult {
        ensure!(
            instruction_asset_count.non_fungible() <= T::MaxNumberOfNFTs::get(),
            Error::<T>::MaxNumberOfNFTsExceeded
        );
        ensure!(
            instruction_asset_count.fungible() <= T::MaxNumberOfFungibleAssets::get(),
            Error::<T>::MaxNumberOfFungibleAssetsExceeded
        );
        ensure!(
            instruction_asset_count.off_chain() <= T::MaxNumberOfOffChainAssets::get(),
            Error::<T>::MaxNumberOfOffChainAssetsExceeded
        );
        Ok(())
    }

    /// Returns true if the asset_id is on-chain and false otherwise.
    fn is_on_chain_asset(asset_id: &AssetId) -> bool {
        pallet_asset::Assets::<T>::contains_key(asset_id)
    }

    fn base_manual_execution(
        origin: OriginFor<T>,
        inst_id: InstructionId,
        caller_holding: Option<&AssetHolder>,
        input_asset_count: &AssetCount,
        skip_caller_check: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo {
        let origin_data = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
        let caller_did = origin_data.primary_did;
        let caller_sk = origin_data.secondary_key.as_ref();

        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(inst_id).collect();
        let inst_asset_count = AssetCount::from_legs(&inst_legs);
        Self::ensure_valid_cost(&inst_asset_count, input_asset_count)?;

        let inst_details = InstructionDetails::<T>::get(&inst_id);

        // RPC don't need to check the caller
        if !skip_caller_check && inst_details.settlement_type != SettlementType::SettleAfterLock {
            Self::ensure_valid_caller(
                caller_did,
                caller_sk,
                caller_holding,
                inst_details.venue_id,
                &inst_id,
                &inst_legs,
            )?;
        }

        match InstructionStatuses::<T>::get(&inst_id) {
            InstructionStatus::Pending => {
                Self::check_accrue(
                    weight_meter,
                    <T as Config>::WeightInfo::execute_manual_instruction_paused(
                        inst_asset_count.fungible() as u32,
                        inst_asset_count.non_fungible() as u32,
                        inst_asset_count.off_chain() as u32,
                    ),
                )?;
                Self::ensure_manual_settlement_type(inst_details.settlement_type)?;
                Self::execute_instruction_retryable(inst_id, caller_did, weight_meter, true)?;
            }
            InstructionStatus::Failed => {
                Self::check_accrue(
                    weight_meter,
                    <T as Config>::WeightInfo::execute_manual_instruction_paused(
                        inst_asset_count.fungible() as u32,
                        inst_asset_count.non_fungible() as u32,
                        inst_asset_count.off_chain() as u32,
                    ),
                )?;
                Self::execute_instruction_retryable(inst_id, caller_did, weight_meter, true)?;
            }
            InstructionStatus::LockedForExecution => {
                Self::check_accrue(
                    weight_meter,
                    <T as Config>::WeightInfo::execute_locked_instruction(
                        inst_asset_count.fungible() as u32,
                        inst_asset_count.non_fungible() as u32,
                        inst_asset_count.off_chain() as u32,
                    ),
                )?;
                if !skip_caller_check {
                    ensure!(
                        Self::ensure_mediator(&inst_id, &caller_did).is_ok(),
                        Error::<T>::CallerIsNotAMediator
                    );
                }
                Self::simplified_asset_transfer(
                    inst_id,
                    inst_legs.clone(),
                    caller_did,
                    weight_meter,
                )?;
                Self::prune_instruction(&inst_id, &inst_legs)?;
            }
            InstructionStatus::Success(_)
            | InstructionStatus::Unknown
            | InstructionStatus::Rejected(_) => {
                return Err(Error::<T>::InvalidInstructionStatusForExecution.into())
            }
        }

        Self::deposit_event(Event::SettlementManuallyExecuted(caller_did, inst_id));
        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }

    /// Returns `Ok` if [`SettlementType::SettleManual`] and the `block_number` is reached.
    fn ensure_manual_settlement_type(
        settlement_type: SettlementType<BlockNumberFor<T>>,
    ) -> DispatchResult {
        if let SettlementType::SettleManual(block_number) = settlement_type {
            ensure!(
                System::<T>::block_number() >= block_number,
                Error::<T>::InstructionSettleBlockNotReached
            );

            return Ok(());
        }

        Err(Error::<T>::UnexpectedSettlementType.into())
    }

    /// Returns `Ok` if `origin` represents the root, otherwise returns an `Err` with the consumed weight for this function.
    fn ensure_root_origin(origin: OriginFor<T>) -> Result<(), DispatchErrorWithPostInfo> {
        ensure_root(origin).map_err(|e| DispatchErrorWithPostInfo {
            post_info: Some(<T as Config>::WeightInfo::ensure_root_origin()).into(),
            error: e.into(),
        })
    }

    /// Ensure a valid venue signer and unused receipt uid.
    /// The function checks that the signer is allowed by the venue, that the receipt has not been used before.
    fn ensure_valid_receipt(venue_id: VenueId, signer: &T::AccountId, uid: u64) -> DispatchResult {
        ensure!(
            VenueSigners::<T>::get(venue_id, signer),
            Error::<T>::UnauthorizedSigner
        );
        ensure!(
            !ReceiptsUsed::<T>::get(signer, &uid),
            Error::<T>::ReceiptAlreadyClaimed
        );
        Ok(())
    }

    /// Mark a receipt as used for a given venue signer.
    pub fn mark_receipt_as_used(
        venue_id: VenueId,
        signer: &T::AccountId,
        uid: u64,
    ) -> DispatchResult {
        // Ensure the receipt is valid.
        Self::ensure_valid_receipt(venue_id, signer, uid)?;

        ReceiptsUsed::<T>::insert(signer, uid, true);
        Ok(())
    }

    /// Ensures the all receipts are valid. A receipt is considered valid if the signer is allowed by the venue,
    /// if the receipt has not been used before, if the receipt's `leg_id` and `instruction_id` are referencing the
    /// correct instruction/leg and if its signature is valid.
    fn ensure_valid_receipts_details(
        venue_id: VenueId,
        instruction_id: InstructionId,
        receipts_details: &[ReceiptDetails<T::AccountId, T::OffChainSignature, T::Moment>],
    ) -> DispatchResult {
        let mut unique_signers_uid_set = BTreeSet::new();
        let mut unique_legs = BTreeSet::new();
        for receipt_details in receipts_details {
            ensure!(
                receipt_details.instruction_id() == &instruction_id,
                Error::<T>::ReceiptInstructionIdMissmatch
            );
            ensure!(
                unique_signers_uid_set
                    .insert((receipt_details.signer().clone(), receipt_details.uid())),
                Error::<T>::DuplicateReceiptUid
            );
            ensure!(
                unique_legs.insert(receipt_details.leg_id()),
                Error::<T>::MultipleReceiptsForOneLeg
            );

            Self::ensure_valid_receipt(venue_id, receipt_details.signer(), receipt_details.uid())?;

            let leg = InstructionLegs::<T>::get(&instruction_id, &receipt_details.leg_id())
                .ok_or(Error::<T>::LegNotFound)?;
            match leg {
                Leg::OffChain {
                    sender_identity,
                    receiver_identity,
                    ticker,
                    amount,
                } => {
                    ensure!(
                        OffChainAffirmations::<T>::get(instruction_id, receipt_details.leg_id())
                            == AffirmationStatus::Pending,
                        Error::<T>::UnexpectedAffirmationStatus
                    );
                    let receipt = ChainScopedMessage::<T, _>::new(
                        receipt_details.uid(),
                        SETTLEMENT_RECEIPT_LABEL,
                        *receipt_details.expires_at(),
                        Receipt::new(
                            instruction_id,
                            receipt_details.leg_id(),
                            sender_identity,
                            receiver_identity,
                            ticker,
                            amount,
                        ),
                    )
                    .ok_or(Error::<T>::ReceiptExpired)?;
                    let signature = receipt_details.signature();
                    ensure!(
                        receipt.verify_signature(receipt_details.signer(), signature),
                        Error::<T>::InvalidSignature
                    );
                }
                Leg::Fungible { .. } | Leg::NonFungible { .. } => {
                    return Err(Error::<T>::ReceiptForInvalidLegType.into())
                }
            }
        }
        Ok(())
    }

    /// Returns [`WeightMeter`] if the provided `weight_limit` is greater than `minimum_weight`, otherwise returns an error.
    fn ensure_valid_weight_meter(
        minimum_weight: Weight,
        weight_limit: Weight,
    ) -> Result<WeightMeter, DispatchErrorWithPostInfo> {
        // Check that the provided weight limit is greater than the minimum required weight
        WeightMeter::with_limit(minimum_weight, weight_limit).ok_or_else(|| {
            DispatchErrorWithPostInfo {
                post_info: Some(minimum_weight).into(),
                error: Error::<T>::InputWeightIsLessThanMinimum.into(),
            }
        })
    }

    /// Returns `Ok` if the number of assets in [`AffirmationCount`] is greater or equal to the actual number of assets.
    fn ensure_valid_affirmation_count(
        filtered_legs: &FilteredLegs,
        affirmation_count: &AffirmationCount,
    ) -> DispatchResult {
        Self::ensure_valid_cost(
            filtered_legs.sender_asset_count(),
            affirmation_count.sender_asset_count(),
        )?;
        Self::ensure_valid_cost(
            filtered_legs.receiver_asset_count(),
            affirmation_count.receiver_asset_count(),
        )?;
        // Verifies if the number of off-chain assets is under the limit
        ensure!(
            filtered_legs.unfiltered_asset_count().off_chain()
                <= affirmation_count.offchain_count(),
            Error::<T>::NumberOfOffChainTransfersUnderestimated
        );
        Ok(())
    }

    /// Affirms the instruction as a mediator.
    fn base_affirm_instruction_as_mediator(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        let (caller_did, _, instruction) =
            Self::ensure_origin_perm_and_instruction_validity(origin, instruction_id, false)?;

        // Verifies if the caller is a mediator
        let mediator_affirmation_status =
            InstructionMediatorsAffirmations::<T>::get(instruction_id, caller_did);
        ensure!(
            mediator_affirmation_status != MediatorAffirmationStatus::Unknown,
            Error::<T>::CallerIsNotAMediator
        );

        // Verifies if the expiry date is in the future
        if let Some(expiry) = expiry {
            ensure!(
                expiry > <pallet_timestamp::Pallet<T>>::get(),
                Error::<T>::InvalidExpiryDate
            );
        }

        // Updates the mediator's affirmation status to affirmed
        InstructionMediatorsAffirmations::<T>::insert(
            instruction_id,
            caller_did,
            MediatorAffirmationStatus::Affirmed { expiry },
        );
        // If the mediator is not reaffirming the instruction, the number of pending affirmation must be updated
        if MediatorAffirmationStatus::Pending == mediator_affirmation_status {
            InstructionAffirmsPending::<T>::mutate(instruction_id, |n| *n = n.saturating_sub(1));
        }
        // If all affirmations have been received, the instruction will be scheduled for the next block
        let n_pending_affirmations = InstructionAffirmsPending::<T>::get(instruction_id);
        if n_pending_affirmations == 0
            && instruction.settlement_type == SettlementType::SettleOnAffirmation
        {
            let instruction_asset_count = Self::instruction_asset_count(&instruction_id);
            let weight_limit = Self::execute_scheduled_instruction_weight_limit(
                instruction_asset_count.fungible(),
                instruction_asset_count.non_fungible(),
                instruction_asset_count.off_chain(),
            );
            Self::maybe_schedule_instruction(n_pending_affirmations, instruction_id, weight_limit)?;
        }

        Self::deposit_event(Event::MediatorAffirmationReceived(
            caller_did,
            instruction_id,
            expiry,
        ));
        Ok(())
    }

    /// Returns `Ok` if any of the following conditions is true:
    /// - The caller has the permission of the given portfolio and that portfolio is a party in the instruction.
    /// - The caller is the venue creator of the instruction.
    /// - The caller is an instruction mediator.
    /// - The caller is a counter party in an offchain leg.
    fn ensure_valid_caller(
        caller_did: IdentityId,
        caller_sk: Option<&SecondaryKey<T::AccountId>>,
        caller_holding: Option<&AssetHolder>,
        venue_id: Option<VenueId>,
        inst_id: &InstructionId,
        inst_legs: &[(LegId, Leg)],
    ) -> DispatchResult {
        if let Some(caller_holding) = caller_holding {
            Asset::<T>::ensure_holder_permissions(caller_holding, caller_did, caller_sk)?;
            Self::ensure_holder_belongs_to_instruction(&inst_id, caller_holding)?;
            return Ok(());
        }

        if let Some(venue_id) = venue_id {
            if Self::ensure_venue_creator(&venue_id, &caller_did).is_ok() {
                return Ok(());
            }
        }

        if Self::ensure_mediator(&inst_id, &caller_did).is_ok() {
            return Ok(());
        }

        if Self::is_offchain_party(&caller_did, inst_legs) {
            return Ok(());
        }

        Err(Error::<T>::CallerIsNotAParty.into())
    }

    /// Returns `Ok` if the `asset_holder` is a party in the instruction of the given `inst_id`.
    fn ensure_holder_belongs_to_instruction(
        inst_id: &InstructionId,
        asset_holder: &AssetHolder,
    ) -> DispatchResult {
        match UserAffirmations::<T>::get(asset_holder, inst_id) {
            AffirmationStatus::Unknown => Err(Error::<T>::CallerIsNotAParty.into()),
            AffirmationStatus::Pending | AffirmationStatus::Affirmed => Ok(()),
        }
    }

    /// Returns `true` if `caller_did` is a party in any offchain leg in the instruction.
    #[rustfmt::skip]
    fn is_offchain_party(caller_did: &IdentityId, inst_legs: &[(LegId, Leg)]) -> bool {
        for (_, leg) in inst_legs {
            if let Leg::OffChain { sender_identity, receiver_identity, .. } = leg {
                if sender_identity == caller_did || receiver_identity == caller_did {
                    return true;
                }
            }
        }
        false
    }

    /// Returns `Ok` if the given `did` is a mediator in the instruction.
    fn ensure_mediator(inst_id: &InstructionId, did: &IdentityId) -> DispatchResult {
        if InstructionMediatorsAffirmations::<T>::get(inst_id, did)
            == MediatorAffirmationStatus::Unknown
        {
            return Err(Error::<T>::CallerIsNotAMediator.into());
        }

        Ok(())
    }

    /// If the caller is a mediator and all conditions for executing the instruction are met, updates the instruction status to `LockedForExecution`.
    fn base_lock_instruction(
        origin: OriginFor<T>,
        inst_id: InstructionId,
        skip_caller_check: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let caller_did = pallet_identity::Pallet::<T>::ensure_perms(origin.clone())?;

        if !skip_caller_check {
            Self::ensure_mediator(&inst_id, &caller_did)?;
        }

        let inst_details = InstructionDetails::<T>::get(&inst_id);
        ensure!(
            inst_details.settlement_type == SettlementType::SettleAfterLock,
            Error::<T>::UnexpectedSettlementType
        );

        let is_relock =
            if InstructionStatuses::<T>::get(inst_id) == InstructionStatus::LockedForExecution {
                // Allow re-lock without explicit unlock if the lock period + cooldown has elapsed.
                let locked_at =
                    LockedTimestamp::<T>::get(inst_id).ok_or(Error::<T>::LockTimestampNotFound)?;
                let now = pallet_timestamp::Pallet::<T>::get();
                let required = T::MaximumLockPeriod::get().saturating_add(T::RelockCooldown::get());
                ensure!(
                    now - locked_at >= required,
                    Error::<T>::InstructionAlreadyLocked
                );
                true
            } else if let Some(unlocked_at) = UnlockedTimestamp::<T>::take(inst_id) {
                // Explicit unlock path: enforce cooldown.
                let now = pallet_timestamp::Pallet::<T>::get();
                ensure!(
                    now - unlocked_at >= T::RelockCooldown::get(),
                    Error::<T>::RelockCooldownNotExpired
                );
                true
            } else {
                false
            };

        if is_relock {
            InstructionRelockCount::<T>::try_mutate(inst_id, |count| -> DispatchResult {
                ensure!(
                    *count < T::MaxRelockCount::get(),
                    Error::<T>::MaxRelockCountExceeded
                );
                *count = count.saturating_add(1);
                Ok(())
            })?;
        }

        // The order of execution of the legs matter in some edge cases around compliance
        let mut inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(&inst_id).collect();
        inst_legs.sort_by_key(|leg| leg.0);
        let inst_asset_count = AssetCount::from_legs(&inst_legs);

        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::lock_instruction_extrinsic(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
            ),
        )?;

        Self::validate_execute_instruction_pre_conditions(&inst_id, &inst_legs, true)?;

        let inst_memo = InstructionMemos::<T>::get(&inst_id);
        frame_support_with_transaction(|| {
            if let Err(e) = Self::release_locks(&inst_id, &inst_legs) {
                return TransactionOutcome::Rollback(Err(e));
            };

            if Self::transfer_assets(inst_id, &inst_legs, inst_memo, caller_did, weight_meter)
                .is_err()
            {
                return TransactionOutcome::Rollback(Err(
                    Error::<T>::FailedAssetTransferringConditions.into(),
                ));
            }

            TransactionOutcome::Rollback(Ok(()))
        })?;

        InstructionStatuses::<T>::insert(inst_id, InstructionStatus::LockedForExecution);
        LockedTimestamp::<T>::insert(inst_id, pallet_timestamp::Pallet::<T>::get());

        Self::deposit_event(Event::InstructionLocked(caller_did, inst_id));
        Ok(())
    }

    /// Unlocks a locked instruction, moving it back to `Pending` status.
    /// Records the unlock timestamp to enforce the relock cooldown.
    fn base_unlock_instruction(origin: OriginFor<T>, inst_id: InstructionId) -> DispatchResult {
        let caller_did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
        Self::ensure_mediator(&inst_id, &caller_did)?;

        ensure!(
            InstructionStatuses::<T>::get(inst_id) == InstructionStatus::LockedForExecution,
            Error::<T>::InstructionNotLocked
        );

        InstructionStatuses::<T>::insert(inst_id, InstructionStatus::Pending);
        LockedTimestamp::<T>::remove(inst_id);
        UnlockedTimestamp::<T>::insert(inst_id, pallet_timestamp::Pallet::<T>::get());

        Self::deposit_event(Event::InstructionUnlocked(caller_did, inst_id));
        Ok(())
    }

    /// Transfer all assets in the instruction. Only the following checks are assessed:
    /// - The locking period must be below the maximum.
    /// - All assets are locked.
    /// - All senders must have the required balance.
    #[rustfmt::skip]
    fn simplified_asset_transfer(
        inst_id: InstructionId,
        inst_legs: Vec<(LegId, Leg)>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::ensure_maximum_locking_period_not_exceeded(&inst_id)?;

        Self::release_locks(&inst_id, &inst_legs)?;

        let inst_memo = InstructionMemos::<T>::get(&inst_id);
        for (_, leg) in inst_legs {
            match leg {
                Leg::Fungible { sender, receiver, asset_id, amount } => {
                    Asset::<T>::simplified_fungible_transfer(
                        asset_id,
                        sender,
                        receiver,
                        amount,
                        inst_id,
                        inst_memo.clone(),
                        caller_did,
                        weight_meter,
                    )?;
                }
                Leg::NonFungible { sender, receiver, nfts } => {
                    Nft::<T>::simplified_nft_transfer(
                        sender,
                        receiver,
                        nfts,
                        Some(inst_id),
                        inst_memo.clone(),
                        caller_did,
                    )?;
                }
                Leg::OffChain { .. } => continue,
            }
        }

        Self::deposit_event(Event::InstructionExecuted(caller_did, inst_id));
        InstructionStatuses::<T>::insert(
            inst_id,
            InstructionStatus::Success(System::<T>::block_number())
        );

        Ok(())
    }

    /// Returns `Ok` if the maximum locking period was not exceeded.
    fn ensure_maximum_locking_period_not_exceeded(inst_id: &InstructionId) -> DispatchResult {
        let locked_timestamp =
            LockedTimestamp::<T>::get(inst_id).ok_or(Error::<T>::LockTimestampNotFound)?;

        let now = pallet_timestamp::Pallet::<T>::get();
        ensure!(
            now - locked_timestamp <= T::MaximumLockPeriod::get(),
            Error::<T>::ExceededMaximumLockingPeriod
        );

        Ok(())
    }

    /// Consumes the given weight after checking that it can be consumed.
    /// Returns an error if the weight limit is exceeded.
    fn check_accrue(weight_meter: &mut WeightMeter, weight: Weight) -> DispatchResult {
        weight_meter
            .check_accrue(weight)
            .map_err(|_| Error::<T>::WeightLimitExceeded)?;
        Ok(())
    }

    /// Returns the worst case weight for an instruction with `f` fungible legs, `n` nfts being transferred and `o` offchain assets.
    fn execute_scheduled_instruction_weight_limit(f: u32, n: u32, o: u32) -> Weight {
        <T as Config>::WeightInfo::execute_scheduled_instruction(f, n, o)
    }

    /// Returns the minimum weight for calling the `execute_scheduled_instruction` function.
    fn execute_scheduled_instruction_minimum_weight() -> Weight {
        <T as Config>::WeightInfo::execute_scheduled_instruction(0, 0, 1)
    }

    /// Returns the minimum weight for calling the `execute_manual_instruction` extrinsic.
    pub fn execute_manual_instruction_minimum_weight() -> Weight {
        <T as Config>::WeightInfo::execute_locked_instruction(0, 0, 1)
    }

    /// Returns the weight for calling `affirm_with_receipts` while considering the `sender_asset_count` for the sender, `receiver_asset_count`
    /// for the receiver, and `n_offchain` offchain legs.
    fn affirm_with_receipts_actual_weight(
        sender_asset_count: AssetCount,
        receiver_asset_count: AssetCount,
        n_offchain: u32,
    ) -> Weight {
        let affirmation_count =
            AffirmationCount::new(sender_asset_count, receiver_asset_count, n_offchain);
        <T as Config>::WeightInfo::affirm_with_receipts_input(Some(affirmation_count), 0)
    }

    /// Returns the weight for calling `affirm_instruction` while considering the `sender_asset_count` for the sender and`receiver_asset_count`
    /// for the receiver.
    fn affirm_instruction_actual_weight(
        sender_asset_count: AssetCount,
        receiver_asset_count: AssetCount,
    ) -> Weight {
        let affirmation_count = AffirmationCount::new(sender_asset_count, receiver_asset_count, 0);
        <T as Config>::WeightInfo::affirm_instruction_input(Some(affirmation_count), 0)
    }

    /// Returns the base benchmark weight for the specific transfer path.
    /// Compliance/statistics cost is charged dynamically by the pallets via the weight meter.
    fn transfer_funds_actual_weight(from: &AssetHolder, same_did: bool, fund: &Fund) -> Weight {
        match (from, same_did, &fund.description) {
            (AssetHolder::Account(_), true, FundDescription::NonFungible(nfts)) => {
                <T as Config>::WeightInfo::transfer_funds_nft_account_same_did(nfts.len() as u32)
            }
            (AssetHolder::Account(_), false, FundDescription::NonFungible(nfts)) => {
                <T as Config>::WeightInfo::transfer_funds_nft_account_diff_did(nfts.len() as u32)
            }
            (AssetHolder::Portfolio(_), true, FundDescription::NonFungible(nfts)) => {
                <T as Config>::WeightInfo::transfer_funds_nft_portfolio_same_did(nfts.len() as u32)
            }
            (AssetHolder::Portfolio(_), false, FundDescription::NonFungible(nfts)) => {
                <T as Config>::WeightInfo::transfer_funds_nft_portfolio_diff_did(nfts.len() as u32)
            }
            (AssetHolder::Account(_), true, FundDescription::Fungible { .. }) => {
                <T as Config>::WeightInfo::transfer_funds_account_same_did()
            }
            (AssetHolder::Account(_), false, FundDescription::Fungible { .. }) => {
                <T as Config>::WeightInfo::transfer_funds_account_diff_did()
            }
            (AssetHolder::Portfolio(_), true, FundDescription::Fungible { .. }) => {
                <T as Config>::WeightInfo::transfer_funds_portfolio_same_did()
            }
            (AssetHolder::Portfolio(_), false, FundDescription::Fungible { .. }) => {
                <T as Config>::WeightInfo::transfer_funds_portfolio_diff_did()
            }
        }
    }

    /// Returns the miminum weight for calling the `reject_instruction` extrinsic.
    fn reject_instruction_minimum_weight() -> Weight {
        <T as Config>::WeightInfo::base_reject_instruction(0, 0, 1)
    }

    /// Returns the minimum weight required for calling the `lock_instruction` extrinsic.
    pub fn lock_instruction_minimum_weight() -> Weight {
        <T as Config>::WeightInfo::lock_instruction_extrinsic(0, 0, 1)
    }

    pub fn get_actual_weight(call: &Call<T>) -> Option<Weight> {
        match call {
            Call::affirm_instruction { id, holder_set } => {
                let filtered_legs = Self::filtered_legs(*id, &holder_set);
                Some(Self::affirm_instruction_actual_weight(
                    *filtered_legs.sender_asset_count(),
                    *filtered_legs.receiver_asset_count(),
                ))
            }
            Call::affirm_with_receipts { id, holder_set, .. } => {
                let filtered_legs = Self::filtered_legs(*id, &holder_set);
                Some(Self::affirm_with_receipts_actual_weight(
                    *filtered_legs.sender_asset_count(),
                    *filtered_legs.receiver_asset_count(),
                    filtered_legs.unfiltered_asset_count().off_chain(),
                ))
            }
            Call::reject_instruction { id, .. } => {
                let inst_asset_count = Self::instruction_asset_count(id);
                Some(<T as Config>::WeightInfo::reject_instruction(Some(
                    inst_asset_count,
                )))
            }
            _ => None,
        }
    }

    /// Returns an instance of [`AffirmationCount`].
    pub fn affirmation_count(
        instruction_id: InstructionId,
        holder_set: Vec<AssetHolder>,
    ) -> AffirmationCount {
        let holder_set = holder_set.into_iter().collect::<BTreeSet<_>>();
        let filtered_legs = Self::filtered_legs(instruction_id, &holder_set);
        AffirmationCount::new(
            filtered_legs.sender_asset_count().clone(),
            filtered_legs.receiver_asset_count().clone(),
            filtered_legs.unfiltered_asset_count().off_chain(),
        )
    }

    /// Returns a vector containing all errors for the transfer. An empty vec means there's no error.
    ///
    /// `Note:` should only be called as a RPC.
    #[rustfmt::skip]
    pub fn transfer_report(
        leg: Leg,
        skip_locked_check: bool,
        weight_meter: &mut WeightMeter,
    ) -> Vec<DispatchError> {
        match leg {
            Leg::Fungible { sender, receiver, asset_id, amount } => {
                <Asset<T>>::asset_transfer_report(
                    &sender,
                    &receiver,
                    &asset_id,
                    amount,
                    skip_locked_check,
                    weight_meter,
                )
            }
            Leg::NonFungible { sender, receiver, nfts } => {
                <Nft<T>>::nft_transfer_report(
                    &sender,
                    &receiver,
                    &nfts,
                    skip_locked_check,
                    weight_meter
                )
            }
            Leg::OffChain { .. } => {
                Vec::new()
            },
        }
    }

    /// Returns a vector containing all errors for the execution. An empty vec means there's no error.
    ///
    /// `Note:` should only be called as a RPC.
    pub fn execute_instruction_report(inst_id: &InstructionId) -> Vec<DispatchError> {
        let mut execution_errors = Vec::new();

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(inst_id).collect();

        if InstructionAffirmsPending::<T>::get(inst_id) != 0 {
            execution_errors.push(Error::<T>::NotAllAffirmationsHaveBeenReceived.into());
        }

        if let Err(e) = Self::ensure_instruction_is_pending_or_failed(inst_id, false) {
            execution_errors.push(e);
        }

        if let Err(e) = Self::validate_mediators_affirmations(inst_id) {
            execution_errors.push(e);
        }

        if let Err(e) = Self::validate_parties_affirmations(inst_id, &inst_legs) {
            execution_errors.push(e);
        }

        let inst_details = InstructionDetails::<T>::get(inst_id);
        if let Err(e) = Self::ensure_allowed_venue(&inst_legs, inst_details.venue_id) {
            execution_errors.push(e);
        }

        for (_, leg) in inst_legs {
            let transfer_errors = Self::transfer_report(leg, true, &mut weight_meter);
            execution_errors.extend_from_slice(&transfer_errors);
        }

        execution_errors
    }

    /// Returns the weight for executing `execute_manual_instruction`.
    ///
    /// `Note:` should only be called as a RPC.
    pub fn manual_execution_weight(inst_id: InstructionId) -> Option<ExecuteInstructionInfo> {
        let mut weight_meter =
            WeightMeter::max_limit(Self::execute_manual_instruction_minimum_weight());

        let caller_did = SettlementDID.as_id();
        let caller_account_id = DidRecords::<T>::get(&caller_did)?.primary_key?;
        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(inst_id).collect();
        let inst_asset_count = AssetCount::from_legs(&inst_legs);

        match Self::base_manual_execution(
            RawOrigin::Signed(caller_account_id).into(),
            inst_id,
            None,
            &inst_asset_count,
            true,
            &mut weight_meter,
        ) {
            Ok(_) => Some(ExecuteInstructionInfo::new(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
                weight_meter.consumed(),
                None,
            )),
            Err(e) => Some(ExecuteInstructionInfo::new(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
                weight_meter.consumed(),
                Some(e.into()),
            )),
        }
    }

    /// Returns the weight for executing `lock_instruction`.
    ///
    /// `Note:` should only be called as a RPC.
    pub fn lock_instruction_weight(inst_id: InstructionId) -> Result<Weight, DispatchError> {
        let mut weight_meter = WeightMeter::max_limit(Self::lock_instruction_minimum_weight());

        let caller_did = SettlementDID.as_id();
        let caller_account_id = DidRecords::<T>::get(&caller_did)
            .ok_or(Error::<T>::Unauthorized)?
            .primary_key
            .ok_or(Error::<T>::Unauthorized)?;

        Self::base_lock_instruction(
            RawOrigin::Signed(caller_account_id).into(),
            inst_id,
            true,
            &mut weight_meter,
        )?;

        Ok(weight_meter.consumed())
    }

    fn asset_count_from_fund(fund: &FundDescription) -> AssetCount {
        match fund {
            FundDescription::Fungible { .. } => AssetCount::new(1, 0, 0),
            FundDescription::NonFungible(nfts) => AssetCount::new(0, nfts.len() as u32, 0),
        }
    }

    /// Attempts to execute an instruction.
    fn base_try_execute_instruction(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        asset_count: AssetCount,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::base_manual_execution(
            origin,
            instruction_id,
            None,
            &asset_count,
            true,
            weight_meter,
        )
        .map_err(|e| e.error)?;
        Ok(())
    }

    /// Initiates a transfer instruction for fungible or non-fungible assets.
    fn base_transfer_and_try_execute(
        origin: OriginFor<T>,
        origin_data: &pallet_identity::PermissionedCallOriginData<T::AccountId>,
        from: AssetHolder,
        to: AssetHolder,
        fund: Fund,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> Result<Option<InstructionId>, DispatchError> {
        // Authorize: spender allowance (account) or custody (portfolio).
        Self::ensure_transfer_source_authorized(&from, origin_data, &fund, weight_meter)?;

        // Prepare the leg depending on whether it's a fungible or non-fungible transfer
        let leg = match &fund.description {
            FundDescription::Fungible { asset_id, amount } => Leg::Fungible {
                sender: from.clone(),
                receiver: to.clone(),
                asset_id: *asset_id,
                amount: *amount,
            },
            FundDescription::NonFungible(nfts) => {
                Nft::<T>::ensure_within_nfts_transfer_limits(nfts)?;
                Leg::NonFungible {
                    sender: from.clone(),
                    receiver: to.clone(),
                    nfts: nfts.clone(),
                }
            }
        };

        #[cfg(feature = "runtime-benchmarks")]
        {
            if bench_base_weight {
                return Ok(Some(InstructionId(0)));
            }
        }

        // `base_add_instruction` writes the memo to `InstructionMemos` only when provided.
        if fund.memo.is_some() {
            Self::check_accrue(
                weight_meter,
                <T as Config>::WeightInfo::set_instruction_memo(),
            )?;
        }

        // Create the instruction with the prepared leg
        let instruction_id = Self::base_add_instruction(
            origin_data.primary_did,
            None,
            SettlementType::SettleManual(System::<T>::block_number()),
            None,
            None,
            vec![leg],
            fund.memo,
            None,
        )?;

        // Affirm the instruction on behalf of the sender.
        Self::unverified_affirm_instruction(
            origin_data.primary_did,
            instruction_id,
            [from].into(),
            None,
        )?;

        // Try affirming if caller is the receiver (spender mode) and receiver affirmation is needed.
        if InstructionAffirmsPending::<T>::get(instruction_id) > 0 {
            // Weight differs based on whether the receiver is an account or a portfolio
            // (different permission-check storage).
            let receiver_affirm_weight = match &to {
                AssetHolder::Account(_) => {
                    <T as Config>::WeightInfo::unsafe_affirm_instruction_receiver_account()
                }
                AssetHolder::Portfolio(_) => {
                    <T as Config>::WeightInfo::unsafe_affirm_instruction_receiver_portfolio()
                }
            };
            Self::check_accrue(weight_meter, receiver_affirm_weight)?;

            let caller_is_permissioned = Self::caller_is_permissioned_and_affirmation_is_pending(
                origin_data.primary_did,
                origin_data.secondary_key.as_ref(),
                &[to.clone()].into(),
                &instruction_id,
            )
            .is_ok();

            // If the caller is not permissioned, the instruction will remain pending until an authorized party affirms it.
            if caller_is_permissioned {
                Self::unverified_affirm_instruction(
                    origin_data.primary_did,
                    instruction_id,
                    [to].into(),
                    None,
                )?;
            }
        }

        let instruction_id = if InstructionAffirmsPending::<T>::get(instruction_id) == 0 {
            // If there are no pending affirmations, execute the instruction immediately.
            let asset_count = Self::asset_count_from_fund(&fund.description);

            Self::base_try_execute_instruction(origin, instruction_id, asset_count, weight_meter)?;

            // The instruction was executed immediately, no need for receiver affirmation.
            None
        } else {
            // The receiver's affirmation is still pending.
            Some(instruction_id)
        };

        Ok(instruction_id)
    }

    /// Receiver affirms the transfer of fungible or non-fungible assets.
    fn base_receiver_affirm_transfer_and_try_execute(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        asset_count: AssetCount,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> DispatchResult {
        let origin_data =
            pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin.clone())?;
        let to = AssetHolder::try_from(origin_data.sender.encode())?;

        // Consume weight for the receiver affirmation
        Self::check_accrue(
            weight_meter,
            Self::receiver_affirm_transfer_weight(asset_count),
        )?;

        // The affirmation count ensures that we are only affirming as the receiver.
        let affirmation_count = AffirmationCount::new(
            // No sender assets to affirm
            AssetCount::default(),
            // One receiver asset to affirm
            asset_count,
            0,
        );

        #[cfg(feature = "runtime-benchmarks")]
        {
            if bench_base_weight {
                // Consume weight for trying to execute the instruction
                Self::check_accrue(weight_meter, Self::try_execute_weight(asset_count))?;
                return Ok(());
            }
        }

        // Affirm the instruction on behalf of the receiver.
        Self::base_affirm_instruction(
            origin.clone(),
            instruction_id,
            [to].into(),
            Some(affirmation_count),
        )?;

        // Try to execute the instruction.
        Self::base_try_execute_instruction(origin, instruction_id, asset_count, weight_meter)?;

        Ok(())
    }
}

impl<T: Config> SettlementFnTrait<T> for Pallet<T> {
    /// Receiver affirms the transfer of fungible or non-fungible assets.
    fn receiver_affirm_transfer_and_try_execute(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        asset_count: AssetCount,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> DispatchResultWithPostInfo {
        Self::base_receiver_affirm_transfer_and_try_execute(
            origin,
            instruction_id,
            asset_count,
            weight_meter,
            #[cfg(feature = "runtime-benchmarks")]
            bench_base_weight,
        )
        .map_err(|error| DispatchErrorWithPostInfo {
            post_info: Some(weight_meter.consumed()).into(),
            error,
        })?;

        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }

    /// Get the try execute weight based on the asset count.
    fn try_execute_weight(asset_count: AssetCount) -> Weight {
        <T as Config>::WeightInfo::execute_manual_instruction(
            asset_count.fungible(),
            asset_count.non_fungible(),
            0,
        )
    }

    /// Get the receiver affirm transfer weight based on the asset count.
    fn receiver_affirm_transfer_weight(asset_count: AssetCount) -> Weight {
        <T as Config>::WeightInfo::affirm_instruction_rcv(
            asset_count.fungible(),
            asset_count.non_fungible(),
        )
    }

    /// Reject a transfer instruction.
    fn reject_transfer(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        asset_count: AssetCount,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo {
        Self::base_reject_instruction(
            origin,
            instruction_id,
            None,
            Some(asset_count),
            true,
            weight_meter,
        )
    }

    /// Get the reject transfer weight meter.
    fn reject_transfer_weight_meter(asset_count: AssetCount) -> WeightMeter {
        WeightMeter::from_limit_unchecked(
            Self::reject_instruction_minimum_weight(),
            <T as Config>::WeightInfo::reject_instruction(Some(asset_count)),
        )
    }

    /// Worst-case weight limit for `transfer_funds`.
    ///
    /// Includes base benchmark + execute/compliance + spender allowance (only when
    /// `from` is `Some(Account)`) + receiver affirmation for the spender-is-receiver path.
    fn transfer_funds_weight_limit(from: Option<&AssetHolder>, fund: &Fund) -> Weight {
        let asset_count = match &fund.description {
            FundDescription::Fungible { .. } => AssetCount::new(1, 0, 0),
            FundDescription::NonFungible(nfts) => AssetCount::new(0, nfts.len() as u32, 0),
        };
        // Base benchmark weight for the specific path (max of same-did vs diff-did).
        let base = match (from, &fund.description) {
            (Some(AssetHolder::Account(_)) | None, FundDescription::NonFungible(_)) => {
                <T as Config>::WeightInfo::transfer_funds_nft_account_same_did(
                    asset_count.non_fungible(),
                )
                .max(
                    <T as Config>::WeightInfo::transfer_funds_nft_account_diff_did(
                        asset_count.non_fungible(),
                    ),
                )
            }
            (Some(AssetHolder::Portfolio(_)), FundDescription::NonFungible(_)) => {
                <T as Config>::WeightInfo::transfer_funds_nft_portfolio_same_did(
                    asset_count.non_fungible(),
                )
                .max(
                    <T as Config>::WeightInfo::transfer_funds_nft_portfolio_diff_did(
                        asset_count.non_fungible(),
                    ),
                )
            }
            (Some(AssetHolder::Account(_)) | None, FundDescription::Fungible { .. }) => {
                <T as Config>::WeightInfo::transfer_funds_account_same_did()
                    .max(<T as Config>::WeightInfo::transfer_funds_account_diff_did())
            }
            (Some(AssetHolder::Portfolio(_)), FundDescription::Fungible { .. }) => {
                <T as Config>::WeightInfo::transfer_funds_portfolio_same_did()
                    .max(<T as Config>::WeightInfo::transfer_funds_portfolio_diff_did())
            }
        };
        // Add worst-case execution + compliance weight.
        let mut limit = base.saturating_add(<T as Config>::WeightInfo::execute_manual_instruction(
            asset_count.fungible(),
            asset_count.non_fungible(),
            0,
        ));
        // Spender approval only applies when caller differs from owner and source is an account.
        // The fungible and non-fungible paths consume different approvals, so charge accordingly.
        if matches!(from, Some(AssetHolder::Account(_))) {
            limit = limit.saturating_add(match &fund.description {
                FundDescription::Fungible { .. } => T::AssetFn::spend_allowance_weight(),
                FundDescription::NonFungible(nfts) => {
                    <T as pallet_nft::Config>::WeightInfo::spend_nft_approval(nfts.len() as u32)
                }
            });
        }
        // Memo write only happens in the cross-DID path when a memo is provided.
        if fund.memo.is_some() {
            limit = limit.saturating_add(<T as Config>::WeightInfo::set_instruction_memo());
        }
        // Spender-is-receiver path: caller is the receiver and affirms inline.
        // `to` isn't available at annotation time, so charge the worst of account vs portfolio.
        limit.saturating_add(
            <T as Config>::WeightInfo::unsafe_affirm_instruction_receiver_account()
                .max(<T as Config>::WeightInfo::unsafe_affirm_instruction_receiver_portfolio()),
        )
    }

    fn transfer_funds(
        origin: OriginFor<T>,
        from: Option<AssetHolder>,
        to: AssetHolder,
        fund: Fund,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> Result<Option<InstructionId>, DispatchError> {
        Self::base_transfer_funds(
            origin,
            from,
            to,
            fund,
            weight_meter,
            #[cfg(feature = "runtime-benchmarks")]
            bench_base_weight,
        )
    }
}

impl<T: Config> AffirmationFnTrait for Pallet<T> {
    fn identity_requires_affirmation(did: &IdentityId) -> bool {
        MandatoryReceiverAffirmation::<T>::get(did)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn set_mandatory_receiver_affirmation(did: IdentityId, policy: AffirmationRequirement) {
        MandatoryReceiverAffirmation::<T>::set(did, policy.into());
    }
}
