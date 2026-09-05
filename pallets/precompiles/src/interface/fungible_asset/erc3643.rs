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

use pallet_revive::precompiles::alloy::sol_types::Revert;
use pallet_revive::precompiles::Error;
use pallet_revive::precompiles::Ext;

use polymesh_precompiles::{IFungibleAsset, IFungibleAssetEvents};
use polymesh_primitives::asset::{AssetId, AssetName};
use polymesh_primitives::ticker::TICKER_LEN;
use polymesh_primitives::Ticker;

use crate::common::Common;
use crate::interface::FungibleAssetInterface;
use crate::interface::ERR_INVALID_SYMBOL;
use crate::Config;

impl<T: Config> FungibleAssetInterface<T> {
    /// Freezes the asset, preventing token transfers. Only an agent of the token can call this function.
    pub(crate) fn pause(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::freeze { asset_id },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Paused(IFungibleAsset::Paused {
                userAddress: caller.address.0.into(),
            }),
        )?;
        Ok(Vec::new())
    }

    /// Unfreezes the token contract, allowing token transfers. Only an agent of the token can call this function.
    pub(crate) fn unpause(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::unfreeze { asset_id },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Unpaused(IFungibleAsset::Unpaused {
                userAddress: caller.address.0.into(),
            }),
        )?;
        Ok(Vec::new())
    }

    /// Sets the token name. Only the owner of the token contract can call this function.
    pub(crate) fn set_name(
        asset_id: AssetId,
        call: &IFungibleAsset::setNameCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let new_asset_name = AssetName::from(&call.name.as_bytes().to_vec());

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::rename_asset {
                asset_id,
                asset_name: new_asset_name.clone(),
            },
        )?;

        Ok(Vec::new())
    }

    /// Sets the token symbol. Only the owner of the token contract can call this function.
    pub(crate) fn set_symbol(
        asset_id: AssetId,
        call: &IFungibleAsset::setSymbolCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let new_symbol = &call.symbol.as_bytes();

        if new_symbol.len() > TICKER_LEN {
            return Err(Error::Revert(Revert {
                reason: ERR_INVALID_SYMBOL.into(),
            }));
        }

        let ticker = Ticker::from_slice_truncated(new_symbol);
        let caller = Common::<T>::caller(env)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::register_unique_ticker { ticker },
        )?;
        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::link_ticker_to_asset_id { ticker, asset_id },
        )?;

        Ok(Vec::new())
    }

    /// Sets the frozen status of a specific address. Only an agent of the token can call this function.
    pub(crate) fn set_address_frozen(
        asset_id: AssetId,
        call: &IFungibleAsset::setAddressFrozenCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        let acc_to_freeze = Common::<T>::asset_holder(env, call.account)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::set_holder_frozen {
                asset_holder: acc_to_freeze,
                asset_id,
                freeze: call.freeze,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::AddressFrozen(IFungibleAsset::AddressFrozen {
                account: call.account,
                freeze: call.freeze,
                owner: caller.address.0.into(),
            }),
        )?;
        Ok(Vec::new())
    }

    /// Freezes an additional amount of tokens for a specific address, on top of any tokens
    /// already frozen. Only an agent of the token can call this function.
    pub(crate) fn freeze_partial_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::freezePartialTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        let acc_to_freeze = Common::<T>::asset_holder(env, call.account)?;
        let amount = Common::<T>::to_balance(call.amount)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::freeze_partial_tokens {
                asset_id,
                asset_holder: acc_to_freeze,
                amount,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::TokensFrozen(IFungibleAsset::TokensFrozen {
                account: call.account,
                amount: call.amount,
            }),
        )?;
        Ok(Vec::new())
    }

    /// Unfreezes an amount of tokens for a specific address, reducing the amount currently
    /// frozen. Only an agent of the token can call this function.
    pub(crate) fn unfreeze_partial_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::unfreezePartialTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        let acc_to_unfreeze = Common::<T>::asset_holder(env, call.account)?;
        let amount = Common::<T>::to_balance(call.amount)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::unfreeze_partial_tokens {
                asset_id,
                asset_holder: acc_to_unfreeze,
                amount,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::TokensUnfrozen(IFungibleAsset::TokensUnfrozen {
                account: call.account,
                amount: call.amount,
            }),
        )?;
        Ok(Vec::new())
    }
}
