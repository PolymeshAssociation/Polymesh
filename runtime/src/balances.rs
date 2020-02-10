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

// Modified by Polymath Inc - 18th November 2019
// Added ability to manage balances of identities with the balances module
// In Polymesh, POLY balances can be held at either the identity or account level

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
//! This is a modified implementation of substrate's balances SRML.
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
//! ### Signed Extensions
//!
//! The balances module defines the following extensions:
//!
//!   - [`TakeFees`]: Consumes fees proportional to the length and weight of the transaction.
//!
//!
//! ## Usage
//!
//! The following examples show how to use the Balances module in your custom module.
//!
//! ### Examples from the SRML
//!
//! The Contract module uses the `Currency` trait to handle gas payment, and its types inherit from `Currency`:
//!
//! ```
//! use frame_support::traits::Currency;
//! # pub trait Trait: frame_system::Trait {
//! # 	type Currency: Currency<Self::AccountId>;
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
//! 	type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
//! }
//! # struct StakingLedger<T: Trait> {
//! # 	stash: <T as frame_system::Trait>::AccountId,
//! # 	total: <<T as Trait>::Currency as frame_support::traits::Currency<<T as frame_system::Trait>::AccountId>>::Balance,
//! # 	phantom: std::marker::PhantomData<T>,
//! # }
//! # const STAKING_ID: [u8; 8] = *b"staking ";
//!
//! fn update_ledger<T: Trait>(
//! 	controller: &T::AccountId,
//! 	ledger: &StakingLedger<T>
//! ) {
//! 	T::Currency::set_lock(
//! 		STAKING_ID,
//! 		&ledger.stash,
//! 		ledger.total,
//! 		T::BlockNumber::max_value(),
//! 		WithdrawReasons::all()
//! 	);
//! 	// <Ledger<T>>::insert(controller, ledger); // Commented out as we don't have access to Staking's storage here.
//! }
//! # fn main() {}
//! ```
//!
//! ## Assumptions
//!
//! * Total issued balanced of all accounts should be less than `Trait::Balance::max_value()`.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use frame_support::traits::{
    Currency, ExistenceRequirement, Get, Imbalance, LockIdentifier, LockableCurrency,
    OnFreeBalanceZero, OnUnbalanced, ReservableCurrency, SignedImbalance, TryDrop,
    UpdateBalanceOutcome, VestingCurrency, WithdrawReason, WithdrawReasons,
};
use frame_support::weights::SimpleDispatchInfo;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    Parameter, StorageValue,
};
use frame_system::{self as system, ensure_root, ensure_signed, IsDeadAccount, OnNewAccount};
use sp_runtime::traits::{
    Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, Saturating,
    SimpleArithmetic, StaticLookup, Zero,
};
use sp_runtime::RuntimeDebug;
use sp_std::{cmp, convert::TryFrom, fmt::Debug, mem, prelude::*, result};

use crate::identity::IdentityTrait;
use primitives::{traits::IdentityCurrency, IdentityId, Key, Permission, Signer};

pub use self::imbalances::{NegativeImbalance, PositiveImbalance};

pub trait Subtrait<I: Instance = DefaultInstance>: frame_system::Trait {
    /// The balance of an account.
    type Balance: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + From<u128>
        + From<Self::BlockNumber>;

    /// This type is no longer needed but kept for compatibility reasons.
    /// A function that is invoked when the free-balance has fallen below the existential deposit and
    /// has been reduced to zero.
    ///
    /// Gives a chance to clean up resources associated with the given account.
    type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

    /// Handler for when a new account is created.
    type OnNewAccount: OnNewAccount<Self::AccountId>;

    /// This type is no longer needed but kept for compatibility reasons.
    /// The minimum amount required to keep an account open.
    type ExistentialDeposit: Get<Self::Balance>;

    /// The fee required to make a transfer.
    type TransferFee: Get<Self::Balance>;

    /// The fee required to create an account.
    type CreationFee: Get<Self::Balance>;

    /// Used to charge fee to identity rather than user directly
    type Identity: IdentityTrait<Self::Balance>;
}

pub trait Trait<I: Instance = DefaultInstance>: frame_system::Trait {
    /// The balance of an account.
    type Balance: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + From<u128>
        + From<Self::BlockNumber>;

    /// This type is no longer needed but kept for compatibility reasons.
    /// A function that is invoked when the free-balance has fallen below the existential deposit and
    /// has been reduced to zero.
    ///
    /// Gives a chance to clean up resources associated with the given account.
    type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

    /// Handler for when a new account is created.
    type OnNewAccount: OnNewAccount<Self::AccountId>;

    /// Handler for the unbalanced reduction when taking fees associated with balance
    /// transfer (which may also include account creation).
    type TransferPayment: OnUnbalanced<NegativeImbalance<Self, I>>;

    /// Handler for the unbalanced reduction when removing a dust account.
    type DustRemoval: OnUnbalanced<NegativeImbalance<Self, I>>;

    /// The overarching event type.
    type Event: From<Event<Self, I>> + Into<<Self as frame_system::Trait>::Event>;

    /// This type is no longer needed but kept for compatibility reasons.
    /// The minimum amount required to keep an account open.
    type ExistentialDeposit: Get<Self::Balance>;

    /// The fee required to make a transfer.
    type TransferFee: Get<Self::Balance>;

    /// The fee required to create an account.
    type CreationFee: Get<Self::Balance>;

    /// Used to charge fee to identity rather than user directly
    type Identity: IdentityTrait<Self::Balance>;
}

impl<T: Trait<I>, I: Instance> Subtrait<I> for T {
    type Balance = T::Balance;
    type OnFreeBalanceZero = T::OnFreeBalanceZero;
    type OnNewAccount = T::OnNewAccount;
    type ExistentialDeposit = T::ExistentialDeposit;
    type TransferFee = T::TransferFee;
    type CreationFee = T::CreationFee;
    type Identity = T::Identity;
}

