use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::pallet_prelude::DispatchError;
use frame_support::{dispatch::DispatchResult, ensure};
use scale_info::TypeInfo;

use polymesh_dart::{
    InstantReceiverAffirmationProof, InstantSenderAffirmationProof, LegEncrypted, LegId, LegRef,
    MediatorAffirmationProof, MediatorId, ReceiverAffirmationProof, ReceiverClaimProof,
    SenderAffirmationProof, SenderCounterUpdateProof, SenderReversalProof, SettlementRef,
};

use super::*;

/// Pending counts.
#[derive(Copy, Clone, Debug)]
pub struct PendingCounts {
    /// Number of pending affirmations.
    pub affirms: u32,
    /// Number of pending finalizations.
    pub finals: u32,
}

/// Leg affirmation party.
#[derive(
    Copy,
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Debug,
    TypeInfo,
    PartialEq,
    Eq
)]
pub enum LegAffirmParty {
    Sender,
    Receiver,
    Mediator(MediatorId),
}

impl LegAffirmParty {
    pub fn is_mediator(&self) -> bool {
        matches!(self, LegAffirmParty::Mediator(_))
    }
}

/// The affirmation status for each party in a confidential settlement leg.
#[derive(
    Copy,
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Debug,
    TypeInfo,
    PartialEq,
    Eq
)]
pub enum AffirmationStatus {
    /// The leg is pending affirmation.
    Pending,
    /// The leg has been affirmed by the sender, receiver, and mediator.
    Affirmed,
    /// The leg has been rejected by one party.
    Rejected,
    /// The leg has been finalized by all parties.
    Finalized,
}

/// Settlement status for a confidential settlement.
#[derive(
    Copy,
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Debug,
    TypeInfo,
    PartialEq,
    Eq
)]
pub enum SettlementStatus {
    Pending,
    Executed,
    Rejected,
    Finalized,
}

pub struct UpdateSettlementStatus<T: Config> {
    /// The settlement reference.
    pub settlement_ref: SettlementRef,
    /// The leg id.
    pub leg_id: LegId,
    /// The status of the settlement.
    pub status: SettlementStatus,
    _marker: core::marker::PhantomData<T>,
}

impl<T: Config> UpdateSettlementStatus<T> {
    /// Create a new `UpdateSettlementStatus` instance.
    pub fn new(leg_ref: LegRef) -> Result<Self, DispatchError> {
        // Get the settlement reference from the leg reference.
        let settlement_ref = leg_ref.settlement;
        // Get the status of the settlement.
        let status =
            SettlementState::<T>::get(settlement_ref).ok_or(Error::<T>::SettlementNotFound)?;

        Ok(Self {
            settlement_ref,
            leg_id: leg_ref.leg_id(),
            status,
            _marker: core::marker::PhantomData,
        })
    }

    /// Ensure the settlement is in a pending state.
    pub fn ensure_pending(&self) -> DispatchResult {
        // Ensure the settlement is in a pending state.
        ensure!(
            self.status == SettlementStatus::Pending,
            Error::<T>::SettlementNotPending
        );
        Ok(())
    }

    /// Get the encrypted leg data.
    fn get_leg(&self) -> Result<LegEncrypted, DispatchError> {
        // Ensure the leg ID is valid.
        let leg_enc = SettlementLegs::<T>::get((self.settlement_ref, self.leg_id))
            .ok_or(Error::<T>::LegNotFound)?;
        Ok(leg_enc)
    }

