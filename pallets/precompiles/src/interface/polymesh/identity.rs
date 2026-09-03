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

use alloc::vec::Vec;

use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::Error;
use pallet_revive::precompiles::Ext;
use sp_runtime::traits::SaturatedConversion;

use polymesh_precompiles::{IPolymeshRuntime, IPolymeshRuntimeEvents};

use crate::common::{revert, Common};
use crate::interface::PolymeshRuntimeInterface;
use crate::Config;

impl<T: Config> PolymeshRuntimeInterface<T> {
    /// Registers a new DID for `targetAccount`. The caller must be an active DID registrar.
    pub(crate) fn register_did(
        call: &IPolymeshRuntime::identityRegisterDidCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let caller = Common::<T>::caller(env)?;
        let target_account = Common::<T>::account_id(env, call.targetAccount)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_identity::Call::<T>::register_did {
                target_account: target_account.clone(),
            },
        )?;

        let did = pallet_identity::Pallet::<T>::get_identity(&target_account)
            .ok_or_else(|| revert("DID lookup failed after registration"))?;

        Common::<T>::deposit_event(
            env,
            IPolymeshRuntimeEvents::DidCreated(IPolymeshRuntime::DidCreated {
                did: did.to_bytes().into(),
                targetAccount: call.targetAccount,
            }),
        )?;

        Ok(IPolymeshRuntime::identityRegisterDidCall::abi_encode_returns(&did.to_bytes().into()))
    }

    /// Registers a new DID for the caller's own account, allowing self onboarding.
    pub(crate) fn self_register_did(
        _call: &IPolymeshRuntime::identitySelfRegisterDidCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let caller = Common::<T>::caller(env)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_identity::Call::<T>::self_register_did {},
        )?;

        let did = pallet_identity::Pallet::<T>::get_identity(&caller.account_id)
            .ok_or_else(|| revert("DID lookup failed after registration"))?;

        Common::<T>::deposit_event(
            env,
            IPolymeshRuntimeEvents::DidCreated(IPolymeshRuntime::DidCreated {
                did: did.to_bytes().into(),
                targetAccount: caller.address.0.into(),
            }),
        )?;

        Ok(
            IPolymeshRuntime::identitySelfRegisterDidCall::abi_encode_returns(
                &did.to_bytes().into(),
            ),
        )
    }

    /// Adds an authorization for `target` to later accept, restricted to the `BecomeAgent` variant of `AuthorizationData`.
    pub(crate) fn add_authorization(
        call: &IPolymeshRuntime::identityAddAuthorizationCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        let target = Common::<T>::to_signatory(env, &call.target)?;
        let data = Common::<T>::to_authorization_data(&call.data)?;
        let expiry = (call.expiry != 0).then(|| call.expiry.saturated_into());

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_identity::Call::<T>::add_authorization {
                target: target.clone(),
                data,
                expiry,
            },
        )?;

        let auth_id = pallet_identity::CurrentAuthId::<T>::get();

        Ok(IPolymeshRuntime::identityAddAuthorizationCall::abi_encode_returns(&auth_id))
    }
}
