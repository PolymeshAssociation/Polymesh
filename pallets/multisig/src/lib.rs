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

//! # Multisig Module
//!
//! The multisig module provides functionality for `n` out of `m` multisigs.
//!
//! ## Overview
//!
//! The multisig module provides functions for:
//!
//! - creating a new multisig,
//! - proposing a multisig transaction,
//! - approving a multisig transaction,
//! - adding new signers to the multisig,
//! - removing existing signers from multisig.
//!
//! ### Terminology
//!
//! - **multisig**: a special type of account that can do transaction only if at least `n` of its `m`
//! signers approve.
//! - **proposal**: a general transaction that the multisig can vote on and accept.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `create_multisig` - Creates a new multisig.
//! - `create_or_approve_proposal` - Creates or approves a multisig proposal given the
//! signer's account key.
//! - `create_proposal` - Creates a multisig proposal given the signer's account key.
//! - `approve` - Approves a multisig proposal given the signer's account key.
//! - `reject` - Rejects a multisig proposal using the caller's secondary key (`AccountId`).
//! - `accept_multisig_signer` - Accepts a multisig signer authorization given the signer's
//! account key.
//! - `add_multisig_signer` - Adds a signer to the multisig.
//! - `remove_multisig_signer` - Removes a signer from the multisig.
//! - `add_multisig_signers_via_creator` - Adds a signer to the multisig when called by the
//! creator of the multisig.
//! - `remove_multisig_signers_via_creator` - Removes a signer from the multisig when called by the
//! creator of the multisig.
//! - `change_sigs_required` - Changes the number of signers required to execute a transaction.
//! - `make_multisig_secondary` - Adds a multisig as a secondary key of the current DID if the current DID is
//! the creator of the multisig.
//! - `make_multisig_primary` - Adds a multisig as the primary key of the current DID if the current DID
//! is the creator of the multisig.
//!
//! ### Other Public Functions
//!
//! - `base_create_multisig` - Creates a multisig account without precondition checks or emitting
//! an event.
//! - `base_create_proposal` - Creates a proposal for a multisig transaction.
//! - `base_create_or_approve_proposal` - Creates or approves a multisig proposal.
//! - `base_accept_multisig_signer` - Accepts and processes an addition of a signer to a multisig.
//! - `get_next_multisig_address` - Gets the next available multisig account ID.
//! - `get_multisig_address` - Constructs a multisig account given a nonce.
//! - `ms_signers` - Helper function that checks if someone is an authorized signer of a multisig or
//! not.
//! - `is_changing_signers_allowed` - Checks whether changing the list of signers is allowed in a
//! multisig.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use core::convert::From;
use frame_support::dispatch::{
    DispatchError, DispatchResult, DispatchResultWithPostInfo, GetDispatchInfo, PostDispatchInfo,
    Weight,
};
use frame_support::storage::{IterableStorageDoubleMap, IterableStorageMap};
use frame_support::traits::{Get, GetCallMetadata};
use frame_support::{decl_module, decl_storage, ensure};
use frame_system::ensure_signed;
use sp_runtime::traits::{Dispatchable, Hash};
use sp_std::convert::TryFrom;
use sp_std::prelude::*;

use pallet_identity::PermissionedCallOriginData;
use pallet_permissions::with_call_metadata;
pub use polymesh_common_utilities::multisig::{MultiSigSubTrait, WeightInfo};
use polymesh_common_utilities::traits::identity::Config as IdentityConfig;
use polymesh_common_utilities::Context;
use polymesh_primitives::multisig::{ProposalState, ProposalVoteCount};
use polymesh_primitives::{
    extract_auth, storage_migrate_on, storage_migration_ver, AuthorizationData, IdentityId,
    KeyRecord, Permissions, Signatory,
};
//use polymesh_runtime_common::RocksDbWeight as DbWeight;
use frame_support::weights::constants::RocksDbWeight as DbWeight;

type IdentityPallet<T> = pallet_identity::Module<T>;

storage_migration_ver!(3);

fn add_base_weight(base_weight: Weight, post_info: &mut PostDispatchInfo) {
    if let Some(actual_weight) = &mut post_info.actual_weight {
        *actual_weight = actual_weight.saturating_add(base_weight);
    } else {
        post_info.actual_weight = Some(base_weight);
    }
}

