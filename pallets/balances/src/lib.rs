// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Modified by Polymath Inc - 16th March 2020
// Implement `BlockRewardsReserveCurrency` trait in the balances module.
// Remove migration functionality from the balances module as Polymesh doesn't needed
// any migration data structure.

//! # Balances Pallet
//!
//! The Balances pallet provides functionality for handling accounts and balances.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! This is a modified implementation of substrate's balances FRAME.
//! The modifications made are as follows:
//!
//! - To curb front running, sending a tip along with your transaction is now prohibited in normal
//! transations.
//! - Added From<u128> trait to Balances type.
//! - Removed existential amount requirement to prevent a replay attack scenario.
//! - Added block rewards reserve that subsidize minting for block rewards.
//! - Added CDD check for POLYX recipients.
//! - Added ability to attach a memo with a transfer.
//! - Added ability to burn your tokens.
//!
//! The Original Balances pallet provides functions for:
//!
//! - Getting and setting free balances.
//! - Retrieving total, reserved and unreserved balances.
//! - Repatriating a reserved balance to a beneficiary account that exists.
//! - Transferring a balance between accounts (when not reserved).
//! - Slashing an account balance.
//! - Account creation and removal.
//! - Managing total issuance.
//! - Setting and managing locks.
//!
//! ### Terminology
//!
//! - **Total Issuance:** The total number of units in existence in a system.
//!
//! - **Reaping an account:** The act of removing an account by resetting its nonce. Happens after its
//! total balance has become zero.
//!
//! - **Free Balance:** The portion of a balance that is not reserved. The free balance is the only
//!   balance that matters for most operations.
//!
//! - **Reserved Balance:** Reserved balance still belongs to the account holder, but is suspended.
//!   Reserved balance can still be slashed, but only after all the free balance has been slashed.
//!
//! - **Imbalance:** A condition when some funds were credited or debited without equal and opposite accounting
//! (i.e. a difference between total issuance and account balances). Functions that result in an imbalance will
//! return an object of the `Imbalance` trait that can be managed within your runtime logic. (If an imbalance is
//! simply dropped, it should automatically maintain any book-keeping such as total issuance.)
//!
//! - **Lock:** A freeze on a specified amount of an account's free balance until a specified block number. Multiple
//! locks always operate over the same funds, so they "overlay" rather than "stack".
//! - **Vesting:** Similar to a lock, this is another, but independent, liquidity restriction that reduces linearly
//! over time.
//!
//! ### Implementations
//!
//! The Balances pallet provides implementations for the following traits. If these traits provide the functionality
//! that you need, then you can avoid coupling with the Balances pallet.
//!
//! - [`Currency`](frame_support::traits::Currency): Functions for dealing with a
//! fungible assets system.
//! - [`ReservableCurrency`](frame_support::traits::ReservableCurrency):
//! Functions for dealing with assets that can be reserved from an account.
//! - [`LockableCurrency`](frame_support::traits::LockableCurrency): Functions for
//! dealing with accounts that allow liquidity restrictions.
//! - [`Imbalance`](frame_support::traits::Imbalance): Functions for handling
//! imbalances between total issuance in the system and account balances. Must be used when a function
//! creates new funds (e.g. a reward) or destroys some funds (e.g. a system fee).
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `transfer` - Transfer some liquid free balance to another account.
//! - `transfer_with_memo` - Transfer some liquid free balance to another account alon with a memo.
//! - `set_balance` - Set the balances of a given account. The origin of this call must be root.
//! - `deposit_block_reward_reserve_balance` - Transfer some liquid free balance to block rewards reserve.
//! - `force_transfer` - Force transfer some balance from one account to another. The origin of this call must be root.
//! - `burn_account_balance` - Burn some liquid free balance.

//! ### Public Functions
//!
//! - `free_balance` - Get the free balance of an account.
//! - `usable_balance` - Get the balance of an account that can be used for transfers, reservations, or any other non-locking, non-transaction-fee activity.
//! - `usable_balance_for_fees` - Get the balance of an account that can be used for paying transaction fees (not tipping, or any other kind of fees, though).
//! - `reserved_balance` - Get the reserved balance of an account.
//! ## Usage
//!
//! The following examples show how to use the Balances pallet in your custom pallet.
//!
//! ### Examples from the FRAME
//!
//! The Contract pallet uses the `Currency` trait to handle gas payment, and its types inherit from `Currency`:
//!
//! ```
//! use frame_support::traits::Currency;
//! # pub trait Config: frame_system::Config {
//! # 	type Currency: Currency<Self::AccountId>;
//! # }
//!
//! pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
//! pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
//!
//! # fn main() {}
//! ```
//!
//! The Staking pallet uses the `LockableCurrency` trait to lock a stash account's funds:
//!
//! ```
//! use frame_support::traits::{WithdrawReasons, LockableCurrency};
//! use sp_runtime::traits::Bounded;
//! pub trait Config: frame_system::Config {
//! 	type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
//! }
//! # struct StakingLedger<T: Config> {
//! # 	stash: <T as frame_system::Config>::AccountId,
//! # 	total: <<T as Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance,
//! # 	phantom: std::marker::PhantomData<T>,
//! # }
//! # const STAKING_ID: [u8; 8] = *b"staking ";
//!
//! fn update_ledger<T: Config>(
//! 	controller: &T::AccountId,
//! 	ledger: &StakingLedger<T>
//! ) {
//! 	T::Currency::set_lock(
//! 		STAKING_ID,
//! 		&ledger.stash,
//! 		ledger.total,
//! 		WithdrawReasons::all()
//! 	);
//! 	// <Ledger<T>>::insert(controller, ledger); // Commented out as we don't have access to Staking's storage here.
//! }
//! # fn main() {}
//! ```
//!
//! ## Genesis config
//!
//! The Balances pallet depends on the [`GenesisConfig`].
//!
//! ## Assumptions
//!
//! * Total issued balanced of all accounts should be less than `Config::Balance::max_value()`.

