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
use frame_support::dispatch::{
    DispatchError, DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo,
    PostDispatchInfo,
};
use frame_support::storage::{
    with_transaction as frame_storage_with_transaction, TransactionOutcome,
};
use frame_support::traits::schedule::{DispatchTime, Named};
use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::{decl_error, decl_module, decl_storage, ensure, IterableStorageDoubleMap};
use frame_system::{ensure_root, RawOrigin};
use sp_runtime::traits::{One, Verify};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::convert::TryFrom;
use sp_std::prelude::*;
use sp_std::vec;

use pallet_base::{ensure_string_limited, try_next_post};
use pallet_identity::PermissionedCallOriginData;
use polymesh_common_utilities::constants::queue_priority::SETTLEMENT_INSTRUCTION_EXECUTION_PRIORITY;
use polymesh_common_utilities::traits::portfolio::PortfolioSubTrait;
pub use polymesh_common_utilities::traits::settlement::{Event, RawEvent, WeightInfo};
use polymesh_common_utilities::traits::{asset, compliance_manager, identity, nft, CommonConfig};
use polymesh_common_utilities::with_transaction;
use polymesh_common_utilities::SystematicIssuers::Settlement as SettlementDID;
use polymesh_primitives::settlement::{
    AffirmationStatus, ExecuteInstructionInfo, Instruction, InstructionId, InstructionInfo,
    InstructionMemo, InstructionStatus, Leg, LegAsset, LegId, LegStatus, LegV2, Receipt,
    ReceiptDetails, SettlementType, TransferData, Venue, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::{
    storage_migrate_on, storage_migration_ver, IdentityId, PortfolioId, SecondaryKey, Ticker,
    WeightMeter,
};

type Identity<T> = pallet_identity::Module<T>;
type System<T> = frame_system::Pallet<T>;
type Asset<T> = pallet_asset::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type Nft<T> = pallet_nft::Module<T>;
type EnsureValidInstructionResult<AccountId, Moment, BlockNumber> = Result<
    (
        IdentityId,
        Option<SecondaryKey<AccountId>>,
        Instruction<Moment, BlockNumber>,
    ),
    DispatchError,
>;

pub trait Config:
    asset::Config
    + CommonConfig
    + compliance_manager::Config
    + frame_system::Config
    + identity::Config
    + nft::Config
    + pallet_timestamp::Config
{
    /// The overarching event type.
    type RuntimeEvent: From<Event<Self>> + Into<<Self as frame_system::Config>::RuntimeEvent>;

    /// A call type used by the scheduler.
    type Proposal: From<Call<Self>> + Into<<Self as identity::Config>::Proposal>;

    /// Scheduler of settlement instructions.
    type Scheduler: Named<Self::BlockNumber, <Self as Config>::Proposal, Self::SchedulerOrigin>;

    /// Maximum number of fungible assets that can be in a single instruction.
    type MaxNumberOfFungibleAssets: Get<u32>;

    /// Weight information for extrinsic of the settlement pallet.
    type WeightInfo: WeightInfo;

    /// Maximum number of NFTs that can be transferred in a leg.
    type MaxNumberOfNFTsPerLeg: Get<u32>;

    /// Maximum number of NFTs that can be transferred in a instruction.
    type MaxNumberOfNFTs: Get<u32>;
}

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
        /// Instruction leg amount can't be zero.
        ZeroAmount,
        /// Instruction settlement block has not yet been reached.
        InstructionSettleBlockNotReached,
        /// The caller is not a party of this instruction.
        CallerIsNotAParty,
        /// Expected a different type of asset in a leg.
        InvalidLegAsset,
        /// The number of nfts being transferred in the instruction was exceeded.
        MaxNumberOfNFTsExceeded,
        /// The maximum number of nfts being transferred in one leg was exceeded.
        MaxNumberOfNFTsPerLegExceeded,
        /// The given number of nfts being transferred was underestimated.
        NumberOfTransferredNFTsUnderestimated,
        /// Deprecated function has been called on a v2 instruction.
        DeprecatedCallOnV2Instruction,
        /// Off-chain receipts are not accepted for non-fungible tokens.
        ReceiptForNonFungibleAsset,
        /// The maximum weight limit for executing the function was exceeded.
        WeightLimitExceeded,
        /// The input weight is less than the minimum required.
        InputWeightIsLessThanMinimum,
    }
}

