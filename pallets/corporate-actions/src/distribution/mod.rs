// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
//! at which benefits are forfeit and may be reclaimed by a permissioned external agent.
//!
//! As aforementioned, once `payment_at` is due, benefits may be withdrawn.
//! This can be done either through `claim`, which is pull-based. That is, holders withdraw themselves.
//! The other mechanism is via `push_benefit`, which with a permissioned external agent can push to a holder.
//! Once `expires_at` is reached, however, the remaining amount to distribute is forfeit,
//! and cannot be claimed by any holder, or pushed to them.
//! Instead, that amount can be reclaimed by the agent.
//!
//! Before `payment_at` is due, however,
//! a planned distribution can be cancelled by calling `remove_distribution`.
//!
//! ## Overview
//!
//! The module provides functions for:
//!
//! - Starting a distribution.
//! - Claiming or pushing benefits of a distribution.
//! - Reclaiming unclaimed dividends.
//!
//! ### Terminology
//!
//! - **Currency:** The ticker being distributed to holders as a benefit, e.g., USDC or some such.
//! - **Payment-at date:** The date at which benefits may be claimed by or pushed to holders.
//! - **Expires-at date:** The date at which benefits are forfeit, and may be reclaimed by a permissioned external agent.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `distribute` starts a capital distribution.
//! - `claim` claims (pull) a benefit of an active capital distribution on behalf of a holder.
//! - `push_benefit` pushes a benefit of an active capital distribution to a holder.
//! - `reclaim` reclaims forfeited benefits of a capital distribution that has expired.
//! - `remove_distribution` removes a capital distribution which hasn't reached its payment date yet.

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use crate as ca;
use ca::{CAId, Config, Tax};
use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    weights::Weight,
};
use pallet_asset::{self as asset, checkpoint};
use pallet_identity::{self as identity, PermissionedCallOriginData};
use polymesh_common_utilities::{
    constants::currency::ONE_UNIT,
    portfolio::PortfolioSubTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    with_transaction,
};
use polymesh_primitives::{
    storage_migration_ver, Balance, EventDid, IdentityId, Moment, PortfolioId, PortfolioNumber,
    SecondaryKey, Ticker,
};
use scale_info::TypeInfo;
use sp_runtime::traits::Zero;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

type Asset<T> = asset::Module<T>;
type Checkpoint<T> = checkpoint::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;
type CA<T> = ca::Module<T>;
type Identity<T> = identity::Module<T>;
type Portfolio<T> = pallet_portfolio::Module<T>;

/// The value `per_share` must take to get 1 `currency`.
pub const PER_SHARE_PRECISION: Balance = 1_000_000;

/// A capital distribution's various details.
///
/// All information contained is used by on-chain logic.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
pub struct Distribution {
    /// The portfolio to distribute from.
    pub from: PortfolioId,
    /// The currency that payouts happen in.
    pub currency: Ticker,
    /// Amount per share to pay out, in per-million,
    /// i.e. `1 / 10^6`th of one `currency` token.
    pub per_share: Balance,
    /// Total amount to be distributed at most.
    pub amount: Balance,
    /// Amount left to distribute.
    pub remaining: Balance,
    /// Whether a permissioned external agent has claimed remaining funds.
    pub reclaimed: bool,
    /// A timestamp of payout start
    pub payment_at: Moment,
    /// An optional timestamp for payout end
    pub expires_at: Option<Moment>,
}

/// Has the distribution expired?
fn expired(expiry: Option<Moment>, now: Moment) -> bool {
    expiry.filter(|&e| e <= now).is_some()
}

/// Weight abstraction for the corporate actions module.
pub trait WeightInfo {
    fn distribute() -> Weight;
    fn claim(target_ids: u32, did_whts: u32) -> Weight;
    fn push_benefit(target_ids: u32, did_whts: u32) -> Weight;
    fn reclaim() -> Weight;
    fn remove_distribution() -> Weight;
}

