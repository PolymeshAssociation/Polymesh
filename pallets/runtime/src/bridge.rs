//! Bridge from Ethereum to Polymesh
//!
//! This module implements a one-way bridge between Polymath Classic on the Ethereum side, and
//! Polymesh native. It mints POLY on Polymesh in return for permanently locked ERC20 POLY tokens.

use crate::multisig;
use codec::{Decode, Encode};
use core::result::Result as StdResult;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::{Currency, Get};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    weights::{DispatchClass, FunctionOf, SimpleDispatchInfo},
};
use frame_system::{self as system, ensure_signed};
use polymesh_primitives::{traits::IdentityCurrency, AccountKey, IdentityId, Signatory};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::CommonTrait;
use polymesh_runtime_identity as identity;
use sp_core::H256;
use sp_runtime::traits::{One, Zero};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::{convert::TryFrom, prelude::*};

pub trait Trait: multisig::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Proposal: From<Call<Self>> + Into<<Self as identity::Trait>::Proposal>;
    /// The maximum number of timelocked bridge transactions that can be scheduled to be
    /// executed in a single block. Any excess bridge transactions are scheduled in later
    /// blocks.
    type MaxTimelockedTxsPerBlock: Get<u32>;
    /// The block number range in which to look for available blocks to put a timelocked
    /// transaction.
    type BlockRangeForTimelock: Get<Self::BlockNumber>;
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
        /// Failure to credit the recipient account or identity.
        CannotCreditRecipient,
        /// The origin is not the controller or the admin address.
        BadCaller,
        /// The origin is not the admin address.
        BadAdmin,
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
        /// The transaction is frozen.
        FrozenTx,
        /// There is no such frozen transaction.
        NoSuchFrozenTx,
        /// There is no proposal corresponding to a given bridge transaction.
        NoSuchProposal,
        /// All the blocks in the timelock block range are full.
        TimelockBlockRangeFull,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        /// The multisig account of the bridge controller. The genesis signers must accept their
        /// authorizations to be able to get their proposals delivered.
        Controller get(fn controller) build(|config: &GenesisConfig<T>| {
            if config.signatures_required > u64::try_from(config.signers.len()).unwrap_or_default()
            {
                panic!("too many signatures required");
            }
            if config.signatures_required == 0 {
                // Default to the empty signer set.
                return Default::default();
            }
            <multisig::Module<T>>::create_multisig_account(
                config.creator.clone(),
                config.signers.as_slice(),
                config.signatures_required
            ).expect("cannot create the bridge multisig")
        }): T::AccountId;
        /// Pending issuance transactions to identities.
        PendingTxs get(fn pending_txs): map hasher(blake2_128_concat) IdentityId => Vec<BridgeTx<T::AccountId, T::Balance>>;
        /// Frozen transactions.
        FrozenTxs get(fn frozen_txs): map hasher(blake2_128_concat) BridgeTx<T::AccountId, T::Balance> => bool;
        /// Handled bridge transactions.
        HandledTxs get(fn handled_txs): map hasher(blake2_128_concat) BridgeTx<T::AccountId, T::Balance> => bool;
        /// The admin key.
        Admin get(fn admin) config(): T::AccountId;
        /// Whether or not the bridge operation is frozen.
        Frozen get(fn frozen): bool;
        /// The bridge transaction timelock period, in blocks, since the acceptance of the
        /// transaction proposal during which the admin key can freeze the transaction.
        Timelock get(fn timelock) config(): T::BlockNumber;
        /// The list of timelocked transactions with the block numbers in which those transactions
        /// become unlocked.
        TimelockedTxs get(fn timelocked_txs):
            linked_map hasher(twox_64_concat) T::BlockNumber => Vec<BridgeTx<T::AccountId, T::Balance>>;
    }
    add_extra_genesis {
        // TODO: Remove multisig creator and add systematic CDD for the bridge multisig.
        /// AccountId of the multisig creator. Set to Alice for easier testing.
        config(creator): T::AccountId;
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
        Balance = <T as CommonTrait>::Balance,
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
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
        /// A vector of timelocked balances of a recipient, each with the number of the block in
        /// which the balance gets unlocked.
        TimelockedBalancesOfRecipient(Vec<(BlockNumber, Balance)>),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        const MaxTimelockedTxsPerBlock: u32 = T::MaxTimelockedTxsPerBlock::get();
        const BlockRangeForTimelock: T::BlockNumber = T::BlockRangeForTimelock::get();

        fn deposit_event() = default;

        /// Issue tokens in timelocked transactions.
        fn on_initialize(block_number: T::BlockNumber) {
            Self::handle_timelocked_txs(block_number);
        }

        /// Change the controller account as admin.
        #[weight = SimpleDispatchInfo::FixedOperational(20_000)]
        pub fn change_controller(origin, controller: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
            <Controller<T>>::put(controller);
            Ok(())
        }

        /// Change the bridge admin key.
        #[weight = SimpleDispatchInfo::FixedOperational(20_000)]
        pub fn change_admin(origin, admin: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
            <Admin<T>>::put(admin);
            Ok(())
        }

        /// Change the timelock period.
        #[weight = SimpleDispatchInfo::FixedOperational(20_000)]
        pub fn change_timelock(origin, timelock: T::BlockNumber) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
            <Timelock<T>>::put(timelock);
            Ok(())
        }

        /// Freezes the entire operation of the bridge module if it is not already frozen. The only
        /// available operations in the frozen state are the following admin methods:
        ///
        /// * `change_controller`,
        /// * `change_admin`,
        /// * `change_timelock`,
        /// * `unfreeze`,
        /// * `freeze_bridge_txs`,
        /// * `unfreeze_bridge_txs`.
        #[weight = SimpleDispatchInfo::FixedOperational(50_000)]
        pub fn freeze(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
            ensure!(!Self::frozen(), Error::<T>::Frozen);
            <Frozen>::put(true);
            Self::deposit_event(RawEvent::Frozen);
            Ok(())
        }

        /// Unfreezes the operation of the bridge module if it is frozen.
        #[weight = SimpleDispatchInfo::FixedOperational(50_000)]
        pub fn unfreeze(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
            ensure!(Self::frozen(), Error::<T>::NotFrozen);
            <Frozen>::put(false);
            Self::deposit_event(RawEvent::Unfrozen);
            Ok(())
        }

        /// Proposes a bridge transaction, which amounts to making a multisig proposal for the
        /// bridge transaction if the transaction is new or approving an existing proposal if the
        /// transaction has already been proposed.
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
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
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
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
        #[weight = SimpleDispatchInfo::FixedNormal(750_000)]
        pub fn handle_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId, T::Balance>) ->
            DispatchResult
        {
            let sender = ensure_signed(origin)?;
            //TODO: Review admin permissions to handle bridge txs before mainnet
            ensure!(sender == Self::controller() || sender == Self::admin(), Error::<T>::BadCaller);

            ensure!(!Self::handled_txs(&bridge_tx), Error::<T>::ProposalAlreadyHandled);
            if Self::frozen() {
                if !Self::frozen_txs(&bridge_tx) {
                    // Move the transaction to the list of frozen transactions.
                    <FrozenTxs<T>>::insert(&bridge_tx, true);
                    Self::deposit_event(RawEvent::FrozenTx(bridge_tx));
                }
                return Ok(());
            }
            ensure!(!Self::frozen_txs(&bridge_tx), Error::<T>::FrozenTx);
            let timelock = Self::timelock();
            if timelock.is_zero() {
                Self::handle_bridge_tx_now(bridge_tx)
            } else {
                Self::handle_bridge_tx_later(bridge_tx, timelock)
            }
        }

        /// Freezes given bridge transactions.
        ///
        /// # Weight
        /// `50_000 + 200_000 * bridge_txs.len()`
        #[weight = FunctionOf(
            |(bridge_txs,): (
                &Vec<BridgeTx<T::AccountId, T::Balance>>,
            )| {
                50_000 + 200_000 * u32::try_from(bridge_txs.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn freeze_txs(origin, bridge_txs: Vec<BridgeTx<T::AccountId, T::Balance>>) ->
            DispatchResult
        {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
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
        ///
        /// # Weight
        /// `50_000 + 700_000 * bridge_txs.len()`
        #[weight = FunctionOf(
            |(bridge_txs,): (
                &Vec<BridgeTx<T::AccountId, T::Balance>>,
            )| {
                50_000 + 700_000 * u32::try_from(bridge_txs.len()).unwrap_or_default()
            },
            DispatchClass::Normal,
            true
        )]
        pub fn unfreeze_txs(origin, bridge_txs: Vec<BridgeTx<T::AccountId, T::Balance>>) ->
            DispatchResult
        {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
            for bridge_tx in bridge_txs {
                ensure!(!Self::handled_txs(&bridge_tx), Error::<T>::ProposalAlreadyHandled);
                ensure!(Self::frozen_txs(&bridge_tx), Error::<T>::NoSuchFrozenTx);
                <FrozenTxs<T>>::remove(&bridge_tx);
                Self::deposit_event(RawEvent::UnfrozenTx(bridge_tx.clone()));
                if let Err(e) = Self::handle_bridge_tx_now(bridge_tx) {
                    sp_runtime::print(e);
                }
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
                resolution.map_err(|_| Error::<T>::CannotCreditRecipient)?;
            } else {
                return Ok(Some(PendingTx { did, bridge_tx }));
            }
        } else if let Some(account_id) = account_id {
            // Issue to an account not associated with an identity.
            let neg_imbalance = <balances::Module<T>>::issue(*amount);
            <balances::Module<T>>::resolve_into_existing(account_id, neg_imbalance)
                .map_err(|_| Error::<T>::CannotCreditRecipient)?;
        }
        Ok(None)
    }

    /// Handles a bridge transaction proposal immediately.
    fn handle_bridge_tx_now(bridge_tx: BridgeTx<T::AccountId, T::Balance>) -> DispatchResult {
        if let Some(PendingTx { did, bridge_tx }) = Self::issue(bridge_tx.clone())? {
            <PendingTxs<T>>::mutate(did, |txs| txs.push(bridge_tx.clone()));
            Self::deposit_event(RawEvent::Pending(PendingTx { did, bridge_tx }));
        } else {
            <HandledTxs<T>>::insert(&bridge_tx, true);
            Self::deposit_event(RawEvent::Bridged(bridge_tx));
        }
        Ok(())
    }

    /// Handles a bridge transaction proposal after `timelock` blocks.
    fn handle_bridge_tx_later(
        bridge_tx: BridgeTx<T::AccountId, T::Balance>,
        timelock: T::BlockNumber,
    ) -> DispatchResult {
        let current_block_number = <system::Module<T>>::block_number();
        let mut unlock_block_number = current_block_number + timelock;
        let range = T::BlockRangeForTimelock::get();
        let max_unlock_block_number = unlock_block_number + range - One::one();
        let max_timelocked_txs_per_block = T::MaxTimelockedTxsPerBlock::get() as usize;
        while Self::timelocked_txs(unlock_block_number).len() >= max_timelocked_txs_per_block
            && unlock_block_number <= max_unlock_block_number
        {
            unlock_block_number += One::one();
        }
        ensure!(
            unlock_block_number <= max_unlock_block_number,
            Error::<T>::TimelockBlockRangeFull
        );
        <TimelockedTxs<T>>::mutate(unlock_block_number, |txs| {
            txs.push(bridge_tx);
        });
        Ok(())
    }

    /// Handles the timelocked transactions that are set to unlock at the given block number.
    fn handle_timelocked_txs(block_number: T::BlockNumber) {
        let txs = <TimelockedTxs<T>>::take(block_number);
        for tx in txs {
            if let Err(e) = Self::handle_bridge_tx_now(tx) {
                sp_runtime::print(e);
            }
        }
    }

    /// Emits an event containing the timelocked balances of a given `IssueRecipient`.
    ///
    /// TODO: Convert this method to an RPC call.
    pub fn get_timelocked_balances_of_recipient(
        issue_recipient: IssueRecipient<T::AccountId>,
    ) -> DispatchResult {
        ensure!(!Self::frozen(), Error::<T>::Frozen);
        let mut timelocked_balances = Vec::new();
        for (n, txs) in <TimelockedTxs<T>>::enumerate() {
            let sum_balance = |accum, tx: &BridgeTx<_, _>| {
                if tx.recipient == issue_recipient {
                    accum + tx.amount
                } else {
                    accum
                }
            };
            let recipients_balance: T::Balance = txs.iter().fold(Zero::zero(), sum_balance);
            timelocked_balances.push((n, recipients_balance));
        }
        Self::deposit_event(RawEvent::TimelockedBalancesOfRecipient(timelocked_balances));
        Ok(())
    }
}
