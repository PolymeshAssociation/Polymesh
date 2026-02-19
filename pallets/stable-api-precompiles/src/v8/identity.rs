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

//! Identity pallet functions for the Polymesh Stable API v8 precompile.

use alloc::vec::Vec;

use pallet_revive::precompiles::{
    alloy::{primitives::FixedBytes, sol_types::SolCall},
    Error as PrecompileError, Ext,
};
use pallet_revive::AddressMapper;
use sp_runtime::traits::Get;

use super::IPolymeshStableApiV8;

pub(crate) fn get_key_did<T>(
    call: &IPolymeshStableApiV8::getKeyDidCall,
    env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_identity::Config,
{
    // Charge for 2 storage read (identity lookup and frozen status lookup for secondary keys).
    env.charge(T::DbWeight::get().reads(2))?;

    let account = call.account.into_array().into();
    let account_id = <T as pallet_revive::Config>::AddressMapper::to_account_id(&account);

    let did = pallet_identity::Pallet::<T>::get_identity(&account_id).unwrap_or_default();

    log::trace!(
        target: "runtime::stable-api-precompile",
        "getKeyDid address={:?} did={did:?}", call.account
    );

    let did_bytes: FixedBytes<32> = did.0.into();
    Ok(IPolymeshStableApiV8::getKeyDidCall::abi_encode_returns(
        &did_bytes,
    ))
}

pub(crate) fn get_next_asset_id<T>(
    _call: &IPolymeshStableApiV8::getNextAssetIdCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_identity::Config,
{
    todo!()
}
