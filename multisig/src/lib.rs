//! # Multisig Module

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use rstd::{convert::TryFrom, prelude::*};
use sr_primitives::{
    traits::{Dispatchable, Hash},
    weights::SimpleDispatchInfo,
    DispatchError,
};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, Parameter, StorageValue,
};
use system::ensure_signed;

pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// A forwardable call.
    type Proposal: Parameter + Dispatchable<Origin = Self::Origin>;
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

        fn create_multi_sig(origin, owners: Vec<T::AccountId>, sigs_required: u64) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(owners.len() > 0, "No owners provided");
            ensure!(u64::try_from(owners.len()).unwrap_or_default() >= sigs_required && sigs_required > 0,
                "Sigs required out of bounds"
            );
            let nonce: u64 = Self::ms_nonce();
            let new_nonce: u64 = nonce + 1u64;
            <MultiSigNonce>::put(new_nonce);

            let mut buf = Vec::new();
            buf.extend_from_slice(&nonce.encode());
            buf.extend_from_slice(&sender.encode());
            let h: T::Hash = T::Hashing::hash(&buf[..]);
            let wallet_id;
            match T::AccountId::decode(&mut &h.encode()[..]) {
                Ok(v) => wallet_id = v,
                Err(_) => return Err("Error in decoding multisig address"),
            };

            for owner in owners.clone() {
                <MultiSigOwners<T>>::insert((owner, wallet_id.clone()), true);
            }

            <MultiSigSignsRequired<T>>::insert(&wallet_id, &sigs_required);

            Self::deposit_event(RawEvent::MultiSigCreated(wallet_id, sender, owners, sigs_required));

            Ok(())
        }

        fn approve(origin, multi_sig: T::AccountId, proposal_id: u64) -> Result {
            let sender = ensure_signed(origin)?;
            if let Some(proposal) = Self::proposals((multi_sig.clone(), proposal_id)) {
                let res = match proposal.dispatch(system::RawOrigin::Signed(multi_sig.clone()).into()) {
                    Ok(_) => true,
                    Err(e) => {
                        let e: DispatchError = e.into();
                        sr_primitives::print(e);
                        false
                    }
                };
                Self::deposit_event(RawEvent::ProposalExecuted(multi_sig, proposal_id, res));
                return Ok(());
            } else {
                return Err("Proposal can not be executed");
            }
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
        /// Emitted when a proposal is executed. (Multisig, proposalid, result)
        ProposalExecuted(AccountId, u64, bool),
    }
);
