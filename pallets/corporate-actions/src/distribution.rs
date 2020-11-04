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

//! # Capital Distribution Module
//!
//! The capital distributions module provides functionality for distributing benefits,
//! whether predictable, or unpredictable, to tokenholders.
//!
//! The process works by first initiating the corporate action (CA) through `initiate_corporate_action`,
//! and then attaching a capital distribution to it via `distribute`.
//!
//! When attaching a distribution, the portfolio to withdraw from is provided,
//! as is the currency and amount of it to withdraw from the portfolio.
//! Additionally, a date (`payment_at`) is provided at which withdrawals may first happen,
//! as well as an optional expiry date `expires_at`,
//! at which shares are forfeit and may be reclaimed by the CAA.
//!
//! As aforementioned, once `payment_at` is due, shares may be withdrawn.
//! This can be done either through `claim`, which is pull-based. That is, holders withdraw themselves.
//! The other mechanism is via `push_shares`, which with the CAA can push to a number of holders.
//! Once `expires_at` is reached, however, the remaining amount to distribute is forfeit,
//! and cannot be claimed by any holder, or pushed to them.
//! Instead, that amount can be reclaimed by the CAA.
//!
//! BVefore `payment_at` is due, however,
//! a planned distribution can be cancelled by calling `remove_distribution`.
//!
//! ## Overview
//!
//! The module provides functions for:
//!
//! - Starting a distribution..
//! - Claiming or pushing shares of a distribution.
//! - Reclaiming unclaimed dividends.
//!
//! ### Terminology
//!
//! - **Currency:** The ticker being distributed to holders as a benefit, e.g., USDC or some such.
//! - **Payment-at date:** The date at which benefits may be claimed by or pushed to holders.
//! - **Expires-at date:** The date at which benefits are forfeit, and may be reclaimed by the CAA.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `distribute` starts a capital distribution.
//! - `claim` claims (pull) a share of an active capital distribution on behalf of a holder.
//! - `push_shares` pushes shares of an active capital distribution to many holders.
//! - `reclaim` reclaims forfeited shares of a capital distribution that has expired.
//! - `remove_distribution` removes a capital distribution which hasn't reached its payment date yet.

use crate as ca;
use ca::{CAId, CorporateAction, Tax, Trait};
use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
};
use pallet_asset::{self as asset, checkpoint};
use pallet_identity as identity;
use polymesh_common_utilities::{
    portfolio::PortfolioSubTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    with_transaction, CommonTrait,
};
use polymesh_primitives::{
    calendar::CheckpointId, EventOnly, IdentityId, Moment, PortfolioId, PortfolioNumber, Ticker,
};
use sp_runtime::traits::{CheckedDiv, CheckedMul};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

type Asset<T> = asset::Module<T>;
type Checkpoint<T> = checkpoint::Module<T>;
type CA<T> = ca::Module<T>;
type Identity<T> = identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;

/// A capital distribution's various details.
///
/// All information contained is used by on-chain logic.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Distribution<Balance> {
    /// The portfolio to distribute from.
    pub from: PortfolioId,
    /// The currency that payouts happen in.
    pub currency: Ticker,
    /// Total amount to be distributed.
    pub amount: Balance,
    /// Amount left to distribute.
    pub remaining: Balance,
    /// Whether the CAA has claimed remaining funds.
    pub reclaimed: bool,
    /// An optional timestamp of payout start
    pub payment_at: Moment,
    /// An optional timestamp for payout end
    pub expires_at: Option<Moment>,
}

/// Has the distribution expired?
fn expired(expiry: Option<Moment>, now: Moment) -> bool {
    expiry.filter(|&e| e <= now).is_some()
}

