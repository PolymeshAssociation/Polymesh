//! Bridge from Ethereum to Polymesh
//!
//! This module implements a one-way bridge between Polymath Classic on the Ethereum side, and
//! Polymesh native. It mints POLY on Polymesh in return for permanently locked ERC20 POLY tokens.

use crate::multisig;
use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::Currency;
use frame_support::{decl_error, decl_event, decl_module, decl_storage};
use frame_system::{self as system, ensure_root, ensure_signed};
use polymesh_primitives::{traits::IdentityCurrency, AccountKey, IdentityId, Signatory};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::{traits::CommonTrait, Context};
use polymesh_runtime_identity as identity;
use sp_core::H256;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::{convert::TryFrom, prelude::*};

pub trait Trait: multisig::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Proposal: From<Call<Self>> + Into<<Self as identity::Trait>::Proposal>;
}

/// The intended recipient of POLY exchanged from the locked ERC20 tokens.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueRecipient<AccountId> {
    Account(AccountId),
    Identity(IdentityId),
}

/// A unique lock-and-mint bridge transaction containing Ethereum transaction data and a bridge nonce.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BridgeTx<AccountId, Balance> {
    /// Bridge signer runtime nonce.
    pub nonce: u64,
    /// Recipient of POLY on Polymesh: the deposit address or identity.
    pub recipient: IssueRecipient<AccountId>,
    /// Amount of tokens locked on Ethereum.
    pub amount: Balance,
    /// Ethereum token lock transaction hash.
    pub tx_hash: H256,
}

/// A transaction that is pending a valid identity KYC.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PendingTx<AccountId, Balance> {
    /// The identity on which the KYC is pending.
    pub did: IdentityId,
    /// The pending transaction.
    pub bridge_tx: BridgeTx<AccountId, Balance>,
}

/// Either a pending transaction or a `None` or an error.
type IssueResult<T> = sp_std::result::Result<
    Option<PendingTx<<T as frame_system::Trait>::AccountId, <T as CommonTrait>::Balance>>,
    DispatchError,
