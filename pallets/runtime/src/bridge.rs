//! Bridge from Ethereum to Polymesh
//!
//! This module implements a one-way bridge between Polymath Classic on the Ethereum side, and
//! Polymesh native. It mints POLY on Polymesh in return for permanently locked ERC20 POLY tokens.

use crate::multisig;
use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::Currency;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use frame_system::{self as system, ensure_signed};
use polymesh_primitives::{traits::IdentityCurrency, AccountKey, IdentityId, Signatory};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::CommonTrait;
use polymesh_runtime_identity as identity;
use sp_core::H256;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::{convert::TryFrom, prelude::*};

pub trait Trait: multisig::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Proposal: From<Call<Self>> + Into<<Self as identity::Trait>::Proposal>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The validator does not have an identity.
        IdentityMissing,
        /// Failure to credit the recipient account.
        CannotCreditAccount,
        /// Failure to credit the recipient identity.
        CannotCreditIdentity,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        /// The multisig account of the bridge validator set.
        Validators get(validators) build(|config: &GenesisConfig| {
            if config.signatures_required > u64::try_from(config.signers.len()).unwrap_or_default()
            {
                panic!("too many signatures required");
            }
            if config.signatures_required == 0 {
                /// Default to the empty validator set.
                return Default::default();
            }
            <multisig::Module<T>>::create_multisig_wallet(
                Default::default(),
                config.signers.as_slice(),
                config.signatures_required
            ).expect("cannot create the bridge multisig")
        }): T::AccountId;
        /// Correspondence between validator set change proposals and multisig proposal IDs.
        ChangeValidatorsProposals get(change_validators_proposals): map T::AccountId => Option<u64>;
        /// Correspondence between bridge transaction proposals -- identified with their nonces --
        /// and multisig proposal IDs.
        BridgeTxProposals get(bridge_tx_proposals): map u64 => Option<u64>;
        /// Pending issuance transactions to identities.
        PendingTxs get(pending_txs): map IdentityId => Vec<BridgeTx<T::AccountId>>;
    }
    add_extra_genesis {
        /// The set of initial validators from which a multisig address is created at genesis time.
        config(signers): Vec<Signatory>;
        /// The number of required signatures in the genesis validator set.
        config(signatures_required): u64;
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

/// A transaction that is pending a valid identity KYC.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PendingTx<AccountId> {
    /// The identity on which the KYC is pending.
    pub did: IdentityId,
    /// The pending transaction.
    pub bridge_tx: BridgeTx<AccountId>,
}

/// Either a pending transaction or a `None` or an error.
type IssueResult<AccountId> = sp_std::result::Result<Option<PendingTx<AccountId>>, DispatchError>;