storage_migration_ver!(1);

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
        pub InstructionDetails get(fn instruction_details):
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
        StorageVersion get(fn storage_version) build(|_| Version::new(1)): Version;
        /// Instruction memo
        InstructionMemos get(fn memo): map hasher(twox_64_concat) InstructionId => Option<InstructionMemo>;
        /// Instruction statuses. instruction_id -> InstructionStatus
        InstructionStatuses get(fn instruction_status):
            map hasher(twox_64_concat) InstructionId => InstructionStatus<T::BlockNumber>;
        /// Legs under an instruction. (instruction_id, leg_id) -> Leg
        pub InstructionLegsV2 get(fn instruction_legsv2):
            double_map hasher(twox_64_concat) InstructionId, hasher(twox_64_concat) LegId => LegV2;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::RuntimeOrigin {
        type Error = Error<T>;

        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            storage_migrate_on!(StorageVersion, 1, {
                migration::migrate_v1::<T>();
            });

            Weight::zero()
        }

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
            <T as Config>::WeightInfo::execute_scheduled_instruction(legs.len() as u32, 0)
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
            let legs: Vec<LegV2> = legs.into_iter().map(|leg| leg.into()).collect();
            Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, None, true)?;
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
            <T as Config>::WeightInfo::execute_scheduled_instruction(legs.len() as u32, 0)
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
            let legs: Vec<LegV2> = legs.into_iter().map(|leg| leg.into()).collect();
            with_transaction(|| {
                let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();
                let legs_count = legs.iter().filter(|l| portfolios_set.contains(&l.from)).count() as u32;
                let instruction_id = Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, None, true)?;
                Self::affirm_and_maybe_schedule_instruction(origin, instruction_id, portfolios_set.into_iter(), legs_count, None)
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
        pub fn affirm_instruction(origin, id: InstructionId, portfolios: Vec<PortfolioId>, max_legs_count: u32,) -> DispatchResult {
            Self::affirm_and_maybe_schedule_instruction(origin, id, portfolios.into_iter(), max_legs_count, None)
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
            Self::unsafe_withdraw_instruction_affirmation(did, id, portfolios_set, secondary_key.as_ref(), max_legs_count, None)?;
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
        pub fn reject_instruction(origin, id: InstructionId, portfolio: PortfolioId, num_of_legs: u32) -> DispatchResult {
            Self::base_reject_instruction(origin, id, portfolio, num_of_legs, None)
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
        pub fn placeholder_claim_receipt(_origin) {}

        /// Placeholder for removed `unclaim_receipt`
        #[weight = 1_000]
        pub fn placeholder_unclaim_receipt(_origin) {}

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
        #[weight = <T as Config>::WeightInfo::execute_scheduled_instruction(*legs_count, 0)]
        fn execute_scheduled_instruction(origin, id: InstructionId, legs_count: u32) -> DispatchResultWithPostInfo {
            Self::ensure_root_origin(origin)?;
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_scheduled_instruction_minimum_weight(),
                Self::execute_scheduled_instruction_weight_limit(legs_count, 0),
            )?;
            Self::base_execute_scheduled_instruction(id, &mut weight_meter).map_err(|e| {
                DispatchErrorWithPostInfo {
                    post_info: Some(weight_meter.consumed()).into(),
                    error: e.error,
                }
            })
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

            <InstructionStatuses<T>>::try_mutate(id, |status| {
                ensure!(*status == InstructionStatus::Failed, Error::<T>::InstructionNotFailed);
                *status = InstructionStatus::Pending;
                Result::<_, Error<T>>::Ok(())
            })?;

            // Schedule instruction to be executed in the next block.
            let execution_at = System::<T>::block_number() + One::one();
            let instruction_legs = Self::get_instruction_legs(&id);
            let instruction_data = Self::get_transfer_data(&instruction_legs);
            let weight_limit = Self::execute_scheduled_instruction_weight_limit(
                instruction_data.non_fungible(),
                instruction_data.fungible(),
            );
            Self::schedule_instruction(id, execution_at, weight_limit);
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
            <T as Config>::WeightInfo::execute_scheduled_instruction(legs.len() as u32, 0)
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
            let legs: Vec<LegV2> = legs.into_iter().map(|leg| leg.into()).collect();
            Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, instruction_memo, true)?;
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
            <T as Config>::WeightInfo::execute_scheduled_instruction(legs.len() as u32, 0)
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
            let legs: Vec<LegV2> = legs.into_iter().map(|leg| leg.into()).collect();
            with_transaction(|| {
                let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();
                let legs_count = legs.iter().filter(|l| portfolios_set.contains(&l.from)).count() as u32;
                let instruction_id = Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, instruction_memo, true)?;
                Self::affirm_and_maybe_schedule_instruction(origin, instruction_id, portfolios_set.into_iter(), legs_count, None)
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
        #[weight = <T as Config>::WeightInfo::execute_manual_weight_limit(weight_limit, legs_count)]
        pub fn execute_manual_instruction(
            origin,
            id: InstructionId,
            legs_count: u32,
            portfolio: Option<PortfolioId>,
            weight_limit: Option<Weight>
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_manual_instruction_minimum_weight(),
                weight_limit.unwrap_or(Self::execute_manual_instruction_weight_limit(legs_count)),
            )?;
            Self::base_execute_manual_instruction(origin, id, legs_count, portfolio, &mut weight_meter)
                .map_err(|e| DispatchErrorWithPostInfo {
                    post_info: Some(weight_meter.consumed()).into(),
                    error: e.error,
                })
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
        #[weight =
            <T as Config>::WeightInfo::add_instruction_with_memo_v2(legs.len() as u32)
            .saturating_add( <T as Config>::WeightInfo::execute_scheduled_instruction_v2(legs))
        ]
        pub fn add_instruction_with_memo_v2(
            origin,
            venue_id: VenueId,
            settlement_type: SettlementType<T::BlockNumber>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<LegV2>,
            instruction_memo: Option<InstructionMemo>,
        ) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, instruction_memo, false)?;
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
        #[weight =
            <T as Config>::WeightInfo::add_and_affirm_instruction_with_memo_v2_legs(legs)
            .saturating_add( <T as Config>::WeightInfo::execute_scheduled_instruction_v2(legs))
        ]
        pub fn add_and_affirm_instruction_with_memo_v2(
            origin,
            venue_id: VenueId,
            settlement_type: SettlementType<T::BlockNumber>,
            trade_date: Option<T::Moment>,
            value_date: Option<T::Moment>,
            legs: Vec<LegV2>,
            portfolios: Vec<PortfolioId>,
            instruction_memo: Option<InstructionMemo>,
        ) -> DispatchResult {
            let did = Identity::<T>::ensure_perms(origin.clone())?;
            with_transaction(|| {
                let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();
                let transfer_data = TransferData::from_legs(&legs).map_err(|_| Error::<T>::MaxNumberOfNFTsExceeded)?;
                let instruction_id = Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, instruction_memo, false)?;
                Self::affirm_and_maybe_schedule_instruction(
                    origin,
                    instruction_id,
                    portfolios_set.into_iter(),
                    transfer_data.fungible(),
                    Some(transfer_data.non_fungible()),
                )
            })
        }

        /// Provide affirmation to an existing instruction.
        ///
        /// # Arguments
        /// * `id` - Instruction id to affirm.
        /// * `portfolios` - Portfolios that the sender controls and wants to affirm this instruction.
        /// * `fungible_transfers` - number of fungible transfers in the instruction.
        /// * `nfts_transfers` - total number of NFTs being transferred in the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::affirm_instruction_v2(*fungible_transfers, *nfts_transfers)]
        pub fn affirm_instruction_v2(origin, id: InstructionId, portfolios: Vec<PortfolioId>, fungible_transfers: u32, nfts_transfers: u32) -> DispatchResult {
            Self::affirm_and_maybe_schedule_instruction(
                origin,
                id,
                portfolios.into_iter(),
                fungible_transfers,
                Some(nfts_transfers),
            )
        }

        /// Withdraw an affirmation for a given instruction.
        ///
        /// # Arguments
        /// * `id` - Instruction id for that affirmation get withdrawn.
        /// * `portfolios` - Portfolios that the sender controls and wants to withdraw affirmation.
        /// * `fungible_transfers` - number of fungible transfers in the instruction.
        /// * `nfts_transfers` - total number of NFTs being transferred in the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::withdraw_affirmation_v2(*fungible_transfers, *nfts_transfers)]
        pub fn withdraw_affirmation_v2(origin, id: InstructionId, portfolios: Vec<PortfolioId>, fungible_transfers: u32, nfts_transfers: u32) -> DispatchResult {
            let (did, secondary_key, details) = Self::ensure_origin_perm_and_instruction_validity(origin, id, false)?;
            let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();

            // Withdraw an affirmation.
            Self::unsafe_withdraw_instruction_affirmation(did, id, portfolios_set, secondary_key.as_ref(), fungible_transfers, Some(nfts_transfers))?;
            if details.settlement_type == SettlementType::SettleOnAffirmation {
                // Cancel the scheduled task for the execution of a given instruction.
                let _fix_this = T::Scheduler::cancel_named(id.execution_name());
            }
            Ok(())
        }

        /// Rejects an existing instruction.
        ///
        /// # Arguments
        /// * `id` - Instruction id to reject.
        /// * `portfolio` - Portfolio to reject the instruction.
        /// * `fungible_transfers` - number of fungible transfers in the instruction.
        /// * `nfts_transfers` - total number of NFTs being transferred in the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::reject_instruction_v2(*fungible_transfers, *nfts_transfers)]
        pub fn reject_instruction_v2(origin, id: InstructionId, portfolio: PortfolioId, fungible_transfers: u32, nfts_transfers: u32) -> DispatchResult {
            Self::base_reject_instruction(origin, id, portfolio, fungible_transfers, Some(nfts_transfers))
        }

        /// Root callable extrinsic, used as an internal call to execute a scheduled settlement instruction.
        #[weight = <T as Config>::WeightInfo::execute_scheduled_instruction(*fungible_transfers, *nfts_transfers)]
        fn execute_scheduled_instruction_v2(
            origin,
            id: InstructionId,
            fungible_transfers: u32,
            nfts_transfers: u32
        ) -> DispatchResultWithPostInfo {
            Self::ensure_root_origin(origin)?;
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_scheduled_instruction_minimum_weight(),
                Self::execute_scheduled_instruction_weight_limit(fungible_transfers, nfts_transfers),
            )?;
            Self::base_execute_scheduled_instruction(id, &mut weight_meter).map_err(|e| {
                DispatchErrorWithPostInfo {
                    post_info: Some(weight_meter.consumed()).into(),
                    error: e.error,
                }
            })
        }

        /// Root callable extrinsic, used as an internal call to execute a scheduled settlement instruction.
        #[weight = (*weight_limit).max(<T as Config>::WeightInfo::execute_scheduled_instruction(0, 0))]
        fn execute_scheduled_instruction_v3(origin, id: InstructionId, weight_limit: Weight) -> DispatchResultWithPostInfo {
            Self::ensure_root_origin(origin)?;
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_scheduled_instruction_minimum_weight(),
                weight_limit,
            )?;
            Self::base_execute_scheduled_instruction(id, &mut weight_meter).map_err(|e| {
                DispatchErrorWithPostInfo {
                    post_info: Some(weight_meter.consumed()).into(),
                    error: e.error,
                }
            })
        }
    }
}

