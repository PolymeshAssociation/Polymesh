// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! # Bridge from Ethereum to Polymesh
//!
//! This module implements a one-way bridge between Polymath Classic on the Ethereum side, and
//! Polymesh native. It mints POLYX on Polymesh in return for permanently locked ERC20 POLY tokens.
//!
//! ## Overview
//!
//! The bridge module provides extrinsics that - when used in conjunction with the sudo or
//! [multisig](../../pallet_multisig/index.html) pallets - allow issuing tokens on Polymesh in
//! response to [bridge transactions](BridgeTx).
//!
//! ### Terminology
//!
//! - **bridge transaction**: an immutable data structure constructed by bridge signers containing a
//! unique nonce, the recipient account, the transaction value and the Ethereum transaction hash.
//!
//! - **bridge transaction status**: any bridge transaction has a unique status which is one of the
//! following:
//!   - **absent**: No such transaction is recorded in the bridge module.
//!   - **pending**: The transaction is pending a valid CDD check after a set amount of blocks.
//!   - **frozen**: The transaction has been frozen by the admin.
//!   - **timelocked**: The transaction has been added to the bridge processing queue and is
//!   currently pending its first execution. During this wait the admin can freeze the transaction.
//!   - **handled**: The transaction has been handled successfully and the tokens have been credited
//!   to the recipient account.
//!
//! - **bridge transaction queue**: a single queue of transactions, each identified with the block
//! number at which the transaction will be retried.
//!
//! - **bridge limit**: The maximum number of bridged POLYX per identity within a set interval of
//! blocks.
//!
//! - **bridge limit exempted**: Identities not constrained by the bridge limit.
//!
//! ### Transaction State Transitions
//!
//! Although the bridge is not implemented as a state machine in the strict sense, the status of a
//! bridge transition can be viewed as its state in the abstract state machine diagram below:
//!
//! ```ignore
//!         +------------+      timelock == 0       +------------+
//!         |            |      happy path          |            |
//!         |   absent   +-------------------------->  handled   |
//!         |            +------------+             |            |
//!         +-----+--^---+   admin    |             +------^-----+
//!               |  |                |                    |
//!               |  |          +-----v------+             |
//! timelock != 0 |  | admin    |            |             |
//! or no CDD or  |  +----------+   frozen   |             | happy path
//! limit reached |             |            |             |
//!               |             +----^-^-----+             |
//!               |                  | |                   |
//!         +-----v------+   admin   | |   admin    +------+-----+
//!         |            +-----------+ +------------+            <-----+
//!         | timelocked +-------------------------->  pending   |     |retry
//!         |            |    timelock expired      |            +-----+
//!         +------------+                          +------------+
//! ```
//!
//! **Absent** is the initial state. **Handled** is the final state. Note that there is a feature
//! allowing the admin to introduce new transactions by freezing them since there is an admin
//! transition from **absent** to **frozen**.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `change_controller`: Changes the controller account as admin.
//! - `change_admin`: Changes the bridge admin key.
//! - `change_timelock`: Changes the timelock period.
//! - `freeze`: Freezes transaction handling in the bridge module if it is not already frozen.
//! - `unfreeze`: Unfreezes transaction handling in the bridge module if it is frozen.
//! - `change_bridge_limit`: Changes the bridge limits.
//! - `change_bridge_exempted`: Changes the bridge limit exempted.
//! - `force_handle_bridge_tx`: Forces handling a transaction by bypassing the bridge limit and
//! timelock.
//! - `batch_propose_bridge_tx`: Proposes a vector of bridge transactions.
//! - `propose_bridge_tx`: Proposes a bridge transaction, which amounts to making a multisig.
//! - `handle_bridge_tx`: Handles an approved bridge transaction proposal.
//! - `freeze_txs`: Freezes given bridge transactions.
//! - `unfreeze_txs`: Unfreezes given bridge transactions.
//! - `add_freeze_admin`: Add a freeze admin.
//! - `remove_freeze_admin`: Remove a freeze admin.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_option)]

mod migration;