decl_event! {
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        /// Confirmation of a validator set change.
        ValidatorsChanged(AccountId),
        /// Confirmation of minting POLY on Polymesh in return for the locked ERC20 tokens on
        /// Ethereum.
        Bridged(BridgeTx<AccountId>),
        /// Notification of an approved transaction having moved to a pending state due to the
        /// recipient identity either being non-existent or not having a valid KYC.
        Pending(PendingTx<AccountId>),
        /// Notification of a failure to finalize a pending transaction. The transaction is removed.
        Failed(BridgeTx<AccountId>),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Proposes to change the address of the bridge validator multisig accountn, which amounts
        /// to making a multisig proposal for the validator set change if the change is new or
        /// approving an existing proposal if the change has already been proposed.
        pub fn propose_change_validators(origin, new_validators: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let sender_did =  <identity::Module<T>>::current_did().map_or_else(|| {
                <identity::Module<T>>::get_identity(&sender_key)
                    .ok_or_else(|| Error::<T>::IdentityMissing)
            }, Ok)?;
            let sender_signer = Signatory::from(sender_did);
            let current_validators = Self::validators();
            if current_validators == Default::default() {
                // There are no validators to approve the change. Simply set the given validators.
                <Validators<T>>::put(new_validators);
            } else if current_validators != new_validators {
                // Change validators only if the new validator set has a different address.
                if let Some(proposal_id) = Self::change_validators_proposals(new_validators.clone()) {
                    // This is an existing proposal.
                    <multisig::Module<T>>::approve_as_identity(
                        origin,
                        current_validators,
                        proposal_id
                    )?;
                } else {
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
                }
            }
            Ok(())
        }

        /// Proposes a bridge transaction, which amounts to making a multisig proposal for the
        /// bridge transaction if the transaction is new or approving an existing proposal if the
        /// transaction has already been proposed.
        pub fn propose_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId>) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let sender_did = <identity::Module<T>>::current_did().map_or_else(|| {
                <identity::Module<T>>::get_identity(&sender_key)
                    .ok_or_else(|| Error::<T>::IdentityMissing)
            }, Ok)?;
            let sender_signer = Signatory::from(sender_did);
            let validators = Self::validators();
            ensure!(validators != Default::default(), "bridge validators not set");
            let nonce = bridge_tx.nonce;
            if let Some(proposal_id) = Self::bridge_tx_proposals(&nonce) {
                // This is an existing proposal.
                <multisig::Module<T>>::approve_as_identity(origin, validators, proposal_id)?;
            } else {
                // The proposal is new.
                let proposal = <T as Trait>::Proposal::from(Call::<T>::handle_bridge_tx(bridge_tx));
                let boxed_call = Box::new(proposal.into());
                let proposal_id = <multisig::Module<T>>::create_proposal(
                    validators,
                    boxed_call,
                    sender_signer
                )?;
                <BridgeTxProposals>::insert(nonce, proposal_id);
            }
            Ok(())
        }

        /// Finalizes pending bridge transactions following a receipt of a valid KYC by the
        /// recipient identity.
        pub fn finalize_pending(_origin, did: IdentityId) -> DispatchResult {
            ensure!(<identity::Module<T>>::has_valid_kyc(&did), "recipient DID has no valid KYC");
            let mut new_pending_txs: BTreeMap<_, Vec<BridgeTx<T::AccountId>>> = BTreeMap::new();
            for bridge_tx in Self::pending_txs(&did) {
                match Self::issue(bridge_tx.clone()) {
                    Ok(None) => Self::deposit_event(RawEvent::Bridged(bridge_tx)),
                    Ok(Some(PendingTx {
                        did: to_did,
                        bridge_tx,
                    })) => {
                        let entry = new_pending_txs
                            .entry(to_did)
                            .or_default();
                        entry.push(bridge_tx.clone());
                        Self::deposit_event(RawEvent::Pending(PendingTx {
                            did: to_did,
                            bridge_tx
                        }));
                    }
                    Err(_) => Self::deposit_event(RawEvent::Failed(bridge_tx)),
                }
            }
            for (to_did, txs) in new_pending_txs {
                if to_did == did {
                    <PendingTxs<T>>::insert(did, txs);
                } else {
                    <PendingTxs<T>>::mutate(to_did, |pending_txs| pending_txs.extend(txs));
                }
            }
            Ok(())
        }

        /// Handles an approved validator set change transaction proposal.
        ///
        /// NOTE: Extrinsics without `pub` are exported too. This function is declared as `pub` only
        /// to test that it cannot be called from a wrong `origin`.
        pub fn handle_change_validators(origin, new_validators: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            ensure!(sender == Self::validators(), "should be called by the validator set account");
            // Update the validator set.
            <Validators<T>>::put(new_validators.clone());
            // Remove the record of the ongoing proposal.
            <ChangeValidatorsProposals<T>>::remove(&new_validators);
            Self::deposit_event(RawEvent::ValidatorsChanged(new_validators));
            Ok(())
        }

        /// Handles an approved bridge transaction proposal.
        ///
        /// NOTE: Extrinsics without `pub` are exported too. This function is declared as `pub` only
        /// to test that it cannot be called from a wrong `origin`.
        pub fn handle_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId>) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            ensure!(sender == Self::validators(), "should be called by the validator set account");
            if let Some(PendingTx {
                did,
                bridge_tx,
            }) = Self::issue(bridge_tx.clone())? {
                <PendingTxs<T>>::mutate(did, |txs| txs.push(bridge_tx.clone()));
                Self::deposit_event(RawEvent::Pending(PendingTx {
                    did,
                    bridge_tx
                }));
            } else {
                Self::deposit_event(RawEvent::Bridged(bridge_tx));
            }
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Issues the transacted amount to the recipient or returns a pending transaction.
    fn issue(bridge_tx: BridgeTx<T::AccountId>) -> IssueResult<T::AccountId> {
        let BridgeTx {
            nonce: _,
            recipient,
            value,
            tx_hash: _,
        } = &bridge_tx;
        let (did, account_id) = match recipient {
            IssueRecipient::Account(account_id) => {
                let to_key = AccountKey::try_from(account_id.clone().encode())?;
                (
                    <identity::Module<T>>::get_identity(&to_key),
                    Some(account_id),
                )
            }
            IssueRecipient::Identity(did) => (Some(*did), None),
        };
        if let Some(did) = did {
            // Issue to an identity or to an account associated with one.
            if <identity::Module<T>>::has_valid_kyc(did) {
                let amount = <T::Balance as From<u128>>::from(*value);
                let neg_imbalance = <balances::Module<T>>::issue(amount);
                let resolution = if let Some(account_id) = account_id {
                    <balances::Module<T>>::resolve_into_existing(account_id, neg_imbalance)
                } else {
                    <balances::Module<T>>::resolve_into_existing_identity(&did, neg_imbalance)
                };
                resolution.map_err(|_| Error::<T>::CannotCreditAccount)?;
            } else {
                return Ok(Some(PendingTx {
                    did,
                    bridge_tx: bridge_tx,
                }));
            }
        } else if let Some(account_id) = account_id {
            // Issue to an account not associated with an identity.
            let amount = <T::Balance as From<u128>>::from(*value);
            let neg_imbalance = <balances::Module<T>>::issue(amount);
            <balances::Module<T>>::resolve_into_existing(account_id, neg_imbalance)
                .map_err(|_| Error::<T>::CannotCreditIdentity)?;
        }
        Ok(None)
    }
}
