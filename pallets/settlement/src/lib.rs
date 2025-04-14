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
//! The Settlement module manages all kinds of transfers and settlements of assets.
//!
//! ## Overview
//!
//! The settlement module provides functionality to settle both on-chain and off-chain trades between multiple parties.
//! All trades are settled under venues. An appropriately permissioned external agent can allow or block certain venues
//! from settling trades that involve their tokens. An atomic settlement is called an Instruction. An instruction can
//! contain multiple legs, which are essentially simple one-to-one transfers. When an instruction is settled, either all
//! legs are executed successfully or none are. In other words, if one of the legs fails due to compliance failure, all
//! other legs will also fail.
//!
//! An instruction must be authorized by all the counter parties involved for it to be executed. An instruction can be
//! set to automatically execute in the next block when all authorizations are received or at a particular block number.
//!
//! Off-chain settlements are represented via receipts. If a leg has a receipt attached to it, it will not be executed
//! on-chain. All other legs will be executed on-chain during settlement.
//!
//! ## Dispatchable Functions
//!
//! - `create_venue` - Registers a new venue.
//! - `update_venue_details` - Updates the details of an existing venue.
//! - `update_venue_type` - Updates the type of an existing venue.
//! - `affirm_with_receipts` - Affirms an instruction using receipts for off-chain transfers.
//! - `set_venue_filtering` - Enables or disables venue filtering for a token.
//! - `allow_venues` - Allows additional venues to create instructions involving an asset.
//! - `disallow_venues` - Revokes permission given to venues for creating instructions involving a particular asset.
//! - `update_venue_signers` - Updates the signers of a venue.
//! - `execute_manual_instruction` - Manually executes an instruction.
//! - `add_instruction` - Adds a new instruction.
//! - `add_and_affirm_instruction` - Adds and affirms a new instruction.
//! - `affirm_instruction` - Provides affirmation to an existing instruction.
//! - `withdraw_affirmation` - Withdraws an affirmation for a given instruction.
//! - `reject_instruction` - Rejects an existing instruction.
//! - `execute_scheduled_instruction` - Executes a scheduled instruction.
//! - `affirm_with_receipts_with_count` - Affirms an instruction using receipts for off-chain transfers with a specified count.
//! - `affirm_instruction_with_count` - Provides affirmation to an existing instruction with a specified count.
//! - `reject_instruction_with_count` - Rejects an existing instruction with a specified count.
//! - `withdraw_affirmation_with_count` - Withdraws an affirmation for a given instruction with a specified count.
//! - `add_instruction_with_mediators` - Adds a new instruction with mediators.
//! - `add_and_affirm_with_mediators` - Adds and affirms a new instruction with mediators.
//! - `affirm_instruction_as_mediator` - Affirms the instruction as a mediator.
//! - `withdraw_affirmation_as_mediator` - Removes the mediator's affirmation for the instruction.
//! - `reject_instruction_as_mediator` - Rejects an existing instruction as a mediator.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::dispatch::{
    DispatchError, DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo,
    PostDispatchInfo,
};
use frame_support::pallet_prelude::*;
use frame_support::traits::schedule::{DispatchTime, Named};
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::{ensure, BoundedBTreeSet};
use frame_system::pallet_prelude::*;
use frame_system::{ensure_root, RawOrigin};
use polymesh_primitives::NFTId;
use sp_runtime::traits::{One, Verify};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::convert::TryFrom;
use sp_std::prelude::*;
use sp_std::vec;

