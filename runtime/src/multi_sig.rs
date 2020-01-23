//! # Multisig Module

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use rstd::{convert::TryFrom, prelude::*};
use sr_primitives::{
    traits::{Dispatchable, Hash},
    weights::{ClassifyDispatch, DispatchClass, GetDispatchInfo, WeighData, Weight},
    DispatchError,
};
use srml_support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, Parameter, StorageValue,
};
use system::ensure_signed;
use primitives::{
    Authorization, AuthorizationData, IdentityId, Key, Signer
};

pub trait GetCallWeightTrait<AccountId> {
    fn get_proposal_weight(multi_sig: &AccountId, proposal_id: &u64) -> Weight;
}

impl<T: Trait> GetCallWeightTrait<T::AccountId> for Module<T> {
    fn get_proposal_weight(multi_sig: &T::AccountId, proposal_id: &u64) -> Weight {
        if let Some(proposal) = Self::proposals(((*multi_sig).clone(), *proposal_id)) {
            proposal.get_dispatch_info().weight
        } else {
            0
        }
    }
}

struct ChargeProposal<GetCallWeight, AccountId>(
    rstd::marker::PhantomData<(GetCallWeight, AccountId)>,
);

impl<GetCallWeight, AccountId> ChargeProposal<GetCallWeight, AccountId> {
    fn new() -> Self {
        Self(Default::default())
    }
}

impl<GetCallWeight: GetCallWeightTrait<AccountId>, AccountId> WeighData<(&AccountId, &u64)>
    for ChargeProposal<GetCallWeight, AccountId>
{
    fn weigh_data(&self, (multi_sig, proposal_id): (&AccountId, &u64)) -> Weight {
        let weight = GetCallWeight::get_proposal_weight(multi_sig, proposal_id);
        weight + 10_000
    }
}

impl<GetCallWeight: GetCallWeightTrait<AccountId>, AccountId> ClassifyDispatch<(&AccountId, &u64)>
    for ChargeProposal<GetCallWeight, AccountId>
{
    fn classify_dispatch(&self, _: (&AccountId, &u64)) -> DispatchClass {
        DispatchClass::Normal
    }
}

pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// A forwardable call.
    type Proposal: Parameter + Dispatchable<Origin = Self::Origin> + GetDispatchInfo;
}

decl_storage! {
    trait Store for Module<T: Trait> as Multisig {
        /// Nonce to ensure unique Multisig addresses are generated. starts from 1.
        pub MultiSigNonce get(ms_nonce) build(|_| 1u64): u64;

        /// Owners of a multisig. (mulisig, owner) => true/false
        pub MultiSigOwners get(ms_owners): map (T::AccountId, T::AccountId) => bool;
        /// Confirmations required before processing a multisig tx
        pub MultiSigSignsRequired get(ms_signs_required): map T::AccountId => u64;
        /// Number of transactions proposed in a multisig. Used as tx id. starts from 0
        pub MultiSigTxDone get(ms_tx_done): map T::AccountId => u64;

        /// Proposals presented for voting to a multisig (multisig, proposal id) => Option<proposal>
        /// Deleted after proposal is processed
        pub Proposals get(proposals): map (T::AccountId, u64) => Option<T::Proposal>;

        /// Number of votes in favor of a tx. Mapping from (multisig, tx id) => no. of approvals.
        pub TxApprovals get(tx_approvals): map (T::AccountId, u64) => u64;
        /// Individual multisig owner votes. (multi sig, owner, )
        pub Votes get(votes): map (T::AccountId, T::AccountId, u64) => bool;
    }
}

