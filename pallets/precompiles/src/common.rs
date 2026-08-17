// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! Helpers shared by all Polymesh precompiles.

use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use codec::Encode;
use frame_support::dispatch::{
    DispatchResultWithPostInfo, GetDispatchInfo, PostDispatchInfo, RawOrigin,
};
use frame_support::traits::{Contains, Get, GetCallMetadata, IsType};
use frame_support::weights::Weight;
use frame_system::pallet_prelude::OriginFor;
use sp_runtime::traits::Dispatchable;
use sp_runtime::DispatchError;

use pallet_permissions::with_call_metadata;
use pallet_revive::precompiles::alloy::primitives::{Address, IntoLogData, U256};
use pallet_revive::precompiles::alloy::sol_types::Revert;
use pallet_revive::precompiles::{AddressMapper, Error, Ext, RuntimeCosts, H256};
use pallet_revive::{DispatchRuntimeCall, ExecOrigin, H160};

use polymesh_primitives::{AccountId, AssetHolder, Balance};

use crate::{CallOf, Config};

// ==================== Error Messages ====================
pub const ERR_INVALID_CALLER: &str = "Invalid caller";
pub const ERR_BALANCE_CONVERSION_FAILED: &str = "Balance conversion failed";
pub const ERR_EXTRINSIC_ERROR: &str = "Extrinsic returned an error: ";
pub const ERR_INVALID_ACCOUNT_ID: &str = "Invalid account id";
// ========================================================

/// Build a revert error with the given `reason`.
pub fn revert(reason: impl Into<String>) -> Error {
    Error::Revert(Revert {
        reason: reason.into(),
    })
}

/// Build a revert error with the given `reason`, logging the `err` that caused it.
pub fn revert_err<E: core::fmt::Debug>(err: E, reason: impl Into<String>) -> Error {
    let reason = reason.into();
    log::debug!(target: "runtime::precompiles", "{}: {:?}", reason, err);
    Error::Revert(Revert { reason })
}

/// Convert a dispatch error into a revert error that includes the actual error details.
pub fn extrinsic_error(err: impl Into<DispatchError>) -> Error {
    let err: DispatchError = err.into();
    log::debug!(target: "runtime::precompiles", "Extrinsic call failed: {:?}", err);
    match err {
        DispatchError::Module(module_err) => match module_err.message {
            Some(msg) => revert(alloc::format!("{}{}", ERR_EXTRINSIC_ERROR, msg)),
            None => revert(alloc::format!("{}{:?}", ERR_EXTRINSIC_ERROR, module_err)),
        },
        err => revert(alloc::format!("{}{:?}", ERR_EXTRINSIC_ERROR, err)),
    }
}

/// Weight of swapping the current call metadata in and back out again.
pub fn call_metadata_weight<T: frame_system::Config>() -> Weight {
    <T as frame_system::Config>::DbWeight::get().reads_writes(2, 4)
}

/// Dispatches runtime calls with the call metadata of the call being dispatched.
///
/// Wired into `pallet_revive::Config::DispatchHook` so that runtime calls entering the runtime
/// through the EVM layer are checked against the extrinsic being called and not against the
/// `pallet_revive` extrinsic that carried them.
pub struct DispatchWithCallMetadata<T>(PhantomData<T>);

impl<T> DispatchRuntimeCall<<T as pallet_revive::Config>::RuntimeCall>
    for DispatchWithCallMetadata<T>
where
    T: pallet_revive::Config + pallet_permissions::Config,
    <T as pallet_revive::Config>::RuntimeCall: GetCallMetadata,
{
    fn weight() -> Weight {
        call_metadata_weight::<T>()
    }

    fn dispatch(
        call: <T as pallet_revive::Config>::RuntimeCall,
        origin: OriginFor<T>,
    ) -> DispatchResultWithPostInfo {
        with_call_metadata::<T, _>(call.get_call_metadata(), || call.dispatch(origin))
    }
}

/// The caller of a precompile, in all the representations the precompiles need.
pub struct Caller<T: Config> {
    /// The revive origin of the caller.
    pub origin: ExecOrigin<T>,
    /// The substrate account of the caller.
    pub account_id: T::AccountId,
    /// The ethereum address of the caller.
    pub address: H160,
}

impl<T: Config> Caller<T> {
    /// The origin to use when dispatching runtime calls on behalf of the caller.
    pub fn runtime_origin(&self) -> OriginFor<T> {
        RawOrigin::Signed(self.account_id.clone()).into()
    }
}

/// Helpers shared by all Polymesh precompiles.
pub struct Common<T>(PhantomData<T>);

impl<T: Config> Common<T> {
    /// Weight of swapping the current call metadata in and back out again.
    fn call_metadata_weight() -> Weight {
        call_metadata_weight::<T>()
    }

    /// Get the caller of the precompile.
    pub fn caller(env: &impl Ext<T = T>) -> Result<Caller<T>, Error> {
        let origin = env.caller();
        let account_id = origin
            .account_id()
            .map_err(|err| revert_err(err, ERR_INVALID_CALLER))?
            .clone();
        let address = <T as pallet_revive::Config>::AddressMapper::to_address(&account_id);
        Ok(Caller {
            origin,
            account_id,
            address,
        })
    }