#[cfg(feature = "std")]
mod genesis;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure, fail,
    storage::StorageDoubleMap,
    traits::{
        schedule::{Anon as ScheduleAnon, DispatchTime, LOWEST_PRIORITY},
        Currency,
    },
    weights::{DispatchClass, Pays, Weight},
};
use frame_system::{self as system, ensure_root, ensure_signed, RawOrigin};
use pallet_balances as balances;
use pallet_identity as identity;
use pallet_multisig as multisig;
use polymesh_common_utilities::traits::balances::Config as BalancesConfig;
use polymesh_common_utilities::{
    traits::{balances::CheckCdd, identity::Config as IdentityConfig},
    Context, GC_DID,
};
use polymesh_primitives::{storage_migration_ver, Balance, IdentityId, Signatory};
use sp_core::H256;
use sp_runtime::traits::{Saturating, Zero};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::TryFrom, fmt::Debug, prelude::*};

type Identity<T> = identity::Module<T>;

pub trait Config: multisig::Config + BalancesConfig + pallet_base::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Proposal: From<Call<Self>> + Into<<Self as IdentityConfig>::Proposal>;
    /// Scheduler of timelocked bridge transactions.
    type Scheduler: ScheduleAnon<
        Self::BlockNumber,
        <Self as Config>::Proposal,
        Self::SchedulerOrigin,
    >;
}

/// The status of a bridge transaction.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BridgeTxStatus {
    /// No such transaction in the system.
    Absent,
    /// The transaction is missing a CDD or the bridge module is frozen.  The `u8` parameter is the
    /// capped number of times the module tried processing this transaction.  It will be retried
    /// automatically. Anyone can retry these manually.
    Pending(u8),
    /// The transaction is frozen by the admin. It will not be retried automatically.
    Frozen,
    /// The transaction is pending its first execution. These can not be manually triggered by
    /// normal accounts.
    Timelocked,
    /// The transaction has been successfully credited.
    Handled,
}

impl Default for BridgeTxStatus {
    fn default() -> Self {
        BridgeTxStatus::Absent
    }
}

/// A unique lock-and-mint bridge transaction.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BridgeTx<Account> {
    /// A single transaction hash can have multiple locks. This nonce differentiates between them.
    pub nonce: u32,
    /// The recipient account of POLYX on Polymesh.
    pub recipient: Account,
    /// Amount of POLYX tokens to credit.
    pub amount: Balance,
    /// Ethereum token lock transaction hash. It is not used internally in the bridge and is kept
    /// here for compatibility reasons only.
    pub tx_hash: H256,
}

/// Additional details of a bridge transaction.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BridgeTxDetail<BlockNumber> {
    /// Amount of POLYX tokens to credit.
    pub amount: Balance,
    /// Status of the bridge transaction.
    pub status: BridgeTxStatus,
    /// Block number at which this transaction was executed or is planned to be executed.
    pub execution_block: BlockNumber,
    /// Ethereum token lock transaction hash. It is not used internally in the bridge and is kept
    /// here for compatibility reasons only.
    pub tx_hash: H256,
}

/// The status of a handled transaction for reporting purposes.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandledTxStatus {
    /// The transaction has been successfully handled.
    Success,
    /// Handling the transaction has failed, with the encoding of the error.
    Error(Vec<u8>),
}

impl<T, E: Encode> From<Result<T, E>> for HandledTxStatus {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(_) => Self::Success,
            Err(e) => Self::Error(e.encode()),
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The bridge controller address is not set.
        ControllerNotSet,
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
        /// The identity's minted total has reached the bridge limit.
        BridgeLimitReached,
        /// The identity's minted total has overflowed.
        Overflow,
        /// The block interval duration is zero. Cannot divide.
        DivisionByZero,
        /// The transaction is timelocked.
        TimelockedTx,
    }
}

// A value placed in storage that represents the current version of the this storage. This value
// is used by the `on_runtime_upgrade` logic to determine whether we run storage migration logic.
storage_migration_ver!(1);

