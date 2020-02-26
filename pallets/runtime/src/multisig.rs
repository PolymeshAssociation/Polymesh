//! # MultiSig Module
//!
//! The MultiSig module provides functionality for n of m multisigs.
//!
//! ## Overview
//!
//! The multisig module provides functions for:
//!
//! - Creating a new multisig
//! - Proposing a multisig transaction
//! - Approving a multisig transaction
//! - Adding new signers to the multisig
//! - Removing existing signers from multisig
//!
//! ### Terminology
//!
//! - **MultiSig:** It is a special type of account that can do tranaction only if at least n of its m signers approve.
//! - **Proposal:** It is a general transaction that the multisig can do,
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `create_multisig` - Creates a new multisig.
//! - `create_proposal` - Creates a proposal for a multisig transaction.
//! - `approve_as_identity` - Approves a proposal as an Identity.
//! - `approve_as_key` - Approves a proposal as a Signing key.
//! - `accept_multisig_signer_as_identity` - Accept being added as a signer of a multisig.
//! - `accept_multisig_signer_as_key` - Accept being added as a signer of a multisig.
//! - `add_multisig_signer` - Adds a signer to the multisig.
//! - `remove_multisig_signer` - Removes a signer from the multisig.
//! - `change_sigs_required` - Changes the number of signatures required to execute a transaction.

#![cfg_attr(not(feature = "std"), no_std)]

use polymesh_primitives::{AccountKey, AuthorizationData, AuthorizationError, Signatory};
use polymesh_runtime_common::{
    identity::Trait as IdentityTrait, multisig::AddSignerMultiSig, Context,
};
use polymesh_runtime_identity as identity;

use codec::{Decode, Encode, Error as CodecError};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    weights::{GetDispatchInfo, Weight},
    StorageValue,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{Dispatchable, Hash};
use sp_std::{convert::TryFrom, prelude::*};

/// Either the ID of a successfully created multisig account or an error.
pub type CreateMultisigAccountResult<T> =
    sp_std::result::Result<<T as frame_system::Trait>::AccountId, DispatchError>;
/// Either the ID of a successfully created proposal or an error.
pub type CreateProposalResult = sp_std::result::Result<u64, DispatchError>;

