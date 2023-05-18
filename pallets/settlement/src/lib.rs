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
//! - `affirm_instruction` - Affirms an existing instruction.
//! - `withdraw_affirmation` - Withdraw an existing affirmation to the given instruction.
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
    AffirmationStatus, AssetCount, ExecuteInstructionInfo, FilteredLegs, Instruction,
    InstructionId, InstructionInfo, InstructionMemo, InstructionStatus, Leg, LegAsset, LegId,
    LegStatus, Receipt, ReceiptDetails, SettlementType, Venue, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::{
    storage_migrate_on, storage_migration_ver, Balance, IdentityId, NFTs, PortfolioId,
    SecondaryKey, Ticker, WeightMeter,
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

    /// Maximum number of off-chain assets that can be transferred in a instruction.
    type MaxNumberOfOffChainAssets: Get<u32>;
}

decl_error! {
    /// Errors for the Settlement module.
    pub enum Error for Module<T: Config> {
        /// Venue does not exist.
        InvalidVenue,
        /// Sender does not have required permissions.
        Unauthorized,
        /// Instruction has not been affirmed.
        InstructionNotAffirmed,
        /// Provided instruction is not pending execution.
        InstructionNotPending,
        /// Provided instruction is not failing execution.
        InstructionNotFailed,
        /// Signer is not authorized by the venue.
        UnauthorizedSigner,
        /// Receipt already used.
        ReceiptAlreadyClaimed,
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
        /// Ticker exists in the polymesh chain.
        UnexpectedOnChainAsset,
        /// Ticker could not be found on chain.
        UnexpectedOFFChainAsset,
        /// Off-Chain assets cannot be locked.
        OffChainAssetCantBeLocked,
        /// Off-Chain assets must be Affirmed with Receipts.
        OffChainAssetMustBeAffirmedWithReceipts,
        /// The given number of off-chain transfers was underestimated.
        NumberOfOffChainTransfersUnderestimated,
        /// No leg with the given id was found
        LegNotFound,
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
        pub InstructionLegs get(fn instruction_legs):
            double_map hasher(twox_64_concat) InstructionId, hasher(twox_64_concat) LegId => Option<Leg>;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::RuntimeOrigin {
        type Error = Error<T>;

        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            storage_migrate_on!(StorageVersion, 1, {
                migration::migrate_to_v2::<T>();
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

        /// Placeholder for removed `add_instruction`
        #[weight = 1_000]
        pub fn placeholder_add_instruction(_origin) {}

        /// Placeholder for removed `add_and_affirm_instruction`
        #[weight = 1_000]
        pub fn placeholder_add_and_affirm_instruction(_origin) {}

        /// Placeholder for removed `affirm_instruction`
        #[weight = 1_000]
        pub fn placeholder_affirm_instruction(_origin)  {}

        /// Placeholder for removed `withdraw_affirmation`
        #[weight = 1_000]
        pub fn placeholder_withdraw_affirmation(_origin) {}

        /// Placeholder for removed `reject_instruction`
        #[weight = 1_000]
        pub fn placeholder_reject_instruction(_origin) {}

        /// Accepts an instruction and claims a signed receipt.
        ///
        /// # Arguments
        /// * `id` - Target instruction id.
        /// * `leg_id` - Target leg id for the receipt
        /// * `receipt_uid` - Receipt ID generated by the signer.
        /// * `signer` - Signer of the receipt.
        /// * `signed_data` - Signed receipt.
        /// * `portfolios` - Portfolios that the sender controls and wants to accept this instruction with.
        /// * `fungible_transfers` - number of fungible transfers in the instruction.
        /// * `nfts_transfers` - total number of NFTs being transferred in the instruction.
        /// * `offchain_transfers` - The number of offchain assets in the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::affirm_with_receipts(*fungible_transfers, *nfts_transfers, *offchain_transfers)]
        pub fn affirm_with_receipts(
            origin,
            id: InstructionId,
            receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
            portfolios: Vec<PortfolioId>,
            fungible_transfers: u32,
            nfts_transfers: u32,
            offchain_transfers: u32
        ) -> DispatchResult {
            let input_cost = AssetCount::new(fungible_transfers, nfts_transfers, offchain_transfers);
            Self::affirm_with_receipts_and_maybe_schedule_instruction(origin, id, receipt_details, portfolios, &input_cost)
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
        #[weight = <T as Config>::WeightInfo::execute_scheduled_instruction(*legs_count, 0, 0)]
        fn execute_scheduled_instruction(origin, id: InstructionId, legs_count: u32) -> DispatchResultWithPostInfo {
            Self::ensure_root_origin(origin)?;
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_scheduled_instruction_minimum_weight(),
                Self::execute_scheduled_instruction_weight_limit(legs_count, 0, 0),
            )?;

            Ok(Self::base_execute_scheduled_instruction(id, &mut weight_meter))
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
            let instruction_legs: Vec<(LegId, Leg)> = InstructionLegs::iter_prefix(&id).collect();
            let instruction_asset_count = AssetCount::from_legs(&instruction_legs);
            let weight_limit = Self::execute_scheduled_instruction_weight_limit(
                instruction_asset_count.non_fungible(),
                instruction_asset_count.fungible(),
                instruction_asset_count.off_chain()
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

        /// Placeholder for removed `add_instruction_with_memo`
        #[weight = 1_000]
        pub fn placeholder_add_instruction_with_memo(_origin) {}

       /// Placeholder for removed `add_and_affirm_instruction_with_memo`
       #[weight = 1_000]
        pub fn placeholder_add_and_affirm_instruction_with_memo(_origin)  {}

        /// Manually execute settlement
        ///
        /// # Arguments
        /// * `id` - Target instruction id to reschedule.
        /// * `_legs_count` - Legs included in this instruction.
        ///
        /// # Errors
        /// * `InstructionNotFailed` - Instruction not in a failed state or does not exist.
        #[weight = <T as Config>::WeightInfo::execute_manual_weight_limit(weight_limit, fungible_transfers, nfts_transfers, offchain_transfers)]
        pub fn execute_manual_instruction(
            origin,
            id: InstructionId,
            portfolio: Option<PortfolioId>,
            fungible_transfers: u32,
            nfts_transfers: u32,
            offchain_transfers: u32,
            weight_limit: Option<Weight>
        ) -> DispatchResultWithPostInfo {
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_manual_instruction_minimum_weight(),
                weight_limit.unwrap_or(Self::execute_manual_instruction_weight_limit(
                    fungible_transfers,
                    nfts_transfers,
                    offchain_transfers,
                )),
            )?;
            let input_cost = AssetCount::new(fungible_transfers, nfts_transfers, offchain_transfers);
            Self::base_execute_manual_instruction(origin, id, portfolio, &input_cost, &mut weight_meter)
                .map_err(|e| DispatchErrorWithPostInfo {
                    post_info: Some(weight_meter.consumed()).into(),
                    error: e.error,
                })
        }

        /// Adds a new instruction.
        ///
        /// # Arguments
        /// * `venue_id` - ID of the venue this instruction belongs to.
        /// * `settlement_type` - Defines if the instruction should be settled in the next block, after receiving all affirmations
        /// or waiting till a specific block.
        /// * `trade_date` - Optional date from which people can interact with this instruction.
        /// * `value_date` - Optional date after which the instruction should be settled (not enforced)
        /// * `legs` - Legs included in this instruction.
        /// * `memo` - Memo field for this instruction.
        /// * `weight_limit` - Optional value that defines a maximum weight for executing the instruction.
        ///
        /// # Weight
        /// `950_000_000 + 1_000_000 * legs.len()`
        #[weight =
            <T as Config>::WeightInfo::add_instruction_legs(legs)
            .saturating_add( <T as Config>::WeightInfo::execute_scheduled_instruction_legs(legs))
        ]
        pub fn add_instruction(
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
        /// * `settlement_type` - Defines if the instruction should be settled in the next block, after receiving all affirmations
        /// or waiting till a specific block.
        /// * `trade_date` - Optional date from which people can interact with this instruction.
        /// * `value_date` - Optional date after which the instruction should be settled (not enforced)
        /// * `legs` - Legs included in this instruction.
        /// * `portfolios` - Portfolios that the sender controls and wants to use in this affirmations.
        /// * `memo` - Memo field for this instruction.
        /// * `weight_limit` - Optional value that defines a maximum weight for executing the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight =
            <T as Config>::WeightInfo::add_and_affirm_instruction_legs(legs)
            .saturating_add( <T as Config>::WeightInfo::execute_scheduled_instruction_legs(legs))
        ]
        pub fn add_and_affirm_instruction(
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
                let instruction_asset_count = AssetCount::try_from_legs(&legs).map_err(|_| Error::<T>::MaxNumberOfNFTsExceeded)?;
                let instruction_id = Self::base_add_instruction(did, venue_id, settlement_type, trade_date, value_date, legs, instruction_memo)?;
                Self::affirm_and_maybe_schedule_instruction(
                    origin,
                    instruction_id,
                    portfolios_set.into_iter(),
                    &instruction_asset_count,
                )
            })
        }

        /// Provide affirmation to an existing instruction.
        ///
        /// # Arguments
        /// * `id` - The `InstructionId` of the instruction to be affirmed.
        /// * `portfolios` - Portfolios that the sender controls and wants to affirm this instruction.
        /// * `fungible_transfers` - number of fungible transfers in the instruction.
        /// * `nfts_transfers` - total number of NFTs being transferred in the instruction.
        /// * `weight_limit` - Optional value that defines a maximum weight for executing the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::affirm_instruction(*fungible_transfers, *nfts_transfers)]
        pub fn affirm_instruction(
            origin,
            id: InstructionId,
            portfolios: Vec<PortfolioId>,
            fungible_transfers: u32,
            nfts_transfers: u32,
        ) -> DispatchResult {
            let input_cost =
                AssetCount::new(fungible_transfers, nfts_transfers, T::MaxNumberOfOffChainAssets::get());
            Self::affirm_and_maybe_schedule_instruction(
                origin,
                id,
                portfolios.into_iter(),
                &input_cost,
            )
        }

        /// Withdraw an affirmation for a given instruction.
        ///
        /// # Arguments
        /// * `id` - Instruction id for that affirmation get withdrawn.
        /// * `portfolios` - Portfolios that the sender controls and wants to withdraw affirmation.
        /// * `fungible_transfers` - number of fungible transfers in the instruction.
        /// * `nfts_transfers` - total number of NFTs being transferred in the instruction.
        /// * `offchain_transfers` - The number of offchain assets in the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::withdraw_affirmation(*fungible_transfers, *nfts_transfers, *offchain_transfers)]
        pub fn withdraw_affirmation(
            origin,
            id: InstructionId,
            portfolios: Vec<PortfolioId>,
            fungible_transfers: u32,
            nfts_transfers: u32,
            offchain_transfers: u32
        ) -> DispatchResult {
            let (did, secondary_key, details) = Self::ensure_origin_perm_and_instruction_validity(origin, id, false)?;
            let portfolios_set = portfolios.into_iter().collect::<BTreeSet<_>>();

            let input_cost = AssetCount::new(fungible_transfers, nfts_transfers, offchain_transfers);
            Self::unsafe_withdraw_instruction_affirmation(did, id, portfolios_set, secondary_key.as_ref(), &input_cost)?;
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
        /// * `offchain_transfers` - The number of offchain assets in the instruction.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::reject_instruction(*fungible_transfers, *nfts_transfers, *offchain_transfers)]
        pub fn reject_instruction(
            origin,
            id: InstructionId,
            portfolio: PortfolioId,
            fungible_transfers: u32,
            nfts_transfers: u32,
            offchain_transfers: u32
        ) -> DispatchResult {
            let input_cost = AssetCount::new(fungible_transfers, nfts_transfers, offchain_transfers);
            Self::base_reject_instruction(origin, id, portfolio, &input_cost)
        }

        /// Root callable extrinsic, used as an internal call to execute a scheduled settlement instruction.
        #[weight = <T as Config>::WeightInfo::execute_scheduled_instruction(*fungible_transfers, *nfts_transfers, 0)]
        fn execute_scheduled_instruction_v2(
            origin,
            id: InstructionId,
            fungible_transfers: u32,
            nfts_transfers: u32
        ) -> DispatchResultWithPostInfo{
            Self::ensure_root_origin(origin)?;
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_scheduled_instruction_minimum_weight(),
                Self::execute_scheduled_instruction_weight_limit(fungible_transfers, nfts_transfers, 0),
            )?;
            Ok(Self::base_execute_scheduled_instruction(id, &mut weight_meter))
        }

        /// Root callable extrinsic, used as an internal call to execute a scheduled settlement instruction.
        #[weight = (*weight_limit).max(<T as Config>::WeightInfo::execute_scheduled_instruction(0, 0, 0))]
        fn execute_scheduled_instruction_v3(
            origin,
            id: InstructionId,
            weight_limit: Weight
        ) -> DispatchResultWithPostInfo {
            Self::ensure_root_origin(origin)?;
            let mut weight_meter = Self::ensure_valid_weight_meter(
                Self::execute_scheduled_instruction_minimum_weight(),
                weight_limit,
            )?;
            Ok(Self::base_execute_scheduled_instruction(id, &mut weight_meter))
        }
    }
}

impl<T: Config> Module<T> {
    fn lock_via_leg(leg: &Leg) -> DispatchResult {
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
            LegAsset::OffChain { .. } => Err(Error::<T>::OffChainAssetCantBeLocked.into()),
        }
    }

    fn unlock_via_leg(leg: &Leg) -> DispatchResult {
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
            LegAsset::OffChain { .. } => Err(Error::<T>::OffChainAssetCantBeLocked.into()),
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
        legs: Vec<Leg>,
        memo: Option<InstructionMemo>,
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
                instruction_info.off_chain(),
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
        ));

        Ok(instruction_id)
    }

    /// Makes sure the legs are valid. For all types of assets the sender and receiver must be different,
    /// the amount being transferred must be greater than zero, and if filtering is enabled the venue list is also checked.
    /// The number of each asset type in the legs must be within the valid limits allowed.
    /// Returns a set of the unique counter parties involved in the legs.
    fn ensure_valid_legs(
        legs: &[Leg],
        venue_id: VenueId,
    ) -> Result<InstructionInfo, DispatchError> {
        let mut instruction_asset_count = AssetCount::default();
        let mut parties = BTreeSet::new();
        let mut tickers = BTreeSet::new();
        for leg in legs {
            ensure!(leg.from != leg.to, Error::<T>::SameSenderReceiver);
            match &leg.asset {
                LegAsset::Fungible { ticker, amount } => {
                    Self::ensure_valid_fungible_leg(&mut tickers, *ticker, *amount, &venue_id)?;
                    instruction_asset_count
                        .try_add_fungible()
                        .map_err(|_| Error::<T>::MaxNumberOfFungibleAssetsExceeded)?;
                }
                LegAsset::NonFungible(nfts) => {
                    Self::ensure_valid_nft_leg(&mut tickers, &nfts, &venue_id)?;
                    instruction_asset_count
                        .try_add_non_fungible(&nfts)
                        .map_err(|_| Error::<T>::MaxNumberOfNFTsExceeded)?;
                }
                LegAsset::OffChain { ticker, amount } => {
                    Self::ensure_valid_off_chain_leg(&mut tickers, *ticker, *amount, &venue_id)?;
                    instruction_asset_count
                        .try_add_off_chain()
                        .map_err(|_| Error::<T>::MaxNumberOfOffChainAssetsExceeded)?;
                }
            }
            parties.insert(leg.from);
            parties.insert(leg.to);
        }
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
        Ok(InstructionInfo::new(parties, instruction_asset_count))
    }

    fn unsafe_withdraw_instruction_affirmation(
        did: IdentityId,
        id: InstructionId,
        portfolios: BTreeSet<PortfolioId>,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        input_cost: &AssetCount,
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
        let filtered_legs = Self::filtered_legs(id, &portfolios, input_cost)?;
        for (leg_id, leg_details) in filtered_legs.legs() {
            match Self::instruction_leg_status(id, leg_id) {
                LegStatus::ExecutionToBeSkipped(signer, receipt_uid) => {
                    // Receipt was claimed for this instruction. Therefore, no token unlocking is required, we just unclaim the receipt.
                    <ReceiptsUsed<T>>::insert(&signer, receipt_uid, false);
                    Self::deposit_event(RawEvent::ReceiptUnclaimed(
                        did,
                        id,
                        *leg_id,
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
        Ok(filtered_legs)
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
        let mut instruction_legs: Vec<(LegId, Leg)> =
            InstructionLegs::iter_prefix(&instruction_id).collect();
        instruction_legs.sort_by_key(|leg_id_leg| leg_id_leg.0);

        let instruction_asset_count = AssetCount::from_legs(&instruction_legs);
        weight_meter
            .check_accrue(<T as Config>::WeightInfo::execute_instruction_paused(
                instruction_asset_count.fungible(),
                instruction_asset_count.non_fungible(),
                instruction_asset_count.off_chain(),
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
        instruction_legs: &[(LegId, Leg)],
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
                    LegAsset::OffChain { .. } => {
                        // TODO: off_chain
                    }
                }
            }
        }
        TransactionOutcome::Commit(Ok(Ok(())))
    }

    fn prune_instruction(id: InstructionId, executed: bool) {
        let drained_legs: Vec<(LegId, Leg)> = InstructionLegs::drain_prefix(&id).collect();
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
        for (_, leg) in &drained_legs {
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
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        input_cost: &AssetCount,
    ) -> Result<FilteredLegs, DispatchError> {
        // Checks portfolio's custodian and if it is a counter party with a pending affirmation.
        Self::ensure_portfolios_and_affirmation_status(
            id,
            &portfolios,
            did,
            secondary_key,
            &[AffirmationStatus::Pending],
        )?;

        let filtered_legs = Self::filtered_legs(id, &portfolios, input_cost)?;
        with_transaction(|| {
            for (leg_id, leg) in filtered_legs.legs() {
                ensure!(
                    !leg.asset.is_off_chain(),
                    Error::<T>::OffChainAssetMustBeAffirmedWithReceipts
                );
                Self::lock_via_leg(&leg)?;
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
        Ok(filtered_legs)
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
        input_cost: &AssetCount,
    ) -> Result<FilteredLegs, DispatchError> {
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

            let leg = InstructionLegs::get(&id, &receipt.leg_id).ok_or(Error::<T>::LegNotFound)?;
            ensure!(
                leg.asset.is_off_chain(),
                Error::<T>::ReceiptForInvalidLegType
            );
            ensure!(
                portfolios_set.contains(&leg.from) || portfolios_set.contains(&leg.to),
                Error::<T>::PortfolioMismatch
            );

            let (asset, amount) = leg.asset.ticker_and_amount();
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

        let filtered_legs = Self::filtered_legs(id, &portfolios_set, input_cost)?;
        // Lock tokens that do not have a receipt attached to their leg.
        with_transaction(|| {
            for (leg_id, leg) in filtered_legs.legs() {
                // Receipt for the leg was provided
                if let Some(receipt) = receipt_details
                    .iter()
                    .find(|receipt| receipt.leg_id == *leg_id)
                {
                    <InstructionLegStatus<T>>::insert(
                        id,
                        leg_id,
                        LegStatus::ExecutionToBeSkipped(
                            receipt.signer.clone(),
                            receipt.receipt_uid,
                        ),
                    );
                } else if let Err(_) = Self::lock_via_leg(&leg) {
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
        Ok(filtered_legs)
    }

    pub fn base_affirm_instruction(
        origin: <T as frame_system::Config>::RuntimeOrigin,
        id: InstructionId,
        portfolios: impl Iterator<Item = PortfolioId>,
        input_cost: &AssetCount,
    ) -> Result<FilteredLegs, DispatchError> {
        let (did, sk, _) = Self::ensure_origin_perm_and_instruction_validity(origin, id, false)?;
        let portfolios_set = portfolios.collect::<BTreeSet<_>>();
        // Provide affirmation to the instruction
        Self::unsafe_affirm_instruction(did, id, portfolios_set, sk.as_ref(), input_cost)
    }

    /// It affirms the instruction and may schedule the instruction
    /// depends on the settlement type.
    pub fn affirm_with_receipts_and_maybe_schedule_instruction(
        origin: <T as frame_system::Config>::RuntimeOrigin,
        id: InstructionId,
        receipt_details: Vec<ReceiptDetails<T::AccountId, T::OffChainSignature>>,
        portfolios: Vec<PortfolioId>,
        input_cost: &AssetCount,
    ) -> DispatchResult {
        let filtered_legs =
            Self::base_affirm_with_receipts(origin, id, receipt_details, portfolios, input_cost)?;
        let instruction_asset_count = filtered_legs.instruction_asset_count();
        let weight_limit = Self::execute_scheduled_instruction_weight_limit(
            instruction_asset_count.fungible(),
            instruction_asset_count.non_fungible(),
            instruction_asset_count.off_chain(),
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
        input_cost: &AssetCount,
    ) -> DispatchResult {
        let filtered_legs = Self::base_affirm_instruction(origin, id, portfolios, input_cost)?;
        let instruction_asset_count = filtered_legs.instruction_asset_count();
        let weight_limit = Self::execute_scheduled_instruction_weight_limit(
            instruction_asset_count.fungible(),
            instruction_asset_count.non_fungible(),
            instruction_asset_count.off_chain(),
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
        input_cost: &AssetCount,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        match receipt {
            Some(receipt) => {
                Self::base_affirm_with_receipts(origin, id, vec![receipt], portfolios, input_cost)?
            }
            None => Self::base_affirm_instruction(origin, id, portfolios.into_iter(), input_cost)?,
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

    /// Returns [`FilteredLegs`] where the subset of legs correspond to the legs where the sender is in the given `portfolio_set`.
    fn filtered_legs(
        id: InstructionId,
        portfolio_set: &BTreeSet<PortfolioId>,
        input_cost: &AssetCount,
    ) -> Result<FilteredLegs, DispatchError> {
        let instruction_legs: Vec<(LegId, Leg)> = InstructionLegs::iter_prefix(&id).collect();
        let instruction_asset_count = AssetCount::from_legs(&instruction_legs);
        // Gets all legs where the sender is in the given set
        let legs_from_set: Vec<(LegId, Leg)> = instruction_legs
            .into_iter()
            .filter(|(_, leg)| portfolio_set.contains(&leg.from))
            .collect();
        let subset_asset_count = AssetCount::from_legs(&legs_from_set);
        Self::ensure_valid_input_cost(&subset_asset_count, input_cost)?;
        Ok(FilteredLegs::new(
            id,
            legs_from_set,
            instruction_asset_count,
            subset_asset_count,
        ))
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
        input_cost: &AssetCount,
    ) -> DispatchResult {
        ensure!(
            Self::instruction_status(id) != InstructionStatus::Unknown,
            Error::<T>::UnknownInstruction
        );
        // Gets all legs for the instruction, checks if portfolio is in any of the legs, and validates the input cost.
        let legs: Vec<(LegId, Leg)> = InstructionLegs::iter_prefix(&id).collect();
        ensure!(
            legs.iter()
                .any(|(_, leg)| leg.from == portfolio || leg.to == portfolio),
            Error::<T>::CallerIsNotAParty
        );
        let instruction_asset_count = AssetCount::from_legs(&legs);
        Self::ensure_valid_input_cost(&instruction_asset_count, input_cost)?;

        // Verifies if the caller has the right permissions for this call
        let origin_data = Identity::<T>::ensure_origin_call_permissions(origin)?;
        T::Portfolio::ensure_portfolio_custody_and_permission(
            portfolio,
            origin_data.primary_did,
            origin_data.secondary_key.as_ref(),
        )?;

        Self::unsafe_unclaim_receipts(id, &legs);
        Self::unchecked_release_locks(id, &legs);
        let _ = T::Scheduler::cancel_named(id.execution_name());
        Self::prune_instruction(id, false);
        Self::deposit_event(RawEvent::InstructionRejected(origin_data.primary_did, id));
        Ok(())
    }

    /// Returns ok if the number of fungible assets and nfts being transferred is under the input given by the user.
    fn ensure_valid_input_cost(real_cost: &AssetCount, input_cost: &AssetCount) -> DispatchResult {
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

    /// Executes the instruction of the given `id` returning the consumed weight for executing the instruction.
    fn base_execute_scheduled_instruction(
        id: InstructionId,
        weight_meter: &mut WeightMeter,
    ) -> PostDispatchInfo {
        if let Err(e) = Self::execute_instruction_retryable(id, weight_meter) {
            Self::deposit_event(RawEvent::FailedToExecuteInstruction(id, e));
        }
        PostDispatchInfo::from(Some(weight_meter.consumed()))
    }

    /// Ensures all checks needed for a fungible leg hold. This includes making sure that the `amount` being
    /// transferred is not zero, that `ticker` exists on chain and that `venue_id` is allowed.
    fn ensure_valid_fungible_leg(
        tickers: &mut BTreeSet<Ticker>,
        ticker: Ticker,
        amount: Balance,
        venue_id: &VenueId,
    ) -> DispatchResult {
        ensure!(amount > 0, Error::<T>::ZeroAmount);
        ensure!(
            Self::is_on_chain_asset(&ticker),
            Error::<T>::UnexpectedOFFChainAsset
        );
        Self::ensure_venue_filtering(tickers, ticker, venue_id)?;
        Ok(())
    }

    /// Ensures all checks needed for a non fungible leg hold. This includes making sure that the number of NFTs being
    /// transferred is within the defined limits, that there are no duplicate NFTs in the same leg, that `ticker` exists on chain,
    /// and that `venue_id` is allowed.
    fn ensure_valid_nft_leg(
        tickers: &mut BTreeSet<Ticker>,
        nfts: &NFTs,
        venue_id: &VenueId,
    ) -> DispatchResult {
        ensure!(
            Self::is_on_chain_asset(nfts.ticker()),
            Error::<T>::UnexpectedOFFChainAsset
        );
        <Nft<T>>::ensure_within_nfts_transfer_limits(&nfts)?;
        <Nft<T>>::ensure_no_duplicate_nfts(&nfts)?;
        Self::ensure_venue_filtering(tickers, nfts.ticker().clone(), venue_id)?;
        Ok(())
    }

    /// Ensures all checks needed for an off-chain asset leg hold. This includes making sure that the `amount` being
    /// transferred is not zero, that `ticker` doesn't exist on chain and that `venue_id` is allowed.
    fn ensure_valid_off_chain_leg(
        tickers: &mut BTreeSet<Ticker>,
        ticker: Ticker,
        amount: Balance,
        venue_id: &VenueId,
    ) -> DispatchResult {
        ensure!(amount > 0, Error::<T>::ZeroAmount);
        ensure!(
            !Self::is_on_chain_asset(&ticker),
            Error::<T>::UnexpectedOnChainAsset
        );
        Self::ensure_venue_filtering(tickers, ticker, venue_id)?;
        Ok(())
    }

    /// Returns true if the ticker is on-chain and false otherwise.
    fn is_on_chain_asset(ticker: &Ticker) -> bool {
        pallet_asset::Tokens::contains_key(ticker)
    }

    fn base_execute_manual_instruction(
        origin: T::RuntimeOrigin,
        id: InstructionId,
        portfolio: Option<PortfolioId>,
        input_cost: &AssetCount,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo {
        // check origin has the permissions required and valid instruction
        let (did, sk, instruction_details) =
            Self::ensure_origin_perm_and_instruction_validity(origin, id, true)?;

        // Check for portfolio
        let instruction_legs: Vec<(LegId, Leg)> = InstructionLegs::iter_prefix(&id).collect();
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

        let instruction_asset_count = AssetCount::from_legs(&instruction_legs);
        Self::ensure_valid_input_cost(&instruction_asset_count, input_cost)?;

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

    /// Returns the worst case weight for an instruction with `f` fungible legs, `n` nfts being transferred and `o` offchain assets.
    fn execute_scheduled_instruction_weight_limit(f: u32, n: u32, o: u32) -> Weight {
        <T as Config>::WeightInfo::execute_scheduled_instruction(f, n, o)
    }

    /// Returns the minimum weight for calling the `execute_scheduled_instruction` function.
    fn execute_scheduled_instruction_minimum_weight() -> Weight {
        <T as Config>::WeightInfo::execute_scheduled_instruction(0, 0, 0)
    }

    /// Returns the worst case weight for an instruction with `f` fungible legs, `n` nfts being transferred and `o` offchain assets.
    fn execute_manual_instruction_weight_limit(f: u32, n: u32, o: u32) -> Weight {
        <T as Config>::WeightInfo::execute_manual_instruction(f, n, o)
    }

    /// Returns the minimum weight for calling the `execute_manual_instruction` extrinsic.
    pub fn execute_manual_instruction_minimum_weight() -> Weight {
        <T as Config>::WeightInfo::execute_manual_instruction(0, 0, 0)
    }

    /// Returns an instance of `ExecuteInstructionInfo`, which contains the number of fungible and non fungible assets
    /// in the instruction, and the weight consumed for executing the instruction. If the instruction would fail its
    /// execution, it also contains the error.
    pub fn execute_instruction_info(instruction_id: &InstructionId) -> ExecuteInstructionInfo {
        let instruction_legs: Vec<(LegId, Leg)> =
            InstructionLegs::iter_prefix(&instruction_id).collect();
        let instruction_asset_count = AssetCount::from_legs(&instruction_legs);
        let mut weight_meter =
            WeightMeter::max_limit(Self::execute_scheduled_instruction_minimum_weight());

        match Self::execute_instruction_retryable(*instruction_id, &mut weight_meter) {
            Ok(_) => ExecuteInstructionInfo::new(
                instruction_asset_count.fungible(),
                instruction_asset_count.non_fungible(),
                instruction_asset_count.off_chain(),
                weight_meter.consumed(),
                None,
            ),
            Err(e) => ExecuteInstructionInfo::new(
                instruction_asset_count.fungible(),
                instruction_asset_count.non_fungible(),
                instruction_asset_count.off_chain(),
                weight_meter.consumed(),
                Some(e.into()),
            ),
        }
    }
}

pub mod migration {
    use super::*;
    use sp_runtime::runtime_logger::RuntimeLogger;

    mod v1 {
        use super::*;
        use scale_info::TypeInfo;

        #[derive(Encode, Decode, TypeInfo)]
        #[derive(Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
        pub struct Instruction<Moment, BlockNumber> {
            pub instruction_id: InstructionId,
            pub venue_id: VenueId,
            pub status: InstructionStatus<BlockNumber>,
            pub settlement_type: SettlementType<BlockNumber>,
            pub created_at: Option<Moment>,
            pub trade_date: Option<Moment>,
            pub value_date: Option<Moment>,
        }

        #[derive(Encode, Decode, TypeInfo)]
        #[derive(Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
        pub struct Leg {
            pub from: PortfolioId,
            pub to: PortfolioId,
            pub asset: Ticker,
            pub amount: Balance,
        }

        #[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
        pub enum LegAsset {
            Fungible { ticker: Ticker, amount: Balance },
            NonFungible(NFTs),
        }

        impl Default for LegAsset {
            fn default() -> Self {
                LegAsset::Fungible {
                    ticker: Ticker::default(),
                    amount: Balance::default(),
                }
            }
        }

        #[derive(Encode, Decode, TypeInfo)]
        #[derive(Clone, Debug, Default, Eq, PartialEq)]
        pub struct LegV2 {
            pub from: PortfolioId,
            pub to: PortfolioId,
            pub asset: LegAsset,
        }

        decl_storage! {
            trait Store for Module<T: Config> as Settlement {
                pub InstructionDetails get(fn instruction_details):
                    map hasher(twox_64_concat) InstructionId => Instruction<T::Moment, T::BlockNumber>;
                pub InstructionLegs get(fn instruction_legs):
                    double_map hasher(twox_64_concat) InstructionId, hasher(twox_64_concat) LegId => Leg;
                pub InstructionLegsV2 get(fn instruction_legsv2):
                    double_map hasher(twox_64_concat) InstructionId, hasher(twox_64_concat) LegId => LegV2;
            }
        }

        decl_module! {
            pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
        }
    }

    pub fn migrate_to_v2<T: Config>() {
        RuntimeLogger::init();
        log::info!(" >>> Updating Settlement storage. Migrating Legs and Instructions.");
        migrate_legs::<T>();
        migrate_instruction_details::<T>();
        log::info!(" >>> All legs and Instructions have been migrated.");
    }

    fn migrate_legs<T: Config>() {
        // Migrate all itens from InstructionLegs
        v1::InstructionLegs::drain().for_each(|(id, leg_id, old_leg)| {
            let new_leg = {
                if !pallet_asset::Tokens::contains_key(old_leg.asset) {
                    Leg {
                        from: old_leg.from,
                        to: old_leg.to,
                        asset: LegAsset::OffChain {
                            ticker: old_leg.asset,
                            amount: old_leg.amount,
                        },
                    }
                } else {
                    Leg {
                        from: old_leg.from,
                        to: old_leg.to,
                        asset: LegAsset::Fungible {
                            ticker: old_leg.asset,
                            amount: old_leg.amount,
                        },
                    }
                }
            };
            InstructionLegs::insert(&id, leg_id, new_leg);
        });
        // Migrate all itens from InstructionLegsV2
        v1::InstructionLegsV2::drain().for_each(|(id, leg_id, leg_v2)| {
            let new_leg = {
                match leg_v2.asset {
                    v1::LegAsset::Fungible { ticker, amount } => {
                        if !pallet_asset::Tokens::contains_key(ticker) {
                            Leg {
                                from: leg_v2.from,
                                to: leg_v2.to,
                                asset: LegAsset::OffChain { ticker, amount },
                            }
                        } else {
                            Leg {
                                from: leg_v2.from,
                                to: leg_v2.to,
                                asset: LegAsset::Fungible { ticker, amount },
                            }
                        }
                    }
                    v1::LegAsset::NonFungible(nfts) => Leg {
                        from: leg_v2.from,
                        to: leg_v2.to,
                        asset: LegAsset::NonFungible(nfts),
                    },
                }
            };
            InstructionLegs::insert(&id, leg_id, new_leg);
        });
    }

    fn migrate_instruction_details<T: Config>() -> usize {
        v1::InstructionDetails::<T>::drain().fold(
            0,
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
        )
    }
}
