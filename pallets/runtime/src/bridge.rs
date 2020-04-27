//! Bridge from Ethereum to Polymesh
//!
//! This module implements a one-way bridge between Polymath Classic on the Ethereum side, and
//! Polymesh native. It mints POLYX on Polymesh in return for permanently locked ERC20 POLYX tokens.

use crate::multisig;
use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_support::traits::{Currency, Get};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    weights::{DispatchClass, FunctionOf, SimpleDispatchInfo},
};
use frame_system::{self as system, ensure_signed};
use polymesh_primitives::{AccountKey, Signatory};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::{balances::CheckCdd, CommonTrait};
use polymesh_runtime_identity as identity;
use sp_core::H256;
use sp_runtime::traits::{One, Zero};
use sp_std::{convert::TryFrom, prelude::*};

pub trait Trait: multisig::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Proposal: From<Call<Self>> + Into<<Self as identity::Trait>::Proposal>;
    /// The maximum number of timelocked bridge transactions that can be scheduled to be
    /// executed in a single block. Any excess bridge transactions are scheduled in later
    /// blocks.
    type MaxTimelockedTxsPerBlock: Get<u32>;
}

/// The status of a bridge tx
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BridgeTxStatus {
    /// No such tx in the system.
    Absent,
    /// Tx missing cdd or bridge module frozen.
    /// u8 represents number of times the module tried processing this tx.
    /// It will be retried automatically. Anyone can manually retry these.
    Pending(u8),
    /// Tx frozen by admin. It will not be retried automatically.
    Frozen,
    /// Tx pending first execution. These can not be manually triggered by normal accounts.
    Timelocked,
    /// Tx has been credited.
    Handled,
}

impl Default for BridgeTxStatus {
    fn default() -> Self {
        BridgeTxStatus::Absent
    }
}

/// A unique lock-and-mint bridge transaction
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BridgeTx<Account, Balance> {
    /// A single tx hash can have multiple locks. This nonce differentiates between them.
    pub nonce: u32,
    /// Recipient of POLYX on Polymesh: the deposit address or identity.
    pub recipient: Account,
    /// Amount of POLYX tokens to credit.
    pub amount: Balance,
    /// Ethereum token lock transaction hash.
    pub tx_hash: H256,
    // NB: The bridge module no longer uses eth tx hash. It's here for compatibility reasons.
}

