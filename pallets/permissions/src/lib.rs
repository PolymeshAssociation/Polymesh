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

//! # Permissions module
//!
//! This module implements the functionality allowing to check permissions for an account to call
//! the current extrinsic.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, DecodeWithMemTracking, Encode};
use core::mem;
use frame_support::dispatch::DispatchResult;
use frame_support::pallet_prelude::*;
use frame_support::traits::{CallMetadata, GetCallMetadata};
use scale_info::TypeInfo;
use sp_runtime::traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, TransactionExtension};
use sp_runtime::transaction_validity::{TransactionValidityError, ValidTransaction};
use sp_std::fmt;
use sp_std::marker::PhantomData;
use sp_std::result::Result;

use polymesh_primitives::{ExtrinsicName, IdentityId, PalletName, SecondaryKey};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The type that implements the permission check function.
        type Checker: CheckAccountCallPermissions<Self::AccountId>;
    }

    /// Result of `CheckAccountCallPermissions::check_account_call_permissions`.
    pub struct AccountCallPermissionsData<AccountId> {
        /// The primary identity of the call.
        pub primary_did: IdentityId,
        /// The secondary key of the call, if it is defined.
        pub secondary_key: Option<SecondaryKey<AccountId>>,
    }

    impl<AccountId> AccountCallPermissionsData<AccountId> {
        /// Constructs a new `AccountCallPermissionsData`.
        pub fn new(
            primary_did: IdentityId,
            secondary_key: Option<SecondaryKey<AccountId>>,
        ) -> Self {
            Self {
                primary_did,
                secondary_key,
            }
        }
    }

    /// A permission checker for calls from accounts to extrinsics.
    pub trait CheckAccountCallPermissions<AccountId> {
        /// Checks whether `who` can call the current extrinsic represented by `pallet_name` and
        /// `function_name`.
        ///
        /// Returns:
        ///
        /// - `Some(data)` where `data` contains the primary identity ID on behalf of which the caller
        /// is allowed to make this call and the secondary key of the caller if the caller is a
        /// secondary key of the primary identity.
        ///
        /// - `None` if the call is not allowed.
        fn check_account_call_permissions(
            who: &AccountId,
            pallet_name: impl FnOnce() -> PalletName,
            function_name: impl FnOnce() -> ExtrinsicName,
        ) -> Result<AccountCallPermissionsData<AccountId>, DispatchError>;

        /// Checks whether `who` can call the current extrinsic represented by `pallet_name` and
        /// `function_name` skipping CDD checks. If `must_be_primary_key` is true, ensures that
        /// the caller is a primary key.
        fn ensure_valid_origin_permissions(
            who: &AccountId,
            must_be_primary_key: bool,
            pallet_name: impl FnOnce() -> PalletName,
            function_name: impl FnOnce() -> ExtrinsicName,
        ) -> Result<AccountCallPermissionsData<AccountId>, DispatchError>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The name of the current pallet (aka module name).
    #[pallet::storage]
    #[pallet::unbounded]
    pub type CurrentPalletName<T> = StorageValue<_, PalletName, ValueQuery>;

    /// The name of the current function (aka extrinsic).
    #[pallet::storage]
    #[pallet::unbounded]
    pub type CurrentDispatchableName<T> = StorageValue<_, ExtrinsicName, ValueQuery>;

    #[pallet::error]
    pub enum Error<T> {
        /// The caller is not authorized to call the current extrinsic.
        UnauthorizedCaller,
    }

    impl<T: Config> Pallet<T> {
        /// Checks if the caller identified with the account `who` is permissioned to call the current
        /// extrinsic. Returns `Ok(data)` if successful. Otherwise returns an `Err`.
        pub fn ensure_call_permissions(
            who: &T::AccountId,
        ) -> Result<AccountCallPermissionsData<T::AccountId>, DispatchError> {
            T::Checker::check_account_call_permissions(
                who,
                || CurrentPalletName::<T>::get(),
                || CurrentDispatchableName::<T>::get(),
            )
        }

        /// Checks if the caller is permissioned to call the current extrinsic skipping CDD checks.
        /// If `must_be_primary_key` ensures that the caller is a primary key.
        pub fn ensure_valid_origin_permissions(
            who: &T::AccountId,
            must_be_primary_key: bool,
        ) -> Result<AccountCallPermissionsData<T::AccountId>, DispatchError> {
            T::Checker::ensure_valid_origin_permissions(
                who,
                must_be_primary_key,
                || CurrentPalletName::<T>::get(),
                || CurrentDispatchableName::<T>::get(),
            )
        }
    }
}