fn with_base_weight(
    base_weight: Weight,
    tx: impl FnOnce() -> DispatchResultWithPostInfo,
) -> DispatchResultWithPostInfo {
    match tx() {
        Ok(mut post_info) => {
            add_base_weight(base_weight, &mut post_info);
            Ok(post_info)
        }
        Err(mut err) => {
            add_base_weight(base_weight, &mut err.post_info);
            Err(err)
        }
    }
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + IdentityConfig {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Weight information for extrinsics in the multisig pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            use sp_version::RuntimeVersion;
            let mut weight = Weight::zero();

            // Clear pending proposals if the transaction version is upgraded
            let current_version = <T::Version as Get<RuntimeVersion>>::get().transaction_version;
            if TransactionVersion::<T>::get() < current_version {
                TransactionVersion::<T>::set(current_version);
                // TODO: Multiblock migration.
                let mut removed = 0;
                let res = Proposals::<T>::clear(u32::max_value(), None);
                removed += res.unique;
                let res = ProposalIds::<T>::clear(u32::max_value(), None);
                removed += res.unique;
                let res = ProposalVoteCounts::<T>::clear(u32::max_value(), None);
                removed += res.unique;
                let res = Votes::<T>::clear(u32::max_value(), None);
                removed += res.unique;
                weight.saturating_accrue(DbWeight::get().reads_writes(removed as _, removed as _));
            }

            storage_migrate_on!(StorageVersion<T>, 3, {
                migration::migrate_to_v3::<T>(&mut weight);
            });
            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Creates a multisig
        ///
        /// # Arguments
        /// * `signers` - Signers of the multisig (They need to accept authorization before they are actually added).
        /// * `sigs_required` - Number of sigs required to process a multi-sig tx.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_multisig(signers.len() as u32))]
        pub fn create_multisig(
            origin: OriginFor<T>,
            signers: Vec<T::AccountId>,
            sigs_required: u64,
        ) -> DispatchResultWithPostInfo {
            let PermissionedCallOriginData {
                sender,
                primary_did,
                ..
            } = IdentityPallet::<T>::ensure_origin_call_permissions(origin)?;
            Self::ensure_sigs_in_bounds(&signers, sigs_required)?;
            let account_id =
                Self::base_create_multisig(sender.clone(), signers.as_slice(), sigs_required)?;
            Self::deposit_event(Event::MultiSigCreated(
                primary_did,
                account_id,
                sender,
                signers,
                sigs_required,
            ));
            Ok(().into())
        }

        /// Creates a multisig proposal if it hasn't been created or approves it if it has.
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal` - Proposal to be voted on.
        /// * `expiry` - Optional proposal expiry time.
        ///
        /// If this is 1 out of `m` multisig, the proposal will be immediately executed.
        /// #[deprecated(since = "6.0.0", note = "Please use the `create_proposal` and `approve` instead")]
        #[pallet::call_index(1)]
        #[pallet::weight({
          <T as Config>::WeightInfo::create_or_approve_proposal()
            .saturating_add(<T as Config>::WeightInfo::execute_proposal())
            .saturating_add(proposal.get_dispatch_info().weight)
        })]
        pub fn create_or_approve_proposal(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            proposal: Box<T::Proposal>,
            expiry: Option<T::Moment>,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            with_base_weight(
                <T as Config>::WeightInfo::create_or_approve_proposal(),
                || Self::base_create_or_approve_proposal(&multisig, signer, proposal, expiry),
            )
        }

        /// Creates a multisig proposal
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal` - Proposal to be voted on.
        /// * `expiry` - Optional proposal expiry time.
        ///
        /// If this is 1 out of `m` multisig, the proposal will be immediately executed.
        #[pallet::call_index(2)]
        #[pallet::weight({
          <T as Config>::WeightInfo::create_proposal()
            .saturating_add(<T as Config>::WeightInfo::execute_proposal())
            .saturating_add(proposal.get_dispatch_info().weight)
        })]
        pub fn create_proposal(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            proposal: Box<T::Proposal>,
            expiry: Option<T::Moment>,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            with_base_weight(<T as Config>::WeightInfo::create_proposal(), || {
                Self::base_create_proposal(&multisig, signer, proposal, expiry, false)
            })
        }

        /// Approves a multisig proposal using the caller's secondary key (`AccountId`).
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal_id` - Proposal id to approve.
        /// * `max_weight` - The maximum weight to execute the proposal.
        ///
        /// If quorum is reached, the proposal will be immediately executed.
        #[pallet::call_index(3)]
        #[pallet::weight({
          <T as Config>::WeightInfo::approve()
            .saturating_add(<T as Config>::WeightInfo::execute_proposal())
            .saturating_add(*max_weight)
        })]
        pub fn approve(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            proposal_id: u64,
            max_weight: Weight,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            with_base_weight(<T as Config>::WeightInfo::approve(), || {
                Self::base_approve(&multisig, signer, proposal_id, max_weight)
            })
        }

        /// Rejects a multisig proposal using the caller's secondary key (`AccountId`).
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal_id` - Proposal id to reject.
        /// If quorum is reached, the proposal will be immediately executed.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::reject())]
        pub fn reject(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            proposal_id: u64,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            Self::base_reject(&multisig, signer, proposal_id)?;
            Ok(().into())
        }

        /// Accepts a multisig signer authorization given to signer's key (AccountId).
        ///
        /// # Arguments
        /// * `auth_id` - Auth id of the authorization.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::accept_multisig_signer())]
        pub fn accept_multisig_signer(
            origin: OriginFor<T>,
            auth_id: u64,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            Self::base_accept_multisig_signer(signer, auth_id)?;
            Ok(().into())
        }

        /// Adds a signer to the multisig. This must be called by the multisig itself.
        ///
        /// # Arguments
        /// * `signer` - Signer to add.
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::add_multisig_signer())]
        pub fn add_multisig_signer(
            origin: OriginFor<T>,
            signer: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin)?;
            // Ensure the caller is a MultiSig and get it's creator DID.
            let did =
                CreatorDid::<T>::try_get(&multisig).map_err(|_| Error::<T>::NoSuchMultisig)?;
            Self::base_add_auth_for_signers(did, signer, multisig)?;
            Ok(().into())
        }

        /// Removes a signer from the multisig. This must be called by the multisig itself.
        ///
        /// # Arguments
        /// * `signer` - Signer to remove.
        #[pallet::weight(<T as Config>::WeightInfo::remove_multisig_signer())]
        #[pallet::call_index(7)]
        pub fn remove_multisig_signer(
            origin: OriginFor<T>,
            signer: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin)?;
            Self::ensure_ms(&multisig)?;
            Self::ensure_ms_signer(&multisig, &signer)?;
            ensure!(
                NumberOfSigners::<T>::get(&multisig) > MultiSigSignsRequired::<T>::get(&multisig),
                Error::<T>::NotEnoughSigners
            );
            ensure!(
                Self::is_changing_signers_allowed(&multisig),
                Error::<T>::ChangeNotAllowed
            );
            NumberOfSigners::<T>::mutate(&multisig, |x| *x -= 1u64);
            Self::base_signer_removal(&multisig, signer);
            Ok(().into())
        }

        /// Adds a signer to the multisig. This must be called by the creator identity of the
        /// multisig.
        ///
        /// # Arguments
        /// * `multisig` - Address of the multi sig
        /// * `signers` - Signers to add.
        ///
        /// # Weight
        /// `900_000_000 + 3_000_000 * signers.len()`
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::add_multisig_signers_via_creator(signers.len() as u32))]
        pub fn add_multisig_signers_via_creator(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            signers: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let caller_did = Self::ensure_ms_creator(origin, &multisig)?;
            ensure!(
                !LostCreatorPrivileges::<T>::get(caller_did),
                Error::<T>::CreatorControlsHaveBeenRemoved
            );
            for signer in signers {
                Self::base_add_auth_for_signers(caller_did, signer, multisig.clone())?;
            }
            Ok(().into())
        }

        /// Removes a signer from the multisig.
        /// This must be called by the creator identity of the multisig.
        ///
        /// # Arguments
        /// * `multisig` - Address of the multisig.
        /// * `signers` - Signers to remove.
        ///
        /// # Weight
        /// `900_000_000 + 3_000_000 * signers.len()`
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_multisig_signers_via_creator(signers.len() as u32))]
        pub fn remove_multisig_signers_via_creator(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            signers: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let caller_did = Self::ensure_ms_creator(origin, &multisig)?;
            ensure!(
                !LostCreatorPrivileges::<T>::get(caller_did),
                Error::<T>::CreatorControlsHaveBeenRemoved
            );

            ensure!(
                Self::is_changing_signers_allowed(&multisig),
                Error::<T>::ChangeNotAllowed
            );
            let signers_len: u64 = u64::try_from(signers.len()).unwrap_or_default();

            let pending_num_of_signers = NumberOfSigners::<T>::get(&multisig)
                .checked_sub(signers_len)
                .ok_or(Error::<T>::TooManySigners)?;
            ensure!(
                pending_num_of_signers >= MultiSigSignsRequired::<T>::get(&multisig),
                Error::<T>::NotEnoughSigners
            );

            for signer in &signers {
                Self::ensure_ms_signer(&multisig, &signer)?;
            }

            for signer in signers {
                Self::base_signer_removal(&multisig, signer);
            }

            NumberOfSigners::<T>::insert(&multisig, pending_num_of_signers);
            Ok(().into())
        }

        /// Changes the number of signatures required by a multisig. This must be called by the
        /// multisig itself.
        ///
        /// # Arguments
        /// * `sigs_required` - New number of required signatures.
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::change_sigs_required())]
        pub fn change_sigs_required(
            origin: OriginFor<T>,
            sigs_required: u64,
        ) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin)?;
            Self::ensure_ms(&multisig)?;
            Self::base_change_multisig_required_singatures(&multisig, sigs_required)?;
            Ok(().into())
        }

        /// Adds a multisig as a secondary key of current did if the current did is the creator of the
        /// multisig.
        ///
        /// # Arguments
        /// * `multisig` - multi sig address
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::make_multisig_secondary())]
        pub fn make_multisig_secondary(
            origin: OriginFor<T>,
            multisig: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let did = IdentityPallet::<T>::ensure_perms(origin)?;
            Self::ensure_ms(&multisig)?;
            Self::verify_sender_is_creator(did, &multisig)?;

            // Ensure the key is unlinked.
            IdentityPallet::<T>::ensure_key_did_unlinked(&multisig)?;

            // Add the multisig as a secondary key with no permissions.
            IdentityPallet::<T>::unsafe_join_identity(did, Permissions::empty(), multisig);
            Ok(().into())
        }

        /// Adds a multisig as the primary key of the current did if the current DID is the creator
        /// of the multisig.
        ///
        /// # Arguments
        /// * `multi_sig` - multi sig address
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::make_multisig_primary())]
        pub fn make_multisig_primary(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            optional_cdd_auth_id: Option<u64>,
        ) -> DispatchResultWithPostInfo {
            let did = Self::ensure_ms_creator(origin, &multisig)?;
            Self::ensure_ms(&multisig)?;
            IdentityPallet::<T>::common_rotate_primary_key(
                did,
                multisig,
                None,
                optional_cdd_auth_id,
            )?;
            Ok(().into())
        }

        /// Changes the number of signatures required by a multisig. This must be called by the creator of the multisig.
        ///
        /// # Arguments
        /// * `multisig` - The account identifier ([`AccountId`]) for the multi signature account.
        /// * `signatures_required` - The number of required signatures.
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::change_sigs_required_via_creator())]
        pub fn change_sigs_required_via_creator(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            signatures_required: u64,
        ) -> DispatchResultWithPostInfo {
            let caller_did = Self::ensure_ms_creator(origin, &multisig)?;
            ensure!(
                !LostCreatorPrivileges::<T>::get(caller_did),
                Error::<T>::CreatorControlsHaveBeenRemoved
            );
            Self::base_change_multisig_required_singatures(&multisig, signatures_required)?;
            Ok(().into())
        }

        /// Removes the creator ability to call `add_multisig_signers_via_creator`, `remove_multisig_signers_via_creator`
        /// and `change_sigs_required_via_creator`.
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_creator_controls())]
        pub fn remove_creator_controls(
            origin: OriginFor<T>,
            multisig: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let caller_did = Self::ensure_ms_creator(origin, &multisig)?;
            Self::base_remove_creator_controls(caller_did);
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted after creation of a multisig.
        /// Arguments: caller DID, multisig address, signers (pending approval), signers required.
        MultiSigCreated(
            IdentityId,
            T::AccountId,
            T::AccountId,
            Vec<T::AccountId>,
            u64,
        ),
        /// Event emitted after adding a proposal.
        /// Arguments: caller DID, multisig, proposal ID.
        ProposalAdded(IdentityId, T::AccountId, u64),
        /// Event emitted when a proposal is executed.
        /// Arguments: caller DID, multisig, proposal ID, result.
        ProposalExecuted(IdentityId, T::AccountId, u64, bool),
        /// Event emitted when a signatory is added.
        /// Arguments: caller DID, multisig, added signer.
        MultiSigSignerAdded(IdentityId, T::AccountId, T::AccountId),
        /// Event emitted when a multisig signatory is authorized to be added.
        /// Arguments: caller DID, multisig, authorized signer.
        MultiSigSignerAuthorized(IdentityId, T::AccountId, T::AccountId),
        /// Event emitted when a multisig signatory is removed.
        /// Arguments: caller DID, multisig, removed signer.
        MultiSigSignerRemoved(IdentityId, T::AccountId, T::AccountId),
        /// Event emitted when the number of required signers is changed.
        /// Arguments: caller DID, multisig, new required signers.
        MultiSigSignersRequiredChanged(IdentityId, T::AccountId, u64),
        /// Event emitted when the proposal get approved.
        /// Arguments: caller DID, multisig, authorized signer, proposal id.
        ProposalApproved(IdentityId, T::AccountId, T::AccountId, u64),
        /// Event emitted when a vote is cast in favor of rejecting a proposal.
        /// Arguments: caller DID, multisig, authorized signer, proposal id.
        ProposalRejectionVote(IdentityId, T::AccountId, T::AccountId, u64),
        /// Event emitted when a proposal is rejected.
        /// Arguments: caller DID, multisig, proposal ID.
        ProposalRejected(IdentityId, T::AccountId, u64),
        /// Event emitted when a proposal failed to execute.
        /// Arguments: caller DID, multisig, proposal ID, error.
        ProposalFailedToExecute(IdentityId, T::AccountId, u64, DispatchError),
    }

    /// Multisig module errors.
    #[pallet::error]
    pub enum Error<T> {
        /// The multisig is not attached to a CDD'd identity.
        CddMissing,
        /// The proposal does not exist.
        ProposalMissing,
        /// Multisig address.
        DecodingError,
        /// No signers.
        NoSigners,
        /// Too few or too many required signers.
        RequiredSignersOutOfBounds,
        /// Not a signer.
        NotASigner,
        /// No such multisig.
        NoSuchMultisig,
        /// Not enough signers.
        NotEnoughSigners,
        /// A nonce overflow.
        NonceOverflow,
        /// Already voted.
        AlreadyVoted,
        /// Already a signer.
        AlreadyASigner,
        /// Couldn't charge fee for the transaction.
        FailedToChargeFee,
        /// Identity provided is not the multisig's creator.
        IdentityNotCreator,
        /// Changing multisig parameters not allowed since multisig is a primary key.
        ChangeNotAllowed,
        /// Signer is an account key that is already associated with a multisig.
        SignerAlreadyLinkedToMultisig,
        /// Signer is an account key that is already associated with an identity.
        SignerAlreadyLinkedToIdentity,
        /// Multisig not allowed to add itself as a signer.
        MultisigNotAllowedToLinkToItself,
        /// The function can only be called by the primary key of the did
        NotPrimaryKey,
        /// Proposal was rejected earlier
        ProposalAlreadyRejected,
        /// Proposal has expired
        ProposalExpired,
        /// Proposal was executed earlier
        ProposalAlreadyExecuted,
        /// Max weight not enough to execute proposal.
        MaxWeightTooLow,
        /// Multisig is not attached to an identity
        MultisigMissingIdentity,
        /// More signers than required.
        TooManySigners,
        /// The creator is no longer allowed to call via creator extrinsics.
        CreatorControlsHaveBeenRemoved,
    }

    /// Nonce to ensure unique MultiSig addresses are generated; starts from 1.
    #[pallet::storage]
    #[pallet::getter(fn ms_nonce)]
    pub type MultiSigNonce<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Signers of a multisig. (multisig, signer) => bool.
    #[pallet::storage]
    pub type MultiSigSigners<T: Config> =
        StorageDoubleMap<_, Identity, T::AccountId, Twox64Concat, T::AccountId, bool, ValueQuery>;

    /// Number of approved/accepted signers of a multisig.
    #[pallet::storage]
    #[pallet::getter(fn number_of_signers)]
    pub type NumberOfSigners<T: Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

    /// Confirmations required before processing a multisig tx.
    #[pallet::storage]
    #[pallet::getter(fn ms_signs_required)]
    pub type MultiSigSignsRequired<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

    /// Number of transactions proposed in a multisig. Used as tx id; starts from 0.
    #[pallet::storage]
    #[pallet::getter(fn ms_tx_done)]
    pub type MultiSigTxDone<T: Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

    /// Proposals presented for voting to a multisig.
    ///
    /// multisig -> proposal id => Option<T::Proposal>.
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, T::Proposal>;

    /// A mapping of proposals to their IDs.
    #[pallet::storage]
    #[pallet::getter(fn proposal_ids)]
    pub type ProposalIds<T: Config> =
        StorageDoubleMap<_, Identity, T::AccountId, Blake2_128Concat, T::Proposal, u64>;

    /// Individual multisig signer votes.
    ///
    /// (multisig, proposal_id) -> signer => vote.
    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub type Votes<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        (T::AccountId, u64),
        Twox64Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    /// The multisig creator's identity.
    #[pallet::storage]
    pub type CreatorDid<T: Config> = StorageMap<_, Identity, T::AccountId, IdentityId, ValueQuery>;

    /// The count of approvals/rejections of a multisig proposal.
    ///
    /// multisig -> proposal id => Option<ProposalVoteCount>.
    #[pallet::storage]
    #[pallet::getter(fn proposal_vote_counts)]
    pub type ProposalVoteCounts<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, ProposalVoteCount>;

    /// The state of a multisig proposal
    ///
    /// multisig -> proposal id => Option<ProposalState>.
    #[pallet::storage]
    #[pallet::getter(fn proposal_states)]
    pub type ProposalStates<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Twox64Concat,
        u64,
        ProposalState<T::Moment>,
    >;

    /// Tracks creators who are no longer allowed to call via_creator extrinsics.
    #[pallet::storage]
    #[pallet::getter(fn lost_creator_privileges)]
    pub type LostCreatorPrivileges<T: Config> =
        StorageMap<_, Identity, IdentityId, bool, ValueQuery>;

    /// The last transaction version, used for `on_runtime_upgrade`.
    #[pallet::storage]
    #[pallet::getter(fn transaction_version)]
    pub(super) type TransactionVersion<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Storage version.
    #[pallet::storage]
    #[pallet::getter(fn storage_version)]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        #[doc = " The last transaction version, used for `on_runtime_upgrade`."]
        pub transaction_version: u32,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                transaction_version: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            MultiSigNonce::<T>::put(1);
            TransactionVersion::<T>::put(0);
            StorageVersion::<T>::put(Version::new(3));
        }
    }
}

