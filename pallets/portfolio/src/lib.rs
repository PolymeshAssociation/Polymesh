// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
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
//! - `move_portfolio`: Moves specified amounts of assets from one portfolio to another portfolio
//!   of the same DID.
//! - `rename_portfolio`: Renames a user portfolio.
//!
//! ### Public Functions
//!
//! - `default_portfolio_balance`: Returns the ticker balance of the identity's default portfolio.
//! - `user_portfolio_balance`: Returns the ticker balance of an identity's user portfolio.
//! - `set_default_portfolio_balance`: Sets the ticker balance of the identity's default portfolio.
//! - `rpc_get_portfolios`: An RPC function that lists all user-defined portfolio number-name pairs.
//! - `rpc_get_portfolio_assets`: Ensures that there is no portfolio with the desired name yet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    IterableStorageDoubleMap,
};
use frame_system as system;
use pallet_identity::PermissionedCallOriginData;
use polymesh_common_utilities::{identity::Trait as IdentityTrait, CommonTrait};
use polymesh_primitives::{IdentityId, PortfolioId, PortfolioName, PortfolioNumber, Ticker};
use sp_arithmetic::traits::Saturating;
use sp_std::{convert::TryFrom, iter, prelude::Vec};

type Identity<T> = pallet_identity::Module<T>;

/// The ticker and balance of an asset to be moved from one portfolio to another.
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovePortfolioItem<Balance> {
    /// The ticker of the asset to be moved.
    pub ticker: Ticker,
    /// The balance of the asset to be moved.
    pub amount: Balance,
}

pub trait Trait: CommonTrait + IdentityTrait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Session {
        /// The set of existing portfolios with their names. If a certain pair of a DID and
        /// portfolio number maps to `None` then such a portfolio doesn't exist. Conversely, if a
        /// pair maps to `Some(name)` then such a portfolio exists and is called `name`.
        pub Portfolios get(fn portfolios):
            double_map hasher(twox_64_concat) IdentityId, hasher(twox_64_concat) PortfolioNumber =>
            Option<PortfolioName>;
        /// The asset balances of portfolios.
        pub PortfolioAssetBalances get(fn portfolio_asset_balances):
            double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) Ticker =>
            T::Balance;
        /// The next portfolio sequence number of an identity.
        pub NextPortfolioNumber get(fn next_portfolio_number):
            map hasher(twox_64_concat) IdentityId => PortfolioNumber;
    }
}