/// Additional details about a bridge tx
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BridgeTxDetail<Balance, BlockNumber> {
    /// Amount of POLYX tokens to credit.
    pub amount: Balance,
    /// Status of the bridge tx.
    pub status: BridgeTxStatus,
    /// Block number at which this tx was executed or is planned to be executed.
    pub execution_block: BlockNumber,
    /// Ethereum token lock transaction hash.
    pub tx_hash: H256,
    // NB: The bridge module no longer uses eth tx hash. It's here for compatibility reasons.
}

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
        /// The transaction is time locked
        TimelockedTx,
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

        /// Status of bridge transactions
        BridgeTxDetails get(fn bridge_tx_details):
            double_map
                hasher(blake2_128_concat) T::AccountId,
                hasher(blake2_128_concat) u32
            =>
                BridgeTxDetail<T::Balance, T::BlockNumber>;

        /// The admin key.
        Admin get(fn admin) config(): T::AccountId;

        /// Whether or not the bridge operation is frozen.
        Frozen get(fn frozen): bool;

        /// The bridge transaction timelock period, in blocks, since the acceptance of the
        /// transaction proposal during which the admin key can freeze the transaction.
        Timelock get(fn timelock) config(): T::BlockNumber;

        /// The list of timelocked transactions with the block numbers in which those transactions
        /// become unlocked. Pending transactions are also included here if they will be tried automatically.
        TimelockedTxs get(fn timelocked_txs):
            map hasher(twox_64_concat) T::BlockNumber => Vec<BridgeTx<T::AccountId, T::Balance>>;
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
        /// Confirmation of Admin change.
        AdminChanged(AccountId),
        /// Confirmation of default timelock change.
        TimelockChanged(BlockNumber),
        /// Confirmation of minting POLYX on Polymesh in return for the locked ERC20 tokens on
        /// Ethereum.
        Bridged(BridgeTx<AccountId, Balance>),
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
            <Controller<T>>::put(controller.clone());
            Self::deposit_event(RawEvent::ControllerChanged(controller));
            Ok(())
        }

        /// Change the bridge admin key.
        #[weight = SimpleDispatchInfo::FixedOperational(20_000)]
        pub fn change_admin(origin, admin: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
            <Admin<T>>::put(admin.clone());
            Self::deposit_event(RawEvent::AdminChanged(admin));
            Ok(())
        }

        /// Change the timelock period.
        #[weight = SimpleDispatchInfo::FixedOperational(20_000)]
        pub fn change_timelock(origin, timelock: T::BlockNumber) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
            <Timelock<T>>::put(timelock.clone());
            Self::deposit_event(RawEvent::TimelockChanged(timelock));
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

        /// Handles an approved bridge transaction proposal.
        #[weight = SimpleDispatchInfo::FixedNormal(750_000)]
        pub fn handle_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId, T::Balance>) ->
            DispatchResult
        {
            let sender = ensure_signed(origin)?;
            let mut tx_details = Self::bridge_tx_details(&bridge_tx.recipient, &bridge_tx.nonce);
            match tx_details.status {
                // New bridge tx
                BridgeTxStatus::Absent => {
                    //TODO: Review admin permissions to handle bridge txs before mainnet
                    ensure!(sender == Self::controller() || sender == Self::admin(), Error::<T>::BadCaller);
                    let timelock = Self::timelock();
                    if timelock.is_zero() {
                        return Self::handle_bridge_tx_now(bridge_tx, false);
                    } else {
                        return Self::handle_bridge_tx_later(bridge_tx, timelock);
                    }
                }
                // Pending cdd bridge tx
                BridgeTxStatus::Pending(_) => {
                    return Self::handle_bridge_tx_now(bridge_tx, true);
                }
                // Pre frozen tx. We just set the correct amount.
                BridgeTxStatus::Frozen => {
                    //TODO: Review admin permissions to handle bridge txs before mainnet
                    ensure!(sender == Self::controller() || sender == Self::admin(), Error::<T>::BadCaller);
                    tx_details.amount = bridge_tx.amount;
                    <BridgeTxDetails<T>>::insert(&bridge_tx.recipient, &bridge_tx.nonce, tx_details);
                    Ok(())
                }
                BridgeTxStatus::Timelocked => {
                    return Err(Error::<T>::TimelockedTx.into());
                }
                BridgeTxStatus::Handled => {
                    return Err(Error::<T>::ProposalAlreadyHandled.into());
                }
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
                let tx_details = Self::bridge_tx_details(&bridge_tx.recipient, &bridge_tx.nonce);
                ensure!(tx_details.status != BridgeTxStatus::Handled, Error::<T>::ProposalAlreadyHandled);
                <BridgeTxDetails<T>>::mutate(&bridge_tx.recipient, &bridge_tx.nonce, |tx_detail| tx_detail.status = BridgeTxStatus::Frozen);
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
            // NB: An admin can call Freeze + Unfreeze on a transaction to bypass the timelock
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
            for bridge_tx in bridge_txs {
                let tx_details = Self::bridge_tx_details(&bridge_tx.recipient, &bridge_tx.nonce);
                ensure!(tx_details.status == BridgeTxStatus::Frozen, Error::<T>::NoSuchFrozenTx);
                <BridgeTxDetails<T>>::mutate(&bridge_tx.recipient, &bridge_tx.nonce, |tx_detail| tx_detail.status = BridgeTxStatus::Absent);
                Self::deposit_event(RawEvent::UnfrozenTx(bridge_tx.clone()));
                if let Err(e) = Self::handle_bridge_tx_now(bridge_tx, true) {
                    sp_runtime::print(e);
                }
            }
            Ok(())
        }

    }
}

impl<T: Trait> Module<T> {
    /// Issues the transacted amount to the recipient or returns a pending transaction.
    fn issue(recipient: &T::AccountId, amount: &T::Balance) -> DispatchResult {
        ensure!(
            T::CddChecker::check_key_cdd(&AccountKey::try_from(recipient.encode())?),
            Error::<T>::NoValidCdd
        );

        let _pos_imbalance = <balances::Module<T>>::deposit_creating(&recipient, *amount);

        Ok(())
    }

