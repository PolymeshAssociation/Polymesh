// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymath
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! # Portfolio Module
//!
//! ## Overview
//!
//! The portfolio module provides the essential extrinsics to manage asset portfolios, public
//! functions for integration of portfolios into other pallets, and implementations of RPC getters.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `create_portfolio`: Creates a new user portfolio.
//! - `delete_portfolio`: Deletes an existing user portfolio.
//! - `move_portfolio_funds`: Moves specified amounts of assets from one portfolio to another portfolio
//!   of the same DID.
//! - `rename_portfolio`: Renames a user portfolio.
//!
//! ### Public Functions
//!
//! - `default_portfolio_balance`: Returns the ticker balance of the identity's default portfolio.
//! - `user_portfolio_balance`: Returns the ticker balance of an identity's user portfolio.
//! - `set_default_portfolio_balance`: Sets the ticker balance of the identity's default portfolio.
//! - `unchecked_transfer_portfolio_balance`: Transfers funds from one portfolio to another.
//! - `ensure_portfolio_custody`: Makes sure that the given identity has custodian access over the portfolio.
//! - `ensure_portfolio_transfer_validity`: Makes sure that a transfer between two portfolios is valid.
//! - `quit_portfolio_custody`: Returns the custody of the portfolio to the owner unilaterally.

#![feature(const_option)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use core::{iter, mem};
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchResult, Weight},
    ensure,
};
use pallet_identity::{self as identity, PermissionedCallOriginData};
use polymesh_common_utilities::traits::balances::Memo;
use polymesh_common_utilities::traits::portfolio::PortfolioSubTrait;
pub use polymesh_common_utilities::traits::portfolio::{Config, Event, WeightInfo};
use polymesh_primitives::{
    extract_auth, identity_id::PortfolioValidityResult, storage_migration_ver, Balance, IdentityId,
    PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber, SecondaryKey, Ticker,
};
use scale_info::TypeInfo;
use sp_arithmetic::traits::Zero;
use sp_std::prelude::Vec;

type Identity<T> = identity::Module<T>;

/// The ticker and balance of an asset to be moved from one portfolio to another.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovePortfolioItem {
    /// The ticker of the asset to be moved.
    pub ticker: Ticker,
    /// The balance of the asset to be moved.
    pub amount: Balance,
    /// The memo of the asset to be moved.
    /// Currently only used in events.
    pub memo: Option<Memo>,
}

decl_storage! {
    trait Store for Module<T: Config> as Portfolio {
        /// The next portfolio sequence number of an identity.
        pub NextPortfolioNumber get(fn next_portfolio_number):
            map hasher(identity) IdentityId => PortfolioNumber;

        /// The set of existing portfolios with their names. If a certain pair of a DID and
        /// portfolio number maps to `None` then such a portfolio doesn't exist. Conversely, if a
        /// pair maps to `Some(name)` then such a portfolio exists and is called `name`.
        pub Portfolios get(fn portfolios):
            double_map hasher(identity) IdentityId, hasher(twox_64_concat) PortfolioNumber =>
                PortfolioName;

        /// Inverse map of `Portfolios` used to ensure bijectivitiy,
        /// and uniqueness of names in `Portfolios`.
        pub NameToNumber get(fn name_to_number):
            double_map hasher(identity) IdentityId, hasher(blake2_128_concat) PortfolioName =>
                PortfolioNumber;

        /// How many assets with non-zero balance this portfolio contains.
        pub PortfolioAssetCount get(fn portfolio_has_assets):
            map hasher(twox_64_concat) PortfolioId => u64;

        /// The asset balances of portfolios.
        pub PortfolioAssetBalances get(fn portfolio_asset_balances):
            double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) Ticker => Balance;

        /// Amount of assets locked in a portfolio.
        /// These assets show up in portfolio balance but can not be transferred away.
        pub PortfolioLockedAssets get(fn locked_assets):
            double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) Ticker => Balance;

        /// The custodian of a particular portfolio. None implies that the identity owner is the custodian.
        pub PortfolioCustodian get(fn portfolio_custodian):
            map hasher(twox_64_concat) PortfolioId => Option<IdentityId>;

        /// Tracks all the portfolios in custody of a particular identity. Only used by the UIs.
        /// When `true` is stored as the value for a given `(did, pid)`, it means that `pid` is in custody of `did`.
        /// `false` values are never explicitly stored in the map, and are instead inferred by the absence of a key.
        pub PortfoliosInCustody get(fn portfolios_in_custody):
            double_map hasher(identity) IdentityId, hasher(twox_64_concat) PortfolioId => bool;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(1).unwrap()): Version;
    }
}

