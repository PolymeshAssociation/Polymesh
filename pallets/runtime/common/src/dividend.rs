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

//! # Dividend Module
//!
//! The Dividend module provides functionality for distributing dividends to tokenholders.
//!
//! ## Overview
//!
//! The Balances module provides functions for:
//!
//! - Paying dividends
//! - Termination existing dividends
//! - claiming dividends
//! - Claiming back unclaimed dividends
//!
//! ### Terminology
//!
//! - **Payout Currency:** It is the ticker of the currency in which dividends are to be paid.
//! - **Dividend maturity date:** It is the date after which dividends can be claimed by tokenholders
//! - **Dividend expiry date:** Tokenholders can claim dividends before this date.
//! After this date, issuer can reclaim the remaining dividend.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `new` - Creates a new dividend
//! - `cancel` - Cancels an existing dividend
//! - `claim` - Allows tokenholders to claim/collect their fair share of the dividend
//! - `claim_unclaimed` - Allows token issuer to claim unclaimed dividend
//!
//! ### Public Functions
//!
//! - `get_dividend` - Returns details about a dividend

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::UnixTime,
};
use frame_system::ensure_signed;
use pallet_asset::{self as asset, BalanceOf, Trait as AssetTrait};
use pallet_identity as identity;
use polymesh_common_utilities::{
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    CommonTrait, Context,
};
use polymesh_primitives::{calendar::CheckpointId, IdentityId, Signatory, Ticker};
use sp_runtime::traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, SaturatedConversion, Zero,
};
use sp_std::prelude::*;

/// The module's configuration trait.
pub trait Trait: AssetTrait + frame_system::Trait + pallet_timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

/// Details about the dividend
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Dividend<U, V> {
    /// Total amount to be distributed
    pub amount: U,
    /// Amount left to distribute
    pub amount_left: U,
    /// Whether the owner has claimed remaining funds
    pub remaining_claimed: bool,
    /// An optional timestamp of payout start
    pub matures_at: Option<V>,
    /// An optional timestamp for payout end
    pub expires_at: Option<V>,
    /// The payout SimpleToken currency ticker.
    pub payout_currency: Ticker,
    /// The checkpoint
    pub checkpoint_id: CheckpointId,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as dividend {
        /// Dividend records; (ticker, dividend ID) => dividend entry
        /// Note: contrary to checkpoint IDs, dividend IDs are 0-indexed.
        Dividends get(fn dividends): map hasher(blake2_128_concat) (Ticker, u32) => Dividend<T::Balance, T::Moment>;
        /// How many dividends were created for a ticker so far; (ticker) => count
        DividendCount get(fn dividend_count): map hasher(blake2_128_concat) Ticker => u32;
        /// Payout flags, decide whether a user already was paid their dividend
        /// (DID, ticker, dividend_id) -> whether they got their payout
        UserPayoutCompleted get(fn payout_completed): map hasher(blake2_128_concat) (IdentityId, Ticker, u32) => bool;
    }
}