impl<T: Config> Pallet<T> {
    fn ensure_primary_key(did: &IdentityId, sender: &T::AccountId) -> DispatchResult {
        ensure!(
            IdentityPallet::<T>::is_primary_key(did, sender),
            Error::<T>::NotPrimaryKey
        );
        Ok(())
    }

    fn ensure_ms_creator(
        origin: T::RuntimeOrigin,
        multisig: &T::AccountId,
    ) -> Result<IdentityId, DispatchError> {
        let (sender, did) = IdentityPallet::<T>::ensure_did(origin)?;
        Self::verify_sender_is_creator(did, multisig)?;
        Self::ensure_primary_key(&did, &sender)?;
        Ok(did)
    }

    fn ensure_ms(sender: &T::AccountId) -> DispatchResult {
        ensure!(
            CreatorDid::<T>::contains_key(sender),
            Error::<T>::NoSuchMultisig
        );
        Ok(())
    }

    fn ensure_ms_signer(ms: &T::AccountId, signer: &T::AccountId) -> DispatchResult {
        ensure!(
            MultiSigSigners::<T>::get(ms, signer),
            Error::<T>::NotASigner
        );
        Ok(())
    }

    fn ensure_sigs_in_bounds(signers: &[T::AccountId], required: u64) -> DispatchResult {
        ensure!(!signers.is_empty(), Error::<T>::NoSigners);
        ensure!(
            u64::try_from(signers.len()).unwrap_or_default() >= required && required > 0,
            Error::<T>::RequiredSignersOutOfBounds
        );
        Ok(())
    }

