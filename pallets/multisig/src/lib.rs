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
//! - `create_multisig_account` - Creates a multisig account without precondition checks or emitting
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
use frame_support::storage::{StorageDoubleMap, StorageValue};
use frame_support::traits::{Get, GetCallMetadata};
use frame_support::{decl_error, decl_module, decl_storage, ensure};
use frame_system::{ensure_signed, Config as FrameConfig};
use sp_runtime::traits::{Dispatchable, Hash};
use sp_std::convert::TryFrom;
use sp_std::prelude::*;
use sp_std::result::Result as StdResult;

use pallet_identity::PermissionedCallOriginData;
use pallet_permissions::with_call_metadata;
pub use polymesh_common_utilities::multisig::{Event, MultiSigSubTrait, RawEvent, WeightInfo};
use polymesh_common_utilities::traits::identity::Config as IdentityConfig;
use polymesh_common_utilities::Context;
use polymesh_primitives::multisig::{ProposalDetails, ProposalStatus};
use polymesh_primitives::{
    extract_auth, storage_migrate_on, storage_migration_ver, AuthorizationData, IdentityId,
    KeyRecord, Permissions, Signatory,
};
//use polymesh_runtime_common::RocksDbWeight as DbWeight;
use frame_support::weights::constants::RocksDbWeight as DbWeight;

/// Either the ID of a successfully created multisig account or an error.
type CreateMultisigAccountResult<T> = StdResult<<T as FrameConfig>::AccountId, DispatchError>;
type Identity<T> = pallet_identity::Module<T>;

pub const NAME: &[u8] = b"MultiSig";

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

pub trait Config: frame_system::Config + IdentityConfig {
    /// The overarching event type.
    type RuntimeEvent: From<Event<Self>> + Into<<Self as frame_system::Config>::RuntimeEvent>;
    /// Weight information for extrinsics in the multisig pallet.
    type WeightInfo: WeightInfo;
}

decl_storage! {
    trait Store for Module<T: Config> as MultiSig {
        /// Nonce to ensure unique MultiSig addresses are generated; starts from 1.
        pub MultiSigNonce get(fn ms_nonce) build(|_| 1u64): u64;
        /// Signers of a multisig. (multisig, signer) => bool.
        pub MultiSigSigners: double_map hasher(identity) T::AccountId, hasher(twox_64_concat) T::AccountId => bool;
        /// Number of approved/accepted signers of a multisig.
        pub NumberOfSigners get(fn number_of_signers): map hasher(identity) T::AccountId => u64;
        /// Confirmations required before processing a multisig tx.
        pub MultiSigSignsRequired get(fn ms_signs_required): map hasher(identity) T::AccountId => u64;
        /// Number of transactions proposed in a multisig. Used as tx id; starts from 0.
        pub MultiSigTxDone get(fn ms_tx_done): map hasher(identity) T::AccountId => u64;
        /// Proposals presented for voting to a multisig.
        ///
        /// multisig -> proposal id => Option<T::Proposal>.
        pub Proposals get(fn proposals):
            double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) u64 => Option<T::Proposal>;
        /// A mapping of proposals to their IDs.
        pub ProposalIds get(fn proposal_ids):
            double_map hasher(identity) T::AccountId, hasher(blake2_128_concat) T::Proposal => Option<u64>;
        /// Individual multisig signer votes.
        ///
        /// (multisig, proposal_id) -> signer => vote.
        pub Votes get(fn votes):
            double_map hasher(twox_64_concat) (T::AccountId, u64), hasher(twox_64_concat) T::AccountId => bool;
        /// The multisig creator's identity.
        pub CreatorDid: map hasher(identity) T::AccountId => IdentityId;
        /// Details of a multisig proposal
        ///
        /// multisig -> proposal id => ProposalDetails.
        pub ProposalDetail get(fn proposal_detail):
            double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) u64 => ProposalDetails<T::Moment>;
        /// Tracks creators who are no longer allowed to call via_creator extrinsics.
        pub LostCreatorPrivileges get(fn lost_creator_privileges): map hasher(identity) IdentityId => bool;

        /// The last transaction version, used for `on_runtime_upgrade`.
        TransactionVersion get(fn transaction_version) config(): u32;
        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(3)): Version;
    }
}