storage_migration_ver!(2);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The portfolio doesn't exist.
        PortfolioDoesNotExist,
        /// Insufficient balance for a transaction.
        InsufficientPortfolioBalance,
        /// The source and destination portfolios should be different.
        DestinationIsSamePortfolio,
        /// The portfolio couldn't be renamed because the chosen name is already in use.
        PortfolioNameAlreadyInUse,
        /// The secondary key is not authorized to access the portfolio(s).
        SecondaryKeyNotAuthorizedForPortfolio,
        /// The porfolio's custody is with someone other than the caller.
        UnauthorizedCustodian,
        /// Can not unlock more tokens than what are locked
        InsufficientTokensLocked,
        /// The portfolio still has some asset balance left
        PortfolioNotEmpty,
        /// The portfolios belong to different identities
        DifferentIdentityPortfolios
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// The event logger.
        fn deposit_event() = default;

        /// Creates a portfolio with the given `name`.
        #[weight = <T as Config>::WeightInfo::create_portfolio()]
        pub fn create_portfolio(origin, name: PortfolioName) {
            let did = Identity::<T>::ensure_perms(origin)?;
            Self::ensure_name_unique(&did, &name)?;

            let num = Self::get_next_portfolio_number(&did);
            NameToNumber::insert(&did, &name, num);
            Portfolios::insert(&did, &num, name.clone());
            Self::deposit_event(Event::PortfolioCreated(did, num, name));
        }

        /// Deletes a user portfolio. A portfolio can be deleted only if it has no funds.
        ///
        /// # Errors
        /// * `PortfolioDoesNotExist` if `num` doesn't reference a valid portfolio.
        /// * `PortfolioNotEmpty` if the portfolio still holds any asset
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::delete_portfolio()]
        pub fn delete_portfolio(origin, num: PortfolioNumber) {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;

            let pid = PortfolioId::user_portfolio(primary_did, num);

            // Ensure the portfolio is empty. Otherwise we would end up with unreachable assets.
            ensure!(PortfolioAssetCount::get(pid) == 0, Error::<T>::PortfolioNotEmpty);

            // Check that the portfolio exists and the secondary key has access to it.
            Self::ensure_user_portfolio_validity(primary_did, num)?;
            Self::ensure_portfolio_custody_and_permission(pid, primary_did, secondary_key.as_ref())?;

            // Delete from storage.
            let portfolio = Portfolios::get(&primary_did, &num);
            Portfolios::remove(&primary_did, &num);
            NameToNumber::remove(&primary_did, &portfolio);
            PortfolioAssetCount::remove(&pid);
            PortfolioAssetBalances::remove_prefix(&pid, None);
            PortfolioLockedAssets::remove_prefix(&pid, None);
            PortfoliosInCustody::remove(&Self::custodian(&pid), &pid);
            PortfolioCustodian::remove(&pid);

            // Emit event.
            Self::deposit_event(Event::PortfolioDeleted(primary_did, num));
        }

        /// Moves a token amount from one portfolio of an identity to another portfolio of the same
        /// identity. Must be called by the custodian of the sender.
        /// Funds from deleted portfolios can also be recovered via this method.
        ///
        /// A short memo can be added to to each token amount moved.
        ///
        /// # Errors
        /// * `PortfolioDoesNotExist` if one or both of the portfolios reference an invalid portfolio.
        /// * `destination_is_same_portfolio` if both sender and receiver portfolio are the same
        /// * `DifferentIdentityPortfolios` if the sender and receiver portfolios belong to different identities
        /// * `UnauthorizedCustodian` if the caller is not the custodian of the from portfolio
        /// * `InsufficientPortfolioBalance` if the sender does not have enough free balance
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::move_portfolio_funds(items.len() as u32)]
        pub fn move_portfolio_funds(
            origin,
            from: PortfolioId,
            to: PortfolioId,
            items: Vec<MovePortfolioItem>,
        ) {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;

            // Ensure the source and destination portfolios are in fact different.
            ensure!(from != to, Error::<T>::DestinationIsSamePortfolio);
            // Ensure the source and destination DID are in fact same.
            ensure!(from.did == to.did, Error::<T>::DifferentIdentityPortfolios);

            // Ensure the sender is the custodian & secondary key has access to the portfolio.
            Self::ensure_portfolio_custody_and_permission(from, primary_did, secondary_key.as_ref())?;

            // Ensure the receiving portfolio exists.
            Self::ensure_portfolio_validity(&to)?;
            // Ensure that the secondary key has access to the receiver's portfolio.
            Self::ensure_user_portfolio_permission(secondary_key.as_ref(), to)?;

            // Ensure there are sufficient funds for all moves.
            for item in &items {
                Self::ensure_sufficient_balance(&from, &item.ticker, item.amount)?;
            }

            // Commit changes.
            for item in items {
                Self::unchecked_transfer_portfolio_balance(&from, &to, &item.ticker, item.amount);
                Self::deposit_event(Event::MovedBetweenPortfolios(
                    primary_did,
                    from,
                    to,
                    item.ticker,
                    item.amount,
                    item.memo
                ));
            }
        }

        /// Renames a non-default portfolio.
        ///
        /// # Errors
        /// * `PortfolioDoesNotExist` if `num` doesn't reference a valid portfolio.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::rename_portfolio(to_name.len() as u32)]
        pub fn rename_portfolio(
            origin,
            num: PortfolioNumber,
            to_name: PortfolioName,
        ) {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;

            // Enforce custody & validty of portfolio.
            Self::ensure_user_portfolio_permission(secondary_key.as_ref(), PortfolioId::user_portfolio(primary_did, num))?;
            Self::ensure_user_portfolio_validity(primary_did, num)?;

            // Enforce new name uniqueness.
            // A side-effect of the ordering here is that you cannot rename e.g., "foo" to "foo".
            // You would first need to rename to "bar" and then back to "foo".
            Self::ensure_name_unique(&primary_did, &to_name)?;

            // Change the name in storage.
            Portfolios::mutate(&primary_did, &num, |p| {
                NameToNumber::remove(&primary_did, mem::replace(p, to_name.clone()));
                NameToNumber::insert(&primary_did, &to_name, num);
            });

            // Emit Event.
            Self::deposit_event(Event::PortfolioRenamed(
                primary_did,
                num,
                to_name,
            ));
        }

        /// When called by the custodian of `portfolio_id`,
        /// allows returning the custody of the portfolio to the portfolio owner unilaterally.
        ///
        /// # Errors
        /// * `UnauthorizedCustodian` if the caller is not the current custodian of `portfolio_id`.
        ///
        /// # Permissions
        /// * Portfolio
        #[weight = <T as Config>::WeightInfo::quit_portfolio_custody()]
        pub fn quit_portfolio_custody(origin, pid: PortfolioId) {
            let did = Identity::<T>::ensure_perms(origin)?;
            let custodian = Self::custodian(&pid);
            ensure!(did == custodian, Error::<T>::UnauthorizedCustodian);

            PortfolioCustodian::remove(&pid);
            PortfoliosInCustody::remove(&custodian, &pid);
            Self::deposit_event(Event::PortfolioCustodianChanged(
                did,
                pid,
                pid.did,
            ));
        }

        #[weight = <T as Config>::WeightInfo::accept_portfolio_custody()]
        pub fn accept_portfolio_custody(origin, auth_id: u64) -> DispatchResult {
            Self::base_accept_portfolio_custody(origin, auth_id)
        }

        fn on_runtime_upgrade() -> Weight {
            use polymesh_primitives::storage_migrate_on;

            // Remove old name to number mappings.
            // In version 4.0.0 (first mainnet deployment) when a portfolio was removed
            // the NameToNumber mapping was left out of date, this upgrade removes dangling
            // NameToNumber mappings.
            // https://github.com/PolymeshAssociation/Polymesh/pull/1200
            storage_migrate_on!(StorageVersion::get(), 1, {
                NameToNumber::iter()
                    .filter(|(identity, _, number)| !Portfolios::contains_key(identity, number))
                    .for_each(|(identity, name, _)| NameToNumber::remove(identity, name));
            });
            storage_migrate_on!(StorageVersion::get(), 2, {
                Portfolios::iter()
                    .filter(|(identity, number, name)| number == &Self::name_to_number(identity, name))
                    .for_each(|(identity, number, name)| {
                            NameToNumber::insert(identity, name, number);
                    }
                );
            });

            0
        }
    }
}

