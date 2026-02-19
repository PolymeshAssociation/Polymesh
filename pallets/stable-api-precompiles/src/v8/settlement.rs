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

//! Settlement pallet functions for the Polymesh Stable API v8 precompile.

use alloc::vec::Vec;

use pallet_revive::precompiles::{Error as PrecompileError, Ext};

use super::IPolymeshStableApiV8;

pub(crate) fn create_venue<T>(
    _call: &IPolymeshStableApiV8::createVenueCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_settlement::Config,
{
    todo!()
}

pub(crate) fn settlement_execute<T>(
    _call: &IPolymeshStableApiV8::settlementExecuteCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_settlement::Config,
{
    todo!()
}

pub(crate) fn add_and_affirm_instruction<T>(
    _call: &IPolymeshStableApiV8::addAndAffirmInstructionCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_settlement::Config,
{
    todo!()
}
