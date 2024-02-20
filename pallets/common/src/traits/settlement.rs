use frame_support::decl_event;
use frame_support::dispatch::DispatchError;
use frame_support::weights::Weight;
use sp_std::vec::Vec;

use polymesh_primitives::settlement::{
    AffirmationCount, AssetCount, InstructionId, Leg, LegId, ReceiptMetadata, SettlementType,
    VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::{IdentityId, Memo, PortfolioId, Ticker};

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
            Option<ReceiptMetadata>,
        ),
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
        InstructionCreated(
            IdentityId,
            VenueId,
            InstructionId,
            SettlementType<BlockNumber>,
            Option<Moment>,
            Option<Moment>,
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
        MediatorAffirmationReceived(IdentityId, InstructionId, Option<Moment>),
        /// An instruction affirmation has been withdrawn by a mediator.
        /// Parameters: [`IdentityId`] of the mediator and [`InstructionId`] of the instruction.
        MediatorAffirmationWithdrawn(IdentityId, InstructionId),
    }
);

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
    fn reject_instruction(f: u32, n: u32, o: u32) -> Weight;
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
    fn reject_instruction_as_mediator(f: u32, n: u32, o: u32) -> Weight;

    fn add_and_affirm_with_mediators_legs(legs: &[Leg], n_mediators: u32) -> Weight {
        let (f, n, o) = Self::get_transfer_by_asset(legs);
        Self::add_and_affirm_with_mediators(f, n, o, n_mediators)
    }
    fn add_instruction_with_mediators_legs(legs: &[Leg], n_mediators: u32) -> Weight {
        let (f, n, o) = Self::get_transfer_by_asset(legs);
        Self::add_instruction_with_mediators(f, n, o, n_mediators)
    }
    fn add_instruction_legs(legs: &[Leg]) -> Weight {
        let (f, n, o) = Self::get_transfer_by_asset(legs);
        Self::add_instruction(f, n, o)
    }
    fn add_and_affirm_instruction_legs(legs: &[Leg]) -> Weight {
        let (f, n, o) = Self::get_transfer_by_asset(legs);
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
    fn get_transfer_by_asset(legs: &[Leg]) -> (u32, u32, u32) {
        let asset_count =
            AssetCount::try_from_legs(legs).unwrap_or(AssetCount::new(1024, 1024, 1024));
        (
            asset_count.fungible(),
            asset_count.non_fungible(),
            asset_count.off_chain(),
        )
    }
    fn affirm_with_receipts_input(affirmation_count: Option<AffirmationCount>) -> Weight {
        match affirmation_count {
            Some(affirmation_count) => {
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
            None => Self::affirm_with_receipts(10, 100, 10),
        }
    }
    fn affirm_instruction_input(affirmation_count: Option<AffirmationCount>) -> Weight {
        match affirmation_count {
            Some(affirmation_count) => {
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
            None => Self::affirm_instruction(10, 100),
        }
    }
    fn withdraw_affirmation_input(affirmation_count: Option<AffirmationCount>) -> Weight {
        match affirmation_count {
            Some(affirmation_count) => {
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
            None => Self::withdraw_affirmation(10, 100, 10),
        }
    }
    fn reject_instruction_input(asset_count: Option<AssetCount>, as_mediator: bool) -> Weight {
        match asset_count {
            Some(asset_count) => {
                if as_mediator {
                    return Self::reject_instruction_as_mediator(
                        asset_count.fungible(),
                        asset_count.non_fungible(),
                        asset_count.off_chain(),
                    );
                }
                Self::reject_instruction(
                    asset_count.fungible(),
                    asset_count.non_fungible(),
                    asset_count.off_chain(),
                )
            }
            None => {
                let (f, n, o) = (10, 100, 10);
                if as_mediator {
                    return Self::reject_instruction_as_mediator(f, n, o);
                }
                Self::reject_instruction(f, n, o)
            }
        }
    }
}
