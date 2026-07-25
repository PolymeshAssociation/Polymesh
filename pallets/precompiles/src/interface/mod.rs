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

use pallet_revive::precompiles::alloy::primitives::IntoLogData;
use pallet_revive::precompiles::alloy::sol_types::Revert;
use pallet_revive::precompiles::{alloy, AddressMatcher, Ext, Precompile};
use pallet_revive::precompiles::{AddressMapper, Error, RuntimeCosts, H256};
use pallet_revive::H160;

use polymesh_primitives::asset::AssetId;
use polymesh_primitives::Balance;

mod erc20;
mod polymesh_specific;

// Import the Solidity interface. Generates:
//   - `IPolymeshInterface::IPolymeshInterfaceCalls` enum
alloy::sol! {
    #[sol(all_derives)]
    "src/interface/IPolymeshInterface.sol"
}

use IPolymeshInterface::{IPolymeshInterfaceCalls, IPolymeshInterfaceEvents};

// ==================== Error Messages ====================
pub(crate) const ERR_INVALID_CALLER: &str = "Invalid caller";
pub(crate) const ERR_BALANCE_CONVERSION_FAILED: &str = "Balance conversion failed";
pub(crate) const ERR_EXTRINSIC_ERROR: &str = "Extrinsic returned an error: ";
pub(crate) const ERR_ASSET_NOT_FOUND: &str = "Asset not found";
pub(crate) const ERR_INVALID_ACCOUNT_ID: &str = "Invalid account id";
pub(crate) const ERR_INVALID_ASSET_NAME: &str = "Asset name is not valid UTF-8";
// ========================================================

/// All precompile calls exposed by the Polymesh runtime.
pub struct PolymeshInterface<T>(PhantomData<T>);

impl<T> Precompile for PolymeshInterface<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    type T = T;
    type Interface = IPolymeshInterface::IPolymeshInterfaceCalls;

    const MATCHER: AddressMatcher = AddressMatcher::Prefix(NonZero::new(8).unwrap());
    const HAS_CONTRACT_INFO: bool = false;

    fn call(
        address: &[u8; 20],
        input: &Self::Interface,
        env: &mut impl Ext<T = Self::T>,
    ) -> Result<Vec<u8>, Error> {
        frame_support::ensure!(
            !env.is_delegate_call(),
            pallet_revive::Error::<Self::T>::PrecompileDelegateDenied,
        );

        let asset_id = Self::asset_id_from_address(address)?;
        let contract_addr = H160::from(*address);

        match input {
            // State-changing calls - check read-only
            IPolymeshInterfaceCalls::transfer(_)
            | IPolymeshInterfaceCalls::issue(_)
            | IPolymeshInterfaceCalls::approve(_)
            | IPolymeshInterfaceCalls::transferFrom(_)
            | IPolymeshInterfaceCalls::redeem(_)
            | IPolymeshInterfaceCalls::permit(_)
                if env.is_read_only() =>
            {
                Err(Error::Error(
                    pallet_revive::Error::<Self::T>::StateChangeDenied.into(),
                ))
            }

            // ERC20 functions
            IPolymeshInterfaceCalls::transfer(call) => Self::transfer(asset_id, call, env),
            IPolymeshInterfaceCalls::totalSupply(_) => Self::total_supply(asset_id, env),
            IPolymeshInterfaceCalls::balanceOf(call) => Self::balance_of(asset_id, call, env),
            IPolymeshInterfaceCalls::allowance(call) => Self::allowance(asset_id, call, env),
            IPolymeshInterfaceCalls::approve(call) => Self::approve(asset_id, call, env),
            IPolymeshInterfaceCalls::transferFrom(call) => Self::transfer_from(asset_id, call, env),

            // ERC20Permit functions (EIP-2612)
            IPolymeshInterfaceCalls::permit(call) => {
                Self::permit(asset_id, contract_addr, call, env)
            }
            IPolymeshInterfaceCalls::nonces(call) => Self::nonces(contract_addr, call, env),
            IPolymeshInterfaceCalls::DOMAIN_SEPARATOR(_) => {
                Self::domain_separator(asset_id, contract_addr, env)
            }

            // ERC20Metadata functions
            IPolymeshInterfaceCalls::name(_) => Self::name(asset_id, env),
            IPolymeshInterfaceCalls::symbol(_) => Self::symbol(asset_id, env),
            IPolymeshInterfaceCalls::decimals(_) => Self::decimals(asset_id, env),

            // Polymesh-specific functions
            IPolymeshInterfaceCalls::issue(call) => Self::issue(asset_id, call, env),
            IPolymeshInterfaceCalls::redeem(call) => Self::redeem(asset_id, call, env),
        }
    }
}

impl<T> PolymeshInterface<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    /// Returns the [`AssetId`] from the address.
    pub(crate) fn asset_id_from_address(address: &[u8; 20]) -> Result<AssetId, Error> {
        let bytes: [u8; 4] = address[0..4].try_into().expect("slice is 4 bytes; qed");
        let asset_index = u32::from_be_bytes(bytes);

        pallet_asset::Erc20IndexToAssetId::<T>::get(asset_index).ok_or_else(|| {
            Error::Revert(Revert {
                reason: ERR_ASSET_NOT_FOUND.into(),
            })
        })
    }

    /// Get the caller as an `H160` address.
    pub(crate) fn caller(env: &mut impl Ext<T = T>) -> Result<H160, Error> {
        env.caller()
            .account_id()
            .map(<T as pallet_revive::Config>::AddressMapper::to_address)
            .map_err(|_| {
                Error::Revert(Revert {
                    reason: ERR_INVALID_CALLER.into(),
                })
            })
    }

    /// Convert a `U256` value to the balance type [`Balance`].
    pub(crate) fn to_balance(value: alloy::primitives::U256) -> Result<Balance, Error> {
        value.try_into().map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_BALANCE_CONVERSION_FAILED.into(),
            })
        })
    }

    /// Convert a [`Balance`] to a `U256` value.
    pub(crate) fn to_u256(value: Balance) -> Result<alloy::primitives::U256, Error> {
        alloy::primitives::U256::try_from(value).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_BALANCE_CONVERSION_FAILED.into(),
            })
        })
    }

    /// Deposit an event to the runtime.
    pub(crate) fn deposit_event(
        env: &mut impl Ext<T = T>,
        event: IPolymeshInterfaceEvents,
    ) -> Result<(), Error> {
        let (topics, data) = event.into_log_data().split();
        let topics = topics.into_iter().map(|v| H256(v.0)).collect::<Vec<_>>();
        env.frame_meter_mut()
            .charge_weight_token(RuntimeCosts::DepositEvent {
                num_topic: topics.len() as u32,
                len: topics.len() as u32,
            })?;
        env.deposit_event(topics, data.to_vec());
        Ok(())
    }
}
