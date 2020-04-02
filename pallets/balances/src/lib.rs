// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

// Modified by Polymath Inc - 16th March 2020
// Added ability to manage balances of identities with the balances module
// In Polymesh, POLY balances can be held at either the identity or account level
// Implement `BlockRewardsReserveCurrency` trait in the balances module.
// Remove migration functionality from the balances module as Polymesh doesn't needed
// any migration data structure.

//! # Balances Module
//!
//! The Balances module provides functionality for handling accounts and balances.
//!
//! - [`balances::Trait`](./trait.Trait.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! This is a modified implementation of substrate's balances FRAME.
//! The modifications made are as follows:
//!
//! - Added ability to pay transaction fees from identity's balance instead of user's balance.
//! - To curb front running, sending a tip along with your transaction is now prohibited.
//! - Added ability to store balance at identity level and use that to pay tx fees.
//! - Added From<u128> trait to Balances type.
//! - Removed existential amount requirement to prevent a replay attack scenario.
//!
//! The Original Balances module provides functions for:
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
//! - **Reaping an account:** The act of removing an account by resetting its nonce. Happens after its balance is set
//! to zero.
//! - **Free Balance:** The portion of a balance that is not reserved. The free balance is the only balance that matters
//! for most operations.
//! - **Reserved Balance:** Reserved balance still belongs to the account holder, but is suspended. Reserved balance
//! can still be slashed, but only after all the free balance has been slashed.
//! - **Imbalance:** A condition when some funds were credited or debited without equal and opposite accounting
//! (i.e. a difference between total issuance and account balances). Functions that result in an imbalance will
//! return an object of the `Imbalance` trait that can be managed within your runtime logic. (If an imbalance is
//! simply dropped, it should automatically maintain any book-keeping such as total issuance.)
//! - **Lock:** A freeze on a specified amount of an account's free balance until a specified block number. Multiple
//! locks always operate over the same funds, so they "overlay" rather than "stack".
//! - **Vesting:** Similar to a lock, this is another, but independent, liquidity restriction that reduces linearly
//! over time.
//!
//! ### Implementations
//!
//! The Balances module provides implementations for the following traits. If these traits provide the functionality
//! that you need, then you can avoid coupling with the Balances module.
//!
//! - [`Currency`](../frame_support/traits/trait.Currency.html): Functions for dealing with a
//! fungible assets system.
//! - [`ReservableCurrency`](../frame_support/traits/trait.ReservableCurrency.html):
//! Functions for dealing with assets that can be reserved from an account.
//! - [`LockableCurrency`](../frame_support/traits/trait.LockableCurrency.html): Functions for
//! dealing with accounts that allow liquidity restrictions.
//! - [`Imbalance`](../frame_support/traits/trait.Imbalance.html): Functions for handling
//! imbalances between total issuance in the system and account balances. Must be used when a function
//! creates new funds (e.g. a reward) or destroys some funds (e.g. a system fee).
//! - [`IsDeadAccount`](../srml_system/trait.IsDeadAccount.html): Determiner to say whether a
//! given account is unused.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `transfer` - Transfer some liquid free balance to another account.
//! - `top_up_identity_balance` - Move some poly from balance of self to balance of an identity.
//! - `reclaim_identity_balance` - Claim back poly from an identity. Can only be called by master key of the identity.
//! - `change_charge_did_flag` - Change setting that governs if user pays fee via their own balance or identity's balance.
//! - `set_balance` - Set the balances of a given account. The origin of this call must be root.
//!
//! ### Public Functions
//!
//! - `vesting_balance` - Get the amount that is currently being vested and cannot be transferred out of this account.
//!
//! ## Usage
//!
//! The following examples show how to use the Balances module in your custom module.
//!
//! ### Examples from the FRAME
//!
//! The Contract module uses the `Currency` trait to handle gas payment, and its types inherit from `Currency`:
//!
//! ```
//! use frame_support::traits::Currency;
//! # pub trait Trait: frame_system::Trait {
//! # type Currency: Currency<Self::AccountId>;
//! # }
//!
//! pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
//! pub type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;
//!
//! # fn main() {}
//! ```
//!
//! The Staking module uses the `LockableCurrency` trait to lock a stash account's funds:
//!
//! ```
//! use frame_support::traits::{WithdrawReasons, LockableCurrency};
//! use sp_runtime::traits::Bounded;
//! pub trait Trait: frame_system::Trait {
//! type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
//! }
//! # struct StakingLedger<T: Trait> {
//!     # stash: <T as frame_system::Trait>::AccountId,
//!     # total: <<T as Trait>::Currency as frame_support::traits::Currency<<T as frame_system::Trait>::AccountId>>::Balance,
//!     # phantom: std::marker::PhantomData<T>,
//! # }
//! # const STAKING_ID: [u8; 8] = *b"staking ";
//!
//! fn update_ledger<T: Trait>(
//!     controller: &T::AccountId,
//!     ledger: &StakingLedger<T>
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
//! The Balances module depends on the [`GenesisConfig`](./struct.GenesisConfig.html).
//!
//! ## Assumptions
//!
//! * Total issued balanced of all accounts should be less than `Trait::Balance::max_value()`.

