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

//! # Permissions module
//!
//! This module implements the functionality allowing to check permissions for an account to call
//! the current extrinsic.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    storage::StorageValue,
    traits::GetCallMetadata,
};
use polymesh_common_utilities::traits::{
    AccountCallPermissionsData, CheckAccountCallPermissions, PermissionChecker as Trait,
};
use polymesh_primitives::{DispatchableName, PalletName};
use sp_runtime::{
    traits::{DispatchInfoOf, PostDispatchInfoOf, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
};
use sp_std::{fmt, marker::PhantomData, result::Result};

decl_storage! {
    trait Store for Module<T: Trait> as Permissions {
        /// The name of the current pallet (aka module name).
        pub CurrentPalletName get(fn current_pallet_name): PalletName;
        /// The name of the current function (aka extrinsic).
        pub CurrentDispatchableName get(fn current_dispatchable_name): DispatchableName;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The caller is not authorized to call the current extrinsic.
        UnauthorizedCaller,
    }
}

impl<T: Trait> Module<T> {
    /// Checks if the caller identified with the account `who` is permissioned to call the current
    /// extrinsic. Returns `Ok(data)` if successful. Otherwise returns an `Err`.
    pub fn ensure_call_permissions(
        who: &T::AccountId,
    ) -> Result<AccountCallPermissionsData<T::AccountId>, DispatchError> {
        if let Some(data) = T::Checker::check_account_call_permissions(
            who,
            &Self::current_pallet_name(),
            &Self::current_dispatchable_name(),
        ) {
            return Ok(data);
        }
        Err(Error::<T>::UnauthorizedCaller.into())
    }
}

/// A signed extension used in checking call permissions.
#[derive(Encode, Decode, Clone, Eq, PartialEq, Default)]
pub struct StoreCallMetadata<T: Trait + Send + Sync>(PhantomData<T>);

impl<T: Trait + Send + Sync> fmt::Debug for StoreCallMetadata<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StoreCallMetadata<{:?}>", self.0)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<T: Trait + Send + Sync> StoreCallMetadata<T>
where
    T::Call: GetCallMetadata,
{
    /// Constructs a new store for call metadata.
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Replaces the current call metadata with the given ones and returns the old, replaced call
    /// metadata.
    pub fn swap_call_metadata(
        pallet_name: PalletName,
        dispatchable_name: DispatchableName,
    ) -> (PalletName, DispatchableName) {
        let old_pallet_name = <CurrentPalletName>::get();
        let old_dispatchable_name = <CurrentDispatchableName>::get();
        Self::set_call_metadata(pallet_name, dispatchable_name);
        (old_pallet_name, old_dispatchable_name)
    }

    /// Stores call metadata in runtime storage.
    pub fn set_call_metadata(pallet_name: PalletName, dispatchable_name: DispatchableName) {
        <CurrentPalletName>::put(pallet_name);
        <CurrentDispatchableName>::put(dispatchable_name);
    }

    /// Erases call metadata from runtime storage.
    fn clear_call_metadata() {
        <CurrentPalletName>::kill();
        <CurrentDispatchableName>::kill();
    }
}

impl<T: Trait + Send + Sync> SignedExtension for StoreCallMetadata<T>
where
    T::Call: GetCallMetadata,
{
    const IDENTIFIER: &'static str = "StoreCallMetadata";
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        _who: &Self::AccountId,
        _call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        Ok(ValidTransaction::default())
    }

    fn pre_dispatch(
        self,
        _who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let metadata = call.get_call_metadata();
        Self::set_call_metadata(
            metadata.pallet_name.as_bytes().into(),
            metadata.function_name.as_bytes().into(),
        );
        Ok(())
    }

    fn post_dispatch(
        _pre: Self::Pre,
        _info: &DispatchInfoOf<Self::Call>,
        _post_info: &PostDispatchInfoOf<Self::Call>,
        _len: usize,
        _result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        Self::clear_call_metadata();
        Ok(())
    }
}