decl_module! {
    // Simple declaration of the `Module` type. Lets the macro know what it's working on.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn create_multi_sig(origin, owners: Vec<T::AccountId>, sigs_required: u64) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(owners.len() > 0, "No owners provided");
            ensure!(u64::try_from(owners.len()).unwrap_or_default() >= sigs_required && sigs_required > 0,
                "Sigs required out of bounds"
            );
            let nonce: u64 = Self::ms_nonce();
            let new_nonce: u64 = nonce + 1u64;
            <MultiSigNonce>::put(new_nonce);

            let mut buf = Vec::new();
            buf.extend_from_slice(&b"MULTI_SIG".encode());
            buf.extend_from_slice(&nonce.encode());
            buf.extend_from_slice(&sender.encode());
            let h: T::Hash = T::Hashing::hash(&buf[..]);
            let wallet_id;
            match T::AccountId::decode(&mut &h.encode()[..]) {
                Ok(v) => wallet_id = v,
                Err(_) => return Err("Error in decoding multisig address"),
            };

            for owner in owners.clone() {
                <MultiSigOwners<T>>::insert((wallet_id.clone(), owner), true);
            }

            <MultiSigSignsRequired<T>>::insert(&wallet_id, &sigs_required);

            Self::deposit_event(RawEvent::MultiSigCreated(wallet_id, sender, owners, sigs_required));

            Ok(())
        }

        pub fn create_proposal(origin, multi_sig: T::AccountId, proposal: Box<T::Proposal>) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::ms_owners((multi_sig.clone(), sender.clone())), "not an owner");
            let proposal_id = Self::ms_tx_done(multi_sig.clone());
            <Proposals<T>>::insert((multi_sig.clone(), proposal_id), proposal);
            let next_proposal_id: u64 = proposal_id + 1u64;
            <MultiSigTxDone<T>>::insert(multi_sig.clone(), next_proposal_id);
            Self::deposit_event(RawEvent::ProposalAdded(multi_sig.clone(), proposal_id));
            Self::approve_for(multi_sig, proposal_id, sender)
        }

        pub fn approve(origin, multi_sig: T::AccountId, proposal_id: u64) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::ms_owners((multi_sig.clone(), sender.clone())), "not an owner");
            Self::approve_for(multi_sig, proposal_id, sender)
        }

        //#[weight = <ChargeProposal<Module<T>, T::AccountId>>::new()]
        pub fn execute(origin, multi_sig: T::AccountId, proposal_id: u64) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::ms_owners((multi_sig.clone(), sender.clone())), "not an owner");
            Self::execute_tx(multi_sig, proposal_id)
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        /// Event for multi sig creation. (Multisig address, Creator address, Owners, Sigs required)
        MultiSigCreated(AccountId, AccountId, Vec<AccountId>, u64),
        /// Event for adding a proposal (Multisig, proposalid)
        ProposalAdded(AccountId, u64),
        /// Emitted when a proposal is executed. (Multisig, proposalid, result)
        ProposalExecuted(AccountId, u64, bool),
    }
);

impl<T: Trait> Module<T> {
    fn approve_for(multi_sig: T::AccountId, proposal_id: u64, signer: T::AccountId) -> Result {
        let multi_sig_signer_proposal = (multi_sig.clone(), signer.clone(), proposal_id);
        let multi_sig_proposal = (multi_sig.clone(), proposal_id);
        ensure!(!Self::votes(&multi_sig_signer_proposal), "Already approved");
        ensure!(
            Self::proposals(&multi_sig_proposal).is_some(),
            "Invalid proposal"
        );
        <Votes<T>>::insert(&multi_sig_signer_proposal, true);
        let approvals: u64 = Self::tx_approvals(&multi_sig_proposal) + 1u64;
        <TxApprovals<T>>::insert(&multi_sig_proposal, approvals);
        Ok(())
    }

    fn execute_tx(multi_sig: T::AccountId, proposal_id: u64) -> Result {
        let multi_sig_proposal = (multi_sig.clone(), proposal_id);
        if let Some(proposal) = Self::proposals(&multi_sig_proposal) {
            let approvals = Self::tx_approvals(&multi_sig_proposal);
            let approvals_needed = Self::ms_signs_required(multi_sig.clone());
            if approvals >= approvals_needed {
                let res =
                    match proposal.dispatch(system::RawOrigin::Signed(multi_sig.clone()).into()) {
                        Ok(_) => true,
                        Err(e) => {
                            let e: DispatchError = e.into();
                            sr_primitives::print(e);
                            false
                        }
                    };
                Self::deposit_event(RawEvent::ProposalExecuted(multi_sig, proposal_id, res));
            }
            return Ok(());
        } else {
            return Err("Invalid proposal");
        }
    }

    fn charge_fee(multi_sig: T::AccountId, proposal_id: u64) -> Result {
        let _weight = match Self::proposals((multi_sig.clone(), proposal_id)) {
            Some(proposal) => proposal.get_dispatch_info().weight,
            _ => 0,
        };
        Ok(())
    }
}