#![cfg_attr(not(feature = "std"), no_std)]

use polymesh_primitives::{
    traits::{BlockRewardsReserveCurrency, IdentityCurrency},
    AccountKey, IdentityId, Permission, Signatory,
};
use polymesh_runtime_common::traits::{
    balances::{AccountData, BalancesTrait, CheckCdd, Memo, RawEvent, Reasons},
    identity::IdentityTrait,
    NegativeImbalance, PositiveImbalance,
};

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage, ensure,
    traits::{
        BalanceStatus as Status, Currency, ExistenceRequirement, Imbalance, IsDeadAccount,
        LockIdentifier, LockableCurrency, OnKilledAccount, ReservableCurrency, SignedImbalance,
        StoredMap, WithdrawReason, WithdrawReasons,
    },
    weights::SimpleDispatchInfo,
    StorageValue,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::{
    traits::{
        Bounded, CheckedAdd, CheckedSub, Hash, MaybeSerializeDeserialize, Saturating, StaticLookup,
        Zero,
    },
    DispatchError, DispatchResult, RuntimeDebug,
};

use sp_std::{
    cmp, convert::Infallible, convert::TryFrom, fmt::Debug, mem, prelude::*, result, vec,
};

pub use polymesh_runtime_common::traits::balances::Trait;
pub type Event<T> = polymesh_runtime_common::traits::balances::Event<T>;

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Account liquidity restrictions prevent withdrawal
        LiquidityRestrictions,
        /// Got an overflow after adding
        Overflow,
        /// Balance too low to send value
        InsufficientBalance,
        /// Value too low to create account due to existential deposit
        ExistentialDeposit,
        /// Transfer/payment would kill account
        KeepAlive,
        /// AccountId is not attached with Identity
        UnAuthorized,
        /// Receiver does not have a valid CDD
        ReceiverCddMissing
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

