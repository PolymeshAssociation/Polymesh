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

//! Asset pallet functions for the Polymesh Stable API v8 precompile.

use alloc::vec::Vec;

use pallet_revive::precompiles::{Error as PrecompileError, Ext};

use super::IPolymeshStableApiV8;

pub(crate) fn asset_create_and_issue<T>(
    _call: &IPolymeshStableApiV8::assetCreateAndIssueCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_asset::Config,
{
    todo!()
}

pub(crate) fn asset_issue<T>(
    _call: &IPolymeshStableApiV8::assetIssueCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_asset::Config,
{
    todo!()
}

pub(crate) fn asset_redeem<T>(
    _call: &IPolymeshStableApiV8::assetRedeemCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_asset::Config,
{
    todo!()
}

pub(crate) fn asset_balance_of<T>(
    _call: &IPolymeshStableApiV8::assetBalanceOfCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_asset::Config,
{
    todo!()
}

pub(crate) fn asset_total_supply<T>(
    _call: &IPolymeshStableApiV8::assetTotalSupplyCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_asset::Config,
{
    todo!()
}

pub(crate) fn asset_metadata_local_name_to_key<T>(
    _call: &IPolymeshStableApiV8::assetMetadataLocalNameToKeyCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_asset::Config,
{
    todo!()
}

pub(crate) fn asset_metadata_value<T>(
    _call: &IPolymeshStableApiV8::assetMetadataValueCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_asset::Config,
{
    todo!()
}
