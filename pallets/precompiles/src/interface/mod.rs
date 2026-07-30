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

use polymesh_precompiles::{IFungibleAssetCalls, IFungibleAssetEvents};
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::{with_transaction, Balance};

mod erc20;
mod polymesh_specific;

// ==================== Error Messages ====================
pub(crate) const ERR_INVALID_CALLER: &str = "Invalid caller";
pub(crate) const ERR_BALANCE_CONVERSION_FAILED: &str = "Balance conversion failed";
pub(crate) const ERR_EXTRINSIC_ERROR: &str = "Extrinsic returned an error: ";
pub(crate) const ERR_ASSET_NOT_FOUND: &str = "Asset not found";
pub(crate) const ERR_INVALID_ACCOUNT_ID: &str = "Invalid account id";
pub(crate) const ERR_INVALID_ASSET_NAME: &str = "Asset name is not valid UTF-8";
pub(crate) const ERR_INST_NOT_EXECUTED: &str = "Instruction was not executed; Most likely the instruction is missing an affirmation from the receiver/mediator";
// ========================================================

/// All precompile calls exposed by the Polymesh runtime.
pub struct FungibleAssetInterface<T>(PhantomData<T>);

impl<T> Precompile for FungibleAssetInterface<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    type T = T;
    type Interface = IFungibleAssetCalls;

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

        with_transaction(|| {
            match input {
                // State-changing calls - check read-only
                IFungibleAssetCalls::transfer(_)
                | IFungibleAssetCalls::mint(_)
                | IFungibleAssetCalls::approve(_)
                | IFungibleAssetCalls::transferFrom(_)
                | IFungibleAssetCalls::burn(_)
                | IFungibleAssetCalls::permit(_)
                    if env.is_read_only() =>
                {
                    Err(Error::Error(
                        pallet_revive::Error::<Self::T>::StateChangeDenied.into(),
                    ))
                }

                // ERC20 functions
                IFungibleAssetCalls::transfer(call) => Self::transfer(asset_id, call, env),
                IFungibleAssetCalls::totalSupply(_) => Self::total_supply(asset_id, env),
                IFungibleAssetCalls::balanceOf(call) => Self::balance_of(asset_id, call, env),
                IFungibleAssetCalls::allowance(call) => Self::allowance(asset_id, call, env),
                IFungibleAssetCalls::approve(call) => Self::approve(asset_id, call, env),
                IFungibleAssetCalls::transferFrom(call) => Self::transfer_from(asset_id, call, env),

                // ERC20Permit functions (EIP-2612)
                IFungibleAssetCalls::permit(call) => {
                    Self::permit(asset_id, contract_addr, call, env)
                }
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
            }
        })
    }
}

impl<T> FungibleAssetInterface<T>
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

        pallet_asset::IndexToAssetId::<T>::get(asset_index).ok_or_else(|| {
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

    /// Convert a dispatch error into a revert error that includes the actual error details.
    pub(crate) fn extrinsic_error(err: impl Into<sp_runtime::DispatchError>) -> Error {
        let err: sp_runtime::DispatchError = err.into();
        log::debug!(target: "runtime::precompiles", "Extrinsic call failed: {:?}", err);
        let reason = match err {
            sp_runtime::DispatchError::Module(module_err) => match module_err.message {
                Some(msg) => alloc::format!("{}{}", ERR_EXTRINSIC_ERROR, msg),
                None => alloc::format!("{}{:?}", ERR_EXTRINSIC_ERROR, module_err),
            },
            err => alloc::format!("{}{:?}", ERR_EXTRINSIC_ERROR, err),
        };
        Error::Revert(Revert {
            reason: reason.into(),
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
        event: IFungibleAssetEvents,
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