type Identity<T> = identity::Module<T>;
type Checkpoint<T> = pallet_asset::checkpoint::Module<T>;

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        /// Creates a new dividend entry without payout. Token must have at least one checkpoint.
        #[weight = 2_000_000_000]
        pub fn new(origin,
            amount: T::Balance,
            ticker: Ticker,
            matures_at: T::Moment,
            expires_at: T::Moment,
            payout_ticker: Ticker,
            checkpoint_id: CheckpointId
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let sender = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSecondaryKeyForDid
            );
            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(&ticker, did), Error::<T>::NotAnOwner);

            // Check if sender has enough funds in payout currency
            let balance = <BalanceOf<T>>::get(payout_ticker, did);
            ensure!(balance >= amount, Error::<T>::InsufficientFunds);

            // Unpack the checkpoint ID, use the latest or create a new one, in that order
            let checkpoint_id = if checkpoint_id > CheckpointId(0) {
                checkpoint_id
            } else {
                let count = <Checkpoint<T>>::total_of(&ticker);
                if count > CheckpointId(0) {
                    count
                } else {
                    let now_as_secs =
                        <T as AssetTrait>::UnixTime::now().as_secs().saturated_into::<u64>();
                    <Checkpoint<T>>::create_at_by(did, ticker, now_as_secs)?
                }
            };
            // Check if checkpoint exists
            ensure!(
                <Checkpoint<T>>::total_of(&ticker) >= checkpoint_id,
                Error::<T>::NoSuchCheckpoint
            );

            let now = <pallet_timestamp::Module<T>>::get();
            let zero_ts = Zero::zero(); // A 0 timestamp
            // Check maturity/expiration dates
            match (&matures_at, &expires_at) {
                (_start, end) if  end == &zero_ts => {
                },
                (start, end) if start == &zero_ts => {
                    // Ends in the future
                    ensure!(end > &now, Error::<T>::PayoutMustEndInFuture);
                },
                (start, end) if start == &zero_ts && end == &zero_ts => {}
                (start, end) => {
                    // Ends in the future
                    ensure!(end > &now, Error::<T>::PayoutMustEndInFuture);
                    // Ends after start
                    ensure!(end > start, Error::<T>::PayoutMustEndAfterStart);
                },
            }

            // Subtract the amount
            let new_balance = balance.checked_sub(&amount).ok_or(Error::<T>::BalanceUnderflow)?;
            <<T as IdentityTrait>::ProtocolFee>::charge_fee(ProtocolOp::DividendNew)?;
            <BalanceOf<T>>::insert(payout_ticker, did, new_balance);

            // Insert dividend entry into storage
            let new_dividend = Dividend {
                amount,
                amount_left: amount,
                remaining_claimed: false,
                matures_at: if matures_at > zero_ts { Some(matures_at) } else { None },
                expires_at: if expires_at > zero_ts { Some(expires_at) } else { None },
                payout_currency: payout_ticker,
                checkpoint_id,
            };

            let dividend_id = Self::add_dividend_entry(&ticker, new_dividend)?;

            // Dispatch event
            Self::deposit_event(RawEvent::DividendCreated(did, ticker, amount, dividend_id));

            Ok(())
        }

        /// Lets the owner cancel a dividend before start/maturity date
        #[weight = 700_000_000]
        pub fn cancel(origin, ticker: Ticker, dividend_id: u32) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let sender = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSecondaryKeyForDid
            );
            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(&ticker, did), Error::<T>::NotAnOwner);

            // Check that the dividend has not started yet
            let entry: Dividend<_, _> = Self::get_dividend(&ticker, dividend_id)
                .ok_or(Error::<T>::NoSuchDividend)?;
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(
                entry.matures_at.map_or(false, |ref start| *start > now),
                Error::<T>::MustMatureInFuture
            );

            // Pay amount back to owner
            <BalanceOf<T>>::mutate(
                entry.payout_currency, did,
                |balance: &mut T::Balance| -> DispatchResult {
                    *balance  = balance
                        .checked_add(&entry.amount)
                        .ok_or(Error::<T>::FailedToPayBackToOwner)?;
                    Ok(())
                }
            )?;

            <Dividends<T>>::remove((ticker, dividend_id));

            Self::deposit_event(RawEvent::DividendCanceled(did, ticker, dividend_id));

            Ok(())
        }

        /// Withdraws from a dividend the adequate share of the `amount` field. All dividend shares
        /// are rounded by truncation (down to first integer below)
        #[weight = 1_000_000_000]
        pub fn claim(origin, ticker: Ticker, dividend_id: u32) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let sender = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSecondaryKeyForDid
            );
            // Check if sender wasn't already paid their share
            ensure!(
                !<UserPayoutCompleted>::get((did, ticker, dividend_id)),
                Error::<T>::HasAlreadyBeenPaid
            );

            // Look dividend entry up
            let dividend = Self::get_dividend(&ticker, dividend_id).ok_or(Error::<T>::NoSuchDividend)?;

            let balance_at_checkpoint =
                <asset::Module<T>>::get_balance_at(ticker, did, dividend.checkpoint_id);

            // Check if the owner hadn't yanked the remaining amount out
            ensure!(!dividend.remaining_claimed, Error::<T>::RemainingFundsAlreadyClaimed);

            let now = <pallet_timestamp::Module<T>>::get();

            // Check if the current time is within maturity/expiration bounds
            if let Some(start) = dividend.matures_at.as_ref() {
                ensure!(now > *start, Error::<T>::CannotPayBeforeMaturity);
            }

            if let Some(end) = dividend.expires_at.as_ref() {
                ensure!(*end > now, Error::<T>::CannotPayAfterExpiration);
            }

            // Compute the share
            ensure!(<asset::Tokens<T>>::contains_key(&ticker), Error::<T>::NoSuchToken);
            let supply_at_checkpoint = <Checkpoint<T>>::total_supply_at((ticker, dividend.checkpoint_id));

            let balance_amount_product = balance_at_checkpoint
                .checked_mul(&dividend.amount)
                .ok_or(Error::<T>::BalanceAmountProductOverflowed)?;

            let share = balance_amount_product
                .checked_div(&supply_at_checkpoint)
                .ok_or(Error::<T>::BalanceAmountProductSupplyDivisionFailed)?;

            // Adjust the paid_out amount
            <Dividends<T>>::mutate((ticker, dividend_id), |entry| -> DispatchResult {
                entry.amount_left = entry.amount_left.checked_sub(&share)
                    .ok_or(Error::<T>::CouldNotIncreaseAmount)?;
                Ok(())
            })?;

            // Perform the payout in designated tokens
            <BalanceOf<T>>::mutate(
                dividend.payout_currency, did,
                |balance| -> DispatchResult {
                    *balance = balance.checked_add(&share).ok_or(Error::<T>::CouldNotAddShare)?;
                    Ok(())
                }
            )?;

            // Create payout entry
            <UserPayoutCompleted>::insert((did, ticker, dividend_id), true);

            // Dispatch event
            Self::deposit_event(RawEvent::DividendPaidOutToUser(did, ticker, dividend_id, share));
            Ok(())
        }

        /// After a dividend had expired, collect the remaining amount to owner address
        #[weight = 900_000_000]
        pub fn claim_unclaimed(origin, ticker: Ticker, dividend_id: u32) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let sender = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSecondaryKeyForDid
            );
            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(&ticker, did), Error::<T>::NotAnOwner);

            let entry = Self::get_dividend(&ticker, dividend_id).ok_or(Error::<T>::NoSuchDividend)?;

            // Check that the expiry date had passed
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(entry.expires_at.map_or(false, |ref end| *end < now), Error::<T>::NotEnded);
            // Transfer the computed amount
            <BalanceOf<T>>::mutate(
                entry.payout_currency, did,
                |balance: &mut T::Balance| -> DispatchResult {
                    *balance = balance
                        .checked_add(&entry.amount_left)
                        .ok_or(Error::<T>::FailedToPayBackToOwner)?;
                    Ok(())
                }
            )?;

            // Set amount_left, flip remaining_claimed
            <Dividends<T>>::mutate((ticker, dividend_id), |entry| -> DispatchResult {
                entry.amount_left = 0.into();
                entry.remaining_claimed = true;
                Ok(())
            })?;

            Self::deposit_event(RawEvent::DividendRemainingClaimed(did, ticker, dividend_id, entry.amount_left));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
    {
        /// A new dividend was created (ticker, amount, dividend ID)
        DividendCreated(IdentityId, Ticker, Balance, u32),

        /// A dividend was canceled (ticker, dividend ID)
        DividendCanceled(IdentityId, Ticker, u32),

        /// Dividend was paid to a user (who, ticker, dividend ID, share)
        DividendPaidOutToUser(IdentityId, Ticker, u32, Balance),

        /// Unclaimed dividend was claimed back (ticker, dividend ID, amount)
        DividendRemainingClaimed(IdentityId, Ticker, u32, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Claiming unclaimed payouts requires an end date
        NotEnded,
        /// The sender must be a secondary key for the DID.
        SenderMustBeSecondaryKeyForDid,
        /// The dividend was not found.
        NoSuchDividend,
        /// The user is not an owner of the asset.
        NotAnOwner,
        /// Insufficient funds.
        InsufficientFunds,
        /// The checkpoint for the dividend does not exist.
        NoSuchCheckpoint,
        /// Dividend payout must end in the future.
        PayoutMustEndInFuture,
        /// Dividend payout must end after it starts.
        PayoutMustEndAfterStart,
        /// Underflow while calculating the new value for balance.
        BalanceUnderflow,
        /// Dividend must mature in the future.
        MustMatureInFuture,
        /// Failed to pay back to the owner account.
        FailedToPayBackToOwner,
        /// The user has already been paid their share.
        HasAlreadyBeenPaid,
        /// The remaining payout funds were already claimed.
        RemainingFundsAlreadyClaimed,
        /// Attempted to pay out before maturity.
        CannotPayBeforeMaturity,
        /// Attempted to pay out after expiration.
        CannotPayAfterExpiration,
        /// The dividend token entry was not found.
        NoSuchToken,
        /// Multiplication of the balance with the total payout amount overflowed.
        BalanceAmountProductOverflowed,
        /// A failed division of the balance amount product by the total supply.
        BalanceAmountProductSupplyDivisionFailed,
        /// Could not increase the paid out amount.
        CouldNotIncreaseAmount,
        /// Could not add the share to sender's balance.
        CouldNotAddShare,
    }
}

impl<T: Trait> Module<T> {
    /// A helper method for dividend creation. Returns dividend ID
    /// #[inline]
    fn add_dividend_entry(
        ticker: &Ticker,
        d: Dividend<T::Balance, T::Moment>,
    ) -> core::result::Result<u32, &'static str> {
        let old_count = <DividendCount>::get(ticker);
        let new_count = old_count
            .checked_add(1)
            .ok_or("Could not add 1 to dividend count")?;

        <Dividends<T>>::insert((*ticker, old_count), d);
        <DividendCount>::insert(*ticker, new_count);

        Ok(old_count)
    }

    /// Retrieves a dividend checking that it exists beforehand.
    pub fn get_dividend(
        ticker: &Ticker,
        dividend_id: u32,
    ) -> Option<Dividend<T::Balance, T::Moment>> {
        // Check that the dividend entry exists
        let ticker_div_id = (*ticker, dividend_id);
        if <Dividends<T>>::contains_key(&ticker_div_id) {
            Some(<Dividends<T>>::get(&ticker_div_id))
        } else {
            None
        }
    }
}