    fn set_party_status(
        &self,
        party: LegAffirmParty,
        expected_old_status: AffirmationStatus,
        new_status: AffirmationStatus,
    ) -> Result<u32, DispatchError> {
        // Try to set the party's affirmation status.
        LegAffirmationStatus::<T>::try_mutate(
            (self.settlement_ref, self.leg_id, party),
            |status| -> Result<(), DispatchError> {
                // Check if the current status matches the expected old status.
                if Some(expected_old_status) == *status {
                    // Update the party's affirmation status to the new status.
                    *status = Some(new_status);
                    Ok(())
                } else if let Some(current_status) = status {
                    // The party's current status does not match the expected old status.
                    match (current_status, new_status) {
                        (AffirmationStatus::Rejected, _) => Err(Error::<T>::AlreadyRejected.into()),
                        (AffirmationStatus::Finalized, _) => {
                            Err(Error::<T>::AlreadyFinalized.into())
                        }
                        (AffirmationStatus::Affirmed, _) if party.is_mediator() => {
                            Err(Error::<T>::AlreadyAffirmed.into())
                        }
                        _ => Err(Error::<T>::InvalidAffirmationStatusTransition.into()),
                    }
                } else if party.is_mediator() {
                    Err(Error::<T>::WrongMediatorId.into())
                } else {
                    Err(Error::<T>::LegNotFound.into())
                }
            },
        )?;

        // Update the pending finalizations count based on the party and status transition.
        let pending_final = match (party, expected_old_status, new_status) {
            (
                LegAffirmParty::Sender | LegAffirmParty::Receiver,
                AffirmationStatus::Pending,
                AffirmationStatus::Affirmed,
            ) => {
                // If the party is not a mediator and they accepted, increase the number of pending finalizations for the settlement.
                SettlementPendingFinalizations::<T>::mutate(self.settlement_ref, |count| {
                    *count = count.saturating_add(1);
                    *count
                })
            }
            (
                LegAffirmParty::Sender | LegAffirmParty::Receiver,
                AffirmationStatus::Affirmed,
                AffirmationStatus::Finalized,
            ) => {
                // If the party is not a mediator and they finalized, decrease the number of pending finalizations for the settlement.
                SettlementPendingFinalizations::<T>::mutate(self.settlement_ref, |count| {
                    *count = count.saturating_sub(1);
                    *count
                })
            }
            _ => SettlementPendingFinalizations::<T>::get(self.settlement_ref),
        };

        Ok(pending_final)
    }

    fn party_affirms(
        &self,
        party: LegAffirmParty,
        accept: bool,
        is_instant: bool,
    ) -> Result<PendingCounts, DispatchError> {
        let new_status = match (accept, is_instant) {
            (true, true) => AffirmationStatus::Finalized,
            (true, false) => AffirmationStatus::Affirmed,
            (false, _) => AffirmationStatus::Rejected,
        };
        // Try to set the party's affirmation status.
        let pending_final = self.set_party_status(party, AffirmationStatus::Pending, new_status)?;

        // Try to decrease the number of pending affirmations for the settlement.
        let pending_affirms = SettlementPendingAffirmations::<T>::try_mutate(
            self.settlement_ref,
            |count| -> Result<u32, DispatchError> {
                *count = count
                    .checked_sub(1)
                    .ok_or(Error::<T>::NoPendingAffirmations)?;
                Ok(*count)
            },
        )?;
        Ok(PendingCounts {
            affirms: pending_affirms,
            finals: pending_final,
        })
    }

    fn party_finalizes(&self, party: LegAffirmParty) -> Result<u32, DispatchError> {
        // Try to set the party's affirmation status to finalized.
        self.set_party_status(
            party,
            AffirmationStatus::Affirmed,
            AffirmationStatus::Finalized,
        )
    }

    /// Verify a sender affirmation proof for a specific leg in the settlement.
    pub fn sender_affirmation(
        &self,
        proof: SenderAffirmationProof<PolymeshLimits, AccountTreeConfig>,
        root: AccountTreeRoot,
    ) -> DispatchResult {
        self.ensure_pending()?;

        // The receiver is only allowed to submit this proof if the settlement has been executed and they have affirmed.
        let pending = self.party_affirms(LegAffirmParty::Sender, true, false)?;

        // verify the proof.
        Pallet::<T>::submit_and_wait(VerifyDartAssetRequest::SenderAffirmation {
            leg_enc: self.get_leg()?,
            root,
            proof: proof.clone(),
        })?;

        // If the sender has affirmed, update the status of the settlement.
        self.check_for_execution(pending)?;

        Ok(())
    }

    /// Verify a receiver affirmation proof for a specific leg in the settlement.
    pub fn receiver_affirmation(
        &self,
        proof: ReceiverAffirmationProof<PolymeshLimits, AccountTreeConfig>,
        root: AccountTreeRoot,
    ) -> DispatchResult {
        self.ensure_pending()?;

        // Affirm as the receiver if they have not already affirmed.
        let pending = self.party_affirms(LegAffirmParty::Receiver, true, false)?;

        // verify the proof.
        Pallet::<T>::submit_and_wait(VerifyDartAssetRequest::ReceiverAffirmation {
            leg_enc: self.get_leg()?,
            root,
            proof: proof.clone(),
        })?;

        // If the receiver has affirmed, update the status of the settlement.
        self.check_for_execution(pending)?;
        Ok(())
    }

