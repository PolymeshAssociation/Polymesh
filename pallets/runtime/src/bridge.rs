//! Bridge from Ethereum to Polymesh
//!
//! This module implements a one-way bridge between Polymath Classic on the Ethereum side, and
//! Polymesh native. It mints POLY on Polymesh in return for permanently locked ERC20 POLY tokens.

use crate::multisig;
use codec::{Decode, Encode};
use core::result::Result as StdResult;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::Currency;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use frame_system::{self as system, ensure_signed};
use polymesh_primitives::{traits::IdentityCurrency, AccountKey, IdentityId, Signatory};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::CommonTrait;
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

/// Converts an `IssueRecipient` to the identity and the account of the recipient. The returned
/// error covers the case when there is a bug in the code and the account ID does not encode as
/// bytes.
fn identity_and_account<'a, T: 'a + Trait>(
    recipient: &'a IssueRecipient<T::AccountId>,
) -> StdResult<(Option<IdentityId>, Option<&'a T::AccountId>), &'static str> {
    Ok(match recipient {
        IssueRecipient::Account(account_id) => {
            let to_key = AccountKey::try_from(account_id.encode())?;
            (
                <identity::Module<T>>::get_identity(&to_key),
                Some(account_id),
            )
        }
        IssueRecipient::Identity(did) => (Some(*did), None),
    })
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

/// A transaction that is pending a valid identity CDD.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PendingTx<AccountId, Balance> {
    /// The identity on which the CDD is pending.
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
        /// The bridge controller address is not set.
        ControllerNotSet,
        /// The signer does not have an identity.
        IdentityMissing,
        /// Failure to credit the recipient account.
        CannotCreditAccount,
        /// Failure to credit the recipient identity.
        CannotCreditIdentity,
        /// The origin is not the controller address.
        BadCaller,
        /// The recipient DID has no valid CDD.
        NoValidCdd,
        /// The bridge transaction proposal has already been handled and the funds minted.
        ProposalAlreadyHandled,
        /// Unauthorized to perform an operation.
        Unauthorized,
        /// The bridge is already frozen.
        Frozen,
        /// The bridge is not frozen.
        NotFrozen,
        /// There is no such frozen transaction.
        NoSuchFrozenTx,
        /// There is no proposal corresponding to a given bridge transaction.
        NoSuchProposal,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        /// The multisig account of the bridge controller. The genesis signers must accept their
        /// authorizations to be able to get their proposals delivered.
        Controller get(controller) build(|config: &GenesisConfig| {
            if config.signatures_required > u64::try_from(config.signers.len()).unwrap_or_default()
            {
                panic!("too many signatures required");
            }
            if config.signatures_required == 0 {
                // Default to the empty signer set.
                return Default::default();
            }
            <multisig::Module<T>>::create_multisig_account(
                Default::default(),
                config.signers.as_slice(),
                config.signatures_required
            ).expect("cannot create the bridge multisig")
        }): T::AccountId;
        /// Pending issuance transactions to identities.
        PendingTxs get(pending_txs): map IdentityId => Vec<BridgeTx<T::AccountId, T::Balance>>;
        /// Frozen transactions.
        FrozenTxs get(frozen_txs): map BridgeTx<T::AccountId, T::Balance> => bool;
        /// Handled bridge transactions.
        HandledTxs get(handled_txs): map BridgeTx<T::AccountId, T::Balance> => bool;
        /// The admin key.
        AdminKey get(admin_key) config(): AccountKey;
        /// Whether or not the bridge operation is frozen.
        Frozen get(frozen): bool;
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
        ControllerChanged(AccountId),
        /// Confirmation of minting POLY on Polymesh in return for the locked ERC20 tokens on
        /// Ethereum.
        Bridged(BridgeTx<AccountId, Balance>),
        /// Notification of an approved transaction having moved to a pending state due to the
        /// recipient identity either being non-existent or not having a valid CDD.
        Pending(PendingTx<AccountId, Balance>),
        /// Notification of a failure to finalize a pending transaction. The transaction is removed.
        Failed(BridgeTx<AccountId, Balance>),
        /// Notification of freezing the bridge.
        Frozen,
        /// Notification of unfreezing the bridge.
        Unfrozen,
        /// Notification of freezing a transaction.
        FrozenTx(BridgeTx<AccountId, Balance>),
        /// Notification of unfreezing a transaction.
        UnfrozenTx(BridgeTx<AccountId, Balance>),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Change the controller account as admin.
        pub fn change_controller(origin, account_id: T::AccountId) -> DispatchResult {
            Self::check_admin(origin)?;
            <Controller<T>>::put(account_id);
            Ok(())
        }

        /// Change the bridge admin key.
        pub fn change_admin_key(origin, account_key: AccountKey) -> DispatchResult {
            Self::check_admin(origin)?;
            <AdminKey>::put(account_key);
            Ok(())
        }

        /// Freezes the entire operation of the bridge module if it is not already frozen. The only
        /// available operations in the frozen state are the following admin methods:
        ///
        /// * `change_controller`,
        /// * `change_admin_key`,
        /// * `unfreeze`,
        /// * `freeze_bridge_txs`,
        /// * `unfreeze_bridge_txs`.
        pub fn freeze(origin) -> DispatchResult {
            Self::check_admin(origin)?;
            ensure!(!Self::frozen(), Error::<T>::Frozen);
            <Frozen>::put(true);
            Self::deposit_event(RawEvent::Frozen);
            Ok(())
        }

        /// Unfreezes the operation of the bridge module if it is frozen.
        pub fn unfreeze(origin) -> DispatchResult {
            Self::check_admin(origin)?;
            ensure!(Self::frozen(), Error::<T>::NotFrozen);
            <Frozen>::put(false);
            Self::deposit_event(RawEvent::Unfrozen);
            Ok(())
        }

        /// Proposes a bridge transaction, which amounts to making a multisig proposal for the
        /// bridge transaction if the transaction is new or approving an existing proposal if the
        /// transaction has already been proposed.
        pub fn propose_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId, T::Balance>) ->
            DispatchResult
        {
            ensure!(!Self::frozen(), Error::<T>::Frozen);
            let controller = Self::controller();
            ensure!(controller != Default::default(), Error::<T>::ControllerNotSet);
            let proposal = <T as Trait>::Proposal::from(Call::<T>::handle_bridge_tx(bridge_tx));
            let boxed_proposal = Box::new(proposal.into());
            <multisig::Module<T>>::create_or_approve_proposal_as_identity(
                origin,
                controller,
                boxed_proposal
            )
        }

        /// Finalizes pending bridge transactions following a receipt of a valid CDD by the
        /// recipient identity.
        pub fn finalize_pending(_origin, did: IdentityId) -> DispatchResult {
            ensure!(!Self::frozen(), Error::<T>::Frozen);
            ensure!(<identity::Module<T>>::has_valid_cdd(did), Error::<T>::NoValidCdd);
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

        /// Handles an approved bridge transaction proposal.
        ///
        /// NOTE: Extrinsics without `pub` are exported too. This function is declared as `pub` only
        /// to test that it cannot be called from a wrong `origin`.
        pub fn handle_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId, T::Balance>) ->
            DispatchResult
        {
            let sender = ensure_signed(origin.clone())?;
            ensure!(sender == Self::controller(), Error::<T>::BadCaller);
            ensure!(!Self::handled_txs(&bridge_tx), Error::<T>::ProposalAlreadyHandled);
            if Self::frozen() {
                if !Self::frozen_txs(&bridge_tx) {
                    // Move the transaction to the list of frozen transactions.
                    <FrozenTxs<T>>::insert(&bridge_tx, true);
                    Self::deposit_event(RawEvent::FrozenTx(bridge_tx));
                }
                return Ok(());
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
                <HandledTxs<T>>::insert(&bridge_tx, true);
                Self::deposit_event(RawEvent::Bridged(bridge_tx));
            }
            Ok(())
        }

        /// Freezes given bridge transactions.
        pub fn freeze_txs(origin, bridge_txs: Vec<BridgeTx<T::AccountId, T::Balance>>) ->
            DispatchResult
        {
            Self::check_admin(origin)?;
            for bridge_tx in bridge_txs {
                let proposal =
                    <T as Trait>::Proposal::from(Call::<T>::handle_bridge_tx(bridge_tx.clone())).into();
                let proposal_id = <multisig::Module<T>>::proposal_ids(&Self::controller(), &proposal);
                ensure!(proposal_id.is_some(), Error::<T>::NoSuchProposal);
                ensure!(!Self::handled_txs(&bridge_tx), Error::<T>::ProposalAlreadyHandled);
                <FrozenTxs<T>>::insert(&bridge_tx, true);
                Self::deposit_event(RawEvent::FrozenTx(bridge_tx));
            }
            Ok(())
        }

        /// Unfreezes given bridge transactions.
        pub fn unfreeze_txs(origin, bridge_txs: Vec<BridgeTx<T::AccountId, T::Balance>>) ->
            DispatchResult
        {
            Self::check_admin(origin)?;
            for bridge_tx in bridge_txs {
                ensure!(!Self::handled_txs(&bridge_tx), Error::<T>::ProposalAlreadyHandled);
                ensure!(Self::frozen_txs(&bridge_tx), Error::<T>::NoSuchFrozenTx);
                <FrozenTxs<T>>::remove(&bridge_tx);
                Self::deposit_event(RawEvent::UnfrozenTx(bridge_tx.clone()));
                if let Some(PendingTx {
                        did,
                        bridge_tx,
                }) = Self::issue(bridge_tx.clone())? {
                    <PendingTxs<T>>::mutate(did, |pending_txs| {
                        pending_txs.push(bridge_tx.clone())
                    });
                    Self::deposit_event(RawEvent::Pending(PendingTx {
                        did,
                        bridge_tx
                    }));
                } else {
                    <HandledTxs<T>>::insert(&bridge_tx, true);
                    Self::deposit_event(RawEvent::Bridged(bridge_tx));
                }
            }
            Ok(())
        }

        /// Performs the admin authorization check. The check is successful iff the origin is the
        /// bridge admin key.
        pub fn check_admin(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let account_key = AccountKey::try_from(sender.encode())?;
            ensure!(account_key == Self::admin_key(), Error::<T>::Unauthorized);
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
        let (did, account_id) = identity_and_account::<T>(recipient)?;
        if let Some(did) = did {
            // Issue to an identity or to an account associated with one.
            if <identity::Module<T>>::has_valid_cdd(did) {
                let neg_imbalance = <balances::Module<T>>::issue(*amount);
                let resolution = if let Some(account_id) = account_id {
                    <balances::Module<T>>::resolve_into_existing(account_id, neg_imbalance)
                } else {
                    <balances::Module<T>>::resolve_into_existing_identity(&did, neg_imbalance)
                };
                resolution.map_err(|_| Error::<T>::CannotCreditAccount)?;
            } else {
                return Ok(Some(PendingTx { did, bridge_tx }));
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