decl_storage! {
    trait Store for Module<T: Trait> as CapitalDistribution {
        /// All capital distributions, tied to their respective corporate actions (CAs).
        ///
        /// (CAId) => Distribution
        Distributions get(fn distributions):
            map hasher(blake2_128_concat) CAId => Option<Distribution<T::Balance>>;

        /// Has an asset holder been paid yet?
        ///
        /// (CAId, DID) -> Was DID paid in the CAId?
        HolderPaid get(fn holder_paid):
            double_map hasher(blake2_128_concat) CAId, hasher(blake2_128_concat) IdentityId => bool;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Start and attach a capital distribution, to the CA identified by `ca_id`,
        /// with `amount` funds in `currency` withdrawn from `portfolio` belonging to `origin`'s DID.
        ///
        /// The distribution will commence at `payment_at` and expire at `expires_at`,
        /// if provided, or if `None`, then there's no expiry.
        ///
        /// ## Arguments
        /// - `origin` which must be a signer for the CAA of `ca_id`.
        /// - `ca_id` identifies the CA to start a capital distribution for.
        /// - `portfolio` specifies the portfolio number of the CAA to distribute `amount` from.
        /// - `currency` to withdraw and distribute from the `portfolio`.
        /// - `amount` of `currency` to withdraw and distribute.
        /// - `payment_at` specifies when shares may first be pushed or claimed.
        /// - `expires_at` speciies, if provided, when remaining shares are forfeit
        ///    and may be reclaimed by `origin`.
        ///
        /// # Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        /// - `DistributingAsset` if `ca_id.ticker == currency`.
        /// - `ExpiryBeforePayment` if `expires_at.unwrap() <= payment_at`.
        /// - `NowAfterPayment` if `payment_at < now`.
        /// - `NoSuchCA` if `ca_id` does not identify an existing CA.
        /// - `NoRecordDate` if CA has no record date.
        /// - `RecordDateAfterStart` if CA's record date > payment_at.
        /// - `InsufficientPortfolioBalance` if `portfolio` has less than `amount` of `currency`.
        /// - `InsufficientBalance` if the protocol fee couldn't be charged.
        #[weight = 2_000_000_000]
        pub fn distribute(
            origin,
            ca_id: CAId,
            portfolio: PortfolioNumber,
            currency: Ticker,
            amount: T::Balance,
            payment_at: Moment,
            expires_at: Option<Moment>,
        ) {
            // Ensure CA's asset is distinct from the distributed currency.
            ensure!(ca_id.ticker != currency, Error::<T>::DistributingAsset);

            // Ensure that any expiry date doesn't come before the payment date.
            ensure!(!expired(expires_at, payment_at), Error::<T>::ExpiryBeforePayment);

            // Ensure `now <= payment_at`.
            ensure!(<Checkpoint<T>>::now_unix() <= payment_at, Error::<T>::NowAfterPayment);

            // Ensure origin is CAA, `ca_id` exists, and that its a benefit.
            // and that sufficient funds exist.
            let caa = <CA<T>>::ensure_ca_agent(origin, ca_id.ticker)?;
            let from = PortfolioId::user_portfolio(caa, portfolio);
            let caa = caa.for_event();
            let ca = <CA<T>>::ensure_ca_exists(ca_id)?;
            ensure!(ca.kind.is_benefit(), Error::<T>::CANotBenefit);

            // Ensure CA has a record `date <= payment_at`.
            // If we cannot, deriving a checkpoint,
            // used to determine each holder's allotment of the total `amount`,
            // is not possible.
            <CA<T>>::ensure_record_date_before_start(&ca, payment_at)?;

            // Has to be in a transaction, as both operations below are check-and-commit in kind,
            // but if either fail, both must with storage changes reverted.
            with_transaction(|| {
                // Ensure CAA's portfolio `from` has at least `amount` and lock those.
                <Portfolio<T>>::lock_tokens(&from, &currency, &amount)?;

                // Charge the protocol fee.
                T::ProtocolFee::charge_fee(ProtocolOp::DividendNew)
            })?;

            // Commit to storage.
            let distribution = Distribution {
                from,
                currency,
                amount,
                remaining: amount,
                reclaimed: false,
                payment_at,
                expires_at,
            };
            <Distributions<T>>::insert(ca_id, distribution);

            // Emit event.
            Self::deposit_event(Event::<T>::Created(caa, ca_id, distribution));
        }

        /// Claim a share of the capital distribution attached to `ca_id`.
        ///
        /// Taxes are withheld as specified by the CA.
        /// Post-tax earnings are then transferred to the default portfolio of the `origin`'s DID.
        ///
        /// All shares are rounded by truncation (down to first integer below).
        ///
        /// ## Arguments
        /// - `origin` which must be a holder of for the CAA of `ca_id`.
        /// - `ca_id` identifies the CA to start a capital distribution for.
        ///
        /// # Errors
        /// - `HolderAlreadyPaid` if `origin`'s DID has already received its share.
        /// - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
        /// - `CannotClaimBeforeStart` if `now < payment_at`.
        /// - `CannotClaimAfterExpiry` if `now > expiry_at.unwrap()`.
        /// - `NoSuchCA` if `ca_id` does not identify an existing CA.
        /// - `NotTargetedByCA` if the CA does not target `origin`'s DID.
        /// - `BalanceAmountProductOverflowed` if `ba = balance * amount` would overflow.
        /// - `BalanceAmountProductSupplyDivisionFailed` if `ba * supply` would overflow.
        /// - Other errors can occur if the complicance manager rejects the transfer.
        #[weight = 1_000_000_000]
        pub fn claim(origin, ca_id: CAId) {
            let did = <Identity<T>>::ensure_perms(origin)?;

            // Ensure holder not paid yet.
            ensure!(!HolderPaid::get(ca_id, did), Error::<T>::HolderAlreadyPaid);

            // Ensure we have an active distribution.
            let mut dist = Self::ensure_active_distribution(ca_id)?;

            // Fetch the CA data (cannot fail) + ensure CA targets DID.
            let ca = <CA<T>>::ensure_ca_exists(ca_id)?;
            <CA<T>>::ensure_ca_targets(&ca, &did)?;

            // Extract CP + total supply at the record date.
            let cp_id = <CA<T>>::record_date_cp(&ca, ca_id);
            let supply = <CA<T>>::supply_at_cp(ca_id, cp_id);

            // Transfer DID's share to them.
            Self::transfer_share(did, cp_id, supply, ca_id, &ca, &mut dist)?;

            // Commit `dist` changes.
            <Distributions<T>>::insert(ca_id, dist);
        }

        /// Push shares of an ongoing distribution to at most `max` token holders.
        ///
        /// Depending on the number of token holders,
        /// `max` may be insufficient to push shares to all of them.
        ///
        /// Taxes are withheld as specified by the CA.
        /// Post-tax earnings are then transferred to the default portfolio of the `origin`'s DID.
        ///
        /// All shares are rounded by truncation (down to first integer below).
        ///
        /// ## Arguments
        /// - `origin` which must be a holder of for the CAA of `ca_id`.
        /// - `ca_id` identifies the CA with a capital distributions to push shares for.
        /// - `max` number of holders to push to.
        ///
        /// # Errors
        /// - `UnauthorizedAsAgent` if `origin` is not the `ticker`'s CAA or owner.
        /// - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
        /// - `CannotClaimBeforeStart` if `now < payment_at`.
        /// - `CannotClaimAfterExpiry` if `now > expiry_at.unwrap()`.
        /// - `NoSuchCA` if `ca_id` does not identify an existing CA.
        #[weight = 1_000_000_000]
        pub fn push_shares(origin, ca_id: CAId, max: u32) {
            // N.B. we allow the asset owner to call this as well, not just the CAA.
            let caa_ish = Self::ensure_caa_or_owner(origin, ca_id.ticker)?.for_event();

            // Ensure we have an active distribution.
            let mut dist = Self::ensure_active_distribution(ca_id)?;

            // Fetch the CA data (cannot fail).
            let ca = <CA<T>>::ensure_ca_exists(ca_id)?;

            // Extract CP + total supply at the record date.
            let cp_id = <CA<T>>::record_date_cp(&ca, ca_id);
            let supply = <CA<T>>::supply_at_cp(ca_id, cp_id);

            let mut count: u32 = 0;
            let count_fail = <asset::BalanceOf<T>>::iter_prefix(ca_id.ticker)
                .map(|(did, _)| did)
                .filter(|did| ca.targets.targets(did))
                .filter(|did| !HolderPaid::get(ca_id, did))
                .take(max as usize)
                .inspect(|_| count += 1)
                // N.B. we don't halt on first error to ensure progress is possible on each call.
                // Progress won't happen if an error occurs for all DIDs.
                .filter(|did| Self::transfer_share(*did, cp_id, supply, ca_id, &ca, &mut dist).is_err())
                .inspect(|did| Self::deposit_event(Event::<T>::SharePushFailed(caa_ish, ca_id, *did)))
                .count() as u32;

            // Commit `dist` changes.
            if count - count_fail > 0 {
                <Distributions<T>>::insert(ca_id, dist);
            }

            // Emit some stats re. how pushing went overall.
            Self::deposit_event(Event::<T>::SharesPushed(caa_ish, ca_id, max, count, count_fail));
        }

        /// Assuming a distribution has expired,
        /// unlock the remaining amount in the distributor portfolio.
        ///
        /// ## Arguments
        /// - `origin` which must be the creator of the capital distribution tied to `ca_id`.
        /// - `ca_id` identifies the CA with a capital distribution to reclaim for.
        ///
        /// # Errors
        /// - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
        /// - `NotDistributionCreator` if `origin` is not the original creator of the distribution.
        /// - `AlreadyReclaimed` if this function has already been called successfully.
        /// - `NotExpired` if `now < expiry`.
        #[weight = 900_000_000]
        pub fn reclaim(origin, ca_id: CAId) {
            // Ensure DID is the dist creator, they haven't reclaimed, and that expiry has passed.
            let did = <Identity<T>>::ensure_perms(origin)?;
            let dist = Self::ensure_distribution_exists(ca_id)?;
            ensure!(did == dist.from.did, Error::<T>::NotDistributionCreator);
            let did = did.for_event();
            ensure!(!dist.reclaimed, Error::<T>::AlreadyReclaimed);
            ensure!(expired(dist.expires_at, <Checkpoint<T>>::now_unix()), Error::<T>::NotExpired);

            // Unlock `remaining` of `currency` from DID's portfolio.
            // This cannot fail, as we've already locked the requisite amount prior.
            <Portfolio<T>>::unlock_tokens(&dist.from, &dist.currency, &dist.remaining).unwrap();

            // Zero `remaining` + note that we've reclaimed.
            <Distributions<T>>::insert(ca_id, Distribution { reclaimed: true, remaining: 0.into(), ..dist });

            // Emit event.
            Self::deposit_event(Event::<T>::Reclaimed(did, ca_id, dist.remaining));
        }

        /// Removes a distribution that hasn't started yet,
        /// unlocking the full amount in the distributor portfolio.
        ///
        /// ## Arguments
        /// - `origin` which must be a signer for the CAA of `ca_id`.
        /// - `ca_id` identifies the CA with a not-yet-started capital distribution to remove.
        ///
        /// # Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        /// - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
        /// - `DistributionStarted` if `payment_at >= now`.
        #[weight = 900_000_000]
        pub fn remove_distribution(origin, ca_id: CAId) {
            // Ensure origin is CAA, the distribution exists, and that `now < payment_at`.
            let caa = <CA<T>>::ensure_ca_agent(origin, ca_id.ticker)?;
            let dist = Self::ensure_distribution_exists(ca_id)?;
            ensure!(<Checkpoint<T>>::now_unix() < dist.payment_at, Error::<T>::DistributionStarted);

            // Unlock and remove chain data.
            Self::unlock(&dist, dist.amount);
            <Distributions<T>>::remove(ca_id);
            HolderPaid::remove_prefix(ca_id);

            // Emit event.
            Self::deposit_event(Event::<T>::Removed(caa, ca_id));
        }
    }
}

