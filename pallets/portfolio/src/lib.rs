// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association
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
//! - `default_portfolio_balance`: Returns the asset balance of the identity's default portfolio.
//! - `user_portfolio_balance`: Returns the asset balance of an identity's user portfolio.
//! - `set_portfolio_balance`: Sets the asset balance of a portfolio.
//! - `unchecked_transfer_portfolio_balance`: Transfers funds from one portfolio to another.
//! - `ensure_portfolio_custody`: Makes sure that the given identity has custodian access over the portfolio.
//! - `ensure_portfolio_transfer_validity`: Makes sure that a transfer between two portfolios is valid.
//! - `quit_portfolio_custody`: Returns the custody of the portfolio to the owner unilaterally.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use core::mem;
use frame_support::dispatch::DispatchResult;
use frame_support::ensure;
use frame_support::pallet_prelude::{DispatchError, Get};
use frame_support::weights::Weight;
use sp_arithmetic::traits::Zero;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

use pallet_identity::PermissionedCallOriginData;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::traits::{
    AffirmationFnConfig, AffirmationFnTrait, AssetFnConfig, AssetFnTrait, NFTTrait,
    PortfolioFnTrait,
};
use polymesh_primitives::{
    extract_auth, storage_migration_ver, Balance, Fund, FundDescription, IdentityId, Memo, NFTId,
    PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber, SecondaryKey,
};

fn count_token_moves(funds: &[Fund]) -> (u32, u32) {
    let mut fungible_moves = 0;
    let mut nfts_moves = 0;
    for fund in funds {
        match &fund.description {
            FundDescription::Fungible { .. } => {
                fungible_moves += 1;
            }
            FundDescription::NonFungible(nfts) => {
                nfts_moves += nfts.len();
            }
        }
    }
    (fungible_moves, nfts_moves as u32)
}

