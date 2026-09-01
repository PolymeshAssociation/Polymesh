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

use pallet_revive::precompiles::{AddressMatcher, Error, Ext, Precompile};
use pallet_revive::H160;

use polymesh_precompiles::{IFungibleAssetCalls, FUNGIBLE_ASSET_CODE};

use crate::common::{AssetKind, Common};
use crate::Config;

mod erc20;
mod erc3643;
mod erc7943;
mod polymesh_specific;

// ==================== Error Messages ====================
pub(crate) const ERR_INST_NOT_EXECUTED: &str = "Instruction was not executed; Most likely the instruction is missing an affirmation from the receiver/mediator";
pub(crate) const ERR_INVALID_SYMBOL: &str = "Invalid symbol; Ticker is too long";
// ========================================================

pub const DECIMALS: u8 = 6;

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

        let asset_id = Common::<T>::asset_id_from_address(env, address, AssetKind::Fungible)?;
        let contract_addr = H160::from(*address);

        // Calls allowed in a read-only (`STATICCALL`/`eth_call`) context, i.e. exactly those
        // declared `view` in `FungibleAssetStub.sol`. This is a whitelist so that a call added to
        // `IFungibleAsset` and left unclassified is *rejected* here rather than silently allowed
        // to change state; the wildcard arm means the compiler cannot warn about the omission.
        if env.is_read_only() {
            match input {
                IFungibleAssetCalls::totalSupply(_)
                | IFungibleAssetCalls::balanceOf(_)
                | IFungibleAssetCalls::allowance(_)
                | IFungibleAssetCalls::nonces(_)
                | IFungibleAssetCalls::DOMAIN_SEPARATOR(_)
                | IFungibleAssetCalls::name(_)
                | IFungibleAssetCalls::symbol(_)
                | IFungibleAssetCalls::decimals(_)
                | IFungibleAssetCalls::canTransfer(_)
                | IFungibleAssetCalls::canSend(_)
                | IFungibleAssetCalls::canReceive(_)
                | IFungibleAssetCalls::getFrozenTokens(_) => {}
                _ => return Err(Common::<T>::state_change_denied()),
            }
        }

        match input {
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
            IFungibleAssetCalls::canSend(call) => Self::can_send(asset_id, call, env),
            IFungibleAssetCalls::canReceive(call) => Self::can_receive(asset_id, call, env),
            IFungibleAssetCalls::getFrozenTokens(call) => {
                Self::get_frozen_tokens(asset_id, call, env)
            }
            IFungibleAssetCalls::setFrozenTokens(call) => {
                Self::set_frozen_tokens(asset_id, call, env)
            }

            // ERC3643 functions
            IFungibleAssetCalls::pause(_) => Self::pause(asset_id, env),
            IFungibleAssetCalls::unpause(_) => Self::unpause(asset_id, env),
            IFungibleAssetCalls::setName(call) => Self::set_name(asset_id, call, env),
            IFungibleAssetCalls::setSymbol(call) => Self::set_symbol(asset_id, call, env),
            IFungibleAssetCalls::setAddressFrozen(call) => {
                Self::set_address_frozen(asset_id, call, env)
            }
        }
    }
}