impl<T: Config> Module<T> {
    fn lock_via_leg(leg: &LegV2) -> DispatchResult {
        match &leg.asset {
            LegAsset::Fungible { ticker, amount } => {
                T::Portfolio::lock_tokens(&leg.from, &ticker, *amount)
            }
            LegAsset::NonFungible(nfts) => with_transaction(|| {
                for nft_id in nfts.ids() {
                    T::Portfolio::lock_nft(&leg.from, nfts.ticker(), &nft_id)?;
                }
                Ok(())
            }),
        }
    }

    fn unlock_via_leg(leg: &LegV2) -> DispatchResult {
        match &leg.asset {
            LegAsset::Fungible { ticker, amount } => {
                T::Portfolio::unlock_tokens(&leg.from, &ticker, *amount)
            }
            LegAsset::NonFungible(nfts) => with_transaction(|| {
                for nft_id in nfts.ids() {
                    T::Portfolio::unlock_nft(&leg.from, nfts.ticker(), &nft_id)?;
                }
                Ok(())
            }),
        }
    }

    /// Ensure origin call permission and the given instruction validity.
    fn ensure_origin_perm_and_instruction_validity(
        origin: <T as frame_system::Config>::RuntimeOrigin,
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
        legs: Vec<LegV2>,
        memo: Option<InstructionMemo>,
        emit_deprecated_event: bool,
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
        Self::venue_for_management(venue_id, did)?;

        // Verifies if all legs are valid.
        let instruction_info = Self::ensure_valid_legs(&legs, venue_id)?;

        // Advance and get next `instruction_id`.
        let instruction_id = InstructionCounter::try_mutate(try_next_post::<T, _>)?;

        let instruction = Instruction {
            instruction_id,
            venue_id,
            settlement_type,
            created_at: Some(<pallet_timestamp::Pallet<T>>::get()),
            trade_date,
            value_date,
        };

        InstructionStatuses::<T>::insert(instruction_id, InstructionStatus::Pending);

        // Write data to storage.
        for counter_party in instruction_info.parties() {
            UserAffirmations::insert(counter_party, instruction_id, AffirmationStatus::Pending);
        }

        if let SettlementType::SettleOnBlock(block_number) = settlement_type {
            let weight_limit = Self::execute_scheduled_instruction_weight_limit(
                instruction_info.fungible_transfers(),
                instruction_info.nfts_transferred(),
            );
            Self::schedule_instruction(instruction_id, block_number, weight_limit);
        }

        <InstructionDetails<T>>::insert(instruction_id, instruction);

        InstructionAffirmsPending::insert(
            instruction_id,
            u64::try_from(instruction_info.parties().len()).unwrap_or_default(),
        );
        VenueInstructions::insert(venue_id, instruction_id, ());
        if let Some(ref memo) = memo {
            InstructionMemos::insert(instruction_id, &memo);
        }

        if emit_deprecated_event {
            let legs: Result<Vec<Leg>, &str> = legs
                .into_iter()
                .map(|leg_v2| Leg::try_from(leg_v2))
                .collect();
            let legs: Vec<Leg> = legs.map_err(|_| Error::<T>::InvalidLegAsset)?;
            legs.iter().enumerate().for_each(|(index, leg)| {
                InstructionLegs::insert(instruction_id, LegId(index as u64), leg.clone())
            });
            Self::deposit_event(RawEvent::InstructionCreated(
                did,
                venue_id,
                instruction_id,
                settlement_type,
                trade_date,
                value_date,
                legs,
                memo,
            ))
        } else {
            legs.iter().enumerate().for_each(|(index, leg)| {
                InstructionLegsV2::insert(instruction_id, LegId(index as u64), leg.clone())
            });
            Self::deposit_event(RawEvent::InstructionV2Created(
                did,
                venue_id,
                instruction_id,
                settlement_type,
                trade_date,
                value_date,
                legs,
                memo,
            ));
        }

        Ok(instruction_id)
    }