decl_storage! {
    trait Store for Module<T: Trait> as Balances {
        /// The total units issued in the system.
        pub TotalIssuance get(fn total_issuance) build(|config: &GenesisConfig<T>| {
            config.balances.iter().fold(Zero::zero(), |acc: T::Balance, &(_, n)| acc + n)
        }): T::Balance;

        /// The balance of an account.
        ///
        /// NOTE: THIS MAY NEVER BE IN EXISTENCE AND YET HAVE A `total().is_zero()`. If the total
        /// is ever zero, then the entry *MUST* be removed.
        ///
        /// NOTE: This is only used in the case that this module is used to store balances.
        pub Account: map hasher(blake2_256) T::AccountId => AccountData<T::Balance>;

        /// Any liquidity locks on some account balances.
        /// NOTE: Should only be accessed when setting, changing and freeing a lock.
        pub Locks get(fn locks): map hasher(blake2_256) T::AccountId => Vec<BalanceLock<T::Balance>>;

        /// Balance held by the identity. It can be spent by its signing keys.
        pub IdentityBalance get(identity_balance): map hasher(blake2_256) IdentityId => T::Balance;

        // Polymesh-Note : Change to facilitate the DID charging
        /// Signing key => Charge Fee to did?. Default is false i.e. the fee will be charged from user balance
        pub ChargeDid get(charge_did): map hasher(blake2_256) AccountKey => bool;

        // Polymesh-Note : Change to facilitate the BRR functionality
        /// AccountId of the block rewards reserve
        pub BlockRewardsReserve get(block_rewards_reserve) build(|_| {
            let h: T::Hash = T::Hashing::hash(&(b"BLOCK_REWARDS_RESERVE").encode());
            T::AccountId::decode(&mut &h.encode()[..]).unwrap_or_default()
        }): T::AccountId;
    }
    add_extra_genesis {
        config(balances): Vec<(T::AccountId, T::Balance)>;
        // ^^ begin, length, amount liquid at genesis
        build(|config: &GenesisConfig<T>| {
            for &(ref who, free) in config.balances.iter() {
                T::AccountStore::insert(who, AccountData { free, .. Default::default() });
            }
        });
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// This is no longer needed but kept for compatibility reasons
        /// The minimum amount required to keep an account open.
        const ExistentialDeposit: T::Balance = 0.into();

        fn deposit_event() = default;

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
        ///
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn transfer(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            #[compact] value: T::Balance
        ) {
            let transactor = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;
            Self::transfer_core(&transactor, &dest, value, None, ExistenceRequirement::AllowDeath)?;
        }

        /// Transfer the native currency with the help of identifier string
        /// this functionality can help to differentiate the transfers.
        ///
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(1_005_000)]
        pub fn transfer_with_memo(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            #[compact] value: T::Balance,
            memo: Option<Memo>
        ) {
            let transactor = ensure_signed(origin)?;
            let dest = T::Lookup::lookup(dest)?;
            Self::transfer_core(&transactor, &dest, value, memo, ExistenceRequirement::AllowDeath)?;
        }

        /// Move some poly from balance of self to balance of an identity.
        /// no-op when,
        /// - value is zero
        ///
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn top_up_identity_balance(
            origin,
            did: IdentityId,
            #[compact] value: T::Balance
        ) {
            if value.is_zero() { return Ok(()) }
            let transactor = ensure_signed(origin)?;
            match <Self as Currency<_>>::withdraw(
                &transactor,
                value,
                WithdrawReason::TransactionPayment.into(),
                ExistenceRequirement::KeepAlive,
            ) {
                Ok(_) => {
                    let new_balance = Self::identity_balance(&did) + value;
                    <IdentityBalance<T>>::insert(did, new_balance);
                    return Ok(())
                },
                Err(err) => return Err(err),
            };
        }

        /// Claim back poly from an identity. Can only be called by master key of the identity.
        /// no-op when,
        /// - value is zero
        ///
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn reclaim_identity_balance(
            origin,
            did: IdentityId,
            #[compact] value: T::Balance
        ) {
            if value.is_zero() { return Ok(()) }
            let transactor = ensure_signed(origin)?;
            let encoded_transactor = AccountKey::try_from(transactor.encode())?;
            ensure!(<T::Identity>::is_master_key(did, &encoded_transactor), Error::<T>::UnAuthorized);
            // Not managing imbalances because they will cancel out.
            // withdraw function will create negative imbalance and
            // deposit function will create positive imbalance
            let _ = Self::withdraw_identity_balance(&did, value)?;
            let _ = <Self as Currency<_>>::deposit_creating(&transactor, value);
            return Ok(())
        }

        /// Change setting that governs if user pays fee via their own balance or identity's balance.
        ///
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        pub fn change_charge_did_flag(origin, charge_did: bool) {
            let transactor = ensure_signed(origin)?;
            let encoded_transactor = AccountKey::try_from(transactor.encode())?;
            <ChargeDid>::insert(encoded_transactor, charge_did);
        }

        /// Set the balances of a given account.
        ///
        /// This will alter `FreeBalance` and `ReservedBalance` in storage. it will
        /// also decrease the total issuance of the system (`TotalIssuance`).
        ///
        /// The dispatch origin for this call is `root`.
        ///
        /// # <weight>
        /// - Independent of the arguments.
        /// - Contains a limited number of reads and writes.
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedOperational(50_000)]
        fn set_balance(
            origin,
            who: <T::Lookup as StaticLookup>::Source,
            #[compact] new_free: T::Balance,
            #[compact] new_reserved: T::Balance
        ) {
            ensure_root(origin)?;
            let who = T::Lookup::lookup(who)?;

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
            });
            Self::deposit_event(RawEvent::BalanceSet(who, free, reserved));
        }

        /// Exactly as `transfer`, except the origin must be root and the source account may be
        /// specified.
        ///
        /// # </weight>
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn force_transfer(
            origin,
            source: <T::Lookup as StaticLookup>::Source,
            dest: <T::Lookup as StaticLookup>::Source,
            #[compact] value: T::Balance
        ) {
            ensure_root(origin)?;
            let source = T::Lookup::lookup(source)?;
            let dest = T::Lookup::lookup(dest)?;
            Self::transfer_core(&source, &dest, value, None, ExistenceRequirement::AllowDeath)?;
        }
    }
}