decl_event!(
	pub enum Event<T, I: Instance = DefaultInstance> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait<I>>::Balance
	{
		/// A new account was created.
		NewAccount(AccountId, Balance),
		/// An account was reaped.
		ReapedAccount(AccountId),
		/// Transfer succeeded (from, to, value, fees).
		Transfer(AccountId, AccountId, Balance, Balance),
        /// A balance was set by root (who, free, reserved).
		BalanceSet(AccountId, Balance, Balance),
		/// Some amount was deposited (e.g. for transaction fees).
		Deposit(AccountId, Balance),
	}
);

decl_error! {
    pub enum Error for Module<T: Trait<I>, I: Instance> {
        /// Vesting balance too high to send value
        VestingBalance,
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
        /// A vesting schedule already exists for this account
        ExistingVestingSchedule,
        /// Beneficiary account must pre-exist
        DeadAccount,
        /// AccountId is not attached with Identity
        UnAuthorized,
    }
}

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct VestingSchedule<Balance, BlockNumber> {
    /// Locked amount at genesis.
    pub locked: Balance,
    /// Amount that gets unlocked every block after `starting_block`.
    pub per_block: Balance,
    /// Starting block for unlocking(vesting).
    pub starting_block: BlockNumber,
}

impl<Balance: SimpleArithmetic + Copy, BlockNumber: SimpleArithmetic + Copy>
    VestingSchedule<Balance, BlockNumber>
{
    /// Amount locked at block `n`.
    pub fn locked_at(&self, n: BlockNumber) -> Balance
    where
        Balance: From<BlockNumber>,
    {
        // Number of blocks that count toward vesting
        // Saturating to 0 when n < starting_block
        let vested_block_count = n.saturating_sub(self.starting_block);
        // Return amount that is still locked in vesting
        if let Some(x) = Balance::from(vested_block_count).checked_mul(&self.per_block) {
            self.locked.max(x) - x
        } else {
            Zero::zero()
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct BalanceLock<Balance, BlockNumber> {
    pub id: LockIdentifier,
    pub amount: Balance,
    pub until: BlockNumber,
    pub reasons: WithdrawReasons,
}

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as Balances {
        /// The total units issued in the system.
        pub TotalIssuance get(fn total_issuance) build(|config: &GenesisConfig<T, I>| {
            config.balances.iter().fold(Zero::zero(), |acc: T::Balance, &(_, n)| acc + n)
        }): T::Balance;

        /// Information regarding the vesting of a given account.
        pub Vesting get(fn vesting) build(|config: &GenesisConfig<T, I>| {
            // Generate initial vesting configuration
            // * who - Account which we are generating vesting configuration for
            // * begin - Block when the account will start to vest
            // * length - Number of blocks from `begin` until fully vested
            // * liquid - Number of units which can be spent before vesting begins
            config.vesting.iter().filter_map(|&(ref who, begin, length, liquid)| {
                let length = <T::Balance as From<T::BlockNumber>>::from(length);

                config.balances.iter()
                    .find(|&&(ref w, _)| w == who)
                    .map(|&(_, balance)| {
                        // Total genesis `balance` minus `liquid` equals funds locked for vesting
                        let locked = balance.saturating_sub(liquid);
                        // Number of units unlocked per block after `begin`
                        let per_block = locked / length.max(sp_runtime::traits::One::one());

                        (who.clone(), VestingSchedule {
                            locked: locked,
                            per_block: per_block,
                            starting_block: begin
                        })
                    })
            }).collect::<Vec<_>>()
        }): map T::AccountId => Option<VestingSchedule<T::Balance, T::BlockNumber>>;

        /// The 'free' balance of a given account.
        ///
        /// This is the only balance that matters in terms of most operations on tokens. It
        /// alone is used to determine the balance when in the contract execution environment.
        pub FreeBalance get(fn free_balance)
            build(|config: &GenesisConfig<T, I>| config.balances.clone()):
            map T::AccountId => T::Balance;

        /// The amount of the balance of a given account that is externally reserved; this can still get
        /// slashed, but gets slashed last of all.
        ///
        /// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
        /// that are still 'owned' by the account holder, but which are suspendable.
        pub ReservedBalance get(fn reserved_balance): map T::AccountId => T::Balance;

        /// Any liquidity locks on some account balances.
        pub Locks get(fn locks): map T::AccountId => Vec<BalanceLock<T::Balance, T::BlockNumber>>;

        /// Balance held by the identity. It can be spent by its signing keys.
        pub IdentityBalance get(identity_balance): map IdentityId => T::Balance;

        /// Signing key => Charge Fee to did?. Default is false i.e. the fee will be charged from user balance
        pub ChargeDid get(charge_did): map Key => bool;
    }
    add_extra_genesis {
        config(balances): Vec<(T::AccountId, T::Balance)>;
        config(vesting): Vec<(T::AccountId, T::BlockNumber, T::BlockNumber, T::Balance)>;
        // ^^ begin, length, amount liquid at genesis
    }
}

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance = DefaultInstance> for enum Call where origin: T::Origin {
        type Error = Error<T, I>;

        /// This is no longer needede but kept for compatibility reasons
        /// The minimum amount required to keep an account open.
        const ExistentialDeposit: T::Balance = 0.into();

        /// The fee required to make a transfer.
        const TransferFee: T::Balance = T::TransferFee::get();

        /// The fee required to create an account.
        const CreationFee: T::Balance = T::CreationFee::get();

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
            <Self as Currency<_>>::transfer(&transactor, &dest, value, ExistenceRequirement::AllowDeath)?;
        }

        /// Move some poly from balance of self to balance of an identity.
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn top_up_identity_balance(
            origin,
            did: IdentityId,
            #[compact] value: T::Balance
        ) {
            let transactor = ensure_signed(origin)?;
            match <Self as Currency<_>>::withdraw(
                &transactor,
                value,
                WithdrawReason::TransactionPayment.into(),
                ExistenceRequirement::KeepAlive,
            ) {
                Ok(_) => {
                    let new_balance = Self::identity_balance(&did) + value;
                    <IdentityBalance<T, I>>::insert(did, new_balance);
                    return Ok(())
                },
                Err(err) => return Err(err),
            };
        }

        /// Claim back poly from an identity. Can only be called by master key of the identity.
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn reclaim_identity_balance(
            origin,
            did: IdentityId,
            #[compact] value: T::Balance
        ) {
            let transactor = ensure_signed(origin)?;
            let encoded_transactor = Key::try_from(transactor.encode())?;
            if !<T::Identity>::is_master_key(did, &encoded_transactor) { return Err (Error::<T, I>::UnAuthorized)?}
            // Not managing imbalances because they will cancel out.
            // withdraw function will create negative imbalance and
            // deposit function will create positive imbalance
            let _ = Self::withdraw_identity_balance(&did, value)?;
            let _ = <Self as Currency<_>>::deposit_creating(&transactor, value);
            return Ok(())
        }

        /// Change setting that governs if user pays fee via their own balance or identity's balance.
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        pub fn change_charge_did_flag(origin, charge_did: bool) {
            let transactor = ensure_signed(origin)?;
            let encoded_transactor = Key::try_from(transactor.encode())?;
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

            let current_free = <FreeBalance<T, I>>::get(&who);
            if new_free > current_free {
                mem::drop(PositiveImbalance::<T, I>::new(new_free - current_free));
            } else if new_free < current_free {
                mem::drop(NegativeImbalance::<T, I>::new(current_free - new_free));
            }
            Self::set_free_balance(&who, new_free);

            let current_reserved = <ReservedBalance<T, I>>::get(&who);
            if new_reserved > current_reserved {
                mem::drop(PositiveImbalance::<T, I>::new(new_reserved - current_reserved));
            } else if new_reserved < current_reserved {
                mem::drop(NegativeImbalance::<T, I>::new(current_reserved - new_reserved));
            }
            Self::set_reserved_balance(&who, new_reserved);
        }

        /// Exactly as `transfer`, except the origin must be root and the source account may be
        /// specified.
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
            <Self as Currency<_>>::transfer(&source, &dest, value, ExistenceRequirement::AllowDeath)?;
        }
    }
}