    fn ensure_proposal_is_active(multisig: &T::AccountId, proposal_id: u64) -> DispatchResult {
        match ProposalStates::<T>::get(multisig, proposal_id) {
            None => Err(Error::<T>::ProposalMissing.into()),
            Some(ProposalState::Rejected) => Err(Error::<T>::ProposalAlreadyRejected.into()),
            Some(ProposalState::ExecutionSuccessful | ProposalState::ExecutionFailed) => {
                Err(Error::<T>::ProposalAlreadyExecuted.into())
            }
            Some(ProposalState::Active { until: None }) => Ok(()),
            Some(ProposalState::Active { until: Some(until) }) => {
                // Ensure proposal is not expired
                ensure!(
                    until > pallet_timestamp::Pallet::<T>::get(),
                    Error::<T>::ProposalExpired
                );
                Ok(())
            }
        }
    }

    /// Adds an authorization for the accountKey to become a signer of multisig.
    fn base_add_auth_for_signers(
        multisig_owner: IdentityId,
        target: T::AccountId,
        multisig: T::AccountId,
    ) -> DispatchResult {
        IdentityPallet::<T>::add_auth(
            multisig_owner,
            Signatory::Account(target.clone()),
            AuthorizationData::AddMultiSigSigner(multisig.clone()),
            None,
        )?;
        Self::deposit_event(Event::MultiSigSignerAuthorized(
            multisig_owner,
            multisig,
            target,
        ));
        Ok(())
    }