impl<T: Trait> Module<T> {
    // PRIVATE MUTABLES

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
        // POLYMESH-NOTE: Removed Existential Deposit logic
        Some(new)
    }

    /// Mutate an account to some new value, or delete it entirely with `None`.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn mutate_account<R>(
        who: &T::AccountId,
        f: impl FnOnce(&mut AccountData<T::Balance>) -> R,
    ) -> R {
        Self::try_mutate_account(who, |a| -> Result<R, Infallible> { Ok(f(a)) })
            .expect("Error is infallible; qed")
    }

    /// Mutate an account to some new value, or delete it entirely with `None`.
    /// This will do nothing if the result of `f` is an `Err`.
    ///
    /// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
    /// when it is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn try_mutate_account<R, E>(
        who: &T::AccountId,
        f: impl FnOnce(&mut AccountData<T::Balance>) -> Result<R, E>,
    ) -> Result<R, E> {
        T::AccountStore::try_mutate_exists(who, |maybe_account| {
            let mut account = maybe_account.take().unwrap_or_default();
            let was_zero = account.total().is_zero();
            f(&mut account).map(move |result| {
                let maybe_endowed = if was_zero { Some(account.free) } else { None };
                // `post_mutation` always return the same account store
                *maybe_account = Self::post_mutation(who, account);
                (maybe_endowed, result)
            })
        })
        .map(|(maybe_endowed, result)| {
            if let Some(endowed) = maybe_endowed {
                Self::deposit_event(RawEvent::Endowed(who.clone(), endowed));
            }
            result
        })
    }

    /// Update the account entry for `who`, given the locks.
    fn update_locks(who: &T::AccountId, locks: &[BalanceLock<T::Balance>]) {
        Self::mutate_account(who, |b| {
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
                system::Module::<T>::dec_ref(who);
            }
        } else {
            Locks::<T>::insert(who, locks);
            if !existed {
                system::Module::<T>::inc_ref(who);
            }
        }
    }

    // Transfer some free balance from `transactor` to `dest`.
    // Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
    fn transfer_core(
        transactor: &T::AccountId,
        dest: &T::AccountId,
        value: T::Balance,
        memo: Option<Memo>,
        _existence_requirement: ExistenceRequirement,
    ) -> DispatchResult {
        if !T::CddChecker::check_key_cdd(&AccountKey::try_from((*dest).encode())?) {
            return Err(Error::<T>::ReceiverCddMissing.into());
        }
        if value.is_zero() || transactor == dest {
            return Ok(());
        }

        Self::try_mutate_account(dest, |to_account| -> DispatchResult {
            Self::try_mutate_account(transactor, |from_account| -> DispatchResult {
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
                    WithdrawReason::Transfer.into(),
                    from_account.free,
                )?;

                Ok(())
            })
        })?;

        if let Some(memo) = memo {
            // Emit TransferWithMemo event.
            Self::deposit_event(RawEvent::TransferWithMemo(
                transactor.clone(),
                dest.clone(),
                value,
                memo,
            ));
        } else {
            // Emit transfer event.
            Self::deposit_event(RawEvent::Transfer(transactor.clone(), dest.clone(), value));
        }
        Ok(())
    }
}