    /// Verify an instant sender affirmation proof for a specific leg in the settlement.
    pub fn instant_sender_affirmation(
        &self,
        proof: InstantSenderAffirmationProof<PolymeshLimits, AccountTreeConfig>,
        root: AccountTreeRoot,
    ) -> DispatchResult {
        self.ensure_pending()?;

        // The receiver is only allowed to submit this proof if the settlement has been executed and they have affirmed.
        let pending = self.party_affirms(LegAffirmParty::Sender, true, true)?;

        // verify the proof.
        Pallet::<T>::submit_and_wait(VerifyDartAssetRequest::InstantSenderAffirmation {
            leg_enc: self.get_leg()?,
            root,
            proof: proof.clone(),
        })?;

        // If the sender has affirmed, update the status of the settlement.
        self.check_for_execution(pending)?;

        Ok(())
    }

    /// Verify an instant receiver affirmation proof for a specific leg in the settlement.
    pub fn instant_receiver_affirmation(
        &self,
        proof: InstantReceiverAffirmationProof<PolymeshLimits, AccountTreeConfig>,
        root: AccountTreeRoot,
    ) -> DispatchResult {
        self.ensure_pending()?;

        // Affirm as the receiver if they have not already affirmed.
        let pending = self.party_affirms(LegAffirmParty::Receiver, true, true)?;

        // verify the proof.
        Pallet::<T>::submit_and_wait(VerifyDartAssetRequest::InstantReceiverAffirmation {
            leg_enc: self.get_leg()?,
            root,
            proof: proof.clone(),
        })?;

        // If the receiver has affirmed, update the status of the settlement.
        self.check_for_execution(pending)?;
        Ok(())
    }

    /// Verify a mediator affirmation proof for a specific leg in the settlement.
    pub fn mediator_affirmation(&self, proof: MediatorAffirmationProof<PolymeshLimits>) -> DispatchResult {
        self.ensure_pending()?;

        let accept = proof.accept;
        let mediator_id = proof.key_index;
        // Affirm as the mediator if they have not already affirmed.
        let pending = self.party_affirms(LegAffirmParty::Mediator(mediator_id), accept, false)?;

        // verify the proof.
        Pallet::<T>::submit_and_wait(VerifyDartAssetRequest::MediatorAffirmation {
            leg_enc: self.get_leg()?,
            proof,
        })?;

        if accept {
            // If the mediator has accepted, update the status of the settlement.
            self.check_for_execution(pending)
        } else {
            // If the mediator has rejected, reject the settlement.
            self.reject(pending.finals)
        }
    }

    /// Verify the sender's counter update proof for a specific leg in the settlement.
    pub fn sender_counter_update(
        &self,
        proof: SenderCounterUpdateProof<PolymeshLimits, AccountTreeConfig>,
        root: AccountTreeRoot,
    ) -> DispatchResult {
        // The sender can only update the counter if the settlement has been executed.
        ensure!(
            self.status == SettlementStatus::Executed,
            Error::<T>::SettlementNotExecuted
        );

        // The sender is only allowed to submit this proof if the settlement has been executed and they have affirmed.
        let pending_final = self.party_finalizes(LegAffirmParty::Sender)?;

        // verify the proof.
        Pallet::<T>::submit_and_wait(VerifyDartAssetRequest::SenderCounterUpdate {
            leg_enc: self.get_leg()?,
            root,
            proof: proof.clone(),
        })?;

        // If the sender has finalized the leg, update the status of the settlement.
        self.check_for_finalization(pending_final)?;
        Ok(())
    }

    /// Verify the sender's reversal proof for a specific leg in the settlement to claim back their assets.
    ///
    /// This can only be done if the settlement has been rejected or is still pending.
    pub fn sender_reversal(
        &self,
        proof: SenderReversalProof<PolymeshLimits, AccountTreeConfig>,
        root: AccountTreeRoot,
    ) -> DispatchResult {
        let status = self.status;
        // The sender can only reverse if the settlement has been rejected or is still pending.
        ensure!(
            status == SettlementStatus::Rejected || status == SettlementStatus::Pending,
            Error::<T>::SettlementNotRejected
        );

        // The sender is only allowed to submit this proof if they have affirmed and the settlement has been rejected or is still pending.
        let pending_final = self.party_finalizes(LegAffirmParty::Sender)?;

        // verify the proof.
        Pallet::<T>::submit_and_wait(VerifyDartAssetRequest::SenderReversal {
            leg_enc: self.get_leg()?,
            root,
            proof: proof.clone(),
        })?;

        // If the settlement was rejected.
        if status == SettlementStatus::Rejected {
            // Finalize the settlement if the sender has finalized the leg.
            self.check_for_finalization(pending_final)
        } else {
            // If the settlement was pending, then reject the settlement.
            self.reject(pending_final)
        }
    }

