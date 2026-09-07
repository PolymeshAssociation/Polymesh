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

//! Polymesh Runtime Precompile
//!
//! Routes ABI-encoded function calls to general-purpose runtime extrinsics that, unlike the
//! ones behind [`crate::FungibleAssetInterface`], are not scoped to a single asset.

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::num::NonZero;

use pallet_revive::precompiles::{AddressMatcher, Error, Ext, Precompile};

use polymesh_precompiles::{IPolymeshRuntimeCalls, POLYMESH_RUNTIME_CODE};

use crate::common::Common;
use crate::Config;

mod asset;
mod external_agents;
mod identity;

/// General-purpose Polymesh runtime extrinsics, exposed as a precompile at a single fixed address
pub struct PolymeshRuntimeInterface<T>(PhantomData<T>);

impl<T: Config> Precompile for PolymeshRuntimeInterface<T> {
    type T = T;
    type Interface = IPolymeshRuntimeCalls;

    const MATCHER: AddressMatcher = AddressMatcher::Fixed(NonZero::new(65_535).unwrap());
    const HAS_CONTRACT_INFO: bool = false;
    const CODE: &[u8] = POLYMESH_RUNTIME_CODE;

    fn call(
        _address: &[u8; 20],
        input: &Self::Interface,
        env: &mut impl Ext<T = Self::T>,
    ) -> Result<Vec<u8>, Error> {
        Common::<T>::ensure_direct_call(env)?;

        // Every call in `IPolymeshRuntime` dispatches an extrinsic — none is declared `view` — so
        // the read-only whitelist is empty and the whole interface is rejected in a read-only
        // (`STATICCALL`/`eth_call`) context. Anything added here must stay state-changing, or it
        // needs a whitelist like the ones in `fungible_asset` and `nft`.
        if env.is_read_only() {
            return Err(Common::<T>::state_change_denied());
        }

        match input {
            IPolymeshRuntimeCalls::assetCreateAsset(call) => Self::create_asset(call, env),
            IPolymeshRuntimeCalls::assetRegisterTicker(call) => Self::register_ticker(call, env),
            IPolymeshRuntimeCalls::identityRegisterDid(call) => Self::register_did(call, env),
            IPolymeshRuntimeCalls::identitySelfRegisterDid(call) => {
                Self::self_register_did(call, env)
            }
            IPolymeshRuntimeCalls::externalAgentsAuthorizeBecomeAgent(call) => {
                Self::authorize_become_agent(call, env)
            }
            IPolymeshRuntimeCalls::externalAgentsAcceptBecomeAgent(call) => {
                Self::accept_become_agent(call, env)
            }
        }
    }
}
