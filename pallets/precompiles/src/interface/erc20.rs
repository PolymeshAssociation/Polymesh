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

use alloc::vec::Vec;

use frame_support::traits::Get;
use frame_support::weights::Weight;
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::Error;
use pallet_revive::precompiles::Ext;
use pallet_revive::H160;

use pallet_asset::AssetIdTicker;
use pallet_asset::{Allowances, AssetBalance, AssetNames};
use polymesh_precompiles::{IFungibleAsset, IFungibleAssetEvents};
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::portfolio::{Fund, FundDescription};
use polymesh_primitives::traits::SettlementFnTrait;
use polymesh_primitives::WeightMeter;

use crate::common::{revert, Common};
use crate::interface::FungibleAssetInterface;
use crate::interface::{ERR_ASSET_NOT_FOUND, ERR_INST_NOT_EXECUTED};
use crate::Config;

impl<T: Config> FungibleAssetInterface<T> {
    /// Moves a `value` amount of tokens from the caller’s account to `to`.
    pub(crate) fn transfer(
        asset_id: AssetId,
        call: &IFungibleAsset::transferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        // Converts `value` and charges the weight for the transfer_asset call
        let amount = Common::<T>::to_balance(call.value)?;
        let fund = Fund::new(
            FundDescription::Fungible {
                asset_id: asset_id,
                amount: amount,
            },
            None,
        );

        let worst_case_weight =
            <T as pallet_asset::Config>::SettlementFn::transfer_funds_weight_limit(None, &fund);
        let charged_amount = env.charge(worst_case_weight)?;

        let caller = Common::<T>::caller(env)?;
        let to = Common::<T>::asset_holder(call.to)?;

        let mut weight_meter = WeightMeter::from_limit_unchecked(Weight::zero(), worst_case_weight);

        let result = Common::<T>::with_runtime_call(
            env,
            pallet_settlement::Call::<T>::transfer_funds {
                from: None,
                to: to.clone(),
                fund: fund.clone(),
            },
            || {
                <T as pallet_asset::Config>::SettlementFn::transfer_funds(
                    caller.runtime_origin(),
                    None,
                    to,
                    fund,
                    &mut weight_meter,
                    #[cfg(feature = "runtime-benchmarks")]
                    false,
                )
            },
        )?;

        match result {
            Err(e) => return Err(crate::common::extrinsic_error(e)),
            Ok(inst_id) => {
                env.adjust_gas(charged_amount, weight_meter.consumed());

                // Instruction was created but not executed
                if inst_id.is_some() {
                    return Err(revert(ERR_INST_NOT_EXECUTED));
                }

                Common::<T>::deposit_event(
                    env,
                    IFungibleAssetEvents::Transfer(IFungibleAsset::Transfer {
                        from: caller.address.0.into(),
                        to: call.to,
                        value: call.value,
                    }),
                )?;

                Ok(IFungibleAsset::transferCall::abi_encode_returns(&true))
            }
        }
    }

    /// Returns the value of tokens in existence.
    pub(crate) fn total_supply(
        asset_id: AssetId,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let asset_details = pallet_asset::Pallet::<T>::try_get_asset_details(&asset_id)
            .map_err(|_| revert(ERR_ASSET_NOT_FOUND))?;

        let value = Common::<T>::to_u256(asset_details.total_supply)?;
        Ok(IFungibleAsset::totalSupplyCall::abi_encode_returns(&value))
    }

    /// Returns the value of tokens owned by account.
    pub(crate) fn balance_of(
        asset_id: AssetId,
        call: &IFungibleAsset::balanceOfCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let account = Common::<T>::account_id32(call.account)?;

        let acc_balance = AssetBalance::<T>::get(&account, &asset_id);
        let value = Common::<T>::to_u256(acc_balance)?;
        Ok(IFungibleAsset::balanceOfCall::abi_encode_returns(&value))
    }

    /// Returns the remaining number of tokens that spender will be allowed to spend on behalf of owner through {transferFrom}.
    pub(crate) fn allowance(
        asset_id: AssetId,
        call: &IFungibleAsset::allowanceCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let owner = Common::<T>::account_id(call.owner);
        let spender = Common::<T>::account_id(call.spender);

        let allowance = Allowances::<T>::get((&owner, &spender, &asset_id));
        let value = Common::<T>::to_u256(allowance)?;
        Ok(IFungibleAsset::allowanceCall::abi_encode_returns(&value))
    }