decl_event! {
    pub enum Event<T> where
        Balance = <T as CommonTrait>::Balance,
    {
        /// The portfolio has been successfully created.
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio number
        /// * portfolio name
        PortfolioCreated(IdentityId, PortfolioNumber, PortfolioName),
        /// The portfolio has been successfully removed.
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio number
        PortfolioDeleted(IdentityId, PortfolioNumber),
        /// A token amount has been moved from one portfolio to another. `None` denotes the default
        /// portfolio of the DID.
        ///
        /// # Parameters
        /// * origin DID
        /// * source portfolio
        /// * destination portfolio
        /// * asset ticker
        /// * asset balance that was moved
        MovedBetweenPortfolios(
            IdentityId,
            Option<PortfolioNumber>,
            Option<PortfolioNumber>,
            Ticker,
            Balance
        ),
        /// The portfolio identified with `num` has been renamed to `name`.
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio number
        /// * portfolio name
        PortfolioRenamed(IdentityId, PortfolioNumber, PortfolioName),
        /// All non-default portfolio numbers and names of a DID.
        ///
        /// # Parameters
        /// * origin DID
        /// * vector of number-name pairs
        UserPortfolios(IdentityId, Vec<(PortfolioNumber, PortfolioName)>),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The portfolio doesn't exist.
        PortfolioDoesNotExist,
        /// Insufficient balance for a transaction.
        InsufficientPortfolioBalance,
        /// The source and destination portfolios should be different.
        DestinationIsSamePortfolio,
        /// The portfolio couldn't be renamed because the chosen name is already in use.
        PortfolioNameAlreadyInUse,
        /// The secondary key is restricted and hence cannot create new portfolios.
        SecondaryKeyCannotCreatePortfolios,
        /// The secondary key is not authorized to access the portfolio(s).
        SecondaryKeyNotAuthorizedForPortfolio,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// The event logger.
        fn deposit_event() = default;

        /// Creates a portfolio with the given `name`.
        #[weight = 600_000_000]
        pub fn create_portfolio(origin, name: PortfolioName) -> DispatchResult {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            if let Some(sk) = secondary_key {
                // Check that the secondary signer is not limited to particular portfolios.
                ensure!(
                    sk.permissions.portfolio.is_unrestricted(),
                    Error::<T>::SecondaryKeyCannotCreatePortfolios
                );
            }
            Self::ensure_name_unique(&primary_did, &name)?;
            let num = Self::get_next_portfolio_number(&primary_did);
            <Portfolios>::insert(&primary_did, &num, name.clone());
            Self::deposit_event(RawEvent::PortfolioCreated(primary_did, num, name));
            Ok(())
        }

        /// Deletes a user portfolio and moves all its assets to the default portfolio.
        #[weight = 1_000_000_000]
        pub fn delete_portfolio(origin, num: PortfolioNumber) -> DispatchResult {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            if let Some(sk) = secondary_key {
                // Check that the secondary signer is allowed to work with this portfolio.
                ensure!(
                    sk.has_portfolio_permission(iter::once(num)),
                    Error::<T>::SecondaryKeyNotAuthorizedForPortfolio
                );
            }
            // Check that the portfolio exists.
            ensure!(Self::portfolios(&primary_did, &num).is_some(), Error::<T>::PortfolioDoesNotExist);
            let portfolio_id = PortfolioId::user_portfolio(primary_did, num);
            let def_portfolio_id = PortfolioId::default_portfolio(primary_did);
            // Move all the assets from the portfolio that is being deleted to the default
            // portfolio.
            for (ticker, balance) in <PortfolioAssetBalances<T>>::iter_prefix(&portfolio_id) {
                <PortfolioAssetBalances<T>>::mutate(&def_portfolio_id, ticker, |v| {
                    *v = v.saturating_add(balance)
                });
                Self::deposit_event(RawEvent::MovedBetweenPortfolios(
                    primary_did,
                    Some(num),
                    None,
                    ticker,
                    balance,
                ));
            }
            <PortfolioAssetBalances<T>>::remove_prefix(&portfolio_id);
            <Portfolios>::remove(&primary_did, &num);
            Self::deposit_event(RawEvent::PortfolioDeleted(primary_did, num));
            Ok(())
        }

        /// Moves a token amount from one portfolio of an identity to another portfolio of the same
        /// identity.
        #[weight = 1_000_000_000 + 10_050_000 * u64::try_from(items.len()).unwrap_or_default()]
        pub fn move_portfolio(
            origin,
            from_num: Option<PortfolioNumber>,
            to_num: Option<PortfolioNumber>,
            items: Vec<MovePortfolioItem<<T as CommonTrait>::Balance>>,
        ) -> DispatchResult {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            // Check that the source and destination portfolios are in fact different.
            ensure!(from_num != to_num, Error::<T>::DestinationIsSamePortfolio);
            // Check that the source portfolio exists.
            if let Some(from_num) = from_num {
                if let Some(sk) = &secondary_key {
                    ensure!(
                        sk.has_portfolio_permission(iter::once(from_num)),
                        Error::<T>::SecondaryKeyNotAuthorizedForPortfolio
                    );
                }
                ensure!(
                    Self::portfolios(&primary_did, from_num).is_some(),
                    Error::<T>::PortfolioDoesNotExist
                );
            }
            // Check that the destination portfolio exists.
            if let Some(to_num) = to_num {
                if let Some(sk) = &secondary_key {
                    ensure!(
                        sk.has_portfolio_permission(iter::once(to_num)),
                        Error::<T>::SecondaryKeyNotAuthorizedForPortfolio
                    );
                }
                ensure!(
                    Self::portfolios(&primary_did, to_num).is_some(),
                    Error::<T>::PortfolioDoesNotExist
                );
            }
            let get_portfolio_id = |num: Option<PortfolioNumber>| {
                num
                    .map(|num| PortfolioId::user_portfolio(primary_did, num))
                    .unwrap_or_else(|| PortfolioId::default_portfolio(primary_did))
            };
            let from_portfolio_id = get_portfolio_id(from_num);
            let to_portfolio_id = get_portfolio_id(to_num);
            for item in items {
                let from_balance = Self::portfolio_asset_balances(&from_portfolio_id, &item.ticker);
                ensure!(from_balance >= item.amount, Error::<T>::InsufficientPortfolioBalance);
                <PortfolioAssetBalances<T>>::insert(
                    &from_portfolio_id,
                    &item.ticker,
                    // Cannot underflow, as verified by `ensure!` above.
                    from_balance - item.amount
                );
                let to_balance = Self::portfolio_asset_balances(&to_portfolio_id, &item.ticker);
                <PortfolioAssetBalances<T>>::insert(
                    &to_portfolio_id,
                    &item.ticker,
                    to_balance.saturating_add(item.amount)
                );
                Self::deposit_event(RawEvent::MovedBetweenPortfolios(
                    primary_did,
                    from_num,
                    to_num,
                    item.ticker,
                    item.amount
                ));
            }
            Ok(())
        }

        /// Renames a non-default portfolio.
        #[weight = 600_000_000]
        pub fn rename_portfolio(
            origin,
            num: PortfolioNumber,
            to_name: PortfolioName,
        ) -> DispatchResult {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            if let Some(sk) = secondary_key {
                // Check that the secondary signer is allowed to work with this portfolio.
                ensure!(
                    sk.has_portfolio_permission(iter::once(num)),
                    Error::<T>::SecondaryKeyNotAuthorizedForPortfolio
                );
            }
            // Check that the portfolio exists.
            ensure!(Self::portfolios(&primary_did, &num).is_some(), Error::<T>::PortfolioDoesNotExist);
            Self::ensure_name_unique(&primary_did, &to_name)?;
            <Portfolios>::mutate(&primary_did, &num, |p| *p = Some(to_name.clone()));
            Self::deposit_event(RawEvent::PortfolioRenamed(
                primary_did,
                num,
                to_name,
            ));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Returns the ticker balance of the identity's default portfolio.
    pub fn default_portfolio_balance(
        did: IdentityId,
        ticker: &Ticker,
    ) -> <T as CommonTrait>::Balance {
        Self::portfolio_asset_balances(PortfolioId::default_portfolio(did), ticker)
    }

    /// Returns the ticker balance of an identity's user portfolio.
    pub fn user_portfolio_balance(
        did: IdentityId,
        num: PortfolioNumber,
        ticker: &Ticker,
    ) -> <T as CommonTrait>::Balance {
        Self::portfolio_asset_balances(PortfolioId::user_portfolio(did, num), ticker)
    }

    /// Sets the ticker balance of the identity's default portfolio to the given value.
    pub fn set_default_portfolio_balance(
        did: IdentityId,
        ticker: &Ticker,
        balance: <T as CommonTrait>::Balance,
    ) {
        <PortfolioAssetBalances<T>>::insert(PortfolioId::default_portfolio(did), ticker, balance);
    }

    /// Returns the next portfolio number of a given identity and increments the stored number.
    fn get_next_portfolio_number(did: &IdentityId) -> PortfolioNumber {
        let num = Self::next_portfolio_number(did);
        <NextPortfolioNumber>::insert(did, num + 1);
        num
    }

    /// An RPC function that lists all user-defined portfolio number-name pairs.
    pub fn rpc_get_portfolios(did: IdentityId) -> Vec<(PortfolioNumber, PortfolioName)> {
        <Portfolios>::iter_prefix(&did).collect()
    }

    /// An RPC function that lists all token-balance pairs of a portfolio.
    pub fn rpc_get_portfolio_assets(
        portfolio_id: PortfolioId,
    ) -> Vec<(Ticker, <T as CommonTrait>::Balance)> {
        <PortfolioAssetBalances<T>>::iter_prefix(&portfolio_id).collect()
    }

    /// Ensures that there is no portfolio with the desired `name` yet.
    fn ensure_name_unique(did: &IdentityId, name: &PortfolioName) -> DispatchResult {
        let name_uniq = <Portfolios>::iter_prefix(&did).all(|n| &n.1 != name);
        ensure!(name_uniq, Error::<T>::PortfolioNameAlreadyInUse);
        Ok(())
    }
}