>;

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The bridge relayer set address is not set.
        RelayersNotSet,
        /// The signer does not have an identity.
        IdentityMissing,
        /// Failure to credit the recipient account.
        CannotCreditAccount,
        /// Failure to credit the recipient identity.
        CannotCreditIdentity,
        /// The origin is not the relayer set multisig.
        BadCaller,
        /// The recipient DID has no valid KYC.
        NoValidKyc,
        /// The bridge transaction proposal has already been handled and the funds minted.
        ProposalAlreadyHandled,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        /// The multisig account of the bridge relayer set. The genesis signers must accept their
        /// authorizations to be able to get their proposals delivered.
        Relayers get(relayers) build(|config: &GenesisConfig| {
            if config.signatures_required > u64::try_from(config.signers.len()).unwrap_or_default()
            {
                panic!("too many signatures required");
            }
            if config.signatures_required == 0 {
                /// Default to the empty signer set.
                return Default::default();
            }
            <multisig::Module<T>>::create_multisig_account(
                Default::default(),
                config.signers.as_slice(),
                config.signatures_required
            ).expect("cannot create the bridge multisig")
        }): T::AccountId;
        /// Correspondence between bridge transaction proposals and multisig proposal IDs.
        BridgeTxProposals get(bridge_tx_proposals): map BridgeTx<T::AccountId, T::Balance> => Option<u64>;
        /// Pending issuance transactions to identities.
        PendingTxs get(pending_txs): map IdentityId => Vec<BridgeTx<T::AccountId, T::Balance>>;
        /// Handled bridge transaction proposals.
        HandledProposals get(handled_proposals): map BridgeTx<T::AccountId, T::Balance> => bool;
    }
    add_extra_genesis {
        /// The set of initial signers from which a multisig address is created at genesis time.
        config(signers): Vec<Signatory>;
        /// The number of required signatures in the genesis signer set.
        config(signatures_required): u64;
    }
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = <T as CommonTrait>::Balance
    {
        /// Confirmation of a signer set change.
        RelayersChanged(AccountId),
        /// Confirmation of minting POLY on Polymesh in return for the locked ERC20 tokens on
        /// Ethereum.
        Bridged(BridgeTx<AccountId, Balance>),
        /// Notification of an approved transaction having moved to a pending state due to the
        /// recipient identity either being non-existent or not having a valid KYC.
        Pending(PendingTx<AccountId, Balance>),
        /// Notification of a failure to finalize a pending transaction. The transaction is removed.
        Failed(BridgeTx<AccountId, Balance>),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Change the signer set account as root.
        pub fn change_relayers(origin, account_id: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <Relayers<T>>::put(account_id);
            Ok(())
        }

        /// Proposes a bridge transaction, which amounts to making a multisig proposal for the
        /// bridge transaction if the transaction is new or approving an existing proposal if the
        /// transaction has already been proposed.
        pub fn propose_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId, T::Balance>) ->
            DispatchResult
        {
            let sender = ensure_signed(origin.clone())?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let sender_did = Context::current_identity_or::<identity::Module<T>>(&sender_key)?;
            let sender_signer = Signatory::from(sender_did);
            let relayers = Self::relayers();
            if relayers == Default::default() {
                return Err(Error::<T>::RelayersNotSet.into());
            }
            if let Some(proposal_id) = Self::bridge_tx_proposals(&bridge_tx) {
                // This is an existing proposal.
                <multisig::Module<T>>::approve_as_identity(origin, relayers, proposal_id)?;
            } else {
                // The proposal is new.
                let proposal = <T as Trait>::Proposal::from(
                    Call::<T>::handle_bridge_tx(bridge_tx.clone())
                );
                let boxed_call = Box::new(proposal.into());
                let proposal_id = <multisig::Module<T>>::create_proposal(
                    relayers,
                    sender_signer,
                    boxed_call
                )?;
                <BridgeTxProposals<T>>::insert(bridge_tx, proposal_id);
            }
            Ok(())
        }

        /// Finalizes pending bridge transactions following a receipt of a valid KYC by the
        /// recipient identity.
        pub fn finalize_pending(_origin, did: IdentityId) -> DispatchResult {
            if <identity::Module<T>>::has_valid_kyc(did).is_none() {
                return Err(Error::<T>::NoValidKyc.into());
            }
            let mut new_pending_txs: BTreeMap<_, Vec<BridgeTx<T::AccountId, T::Balance>>> =
                BTreeMap::new();
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

        /// Handles an approved signer set multisig account change proposal.
        pub fn handle_relayers(origin, account_id: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;
            if sender != Self::relayers() {
                return Err(Error::<T>::BadCaller.into());
            }
            // Update the bridge signers.
            <Relayers<T>>::put(account_id.clone());
            Self::deposit_event(RawEvent::RelayersChanged(account_id));
            Ok(())
        }

        /// Handles an approved bridge transaction proposal.
        ///
        /// NOTE: Extrinsics without `pub` are exported too. This function is declared as `pub` only
        /// to test that it cannot be called from a wrong `origin`.
        pub fn handle_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId, T::Balance>) ->
            DispatchResult
        {
            let sender = ensure_signed(origin.clone())?;
            if sender != Self::relayers() {
                return Err(Error::<T>::BadCaller.into());
            }
            if Self::handled_proposals(&bridge_tx) {
                return Err(Error::<T>::ProposalAlreadyHandled.into());
            }
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
                <HandledProposals<T>>::insert(&bridge_tx, true);
                Self::deposit_event(RawEvent::Bridged(bridge_tx));
            }
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Issues the transacted amount to the recipient or returns a pending transaction.
    fn issue(bridge_tx: BridgeTx<T::AccountId, T::Balance>) -> IssueResult<T> {
        let BridgeTx {
            nonce: _,
            recipient,
            amount,
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
            if <identity::Module<T>>::has_valid_kyc(did).is_some() {
                let neg_imbalance = <balances::Module<T>>::issue(*amount);
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
            let neg_imbalance = <balances::Module<T>>::issue(*amount);
            <balances::Module<T>>::resolve_into_existing(account_id, neg_imbalance)
                .map_err(|_| Error::<T>::CannotCreditIdentity)?;
        }
        Ok(None)
    }
}