decl_event! {
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
    {
        /// A capital distribution, with details included,
        /// was created by the DID (the CAA) for the CA specified by the `CAId`.
        ///
        /// (CAA of CAId's ticker, CA's ID, distribution details)
        Created(EventOnly<IdentityId>, CAId, Distribution<Balance>),

        /// A token holder's share of a capital distribution for the given `CAId` was claimed.
        ///
        /// (Holder/Claimant DID, CA's ID, updated distribution details, DID's share, DID's tax %)
        ShareClaimed(EventOnly<IdentityId>, CAId, Distribution<Balance>, Balance, Tax),

        /// An attempt to push a holders share of a capital distribution failed.
        ///
        /// (CAA/owner of CA's ticker, CA's ID, holder that couldn't be pushed to)
        SharePushFailed(EventOnly<IdentityId>, CAId, IdentityId),

        /// Stats from `push_shares` was emitted.
        ///
        /// (CAA/owner of CA's ticker, CA's ID, max requested DIDs, processed DIDs, failed DIDs)
        SharesPushed(EventOnly<IdentityId>, CAId, u32, u32, u32),

        /// Stats from `push_shares` was emitted.
        ///
        /// (CAA/owner of CA's ticker, CA's ID, max requested DIDs, processed DIDs, failed DIDs)
        Reclaimed(EventOnly<IdentityId>, CAId, Balance),

        /// A capital distribution was removed.
        ///
        /// (Ticker's CAA, CA's ID)
        Removed(IdentityId, CAId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// A corporate ballot was made for a non-benefit CA.
        CANotBenefit,
        /// The amount to distribute was less than available in the CAAs provided portfolio.
        InsufficientFunds,
        /// A distributions provided expiry date was strictly before its payment date.
        /// In other words, everything to distribute would immediately be forfeited.
        ExpiryBeforePayment,
        /// Start-of-payment date is in the past, since it is strictly before now.
        NowAfterPayment,
        /// Currency that is distributed is the same as the CA's ticker.
        /// CAA is attempting a form of stock split, which is not what the extrinsic is for.
        DistributingAsset,
        /// The token holder has already been paid their share.
        HolderAlreadyPaid,
        /// A capital distribution doesn't exist for this CA.
        NoSuchDistribution,
        /// Distribution allotment cannot be claimed as the current time is before start-of-payment.
        CannotClaimBeforeStart,
        /// Distribution's expiry has passed. DID cannot claim anymore and has forfeited the benefits.
        CannotClaimAfterExpiry,
        /// Multiplication of the balance with the total payout amount overflowed.
        BalanceAmountProductOverflowed,
        /// A failed division of the balance amount product by the total supply.
        BalanceAmountProductSupplyDivisionFailed,
        /// DID is not the one who created the distribution.
        NotDistributionCreator,
        /// DID who created the distribution already did reclaim.
        AlreadyReclaimed,
        /// Distribution had not expired yet, or there's no expiry date.
        NotExpired,
        /// A distribution has been activated, as `payment_at <= now` holds.
        DistributionStarted,
    }
}

impl<T: Trait> Module<T> {
    /// Transfer share of `did` to them.
    fn transfer_share(
        did: IdentityId,
        cp_id: Option<CheckpointId>,
        supply: T::Balance,
        ca_id: CAId,
        ca: &CorporateAction,
        dist: &mut Distribution<T::Balance>,
    ) -> DispatchResult {
        // Compute `balance * amount / supply`, i.e. DID's share.
        let balance = <CA<T>>::balance_at_cp(did, ca_id, cp_id);
        let share = Self::share_of(balance, dist.amount, supply)?;

        // Unlock `share` of `currency` from CAAs portfolio.
        Self::unlock(&dist, share);

        // Compute withholding tax + gain.
        let tax = ca.tax_of(&did);
        let gain = share - tax * share;

        // Transfer remainder (`gain`) to DID.
        let to = PortfolioId::default_portfolio(did);
        <Asset<T>>::base_transfer(dist.from, to, &dist.currency, gain).map_err(|e| e.error)?;

        // Note that DID was paid.
        HolderPaid::insert(ca_id, did, true);
        let did = did.for_event();

        // Commit `dist` change to storage.
        dist.remaining -= share;

        // Emit event.
        Self::deposit_event(Event::<T>::ShareClaimed(did, ca_id, *dist, share, tax));

        Ok(())
    }

    /// Unlock `amount` of `dist.currency` in the `dist.from` portfolio.
    /// Assumes that at least `amount` is locked.
    fn unlock(dist: &Distribution<T::Balance>, amount: T::Balance) {
        <Portfolio<T>>::unlock_tokens(&dist.from, &dist.currency, &amount).unwrap();
    }

    // Compute `balance * amount / supply`, i.e. DID's share.
    fn share_of(
        balance: T::Balance,
        amount: T::Balance,
        supply: T::Balance,
    ) -> Result<T::Balance, DispatchError> {
        balance
            .checked_mul(&amount)
            .ok_or(Error::<T>::BalanceAmountProductOverflowed)?
            .checked_div(&supply)
            .ok_or_else(|| Error::<T>::BalanceAmountProductSupplyDivisionFailed.into())
    }

    /// Ensure `ca_id` has some distribution and return it.
    fn ensure_distribution_exists(ca_id: CAId) -> Result<Distribution<T::Balance>, DispatchError> {
        <Distributions<T>>::get(ca_id).ok_or_else(|| Error::<T>::NoSuchDistribution.into())
    }

    /// Ensure `ca_id` has a started and non-expired, i.e. active, distribution, which is returned.
    fn ensure_active_distribution(ca_id: CAId) -> Result<Distribution<T::Balance>, DispatchError> {
        // Fetch the distribution, ensuring it exists + start date is satisfied + not expired.
        let dist = Self::ensure_distribution_exists(ca_id)?;
        let now = <Checkpoint<T>>::now_unix();
        ensure!(now > dist.payment_at, Error::<T>::CannotClaimBeforeStart);
        ensure!(
            !expired(dist.expires_at, now),
            Error::<T>::CannotClaimAfterExpiry
        );
        Ok(dist)
    }

    /// Ensure that `origin` is authorized as a CA agent of the asset `ticker` or its owner.
    /// When `origin` is unsigned, `BadOrigin` occurs.
    /// If DID is not the CAA of `ticker` or its owner, `UnauthorizedAsAgent` occurs.
    fn ensure_caa_or_owner(origin: T::Origin, ticker: Ticker) -> Result<IdentityId, DispatchError> {
        let did = <Identity<T>>::ensure_perms(origin)?;
        ensure!(
            <CA<T>>::agent(ticker).filter(|caa| caa == &did).is_some()
                || <Asset<T>>::is_owner(&ticker, did),
            ca::Error::<T>::UnauthorizedAsAgent
        );
        Ok(did)
    }
}