    /// Removes a signer from the valid signer list for a given multisig.
    fn base_signer_removal(multisig: &T::AccountId, signer: T::AccountId) {
        IdentityPallet::<T>::remove_key_record(&signer, None);
        MultiSigSigners::<T>::remove(multisig, &signer);
        Self::deposit_event(Event::MultiSigSignerRemoved(
            Context::current_identity::<IdentityPallet<T>>().unwrap_or_default(),
            multisig.clone(),
            signer,
        ));
    }

    /// Changes the required signature count for a given multisig.
    fn base_change_sigs_required(multisig: &T::AccountId, sigs_required: u64) {
        MultiSigSignsRequired::<T>::insert(multisig, &sigs_required);
        Self::deposit_event(Event::MultiSigSignersRequiredChanged(
            Context::current_identity::<IdentityPallet<T>>().unwrap_or_default(),
            multisig.clone(),
            sigs_required,
        ));
    }

    /// Creates a multisig without precondition checks or emitting an event.
    pub fn base_create_multisig(
        sender: T::AccountId,
        signers: &[T::AccountId],
        sigs_required: u64,
    ) -> Result<T::AccountId, DispatchError> {
        let sender_did = Context::current_identity_or::<IdentityPallet<T>>(&sender)?;
        let new_nonce = Self::ms_nonce()
            .checked_add(1)
            .ok_or(Error::<T>::NonceOverflow)?;
        MultiSigNonce::<T>::put(new_nonce);
        let account_id = Self::get_multisig_address(sender, new_nonce)?;
        for signer in signers {
            IdentityPallet::<T>::add_auth(
                sender_did,
                Signatory::Account(signer.clone()),
                AuthorizationData::AddMultiSigSigner(account_id.clone()),
                None,
            )?;
        }
        MultiSigSignsRequired::<T>::insert(&account_id, &sigs_required);
        CreatorDid::<T>::insert(account_id.clone(), sender_did);
        Ok(account_id)
    }

