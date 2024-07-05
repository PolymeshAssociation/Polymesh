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

//! # Trait Interface to the Multisig Module
//!
//! The interface allows to process addition of a multisig signer from modules other than the
//! multisig module itself.

use frame_support::decl_event;
use frame_support::dispatch::DispatchError;
use frame_support::pallet_prelude::Weight;
use sp_std::vec::Vec;

use polymesh_primitives::IdentityId;

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// Event emitted after creation of a multisig.
        /// Arguments: caller DID, multisig address, signers (pending approval), signers required.
        MultiSigCreated(IdentityId, AccountId, AccountId, Vec<AccountId>, u64),
        /// Event emitted after adding a proposal.
        /// Arguments: caller DID, multisig, proposal ID.
        ProposalAdded(IdentityId, AccountId, u64),
        /// Event emitted when a proposal is executed.
        /// Arguments: caller DID, multisig, proposal ID, result.
        ProposalExecuted(IdentityId, AccountId, u64, bool),
        /// Event emitted when a signatory is added.
        /// Arguments: caller DID, multisig, added signer.
        MultiSigSignerAdded(IdentityId, AccountId, AccountId),
        /// Event emitted when a multisig signatory is authorized to be added.
        /// Arguments: caller DID, multisig, authorized signer.
        MultiSigSignerAuthorized(IdentityId, AccountId, AccountId),
        /// Event emitted when a multisig signatory is removed.
        /// Arguments: caller DID, multisig, removed signer.
        MultiSigSignerRemoved(IdentityId, AccountId, AccountId),
        /// Event emitted when the number of required signers is changed.
        /// Arguments: caller DID, multisig, new required signers.
        MultiSigSignersRequiredChanged(IdentityId, AccountId, u64),
        /// Event emitted when the proposal get approved.
        /// Arguments: caller DID, multisig, authorized signer, proposal id.
        ProposalApproved(IdentityId, AccountId, AccountId, u64),
        /// Event emitted when a vote is cast in favor of rejecting a proposal.
        /// Arguments: caller DID, multisig, authorized signer, proposal id.
        ProposalRejectionVote(IdentityId, AccountId, AccountId, u64),
        /// Event emitted when a proposal is rejected.
        /// Arguments: caller DID, multisig, proposal ID.
        ProposalRejected(IdentityId, AccountId, u64),
        /// Event emitted when a proposal failed to execute.
        /// Arguments: caller DID, multisig, proposal ID, error.
        ProposalFailedToExecute(IdentityId, AccountId, u64, DispatchError),
    }
);

pub trait WeightInfo {
    fn create_multisig(signers: u32) -> Weight;
    fn create_or_approve_proposal() -> Weight;
    fn create_proposal() -> Weight;
    fn approve() -> Weight;
    fn execute_proposal() -> Weight;
    fn reject() -> Weight;
    fn accept_multisig_signer() -> Weight;
    fn add_multisig_signer() -> Weight;
    fn remove_multisig_signer() -> Weight;
    fn add_multisig_signers_via_creator(signers: u32) -> Weight;
    fn remove_multisig_signers_via_creator(signers: u32) -> Weight;
    fn change_sigs_required() -> Weight;
    fn make_multisig_secondary() -> Weight;
    fn make_multisig_primary() -> Weight;
    fn change_sigs_required_via_creator() -> Weight;
    fn remove_creator_controls() -> Weight;
}

/// This trait is used to add a signer to a multisig and enable unlinking multisig from an identity
pub trait MultiSigSubTrait<AccountId> {
    /// Returns `true` if the given `account_id` is a multisign account, otherwise returns `false`.
    fn is_multisig(account_id: &AccountId) -> bool;
}
