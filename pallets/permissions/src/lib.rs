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

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use codec::{Decode, Encode};
use frame_support::{
    debug, decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    traits::{CallMetadata, GetCallMetadata},
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

#[cfg(feature = "std")]
use std::{cell::RefCell, thread_local};

#[cfg(feature = "std")]
thread_local! {
    static CURRENT_PALLET_NAME: RefCell<PalletName> = RefCell::new(PalletName::default());
    static CURRENT_DISPATCHABLE_NAME: RefCell<DispatchableName> = RefCell::new(DispatchableName::default());
}

/// It sets the TLS value and prints an error in logs if it fails.
/// If the key has been destroyed (which may happen if this is called in a destructor), this function will return an AccessError.
macro_rules! tls_set {
    ($tls:ident, $value:expr) => {
        if let Err(err) = $tls.try_with(|c| *c.borrow_mut() = $value) {
            debug::error!("TLS set of '{}' has failed: {:?}", stringify!($tls), err);
        }
    };
}

/// It gets the TLS value and prints an error in logs if it fails.
/// If the key has been destroyed (which may happen if this is called in a destructor), this function will return an AccessError.
macro_rules! tls_get {
    ($tls:ident) => {{
        match $tls.try_with(|c| c.borrow().clone()) {
            Ok(v) => Some(v),
            Err(err) => {
                debug::error!("TLS get of '{}' has failed: {:?}", stringify!($tls), err);
                None
            }
        }
    }};
}

decl_storage! {
    trait Store for Module<T: Trait> as Permissions {
        /// The name of the current pallet (aka module name).
        pub CurrentPalletName get(fn current_pallet_name): PalletName;
        /// The name of the current function (aka extrinsic).
        pub CurrentDispatchableName get(fn current_dispatchable_name): DispatchableName;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // This definition is needed because the construct_runtime! macro uses it to generate metadata.
        // Without this definition, the metadata won't have details about the errors of this module.
        // That will lead to UIs either throwing fits or showing incorrect error messages.
        type Error = Error<T>;
    }
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
        let pallet = StoreCallMetadata::<T>::current_pallet_name();
        let dispatchable = StoreCallMetadata::<T>::current_dispatchable_name();

        T::Checker::check_account_call_permissions(who, &pallet, &dispatchable)
            .ok_or_else(|| Error::<T>::UnauthorizedCaller.into())
    }
}

/// A signed extension used in checking call permissions.
#[derive(Encode, Decode, Clone, Eq, PartialEq, Default)]
pub struct StoreCallMetadata<T: Trait>(PhantomData<T>);

impl<T: Trait> fmt::Debug for StoreCallMetadata<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StoreCallMetadata<{:?}>", self.0)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<T: Trait> StoreCallMetadata<T> {
    /// Constructs a new store for call metadata.
    pub fn new() -> Self {
        Self(Default::default())
    }
}

#[cfg(feature = "std")]
impl<T: Trait> StoreCallMetadata<T> {
    /// Stores call metadata in runtime storage.
    pub fn set_call_metadata(pallet_name: PalletName, dispatchable_name: DispatchableName) {
        tls_set!(CURRENT_PALLET_NAME, pallet_name);
        tls_set!(CURRENT_DISPATCHABLE_NAME, dispatchable_name);
    }

    /// Erases call metadata from runtime storage.
    fn clear_call_metadata() {
        Self::set_call_metadata(PalletName::default(), DispatchableName::default());
    }

    fn current_pallet_name() -> PalletName {
        tls_get!(CURRENT_PALLET_NAME).unwrap_or_default()
    }

    fn current_dispatchable_name() -> DispatchableName {
        tls_get!(CURRENT_DISPATCHABLE_NAME).unwrap_or_default()
    }
}

#[cfg(not(feature = "std"))]
impl<T: Trait> StoreCallMetadata<T> {
    pub fn set_call_metadata(pallet_name: PalletName, dispatchable_name: DispatchableName) {
        <CurrentPalletName>::put(pallet_name);
        <CurrentDispatchableName>::put(dispatchable_name);
    }

    /// Erases call metadata from runtime storage.
    fn clear_call_metadata() {
        <CurrentPalletName>::kill();
        <CurrentDispatchableName>::kill();
    }

    fn current_pallet_name() -> PalletName {
        <CurrentPalletName>::get()
    }

    fn current_dispatchable_name() -> DispatchableName {
        <CurrentDispatchableName>::get()
    }
}

impl<T: Trait + Send + Sync> SignedExtension for StoreCallMetadata<T> {
    const IDENTIFIER: &'static str = "StoreCallMetadata";
    type AccountId = T::AccountId;
    type Call = <T as Trait>::Call;
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

/// Transacts `tx` while setting the call metadata to that of `call` until the call is
/// finished. Restores the current call metadata at the end.
///
/// Returns the result of `tx`.
pub fn with_call_metadata<Succ, Err>(
    metadata: CallMetadata,
    tx: impl FnOnce() -> Result<Succ, Err>,
) -> Result<Succ, Err> {
    // Set the dispatchable call metadata and save the current call metadata.
    let (pallet_name, function_name) = swap_call_metadata(
        metadata.pallet_name.as_bytes().into(),
        metadata.function_name.as_bytes().into(),
    );
    let result = tx();
    // Restore the current call metadata.
    let _ = swap_call_metadata(pallet_name, function_name);
    result
}

/// Replaces the current call metadata with the given ones and returns the old, replaced call
/// metadata.
#[cfg(not(feature = "std"))]
pub fn swap_call_metadata(
    pallet_name: PalletName,
    dispatchable_name: DispatchableName,
) -> (PalletName, DispatchableName) {
    let old_pallet_name = <CurrentPalletName>::get();
    let old_dispatchable_name = <CurrentDispatchableName>::get();
    <CurrentPalletName>::put(pallet_name);
    <CurrentDispatchableName>::put(dispatchable_name);

    (old_pallet_name, old_dispatchable_name)
}

#[cfg(feature = "std")]
pub fn swap_call_metadata(
    pallet_name: PalletName,
    dispatchable_name: DispatchableName,
) -> (PalletName, DispatchableName) {
    let old_pallet_name = tls_get!(CURRENT_PALLET_NAME).unwrap_or_default();
    let old_dispatchable_name = tls_get!(CURRENT_DISPATCHABLE_NAME).unwrap_or_default();

    tls_set!(CURRENT_PALLET_NAME, pallet_name);
    tls_set!(CURRENT_DISPATCHABLE_NAME, dispatchable_name);

    (old_pallet_name, old_dispatchable_name)
}