    /// Creates a new proposal.
    pub fn base_create_proposal(
        multisig: &T::AccountId,
        sender_signer: T::AccountId,
        proposal: Box<T::Proposal>,
        expiry: Option<T::Moment>,
        proposal_to_id: bool,
    ) -> DispatchResultWithPostInfo {
        Self::ensure_ms_signer(multisig, &sender_signer)?;
        let max_weight = proposal.get_dispatch_info().weight;
        let caller_did = CreatorDid::<T>::get(multisig);
        let proposal_id = Self::ms_tx_done(multisig);
        Proposals::<T>::insert(multisig, proposal_id, &*proposal);
        if proposal_to_id {
            // Only use the `Proposal` -> id map for `create_or_approve_proposal` calls.
            ProposalIds::<T>::insert(multisig, *proposal, proposal_id);
        }
        ProposalVoteCounts::<T>::insert(multisig, proposal_id, ProposalVoteCount::default());
        ProposalStates::<T>::insert(multisig, proposal_id, ProposalState::new(expiry));
        // Since proposal_ids are always only incremented by 1, they can not overflow.
        let next_proposal_id: u64 = proposal_id + 1u64;
        MultiSigTxDone::<T>::insert(multisig, next_proposal_id);
        Self::deposit_event(Event::ProposalAdded(
            caller_did,
            multisig.clone(),
            proposal_id,
        ));
        Self::base_approve(multisig, sender_signer, proposal_id, max_weight)
    }

    /// Creates or approves a multisig proposal.
    pub fn base_create_or_approve_proposal(
        multisig: &T::AccountId,
        sender_signer: T::AccountId,
        proposal: Box<T::Proposal>,
        expiry: Option<T::Moment>,
    ) -> DispatchResultWithPostInfo {
        if let Some(proposal_id) = Self::proposal_ids(multisig, &*proposal) {
            let max_weight = proposal.get_dispatch_info().weight;
            // This is an existing proposal.
            Self::base_approve(multisig, sender_signer, proposal_id, max_weight)
        } else {
            // The proposal is new.
            Self::base_create_proposal(multisig, sender_signer, proposal, expiry, true)?;
            Ok(().into())
        }
    }

    /// Approves a multisig proposal and executes it if enough signatures have been received.
    fn base_approve(
        multisig: &T::AccountId,
        signer: T::AccountId,
        proposal_id: u64,
        max_weight: Weight,
    ) -> DispatchResultWithPostInfo {
        Self::ensure_proposal_is_active(multisig, proposal_id)?;
        Self::ensure_ms_signer(multisig, &signer)?;
        ensure!(
            !Self::votes((multisig, proposal_id), &signer),
            Error::<T>::AlreadyVoted
        );
        ensure!(
            Proposals::<T>::contains_key(multisig, proposal_id),
            Error::<T>::ProposalMissing
        );

        let mut vote_count = ProposalVoteCounts::<T>::try_get(multisig, proposal_id)
            .map_err(|_| Error::<T>::ProposalMissing)?;
        vote_count.approvals += 1u64;
        let creator_did = CreatorDid::<T>::get(multisig);
        let execute_proposal = vote_count.approvals >= Self::ms_signs_required(multisig);

        // Update storage
        Votes::<T>::insert((multisig, proposal_id), &signer, true);
        ProposalVoteCounts::<T>::insert(multisig, proposal_id, vote_count);
        // emit proposal approved event
        Self::deposit_event(Event::ProposalApproved(
            creator_did,
            multisig.clone(),
            signer,
            proposal_id,
        ));
        if execute_proposal {
            Self::execute_proposal(multisig, proposal_id, creator_did, max_weight)
        } else {
            Ok(().into())
        }
    }