decl_storage! {
    trait Store for Module<T: Config> as Bridge {
        /// The multisig account of the bridge controller. The genesis signers accept their
        /// authorizations and are able to get their proposals delivered. The bridge creator
        /// transfers some POLY to their identity.
        Controller get(fn controller) build(genesis::controller): T::AccountId;

        /// Details of bridge transactions identified with pairs of the recipient account and the
        /// bridge transaction nonce.
        pub BridgeTxDetails get(fn bridge_tx_details) build(genesis::bridge_tx_details): double_map
                hasher(blake2_128_concat) T::AccountId,
                hasher(blake2_128_concat) u32
            =>
                BridgeTxDetail<T::BlockNumber>;

        /// The admin key.
        Admin get(fn admin) config(): T::AccountId;

        /// Whether or not the bridge operation is frozen.
        Frozen get(fn frozen): bool;

        /// Freeze bridge admins.  These accounts can only freeze the bridge.
        FreezeAdmins get(fn freeze_admins): map hasher(blake2_128_concat) T::AccountId => bool;

        /// The bridge transaction timelock period, in blocks, since the acceptance of the
        /// transaction proposal during which the admin key can freeze the transaction.
        Timelock get(fn timelock) config(): T::BlockNumber;

        /// The maximum number of bridged POLYX per identity within a set interval of
        /// blocks. Fields: POLYX amount and the block interval duration.
        BridgeLimit get(fn bridge_limit) config(): (Balance, T::BlockNumber);

        /// Amount of POLYX bridged by the identity in last block interval. Fields: the bridged
        /// amount and the last interval number.
        PolyxBridged get(fn polyx_bridged): map hasher(twox_64_concat) IdentityId => (Balance, T::BlockNumber);

        /// Identities not constrained by the bridge limit.
        BridgeLimitExempted get(fn bridge_exempted): map hasher(twox_64_concat) IdentityId => bool;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1).unwrap()): Version;
    }
    add_extra_genesis {
        /// AccountId of the multisig creator.
        config(creator): T::AccountId;
        /// The set of initial signers from which a multisig address is created at genesis time.
        config(signers): Vec<Signatory<T::AccountId>>;
        /// The number of required signatures in the genesis signer set.
        config(signatures_required): u64;
        /// Complete transactions at genesis.
        config(complete_txs): Vec<BridgeTx<T::AccountId>>;
    }
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        BlockNumber = <T as frame_system::Config>::BlockNumber,
    {
        /// Confirmation of a signer set change.
        ControllerChanged(IdentityId, AccountId),
        /// Confirmation of Admin change.
        AdminChanged(IdentityId, AccountId),
        /// Confirmation of default timelock change.
        TimelockChanged(IdentityId, BlockNumber),
        /// Confirmation of POLYX upgrade on Polymesh from POLY tokens on Ethereum.
        Bridged(IdentityId, BridgeTx<AccountId>),
        /// Notification of freezing the bridge.
        Frozen(IdentityId),
        /// Notification of unfreezing the bridge.
        Unfrozen(IdentityId),
        /// Notification of freezing a transaction.
        FrozenTx(IdentityId, BridgeTx<AccountId>),
        /// Notification of unfreezing a transaction.
        UnfrozenTx(IdentityId, BridgeTx<AccountId>),
        /// Exemption status of an identity has been updated.
        ExemptedUpdated(IdentityId, IdentityId, bool),
        /// Bridge limit has been updated.
        BridgeLimitUpdated(IdentityId, Balance, BlockNumber),
        /// An event emitted after a vector of transactions is handled. The parameter is a vector of
        /// tuples of recipient account, its nonce, and the status of the processed transaction.
        TxsHandled(Vec<(AccountId, u32, HandledTxStatus)>),
        /// Bridge Tx Scheduled.
        BridgeTxScheduled(IdentityId, BridgeTx<AccountId>, BlockNumber),
        /// Failed to schedule Bridge Tx.
        BridgeTxScheduleFailed(IdentityId, BridgeTx<AccountId>, Vec<u8>),
        /// A new freeze admin has been added.
        FreezeAdminAdded(IdentityId, AccountId),
        /// A freeze admin has been removed.
        FreezeAdminRemoved(IdentityId, AccountId),
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        fn on_runtime_upgrade() -> Weight {
            migration::on_runtime_upgrade::<T>()
        }

        /// Changes the controller account as admin.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        #[weight = (300_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn change_controller(origin, controller: T::AccountId) -> DispatchResult {
            Self::base_change_controller(origin, controller)
        }

        /// Changes the bridge admin key.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        #[weight = (300_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn change_admin(origin, admin: T::AccountId) -> DispatchResult {
            Self::base_change_admin(origin, admin)
        }

        /// Changes the timelock period.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        #[weight = (300_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn change_timelock(origin, timelock: T::BlockNumber) -> DispatchResult {
            Self::base_change_timelock(origin, timelock)
        }

        /// Freezes transaction handling in the bridge module if it is not already frozen. When the
        /// bridge is frozen, attempted transactions get postponed instead of getting handled.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        #[weight = (300_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn freeze(origin) -> DispatchResult {
            Self::set_freeze(origin, true)
        }

        /// Unfreezes transaction handling in the bridge module if it is frozen.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        #[weight = (300_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn unfreeze(origin) -> DispatchResult {
            Self::set_freeze(origin, false)
        }

        /// Changes the bridge limits.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        /// - `DivisionByZero` if `duration` is zero.
        #[weight = (500_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn change_bridge_limit(origin, amount: Balance, duration: T::BlockNumber) -> DispatchResult {
            Self::base_change_bridge_limit(origin, amount, duration)
        }

        /// Changes the bridge limit exempted list.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        #[weight = (500_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn change_bridge_exempted(origin, exempted: Vec<(IdentityId, bool)>) -> DispatchResult {
            Self::base_change_bridge_exempted(origin, exempted)
        }

        /// Forces handling a transaction by bypassing the bridge limit and timelock.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        /// - `NoValidCdd` if `bridge_tx.recipient` does not have a valid CDD claim.
        #[weight = (600_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn force_handle_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId>) -> DispatchResult {
            Self::base_force_handle_bridge_tx(origin, bridge_tx)
        }

        /// Proposes a vector of bridge transactions. The vector is processed until the first
        /// proposal which causes an error, in which case the error is returned and the rest of
        /// proposals are not processed.
        ///
        /// ## Errors
        /// - `ControllerNotSet` if `Controllers` was not set.
        ///
        /// # Weight
        /// `500_000_000 + 7_000_000 * bridge_txs.len()`
        #[weight =(
            500_000_000 + 7_000_000 * u64::try_from(bridge_txs.len()).unwrap_or_default(),
            DispatchClass::Operational,
            Pays::Yes
        )]
        pub fn batch_propose_bridge_tx(origin, bridge_txs: Vec<BridgeTx<T::AccountId>>) ->
            DispatchResult
        {
            Self::base_batch_propose_bridge_tx(origin, bridge_txs, true)
        }

        /// Proposes a bridge transaction, which amounts to making a multisig proposal for the
        /// bridge transaction if the transaction is new or approving an existing proposal if the
        /// transaction has already been proposed.
        ///
        /// ## Errors
        /// - `ControllerNotSet` if `Controllers` was not set.
        #[weight = (500_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn propose_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId>) ->
            DispatchResult
        {
            Self::base_batch_propose_bridge_tx(origin, vec![bridge_tx], false)
        }

        /// Handles an approved bridge transaction proposal.
        ///
        /// ## Errors
        /// - `BadCaller` if `origin` is not `Self::controller` or  `Self::admin`.
        /// - `TimelockedTx` if the transaction status is `Timelocked`.
        /// - `ProposalAlreadyHandled` if the transaction status is `Handled`.
        #[weight = (900_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn handle_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId>) ->
            DispatchResult
        {
            Self::base_handle_bridge_tx(origin, bridge_tx)
        }

        /// Freezes given bridge transactions.
        /// If any bridge txn is already handled then this function will just ignore it and process next one.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        ///
        /// # Weight
        /// `400_000_000 + 2_000_000 * bridge_txs.len()`
        #[weight = (
            400_000_000 + 2_000_000 * u64::try_from(bridge_txs.len()).unwrap_or_default(),
            DispatchClass::Operational,
            Pays::Yes
        )]
        pub fn freeze_txs(origin, bridge_txs: Vec<BridgeTx<T::AccountId>>) -> DispatchResult {
            Self::base_freeze_txs(origin, bridge_txs)
        }

        /// Unfreezes given bridge transactions.
        /// If any bridge txn is already handled then this function will just ignore it and process next one.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        ///
        /// # Weight
        /// `400_000_000 + 7_000_000 * bridge_txs.len()`
        #[weight = (
            400_000_000 + 7_000_000 * u64::try_from(bridge_txs.len()).unwrap_or_default(),
            DispatchClass::Operational,
            Pays::Yes
        )]
        pub fn unfreeze_txs(origin, bridge_txs: Vec<BridgeTx<T::AccountId>>) -> DispatchResult {
            Self::base_unfreeze_txs(origin, bridge_txs)
        }

        /// Root callable extrinsic, used as an internal call to handle a scheduled timelocked bridge transaction.
        ///
        /// # Errors
        /// - `BadOrigin` if `origin` is not root.
        /// - `ProposalAlreadyHandled` if transaction status is `Handled`.
        /// - `FrozenTx` if transaction status is `Frozen`.
        #[weight = (
            500_000_000,
            DispatchClass::Operational,
            Pays::Yes
        )]
        fn handle_scheduled_bridge_tx(origin, bridge_tx: BridgeTx<T::AccountId>) {
            Self::base_handle_scheduled_bridge_tx(origin, bridge_tx)?;
        }

        /// Add a freeze admin.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        #[weight = (300_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn add_freeze_admin(origin, freeze_admin: T::AccountId) -> DispatchResult {
            Self::base_add_freeze_admin(origin, freeze_admin)
        }

        /// Remove a freeze admin.
        ///
        /// ## Errors
        /// - `BadAdmin` if `origin` is not `Self::admin()` account.
        #[weight = (300_000_000, DispatchClass::Operational, Pays::Yes)]
        pub fn remove_freeze_admin(origin, freeze_admin: T::AccountId) -> DispatchResult {
            Self::base_remove_freeze_admin(origin, freeze_admin)
        }
    }
}