impl<T: Trait<I>, I: Instance> Module<T, I> {
    //type Error = Error<T, I>;

    // PRIVATE MUTABLES

    /// Set the reserved balance of an account to some new value.
    ///
    /// Doesn't do any preparatory work for creating a new account, so should only be used when it
    /// is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn set_reserved_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
        <ReservedBalance<T, I>>::insert(who, balance);
        UpdateBalanceOutcome::Updated
    }

    /// Set the free balance of an account to some new value.
    ///
    /// Doesn't do any preparatory work for creating a new account, so should only be used when it
    /// is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn set_free_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
        // Commented out for now - but consider it instructive.
        // assert!(!Self::total_balance(who).is_zero());
        <FreeBalance<T, I>>::insert(who, balance);
        UpdateBalanceOutcome::Updated
    }

    /// Register a new account
    ///
    /// This just calls appropriate hooks. It doesn't (necessarily) make any state changes.
    fn new_account(who: &T::AccountId, balance: T::Balance) {
        T::OnNewAccount::on_new_account(&who);
        Self::deposit_event(RawEvent::NewAccount(who.clone(), balance));
    }
}

// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.
mod imbalances {
    use super::{
        result, DefaultInstance, Imbalance, Instance, Saturating, StorageValue, Subtrait, Trait,
        TryDrop, Zero,
    };
    use sp_std::mem;

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been created without any equal and opposite accounting.
    #[must_use]
    pub struct PositiveImbalance<T: Subtrait<I>, I: Instance = DefaultInstance>(T::Balance);

    impl<T: Subtrait<I>, I: Instance> PositiveImbalance<T, I> {
        /// Create a new positive imbalance from a balance.
        pub fn new(amount: T::Balance) -> Self {
            PositiveImbalance(amount)
        }
    }

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been destroyed without any equal and opposite accounting.
    #[must_use]
    pub struct NegativeImbalance<T: Subtrait<I>, I: Instance = DefaultInstance>(T::Balance);

    impl<T: Subtrait<I>, I: Instance> NegativeImbalance<T, I> {
        /// Create a new negative imbalance from a balance.
        pub fn new(amount: T::Balance) -> Self {
            NegativeImbalance(amount)
        }
    }

    impl<T: Trait<I>, I: Instance> TryDrop for PositiveImbalance<T, I> {
        fn try_drop(self) -> result::Result<(), Self> {
            self.drop_zero()
        }
    }

    impl<T: Trait<I>, I: Instance> Imbalance<T::Balance> for PositiveImbalance<T, I> {
        type Opposite = NegativeImbalance<T, I>;

        fn zero() -> Self {
            Self(Zero::zero())
        }
        fn drop_zero(self) -> result::Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }
        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;