    /// Executes a proposal if it has enough approvals
    pub(crate) fn execute_proposal(
        multisig: &T::AccountId,
        proposal_id: u64,
        creator_did: IdentityId,
        max_weight: Weight,
    ) -> DispatchResultWithPostInfo {
        // Get the proposal.
        let proposal = Proposals::<T>::try_get(multisig, proposal_id)
            .map_err(|_| Error::<T>::ProposalMissing)?;

        // Ensure `max_weight` was enough to cover the worst-case weight.
        let proposal_weight = proposal.get_dispatch_info().weight;
        ensure!(
            proposal_weight.all_lte(max_weight),
            Error::<T>::MaxWeightTooLow
        );

        let (res, actual_weight) = match with_call_metadata(proposal.get_call_metadata(), || {
            proposal.dispatch(frame_system::RawOrigin::Signed(multisig.clone()).into())
        }) {
            Ok(post_info) => {
                ProposalStates::<T>::insert(
                    multisig,
                    proposal_id,
                    ProposalState::ExecutionSuccessful,
                );
                (true, post_info.actual_weight)
            }
            Err(e) => {
                ProposalStates::<T>::insert(multisig, proposal_id, ProposalState::ExecutionFailed);
                Self::deposit_event(Event::ProposalFailedToExecute(
                    creator_did,
                    multisig.clone(),
                    proposal_id,
                    e.error,
                ));
                (false, e.post_info.actual_weight)
            }
        };
        Self::deposit_event(Event::ProposalExecuted(
            creator_did,
            multisig.clone(),
            proposal_id,
            res,
        ));
        // If the proposal call doesn't return an `actual_weight`, then default to `proposal_weight`.
        // Also include the overhead of this `execute_proposal` method.
        let actual_weight = actual_weight
            .unwrap_or(proposal_weight)
            .saturating_add(<T as Config>::WeightInfo::execute_proposal());
        Ok(Some(actual_weight).into())
    }

    /// Rejects a multisig proposal
    fn base_reject(
        multisig: &T::AccountId,
        signer: T::AccountId,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::ensure_proposal_is_active(multisig, proposal_id)?;
        Self::ensure_ms_signer(multisig, &signer)?;

        let mut vote_count = ProposalVoteCounts::<T>::try_get(multisig, proposal_id)
            .map_err(|_| Error::<T>::ProposalMissing)?;

        // Only allow the original proposer to change their vote if no one else has voted
        let mut proposal_owner = false;
        if Votes::<T>::get((multisig, proposal_id), &signer) {
            if vote_count.rejections != 0 || vote_count.approvals != 1 {
                return Err(Error::<T>::AlreadyVoted.into());
            }
            proposal_owner = true;
        }

        vote_count.rejections += 1u64;
        let current_did = Context::current_identity::<IdentityPallet<T>>().unwrap_or_default();
        let approvals_needed = Self::ms_signs_required(multisig.clone());
        let ms_signers = Self::number_of_signers(multisig.clone());
        if vote_count.rejections > ms_signers.saturating_sub(approvals_needed) || proposal_owner {
            if proposal_owner {
                vote_count.approvals = 0;
            }
            ProposalStates::<T>::insert(multisig, proposal_id, ProposalState::Rejected);
            Self::deposit_event(Event::ProposalRejected(
                current_did,
                multisig.clone(),
                proposal_id,
            ));
        }
        // Update storage
        Votes::<T>::insert((multisig, proposal_id), &signer, true);
        ProposalVoteCounts::<T>::insert(multisig, proposal_id, vote_count);
        // emit proposal rejected event
        Self::deposit_event(Event::ProposalRejectionVote(
            current_did,
            multisig.clone(),
            signer,
            proposal_id,
        ));
        Ok(())
    }

    /// Accepts and processed an addition of a signer to a multisig.
    pub fn base_accept_multisig_signer(signer: T::AccountId, auth_id: u64) -> DispatchResult {
        IdentityPallet::<T>::accept_auth_with(
            &Signatory::Account(signer.clone()),
            auth_id,
            |data, auth_by| {
                let multisig = extract_auth!(data, AddMultiSigSigner(ms));

                Self::ensure_ms(&multisig)?;

                ensure!(
                    Self::is_changing_signers_allowed(&multisig),
                    Error::<T>::ChangeNotAllowed
                );

                ensure!(
                    !MultiSigSigners::<T>::get(&multisig, &signer),
                    Error::<T>::AlreadyASigner
                );

                let (to_identity, to_multisig) = IdentityPallet::<T>::is_key_linked(&signer);
                // Don't allow a signer key that is a primary key, secondary key.
                ensure!(!to_identity, Error::<T>::SignerAlreadyLinkedToIdentity);
                // Don't allow a signer key that is already a signer to another multisig.
                ensure!(!to_multisig, Error::<T>::SignerAlreadyLinkedToMultisig);
                // Don't allow a multisig to add itself as a signer to itself
                // NB - you can add a multisig as a signer to a different multisig
                ensure!(
                    signer != multisig,
                    Error::<T>::MultisigNotAllowedToLinkToItself
                );

                let ms_identity = CreatorDid::<T>::get(&multisig);
                IdentityPallet::<T>::ensure_auth_by(ms_identity, auth_by)?;

                MultiSigSigners::<T>::insert(&multisig, &signer, true);
                NumberOfSigners::<T>::mutate(&multisig, |x| *x += 1u64);

                IdentityPallet::<T>::add_key_record(
                    &signer,
                    KeyRecord::MultiSigSignerKey(multisig.clone()),
                );
                Self::deposit_event(Event::MultiSigSignerAdded(ms_identity, multisig, signer));
                Ok(())
            },
        )
    }