pub trait Trait: frame_system::Trait + IdentityTrait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as MultiSig {
        /// Nonce to ensure unique MultiSig addresses are generated. starts from 1.
        pub MultiSigNonce get(ms_nonce) build(|_| 1u64): u64;
        /// Signers of a multisig. (mulisig, signer) => signer.
        pub MultiSigSigners: double_map hasher(blake2_256) T::AccountId, blake2_256(Signatory) => Signatory;
        /// Number of approved/accepted signers of a multisig.
        pub NumberOfSigners get(number_of_signers): map T::AccountId => u64;
        /// Confirmations required before processing a multisig tx
        pub MultiSigSignsRequired get(ms_signs_required): map T::AccountId => u64;
        /// Number of transactions proposed in a multisig. Used as tx id. starts from 0
        pub MultiSigTxDone get(ms_tx_done): map T::AccountId => u64;
        /// Proposals presented for voting to a multisig (multisig, proposal id) => Option<proposal>.
        pub Proposals get(proposals): map (T::AccountId, u64) => Option<T::Proposal>;
        /// A mapping of proposals to their IDs.
        pub ProposalIds get(proposal_ids):
            double_map hasher(blake2_256) T::AccountId, blake2_256(T::Proposal) => Option<u64>;
        /// Number of votes in favor of a tx. Mapping from (multisig, tx id) => no. of approvals.
        pub TxApprovals get(tx_approvals): map (T::AccountId, u64) => u64;
        /// Individual multisig signer votes. (multi sig, signer, )
        pub Votes get(votes): map (T::AccountId, Signatory, u64) => bool;
    }
}
use core::convert::From;

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Creates a multisig
        ///
        /// # Arguments
        /// * `signers` - Signers of the multisig (They need to accept authorization before they are actually added).
        /// * `sigs_required` - Number of sigs required to process a multi-sig tx.
        pub fn create_multisig(origin, signers: Vec<Signatory>, sigs_required: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(signers.len() > 0, Error::<T>::NoSigners);
            ensure!(u64::try_from(signers.len()).unwrap_or_default() >= sigs_required && sigs_required > 0,
                Error::<T>::RequiredSignaturesOutOfBounds
            );
            let account_id = Self::create_multisig_account(
                sender.clone(),
                signers.as_slice(),
                sigs_required
            )?;
            Self::deposit_event(RawEvent::MultiSigCreated(account_id, sender, signers, sigs_required));
            Ok(())
        }

        /// Creates a multisig proposal if it hasn't been created or approves it if it has.
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal` - Proposal to be voted on.
        /// If this is 1 of m multisig, the proposal will be immediately executed.
        pub fn create_or_approve_proposal_as_identity(
            origin,
            multisig: T::AccountId,
            proposal: Box<T::Proposal>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let signer_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;
            let sender_signer = Signatory::from(signer_did);
            Self::create_or_approve_proposal(multisig, sender_signer, proposal)
        }

        /// Creates a multisig proposal if it hasn't been created or approves it if it has.
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal` - Proposal to be voted on.
        /// If this is 1 of m multisig, the proposal will be immediately executed.
        pub fn create_or_approve_proposal_as_key(
            origin,
            multisig: T::AccountId,
            proposal: Box<T::Proposal>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_signer = Signatory::from(AccountKey::try_from(sender.encode())?);
            Self::create_or_approve_proposal(multisig, sender_signer, proposal)
        }

        /// Creates a multisig proposal
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal` - Proposal to be voted on.
        /// If this is 1 of m multisig, the proposal will be immediately executed.
        pub fn create_proposal_as_identity(origin, multisig: T::AccountId, proposal: Box<T::Proposal>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let signer_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;

            let sender_signer = Signatory::from(signer_did);
            Self::create_proposal(multisig, sender_signer, proposal)?;
            Ok(())
        }

        /// Creates a multisig proposal
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal` - Proposal to be voted on.
        /// If this is 1 of m multisig, the proposal will be immediately executed.
        pub fn create_proposal_as_key(origin, multisig: T::AccountId, proposal: Box<T::Proposal>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_signer = Signatory::from(AccountKey::try_from(sender.encode())?);
            Self::create_proposal(multisig, sender_signer, proposal)?;
            Ok(())
        }

        /// Approves a multisig proposal using caller's identity
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal_id` - Proposal id to approve.
        /// If quorum is reached, the proposal will be immediately executed.
        pub fn approve_as_identity(origin, multisig: T::AccountId, proposal_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let signer_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;
            let signer = Signatory::from(signer_did);
            Self::approve_for(multisig, signer, proposal_id)
        }

        /// Approves a multisig proposal using caller's signing key (AccountId)
        ///
        /// # Arguments
        /// * `multisig` - MultiSig address.
        /// * `proposal_id` - Proposal id to approve.
        /// If quorum is reached, the proposal will be immediately executed.
        pub fn approve_as_key(origin, multisig: T::AccountId, proposal_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let signer = Signatory::from(AccountKey::try_from(sender.encode())?);
            Self::approve_for(multisig, signer, proposal_id)
        }

        /// Accept a multisig signer authorization given to signer's identity
        ///
        /// # Arguments
        /// * `proposal_id` - Auth id of the authorization.
        pub fn accept_multisig_signer_as_identity(origin, auth_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let signer_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;

            let signer = Signatory::from(signer_did);
            Self::_accept_multisig_signer(signer, auth_id)
        }

        /// Accept a multisig signer authorization given to signer's key (AccountId)
        ///
        /// # Arguments
        /// * `proposal_id` - Auth id of the authorization.
        pub fn accept_multisig_signer_as_key(origin, auth_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let signer = Signatory::from(AccountKey::try_from(sender.encode())?);
            Self::_accept_multisig_signer(signer, auth_id)
        }

        /// Add a signer to the multisig. This must be called by the multisig itself.
        ///
        /// # Arguments
        /// * `signer` - Signatory to add.
        pub fn add_multisig_signer(origin, signer: Signatory) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_signer = Signatory::from(AccountKey::try_from(sender.encode())?);
            ensure!(<MultiSigSignsRequired<T>>::exists(&sender), Error::<T>::NoSuchMultisig);
            <identity::Module<T>>::add_auth(
                sender_signer,
                signer,
                AuthorizationData::AddMultiSigSigner,
                None
            );
            Self::deposit_event(RawEvent::MultiSigSignerAuthorized(sender, signer));
            Ok(())
        }

        /// Remove a signer from the multisig. This must be called by the multisig itself.
        ///
        /// # Arguments
        /// * `signer` - Signatory to remove.
        pub fn remove_multisig_signer(origin, signer: Signatory) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<MultiSigSignsRequired<T>>::exists(&sender), Error::<T>::NoSuchMultisig);
            ensure!(<MultiSigSigners<T>>::exists(&sender, &signer), Error::<T>::NotASigner);
            ensure!(
                <NumberOfSigners<T>>::get(&sender) > <MultiSigSignsRequired<T>>::get(&sender),
                Error::<T>::NotEnoughSigners
            );
            <NumberOfSigners<T>>::mutate(&sender, |x| *x = *x - 1u64);
            <MultiSigSigners<T>>::remove(&sender, signer);
            Self::deposit_event(RawEvent::MultiSigSignerRemoved(sender, signer));
            Ok(())
        }

        /// Change number of sigs required by a multisig. This must be called by the multisig itself.
        ///
        /// # Arguments
        /// * `sigs_required` - New number of sigs required.
        pub fn change_sigs_required(origin, sigs_required: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<MultiSigSignsRequired<T>>::exists(&sender), Error::<T>::NoSuchMultisig);
            ensure!(
                <NumberOfSigners<T>>::get(&sender) >= sigs_required,
                Error::<T>::NotEnoughSigners
            );
            <MultiSigSignsRequired<T>>::insert(&sender, &sigs_required);
            Self::deposit_event(RawEvent::MultiSigSignaturesRequiredChanged(sender, sigs_required));
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// Event for multi sig creation. (MultiSig address, Creator address, Signers(pending approval), Sigs required)
        MultiSigCreated(AccountId, AccountId, Vec<Signatory>, u64),
        /// Event for adding a proposal (MultiSig, proposalid)
        ProposalAdded(AccountId, u64),
        /// Emitted when a proposal is executed. (MultiSig, proposalid, result)
        ProposalExecuted(AccountId, u64, bool),
        /// Signatory added (Authorization accepted) (MultiSig, signer_added)
        MultiSigSignerAdded(AccountId, Signatory),
        /// Multi Sig Signatory Authorized to be added (MultiSig, signer_authorized)
        MultiSigSignerAuthorized(AccountId, Signatory),
        /// Multi Sig Signatory removed (MultiSig, signer_removed)
        MultiSigSignerRemoved(AccountId, Signatory),
        /// Change in signatures required by a multisig (MultiSig, new_sigs_required)
        MultiSigSignaturesRequiredChanged(AccountId, u64),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The multisig is not attached to an identity
        IdentityMissing,
        /// The proposal does not exist
        ProposalMissing,
        /// MultiSig address
        DecodingError,
        /// No signers.
        NoSigners,
        /// Too few or too many required signatures.
        RequiredSignaturesOutOfBounds,
        /// Not a signer.
        NotASigner,
        /// No such multisig.
        NoSuchMultisig,
        /// Not a multisig authorization.
        NotAMultisigAuth,
        /// Not enough signers.
        NotEnoughSigners,
        /// A nonce overflow.
        NonceOverflow,
        /// Already approved.
        AlreadyApproved,
        /// Already a signer.
        AlreadyASigner,
    }
}

impl<T: Trait> Module<T> {
    /// Creates a multisig account without precondition checks or emitting an event.
    pub fn create_multisig_account(
        sender: T::AccountId,
        signers: &[Signatory],
        sigs_required: u64,
    ) -> CreateMultisigAccountResult<T> {
        let new_nonce = Self::ms_nonce()
            .checked_add(1)
            .ok_or(Error::<T>::NonceOverflow)?;
        <MultiSigNonce>::put(new_nonce);
        let account_id =
            Self::get_multisig_address(sender, new_nonce).map_err(|_| Error::<T>::DecodingError)?;
        for signer in signers {
            <identity::Module<T>>::add_auth(
                Signatory::from(AccountKey::try_from(account_id.encode())?),
                *signer,
                AuthorizationData::AddMultiSigSigner,
                None,
            );
        }
        <MultiSigSignsRequired<T>>::insert(&account_id, &sigs_required);
        Ok(account_id)
    }

    /// Creates a new proposal
    pub fn create_proposal(
        multisig: T::AccountId,
        sender_signer: Signatory,
        proposal: Box<T::Proposal>,
    ) -> CreateProposalResult {
        ensure!(
            <MultiSigSigners<T>>::exists(&multisig, &sender_signer),
            Error::<T>::NotASigner
        );
        let proposal_id = Self::ms_tx_done(multisig.clone());
        <Proposals<T>>::insert((multisig.clone(), proposal_id), proposal.clone());
        <ProposalIds<T>>::insert(multisig.clone(), *proposal, proposal_id);
        // Since proposal_ids are always only incremented by 1, they can not overflow.
        let next_proposal_id: u64 = proposal_id + 1u64;
        <MultiSigTxDone<T>>::insert(multisig.clone(), next_proposal_id);
        Self::deposit_event(RawEvent::ProposalAdded(multisig.clone(), proposal_id));
        Self::approve_for(multisig, sender_signer, proposal_id)?;
        Ok(proposal_id)
    }

    /// Creates or approves a multisig proposal
    pub fn create_or_approve_proposal(
        multisig: T::AccountId,
        sender_signer: Signatory,
        proposal: Box<T::Proposal>,
    ) -> DispatchResult {
        if let Some(proposal_id) = Self::proposal_ids(&multisig, &*proposal) {
            // This is an existing proposal.
            Self::approve_for(multisig, sender_signer, proposal_id)?;
        } else {
            // The proposal is new.
            Self::create_proposal(multisig, sender_signer, proposal)?;
        }
        Ok(())
    }

    /// Approves a multisig transaction and executes the proposal if enough sigs have been received
    pub fn approve_for(
        multisig: T::AccountId,
        signer: Signatory,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            <MultiSigSigners<T>>::exists(&multisig, &signer),
            Error::<T>::NotASigner
        );
        let multisig_signer_proposal = (multisig.clone(), signer, proposal_id);
        let multisig_proposal = (multisig.clone(), proposal_id);
        ensure!(
            !Self::votes(&multisig_signer_proposal),
            Error::<T>::AlreadyApproved
        );
        if let Some(proposal) = Self::proposals(&multisig_proposal) {
            Self::charge_fee(multisig.clone(), proposal.get_dispatch_info().weight)?;
            <Votes<T>>::insert(&multisig_signer_proposal, true);
            // Since approvals are always only incremented by 1, they can not overflow.
            let approvals: u64 = Self::tx_approvals(&multisig_proposal) + 1u64;
            <TxApprovals<T>>::insert(&multisig_proposal, approvals);
            let approvals_needed = Self::ms_signs_required(multisig.clone());
            if approvals >= approvals_needed {
                let who_key = AccountKey::try_from(multisig.clone().encode())?;
                match <identity::Module<T>>::get_identity(&who_key) {
                    Some(id) => {
                        <identity::CurrentDid>::put(id);
                    }
                    _ => return Err(Error::<T>::IdentityMissing.into()),
                };
                let res = match proposal
                    .dispatch(frame_system::RawOrigin::Signed(multisig.clone()).into())
                {
                    Ok(_) => true,
                    Err(e) => {
                        let e: DispatchError = e.into();
                        sp_runtime::print(e);
                        false
                    }
                };
                Self::deposit_event(RawEvent::ProposalExecuted(multisig, proposal_id, res));
                return Ok(());
            } else {
                return Ok(());
            }
        } else {
            return Err(Error::<T>::ProposalMissing.into());
        }
    }

    /// Charges appropriate fee for the proposal
    fn charge_fee(_multisig: T::AccountId, _weight: Weight) -> DispatchResult {
        // TODO use this weight to charge appropriate fee
        Ok(())
    }

    /// Accept and process addition of a signer to a multisig
    pub fn _accept_multisig_signer(signer: Signatory, auth_id: u64) -> DispatchResult {
        ensure!(
            <identity::Authorizations<T>>::exists(signer, auth_id),
            AuthorizationError::Invalid
        );

        let auth = <identity::Authorizations<T>>::get(signer, auth_id);

        ensure!(
            auth.authorization_data == AuthorizationData::AddMultiSigSigner,
            Error::<T>::NotAMultisigAuth
        );

        let wallet_id = {
            if let Signatory::AccountKey(multisig_key) = auth.authorized_by {
                T::AccountId::decode(&mut &multisig_key.as_slice()[..])
                    .map_err(|_| Error::<T>::DecodingError)
            } else {
                Err(Error::<T>::DecodingError.into())
            }
        }?;

        ensure!(
            <MultiSigSignsRequired<T>>::exists(&wallet_id),
            Error::<T>::NoSuchMultisig
        );
        ensure!(
            !<MultiSigSigners<T>>::exists(&wallet_id, &signer),
            Error::<T>::AlreadyASigner
        );
        let wallet_signer = Signatory::from(AccountKey::try_from(wallet_id.encode())?);
        <identity::Module<T>>::consume_auth(wallet_signer, signer, auth_id)?;

        <MultiSigSigners<T>>::insert(wallet_id.clone(), signer, signer);
        <NumberOfSigners<T>>::mutate(wallet_id.clone(), |x| *x = *x + 1u64);

        Self::deposit_event(RawEvent::MultiSigSignerAdded(wallet_id, signer));

        Ok(())
    }

    pub fn get_next_multisig_address(sender: T::AccountId) -> T::AccountId {
        // Nonce is always only incremented by small numbers and hence can never overflow 64 bits.
        // Also, this is just a helper function that does not modify state.
        let new_nonce = Self::ms_nonce() + 1;
        Self::get_multisig_address(sender, new_nonce).unwrap_or_default()
    }

    pub fn get_multisig_address(
        sender: T::AccountId,
        nonce: u64,
    ) -> Result<T::AccountId, CodecError> {
        let h: T::Hash = T::Hashing::hash(&(b"MULTI_SIG", nonce, sender).encode());
        T::AccountId::decode(&mut &h.encode()[..])
    }

    /// Helper function that checks if someone is an authorized signer of a multisig or not
    pub fn ms_signers(multi_sig: T::AccountId, signer: Signatory) -> bool {
        <MultiSigSigners<T>>::exists(multi_sig, signer)
    }
}

impl<T: Trait> AddSignerMultiSig for Module<T> {
    fn accept_multisig_signer(signer: Signatory, auth_id: u64) -> DispatchResult {
        Self::_accept_multisig_signer(signer, auth_id)
    }
}