use pallet_asset::{BalanceOf, Frozen, MandatoryMediators};
use pallet_base::{ensure_string_limited, try_next_post};
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::constants::queue_priority::SETTLEMENT_INSTRUCTION_EXECUTION_PRIORITY;
use polymesh_primitives::settlement::{
    AffirmationCount, AffirmationStatus, AssetCount, ExecuteInstructionInfo, FilteredLegs,
    FungibleTxSummary, Instruction, InstructionId, InstructionInfo, InstructionStatus, Leg, LegId,
    LegStatus, MediatorAffirmationStatus, Receipt, ReceiptDetails, ReceiptMetadata, SettlementType,
    Venue, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::traits::ComplianceFnConfig;
use polymesh_primitives::SystematicIssuers::Settlement as SettlementDID;
use polymesh_primitives::{
    storage_migration_ver, traits::PortfolioSubTrait, with_transaction, Balance, IdentityId, Memo,
    NFTs, PortfolioId, SecondaryKey, WeightMeter,
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
        /// Scheduling of instruction fails.
        SchedulingFailed(InstructionId, DispatchError),
        /// Instruction is rescheduled.
        /// (caller DID, instruction_id)
        InstructionRescheduled(IdentityId, InstructionId),
        /// An existing venue's signers has been updated (did, venue_id, signers, update_type)
        VenueSignersUpdated(IdentityId, VenueId, Vec<T::AccountId>, bool),
        /// Settlement manually executed (did, id)
        SettlementManuallyExecuted(IdentityId, InstructionId),
        /// A new instruction has been created
        /// (did, venue_id, instruction_id, settlement_type, trade_date, value_date, legs, memo)
        InstructionCreated(
            IdentityId,
            Option<VenueId>,
            InstructionId,
            SettlementType<T::BlockNumber>,
            Option<T::Moment>,
            Option<T::Moment>,
            Vec<Leg>,
            Option<Memo>,
        ),
        /// Failed to execute instruction.
        FailedToExecuteInstruction(InstructionId, DispatchError),
        /// An instruction has been automatically affirmed.
        /// Parameters: [`IdentityId`] of the caller, [`PortfolioId`] of the receiver, and [`InstructionId`] of the instruction.
        InstructionAutomaticallyAffirmed(IdentityId, PortfolioId, InstructionId),
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
        fn withdraw_affirmation(f: u32, n: u32, o: u32) -> Weight;
        fn execute_instruction_paused(f: u32, n: u32, o: u32) -> Weight;
        fn execute_scheduled_instruction(f: u32, n: u32, o: u32) -> Weight;
        fn ensure_root_origin() -> Weight;
        fn affirm_with_receipts_rcv(f: u32, n: u32, o: u32) -> Weight;
        fn affirm_instruction_rcv(f: u32, n: u32) -> Weight;
        fn withdraw_affirmation_rcv(f: u32, n: u32, o: u32) -> Weight;
        fn add_instruction_with_mediators(f: u32, n: u32, o: u32, m: u32) -> Weight;
        fn add_and_affirm_with_mediators(f: u32, n: u32, o: u32, m: u32) -> Weight;
        fn affirm_instruction_as_mediator() -> Weight;
        fn withdraw_affirmation_as_mediator() -> Weight;
        fn valid_caller_portfolio() -> Weight;
        fn valid_caller_venue() -> Weight;
        fn valid_caller_mediator() -> Weight;
        fn manual_execution_common(f: u32, n: u32, o: u32) -> Weight;
        fn validate_mediators_affirmations(n: u32) -> Weight;
        fn assets_can_be_transferred_common(n: u32) -> Weight;
        fn validate_execute_instruction_conditions_common(f: u32, n: u32, o: u32) -> Weight;
        fn ensure_assets_are_not_frozen(f: u32) -> Weight;
        fn ensure_valid_cdd_claims(f: u32) -> Weight;
        fn valid_receivers_portfolio(f: u32) -> Weight;
        fn senders_are_funded(f: u32) -> Weight;
        fn senders_balance_read(f: u32) -> Weight;
        fn maximum_lock_period() -> Weight;
        fn transfer_assets(f: u32, n: u32) -> Weight;
        fn prune_instruction(f: u32, n: u32, o: u32) -> Weight;
        fn reject_instruction_common(f: u32, n: u32, o: u32) -> Weight;
        fn execute_instruction_common(f: u32, n: u32, o: u32) -> Weight;
        fn lock_instruction_common(f: u32, n: u32, o: u32) -> Weight;

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
            weight_limit: &Option<Weight>,
            f: &u32,
            n: &u32,
            o: &u32,
        ) -> Weight {
            if let Some(weight_limit) = weight_limit {
                return *weight_limit;
            }
            Self::execute_manual_instruction(*f, *n, *o)
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
        fn withdraw_affirmation_input(
            affirmation_count: Option<AffirmationCount>,
            portfolios: u32,
        ) -> Weight {
            match affirmation_count {
                Some(affirmation_count) => {
                    let max_portfolios = affirmation_count.max_portfolios();
                    if portfolios > max_portfolios {
                        // Too many portfolios, return worse-case weight based on portfolio count.
                        return Self::withdraw_affirmation(portfolios, portfolios, 10);
                    }
                    // The weight for the assets being sent
                    let sender_asset_count = affirmation_count.sender_asset_count();
                    let sender_side_weight = Self::withdraw_affirmation(
                        sender_asset_count.fungible(),
                        sender_asset_count.non_fungible(),
                        affirmation_count.offchain_count(),
                    );
                    // The weight for the assets being received
                    let receiver_asset_count = affirmation_count.receiver_asset_count();
                    let receiver_side_weight = Self::withdraw_affirmation_rcv(
                        receiver_asset_count.fungible(),
                        receiver_asset_count.non_fungible(),
                        0,
                    );
                    // Common reads/writes are being added twice
                    let duplicated_weight = Self::withdraw_affirmation_rcv(0, 0, 0);
                    // The actual weight is the sum of the sender and receiver weights subtracted by the duplicated weight
                    sender_side_weight
                        .saturating_add(receiver_side_weight)
                        .saturating_sub(duplicated_weight)
                }
                None => {
                    if portfolios > (10 + 100) * 2 {
                        // Too many portfolios, return worse-case weight based on portfolio count.
                        Self::withdraw_affirmation(portfolios, portfolios, 10)
                    } else {
                        Self::withdraw_affirmation(10, 100, 10)
                    }
                }
            }
        }

        fn reject_instruction(inst_asset_count: &AssetCount) -> Weight {
            let reject_common = Self::reject_instruction_common(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
            );
            let caller_validation = Self::valid_caller_portfolio();
            let prune = Self::prune_instruction(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
            );

            reject_common
                .saturating_add(caller_validation)
                .saturating_add(prune)
        }

        fn reject_instruction_input(inst_asset_count: Option<AssetCount>) -> Weight {
            if let Some(inst_asset_count) = inst_asset_count {
                return Self::reject_instruction(&inst_asset_count);
            }

            Self::reject_instruction(&AssetCount::new(10, 100, 10))
        }
        fn lock_instruction(weight_limit: Weight) -> Weight {
            let minimum_weight = Self::lock_instruction_common(0, 0, 1).saturating_add(
                Self::validate_execute_instruction_conditions_common(0, 0, 1),
            );
            weight_limit.max(minimum_weight)
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
    {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A call type used by the scheduler.
        type Proposal: From<Call<Self>> + Into<<Self as pallet_identity::Config>::Proposal>;

        /// Scheduler of settlement instructions.
        type Scheduler: Named<Self::BlockNumber, <Self as Config>::Proposal, Self::SchedulerOrigin>;

        /// Portfolio module.
        type Portfolio: PortfolioSubTrait<Self::AccountId>;

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
        type MaxNumberOfPortfolios: Get<u32>;

        /// Maximum number of venue signers.
        #[pallet::constant]
        type MaxNumberOfVenueSigners: Get<u32>;

        /// Maximum number mediators in the instruction level (this does not include asset mediators).
        #[pallet::constant]
        type MaxInstructionMediators: Get<u32>;

        /// The maximum time period that an instruction can be held in the `LockedForExecution` status.
        #[pallet::constant]
        type MaximumLockPeriod: Get<Self::Moment>;
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
        /// Instructions of type `SettleOnComplianceCheck` must have at least one mediator.
        SettlementTypeRequiresMediators,
        /// The instruction has a frozen asset.
        InstructionWithAFrozenAsset,
        /// The instruction has an identity with an invalid CDD claim.
        InstructionWithAnInvalidCDDClaim,
        /// One of the instruction receivers is not compliant.
        IntructionReceiverIsNotCompliant,
        /// One of the instruction senders is not compliant.
        IntructionSenderIsNotCompliant,
        /// Of of the sender doesn't have enough balance to execute the instruction.
        SenderHasInsufficientBalance,
        /// The instruction is trying to transfer the same nft more than once.
        DuplicatedNFTId,
        /// The instruction has been locked for too much time.
        ExceededMaximumLockingPeriod,
        /// All locked instruction must register a lock timestamp.
        LockTimestampNotFound,
        /// Unexpected settlement type.
        UnexpectedSettlementType,
        /// [`InstructionStatus::Unknow`] or [`InstructionStatus::LockedForExecution`] can't be rejected.
        InvalidInstructionStatusForRejection,
    }

    storage_migration_ver!(3);

    #[pallet::pallet]
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
        Instruction<T::Moment, T::BlockNumber>,
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
        PortfolioId,
        AffirmationStatus,
        ValueQuery,
    >;

    /// Helps a user track their pending instructions and affirmations (only needed for UI).
    /// (counter_party, instruction_id) -> AffirmationStatus
    #[pallet::storage]
    pub type UserAffirmations<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PortfolioId,
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
    pub type InstructionStatuses<T: Config> =
        StorageMap<_, Twox64Concat, InstructionId, InstructionStatus<T::BlockNumber>, ValueQuery>;

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

    /// Storage version.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig;

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            VenueCounter::<T>::put(VenueId(1));
            InstructionCounter::<T>::put(InstructionId(1));
            StorageVersion::<T>::put(Version::new(3));
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
            signers: Vec<T::AccountId>,
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
            // NB: Venue counter starts with 1.
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
            Self::ensure_venue_creator(&id, caller_did)?;

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

            let mut venue = Self::ensure_venue_creator(&id, caller_did)?;
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
        /// * `portfolios` - a vector of [`PortfolioId`] under the caller's control and intended for affirmation.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::affirm_with_receipts_input(None, portfolios.len() as u32))]
        #[pallet::call_index(3)]
        pub fn affirm_with_receipts(
            origin: OriginFor<T>,
            id: InstructionId,
            receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
            portfolios: BoundedBTreeSet<PortfolioId, T::MaxNumberOfPortfolios>,
        ) -> DispatchResultWithPostInfo {
            Self::affirm_with_receipts_and_maybe_schedule_instruction(
                origin,
                id,
                receipt_details,
                portfolios.into_inner(),
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
            let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
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
            let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
            for venue in &venues {
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
            let did = <ExternalAgents<T>>::ensure_perms(origin, asset_id)?;
            for venue in &venues {
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
            signers: Vec<T::AccountId>,
            add_signers: bool,
        ) -> DispatchResult {
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;

            Self::base_update_venue_signers(did, id, signers, add_signers)?;
            Ok(())
        }

        /// Manually executes an instruction.
        ///
        /// # Arguments
        /// * `id` - The [`InstructionId`] of the instruction to be executed.
        /// * `portfolio` - An optional [`PortfolioId`] that belongs to the caller, which must be a counterparty
        ///   in the instruction. If `None`, the caller must either be the venue creator or a counterparty in an [`Leg::OffChain`].
        /// * `fungible_transfers` - The number of fungible asset transfers in the instruction.
        /// * `nfts_transfers` - The number of non-fungible token (NFT) transfers in the instruction.
        /// * `offchain_transfers` - The number of off-chain asset transfers in the instruction.
        /// * `weight_limit` - An optional maximum [`Weight`] value to be charged for executing the instruction.
        ///   If the `weight_limit` is less than the required weight, the execution will fail.
        ///
        /// # Permissions
        /// The caller must meet one of the following conditions:
        /// - Be the creator of the venue associated with the instruction.
        /// - Be a counterparty in the instruction.
        ///
        /// # Notes
        /// - The caller can use the RPC method `get_execute_instruction_info` to retrieve an instance of
        ///   [`ExecuteInstructionInfo`], which provides the counts for fungible, NFT, and off-chain transfers.
        #[pallet::weight(<T as Config>::WeightInfo::execute_manual_weight_limit(weight_limit, fungible_transfers, nfts_transfers, offchain_transfers))]
        #[pallet::call_index(8)]
        pub fn execute_manual_instruction(
            origin: OriginFor<T>,
            id: InstructionId,
            portfolio: Option<PortfolioId>,
            fungible_transfers: u32,
            nfts_transfers: u32,
            offchain_transfers: u32,
            weight_limit: Option<Weight>,
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_manual_instruction_minimum_weight(),
                weight_limit.unwrap_or(Self::execute_manual_instruction_weight_limit(
                    fungible_transfers,
                    nfts_transfers,
                    offchain_transfers,
                )),
            )?;
            Self::base_manual_execution(origin, id, portfolio, &mut weight_meter).map_err(|e| {
                DispatchErrorWithPostInfo {
                    post_info: Some(weight_meter.consumed()).into(),
                    error: e.error,
                }
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
            settlement_type: SettlementType<T::BlockNumber>,
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
        /// * `portfolios`: A vector of [`PortfolioId`] under the caller's control and intended for affirmation.
        /// * `memo`: An optional [`Memo`] field for this instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::add_and_affirm_instruction_legs(legs, portfolios.len() as u32))]
        #[pallet::call_index(10)]
        pub fn add_and_affirm_instruction(
            origin: OriginFor<T>,
            venue_id: Option<VenueId>,
            settlement_type: SettlementType<T::BlockNumber>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
            portfolios: BoundedBTreeSet<PortfolioId, T::MaxNumberOfPortfolios>,
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
                portfolios.into_inner(),
                None,
            )
            .map_err(|e| e.error)?;
            Ok(())
        }

        /// Provide affirmation to an existing instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being affirmed.
        /// * `portfolios` - a vector of [`PortfolioId`] under the caller's control and intended for affirmation.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::affirm_instruction_input(None, portfolios.len() as u32))]
        #[pallet::call_index(11)]
        pub fn affirm_instruction(
            origin: OriginFor<T>,
            id: InstructionId,
            portfolios: BoundedBTreeSet<PortfolioId, T::MaxNumberOfPortfolios>,
        ) -> DispatchResultWithPostInfo {
            Self::affirm_and_maybe_schedule_instruction(origin, id, portfolios.into_inner(), None)
        }

        /// Withdraw an affirmation for a given instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction getting an affirmation withdrawn.
        /// * `portfolios` - a vector of [`PortfolioId`] under the caller's control and intended for affirmation withdrawal.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_affirmation_input(None, portfolios.len() as u32))]
        #[pallet::call_index(12)]
        pub fn withdraw_affirmation(
            origin: OriginFor<T>,
            id: InstructionId,
            portfolios: BoundedBTreeSet<PortfolioId, T::MaxNumberOfPortfolios>,
        ) -> DispatchResultWithPostInfo {
            Self::base_withdraw_affirmation(origin, id, portfolios.into_inner(), None)
        }

        /// Rejects an existing instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being rejected.
        /// * `portfolio` - the [`PortfolioId`] that belongs to the instruction and is rejecting it.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::reject_instruction_input(None))]
        #[pallet::call_index(13)]
        pub fn reject_instruction(
            origin: OriginFor<T>,
            id: InstructionId,
            portfolio: PortfolioId,
        ) -> DispatchResultWithPostInfo {
            Self::base_reject_instruction(
                origin,
                id,
                Some(portfolio),
                &mut WeightMeter::max_limit_no_minimum(),
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
        /// * `portfolios` - a vector of [`PortfolioId`] under the caller's control and intended for affirmation.
        /// * `number_of_assets` - an optional [`AffirmationCount`] that will be used for a precise fee estimation before executing the extrinsic.
        ///
        /// Note: calling the rpc method `get_affirmation_count` returns an instance of [`AffirmationCount`].
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::affirm_with_receipts_input(*number_of_assets, portfolios.len() as u32))]
        #[pallet::call_index(15)]
        pub fn affirm_with_receipts_with_count(
            origin: OriginFor<T>,
            id: InstructionId,
            receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
            portfolios: BoundedBTreeSet<PortfolioId, T::MaxNumberOfPortfolios>,
            number_of_assets: Option<AffirmationCount>,
        ) -> DispatchResult {
            Self::affirm_with_receipts_and_maybe_schedule_instruction(
                origin,
                id,
                receipt_details,
                portfolios.into_inner(),
                number_of_assets,
            )
            .map_err(|e| e.error)?;
            Ok(())
        }

        /// Provide affirmation to an existing instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being affirmed.
        /// * `portfolios` - a vector of [`PortfolioId`] under the caller's control and intended for affirmation.
        /// * `number_of_assets` - an optional [`AffirmationCount`] that will be used for a precise fee estimation before executing the extrinsic.
        ///
        /// Note: calling the rpc method `get_affirmation_count` returns an instance of [`AffirmationCount`].
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::affirm_instruction_input(*number_of_assets, portfolios.len() as u32))]
        #[pallet::call_index(16)]
        pub fn affirm_instruction_with_count(
            origin: OriginFor<T>,
            id: InstructionId,
            portfolios: BoundedBTreeSet<PortfolioId, T::MaxNumberOfPortfolios>,
            number_of_assets: Option<AffirmationCount>,
        ) -> DispatchResult {
            Self::affirm_and_maybe_schedule_instruction(
                origin,
                id,
                portfolios.into_inner(),
                number_of_assets,
            )
            .map_err(|e| e.error)?;
            Ok(())
        }

        /// Rejects an existing instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction being rejected.
        /// * `portfolio` - the [`PortfolioId`] that belongs to the instruction and is rejecting it.
        /// * `number_of_assets` - an optional [`AssetCount`] that will be used for a precise fee estimation before executing the extrinsic.
        ///
        /// Note: calling the rpc method `get_execute_instruction_info` returns an instance of [`ExecuteInstructionInfo`], which contain the asset count.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::reject_instruction_input(*number_of_assets))]
        #[pallet::call_index(17)]
        pub fn reject_instruction_with_count(
            origin: OriginFor<T>,
            id: InstructionId,
            portfolio: PortfolioId,
            number_of_assets: Option<AssetCount>,
        ) -> DispatchResult {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::reject_instruction_minimum_weight(),
                <T as Config>::WeightInfo::reject_instruction_input(number_of_assets),
            )
            .map_err(|e| e.error)?;

            Self::base_reject_instruction(origin, id, Some(portfolio), &mut weight_meter)
                .map_err(|e| e.error)?;
            Ok(())
        }

        /// Withdraw an affirmation for a given instruction.
        ///
        /// # Arguments
        /// * `id` - the [`InstructionId`] of the instruction getting an affirmation withdrawn.
        /// * `portfolios` - a vector of [`PortfolioId`] under the caller's control and intended for affirmation withdrawal.
        /// * `number_of_assets` - an optional [`AffirmationCount`] that will be used for a precise fee estimation before executing the extrinsic.
        ///
        /// Note: calling the rpc method `get_affirmation_count` returns an instance of [`AffirmationCount`].
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_affirmation_input(*number_of_assets, portfolios.len() as u32))]
        #[pallet::call_index(18)]
        pub fn withdraw_affirmation_with_count(
            origin: OriginFor<T>,
            id: InstructionId,
            portfolios: BoundedBTreeSet<PortfolioId, T::MaxNumberOfPortfolios>,
            number_of_assets: Option<AffirmationCount>,
        ) -> DispatchResult {
            Self::base_withdraw_affirmation(origin, id, portfolios.into_inner(), number_of_assets)
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
            settlement_type: SettlementType<T::BlockNumber>,
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
        /// * `portfolios`: A vector of [`PortfolioId`] under the caller's control and intended for affirmation.
        /// * `instruction_memo`: An optional [`Memo`] field for this instruction.
        /// * `mediators`: A set of [`IdentityId`] of all the mandatory mediators for the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::add_and_affirm_with_mediators_legs(legs, portfolios.len() as u32, mediators.len() as u32))]
        #[pallet::call_index(20)]
        pub fn add_and_affirm_with_mediators(
            origin: OriginFor<T>,
            venue_id: Option<VenueId>,
            settlement_type: SettlementType<T::BlockNumber>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<Leg>,
            portfolios: BoundedBTreeSet<PortfolioId, T::MaxNumberOfPortfolios>,
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
                portfolios.into_inner(),
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

        /// Removes the mediator's affirmation for the instruction - should only be called by mediators, otherwise it will fail.
        ///
        /// # Arguments
        /// * `origin`: The secondary key of the sender.
        /// * `instruction_id`: The [`InstructionId`] that will have the affirmation removed.
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_affirmation_as_mediator())]
        #[pallet::call_index(22)]
        pub fn withdraw_affirmation_as_mediator(
            origin: OriginFor<T>,
            instruction_id: InstructionId,
        ) -> DispatchResult {
            Self::base_withdraw_affirmation_as_mediator(origin, instruction_id)?;
            Ok(())
        }

        /// Rejects an existing instruction - should only be called by mediators, otherwise it will fail.
        ///
        /// # Arguments
        /// * `instruction_id` - the [`InstructionId`] of the instruction being rejected.
        /// * `number_of_assets` - an optional [`AssetCount`] that will be used for a precise fee estimation before executing the extrinsic.
        ///
        /// Note: calling the rpc method `get_execute_instruction_info` returns an instance of [`ExecuteInstructionInfo`], which contain the asset count.
        #[pallet::weight(<T as Config>::WeightInfo::reject_instruction_input(*number_of_assets))]
        #[pallet::call_index(23)]
        pub fn reject_instruction_as_mediator(
            origin: OriginFor<T>,
            instruction_id: InstructionId,
            number_of_assets: Option<AssetCount>,
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::reject_instruction_minimum_weight(),
                <T as Config>::WeightInfo::reject_instruction_input(number_of_assets),
            )?;

            Self::base_reject_instruction(origin, instruction_id, None, &mut weight_meter)
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
        /// * `instruction_id` - The [`InstructionId`] of the instruction to be locked.
        /// * `weight_limit` - An optional maximum [`Weight`] value to be charged for locking the instruction.
        #[pallet::weight(<T as Config>::WeightInfo::lock_instruction(*weight_limit))]
        #[pallet::call_index(24)]
        pub fn lock_instruction(
            origin: OriginFor<T>,
            instruction_id: InstructionId,
            weight_limit: Weight,
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::lock_instruction_minimum_weight(),
                weight_limit,
            )?;

            Self::base_lock_instruction(origin, instruction_id, &mut weight_meter)?;

            Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Locks the assets of a leg.
    #[rustfmt::skip]
    fn lock_via_leg(leg: &Leg) -> DispatchResult {
        match leg {
            Leg::Fungible { sender, asset_id, amount, .. } => {
                T::Portfolio::lock_tokens(&sender, &asset_id, *amount)?;
                Ok(())
            }
            Leg::NonFungible { sender, nfts, .. } => {
                for nft_id in nfts.ids() {
                    T::Portfolio::lock_nft(&sender, nfts.asset_id(), &nft_id)?;
                }
                Ok(())
            }
            Leg::OffChain { .. } => Err(Error::<T>::OffChainAssetCantBeLocked.into()),
        }
    }

    /// Unlocks the assets of a leg.
    #[rustfmt::skip]
    fn unlock_via_leg(leg: &Leg) -> DispatchResult {
        match leg {
            Leg::Fungible { sender, asset_id, amount, .. } => {
                T::Portfolio::unlock_tokens(&sender, &asset_id, *amount)?;
                Ok(())
            }
            Leg::NonFungible { sender, nfts, .. } => {
                for nft_id in nfts.ids() {
                    T::Portfolio::unlock_nft(&sender, nfts.asset_id(), &nft_id)?;
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
    ) -> EnsureValidInstructionResult<T::AccountId, T::Moment, T::BlockNumber> {
        let origin_data = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
        Ok((
            origin_data.primary_did,
            origin_data.secondary_key,
            Self::ensure_instruction_validity(id, is_execute)?,
        ))
    }

    /// Returns `Ok(Venue)` if `venue_id` was created by `did`.
    fn ensure_venue_creator(venue_id: &VenueId, did: IdentityId) -> Result<Venue, DispatchError> {
        let venue = VenueInfo::<T>::get(venue_id).ok_or(Error::<T>::InvalidVenue)?;
        ensure!(venue.creator == did, Error::<T>::Unauthorized);
        Ok(venue)
    }

    pub fn base_add_instruction(
        did: IdentityId,
        venue_id: Option<VenueId>,
        settlement_type: SettlementType<T::BlockNumber>,
        trade_date: Option<T::Moment>,
        value_date: Option<T::Moment>,
        legs: Vec<Leg>,
        memo: Option<Memo>,
        mediators: Option<BoundedBTreeSet<IdentityId, T::MaxInstructionMediators>>,
    ) -> Result<InstructionId, DispatchError> {
        // Verifies if the block number is in the future so that `T::Scheduler::schedule_named` doesn't fail.
        if let SettlementType::SettleOnBlock(block_number) = &settlement_type {
            ensure!(
                *block_number > System::<T>::block_number(),
                Error::<T>::SettleOnPastBlock
            );
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
            Self::ensure_venue_creator(&venue_id, did)?;
        }

        // Verifies if all legs are valid.
        let mut instruction_info = Self::ensure_valid_legs(&legs, &venue_id)?;

        // Adds the instruction mediators
        if let Some(mediators) = mediators {
            instruction_info.extend_mediators(mediators.into())
        }

        if settlement_type == SettlementType::SettleOnComplianceCheck
            && instruction_info.mediators().is_empty()
        {
            return Err(Error::<T>::SettlementTypeRequiresMediators.into());
        }

        // Advance and get next `instruction_id`.
        let instruction_id = InstructionCounter::<T>::try_mutate(try_next_post::<T, _>)?;

        // All checks have been made - Write data to storage.
        InstructionStatuses::<T>::insert(instruction_id, InstructionStatus::Pending);

        for portfolio_id in instruction_info.portfolios_pending_approval() {
            UserAffirmations::<T>::insert(portfolio_id, instruction_id, AffirmationStatus::Pending);
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

        for portfolio_id in instruction_info.portfolios_pre_approved_difference() {
            UserAffirmations::<T>::insert(
                portfolio_id,
                instruction_id,
                AffirmationStatus::Affirmed,
            );
            AffirmsReceived::<T>::insert(instruction_id, portfolio_id, AffirmationStatus::Affirmed);
            Self::deposit_event(Event::InstructionAutomaticallyAffirmed(
                did,
                *portfolio_id,
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
            Self::schedule_instruction(instruction_id, block_number, weight_limit);
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
        // Tracks all portfolios that have not been pre-affirmed
        let mut portfolios_pending_approval = BTreeSet::new();
        // Tracks all portfolios that have pre-approved the transfer.
        let mut portfolios_pre_approved = BTreeSet::new();
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
            pallet_identity::Pallet::<T>::ensure_id_record_exists(sender.did)?;
            pallet_identity::Pallet::<T>::ensure_id_record_exists(receiver.did)?;
            T::Portfolio::ensure_portfolio_validity(sender)?;
            T::Portfolio::ensure_portfolio_validity(receiver)?;

            portfolios_pending_approval.insert(*sender);
            if T::Portfolio::skip_portfolio_affirmation(receiver, asset_id) {
                portfolios_pre_approved.insert(*receiver);
            } else {
                portfolios_pending_approval.insert(*receiver);
            }

            let asset_mediators = MandatoryMediators::<T>::get(asset_id);
            mediators.extend(asset_mediators.iter());
        }
        // The maximum number of each asset type in one instruction is checked here
        Self::ensure_within_instruction_max(&instruction_asset_count)?;

        Ok(InstructionInfo::new(
            instruction_asset_count,
            portfolios_pending_approval,
            portfolios_pre_approved,
            mediators,
        ))
    }

    fn unsafe_withdraw_instruction_affirmation(
        did: IdentityId,
        id: InstructionId,
        portfolios: BTreeSet<PortfolioId>,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        affirmation_count: Option<AffirmationCount>,
    ) -> Result<FilteredLegs, DispatchError> {
        // checks custodianship of portfolios and affirmation status
        Self::ensure_portfolios_and_affirmation_status(
            id,
            &portfolios,
            did,
            secondary_key,
            &[AffirmationStatus::Affirmed],
        )?;
        // Unlock tokens that were previously locked during the affirmation
        let filtered_legs = Self::filtered_legs(id, &portfolios);
        // If the fee was estimated in advance, the input values must be at least equal to the actual values
        if let Some(affirmation_count) = affirmation_count {
            Self::ensure_valid_affirmation_count(&filtered_legs, &affirmation_count)?;
        }
        for (leg_id, leg) in filtered_legs.sender_subset() {
            match InstructionLegStatus::<T>::get(id, leg_id) {
                LegStatus::ExecutionToBeSkipped(_, _) => {
                    return Err(Error::<T>::UnexpectedLegStatus.into())
                }
                LegStatus::ExecutionPending => {
                    Self::unlock_via_leg(&leg)?;
                }
                LegStatus::PendingTokenLock => {
                    return Err(Error::<T>::InstructionNotAffirmed.into());
                }
            };
            <InstructionLegStatus<T>>::insert(id, leg_id, LegStatus::PendingTokenLock);
        }

        // Updates storage.
        for portfolio in &portfolios {
            UserAffirmations::<T>::insert(portfolio, id, AffirmationStatus::Pending);
            AffirmsReceived::<T>::remove(id, portfolio);
            Self::deposit_event(Event::AffirmationWithdrawn(did, *portfolio, id));
        }

        InstructionAffirmsPending::<T>::mutate(id, |affirms_pending| {
            *affirms_pending += u64::try_from(portfolios.len()).unwrap_or_default()
        });
        Ok(filtered_legs)
    }

    fn ensure_instruction_validity(
        id: InstructionId,
        is_execute: bool,
    ) -> Result<Instruction<T::Moment, T::BlockNumber>, DispatchError> {
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

    /// Tries to execute the instruction. If the execution succeeds, all assets are transferred, the instruction
    /// is pruned and the status is set to `Success`. If the execution fails, an event is emitted and
    /// the instruction status is set to `Failed`.
    fn execute_instruction_retryable(
        inst_id: InstructionId,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        if let Err(e) = Self::execute_instruction(inst_id, caller_did, weight_meter) {
            Self::deposit_event(Event::FailedToExecuteInstruction(inst_id, e));
            InstructionStatuses::<T>::insert(inst_id, InstructionStatus::Failed);
            return Err(e);
        }
        Ok(())
    }

    /// Tries to execute the instruction. If the execution succeeds, all assets are transferred, the instruction
    /// is pruned and the status is set to `Success`. If the execution fails, all state changes are reverted.
    fn execute_instruction(
        inst_id: InstructionId,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(&inst_id).collect();
        let inst_asset_count = AssetCount::from_legs(&inst_legs);

        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::execute_instruction_common(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
            ),
        )?;

        Self::validate_execute_instruction_conditions(
            &inst_id,
            &inst_legs,
            &inst_asset_count,
            weight_meter,
        )?;

        let inst_memo = InstructionMemos::<T>::get(&inst_id);
        with_transaction(|| {
            Self::transfer_assets(
                inst_id,
                inst_legs.clone(),
                inst_memo,
                caller_did,
                &inst_asset_count,
                weight_meter,
            )?;
            Self::prune_instruction(&inst_id, &inst_legs, &inst_asset_count, weight_meter)?;
            InstructionStatuses::<T>::insert(
                inst_id,
                InstructionStatus::Success(System::<T>::block_number()),
            );
            Self::deposit_event(Event::InstructionExecuted(caller_did, inst_id));
            Ok(())
        })
    }

    /// Returns `Ok` if all conditions for executing the instruction are met.
    /// The conditions for executing an instruction are:
    /// - All affirmations have been received
    /// - Instruction is pending or has failed at least one time
    /// - All mediator's affirmations are still valid
    /// - All assets are in the allowed venue list
    /// - All senders have the right amount of assets being transferred
    /// - All senders and receivers are compliant and have valid CDD claims
    /// - All assets' statistics are still valid
    /// - There are no frozen assets
    fn validate_execute_instruction_conditions(
        inst_id: &InstructionId,
        inst_legs: &[(LegId, Leg)],
        inst_asset_count: &AssetCount,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::validate_execute_instruction_conditions_common(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
            ),
        )?;

        ensure!(
            InstructionAffirmsPending::<T>::get(inst_id) == 0,
            Error::<T>::NotAllAffirmationsHaveBeenReceived
        );

        Self::ensure_instruction_is_pending_or_failed(inst_id)?;

        Self::validate_mediators_affirmations(inst_id, weight_meter)?;

        Self::ensure_no_missing_affirmation(inst_id, inst_legs)?;

        let instruction_details = InstructionDetails::<T>::get(inst_id);
        Self::ensure_allowed_venue(&inst_legs, instruction_details.venue_id)?;

        Self::ensure_assets_can_be_transferred(inst_id, &inst_legs, weight_meter)?;
        Ok(())
    }

    /// Returns `Ok` if all mediator's affirmation are still valid.
    fn validate_mediators_affirmations(
        inst_id: &InstructionId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let current_timestamp = <pallet_timestamp::Pallet<T>>::get();

        let mediators_affirmations: Vec<_> =
            InstructionMediatorsAffirmations::<T>::iter_prefix_values(&inst_id).collect();
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::validate_mediators_affirmations(
                mediators_affirmations.len() as u32,
            ),
        )?;

        for affirmation in mediators_affirmations {
            match affirmation {
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
    fn ensure_no_missing_affirmation(
        instruction_id: &InstructionId,
        instruction_legs: &[(LegId, Leg)],
    ) -> DispatchResult {
        let mut unique_portfolios = BTreeSet::new();

        for (leg_id, leg) in instruction_legs {
            match leg {
                Leg::Fungible { sender, receiver, .. }
                | Leg::NonFungible { sender, receiver, .. } => {
                    if unique_portfolios.insert(sender) {
                        let sdr_affirmation_status =
                            UserAffirmations::<T>::get(sender, instruction_id);
                        ensure!(
                            sdr_affirmation_status == AffirmationStatus::Affirmed,
                            Error::<T>::NotAllAffirmationsHaveBeenReceived
                        );
                        let sdr_affirmation_status =
                            AffirmsReceived::<T>::get(instruction_id, sender);
                        ensure!(
                            sdr_affirmation_status == AffirmationStatus::Affirmed,
                            Error::<T>::NotAllAffirmationsHaveBeenReceived
                        );
                    }
                    if unique_portfolios.insert(receiver) {
                        let rcv_affirmation_status =
                            UserAffirmations::<T>::get(receiver, instruction_id);
                        ensure!(
                            rcv_affirmation_status == AffirmationStatus::Affirmed,
                            Error::<T>::NotAllAffirmationsHaveBeenReceived
                        );
                        let rcv_affirmation_status =
                            AffirmsReceived::<T>::get(instruction_id, receiver);
                        ensure!(
                            rcv_affirmation_status == AffirmationStatus::Affirmed,
                            Error::<T>::NotAllAffirmationsHaveBeenReceived
                        );
                    }
                }
                Leg::OffChain { .. } => {
                    ensure!(
                        OffChainAffirmations::<T>::get(instruction_id, leg_id)
                            == AffirmationStatus::Affirmed,
                        Error::<T>::NotAllAffirmationsHaveBeenReceived,
                    );
                }
            }
        }

        Ok(())
    }

    /// Returns `Ok` if all assets can be transferred.
    #[rustfmt::skip]
    fn ensure_assets_can_be_transferred(
        inst_id: &InstructionId,
        inst_legs: &[(LegId, Leg)],
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::assets_can_be_transferred_common(
                inst_legs.len() as u32
            ),
        )?;

        let mut nfts_transferred = BTreeMap::new();
        let mut fungible_tx_summary = FungibleTxSummary::new();

        // Aggregates the total amount of assets sent and received per DID and per Portfolio
        for (leg_id, leg) in inst_legs {
            let leg_status = InstructionLegStatus::<T>::get(inst_id, leg_id);
            match leg {
                Leg::Fungible { sender, receiver, asset_id, amount } => {
                    ensure!(
                        leg_status == LegStatus::ExecutionPending,
                        Error::<T>::UnexpectedLegStatus
                    );
                    fungible_tx_summary
                        .add_fungible_transfer(*sender, *receiver, *asset_id, *amount);
                }
                Leg::NonFungible { sender, receiver, nfts } => {
                    ensure!(
                        leg_status == LegStatus::ExecutionPending,
                        Error::<T>::UnexpectedLegStatus
                    );
                    Self::ensure_valid_nft_transfer(
                        sender,
                        receiver,
                        nfts,
                        &mut nfts_transferred,
                        weight_meter,
                    )?;
                }
                Leg::OffChain { .. } => {
                    if let LegStatus::ExecutionToBeSkipped(_, _) = leg_status {
                        continue;
                    }
                    return Err(Error::<T>::UnexpectedLegStatus.into());
                }
            }
        }

        Self::ensure_valid_fungible_transfers(&fungible_tx_summary, weight_meter)?;
        Ok(())
    }

    /// Returns `Ok` if the nfts can be transferred. Adds the nfts to the `nfts_transferred` map.
    fn ensure_valid_nft_transfer(
        sender_pid: &PortfolioId,
        receiver_pid: &PortfolioId,
        nfts: &NFTs,
        nfts_transferred: &mut BTreeMap<AssetId, BTreeSet<NFTId>>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        for nft_id in nfts.ids() {
            // It should not be possible to transfer the same NFT twice in the same instruction
            if let Some(transferred_ids) = nfts_transferred.get(nfts.asset_id()) {
                ensure!(
                    !transferred_ids.contains(nft_id),
                    Error::<T>::DuplicatedNFTId
                );
            }

            Nft::<T>::validate_nft_transfer(
                sender_pid,
                receiver_pid,
                nfts,
                false,
                Some(weight_meter),
            )?;

            nfts_transferred
                .entry(*nfts.asset_id())
                .and_modify(|nft_ids| {
                    nft_ids.insert(*nft_id);
                })
                .or_insert(BTreeSet::from([*nft_id]));
        }
        Ok(())
    }

    /// Returns `Ok` if all non fungible transfers are valid.
    fn ensure_valid_fungible_transfers(
        fungible_tx_summary: &FungibleTxSummary,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::ensure_assets_are_not_frozen(fungible_tx_summary.assets(), weight_meter)?;
        Self::ensure_valid_cdd_claims(fungible_tx_summary.dids(), weight_meter)?;
        Self::ensure_receivers_are_compliant_and_their_portfolios_exist(
            fungible_tx_summary.total_rcv_per_did(),
            fungible_tx_summary.rcv_portfolios(),
            weight_meter,
        )?;
        Self::ensure_senders_are_compliant_and_funded(
            fungible_tx_summary.total_sent_per_did(),
            fungible_tx_summary.total_sent_per_portfolio(),
            weight_meter,
        )?;
        Self::ensure_valid_statistics(fungible_tx_summary, weight_meter)?;
        Ok(())
    }

    /// Returns `Ok` if all assets are not frozen.
    fn ensure_assets_are_not_frozen(
        unique_assets: &BTreeSet<AssetId>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::ensure_assets_are_not_frozen(unique_assets.len() as u32),
        )?;

        for asset_id in unique_assets {
            ensure!(
                !Frozen::<T>::get(asset_id),
                Error::<T>::InstructionWithAFrozenAsset
            );
        }
        Ok(())
    }

    /// Returns `Ok` if all identities have a valid CDD claim.
    fn ensure_valid_cdd_claims(
        unique_dids: &BTreeSet<IdentityId>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::ensure_valid_cdd_claims(unique_dids.len() as u32),
        )?;

        for did in unique_dids {
            ensure!(
                pallet_identity::Pallet::<T>::has_valid_cdd(*did),
                Error::<T>::InstructionWithAnInvalidCDDClaim
            );
        }
        Ok(())
    }

    /// Returns `Ok` if all receivers are compliant and their portfolios exist.
    fn ensure_receivers_are_compliant_and_their_portfolios_exist(
        total_rcv_per_did: &BTreeMap<(AssetId, IdentityId), Balance>,
        rcv_portfolios: &BTreeSet<PortfolioId>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::valid_receivers_portfolio(rcv_portfolios.len() as u32),
        )?;

        for portfolio_id in rcv_portfolios {
            T::Portfolio::ensure_portfolio_validity(portfolio_id)?;
        }

        for ((asset_id, did), _) in total_rcv_per_did.iter() {
            if !pallet_compliance_manager::Pallet::<T>::is_compliant(
                asset_id,
                None,
                Some(*did),
                weight_meter,
            )? {
                return Err(Error::<T>::IntructionReceiverIsNotCompliant.into());
            }
        }

        Ok(())
    }

    /// Returns `Ok` if all sender's portfolio have the right balance, the tokens are locked and they are compliant.
    fn ensure_senders_are_compliant_and_funded(
        total_sent_per_did: &BTreeMap<(AssetId, IdentityId), Balance>,
        total_sent_per_portfolio: &BTreeMap<(AssetId, PortfolioId), Balance>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::senders_are_funded(total_sent_per_portfolio.len() as u32)
                .saturating_add(<T as Config>::WeightInfo::senders_balance_read(
                    total_sent_per_did.len() as u32,
                )),
        )?;

        // Each individual portfolio must have all tokens locked and their amount
        for ((asset_id, portfolio_id), balance) in total_sent_per_portfolio.iter() {
            T::Portfolio::ensure_tokens_are_locked(portfolio_id, asset_id, *balance)?;
            T::Portfolio::ensure_portfolio_balance(portfolio_id, asset_id, *balance)?;
        }

        // The aggregate balance of the sender must be equal or greater than the total amount of tokens sent
        for ((asset_id, did), amount) in total_sent_per_did.iter() {
            ensure!(
                BalanceOf::<T>::get(asset_id, did) >= *amount,
                Error::<T>::SenderHasInsufficientBalance
            );

            if !pallet_compliance_manager::Pallet::<T>::is_compliant(
                asset_id,
                Some(*did),
                None,
                weight_meter,
            )? {
                return Err(Error::<T>::IntructionSenderIsNotCompliant.into());
            }
        }

        Ok(())
    }

    /// Returns `Ok` if the statistics requirements of all assets are met.
    fn ensure_valid_statistics(
        instruction_tx_summary: &FungibleTxSummary,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        for asset_id in instruction_tx_summary.assets() {
            Asset::<T>::ensure_valid_statistics(
                *asset_id,
                &instruction_tx_summary.total_rcv_per_did_given_asset(asset_id),
                &instruction_tx_summary.total_sent_per_did_given_asset(asset_id),
                weight_meter,
            )?;
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
    #[rustfmt::skip]
    fn prune_instruction(
        inst_id: &InstructionId,
        inst_legs: &[(LegId, Leg)],
        inst_asset_count: &AssetCount,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::prune_instruction(
                inst_asset_count.fungible() as u32,
                inst_asset_count.non_fungible() as u32,
                inst_asset_count.off_chain() as u32,
            ),
        )?;

        let inst_details = InstructionDetails::<T>::take(inst_id);

        if let Some(venue_id) = inst_details.venue_id {
            VenueInstructions::<T>::remove(&venue_id, inst_id);
        }

        InstructionAffirmsPending::<T>::remove(inst_id);

        let _ = InstructionMediatorsAffirmations::<T>::clear_prefix(
            inst_id,
            T::MaxInstructionMediators::get(),
            None,
        );

        // Removes all affirmations related to the instruction
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

    pub fn unsafe_affirm_instruction(
        did: IdentityId,
        id: InstructionId,
        portfolios: BTreeSet<PortfolioId>,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        affirmation_count: Option<AffirmationCount>,
    ) -> Result<FilteredLegs, DispatchError> {
        // Checks portfolio's custodian and if it is a counter party with a pending affirmation.
        Self::ensure_portfolios_and_affirmation_status(
            id,
            &portfolios,
            did,
            secondary_key,
            &[AffirmationStatus::Pending],
        )?;

        let filtered_legs = Self::filtered_legs(id, &portfolios);
        // If the fee was estimated in advance, the input values must be at least equal to the actual values
        if let Some(affirmation_count) = affirmation_count {
            Self::ensure_valid_affirmation_count(&filtered_legs, &affirmation_count)?
        }
        for (leg_id, leg) in filtered_legs.sender_subset() {
            Self::lock_via_leg(&leg)?;
            <InstructionLegStatus<T>>::insert(id, leg_id, LegStatus::ExecutionPending);
        }

        let affirms_pending = InstructionAffirmsPending::<T>::get(id);

        // Updates storage
        for portfolio in &portfolios {
            UserAffirmations::<T>::insert(portfolio, id, AffirmationStatus::Affirmed);
            AffirmsReceived::<T>::insert(id, portfolio, AffirmationStatus::Affirmed);
            Self::deposit_event(Event::InstructionAffirmed(did, *portfolio, id));
        }
        InstructionAffirmsPending::<T>::insert(
            id,
            affirms_pending.saturating_sub(u64::try_from(portfolios.len()).unwrap_or_default()),
        );
        Ok(filtered_legs)
    }

    /// Unlocks all assets in the instruction.
    fn release_locks(inst_id: &InstructionId, inst_legs: &[(LegId, Leg)]) -> DispatchResult {
        for (leg_id, leg) in inst_legs {
            if InstructionLegStatus::<T>::get(inst_id, leg_id) == LegStatus::ExecutionPending {
                Self::unlock_via_leg(&leg)?;
            }
        }
        Ok(())
    }

    /// Schedule a given instruction to be executed on the next block only if the
    /// settlement type is `SettleOnAffirmation` and no. of affirms pending is 0.
    fn maybe_schedule_instruction(affirms_pending: u64, id: InstructionId, weight_limit: Weight) {
        if affirms_pending == 0
            && InstructionDetails::<T>::get(id).settlement_type
                == SettlementType::SettleOnAffirmation
        {
            // Schedule instruction to be executed in the next block.
            let execution_at = System::<T>::block_number() + One::one();
            Self::schedule_instruction(id, execution_at, weight_limit);
        }
    }

    /// Schedule execution of given instruction at given block number.
    ///
    /// NB - It is expected to execute the given instruction into the given block number but
    /// it is not a guaranteed behavior, Scheduler may have other high priority task scheduled
    /// for the given block so there are chances where the instruction execution block no. may drift.
    pub(crate) fn schedule_instruction(
        id: InstructionId,
        execution_at: T::BlockNumber,
        weight_limit: Weight,
    ) {
        let call = Call::<T>::execute_scheduled_instruction { id, weight_limit }.into();
        if let Err(_) = T::Scheduler::schedule_named(
            id.execution_name(),
            DispatchTime::At(execution_at),
            None,
            SETTLEMENT_INSTRUCTION_EXECUTION_PRIORITY,
            RawOrigin::Root.into(),
            call,
        ) {
            Self::deposit_event(Event::SchedulingFailed(
                id,
                Error::<T>::FailedToSchedule.into(),
            ));
        }
    }

    /// Affirms all legs from the instruction of the given `instruction_id`, where `portfolios` are a counter party.
    /// If the portfolio is the sender, the asset is also locked.
    pub fn base_affirm_with_receipts(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        receipts_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: BTreeSet<PortfolioId>,
        affirmation_count: Option<AffirmationCount>,
    ) -> Result<FilteredLegs, DispatchError> {
        ensure!(
            receipts_details.len() <= T::MaxNumberOfOffChainAssets::get() as usize,
            Error::<T>::MaxNumberOfReceiptsExceeded
        );

        let (did, secondary_key, instruction_details) =
            Self::ensure_origin_perm_and_instruction_validity(origin, instruction_id, false)?;

        // Verify portfolio custodianship and check if it is a counter party with a pending affirmation.
        Self::ensure_portfolios_and_affirmation_status(
            instruction_id,
            &portfolios,
            did,
            secondary_key.as_ref(),
            &[AffirmationStatus::Pending],
        )?;

        Self::ensure_valid_receipts_details(
            instruction_details.venue_id,
            instruction_id,
            &receipts_details,
        )?;

        // Lock tokens for all legs that are not of type [`Leg::OffChain`]
        let filtered_legs = Self::filtered_legs(instruction_id, &portfolios);
        // If the fee was estimated in advance, the input values must be at least equal to the actual values
        if let Some(affirmation_count) = affirmation_count {
            Self::ensure_valid_affirmation_count(&filtered_legs, &affirmation_count)?
        }
        for (leg_id, leg) in filtered_legs.sender_subset() {
            Self::lock_via_leg(&leg)?;
            <InstructionLegStatus<T>>::insert(instruction_id, leg_id, LegStatus::ExecutionPending);
        }

        // Casting is safe since `Self::ensure_portfolios_and_affirmation_status` is called
        let affirms_pending = InstructionAffirmsPending::<T>::get(instruction_id)
            .saturating_sub(portfolios.len() as u64)
            .saturating_sub(receipts_details.len() as u64);
        InstructionAffirmsPending::<T>::insert(instruction_id, affirms_pending);

        // Update storage
        for receipt_detail in receipts_details {
            <InstructionLegStatus<T>>::insert(
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
            <ReceiptsUsed<T>>::insert(receipt_detail.signer(), receipt_detail.uid(), true);
            Self::deposit_event(Event::ReceiptClaimed(
                did,
                instruction_id,
                receipt_detail.leg_id(),
                receipt_detail.uid(),
                receipt_detail.signer().clone(),
                receipt_detail.metadata().clone(),
            ));
        }

        for portfolio in portfolios {
            UserAffirmations::<T>::insert(portfolio, instruction_id, AffirmationStatus::Affirmed);
            AffirmsReceived::<T>::insert(instruction_id, portfolio, AffirmationStatus::Affirmed);
            Self::deposit_event(Event::InstructionAffirmed(did, portfolio, instruction_id));
        }

        Ok(filtered_legs)
    }

    pub fn base_affirm_instruction(
        origin: OriginFor<T>,
        id: InstructionId,
        portfolios: BTreeSet<PortfolioId>,
        affirmation_count: Option<AffirmationCount>,
    ) -> Result<FilteredLegs, DispatchError> {
        let (did, sk, _) = Self::ensure_origin_perm_and_instruction_validity(origin, id, false)?;
        // Provide affirmation to the instruction
        Self::unsafe_affirm_instruction(did, id, portfolios, sk.as_ref(), affirmation_count)
    }

    /// Affirms all legs from the instruction of the given `id`, where `portfolios` are a counter party.
    /// If the portfolio is the sender, the asset is also locked. If all affirmation have been received and
    /// the settlement type is [`SettlementType::SettleOnAffirmation`] the instruction will be scheduled for
    /// the next block.
    pub fn affirm_with_receipts_and_maybe_schedule_instruction(
        origin: OriginFor<T>,
        id: InstructionId,
        receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: BTreeSet<PortfolioId>,
        affirmation_count: Option<AffirmationCount>,
    ) -> DispatchResultWithPostInfo {
        let filtered_legs = Self::base_affirm_with_receipts(
            origin,
            id,
            receipt_details,
            portfolios,
            affirmation_count,
        )?;
        let instruction_asset_count = filtered_legs.unfiltered_asset_count();
        let weight_limit = Self::execute_scheduled_instruction_weight_limit(
            instruction_asset_count.fungible(),
            instruction_asset_count.non_fungible(),
            instruction_asset_count.off_chain(),
        );
        // Schedule instruction to be executed in the next block (expected) if conditions are met.
        Self::maybe_schedule_instruction(InstructionAffirmsPending::<T>::get(id), id, weight_limit);
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
        portfolios: BTreeSet<PortfolioId>,
        affirmation_count: Option<AffirmationCount>,
    ) -> DispatchResultWithPostInfo {
        let filtered_legs =
            Self::base_affirm_instruction(origin, id, portfolios, affirmation_count)?;
        let instruction_asset_count = filtered_legs.unfiltered_asset_count();
        let weight_limit = Self::execute_scheduled_instruction_weight_limit(
            instruction_asset_count.fungible(),
            instruction_asset_count.non_fungible(),
            instruction_asset_count.off_chain(),
        );
        // Schedule the instruction if conditions are met
        Self::maybe_schedule_instruction(InstructionAffirmsPending::<T>::get(id), id, weight_limit);
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
        receipt: Option<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: BTreeSet<PortfolioId>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        match receipt {
            Some(receipt) => {
                Self::base_affirm_with_receipts(origin, id, vec![receipt], portfolios, None)?
            }
            None => Self::base_affirm_instruction(origin, id, portfolios, None)?,
        };
        Self::execute_settle_on_affirmation_instruction(
            id,
            InstructionAffirmsPending::<T>::get(id),
            InstructionDetails::<T>::get(id).settlement_type,
            caller_did,
            weight_meter,
        )?;
        Ok(())
    }

    fn execute_settle_on_affirmation_instruction(
        id: InstructionId,
        affirms_pending: u64,
        settlement_type: SettlementType<T::BlockNumber>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // We assume `settlement_type == SettleOnAffirmation`,
        // to be defensive, however, this is checked before instruction execution.
        if settlement_type == SettlementType::SettleOnAffirmation && affirms_pending == 0 {
            // We use execute_instruction here directly
            // and not the execute_instruction_retryable variant
            // because direct settlement is not retryable.
            Self::execute_instruction(id, caller_did, weight_meter)?;
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
            let user_affirmation = UserAffirmations::<T>::get(portfolio, id);
            ensure!(
                expected_statuses.contains(&user_affirmation),
                Error::<T>::UnexpectedAffirmationStatus
            );
        }
        Ok(())
    }

    /// Returns [`FilteredLegs`] where the orginal set is all legs in the instruction of the given
    /// `id` and the subset of legs are all legs where the sender is in the given `portfolio`.
    fn filtered_legs(id: InstructionId, portfolio: &BTreeSet<PortfolioId>) -> FilteredLegs {
        let instruction_legs: Vec<(LegId, Leg)> = InstructionLegs::<T>::iter_prefix(&id).collect();
        FilteredLegs::filter_sender(instruction_legs, portfolio)
    }

    fn get_instruction_asset_count(id: &InstructionId) -> AssetCount {
        // Get the weight limit for the instruction
        let legs: Vec<(LegId, Leg)> = InstructionLegs::<T>::iter_prefix(id).collect();
        AssetCount::from_legs(&legs)
    }

    fn base_update_venue_signers(
        did: IdentityId,
        venue_id: VenueId,
        signers: Vec<T::AccountId>,
        add_signers: bool,
    ) -> DispatchResult {
        // Ensure venue exists & sender is its creator.
        Self::ensure_venue_creator(&venue_id, did)?;

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
        caller_pid: Option<PortfolioId>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo {
        let origin_data = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
        let caller_did = origin_data.primary_did;

        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(&inst_id).collect();
        let inst_asset_count = AssetCount::from_legs(&inst_legs);
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::reject_instruction_common(
                inst_asset_count.fungible() as u32,
                inst_asset_count.non_fungible() as u32,
                inst_asset_count.off_chain() as u32,
            ),
        )?;

        let inst_status = InstructionStatuses::<T>::get(inst_id);
        ensure!(
            inst_status != InstructionStatus::Unknown
                && inst_status != InstructionStatus::LockedForExecution,
            Error::<T>::InvalidInstructionStatusForRejection
        );

        let inst_details = InstructionDetails::<T>::get(&inst_id);
        Self::ensure_valid_caller(
            caller_did,
            origin_data.secondary_key.as_ref(),
            caller_pid,
            inst_details.venue_id,
            &inst_id,
            &inst_legs,
            weight_meter,
        )?;

        Self::release_locks(&inst_id, &inst_legs)?;

        // Note: ignoring the error here is fine, since the instruction might not be scheduled yet
        let _ = T::Scheduler::cancel_named(inst_id.execution_name());

        let inst_asset_count = AssetCount::from_legs(&inst_legs);
        Self::prune_instruction(&inst_id, &inst_legs, &inst_asset_count, weight_meter)?;
        InstructionStatuses::<T>::insert(
            inst_id,
            InstructionStatus::Rejected(System::<T>::block_number()),
        );

        Self::deposit_event(Event::InstructionRejected(caller_did, inst_id));

        // returns the actual weight of the call
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
        if let Some(_) = venue_id {
            // Avoids reading the storage multiple times for the same asset_id
            let mut tickers: BTreeSet<AssetId> = BTreeSet::new();
            for (_, leg) in instruction_legs {
                if let Some(asset_id) = leg.asset_id() {
                    Self::ensure_venue_filtering(&mut tickers, *asset_id, &venue_id)?;
                }
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
        if let Some(venue_id) = venue_id {
            if tickers.insert(asset_id) && VenueFiltering::<T>::get(asset_id) {
                ensure!(
                    VenueAllowList::<T>::get(asset_id, venue_id),
                    Error::<T>::UnauthorizedVenue
                );
            }
        }
        Ok(())
    }

    /// Executes the instruction of the given `id` returning the consumed weight for executing the instruction.
    fn base_execute_scheduled_instruction(
        inst_id: InstructionId,
        weight_meter: &mut WeightMeter,
    ) -> PostDispatchInfo {
        // Note: Ignores the error because we want to emit the event and update the instruction status.
        // All other state modification are wrapped in a transaction.
        let _ = Self::execute_instruction_retryable(inst_id, SettlementDID.as_id(), weight_meter);
        PostDispatchInfo::from(Some(weight_meter.consumed()))
    }

    /// Returns `Ok` if the leg is valid, otherwise returns an error.
    /// See also: [`Pallet::ensure_valid_fungible_leg`], [`Pallet::ensure_valid_nft_leg`] and [`Pallet::ensure_valid_off_chain_leg`].
    #[rustfmt::skip]
    fn ensure_valid_leg(
        leg: &Leg,
        venue_id: &Option<VenueId>,
        tickers: &mut BTreeSet<AssetId>,
        instruction_asset_count: &mut AssetCount,
    ) -> DispatchResult {
        match leg {
            Leg::Fungible { sender, receiver, asset_id, amount } => {
                ensure!(sender.did != receiver.did, Error::<T>::SameSenderReceiver);
                Self::ensure_valid_fungible_leg(tickers, *asset_id, *amount, venue_id)?;
                instruction_asset_count
                    .try_add_fungible()
                    .map_err(|_| Error::<T>::MaxNumberOfFungibleAssetsExceeded)?;
                Ok(())
            }
            Leg::NonFungible { sender, receiver, nfts } => {
                ensure!(sender.did != receiver.did, Error::<T>::SameSenderReceiver);
                Self::ensure_valid_nft_leg(tickers, &nfts, venue_id)?;
                instruction_asset_count
                    .try_add_non_fungible(&nfts)
                    .map_err(|_| Error::<T>::MaxNumberOfNFTsExceeded)?;
                Ok(())
            }
            Leg::OffChain { sender_identity, receiver_identity, amount, .. } => {
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

    /// Manually executes an instruction.
    fn base_manual_execution(
        origin: OriginFor<T>,
        inst_id: InstructionId,
        caller_pid: Option<PortfolioId>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo {
        let origin_data = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
        let caller_did = origin_data.primary_did;
        let caller_sk = origin_data.secondary_key.as_ref();

        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(inst_id).collect();
        let inst_asset_count = AssetCount::from_legs(&inst_legs);

        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::manual_execution_common(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
            ),
        )?;

        let inst_details = InstructionDetails::<T>::get(&inst_id);
        Self::ensure_valid_caller(
            caller_did,
            caller_sk,
            caller_pid,
            inst_details.venue_id,
            &inst_id,
            &inst_legs,
            weight_meter,
        )?;

        let inst_memo = InstructionMemos::<T>::get(&inst_id);

        match InstructionStatuses::<T>::get(&inst_id) {
            InstructionStatus::Pending => {
                Self::ensure_manual_settlement_type(inst_details.settlement_type)?;
                Self::validate_execute_instruction_conditions(
                    &inst_id,
                    &inst_legs,
                    &inst_asset_count,
                    weight_meter,
                )?;
                Self::transfer_assets(
                    inst_id,
                    inst_legs.clone(),
                    inst_memo,
                    caller_did,
                    &inst_asset_count,
                    weight_meter,
                )?;
            }
            InstructionStatus::Failed => {
                Self::validate_execute_instruction_conditions(
                    &inst_id,
                    &inst_legs,
                    &inst_asset_count,
                    weight_meter,
                )?;
                Self::transfer_assets(
                    inst_id,
                    inst_legs.clone(),
                    inst_memo,
                    caller_did,
                    &inst_asset_count,
                    weight_meter,
                )?;
            }
            InstructionStatus::LockedForExecution => {
                Self::transfer_assets(
                    inst_id,
                    inst_legs.clone(),
                    inst_memo,
                    caller_did,
                    &inst_asset_count,
                    weight_meter,
                )?;
            }
            _ => return Err(Error::<T>::InvalidInstructionStatusForExecution.into()),
        }

        InstructionStatuses::<T>::insert(
            inst_id,
            InstructionStatus::Success(System::<T>::block_number()),
        );
        Self::prune_instruction(&inst_id, &inst_legs, &inst_asset_count, weight_meter)?;
        Self::deposit_event(Event::InstructionExecuted(caller_did, inst_id));
        Self::deposit_event(Event::SettlementManuallyExecuted(caller_did, inst_id));
        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }

    /// Returns `Ok` if any of the following conditions is true:
    /// - The caller has the permission of the given portfolio and that portfolio is a party in the instruction.
    /// - The caller is the venue creator of the instruction.
    /// - The caller is an instruction mediator.
    /// - The caller is a counter party in an offchain leg.
    fn ensure_valid_caller(
        caller_did: IdentityId,
        caller_sk: Option<&SecondaryKey<T::AccountId>>,
        caller_pid: Option<PortfolioId>,
        venue_id: Option<VenueId>,
        inst_id: &InstructionId,
        inst_legs: &[(LegId, Leg)],
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Checks if the caller has the permission of the given portfolio and that portfolio is a party in the instruction
        if let Some(caller_pid) = caller_pid {
            Self::check_accrue(
                weight_meter,
                <T as Config>::WeightInfo::valid_caller_portfolio(),
            )?;

            T::Portfolio::ensure_portfolio_custody_and_permission(
                caller_pid, caller_did, caller_sk,
            )?;
            Self::ensure_portfolio_belongs_to_instruction(&inst_id, &caller_pid)?;
            return Ok(());
        }

        // Checks if the caller is the venue creator
        if let Some(venue_id) = venue_id {
            Self::check_accrue(
                weight_meter,
                <T as Config>::WeightInfo::valid_caller_venue(),
            )?;

            if Self::ensure_venue_creator(&venue_id, caller_did).is_ok() {
                return Ok(());
            }
        }

        // Checks if the caller is a mediator
        if Self::ensure_mediator(&inst_id, &caller_did).is_ok() {
            Self::check_accrue(
                weight_meter,
                <T as Config>::WeightInfo::valid_caller_mediator(),
            )?;

            return Ok(());
        }

        // Checks if the caller is a counter party in an offchain leg
        if Self::is_offchain_party(&caller_did, inst_legs) {
            return Ok(());
        }

        Err(Error::<T>::CallerIsNotAParty.into())
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

    /// Returns `Ok` if [`SettlementType::SettleManual`] and the `block_number` is reached.
    fn ensure_manual_settlement_type(
        settlement_type: SettlementType<T::BlockNumber>,
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

    /// Returns `Ok` if the `pid` is a party in the instruction of the given `inst_id`.
    fn ensure_portfolio_belongs_to_instruction(
        inst_id: &InstructionId,
        pid: &PortfolioId,
    ) -> DispatchResult {
        match UserAffirmations::<T>::get(pid, inst_id) {
            AffirmationStatus::Unknown => Err(Error::<T>::CallerIsNotAParty.into()),
            AffirmationStatus::Pending | AffirmationStatus::Affirmed => Ok(()),
        }
    }

    /// Ensures the all receipts are valid. A receipt is considered valid if the signer is allowed by the venue,
    /// if the receipt has not been used before, if the receipt's `leg_id` and `instruction_id` are referencing the
    /// correct instruction/leg and if its signature is valid.
    #[rustfmt::skip]
    fn ensure_valid_receipts_details(
        venue_id: Option<VenueId>,
        instruction_id: InstructionId,
        receipts_details: &[ReceiptDetails<T::AccountId, T::OffChainSignature>],
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

            if let Some(venue_id) = venue_id {
                ensure!(
                    VenueSigners::<T>::get(venue_id, receipt_details.signer()),
                    Error::<T>::UnauthorizedSigner
                );
            }

            ensure!(
                !ReceiptsUsed::<T>::get(receipt_details.signer(), &receipt_details.uid()),
                Error::<T>::ReceiptAlreadyClaimed
            );

            let leg = InstructionLegs::<T>::get(&instruction_id, &receipt_details.leg_id())
                .ok_or(Error::<T>::LegNotFound)?;
            match leg {
                Leg::OffChain { sender_identity, receiver_identity, ticker, amount } => {
                    ensure!(
                        OffChainAffirmations::<T>::get(instruction_id, receipt_details.leg_id())
                            == AffirmationStatus::Pending,
                        Error::<T>::UnexpectedAffirmationStatus
                    );
                    let receipt = Receipt::new(
                        receipt_details.uid(),
                        instruction_id,
                        receipt_details.leg_id(),
                        sender_identity,
                        receiver_identity,
                        ticker,
                        amount,
                    );
                    ensure!(
                        receipt_details
                            .signature()
                            .verify(&receipt.encode()[..], receipt_details.signer()),
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
        WeightMeter::from_limit(minimum_weight, weight_limit).map_err(|_| {
            DispatchErrorWithPostInfo {
                post_info: Some(weight_limit).into(),
                error: Error::<T>::InputWeightIsLessThanMinimum.into(),
            }
        })
    }

    fn base_withdraw_affirmation(
        origin: OriginFor<T>,
        id: InstructionId,
        portfolios: BTreeSet<PortfolioId>,
        affirmation_count: Option<AffirmationCount>,
    ) -> DispatchResultWithPostInfo {
        let (did, secondary_key, details) =
            Self::ensure_origin_perm_and_instruction_validity(origin, id, false)?;
        let filtered_legs = Self::unsafe_withdraw_instruction_affirmation(
            did,
            id,
            portfolios,
            secondary_key.as_ref(),
            affirmation_count,
        )?;
        if details.settlement_type == SettlementType::SettleOnAffirmation {
            // Cancel the scheduled task for the execution of a given instruction.
            let _fix_this = T::Scheduler::cancel_named(id.execution_name());
        }
        Ok(PostDispatchInfo::from(Some(
            Self::withdraw_affirmation_actual_weight(
                filtered_legs.sender_asset_count().clone(),
                filtered_legs.receiver_asset_count().clone(),
                filtered_legs.unfiltered_asset_count().off_chain(),
            ),
        )))
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
            let instruction_asset_count = Self::get_instruction_asset_count(&instruction_id);
            let weight_limit = Self::execute_scheduled_instruction_weight_limit(
                instruction_asset_count.fungible(),
                instruction_asset_count.non_fungible(),
                instruction_asset_count.off_chain(),
            );
            Self::maybe_schedule_instruction(n_pending_affirmations, instruction_id, weight_limit);
        }

        Self::deposit_event(Event::MediatorAffirmationReceived(
            caller_did,
            instruction_id,
            expiry,
        ));
        Ok(())
    }

    /// Removes the mediator's affirmation for the instruction
    fn base_withdraw_affirmation_as_mediator(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
    ) -> DispatchResult {
        let (caller_did, _, instruction) =
            Self::ensure_origin_perm_and_instruction_validity(origin, instruction_id, false)?;

        Self::ensure_mediator_has_affirmed_instruction(&caller_did, &instruction_id)?;

        // Updates the mediator's affirmation status to pending and add one to the number of pending affirmations
        InstructionMediatorsAffirmations::<T>::insert(
            instruction_id,
            caller_did,
            MediatorAffirmationStatus::Pending,
        );
        let n_pending_before_withdrawal =
            InstructionAffirmsPending::<T>::mutate(instruction_id, |n| {
                let before = n.clone();
                *n = n.saturating_add(1);
                before
            });
        if n_pending_before_withdrawal == 0
            && instruction.settlement_type == SettlementType::SettleOnAffirmation
        {
            // Cancel the scheduled task
            let _ = T::Scheduler::cancel_named(instruction_id.execution_name());
        }
        Self::deposit_event(Event::MediatorAffirmationWithdrawn(
            caller_did,
            instruction_id,
        ));
        Ok(())
    }

    /// If the caller is a mediator and all conditions for executing the instruction are met,
    /// updates the instruction status to `LockedForExecution`.
    fn base_lock_instruction(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let caller_did = pallet_identity::Pallet::<T>::ensure_perms(origin.clone())?;

        Self::ensure_mediator_has_affirmed_instruction(&caller_did, &instruction_id)?;

        let inst_details = InstructionDetails::<T>::get(&instruction_id);
        ensure!(
            inst_details.settlement_type == SettlementType::SettleOnComplianceCheck,
            Error::<T>::UnexpectedSettlementType
        );

        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(&instruction_id).collect();
        let inst_asset_count = AssetCount::from_legs(&inst_legs);

        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::lock_instruction_common(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
                inst_asset_count.off_chain(),
            ),
        )?;

        Self::validate_execute_instruction_conditions(
            &instruction_id,
            &inst_legs,
            &inst_asset_count,
            weight_meter,
        )?;

        InstructionStatuses::<T>::insert(instruction_id, InstructionStatus::LockedForExecution);
        LockedTimestamp::<T>::insert(instruction_id, pallet_timestamp::Pallet::<T>::get());

        Self::deposit_event(Event::InstructionLocked(caller_did, instruction_id));
        Ok(())
    }

    /// Returns `Ok` if `did` is a mediator and has affirmed the instruction.
    fn ensure_mediator_has_affirmed_instruction(
        did: &IdentityId,
        instruction_id: &InstructionId,
    ) -> DispatchResult {
        let mediator_affirmation_status =
            InstructionMediatorsAffirmations::<T>::get(instruction_id, did);

        match mediator_affirmation_status {
            MediatorAffirmationStatus::Affirmed { .. } => Ok(()),
            MediatorAffirmationStatus::Unknown => Err(Error::<T>::CallerIsNotAMediator.into()),
            MediatorAffirmationStatus::Pending => {
                Err(Error::<T>::UnexpectedAffirmationStatus.into())
            }
        }
    }

    /// Returns `Ok` if the instruction status is `Pending` or `Failed`.
    fn ensure_instruction_is_pending_or_failed(instruction_id: &InstructionId) -> DispatchResult {
        let instruction_status = InstructionStatuses::<T>::get(instruction_id);

        match instruction_status {
            InstructionStatus::Pending | InstructionStatus::Failed => Ok(()),
            _ => Err(Error::<T>::InvalidInstructionStatusForExecution.into()),
        }
    }

    /// Transfer all assets in the instruction. Only the following checks are assessed:
    /// - If the instruction is locked for execution, the locking period must be below the maximum.
    /// - All assets must be locked.
    /// - All senders must have the required balance.
    #[rustfmt::skip]
    fn transfer_assets(
        instruction_id: InstructionId,
        legs: Vec<(LegId, Leg)>,
        instruction_memo: Option<Memo>,
        caller_did: IdentityId,
        inst_asset_count: &AssetCount,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::transfer_assets(
                inst_asset_count.fungible(),
                inst_asset_count.non_fungible(),
            ),
        )?;

        Self::ensure_maximum_locking_period_not_exceeded(&instruction_id, weight_meter)?;

        for (_, leg) in legs {
            match leg {
                Leg::Fungible { sender, receiver, asset_id, amount } => {
                    T::Portfolio::unlock_tokens(&sender, &asset_id, amount)?;
                    Asset::<T>::simplified_fungible_transfer(
                        asset_id,
                        sender,
                        receiver,
                        amount,
                        instruction_id.clone(),
                        instruction_memo.clone(),
                        caller_did,
                        weight_meter,
                    )?;
                }
                Leg::NonFungible { sender, receiver, nfts } => {
                    for nft_id in nfts.ids() {
                        T::Portfolio::unlock_nft(&sender, nfts.asset_id(), nft_id)?;
                    }
                    Nft::<T>::simplified_nft_transfer(
                        sender,
                        receiver,
                        nfts,
                        instruction_id,
                        instruction_memo.clone(),
                        caller_did,
                    )?;
                }
                Leg::OffChain { .. } => continue,
            }
        }

        Ok(())
    }

    /// Returns `Ok` if the maximum locking period was not exceeded.
    fn ensure_maximum_locking_period_not_exceeded(
        inst_id: &InstructionId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::check_accrue(
            weight_meter,
            <T as Config>::WeightInfo::maximum_lock_period(),
        )?;

        let inst_status = InstructionStatuses::<T>::get(inst_id);

        if inst_status == InstructionStatus::LockedForExecution {
            let locked_timestamp =
                LockedTimestamp::<T>::get(inst_id).ok_or(Error::<T>::LockTimestampNotFound)?;

            let now = pallet_timestamp::Pallet::<T>::get();
            ensure!(
                now - locked_timestamp <= T::MaximumLockPeriod::get(),
                Error::<T>::ExceededMaximumLockingPeriod
            );
        }

        Ok(())
    }

    /// Consumes the given weight after checking that it can be consumed. Returns an error otherwise.
    fn check_accrue(weight_meter: &mut WeightMeter, weight: Weight) -> DispatchResult {
        weight_meter
            .check_accrue(weight)
            .map_err(|_| Error::<T>::WeightLimitExceeded)?;
        Ok(())
    }

    /// Returns the worst case weight for an instruction with `f` fungible legs, `n` nfts being transferred and `o` offchain assets.
    fn execute_scheduled_instruction_weight_limit(f: u32, n: u32, o: u32) -> Weight {
        <T as Config>::WeightInfo::execute_scheduled_instruction(f + 1, n + 1, o + 1)
    }

    /// Returns the minimum weight for calling the `execute_scheduled_instruction` function.
    fn execute_scheduled_instruction_minimum_weight() -> Weight {
        <T as Config>::WeightInfo::execute_scheduled_instruction(0, 0, 1)
    }

    /// Returns the worst case weight for an instruction with `f` fungible legs, `n` nfts being transferred and `o` offchain assets.
    fn execute_manual_instruction_weight_limit(f: u32, n: u32, o: u32) -> Weight {
        <T as Config>::WeightInfo::execute_manual_instruction(f + 1, n + 1, o + 1)
    }

    /// Returns the minimum weight for calling the `execute_manual_instruction` extrinsic.
    /// For the minimum weight the instruction must have on leg and of `SettlementType::SettleOnComplianceCheck`.
    pub fn execute_manual_instruction_minimum_weight() -> Weight {
        let common_weight = <T as Config>::WeightInfo::manual_execution_common(0, 0, 1);
        let caller_validation_weight = <T as Config>::WeightInfo::valid_caller_mediator();
        let transfer_weight = <T as Config>::WeightInfo::transfer_assets(0, 0);
        let lock_assessement_weight = <T as Config>::WeightInfo::maximum_lock_period();
        let prune_weight = <T as Config>::WeightInfo::prune_instruction(0, 0, 1);
        common_weight
            .saturating_add(caller_validation_weight)
            .saturating_add(transfer_weight)
            .saturating_add(lock_assessement_weight)
            .saturating_add(prune_weight)
    }

    /// Returns the minimum weight for calling the `lock_instruction` function.
    pub fn lock_instruction_minimum_weight() -> Weight {
        let lock_common_weight = <T as Config>::WeightInfo::lock_instruction_common(0, 0, 1);
        let validate_common_weight =
            <T as Config>::WeightInfo::validate_execute_instruction_conditions_common(0, 0, 1);
        lock_common_weight.saturating_add(validate_common_weight)
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

    /// Returns the weight for calling `withdraw_affirmation` while considering the `sender_asset_count` for the sender and`receiver_asset_count`
    /// for the receiver, and `n_offchain` offchain legs.
    fn withdraw_affirmation_actual_weight(
        sender_asset_count: AssetCount,
        receiver_asset_count: AssetCount,
        n_offchain: u32,
    ) -> Weight {
        let affirmation_count =
            AffirmationCount::new(sender_asset_count, receiver_asset_count, n_offchain);
        <T as Config>::WeightInfo::withdraw_affirmation_input(Some(affirmation_count), 0)
    }

    /// Returns the miminum weight for calling the `reject_instruction` extrinsic.
    fn reject_instruction_minimum_weight() -> Weight {
        let reject_common = <T as Config>::WeightInfo::reject_instruction_common(1, 0, 0);
        let caller_validation = <T as Config>::WeightInfo::valid_caller_mediator();
        let prune = <T as Config>::WeightInfo::prune_instruction(1, 0, 0);

        reject_common
            .saturating_add(caller_validation)
            .saturating_add(prune)
    }

    fn reject_instruction_weight(inst_asset_count: &AssetCount) -> Weight {
        <T as Config>::WeightInfo::reject_instruction(inst_asset_count)
    }

    pub fn get_actual_weight(call: &Call<T>) -> Option<Weight> {
        match call {
            Call::affirm_instruction { id, portfolios } => {
                let filtered_legs = Self::filtered_legs(*id, &portfolios);
                Some(Self::affirm_instruction_actual_weight(
                    *filtered_legs.sender_asset_count(),
                    *filtered_legs.receiver_asset_count(),
                ))
            }
            Call::affirm_with_receipts { id, portfolios, .. } => {
                let filtered_legs = Self::filtered_legs(*id, &portfolios);
                Some(Self::affirm_with_receipts_actual_weight(
                    *filtered_legs.sender_asset_count(),
                    *filtered_legs.receiver_asset_count(),
                    filtered_legs.unfiltered_asset_count().off_chain(),
                ))
            }
            Call::withdraw_affirmation { id, portfolios } => {
                let filtered_legs = Self::filtered_legs(*id, &portfolios);
                Some(Self::withdraw_affirmation_actual_weight(
                    *filtered_legs.sender_asset_count(),
                    *filtered_legs.receiver_asset_count(),
                    filtered_legs.unfiltered_asset_count().off_chain(),
                ))
            }
            Call::reject_instruction { id, .. } => {
                let asset_count = Self::get_instruction_asset_count(id);
                Some(Self::reject_instruction_weight(&asset_count))
            }
            _ => None,
        }
    }

    /// Returns an instance of [`ExecuteInstructionInfo`].
    pub fn execute_instruction_info(
        instruction_id: &InstructionId,
    ) -> Option<ExecuteInstructionInfo> {
        if !InstructionDetails::<T>::contains_key(instruction_id) {
            return None;
        }

        let caller_did = SettlementDID.as_id();
        let instruction_asset_count = Self::get_instruction_asset_count(instruction_id);
        let mut weight_meter =
            WeightMeter::max_limit(Self::execute_manual_instruction_minimum_weight());
        match Self::execute_instruction_retryable(*instruction_id, caller_did, &mut weight_meter) {
            Ok(_) => Some(ExecuteInstructionInfo::new(
                instruction_asset_count.fungible(),
                instruction_asset_count.non_fungible(),
                instruction_asset_count.off_chain(),
                weight_meter.consumed(),
                None,
            )),
            Err(e) => Some(ExecuteInstructionInfo::new(
                instruction_asset_count.fungible(),
                instruction_asset_count.non_fungible(),
                instruction_asset_count.off_chain(),
                weight_meter.consumed(),
                Some(e.into()),
            )),
        }
    }

    /// Returns an instance of [`AffirmationCount`].
    pub fn affirmation_count(
        instruction_id: InstructionId,
        portfolios: Vec<PortfolioId>,
    ) -> AffirmationCount {
        let portfolios = portfolios.into_iter().collect::<BTreeSet<_>>();
        let filtered_legs = Self::filtered_legs(instruction_id, &portfolios);
        AffirmationCount::new(
            filtered_legs.sender_asset_count().clone(),
            filtered_legs.receiver_asset_count().clone(),
            filtered_legs.unfiltered_asset_count().off_chain(),
        )
    }

    /// Returns a vector containing all errors for the execution. An empty vec means there's no error.
    #[rustfmt::skip]
    pub fn execute_instruction_report(
        inst_id: &InstructionId,
        weight_meter: &mut WeightMeter,
    ) -> Vec<DispatchError> {
        let mut execution_errors = Vec::new();

        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(inst_id).collect();
        if InstructionAffirmsPending::<T>::get(inst_id) != 0 {
            execution_errors.push(Error::<T>::NotAllAffirmationsHaveBeenReceived.into());
        }

        if let Err(e) = Self::ensure_instruction_is_pending_or_failed(inst_id) {
            execution_errors.push(e);
        }

        if let Err(e) = Self::validate_mediators_affirmations(inst_id, weight_meter) {
            execution_errors.push(e);
        }

        if let Err(e) = Self::ensure_no_missing_affirmation(inst_id, &inst_legs) {
            execution_errors.push(e);
        }

        let inst_details = InstructionDetails::<T>::get(inst_id);
        if let Err(e) = Self::ensure_allowed_venue(&inst_legs, inst_details.venue_id) {
            execution_errors.push(e);
        }

        if let Err(e) = Self::ensure_assets_can_be_transferred(inst_id, &inst_legs, weight_meter)
        {
            execution_errors.push(e);
        }

        if let Err(e) = Self::ensure_maximum_locking_period_not_exceeded(inst_id, weight_meter) {
            execution_errors.push(e);
        }

        execution_errors
    }
}