    /// Gets the next available multisig account ID.
    pub fn get_next_multisig_address(sender: T::AccountId) -> Result<T::AccountId, DispatchError> {
        // Nonce is always only incremented by small numbers and hence can never overflow 64 bits.
        // Also, this is just a helper function that does not modify state.
        let new_nonce = Self::ms_nonce() + 1;
        Self::get_multisig_address(sender, new_nonce)
    }

    /// Constructs a multisig account given a nonce.
    pub fn get_multisig_address(
        sender: T::AccountId,
        nonce: u64,
    ) -> Result<T::AccountId, DispatchError> {
        let h: T::Hash = T::Hashing::hash(&(b"MULTI_SIG", nonce, sender).encode());
        Ok(T::AccountId::decode(&mut &h.encode()[..]).map_err(|_| Error::<T>::DecodingError)?)
    }

    /// Helper function that checks if someone is an authorized signer of a multisig or not.
    pub fn ms_signers(multi_sig: T::AccountId, signer: T::AccountId) -> bool {
        MultiSigSigners::<T>::get(multi_sig, signer)
    }

    /// Checks whether changing the list of signers is allowed in a multisig.
    pub fn is_changing_signers_allowed(multisig: &T::AccountId) -> bool {
        if IdentityPallet::<T>::cdd_auth_for_primary_key_rotation() {
            if let Some(did) = IdentityPallet::<T>::get_identity(multisig) {
                if IdentityPallet::<T>::is_primary_key(&did, multisig) {
                    return false;
                }
            }
        }
        true
    }

    pub fn verify_sender_is_creator(
        sender_did: IdentityId,
        multisig: &T::AccountId,
    ) -> DispatchResult {
        let creator_did =
            CreatorDid::<T>::try_get(multisig).map_err(|_| Error::<T>::MultisigMissingIdentity)?;
        ensure!(creator_did == sender_did, Error::<T>::IdentityNotCreator);
        Ok(())
    }

    /// Changes the number of required signatures for the given `multisig` to `signatures_required`.
    fn base_change_multisig_required_singatures(
        multisig: &T::AccountId,
        signatures_required: u64,
    ) -> DispatchResult {
        ensure!(
            NumberOfSigners::<T>::get(multisig) >= signatures_required,
            Error::<T>::NotEnoughSigners
        );
        ensure!(
            Self::is_changing_signers_allowed(multisig),
            Error::<T>::ChangeNotAllowed
        );
        Self::base_change_sigs_required(multisig, signatures_required);
        Ok(())
    }

    /// Removes the creator ability to call `add_multisig_signers_via_creator`, `remove_multisig_signers_via_creator`
    /// and `change_sigs_required_via_creator`.
    fn base_remove_creator_controls(creator_did: IdentityId) {
        LostCreatorPrivileges::<T>::insert(creator_did, true);
    }
}

impl<T: Config> MultiSigSubTrait<T::AccountId> for Pallet<T> {
    fn is_multisig(account_id: &T::AccountId) -> bool {
        CreatorDid::<T>::contains_key(account_id)
    }
}

pub mod migration {
    use super::*;
    use sp_runtime::runtime_logger::RuntimeLogger;

    mod v2 {
        use super::*;

        decl_storage! {
            trait Store for Module<T: Config> as MultiSig {
                pub MultiSigToIdentity : map hasher(identity) T::AccountId => IdentityId;

                pub MultiSigSigners: double_map hasher(identity) T::AccountId, hasher(twox_64_concat) Signatory<T::AccountId> => bool;
            }
        }

        decl_module! {
            pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
        }
    }

    pub fn migrate_to_v3<T: Config>(weight: &mut Weight) {
        RuntimeLogger::init();
        // Remove old storage.
        polymesh_primitives::migrate::kill_item(b"MultiSig", b"ProposalDetail");

        migrate_signatory::<T>(weight);
        migrate_creator_did::<T>(weight);
    }

    fn migrate_signatory<T: Config>(weight: &mut Weight) {
        log::info!(" >>> Migrate Signatory values to only AccountId");
        let mut sig_count = 0;
        let mut reads = 0;
        let mut writes = 0;
        v2::MultiSigSigners::<T>::drain().for_each(|(ms, signer, value)| {
            reads += 1;
            sig_count += 1;
            match signer {
                Signatory::Account(signer) => {
                    writes += 1;
                    MultiSigSigners::<T>::insert(ms, signer, value);
                }
                _ => {
                    // Shouldn't be any Identity signatories.
                }
            }
        });
        weight.saturating_accrue(DbWeight::get().reads_writes(reads, writes));
        log::info!(" >>> {sig_count} Signers migrated.");
    }

    fn migrate_creator_did<T: Config>(weight: &mut Weight) {
        log::info!(" >>> Migrate MultiSigToIdentity to CreatorDid");
        let mut did_count = 0;
        let mut reads = 0;
        let mut writes = 0;
        v2::MultiSigToIdentity::<T>::drain().for_each(|(ms, did)| {
            reads += 1;
            did_count += 1;
            CreatorDid::<T>::insert(ms, did);
            writes += 1;
        });
        weight.saturating_accrue(DbWeight::get().reads_writes(reads, writes));
        log::info!(" >>> {did_count} CreatorDids migrated.");
    }
}
