//! Bridge from Ethereum to Polymesh
//!
//! This module implements a one-way bridge between Polymath Classic on the Ethereum side, and
//! Polymesh native. It mints POLY on Polymesh in return for permanently locked ERC20 POLY tokens.

use crate::{balances, identity, multisig};
use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_support::traits::Currency;
use frame_support::{decl_event, decl_module, decl_storage, ensure};
use frame_system::{self as system, ensure_signed};
use primitives::traits::IdentityCurrency;
use primitives::{IdentityId, Key, Signer};
use sp_core::H256;
use sp_std::{convert::TryFrom, prelude::*};

pub trait Trait: balances::Trait + multisig::Trait {
    type Balance: From<u128> + Into<<Self as balances::Trait>::Balance>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Proposal: From<Call<Self>> + Into<<Self as identity::Trait>::Proposal>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        /// The multisig account of the bridge validator set.
        Validators get(validators): T::AccountId;
        /// Correspondence between validator set change proposals and multisig proposal IDs.
        ChangeValidatorsProposals get(change_validators_proposals): map T::AccountId => u64;
        /// Correspondence between bridge transaction proposals and multisig proposal IDs.
        BridgeTxProposals get(bridge_tx_proposals): map hasher(blake2_256) BridgeTx<T::AccountId> => u64;
        /// Pending issuance transactions to identities.
        PendingTxs get(pending_txs): map IdentityId => Vec<PendingTx>;
    }
}

/// The intended recipient of POLY exchanged from the locked ERC20 tokens.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueRecipient<AccountId> {
    Account(AccountId),
    Identity(IdentityId),
}

/// A unique lock-and-mint bridge transaction containing Ethereum transaction data and a bridge nonce.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BridgeTx<AccountId> {
    /// Bridge validator runtime nonce.
    pub nonce: u64,
    /// Recipient of POLY on Polymesh: the deposit address or identity.
    pub recipient: IssueRecipient<AccountId>,
    /// Amount of tokens locked on Ethereum.
    pub value: u128,
    /// Ethereum token lock transaction hash.
    pub tx_hash: H256,
}

/// A pending bridge transaction that was approved by validators but delayed due to the recipient
/// identity not having a valid KYC.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PendingTx {
    /// Bridge validator runtime nonce.
    pub nonce: u64,
    /// Amount of tokens locked on Ethereum.
    pub value: u128,
    /// Ethereum token lock transaction hash.
    pub tx_hash: H256,
}