// TODO: Benchmark modified extrinsics. Currently using Substrate values based on non-modified code.
// Specifically CDD checks should be considered!

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{
        BalanceStatus as Status, Currency, ExistenceRequirement, Get, Imbalance, LockIdentifier,
        LockableCurrency, OnUnbalanced, ReservableCurrency, SignedImbalance, StoredMap,
        WithdrawReasons,
    },
};
use frame_system::{self as system, ensure_root, ensure_signed, pallet_prelude::*};
pub use polymesh_common_utilities::traits::balances::WeightInfo;
use polymesh_common_utilities::{
    traits::{
        balances::{AccountData, BalancesTrait, CheckCdd, LockableCurrencyExt, Memo, Reasons},
        identity::{IdentityFnTrait, Trait as IdentityTrait},
        NegativeImbalance, PositiveImbalance,
    },
    Context, SystematicIssuers, GC_DID,
};
use polymesh_primitives::{traits::BlockRewardsReserveCurrency, IdentityId};
use sp_runtime::{
    traits::{
        AccountIdConversion, Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize,
        Saturating, StaticLookup, StoredMapError, Zero,
    },
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::prelude::*;
use sp_std::{cmp, fmt::Debug, mem, result};

// pub type Event<T> = polymesh_common_utilities::traits::balances::Event<T>;
type CallPermissions<T> = pallet_permissions::Module<T>;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + IdentityTrait {
        /// The balance of an account.
        /*type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;*/

        /// Handler for the unbalanced reduction when removing a dust account.
        type DustRemoval: OnUnbalanced<NegativeImbalance<Self>>;

        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The minimum amount required to keep an account open.
        #[pallet::constant]
        type ExistentialDeposit: Get<Self::Balance>;

        /// The means of storing the balances of an account.
        type AccountStore: StoredMap<Self::AccountId, AccountData<Self::Balance>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The maximum number of locks that should exist on an account.
        /// Not strictly enforced, but used for weight estimation.
        type MaxLocks: Get<u32>;

        /// Used to check if an account is linked to a CDD'd identity
        type CddChecker: CheckCdd<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Transfer some liquid free balance to another account.
        ///
        /// `transfer` will set the `FreeBalance` of the sender and receiver.
        /// It will decrease the total issuance of the system by the `TransferFee`.
        ///
        /// The dispatch origin for this call must be `Signed` by the transactor.
        ///
        /// # <weight>
        /// - Dependent on arguments but not critical, given proper implementations for
        ///   input config types. See related functions below.
        /// - It contains a limited number of reads and writes internally and no complex computation.
        ///
        /// Related functions:
        ///
        ///   - `ensure_can_withdraw` is always called internally but has a bounded complexity.
        ///   - Transferring balances to accounts that did not exist before will cause
        ///      `T::OnNewAccount::on_new_account` to be called.
        /// ---------------------------------
        /// - Base Weight: 73.64 µs, worst case scenario (account created, account removed)
        /// - DB Weight: 1 Read and 1 Write to destination account
        /// - Origin account is already in memory, so no DB operations for them.
        /// # </weight>
        #[pallet::weight(<T as Config>::WeightInfo::transfer())]
        pub fn transfer(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let transactor = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;
            // Polymesh modified code. CDD is checked before processing transfer.
            Self::safe_transfer_core(
                &transactor,
                &dest,
                value,
                None,
                ExistenceRequirement::AllowDeath,
            )
        }

        // Polymesh modified code. New function to transfer with a memo.
        /// Transfer the native currency with the help of identifier string
        /// this functionality can help to differentiate the transfers.
        ///
        /// # <weight>
        /// - Base Weight: 73.64 µs, worst case scenario (account created, account removed)
        /// - DB Weight: 1 Read and 1 Write to destination account.
        /// - Origin account is already in memory, so no DB operations for them.
        /// # </weight>
        #[pallet::weight(<T as Config>::WeightInfo::transfer_with_memo())]
        pub fn transfer_with_memo(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: T::Balance,
            memo: Option<Memo>,
        ) -> DispatchResultWithPostInfo {
            let transactor = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;
            Self::safe_transfer_core(
                &transactor,
                &dest,
                value,
                memo,
                ExistenceRequirement::AllowDeath,
            )
        }

        // Polymesh specific change. New function to transfer balance to BRR.
        /// Move some POLYX from balance of self to balance of BRR.
        #[pallet::weight(<T as Config>::WeightInfo::deposit_block_reward_reserve_balance())]
        pub fn deposit_block_reward_reserve_balance(
            origin: OriginFor<T>,
            #[pallet::compact] value: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let transactor = ensure_signed(origin)?;
            CallPermissions::<T>::ensure_call_permissions(&transactor)?;
            let dest = Self::block_rewards_reserve();
            Self::transfer_core(
                &transactor,
                &dest,
                value,
                None,
                ExistenceRequirement::AllowDeath,
            )
        }

        /// Set the balances of a given account.
        ///
        /// This will alter `FreeBalance` and `ReservedBalance` in storage. it will
        /// also decrease the total issuance of the system (`TotalIssuance`).
        ///
        /// The dispatch origin for this call is `root`.
        ///
        #[pallet::weight(<T as Config>::WeightInfo::set_balance())]
        pub(super) fn set_balance(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] new_free: T::Balance,
            #[pallet::compact] new_reserved: T::Balance,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let who = T::Lookup::lookup(who)?;
            let caller_id = Context::current_identity_or::<T::IdentityFn>(&who).unwrap_or(GC_DID);

            let (free, reserved) = Self::mutate_account(&who, |account| {
                if new_free > account.free {
                    mem::drop(PositiveImbalance::<T>::new(new_free - account.free));
                } else if new_free < account.free {
                    mem::drop(NegativeImbalance::<T>::new(account.free - new_free));
                }

                if new_reserved > account.reserved {
                    mem::drop(PositiveImbalance::<T>::new(new_reserved - account.reserved));
                } else if new_reserved < account.reserved {
                    mem::drop(NegativeImbalance::<T>::new(account.reserved - new_reserved));
                }

                account.free = new_free;
                account.reserved = new_reserved;

                (account.free, account.reserved)
            })?;
            Self::deposit_event(Event::BalanceSet(caller_id, who, free, reserved));
            Ok(().into())
        }

        /// Exactly as `transfer`, except the origin must be root and the source account may be
        /// specified.
        /// # <weight>
        /// - Same as transfer, but additional read and write because the source account is
        ///   not assumed to be in the overlay.
        /// # </weight>
        #[pallet::weight(<T as Config>::WeightInfo::force_transfer())]
        pub fn force_transfer(
            origin: OriginFor<T>,
            source: <T::Lookup as StaticLookup>::Source,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: T::Balance,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let source = T::Lookup::lookup(source)?;
            let dest = T::Lookup::lookup(dest)?;
            Self::transfer_core(
                &source,
                &dest,
                value,
                None,
                ExistenceRequirement::AllowDeath,
            )?;
            Ok(().into())
        }

        // Polymesh modified code. New dispatchable function that anyone can call to burn their balance.
        /// Burns the given amount of tokens from the caller's free, unlocked balance.
        #[pallet::weight(<T as Config>::WeightInfo::burn_account_balance())]
        pub fn burn_account_balance(
            origin: OriginFor<T>,
            amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            CallPermissions::<T>::ensure_call_permissions(&who)?;
            let caller_id = Context::current_identity_or::<T::IdentityFn>(&who)?;
            // Withdraw the account balance and burn the resulting imbalance by dropping it.
            let _ = <Self as Currency<T::AccountId>>::withdraw(
                &who,
                amount,
                // There is no specific "burn" reason in Substrate. However, if the caller is
                // allowed to transfer then they should also be allowed to burn.
                WithdrawReasons::TRANSFER.into(),
                ExistenceRequirement::AllowDeath,
            )?;
            Self::deposit_event(Event::AccountBalanceBurned(caller_id, who, amount));
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance")]
    pub enum Event<T: Config> {
        /// An account was created with some free balance. \[did, account, free_balance]
        Endowed(Option<IdentityId>, T::AccountId, T::Balance),
        /// Transfer succeeded (from_did, from, to_did, to, value, memo).
        Transfer(
            Option<IdentityId>,
            T::AccountId,
            Option<IdentityId>,
            T::AccountId,
            T::Balance,
            Option<Memo>,
        ),
        /// A balance was set by root (did, who, free, reserved).
        BalanceSet(IdentityId, T::AccountId, T::Balance, T::Balance),
        /// The account and the amount of unlocked balance of that account that was burned.
        /// (caller Id, caller account, amount)
        AccountBalanceBurned(IdentityId, T::AccountId, T::Balance),
        /// Some balance was reserved (moved from free to reserved). \[who, value]
        Reserved(T::AccountId, T::Balance),
        /// Some balance was unreserved (moved from reserved to free). \[who, value]
        Unreserved(T::AccountId, T::Balance),
        /// Some balance was moved from the reserve of the first account to the second account.
        /// Final argument indicates the destination balance type.
        /// \[from, to, balance, destination_status]
        ReserveRepatriated(T::AccountId, T::AccountId, T::Balance, Status),
    }

    /// Old name generated by `decl_event`.
    #[deprecated(note = "use `Event` instead")]
    pub type RawEvent<T> = Event<T>;

    #[pallet::error]
    pub enum Error<T> {
        /// Account liquidity restrictions prevent withdrawal
        LiquidityRestrictions,
        /// Got an overflow after adding
        Overflow,
        /// Balance too low to send value
        InsufficientBalance,
        /// Value too low to create account due to existential deposit
        ExistentialDeposit,
        /// Receiver does not have a valid CDD
        ReceiverCddMissing,
    }

    /// The total units issued in the system.
    #[pallet::storage]
    #[pallet::getter(fn total_issuance)]
    pub type TotalIssuance<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    /// Any liquidity locks on some account balances.
    /// NOTE: Should only be accessed when setting, changing and freeing a lock.
    #[pallet::storage]
    #[pallet::getter(fn locks)]
    pub type Locks<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<BalanceLock<T::Balance>>, ValueQuery>;

    /// Storage version of the pallet.
    ///
    /// This is set to v2.0.0 for new networks.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub balances: Vec<(T::AccountId, T::Balance)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                balances: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let total = self
                .balances
                .iter()
                .fold(Zero::zero(), |acc: T::Balance, &(_, n)| acc + n);
            <TotalIssuance<T>>::put(total);

            <StorageVersion<T>>::put(Releases::V2_0_0);

            // ensure no duplicates exist.
            let endowed_accounts = self
                .balances
                .iter()
                .map(|(x, _)| x)
                .cloned()
                .collect::<std::collections::BTreeSet<_>>();

            assert!(
                endowed_accounts.len() == self.balances.len(),
                "duplicate balances in genesis."
            );

            for &(ref who, free) in self.balances.iter() {
                assert!(T::AccountStore::insert(
                    who,
                    AccountData {
                        free,
                        ..Default::default()
                    }
                )
                .is_ok());
            }
        }
    }
}

