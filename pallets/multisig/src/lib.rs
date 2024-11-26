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
//! - `create_proposal` - Creates a multisig proposal given the signer's account key.
//! - `approve` - Approves a multisig proposal given the signer's account key.
//! - `reject` - Rejects a multisig proposal using the caller's secondary key (`AccountId`).
//! - `accept_multisig_signer` - Accepts a multisig signer authorization given the signer's
//! account key.
//! - `add_multisig_signer` - Adds a signer to the multisig.
//! - `remove_multisig_signer` - Removes a signer from the multisig.
//! - `add_multisig_signers_via_admin` - Adds a signer to the multisig when called by the
//! admin of the multisig.
//! - `remove_multisig_signers_via_admin` - Removes a signer from the multisig when called by the
//! admin of the multisig.
//! - `change_sigs_required` - Changes the number of signers required to execute a transaction.
//!
//! ### Other Public Functions
//!
//! - `base_create_multisig` - Creates a multisig account without precondition checks or emitting
//! an event.
//! - `base_create_proposal` - Creates a proposal for a multisig transaction.
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
use frame_support::ensure;
use frame_support::storage::{IterableStorageDoubleMap, IterableStorageMap};
use frame_support::traits::{Get, GetCallMetadata, IsSubType, UnfilteredDispatchable};
use frame_support::BoundedVec;
use frame_system::ensure_signed;
use sp_runtime::traits::{Dispatchable, Hash};
use sp_std::convert::TryFrom;
use sp_std::prelude::*;

