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

use polymesh_primitives::{
    AccountKey, AuthorizationData, AuthorizationError, IdentityId, Signatory,
};
use polymesh_runtime_common::{
    identity::Trait as IdentityTrait, multisig::AddSignerMultiSig, Context,
};
use polymesh_runtime_identity as identity;

use codec::{Decode, Encode, Error as CodecError};
use core::convert::{From, TryInto};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    weights::GetDispatchInfo,
    StorageValue,
};
use frame_system::{self as system, ensure_signed};
use pallet_transaction_payment::{CddAndFeeDetails, ChargeTxFee};
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
        /// Individual multisig signer votes. (multi sig, signer, proposal) => vote
        pub Votes get(votes): map (T::AccountId, Signatory, u64) => bool;
        /// Maps a multisig to its creator's identity
        pub MultiSigCreator get(ms_creator): map T::AccountId => IdentityId;
        /// Maps a key to a multisig address
        pub KeyToMultiSig get(key_to_ms): map T::AccountId => T::AccountId;
    }
}

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
            ensure!(!signers.is_empty(), Error::<T>::NoSigners);
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
            ensure!(<MultiSigSignsRequired<T>>::exists(&sender), Error::<T>::NoSuchMultisig);
            let sender_signer = Signatory::from(AccountKey::try_from(sender.encode())?);
            Self::unsafe_add_auth_for_signers(sender_signer, signer, sender);
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
            Self::unsafe_signer_removal(sender, &signer);
            Ok(())
        }

        /// Add a signer to the multisig.
        /// This must be called by the creator identity of the multisig.
        ///
        /// # Arguments
        /// * `multisig` - Address of the multi sig
        /// * `signers` - Signatories to add.
        pub fn add_multisig_signers_via_creator(origin, multisig: T::AccountId, signers: Vec<Signatory>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<MultiSigSignsRequired<T>>::exists(&multisig), Error::<T>::NoSuchMultisig);
            let sender_key = AccountKey::try_from(sender.encode())?;
            let signer_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;
            ensure!(
                <MultiSigCreator<T>>::get(&multisig) == signer_did,
                Error::<T>::IdentityNotCreator
            );
            let multisig_signer = Signatory::from(AccountKey::try_from(multisig.encode())?);
            for signer in signers {
                Self::unsafe_add_auth_for_signers(multisig_signer, signer, multisig.clone());
            }
            Ok(())
        }

        /// Remove a signer from the multisig.
        /// This must be called by the creator identity of the multisig.
        ///
        /// # Arguments
        /// * `multisig` - Address of the multi sig
        /// * `signers` - Signatories to remove.
        pub fn remove_multisig_signers_via_creator(origin, multisig: T::AccountId, signers: Vec<Signatory>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<MultiSigSignsRequired<T>>::exists(&multisig), Error::<T>::NoSuchMultisig);
            let sender_key = AccountKey::try_from(sender.encode())?;
            let signer_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;
            ensure!(
                <MultiSigCreator<T>>::get(&multisig) == signer_did,
                Error::<T>::IdentityNotCreator
            );
            let signers_len:u64 = u64::try_from(signers.len()).unwrap_or_default();

            // NB: the below check can be underflowed but that doesnt matter
            // because the checks in the next loop will fail in that case.
            ensure!(
                <NumberOfSigners<T>>::get(&multisig) - signers_len >= <MultiSigSignsRequired<T>>::get(&multisig),
                Error::<T>::NotEnoughSigners
            );

            for signer in signers {
                ensure!(<MultiSigSigners<T>>::exists(&multisig, &signer), Error::<T>::NotASigner);
                Self::unsafe_signer_removal(multisig.clone(), &signer);
            }

            <NumberOfSigners<T>>::mutate(&multisig, |x| *x = *x - signers_len);

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
            Self::unsafe_change_sigs_required(sender, sigs_required);
            Ok(())
        }

        /// This function allows to replace all existing signers of the given multisig & also change no. of signature required
        /// NOTE - Once this function get executed no other function of the multisig is allowed to execute until unless
        /// potential signers accept the authorization and there count should be greater than or equal to the signature required
        ///
        /// # Arguments
        /// * signers - Vector of signers for a given multisig
        /// * sigs_required - Number of signature required for a given multisig
        pub fn change_all_signers_and_sigs_required(origin, signers: Vec<Signatory>, sigs_required: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_signer = Signatory::from(AccountKey::try_from(sender.encode())?);
            ensure!(<MultiSigSignsRequired<T>>::exists(&sender), Error::<T>::NoSuchMultisig);
            ensure!(signers.len() > 0, Error::<T>::NoSigners);
            ensure!(u64::try_from(signers.len()).unwrap_or_default() >= sigs_required && sigs_required > 0,
                Error::<T>::RequiredSignaturesOutOfBounds
            );

            // Collect the list of all signers present for the given multisig
            let current_signers = <MultiSigSigners<T>>::iter_prefix(&sender).collect::<Vec<Signatory>>();
            // Collect all those signers who need to be removed. It means those signers that are not exist in the signers vector
            // but present in the current_signers vector
            let old_signers = current_signers.clone().into_iter().filter(|x| !signers.contains(x)).collect::<Vec<Signatory>>();
            // Collect all those signers who need to be added. It means those signers that are not exist in the current_signers vector
            // but present in the signers vector
            let new_signers = signers.into_iter().filter(|x| !current_signers.contains(x)).collect::<Vec<Signatory>>();
            // Removing the signers from the valid multi-signers list first
            old_signers.iter()
                .for_each(|signer| {
                    Self::unsafe_signer_removal(sender.clone(), signer);
                });

            // Add the new signers for the given multi-sig
            new_signers.into_iter()
                .for_each(|signer| {
                    Self::unsafe_add_auth_for_signers(sender_signer, signer, sender.clone())
                });
            // Change the no. of signers for a multisig
            <NumberOfSigners<T>>::mutate(&sender, |x| *x = *x - u64::try_from(old_signers.len()).unwrap_or_default());
            // Change the required signature count
            Self::unsafe_change_sigs_required(sender, sigs_required);

            Ok(())
        }

        /// Adds a multisig as a signer of current did if the current did is the creator of the multisig
        ///
        /// # Arguments
        /// * `multi_sig` - multi sig address
        pub fn make_multisig_signer(origin, multi_sig: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<MultiSigSignsRequired<T>>::exists(&multi_sig), Error::<T>::NoSuchMultisig);
            let sender_key = AccountKey::try_from(sender.encode())?;
            let signer_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;
            ensure!(
                <MultiSigCreator<T>>::get(&multi_sig) == signer_did,
                Error::<T>::IdentityNotCreator
            );
            <identity::Module<T>>::unsafe_join_identity(
                signer_did,
                Signatory::from(AccountKey::try_from(multi_sig.encode())?)
            )
        }

        /// Adds a multisig as the master key of the current did if the current did is the creator of the multisig
        ///
        /// # Arguments
        /// * `multi_sig` - multi sig address
        pub fn make_multisig_master(origin, multi_sig: T::AccountId, optional_cdd_auth_id: Option<u64>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<MultiSigSignsRequired<T>>::exists(&multi_sig), Error::<T>::NoSuchMultisig);
            let sender_key = AccountKey::try_from(sender.encode())?;
            let signer_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;
            ensure!(
                <MultiSigCreator<T>>::get(&multi_sig) == signer_did,
                Error::<T>::IdentityNotCreator
            );
            <identity::Module<T>>::unsafe_master_key_rotation(
                AccountKey::try_from(multi_sig.encode())?,
                signer_did,
                optional_cdd_auth_id
            )
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
        /// The multisig is not attached to a CDD'd identity
        CddMissing,
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
        /// Couldn't charge fee for the transaction
        FailedToChargeFee,
        /// Identity provided is not the multisig's creator
        IdentityNotCreator,
        /// Signer is an account key that is already associated with a multisig
        SignerAlreadyLinked
    }
}