decl_storage! {
    trait Store for Module<T: Config> as CapitalDistribution {
        /// All capital distributions, tied to their respective corporate actions (CAs).
        ///
        /// (CAId) => Distribution
        Distributions get(fn distributions): map hasher(blake2_128_concat) CAId => Option<Distribution>;

        /// Has an asset holder been paid yet?
        ///
        /// (CAId, DID) -> Was DID paid in the CAId?
        HolderPaid get(fn holder_paid): map hasher(blake2_128_concat) (CAId, IdentityId) => bool;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(0).unwrap()): Version;
    }
}

storage_migration_ver!(0);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Start and attach a capital distribution, to the CA identified by `ca_id`,
        /// with `amount` funds in `currency` withdrawn from `portfolio` belonging to `origin`'s DID.
        ///
        /// The distribution will commence at `payment_at` and expire at `expires_at`,
        /// if provided, or if `None`, then there's no expiry.
        ///
        /// The funds will be locked in `portfolio` from when `distribute` is called.
        /// When there's no expiry, some funds may be locked indefinitely in `portfolio`,
        /// due to claimants not withdrawing or no benefits being pushed to them.
        /// For indivisible currencies, unlocked amounts, of less than one whole unit,
        /// will not be transferable from `portfolio`.
        /// However, if we imagine that users `Alice` and `Bob` both are entitled to 1.5 units,
        /// and only receive `1` units each, then `0.5 + 0.5 = 1` units are left in `portfolio`,
        /// which is now transferrable.
        ///
        /// ## Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
        /// - `ca_id` identifies the CA to start a capital distribution for.
        /// - `portfolio` specifies the portfolio number of the agent to distribute `amount` from.
        /// - `currency` to withdraw and distribute from the `portfolio`.
        /// - `per_share` amount of `currency` to withdraw and distribute.
        ///    Specified as a per-million, i.e. `1 / 10^6`th of one `currency` token.
        /// - `amount` of `currency` to withdraw and distribute at most.
        /// - `payment_at` specifies when benefits may first be pushed or claimed.
        /// - `expires_at` specifies, if provided, when remaining benefits are forfeit
        ///    and may be reclaimed by `origin`.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        /// - `ExpiryBeforePayment` if `expires_at.unwrap() <= payment_at`.
        /// - `NoSuchCA` if `ca_id` does not identify an existing CA.
        /// - `NoRecordDate` if CA has no record date.
        /// - `RecordDateAfterStart` if CA's record date > payment_at.
        /// - `UnauthorizedCustodian` if the caller is not the custodian of `portfolio`.
        /// - `InsufficientPortfolioBalance` if `portfolio` has less than `amount` of `currency`.
        /// - `InsufficientBalance` if the protocol fee couldn't be charged.
        /// - `CANotBenefit` if the CA is not of kind PredictableBenefit/UnpredictableBenefit
        /// - `DistributionAmountIsZero` if the `amount` is zero.
        /// - `DistributionPerShareIsZero` if the `per_share` is zero.
        ///
        /// # Permissions
        /// * Asset
        /// * Portfolio
        #[weight = <T as Config>::DistWeightInfo::distribute()]
        pub fn distribute(
            origin,
            ca_id: CAId,
            portfolio: Option<PortfolioNumber>,
            currency: Ticker,
            per_share: Balance,
            amount: Balance,
            payment_at: Moment,
            expires_at: Option<Moment>,
        ) -> DispatchResult {
            Self::base_distribute(
                origin,
                ca_id,
                portfolio,
                currency,
                per_share,
                amount,
                payment_at,
                expires_at,
            )
        }

        /// Claim a benefit of the capital distribution attached to `ca_id`.
        ///
        /// Taxes are withheld as specified by the CA.
        /// Post-tax earnings are then transferred to the default portfolio of the `origin`'s DID.
        ///
        /// All benefits are rounded by truncation, down to first integer below.
        /// Moreover, before post-tax earnings, in indivisible currencies are transferred,
        /// they are rounded down to a whole unit.
        ///
        /// ## Arguments
        /// - `origin` which must be a holder of the asset and eligible for the distribution.
        /// - `ca_id` identifies the CA to start a capital distribution for.
        ///
        /// # Errors
        /// - `HolderAlreadyPaid` if `origin`'s DID has already received its benefit.
        /// - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
        /// - `CannotClaimBeforeStart` if `now < payment_at`.
        /// - `CannotClaimAfterExpiry` if `now > expiry_at.unwrap()`.
        /// - `NoSuchCA` if `ca_id` does not identify an existing CA.
        /// - `NotTargetedByCA` if the CA does not target `origin`'s DID.
        /// - `BalanceAmountProductOverflowed` if `ba = balance * amount` would overflow.
        /// - `BalanceAmountProductSupplyDivisionFailed` if `ba * supply` would overflow.
        /// - Other errors can occur if the compliance manager rejects the transfer.
        #[weight = <T as Config>::DistWeightInfo::claim(T::MaxTargetIds::get(), T::MaxDidWhts::get())]
        pub fn claim(origin, ca_id: CAId) {
            Self::base_claim(origin, ca_id)?;
        }

        /// Push benefit of an ongoing distribution to the given `holder`.
        ///
        /// Taxes are withheld as specified by the CA.
        /// Post-tax earnings are then transferred to the default portfolio of the `origin`'s DID.
        ///
        /// All benefits are rounded by truncation, down to first integer below.
        /// Moreover, before post-tax earnings, in indivisible currencies are transferred,
        /// they are rounded down to a whole unit.
        ///
        /// ## Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
        /// - `ca_id` identifies the CA with a capital distributions to push benefits for.
        /// - `holder` to push benefits to.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        /// - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
        /// - `CannotClaimBeforeStart` if `now < payment_at`.
        /// - `CannotClaimAfterExpiry` if `now > expiry_at.unwrap()`.
        /// - `NoSuchCA` if `ca_id` does not identify an existing CA.
        /// - `NotTargetedByCA` if the CA does not target `holder`.
        /// - `BalanceAmountProductOverflowed` if `ba = balance * amount` would overflow.
        /// - `BalanceAmountProductSupplyDivisionFailed` if `ba * supply` would overflow.
        /// - Other errors can occur if the compliance manager rejects the transfer.
        #[weight = <T as Config>::DistWeightInfo::push_benefit(T::MaxTargetIds::get(), T::MaxDidWhts::get())]
        pub fn push_benefit(origin, ca_id: CAId, holder: IdentityId) {
            Self::base_push_benefit(origin, ca_id, holder)?;
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
        /// - `AlreadyReclaimed` if this function has already been called successfully.
        /// - `NotExpired` if `now < expiry`.
        #[weight = <T as Config>::DistWeightInfo::reclaim()]
        pub fn reclaim(origin, ca_id: CAId) {
            Self::base_reclaim(origin, ca_id)?;
        }

        /// Removes a distribution that hasn't started yet,
        /// unlocking the full amount in the distributor portfolio.
        ///
        /// ## Arguments
        /// - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
        /// - `ca_id` identifies the CA with a not-yet-started capital distribution to remove.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        /// - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
        /// - `DistributionStarted` if `payment_at <= now`.
        #[weight = <T as Config>::DistWeightInfo::remove_distribution()]
        pub fn remove_distribution(origin, ca_id: CAId) {
            Self::base_remove_distribution(origin, ca_id)?;
        }
    }
}

