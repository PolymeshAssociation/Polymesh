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
use codec::Encode;
use core::marker::PhantomData;
use core::num::NonZero;

use ethereum_standards::IERC20;
use ethereum_standards::IERC20::{IERC20Calls, IERC20Events};
use frame_support::dispatch::RawOrigin;
use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::primitives::IntoLogData;
use pallet_revive::precompiles::alloy::sol_types::{Revert, SolCall};
use pallet_revive::precompiles::{alloy, AddressMatcher, Ext, Precompile};
use pallet_revive::precompiles::{AddressMapper, Error, RuntimeCosts, H256};
use pallet_revive::H160;

use pallet_asset::{Allowances, AssetBalance, AssetNames};
use pallet_asset::{AssetIdTicker, WeightInfo as AssetWeightInfo};
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::portfolio::{Fund, FundDescription};
use polymesh_primitives::traits::SettlementFnTrait;
use polymesh_primitives::{AccountId as AccountId32, AssetHolder, Balance};

/// 4-byte prefix for ERC20 precompile instance.
const POLYMESH_ERC20_PREFIX: &[u8; 4] = b"POLY";

// ==================== Error Messages ====================
const ERR_INVALID_ADDRESS: &str = "Address does not map to a native Polymesh Asset";
const ERR_INVALID_CALLER: &str = "Invalid caller";
const ERR_BALANCE_CONVERSION_FAILED: &str = "Balance conversion failed";
const ERR_EXTRINSIC_ERROR: &str = "Extrinsic returned an error: ";
const ERR_ASSET_NOT_FOUND: &str = "Asset not found";
const ERR_INVALID_ACCOUNT_ID: &str = "Invalid account id";
const ERR_INVALID_ASSET_NAME: &str = "Asset name is not valid UTF-8";
// ========================================================

/// An [`AssetIdConverter`] that stores the asset id directly inside the address.
pub struct AssetIdConverter;

impl AssetIdConverter {
    /// Extracts the asset id from the address.
    fn asset_id_from_address(addr: &[u8; 20]) -> Result<AssetId, Error> {
        // Verify that the address belongs to this precompile's domain space
        if &addr[0..4] != POLYMESH_ERC20_PREFIX {
            return Err(Error::Revert(Revert {
                reason: ERR_INVALID_ADDRESS.into(),
            }));
        }

        // Extract the remaining 16 bytes into the native AssetId type
        let mut asset_id = [0u8; 16];
        asset_id.copy_from_slice(&addr[4..20]);
        Ok(asset_id.into())
    }
}

/// An ERC20 precompile with EIP-2612 permit support.
pub struct ERC20<T>(PhantomData<T>);

impl<T> Precompile for ERC20<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    type T = T;
    type Interface = IERC20::IERC20Calls;

    const MATCHER: AddressMatcher = AddressMatcher::Fixed(NonZero::new(8).unwrap());
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

        let asset_id = AssetIdConverter::asset_id_from_address(address)?.into();
        let contract_addr = H160::from(*address);

        match input {
            // State-changing calls - check read-only
            IERC20Calls::transfer(_)
            | IERC20Calls::approve(_)
            | IERC20Calls::transferFrom(_)
            | IERC20Calls::permit(_)
                if env.is_read_only() =>
            {
                Err(Error::Error(
                    pallet_revive::Error::<Self::T>::StateChangeDenied.into(),
                ))
            }

            // ERC20 functions
            IERC20Calls::transfer(call) => Self::transfer(asset_id, call, env),
            IERC20Calls::totalSupply(_) => Self::total_supply(asset_id, env),
            IERC20Calls::balanceOf(call) => Self::balance_of(asset_id, call, env),
            IERC20Calls::allowance(call) => Self::allowance(asset_id, call, env),
            IERC20Calls::approve(call) => Self::approve(asset_id, call, env),
            IERC20Calls::transferFrom(call) => Self::transfer_from(asset_id, call, env),

            // ERC20Permit functions (EIP-2612)
            IERC20Calls::permit(call) => Self::permit(asset_id, contract_addr, call, env),
            IERC20Calls::nonces(call) => Self::nonces(contract_addr, call, env),
            IERC20Calls::DOMAIN_SEPARATOR(_) => {
                Self::domain_separator(asset_id, contract_addr, env)
            }

            // ERC20Metadata functions
            IERC20Calls::name(_) => Self::name(asset_id, env),
            IERC20Calls::symbol(_) => Self::symbol(asset_id, env),
            IERC20Calls::decimals(_) => Self::decimals(asset_id, env),
        }
    }
}

