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
use polymesh_primitives::{
    traits::{BlockRewardsReserveCurrency, IdentityCurrency},
    AccountKey, IdentityId, Permission, Signatory,
};
use polymesh_runtime_common::traits::{
    balances::{BalancesTrait, RawEvent},
    identity::IdentityTrait,
    NegativeImbalance, PositiveImbalance,
};

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    traits::{
        Currency, ExistenceRequirement, Get, Imbalance, LockIdentifier, LockableCurrency,
        OnFreeBalanceZero, OnUnbalanced, ReservableCurrency, SignedImbalance, UpdateBalanceOutcome,
        VestingCurrency, WithdrawReason, WithdrawReasons,
    },
    weights::SimpleDispatchInfo,
    StorageValue,
};

use frame_system::{self as system, ensure_root, ensure_signed, IsDeadAccount, OnNewAccount};
use sp_runtime::{
    traits::{
        Bounded, CheckedAdd, CheckedSub, Hash, MaybeSerializeDeserialize, Saturating,
        SimpleArithmetic, StaticLookup, Zero,
    },
    RuntimeDebug,
};
use sp_std::{cmp, convert::TryFrom, fmt::Debug, mem, prelude::*, result, vec};

pub use polymesh_runtime_common::traits::balances::Trait;

decl_error! {
    pub enum Error for Module<T: Trait> {
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

pub type Event<T> = polymesh_runtime_common::balances::Event<T>;

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
    trait Store for Module<T: Trait> as Balances {
        /// The total units issued in the system.
        pub TotalIssuance get(fn total_issuance) build(|config: &GenesisConfig<T>| {
            config.balances.iter().fold(Zero::zero(), |acc: T::Balance, &(_, n)| acc + n)
        }): T::Balance;

        /// Information regarding the vesting of a given account.
        pub Vesting get(fn vesting) build(|config: &GenesisConfig<T>| {
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
            build(|config: &GenesisConfig<T>| config.balances.clone()):
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
        pub ChargeDid get(charge_did): map AccountKey => bool;

        /// AccountId of the block rewards reserve
        pub BlockRewardsReserve get(block_rewards_reserve) build(|_| {
            let h: T::Hash = T::Hashing::hash(&(b"BLOCK_REWARDS_RESERVE").encode());
            T::AccountId::decode(&mut &h.encode()[..]).unwrap_or_default()
        }): T::AccountId;
    }
    add_extra_genesis {
        config(balances): Vec<(T::AccountId, T::Balance)>;
        config(vesting): Vec<(T::AccountId, T::BlockNumber, T::BlockNumber, T::Balance)>;
        // ^^ begin, length, amount liquid at genesis
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

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
                    <IdentityBalance<T>>::insert(did, new_balance);
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
            let encoded_transactor = AccountKey::try_from(transactor.encode())?;
            if !<T::Identity>::is_master_key(did, &encoded_transactor) {
                return Err (Error::<T>::UnAuthorized)?
            }
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

            let current_free = <FreeBalance<T>>::get(&who);
            if new_free > current_free {
                mem::drop(PositiveImbalance::<T>::new(new_free - current_free));
            } else if new_free < current_free {
                mem::drop(NegativeImbalance::<T>::new(current_free - new_free));
            }
            Self::set_free_balance(&who, new_free);

            let current_reserved = <ReservedBalance<T>>::get(&who);
            if new_reserved > current_reserved {
                mem::drop(PositiveImbalance::<T>::new(new_reserved - current_reserved));
            } else if new_reserved < current_reserved {
                mem::drop(NegativeImbalance::<T>::new(current_reserved - new_reserved));
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

impl<T: Trait> Module<T> {
    // PRIVATE MUTABLES

    /// Set the reserved balance of an account to some new value.
    ///
    /// Doesn't do any preparatory work for creating a new account, so should only be used when it
    /// is known that the account already exists.
    ///
    /// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
    /// the caller will do this.
    fn set_reserved_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
        <ReservedBalance<T>>::insert(who, balance);
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
        <FreeBalance<T>>::insert(who, balance);

        if balance <= Zero::zero() {
            T::OnFreeBalanceZero::on_free_balance_zero(who);
        }

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
        let brr_balance = <FreeBalance<T>>::get(&brr);
        if brr_balance > Zero::zero() {
            let new_brr_balance = brr_balance.saturating_sub(amount);
            amount = amount - (brr_balance - new_brr_balance);
            <FreeBalance<T>>::insert(&brr, new_brr_balance);
        }
        <TotalIssuance<T>>::mutate(|v| *v = v.saturating_add(amount));
    }

    fn drop_negative_imbalance(amount: T::Balance) {
        <TotalIssuance<T>>::mutate(|v| *v = v.saturating_sub(amount));
    }

    fn issue_using_block_rewards_reserve(mut amount: T::Balance) -> NegativeImbalance<T> {
        let brr = <BlockRewardsReserve<T>>::get();
        let brr_balance = <FreeBalance<T>>::get(&brr);
        let amount_to_mint = if brr_balance > Zero::zero() {
            let new_brr_balance = brr_balance.saturating_sub(amount);
            <FreeBalance<T>>::insert(&brr, new_brr_balance);
            amount - (brr_balance - new_brr_balance)
        } else {
            amount
        };
        <TotalIssuance<T>>::mutate(|issued| {
            *issued = issued.checked_add(&amount_to_mint).unwrap_or_else(|| {
                amount = T::Balance::max_value() - *issued;
                T::Balance::max_value()
            })
        });
        NegativeImbalance::new(amount)
    }

    fn block_rewards_reserve_balance() -> T::Balance {
        let brr = Self::block_rewards_reserve();
        Self::free_balance(&brr)
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
        Self::free_balance(who) + Self::reserved_balance(who)
    }

    fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
        Self::free_balance(who) >= value
    }

    fn total_issuance() -> Self::Balance {
        <TotalIssuance<T>>::get()
    }

    fn minimum_balance() -> Self::Balance {
        0u128.into()
    }

    fn free_balance(who: &T::AccountId) -> Self::Balance {
        <FreeBalance<T>>::get(who)
    }

    fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
        <TotalIssuance<T>>::mutate(|issued| {
            *issued = issued.checked_sub(&amount).unwrap_or_else(|| {
                amount = *issued;
                Zero::zero()
            });
        });
        PositiveImbalance::new(amount)
    }

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
            Err(Error::<T>::VestingBalance)?
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
            Err(Error::<T>::LiquidityRestrictions.into())
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
        let liability = value.checked_add(&fee).ok_or(Error::<T>::Overflow)?;
        let new_from_balance = from_balance
            .checked_sub(&liability)
            .ok_or(Error::<T>::InsufficientBalance)?;

        Self::ensure_can_withdraw(
            transactor,
            value,
            WithdrawReason::Transfer.into(),
            new_from_balance,
        )?;

        // NOTE: total stake being stored in the same type means that this could never overflow
        // but better to be safe than sorry.
        let new_to_balance = to_balance.checked_add(&value).ok_or(Error::<T>::Overflow)?;

        if transactor != dest {
            Self::set_free_balance(transactor, new_from_balance);
            if !<FreeBalance<T>>::exists(dest) {
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
            Err(Error::<T>::InsufficientBalance)?
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
            Err(Error::<T>::DeadAccount)?
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
        if !<FreeBalance<T>>::exists(who) {
            Self::new_account(&who, balance);
        }
        Self::set_free_balance(who, balance);
        (imbalance, UpdateBalanceOutcome::Updated)
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
    fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
        Self::free_balance(who)
            .checked_sub(&value)
            .map_or(false, |new_balance| {
                Self::ensure_can_withdraw(who, value, WithdrawReason::Reserve.into(), new_balance)
                    .is_ok()
            })
    }

    fn reserved_balance(who: &T::AccountId) -> Self::Balance {
        <ReservedBalance<T>>::get(who)
    }

    fn reserve(who: &T::AccountId, value: Self::Balance) -> result::Result<(), DispatchError> {
        let b = Self::free_balance(who);
        if b < value {
            Err(Error::<T>::InsufficientBalance)?
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
            Err(Error::<T>::DeadAccount)?
        }
        let b = Self::reserved_balance(slashed);
        let slash = cmp::min(b, value);
        Self::set_free_balance(beneficiary, Self::free_balance(beneficiary) + slash);
        Self::set_reserved_balance(slashed, b - slash);
        Ok(value - slash)
    }
}

impl<T: Trait> LockableCurrency<T::AccountId> for Module<T>
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
        <Locks<T>>::insert(who, locks);
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
        <Locks<T>>::insert(who, locks);
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
        <Locks<T>>::insert(who, locks);
    }
}

impl<T: Trait> VestingCurrency<T::AccountId> for Module<T>
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
        if <Vesting<T>>::exists(who) {
            Err(Error::<T>::ExistingVestingSchedule)?
        }
        let vesting_schedule = VestingSchedule {
            locked,
            per_block,
            starting_block,
        };
        <Vesting<T>>::insert(who, vesting_schedule);
        Ok(())
    }

    /// Remove a vesting schedule for a given account.
    fn remove_vesting_schedule(who: &T::AccountId) {
        <Vesting<T>>::remove(who);
    }
}

impl<T: Trait> IsDeadAccount<T::AccountId> for Module<T>
where
    T::Balance: MaybeSerializeDeserialize + Debug,
{
    fn is_dead_account(who: &T::AccountId) -> bool {
        Self::total_balance(who).is_zero()
    }
}