impl<T: Config> Module<T> {
    pub fn controller_key() -> T::AccountId {
        Self::controller()
    }

    fn ensure_admin_did(origin: T::Origin) -> Result<IdentityId, DispatchError> {
        let sender = Self::ensure_admin(origin)?;
        Context::current_identity_or::<Identity<T>>(&sender)
    }

    fn ensure_admin(origin: T::Origin) -> Result<T::AccountId, DispatchError> {
        let sender = ensure_signed(origin)?;
        ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
        Ok(sender)
    }

    fn ensure_freeze_admin_did(origin: T::Origin) -> Result<IdentityId, DispatchError> {
        let sender = ensure_signed(origin)?;
        if !<FreezeAdmins<T>>::get(&sender) {
            // Not a freeze admin, check if they are the main admin.
            ensure!(sender == Self::admin(), Error::<T>::BadAdmin);
        }
        Context::current_identity_or::<Identity<T>>(&sender)
    }

    fn ensure_controller_set() -> DispatchResult {
        ensure!(
            Self::controller() != Default::default(),
            Error::<T>::ControllerNotSet
        );
        Ok(())
    }

    fn get_tx_details(tx: &BridgeTx<T::AccountId>) -> BridgeTxDetail<T::BlockNumber> {
        Self::bridge_tx_details(&tx.recipient, &tx.nonce)
    }