/// A single lock on a balance. There can be many of these on an account and they "overlap", so the
/// same balance is frozen by multiple locks.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct BalanceLock<Balance> {
    /// An identifier for this lock. Only one lock may be in existence for each identifier.
    pub id: LockIdentifier,
    /// The amount which the free balance may not drop below when this lock is in effect.
    pub amount: Balance,
    /// If true, then the lock remains in effect even for payment of transaction fees.
    pub reasons: Reasons,
}

// A value placed in storage that represents the current version of the Balances storage.
// This value is used by the `on_runtime_upgrade` logic to determine whether we run
// storage migration logic. This should match directly with the semantic versions of the Rust crate.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
    V1_0_0,
    V2_0_0,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V1_0_0
    }
}

impl<T: Config> Pallet<T> {
    /// Get the free balance of an account.
    pub fn free_balance(who: impl sp_std::borrow::Borrow<T::AccountId>) -> T::Balance {
        Self::account(who.borrow()).free
    }

    /// Get the balance of an account that can be used for transfers, reservations, or any other
    /// non-locking, non-transaction-fee activity. Will be at most `free_balance`.
    pub fn usable_balance(who: impl sp_std::borrow::Borrow<T::AccountId>) -> T::Balance {
        Self::account(who.borrow()).usable(Reasons::Misc)
    }