    /// Sets a value amount of tokens as the allowance of spender over the caller’s tokens.
    pub(crate) fn approve(
        asset_id: AssetId,
        call: &IFungibleAsset::approveCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let spender = Common::<T>::account_id(call.spender);
        let amount = Common::<T>::to_balance(call.value)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::approve {
                asset_id,
                spender,
                amount,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Approval(IFungibleAsset::Approval {
                owner: caller.address.0.into(),
                spender: call.spender,
                value: call.value,
            }),
        )?;

        Ok(IFungibleAsset::approveCall::abi_encode_returns(&true))
    }

    /// Moves a value amount of tokens from from to to using the allowance mechanism.
    pub(crate) fn transfer_from(
        asset_id: AssetId,
        call: &IFungibleAsset::transferFromCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let from = Common::<T>::asset_holder(call.from)?;

        let fund = Fund::new(
            FundDescription::Fungible {
                asset_id: asset_id,
                amount: Common::<T>::to_balance(call.value)?,
            },
            None,
        );

        let worst_case_weight =
            <T as pallet_asset::Config>::SettlementFn::transfer_funds_weight_limit(
                Some(&from),
                &fund,
            );
        let charged_amount = env.charge(worst_case_weight)?;

        let spender = Common::<T>::caller(env)?;
        let to = Common::<T>::asset_holder(call.to)?;

        let mut weight_meter = WeightMeter::from_limit_unchecked(Weight::zero(), worst_case_weight);

        let result = Common::<T>::with_runtime_call(
            env,
            pallet_settlement::Call::<T>::transfer_funds {
                from: Some(from.clone()),
                to: to.clone(),
                fund: fund.clone(),
            },
            || {
                <T as pallet_asset::Config>::SettlementFn::transfer_funds(
                    spender.runtime_origin(),
                    Some(from),
                    to,
                    fund,
                    &mut weight_meter,
                    #[cfg(feature = "runtime-benchmarks")]
                    false,
                )
            },
        )?;

        match result {
            Err(e) => return Err(crate::common::extrinsic_error(e)),
            Ok(inst_id) => {
                env.adjust_gas(charged_amount, weight_meter.consumed());

                // Instruction was created but not executed
                if inst_id.is_some() {
                    return Err(revert(ERR_INST_NOT_EXECUTED));
                }

                Common::<T>::deposit_event(
                    env,
                    IFungibleAssetEvents::Transfer(IFungibleAsset::Transfer {
                        from: call.from,
                        to: call.to,
                        value: call.value,
                    }),
                )?;

                Ok(IFungibleAsset::transferFromCall::abi_encode_returns(&true))
            }
        }
    }

    // ==================== ERC20Permit Functions (EIP-2612) ====================

    /// Sets value as the allowance of spender over owner’s tokens, given owner’s signed approval
    pub(crate) fn permit(
        _asset_id: AssetId,
        _verifying_contract: H160,
        _call: &IFungibleAsset::permitCall,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        log::warn!("ERC20permitPermit is not implemented yet");
        Err(revert("permit is not implemented yet"))
    }

    /// Get the current nonce for an owner address.
    pub(crate) fn nonces(
        _verifying_contract: H160,
        _call: &IFungibleAsset::noncesCall,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        log::warn!("nonces is not implemented yet");
        Err(revert("nonces is not implemented yet"))
    }

    /// Get the EIP-712 domain separator for this contract.
    pub(crate) fn domain_separator(
        _asset_id: AssetId,
        _verifying_contract: H160,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        log::warn!("domain_separator is not implemented yet");
        Err(revert("domain_separator is not implemented yet"))
    }

    /// Returns the name of the token.
    pub(crate) fn name(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let name = AssetNames::<T>::get(asset_id).unwrap_or_default();
        let name = alloc::string::String::from_utf8_lossy(name.0.as_ref()).into_owned();

        Ok(IFungibleAsset::nameCall::abi_encode_returns(&name))
    }

    /// Returns the symbol of the token.
    pub(crate) fn symbol(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let ticker = AssetIdTicker::<T>::get(asset_id).unwrap_or_default();

        // Removes all trailing null bytes
        let trim_ticker = ticker
            .as_ref()
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<_>>();
        let ticker = alloc::string::String::from_utf8_lossy(&trim_ticker).into_owned();

        Ok(IFungibleAsset::symbolCall::abi_encode_returns(&ticker))
    }

    /// Returns the decimals places of the token
    pub(crate) fn decimals(
        _asset_id: AssetId,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        Ok(IFungibleAsset::decimalsCall::abi_encode_returns(&6))
    }
}