    /// Makes sure the legs are valid. For both types of assets the sender and receiver must be different,
    /// the amount being transferred must be greater than zero, and if filtering is enabled the venue list is also checked.
    /// The number of fungible an non fungible assets in the legs must be within the valid limits allowed.
    /// Returns a set of the unique counter parties involved in the legs.
    fn ensure_valid_legs(
        legs: &[LegV2],
        venue_id: VenueId,
    ) -> Result<InstructionInfo, DispatchError> {
        let mut nfts_transfers = 0;
        let mut fungible_transfers = 0;
        let mut parties = BTreeSet::new();
        let mut tickers = BTreeSet::new();
        for leg in legs {
            ensure!(leg.from != leg.to, Error::<T>::SameSenderReceiver);
            match &leg.asset {
                LegAsset::Fungible { ticker, amount } => {
                    ensure!(*amount > 0, Error::<T>::ZeroAmount);
                    Self::ensure_venue_filtering(&mut tickers, ticker.clone(), &venue_id)?;
                    fungible_transfers += 1;
                }
                LegAsset::NonFungible(nfts) => {
                    <Nft<T>>::ensure_within_nfts_transfer_limits(&nfts)?;
                    Self::ensure_venue_filtering(&mut tickers, nfts.ticker().clone(), &venue_id)?;
                    <Nft<T>>::ensure_no_duplicate_nfts(&nfts)?;
                    nfts_transfers += nfts.len();
                }
            }
            parties.insert(leg.from);
            parties.insert(leg.to);
        }
        ensure!(
            nfts_transfers <= T::MaxNumberOfNFTs::get() as usize,
            Error::<T>::MaxNumberOfNFTsExceeded
        );
        ensure!(
            fungible_transfers <= T::MaxNumberOfFungibleAssets::get(),
            Error::<T>::InstructionHasTooManyLegs
        );
        Ok(InstructionInfo::new(
            parties,
            TransferData::new(fungible_transfers, nfts_transfers as u32),
        ))
    }

    fn unsafe_withdraw_instruction_affirmation(
        did: IdentityId,
        id: InstructionId,
        portfolios: BTreeSet<PortfolioId>,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        fungible_transfers: u32,
        nfts_transfers: Option<u32>,
    ) -> Result<TransferData, DispatchError> {
        // checks custodianship of portfolios and affirmation status
        Self::ensure_portfolios_and_affirmation_status(
            id,
            &portfolios,
            did,
            secondary_key,
            &[AffirmationStatus::Affirmed],
        )?;
        // Unlock tokens that were previously locked during the affirmation
        let (instruction_data, filtered_legs) =
            Self::filtered_legs(&id, &portfolios, fungible_transfers, nfts_transfers)?;
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
        Ok(instruction_data)
    }