decl_event! {
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        /// Confirmation of a validator set change.
        ValidatorsChanged(AccountId),
        /// Confirmation of minting POLY on Polymesh in return for the locked ERC20 tokens on
        /// Ethereum.
        Bridged(BridgeTx<AccountId>),
        /// Notification of an approved transaction having moved to a pending state due to the
        /// recipient identity either being non-existent or not having a valid KYC.
        Pending(PendingTx),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Proposes to change the address of the bridge validator multisig accountn, which amounts
        /// to making a multisig proposal for the validator set change if the change is new or
        /// approving an existing proposal if the change has already been proposed.
        pub fn propose_change_validators(origin, new_validators: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let sender_signer = Signer::from(Key::try_from(sender.encode())?);
            let current_validators = Self::validators();
            if current_validators == Default::default() {
                // There are no validators to approve the change. Simply set the given validators.
                <Validators<T>>::put(new_validators);
            } else {
                let proposal_id = Self::change_validators_proposals(new_validators.clone());
                if proposal_id == 0 {
                    // This is a new proposal.
                    let proposal = <T as Trait>::Proposal::from(
                        Call::<T>::handle_change_validators(new_validators.clone())
                    );
                    let boxed_call = Box::new(proposal.into());
                    let proposal_id = <multisig::Module<T>>::create_proposal(
                        current_validators,
                        boxed_call,
                        sender_signer
                    )?;
                    <ChangeValidatorsProposals<T>>::insert(new_validators, proposal_id);
                } else {
                    // This is an existing proposal.
                    <multisig::Module<T>>::approve_as_key(origin, current_validators, proposal_id)?;
                }
            }
            Ok(())
        }

        /// Proposes a bridge transaction, which amounts to making a multisig proposal for the
        /// bridge transaction if the transaction is new or approving an existing proposal if the
        /// transaction has already been proposed.
        pub fn propose_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId>) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let sender_signer = Signer::from(Key::try_from(sender.encode())?);
            let validators = Self::validators();
            ensure!(validators != Default::default(), "bridge validators not set");
            let proposal_id = Self::bridge_tx_proposals(bridge_tx.clone());
            if proposal_id == 0 {
                // The proposal is new.
                let proposal = <T as Trait>::Proposal::from(
                    Call::<T>::handle_bridge_tx(bridge_tx.clone())
                );
                let boxed_call = Box::new(proposal.into());
                let proposal_id = <multisig::Module<T>>::create_proposal(
                    validators,
                    boxed_call,
                    sender_signer
                )?;
                <BridgeTxProposals<T>>::insert(bridge_tx, proposal_id);
            } else {
                // This is an existing proposal.
                <multisig::Module<T>>::approve_as_key(origin, validators, proposal_id)?;
            }
            Ok(())
        }

        /// Finalizes pending bridge transactions following a receipt of a valid KYC by the
        /// recipient identity.
        pub fn finalize_pending(origin, did: IdentityId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = Key::try_from(sender.encode())?;
            let signer = Signer::Key(sender_key.clone());
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer),
                    "sender must be a signing key for DID");
            ensure!(<identity::Module<T>>::has_valid_kyc(&did), "recipient DID has no valid KYC");
            for PendingTx {
                nonce,
                value,
                tx_hash,
            } in Self::pending_txs(did).drain(..) {
                let amount = <T as Trait>::Balance::from(value).into();
                let neg_imbalance = <balances::Module<T>>::issue(amount);
                <balances::Module<T>>::resolve_into_existing_identity(&did, neg_imbalance)
                    .map_err(|_| "failed to credit the issued amount")?;
                Self::deposit_event(RawEvent::Bridged(BridgeTx {
                    nonce,
                    recipient: IssueRecipient::Identity(did),
                    value,
                    tx_hash,
                }));
            }
            Ok(())
        }

        /// Handles an approved validator set change transaction proposal.
        fn handle_change_validators(_origin, new_validators: T::AccountId) -> DispatchResult {
            // Update the validator set.
            <Validators<T>>::put(new_validators.clone());
            // Remove the record of the ongoing proposal.
            <ChangeValidatorsProposals<T>>::remove(&new_validators);
            Self::deposit_event(RawEvent::ValidatorsChanged(new_validators));
            Ok(())
        }

        /// Handles an approved bridge transaction proposal.
        fn handle_bridge_tx(_origin, bridge_tx: BridgeTx<T::AccountId>) -> DispatchResult {
            // Not removing the ongoing proposal since that would have been unnecessary due to
            // proposal uniqueness.
            //
            // TODO: use `origin`?
            // let sender = ensure_signed(origin.clone())?;
            // let sender_key = Key::try_from(sender.encode())?;
            // let did = <identity::Module<T>>::get_identity(&sender_key)
            //     .ok_or_else(|| identity::Error::<T>::NoDIDFound)?;
            let BridgeTx {
                nonce,
                recipient,
                value,
                tx_hash,
            } = &bridge_tx;
            let (did, account_id) = match recipient {
                IssueRecipient::Account(account_id) => {
                    let to_key = Key::try_from(account_id.clone().encode())?;
                    (<identity::Module<T>>::get_identity(&to_key), Some(account_id))
                }
                IssueRecipient::Identity(did) => (Some(*did), None),
            };
            if let Some(did) = did {
                // Issue to an identity or to an account associated with one.
                if <identity::Module<T>>::has_valid_kyc(did) {
                    let amount = <T as Trait>::Balance::from(*value).into();
                    let neg_imbalance = <balances::Module<T>>::issue(amount);
                    let resolution = if let Some(account_id) = account_id {
                        <balances::Module<T>>::resolve_into_existing(account_id, neg_imbalance)
                    } else {
                        <balances::Module<T>>::resolve_into_existing_identity(&did, neg_imbalance)
                    };
                    resolution.map_err(|_| "failed to credit the issued amount")?;
                    Self::deposit_event(RawEvent::Bridged(bridge_tx));
                } else {
                    // Postpone issuance and change the recipient to the identity.
                    let pending_tx = PendingTx {
                        nonce: *nonce,
                        value: *value,
                        tx_hash: *tx_hash,
                    };
                    <PendingTxs>::mutate(did, |txs| txs.push(pending_tx.clone()));
                    Self::deposit_event(RawEvent::Pending(pending_tx));
                }
            } else if let Some(account_id) = account_id {
                // Issue to an account not associated with an identity.
                let amount = <T as Trait>::Balance::from(*value).into();
                let neg_imbalance = <balances::Module<T>>::issue(amount);
                <balances::Module<T>>::resolve_into_existing(account_id, neg_imbalance);
                Self::deposit_event(RawEvent::Bridged(bridge_tx));
            }
            Ok(())
        }
    }
}