    /// Get the balance of an account that can be used for paying transaction fees (not tipping,
    /// or any other kind of fees, though). Will be at most `free_balance`.
    pub fn usable_balance_for_fees(who: impl sp_std::borrow::Borrow<T::AccountId>) -> T::Balance {
        Self::account(who.borrow()).usable(Reasons::Fee)
    }

    /// Get the reserved balance of an account.
    pub fn reserved_balance(who: impl sp_std::borrow::Borrow<T::AccountId>) -> T::Balance {
        Self::account(who.borrow()).reserved
    }

    pub fn block_rewards_reserve() -> T::AccountId {
        SystematicIssuers::BlockRewardReserve
            .as_module_id()
            .into_account()
    }

    /// Get both the free and reserved balances of an account.
    fn account(who: &T::AccountId) -> AccountData<T::Balance> {
        T::AccountStore::get(&who)
    }

    /// Places the `free` and `reserved` parts of `new` into `account`. Also does any steps needed
    /// after mutating an account. This includes DustRemoval unbalancing, in the case than the `new`
    /// account's total balance is non-zero but below ED.
    ///
    /// Returns the final free balance, iff the account was previously of total balance zero, known
    /// as its "endowment".
    fn post_mutation(
        _who: &T::AccountId,
        new: AccountData<T::Balance>,
    ) -> Option<AccountData<T::Balance>> {
        // Polymesh modified code. Removed Existential Deposit logic
        Some(new)
    }

    /// Mutate an account to some new value, or delete it entirely with `None`.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    pub fn mutate_account<R>(
        who: &T::AccountId,
        f: impl FnOnce(&mut AccountData<T::Balance>) -> R,
    ) -> Result<R, StoredMapError> {
        Self::try_mutate_account(who, |a, _| -> Result<R, StoredMapError> { Ok(f(a)) })
    }

    /// Mutate an account to some new value, or delete it entirely with `None`.
    /// This will do nothing if the result of `f` is an `Err`.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn try_mutate_account<R, E: From<StoredMapError>>(
        who: &T::AccountId,
        f: impl FnOnce(&mut AccountData<T::Balance>, bool) -> Result<R, E>,
    ) -> Result<R, E> {
        T::AccountStore::try_mutate_exists(who, |maybe_account| {
            let is_new = maybe_account.is_none();
            let mut account = maybe_account.take().unwrap_or_default();
            f(&mut account, is_new).map(move |result| {
                let maybe_endowed = if is_new { Some(account.free) } else { None };
                // `post_mutation` always return the same account store
                *maybe_account = Self::post_mutation(who, account);
                (maybe_endowed, result)
            })
        })
        .map(|(maybe_endowed, result)| {
            if let Some(endowed) = maybe_endowed {
                // Polymesh-note: Modified the code in the favour of Polymesh code base
                let who_id = T::IdentityFn::get_identity(who);
                Self::deposit_event(Event::Endowed(who_id, who.clone(), endowed));
            }
            result
        })
    }

    /// Update the account entry for `who`, given the locks.
    fn update_locks(who: &T::AccountId, locks: &[BalanceLock<T::Balance>]) {
        if locks.len() as u32 > T::MaxLocks::get() {
            frame_support::debug::warn!(
                "Warning: A user has more currency locks than expected. \
				A runtime configuration adjustment may be needed."
            );
        }
        // No way this can fail since we do not alter the existential balances.
        let _ = Self::mutate_account(who, |b| {
            b.misc_frozen = Zero::zero();
            b.fee_frozen = Zero::zero();
            for l in locks.iter() {
                if l.reasons == Reasons::All || l.reasons == Reasons::Misc {
                    b.misc_frozen = b.misc_frozen.max(l.amount);
                }
                if l.reasons == Reasons::All || l.reasons == Reasons::Fee {
                    b.fee_frozen = b.fee_frozen.max(l.amount);
                }
            }
        });

        let existed = Locks::<T>::contains_key(who);
        if locks.is_empty() {
            Locks::<T>::remove(who);
            if existed {
                // TODO: use Locks::<T, I>::hashed_key
                // https://github.com/paritytech/substrate/issues/4969
                system::Pallet::<T>::dec_consumers(who);
            }
        } else {
            Locks::<T>::insert(who, locks);
            if !existed {
                if system::Pallet::<T>::inc_consumers(who).is_err() {
                    // No providers for the locks. This is impossible under normal circumstances
                    // since the funds that are under the lock will themselves be stored in the
                    // account and therefore will need a reference.
                    frame_support::debug::warn!(
                        "Warning: Attempt to introduce lock consumer reference, yet no providers. \
						This is unexpected but should be safe."
                    );
                }
            }
        }
    }

    // Polymesh modified code. New wrapper function for the transfer_core function that checks for CDD.
    /// Checks CDD and then only performs the transfer
    fn safe_transfer_core(
        transactor: &T::AccountId,
        dest: &T::AccountId,
        value: T::Balance,
        memo: Option<Memo>,
        existence_requirement: ExistenceRequirement,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            T::CddChecker::check_key_cdd(dest),
            Error::<T>::ReceiverCddMissing
        );

        Self::transfer_core(transactor, dest, value, memo, existence_requirement)
    }

    /// Common functionality for transfers.
    /// It does not emit any event.
    ///
    /// # Return
    /// On success, It will return the applied feed.
    // Transfer some free balance from `transactor` to `dest`.
    // Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
    fn transfer_core(
        transactor: &T::AccountId,
        dest: &T::AccountId,
        value: T::Balance,
        memo: Option<Memo>,
        _existence_requirement: ExistenceRequirement,
    ) -> DispatchResultWithPostInfo {
        if value.is_zero() || transactor == dest {
            return Ok(().into());
        }

        Self::try_mutate_account(dest, |to_account, _| -> DispatchResult {
            Self::try_mutate_account(transactor, |from_account, _| -> DispatchResult {
                from_account.free = from_account
                    .free
                    .checked_sub(&value)
                    .ok_or(Error::<T>::InsufficientBalance)?;

                // NOTE: total stake being stored in the same type means that this could never overflow
                // but better to be safe than sorry.
                to_account.free = to_account
                    .free
                    .checked_add(&value)
                    .ok_or(Error::<T>::Overflow)?;

                Self::ensure_can_withdraw(
                    transactor,
                    value,
                    WithdrawReasons::TRANSFER.into(),
                    from_account.free,
                )?;

                Ok(())
            })
        })?;

        let transactor_id = T::IdentityFn::get_identity(transactor);
        let dest_id = T::IdentityFn::get_identity(dest);

        Self::deposit_event(Event::Transfer(
            transactor_id,
            transactor.clone(),
            dest_id,
            dest.clone(),
            value,
            memo,
        ));

        Ok(().into())
    }
}