    /// Verify the receiver's claim proof for a specific leg in the settlement.
    pub fn receiver_claim(
        &self,
        proof: ReceiverClaimProof<PolymeshLimits, AccountTreeConfig>,
        root: AccountTreeRoot,
    ) -> DispatchResult {
        // The receiver can only claim if the settlement has been executed.
        ensure!(
            self.status == SettlementStatus::Executed,
            Error::<T>::SettlementNotExecuted
        );

        // The receiver is only allowed to submit this proof if the settlement has been executed and they have affirmed.
        let pending_final = self.party_finalizes(LegAffirmParty::Receiver)?;

        // verify the proof.
        Pallet::<T>::submit_and_wait(VerifyDartAssetRequest::ReceiverClaim {
            leg_enc: self.get_leg()?,
            root,
            proof: proof.clone(),
        })?;

        // If the receiver has finalized the leg, update the status of the settlement.
        self.check_for_finalization(pending_final)?;
        Ok(())
    }

    fn check_for_execution(&self, pending: PendingCounts) -> DispatchResult {
        if self.status == SettlementStatus::Pending && pending.affirms == 0 {
            // All parties have affirmed, so execute the settlement.
            self.execute()?;
            if pending.finals == 0 {
                // All parties have also finalized, so finalize the settlement.
                self.finalize()?;
            }
        }
        Ok(())
    }

    fn check_for_finalization(&self, pending_final: u32) -> DispatchResult {
        if self.status == SettlementStatus::Executed && pending_final == 0 {
            self.finalize()?;
        }
        Ok(())
    }

    fn set_status(&self, status: SettlementStatus) {
        // Set the settlement status in the storage.
        SettlementState::<T>::insert(self.settlement_ref, status);

        // Emit an event for the settlement status update.
        Pallet::<T>::deposit_event(Event::<T>::SettlementStatusUpdated {
            settlement_ref: self.settlement_ref,
            status,
        });
    }

    /// Reject the settlement, marking it as rejected.
    fn reject(&self, pending_finals: u32) -> DispatchResult {
        // Ensure the settlement is in a pending state before rejecting.
        ensure!(
            self.status == SettlementStatus::Pending,
            Error::<T>::SettlementNotPending
        );

        // Prune the pending affirmation count.
        SettlementPendingAffirmations::<T>::remove(self.settlement_ref);

        if pending_finals > 0 {
            // Set the settlement status to rejected, since there are still pending finalizations.
            self.set_status(SettlementStatus::Rejected);

            Ok(())
        } else {
            // Finalize the settlement if there are no pending finalizations.
            self.finalize()
        }
    }

    /// Execute the settlement, marking it as executed.
    fn execute(&self) -> DispatchResult {
        // Ensure the settlement is in a pending state before executing.
        ensure!(
            self.status == SettlementStatus::Pending,
            Error::<T>::SettlementNotPending
        );

        // Prune the pending affirmation count.
        SettlementPendingAffirmations::<T>::remove(self.settlement_ref);

        // Set the settlement status to executed.
        self.set_status(SettlementStatus::Executed);

        Ok(())
    }

    /// Finalize the settlement, marking it as finalized.
    fn finalize(&self) -> DispatchResult {
        // Prune all settlement state.
        // Remove pending finalizations for the settlement.
        SettlementPendingFinalizations::<T>::remove(self.settlement_ref);
        // Remove the settlement memo.
        SettlementMemo::<T>::remove(self.settlement_ref);
        // Remove the settlement legs and affirmation statuses.
        let leg_count = SettlementLegCount::<T>::take(self.settlement_ref)
            .map(|c| c.0)
            .unwrap_or(0);
        for leg_id in 0..leg_count {
            let leg_id = leg_id as LegId;

            // Remove affirmation statuses for sender and receiver.
            LegAffirmationStatus::<T>::remove((
                self.settlement_ref,
                leg_id,
                LegAffirmParty::Sender,
            ));
            LegAffirmationStatus::<T>::remove((
                self.settlement_ref,
                leg_id,
                LegAffirmParty::Receiver,
            ));

            // Remove the leg.
            if let Some(leg) = SettlementLegs::<T>::take((self.settlement_ref, leg_id)) {
                // Remove affirmation statuses for mediators.
                let mediators = leg.mediator_count().ok().unwrap_or(0) as u8;
                for mediator_index in 0..mediators {
                    LegAffirmationStatus::<T>::remove((
                        self.settlement_ref,
                        leg_id,
                        LegAffirmParty::Mediator(mediator_index),
                    ));
                }
            }
        }

        // Set the settlement status to finalized.
        self.set_status(SettlementStatus::Finalized);

        Ok(())
    }
}