use pallet_identity::PermissionedCallOriginData;
use pallet_permissions::with_call_metadata;
pub use polymesh_common_utilities::multisig::{MultiSigSubTrait, WeightInfo};
use polymesh_common_utilities::traits::identity::Config as IdentityConfig;
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

        /// The overarching call type for proposals.
        type Proposal: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
            + GetCallMetadata
            + GetDispatchInfo
            + From<Call<Self>>
            + From<frame_system::Call<Self>>
            + UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// Weight information for extrinsics in the multisig pallet.
        type WeightInfo: WeightInfo;

        /// Maximum number of signers that can be added/removed in one call.
        #[pallet::constant]
        type MaxSigners: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
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
                let res = ProposalVoteCounts::<T>::clear(u32::max_value(), None);
                removed += res.unique;
                let res = ProposalStates::<T>::clear(u32::max_value(), None);
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
        /// * `permissions` - optional custom permissions.  Only the primary key can provide custom permissions.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_multisig(signers.len() as u32))]
        pub fn create_multisig(
            origin: OriginFor<T>,
            signers: BoundedVec<T::AccountId, T::MaxSigners>,
            sigs_required: u64,
            permissions: Option<Permissions>,
        ) -> DispatchResultWithPostInfo {
            let (caller, caller_did, permissions) = match permissions {
                Some(permissions) => {
                    // Only the primary key can add a secondary key with custom permissions.
                    let (caller, did) = IdentityPallet::<T>::ensure_primary_key(origin)?;
                    (caller, did, permissions)
                }
                None => {
                    // Default to empty permissions for the new secondary key.
                    let PermissionedCallOriginData {
                        sender: caller,
                        primary_did,
                        ..
                    } = IdentityPallet::<T>::ensure_origin_call_permissions(origin)?;
                    (caller, primary_did, Permissions::empty())
                }
            };
            let signers_len: u64 = u64::try_from(signers.len()).unwrap_or_default();
            Self::ensure_sigs_in_bounds(signers_len, sigs_required)?;
            Self::base_create_multisig(caller, caller_did, signers, sigs_required, permissions)?;
            Ok(().into())
        }

        /// Creates a multisig proposal
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal` - Proposal to be voted on.
        /// * `expiry` - Optional proposal expiry time.
        ///
        /// If this is 1 out of `m` multisig, the proposal will be immediately executed.
        #[pallet::call_index(1)]
        #[pallet::weight({
          <T as Config>::WeightInfo::create_proposal()
            .saturating_add(<T as Config>::WeightInfo::execute_proposal())
            .saturating_add(proposal.get_dispatch_info().weight)
        })]
        pub fn create_proposal(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            proposal: Box<<T as Config>::Proposal>,
            expiry: Option<T::Moment>,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            with_base_weight(<T as Config>::WeightInfo::create_proposal(), || {
                Self::base_create_proposal(&multisig, signer, &proposal, expiry)
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
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::approve_and_execute(max_weight))]
        pub fn approve(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            proposal_id: u64,
            max_weight: Option<Weight>,
        ) -> DispatchResultWithPostInfo {
            let max_weight = <T as Config>::WeightInfo::default_max_weight(&max_weight);
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
        #[pallet::call_index(3)]
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
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::accept_multisig_signer())]
        pub fn accept_multisig_signer(
            origin: OriginFor<T>,
            auth_id: u64,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            Self::base_accept_multisig_signer(signer, auth_id)?;
            Ok(().into())
        }

        /// Adds signers to the multisig.  This must be called by the multisig itself.
        ///
        /// # Arguments
        /// * `signers` - Signers to add.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::add_multisig_signers(signers.len() as u32))]
        pub fn add_multisig_signers(
            origin: OriginFor<T>,
            signers: BoundedVec<T::AccountId, T::MaxSigners>,
        ) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin)?;
            Self::base_add_signers(None, multisig, signers)?;
            Ok(().into())
        }

        /// Removes signers from the multisig.  This must be called by the multisig itself.
        ///
        /// # Arguments
        /// * `signers` - Signers to remove.
        #[pallet::weight(<T as Config>::WeightInfo::remove_multisig_signers(signers.len() as u32))]
        #[pallet::call_index(6)]
        pub fn remove_multisig_signers(
            origin: OriginFor<T>,
            signers: BoundedVec<T::AccountId, T::MaxSigners>,
        ) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin)?;
            // Remove the signers from the multisig.
            Self::base_remove_signers(None, multisig, signers)?;
            Ok(().into())
        }

        /// Adds a signer to the multisig.  This must be called by the admin identity of the
        /// multisig.
        ///
        /// # Arguments
        /// * `multisig` - Address of the multi sig
        /// * `signers` - Signers to add.
        ///
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::add_multisig_signers_via_admin(signers.len() as u32))]
        pub fn add_multisig_signers_via_admin(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            signers: BoundedVec<T::AccountId, T::MaxSigners>,
        ) -> DispatchResultWithPostInfo {
            let caller_did = Self::ensure_ms_admin(origin, &multisig)?;
            Self::base_add_signers(Some(caller_did), multisig, signers)?;
            Ok(().into())
        }

        /// Removes a signer from the multisig.
        /// This must be called by the admin identity of the multisig.
        ///
        /// # Arguments
        /// * `multisig` - Address of the multisig.
        /// * `signers` - Signers to remove.
        ///
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_multisig_signers_via_admin(signers.len() as u32))]
        pub fn remove_multisig_signers_via_admin(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            signers: BoundedVec<T::AccountId, T::MaxSigners>,
        ) -> DispatchResultWithPostInfo {
            // Ensure the caller is the admin and that they haven't lost permissions.
            let caller_did = Self::ensure_ms_admin(origin, &multisig)?;

            // Remove the signers from the multisig.
            Self::base_remove_signers(Some(caller_did), multisig, signers)?;

            Ok(().into())
        }

        /// Changes the number of signatures required by a multisig.  This must be called by the
        /// multisig itself.
        ///
        /// # Arguments
        /// * `sigs_required` - New number of required signatures.
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::change_sigs_required())]
        pub fn change_sigs_required(
            origin: OriginFor<T>,
            sigs_required: u64,
        ) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin)?;
            Self::base_change_multisig_required_signatures(None, &multisig, sigs_required)?;
            Ok(().into())
        }

        /// Changes the number of signatures required by a multisig.  This must be called by the admin of the multisig.
        ///
        /// # Arguments
        /// * `multisig` - The account identifier ([`AccountId`]) for the multi signature account.
        /// * `signatures_required` - The number of required signatures.
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::change_sigs_required_via_admin())]
        pub fn change_sigs_required_via_admin(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            signatures_required: u64,
        ) -> DispatchResultWithPostInfo {
            let caller_did = Self::ensure_ms_admin(origin, &multisig)?;
            Self::base_change_multisig_required_signatures(
                Some(caller_did),
                &multisig,
                signatures_required,
            )?;
            Ok(().into())
        }

        /// Add an admin identity to the multisig.  This must be called by the multisig itself.
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::add_admin())]
        pub fn add_admin(
            origin: OriginFor<T>,
            admin_did: IdentityId,
        ) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin)?;
            let caller_did = Self::ensure_ms_has_did(&multisig)?;
            AdminDid::<T>::insert(&multisig, admin_did);
            Self::deposit_event(Event::MultiSigAddedAdmin {
                caller_did,
                multisig,
                admin_did,
            });
            Ok(().into())
        }

        /// Removes the admin identity from the `multisig`.  This must be called by the admin of the multisig.
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_admin_via_admin())]
        pub fn remove_admin_via_admin(
            origin: OriginFor<T>,
            multisig: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let admin_did = Self::ensure_ms_admin(origin, &multisig)?;
            AdminDid::<T>::remove(&multisig);
            Self::deposit_event(Event::MultiSigRemovedAdmin {
                caller_did: admin_did,
                multisig,
                admin_did,
            });
            Ok(().into())
        }

        /// Removes the paying identity from the `multisig`.  This must be called by the multisig itself.
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_payer())]
        pub fn remove_payer(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin)?;
            let caller_did = Self::ensure_ms_has_did(&multisig)?;
            let paying_did = PayingDid::<T>::take(&multisig).ok_or(Error::<T>::NoPayingDid)?;
            Self::deposit_event(Event::MultiSigRemovedPayingDid {
                caller_did,
                multisig,
                paying_did,
            });
            Ok(().into())
        }

        /// Removes the paying identity from the `multisig`.  This must be called by the paying identity of the multisig.
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_payer_via_payer())]
        pub fn remove_payer_via_payer(
            origin: OriginFor<T>,
            multisig: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let paying_did = Self::ensure_ms_payer(origin, &multisig)?;
            PayingDid::<T>::remove(&multisig);
            Self::deposit_event(Event::MultiSigRemovedPayingDid {
                caller_did: paying_did,
                multisig,
                paying_did,
            });
            Ok(().into())
        }

        /// Approves a multisig join identity proposal.
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `auth_id` - The join identity authorization to approve.
        ///
        /// If quorum is reached, the join identity proposal will be immediately executed.
        #[pallet::call_index(15)]
        #[pallet::weight({
          <T as Config>::WeightInfo::create_join_identity()
            .saturating_add(<T as Config>::WeightInfo::approve_join_identity())
            .saturating_add(<T as Config>::WeightInfo::execute_proposal())
            .saturating_add(<T as Config>::WeightInfo::join_identity())
        })]
        pub fn approve_join_identity(
            origin: OriginFor<T>,
            multisig: T::AccountId,
            auth_id: u64,
        ) -> DispatchResultWithPostInfo {
            let signer = ensure_signed(origin)?;
            if let Some(proposal_id) = AuthToProposalId::<T>::get(&multisig, auth_id) {
                let max_weight = <T as Config>::WeightInfo::join_identity();
                with_base_weight(<T as Config>::WeightInfo::approve_join_identity(), || {
                    Self::base_approve(&multisig, signer, proposal_id, max_weight)
                })
            } else {
                let proposal = Call::<T>::join_identity { auth_id }.into();
                let proposal_id = Self::next_proposal_id(&multisig);
                AuthToProposalId::<T>::insert(&multisig, auth_id, proposal_id);
                with_base_weight(<T as Config>::WeightInfo::create_join_identity(), || {
                    Self::base_create_proposal(&multisig, signer, &proposal, None)
                })
            }
        }

        /// Accept a JoinIdentity authorization for this multisig.  This must be called by the multisig itself.
        #[pallet::call_index(16)]
        #[pallet::weight(<T as Config>::WeightInfo::join_identity())]
        pub fn join_identity(origin: OriginFor<T>, auth_id: u64) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin.clone())?;
            Self::ensure_ms(&multisig)?;
            AuthToProposalId::<T>::remove(multisig, auth_id);
            pallet_identity::Module::<T>::join_identity(origin, auth_id)?;
            Ok(().into())
        }

        /// Removes the admin identity from the `multisig`.  This must be called by the multisig itself.
        #[pallet::call_index(17)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_admin())]
        pub fn remove_admin(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let multisig = ensure_signed(origin)?;
            let caller_did = Self::ensure_ms_has_did(&multisig)?;
            let admin_did = AdminDid::<T>::take(&multisig).ok_or(Error::<T>::AdminNotFound)?;
            Self::deposit_event(Event::MultiSigRemovedAdmin {
                caller_did,
                multisig,
                admin_did,
            });
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A Multisig has been created.
        MultiSigCreated {
            caller_did: IdentityId,
            multisig: T::AccountId,
            caller: T::AccountId,
            signers: BoundedVec<T::AccountId, T::MaxSigners>,
            sigs_required: u64,
        },
        /// A Multisig proposal has been created.
        ProposalAdded {
            caller_did: Option<IdentityId>,
            multisig: T::AccountId,
            proposal_id: u64,
        },
        /// A Multisig proposal has been executed.
        ProposalExecuted {
            caller_did: Option<IdentityId>,
            multisig: T::AccountId,
            proposal_id: u64,
            result: DispatchResult,
        },
        /// A new signer has been added to a Multisig.
        MultiSigSignerAdded {
            caller_did: IdentityId,
            multisig: T::AccountId,
            signer: T::AccountId,
        },
        /// New keys have been authorized to be signers on a Multisig.
        MultiSigSignersAuthorized {
            caller_did: IdentityId,
            multisig: T::AccountId,
            signers: BoundedVec<T::AccountId, T::MaxSigners>,
        },
        /// Signers have been removed from a Multisig.
        MultiSigSignersRemoved {
            caller_did: IdentityId,
            multisig: T::AccountId,
            signers: BoundedVec<T::AccountId, T::MaxSigners>,
        },
        /// A Multisig has changed its required number of approvals.
        MultiSigSignersRequiredChanged {
            caller_did: Option<IdentityId>,
            multisig: T::AccountId,
            sigs_required: u64,
        },
        /// A signer has voted to approve a Multisig proposal.
        ProposalApprovalVote {
            caller_did: Option<IdentityId>,
            multisig: T::AccountId,
            signer: T::AccountId,
            proposal_id: u64,
        },
        /// A signer has voted to reject a Multisig proposal.
        ProposalRejectionVote {
            caller_did: Option<IdentityId>,
            multisig: T::AccountId,
            signer: T::AccountId,
            proposal_id: u64,
        },
        /// A Multisig proposal has been approved.
        ProposalApproved {
            caller_did: Option<IdentityId>,
            multisig: T::AccountId,
            proposal_id: u64,
        },
        /// A Multisig proposal has been rejected.
        ProposalRejected {
            caller_did: Option<IdentityId>,
            multisig: T::AccountId,
            proposal_id: u64,
        },
        /// A Multisig has added an admin DID.
        MultiSigAddedAdmin {
            caller_did: IdentityId,
            multisig: T::AccountId,
            admin_did: IdentityId,
        },
        /// A Multisig has removed it's admin DID.
        MultiSigRemovedAdmin {
            caller_did: IdentityId,
            multisig: T::AccountId,
            admin_did: IdentityId,
        },
        /// A Multisig has removed it's paying DID.
        MultiSigRemovedPayingDid {
            caller_did: IdentityId,
            multisig: T::AccountId,
            paying_did: IdentityId,
        },
    }

    /// Multisig module errors.
    #[pallet::error]
    pub enum Error<T> {
        /// The proposal does not exist.
        ProposalMissing,
        /// Multisig address.
        DecodingError,
        /// Required number of signers must be greater then zero.
        RequiredSignersIsZero,
        /// Not a signer.
        NotASigner,
        /// No such multisig.
        NoSuchMultisig,
        /// Not enough signers.  The number of signers has to be greater then or equal to
        /// the required number of signers to approve proposals.
        NotEnoughSigners,
        /// A nonce overflow.
        NonceOverflow,
        /// Already voted.
        AlreadyVoted,
        /// Already a signer.
        AlreadyASigner,
        /// Identity provided is not the multisig's admin.
        IdentityNotAdmin,
        /// Identity provided is not the multisig's payer.
        IdentityNotPayer,
        /// Changing multisig parameters not allowed since multisig is a primary key.
        ChangeNotAllowed,
        /// Signer is an account key that is already associated with a multisig.
        SignerAlreadyLinkedToMultisig,
        /// Signer is an account key that is already associated with an identity.
        SignerAlreadyLinkedToIdentity,
        /// A multisig can't be a signer of another multisig.
        NestingNotAllowed,
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
        /// Tried to add/remove too many signers.
        TooManySigners,
        /// Multisig doesn't have a paying DID.
        NoPayingDid,
        /// Expiry must be in the future.
        InvalidExpiryDate,
        /// The proposal has been invalidated after a multisg update.
        InvalidatedProposal,
        /// Multisig has no admin.
        AdminNotFound,
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

    /// Next proposal id for a multisig.  Starts from 0.
    ///
    /// multisig => next proposal id
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T: Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

    /// Proposals presented for voting to a multisig.
    ///
    /// multisig -> proposal id => Option<Proposal>.
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, <T as Config>::Proposal>;

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

    /// The multisig's paying identity.  The primary key of this identity
    /// pays the transaction/protocal fees of the multisig proposals.
    ///
    /// multisig -> Option<IdentityId>.
    #[pallet::storage]
    pub type PayingDid<T: Config> = StorageMap<_, Identity, T::AccountId, IdentityId>;

    /// The multisig's admin identity.  The primary key of this identity
    /// has admin control over the multisig.
    ///
    /// multisig -> Option<IdentityId>.
    #[pallet::storage]
    pub type AdminDid<T: Config> = StorageMap<_, Identity, T::AccountId, IdentityId>;

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

    /// Proposal execution reentry guard.
    #[pallet::storage]
    #[pallet::getter(fn execution_reentry)]
    pub(super) type ExecutionReentry<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Pending join identity authorization proposals.
    ///
    /// multisig -> auth id => Option<proposal id>.
    #[pallet::storage]
    pub type AuthToProposalId<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, u64>;

    /// The last transaction version, used for `on_runtime_upgrade`.
    #[pallet::storage]
    #[pallet::getter(fn transaction_version)]
    pub(super) type TransactionVersion<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// The last proposal id before the multisig changed signers or signatures required.
    ///
    /// multisig => Option<proposal id>
    #[pallet::storage]
    #[pallet::getter(fn last_invalid_proposal)]
    pub type LastInvalidProposal<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, OptionQuery>;

    /// Storage version.
    #[pallet::storage]
    #[pallet::getter(fn storage_version)]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {}

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
    pub fn get_paying_did(multisig: &T::AccountId) -> Option<IdentityId> {
        PayingDid::<T>::get(multisig)
    }

    pub fn ensure_ms_get_did(multisig: &T::AccountId) -> Result<Option<IdentityId>, DispatchError> {
        Self::ensure_ms(multisig)?;
        Ok(IdentityPallet::<T>::get_identity(multisig))
    }

    pub fn ensure_ms_has_did(multisig: &T::AccountId) -> Result<IdentityId, DispatchError> {
        Self::ensure_ms_get_did(multisig)?.ok_or(Error::<T>::MultisigMissingIdentity.into())
    }

    fn ensure_max_signers(multisig: &T::AccountId, len: u64) -> Result<u64, DispatchError> {
        let pending_num_of_signers = NumberOfSigners::<T>::get(&multisig)
            .checked_add(len)
            .ok_or(Error::<T>::TooManySigners)?;
        ensure!(
            pending_num_of_signers <= T::MaxSigners::get() as u64,
            Error::<T>::TooManySigners
        );
        Ok(pending_num_of_signers)
    }

    fn ensure_ms_admin(
        origin: T::RuntimeOrigin,
        multisig: &T::AccountId,
    ) -> Result<IdentityId, DispatchError> {
        let (_, caller_did) = IdentityPallet::<T>::ensure_primary_key(origin)?;
        let admin_did = AdminDid::<T>::get(multisig);
        ensure!(admin_did == Some(caller_did), Error::<T>::IdentityNotAdmin);
        Ok(caller_did)
    }

    fn ensure_ms_payer(
        origin: T::RuntimeOrigin,
        multisig: &T::AccountId,
    ) -> Result<IdentityId, DispatchError> {
        let (_, caller_did) = IdentityPallet::<T>::ensure_primary_key(origin)?;
        let payer_did = PayingDid::<T>::get(multisig);
        ensure!(payer_did == Some(caller_did), Error::<T>::IdentityNotPayer);
        Ok(caller_did)
    }

    fn ensure_ms(multisig: &T::AccountId) -> DispatchResult {
        ensure!(
            MultiSigSignsRequired::<T>::contains_key(multisig),
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

    fn ensure_sigs_in_bounds(num_signers: u64, required: u64) -> DispatchResult {
        ensure!(required > 0, Error::<T>::RequiredSignersIsZero);
        ensure!(num_signers >= required, Error::<T>::NotEnoughSigners);
        Ok(())
    }

    fn ensure_proposal_is_active(multisig: &T::AccountId, proposal_id: u64) -> DispatchResult {
        match ProposalStates::<T>::get(multisig, proposal_id) {
            None => Err(Error::<T>::ProposalMissing.into()),
            Some(ProposalState::Rejected) => Err(Error::<T>::ProposalAlreadyRejected.into()),
            Some(ProposalState::ExecutionSuccessful | ProposalState::ExecutionFailed) => {
                Err(Error::<T>::ProposalAlreadyExecuted.into())
            }
            Some(ProposalState::Active { until: None }) => {
                Self::ensure_valid_proposal(multisig, proposal_id)?;
                Ok(())
            }
            Some(ProposalState::Active { until: Some(until) }) => {
                Self::ensure_valid_proposal(multisig, proposal_id)?;
                // Ensure proposal is not expired
                ensure!(
                    until > pallet_timestamp::Pallet::<T>::get(),
                    Error::<T>::ProposalExpired
                );
                Ok(())
            }
        }
    }

    fn base_authorize_signers(
        caller_did: IdentityId,
        multisig: &T::AccountId,
        signers: &BoundedVec<T::AccountId, T::MaxSigners>,
    ) -> DispatchResult {
        for signer in signers {
            IdentityPallet::<T>::add_auth(
                caller_did,
                Signatory::Account(signer.clone()),
                AuthorizationData::AddMultiSigSigner(multisig.clone()),
                None,
            )?;
        }
        Ok(())
    }

    fn base_add_signers(
        caller_did: Option<IdentityId>,
        multisig: T::AccountId,
        signers: BoundedVec<T::AccountId, T::MaxSigners>,
    ) -> DispatchResult {
        // Ensure `multisig` is a MultiSig and get it's DID.
        let ms_did = Self::ensure_ms_has_did(&multisig)?;
        // Don't allow adding too many signers.
        Self::ensure_max_signers(&multisig, signers.len() as u64)?;

        Self::base_authorize_signers(ms_did, &multisig, &signers)?;
        Self::deposit_event(Event::MultiSigSignersAuthorized {
            caller_did: caller_did.unwrap_or(ms_did),
            multisig,
            signers,
        });
        Ok(())
    }

    fn base_remove_signers(
        caller_did: Option<IdentityId>,
        multisig: T::AccountId,
        signers: BoundedVec<T::AccountId, T::MaxSigners>,
    ) -> DispatchResult {
        // Ensure `multisig` is a MultiSig and get it's DID.
        let ms_did = Self::ensure_ms_has_did(&multisig)?;
        ensure!(
            Self::is_changing_signers_allowed(&multisig),
            Error::<T>::ChangeNotAllowed
        );
        let signers_len: u64 = u64::try_from(signers.len()).unwrap_or_default();

        let pending_num_of_signers = NumberOfSigners::<T>::get(&multisig)
            .checked_sub(signers_len)
            .ok_or(Error::<T>::TooManySigners)?;
        let sigs_required = MultiSigSignsRequired::<T>::get(&multisig);
        Self::ensure_sigs_in_bounds(pending_num_of_signers, sigs_required)?;

        for signer in &signers {
            Self::ensure_ms_signer(&multisig, signer)?;
            IdentityPallet::<T>::remove_key_record(signer, None);
            MultiSigSigners::<T>::remove(&multisig, signer);
        }

        NumberOfSigners::<T>::insert(&multisig, pending_num_of_signers);
        Self::set_invalid_proposals(&multisig);
        Self::deposit_event(Event::MultiSigSignersRemoved {
            caller_did: caller_did.unwrap_or(ms_did),
            multisig: multisig.clone(),
            signers,
        });
        Ok(())
    }

    // Creates a multisig without precondition checks or emitting an event.
    fn base_create_multisig(
        caller: T::AccountId,
        caller_did: IdentityId,
        signers: BoundedVec<T::AccountId, T::MaxSigners>,
        sigs_required: u64,
        permissions: Permissions,
    ) -> DispatchResult {
        // Generate new MultiSig address.
        let new_nonce = Self::ms_nonce()
            .checked_add(1)
            .ok_or(Error::<T>::NonceOverflow)?;
        MultiSigNonce::<T>::put(new_nonce);
        let multisig = Self::get_multisig_address(&caller, new_nonce)?;

        // Ensure the `multisig` is unlinked.
        IdentityPallet::<T>::ensure_key_did_unlinked(&multisig)?;

        Self::base_authorize_signers(caller_did, &multisig, &signers)?;
        MultiSigSignsRequired::<T>::insert(&multisig, &sigs_required);
        PayingDid::<T>::insert(&multisig, caller_did);

        Self::deposit_event(Event::MultiSigCreated {
            caller_did,
            multisig: multisig.clone(),
            caller,
            signers,
            sigs_required,
        });

        // Add the multisig as a secondary key.
        IdentityPallet::<T>::unsafe_join_identity(caller_did, permissions, multisig);

        Ok(())
    }

    // Creates a new proposal.
    fn base_create_proposal(
        multisig: &T::AccountId,
        signer: T::AccountId,
        proposal: &<T as Config>::Proposal,
        expiry: Option<T::Moment>,
    ) -> DispatchResultWithPostInfo {
        Self::ensure_ms_signer(multisig, &signer)?;
        let max_weight = proposal.get_dispatch_info().weight;
        let caller_did = Self::ensure_ms_get_did(multisig)?;
        let proposal_id = Self::next_proposal_id(multisig);
        Self::ensure_valid_expiry(&expiry)?;

        Proposals::<T>::insert(multisig, proposal_id, &*proposal);
        ProposalVoteCounts::<T>::insert(multisig, proposal_id, ProposalVoteCount::default());
        ProposalStates::<T>::insert(multisig, proposal_id, ProposalState::new(expiry));

        // Since proposal_ids are always only incremented by 1, they can not overflow.
        let next_proposal_id: u64 = proposal_id + 1u64;
        NextProposalId::<T>::insert(multisig, next_proposal_id);
        Self::deposit_event(Event::ProposalAdded {
            caller_did,
            multisig: multisig.clone(),
            proposal_id,
        });
        Self::base_approve(multisig, signer, proposal_id, max_weight)
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
        let caller_did = Self::ensure_ms_get_did(multisig)?;
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
        let execute_proposal = vote_count.approvals >= Self::ms_signs_required(multisig);

        // Update storage
        Votes::<T>::insert((multisig, proposal_id), &signer, true);
        ProposalVoteCounts::<T>::insert(multisig, proposal_id, vote_count);
        // emit proposal approval vote event.
        Self::deposit_event(Event::ProposalApprovalVote {
            caller_did,
            multisig: multisig.clone(),
            signer,
            proposal_id,
        });
        if execute_proposal {
            Self::execute_proposal(multisig, proposal_id, caller_did, max_weight)
        } else {
            Ok(().into())
        }
    }

    // Executes a proposal if it has enough approvals
    fn execute_proposal(
        multisig: &T::AccountId,
        proposal_id: u64,
        caller_did: Option<IdentityId>,
        max_weight: Weight,
    ) -> DispatchResultWithPostInfo {
        // emit proposal approved event
        Self::deposit_event(Event::ProposalApproved {
            caller_did,
            multisig: multisig.clone(),
            proposal_id,
        });
        // Take the proposal.
        let proposal = Proposals::<T>::take(multisig, proposal_id)
            .ok_or_else(|| Error::<T>::ProposalMissing)?;

        // Ensure `max_weight` was enough to cover the worst-case weight.
        let proposal_weight = proposal.get_dispatch_info().weight;
        ensure!(
            proposal_weight.all_lte(max_weight),
            Error::<T>::MaxWeightTooLow
        );

        let (result, actual_weight) = match with_call_metadata(proposal.get_call_metadata(), || {
            // Check execution reentry guard.
            ensure!(!Self::execution_reentry(), Error::<T>::NestingNotAllowed,);

            // Enable reentry guard before executing the proposal.
            ExecutionReentry::<T>::set(true);
            // Execute proposal.
            let res = proposal.dispatch(frame_system::RawOrigin::Signed(multisig.clone()).into());
            // Make sure to reset the reentry guard, even if the proposal throws an error.
            ExecutionReentry::<T>::set(false);
            res
        }) {
            Ok(post_info) => {
                ProposalStates::<T>::insert(
                    multisig,
                    proposal_id,
                    ProposalState::ExecutionSuccessful,
                );
                (Ok(()), post_info.actual_weight)
            }
            Err(e) => {
                ProposalStates::<T>::insert(multisig, proposal_id, ProposalState::ExecutionFailed);
                (Err(e.error), e.post_info.actual_weight)
            }
        };

        Self::deposit_event(Event::ProposalExecuted {
            caller_did,
            multisig: multisig.clone(),
            proposal_id,
            result,
        });
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
        let caller_did = Self::ensure_ms_get_did(multisig)?;

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

        // emit proposal reject vote event.
        Self::deposit_event(Event::ProposalRejectionVote {
            caller_did,
            multisig: multisig.clone(),
            signer: signer.clone(),
            proposal_id,
        });
        // Record the signer's vote.
        Votes::<T>::insert((multisig, proposal_id), &signer, true);
        vote_count.rejections += 1u64;

        let approvals_needed = Self::ms_signs_required(&multisig);
        let ms_signers = Self::number_of_signers(&multisig);
        if vote_count.rejections > ms_signers.saturating_sub(approvals_needed) || proposal_owner {
            if proposal_owner {
                vote_count.approvals = 0;
            }
            // Remove the proposal from storage.
            Proposals::<T>::remove(multisig, proposal_id);
            ProposalStates::<T>::insert(multisig, proposal_id, ProposalState::Rejected);
            Self::deposit_event(Event::ProposalRejected {
                caller_did,
                multisig: multisig.clone(),
                proposal_id,
            });
        }
        // Update vote counts.
        ProposalVoteCounts::<T>::insert(multisig, proposal_id, vote_count);
        Ok(())
    }

    // Accepts and processed an addition of a signer to a multisig.
    fn base_accept_multisig_signer(signer: T::AccountId, auth_id: u64) -> DispatchResult {
        IdentityPallet::<T>::accept_auth_with(
            &Signatory::Account(signer.clone()),
            auth_id,
            |data, auth_by| {
                let multisig = extract_auth!(data, AddMultiSigSigner(ms));

                // Don't allow a multisig to be a signer of another multisig.
                // Nesting is not allowed.
                ensure!(!Self::is_multisig(&signer), Error::<T>::NestingNotAllowed);

                // Ensure the multisig has a DID and get it.
                let ms_identity = Self::ensure_ms_has_did(&multisig)?;

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

                IdentityPallet::<T>::ensure_auth_by(ms_identity, auth_by)?;

                Self::set_invalid_proposals(&multisig);

                // Update number of signers for this multisig.
                let pending_num_of_signers = Self::ensure_max_signers(&multisig, 1)?;
                NumberOfSigners::<T>::insert(&multisig, pending_num_of_signers);

                // Add and link the signer to the multisig.
                MultiSigSigners::<T>::insert(&multisig, &signer, true);
                IdentityPallet::<T>::add_key_record(
                    &signer,
                    KeyRecord::MultiSigSignerKey(multisig.clone()),
                );

                Self::deposit_event(Event::MultiSigSignerAdded {
                    caller_did: ms_identity,
                    multisig,
                    signer,
                });
                Ok(())
            },
        )
    }

    /// Gets the next available multisig account ID.
    pub fn get_next_multisig_address(caller: T::AccountId) -> Result<T::AccountId, DispatchError> {
        // Nonce is always only incremented by small numbers and hence can never overflow 64 bits.
        // Also, this is just a helper function that does not modify state.
        let new_nonce = Self::ms_nonce() + 1;
        Self::get_multisig_address(&caller, new_nonce)
    }

    /// Constructs a multisig account given a nonce.
    pub fn get_multisig_address(
        caller: &T::AccountId,
        nonce: u64,
    ) -> Result<T::AccountId, DispatchError> {
        let h: T::Hash = T::Hashing::hash(&(b"MULTI_SIG", nonce, caller).encode());
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

    // Changes the number of required signatures for the given `multisig` to `signatures_required`.
    fn base_change_multisig_required_signatures(
        caller_did: Option<IdentityId>,
        multisig: &T::AccountId,
        signatures_required: u64,
    ) -> DispatchResult {
        let ms_did = Self::ensure_ms_get_did(&multisig)?;
        let num_signers = NumberOfSigners::<T>::get(multisig);
        Self::ensure_sigs_in_bounds(num_signers, signatures_required)?;
        ensure!(
            Self::is_changing_signers_allowed(multisig),
            Error::<T>::ChangeNotAllowed
        );
        MultiSigSignsRequired::<T>::insert(multisig, &signatures_required);
        Self::set_invalid_proposals(&multisig);
        Self::deposit_event(Event::MultiSigSignersRequiredChanged {
            caller_did: caller_did.or(ms_did),
            multisig: multisig.clone(),
            sigs_required: signatures_required,
        });
        Ok(())
    }

    /// Returns `Ok` if `expiry` is in the future. Otherwise, returns [`Error::InvalidExpiryDate`].
    fn ensure_valid_expiry(expiry: &Option<T::Moment>) -> DispatchResult {
        if let Some(expiry) = expiry {
            ensure!(
                expiry > &pallet_timestamp::Pallet::<T>::get(),
                Error::<T>::InvalidExpiryDate
            );
        }
        Ok(())
    }

    /// Returns `Ok` if `proposal_id` is valid. Otherwise, returns [`Error::InvalidatedProposal`].
    fn ensure_valid_proposal(multisig: &T::AccountId, proposal_id: u64) -> DispatchResult {
        if let Some(last_invalid_proposal) = Self::last_invalid_proposal(multisig) {
            ensure!(
                proposal_id > last_invalid_proposal,
                Error::<T>::InvalidatedProposal
            );
        }
        Ok(())
    }

    /// Sets [`LastInvalidProposal`] with the proposal id of the last proposal.
    fn set_invalid_proposals(multisig: &T::AccountId) {
        let next_proposal_id = Self::next_proposal_id(multisig);

        // There are no proposals for the multisig
        if next_proposal_id == 0 {
            return;
        }

        LastInvalidProposal::<T>::insert(multisig, next_proposal_id.saturating_sub(1));
    }
}

impl<T: Config> MultiSigSubTrait<T::AccountId> for Pallet<T> {
    fn is_multisig(account_id: &T::AccountId) -> bool {
        MultiSigSignsRequired::<T>::contains_key(account_id)
    }
}

pub mod migration {
    use super::*;
    use frame_support::storage::migration::move_prefix;
    use frame_support::storage::StorageMap;
    use frame_support::storage::StoragePrefixedMap;
    use sp_runtime::runtime_logger::RuntimeLogger;

    mod v2 {
        use super::*;
        use frame_support::{decl_module, decl_storage};

        decl_storage! {
            trait Store for Module<T: Config> as MultiSig {
                pub MultiSigToIdentity : map hasher(identity) T::AccountId => IdentityId;
                pub MultiSigTxDone: map hasher(identity) T::AccountId => u64;

                pub OldMultiSigSigners: double_map hasher(identity) T::AccountId, hasher(twox_64_concat) Signatory<T::AccountId> => bool;

                pub LostCreatorPrivileges: map hasher(identity) IdentityId => bool;
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
        polymesh_primitives::migrate::kill_item(b"MultiSig", b"ProposalIds");

        migrate_signers::<T>(weight);
        migrate_creator_did::<T>(weight);
        migrate_tx_done::<T>(weight);
    }

    fn migrate_signers<T: Config>(weight: &mut Weight) {
        log::info!(" >>> Migrate Signatory values to only AccountId");
        let mut sig_count = 0;
        let mut reads = 0;
        let mut writes = 0;
        move_prefix(
            &MultiSigSigners::<T>::final_prefix(),
            &v2::OldMultiSigSigners::<T>::final_prefix(),
        );
        v2::OldMultiSigSigners::<T>::drain().for_each(|(ms, signer, value)| {
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
        log::info!(" >>> {sig_count} Multisig.Signers migrated.");
    }

    fn migrate_tx_done<T: Config>(weight: &mut Weight) {
        log::info!(" >>> Migrate MultiSigTxDone to NextProposalId");
        let mut count = 0;
        let mut reads = 0;
        let mut writes = 0;
        v2::MultiSigTxDone::<T>::drain().for_each(|(ms, next_id)| {
            reads += 1;
            count += 1;
            NextProposalId::<T>::insert(ms, next_id);
            writes += 1;
        });
        weight.saturating_accrue(DbWeight::get().reads_writes(reads, writes));
        log::info!(" >>> {count} Multisig.NextProposalId migrated.");
    }

    fn migrate_creator_did<T: Config>(weight: &mut Weight) {
        log::info!(" >>> Migrate MultiSigToIdentity to PayingDid and AdminDid");
        let mut did_count = 0;
        let mut reads = 0;
        let mut writes = 0;
        v2::MultiSigToIdentity::<T>::drain().for_each(|(ms, did)| {
            reads += 1;
            did_count += 1;
            PayingDid::<T>::insert(&ms, did);
            writes += 1;
            let lost_creator_privileges = v2::LostCreatorPrivileges::take(did);
            if !lost_creator_privileges {
                AdminDid::<T>::insert(&ms, did);
                writes += 1;
            }
        });
        weight.saturating_accrue(DbWeight::get().reads_writes(reads, writes));
        log::info!(" >>> {did_count} Multisig.Creator Dids migrated.");
    }
}