impl<T> BalancesTrait<T::AccountId, T::Balance, NegativeImbalance<T>> for Module<T>
where
    T: Config,
{
    fn withdraw(
        who: &T::AccountId,
        value: T::Balance,
        reasons: WithdrawReasons,
        liveness: ExistenceRequirement,
    ) -> sp_std::result::Result<NegativeImbalance<T>, DispatchError> {
        <Self as Currency<T::AccountId>>::withdraw(who, value, reasons, liveness)
    }
}

// Polymesh modified code. Managed BRR related functions.
impl<T: Config> BlockRewardsReserveCurrency<T::Balance, NegativeImbalance<T>> for Module<T> {
    // Polymesh modified code. Drop behavious modified to reduce BRR balance instead of inflating total supply.
    fn drop_positive_imbalance(mut amount: T::Balance) {
        if amount.is_zero() {
            return;
        }
        let brr = Self::block_rewards_reserve();
        let _ = Self::try_mutate_account(&brr, |account, _| -> DispatchResult {
            if account.free > Zero::zero() {
                let old_brr_free_balance = account.free;
                let new_brr_free_balance = old_brr_free_balance.saturating_sub(amount);
                account.free = new_brr_free_balance;
                // Calculate how much amount to mint that is not available with the Brr
                // eg. amount = 100 and the account.free = 60 then `amount_to_mint` = 40
                amount -= old_brr_free_balance - new_brr_free_balance;
            }
            <TotalIssuance<T>>::mutate(|v| *v = v.saturating_add(amount));
            Ok(())
        });
    }

    fn drop_negative_imbalance(amount: T::Balance) {
        <TotalIssuance<T>>::mutate(|v| *v = v.saturating_sub(amount));
    }

    // Polymesh modified code. Instead of minting new tokens, this function tries to transfer tokens from BRR to the beneficiary.
    // If BRR does not have enough free funds, new tokens are issued.
    fn issue_using_block_rewards_reserve(mut amount: T::Balance) -> NegativeImbalance<T> {
        if amount.is_zero() {
            return NegativeImbalance::zero();
        }
        let brr = Self::block_rewards_reserve();
        Self::try_mutate_account(
            &brr,
            |account, _| -> Result<NegativeImbalance<T>, StoredMapError> {
                let amount_to_mint = if account.free > Zero::zero() {
                    let old_brr_free_balance = account.free;
                    let new_brr_free_balance = old_brr_free_balance.saturating_sub(amount);
                    account.free = new_brr_free_balance;
                    // Calculate how much amount to mint that is not available with the Brr
                    // eg. amount = 100 and the account.free = 60 then `amount_to_mint` = 40
                    amount - (old_brr_free_balance - new_brr_free_balance)
                } else {
                    amount
                };
                <TotalIssuance<T>>::mutate(|issued| {
                    *issued = issued.checked_add(&amount_to_mint).unwrap_or_else(|| {
                        amount = T::Balance::max_value() - *issued;
                        T::Balance::max_value()
                    })
                });
                Ok(NegativeImbalance::new(amount))
            },
        )
        .unwrap_or_else(|_x| NegativeImbalance::new(Zero::zero()))
    }

    // Polymesh modified code. Returns balance of BRR
    fn block_rewards_reserve_balance() -> T::Balance {
        let brr = Self::block_rewards_reserve();
        <Self as Currency<T::AccountId>>::free_balance(&brr)
    }
}

