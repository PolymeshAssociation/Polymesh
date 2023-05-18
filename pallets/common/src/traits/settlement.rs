use frame_support::decl_event;
use frame_support::dispatch::DispatchError;
use frame_support::weights::Weight;
use sp_std::vec::Vec;

use polymesh_primitives::settlement::{
    InstructionId, InstructionMemo, Leg, LegId, LegV2, ReceiptMetadata, SettlementType,
    TransferData, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::{IdentityId, PortfolioId, Ticker};

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
        /// A new instruction has been created
        /// (did, venue_id, instruction_id, settlement_type, trade_date, value_date, legs, memo)
        InstructionV2Created(
            IdentityId,
            VenueId,
            InstructionId,
            SettlementType<BlockNumber>,
            Option<Moment>,
            Option<Moment>,
            Vec<LegV2>,
            Option<InstructionMemo>,
        ),
        /// Failed to execute instruction.
        FailedToExecuteInstruction(InstructionId, DispatchError),
    }
);

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
    fn reschedule_instruction() -> Weight;
    fn execute_manual_instruction(l: u32) -> Weight;

    // Some multiple paths based extrinsic.
    // TODO: Will be removed once we get the worst case weight.
    fn add_instruction_with_settle_on_block_type(u: u32) -> Weight;
    fn add_and_affirm_instruction_with_settle_on_block_type(u: u32) -> Weight;
    fn add_instruction_with_memo_and_settle_on_block_type(u: u32) -> Weight;
    fn add_and_affirm_instruction_with_memo_and_settle_on_block_type(u: u32) -> Weight;
    fn add_instruction_with_memo_v2(f: u32) -> Weight;
    fn add_and_affirm_instruction_with_memo_v2(f: u32, n: u32) -> Weight;
    fn affirm_instruction_v2(f: u32, n: u32) -> Weight;
    fn withdraw_affirmation_v2(f: u32, n: u32) -> Weight;
    fn reject_instruction_v2(f: u32, n: u32) -> Weight;
    fn add_and_affirm_instruction_with_memo_v2_legs(legs_v2: &[LegV2]) -> Weight {
        let (f, n) = Self::get_transfer_by_asset(legs_v2);
        Self::add_and_affirm_instruction_with_memo_v2(f, n)
    }
    fn execute_scheduled_instruction_v2(legs_v2: &[LegV2]) -> Weight {
        let (f, n) = Self::get_transfer_by_asset(legs_v2);
        Self::execute_scheduled_instruction(f, n)
    }
    fn execute_instruction_paused(f: u32, n: u32) -> Weight;
    fn execute_scheduled_instruction(f: u32, n: u32) -> Weight;
    fn get_transfer_by_asset(legs_v2: &[LegV2]) -> (u32, u32) {
        let transfer_data =
            TransferData::from_legs(legs_v2).unwrap_or(TransferData::new(u32::MAX, u32::MAX));
        (transfer_data.fungible(), transfer_data.non_fungible())
    }
    fn execute_manual_weight_limit(weight_limit: &Option<Weight>, n_legs: &u32) -> Weight {
        if let Some(weight_limit) = weight_limit {
            return *weight_limit;
        }
        Self::execute_manual_instruction(*n_legs)
    }
    fn ensure_root_origin() -> Weight;
}