impl<T: Trait> Module<T> {
    /// Private immutables

    /// Add authorization for the accountKey to become a signer of multisig
    fn unsafe_add_auth_for_signers(from: Signatory, target: Signatory, authorizer: T::AccountId) {
        <identity::Module<T>>::add_auth(from, target, AuthorizationData::AddMultiSigSigner, None);
        Self::deposit_event(RawEvent::MultiSigSignerAuthorized(authorizer, target));
    }

    /// Remove signer from the valid signer list for a given multisig
    fn unsafe_signer_removal(multisig: T::AccountId, signer: &Signatory) {
        if let Signatory::AccountKey(key) = signer {
            if let Ok(signer_key) = T::AccountId::decode(&mut &key.as_slice()[..]) {
                <KeyToMultiSig<T>>::remove(&signer_key);
            }
        }
        <MultiSigSigners<T>>::remove(&multisig, signer);
        Self::deposit_event(RawEvent::MultiSigSignerRemoved(multisig, *signer));
    }

    /// Change the required signature count for a given multisig
    fn unsafe_change_sigs_required(multisig: T::AccountId, sigs_required: u64) {
        <MultiSigSignsRequired<T>>::insert(&multisig, &sigs_required);
        Self::deposit_event(RawEvent::MultiSigSignaturesRequiredChanged(
            multisig,
            sigs_required,
        ));
    }