impl<T: Config> Currency<T::AccountId> for Pallet<T>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    type Balance = T::Balance;
    type PositiveImbalance = PositiveImbalance<T>;
    type NegativeImbalance = NegativeImbalance<T>;

    fn total_balance(who: &T::AccountId) -> Self::Balance {
        Self::account(who).total()
    }

    // Check if `value` amount of free balance can be slashed from `who`.
    fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
        if value.is_zero() {
            return true;
        }
        Self::free_balance(who) >= value
    }

    fn total_issuance() -> Self::Balance {
        <TotalIssuance<T>>::get()
    }

    fn minimum_balance() -> Self::Balance {
        Zero::zero()
    }

    fn free_balance(who: &T::AccountId) -> Self::Balance {
        Self::account(who).free
    }

    // Burn funds from the total issuance, returning a positive imbalance for the amount burned.
    // Is a no-op if amount to be burned is zero.
    fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
        if amount.is_zero() {
            return PositiveImbalance::zero();
        }
        <TotalIssuance<T>>::mutate(|issued| {
            *issued = issued.checked_sub(&amount).unwrap_or_else(|| {
                amount = *issued;
                Zero::zero()
            });
        });
        PositiveImbalance::new(amount)
    }

    // Create new funds into the total issuance, returning a negative imbalance
    // for the amount issued.
    // Is a no-op if amount to be issued it zero.
    fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
        if amount.is_zero() {
            return NegativeImbalance::zero();
        }
        <TotalIssuance<T>>::mutate(|issued| {
            *issued = issued.checked_add(&amount).unwrap_or_else(|| {
                amount = Self::Balance::max_value() - *issued;
                Self::Balance::max_value()
            })
        });
        NegativeImbalance::new(amount)
    }

    // Ensure that an account can withdraw from their free balance given any existing withdrawal
    // restrictions like locks and vesting balance.
    // Is a no-op if amount to be withdrawn is zero.
    //
    // # <weight>
    // Despite iterating over a list of locks, they are limited by the number of
    // lock IDs, which means the number of runtime pallets that intend to use and create locks.
    // # </weight>
    fn ensure_can_withdraw(
        who: &T::AccountId,
        amount: T::Balance,
        reasons: WithdrawReasons,
        new_balance: T::Balance,
    ) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }
        let min_balance = Self::account(who).frozen(reasons.into());
        ensure!(
            new_balance >= min_balance,
            Error::<T>::LiquidityRestrictions
        );
        Ok(())
    }

    // Important-Note - Use the transfer carefully as this function is not resilient for the cdd check of receiver.
    // Transfer some free balance from `transactor` to `dest`, respecting existence requirements.
    // Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
    fn transfer(
        transactor: &T::AccountId,
        dest: &T::AccountId,
        value: Self::Balance,
        existence_requirement: ExistenceRequirement,
    ) -> DispatchResult {
        // Calling `transfer_core()` instead of the `safe_transfer_core()` to support the
        // transfer to the smart extensions using the pallet-contracts.
        Self::transfer_core(transactor, dest, value, None, existence_requirement)
            .map_err(|err_with_post| err_with_post.error)?;
        Ok(())
    }

    /// Slash a target account `who`, returning the negative imbalance created and any left over
    /// amount that could not be slashed.
    ///
    /// Is a no-op if `value` to be slashed is zero or the account does not exist.
    ///
    /// NOTE: `slash()` prefers free balance, but assumes that reserve balance can be drawn
    /// from in extreme circumstances. `can_slash()` should be used prior to `slash()` to avoid having
    /// to draw from reserved funds, however we err on the side of punishment if things are inconsistent
    /// or `can_slash` wasn't used appropriately.
    fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        if value.is_zero() {
            return (NegativeImbalance::zero(), Zero::zero());
        }
        if Self::total_balance(&who).is_zero() {
            return (NegativeImbalance::zero(), value);
        }

        for attempt in 0..2 {
            match Self::try_mutate_account(who,
				|account, _is_new| -> Result<(Self::NegativeImbalance, Self::Balance), StoredMapError> {
					// Best value is the most amount we can slash following liveness rules.
					let best_value = match attempt {
						// First attempt we try to slash the full amount, and see if liveness issues happen.
						0 => value,
						// If acting as a critical provider (i.e. first attempt failed), then slash
						// as much as possible while leaving at least at ED.
						_ => value.min((account.free + account.reserved).saturating_sub(T::ExistentialDeposit::get())),
					};

					let free_slash = cmp::min(account.free, best_value);
					account.free -= free_slash; // Safe because of above check
					let remaining_slash = best_value - free_slash; // Safe because of above check

					if !remaining_slash.is_zero() {
						// If we have remaining slash, take it from reserved balance.
						let reserved_slash = cmp::min(account.reserved, remaining_slash);
						account.reserved -= reserved_slash; // Safe because of above check
						Ok((
							NegativeImbalance::new(free_slash + reserved_slash),
							value - free_slash - reserved_slash, // Safe because value is gt or eq total slashed
						))
					} else {
						// Else we are done!
						Ok((
							NegativeImbalance::new(free_slash),
							value - free_slash, // Safe because value is gt or eq to total slashed
						))
					}
				}
			) {
				Ok(r) => return r,
				Err(_) => (),
			}
        }

        // Should never get here. But we'll be defensive anyway.
        (Self::NegativeImbalance::zero(), value)
    }

    /// Deposit some `value` into the free balance of an existing target account `who`.
    ///
    /// Is a no-op if the `value` to be deposited is zero.
    fn deposit_into_existing(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, DispatchError> {
        if value.is_zero() {
            return Ok(PositiveImbalance::zero());
        }

        Self::try_mutate_account(
            who,
            |account, _| -> Result<Self::PositiveImbalance, DispatchError> {
                // Polymesh modified code. Removed existential deposit requirements.
                // This function is now logically equivalent to `deposit_creating`.

                account.free = account
                    .free
                    .checked_add(&value)
                    .ok_or(Error::<T>::Overflow)?;
                Ok(PositiveImbalance::new(value))
            },
        )
    }

    /// Deposit some `value` into the free balance of `who`, possibly creating a new account.
    ///
    /// This function is a no-op if:
    /// - the `value` to be deposited is zero; or
    /// - `value` is so large it would cause the balance of `who` to overflow.
    fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        if value.is_zero() {
            return Self::PositiveImbalance::zero();
        }

        let r = Self::try_mutate_account(
            who,
            |account, _| -> Result<Self::PositiveImbalance, DispatchError> {
                // Polymesh modified code. Removed existential deposit requirements.
                // This function is now logically equivalent to `deposit_into_existing`.

                // defensive only: overflow should never happen, however in case it does, then this
                // operation is a no-op.
                account.free = match account.free.checked_add(&value) {
                    Some(x) => x,
                    None => return Ok(Self::PositiveImbalance::zero()),
                };

                Ok(PositiveImbalance::new(value))
            },
        )
        .unwrap_or_else(|_| Self::PositiveImbalance::zero());

        r
    }

    /// Withdraw some free balance from an account
    ///
    /// Is a no-op if value to be withdrawn is zero.
    fn withdraw(
        who: &T::AccountId,
        value: Self::Balance,
        reasons: WithdrawReasons,
        _liveness: ExistenceRequirement,
    ) -> result::Result<Self::NegativeImbalance, DispatchError> {
        if value.is_zero() {
            return Ok(NegativeImbalance::zero());
        }

        Self::try_mutate_account(
            who,
            |account, _| -> Result<Self::NegativeImbalance, DispatchError> {
                let new_free_account = account
                    .free
                    .checked_sub(&value)
                    .ok_or(Error::<T>::InsufficientBalance)?;

                // Polymesh modified code. Removed existential deposit requirements.

                Self::ensure_can_withdraw(who, value, reasons, new_free_account)?;

                account.free = new_free_account;

                Ok(NegativeImbalance::new(value))
            },
        )
    }

    /// Force the new free balance of a target account `who` to some new value `balance`.
    fn make_free_balance_be(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        Self::try_mutate_account(who, |account, _is_new|
			-> Result<SignedImbalance<Self::Balance, Self::PositiveImbalance>, DispatchError>
		{
                // Polymesh modified code. Removed existential deposit requirements.

			let imbalance = if account.free <= value {
				SignedImbalance::Positive(PositiveImbalance::new(value - account.free))
			} else {
				SignedImbalance::Negative(NegativeImbalance::new(account.free - value))
			};
			account.free = value;
			Ok(imbalance)
		}).unwrap_or_else(|_| SignedImbalance::Positive(Self::PositiveImbalance::zero()))
    }
}