decl_module! {
    /// A multisig module.
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {
        type Error = Error<T>;

        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            use sp_version::RuntimeVersion;
            use polymesh_primitives::migrate::kill_item;
            let mut weight = Weight::zero();

            // Kill pending proposals if the transaction version is upgraded
            let current_version = <T::Version as Get<RuntimeVersion>>::get().transaction_version;
            if TransactionVersion::get() < current_version {
                TransactionVersion::set(current_version);
                // TODO: Replace this code with `Proposal*::remove*` calls.
                // Doing so will provide compile-time checks.  The current code
                // will fail silently if storage names changes.
                for item in &["Proposals", "ProposalIds", "ProposalDetail", "Votes"] {
                    kill_item(NAME, item.as_bytes())
                }
                // TODO: Improve this weight.
                weight.saturating_accrue(DbWeight::get().reads_writes(4, 4));
            }

            storage_migrate_on!(StorageVersion, 3, {
                migration::migrate_to_v3::<T>(&mut weight);
            });
            weight
        }

        /// Creates a multisig
        ///
        /// # Arguments
        /// * `signers` - Signers of the multisig (They need to accept authorization before they are actually added).
        /// * `sigs_required` - Number of sigs required to process a multi-sig tx.
        #[weight = <T as Config>::WeightInfo::create_multisig(signers.len() as u32)]
        pub fn create_multisig(origin, signers: Vec<T::AccountId>, sigs_required: u64) {
            let PermissionedCallOriginData {
                sender,
                primary_did,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            Self::ensure_sigs_in_bounds(&signers, sigs_required)?;
            let account_id = Self::create_multisig_account(
                sender.clone(),
                signers.as_slice(),
                sigs_required
            )?;
            Self::deposit_event(RawEvent::MultiSigCreated(primary_did, account_id, sender, signers, sigs_required));
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
        #[weight = {
          <T as Config>::WeightInfo::create_or_approve_proposal()
            .saturating_add(<T as Config>::WeightInfo::execute_proposal())
            .saturating_add(proposal.get_dispatch_info().weight)
        }]
        pub fn create_or_approve_proposal(
            origin,
            multisig: T::AccountId,
            proposal: Box<T::Proposal>,
            expiry: Option<T::Moment>,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            with_base_weight(<T as Config>::WeightInfo::create_or_approve_proposal(), || {
                Self::base_create_or_approve_proposal(multisig, signer, proposal, expiry)
            })
        }

        /// Creates a multisig proposal
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal` - Proposal to be voted on.
        /// * `expiry` - Optional proposal expiry time.
        ///
        /// If this is 1 out of `m` multisig, the proposal will be immediately executed.
        #[weight = {
          <T as Config>::WeightInfo::create_proposal()
            .saturating_add(<T as Config>::WeightInfo::execute_proposal())
            .saturating_add(proposal.get_dispatch_info().weight)
        }]
        pub fn create_proposal(
            origin,
            multisig: T::AccountId,
            proposal: Box<T::Proposal>,
            expiry: Option<T::Moment>,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            with_base_weight(<T as Config>::WeightInfo::create_proposal(), || {
                Self::base_create_proposal(multisig, signer, proposal, expiry, false)
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
        #[weight = {
          <T as Config>::WeightInfo::approve()
            .saturating_add(<T as Config>::WeightInfo::execute_proposal())
            .saturating_add(*max_weight)
        }]
        pub fn approve(origin, multisig: T::AccountId, proposal_id: u64, max_weight: Weight) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            with_base_weight(<T as Config>::WeightInfo::approve(), || {
                Self::base_approve(multisig, signer, proposal_id, max_weight)
            })
        }

        /// Rejects a multisig proposal using the caller's secondary key (`AccountId`).
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal_id` - Proposal id to reject.
        /// If quorum is reached, the proposal will be immediately executed.
        #[weight = <T as Config>::WeightInfo::reject()]
        pub fn reject(origin, multisig: T::AccountId, proposal_id: u64) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            Self::base_reject(multisig, signer, proposal_id)
        }

        /// Accepts a multisig signer authorization given to signer's key (AccountId).
        ///
        /// # Arguments
        /// * `auth_id` - Auth id of the authorization.
        #[weight = <T as Config>::WeightInfo::accept_multisig_signer()]
        pub fn accept_multisig_signer(origin, auth_id: u64) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            Self::base_accept_multisig_signer(signer, auth_id)
        }

        /// Adds a signer to the multisig. This must be called by the multisig itself.
        ///
        /// # Arguments
        /// * `signer` - Signer to add.
        #[weight = <T as Config>::WeightInfo::add_multisig_signer()]
        pub fn add_multisig_signer(origin, signer: T::AccountId) {
            let sender = ensure_signed(origin)?;
            // Ensure the caller is a MultiSig and get it's creator DID.
            let did = CreatorDid::<T>::try_get(&sender)
                .map_err(|_| Error::<T>::NoSuchMultisig)?;
            Self::base_add_auth_for_signers(did, signer, sender)?;
        }

        /// Removes a signer from the multisig. This must be called by the multisig itself.
        ///
        /// # Arguments
        /// * `signer` - Signer to remove.
        #[weight = <T as Config>::WeightInfo::remove_multisig_signer()]
        pub fn remove_multisig_signer(origin, signer: T::AccountId) {
            let sender = ensure_signed(origin)?;
            Self::ensure_ms(&sender)?;
            Self::ensure_ms_signer(&sender, &signer)?;
            ensure!(
                <NumberOfSigners<T>>::get(&sender) > <MultiSigSignsRequired<T>>::get(&sender),
                Error::<T>::NotEnoughSigners
            );
            ensure!(Self::is_changing_signers_allowed(&sender), Error::<T>::ChangeNotAllowed);
            <NumberOfSigners<T>>::mutate(&sender, |x| *x -= 1u64);
            Self::base_signer_removal(sender, signer);
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
        #[weight = <T as Config>::WeightInfo::add_multisig_signers_via_creator(signers.len() as u32)]
        pub fn add_multisig_signers_via_creator(origin, multisig: T::AccountId, signers: Vec<T::AccountId>) {
            let caller_did = Self::ensure_ms_creator(origin, &multisig)?;
            ensure!(
                !LostCreatorPrivileges::get(caller_did),
                Error::<T>::CreatorControlsHaveBeenRemoved
            );
            for signer in signers {
                Self::base_add_auth_for_signers(caller_did, signer, multisig.clone())?;
            }
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
        #[weight = <T as Config>::WeightInfo::remove_multisig_signers_via_creator(signers.len() as u32)]
        pub fn remove_multisig_signers_via_creator(origin, multisig: T::AccountId, signers: Vec<T::AccountId>) {
            let caller_did = Self::ensure_ms_creator(origin, &multisig)?;
            ensure!(
                !LostCreatorPrivileges::get(caller_did),
                Error::<T>::CreatorControlsHaveBeenRemoved
            );

            ensure!(Self::is_changing_signers_allowed(&multisig), Error::<T>::ChangeNotAllowed);
            let signers_len: u64 = u64::try_from(signers.len()).unwrap_or_default();

            let pending_num_of_signers = <NumberOfSigners<T>>::get(&multisig).checked_sub(signers_len)
                .ok_or(Error::<T>::TooManySigners)?;
            ensure!(
                pending_num_of_signers >= <MultiSigSignsRequired<T>>::get(&multisig),
                Error::<T>::NotEnoughSigners
            );

            for signer in &signers {
                Self::ensure_ms_signer(&multisig, &signer)?;
            }

            for signer in signers {
                Self::base_signer_removal(multisig.clone(), signer);
            }

            <NumberOfSigners<T>>::insert(&multisig, pending_num_of_signers);
        }

        /// Changes the number of signatures required by a multisig. This must be called by the
        /// multisig itself.
        ///
        /// # Arguments
        /// * `sigs_required` - New number of required signatures.
        #[weight = <T as Config>::WeightInfo::change_sigs_required()]
        pub fn change_sigs_required(origin, sigs_required: u64) {
            let account_id = ensure_signed(origin)?;
            Self::ensure_ms(&account_id)?;
            Self::base_change_multisig_required_singatures(account_id, sigs_required)?;
        }

        /// Adds a multisig as a secondary key of current did if the current did is the creator of the
        /// multisig.
        ///
        /// # Arguments
        /// * `multisig` - multi sig address
        #[weight = <T as Config>::WeightInfo::make_multisig_secondary()]
        pub fn make_multisig_secondary(origin, multisig: T::AccountId) {
            let did = <Identity<T>>::ensure_perms(origin)?;
            Self::ensure_ms(&multisig)?;
            Self::verify_sender_is_creator(did, &multisig)?;

            // Ensure the key is unlinked.
            <Identity<T>>::ensure_key_did_unlinked(&multisig)?;

            // Add the multisig as a secondary key with no permissions.
            <Identity<T>>::unsafe_join_identity(did, Permissions::empty(), multisig);
        }

        /// Adds a multisig as the primary key of the current did if the current DID is the creator
        /// of the multisig.
        ///
        /// # Arguments
        /// * `multi_sig` - multi sig address
        #[weight = <T as Config>::WeightInfo::make_multisig_primary()]
        pub fn make_multisig_primary(origin, multisig: T::AccountId, optional_cdd_auth_id: Option<u64>) -> DispatchResult {
            let did = Self::ensure_ms_creator(origin, &multisig)?;
            Self::ensure_ms(&multisig)?;
            <Identity<T>>::common_rotate_primary_key(did, multisig, None, optional_cdd_auth_id)
        }

        /// Changes the number of signatures required by a multisig. This must be called by the creator of the multisig.
        ///
        /// # Arguments
        /// * `multisig_account` - The account identifier ([`AccountId`]) for the multi signature account.
        /// * `signatures_required` - The number of required signatures.
        #[weight = <T as Config>::WeightInfo::change_sigs_required_via_creator()]
        pub fn change_sigs_required_via_creator(origin, multisig_account: T::AccountId, signatures_required: u64) {
            let caller_did = Self::ensure_ms_creator(origin, &multisig_account)?;
            ensure!(
                !LostCreatorPrivileges::get(caller_did),
                Error::<T>::CreatorControlsHaveBeenRemoved
            );
            Self::base_change_multisig_required_singatures(multisig_account, signatures_required)?;
        }

        /// Removes the creator ability to call `add_multisig_signers_via_creator`, `remove_multisig_signers_via_creator`
        /// and `change_sigs_required_via_creator`.
        #[weight = <T as Config>::WeightInfo::remove_creator_controls()]
        pub fn remove_creator_controls(origin, multisig_account: T::AccountId) {
            let caller_did = Self::ensure_ms_creator(origin, &multisig_account)?;
            Self::base_remove_creator_controls(caller_did);
        }
    }
}