decl_event! {
    pub enum Event {
        /// A capital distribution, with details included,
        /// was created by the DID (permissioned agent) for the CA identified by `CAId`.
        ///
        /// (Agent DID, CA's ID, distribution details)
        Created(EventDid, CAId, Distribution),

        /// A token holder's benefit of a capital distribution for the given `CAId` was claimed.
        ///
        /// (Caller DID, Holder/Claimant DID, CA's ID, updated distribution details, DID's benefit, DID's tax %)
        BenefitClaimed(EventDid, EventDid, CAId, Distribution, Balance, Tax),

        /// Stats from `push_benefit` was emitted.
        ///
        /// (Agent DID, CA's ID, max requested DIDs, processed DIDs, failed DIDs)
        Reclaimed(EventDid, CAId, Balance),

        /// A capital distribution was removed.
        ///
        /// (Agent DID, CA's ID)
        Removed(EventDid, CAId),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// A capital distribution was made for a non-benefit CA.
        CANotBenefit,
        /// A distribution already exists for this CA.
        AlreadyExists,
        /// A distributions provided expiry date was strictly before its payment date.
        /// In other words, everything to distribute would immediately be forfeited.
        ExpiryBeforePayment,
        /// The token holder has already been paid their benefit.
        HolderAlreadyPaid,
        /// A capital distribution doesn't exist for this CA.
        NoSuchDistribution,
        /// Distribution allotment cannot be claimed as the current time is before start-of-payment.
        CannotClaimBeforeStart,
        /// Distribution's expiry has passed. DID cannot claim anymore and has forfeited the benefits.
        CannotClaimAfterExpiry,
        /// Multiplication of the balance with the per share payout amount overflowed.
        BalancePerShareProductOverflowed,
        /// DID is not the one who created the distribution.
        NotDistributionCreator,
        /// DID who created the distribution already did reclaim.
        AlreadyReclaimed,
        /// Distribution had not expired yet, or there's no expiry date.
        NotExpired,
        /// A distribution has been activated, as `payment_at <= now` holds.
        DistributionStarted,
        /// A distribution has insufficient remaining amount of currency to distribute.
        InsufficientRemainingAmount,
        /// Distribution `amount` cannot be zero.
        DistributionAmountIsZero,
        /// Distribution `per_share` cannot be zero.
        DistributionPerShareIsZero,
    }
}