            mem::forget(self);
            (Self(first), Self(second))
        }
        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);

            self
        }
        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
        }
        fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            mem::forget((self, other));

            if a >= b {
                Ok(Self(a - b))
            } else {
                Err(NegativeImbalance::new(b - a))
            }
        }
        fn peek(&self) -> T::Balance {
            self.0.clone()
        }
    }

    impl<T: Trait<I>, I: Instance> TryDrop for NegativeImbalance<T, I> {
        fn try_drop(self) -> result::Result<(), Self> {
            self.drop_zero()
        }
    }

    impl<T: Trait<I>, I: Instance> Imbalance<T::Balance> for NegativeImbalance<T, I> {
        type Opposite = PositiveImbalance<T, I>;

        fn zero() -> Self {
            Self(Zero::zero())
        }
        fn drop_zero(self) -> result::Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }
        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;

            mem::forget(self);
            (Self(first), Self(second))
        }
        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);

            self
        }
        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            mem::forget(other);
        }
        fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            mem::forget((self, other));

            if a >= b {
                Ok(Self(a - b))
            } else {
                Err(PositiveImbalance::new(b - a))
            }
        }
        fn peek(&self) -> T::Balance {
            self.0.clone()
        }
    }

    impl<T: Subtrait<I>, I: Instance> Drop for PositiveImbalance<T, I> {
        /// Basic drop handler will just square up the total issuance.
        fn drop(&mut self) {
            <super::TotalIssuance<super::ElevatedTrait<T, I>, I>>::mutate(|v| {
                *v = v.saturating_add(self.0)
            });
        }
    }

    impl<T: Subtrait<I>, I: Instance> Drop for NegativeImbalance<T, I> {
        /// Basic drop handler will just square up the total issuance.
        fn drop(&mut self) {
            <super::TotalIssuance<super::ElevatedTrait<T, I>, I>>::mutate(|v| {
                *v = v.saturating_sub(self.0)
            });
        }
    }
}

// TODO: #2052
// Somewhat ugly hack in order to gain access to module's `increase_total_issuance_by`
// using only the Subtrait (which defines only the types that are not dependent
// on Positive/NegativeImbalance). Subtrait must be used otherwise we end up with a
// circular dependency with Trait having some types be dependent on PositiveImbalance<Trait>
// and PositiveImbalance itself depending back on Trait for its Drop impl (and thus
// its type declaration).
// This works as long as `increase_total_issuance_by` doesn't use the Imbalance
// types (basically for charging fees).
// This should eventually be refactored so that the three type items that do
// depend on the Imbalance type (TransactionPayment, TransferPayment, DustRemoval)
// are placed in their own SRML module.
struct ElevatedTrait<T: Subtrait<I>, I: Instance>(T, I);
impl<T: Subtrait<I>, I: Instance> Clone for ElevatedTrait<T, I> {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}
impl<T: Subtrait<I>, I: Instance> PartialEq for ElevatedTrait<T, I> {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!()
    }
}
impl<T: Subtrait<I>, I: Instance> Eq for ElevatedTrait<T, I> {}
impl<T: Subtrait<I>, I: Instance> frame_system::Trait for ElevatedTrait<T, I> {
    type Origin = T::Origin;
    type Call = T::Call;
    type Index = T::Index;
    type BlockNumber = T::BlockNumber;
    type Hash = T::Hash;
    type Hashing = T::Hashing;
    type AccountId = T::AccountId;
    type Lookup = T::Lookup;
    type Header = T::Header;
    type Event = ();
    type BlockHashCount = T::BlockHashCount;
    type MaximumBlockWeight = T::MaximumBlockWeight;
    type MaximumBlockLength = T::MaximumBlockLength;
    type AvailableBlockRatio = T::AvailableBlockRatio;
    type Version = T::Version;
    type ModuleToIndex = T::ModuleToIndex;
}
impl<T: Subtrait<I>, I: Instance> Trait<I> for ElevatedTrait<T, I> {
    type Balance = T::Balance;
    type OnFreeBalanceZero = T::OnFreeBalanceZero;
    type OnNewAccount = T::OnNewAccount;
    type Event = ();
    type TransferPayment = ();
    type DustRemoval = ();
    type ExistentialDeposit = T::ExistentialDeposit;
    type TransferFee = T::TransferFee;
    type CreationFee = T::CreationFee;
    type Identity = T::Identity;
}