impl<T: Config> Module<T> {
    /// Returns the custodian of `pid`.
    fn custodian(pid: &PortfolioId) -> IdentityId {
        PortfolioCustodian::get(&pid).unwrap_or(pid.did)
    }

    /// Returns the ticker balance of the identity's default portfolio.
    pub fn default_portfolio_balance(did: IdentityId, ticker: &Ticker) -> Balance {
        Self::portfolio_asset_balances(PortfolioId::default_portfolio(did), ticker)
    }

    /// Returns the ticker balance of an identity's user portfolio.
    pub fn user_portfolio_balance(
        did: IdentityId,
        num: PortfolioNumber,
        ticker: &Ticker,
    ) -> Balance {
        Self::portfolio_asset_balances(PortfolioId::user_portfolio(did, num), ticker)
    }

    /// Sets the ticker balance of the identity's default portfolio to `new`.
    pub fn set_default_portfolio_balance(did: IdentityId, ticker: &Ticker, new: Balance) {
        let pid = PortfolioId::default_portfolio(did);
        PortfolioAssetBalances::mutate(&pid, ticker, |old| {
            Self::transition_asset_count(&pid, *old, new);
            *old = new;
        });
    }

    /// Returns the next portfolio number of a given identity and increments the stored number.
    fn get_next_portfolio_number(did: &IdentityId) -> PortfolioNumber {
        NextPortfolioNumber::mutate(did, |num| mem::replace(num, PortfolioNumber(num.0 + 1)))
    }