impl<T> BalancesTrait<T::AccountId, T::Balance, NegativeImbalance<T>> for Module<T>
where
    T: Trait,
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

impl<T: Trait> BlockRewardsReserveCurrency<T::Balance, NegativeImbalance<T>> for Module<T> {
    fn drop_positive_imbalance(mut amount: T::Balance) {
        let brr = <BlockRewardsReserve<T>>::get();
        Self::try_mutate_account(&brr, |account| -> DispatchResult {
            if account.free > Zero::zero() {
                let old_brr_free_balance = account.free;
                let new_brr_free_balance = old_brr_free_balance.saturating_sub(amount);
                account.free = new_brr_free_balance;
                // Calculate how much amount to mint that is not available with the Brr
                // eg. amount = 100 and the account.free = 60 then `amount_to_mint` = 40
                amount = amount - (old_brr_free_balance - new_brr_free_balance);
            }
            <TotalIssuance<T>>::mutate(|v| *v = v.saturating_add(amount));
            Ok(())
        });
    }

    fn drop_negative_imbalance(amount: T::Balance) {
        <TotalIssuance<T>>::mutate(|v| *v = v.saturating_sub(amount));
    }

    fn issue_using_block_rewards_reserve(mut amount: T::Balance) -> NegativeImbalance<T> {
        let brr = <BlockRewardsReserve<T>>::get();
        Self::try_mutate_account(&brr, |account| -> Result<NegativeImbalance<T>, ()> {
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
        })
        .unwrap_or_else(|_x| NegativeImbalance::new(Zero::zero()))
    }

    fn block_rewards_reserve_balance() -> T::Balance {
        let brr = Self::block_rewards_reserve();
        <Self as Currency<T::AccountId>>::free_balance(&brr)
    }
}

impl<T: Trait> Currency<T::AccountId> for Module<T>
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
        0u128.into()
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
    // lock IDs, which means the number of runtime modules that intend to use and create locks.
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

    // Transfer some free balance from `transactor` to `dest`.
    // Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
    fn transfer(
        transactor: &T::AccountId,
        dest: &T::AccountId,
        value: Self::Balance,
        existence_requirement: ExistenceRequirement,
    ) -> DispatchResult {
        Self::transfer_core(transactor, dest, value, None, existence_requirement)?;
        Ok(())
    }

    /// Slash a target account `who`, returning the negative imbalance created and any left over
    /// amount that could not be slashed.
    ///
    /// Is a no-op if `value` to be slashed is zero.
    ///
    /// NOTE: `slash()` prefers free balance, but assumes that reserve balance can be drawn
    /// from in extreme circumstances. `can_slash()` should be used prior to `slash()` to avoid having
    /// to draw from reserved funds, however we err on the side of punishment if things are inconsistent
    /// or `can_slash` wasn't used appropriately.
    fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        if value.is_zero() {
            return (NegativeImbalance::zero(), Zero::zero());
        }

        Self::mutate_account(who, |account| {
            let free_slash = cmp::min(account.free, value);
            account.free -= free_slash;

            let remaining_slash = value - free_slash;
            if !remaining_slash.is_zero() {
                let reserved_slash = cmp::min(account.reserved, remaining_slash);
                account.reserved -= reserved_slash;
                (
                    NegativeImbalance::new(free_slash + reserved_slash),
                    remaining_slash - reserved_slash,
                )
            } else {
                (NegativeImbalance::new(value), Zero::zero())
            }
        })
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
            |account| -> Result<Self::PositiveImbalance, DispatchError> {
                // POLYMESH-NOTE: Remove ensure check in the favour of Polymesh blockchain logic because dead account logic
                // don't exist in the Polymesh blockchain

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

        Self::try_mutate_account(
            who,
            |account| -> Result<Self::PositiveImbalance, Self::PositiveImbalance> {
                // POLYMESH-NOTE: Remove the ExistentialDeposit in the favour of Polymesh blockchain logic where
                // Existential deposit is always zero

                // defensive only: overflow should never happen, however in case it does, then this
                // operation is a no-op.
                account.free = account
                    .free
                    .checked_add(&value)
                    .ok_or(Self::PositiveImbalance::zero())?;

                Ok(PositiveImbalance::new(value))
            },
        )
        .unwrap_or_else(|x| x)
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
            |account| -> Result<Self::NegativeImbalance, DispatchError> {
                let new_free_account = account
                    .free
                    .checked_sub(&value)
                    .ok_or(Error::<T>::InsufficientBalance)?;

                // Note: In Polymesh related code we don't need the ExistenceRequirement check
                // so we remove the code statements that are present in the substrate code

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
        Self::try_mutate_account(
            who,
            |account| -> Result<SignedImbalance<Self::Balance, Self::PositiveImbalance>, ()> {
                // POLYMESH-NOTE: Remove the ExistentialDeposit Deposit logic in the favour of Polymesh blockchain

                // If we're attempting to set an existing account to less than ED, then
                // bypass the entire operation. It's a no-op if you follow it through, but
                // since this is an instance where we might account for a negative imbalance
                // (in the dust cleaner of set_account) before we account for its actual
                // equal and opposite cause (returned as an Imbalance), then in the
                // instance that there's no other accounts on the system at all, we might
                // underflow the issuance and our arithmetic will be off.

                let imbalance = if account.free <= value {
                    SignedImbalance::Positive(PositiveImbalance::new(value - account.free))
                } else {
                    SignedImbalance::Negative(NegativeImbalance::new(account.free - value))
                };
                account.free = value;
                Ok(imbalance)
            },
        )
        .unwrap_or(SignedImbalance::Positive(Self::PositiveImbalance::zero()))
    }
}

