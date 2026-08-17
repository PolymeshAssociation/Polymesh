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

//! Polymesh ERC20 Precompile
//!
//! Routes ABI-encoded function calls to domain modules for ERC20 operations.

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::num::NonZero;

use frame_support::traits::Get;
use pallet_revive::precompiles::{AddressMatcher, Error, Ext, Precompile};
use pallet_revive::H160;

use polymesh_precompiles::{IFungibleAssetCalls, FUNGIBLE_ASSET_CODE};
use polymesh_primitives::asset::AssetId;

use crate::common::{revert, revert_err, Common};
use crate::Config;

mod erc20;
mod erc7943;
mod polymesh_specific;

// ==================== Error Messages ====================
pub(crate) const ERR_ASSET_NOT_FOUND: &str = "Asset not found";
pub(crate) const ERR_ASSET_NOT_FUNGIBLE: &str = "Asset is not fungible";
pub(crate) const ERR_INST_NOT_EXECUTED: &str = "Instruction was not executed; Most likely the instruction is missing an affirmation from the receiver/mediator";
// ========================================================

/// All precompile calls exposed by the Polymesh runtime.
pub struct FungibleAssetInterface<T>(PhantomData<T>);

impl<T: Config> Precompile for FungibleAssetInterface<T> {
    type T = T;
    type Interface = IFungibleAssetCalls;

    const MATCHER: AddressMatcher = AddressMatcher::VarPrefix {
        id: NonZero::new(8).unwrap(),
        data_bytes: 16,
    };
    const HAS_CONTRACT_INFO: bool = false;
    const CODE: &[u8] = FUNGIBLE_ASSET_CODE;

    fn call(
        address: &[u8; 20],
        input: &Self::Interface,
        env: &mut impl Ext<T = Self::T>,
    ) -> Result<Vec<u8>, Error> {
        Common::<T>::ensure_direct_call(env)?;

        let asset_id = Self::asset_id_from_address(address, env)?;
        let contract_addr = H160::from(*address);

        match input {
            // State-changing calls - check read-only
            IFungibleAssetCalls::transfer(_)
            | IFungibleAssetCalls::mint(_)
            | IFungibleAssetCalls::approve(_)
            | IFungibleAssetCalls::transferFrom(_)
            | IFungibleAssetCalls::burn(_)
            | IFungibleAssetCalls::permit(_)
            | IFungibleAssetCalls::forcedTransfer(_)
                if env.is_read_only() =>
            {
                Err(Common::<T>::state_change_denied())
            }

            // ERC20 functions
            IFungibleAssetCalls::transfer(call) => Self::transfer(asset_id, call, env),
            IFungibleAssetCalls::totalSupply(_) => Self::total_supply(asset_id, env),
            IFungibleAssetCalls::balanceOf(call) => Self::balance_of(asset_id, call, env),
            IFungibleAssetCalls::allowance(call) => Self::allowance(asset_id, call, env),
            IFungibleAssetCalls::approve(call) => Self::approve(asset_id, call, env),
            IFungibleAssetCalls::transferFrom(call) => Self::transfer_from(asset_id, call, env),

            // ERC20Permit functions (EIP-2612)
            IFungibleAssetCalls::permit(call) => Self::permit(asset_id, contract_addr, call, env),
            IFungibleAssetCalls::nonces(call) => Self::nonces(contract_addr, call, env),
            IFungibleAssetCalls::DOMAIN_SEPARATOR(_) => {
                Self::domain_separator(asset_id, contract_addr, env)
            }

            // ERC20Metadata functions
            IFungibleAssetCalls::name(_) => Self::name(asset_id, env),
            IFungibleAssetCalls::symbol(_) => Self::symbol(asset_id, env),
            IFungibleAssetCalls::decimals(_) => Self::decimals(asset_id, env),

            // Polymesh-specific functions
            IFungibleAssetCalls::mint(call) => Self::issue(asset_id, call, env),
            IFungibleAssetCalls::burn(call) => Self::redeem(asset_id, call, env),

            // ERC7943 functions
            IFungibleAssetCalls::canTransfer(call) => Self::can_transfer(asset_id, call, env),
            IFungibleAssetCalls::forcedTransfer(call) => Self::forced_transfer(asset_id, call, env),
        }
    }
}

impl<T: Config> FungibleAssetInterface<T> {
    /// Returns the [`AssetId`] from the address.
    pub(crate) fn asset_id_from_address(
        address: &[u8; 20],
        env: &mut impl Ext<T = T>,
    ) -> Result<AssetId, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let bytes: [u8; 16] = address[0..16].try_into().expect("slice is 16 bytes; qed");
        let asset_id = AssetId::from_raw(bytes);

        match pallet_asset::Assets::<T>::try_get(asset_id) {
            Ok(asset_details) => {
                if asset_details.asset_type.is_non_fungible() {
                    return Err(revert(ERR_ASSET_NOT_FUNGIBLE));
                }
                Ok(asset_id)
            }
            Err(err) => Err(revert_err(err, ERR_ASSET_NOT_FOUND)),
        }
    }
}