    /// Ensures that there is no portfolio with the desired `name` yet.
    fn ensure_name_unique(did: &IdentityId, name: &PortfolioName) -> DispatchResult {
        pallet_base::ensure_string_limited::<T>(name)?;
        ensure!(
            !NameToNumber::contains_key(did, name),
            Error::<T>::PortfolioNameAlreadyInUse
        );
        Ok(())
    }

    /// Transfers some funds from one portfolio to another.
    /// This function does not do any data validity checks.
    /// The caller must make sure that the portfolio, custodianship and free balance are valid before calling this function.
    pub fn unchecked_transfer_portfolio_balance(
        from: &PortfolioId,
        to: &PortfolioId,
        ticker: &Ticker,
        amount: Balance,
    ) {
        PortfolioAssetBalances::mutate(from, ticker, |balance| {
            let old = mem::replace(balance, balance.saturating_sub(amount));
            Self::transition_asset_count(from, old, *balance);
        });

        PortfolioAssetBalances::mutate(to, ticker, |balance| {
            let old = mem::replace(balance, balance.saturating_add(amount));
            Self::transition_asset_count(to, old, *balance);
        });
    }

    /// Handle cases where the balance for a ticker goes to and from 0.
    fn transition_asset_count(pid: &PortfolioId, old: Balance, new: Balance) {
        match (old.is_zero(), new.is_zero()) {
            // 0 -> 1+, so increment count.
            (true, false) => PortfolioAssetCount::mutate(pid, |c| *c = c.saturating_add(1)),
            // 1+ -> 0, so decrement count.
            (false, true) => PortfolioAssetCount::mutate(pid, |c| *c = c.saturating_sub(1)),
            _ => {}
        }
    }

    /// Ensure that the `portfolio` exists.
    pub fn ensure_portfolio_validity(portfolio: &PortfolioId) -> DispatchResult {
        // Default portfolio are always valid. Custom portfolios must be created explicitly.
        if let PortfolioKind::User(num) = portfolio.kind {
            Self::ensure_user_portfolio_validity(portfolio.did, num)?;
        }
        Ok(())
    }

    /// Ensure that the `PortfolioNumber` is valid.
    fn ensure_user_portfolio_validity(did: IdentityId, num: PortfolioNumber) -> DispatchResult {
        ensure!(
            Portfolios::contains_key(did, num),
            Error::<T>::PortfolioDoesNotExist
        );
        Ok(())
    }