impl<T: Config> ReservableCurrency<T::AccountId> for Pallet<T>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    /// Check if `who` can reserve `value` from their free balance.
    ///
    /// Always `true` if value to be reserved is zero.
    fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
        if value.is_zero() {
            return true;
        }
        Self::account(who)
            .free
            .checked_sub(&value)
            .map_or(false, |new_balance| {
                Self::ensure_can_withdraw(who, value, WithdrawReasons::RESERVE, new_balance).is_ok()
            })
    }

    fn reserved_balance(who: &T::AccountId) -> Self::Balance {
        Self::account(who).reserved
    }

    /// Move `value` from the free balance from `who` to their reserved balance.
    ///
    /// Is a no-op if value to be reserved is zero.
    fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
        if value.is_zero() {
            return Ok(());
        }

        Self::try_mutate_account(who, |account, _| -> DispatchResult {
            account.free = account
                .free
                .checked_sub(&value)
                .ok_or(Error::<T>::InsufficientBalance)?;
            account.reserved = account
                .reserved
                .checked_add(&value)
                .ok_or(Error::<T>::Overflow)?;
            Self::ensure_can_withdraw(&who, value.clone(), WithdrawReasons::RESERVE, account.free)
        })?;

        Self::deposit_event(Event::Reserved(who.clone(), value));
        Ok(())
    }

    /// Unreserve some funds, returning any amount that was unable to be unreserved.
    ///
    /// Is a no-op if the value to be unreserved is zero or the account does not exist.
    fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
        if value.is_zero() {
            return Zero::zero();
        }
        if Self::total_balance(&who).is_zero() {
            return value;
        }

        let actual = match Self::mutate_account(who, |account| {
            let actual = cmp::min(account.reserved, value);
            account.reserved -= actual;
            // defensive only: this can never fail since total issuance which is at least free+reserved
            // fits into the same data type.
            account.free = account.free.saturating_add(actual);
            actual
        }) {
            Ok(x) => x,
            Err(_) => {
                // This should never happen since we don't alter the total amount in the account.
                // If it ever does, then we should fail gracefully though, indicating that nothing
                // could be done.
                return value;
            }
        };

        Self::deposit_event(Event::Unreserved(who.clone(), actual.clone()));
        value - actual
    }

    /// Slash from reserved balance, returning the negative imbalance created,
    /// and any amount that was unable to be slashed.
    ///
    /// Is a no-op if the value to be slashed is zero or the account does not exist.
    fn slash_reserved(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> (Self::NegativeImbalance, Self::Balance) {
        if value.is_zero() {
            return (NegativeImbalance::zero(), Zero::zero());
        }
        if Self::total_balance(&who).is_zero() {
            return (NegativeImbalance::zero(), value);
        }

        // NOTE: `mutate_account` may fail if it attempts to reduce the balance to the point that an
        //   account is attempted to be illegally destroyed.

        for attempt in 0..2 {
            match Self::mutate_account(who, |account| {
                let best_value = match attempt {
                    0 => value,
                    // If acting as a critical provider (i.e. first attempt failed), then ensure
                    // slash leaves at least the ED.
                    _ => value.min(
                        (account.free + account.reserved)
                            .saturating_sub(T::ExistentialDeposit::get()),
                    ),
                };

                let actual = cmp::min(account.reserved, best_value);
                account.reserved -= actual;

                // underflow should never happen, but it if does, there's nothing to be done here.
                (NegativeImbalance::new(actual), value - actual)
            }) {
                Ok(r) => return r,
                Err(_) => (),
            }
        }
        // Should never get here as we ensure that ED is left in the second attempt.
        // In case we do, though, then we fail gracefully.
        (Self::NegativeImbalance::zero(), value)
    }

    /// Move the reserved balance of one account into the balance of another, according to `status`.
    ///
    /// Is a no-op if:
    /// - the value to be moved is zero; or
    /// - the `slashed` id equal to `beneficiary` and the `status` is `Reserved`.
    fn repatriate_reserved(
        slashed: &T::AccountId,
        beneficiary: &T::AccountId,
        value: Self::Balance,
        status: Status,
    ) -> Result<Self::Balance, DispatchError> {
        if value.is_zero() {
            return Ok(Zero::zero());
        }

        if slashed == beneficiary {
            return match status {
                Status::Free => Ok(Self::unreserve(slashed, value)),
                Status::Reserved => Ok(value.saturating_sub(Self::reserved_balance(slashed))),
            };
        }

        let actual = Self::try_mutate_account(
            beneficiary,
            |to_account, _| -> Result<Self::Balance, DispatchError> {
                // Polymesh modified code. Removed existential deposit requirements.
                Self::try_mutate_account(
                    slashed,
                    |from_account, _| -> Result<Self::Balance, DispatchError> {
                        let actual = cmp::min(from_account.reserved, value);
                        match status {
                            Status::Free => {
                                to_account.free = to_account
                                    .free
                                    .checked_add(&actual)
                                    .ok_or(Error::<T>::Overflow)?
                            }
                            Status::Reserved => {
                                to_account.reserved = to_account
                                    .reserved
                                    .checked_add(&actual)
                                    .ok_or(Error::<T>::Overflow)?
                            }
                        }
                        from_account.reserved -= actual;
                        Ok(actual)
                    },
                )
            },
        )?;

        Self::deposit_event(Event::ReserveRepatriated(
            slashed.clone(),
            beneficiary.clone(),
            actual,
            status,
        ));
        Ok(value - actual)
    }
}