impl<T: Trait<I>, I: Instance> Currency<T::AccountId> for Module<T, I>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    type Balance = T::Balance;
    type PositiveImbalance = PositiveImbalance<T, I>;
    type NegativeImbalance = NegativeImbalance<T, I>;

    fn total_balance(who: &T::AccountId) -> Self::Balance {
        Self::free_balance(who) + Self::reserved_balance(who)
    }

    fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
        Self::free_balance(who) >= value
    }

    fn total_issuance() -> Self::Balance {
        <TotalIssuance<T, I>>::get()
    }

    fn minimum_balance() -> Self::Balance {
        0u128.into()
    }

    fn free_balance(who: &T::AccountId) -> Self::Balance {
        <FreeBalance<T, I>>::get(who)
    }

    fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
        <TotalIssuance<T, I>>::mutate(|issued| {
            *issued = issued.checked_sub(&amount).unwrap_or_else(|| {
                amount = *issued;
                Zero::zero()
            });
        });
        PositiveImbalance::new(amount)
    }

    fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
        <TotalIssuance<T, I>>::mutate(|issued| {
            *issued = issued.checked_add(&amount).unwrap_or_else(|| {
                amount = Self::Balance::max_value() - *issued;
                Self::Balance::max_value()
            })
        });
        NegativeImbalance::new(amount)
    }

    // # <weight>
    // Despite iterating over a list of locks, they are limited by the number of
    // lock IDs, which means the number of runtime modules that intend to use and create locks.
    // # </weight>
    fn ensure_can_withdraw(
        who: &T::AccountId,
        _amount: T::Balance,
        reasons: WithdrawReasons,
        new_balance: T::Balance,
    ) -> DispatchResult {
        if reasons.intersects(WithdrawReason::Reserve | WithdrawReason::Transfer)
            && Self::vesting_balance(who) > new_balance
        {
            Err(Error::<T, I>::VestingBalance)?
        }
        let locks = Self::locks(who);
        if locks.is_empty() {
            return Ok(());
        }

        let now = <frame_system::Module<T>>::block_number();
        if locks
            .into_iter()
            .all(|l| now >= l.until || new_balance >= l.amount || !l.reasons.intersects(reasons))
        {
            Ok(())
        } else {
            Err(Error::<T, I>::LiquidityRestrictions.into())
        }
    }

    fn transfer(
        transactor: &T::AccountId,
        dest: &T::AccountId,
        value: Self::Balance,
        _existence_requirement: ExistenceRequirement,
    ) -> DispatchResult {
        let from_balance = Self::free_balance(transactor);
        let to_balance = Self::free_balance(dest);
        let would_create = to_balance.is_zero();
        let fee = if would_create {
            T::CreationFee::get()
        } else {
            T::TransferFee::get()
        };
        let liability = value.checked_add(&fee).ok_or(Error::<T, I>::Overflow)?;
        let new_from_balance = from_balance
            .checked_sub(&liability)
            .ok_or(Error::<T, I>::InsufficientBalance)?;

        Self::ensure_can_withdraw(
            transactor,
            value,
            WithdrawReason::Transfer.into(),
            new_from_balance,
        )?;

        // NOTE: total stake being stored in the same type means that this could never overflow
        // but better to be safe than sorry.
        let new_to_balance = to_balance
            .checked_add(&value)
            .ok_or(Error::<T, I>::Overflow)?;

        if transactor != dest {
            Self::set_free_balance(transactor, new_from_balance);
            if !<FreeBalance<T, I>>::exists(dest) {
                Self::new_account(dest, new_to_balance);
            }

            Self::set_free_balance(dest, new_to_balance);
            T::TransferPayment::on_unbalanced(NegativeImbalance::new(fee));

            Self::deposit_event(RawEvent::Transfer(
                transactor.clone(),
                dest.clone(),
                value,
                fee,
            ));
        }

        Ok(())
    }

    fn withdraw(
        who: &T::AccountId,
        value: Self::Balance,
        reasons: WithdrawReasons,
        _liveness: ExistenceRequirement,
    ) -> result::Result<Self::NegativeImbalance, DispatchError> {
        if let Some(new_balance) = Self::free_balance(who).checked_sub(&value) {
            Self::ensure_can_withdraw(who, value, reasons, new_balance)?;
            Self::set_free_balance(who, new_balance);
            Ok(NegativeImbalance::new(value))
        } else {
            Err(Error::<T, I>::InsufficientBalance)?
        }
    }

    fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        let free_balance = Self::free_balance(who);
        let free_slash = cmp::min(free_balance, value);
        Self::set_free_balance(who, free_balance - free_slash);
        let remaining_slash = value - free_slash;
        // NOTE: `slash()` prefers free balance, but assumes that reserve balance can be drawn
        // from in extreme circumstances. `can_slash()` should be used prior to `slash()` to avoid having
        // to draw from reserved funds, however we err on the side of punishment if things are inconsistent
        // or `can_slash` wasn't used appropriately.
        if !remaining_slash.is_zero() {
            let reserved_balance = Self::reserved_balance(who);
            let reserved_slash = cmp::min(reserved_balance, remaining_slash);
            Self::set_reserved_balance(who, reserved_balance - reserved_slash);
            (
                NegativeImbalance::new(free_slash + reserved_slash),
                remaining_slash - reserved_slash,
            )
        } else {
            (NegativeImbalance::new(value), Zero::zero())
        }
    }

    fn deposit_into_existing(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> result::Result<Self::PositiveImbalance, DispatchError> {
        if Self::total_balance(who).is_zero() {
            Err(Error::<T, I>::DeadAccount)?
        }
        Self::set_free_balance(who, Self::free_balance(who) + value);
        Ok(PositiveImbalance::new(value))
    }

    fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        let (imbalance, _) = Self::make_free_balance_be(who, Self::free_balance(who) + value);
        if let SignedImbalance::Positive(p) = imbalance {
            p
        } else {
            // Impossible, but be defensive.
            Self::PositiveImbalance::zero()
        }
    }

    fn make_free_balance_be(
        who: &T::AccountId,
        balance: Self::Balance,
    ) -> (
        SignedImbalance<Self::Balance, Self::PositiveImbalance>,
        UpdateBalanceOutcome,
    ) {
        let original = Self::free_balance(who);
        let imbalance = if original <= balance {
            SignedImbalance::Positive(PositiveImbalance::new(balance - original))
        } else {
            SignedImbalance::Negative(NegativeImbalance::new(original - balance))
        };
        if !<FreeBalance<T, I>>::exists(who) {
            Self::new_account(&who, balance);
        }
        Self::set_free_balance(who, balance);
        (imbalance, UpdateBalanceOutcome::Updated)
    }
}

impl<T: Trait<I>, I: Instance> IdentityCurrency<T::AccountId> for Module<T, I>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    fn withdraw_identity_balance(
        who: &IdentityId,
        value: Self::Balance,
    ) -> result::Result<Self::NegativeImbalance, DispatchError> {
        if let Some(new_balance) = Self::identity_balance(who).checked_sub(&value) {
            <IdentityBalance<T, I>>::insert(who, new_balance);
            Ok(NegativeImbalance::new(value))
        } else {
            Err(Error::<T, I>::Overflow)?
        }
    }

    fn charge_fee_to_identity(who: &Key) -> Option<IdentityId> {
        if <Module<T, I>>::charge_did(who) {
            if let Some(did) = <T::Identity>::get_identity(&who) {
                if <T::Identity>::is_signer_authorized_with_permissions(
                    did,
                    &Signer::Key(who.clone()),
                    vec![Permission::SpendFunds],
                ) {
                    return Some(did);
                }
            }
        }
        return None;
    }
}