    /// Ensure that the `secondary_key` has access to `portfolio`.
    pub fn ensure_user_portfolio_permission(
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        portfolio: PortfolioId,
    ) -> DispatchResult {
        // If `sk` is None, caller is primary key and has full permissions.
        if let Some(sk) = secondary_key {
            // Check that the secondary signer is allowed to work with this portfolio.
            ensure!(
                sk.has_portfolio_permission(iter::once(portfolio)),
                Error::<T>::SecondaryKeyNotAuthorizedForPortfolio
            );
        }
        Ok(())
    }

    /// Makes sure that the portfolio's custody is with the provided identity
    pub fn ensure_portfolio_custody(
        portfolio: PortfolioId,
        custodian: IdentityId,
    ) -> DispatchResult {
        // If a custodian is assigned, only they are allowed.
        // Else, only the portfolio owner is allowed
        ensure!(
            Self::portfolio_custodian(portfolio).unwrap_or(portfolio.did) == custodian,
            Error::<T>::UnauthorizedCustodian
        );

        Ok(())
    }

    /// Makes sure that a portfolio transfer is valid. Portfolio access is not checked.
    pub fn ensure_portfolio_transfer_validity(
        from_portfolio: &PortfolioId,
        to_portfolio: &PortfolioId,
        ticker: &Ticker,
        amount: Balance,
    ) -> DispatchResult {
        // 1. Ensure from and to portfolio are different
        ensure!(
            from_portfolio != to_portfolio,
            Error::<T>::DestinationIsSamePortfolio
        );

        // 2. Ensure that the portfolios exist
        Self::ensure_portfolio_validity(from_portfolio)?;
        Self::ensure_portfolio_validity(to_portfolio)?;

        // 3. Ensure sender has enough free balance
        Self::ensure_sufficient_balance(&from_portfolio, ticker, amount)
    }

    /// Granular `ensure_portfolio_transfer_validity`.
    pub fn ensure_portfolio_transfer_validity_granular(
        from_portfolio: &PortfolioId,
        to_portfolio: &PortfolioId,
        ticker: &Ticker,
        amount: Balance,
    ) -> PortfolioValidityResult {
        let receiver_is_same_portfolio = from_portfolio == to_portfolio;
        let sender_portfolio_does_not_exist =
            Self::ensure_portfolio_validity(from_portfolio).is_err();
        let receiver_portfolio_does_not_exist =
            Self::ensure_portfolio_validity(to_portfolio).is_err();
        let sender_insufficient_balance =
            Self::ensure_sufficient_balance(&from_portfolio, ticker, amount).is_err();
        PortfolioValidityResult {
            receiver_is_same_portfolio,
            sender_portfolio_does_not_exist,
            receiver_portfolio_does_not_exist,
            sender_insufficient_balance,
            result: !receiver_is_same_portfolio
                && !sender_portfolio_does_not_exist
                && !receiver_portfolio_does_not_exist
                && !sender_insufficient_balance,
        }
    }

    /// Reduces the balance of a portfolio.
    /// Throws an error if enough free balance is not available.
    pub fn reduce_portfolio_balance(
        pid: &PortfolioId,
        ticker: &Ticker,
        amount: Balance,
    ) -> DispatchResult {
        // Ensure portfolio has enough free balance
        let total_balance = Self::portfolio_asset_balances(&pid, ticker);
        let locked_balance = Self::locked_assets(&pid, ticker);
        let remaining_balance = total_balance
            .checked_sub(amount)
            .filter(|rb| rb >= &locked_balance)
            .ok_or(Error::<T>::InsufficientPortfolioBalance)?;

        // Update portfolio balance.
        PortfolioAssetBalances::insert(pid, ticker, remaining_balance);
        Self::transition_asset_count(pid, total_balance, remaining_balance);

        Ok(())
    }

    /// Ensures that the portfolio's custody is with the provided identity
    /// And the secondary key has the relevant portfolio permission
    pub fn ensure_portfolio_custody_and_permission(
        portfolio: PortfolioId,
        custodian: IdentityId,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
    ) -> DispatchResult {
        Self::ensure_portfolio_custody(portfolio, custodian)?;
        Self::ensure_user_portfolio_permission(secondary_key, portfolio)
    }