impl<T: Config> LockableCurrency<T::AccountId> for Pallet<T>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    type Moment = T::BlockNumber;

    type MaxLocks = T::MaxLocks;

    // Polymesh-note: The implementations below differ from substrate in terms
    // of performance (ours uses in-place modification), but are functionally equivalent.

    // Set a lock on the balance of `who`.
    // Is a no-op if lock amount is zero or `reasons` `is_none()`.
    fn set_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        reasons: WithdrawReasons,
    ) {
        if amount.is_zero() || reasons.is_empty() {
            return;
        }
        let new_lock = BalanceLock {
            id,
            amount,
            reasons: reasons.into(),
        };
        let mut locks = Self::locks(who);
        if let Some(pos) = locks.iter().position(|l| l.id == id) {
            locks[pos] = new_lock;
        } else {
            locks.push(new_lock);
        }
        Self::update_locks(who, &locks[..]);
    }

    // Extend a lock on the balance of `who`.
    // Is a no-op if lock amount is zero or `reasons` `is_none()`.
    fn extend_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        reasons: WithdrawReasons,
    ) {
        if amount.is_zero() || reasons.is_empty() {
            return;
        }
        let reasons = reasons.into();
        let mut locks = Self::locks(who);
        if let Some(pos) = locks.iter().position(|l| l.id == id) {
            let slot = &mut locks[pos];
            slot.amount = slot.amount.max(amount);
            slot.reasons = slot.reasons | reasons;
        } else {
            locks.push(BalanceLock {
                id,
                amount,
                reasons,
            });
        }
        Self::update_locks(who, &locks[..]);
    }

    fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
        let mut locks = Self::locks(who);
        locks.retain(|l| l.id != id);
        Self::update_locks(who, &locks[..]);
    }
}

impl<T: Config> LockableCurrencyExt<T::AccountId> for Pallet<T>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    fn reduce_lock(id: LockIdentifier, who: &T::AccountId, amount: T::Balance) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }
        let mut locks = Self::locks(who);
        locks
            .iter()
            .position(|l| l.id == id)
            .and_then(|p| {
                let slot = &mut locks[p].amount;
                let new = slot.checked_sub(&amount).map(|n| *slot = n);
                if slot.is_zero() {
                    locks.swap_remove(p);
                }
                new
            })
            .ok_or(Error::<T>::InsufficientBalance)?;
        Self::update_locks(who, &locks[..]);
        Ok(())
    }

    fn increase_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        reasons: WithdrawReasons,
        check_sum: impl FnOnce(T::Balance) -> DispatchResult,
    ) -> DispatchResult {
        if amount.is_zero() || reasons.is_empty() {
            return Ok(());
        }
        let reasons = reasons.into();
        let mut locks = Self::locks(who);
        check_sum(if let Some(pos) = locks.iter().position(|l| l.id == id) {
            let slot = &mut locks[pos];
            slot.amount = slot
                .amount
                .checked_add(&amount)
                .ok_or(Error::<T>::Overflow)?;
            slot.reasons = slot.reasons | reasons;
            slot.amount
        } else {
            locks.push(BalanceLock {
                id,
                amount,
                reasons,
            });
            amount
        })?;
        Self::update_locks(who, &locks[..]);
        Ok(())
    }
}
