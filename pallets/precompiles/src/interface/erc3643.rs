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
use pallet_revive::precompiles::Ext;
use pallet_revive::precompiles::{AddressMapper, Error};

use polymesh_precompiles::{IFungibleAsset, IFungibleAssetEvents};
use polymesh_primitives::asset::{AssetId, AssetName};
use polymesh_primitives::ticker::TICKER_LEN;
use polymesh_primitives::Ticker;

use crate::interface::FungibleAssetInterface;
use crate::interface::ERR_INVALID_SYMBOL;
use crate::Config;

impl<T> FungibleAssetInterface<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    /// Freezes the asset, preventing token transfers. Only an agent of the token can call this function.
    pub(crate) fn pause(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::freeze())?;

        let caller = Self::caller(env)?;
        let caller_acc = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        match pallet_asset::Pallet::<T>::freeze(RawOrigin::Signed(caller_acc).into(), asset_id) {
            Ok(_) => {
                Self::deposit_event(
                    env,
                    IFungibleAssetEvents::Paused(IFungibleAsset::Paused {
                        userAddress: caller.0.into(),
                    }),
                )?;
                Ok(Vec::new())
            }
            Err(e) => Err(Self::extrinsic_error(e)),
        }
    }

    /// Unfreezes the token contract, allowing token transfers. Only an agent of the token can call this function.
    pub(crate) fn unpause(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::unfreeze())?;

        let caller = Self::caller(env)?;
        let caller_acc = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        match pallet_asset::Pallet::<T>::unfreeze(RawOrigin::Signed(caller_acc).into(), asset_id) {
            Ok(_) => {
                Self::deposit_event(
                    env,
                    IFungibleAssetEvents::Unpaused(IFungibleAsset::Unpaused {
                        userAddress: caller.0.into(),
                    }),
                )?;
                Ok(Vec::new())
            }
            Err(e) => Err(Self::extrinsic_error(e)),
        }
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
        let caller = Self::caller(env)?;
        let caller_acc = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        if let Err(e) = pallet_asset::Pallet::<T>::register_unique_ticker(
            RawOrigin::Signed(caller_acc.clone()).into(),
            ticker,
        ) {
            return Err(Self::extrinsic_error(e));
        }

        Ok(Vec::new())
    }

    /// Sets the frozen status of a specific address. Only an agent of the token can call this function.
    pub(crate) fn set_address_frozen(
        asset_id: AssetId,
        call: &IFungibleAsset::setAddressFrozenCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        let acc_to_freeze = Common::<T>::asset_holder(call.account)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::set_address_frozen {
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
}