impl<T: Trait> IdentityCurrency<T::AccountId> for Module<T>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    fn withdraw_identity_balance(
        who: &IdentityId,
        value: Self::Balance,
    ) -> result::Result<Self::NegativeImbalance, DispatchError> {
        if let Some(new_balance) = Self::identity_balance(who).checked_sub(&value) {
            <IdentityBalance<T>>::insert(who, new_balance);
            Ok(NegativeImbalance::new(value))
        } else {
            Err(Error::<T>::Overflow)?
        }
    }

    fn charge_fee_to_identity(who: &AccountKey) -> Option<IdentityId> {
        if <Module<T>>::charge_did(who) {
            if let Some(did) = <T::Identity>::get_identity(&who) {
                if <T::Identity>::is_signer_authorized_with_permissions(
                    did,
                    &Signatory::AccountKey(who.clone()),
                    vec![Permission::SpendFunds],
                ) {
                    return Some(did);
                }
            }
        }
        return None;
    }

    fn deposit_into_existing_identity(
        who: &IdentityId,
        value: Self::Balance,
    ) -> result::Result<Self::PositiveImbalance, DispatchError> {
        if value.is_zero() {
            return Ok(PositiveImbalance::zero());
        }
        if let Some(new_balance) = Self::identity_balance(who).checked_add(&value) {
            <IdentityBalance<T>>::insert(who, new_balance);
            Ok(PositiveImbalance::new(value))
        } else {
            Err(Error::<T>::Overflow)?
        }
    }

    fn resolve_into_existing_identity(
        who: &IdentityId,
        value: Self::NegativeImbalance,
    ) -> result::Result<(), Self::NegativeImbalance> {
        let v = value.peek();
        match Self::deposit_into_existing_identity(who, v) {
            Ok(opposite) => Ok(drop(value.offset(opposite))),
            _ => Err(value),
        }
    }
}