impl<T: Trait<I>, I: Instance> ReservableCurrency<T::AccountId> for Module<T, I>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
        Self::free_balance(who)
            .checked_sub(&value)
            .map_or(false, |new_balance| {
                Self::ensure_can_withdraw(who, value, WithdrawReason::Reserve.into(), new_balance)
                    .is_ok()
            })
    }

    fn reserved_balance(who: &T::AccountId) -> Self::Balance {
        <ReservedBalance<T, I>>::get(who)
    }

    fn reserve(who: &T::AccountId, value: Self::Balance) -> result::Result<(), DispatchError> {
        let b = Self::free_balance(who);
        if b < value {
            Err(Error::<T, I>::InsufficientBalance)?
        }
        let new_balance = b - value;
        Self::ensure_can_withdraw(who, value, WithdrawReason::Reserve.into(), new_balance)?;
        Self::set_reserved_balance(who, Self::reserved_balance(who) + value);
        Self::set_free_balance(who, new_balance);
        Ok(())
    }

    fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
        let b = Self::reserved_balance(who);
        let actual = cmp::min(b, value);
        Self::set_free_balance(who, Self::free_balance(who) + actual);
        Self::set_reserved_balance(who, b - actual);
        value - actual
    }

    fn slash_reserved(
        who: &T::AccountId,
        value: Self::Balance,
    ) -> (Self::NegativeImbalance, Self::Balance) {
        let b = Self::reserved_balance(who);
        let slash = cmp::min(b, value);
        // underflow should never happen, but it if does, there's nothing to be done here.
        Self::set_reserved_balance(who, b - slash);
        (NegativeImbalance::new(slash), value - slash)
    }

    fn repatriate_reserved(
        slashed: &T::AccountId,
        beneficiary: &T::AccountId,
        value: Self::Balance,
    ) -> result::Result<Self::Balance, DispatchError> {
        if Self::total_balance(beneficiary).is_zero() {
            Err(Error::<T, I>::DeadAccount)?
        }
        let b = Self::reserved_balance(slashed);
        let slash = cmp::min(b, value);
        Self::set_free_balance(beneficiary, Self::free_balance(beneficiary) + slash);
        Self::set_reserved_balance(slashed, b - slash);
        Ok(value - slash)
    }
}

