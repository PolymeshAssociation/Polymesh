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
use codec::Encode;

use frame_support::dispatch::RawOrigin;
use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::sol_types::{Revert, SolCall};
use pallet_revive::precompiles::Ext;
use pallet_revive::precompiles::{AddressMapper, Error};
use pallet_revive::H160;

use pallet_asset::AssetIdTicker;
use pallet_asset::{Allowances, AssetBalance, AssetNames, WeightInfo as AssetWeightInfo};
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::portfolio::{Fund, FundDescription};
use polymesh_primitives::traits::SettlementFnTrait;
use polymesh_primitives::{AccountId as AccountId32, AssetHolder};

use crate::interface::{IPolymeshInterface, IPolymeshInterfaceEvents, PolymeshInterface};
use crate::interface::{ERR_ASSET_NOT_FOUND, ERR_EXTRINSIC_ERROR};
use crate::interface::{ERR_INVALID_ACCOUNT_ID, ERR_INVALID_ASSET_NAME};

impl<T> PolymeshInterface<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    /// Moves a `value` amount of tokens from the caller’s account to `to`.
    pub(crate) fn transfer(
        asset_id: AssetId,
        call: &IPolymeshInterface::transferCall,
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
            #[cfg(feature = "runtime-benchmarks")]
            false,
        ) {
            // TODO: improve error message
            return Err(Error::Revert(Revert {
                reason: ERR_EXTRINSIC_ERROR.into(),
            }));
        }

        Self::deposit_event(
            env,
            IPolymeshInterfaceEvents::Transfer(IPolymeshInterface::Transfer {
                from: caller.0.into(),
                to: call.to,
                value: call.value,
            }),
        )?;
        Ok(IPolymeshInterface::transferCall::abi_encode_returns(&true))
    }

    /// Returns the value of tokens in existence.
    pub(crate) fn total_supply(
        asset_id: AssetId,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let asset_details =
            pallet_asset::Pallet::<T>::try_get_asset_details(&asset_id).map_err(|_| {
                Error::Revert(Revert {
                    reason: ERR_ASSET_NOT_FOUND.into(),
                })
            })?;

        let value = Self::to_u256(asset_details.total_supply)?;
        Ok(IPolymeshInterface::totalSupplyCall::abi_encode_returns(
            &value,
        ))
    }

    /// Returns the value of tokens owned by account.
    pub(crate) fn balance_of(
        asset_id: AssetId,
        call: &IPolymeshInterface::balanceOfCall,
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
        Ok(IPolymeshInterface::balanceOfCall::abi_encode_returns(
            &value,
        ))
    }

    /// Returns the remaining number of tokens that spender will be allowed to spend on behalf of owner through {transferFrom}.
    pub(crate) fn allowance(
        asset_id: AssetId,
        call: &IPolymeshInterface::allowanceCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let owner = call.owner.into_array().into();
        let owner = <T as pallet_revive::Config>::AddressMapper::to_account_id(&owner);

        let spender = call.spender.into_array().into();
        let spender = <T as pallet_revive::Config>::AddressMapper::to_account_id(&spender);

        let allowance = Allowances::<T>::get((&owner, &spender, &asset_id));
        let value = Self::to_u256(allowance)?;
        Ok(IPolymeshInterface::allowanceCall::abi_encode_returns(
            &value,
        ))
    }

    /// Sets a value amount of tokens as the allowance of spender over the caller’s tokens.
    pub(crate) fn approve(
        asset_id: AssetId,
        call: &IPolymeshInterface::approveCall,
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
            IPolymeshInterfaceEvents::Approval(IPolymeshInterface::Approval {
                owner: owner.0.into(),
                spender: call.spender,
                value: call.value,
            }),
        )?;

        Ok(IPolymeshInterface::approveCall::abi_encode_returns(&true))
    }

    /// Moves a value amount of tokens from from to to using the allowance mechanism.
    pub(crate) fn transfer_from(
        asset_id: AssetId,
        call: &IPolymeshInterface::transferFromCall,
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
            IPolymeshInterfaceEvents::Transfer(IPolymeshInterface::Transfer {
                from: call.from,
                to: call.to,
                value: call.value,
            }),
        )?;

        Ok(IPolymeshInterface::transferFromCall::abi_encode_returns(
            &true,
        ))
    }

    // ==================== ERC20Permit Functions (EIP-2612) ====================

    /// Sets value as the allowance of spender over owner’s tokens, given owner’s signed approval
    pub(crate) fn permit(
        _asset_id: AssetId,
        _verifying_contract: H160,
        _call: &IPolymeshInterface::permitCall,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        log::warn!("ERC20permitPermit is not implemented yet");
        Err(Error::Revert(Revert {
            reason: "permit is not implemented yet".into(),
        }))
    }

    /// Get the current nonce for an owner address.
    pub(crate) fn nonces(
        _verifying_contract: H160,
        _call: &IPolymeshInterface::noncesCall,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        log::warn!("nonces is not implemented yet");
        Err(Error::Revert(Revert {
            reason: "nonces is not implemented yet".into(),
        }))
    }

    /// Get the EIP-712 domain separator for this contract.
    pub(crate) fn domain_separator(
        _asset_id: AssetId,
        _verifying_contract: H160,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        log::warn!("domain_separator is not implemented yet");
        Err(Error::Revert(Revert {
            reason: "domain_separator is not implemented yet".into(),
        }))
    }

    /// Returns the name of the token.
    pub(crate) fn name(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let name = AssetNames::<T>::get(asset_id).unwrap_or_default();
        let name = alloc::string::String::from_utf8(name.0).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ASSET_NAME.into(),
            })
        })?;

        Ok(IPolymeshInterface::nameCall::abi_encode_returns(&name))
    }

    /// Returns the symbol of the token.
    pub(crate) fn symbol(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let ticker = AssetIdTicker::<T>::get(asset_id).unwrap_or_default();
        let ticker = alloc::string::String::from_utf8(ticker.as_ref().to_vec()).map_err(|_| {
            Error::Revert(Revert {
                reason: "Invalid asset ticker".into(),
            })
        })?;

        Ok(IPolymeshInterface::symbolCall::abi_encode_returns(&ticker))
    }

    /// Returns the decimals places of the token
    pub(crate) fn decimals(
        _asset_id: AssetId,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        Ok(IPolymeshInterface::decimalsCall::abi_encode_returns(&6))
    }
}