    /// Creates a multisig account without precondition checks or emitting an event.
    pub fn create_multisig_account(
        sender: T::AccountId,
        signers: &[Signatory],
        sigs_required: u64,
    ) -> CreateMultisigAccountResult<T> {
        let sender_key = AccountKey::try_from(sender.encode())?;
        let signer_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;
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
        <MultiSigCreator<T>>::insert(&account_id, &signer_did);
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
            <Votes<T>>::insert(&multisig_signer_proposal, true);
            // Since approvals are always only incremented by 1, they can not overflow.
            let approvals: u64 = Self::tx_approvals(&multisig_proposal) + 1u64;
            <TxApprovals<T>>::insert(&multisig_proposal, approvals);
            let approvals_needed = Self::ms_signs_required(multisig.clone());
            if approvals >= approvals_needed {
                let ms_key = AccountKey::try_from(multisig.clone().encode())?;
                if let Some(did) = <identity::Module<T>>::get_identity(&ms_key) {
                    ensure!(
                        <identity::Module<T>>::has_valid_cdd(did),
                        Error::<T>::CddMissing
                    );
                    T::CddHandler::set_current_identity(&did);
                } else {
                    let creator_identity = Self::ms_creator(&multisig);
                    ensure!(
                        <identity::Module<T>>::has_valid_cdd(creator_identity),
                        Error::<T>::CddMissing
                    );
                    T::CddHandler::set_current_identity(&creator_identity);
                }
                ensure!(
                    T::ChargeTxFeeTarget::charge_fee(
                        proposal.encode().len().try_into().unwrap_or_default(),
                        proposal.get_dispatch_info(),
                    )
                    .is_ok(),
                    Error::<T>::FailedToChargeFee
                );
                let res = match proposal
                    .dispatch(frame_system::RawOrigin::Signed(multisig.clone()).into())
                {
                    Ok(_) => true,
                    Err(e) => {
                        let e: DispatchError = e;
                        sp_runtime::print(e);
                        false
                    }
                };
                Self::deposit_event(RawEvent::ProposalExecuted(multisig, proposal_id, res));
                Ok(())
            } else {
                Ok(())
            }
        } else {
            Err(Error::<T>::ProposalMissing.into())
        }
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
                Err(Error::<T>::DecodingError)
            }
        }?;

        if let Signatory::AccountKey(key) = signer {
            let signer_key = T::AccountId::decode(&mut &key.as_slice()[..])
                .map_err(|_| Error::<T>::DecodingError)?;
            ensure!(
                !<KeyToMultiSig<T>>::exists(&signer_key),
                Error::<T>::SignerAlreadyLinked
            );
            <KeyToMultiSig<T>>::insert(signer_key, wallet_id.clone())
        }

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
        <NumberOfSigners<T>>::mutate(wallet_id.clone(), |x| *x += 1u64);

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
