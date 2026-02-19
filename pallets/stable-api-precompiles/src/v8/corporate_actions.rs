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

//! Corporate actions pallet functions for the Polymesh Stable API v8 precompile.

use alloc::vec::Vec;

use pallet_revive::precompiles::{Error as PrecompileError, Ext};

use super::IPolymeshStableApiV8;

pub(crate) fn distribution_summary<T>(
    _call: &IPolymeshStableApiV8::distributionSummaryCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_corporate_actions::Config,
{
    todo!()
}

pub(crate) fn dividend_claim<T>(
    _call: &IPolymeshStableApiV8::dividendClaimCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_corporate_actions::Config,
{
    todo!()
}

pub(crate) fn create_dividend<T>(
    _call: &IPolymeshStableApiV8::createDividendCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_corporate_actions::Config,
{
    todo!()
}