decl_error! {
    /// Multisig module errors.
    pub enum Error for Module<T: Config> {
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
}

impl<T: Config> Module<T> {
    fn ensure_primary_key(did: &IdentityId, sender: &T::AccountId) -> DispatchResult {
        ensure!(
            <Identity<T>>::is_primary_key(did, sender),
            Error::<T>::NotPrimaryKey
        );
        Ok(())
    }

    fn ensure_ms_creator(
        origin: T::RuntimeOrigin,
        multisig: &T::AccountId,
    ) -> Result<IdentityId, DispatchError> {
        let (sender, did) = Identity::<T>::ensure_did(origin)?;
        Self::verify_sender_is_creator(did, multisig)?;
        Self::ensure_primary_key(&did, &sender)?;
        Ok(did)
    }

    fn ensure_ms(sender: &T::AccountId) -> DispatchResult {
        ensure!(
            <CreatorDid<T>>::contains_key(sender),
            Error::<T>::NoSuchMultisig
        );
        Ok(())
    }

    fn ensure_ms_signer(ms: &T::AccountId, signer: &T::AccountId) -> DispatchResult {
        ensure!(
            <MultiSigSigners<T>>::get(ms, signer),
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

    /// Adds an authorization for the accountKey to become a signer of multisig.
    fn base_add_auth_for_signers(
        multisig_owner: IdentityId,
        target: T::AccountId,
        multisig: T::AccountId,
    ) -> DispatchResult {
        <Identity<T>>::add_auth(
            multisig_owner,
            Signatory::Account(target.clone()),
            AuthorizationData::AddMultiSigSigner(multisig.clone()),
            None,
        )?;
        Self::deposit_event(RawEvent::MultiSigSignerAuthorized(
            multisig_owner,
            multisig,
            target,
        ));
        Ok(())
    }

    /// Removes a signer from the valid signer list for a given multisig.
    fn base_signer_removal(multisig: T::AccountId, signer: T::AccountId) {
        Identity::<T>::remove_key_record(&signer, None);
        <MultiSigSigners<T>>::remove(&multisig, &signer);
        Self::deposit_event(RawEvent::MultiSigSignerRemoved(
            Context::current_identity::<Identity<T>>().unwrap_or_default(),
            multisig,
            signer,
        ));
    }

    /// Changes the required signature count for a given multisig.
    fn base_change_sigs_required(multisig: T::AccountId, sigs_required: u64) {
        <MultiSigSignsRequired<T>>::insert(&multisig, &sigs_required);
        Self::deposit_event(RawEvent::MultiSigSignersRequiredChanged(
            Context::current_identity::<Identity<T>>().unwrap_or_default(),
            multisig,
            sigs_required,
        ));
    }

    /// Creates a multisig account without precondition checks or emitting an event.
    pub fn create_multisig_account(
        sender: T::AccountId,
        signers: &[T::AccountId],
        sigs_required: u64,
    ) -> CreateMultisigAccountResult<T> {
        let sender_did = Context::current_identity_or::<Identity<T>>(&sender)?;
        let new_nonce = Self::ms_nonce()
            .checked_add(1)
            .ok_or(Error::<T>::NonceOverflow)?;
        MultiSigNonce::put(new_nonce);
        let account_id = Self::get_multisig_address(sender, new_nonce)?;
        for signer in signers {
            <Identity<T>>::add_auth(
                sender_did,
                Signatory::Account(signer.clone()),
                AuthorizationData::AddMultiSigSigner(account_id.clone()),
                None,
            )?;
        }
        <MultiSigSignsRequired<T>>::insert(&account_id, &sigs_required);
        <CreatorDid<T>>::insert(account_id.clone(), sender_did);
        Ok(account_id)
    }

    /// Creates a new proposal.
    pub fn base_create_proposal(
        multisig: T::AccountId,
        sender_signer: T::AccountId,
        proposal: Box<T::Proposal>,
        expiry: Option<T::Moment>,
        proposal_to_id: bool,
    ) -> DispatchResultWithPostInfo {
        Self::ensure_ms_signer(&multisig, &sender_signer)?;
        let max_weight = proposal.get_dispatch_info().weight;
        let caller_did = <CreatorDid<T>>::get(&multisig);
        let proposal_id = Self::ms_tx_done(multisig.clone());
        <Proposals<T>>::insert(multisig.clone(), proposal_id, proposal.clone());
        if proposal_to_id {
            // Only use the `Proposal` -> id map for `create_or_approve_proposal` calls.
            <ProposalIds<T>>::insert(multisig.clone(), *proposal, proposal_id);
        }
        <ProposalDetail<T>>::insert(&multisig, proposal_id, ProposalDetails::new(expiry));
        // Since proposal_ids are always only incremented by 1, they can not overflow.
        let next_proposal_id: u64 = proposal_id + 1u64;
        <MultiSigTxDone<T>>::insert(multisig.clone(), next_proposal_id);
        Self::deposit_event(RawEvent::ProposalAdded(
            caller_did,
            multisig.clone(),
            proposal_id,
        ));
        Self::base_approve(multisig, sender_signer, proposal_id, max_weight)
    }

    /// Creates or approves a multisig proposal.
    pub fn base_create_or_approve_proposal(
        multisig: T::AccountId,
        sender_signer: T::AccountId,
        proposal: Box<T::Proposal>,
        expiry: Option<T::Moment>,
    ) -> DispatchResultWithPostInfo {
        if let Some(proposal_id) = Self::proposal_ids(&multisig, &*proposal) {
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
        multisig: T::AccountId,
        signer: T::AccountId,
        proposal_id: u64,
        max_weight: Weight,
    ) -> DispatchResultWithPostInfo {
        Self::ensure_ms_signer(&multisig, &signer)?;
        ensure!(
            !Self::votes((&multisig, proposal_id), &signer),
            Error::<T>::AlreadyVoted
        );
        ensure!(
            <Proposals<T>>::contains_key(&multisig, proposal_id),
            Error::<T>::ProposalMissing
        );

        let mut proposal_details = Self::proposal_detail(&multisig, proposal_id);
        proposal_details.approvals += 1u64;
        let creator_did = <CreatorDid<T>>::get(&multisig);
        let mut execute_proposal = false;
        match proposal_details.status {
            ProposalStatus::Invalid => return Err(Error::<T>::ProposalMissing.into()),
            ProposalStatus::Rejected => return Err(Error::<T>::ProposalAlreadyRejected.into()),
            ProposalStatus::ExecutionSuccessful | ProposalStatus::ExecutionFailed => {
                return Err(Error::<T>::ProposalAlreadyExecuted.into())
            }
            ProposalStatus::ActiveOrExpired => {
                // Ensure proposal is not expired
                if let Some(expiry) = proposal_details.expiry {
                    ensure!(
                        expiry > <pallet_timestamp::Pallet<T>>::get(),
                        Error::<T>::ProposalExpired
                    );
                }
                if proposal_details.approvals >= Self::ms_signs_required(&multisig) {
                    execute_proposal = true;
                }
            }
        }
        // Update storage
        <Votes<T>>::insert((&multisig, proposal_id), &signer, true);
        <ProposalDetail<T>>::insert(&multisig, proposal_id, proposal_details);
        // emit proposal approved event
        Self::deposit_event(RawEvent::ProposalApproved(
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
        multisig: T::AccountId,
        proposal_id: u64,
        creator_did: IdentityId,
        max_weight: Weight,
    ) -> DispatchResultWithPostInfo {
        // Get the proposal.
        let proposal = <Proposals<T>>::try_get(&multisig, proposal_id)
            .map_err(|_| Error::<T>::ProposalMissing)?;

        // Ensure `max_weight` was enough to cover the worst-case weight.
        let proposal_weight = proposal.get_dispatch_info().weight;
        ensure!(
            proposal_weight.all_lte(max_weight),
            Error::<T>::MaxWeightTooLow
        );

        let update_proposal_status = |status| {
            <ProposalDetail<T>>::mutate(&multisig, proposal_id, |proposal_details| {
                proposal_details.status = status
            })
        };
        let (res, actual_weight) = match with_call_metadata(proposal.get_call_metadata(), || {
            proposal.dispatch(frame_system::RawOrigin::Signed(multisig.clone()).into())
        }) {
            Ok(post_info) => {
                update_proposal_status(ProposalStatus::ExecutionSuccessful);
                (true, post_info.actual_weight)
            }
            Err(e) => {
                update_proposal_status(ProposalStatus::ExecutionFailed);
                Self::deposit_event(RawEvent::ProposalFailedToExecute(
                    creator_did,
                    multisig.clone(),
                    proposal_id,
                    e.error,
                ));
                (false, e.post_info.actual_weight)
            }
        };
        Self::deposit_event(RawEvent::ProposalExecuted(
            creator_did,
            multisig,
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
        multisig: T::AccountId,
        signer: T::AccountId,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::ensure_ms_signer(&multisig, &signer)?;

        let mut proposal_details = Self::proposal_detail(&multisig, proposal_id);

        // Only allow the original proposer to change their vote if no one else has voted
        let mut proposal_owner = false;
        if Votes::<T>::get((&multisig, proposal_id), &signer) {
            if proposal_details.rejections != 0 || proposal_details.approvals != 1 {
                return Err(Error::<T>::AlreadyVoted.into());
            }
            proposal_owner = true;
        }

        proposal_details.rejections += 1u64;
        let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
        match proposal_details.status {
            ProposalStatus::Invalid => return Err(Error::<T>::ProposalMissing.into()),
            ProposalStatus::Rejected => return Err(Error::<T>::ProposalAlreadyRejected.into()),
            ProposalStatus::ExecutionSuccessful | ProposalStatus::ExecutionFailed => {
                return Err(Error::<T>::ProposalAlreadyExecuted.into())
            }
            ProposalStatus::ActiveOrExpired => {
                // Ensure proposal is not expired
                if let Some(expiry) = proposal_details.expiry {
                    ensure!(
                        expiry > <pallet_timestamp::Pallet<T>>::get(),
                        Error::<T>::ProposalExpired
                    );
                }
                let approvals_needed = Self::ms_signs_required(multisig.clone());
                let ms_signers = Self::number_of_signers(multisig.clone());
                if proposal_details.rejections > ms_signers.saturating_sub(approvals_needed)
                    || proposal_owner
                {
                    if proposal_owner {
                        proposal_details.approvals = 0;
                    }
                    proposal_details.status = ProposalStatus::Rejected;
                    Self::deposit_event(RawEvent::ProposalRejected(
                        current_did,
                        multisig.clone(),
                        proposal_id,
                    ));
                }
            }
        }
        // Update storage
        <Votes<T>>::insert((&multisig, proposal_id), &signer, true);
        <ProposalDetail<T>>::insert(&multisig, proposal_id, proposal_details);
        // emit proposal rejected event
        Self::deposit_event(RawEvent::ProposalRejectionVote(
            current_did,
            multisig,
            signer,
            proposal_id,
        ));
        Ok(())
    }

    /// Accepts and processed an addition of a signer to a multisig.
    pub fn base_accept_multisig_signer(signer: T::AccountId, auth_id: u64) -> DispatchResult {
        <Identity<T>>::accept_auth_with(
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
                    !<MultiSigSigners<T>>::get(&multisig, &signer),
                    Error::<T>::AlreadyASigner
                );

                let (to_identity, to_multisig) = Identity::<T>::is_key_linked(&signer);
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

                let ms_identity = <CreatorDid<T>>::get(&multisig);
                <Identity<T>>::ensure_auth_by(ms_identity, auth_by)?;

                <MultiSigSigners<T>>::insert(&multisig, &signer, true);
                <NumberOfSigners<T>>::mutate(&multisig, |x| *x += 1u64);

                Identity::<T>::add_key_record(
                    &signer,
                    KeyRecord::MultiSigSignerKey(multisig.clone()),
                );
                Self::deposit_event(RawEvent::MultiSigSignerAdded(
                    ms_identity,
                    multisig,
                    signer.clone(),
                ));
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
        <MultiSigSigners<T>>::get(multi_sig, signer)
    }

    /// Checks whether changing the list of signers is allowed in a multisig.
    pub fn is_changing_signers_allowed(multisig: &T::AccountId) -> bool {
        if <Identity<T>>::cdd_auth_for_primary_key_rotation() {
            if let Some(did) = <Identity<T>>::get_identity(multisig) {
                if <Identity<T>>::is_primary_key(&did, multisig) {
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
            CreatorDid::<T>::try_get(&multisig).map_err(|_| Error::<T>::MultisigMissingIdentity)?;
        ensure!(creator_did == sender_did, Error::<T>::IdentityNotCreator);
        Ok(())
    }

    /// Changes the number of required signatures for the given `multisig_account` to `signatures_required`.
    fn base_change_multisig_required_singatures(
        multisig_account: T::AccountId,
        signatures_required: u64,
    ) -> DispatchResult {
        ensure!(
            <NumberOfSigners<T>>::get(&multisig_account) >= signatures_required,
            Error::<T>::NotEnoughSigners
        );
        ensure!(
            Self::is_changing_signers_allowed(&multisig_account),
            Error::<T>::ChangeNotAllowed
        );
        Self::base_change_sigs_required(multisig_account, signatures_required);
        Ok(())
    }

    /// Removes the creator ability to call `add_multisig_signers_via_creator`, `remove_multisig_signers_via_creator`
    /// and `change_sigs_required_via_creator`.
    fn base_remove_creator_controls(creator_did: IdentityId) {
        LostCreatorPrivileges::insert(creator_did, true);
    }
}

impl<T: Config> MultiSigSubTrait<T::AccountId> for Module<T> {
    fn is_multisig(account_id: &T::AccountId) -> bool {
        <CreatorDid<T>>::contains_key(account_id)
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