    /// Ensure `portfolio` has sufficient balance of `ticker` to lock/withdraw `amount`.
    pub fn ensure_sufficient_balance(
        portfolio: &PortfolioId,
        ticker: &Ticker,
        amount: Balance,
    ) -> DispatchResult {
        Self::portfolio_asset_balances(portfolio, ticker)
            .saturating_sub(Self::locked_assets(portfolio, ticker))
            .checked_sub(amount)
            .ok_or_else(|| Error::<T>::InsufficientPortfolioBalance.into())
            .map(drop)
    }

    /// Locks `amount` of `ticker` in `portfolio` without checking that this is sane.
    ///
    /// Locks are stacked so if there were X tokens already locked, there will now be X + N tokens locked
    pub fn unchecked_lock_tokens(portfolio: &PortfolioId, ticker: &Ticker, amount: Balance) {
        PortfolioLockedAssets::mutate(portfolio, ticker, |l| *l = l.saturating_add(amount));
    }

    fn base_accept_portfolio_custody(origin: T::Origin, auth_id: u64) -> DispatchResult {
        let to = Identity::<T>::ensure_perms(origin)?;
        Identity::<T>::accept_auth_with(&to.into(), auth_id, |data, from| {
            let pid = extract_auth!(data, PortfolioCustody(p));

            let curr = Self::custodian(&pid);
            <Identity<T>>::ensure_auth_by(from, curr)?;

            // Transfer custody of `pid` over to `to`, removing it from `curr`.
            PortfoliosInCustody::remove(&curr, &pid);
            if pid.did == to {
                // Set the custodian to the default value `None` meaning that the owner is the custodian.
                PortfolioCustodian::remove(&pid);
            } else {
                PortfolioCustodian::insert(&pid, to);
                PortfoliosInCustody::insert(&to, &pid, true);
            }

            Self::deposit_event(Event::PortfolioCustodianChanged(to, pid, to));
            Ok(())
        })
    }
}

impl<T: Config> PortfolioSubTrait<T::AccountId> for Module<T> {
    /// Locks some user tokens so that they can not be used for transfers.
    /// This is used internally by the settlement engine to prevent users from using the same funds
    /// in multiple ongoing settlements
    ///
    /// # Errors
    /// * `InsufficientPortfolioBalance` if the portfolio does not have enough free balance to lock
    fn lock_tokens(portfolio: &PortfolioId, ticker: &Ticker, amount: Balance) -> DispatchResult {
        Self::ensure_sufficient_balance(portfolio, ticker, amount)?;
        Self::unchecked_lock_tokens(portfolio, ticker, amount);
        Ok(())
    }

    /// Unlocks some locked tokens of a user.
    /// Since this is only ever called by the settlement engine,
    /// it will never be called under circumstances when it has to return an error.
    /// We are being defensive with the checks anyway.
    ///
    /// # Errors
    /// * `InsufficientTokensLocked` if the portfolio does not have enough locked tokens to unlock
    fn unlock_tokens(portfolio: &PortfolioId, ticker: &Ticker, amount: Balance) -> DispatchResult {
        // 1. Ensure portfolio has enough locked tokens
        let locked = Self::locked_assets(portfolio, ticker);
        ensure!(locked >= amount, Error::<T>::InsufficientTokensLocked);

        // 2. Unlock tokens. Can not underflow due to above ensure.
        PortfolioLockedAssets::insert(portfolio, ticker, locked - amount);
        Ok(())
    }

    /// Ensures that the portfolio's custody is with the provided identity
    fn ensure_portfolio_custody(portfolio: PortfolioId, custodian: IdentityId) -> DispatchResult {
        Self::ensure_portfolio_custody(portfolio, custodian)
    }

    /// Ensures that the portfolio's custody is with the provided identity
    /// And the secondary key has the relevant portfolio permission
    fn ensure_portfolio_custody_and_permission(
        portfolio: PortfolioId,
        custodian: IdentityId,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
    ) -> DispatchResult {
        Self::ensure_portfolio_custody_and_permission(portfolio, custodian, secondary_key)
    }
}
