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

//! Portfolio pallet functions for the Polymesh Stable API v8 precompile.

use alloc::vec::Vec;

use pallet_revive::precompiles::{Error as PrecompileError, Ext};

use super::IPolymeshStableApiV8;

pub(crate) fn create_portfolio<T>(
    _call: &IPolymeshStableApiV8::createPortfolioCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_portfolio::Config,
{
    todo!()
}

pub(crate) fn accept_portfolio_custody<T>(
    _call: &IPolymeshStableApiV8::acceptPortfolioCustodyCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_portfolio::Config,
{
    todo!()
}

pub(crate) fn quit_portfolio_custody<T>(
    _call: &IPolymeshStableApiV8::quitPortfolioCustodyCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_portfolio::Config,
{
    todo!()
}

pub(crate) fn move_portfolio_funds<T>(
    _call: &IPolymeshStableApiV8::movePortfolioFundsCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_portfolio::Config,
{
    todo!()
}

pub(crate) fn portfolio_asset_balances<T>(
    _call: &IPolymeshStableApiV8::portfolioAssetBalancesCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_portfolio::Config,
{
    todo!()
}

pub(crate) fn check_portfolios_in_custody<T>(
    _call: &IPolymeshStableApiV8::checkPortfoliosInCustodyCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_portfolio::Config,
{
    todo!()
}

pub(crate) fn create_custody_portfolio<T>(
    _call: &IPolymeshStableApiV8::createCustodyPortfolioCall,
    _env: &mut impl Ext<T = T>,
) -> Result<Vec<u8>, PrecompileError>
where
    T: pallet_revive::Config + pallet_portfolio::Config,
{
    todo!()
}