    /// Issues the transacted amount to the recipient.
    fn issue(
        recipient: &T::AccountId,
        amount: &Balance,
        exempted_did: Option<IdentityId>,
    ) -> DispatchResult {
        let did = exempted_did
            .or_else(|| T::CddChecker::get_key_cdd_did(&recipient))
            .ok_or(Error::<T>::NoValidCdd)?;
        let is_exempted = exempted_did.is_some() || Self::bridge_exempted(did);

        if !is_exempted {
            let (limit, interval_duration) = Self::bridge_limit();
            ensure!(!interval_duration.is_zero(), Error::<T>::DivisionByZero);

            let current_interval = <system::Module<T>>::block_number() / interval_duration;
            let (bridged, last_interval) = Self::polyx_bridged(did);
            let total_mint = if last_interval == current_interval {
                amount.checked_add(bridged).ok_or(Error::<T>::Overflow)?
            } else {
                *amount
            };
            ensure!(total_mint <= limit, Error::<T>::BridgeLimitReached);
            <PolyxBridged<T>>::insert(did, (total_mint, current_interval))
        }

        let _pos_imbalance = <balances::Module<T>>::deposit_creating(&recipient, *amount);

        Ok(())
    }

    /// Handles a bridge transaction proposal immediately.
    fn handle_bridge_tx_now(
        bridge_tx: BridgeTx<T::AccountId>,
        mut tx_details: BridgeTxDetail<T::BlockNumber>,
        untrusted_manual_retry: bool,
        exempted_did: Option<IdentityId>,
    ) -> DispatchResult {
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
            // Un-trusted manual retries not allowed during frozen state.
            ensure!(!untrusted_manual_retry, Error::<T>::Frozen);
            // Bridge module frozen. Retry this tx again later.
            return Self::handle_bridge_tx_later(bridge_tx, tx_details, Self::timelock());
        }

        let amount = if untrusted_manual_retry {
            // NB: The amount should be fetched from storage since the amount in `bridge_tx`
            // may be altered in a manual retry.
            tx_details.amount
        } else {
            bridge_tx.amount
        };