    /// Ensure the precompile isn't being called through a delegate call.
    pub fn ensure_direct_call(env: &impl Ext<T = T>) -> Result<(), Error> {
        frame_support::ensure!(
            !env.is_delegate_call(),
            pallet_revive::Error::<T>::PrecompileDelegateDenied,
        );
        Ok(())
    }

    /// The error returned for state changing calls made in a read-only context.
    pub fn state_change_denied() -> Error {
        Error::Error(pallet_revive::Error::<T>::StateChangeDenied.into())
    }

    /// Convert an ethereum address into a substrate account.
    pub fn account_id(address: Address) -> T::AccountId {
        let address = H160::from(address.into_array());
        <T as pallet_revive::Config>::AddressMapper::to_account_id(&address)
    }

    /// Convert an ethereum address into an [`AssetHolder`].
    pub fn asset_holder(address: Address) -> Result<AssetHolder, Error> {
        Self::account_holder(&Self::account_id(address))
    }

    /// Convert an ethereum address into a [`AccountId`], for storage keyed by the primitive type.
    pub fn account_id32(address: Address) -> Result<AccountId, Error> {
        let account_id: [u8; 32] = Self::account_id(address)
            .encode()
            .try_into()
            .map_err(|err| revert_err(err, ERR_INVALID_ACCOUNT_ID))?;
        Ok(AccountId::from(account_id))
    }

    /// Convert a substrate account into an [`AssetHolder`].
    pub fn account_holder(account_id: &T::AccountId) -> Result<AssetHolder, Error> {
        AssetHolder::try_from(account_id.encode())
            .map_err(|err| revert_err(err, ERR_INVALID_ACCOUNT_ID))
    }

    /// Convert a `U256` value to the balance type [`Balance`].
    pub fn to_balance(value: U256) -> Result<Balance, Error> {
        value
            .try_into()
            .map_err(|err| revert_err(err, ERR_BALANCE_CONVERSION_FAILED))
    }

    /// Convert a [`Balance`] to a `U256` value.
    pub fn to_u256(value: Balance) -> Result<U256, Error> {
        U256::try_from(value).map_err(|err| revert_err(err, ERR_BALANCE_CONVERSION_FAILED))
    }

    /// Deposit an event to the runtime.
    pub fn deposit_event(env: &mut impl Ext<T = T>, event: impl IntoLogData) -> Result<(), Error> {
        let (topics, data) = event.into_log_data().split();
        let topics = topics.into_iter().map(|v| H256(v.0)).collect::<Vec<_>>();
        env.frame_meter_mut()
            .charge_weight_token(RuntimeCosts::DepositEvent {
                num_topic: topics.len() as u32,
                len: data.len() as u32,
            })?;
        env.deposit_event(topics, data.to_vec());
        Ok(())
    }

    /// Dispatch a runtime `call` on behalf of `origin`.
    ///
    /// The call is dispatched with its own call metadata, so that the secondary key permissions
    /// of the caller are checked against the extrinsic being called and not against the
    /// `pallet_revive` extrinsic that entered the precompile.
    pub fn call_runtime(
        env: &mut impl Ext<T = T>,
        origin: OriginFor<T>,
        call: impl Into<CallOf<T>>,
    ) -> Result<PostDispatchInfo, Error> {
        let call: CallOf<T> = call.into();
        let metadata_weight = Self::call_metadata_weight();
        let dispatch_info = call.get_dispatch_info();
        let charged = env.charge(dispatch_info.call_weight.saturating_add(metadata_weight))?;

        let result = with_call_metadata::<T, _>(call.get_call_metadata(), || call.dispatch(origin));

        let (post_info, error) = match result {
            Ok(post_info) => (post_info, None),
            Err(err) => (err.post_info, Some(err.error)),
        };
        env.adjust_gas(
            charged,
            post_info
                .calc_actual_weight(&dispatch_info)
                .saturating_add(metadata_weight),
        );

        match error {
            Some(err) => Err(extrinsic_error(err)),
            None => Ok(post_info),
        }
    }

    /// Run `f` as if the runtime `call` was dispatched.
    ///
    /// Used where the precompile needs the return value of an internal function instead of the
    /// extrinsic's `PostDispatchInfo`; `f` is responsible for charging the weight it uses.
    pub fn with_runtime_call<R>(
        env: &mut impl Ext<T = T>,
        call: impl Into<CallOf<T>>,
        f: impl FnOnce() -> R,
    ) -> Result<R, Error> {
        let call: CallOf<T> = call.into();
        env.charge(Self::call_metadata_weight())?;

        if !<T as frame_system::Config>::BaseCallFilter::contains(call.into_ref()) {
            return Err(extrinsic_error(frame_system::Error::<T>::CallFiltered));
        }

        Ok(with_call_metadata::<T, _>(call.get_call_metadata(), f))
    }
}