impl<T: Trait> ReservableCurrency<T::AccountId> for Module<T>
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
                Self::ensure_can_withdraw(who, value, WithdrawReason::Reserve.into(), new_balance)
                    .is_ok()
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

        Self::try_mutate_account(who, |account| -> DispatchResult {
            account.free = account
                .free
                .checked_sub(&value)
                .ok_or(Error::<T>::InsufficientBalance)?;
            account.reserved = account
                .reserved
                .checked_add(&value)
                .ok_or(Error::<T>::Overflow)?;
            Self::ensure_can_withdraw(who, value, WithdrawReason::Reserve.into(), account.free)
        })
    }

    /// Unreserve some funds, returning any amount that was unable to be unreserved.
    ///
    /// Is a no-op if the value to be unreserved is zero.
    fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
        if value.is_zero() {
            return Zero::zero();
        }

        Self::mutate_account(who, |account| {
            let actual = cmp::min(account.reserved, value);
            account.reserved -= actual;
            // defensive only: this can never fail since total issuance which is at least free+reserved
            // fits into the same data type.
            account.free = account.free.saturating_add(actual);
            value - actual
        })
    }

    /// Slash from reserved balance, returning the negative imbalance created,
    /// and any amount that was unable to be slashed.
    ///
    /// Is a no-op if the value to be slashed is zero.
    fn slash_reserved(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> (Self::NegativeImbalance, Self::Balance) {
        if value.is_zero() {
            return (NegativeImbalance::zero(), Zero::zero());
        }

        Self::mutate_account(who, |account| {
            // underflow should never happen, but it if does, there's nothing to be done here.
            let actual = cmp::min(account.reserved, value);
            account.reserved -= actual;
            (NegativeImbalance::new(actual), value - actual)
        })
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

        Self::try_mutate_account(
            beneficiary,
            |to_account| -> Result<Self::Balance, DispatchError> {
                // POLYMESH-NOTE: Polymesh is not declaring the accounts as `DEAD` even if the total balance is zero
                Self::try_mutate_account(
                    slashed,
                    |from_account| -> Result<Self::Balance, DispatchError> {
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
                        Ok(value - actual)
                    },
                )
            },
        )
    }
}

/// Implement `OnKilledAccount` to remove the local account, if using local account storage.
///
/// NOTE: You probably won't need to use this! This only needs to be "wired in" to System module
/// if you're using the local balance storage. **If you're using the composite system account
/// storage (which is the default in most examples and tests) then there's no need.**
impl<T: Trait> OnKilledAccount<T::AccountId> for Module<T> {
    fn on_killed_account(who: &T::AccountId) {
        Account::<T>::remove(who);
    }
}

impl<T: Trait> LockableCurrency<T::AccountId> for Module<T>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    type Moment = T::BlockNumber;

    // Set a lock on the balance of `who`.
    // Is a no-op if lock amount is zero or `reasons` `is_none()`.
    fn set_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        reasons: WithdrawReasons,
    ) {
        if amount.is_zero() || reasons.is_none() {
            return;
        }
        let mut new_lock = Some(BalanceLock {
            id,
            amount,
            reasons: reasons.into(),
        });
        let mut locks = Self::locks(who)
            .into_iter()
            .filter_map(|l| if l.id == id { new_lock.take() } else { Some(l) })
            .collect::<Vec<_>>();
        if let Some(lock) = new_lock {
            locks.push(lock)
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
        if amount.is_zero() || reasons.is_none() {
            return;
        }
        let mut new_lock = Some(BalanceLock {
            id,
            amount,
            reasons: reasons.into(),
        });
        let mut locks = Self::locks(who)
            .into_iter()
            .filter_map(|l| {
                if l.id == id {
                    new_lock.take().map(|nl| BalanceLock {
                        id: l.id,
                        amount: l.amount.max(nl.amount),
                        reasons: l.reasons | nl.reasons,
                    })
                } else {
                    Some(l)
                }
            })
            .collect::<Vec<_>>();
        if let Some(lock) = new_lock {
            locks.push(lock)
        }
        Self::update_locks(who, &locks[..]);
    }

    fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
        let mut locks = Self::locks(who);
        locks.retain(|l| l.id != id);
        Self::update_locks(who, &locks[..]);
    }
}

impl<T: Trait> IsDeadAccount<T::AccountId> for Module<T>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    fn is_dead_account(who: &T::AccountId) -> bool {
        // this should always be exactly equivalent to `Self::account(who).total().is_zero()`
        !T::AccountStore::is_explicit(who)
    }
}
