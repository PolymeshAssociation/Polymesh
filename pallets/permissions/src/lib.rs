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
    dispatch::DispatchResult,
    storage::StorageValue,
    traits::{CallMetadata, GetCallMetadata},
};
use polymesh_common_utilities::traits::{CheckAccountCallPermissions, PermissionChecker as Trait};
use sp_runtime::{
    traits::{DispatchInfoOf, PostDispatchInfoOf, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
};
use sp_std::{fmt, marker::PhantomData, prelude::Vec, result::Result};

decl_storage! {
    trait Store for Module<T: Trait> as Permissions {
        /// The name of the pallet (aka module name).
        pub PalletName get(fn pallet_name): Vec<u8>;
        /// The name of the function (aka extrinsic).
        pub FunctionName get(fn function_name): Vec<u8>;
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
    /// Checks if `who` is permissioned to call the current extrinsic. Returns `Ok` if
    /// successful. Otherwise returns an `Err`.
    pub fn ensure_call_permissions(who: &T::AccountId) -> DispatchResult {
        let pallet_name = Self::pallet_name();
        let function_name = Self::function_name();
        if T::Checker::check_account_call_permissions(
            who,
            pallet_name.as_ref(),
            function_name.as_ref(),
        ) {
            return Ok(());
        }
        Err(Error::<T>::UnauthorizedCaller.into())
    }
}

/// A signed extension used in checking call permissions.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct StoreCallMetadata<T: Trait + Send + Sync>(PhantomData<T>);

impl<T: Trait + Send + Sync> Default for StoreCallMetadata<T> {
    fn default() -> Self {
        StoreCallMetadata(PhantomData::<T>::default())
    }
}

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
    fn store_call_metadata(pallet_name: &str, function_name: &str) {
        <PalletName>::put(pallet_name.as_bytes());
        <FunctionName>::put(function_name.as_bytes());
    }

    fn clear_call_metadata() {
        <PalletName>::kill();
        <FunctionName>::kill();
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
        Self::store_call_metadata(metadata.pallet_name, metadata.function_name);
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