/// A signed extension used in checking call permissions.
#[derive(Decode, DecodeWithMemTracking, Encode)]
#[derive(Clone, Default, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct StoreCallMetadata<T: Config>(PhantomData<T>);

impl<T: Config> fmt::Debug for StoreCallMetadata<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StoreCallMetadata<{:?}>", self.0)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<T: Config> StoreCallMetadata<T> {
    /// Constructs a new store for call metadata.
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Stores call metadata in runtime storage.
    pub fn set_call_metadata(pallet_name: PalletName, extrinsic_name: ExtrinsicName) {
        CurrentPalletName::<T>::put(pallet_name);
        CurrentDispatchableName::<T>::put(extrinsic_name);
    }

    /// Erases call metadata from runtime storage.
    fn clear_call_metadata() {
        CurrentPalletName::<T>::kill();
        CurrentDispatchableName::<T>::kill();
    }
}

impl<T: Config + Send + Sync> TransactionExtension<T::RuntimeCall> for StoreCallMetadata<T>
where
    T::RuntimeCall: GetCallMetadata,
{
    const IDENTIFIER: &'static str = "StoreCallMetadata";
    type Implicit = ();
    type Val = ();
    type Pre = ();

    fn weight(&self, _: &T::RuntimeCall) -> Weight {
        Weight::zero()
    }

    fn validate(
        &self,
        origin: <T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        _call: &T::RuntimeCall,
        _info: &DispatchInfoOf<T::RuntimeCall>,
        _len: usize,
        _: (),
        _implication: &impl Encode,
        _source: TransactionSource,
    ) -> Result<
        (
            ValidTransaction,
            Self::Val,
            <T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        ),
        TransactionValidityError,
    > {
        Ok((ValidTransaction::default(), (), origin))
    }

    fn prepare(
        self,
        _val: Self::Val,
        _origin: &<T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        call: &T::RuntimeCall,
        _info: &DispatchInfoOf<T::RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let metadata = call.get_call_metadata();
        Self::set_call_metadata(metadata.pallet_name.into(), metadata.function_name.into());
        Ok(())
    }

    fn post_dispatch(
        _pre: Self::Pre,
        _info: &DispatchInfoOf<T::RuntimeCall>,
        _post_info: &mut PostDispatchInfoOf<T::RuntimeCall>,
        _len: usize,
        _result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        Self::clear_call_metadata();
        Ok(())
    }
}

/// Transacts `tx` while setting the call metadata to that of `call` until the call is
/// finished. Restores the current call metadata at the end.
///
/// Returns the result of `tx`.
pub fn with_call_metadata<T: Config, R>(metadata: CallMetadata, tx: impl FnOnce() -> R) -> R {
    // Set the extrinsic call metadata and save the current call metadata.
    let (pallet_name, function_name) =
        swap_call_metadata::<T>(metadata.pallet_name.into(), metadata.function_name.into());
    let result = tx();
    // Restore the current call metadata.
    let _ = swap_call_metadata::<T>(pallet_name, function_name);
    result
}

/// Replaces the current call metadata with the given ones and returns the old,
/// replaced call metadata.
pub fn swap_call_metadata<T: Config>(
    pallet_name: PalletName,
    extrinsic_name: ExtrinsicName,
) -> (PalletName, ExtrinsicName) {
    (
        CurrentPalletName::<T>::mutate(|s| mem::replace(s, pallet_name)),
        CurrentDispatchableName::<T>::mutate(|s| mem::replace(s, extrinsic_name)),
    )
}
