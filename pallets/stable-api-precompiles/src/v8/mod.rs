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

//! Polymesh Stable API **v8** precompile.
//!
//! Routes ABI-encoded function calls to domain modules for portfolio, settlement,
//! asset, identity, NFT, and corporate actions operations.

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::num::NonZero;

use pallet_revive::precompiles::{
    alloy::sol, AddressMatcher, Error as PrecompileError, Ext, Precompile,
};

mod asset;
mod corporate_actions;
mod identity;
mod nft;
mod portfolio;
mod settlement;

// Import the Solidity interface. Generates:
//   - `IPolymeshStableApiV8::IPolymeshStableApiV8Calls` enum (23 variants)
//   - Struct types: PolymeshPortfolioId, PolymeshLeg, PolymeshCAId, etc.
sol! {
    #[sol(all_derives)]
    "src/v8/IPolymeshStableApiV8.sol"
}

use IPolymeshStableApiV8::IPolymeshStableApiV8Calls;

/// Polymesh Stable API v8 precompile.
///
/// Routes ABI-encoded function calls to runtime pallets for portfolio, settlement,
/// asset, identity, NFT, and corporate actions operations.
pub struct PolymeshStableApiV8<T>(PhantomData<T>);

impl<T> Precompile for PolymeshStableApiV8<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_portfolio::Config
        + pallet_settlement::Config
        + pallet_identity::Config
        + pallet_nft::Config
        + pallet_corporate_actions::Config,
{
    type T = T;
    type Interface = IPolymeshStableApiV8Calls;

    // Precompile address = version number (v8 = 8, v9 = 9, …).
    // Fixed(8) → 0x0000000000000000000000000000000000080000
    const MATCHER: AddressMatcher = AddressMatcher::Fixed(NonZero::new(8).unwrap());
    const HAS_CONTRACT_INFO: bool = false;

    fn call(
        _address: &[u8; 20],
        input: &Self::Interface,
        env: &mut impl Ext<T = Self::T>,
    ) -> Result<Vec<u8>, PrecompileError> {
        log::trace!(target: "runtime::stable-api-precompile", "v8 call entered");

        match input {
            // ── Read-only guard ──
            // Write operations MUST fail in a read-only context.
            IPolymeshStableApiV8Calls::createPortfolio(_)
            | IPolymeshStableApiV8Calls::acceptPortfolioCustody(_)
            | IPolymeshStableApiV8Calls::quitPortfolioCustody(_)
            | IPolymeshStableApiV8Calls::movePortfolioFunds(_)
            | IPolymeshStableApiV8Calls::createCustodyPortfolio(_)
            | IPolymeshStableApiV8Calls::createVenue(_)
            | IPolymeshStableApiV8Calls::settlementExecute(_)
            | IPolymeshStableApiV8Calls::addAndAffirmInstruction(_)
            | IPolymeshStableApiV8Calls::assetCreateAndIssue(_)
            | IPolymeshStableApiV8Calls::assetIssue(_)
            | IPolymeshStableApiV8Calls::assetRedeem(_)
            | IPolymeshStableApiV8Calls::dividendClaim(_)
            | IPolymeshStableApiV8Calls::createDividend(_)
                if env.is_read_only() =>
            {
                Err(PrecompileError::Error(
                    pallet_revive::Error::<T>::StateChangeDenied.into(),
                ))
            }

            // ── Portfolio (pallet_portfolio) ──
            IPolymeshStableApiV8Calls::createPortfolio(call) => {
                portfolio::create_portfolio::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::acceptPortfolioCustody(call) => {
                portfolio::accept_portfolio_custody::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::quitPortfolioCustody(call) => {
                portfolio::quit_portfolio_custody::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::movePortfolioFunds(call) => {
                portfolio::move_portfolio_funds::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::portfolioAssetBalances(call) => {
                portfolio::portfolio_asset_balances::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::checkPortfoliosInCustody(call) => {
                portfolio::check_portfolios_in_custody::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::createCustodyPortfolio(call) => {
                portfolio::create_custody_portfolio::<T>(call, env)
            }

            // ── Settlement (pallet_settlement) ──
            IPolymeshStableApiV8Calls::createVenue(call) => {
                settlement::create_venue::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::settlementExecute(call) => {
                settlement::settlement_execute::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::addAndAffirmInstruction(call) => {
                settlement::add_and_affirm_instruction::<T>(call, env)
            }

            // ── Asset (pallet_asset) ──
            IPolymeshStableApiV8Calls::assetCreateAndIssue(call) => {
                asset::asset_create_and_issue::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::assetIssue(call) => asset::asset_issue::<T>(call, env),
            IPolymeshStableApiV8Calls::assetRedeem(call) => asset::asset_redeem::<T>(call, env),
            IPolymeshStableApiV8Calls::assetBalanceOf(call) => {
                asset::asset_balance_of::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::assetTotalSupply(call) => {
                asset::asset_total_supply::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::assetMetadataLocalNameToKey(call) => {
                asset::asset_metadata_local_name_to_key::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::assetMetadataValue(call) => {
                asset::asset_metadata_value::<T>(call, env)
            }

            // ── Identity (pallet_identity) ──
            IPolymeshStableApiV8Calls::getKeyDid(call) => identity::get_key_did::<T>(call, env),
            IPolymeshStableApiV8Calls::getNextAssetId(call) => {
                identity::get_next_asset_id::<T>(call, env)
            }

            // ── NFT (pallet_nft) ──
            IPolymeshStableApiV8Calls::nftOwner(call) => nft::nft_owner::<T>(call, env),
            IPolymeshStableApiV8Calls::holdsNfts(call) => nft::holds_nfts::<T>(call, env),

            // ── Corporate Actions (pallet_corporate_actions) ──
            IPolymeshStableApiV8Calls::distributionSummary(call) => {
                corporate_actions::distribution_summary::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::dividendClaim(call) => {
                corporate_actions::dividend_claim::<T>(call, env)
            }
            IPolymeshStableApiV8Calls::createDividend(call) => {
                corporate_actions::create_dividend::<T>(call, env)
            }
        }
    }
}