impl<T: Trait<I>, I: Instance> LockableCurrency<T::AccountId> for Module<T, I>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    type Moment = T::BlockNumber;

    fn set_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        until: T::BlockNumber,
        reasons: WithdrawReasons,
    ) {
        let now = <frame_system::Module<T>>::block_number();
        let mut new_lock = Some(BalanceLock {
            id,
            amount,
            until,
            reasons,
        });
        let mut locks = Self::locks(who)
            .into_iter()
            .filter_map(|l| {
                if l.id == id {
                    new_lock.take()
                } else if l.until > now {
                    Some(l)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if let Some(lock) = new_lock {
            locks.push(lock)
        }
        <Locks<T, I>>::insert(who, locks);
    }

    fn extend_lock(
        id: LockIdentifier,
        who: &T::AccountId,
        amount: T::Balance,
        until: T::BlockNumber,
        reasons: WithdrawReasons,
    ) {
        let now = <frame_system::Module<T>>::block_number();
        let mut new_lock = Some(BalanceLock {
            id,
            amount,
            until,
            reasons,
        });
        let mut locks = Self::locks(who)
            .into_iter()
            .filter_map(|l| {
                if l.id == id {
                    new_lock.take().map(|nl| BalanceLock {
                        id: l.id,
                        amount: l.amount.max(nl.amount),
                        until: l.until.max(nl.until),
                        reasons: l.reasons | nl.reasons,
                    })
                } else if l.until > now {
                    Some(l)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if let Some(lock) = new_lock {
            locks.push(lock)
        }
        <Locks<T, I>>::insert(who, locks);
    }

    fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
        let now = <frame_system::Module<T>>::block_number();
        let locks = Self::locks(who)
            .into_iter()
            .filter_map(|l| {
                if l.until > now && l.id != id {
                    Some(l)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        <Locks<T, I>>::insert(who, locks);
    }
}

impl<T: Trait<I>, I: Instance> VestingCurrency<T::AccountId> for Module<T, I>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    type Moment = T::BlockNumber;

    /// Get the amount that is currently being vested and cannot be transferred out of this account.
    fn vesting_balance(who: &T::AccountId) -> T::Balance {
        if let Some(v) = Self::vesting(who) {
            Self::free_balance(who).min(v.locked_at(<frame_system::Module<T>>::block_number()))
        } else {
            Zero::zero()
        }
    }

    /// Adds a vesting schedule to a given account.
    ///
    /// If there already exists a vesting schedule for the given account, an `Err` is returned
    /// and nothing is updated.
    fn add_vesting_schedule(
        who: &T::AccountId,
        locked: T::Balance,
        per_block: T::Balance,
        starting_block: T::BlockNumber,
    ) -> DispatchResult {
        if <Vesting<T, I>>::exists(who) {
            Err(Error::<T, I>::ExistingVestingSchedule)?
        }
        let vesting_schedule = VestingSchedule {
            locked,
            per_block,
            starting_block,
        };
        <Vesting<T, I>>::insert(who, vesting_schedule);
        Ok(())
    }

    /// Remove a vesting schedule for a given account.
    fn remove_vesting_schedule(who: &T::AccountId) {
        <Vesting<T, I>>::remove(who);
    }
}

impl<T: Trait<I>, I: Instance> IsDeadAccount<T::AccountId> for Module<T, I>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    fn is_dead_account(who: &T::AccountId) -> bool {
        Self::total_balance(who).is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        assert_err, assert_ok,
        dispatch::DispatchResult,
        impl_outer_origin, parameter_types,
        traits::Get,
        weights::{DispatchInfo, Weight},
    };
    use frame_system::EnsureSignedBy;
    use pallet_transaction_payment::ChargeTransactionPayment;
    use sp_core::H256;
    use sp_io::{self};
    use sp_runtime::{
        testing::Header,
        traits::{Convert, IdentityLookup, SignedExtension, Verify},
        AnySignature, Perbill,
    };
    use std::{cell::RefCell, result::Result};
    use test_client::AccountKeyring;

    use crate::{group, identity};

    impl_outer_origin! {
        pub enum Origin for Runtime {}
    }

    thread_local! {
        static EXISTENTIAL_DEPOSIT: RefCell<u128> = RefCell::new(0);
        static TRANSFER_FEE: RefCell<u128> = RefCell::new(0);
        static CREATION_FEE: RefCell<u128> = RefCell::new(0);
    }

    pub struct ExistentialDeposit;
    impl Get<u128> for ExistentialDeposit {
        fn get() -> u128 {
            EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
        }
    }

    pub struct TransferFee;
    impl Get<u128> for TransferFee {
        fn get() -> u128 {
            TRANSFER_FEE.with(|v| *v.borrow())
        }
    }

    pub struct CreationFee;
    impl Get<u128> for CreationFee {
        fn get() -> u128 {
            CREATION_FEE.with(|v| *v.borrow())
        }
    }

    // Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Runtime;
    type AccountId = <AnySignature as Verify>::Signer;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: u32 = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
        pub const MinimumPeriod: u64 = 3;
    }

    impl frame_system::Trait for Runtime {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Call = ();
        type Hash = H256;
        type Hashing = ::sp_runtime::traits::BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
    }

    parameter_types! {
        pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
    }

    impl group::Trait<group::Instance1> for Runtime {
        type Event = ();
        type AddOrigin = EnsureSignedBy<One, AccountId>;
        type RemoveOrigin = EnsureSignedBy<Two, AccountId>;
        type SwapOrigin = EnsureSignedBy<Three, AccountId>;
        type ResetOrigin = EnsureSignedBy<Four, AccountId>;
        type MembershipInitialized = ();
        type MembershipChanged = ();
    }

    impl identity::Trait for Runtime {
        type Event = ();
        type Proposal = Call<Runtime>;
        type AcceptTransferTarget = Runtime;
        type AddSignerMultiSigTarget = Runtime;
    }
    impl crate::asset::AcceptTransfer for Runtime {
        fn accept_ticker_transfer(_: IdentityId, _: u64) -> DispatchResult {
            unimplemented!()
        }
        fn accept_token_ownership_transfer(_: IdentityId, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }
    impl crate::multisig::AddSignerMultiSig for Runtime {
        fn accept_multisig_signer(_: Signer, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }
    impl pallet_timestamp::Trait for Runtime {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl Trait for Runtime {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type Identity = crate::identity::Module<Runtime>;
    }

    thread_local! {
        static TRANSACTION_BASE_FEE: RefCell<u128> = RefCell::new(0);
        static TRANSACTION_BYTE_FEE: RefCell<u128> = RefCell::new(1);
        static WEIGHT_TO_FEE: RefCell<u128> = RefCell::new(1);
    }

    pub struct TransactionBaseFee;
    impl Get<u128> for TransactionBaseFee {
        fn get() -> u128 {
            TRANSACTION_BASE_FEE.with(|v| *v.borrow())
        }
    }

    pub struct TransactionByteFee;
    impl Get<u128> for TransactionByteFee {
        fn get() -> u128 {
            TRANSACTION_BYTE_FEE.with(|v| *v.borrow())
        }
    }

    pub struct WeightToFee(u128);
    impl Convert<Weight, u128> for WeightToFee {
        fn convert(t: Weight) -> u128 {
            WEIGHT_TO_FEE.with(|v| *v.borrow() * (t as u128))
        }
    }

    impl pallet_transaction_payment::Trait for Runtime {
        type Currency = Module<Runtime>;
        type OnTransactionPayment = ();
        type TransactionBaseFee = TransactionBaseFee;
        type TransactionByteFee = TransactionByteFee;
        type WeightToFee = WeightToFee;
        type FeeMultiplierUpdate = ();
    }

    pub struct ExtBuilder {
        transaction_base_fee: u128,
        transaction_byte_fee: u128,
        weight_to_fee: u128,
        existential_deposit: u128,
        transfer_fee: u128,
        creation_fee: u128,
        monied: bool,
        vesting: bool,
    }
    impl Default for ExtBuilder {
        fn default() -> Self {
            Self {
                transaction_base_fee: 0,
                transaction_byte_fee: 0,
                weight_to_fee: 0,
                existential_deposit: 0,
                transfer_fee: 0,
                creation_fee: 0,
                monied: false,
                vesting: false,
            }
        }
    }
    impl ExtBuilder {
        pub fn transaction_fees(
            mut self,
            base_fee: u128,
            byte_fee: u128,
            weight_fee: u128,
        ) -> Self {
            self.transaction_base_fee = base_fee;
            self.transaction_byte_fee = byte_fee;
            self.weight_to_fee = weight_fee;
            self
        }
        pub fn existential_deposit(mut self, existential_deposit: u128) -> Self {
            self.existential_deposit = existential_deposit;
            self
        }
        #[allow(dead_code)]
        pub fn transfer_fee(mut self, transfer_fee: u128) -> Self {
            self.transfer_fee = transfer_fee;
            self
        }
        pub fn monied(mut self, monied: bool) -> Self {
            self.monied = monied;
            if self.existential_deposit == 0 {
                self.existential_deposit = 1;
            }
            self
        }
        pub fn set_associated_consts(&self) {
            EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
            TRANSFER_FEE.with(|v| *v.borrow_mut() = self.transfer_fee);
            CREATION_FEE.with(|v| *v.borrow_mut() = self.creation_fee);
            TRANSACTION_BASE_FEE.with(|v| *v.borrow_mut() = self.transaction_base_fee);
            TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.transaction_byte_fee);
            WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
        }
        pub fn build(self) -> sp_io::TestExternalities {
            self.set_associated_consts();
            let mut t = frame_system::GenesisConfig::default()
                .build_storage::<Runtime>()
                .unwrap();
            GenesisConfig::<Runtime> {
                balances: if self.monied {
                    vec![
                        (
                            AccountKeyring::Alice.public(),
                            10 * self.existential_deposit,
                        ),
                        (AccountKeyring::Bob.public(), 20 * self.existential_deposit),
                        (
                            AccountKeyring::Charlie.public(),
                            30 * self.existential_deposit,
                        ),
                        (AccountKeyring::Dave.public(), 40 * self.existential_deposit),
                        // (12, 10 * self.existential_deposit),
                    ]
                } else {
                    vec![]
                },
                vesting: if self.vesting && self.monied {
                    vec![
                        (
                            AccountKeyring::Alice.public(),
                            0,
                            10,
                            5 * self.existential_deposit,
                        ),
                        (AccountKeyring::Bob.public(), 10, 20, 0),
                        // (12, 10, 20, 5 * self.existential_deposit),
                    ]
                } else {
                    vec![]
                },
            }
            .assimilate_storage(&mut t)
            .unwrap();
            t.into()
        }
    }

    pub type Balances = Module<Runtime>;
    pub type Identity = identity::Module<Runtime>;
    pub type TransactionPayment = pallet_transaction_payment::Module<Runtime>;

    pub const CALL: &<Runtime as frame_system::Trait>::Call = &();

    /// create a transaction info struct from weight. Handy to avoid building the whole struct.
    pub fn info_from_weight(w: Weight) -> DispatchInfo {
        DispatchInfo {
            weight: w,
            ..Default::default()
        }
    }

    fn make_account(
        account_id: &AccountId,
    ) -> Result<(<Runtime as frame_system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        Identity::register_did(signed_id.clone(), vec![]);
        let did = Identity::get_identity(&Key::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    #[ignore]
    fn signed_extension_charge_transaction_payment_work() {
        ExtBuilder::default()
            .existential_deposit(10)
            .transaction_fees(10, 1, 5)
            .monied(true)
            .build()
            .execute_with(|| {
                let len = 10;
                let alice_pub = AccountKeyring::Alice.public();
                assert!(
                    <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                        ChargeTransactionPayment::from(0),
                        &alice_pub,
                        CALL,
                        info_from_weight(5),
                        len
                    )
                    .is_ok()
                );
                assert_eq!(Balances::free_balance(&alice_pub), 100 - 20 - 25);
                assert!(
                    <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                        ChargeTransactionPayment::from(0 /* 0 tip */),
                        &alice_pub,
                        CALL,
                        info_from_weight(3),
                        len
                    )
                    .is_ok()
                );
                assert_eq!(Balances::free_balance(&alice_pub), 100 - 20 - 25 - 20 - 15);
            });
    }

    #[test]
    fn tipping_fails() {
        ExtBuilder::default()
            .existential_deposit(10)
            .transaction_fees(10, 1, 5)
            .monied(true)
            .build()
            .execute_with(|| {
                let len = 10;
                assert!(
                    <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                        ChargeTransactionPayment::from(5 /* 5 tip */),
                        &AccountKeyring::Alice.public(),
                        CALL,
                        info_from_weight(3),
                        len
                    )
                    .is_err()
                );
            });
    }

    #[test]
    #[ignore]
    fn should_charge_identity() {
        ExtBuilder::default()
            .existential_deposit(10)
            .transaction_fees(10, 1, 5)
            .monied(true)
            .build()
            .execute_with(|| {
                let dave_pub = AccountKeyring::Dave.public();
                let (signed_acc_id, acc_did) = make_account(&dave_pub).unwrap();
                let len = 10;
                assert!(
                    <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                        ChargeTransactionPayment::from(0 /* 0 tip */),
                        &dave_pub,
                        CALL,
                        info_from_weight(3),
                        len
                    )
                    .is_ok()
                );

                assert_ok!(Balances::change_charge_did_flag(
                    signed_acc_id.clone(),
                    true
                ));
                assert!(
                    <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                        ChargeTransactionPayment::from(0 /* 0 tip */),
                        &dave_pub,
                        CALL,
                        info_from_weight(3),
                        len
                    )
                    .is_err()
                ); // no balance in identity
                assert_eq!(Balances::free_balance(&dave_pub), 365);
                assert_ok!(Balances::top_up_identity_balance(
                    signed_acc_id.clone(),
                    acc_did,
                    300
                ));
                assert_eq!(Balances::free_balance(&dave_pub), 65);
                assert_eq!(Balances::identity_balance(acc_did), 300);
                assert!(
                    <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                        ChargeTransactionPayment::from(0 /* 0 tip */),
                        &dave_pub,
                        CALL,
                        info_from_weight(3),
                        len
                    )
                    .is_ok()
                );
                assert_ok!(Balances::reclaim_identity_balance(
                    signed_acc_id.clone(),
                    acc_did,
                    230
                ));
                assert_err!(
                    Balances::reclaim_identity_balance(signed_acc_id, acc_did, 230),
                    "too few free funds in account"
                );
                assert_eq!(Balances::free_balance(&dave_pub), 295);
                assert_eq!(Balances::identity_balance(acc_did), 35);
            });
    }
}