    /// Handles a bridge transaction proposal immediately.
    fn handle_bridge_tx_now(
        bridge_tx: BridgeTx<T::AccountId, T::Balance>,
        untrusted_manual_retry: bool,
    ) -> DispatchResult {
        let mut tx_details = Self::bridge_tx_details(&bridge_tx.recipient, &bridge_tx.nonce);
        // NB: This function does not care if a transaction is timelocked. Therefore, this should only be called
        // after timelock has expired or timelock is to be bypassed by an admin.
        ensure!(
            tx_details.status != BridgeTxStatus::Handled,
            Error::<T>::ProposalAlreadyHandled
        );
        ensure!(
            tx_details.status != BridgeTxStatus::Frozen,
            Error::<T>::FrozenTx
        );

        if Self::frozen() {
            // Untruested manual retries not allowed during frozen state.
            ensure!(!untrusted_manual_retry, Error::<T>::Frozen);
            // Bridge module frozen. Retry this tx again later.
            return Self::handle_bridge_tx_later(bridge_tx, Self::timelock());
        }

        let amount = if untrusted_manual_retry {
            // NB: The amount should be fetched from storage since the amount in `bridge_tx`
            // may be altered in a manual retry
            tx_details.amount
        } else {
            bridge_tx.amount
        };
        if Self::issue(&bridge_tx.recipient, &amount).is_ok() {
            tx_details.status = BridgeTxStatus::Handled;
            tx_details.execution_block = <system::Module<T>>::block_number();
            <BridgeTxDetails<T>>::insert(&bridge_tx.recipient, &bridge_tx.nonce, tx_details);
            Self::deposit_event(RawEvent::Bridged(bridge_tx));
        } else if !untrusted_manual_retry {
            // NB: If this was a manual retry, tx's automated retry schedule is not updated.
            // Recipient missing CDD or limit reached. Retry this tx again later.
            return Self::handle_bridge_tx_later(bridge_tx, Self::timelock());
        }
        Ok(())
    }

    /// Handles a bridge transaction proposal after `timelock` blocks.
    fn handle_bridge_tx_later(
        bridge_tx: BridgeTx<T::AccountId, T::Balance>,
        timelock: T::BlockNumber,
    ) -> DispatchResult {
        let mut already_tried = 0;
        let mut tx_details = Self::bridge_tx_details(&bridge_tx.recipient, &bridge_tx.nonce);
        match tx_details.status {
            BridgeTxStatus::Absent => {
                tx_details.status = BridgeTxStatus::Timelocked;
                tx_details.amount = bridge_tx.amount.clone();
            }
            BridgeTxStatus::Pending(x) => {
                tx_details.status = BridgeTxStatus::Pending(x + 1);
                already_tried = x + 1;
            }
            BridgeTxStatus::Timelocked => {
                tx_details.status = BridgeTxStatus::Pending(1);
                already_tried = 1;
            }
            BridgeTxStatus::Frozen => {
                return Err(Error::<T>::FrozenTx.into());
            }
            BridgeTxStatus::Handled => {
                return Err(Error::<T>::ProposalAlreadyHandled.into());
            }
        }
        tx_details.tx_hash = bridge_tx.tx_hash.clone();

        if already_tried > 24 {
            // Limits the exponential backoff to *almost infinity* (~180 years)
            already_tried = 24;
        }

        let current_block_number = <system::Module<T>>::block_number();
        let mut unlock_block_number =
            current_block_number + timelock + T::BlockNumber::from(2u32.pow(already_tried.into()));
        let max_timelocked_txs_per_block = T::MaxTimelockedTxsPerBlock::get() as usize;
        while Self::timelocked_txs(unlock_block_number).len() >= max_timelocked_txs_per_block {
            unlock_block_number += One::one();
        }

        tx_details.execution_block = unlock_block_number;
        <BridgeTxDetails<T>>::insert(&bridge_tx.recipient, &bridge_tx.nonce, tx_details);
        <TimelockedTxs<T>>::mutate(unlock_block_number, |txs| {
            txs.push(bridge_tx);
        });
        Ok(())
    }

    /// Handles the timelocked transactions that are set to unlock at the given block number.
    fn handle_timelocked_txs(block_number: T::BlockNumber) {
        let txs = <TimelockedTxs<T>>::take(block_number);
        for tx in txs {
            if let Err(e) = Self::handle_bridge_tx_now(tx, false) {
                sp_runtime::print(e);
            }
        }
    }

    // /// Emits an event containing the timelocked balances of a given `IssueRecipient`.
    // ///
    // /// TODO: Convert this method to an RPC call.
    // pub fn get_timelocked_balances_of_recipient(
    //     issue_recipient: IssueRecipient<T::AccountId>,
    // ) -> DispatchResult {
    //     ensure!(!Self::frozen(), Error::<T>::Frozen);
    //     let mut timelocked_balances = Vec::new();
    //     for (n, txs) in <TimelockedTxs<T>>::enumerate() {
    //         let sum_balance = |accum, tx: &BridgeTx<_, _>| {
    //             if tx.recipient == issue_recipient {
    //                 accum + tx.amount
    //             } else {
    //                 accum
    //             }
    //         };
    //         let recipients_balance: T::Balance = txs.iter().fold(Zero::zero(), sum_balance);
    //         timelocked_balances.push((n, recipients_balance));
    //     }
    //     Self::deposit_event(RawEvent::TimelockedBalancesOfRecipient(timelocked_balances));
    //     Ok(())
    // }
}