        // Try to handle the transaction.
        match Self::issue(&bridge_tx.recipient, &amount, exempted_did) {
            Ok(_) => {
                tx_details.status = BridgeTxStatus::Handled;
                tx_details.execution_block = <system::Module<T>>::block_number();
                <BridgeTxDetails<T>>::insert(&bridge_tx.recipient, &bridge_tx.nonce, tx_details);
                let current_did = Context::current_identity::<Identity<T>>().unwrap_or_else(|| GC_DID);
                Self::deposit_event(RawEvent::Bridged(current_did, bridge_tx));
                Ok(())
            }
            Err(e) => {
                // NB: If this was a manual retry, tx's automated retry schedule is not updated.
                if untrusted_manual_retry {
                    return Err(e);
                }
                // Recipient missing CDD or limit reached. Retry this tx again later.
                if let Err(sched_e) =
                    Self::handle_bridge_tx_later(bridge_tx.clone(), tx_details, Self::timelock())
                {
                    // Report scheduling error as an event.
                    let current_did =
                        Context::current_identity::<Identity<T>>().unwrap_or_else(|| GC_DID);
                    Self::deposit_event(RawEvent::BridgeTxScheduleFailed(
                        current_did,
                        bridge_tx,
                        sched_e.encode(),
                    ));
                }
                Err(e)
            }
        }
    }

    /// Handles a bridge transaction proposal after `timelock` blocks.
    fn handle_bridge_tx_later(
        bridge_tx: BridgeTx<T::AccountId>,
        mut tx_details: BridgeTxDetail<T::BlockNumber>,
        timelock: T::BlockNumber,
    ) -> DispatchResult {
        let mut already_tried = 0;
        match tx_details.status {
            BridgeTxStatus::Absent => {
                tx_details.status = BridgeTxStatus::Timelocked;
                tx_details.amount = bridge_tx.amount;
            }
            BridgeTxStatus::Pending(x) => {
                tx_details.status = BridgeTxStatus::Pending(x + 1);
                already_tried = x + 1;
            }
            BridgeTxStatus::Timelocked => {
                tx_details.status = BridgeTxStatus::Pending(1);
                already_tried = 1;
            }
            BridgeTxStatus::Frozen => fail!(Error::<T>::FrozenTx),
            BridgeTxStatus::Handled => fail!(Error::<T>::ProposalAlreadyHandled),
        }
        tx_details.tx_hash = bridge_tx.tx_hash;

        if already_tried > 24 {
            // Limits the exponential backoff to 2^24 blocks (about 3 years).
            already_tried = 24;
        }

        let unlock_block_number = <system::Module<T>>::block_number()
            .saturating_add(timelock)
            .saturating_add(T::BlockNumber::from(2u32.pow(already_tried.into())));
        tx_details.execution_block = unlock_block_number;

        // Schedule next retry.
        Self::schedule_call(unlock_block_number, bridge_tx.clone())?;

        // Update transaction details.
        <BridgeTxDetails<T>>::insert(&bridge_tx.recipient, &bridge_tx.nonce, tx_details);

        Ok(())
    }

    /// Proposes a vector of bridge transaction. The bridge controller must be set.
    fn base_batch_propose_bridge_tx(
        origin: T::Origin,
        bridge_txs: Vec<BridgeTx<T::AccountId>>,
        send_event: bool,
    ) -> DispatchResult {
        let sender = ensure_signed(origin)?;
        Self::ensure_controller_set()?;

        let sender_signer = Signatory::Account(sender);
        let propose = |tx| {
            let proposal = <T as Config>::Proposal::from(Call::<T>::handle_bridge_tx(tx));
            let boxed_proposal = Box::new(proposal.into());
            <multisig::Module<T>>::create_or_approve_proposal(
                Self::controller(),
                sender_signer.clone(),
                boxed_proposal,
                None,
                true,
            )
        };
        let txs_result = Self::apply_handler(propose, bridge_txs);
        if send_event {
            Self::deposit_event(RawEvent::TxsHandled(txs_result));
        }
        Ok(())
    }

    /// Handles an approved bridge transaction proposal.
    fn base_handle_bridge_tx(
        origin: T::Origin,
        bridge_tx: BridgeTx<T::AccountId>,
    ) -> DispatchResult {
        let sender = ensure_signed(origin)?;
        let ensure_caller = || -> DispatchResult {
            ensure!(
                sender == Self::controller() || sender == Self::admin(),
                Error::<T>::BadCaller
            );
            Ok(())
        };

        let mut tx_details = Self::get_tx_details(&bridge_tx);
        match tx_details.status {
            // New bridge tx.
            BridgeTxStatus::Absent => {
                // Ensure the caller is either the admin or controller.
                ensure_caller()?;
                let timelock = Self::timelock();
                if timelock.is_zero() {
                    Self::handle_bridge_tx_now(bridge_tx, tx_details, false, None)
                } else {
                    Self::handle_bridge_tx_later(bridge_tx, tx_details, timelock)
                }
            }
            // Pending cdd bridge tx.
            BridgeTxStatus::Pending(_) => {
                // NB: Anyone can retry a `Pending` transaction.
                Self::handle_bridge_tx_now(bridge_tx, tx_details, true, None)
            }
            // Pre frozen tx. We just set the correct amount.
            BridgeTxStatus::Frozen => {
                // Ensure the caller is either the admin or controller.
                ensure_caller()?;
                tx_details.amount = bridge_tx.amount;
                <BridgeTxDetails<T>>::insert(&bridge_tx.recipient, &bridge_tx.nonce, tx_details);
                Ok(())
            }
            BridgeTxStatus::Timelocked => fail!(Error::<T>::TimelockedTx),
            BridgeTxStatus::Handled => fail!(Error::<T>::ProposalAlreadyHandled),
        }
    }

    /// Forces handling a transaction by bypassing the bridge limit and timelock.
    fn base_force_handle_bridge_tx(
        origin: T::Origin,
        bridge_tx: BridgeTx<T::AccountId>,
    ) -> DispatchResult {
        Self::ensure_admin(origin)?;
        // NB: To avoid code duplication, this uses a hacky approach of temporarily exempting the did.
        let exempted_did =
            T::CddChecker::get_key_cdd_did(&bridge_tx.recipient).ok_or(Error::<T>::NoValidCdd)?;
        let tx_details = Self::get_tx_details(&bridge_tx);
        Self::handle_bridge_tx_now(bridge_tx, tx_details, false, Some(exempted_did))
    }

    /// Applies a handler `f` to a vector of transactions `bridge_txs` and outputs a vector of
    /// processing results.
    fn apply_handler(
        f: impl Fn(BridgeTx<T::AccountId>) -> DispatchResult,
        bridge_txs: Vec<BridgeTx<T::AccountId>>,
    ) -> Vec<(T::AccountId, u32, HandledTxStatus)> {
        bridge_txs
            .into_iter()
            .map(|tx: BridgeTx<_>| (tx.recipient.clone(), tx.nonce, f(tx).into()))
            .collect()
    }

    /// Schedules a timelocked transaction call with constant arguments and emits an event on success or
    /// prints an error message on failure.
    fn schedule_call(
        block_number: T::BlockNumber,
        bridge_tx: BridgeTx<T::AccountId>,
    ) -> DispatchResult {
        // Schedule the transaction as a dispatchable call.
        let call = Call::<T>::handle_scheduled_bridge_tx(bridge_tx.clone()).into();
        <T as Config>::Scheduler::schedule(
            DispatchTime::At(block_number),
            None,
            LOWEST_PRIORITY,
            RawOrigin::Root.into(),
            call,
        )?;
        let current_did = Context::current_identity::<Identity<T>>().unwrap_or_else(|| GC_DID);
        Self::deposit_event(RawEvent::BridgeTxScheduled(
            current_did,
            bridge_tx,
            block_number,
        ));
        Ok(())
    }

    fn base_handle_scheduled_bridge_tx(
        origin: T::Origin,
        bridge_tx: BridgeTx<T::AccountId>,
    ) -> DispatchResult {
        ensure_root(origin)?;
        let tx_details = Self::get_tx_details(&bridge_tx);
        Self::handle_bridge_tx_now(bridge_tx, tx_details, false, None)
    }

    fn base_change_controller(origin: T::Origin, controller: T::AccountId) -> DispatchResult {
        let did = Self::ensure_admin_did(origin)?;
        <Controller<T>>::put(controller.clone());
        Self::deposit_event(RawEvent::ControllerChanged(did, controller));
        Ok(())
    }

    fn base_change_admin(origin: T::Origin, admin: T::AccountId) -> DispatchResult {
        let did = Self::ensure_admin_did(origin)?;
        <Admin<T>>::put(admin.clone());
        Self::deposit_event(RawEvent::AdminChanged(did, admin));
        Ok(())
    }

    fn base_change_timelock(origin: T::Origin, timelock: T::BlockNumber) -> DispatchResult {
        let did = Self::ensure_admin_did(origin)?;
        <Timelock<T>>::put(timelock);
        Self::deposit_event(RawEvent::TimelockChanged(did, timelock));
        Ok(())
    }

    fn base_add_freeze_admin(origin: T::Origin, freeze_admin: T::AccountId) -> DispatchResult {
        let did = Self::ensure_admin_did(origin)?;
        <FreezeAdmins<T>>::insert(freeze_admin.clone(), true);
        Self::deposit_event(RawEvent::FreezeAdminAdded(did, freeze_admin));
        Ok(())
    }

    fn base_remove_freeze_admin(origin: T::Origin, freeze_admin: T::AccountId) -> DispatchResult {
        let did = Self::ensure_admin_did(origin)?;
        <FreezeAdmins<T>>::remove(freeze_admin.clone());
        Self::deposit_event(RawEvent::FreezeAdminRemoved(did, freeze_admin));
        Ok(())
    }

    fn set_freeze(origin: T::Origin, freeze: bool) -> DispatchResult {
        let did = if freeze {
            Self::ensure_freeze_admin_did(origin)?
        } else {
            Self::ensure_admin_did(origin)?
        };

        let (event, error) = match freeze {
            true => (RawEvent::Frozen(did), Error::<T>::Frozen),
            false => (RawEvent::Unfrozen(did), Error::<T>::NotFrozen),
        };
        ensure!(Self::frozen() != freeze, error);

        Frozen::put(freeze);
        Self::deposit_event(event);

        Ok(())
    }

    fn base_change_bridge_limit(
        origin: T::Origin,
        amount: Balance,
        duration: T::BlockNumber,
    ) -> DispatchResult {
        let did = Self::ensure_admin_did(origin)?;
        // Don't allow `duration` to equal zero.
        ensure!(!duration.is_zero(), Error::<T>::DivisionByZero);

        <BridgeLimit<T>>::put((amount, duration));
        Self::deposit_event(RawEvent::BridgeLimitUpdated(did, amount, duration));
        Ok(())
    }

    fn base_change_bridge_exempted(
        origin: T::Origin,
        exempted: Vec<(IdentityId, bool)>,
    ) -> DispatchResult {
        let did = Self::ensure_admin_did(origin)?;
        for (exempt_did, exempt) in exempted {
            BridgeLimitExempted::insert(exempt_did, exempt);
            Self::deposit_event(RawEvent::ExemptedUpdated(did, exempt_did, exempt));
        }
        Ok(())
    }

    fn base_freeze_txs(
        origin: T::Origin,
        bridge_txs: Vec<BridgeTx<T::AccountId>>,
    ) -> DispatchResult {
        let did = Self::ensure_admin_did(origin)?;
        bridge_txs
            .into_iter()
            .filter_map(|tx| {
                let tx_details = Self::get_tx_details(&tx);
                if tx_details.status != BridgeTxStatus::Handled {
                    Some((tx, tx_details))
                } else {
                    None
                }
            })
            .for_each(|(tx, mut tx_details)| {
                tx_details.status = BridgeTxStatus::Frozen;
                <BridgeTxDetails<T>>::insert(&tx.recipient, &tx.nonce, tx_details);
                Self::deposit_event(RawEvent::FrozenTx(did, tx));
            });
        Ok(())
    }

    fn base_unfreeze_txs(
        origin: T::Origin,
        bridge_txs: Vec<BridgeTx<T::AccountId>>,
    ) -> DispatchResult {
        // NB: An admin can call Freeze + Unfreeze on a transaction to bypass the timelock.
        let did = Self::ensure_admin_did(origin)?;
        let txs_result = bridge_txs
            .into_iter()
            .filter_map(|tx| {
                let tx_details = Self::get_tx_details(&tx);
                if tx_details.status == BridgeTxStatus::Frozen {
                    Some((tx, tx_details))
                } else {
                    None
                }
            })
            .map(|(tx, mut tx_details)| {
                tx_details.status = BridgeTxStatus::Absent;
                <BridgeTxDetails<T>>::insert(&tx.recipient, &tx.nonce, tx_details.clone());
                Self::deposit_event(RawEvent::UnfrozenTx(did, tx.clone()));
                let (recipient, nonce) = (tx.recipient.clone(), tx.nonce);
                let status = Self::handle_bridge_tx_now(tx, tx_details, true, None).into();
                (recipient, nonce, status)
            })
            .collect::<Vec<_>>();

        Self::deposit_event(RawEvent::TxsHandled(txs_result));
        Ok(())
    }
}