    fn ensure_instruction_validity(
        id: InstructionId,
        is_execute: bool,
    ) -> Result<Instruction<T::Moment, T::BlockNumber>, DispatchError> {
        let details = Self::instruction_details(id);
        ensure!(
            Self::instruction_status(id) != InstructionStatus::Unknown,
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
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        if let Err(e) = Self::execute_instruction(id, weight_meter) {
            InstructionStatuses::<T>::insert(id, InstructionStatus::Failed);
            return Err(e);
        }
        Self::prune_instruction(id, true);
        Ok(())
    }

    fn execute_instruction(
        instruction_id: InstructionId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // Verifies that there are no pending affirmations for the given instruction
        ensure!(
            Self::instruction_affirms_pending(instruction_id) == 0,
            Error::<T>::InstructionFailed
        );

        // Verifies that the instruction is not in a Failed or in an Unknown state
        ensure!(
            Self::instruction_status(instruction_id) == InstructionStatus::Pending,
            Error::<T>::InstructionNotPending
        );

        let venue_id = Self::instruction_details(instruction_id).venue_id;

        // NB: The order of execution of the legs matter in some edge cases around compliance.
        // E.g: Consider a token with a total supply of 100 and maximum percentage ownership of 10%.
        // In a given moment, Alice owns 10 tokens, Bob owns 5 and Charlie owns 0.
        // Now, consider one instruction with two legs: 1. Alice transfers 5 tokens to Charlie; 2. Bob transfers 5 tokens to Alice;
        // If the second leg gets executed before the first leg, Alice will momentarily hold 15% of the asset and hence the settlement will fail compliance.
        let mut instruction_legs: Vec<(LegId, LegV2)> = Self::get_instruction_legs(&instruction_id);
        instruction_legs.sort_by_key(|leg_id_leg| leg_id_leg.0);

        let instruction_data = Self::get_transfer_data(&instruction_legs);
        weight_meter
            .check_accrue(<T as Config>::WeightInfo::execute_instruction_paused(
                instruction_data.fungible(),
                instruction_data.non_fungible(),
            ))
            .map_err(|_| Error::<T>::WeightLimitExceeded)?;

        // Verifies if the venue is allowed for all tickers in the instruction
        Self::ensure_allowed_venue(&instruction_legs, venue_id)?;

        // Attempts to release the locks and transfer all fungible an non fungible assets
        if let Err(leg_id) = frame_storage_with_transaction(|| {
            Self::release_asset_locks_and_transfer_pending_legs(
                instruction_id,
                &instruction_legs,
                weight_meter,
            )
        })? {
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

        Self::deposit_event(RawEvent::InstructionExecuted(
            SettlementDID.as_id(),
            instruction_id,
        ));
        Ok(())
    }

    fn release_asset_locks_and_transfer_pending_legs(
        instruction_id: InstructionId,
        instruction_legs: &[(LegId, LegV2)],
        weight_meter: &mut WeightMeter,
    ) -> TransactionOutcome<Result<Result<(), LegId>, DispatchError>> {
        Self::unchecked_release_locks(instruction_id, instruction_legs);
        for (leg_id, leg) in instruction_legs {
            if Self::instruction_leg_status(instruction_id, leg_id) == LegStatus::ExecutionPending {
                match &leg.asset {
                    LegAsset::Fungible { ticker, amount } => {
                        if <Asset<T>>::base_transfer(
                            leg.from,
                            leg.to,
                            &ticker,
                            *amount,
                            weight_meter,
                        )
                        .is_err()
                        {
                            return TransactionOutcome::Rollback(Ok(Err(*leg_id)));
                        }
                    }
                    LegAsset::NonFungible(nfts) => {
                        if <Nft<T>>::base_nft_transfer(&leg.from, &leg.to, &nfts, weight_meter)
                            .is_err()
                        {
                            return TransactionOutcome::Rollback(Ok(Err(*leg_id)));
                        }
                    }
                }
            }
        }
        TransactionOutcome::Commit(Ok(Ok(())))
    }

    fn prune_instruction(id: InstructionId, executed: bool) {
        let legs: Vec<(LegId, LegV2)> = Self::drain_instruction_legs(&id);
        let details = <InstructionDetails<T>>::take(id);
        VenueInstructions::remove(details.venue_id, id);
        #[allow(deprecated)]
        <InstructionLegStatus<T>>::remove_prefix(id, None);
        InstructionAffirmsPending::remove(id);
        #[allow(deprecated)]
        AffirmsReceived::remove_prefix(id, None);

        if executed {
            InstructionStatuses::<T>::insert(
                id,
                InstructionStatus::Success(System::<T>::block_number()),
            );
        } else {
            InstructionStatuses::<T>::insert(
                id,
                InstructionStatus::Rejected(System::<T>::block_number()),
            );
        }

        // We remove duplicates in memory before triggering storage actions
        let mut counter_parties = BTreeSet::new();
        for (_, leg) in &legs {
            counter_parties.insert(leg.from);
            counter_parties.insert(leg.to);
        }
        for counter_party in &counter_parties {
            UserAffirmations::remove(&counter_party, id);
        }
    }

    pub fn unsafe_affirm_instruction(
        did: IdentityId,
        id: InstructionId,
        portfolios: BTreeSet<PortfolioId>,
        fungible_transfers: u32,
        nfts_trasferred: Option<u32>,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
    ) -> Result<TransferData, DispatchError> {
        // Checks portfolio's custodian and if it is a counter party with a pending affirmation.
        Self::ensure_portfolios_and_affirmation_status(
            id,
            &portfolios,
            did,
            secondary_key,
            &[AffirmationStatus::Pending],
        )?;

        let (instruction_data, filtered_legs) =
            Self::filtered_legs(&id, &portfolios, fungible_transfers, nfts_trasferred)?;
        with_transaction(|| {
            for (leg_id, leg_details) in filtered_legs {
                Self::lock_via_leg(&leg_details)?;
                <InstructionLegStatus<T>>::insert(id, leg_id, LegStatus::ExecutionPending);
            }
            Ok(())
        })
        .map_err(|e: DispatchError| e)?;

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
        Ok(instruction_data)
    }

    // Unclaims all receipts for an instruction
    // Should only be used if user is unclaiming, or instruction has failed
    fn unsafe_unclaim_receipts(id: InstructionId, legs: &[(LegId, LegV2)]) {
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

    fn unchecked_release_locks(id: InstructionId, instruction_legs: &[(LegId, LegV2)]) {
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
    fn maybe_schedule_instruction(affirms_pending: u64, id: InstructionId, weight_limit: Weight) {
        if affirms_pending == 0
            && Self::instruction_details(id).settlement_type == SettlementType::SettleOnAffirmation
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
    fn schedule_instruction(id: InstructionId, execution_at: T::BlockNumber, weight_limit: Weight) {
        let call = Call::<T>::execute_scheduled_instruction_v3 { id, weight_limit }.into();
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
        origin: <T as frame_system::Config>::RuntimeOrigin,
        id: InstructionId,
        receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: Vec<PortfolioId>,
        fungible_transfers: u32,
    ) -> Result<TransferData, DispatchError> {
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

            let leg = Self::get_instruction_leg(&id, &receipt.leg_id);
            if let LegAsset::NonFungible(_nfts) = leg.asset {
                return Err(Error::<T>::ReceiptForNonFungibleAsset.into());
            }
            ensure!(
                portfolios_set.contains(&leg.from),
                Error::<T>::PortfolioMismatch
            );

            let (asset, amount) = leg.asset.ticker_and_amount();
            ensure!(
                !pallet_asset::Tokens::contains_key(&asset),
                Error::<T>::UnauthorizedVenue
            );

            let msg = Receipt {
                receipt_uid: receipt.receipt_uid,
                from: leg.from,
                to: leg.to,
                asset,
                amount,
            };
            ensure!(
                receipt.signature.verify(&msg.encode()[..], &receipt.signer),
                Error::<T>::InvalidSignature
            );
        }

        let (instruction_data, filtered_legs) =
            Self::filtered_legs(&id, &portfolios_set, fungible_transfers, None)?;
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
        Ok(instruction_data)
    }

    pub fn base_affirm_instruction(
        origin: <T as frame_system::Config>::RuntimeOrigin,
        id: InstructionId,
        portfolios: impl Iterator<Item = PortfolioId>,
        fungible_transfers: u32,
        nfts_transferred: Option<u32>,
    ) -> Result<TransferData, DispatchError> {
        let (did, sk, _) = Self::ensure_origin_perm_and_instruction_validity(origin, id, false)?;
        let portfolios_set = portfolios.collect::<BTreeSet<_>>();

        // Provide affirmation to the instruction
        Self::unsafe_affirm_instruction(
            did,
            id,
            portfolios_set,
            fungible_transfers,
            nfts_transferred,
            sk.as_ref(),
        )
    }

    // It affirms the instruction and may schedule the instruction
    // depends on the settlement type.
    pub fn affirm_with_receipts_and_maybe_schedule_instruction(
        origin: <T as frame_system::Config>::RuntimeOrigin,
        id: InstructionId,
        receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: Vec<PortfolioId>,
        fungible_transfers: u32,
    ) -> DispatchResult {
        let instruction_data = Self::base_affirm_with_receipts(
            origin,
            id,
            receipt_details,
            portfolios,
            fungible_transfers,
        )?;
        let weight_limit = Self::execute_scheduled_instruction_weight_limit(
            instruction_data.fungible(),
            instruction_data.non_fungible(),
        );
        // Schedule instruction to be execute in the next block (expected) if conditions are met.
        Self::maybe_schedule_instruction(Self::instruction_affirms_pending(id), id, weight_limit);
        Ok(())
    }

    /// Schedule settlement instruction execution in the next block, unless already scheduled.
    /// Used for general purpose settlement.
    pub fn affirm_and_maybe_schedule_instruction(
        origin: <T as frame_system::Config>::RuntimeOrigin,
        id: InstructionId,
        portfolios: impl Iterator<Item = PortfolioId>,
        fungible_transfers: u32,
        nfts_transfers: Option<u32>,
    ) -> DispatchResult {
        let instruction_data = Self::base_affirm_instruction(
            origin,
            id,
            portfolios,
            fungible_transfers,
            nfts_transfers,
        )?;
        let weight_limit = Self::execute_scheduled_instruction_weight_limit(
            instruction_data.fungible(),
            instruction_data.non_fungible(),
        );
        // Schedule the instruction if conditions are met
        Self::maybe_schedule_instruction(Self::instruction_affirms_pending(id), id, weight_limit);
        Ok(())
    }

    /// Affirm with or without receipts, executing the instruction when all affirmations have been received.
    ///
    /// NB - Use this function only in the STO pallet to support DVP settlements.
    pub fn affirm_and_execute_instruction(
        origin: <T as frame_system::Config>::RuntimeOrigin,
        id: InstructionId,
        receipt: Option<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: Vec<PortfolioId>,
        max_legs_count: u32,
        nfts_transferred: Option<u32>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        match receipt {
            Some(receipt) => Self::base_affirm_with_receipts(
                origin,
                id,
                vec![receipt],
                portfolios,
                max_legs_count,
            )?,
            None => Self::base_affirm_instruction(
                origin,
                id,
                portfolios.into_iter(),
                max_legs_count,
                nfts_transferred,
            )?,
        };
        Self::execute_settle_on_affirmation_instruction(
            id,
            Self::instruction_affirms_pending(id),
            Self::instruction_details(id).settlement_type,
            weight_meter,
        )?;
        Self::prune_instruction(id, true);
        Ok(())
    }

    fn execute_settle_on_affirmation_instruction(
        id: InstructionId,
        affirms_pending: u64,
        settlement_type: SettlementType<T::BlockNumber>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // We assume `settlement_type == SettleOnAffirmation`,
        // to be defensive, however, this is checked before instruction execution.
        if settlement_type == SettlementType::SettleOnAffirmation && affirms_pending == 0 {
            // We use execute_instruction here directly
            // and not the execute_instruction_retryable variant
            // because direct settlement is not retryable.
            Self::execute_instruction(id, weight_meter)?;
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

    /// Ensures that the number of fungible and non fungible assets being transferred is under the given limit.
    /// Returns the total number of fungible and non-fungible assets in the instruction.
    fn filtered_legs(
        id: &InstructionId,
        portfolio_set: &BTreeSet<PortfolioId>,
        fungible_transfers: u32,
        nfts_transfers: Option<u32>,
    ) -> Result<(TransferData, Vec<(LegId, LegV2)>), DispatchError> {
        let instruction_legs: Vec<(LegId, LegV2)> = Self::get_instruction_legs(&id);
        let instruction_data = Self::get_transfer_data(&instruction_legs);
        // Gets all legs where the sender is in the given set
        let legs_from_set: Vec<(LegId, LegV2)> = instruction_legs
            .into_iter()
            .filter(|(_, leg_v2)| portfolio_set.contains(&leg_v2.from))
            .collect();
        let transfer_data = Self::get_transfer_data(&legs_from_set);
        Self::ensure_valid_input_cost(&transfer_data, fungible_transfers, nfts_transfers)?;
        Ok((instruction_data, legs_from_set))
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

    fn base_reject_instruction(
        origin: T::RuntimeOrigin,
        id: InstructionId,
        portfolio: PortfolioId,
        fungible_transfers: u32,
        nfts_transfers: Option<u32>,
    ) -> DispatchResult {
        ensure!(
            Self::instruction_status(id) != InstructionStatus::Unknown,
            Error::<T>::UnknownInstruction
        );
        // Gets all legs for the instruction, checks if portfolio is in any of the legs, and validates the input cost.
        let legs_v2: Vec<(LegId, LegV2)> = Self::get_instruction_legs(&id);
        ensure!(
            legs_v2
                .iter()
                .any(|(_, leg_v2)| leg_v2.from == portfolio || leg_v2.to == portfolio),
            Error::<T>::CallerIsNotAParty
        );
        let transfer_data = Self::get_transfer_data(&legs_v2);
        Self::ensure_valid_input_cost(&transfer_data, fungible_transfers, nfts_transfers)?;

        // Verifies if the caller has the right permissions for this call
        let origin_data = Identity::<T>::ensure_origin_call_permissions(origin)?;
        T::Portfolio::ensure_portfolio_custody_and_permission(
            portfolio,
            origin_data.primary_did,
            origin_data.secondary_key.as_ref(),
        )?;

        Self::unsafe_unclaim_receipts(id, &legs_v2);
        Self::unchecked_release_locks(id, &legs_v2);
        let _ = T::Scheduler::cancel_named(id.execution_name());
        Self::prune_instruction(id, false);
        Self::deposit_event(RawEvent::InstructionRejected(origin_data.primary_did, id));
        Ok(())
    }

    /// Returns the number of fungible and non fungible transfers in a slice of legs.
    /// Since this function is only called after the legs have already been inserted, casting is safe.
    fn get_transfer_data(legs_v2: &[(LegId, LegV2)]) -> TransferData {
        let mut transfer_data = TransferData::default();
        for (_, leg_v2) in legs_v2 {
            match &leg_v2.asset {
                LegAsset::Fungible { .. } => transfer_data.add_fungible(),
                LegAsset::NonFungible(nfts) => transfer_data.add_non_fungible(&nfts),
            }
        }
        transfer_data
    }

    /// Returns ok if the number of fungible assets and nfts being transferred is under the input given by the user.
    pub fn ensure_valid_input_cost(
        transfer_data: &TransferData,
        fungible_transfers: u32,
        nfts_transfers: Option<u32>,
    ) -> DispatchResult {
        // Verifies if the number of nfts being transferred is under the limit
        if transfer_data.non_fungible() > 0 {
            let nfts_transfers = nfts_transfers.ok_or(Error::<T>::DeprecatedCallOnV2Instruction)?;
            ensure!(
                transfer_data.non_fungible() <= nfts_transfers,
                Error::<T>::NumberOfTransferredNFTsUnderestimated
            );
        }
        // Verifies if the number of fungible transfers is under the limit
        ensure!(
            transfer_data.fungible() <= fungible_transfers,
            Error::<T>::LegCountTooSmall
        );
        Ok(())
    }

    /// Ensures that all tickers in the instruction that have venue filtering enabled are also
    /// in the venue allowed list.
    fn ensure_allowed_venue(
        instruction_legs: &[(LegId, LegV2)],
        venue_id: VenueId,
    ) -> DispatchResult {
        // Avoids reading the storage multiple times for the same ticker
        let mut tickers: BTreeSet<Ticker> = BTreeSet::new();
        for (_, leg) in instruction_legs {
            let ticker = leg.asset.ticker_and_amount().0;
            Self::ensure_venue_filtering(&mut tickers, ticker, &venue_id)?;
        }
        Ok(())
    }

    /// If `tickers` doesn't contain the given `ticker` and venue_filtering is enabled, ensures that venue_id is in the allowed list
    fn ensure_venue_filtering(
        tickers: &mut BTreeSet<Ticker>,
        ticker: Ticker,
        venue_id: &VenueId,
    ) -> DispatchResult {
        if tickers.insert(ticker) && Self::venue_filtering(ticker) {
            ensure!(
                Self::venue_allow_list(ticker, venue_id),
                Error::<T>::UnauthorizedVenue
            );
        }
        Ok(())
    }

    /// Returns the specified leg for the given instruction and leg id.
    /// If it doesn't exist in the InstructionLegsV2 storage it will be converted from the deprecated InstructionLegs storage.
    pub fn get_instruction_leg(instruction_id: &InstructionId, leg_id: &LegId) -> LegV2 {
        InstructionLegsV2::try_get(instruction_id, leg_id)
            .unwrap_or_else(|_| InstructionLegs::get(instruction_id, leg_id).into())
    }

    /// Returns all legs and their id for the given instruction.
    /// If it doesn't exist in the InstructionLegsV2 storage it will be converted from the deprecated InstructionLegs storage.
    pub fn get_instruction_legs(instruction_id: &InstructionId) -> Vec<(LegId, LegV2)> {
        let instruction_legs: Vec<(LegId, LegV2)> =
            InstructionLegsV2::iter_prefix(instruction_id).collect();

        if instruction_legs.is_empty() {
            return InstructionLegs::iter_prefix(instruction_id)
                .map(|(leg_id, leg)| (leg_id, leg.into()))
                .collect();
        }
        instruction_legs
    }

    /// Removes all legs for the given `instruction_id`, returning a `Vec<(LegId, LegV2)>` containing the removed legs.
    fn drain_instruction_legs(instruction_id: &InstructionId) -> Vec<(LegId, LegV2)> {
        let drained_legs: Vec<(LegId, LegV2)> =
            InstructionLegsV2::drain_prefix(instruction_id).collect();

        if drained_legs.is_empty() {
            return InstructionLegs::drain_prefix(instruction_id)
                .map(|(leg_id, leg)| (leg_id, leg.into()))
                .collect();
        }
        drained_legs
    }

    fn base_execute_scheduled_instruction(
        id: InstructionId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo {
        if let Err(e) = Self::execute_instruction_retryable(id, weight_meter) {
            Self::deposit_event(RawEvent::FailedToExecuteInstruction(id, e));
        }
        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }

    fn base_execute_manual_instruction(
        origin: T::RuntimeOrigin,
        id: InstructionId,
        legs_count: u32,
        portfolio: Option<PortfolioId>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo {
        // check origin has the permissions required and valid instruction
        let (did, sk, instruction_details) =
            Self::ensure_origin_perm_and_instruction_validity(origin, id, true)?;

        // Check for portfolio
        let instruction_legs: Vec<(LegId, LegV2)> = Self::get_instruction_legs(&id);
        match portfolio {
            Some(portfolio) => {
                // Ensure that the caller is a party of this instruction.
                T::Portfolio::ensure_portfolio_custody_and_permission(portfolio, did, sk.as_ref())?;
                ensure!(
                    instruction_legs
                        .iter()
                        .any(|(_, leg)| leg.from == portfolio || leg.to == portfolio),
                    Error::<T>::CallerIsNotAParty
                );
            }
            None => {
                // Ensure venue exists & sender is its creator.
                Self::venue_for_management(instruction_details.venue_id, did)?;
            }
        }

        // check that the instruction leg count matches
        ensure!(
            instruction_legs.len() as u32 <= legs_count,
            Error::<T>::LegCountTooSmall
        );

        // Executes the instruction
        Self::execute_instruction_retryable(id, weight_meter)?;

        Self::deposit_event(RawEvent::SettlementManuallyExecuted(did, id));
        Ok(PostDispatchInfo::from(Some(weight_meter.consumed())))
    }

    /// Returns `Ok` if `origin` represents the root, otherwise returns an `Err` with the consumed weight for this function.
    fn ensure_root_origin(origin: T::RuntimeOrigin) -> Result<(), DispatchErrorWithPostInfo> {
        ensure_root(origin).map_err(|e| DispatchErrorWithPostInfo {
            post_info: Some(<T as Config>::WeightInfo::ensure_root_origin()).into(),
            error: e.into(),
        })
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

    /// Returns the worst case weight for an instruction with `f` fungible legs and `n` nfts being transferred.
    fn execute_scheduled_instruction_weight_limit(f: u32, n: u32) -> Weight {
        <T as Config>::WeightInfo::execute_scheduled_instruction(f, n)
    }

    /// Returns the minimum weight for calling the `execute_scheduled_instruction` function.
    fn execute_scheduled_instruction_minimum_weight() -> Weight {
        <T as Config>::WeightInfo::execute_scheduled_instruction(0, 0)
    }

    /// Returns the worst case weight for an instruction with `l`legs.
    fn execute_manual_instruction_weight_limit(l: u32) -> Weight {
        <T as Config>::WeightInfo::execute_manual_instruction(l)
    }

    /// Returns the minimum weight for calling the `execute_manual_instruction` extrinsic.
    pub fn execute_manual_instruction_minimum_weight() -> Weight {
        <T as Config>::WeightInfo::execute_manual_instruction(0)
    }

    /// Returns an instance of `ExecuteInstructionInfo`, which contains the number of fungible and non fungible assets
    /// in the instruction, and the weight consumed for executing the instruction. If the instruction would fail its
    /// execution, it also contains the error.
    pub fn execute_instruction_info(instruction_id: &InstructionId) -> ExecuteInstructionInfo {
        let instruction_legs: Vec<(LegId, LegV2)> = Self::get_instruction_legs(&instruction_id);
        let transfer_data = Self::get_transfer_data(&instruction_legs);
        let mut weight_meter =
            WeightMeter::max_limit(Self::execute_scheduled_instruction_minimum_weight());

        match Self::execute_instruction_retryable(*instruction_id, &mut weight_meter) {
            Ok(_) => ExecuteInstructionInfo::new(
                transfer_data.fungible(),
                transfer_data.non_fungible(),
                weight_meter.consumed(),
                None,
            ),
            Err(e) => ExecuteInstructionInfo::new(
                transfer_data.fungible(),
                transfer_data.non_fungible(),
                weight_meter.consumed(),
                Some(e.into()),
            ),
        }
    }
}

pub mod migration {
    use super::*;

    mod v1 {
        use super::*;
        use scale_info::TypeInfo;

        /// Old v1 Instruction information.
        #[derive(Encode, Decode, TypeInfo)]
        #[derive(Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
        pub struct Instruction<Moment, BlockNumber> {
            /// Unique instruction id. It is an auto incrementing number
            pub instruction_id: InstructionId,
            /// Id of the venue this instruction belongs to
            pub venue_id: VenueId,
            /// Status of the instruction
            pub status: InstructionStatus<BlockNumber>,
            /// Type of settlement used for this instruction
            pub settlement_type: SettlementType<BlockNumber>,
            /// Date at which this instruction was created
            pub created_at: Option<Moment>,
            /// Date from which this instruction is valid
            pub trade_date: Option<Moment>,
            /// Date after which the instruction should be settled (not enforced)
            pub value_date: Option<Moment>,
        }

        decl_storage! {
            trait Store for Module<T: Config> as Settlement {
                /// Details about an instruction. instruction_id -> instruction_details
                pub InstructionDetails get(fn instruction_details):
                map hasher(twox_64_concat) InstructionId => Instruction<T::Moment, T::BlockNumber>;
                    }
        }

        decl_module! {
            pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
        }
    }

    pub fn migrate_v1<T: Config>() {
        sp_runtime::runtime_logger::RuntimeLogger::init();

        log::info!(" >>> Updating Settlement storage. Migrating Instructions...");
        let total_instructions = v1::InstructionDetails::<T>::drain().fold(
            0usize,
            |total_instructions, (id, instruction_details)| {
                // Migrate Instruction satus.
                InstructionStatuses::<T>::insert(id, instruction_details.status);

                //Migrate Instruction details.
                let instruction = Instruction {
                    instruction_id: id,
                    venue_id: instruction_details.venue_id,
                    settlement_type: instruction_details.settlement_type,
                    created_at: instruction_details.created_at,
                    trade_date: instruction_details.trade_date,
                    value_date: instruction_details.value_date,
                };
                <InstructionDetails<T>>::insert(id, instruction);

                total_instructions + 1
            },
        );

        log::info!(" >>> Migrated {} Instructions.", total_instructions);
    }
}