impl<T> ERC20<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    /// Get the caller as an `H160` address.
    fn caller(env: &mut impl Ext<T = T>) -> Result<H160, Error> {
        env.caller()
            .account_id()
            .map(<T as pallet_revive::Config>::AddressMapper::to_address)
            .map_err(|_| {
                Error::Revert(Revert {
                    reason: ERR_INVALID_CALLER.into(),
                })
            })
    }

    /// Convert a `U256` value to the balance type to [`Balance`].
    fn to_balance(value: alloy::primitives::U256) -> Result<Balance, Error> {
        value.try_into().map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_BALANCE_CONVERSION_FAILED.into(),
            })
        })
    }

    /// Convert a [`Balance`] to a `U256` value.
    fn to_u256(value: Balance) -> Result<alloy::primitives::U256, Error> {
        alloy::primitives::U256::try_from(value).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_BALANCE_CONVERSION_FAILED.into(),
            })
        })
    }

    /// Deposit an event to the runtime.
    fn deposit_event(env: &mut impl Ext<T = T>, event: IERC20Events) -> Result<(), Error> {
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

    /// Moves a `value` amount of tokens from the caller’s account to `to`.
    fn transfer(
        asset_id: AssetId,
        call: &IERC20::transferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        // Converts `value` and charges the weight for the transfer_asset call
        let amount = Self::to_balance(call.value)?;
        let fund = Fund::new(
            FundDescription::Fungible {
                asset_id: asset_id,
                amount: amount,
            },
            None,
        );
        env.charge(
            <T as pallet_asset::Config>::SettlementFn::transfer_funds_weight_limit(None, &fund),
        )?;

        // Calls the `base_transfer_asset` function from the pallet_asset
        let caller = Self::caller(env)?;
        let from = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        if let Err(_e) = pallet_asset::Pallet::<T>::base_transfer_asset(
            RawOrigin::Signed(from).into(),
            asset_id,
            <T as pallet_revive::Config>::AddressMapper::to_account_id(
                &call.to.into_array().into(),
            ),
            amount,
            None,
        ) {
            // TODO: improve error message
            return Err(Error::Revert(Revert {
                reason: ERR_EXTRINSIC_ERROR.into(),
            }));
        }

        Self::deposit_event(
            env,
            IERC20Events::Transfer(IERC20::Transfer {
                from: caller.0.into(),
                to: call.to,
                value: call.value,
            }),
        )?;
        Ok(IERC20::transferCall::abi_encode_returns(&true))
    }

    /// Returns the value of tokens in existence.
    fn total_supply(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let asset_details =
            pallet_asset::Pallet::<T>::try_get_asset_details(&asset_id).map_err(|_| {
                Error::Revert(Revert {
                    reason: ERR_ASSET_NOT_FOUND.into(),
                })
            })?;

        let value = Self::to_u256(asset_details.total_supply)?;
        Ok(IERC20::totalSupplyCall::abi_encode_returns(&value))
    }

    /// Returns the value of tokens owned by account.
    fn balance_of(
        asset_id: AssetId,
        call: &IERC20::balanceOfCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let account = call.account.into_array().into();
        let account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&account);
        let account_id: [u8; 32] = account.encode().try_into().map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        let acc_balance = AssetBalance::<T>::get(&AccountId32::from(account_id), &asset_id);
        let value = Self::to_u256(acc_balance)?;
        Ok(IERC20::balanceOfCall::abi_encode_returns(&value))
    }

    /// Returns the remaining number of tokens that spender will be allowed to spend on behalf of owner through {transferFrom}.
    fn allowance(
        asset_id: AssetId,
        call: &IERC20::allowanceCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let owner = call.owner.into_array().into();
        let owner = <T as pallet_revive::Config>::AddressMapper::to_account_id(&owner);

        let spender = call.spender.into_array().into();
        let spender = <T as pallet_revive::Config>::AddressMapper::to_account_id(&spender);

        let allowance = Allowances::<T>::get((&owner, &spender, &asset_id));
        let value = Self::to_u256(allowance)?;
        Ok(IERC20::allowanceCall::abi_encode_returns(&value))
    }

    /// Sets a value amount of tokens as the allowance of spender over the caller’s tokens.
    fn approve(
        asset_id: AssetId,
        call: &IERC20::approveCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::approve())?;

        let owner = Self::caller(env)?;
        let from = <T as pallet_revive::Config>::AddressMapper::to_account_id(&owner);
        let spender = call.spender.into_array().into();
        let spender = <T as pallet_revive::Config>::AddressMapper::to_account_id(&spender);

        if let Err(_e) = pallet_asset::Pallet::<T>::approve(
            RawOrigin::Signed(from).into(),
            asset_id,
            spender,
            Self::to_balance(call.value)?,
        ) {
            // TODO: improve error message
            return Err(Error::Revert(Revert {
                reason: ERR_EXTRINSIC_ERROR.into(),
            }));
        };

        Self::deposit_event(
            env,
            IERC20Events::Approval(IERC20::Approval {
                owner: owner.0.into(),
                spender: call.spender,
                value: call.value,
            }),
        )?;

        Ok(IERC20::approveCall::abi_encode_returns(&true))
    }

    /// Moves a value amount of tokens from from to to using the allowance mechanism.
    fn transfer_from(
        asset_id: AssetId,
        call: &IERC20::transferFromCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let from = call.from.into_array().into();
        let from = <T as pallet_revive::Config>::AddressMapper::to_account_id(&from);
        let from = AssetHolder::try_from(from.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        let fund = Fund::new(
            FundDescription::Fungible {
                asset_id: asset_id,
                amount: Self::to_balance(call.value)?,
            },
            None,
        );

        env.charge(pallet_settlement::Pallet::<T>::transfer_funds_weight_limit(
            Some(&from),
            &fund,
        ))?;

        let spender = Self::caller(env)?;
        let spender = <T as pallet_revive::Config>::AddressMapper::to_account_id(&spender);

        let to = call.to.into_array().into();
        let to = <T as pallet_revive::Config>::AddressMapper::to_account_id(&to);
        let to = AssetHolder::try_from(to.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        if let Err(_e) = pallet_settlement::Pallet::<T>::transfer_funds(
            RawOrigin::Signed(spender).into(),
            Some(from),
            AssetHolder::from(to.clone()),
            fund,
        ) {
            // TODO: improve error message
            return Err(Error::Revert(Revert {
                reason: ERR_EXTRINSIC_ERROR.into(),
            }));
        }

        Self::deposit_event(
            env,
            IERC20Events::Transfer(IERC20::Transfer {
                from: call.from,
                to: call.to,
                value: call.value,
            }),
        )?;

        Ok(IERC20::transferFromCall::abi_encode_returns(&true))
    }

    // ==================== ERC20Permit Functions (EIP-2612) ====================

    /// Sets value as the allowance of spender over owner’s tokens, given owner’s signed approval
    pub(crate) fn permit(
        _asset_id: AssetId,
        _verifying_contract: H160,
        _call: &IERC20::permitCall,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }

    /// Get the current nonce for an owner address.
    fn nonces(
        _verifying_contract: H160,
        _call: &IERC20::noncesCall,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }

    /// Get the EIP-712 domain separator for this contract.
    fn domain_separator(
        _asset_id: AssetId,
        _verifying_contract: H160,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }

    /// Returns the name of the token.
    fn name(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let name = AssetNames::<T>::get(asset_id).unwrap_or_default();
        let name = alloc::string::String::from_utf8(name.0).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ASSET_NAME.into(),
            })
        })?;

        Ok(IERC20::nameCall::abi_encode_returns(&name))
    }

    /// Returns the symbol of the token.
    fn symbol(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let ticker = AssetIdTicker::<T>::get(asset_id).unwrap_or_default();
        let ticker = alloc::string::String::from_utf8(ticker.as_ref().to_vec()).map_err(|_| {
            Error::Revert(Revert {
                reason: "Invalid asset ticker".into(),
            })
        })?;

        Ok(IERC20::symbolCall::abi_encode_returns(&ticker))
    }

    /// Returns the decimals places of the token
    fn decimals(_asset_id: AssetId, _env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        Ok(IERC20::decimalsCall::abi_encode_returns(&6))
    }
}