impl<T: Config> Module<T> {
    fn base_distribute(
        origin: T::Origin,
        ca_id: CAId,
        portfolio: Option<PortfolioNumber>,
        currency: Ticker,
        per_share: Balance,
        amount: Balance,
        payment_at: Moment,
        expires_at: Option<Moment>,
    ) -> DispatchResult {
        let PermissionedCallOriginData {
            primary_did: agent,
            secondary_key,
            ..
        } = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, ca_id.ticker)?;

        Self::unverified_distribute(
            agent,
            secondary_key,
            ca_id,
            portfolio,
            currency,
            per_share,
            amount,
            payment_at,
            expires_at,
        )
    }

    fn base_claim(origin: T::Origin, ca_id: CAId) -> DispatchResult {
        let did = <Identity<T>>::ensure_perms(origin)?;
        Self::transfer_benefit(did.for_event(), did, ca_id)?;
        Ok(())
    }

    fn base_push_benefit(origin: T::Origin, ca_id: CAId, holder: IdentityId) -> DispatchResult {
        let agent = <ExternalAgents<T>>::ensure_perms(origin, ca_id.ticker)?.for_event();
        Self::transfer_benefit(agent, holder, ca_id)?;
        Ok(())
    }

    fn base_reclaim(origin: T::Origin, ca_id: CAId) -> DispatchResult {
        // Ensure distribution is created, they haven't reclaimed, and that expiry has passed.
        // CA must be authorized and be the custodian.
        let PermissionedCallOriginData {
            primary_did: agent,
            secondary_key,
            ..
        } = <ExternalAgents<T>>::ensure_agent_asset_perms(origin.clone(), ca_id.ticker)?;
        let dist = Self::ensure_distribution_exists(ca_id)?;
        ensure!(!dist.reclaimed, Error::<T>::AlreadyReclaimed);
        ensure!(
            expired(dist.expires_at, <Checkpoint<T>>::now_unix()),
            Error::<T>::NotExpired
        );
        <Portfolio<T>>::ensure_portfolio_custody_and_permission(
            dist.from,
            agent,
            secondary_key.as_ref(),
        )?;

        // Unlock `remaining` of `currency` from DID's portfolio.
        // This won't fail, as we've already locked the requisite amount prior.
        Self::unlock(&dist, dist.remaining)?;

        // Zero `remaining` + note that we've reclaimed.
        Distributions::insert(
            ca_id,
            Distribution {
                reclaimed: true,
                remaining: 0u32.into(),
                ..dist
            },
        );

        // Emit event.
        Self::deposit_event(Event::Reclaimed(agent.for_event(), ca_id, dist.remaining));

        Ok(())
    }

    fn base_remove_distribution(origin: T::Origin, ca_id: CAId) -> DispatchResult {
        let agent = <ExternalAgents<T>>::ensure_perms(origin, ca_id.ticker)?.for_event();
        let dist = Self::ensure_distribution_exists(ca_id)?;
        Self::unverified_remove_distribution(agent, ca_id, &dist)?;

        Ok(())
    }

    /// Kill the distribution identified by `ca_id`.
    ///
    /// Unlike `base_remove_distribution`, this won't check permissions and that the dist exists.
    crate fn unverified_remove_distribution(
        agent: EventDid,
        ca_id: CAId,
        dist: &Distribution,
    ) -> DispatchResult {
        // Cannot remove payment has started.
        Self::ensure_distribution_not_started(&dist)?;

        // Unlock and remove chain data.
        Self::unlock(&dist, dist.amount)?;
        Distributions::remove(ca_id);

        // Emit event.
        Self::deposit_event(Event::Removed(agent, ca_id));
        Ok(())
    }

    /// Ensure that `now < payment_at`.
    crate fn ensure_distribution_not_started(dist: &Distribution) -> DispatchResult {
        ensure!(
            <Checkpoint<T>>::now_unix() < dist.payment_at,
            Error::<T>::DistributionStarted
        );
        Ok(())
    }

    /// Transfer `holder`'s benefit in `ca_id` to them.
    fn transfer_benefit(actor: EventDid, holder: IdentityId, ca_id: CAId) -> DispatchResult {
        // Ensure holder not paid yet.
        ensure!(
            !HolderPaid::get((ca_id, holder)),
            Error::<T>::HolderAlreadyPaid
        );

        // Ensure we have an active distribution.
        let mut dist = Self::ensure_active_distribution(ca_id)?;

        // Fetch the CA data (cannot fail) + ensure CA targets DID.
        let ca = <CA<T>>::ensure_ca_exists(ca_id)?;
        <CA<T>>::ensure_ca_targets(&ca, &holder)?;

        // Extract CP + total supply at the record date.
        let cp_id = <CA<T>>::record_date_cp(&ca, ca_id);

        // Compute `balance * amount / supply`, i.e. DID's benefit.
        let balance = <CA<T>>::balance_at_cp(holder, ca_id, cp_id);
        let benefit = Self::benefit_of(balance, dist.per_share)?;

        // Ensure we have enough remaining.
        dist.remaining = dist
            .remaining
            .checked_sub(benefit)
            .ok_or(Error::<T>::InsufficientRemainingAmount)?;

        // Compute withholding tax + gain.
        let tax = ca.tax_of(&holder);
        let gain = benefit - tax * benefit;

        // Round down to unit multiple if indivisible.
        let gain = if Asset::<T>::is_divisible(&dist.currency) {
            gain
        } else {
            gain / ONE_UNIT * ONE_UNIT
        };

        with_transaction(|| {
            // Unlock `benefit` of `currency` from the calling agent's portfolio.
            Self::unlock(&dist, benefit)?;

            // Transfer remainder (`gain`) to DID.
            let to = PortfolioId::default_portfolio(holder);
            <Asset<T>>::base_transfer(dist.from, to, &dist.currency, gain)
        })?;

        // Note that DID was paid.
        HolderPaid::insert((ca_id, holder), true);
        let holder = holder.for_event();

        // Commit `dist` change to storage.
        Distributions::insert(ca_id, dist);

        // Emit event.
        Self::deposit_event(Event::BenefitClaimed(
            actor, holder, ca_id, dist, benefit, tax,
        ));

        Ok(())
    }

    /// Unlock `amount` of `dist.currency` in the `dist.from` portfolio.
    fn unlock(dist: &Distribution, amount: Balance) -> DispatchResult {
        <Portfolio<T>>::unlock_tokens(&dist.from, &dist.currency, amount)
    }

    // Compute `balance * per_share`, i.e. DID's benefit.
    fn benefit_of(balance: Balance, per_share: Balance) -> Result<Balance, DispatchError> {
        balance
            .checked_mul(per_share)
            // `per_share` was entered as a multiple of 1_000_000.
            .map(|v| v / Balance::from(PER_SHARE_PRECISION))
            .ok_or_else(|| Error::<T>::BalancePerShareProductOverflowed.into())
    }

    /// Ensure `ca_id` has some distribution and return it.
    fn ensure_distribution_exists(ca_id: CAId) -> Result<Distribution, DispatchError> {
        Distributions::get(ca_id).ok_or_else(|| Error::<T>::NoSuchDistribution.into())
    }

    /// Ensure `ca_id` has a started and non-expired, i.e. active, distribution, which is returned.
    fn ensure_active_distribution(ca_id: CAId) -> Result<Distribution, DispatchError> {
        // Fetch the distribution, ensuring it exists + start date is satisfied + not expired.
        let dist = Self::ensure_distribution_exists(ca_id)?;
        let now = <Checkpoint<T>>::now_unix();
        ensure!(now >= dist.payment_at, Error::<T>::CannotClaimBeforeStart);
        ensure!(
            !expired(dist.expires_at, now),
            Error::<T>::CannotClaimAfterExpiry
        );
        Ok(dist)
    }

    /// Create a capital distribution.
    ///
    /// Unlike `base_distribute`, this won't check permissions.
    crate fn unverified_distribute(
        agent: IdentityId,
        secondary_key: Option<SecondaryKey<T::AccountId>>,
        ca_id: CAId,
        portfolio: Option<PortfolioNumber>,
        currency: Ticker,
        per_share: Balance,
        amount: Balance,
        payment_at: Moment,
        expires_at: Option<Moment>,
    ) -> DispatchResult {
        // Ensure valid `amount` and `per_share`.
        ensure!(!amount.is_zero(), Error::<T>::DistributionAmountIsZero);
        ensure!(!per_share.is_zero(), Error::<T>::DistributionPerShareIsZero);

        // Ensure that any expiry date doesn't come before the payment date.
        ensure!(
            !expired(expires_at, payment_at),
            Error::<T>::ExpiryBeforePayment
        );

        // Ensure CA doesn't have a distribution yet.
        ensure!(
            !Distributions::contains_key(ca_id),
            Error::<T>::AlreadyExists
        );

        // Ensure secondary key has perms for `from` + portfolio is valid.
        let from = PortfolioId {
            did: agent,
            kind: portfolio.into(),
        };
        <Portfolio<T>>::ensure_portfolio_custody_and_permission(
            from,
            agent,
            secondary_key.as_ref(),
        )?;
        <Portfolio<T>>::ensure_portfolio_validity(&from)?;

        // Ensure that `ca_id` exists, that its a benefit.
        let agent = agent.for_event();
        let ca = <CA<T>>::ensure_ca_exists(ca_id)?;
        ensure!(ca.kind.is_benefit(), Error::<T>::CANotBenefit);

        // Ensure CA has a record `date <= payment_at`.
        // If we cannot, deriving a checkpoint,
        // used to determine each holder's allotment of the total `amount`,
        // is not possible.
        <CA<T>>::ensure_record_date_before_start(&ca, payment_at)?;

        // Ensure `from` has at least `amount` to later lock (1).
        <Portfolio<T>>::ensure_sufficient_balance(&from, &currency, amount)?;

        // Charge the protocol fee. Last check; we are in commit phase after this.
        T::ProtocolFee::charge_fee(ProtocolOp::CapitalDistributionDistribute)?;

        // (1) Lock `amount` in `from`.
        <Portfolio<T>>::unchecked_lock_tokens(&from, &currency, amount);

        // Commit to storage.
        let distribution = Distribution {
            from,
            currency,
            per_share,
            amount,
            remaining: amount,
            reclaimed: false,
            payment_at,
            expires_at,
        };
        Distributions::insert(ca_id, distribution);

        // Emit event.
        Self::deposit_event(Event::Created(agent, ca_id, distribution));

        Ok(())
    }
}