pub trait WeightInfo {
    fn create_portfolio(l: u32) -> Weight;
    fn delete_portfolio() -> Weight;
    fn rename_portfolio(i: u32) -> Weight;
    fn quit_portfolio_custody() -> Weight;
    fn accept_portfolio_custody() -> Weight;
    fn pre_approve_portfolio() -> Weight;
    fn remove_portfolio_pre_approval() -> Weight;
    fn move_portfolio(funds: &[Fund]) -> Weight {
        let (f, n) = count_token_moves(funds);
        Self::move_portfolio_funds(f, n)
    }
    fn move_portfolio_funds(f: u32, u: u32) -> Weight;
    fn allow_identity_to_create_portfolios() -> Weight;
    fn revoke_create_portfolios_permission() -> Weight;
    fn create_custody_portfolio() -> Weight;
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_permissions::Config
        + pallet_identity::Config
        + AssetFnConfig
        + AffirmationFnConfig
    {
        type WeightInfo: WeightInfo;
        /// Maximum number of fungible assets that can be moved in a single transfer call.
        #[pallet::constant]
        type MaxNumberOfFungibleMoves: Get<u32>;
        /// Maximum number of NFTs that can be moved in a single transfer call.
        #[pallet::constant]
        type MaxNumberOfNFTsMoves: Get<u32>;
        /// NFT module - required for updating the ownership of an NFT.
        type NFT: NFTTrait<Self::RuntimeOrigin>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
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
        /// Custody of a portfolio has been given to a different identity
        ///
        /// # Parameters
        /// * origin DID
        /// * portfolio id
        /// * portfolio custodian did
        PortfolioCustodianChanged(IdentityId, PortfolioId, IdentityId),
        /// Funds have moved between portfolios
        ///
        /// # Parameters
        /// * Origin DID.
        /// * Source portfolio.
        /// * Destination portfolio.
        /// * The type of fund that was moved.
        /// * Optional memo for the move.
        FundsMovedBetweenPortfolios(
            IdentityId,
            PortfolioId,
            PortfolioId,
            FundDescription,
            Option<Memo>,
        ),
        /// A portfolio has pre approved the receivement of an asset.
        ///
        /// # Parameters
        /// * [`IdentityId`] of the caller.
        /// * [`PortfolioId`] that will receive assets without explicit affirmation.
        /// * [`AssetId`] of the asset that has been exempt from explicit affirmation.
        PreApprovedPortfolio(IdentityId, PortfolioId, AssetId),
        /// A portfolio has removed the approval of an asset.
        ///
        /// # Parameters
        /// * [`IdentityId`] of the caller.
        /// * [`PortfolioId`] that had its pre approval revoked.
        /// * [`AssetId`] of the asset that had its pre approval revoked.
        RevokePreApprovedPortfolio(IdentityId, PortfolioId, AssetId),
        /// Allow another identity to create portfolios.
        ///
        /// # Parameters
        /// * [`IdentityId`] of the caller.
        /// * [`IdentityId`] allowed to create portfolios.
        AllowIdentityToCreatePortfolios(IdentityId, IdentityId),
        /// Revoke another identities permission to create portfolios.
        ///
        /// # Parameters
        /// * [`IdentityId`] of the caller.
        /// * [`IdentityId`] permissions to create portfolios is revoked.
        RevokeCreatePortfoliosPermission(IdentityId, IdentityId),
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            Weight::zero()
        }
    }

    /// The next portfolio sequence number of an identity.
    #[pallet::storage]
    pub type NextPortfolioNumber<T: Config> =
        StorageMap<_, Identity, IdentityId, PortfolioNumber, ValueQuery>;

    /// The set of existing portfolios with their names. If a certain pair of a DID and
    /// portfolio number maps to `None` then such a portfolio doesn't exist. Conversely, if a
    /// pair maps to `Some(name)` then such a portfolio exists and is called `name`.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type Portfolios<T: Config> = StorageDoubleMap<
        _,
        Identity,
        IdentityId,
        Twox64Concat,
        PortfolioNumber,
        PortfolioName,
        OptionQuery,
    >;

    /// Inverse map of `Portfolios` used to ensure bijectivitiy,
    /// and uniqueness of names in `Portfolios`.
    #[pallet::storage]
    #[pallet::unbounded]
    pub type NameToNumber<T: Config> = StorageDoubleMap<
        _,
        Identity,
        IdentityId,
        Blake2_128Concat,
        PortfolioName,
        PortfolioNumber,
        OptionQuery,
    >;

    /// How many assets with non-zero balance this portfolio contains.
    #[pallet::storage]
    pub type PortfolioAssetCount<T: Config> =
        StorageMap<_, Twox64Concat, PortfolioId, u64, ValueQuery>;

    /// The asset balances of portfolios.
    #[pallet::storage]
    pub type PortfolioAssetBalances<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PortfolioId,
        Blake2_128Concat,
        AssetId,
        Balance,
        ValueQuery,
    >;

    /// Amount of assets locked in a portfolio.
    /// These assets show up in portfolio balance but can not be transferred away.
    #[pallet::storage]
    pub type PortfolioLockedAssets<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PortfolioId,
        Blake2_128Concat,
        AssetId,
        Balance,
        ValueQuery,
    >;

    /// The custodian of a particular portfolio. None implies that the identity owner is the custodian.
    #[pallet::storage]
    pub type PortfolioCustodian<T: Config> =
        StorageMap<_, Twox64Concat, PortfolioId, IdentityId, OptionQuery>;

    /// Tracks all the portfolios in custody of a particular identity. Only used by the UIs.
    /// When `true` is stored as the value for a given `(did, pid)`, it means that `pid` is in custody of `did`.
    /// `false` values are never explicitly stored in the map, and are instead inferred by the absence of a key.
    #[pallet::storage]
    pub type PortfoliosInCustody<T: Config> =
        StorageDoubleMap<_, Identity, IdentityId, Twox64Concat, PortfolioId, bool, ValueQuery>;

    /// The nft associated to the portfolio.
    #[pallet::storage]
    pub type PortfolioNFT<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PortfolioId,
        Blake2_128Concat,
        (AssetId, NFTId),
        bool,
        ValueQuery,
    >;

    /// All locked nft for a given portfolio.
    #[pallet::storage]
    pub type PortfolioLockedNFT<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PortfolioId,
        Blake2_128Concat,
        (AssetId, NFTId),
        bool,
        ValueQuery,
    >;

    /// All portfolios that don't need to affirm the receivement of a given [`AssetId`].
    #[pallet::storage]
    pub type PreApprovedPortfolios<T: Config> =
        StorageDoubleMap<_, Twox64Concat, PortfolioId, Blake2_128Concat, AssetId, bool, ValueQuery>;

    /// Custodians allowed to create and take custody of portfolios on an id's behalf.
    #[pallet::storage]
    pub type AllowedCustodians<T: Config> =
        StorageDoubleMap<_, Identity, IdentityId, Identity, IdentityId, bool, ValueQuery>;

    /// Storage version.
    #[pallet::storage]
    pub type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    storage_migration_ver!(3);

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            StorageVersion::<T>::put(Version::new(3));
        }
    }

    #[pallet::error]
    pub enum Error<T> {
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
        DifferentIdentityPortfolios,
        /// Duplicate asset among the items.
        NoDuplicateAssetsAllowed,
        /// The NFT does not exist in the portfolio.
        NFTNotFoundInPortfolio,
        /// The NFT is already locked.
        NFTAlreadyLocked,
        /// The NFT has never been locked.
        NFTNotLocked,
        /// Only owned NFTs can be moved between portfolios.
        InvalidTransferNFTNotOwned,
        /// Locked NFTs can not be moved between portfolios.
        InvalidTransferNFTIsLocked,
        /// Trying to move an amount of zero assets.
        EmptyTransfer,
        /// The caller doesn't have permission to create portfolios on the owner's behalf.
        MissingOwnersPermission,
        /// Adding itself as an AllowedCustodian is not permitted.
        SelfAdditionNotAllowed,
        /// The extrinsic expected a different `AuthorizationType` than what the `data.auth_type()` is.
        BadAuthorizationType,
        /// Default portfolios cannot have custodians.
        DefaultPortfoliosCannotHaveCustodians,
        /// The key does not have permission to access the portfolio.
        UnauthorizedPortfolioKey,
        /// Key not found for caller.
        KeyNotFoundForCaller,
        /// Insufficient balance (tokens might not exist or are locked).
        InsufficientBalance,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Creates a portfolio with the given `name`.
        #[pallet::weight(<T as Config>::WeightInfo::create_portfolio(name.len() as u32))]
        #[pallet::call_index(0)]
        pub fn create_portfolio(origin: OriginFor<T>, name: PortfolioName) -> DispatchResult {
            let callers_did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
            Self::base_create_portfolio(callers_did, name)
        }

        /// Deletes a user portfolio. A portfolio can be deleted only if it has no funds.
        ///
        /// # Errors
        /// * `PortfolioDoesNotExist` if `portfolio_number` doesn't reference a valid portfolio.
        /// * `PortfolioNotEmpty` if the portfolio still holds any asset
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::delete_portfolio())]
        #[pallet::call_index(1)]
        pub fn delete_portfolio(
            origin: OriginFor<T>,
            portfolio_number: PortfolioNumber,
        ) -> DispatchResult {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;

            let pid = PortfolioId::user_portfolio(primary_did, portfolio_number);
            // Ensure the portfolio is empty. Otherwise we would end up with unreachable assets.
            ensure!(
                PortfolioAssetCount::<T>::get(&pid) == 0,
                Error::<T>::PortfolioNotEmpty
            );
            ensure!(
                PortfolioNFT::<T>::iter_prefix(&pid).next().is_none(),
                Error::<T>::PortfolioNotEmpty
            );

            // Check that the portfolio exists and the secondary key has access to it.
            let portfolio = Self::ensure_user_portfolio_validity(primary_did, portfolio_number)?;
            Self::ensure_portfolio_custody_and_permission(
                &pid,
                primary_did,
                secondary_key.as_ref(),
            )?;

            // Delete from storage.
            Portfolios::<T>::remove(&primary_did, &portfolio_number);
            NameToNumber::<T>::remove(&primary_did, &portfolio);
            PortfolioAssetCount::<T>::remove(&pid);
            #[allow(deprecated)]
            PortfolioAssetBalances::<T>::remove_prefix(&pid, None);
            #[allow(deprecated)]
            PortfolioLockedAssets::<T>::remove_prefix(&pid, None);
            PortfoliosInCustody::<T>::remove(&Self::custodian(&pid), &pid);
            PortfolioCustodian::<T>::remove(&pid);

            // Emit event.
            Self::deposit_event(Event::PortfolioDeleted(primary_did, portfolio_number));
            Ok(())
        }

        /// Renames a non-default portfolio.
        ///
        /// # Errors
        /// * `PortfolioDoesNotExist` if `portfolio_number` doesn't reference a valid portfolio.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::rename_portfolio(new_portfolio_name.len() as u32))]
        #[pallet::call_index(2)]
        pub fn rename_portfolio(
            origin: OriginFor<T>,
            portfolio_number: PortfolioNumber,
            new_portfolio_name: PortfolioName,
        ) -> DispatchResult {
            let PermissionedCallOriginData {
                primary_did,
                secondary_key,
                ..
            } = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;

            // Enforce custody & validty of portfolio.
            Self::ensure_user_portfolio_permission(
                secondary_key.as_ref(),
                &PortfolioId::user_portfolio(primary_did, portfolio_number),
            )?;
            let old_name = Self::ensure_user_portfolio_validity(primary_did, portfolio_number)?;

            // Enforce new name uniqueness.
            // A side-effect of the ordering here is that you cannot rename e.g., "foo" to "foo".
            // You would first need to rename to "bar" and then back to "foo".
            Self::ensure_name_unique(&primary_did, &new_portfolio_name)?;

            // Remove old name.
            NameToNumber::<T>::remove(&primary_did, old_name);
            // Change the name in storage.
            Portfolios::<T>::insert(&primary_did, &portfolio_number, &new_portfolio_name);
            NameToNumber::<T>::insert(&primary_did, &new_portfolio_name, portfolio_number);

            // Emit Event.
            Self::deposit_event(Event::PortfolioRenamed(
                primary_did,
                portfolio_number,
                new_portfolio_name,
            ));
            Ok(())
        }

        /// When called by the custodian of `portfolio_id`,
        /// allows returning the custody of the portfolio to the portfolio owner unilaterally.
        ///
        /// # Errors
        /// * `UnauthorizedCustodian` if the caller is not the current custodian of `portfolio_id`.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::quit_portfolio_custody())]
        #[pallet::call_index(3)]
        pub fn quit_portfolio_custody(origin: OriginFor<T>, pid: PortfolioId) -> DispatchResult {
            let did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
            let custodian = Self::custodian(&pid);
            ensure!(did == custodian, Error::<T>::UnauthorizedCustodian);

            PortfolioCustodian::<T>::remove(&pid);
            PortfoliosInCustody::<T>::remove(&custodian, &pid);

            let portfolio_did = pid.did;
            Self::deposit_event(Event::PortfolioCustodianChanged(did, pid, portfolio_did));
            Ok(())
        }

        #[pallet::weight(<T as Config>::WeightInfo::accept_portfolio_custody())]
        #[pallet::call_index(4)]
        pub fn accept_portfolio_custody(origin: OriginFor<T>, auth_id: u64) -> DispatchResult {
            Self::base_accept_portfolio_custody(origin, auth_id)
        }

        /// Moves fungigle an non-fungible tokens from one portfolio of an identity to another portfolio of the same
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
        /// * `NoDuplicateAssetsAllowed` the same asset can't be repeated in the items vector.
        /// * `InvalidTransferNFTNotOwned` if the caller is trying to move an NFT he doesn't own.
        /// * `InvalidTransferNFTIsLocked` if the caller is trying to move a locked NFT.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::move_portfolio(funds))]
        #[pallet::call_index(5)]
        pub fn move_portfolio_funds(
            origin: OriginFor<T>,
            from: PortfolioId,
            to: PortfolioId,
            funds: Vec<Fund>,
        ) -> DispatchResult {
            // Verifies if the given portfolios are valid
            let primary_did = Self::ensure_portfolios_validity_and_permissions(origin, &from, &to)?;

            // Verifies if the sender has all the funds
            Self::ensure_valid_funds(&from, &funds)?;

            // Updates the portfolio of the sender and receiver
            Self::unchecked_move_funds(primary_did, from, to, funds);

            Ok(())
        }

        /// Pre-approves the receivement of an asset to a portfolio.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `asset_id` - the [`AssetId`] that will be exempt from affirmation.
        /// * `portfolio_id` - the [`PortfolioId`] that can receive `asset_id` without affirmation.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::pre_approve_portfolio())]
        #[pallet::call_index(6)]
        pub fn pre_approve_portfolio(
            origin: OriginFor<T>,
            asset_id: AssetId,
            portfolio_id: PortfolioId,
        ) -> DispatchResult {
            Self::base_pre_approve_portfolio(origin, asset_id, portfolio_id)
        }

        /// Removes the pre approval of an asset to a portfolio.
        ///
        /// # Arguments
        /// * `origin` - the secondary key of the sender.
        /// * `asset_id` - the [`AssetId`] that will be exempt from affirmation.
        /// * `portfolio_id` - the [`PortfolioId`] that can receive `asset_id` without affirmation.
        ///
        /// # Permissions
        /// * Portfolio
        #[pallet::weight(<T as Config>::WeightInfo::remove_portfolio_pre_approval())]
        #[pallet::call_index(7)]
        pub fn remove_portfolio_pre_approval(
            origin: OriginFor<T>,
            asset_id: AssetId,
            portfolio_id: PortfolioId,
        ) -> DispatchResult {
            Self::base_remove_portfolio_pre_approval(origin, asset_id, portfolio_id)
        }

        /// Adds an identity that will be allowed to create and take custody of a portfolio under the caller's identity.
        ///
        /// # Arguments
        /// * `trusted_identity` - the [`IdentityId`] that will be allowed to call `create_custody_portfolio`.
        ///
        #[pallet::weight(<T as Config>::WeightInfo::allow_identity_to_create_portfolios())]
        #[pallet::call_index(8)]
        pub fn allow_identity_to_create_portfolios(
            origin: OriginFor<T>,
            trusted_identity: IdentityId,
        ) -> DispatchResult {
            Self::base_allow_identity_to_create_portfolios(origin, trusted_identity)
        }

        /// Removes permission of an identity to create and take custody of a portfolio under the caller's identity.
        ///
        /// # Arguments
        /// * `identity` - the [`IdentityId`] that will have the permissions to call `create_custody_portfolio` revoked.
        ///
        #[pallet::weight(<T as Config>::WeightInfo::revoke_create_portfolios_permission())]
        #[pallet::call_index(9)]
        pub fn revoke_create_portfolios_permission(
            origin: OriginFor<T>,
            identity: IdentityId,
        ) -> DispatchResult {
            Self::base_revoke_create_portfolios_permission(origin, identity)
        }

        /// Creates a portfolio under the `portfolio_owner_id` identity and transfers its custody to the caller's identity.
        ///
        /// # Arguments
        /// * `portfolio_owner_id` - the [`IdentityId`] that will own the new portfolio.
        /// * `portfolio_name` - the [`PortfolioName`] of the new portfolio.
        ///
        #[pallet::weight( <T as Config>::WeightInfo::create_custody_portfolio())]
        #[pallet::call_index(10)]
        pub fn create_custody_portfolio(
            origin: OriginFor<T>,
            portfolio_owner_id: IdentityId,
            portfolio_name: PortfolioName,
        ) -> DispatchResult {
            Self::base_create_custody_portfolio(origin, portfolio_owner_id, portfolio_name)
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Creates a portfolio named `portfolio_name` owned by `portfolio_owner_id`.
    fn base_create_portfolio(
        portfolio_owner_id: IdentityId,
        portfolio_name: PortfolioName,
    ) -> DispatchResult {
        Self::ensure_name_unique(&portfolio_owner_id, &portfolio_name)?;
        let portfolio_number = Self::get_next_portfolio_number(&portfolio_owner_id);

        NameToNumber::<T>::insert(&portfolio_owner_id, &portfolio_name, portfolio_number);
        Portfolios::<T>::insert(
            &portfolio_owner_id,
            &portfolio_number,
            portfolio_name.clone(),
        );
        Self::deposit_event(Event::PortfolioCreated(
            portfolio_owner_id,
            portfolio_number,
            portfolio_name,
        ));
        Ok(())
    }

    /// Returns the custodian of `pid`.
    fn custodian(pid: &PortfolioId) -> IdentityId {
        PortfolioCustodian::<T>::get(&pid).unwrap_or(pid.did)
    }

    /// Sets the asset balance of the specified portfolio to `new_balance`.
    pub fn set_portfolio_balance(pid: &PortfolioId, asset_id: AssetId, new_balance: Balance) {
        let current_balance = Self::get_portfolio_balance(pid, &asset_id);

        if new_balance.is_zero() {
            PortfolioAssetBalances::<T>::remove(pid, &asset_id);
        } else {
            PortfolioAssetBalances::<T>::insert(pid, asset_id, new_balance);
        }

        Self::transition_asset_count(pid, current_balance, new_balance);
    }

    /// Returns the next portfolio number of a given identity and increments the stored number.
    fn get_next_portfolio_number(did: &IdentityId) -> PortfolioNumber {
        NextPortfolioNumber::<T>::mutate(did, |num| mem::replace(num, PortfolioNumber(num.0 + 1)))
    }

    /// Ensures that there is no portfolio with the desired `name` yet.
    fn ensure_name_unique(did: &IdentityId, name: &PortfolioName) -> DispatchResult {
        pallet_base::ensure_string_limited::<T>(name)?;
        ensure!(
            !NameToNumber::<T>::contains_key(did, name),
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
        asset_id: AssetId,
        amount: Balance,
    ) {
        // Subtracts the amount from the sender
        let sender_old_balance = Self::get_portfolio_balance(from, &asset_id);
        let sender_new_balance = sender_old_balance.saturating_sub(amount);

        Self::set_portfolio_balance(from, asset_id, sender_new_balance);

        // Adds the amount to the receiver
        let rcv_old_balance = Self::get_portfolio_balance(to, &asset_id);
        let rcv_new_balance = rcv_old_balance.saturating_add(amount);
        Self::set_portfolio_balance(to, asset_id, rcv_new_balance);
    }

    /// Handle cases where the balance for an asset goes to and from 0.
    fn transition_asset_count(pid: &PortfolioId, old: Balance, new: Balance) {
        match (old.is_zero(), new.is_zero()) {
            // 0 -> 1+, so increment count.
            (true, false) => PortfolioAssetCount::<T>::mutate(pid, |c| *c = c.saturating_add(1)),
            // 1+ -> 0, so decrement count.
            (false, true) => PortfolioAssetCount::<T>::mutate(pid, |c| *c = c.saturating_sub(1)),
            _ => {}
        }
    }

    /// Ensure that the `portfolio` exists.
    pub fn ensure_portfolio_validity(portfolio: &PortfolioId) -> DispatchResult {
        // Default portfolio are always valid. Custom portfolios must be created explicitly.
        if let PortfolioKind::User(num) = portfolio.kind {
            ensure!(
                Portfolios::<T>::contains_key(portfolio.did, num),
                Error::<T>::PortfolioDoesNotExist
            );
        }
        Ok(())
    }

    /// Ensure that the `PortfolioNumber` is valid.
    fn ensure_user_portfolio_validity(
        did: IdentityId,
        num: PortfolioNumber,
    ) -> Result<PortfolioName, DispatchError> {
        Ok(Portfolios::<T>::get(did, num).ok_or(Error::<T>::PortfolioDoesNotExist)?)
    }

    /// Ensures that the caller has permission to access the specified portfolio.
    /// **Standard portfolios** (Default or User portfolios):
    ///    - Primary keys have automatic full permissions.
    ///    - Secondary keys must have explicit portfolio permissions via `has_portfolio_permission()`.
    pub fn ensure_user_portfolio_permission(
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
        portfolio_id: &PortfolioId,
    ) -> DispatchResult {
        // If `sk` is None, caller is primary key and has full permissions.
        if let Some(sk) = secondary_key {
            // Check that the secondary signer is allowed to work with this portfolio.
            ensure!(
                sk.has_portfolio_permission(portfolio_id),
                Error::<T>::SecondaryKeyNotAuthorizedForPortfolio
            );
        }
        Ok(())
    }

    /// Makes sure that the portfolio's custody is with the provided identity
    pub fn ensure_portfolio_custody(
        portfolio: &PortfolioId,
        custodian: IdentityId,
    ) -> DispatchResult {
        // If a custodian is assigned, only they are allowed.
        // Else, only the portfolio owner is allowed
        ensure!(
            PortfolioCustodian::<T>::get(portfolio).unwrap_or(portfolio.did) == custodian,
            Error::<T>::UnauthorizedCustodian
        );

        Ok(())
    }

    /// Ensures that the portfolio's custody is with the provided identity
    /// And the secondary key has the relevant portfolio permission
    pub fn ensure_portfolio_custody_and_permission(
        portfolio: &PortfolioId,
        custodian: IdentityId,
        secondary_key: Option<&SecondaryKey<T::AccountId>>,
    ) -> DispatchResult {
        Self::ensure_portfolio_custody(portfolio, custodian)?;
        Self::ensure_user_portfolio_permission(secondary_key, portfolio)
    }

    /// Ensure `portfolio` has sufficient balance of `asset_id` to lock/withdraw `amount`.
    pub fn ensure_sufficient_balance(
        portfolio: &PortfolioId,
        asset_id: &AssetId,
        amount: Balance,
    ) -> DispatchResult {
        T::AssetFn::ensure_granular(asset_id, amount)?;

        let current_balance = Self::get_portfolio_balance(portfolio, asset_id);
        let locked_balance = Self::get_portfolio_locked_balance(portfolio, asset_id);

        ensure!(
            current_balance.saturating_sub(locked_balance) >= amount,
            Error::<T>::InsufficientPortfolioBalance
        );
        Ok(())
    }

    /// Locks `amount` of `asset_id` in `portfolio` without checking that this is sane.
    ///
    /// Locks are stacked so if there were X tokens already locked, there will now be X + N tokens locked
    pub fn unchecked_lock_tokens(portfolio: PortfolioId, asset_id: AssetId, amount: Balance) {
        PortfolioLockedAssets::<T>::mutate(&portfolio, &asset_id, |l| {
            *l = l.saturating_add(amount)
        });
    }

    fn base_accept_portfolio_custody(origin: T::RuntimeOrigin, auth_id: u64) -> DispatchResult {
        let to = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
        pallet_identity::Pallet::<T>::accept_auth_with(&to.into(), auth_id, |data, from| {
            let pid = extract_auth!(data, PortfolioCustody(p));

            ensure!(
                pid.kind != PortfolioKind::Default,
                Error::<T>::DefaultPortfoliosCannotHaveCustodians
            );

            let curr = Self::custodian(&pid);
            pallet_identity::Pallet::<T>::ensure_auth_by(from, curr)?;

            // Transfer custody of `pid` over to `to`, removing it from `curr`.
            PortfoliosInCustody::<T>::remove(&curr, &pid);
            if pid.did == to {
                // Set the custodian to the default value `None` meaning that the owner is the custodian.
                PortfolioCustodian::<T>::remove(&pid);
            } else {
                Self::unverified_take_portfolio_custody(&pid, &to);
            }

            Self::deposit_event(Event::PortfolioCustodianChanged(to, pid, to));
            Ok(())
        })
    }

    /// Updates `portfolio_id` custody to `new_custodian_id`.
    fn unverified_take_portfolio_custody(
        portfolio_id: &PortfolioId,
        new_custodian_id: &IdentityId,
    ) {
        PortfolioCustodian::<T>::insert(&portfolio_id, new_custodian_id);
        PortfoliosInCustody::<T>::insert(&new_custodian_id, &portfolio_id, true);
    }

    /// Verifies if the portfolios are different, if the move is between the same identity, if the receiving portfolio exists,
    /// and if the user has access to both portfolios.
    fn ensure_portfolios_validity_and_permissions(
        origin: T::RuntimeOrigin,
        from: &PortfolioId,
        to: &PortfolioId,
    ) -> Result<IdentityId, DispatchError> {
        let origin_data = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
        // Ensures the source and destination portfolios are in fact different
        ensure!(from != to, Error::<T>::DestinationIsSamePortfolio);
        // Ensures the source and destination DID are in fact the same
        ensure!(from.did == to.did, Error::<T>::DifferentIdentityPortfolios);
        // Ensures the receiving portfolio exists
        Self::ensure_portfolio_validity(&to)?;

        // Ensures the sender is the custodian and that the secondary key has access to the portfolio.
        Self::ensure_portfolio_custody_and_permission(
            from,
            origin_data.primary_did,
            origin_data.secondary_key.as_ref(),
        )?;

        // Ensures the secondary key has access to the receiver's portfolio.
        Self::ensure_user_portfolio_permission(origin_data.secondary_key.as_ref(), to)?;
        Ok(origin_data.primary_did)
    }

    /// Verifies if the sender has all funds for the transfer. For a fungible move to be valid, the sender must have sufficient balance, and for
    /// a non-fungible move, the NFTs must be owned by the sender and can't be locked.
    fn ensure_valid_funds(sender_portfolio: &PortfolioId, funds: &[Fund]) -> DispatchResult {
        let mut unique_assets = BTreeSet::new();
        // Ensure there are sufficient funds for all moves
        for fund in funds {
            match &fund.description {
                FundDescription::Fungible { asset_id, amount } => {
                    ensure!(*amount > 0, Error::<T>::EmptyTransfer);
                    ensure!(
                        unique_assets.insert(asset_id),
                        Error::<T>::NoDuplicateAssetsAllowed
                    );
                    Self::ensure_sufficient_balance(sender_portfolio, &asset_id, *amount)?;
                }
                FundDescription::NonFungible(nfts) => {
                    ensure!(nfts.len() > 0, Error::<T>::EmptyTransfer);
                    Self::ensure_valid_nfts(sender_portfolio, nfts.asset_id(), nfts.ids())?;
                }
            }
        }
        Ok(())
    }

    /// Verifies if the portfolio has the nfts and if they are not locked.
    fn ensure_valid_nfts(
        portfolio: &PortfolioId,
        asset_id: &AssetId,
        nft_ids: &[NFTId],
    ) -> DispatchResult {
        for nft_id in nft_ids {
            ensure!(
                Self::is_nft_owner(portfolio, asset_id, nft_id),
                Error::<T>::InvalidTransferNFTNotOwned
            );
            ensure!(
                !Self::is_nft_locked(portfolio, asset_id, nft_id),
                Error::<T>::InvalidTransferNFTIsLocked
            );
        }
        Ok(())
    }

    /// Moves all funds from the sender portfolio to the receiver portfolio.
    fn unchecked_move_funds(
        origin_did: IdentityId,
        sender_portfolio: PortfolioId,
        receiver_portfolio: PortfolioId,
        funds: Vec<Fund>,
    ) {
        for fund in funds {
            match fund.description {
                FundDescription::Fungible { asset_id, amount } => {
                    Self::unchecked_transfer_portfolio_balance(
                        &sender_portfolio,
                        &receiver_portfolio,
                        asset_id,
                        amount,
                    );
                    Self::deposit_event(Event::FundsMovedBetweenPortfolios(
                        origin_did,
                        sender_portfolio.clone(),
                        receiver_portfolio.clone(),
                        FundDescription::Fungible { asset_id, amount },
                        fund.memo,
                    ));
                }
                FundDescription::NonFungible(nfts) => {
                    for nft_id in nfts.ids() {
                        Self::remove_nft_from_portfolio(&sender_portfolio, nfts.asset_id(), nft_id);
                        Self::add_nft_to_portfolio(
                            receiver_portfolio.clone(),
                            *nfts.asset_id(),
                            *nft_id,
                        );
                        T::NFT::update_nft_owner(
                            *nfts.asset_id(),
                            *nft_id,
                            receiver_portfolio.clone(),
                        );
                    }
                    Self::deposit_event(Event::FundsMovedBetweenPortfolios(
                        origin_did,
                        sender_portfolio.clone(),
                        receiver_portfolio.clone(),
                        FundDescription::NonFungible(nfts),
                        fund.memo,
                    ));
                }
            }
        }
    }

    fn base_pre_approve_portfolio(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        portfolio_id: PortfolioId,
    ) -> DispatchResult {
        let origin_data = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
        Self::ensure_portfolio_validity(&portfolio_id)?;
        Self::ensure_portfolio_custody_and_permission(
            &portfolio_id,
            origin_data.primary_did,
            origin_data.secondary_key.as_ref(),
        )?;

        PreApprovedPortfolios::<T>::insert(&portfolio_id, asset_id, true);
        Self::deposit_event(Event::PreApprovedPortfolio(
            origin_data.primary_did,
            portfolio_id,
            asset_id,
        ));
        Ok(())
    }

    fn base_remove_portfolio_pre_approval(
        origin: T::RuntimeOrigin,
        asset_id: AssetId,
        portfolio_id: PortfolioId,
    ) -> DispatchResult {
        let origin_data = pallet_identity::Pallet::<T>::ensure_origin_call_permissions(origin)?;
        Self::ensure_portfolio_validity(&portfolio_id)?;
        Self::ensure_portfolio_custody_and_permission(
            &portfolio_id,
            origin_data.primary_did,
            origin_data.secondary_key.as_ref(),
        )?;

        PreApprovedPortfolios::<T>::remove(&portfolio_id, asset_id);
        Self::deposit_event(Event::RevokePreApprovedPortfolio(
            origin_data.primary_did,
            portfolio_id,
            asset_id,
        ));
        Ok(())
    }

    fn base_allow_identity_to_create_portfolios(
        origin: T::RuntimeOrigin,
        trusted_identity: IdentityId,
    ) -> DispatchResult {
        let callers_did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
        ensure!(
            callers_did != trusted_identity,
            Error::<T>::SelfAdditionNotAllowed
        );
        AllowedCustodians::<T>::insert(callers_did, trusted_identity, true);
        Self::deposit_event(Event::AllowIdentityToCreatePortfolios(
            callers_did,
            trusted_identity,
        ));
        Ok(())
    }

    fn base_revoke_create_portfolios_permission(
        origin: T::RuntimeOrigin,
        identity: IdentityId,
    ) -> DispatchResult {
        let callers_did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
        AllowedCustodians::<T>::remove(callers_did, identity);
        Self::deposit_event(Event::RevokeCreatePortfoliosPermission(
            callers_did,
            identity,
        ));
        Ok(())
    }

    fn base_create_custody_portfolio(
        origin: T::RuntimeOrigin,
        portfolio_owner_id: IdentityId,
        portfolio_name: PortfolioName,
    ) -> DispatchResult {
        let callers_did = pallet_identity::Pallet::<T>::ensure_perms(origin)?;
        // Ensures that the caller is allowed to create portfolios on `portfolio_owner_id` behalf
        ensure!(
            AllowedCustodians::<T>::get(&portfolio_owner_id, &callers_did),
            Error::<T>::MissingOwnersPermission
        );
        // Creates a new portfolio
        let portfolio_number = NextPortfolioNumber::<T>::get(&portfolio_owner_id);
        let portfolio_id =
            PortfolioId::new(portfolio_owner_id, PortfolioKind::User(portfolio_number));
        Self::base_create_portfolio(portfolio_owner_id, portfolio_name)?;
        // Updates storage for taking ownership of a portfolio
        Self::unverified_take_portfolio_custody(&portfolio_id, &callers_did);
        Self::deposit_event(Event::PortfolioCustodianChanged(
            callers_did,
            portfolio_id,
            callers_did,
        ));
        Ok(())
    }

    /// Returns the balance of `asset_id` in `portfolio`.
    pub fn get_portfolio_balance(portfolio: &PortfolioId, asset_id: &AssetId) -> Balance {
        PortfolioAssetBalances::<T>::get(portfolio, asset_id)
    }

    /// Returns the locked balance of `asset_id` in `portfolio`.
    pub fn get_portfolio_locked_balance(portfolio: &PortfolioId, asset_id: &AssetId) -> Balance {
        PortfolioLockedAssets::<T>::get(portfolio, asset_id)
    }

    /// Sets the locked balance of `asset_id` in `portfolio` to `new_locked_balance`.
    pub fn set_portfolio_locked_balance(
        portfolio: PortfolioId,
        asset_id: AssetId,
        new_locked_balance: Balance,
    ) {
        if new_locked_balance.is_zero() {
            PortfolioLockedAssets::<T>::remove(&portfolio, &asset_id);
        } else {
            PortfolioLockedAssets::<T>::insert(portfolio, asset_id, new_locked_balance);
        }
    }

    /// Returns `true` if the given portfolio owns the `nft_id` of `asset_id` collection.
    pub fn is_nft_owner(portfolio_id: &PortfolioId, asset_id: &AssetId, nft_id: &NFTId) -> bool {
        PortfolioNFT::<T>::contains_key(portfolio_id, (asset_id, nft_id))
    }

    /// Returns `true` if the nft is locked.
    pub fn is_nft_locked(portfolio_id: &PortfolioId, asset_id: &AssetId, nft_id: &NFTId) -> bool {
        PortfolioLockedNFT::<T>::contains_key(portfolio_id, (asset_id, nft_id))
    }

    /// Removes the nft from the owner's portfolio.
    pub fn remove_nft_from_portfolio(
        portfolio_id: &PortfolioId,
        asset_id: &AssetId,
        nft_id: &NFTId,
    ) {
        PortfolioNFT::<T>::remove(portfolio_id, (asset_id, nft_id));
    }

    /// Adds the nft to the owner's portfolio.
    pub fn add_nft_to_portfolio(portfolio_id: PortfolioId, asset_id: AssetId, nft_id: NFTId) {
        PortfolioNFT::<T>::insert(portfolio_id, (asset_id, nft_id), true);
    }

    /// Locks the given nft.
    pub fn lock_nft(portfolio_id: PortfolioId, asset_id: AssetId, nft_id: NFTId) {
        PortfolioLockedNFT::<T>::insert(portfolio_id, (asset_id, nft_id), true);
    }

    /// Unlocks the given nft.
    pub fn unlock_nft(portfolio_id: &PortfolioId, asset_id: &AssetId, nft_id: &NFTId) {
        PortfolioLockedNFT::<T>::remove(portfolio_id, (asset_id, nft_id));
    }

    /// Returns `true` if the portfolio should skip receiver affirmation for `asset_id`.
    ///
    /// The governing identity is the custodian (for custodial portfolios) or the portfolio
    /// owner (for non-custodial portfolios). If the governing identity has not opted in to
    /// mandatory receiver affirmation, affirmation is skipped. If opted in, asset-level and
    /// portfolio+asset pre-approvals can still override.
    pub fn skip_portfolio_affirmation(portfolio_id: &PortfolioId, asset_id: &AssetId) -> bool {
        // For custodial portfolios, the custodian's policy governs; otherwise the owner's.
        let affirmation_did =
            PortfolioCustodian::<T>::get(portfolio_id).unwrap_or(portfolio_id.did);

        // If the governing identity hasn't opted in to mandatory receiver affirmation, skip.
        if !T::AffirmationFn::identity_requires_affirmation(&affirmation_did) {
            return true;
        }

        // Identity opted in — check if asset is globally exempt.
        if T::AssetFn::asset_affirmation_exemption(asset_id) {
            return true;
        }

        // Check per-identity and per-portfolio pre-approvals.
        if T::AssetFn::skip_asset_affirmation(&affirmation_did, asset_id) {
            return true;
        }
        PreApprovedPortfolios::<T>::get(portfolio_id, asset_id)
    }

    /// Adds `amount` of locked balance of `asset_id` in `portfolio`, if the portfolio has sufficient free balance.
    pub fn lock_asset_balance(
        portfolio: PortfolioId,
        asset_id: AssetId,
        amount: Balance,
    ) -> DispatchResult {
        Self::ensure_sufficient_balance(&portfolio, &asset_id, amount)?;
        Self::unchecked_lock_tokens(portfolio, asset_id, amount);
        Ok(())
    }

    /// Subtracts `amount` of locked balance of `asset_id` from `portfolio`, if there is enough locked balance.
    pub fn unlock_asset_balance(
        portfolio: PortfolioId,
        asset_id: AssetId,
        amount: Balance,
    ) -> DispatchResult {
        let current_locked = Self::get_portfolio_locked_balance(&portfolio, &asset_id);
        ensure!(
            current_locked >= amount,
            Error::<T>::InsufficientTokensLocked
        );
        Self::set_portfolio_locked_balance(portfolio, asset_id, current_locked - amount);
        Ok(())
    }
}

impl<T: Config> PortfolioFnTrait for Pallet<T> {
    fn ensure_portfolio_custody(
        portfolio: &PortfolioId,
        custodian: IdentityId,
    ) -> Result<(), DispatchError> {
        Self::ensure_portfolio_custody(portfolio, custodian)
    }
}
